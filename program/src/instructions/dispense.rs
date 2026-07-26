//! Dispensing tokens to wallets
use solana_address;
use pinocchio::{
    AccountView, Address, ProgramResult,
    error::ProgramError,
};
use pinocchio_log::log;

pub struct DispenseAccounts<'a> {
    // Program's authority.
    pub initializer: &'a AccountView,
    pub destination_wallet: &'a AccountView,
}

impl<'a> TryFrom<&'a [AccountView]> for DispenseAccounts<'a> {
    type Error = ProgramError;
    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [
            initializer, destination_wallet
        ] = accounts else {
            return Err(ProgramError::InvalidAccountData);
        };
        Ok(Self { initializer, destination_wallet })
    }
}

pub struct DispenseIxData {
    amount: u64,
}

impl<'a> TryFrom<&'a [u8]> for DispenseIxData {
    type Error = ProgramError;
    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        if data.len() != core::mem::size_of::<DispenseIxData>() {
            return Err(ProgramError::InvalidInstructionData);
        }
        let amount = u64::from_le_bytes(
            data[..8].try_into().map_err(|_| ProgramError::InvalidInstructionData)?
        );

        Ok(Self { amount })
    }
}

pub struct Dispense<'a> {
    pub accounts: DispenseAccounts<'a>,
    pub instruction_data: DispenseIxData,
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountView])> for Dispense<'a> {
    type Error = ProgramError;
    fn try_from((data, ix_accounts): (&'a [u8], &'a [AccountView])) -> Result<Self, Self::Error> {
        let accounts = DispenseAccounts::try_from(ix_accounts)?;
        let instruction_data = DispenseIxData::try_from(data)?;

        Ok(Self { accounts, instruction_data })
    }
}

impl<'a> Dispense<'a> {
    pub const DISCRIMINATOR: &'a u8 = &2;
    pub fn process(&mut self) -> ProgramResult {
        // Dispense token to a given wallet.
        log!("Dispensing tokens to trader wallet");
        Ok(())
    }
}
