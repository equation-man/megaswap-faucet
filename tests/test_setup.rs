//! Setting up testing environment.
use litesvm::LiteSVM;
use solana_sdk::{
    signature::{Signer, Keypair},
};

pub struct MegaSwapFaucetCtx {
    pub svm: LiteSVM,
    // Protocol's authority
    pub initializer: Keypair,
}

pub fn initialize_protocol() -> MegaSwapFaucetCtx {
    let program_id = solana_sdk::pubkey!("9uwR3ZyHXhnA2QvPDHtjg5ei3AT9VTzst6pbzj6eQjLn");
    //let bytes = include_bytes!("../target/deploy/");
    let  mut svm = LiteSVM::new();
    svm.add_program(program_id, bytes);
}
