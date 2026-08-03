//! Dispensing tokens to wallets
use solana_address;
use pinocchio::{
    AccountView, Address, ProgramResult,
    error::ProgramError, cpi::{Signer, Seed},
};
use pinocchio_log::log;
use crate::config::Config;
use crate::instructions::{
    dispense_controller::{
        SimpleRng, get_token_balance, calculate_faucet_payout,
    },
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

pub struct Dispense<'a> {
    pub accounts: DispenseAccounts<'a>,
}

impl<'a> TryFrom<(&'a [u8], &'a mut [AccountView])> for Dispense<'a> {
    type Error = ProgramError;
    fn try_from((data, ix_accounts): (&'a [u8], &'a mut [AccountView])) -> Result<Self, Self::Error> {
        let accounts = DispenseAccounts::try_from(ix_accounts)?;

        Ok(Self { accounts })
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

        // Obtaining the seed. This is not critical. We are just
        // ensuring tokens are minted interchangeably.
        let usr_accnt = self.accounts.destination_wallet.address().as_ref();
        let seed = u32::from_le_bytes([
            usr_accnt[0], usr_accnt[1],
            usr_accnt[2], usr_accnt[3],
        ]);
        let mut rng = SimpleRng::new(seed);
        // Chosing the token to dispense randomly
        let token_choice = (rng.next_u32() as usize) % 2;
        // Auto mint set up. It is okay to hardcode this in this context.
        let critical_floor = 10_000_u64; 
        if token_choice == 0 {
            // Proportional Controller settings. It is okay to hardcode this in this context.
            let faucet_balance = get_token_balance(self.accounts.vault_x_ata)?;
            let dec_multiplier = 10_u64.checked_pow(config.x_decimal as u32).ok_or(ProgramError::InvalidInstructionData)?;
            let target_floor = 100_000_u64.checked_mul(dec_multiplier).ok_or(ProgramError::InvalidInstructionData)?;
            let base_payout = 50_000_u64.checked_mul(dec_multiplier).ok_or(ProgramError::InvalidInstructionData)?;
            let min_payout = 20_000_u64.checked_mul(dec_multiplier).ok_or(ProgramError::InvalidInstructionData)?;
            let max_payout = 70_000_u64.checked_mul(dec_multiplier).ok_or(ProgramError::InvalidInstructionData)?;
            let k_scaled = 1 as u64;
            let precision = config.x_decimal as u128;
            // calculating the dispense amount.
            match calculate_faucet_payout(
                faucet_balance, target_floor, base_payout, k_scaled,
                precision, min_payout, max_payout,
            ) {
                Some(dispense_amount) => {
                    TokenAccount::transfer_tokens(
                        self.accounts.vault_x_ata,
                        self.accounts.destination_x_ata,
                        self.accounts.mint_x,
                        self.accounts.config,
                        dispense_amount,
                        config.x_decimal,
                        Some(&signer_seeds),
                    )?;
                    // Fire the automint if the token falls below a particular threshold.
                    let critical_bal = critical_floor.checked_mul(dec_multiplier).ok_or(ProgramError::InvalidInstructionData)?;
                    let mint_amount = target_floor.checked_sub(faucet_balance).ok_or(ProgramError::InvalidInstructionData)?;
                    if faucet_balance < critical_bal {
                        // Redeeming the dispensed x tokens.
                        TokenAccount::mint_tokens(
                            self.accounts.mint_x,
                            self.accounts.vault_x_ata,
                            self.accounts.config,
                            mint_amount,
                            &signer_seeds,
                        )?;
                    }
                },
                None => { return Ok(()) }
            }
        } else {
            // Proportional Controller settings. It is okay to hardcode this in this context.
            let faucet_balance = get_token_balance(self.accounts.vault_y_ata)?;
            let dec_multiplier = 10_u64.checked_pow(config.y_decimal as u32).ok_or(ProgramError::InvalidInstructionData)?;
            let target_floor = 100_000_u64.checked_mul(dec_multiplier).ok_or(ProgramError::InvalidInstructionData)?;
            let base_payout = 50_000_u64.checked_mul(dec_multiplier).ok_or(ProgramError::InvalidInstructionData)?;
            let min_payout = 20_000_u64.checked_mul(dec_multiplier).ok_or(ProgramError::InvalidInstructionData)?;
            let max_payout = 70_000_u64.checked_mul(dec_multiplier).ok_or(ProgramError::InvalidInstructionData)?;
            let k_scaled = 1 as u64;
            let precision = config.y_decimal as u128;
            // calculating the dispense amount.
            match calculate_faucet_payout(
                faucet_balance, target_floor, base_payout, k_scaled,
                precision, min_payout, max_payout,
            ) {
                Some(dispense_amount) => {
                    TokenAccount::transfer_tokens(
                        self.accounts.vault_y_ata,
                        self.accounts.destination_y_ata,
                        self.accounts.mint_y,
                        self.accounts.config,
                        dispense_amount,
                        config.y_decimal,
                        Some(&signer_seeds),
                    )?;
                    // Fire automint engine if faucet balance falls below critical level.
                    let critical_bal = critical_floor.checked_mul(dec_multiplier).ok_or(ProgramError::InvalidInstructionData)?;
                    let mint_amount = target_floor.checked_sub(faucet_balance).ok_or(ProgramError::InvalidInstructionData)?;
                    if faucet_balance < critical_bal {
                        // Redeeming the dispensed y tokens.
                        TokenAccount::mint_tokens(
                            self.accounts.mint_y,
                            self.accounts.vault_y_ata,
                            self.accounts.config,
                            mint_amount,
                            &signer_seeds,
                        )?;
                    }
                },
                None => { return Ok(()) }
            }
        }
        Ok(())
    }
}
