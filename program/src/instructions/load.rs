//! Loading more tokens into the faucet
use solana_address;
use pinocchio::{
    AccountView, Address, ProgramResult,
    error::ProgramError,
};
use pinocchio_log::log;

use crate::config::Config;
use crate::instructions::*;

pub struct LoadingAccounts<'a> {
    // Program's authority.
    pub initializer: &'a AccountView,
    pub config: &'a AccountView,
}

impl<'a> TryFrom<&'a [AccountView]> for LoadingAccounts<'a> {
    type Error = ProgramError;
    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [
            initializer, config
        ] = accounts else {
            return Err(ProgramError::InvalidAccountData);
        };
        signer_check(initializer)?;
        Ok(Self { initializer, config })
    }
}

#[repr(C, packed)]
pub struct LoadIxData {
    pub amount_to_mint: u64,
}

impl<'a> TryFrom<&'a [u8]> for LoadIxData {
    type Error = ProgramError;
    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        if data.len() != core::mem::size_of::<LoadIxData>() {
            return Err(ProgramError::InvalidInstructionData);
        }
        let amount_to_mint = u64::from_le_bytes(
            data[..8].try_into().map_err(|_| ProgramError::InvalidInstructionData)?
        );

        Ok(Self { amount_to_mint })
    }
}

pub struct LoadTokens<'a> {
    pub accounts: LoadingAccounts<'a>,
    pub instruction_data: LoadIxData,
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountView])> for LoadTokens<'a> {
    type Error = ProgramError;
    fn try_from((data, ix_accounts): (&'a [u8], &'a [AccountView])) -> Result<Self, Self::Error> {
        let accounts = LoadingAccounts::try_from(ix_accounts)?;
        let instruction_data = LoadIxData::try_from(data)?;

        Ok(Self { accounts, instruction_data })
    }
}

impl<'a> LoadTokens<'a> {
    pub const DISCRIMINATOR: &'a u8 = &2;
    pub fn process(&mut self) -> ProgramResult {
        // Mint tokens into the faucet
        log!("Loading tokens into faucet");
        Ok(())
    }
}
