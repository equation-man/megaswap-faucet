//! Main test file. This is the entry to tests.
#![allow(warnings)]
pub mod test_setup;
use crate::test_setup::{
    MegaSwapFaucetCtx, initialize_protocol,
};

#[test]
fn test_init_protocol() {
    let mut ctx_init = initialize_protocol();
    println!("The initialized protocol context is {:#?}", ctx_init);
}
