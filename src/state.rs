use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct PlatformState {
    pub is_initialized: bool,
    pub treasury_account: Pubkey,
    pub platform_fess: u64,
}

impl Sealed for PlatformState {}
impl IsInitialized for PlatformState {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}
impl Pack for PlatformState {
    const LEN: usize = 41;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, PlatformState::LEN];
        let (is_initialized, treasury_account, platform_fess) = array_refs![src, 1, 32, 8];
        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };
        Ok(PlatformState {
            is_initialized,
            treasury_account: Pubkey::new_from_array(*treasury_account),
            platform_fess: u64::from_le_bytes(*platform_fess),
        })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, PlatformState::LEN];
        let (is_initialized_dst, treasury_account_dst, base_percentage_dst) =
            mut_array_refs![dst, 1, 32, 8];
        let PlatformState {
            is_initialized,
            treasury_account,
            platform_fess,
        } = self;
        is_initialized_dst[0] = *is_initialized as u8;
        treasury_account_dst.copy_from_slice(treasury_account.as_ref());
        *base_percentage_dst = platform_fess.to_le_bytes();
    }
}

// Token state will be initialized for different spl-toeks

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct TokenState {
    pub is_initialized: bool,
    pub owner_pubkey: Pubkey,
    pub token_mint: Pubkey,
    pub total_no_of_tokens_listed: u64,
}
impl Sealed for TokenState {}
impl IsInitialized for TokenState {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}
impl Pack for TokenState {
    const LEN: usize = 73;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, TokenState::LEN];
        let (is_initialized, owner_pubkey, token_mint, total_no_of_tokens_listed) =
            array_refs![src, 1, 32, 32, 8];
        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };
        Ok(TokenState {
            is_initialized,
            owner_pubkey: Pubkey::new_from_array(*owner_pubkey),
            token_mint: Pubkey::new_from_array(*token_mint),
            total_no_of_tokens_listed: u64::from_le_bytes(*total_no_of_tokens_listed),
        })
    }
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, TokenState::LEN];
        let (is_initialized_dst, owner_pubkey_dst, token_mint_dst, total_no_of_tokens_listed_dst) =
            mut_array_refs![dst, 1, 32, 32, 8];
        let TokenState {
            is_initialized,
            owner_pubkey,
            token_mint,
            total_no_of_tokens_listed,
        } = self;
        is_initialized_dst[0] = *is_initialized as u8;
        owner_pubkey_dst.copy_from_slice(owner_pubkey.as_ref());
        token_mint_dst.copy_from_slice(token_mint.as_ref());
        *total_no_of_tokens_listed_dst = total_no_of_tokens_listed.to_le_bytes();
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct ListerState {
    pub is_initialized: bool,
    pub seller_pubkey: Pubkey,
    pub token_mint: Pubkey,
    pub token_account_pubkey: Pubkey,
    pub token_amount: u64,
    pub expected_amount_per_token: u64,
}
impl Sealed for ListerState {}
impl IsInitialized for ListerState {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}
impl Pack for ListerState {
    const LEN: usize = 113;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, ListerState::LEN];
        let (
            is_initialized,
            seller_pubkey,
            token_mint,
            token_account_pubkey,
            token_amount,
            expected_amount_per_token,
        ) = array_refs![src, 1, 32, 32, 32, 8, 8];
        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };
        Ok(ListerState {
            is_initialized,
            seller_pubkey: Pubkey::new_from_array(*seller_pubkey),
            token_mint: Pubkey::new_from_array(*token_mint),
            token_account_pubkey: Pubkey::new_from_array(*token_account_pubkey),
            token_amount: u64::from_le_bytes(*token_amount),
            expected_amount_per_token: u64::from_le_bytes(*expected_amount_per_token),
        })
    }
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, ListerState::LEN];
        let (
            is_initialized_dst,
            seller_pubkey_dst,
            token_mint_dst,
            token_account_pubkey_dst,
            token_amount_dst,
            expected_amount_per_token_dst,
        ) = mut_array_refs![dst, 1, 32, 32, 32, 8, 8];
        let ListerState {
            is_initialized,
            seller_pubkey,
            token_mint,
            token_account_pubkey,
            token_amount,
            expected_amount_per_token,
        } = self;
        is_initialized_dst[0] = *is_initialized as u8;
        seller_pubkey_dst.copy_from_slice(seller_pubkey.as_ref());
        token_mint_dst.copy_from_slice(token_mint.as_ref());
        token_account_pubkey_dst.copy_from_slice(token_account_pubkey.as_ref());
        *token_amount_dst = token_amount.to_le_bytes();
        *expected_amount_per_token_dst = expected_amount_per_token.to_le_bytes();
    }
}
