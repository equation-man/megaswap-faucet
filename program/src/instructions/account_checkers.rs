//! Account checkers
use pinocchio::{
    AccountView, error::ProgramError, ProgramResult,
    Address, sysvars::{rent::Rent, Sysvar}
};
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::instructions::{
    InitializeMint2, InitializeAccount3
};

pub fn signer_check(account: &AccountView) -> Result<(), ProgramError> {
    if !account.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    Ok(())
}

pub fn system_account_check(account: &AccountView) -> Result<(), ProgramError> {
    if !account.owned_by(&pinocchio_system::ID) {
        return Err(ProgramError::IncorrectProgramId);
    }
    Ok(())
}

pub struct Mint;
impl Mint {
    fn check(account: &AccountView) -> Result<(), ProgramError> {
        if !account.owned_by(&pinocchio_token::ID) {
            return Err(ProgramError::IncorrectProgramId);
        }
        if account.data_len() != pinocchio_token::state::Mint::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }

    fn init(
        account: &AccountView, payer: &AccountView, decimals: u8,
        mint_authority: &Address, freeze_authority: Option<&Address>
    ) -> ProgramResult {
        let rent = Rent::get()?;
        let lamports = rent.minimum_balance(pinocchio_token::state::Mint::LEN);
        // Creating the mint account
        CreateAccount {
            from: payer,
            to: account,
            lamports,
            space: pinocchio_token::state::Mint::LEN as u64,
            owner: &pinocchio_token::ID,
        }.invoke()?;
        // Initializing the mint account
        InitializeMint2 {
            mint: account,
            decimals,
            mint_authority,
            freeze_authority,
        }.invoke()
    }

    fn init_if_needed(
        account: &AccountView, payer: &AccountView,
        decimals: u8, mint_authority: &Address,
        freeze_authority: Option<&Address>
    ) -> ProgramResult {
        match Self::check(account) {
            Ok(_) => Ok(()),
            Err(_) => Ok(Self::init(account, payer, decimals, mint_authority, freeze_authority)?),
        }
    }
}


pub struct Token;
impl Token {
    fn check(account: &AccountView) -> Result<(), ProgramError> {
        if !account.owned_by(&pinocchio_token::ID) {
            return Err(ProgramError::IncorrectProgramId);
        }
        if account.data_len().ne(&pinocchio_token::state::Account::LEN) {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }

    fn init(
        account: &AccountView, mint: &AccountView,
        payer: &AccountView, owner: &Address
    ) -> ProgramResult {
        let rent = Rent::get()?;
        let lamports = rent.minimum_balance(pinocchio_token::state::Account::LEN);
        // Creating the token account.
        CreateAccount {
            from: payer,
            to: account,
            lamports,
            space: pinocchio_token::state::Account::LEN as u64,
            owner: &pinocchio_token::ID,
        }.invoke()?;

        // Initializing the token Account.
        InitializeAccount3 {
            account,
            mint,
            owner
        }.invoke()
    }

    fn init_if_needed(
        account: &AccountView, mint: &AccountView,
        payer: &AccountView, owner: &Address
    ) -> ProgramResult {
        match Self::check(account) {
            Ok(_) => Ok(()),
            Err(_) => Ok(Self::init(account, mint, payer, owner)?),
        }
    }
}
