//! Dispensing tokens to wallets
use solana_address;
use pinocchio::{
    AccountView, Address, ProgramResult,
    error::ProgramError, cpi::{Signer, Seed},
};
use pinocchio_log::log;
use crate::config::Config;
use crate::instructions::{
    account_checkers::*,
};

pub struct DispenseAccounts<'a> {
    // Program's authority.
    pub destination_wallet: &'a AccountView,
    pub config: &'a AccountView,
}

impl<'a> TryFrom<&'a mut [AccountView]> for DispenseAccounts<'a> {
    type Error = ProgramError;
    fn try_from(accounts: &'a mut [AccountView]) -> Result<Self, Self::Error> {
        let [
            destination_wallet, config,
        ] = accounts else {
            return Err(ProgramError::InvalidAccountData);
        };
        signer_check(destination_wallet)?;
        Ok(Self { destination_wallet, config })
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
            data[0..8].try_into().map_err(|_| ProgramError::InvalidInstructionData)?
        );

        Ok(Self { amount })
    }
}

pub struct Dispense<'a> {
    pub accounts: DispenseAccounts<'a>,
    pub instruction_data: DispenseIxData,
}

impl<'a> TryFrom<(&'a [u8], &'a mut [AccountView])> for Dispense<'a> {
    type Error = ProgramError;
    fn try_from((data, ix_accounts): (&'a [u8], &'a mut [AccountView])) -> Result<Self, Self::Error> {
        let accounts = DispenseAccounts::try_from(ix_accounts)?;
        let instruction_data = DispenseIxData::try_from(data)?;

        Ok(Self { accounts, instruction_data })
    }
}

impl<'a> Dispense<'a> {
    pub const DISCRIMINATOR: &'a u8 = &1;
    pub fn process(&mut self) -> ProgramResult {
        // Dispense token to a given wallet.
        // Load the config account
        let config = Config::load(self.accounts.config)?;
        let version_seed = config.version.to_le_bytes();
        let (proto_config_pda, pda_bump) = Address::find_program_address(
            &[b"config", &version_seed],
            &crate::ID.into()
        );
        let config_bump_binding = [pda_bump];
        let binding = [
            Seed::from(b"config"),
            Seed::from(&version_seed),
            Seed::from(&config_bump_binding),
        ];
        let signer_seeds = [Signer::from(&binding)];
        // from, to, mint, authority, amount, decimals, signer_seeds
        //TokenAccount::transfer_tokens(
        //    config.vault_x_ata,
        //    self.accounts.destination_wallet,
        //    config.mint_x,
        //    self.accounts.config,
        //    self.instruction_data.amount,
        //    config.x_decimal,
        //    Some(&siner_seeds),
        //)?;
        log!("Dispensing tokens {} to trader wallet", self.instruction_data.amount);
        Ok(())
    }
}
