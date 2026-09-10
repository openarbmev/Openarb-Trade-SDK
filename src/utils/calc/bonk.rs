use crate::instruction::utils::bonk::accounts;

use super::common::clamp_slippage_basis_points_u128;

/// Calculates the amount of tokens to receive when buying with SOL
///
/// This function implements the constant product formula (x * y = k) for token swaps,
/// taking into account various fees and slippage protection.
///
/// # Arguments
///
/// * `amount_in` - The amount of SOL to spend (in lamports)
/// * `virtual_base` - Virtual base token reserves
/// * `virtual_quote` - Virtual quote token (SOL) reserves
/// * `real_base` - Real base token reserves
/// * `real_quote` - Real quote token (SOL) reserves
/// * `slippage_basis_points` - Maximum slippage tolerance in basis points (e.g., 100 = 1%).
///   Clamped to [`MAX_SLIPPAGE_BASIS_POINTS`](super::common::MAX_SLIPPAGE_BASIS_POINTS) (9999 = 99.99%).
///
/// # Returns
///
/// The minimum amount of tokens that will be received after fees and slippage
pub fn get_buy_token_amount_from_sol_amount(
    amount_in: u64,
    virtual_base: u128,
    virtual_quote: u128,
    real_base: u128,
    real_quote: u128,
    slippage_basis_points: u128,
) -> u64 {
    let amount_in_u128 = amount_in as u128;
    let bps = clamp_slippage_basis_points_u128(slippage_basis_points);

    // Calculate various fees deducted from input amount
    let protocol_fee = (amount_in_u128 * accounts::PROTOCOL_FEE_RATE / 10000) as u128;
    let platform_fee = (amount_in_u128 * accounts::PLATFORM_FEE_RATE / 10000) as u128;
    let share_fee = (amount_in_u128 * accounts::SHARE_FEE_RATE / 10000) as u128;

    // Calculate net input amount after deducting all fees
    let amount_in_net = amount_in_u128
        .checked_sub(protocol_fee)
        .unwrap()
        .checked_sub(platform_fee)
        .unwrap()
        .checked_sub(share_fee)
        .unwrap();

    // Calculate total reserves (virtual + real)
    let input_reserve = virtual_quote.checked_add(real_quote).unwrap();
    let output_reserve = virtual_base.checked_sub(real_base).unwrap();

    // Apply constant product formula: amount_out = (amount_in * output_reserve) / (input_reserve + amount_in)
    let numerator = amount_in_net.checked_mul(output_reserve).unwrap();
    let denominator = input_reserve.checked_add(amount_in_net).unwrap();
    let mut amount_out = numerator.checked_div(denominator).unwrap();

    // Apply slippage protection (bps already clamped)
    amount_out = amount_out - (amount_out * bps) / 10000;
    amount_out as u64
}

/// Calculates the amount of SOL to receive when selling tokens
///
/// This function implements the constant product formula (x * y = k) for token swaps,
/// calculating the SOL output for a given token input amount, accounting for fees and slippage.
///
/// # Arguments
///
/// * `amount_in` - The amount of tokens to sell
/// * `virtual_base` - Virtual base token reserves
/// * `virtual_quote` - Virtual quote token (SOL) reserves
/// * `real_base` - Real base token reserves
/// * `real_quote` - Real quote token (SOL) reserves
/// * `slippage_basis_points` - Maximum slippage tolerance in basis points (e.g., 100 = 1%).
///   Clamped to [`MAX_SLIPPAGE_BASIS_POINTS`](super::common::MAX_SLIPPAGE_BASIS_POINTS) (9999 = 99.99%).
///
/// # Returns
///
/// The minimum amount of SOL that will be received after fees and slippage
pub fn get_sell_sol_amount_from_token_amount(
    amount_in: u64,
    virtual_base: u128,
    virtual_quote: u128,
    real_base: u128,
    real_quote: u128,
    slippage_basis_points: u128,
) -> u64 {
    let amount_in_u128 = amount_in as u128;
    let bps = clamp_slippage_basis_points_u128(slippage_basis_points);

    // For sell operation, input_reserve is token reserves, output_reserve is SOL reserves
    let input_reserve = virtual_base.checked_sub(real_base).unwrap();
    let output_reserve = virtual_quote.checked_add(real_quote).unwrap();

    // Use constant product formula to calculate SOL amount received from selling tokens
    let numerator = amount_in_u128.checked_mul(output_reserve).unwrap();
    let denominator = input_reserve.checked_add(amount_in_u128).unwrap();
    let sol_amount_out = numerator.checked_div(denominator).unwrap();

    // Calculate various fees
    let protocol_fee = (sol_amount_out * accounts::PROTOCOL_FEE_RATE / 10000) as u128;
    let platform_fee = (sol_amount_out * accounts::PLATFORM_FEE_RATE / 10000) as u128;
    let share_fee = (sol_amount_out * accounts::SHARE_FEE_RATE / 10000) as u128;

    // Net SOL amount after deducting fees
    let sol_amount_net = sol_amount_out
        .checked_sub(protocol_fee)
        .unwrap()
        .checked_sub(platform_fee)
        .unwrap()
        .checked_sub(share_fee)
        .unwrap();

    // Apply slippage protection (bps already clamped)
    let final_amount = sol_amount_net - (sol_amount_net * bps) / 10000;

    final_amount as u64
}

