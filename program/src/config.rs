//! Protocol configuration.
use solana_account_view::{Ref, RefMut};
use solana_address;
use pinocchio::{
    AccountView, Address,
    error::ProgramError,
};

#[repr(C)]
pub struct Config {
    pub limit: [u8; 8], // Token limit to dispense.
    pub mint_x: Address, // Mint address for token x.
    pub mint_y: Address, // Mint address for token y.
    pub vault_x_ata: Address, // ATA for token x pool.
    pub vault_y_ata: Address, // ATA for token y pool.
    pub version: u8, // Protocol version.
}

impl Config {
    pub const LEN: usize = core::mem::size_of::<Self>();
    pub const ALIGN: usize = core::mem::align_of::<Self>();

    // ============== READING DATA ======================
    // Ensures runtime enforced safety, where another mutable reference
    // cant be assigned when data is still held.
    #[inline(always)]
    pub fn load(account_info: &AccountView) -> Result<Ref<'_, Self>, ProgramError> {

        // Validate size
        if account_info.data_len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        // Ownership checks
        if account_info.owner() != &Address::from(crate::ID) {
            return Err(ProgramError::InvalidAccountData);
        }

        // Borrow is scoped and enforced. Contains a borrow guard,
        // solana_account_view::Ref<[u8]> when we intend to return it,
        // we'll enforce Ref::map() since the borrowed data will need to
        // escape the function.
        let data = account_info.try_borrow()?;

        // Alignment checks.
        if (data.as_ptr() as usize) % Self::ALIGN != 0 {
            return Err(ProgramError::InvalidAccountData);
        }

