//! This is the program's entrypoint.
#![allow(warnings)]
#![cfg_attr(not(feature = "std"), no_std)]
use pinocchio::{
    Address, AccountView, ProgramResult, error::ProgramError
};
use pinocchio_pubkey::declare_id;
use pinocchio_log::log;

pub mod config;
pub mod instructions;

use config::*;
use crate::instructions::{
    initialize::*,
    dispense::*,
};

declare_id!("9uwR3ZyHXhnA2QvPDHtjg5ei3AT9VTzst6pbzj6eQjLn");

// We are not using the normal Solana SDK entrypoint
// Disable std and the entrypoint
#[cfg(all(not(feature = "std"), not(feature = "no-entrypoint")))]
mod entrypoint {
    use pinocchio::{default_allocator, nostd_panic_handler, program_entrypoint};
    // Minimum overhead global allocator
    default_allocator!();
    // Zero overhead aborting panic handler for saving CUs
    nostd_panic_handler!();

    // Register the custom raw SVM entrypoint.
    program_entrypoint!(super::process_instructions);
}

fn process_instructions(
    _program_id: &Address,
    accounts: &mut [AccountView],
    instruction_data: &[u8]
) -> ProgramResult {
    match instruction_data.split_first() {
        Some((Initialize::DISCRIMINATOR, data)) => Initialize::try_from((data, accounts))?.process(),
        Some((Dispense::DISCRIMINATOR, data)) => Dispense::try_from((data, accounts))?.process(),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
