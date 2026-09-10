## openarb-trade-sdk v4.0.2

This release focuses on QUIC reliability and low-latency submission stability for Astralane.

### Highlights

- Fixed Astralane QUIC address-family mismatch that could produce `invalid remote address` when DNS returned IPv6 first and local endpoint was IPv4-only.
- Added remote-family-aware local QUIC bind selection:
  - IPv4 remote -> bind `0.0.0.0:0`
  - IPv6 remote -> bind `[::]:0`
- Added Astralane direct-IP candidate support (official region IPs), with IPv4-first selection for better QUIC stability.
- Added automatic endpoint failover and reconnect rotation across candidate addresses, reducing single-endpoint/DNS variance impact.
- Kept existing SDK interfaces compatible while improving submit-path resiliency.

### Also included from recent updates

- BlockRazor gRPC endpoint fixes and gRPC default transport behavior improvements (v4.0.1).
- SWQOS transport path hardening and Binary-Tx response handling improvements (v4.0.0).