#[cfg(test)]
mod tests {
    use super::super::common::MAX_SLIPPAGE_BASIS_POINTS;
    use super::*;

    // Matches defaults used by BonkParams::from_dev_trade
    const VIRTUAL_BASE: u128 = 1_073_025_605_596_382;
    const VIRTUAL_QUOTE: u128 = 30_000_852_951;

    #[test]
    fn buy_slippage_at_10000_bps_does_not_zero_min_out() {
        let with_max = get_buy_token_amount_from_sol_amount(
            1_000_000,
            VIRTUAL_BASE,
            VIRTUAL_QUOTE,
            0,
            0,
            10_000,
        );
        let with_clamp_cap = get_buy_token_amount_from_sol_amount(
            1_000_000,
            VIRTUAL_BASE,
            VIRTUAL_QUOTE,
            0,
            0,
            MAX_SLIPPAGE_BASIS_POINTS as u128,
        );
        assert_eq!(with_max, with_clamp_cap);
        assert!(with_max > 0);
    }

    #[test]
    fn sell_slippage_above_10000_bps_does_not_underflow() {
        let with_overflow = get_sell_sol_amount_from_token_amount(
            1_000_000_000,
            VIRTUAL_BASE,
            VIRTUAL_QUOTE,
            0,
            0,
            50_000,
        );
        let with_clamp_cap = get_sell_sol_amount_from_token_amount(
            1_000_000_000,
            VIRTUAL_BASE,
            VIRTUAL_QUOTE,
            0,
            0,
            MAX_SLIPPAGE_BASIS_POINTS as u128,
        );
        assert_eq!(with_overflow, with_clamp_cap);
        assert!(with_overflow > 0);
    }

    #[test]
    fn normal_slippage_matches_uncapped_formula() {
        const BPS: u128 = 100;

        let buy_no_slip =
            get_buy_token_amount_from_sol_amount(1_000_000, VIRTUAL_BASE, VIRTUAL_QUOTE, 0, 0, 0);
        let buy_with_slip =
            get_buy_token_amount_from_sol_amount(1_000_000, VIRTUAL_BASE, VIRTUAL_QUOTE, 0, 0, BPS);
        let expected_buy = buy_no_slip - ((buy_no_slip as u128 * BPS) / 10_000) as u64;
        assert_eq!(buy_with_slip, expected_buy);
        assert!(buy_with_slip < buy_no_slip);

        let sell_no_slip = get_sell_sol_amount_from_token_amount(
            1_000_000_000,
            VIRTUAL_BASE,
            VIRTUAL_QUOTE,
            0,
            0,
            0,
        );
        let sell_with_slip = get_sell_sol_amount_from_token_amount(
            1_000_000_000,
            VIRTUAL_BASE,
            VIRTUAL_QUOTE,
            0,
            0,
            BPS,
        );
        let expected_sell = sell_no_slip - ((sell_no_slip as u128 * BPS) / 10_000) as u64;
        assert_eq!(sell_with_slip, expected_sell);
        assert!(sell_with_slip < sell_no_slip);
    }
}
