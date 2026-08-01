//! Account checkers
use pinocchio::{
    AccountView, error::ProgramError, ProgramResult,
    Address, sysvars::{rent::Rent, Sysvar},
    cpi::{Signer},
};
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::instructions::{
    InitializeMint2, InitializeAccount3,
    MintTo, TransferChecked, Burn,
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

pub struct MintAccount;
impl MintAccount {
    pub fn check(account: &AccountView) -> Result<(), ProgramError> {
        if !account.owned_by(&pinocchio_token::ID) {
            return Err(ProgramError::IncorrectProgramId);
        }
        if account.data_len() != pinocchio_token::state::Mint::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }

    pub fn init(
        account: &AccountView, payer: &AccountView, decimals: u8,
        mint_authority: &Address, mint_signer: &[Signer],
        freeze_authority: Option<&Address>
    ) -> ProgramResult {
        let rent = Rent::get()?;
        let lamports = rent.minimum_balance(pinocchio_token::state::Mint::LEN);
        // Creating the mint account. we use invoke signer here as the 
        // account being created is a PDA
        CreateAccount {
            from: payer,
            to: account,
            lamports,
            space: pinocchio_token::state::Mint::LEN as u64,
            owner: &pinocchio_token::ID,
        }.invoke_signed(&mint_signer)?;
        // Initializing the mint account
        InitializeMint2 {
            mint: account,
            decimals,
            mint_authority,
            freeze_authority,
        }.invoke()
    }

    pub fn init_if_needed(
        account: &AccountView, payer: &AccountView,
        decimals: u8, mint_authority: &Address,
        mint_signer: &[Signer], freeze_authority: Option<&Address>
    ) -> ProgramResult {
        match Self::check(account) {
            Ok(_) => Ok(()),
            Err(_) => Ok(Self::init(account, payer, decimals, mint_authority, mint_signer, freeze_authority)?),
        }
    }
}


pub struct TokenAccount;
impl TokenAccount {
    pub fn check(account: &AccountView) -> Result<(), ProgramError> {
        if !account.owned_by(&pinocchio_token::ID) {
            return Err(ProgramError::IncorrectProgramId);
        }
        if account.data_len().ne(&pinocchio_token::state::Account::LEN) {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }

    pub fn init(
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

    pub fn init_if_needed(
        account: &AccountView, mint: &AccountView,
        payer: &AccountView, owner: &Address
    ) -> ProgramResult {
        match Self::check(account) {
            Ok(_) => Ok(()),
            Err(_) => Ok(Self::init(account, mint, payer, owner)?),
        }
    }

    pub fn mint_tokens(
        mint: &AccountView, account: &AccountView,
        authority: &AccountView, amount: u64,
        mint_signer: &[Signer]
    ) -> ProgramResult {
        MintTo::<&AccountView> {
            mint,
            account,
            mint_authority: authority,
            multisig_signers: &[],
            amount
        }.invoke_signed(&mint_signer)
    }

    pub fn burn_tokens(
        mint: &AccountView,
        from: &AccountView,
        authority: &AccountView,
        amount: u64,
        signer_seeds: Option<&[Signer]>,
    ) -> ProgramResult {
        let burn_ix = Burn::<&AccountView> {
            mint,
            account: from,
            authority,
            multisig_signers: &[],
            amount,
        };
        match signer_seeds {
            Some(seeds) => burn_ix.invoke_signed(&seeds),
            None => burn_ix.invoke(),
        }
    }

    pub fn transfer_tokens(
        from: &AccountView,
        to: &AccountView,
        mint: &AccountView,
        authority: &AccountView,
        amount: u64,
        decimals: u8,
        signer_seeds: Option<&[Signer]>,
    ) -> ProgramResult {
        let transfer_ix = TransferChecked::<&AccountView> {
            from, mint, to, authority,
            multisig_signers: &[],
            amount, decimals,
        };
        match signer_seeds {
            Some(seeds) => transfer_ix.invoke_signed(&seeds),
            None => transfer_ix.invoke(),
        }
    }
}
