//! Auditable in-tree adaptation of Temporal's official `nozomi-quic-client`.

use arc_swap::ArcSwapOption;
use bytes::Bytes;
use h3::client::SendRequest;
use h3_quinn::OpenStreams;
use http::HeaderValue;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;

pub const MAX_BATCH_SIZE: usize = 16;
pub const MIN_TX_SIZE: usize = 66;
pub const MAX_TX_SIZE: usize = 1232;
const MAX_BATCH_BODY_SIZE: usize = MAX_BATCH_SIZE * (MAX_TX_SIZE + 2);
const RECONNECT_COOLDOWN: Duration = Duration::from_millis(500);

#[derive(Debug)]
pub enum TemporalQuicError {
    Connection(String),
    Request(String),
    Response { status: u16 },
    NotConnected,
    Batch(String),
}

impl TemporalQuicError {
    pub fn is_transport_failure(&self) -> bool {
        matches!(self, Self::Connection(_) | Self::Request(_) | Self::NotConnected)
            || matches!(self, Self::Response { status } if *status >= 500 || matches!(*status, 408 | 425))
    }
}

impl std::fmt::Display for TemporalQuicError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Connection(message) => write!(formatter, "connection: {message}"),
            Self::Request(message) => write!(formatter, "request: {message}"),
            Self::Response { status } => write!(formatter, "HTTP/3 response status: {status}"),
            Self::NotConnected => formatter.write_str("not connected"),
            Self::Batch(message) => write!(formatter, "batch: {message}"),
        }
    }
}

impl std::error::Error for TemporalQuicError {}

type Result<T> = std::result::Result<T, TemporalQuicError>;

struct CachedConnection {
    send_request: SendRequest<OpenStreams, Bytes>,
    driver: JoinHandle<()>,
}

impl Drop for CachedConnection {
    fn drop(&mut self) {
        self.driver.abort();
    }
}

pub struct TemporalQuicSender {
    endpoint: quinn::Endpoint,
    connection: ArcSwapOption<CachedConnection>,
    next_reconnect_at: tokio::sync::Mutex<Option<tokio::time::Instant>>,
    host: String,
    port: u16,
    batch_uri: http::Uri,
    static_headers: http::HeaderMap,
}

impl TemporalQuicSender {
    pub fn new(endpoint: &str, api_key: &str) -> Result<Self> {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let (host, port) = parse_endpoint(endpoint)?;

        let mut roots = rustls::RootCertStore::empty();
        for certificate in rustls_native_certs::load_native_certs().certs {
            let _ = roots.add(certificate);
        }
        let mut tls_config =
            rustls::ClientConfig::builder().with_root_certificates(roots).with_no_client_auth();
        tls_config.alpn_protocols = vec![b"h3".to_vec()];

        let quic_config = quinn::crypto::rustls::QuicClientConfig::try_from(tls_config)
            .map_err(|error| TemporalQuicError::Connection(format!("QUIC TLS config: {error}")))?;
        let mut transport = quinn::TransportConfig::default();
        transport.max_idle_timeout(Some(Duration::from_secs(120).try_into().unwrap()));
        transport.keep_alive_interval(Some(Duration::from_secs(15)));
        let mut client_config = quinn::ClientConfig::new(Arc::new(quic_config));
        client_config.transport_config(Arc::new(transport));

        let bind_addr = if host.contains(':') { "[::]:0" } else { "0.0.0.0:0" };
        let mut quic_endpoint = quinn::Endpoint::client(bind_addr.parse().unwrap())
            .map_err(|error| TemporalQuicError::Connection(format!("endpoint bind: {error}")))?;
        quic_endpoint.set_default_client_config(client_config);

        let batch_uri = format!("/api/sendBatch?c={api_key}").parse().map_err(
            |error: http::uri::InvalidUri| {
                TemporalQuicError::Connection(format!("invalid Batch URI: {error}"))
            },
        )?;
        let host_header = if port == 443 {
            HeaderValue::from_str(&host)
        } else {
            HeaderValue::from_str(&format!("{host}:{port}"))
        }
        .map_err(|error| TemporalQuicError::Connection(format!("invalid host header: {error}")))?;
        let mut static_headers = http::HeaderMap::with_capacity(3);
        static_headers.insert(http::header::HOST, host_header);
        static_headers.insert(
            http::header::CONTENT_TYPE,
            HeaderValue::from_static("application/octet-stream"),
        );

        Ok(Self {
            endpoint: quic_endpoint,
            connection: ArcSwapOption::empty(),
            next_reconnect_at: tokio::sync::Mutex::new(None),
            host,
            port,
            batch_uri,
            static_headers,
        })
    }

