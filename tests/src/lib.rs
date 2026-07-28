//! MegaSwap faucet test entrypoint
#![allow(warnings)]
mod test_helpers;
use crate::test_helpers::{
    MegaSwapFaucetCtx, initialize_protocol,
};
use solana_pubkey::Pubkey;

const PROGRAM_ID: Pubkey = Pubkey::new_from_array(megaswap_faucet::ID);

#[test]
fn test_init_protocol() {
    let mut ctx_init = initialize_protocol(PROGRAM_ID);
}
