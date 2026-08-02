//! Creating PDAs and ATA
use pinocchio::{
    AccountView, Address, error::ProgramError,
    ProgramResult, sysvars::{rent::Rent, Sysvar},
    cpi::{Signer},
};
use pinocchio_system::instructions::CreateAccount;
use pinocchio_associated_token_account::{
    instructions::{Create, CreateIdempotent},
    ID as ATA_ID,
};
use pinocchio_log::log;
use pinocchio::Resize;
use crate::instructions::account_checkers::TokenAccount;

// Trait that all the account types implement.
// Any account that has LEN property should implement this trait
// Static dispatch: zero cost. Implemented for small generic functions
pub trait AccountData {
    const LEN: usize;
}

pub struct AssociatedTokenAccount;
impl AssociatedTokenAccount {
    pub fn check(
        account: &AccountView, authority: &AccountView,
        mint: &AccountView, token_program: &AccountView
    ) -> ProgramResult {
        // Validating token account structure and owner.
        // We are obtaining the reference to the token account and performing layout checks.
        let token = TokenAccount::load(account)?;
        TokenAccount::check_owner(token, authority.address())?;
        // Canonical ATA check
        TokenAccount::check_mint(token, mint.address())?;
        let (expected, _) = Address::find_program_address(
            &[
                authority.address().as_ref(),
                token_program.address().as_ref(),
                mint.address().as_ref()
            ],
            &ATA_ID,
        );
        if expected != *account.address() {
            return Err(ProgramError::InvalidSeeds);
        }
        Ok(())
    }

    pub fn init(
        account: &AccountView, mint: &AccountView,
        payer: &AccountView, owner: &AccountView,
        system_program: &AccountView, token_program: &AccountView,
        ata_program: &AccountView,
    ) -> ProgramResult {
        CreateIdempotent {
            funding_account: payer,
            account,
            wallet: owner,
            mint,
            system_program,
            token_program,
        }.invoke()
    }

    pub fn init_if_needed(
        account: &AccountView, mint: &AccountView,
        payer: &AccountView, owner: &AccountView,
        system_program: &AccountView, token_program: &AccountView,
        ata_program: &AccountView,
    ) -> ProgramResult {
        match Self::check(account, payer, mint, token_program) {
            Ok(_) => Ok(()),
            Err(_) => Ok(Self::init(
                        account, mint, payer, owner,
                        system_program, token_program,
                        ata_program
                    )?),
        }
    }
}

pub struct ProgramAccount;
impl AccountData for ProgramAccount {
    const LEN: usize = core::mem::size_of::<Self>();
}
impl ProgramAccount {
    pub const LEN: usize = core::mem::size_of::<Self>();
    pub fn check<T: AccountData>(account: &AccountView) -> ProgramResult {
        // Check if this account is owned by this program.
        if !account.owned_by(&Address::new_from_array(crate::ID)) {
            return Err(ProgramError::IllegalOwner);
        }
        // Check account data length if it matches expected size
        if account.data_len().ne(&T::LEN) {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }

    pub fn init<T: AccountData>(
        payer: &AccountView, account: &AccountView, signer: &[Signer]
    ) -> ProgramResult {
        // Getting required lamports for rent.
        let space = core::mem::size_of::<T>();
        let rent = Rent::get()?;
        let lamports = rent.try_minimum_balance(space)?;
        // Creating the account.
        CreateAccount {
            from: payer,
            to: account,
            lamports,
            space: space as u64,
            owner: &Address::new_from_array(crate::ID),
        }.invoke_signed(&signer)?;
        Ok(())
    }

    pub fn close(account: &mut AccountView, destination: &mut AccountView) -> ProgramResult {
        {
            // Scope is introduced so mutable access to be dropped
            // as other operations like account.resize() will require it.
            let mut data = account.try_borrow_mut()?;
            // This marks the account as closed, preventing it from being mistaken
            // later as a valid account.
            data[0] = 0xff; // Wrting a tombstone marker
        }

        let new_balance = destination.lamports() + account.lamports();
        destination.set_lamports(new_balance);
        account.set_lamports(0);
        // Shrink the account to 1 byte instead of storing many bytes.
        account.resize(1)?;
        account.close()
    }
}