    pub async fn warmup(&self) -> Result<()> {
        self.reconnect_if_stale(None).await
    }

    pub fn encode_batch(transactions: &[&[u8]]) -> Result<Bytes> {
        if transactions.is_empty() {
            return Err(TemporalQuicError::Batch("empty batch".into()));
        }
        if transactions.len() > MAX_BATCH_SIZE {
            return Err(TemporalQuicError::Batch(format!(
                "too many transactions: {} (max {MAX_BATCH_SIZE})",
                transactions.len()
            )));
        }

        let body_len: usize = transactions.iter().map(|transaction| 2 + transaction.len()).sum();
        if body_len > MAX_BATCH_BODY_SIZE {
            return Err(TemporalQuicError::Batch(format!(
                "batch body too large: {body_len} bytes (max {MAX_BATCH_BODY_SIZE})"
            )));
        }
        let mut body = Vec::with_capacity(body_len);
        for transaction in transactions {
            if !(MIN_TX_SIZE..=MAX_TX_SIZE).contains(&transaction.len()) {
                return Err(TemporalQuicError::Batch(format!(
                    "transaction size {} is outside {MIN_TX_SIZE}..={MAX_TX_SIZE} bytes",
                    transaction.len()
                )));
            }
            body.extend_from_slice(&(transaction.len() as u16).to_be_bytes());
            body.extend_from_slice(transaction);
        }
        Ok(Bytes::from(body))
    }

    pub async fn send_raw(&self, body: Bytes) -> Result<()> {
        let connection = match self.connection.load_full() {
            Some(connection) => connection,
            None => {
                self.reconnect_if_stale(None).await?;
                self.connection.load_full().ok_or(TemporalQuicError::NotConnected)?
            }
        };
        match self.try_send_with_timeout(&connection, body.clone()).await {
            Ok(()) => Ok(()),
            Err(error) if error.is_transport_failure() => {
                self.reconnect_if_stale(Some(&connection)).await?;
                let connection =
                    self.connection.load_full().ok_or(TemporalQuicError::NotConnected)?;
                self.try_send_with_timeout(&connection, body).await
            }
            Err(error) => Err(error),
        }
    }

    async fn try_send_with_timeout(
        &self,
        connection: &CachedConnection,
        body: Bytes,
    ) -> Result<()> {
        tokio::time::timeout(Duration::from_secs(3), self.try_send(connection, body))
            .await
            .map_err(|_| TemporalQuicError::Request("request timed out".into()))?
    }

    async fn resolve_dns(&self) -> Result<SocketAddr> {
        let address = format!("{}:{}", self.host, self.port);
        let resolved = tokio::net::lookup_host(&address)
            .await
            .map_err(|error| {
                TemporalQuicError::Connection(format!("DNS lookup {address}: {error}"))
            })?
            .next()
            .ok_or_else(|| TemporalQuicError::Connection(format!("no addresses for {address}")));
        resolved
    }

    async fn connect(&self) -> Result<Arc<CachedConnection>> {
        let address = self.resolve_dns().await?;
        let quic_connection = self
            .endpoint
            .connect(address, &self.host)
            .map_err(|error| TemporalQuicError::Connection(format!("QUIC connect: {error}")))?
            .await
            .map_err(|error| TemporalQuicError::Connection(format!("QUIC handshake: {error}")))?;
        let h3_connection = h3_quinn::Connection::new(quic_connection);
        let (mut driver, send_request) = h3::client::new(h3_connection)
            .await
            .map_err(|error| TemporalQuicError::Connection(format!("H3 handshake: {error}")))?;
        let driver_handle = tokio::spawn(async move {
            let _ = std::future::poll_fn(|context| driver.poll_close(context)).await;
        });
        Ok(Arc::new(CachedConnection { send_request, driver: driver_handle }))
    }

