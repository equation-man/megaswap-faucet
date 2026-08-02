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
    helpers::*,
};

pub struct DispenseAccounts<'a> {
    // Program's authority.
    pub destination_wallet: &'a AccountView,
    pub destination_x_ata: &'a AccountView,
    pub destination_y_ata: &'a AccountView,
    pub config: &'a AccountView,
    pub mint_x: &'a AccountView,
    pub mint_y: &'a AccountView,
    pub vault_x_ata: &'a AccountView,
    pub vault_y_ata: &'a AccountView,
    pub token_program: &'a AccountView,
}

impl<'a> TryFrom<&'a mut [AccountView]> for DispenseAccounts<'a> {
    type Error = ProgramError;
    fn try_from(accounts: &'a mut [AccountView]) -> Result<Self, Self::Error> {
        let [
            destination_wallet, destination_x_ata, destination_y_ata,
            config, mint_x, mint_y, vault_x_ata,
            vault_y_ata, token_program,
        ] = accounts else {
            return Err(ProgramError::InvalidAccountData);
        };
        signer_check(destination_wallet)?;
        // Load and check structural layout for mint x
        let x_mint = MintAccount::load(mint_x)?;
        MintAccount::check_initialized(x_mint)?;
        MintAccount::check_mint_authority(x_mint, config.address())?;
        // Load and check structural layout for mint y
        let y_mint = MintAccount::load(mint_y)?;
        MintAccount::check_initialized(y_mint)?;
        MintAccount::check_mint_authority(y_mint, config.address())?;
        // Verifying the token accounts.
        AssociatedTokenAccount::check(
            destination_x_ata, destination_wallet,
            mint_x, token_program
        )?;
        AssociatedTokenAccount::check(
            destination_y_ata, destination_wallet,
            mint_y, token_program
        )?;
        AssociatedTokenAccount::check(
            vault_x_ata, config,
            mint_x, token_program
        )?;
        AssociatedTokenAccount::check(
            vault_y_ata, config,
            mint_y, token_program
        )?;
        Ok(Self {
            destination_wallet, destination_x_ata,
            destination_y_ata, config, mint_x, mint_y,
            vault_x_ata, vault_y_ata, token_program,
        })
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
        TokenAccount::transfer_tokens(
            self.accounts.vault_x_ata,
            self.accounts.destination_x_ata,
            self.accounts.mint_x,
            self.accounts.config,
            self.instruction_data.amount,
            config.x_decimal,
            Some(&signer_seeds),
        )?;
         TokenAccount::transfer_tokens(
            self.accounts.vault_y_ata,
            self.accounts.destination_y_ata,
            self.accounts.mint_y,
            self.accounts.config,
            self.instruction_data.amount,
            config.x_decimal,
            Some(&signer_seeds),
        )?;
       log!("Dispensing tokens {} to trader wallet", self.instruction_data.amount);
        Ok(())
    }
}
