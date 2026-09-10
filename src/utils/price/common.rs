/// Calculate the token price in quote based on base and quote reserves
///
/// # Arguments
/// * `base_reserve` - Base reserve in the pool
/// * `quote_reserve` - Quote reserve in the pool
/// * `base_decimals` - Base decimals
/// * `quote_decimals` - Quote decimals
///
/// # Returns
/// Token price in quote as f64
pub fn price_base_in_quote(
    base_reserve: u64,
    quote_reserve: u64,
    base_decimals: u8,
    quote_decimals: u8,
) -> f64 {
    let base = base_reserve as f64 / 10f64.powi(base_decimals as i32);
    let quote = quote_reserve as f64 / 10f64.powi(quote_decimals as i32);
    if base == 0.0 {
        return 0.0;
    }
    quote / base
}

/// Calculate the token price in base based on base and quote reserves
///
/// # Arguments
/// * `base_reserve` - Base reserve in the pool
/// * `quote_reserve` - Quote reserve in the pool
/// * `base_decimals` - Base decimals
/// * `quote_decimals` - Quote decimals
///
/// # Returns
/// Token price in base as f64
pub fn price_quote_in_base(
    base_reserve: u64,
    quote_reserve: u64,
    base_decimals: u8,
    quote_decimals: u8,
) -> f64 {
    let base = base_reserve as f64 / 10f64.powi(base_decimals as i32);
    let quote = quote_reserve as f64 / 10f64.powi(quote_decimals as i32);
    if quote == 0.0 {
        return 0.0;
    }
    base / quote
}