    async fn reconnect_if_stale(&self, stale: Option<&Arc<CachedConnection>>) -> Result<()> {
        let mut next_reconnect_at = self.next_reconnect_at.lock().await;
        if let Some(current) = self.connection.load_full() {
            match stale {
                Some(stale) if Arc::ptr_eq(&current, stale) => {}
                _ => return Ok(()),
            }
        }
        let now = tokio::time::Instant::now();
        if next_reconnect_at.is_some_and(|deadline| now < deadline) {
            return Err(TemporalQuicError::Connection("reconnect cooling down".into()));
        }

        match self.connect().await {
            Ok(connection) => {
                self.connection.store(Some(connection));
                *next_reconnect_at = None;
                Ok(())
            }
            Err(error) => {
                self.connection.store(None);
                *next_reconnect_at = Some(now + RECONNECT_COOLDOWN);
                Err(error)
            }
        }
    }

    async fn try_send(&self, connection: &CachedConnection, body: Bytes) -> Result<()> {
        let mut sender = connection.send_request.clone();
        let mut headers = self.static_headers.clone();
        headers.insert(http::header::CONTENT_LENGTH, HeaderValue::from(body.len() as u64));
        let (mut parts, _) = http::Request::new(()).into_parts();
        parts.method = http::Method::POST;
        parts.uri = self.batch_uri.clone();
        parts.headers = headers;
        let request = http::Request::from_parts(parts, ());
        let mut stream = sender
            .send_request(request)
            .await
            .map_err(|error| TemporalQuicError::Request(format!("send headers: {error}")))?;
        stream
            .send_data(body)
            .await
            .map_err(|error| TemporalQuicError::Request(format!("send body: {error}")))?;
        stream
            .finish()
            .await
            .map_err(|error| TemporalQuicError::Request(format!("finish stream: {error}")))?;
        let response = stream
            .recv_response()
            .await
            .map_err(|error| TemporalQuicError::Request(format!("receive response: {error}")))?;
        let status = response.status();
        while stream
            .recv_data()
            .await
            .map_err(|error| TemporalQuicError::Request(format!("receive response body: {error}")))?
            .is_some()
        {}
        if !status.is_success() {
            return Err(TemporalQuicError::Response { status: status.as_u16() });
        }
        Ok(())
    }
}

fn parse_endpoint(endpoint: &str) -> Result<(String, u16)> {
    let without_scheme = endpoint
        .strip_prefix("http://")
        .or_else(|| endpoint.strip_prefix("https://"))
        .unwrap_or(endpoint);
    let authority = without_scheme.split('/').next().unwrap_or(without_scheme);
    if authority.is_empty() {
        return Err(TemporalQuicError::Connection("Temporal endpoint has no host".into()));
    }
    if let Some((host, port)) = authority.rsplit_once(':') {
        if !host.contains(':') {
            let parsed_port = port.parse::<u16>().map_err(|error| {
                TemporalQuicError::Connection(format!("invalid endpoint port: {error}"))
            })?;
            return Ok((host.to_string(), parsed_port));
        }
    }
    Ok((authority.trim_matches(['[', ']']).to_string(), 443))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_big_endian_batch_lengths() {
        let first = vec![1_u8; MIN_TX_SIZE];
        let second = vec![2_u8; MIN_TX_SIZE + 1];
        let encoded = TemporalQuicSender::encode_batch(&[&first, &second]).unwrap();
        assert_eq!(&encoded[..2], &(MIN_TX_SIZE as u16).to_be_bytes());
        let second_offset = 2 + first.len();
        assert_eq!(
            &encoded[second_offset..second_offset + 2],
            &((MIN_TX_SIZE + 1) as u16).to_be_bytes()
        );
    }

    #[test]
    fn rejects_empty_oversized_and_too_large_batches() {
        assert!(TemporalQuicSender::encode_batch(&[]).is_err());
        let short = vec![0_u8; MIN_TX_SIZE - 1];
        assert!(TemporalQuicSender::encode_batch(&[&short]).is_err());
        let valid = vec![0_u8; MIN_TX_SIZE];
        let too_many: Vec<&[u8]> = (0..=MAX_BATCH_SIZE).map(|_| valid.as_slice()).collect();
        assert!(TemporalQuicSender::encode_batch(&too_many).is_err());
    }

    #[test]
    fn only_transport_and_service_responses_allow_fallback() {
        assert!(TemporalQuicError::Request("timeout".into()).is_transport_failure());
        assert!(TemporalQuicError::Response { status: 503 }.is_transport_failure());
        assert!(!TemporalQuicError::Response { status: 401 }.is_transport_failure());
        assert!(!TemporalQuicError::Response { status: 429 }.is_transport_failure());
        assert!(!TemporalQuicError::Batch("malformed".into()).is_transport_failure());
    }
}