        // Ref::map() here helps the projected reference to escape the function
        // while the guard is still alive.
        Ok(Ref::map(data, |bytes: &[u8]| {
            // Minimizing the unsafe block
            let ptr = bytes.as_ptr() as *const Self;
            unsafe { &*ptr }
        }))
    }

    // # SAFETY.
    // Ensure no mut borrow can occur. when still holding a reference before hand
    // No reentrant CPI (into this program or another) mutates this account's data
    // for the duration the returned reference is held.
    //
    // Violating either of these is UB: It produces a live shared reference aliased with a mutable
    // one into the same memory
    #[inline(always)]
    pub unsafe fn load_unchecked(account_info: &AccountView) -> Result<&Self, ProgramError> {
        // Size check
        if account_info.data_len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        // Ownership check
        if account_info.owner() != &Address::from(crate::ID) {
            return Err(ProgramError::InvalidAccountData);
        }

        // SAFETY: caller guarantees no aliasing mutable borrow exists pwer this function's 
        // `# Safety` contract
        let data = unsafe { account_info.borrow_unchecked() };

        // Alignment check
        if (data.as_ptr() as usize) % Self::ALIGN != 0 {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(unsafe {
            // SAFETY: length, owner, alignment already checked above; aliasing guaranteed
            // by caller
            Self::from_bytes_unchecked(data)
        })
    }

    // Safe conversion.
    pub fn from_bytes(bytes: &[u8]) -> Result<&Self, ProgramError> {
        if bytes.len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        let ptr = bytes.as_ptr();
        // Checking alignment
        if (ptr as usize) % Self::ALIGN != 0 {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(unsafe { Self::from_bytes_unchecked(bytes) })
    }

    // Returns Config from the given bytes, without any check.
    // # SAFETY.
    // Caller must guarantee that:
    // - bytes contains at least size_of::<Self>() bytes.
    // - bytes properly aligns for Self.
    // - The bytes represent a valid Self.
    // - No mutable reference aliases these bytes while the returned reference exists.
    #[inline(always)]
    pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Self {
        // Pointer reinterpretation
        &*(bytes.as_ptr() as *const Self)
    }

    #[inline(always)]
    pub fn limit(&self) -> u64 { u64::from_le_bytes(self.limit) }
    #[inline(always)]
    pub fn mint_x(&self) -> &Address { &self.mint_x }
    #[inline(always)]
    pub fn mint_y(&self) -> &Address { &self.mint_y }
    #[inline(always)]
    pub fn vault_x_ata(&self) -> &Address { &self.vault_x_ata }
    #[inline(always)]
    pub fn vault_y_ata(&self) -> &Address { &self.vault_y_ata }
    #[inline(always)]
    pub fn version(&self) -> u8 { self.version }

    // ==================== WRITING DATA ===========================
    // Return mutable Config from given bytes.
    #[inline(always)]
    pub unsafe fn from_bytes_unchecked_mut(bytes: &mut [u8]) -> &mut Self {
        &mut *(bytes.as_mut_ptr() as *mut Config)
    }

    #[inline(always)]
    pub fn load_mut(account_info: &mut AccountView) -> Result<RefMut<'_, Self>, ProgramError> {
        // Validating size
        if account_info.data_len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        // Ownership checks
        if account_info.owner() != &Address::from(crate::ID) {
            return Err(ProgramError::InvalidAccountData);
        }

        // Here we have solana_account_view::RefMut<[u8]>
        // We will need to keep the borrow guard.
        let data = account_info.try_borrow_mut()?;

        // Alignment check.
        if (data.as_ptr() as usize) % core::mem::align_of::<Self>() != 0 {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(RefMut::map(data, |bytes: &mut [u8]| {
            // This is safe since it just reads the pointer's internal field
            // The cast(*mut Self) just reinterprets how the next operation should treat the bytes
            // at that address, and no memory access has happened yet.
            let ptr = bytes.as_mut_ptr() as *mut Self;
            // This is unsafe since it now reads the memory at the address, to produce a place.
            // Here, we have already assert the safety concerns that the compiler cannot verify,
            // e.g alignment etc so we ran the memory transformation in unsafe so compiler doesn't
            // complain. However, we don't need to worry since we have done the checks above.
            unsafe { &mut *ptr }
        }))
    }

    #[inline(always)]
    pub fn set_limit(&mut self, limit: u64) -> Result<(), ProgramError> {
        self.limit = limit.to_le_bytes();
        Ok(())
    }

    #[inline(always)]
    pub fn set_mint_x(&mut self, mint_x: [u8; 32]) -> Result<(), ProgramError> {
        self.mint_x = mint_x.into();
        Ok(())
    }

    #[inline(always)]
    pub fn set_mint_y(&mut self, mint_y: [u8; 32]) -> Result<(), ProgramError> {
        self.mint_y = mint_y.into();
        Ok(())
    }

    pub fn set_vault_x_ata(&mut self, vault_x_ata: [u8; 32]) -> Result<(), ProgramError> {
        self.vault_x_ata = vault_x_ata.into();
        Ok(())
    }

    pub fn set_vault_y_ata(&mut self, vault_y_ata: [u8; 32]) -> Result<(), ProgramError> {
        self.vault_y_ata = vault_y_ata.into();
        Ok(())
    }

    pub fn set_version(&mut self, version: u8) -> Result<(), ProgramError> {
        self.version = version as u8;
        Ok(())
    }

    #[inline(always)]
    pub fn set_inner(
        &mut self,
        limit: u64, mint_x: [u8; 32], mint_y: [u8; 32],
        vault_x_ata: [u8; 32], vault_y_ata: [u8; 32],
        version: u8
    ) -> Result<(), ProgramError> {
        self.set_limit(limit);
        self.set_mint_x(mint_x);
        self.set_mint_y(mint_y);
        self.set_vault_x_ata(vault_x_ata);
        self.set_vault_y_ata(vault_y_ata);
        self.set_version(version);
        Ok(())
    }

}

// COMPILE TIME ASSERTIONS.
const _: () = {
    assert!(core::mem::size_of::<Config>() == Config::LEN);
};
// We have an alignment of 1. All values are u8
const _: () = {
    assert!(core::mem::align_of::<Config>() == 1);
};
