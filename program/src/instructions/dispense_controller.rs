use core::cmp::{max, min};

/// Proportional Controller(P-Controller) which control
/// how the tokens are dispensed to users.
/// This function calculates dynamically the amount to dispense above or below the
/// floor or base dispense limit.
pub fn calculate_faucet_payout(
    current_faucet_balance: u64, target_floor: u64, base_payout: u64,
    k_scaled: u64, precision: u128, min_payout: u64, max_payout: u64
) -> Option<u64> {
    let mut raw_dispense: u128 = base_payout as u128;
    if current_faucet_balance > target_floor {
        // We have more tokens in the faucet increase the amount to give out
        // Equation: payout = Base + Bonus((k* surplus) / precision)
        let surplus = (current_faucet_balance as u128).checked_sub(target_floor as u128)?;

        let bonus = (k_scaled as u128)
            .checked_mul(surplus)?.checked_div(precision)?;
        raw_dispense = raw_dispense.checked_add(bonus)?;
    } else {
        // We have less balance, below our target floor. Reduce amount to give to preserve the pool
        // Equation: payout = Base - reduction((k*deficit) / precision)
        let deficit = (target_floor as u128).checked_sub(current_faucet_balance as u128)?;
        let reduction = (k_scaled as u128).checked_mul(deficit)?
            .checked_div(precision)?;

        // Drop to 0 if the math reduces payout below absolute 0
        raw_dispense = raw_dispense.checked_sub(reduction).unwrap_or(0);
    }

    // The clamping engine. Enforce min dispense and max dispense.
    let final_payout = max(
        min_payout as u128,
        min(raw_dispense, max_payout as u128)
    ) as u64;

    Some(final_payout)
}
