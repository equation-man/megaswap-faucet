//! MegaSwap faucet test entrypoint
#![allow(warnings)]
mod protocol_ix;
mod helpers;
use crate::protocol_ix::{
    MegaSwapFaucetCtx, initialize_protocol,
    dispense_tokens,
};
use megaswap_faucet::instructions::{
    dispense_controller::calculate_faucet_payout
}; 
use solana_pubkey::Pubkey;
use core::cmp::{max, min};

const PROGRAM_ID: Pubkey = Pubkey::new_from_array(megaswap_faucet::ID);

#[test]
fn test_init_protocol() {
    let mut ctx_init = initialize_protocol(PROGRAM_ID);
    let _ = dispense_tokens(&mut ctx_init, PROGRAM_ID);
}

// Testing the P-Controller.

// Shared default configurations (using 9 decimals, i.e, 1 Token = 1_000_000_000)
const DECIMALS: u64 = 1;//_000_000_000;
const TARGET_FLOOR: u64 = 500_000 * DECIMALS; // 500k tokens
const BASE_PAYOUT: u64 = 5_000 * DECIMALS;    // 5k tokens
const MIN_PAYOUT: u64 = 1_000 * DECIMALS;     // 1k tokens
const MAX_PAYOUT: u64 = 50_000 * DECIMALS;   // 50k tokens

// Gain setup: 1 surplus token = 0.1 extra tokens payout (k_scaled = 1, precision = 10)
const K_SCALED: u64 = 1;
const PRECISION: u128 = 10;

#[test]
fn test_balanced_at_target_floor() {
    // When balance == target_floor, error is 0. Payout must equal exactly base_payout.
    let current_balance = TARGET_FLOOR;
    let result = calculate_faucet_payout(
        current_balance, TARGET_FLOOR, BASE_PAYOUT, 
        K_SCALED, PRECISION, MIN_PAYOUT, MAX_PAYOUT
    );

    println!("Base payout is {}, result payout is {}, target bal is {}", BASE_PAYOUT, result.unwrap(), current_balance);
    
    assert_eq!(result, Some(BASE_PAYOUT));
}

#[test]
fn test_surplus_mid_range() {
    // Balance is 600k tokens (100k surplus). 
    // Bonus math: (1 * 100,000) / 10 = 10,000 tokens bonus.
    // Total: 5,000 (Base) + 10,000 (Bonus) = 15,000 tokens.
    let current_balance = 600_000 * DECIMALS;
    let expected_payout = 15_000 * DECIMALS;
    
    let result = calculate_faucet_payout(
        current_balance, TARGET_FLOOR, BASE_PAYOUT, 
        K_SCALED, PRECISION, MIN_PAYOUT, MAX_PAYOUT
    );
    println!("Base payout is {}, result payout is {}, target bal is {}", BASE_PAYOUT, result.unwrap(), current_balance);
    
    assert_eq!(result, Some(expected_payout));
}

#[test]
fn test_surplus_max_clamp() {
    // Balance is 1,500k tokens (1,000k surplus).
    // Bonus math: (1 * 1,000,000) / 10 = 100,000 tokens bonus.
    // Raw Total: 5,000 + 100,000 = 105,000 tokens. 
    // Must clamp to Max Payout: 50,000 tokens.
    let current_balance = 1_500_000 * DECIMALS;
    
    let result = calculate_faucet_payout(
        current_balance, TARGET_FLOOR, BASE_PAYOUT, 
        K_SCALED, PRECISION, MIN_PAYOUT, MAX_PAYOUT
    );
    println!("Base payout is {}, result payout is {}, target bal is {}", BASE_PAYOUT, result.unwrap(), current_balance);
    
    assert_eq!(result, Some(MAX_PAYOUT));
}

#[test]
fn test_deficit_mid_range() {
    // Balance is 480k tokens (20k deficit).
    // Reduction math: (1 * 20,000) / 10 = 2,000 tokens reduction.
    // Total: 5,000 (Base) - 2,000 (Reduction) = 3,000 tokens.
    let current_balance = 480_000 * DECIMALS;
    let expected_payout = 3_000 * DECIMALS;
    
    let result = calculate_faucet_payout(
        current_balance, TARGET_FLOOR, BASE_PAYOUT, 
        K_SCALED, PRECISION, MIN_PAYOUT, MAX_PAYOUT
    );
    println!("Base payout is {}, result payout is {}, target bal is {}", BASE_PAYOUT, result.unwrap(), current_balance);
    
    assert_eq!(result, Some(expected_payout));
}

#[test]
fn test_deficit_min_clamp() {
    // Balance is 440k tokens (60k deficit).
    // Reduction math: (1 * 60,000) / 10 = 6,000 tokens reduction.
    // Raw Total: 5,000 (Base) - 6,000 = Negative value handled by unwrap_or(0).
    // Must clamp upwards to Min Payout: 1,000 tokens.
    let current_balance = 440_000 * DECIMALS;
    
    let result = calculate_faucet_payout(
        current_balance, TARGET_FLOOR, BASE_PAYOUT, 
        K_SCALED, PRECISION, MIN_PAYOUT, MAX_PAYOUT
    );
    println!("Base payout is {}, result payout is {}, target bal is {}", BASE_PAYOUT, result.unwrap(), current_balance);
    
    assert_eq!(result, Some(MIN_PAYOUT));
}

#[test]
fn test_absolute_empty_pool() {
    // Even if the faucet has literally 0 balance, it must still return the min_payout guardrail.
    let current_balance = 0;
    
    let result = calculate_faucet_payout(
        current_balance, TARGET_FLOOR, BASE_PAYOUT, 
        K_SCALED, PRECISION, MIN_PAYOUT, MAX_PAYOUT
    );
    println!("Base payout is {}, result payout is {}, target bal is {}", BASE_PAYOUT, result.unwrap(), current_balance);
    
    assert_eq!(result, Some(MIN_PAYOUT));
}

#[test]
fn test_extreme_overflow_safety() {
    // Verifies that max integer bounds (u64::MAX) inside the faucet don't panic the checked math engine.
    let current_balance = u64::MAX;
    
    let result = calculate_faucet_payout(
        current_balance, TARGET_FLOOR, BASE_PAYOUT, 
        K_SCALED, PRECISION, MIN_PAYOUT, MAX_PAYOUT
    );
    println!("Base payout is {}, result payout is {}, target bal is {}", BASE_PAYOUT, result.unwrap(), current_balance);
    
    // Should overflow handling safely catch it or gracefully cap it to max payout
    assert_eq!(result, Some(MAX_PAYOUT));
}

