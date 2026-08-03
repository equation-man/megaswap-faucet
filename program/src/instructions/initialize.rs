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
use crate::instructions::{
    account_checkers::*,
    helpers::*,
};

pub struct InitializeAccounts<'a> {
    // Program's authority
    pub initializer: &'a AccountView,
    pub config: &'a mut AccountView,
    pub mint_x: &'a AccountView,
    pub mint_y: &'a AccountView,
    pub vault_x_ata: &'a AccountView,
    pub vault_y_ata: &'a AccountView,
    pub token_program: &'a AccountView,
    pub ata_token_program: &'a AccountView,
    pub system_program: &'a AccountView,
}

impl<'a> TryFrom<&'a mut [AccountView]> for InitializeAccounts<'a> {
    type Error = ProgramError;
    fn try_from(accounts: &'a mut [AccountView]) -> Result<Self, Self::Error> {
        let [
            initializer, config, mint_x,
            mint_y, vault_x_ata, vault_y_ata,
            token_program, ata_token_program, system_program
        ] = accounts else {
            return Err(ProgramError::InvalidAccountData);
        };
        signer_check(initializer)?;
        Ok(Self {
            initializer, config, mint_x, mint_y,
            vault_x_ata, vault_y_ata, token_program,
            ata_token_program, system_program
        })
    }
}

// C layout and do not include padding.
#[repr(C, packed)]
pub struct InitializeInstructionData {
    pub seed_amount: u64,
    pub mint_decimals: u8,
    pub x_decimals: u8,
    pub y_decimals: u8,
    pub protocol_version: u8,
}

impl<'a> TryFrom<&'a [u8]> for InitializeInstructionData {
    type Error = ProgramError;
    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        if data.len() != core::mem::size_of::<InitializeInstructionData>() {
            return Err(ProgramError::InvalidInstructionData);
        }
        let seed_amount = u64::from_le_bytes(
            data[0..8].try_into().map_err(|_| ProgramError::InvalidInstructionData)?
        );
        let mint_decimals = data[8];
        let x_decimals = data[9];
        let y_decimals = data[10];
        let protocol_version = data[11];

        Ok(Self {
            seed_amount, mint_decimals,
            x_decimals, y_decimals, protocol_version,
        })
    }
}

pub struct Initialize<'a> {
    pub accounts: InitializeAccounts<'a>,
    pub instruction_data: InitializeInstructionData,
}

impl<'a> TryFrom<(&'a [u8], &'a mut [AccountView])> for Initialize<'a> {
    type Error = ProgramError;
    fn try_from((data, ix_accounts): (&'a [u8], &'a mut [AccountView])) -> Result<Self, Self::Error> {
        let accounts = InitializeAccounts::try_from(ix_accounts)?;
        let instruction_data = InitializeInstructionData::try_from(data)?;

        Ok(Self { accounts, instruction_data })
    }
}

impl<'a> Initialize<'a> {
    pub const DISCRIMINATOR: &'a u8 = &0;
    pub fn process(&mut self) -> ProgramResult {
        // Derive and create the config PDA.
        let version_seed = self.instruction_data.protocol_version.to_le_bytes();
        // Deriving the PDA address
        let (protocol_config_pda, config_pda_bump) = Address::find_program_address(
            &[b"config", &version_seed],
            &crate::ID.into()
        );
        let config_bump = [config_pda_bump];
        let binding = [
            Seed::from(b"config"),
            Seed::from(&version_seed),
            Seed::from(&config_bump),
        ];
        let signer_seeds = [Signer::from(&binding)];
        ProgramAccount::init::<Config>(
            self.accounts.initializer,
            self.accounts.config,
            &signer_seeds,
        )?;
        // Create mints for token x and y from PDA.
        let (expected_mint_x, mint_x_bump) = Address::find_program_address(
            &[b"mint_x", protocol_config_pda.as_ref()], &crate::ID.into()
        );
        let mint_x_bump_binding = [mint_x_bump];
        let mint_config_binding = protocol_config_pda.as_ref();
        let mint_x_signer_seeds = [
            Seed::from(b"mint_x"),
            Seed::from(mint_config_binding),
            Seed::from(&mint_x_bump_binding),
        ];
        let mint_x_signer = [Signer::from(&mint_x_signer_seeds)];
        MintAccount::init(
            self.accounts.mint_x,
            self.accounts.initializer,
            self.instruction_data.x_decimals,
            &protocol_config_pda,
            &mint_x_signer,
            None // Tokens are not freezable
        )?;
        let (expected_mint_y, mint_y_bump) = Address::find_program_address(
            &[b"mint_y", protocol_config_pda.as_ref()], &crate::ID.into()
        );
        let mint_y_bump_binding = [mint_y_bump];
        let mint_y_signer_seeds = [
            Seed::from(b"mint_y"),
            Seed::from(mint_config_binding),
            Seed::from(&mint_y_bump_binding),
        ];
        let mint_y_signer = [Signer::from(&mint_y_signer_seeds)];
        MintAccount::init(
            self.accounts.mint_y,
            self.accounts.initializer,
            self.instruction_data.y_decimals,
            &protocol_config_pda,
            &mint_y_signer,
            None, // Tokens are not freezable.
        )?;
        // Use mint + owner(PDA) to create ATA that holds the tokens.
        // ATA for token x
        AssociatedTokenAccount::init(
            &self.accounts.vault_x_ata,
            &self.accounts.mint_x,
            &self.accounts.initializer,
            &self.accounts.config, // Owned by the config PDA
            &self.accounts.system_program,
            &self.accounts.token_program,
            &self.accounts.ata_token_program,
        )?;
        AssociatedTokenAccount::init(
            &self.accounts.vault_y_ata,
            &self.accounts.mint_y,
            &self.accounts.initializer,
            &self.accounts.config, // Owned by the config PDA
            &self.accounts.system_program,
            &self.accounts.token_program,
            &self.accounts.ata_token_program,
        )?;
        // Mint supply of the tokens to the ATAs.
        TokenAccount::mint_tokens(
            self.accounts.mint_x,
            self.accounts.vault_x_ata,
            self.accounts.config, // Owner or mint authority is config PDA
            self.instruction_data.seed_amount,
            &signer_seeds, // This is the owner pda signer seeds
        )?;
        TokenAccount::mint_tokens(
            self.accounts.mint_y,
            self.accounts.vault_y_ata,
            self.accounts.config, // Owner or mint authority is config PDA
            self.instruction_data.seed_amount,
            &signer_seeds, // This is the owner pda signer seeds
        )?;

        // Saving adding configuration to config PDA.
        let mut config = Config::load_mut(self.accounts.config)?;
        config.set_inner(
            expected_mint_x.to_bytes(),
            self.instruction_data.x_decimals,
            expected_mint_y.to_bytes(),
            self.instruction_data.y_decimals,
            self.accounts.vault_x_ata.address().to_bytes(),
            self.accounts.vault_y_ata.address().to_bytes(),
            self.instruction_data.protocol_version,
        )?;
        Ok(())
    }
}
