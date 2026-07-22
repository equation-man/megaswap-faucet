//! Initializing the protocol
use solana_address;
use pinocchio::{
    AccountView, Address, ProgramResult,
    error::ProgramError,
    sysvars::{rent::Rent, Sysvar},
    cpi::{Signer, Seed},
};
use pinocchio_log::log;
use pinocchio_associated_token_account::{
    instructions::{Create, CreateIdempotent},
};
use crate::config::Config;
use crate::instructions::*;

pub struct InitializeAccounts<'a> {
    // Program's authority
    pub initializer: &'a AccountView,
    pub token_program: &'a AccountView,
    pub ata_token_program: &'a AccountView,
    pub system_program: &'a AccountView,
}

impl<'a> TryFrom<&'a [AccountView]> for InitializeAccounts<'a> {
    type Error = ProgramError;
    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [
            initializer, token_program,
            ata_token_program, system_program
        ] = accounts else {
            return Err(ProgramError::InvalidAccountData);
        };
        signer_check(initializer)?;
        Ok(Self {
            initializer, token_program,
            ata_token_program, system_program
        })
    }
}

// C layout and do not include padding.
#[repr(C, packed)]
pub struct InitializeInstructionData {
    pub dispense_limit: u64,
    pub protocol_version: u8,
}

impl<'a> TryFrom<&'a [u8]> for InitializeInstructionData {
    type Error = ProgramError;
    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        if data.len() != core::mem::size_of::<InitializeInstructionData>() {
            return Err(ProgramError::InvalidInstructionData);
        }
        let dispense_limit = u64::from_le_bytes(data[0..8].try_into().unwrap());
        let protocol_version = data[8];

        Ok(Self {dispense_limit, protocol_version})
    }
}

pub struct Initialize<'a> {
    pub accounts: InitializeAccounts<'a>,
    pub instruction_data: InitializeInstructionData,
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountView])> for Initialize<'a> {
    type Error = ProgramError;
    fn try_from((data, ix_accounts): (&'a [u8], &'a [AccountView])) -> Result<Self, Self::Error> {
        let accounts = InitializeAccounts::try_from(ix_accounts)?;
        let instruction_data = InitializeInstructionData::try_from(data)?;

        Ok(Self { accounts, instruction_data })
    }
}

impl<'a> Initialize<'a> {
    pub const DISCRIMINATOR: &'a u8 = &0;
    pub fn process(&mut self) -> ProgramResult {
        // Derive and create the config PDA.
        // Create mints for token x and y from PDA.
        // Use mint + owner(PDA) to create ATA that holds the tokens.
        // Mint supply of the tokens to the ATA.
        log!("Initializing the megaswap-faucet protocol");
        Ok(())
    }
}
