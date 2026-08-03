use core::cmp::{max, min};
use pinocchio::{AccountView, error::ProgramError};

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


pub struct SimpleRng { state: u32 }
impl SimpleRng {
    /// Initializing the system. Must be non zero value. (0 breaks the system)
    pub fn new(seed: u32) -> Self {
        Self { state: if seed == 0 { 1 } else { seed } }
    }

    /// Generating the next pseudo random number.
    pub fn next_u32(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        x
    }
}

/// Reading the balance of a token.
pub fn get_token_balance(token_account: &AccountView) -> Result<u64, ProgramError> {
    // Safety Check: Verify account contains enough byte for an spl token
    let data = token_account.try_borrow()?;
    if data.len() < 165 {
        return Err(ProgramError::InvalidAccountData);
    }

    // Splice that data: The 'amount' field starts at byte 64 and is 8 bytes long(u64)
    let balance_bytes: &[u8; 8] = data[64..72]
        .try_into()
        .map_err(|_| ProgramError::InvalidAccountData)?;

    // Converting the bytes to integer.
    let balance = u64::from_le_bytes(*balance_bytes);

    Ok(balance)
}
