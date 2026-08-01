//! Contains utility function for the test.
use std::{fs, path::PathBuf};
use litesvm::LiteSVM;
use solana_transaction::Transaction;
use solana_message::Message;
use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::Keypair;
use solana_signer::Signer;
use solana_pubkey::Pubkey;
use spl_token::ID as TOKEN_PROGRAM_ID;
use solana_program::sysvar::instructions::ID as SYSVARS_ID;
use solana_system_interface::program::ID as SYSTEM_PROGRAM_ID;
use spl_associated_token_account::ID as ASSOCIATED_TOKEN_PROGRAM_ID;
use spl_associated_token_account::{
    get_associated_token_address
};

use crate::helpers::{
    get_token_balance,
};

pub struct MegaSwapFaucetCtx {
    pub svm: LiteSVM,
    // Protocol's authority
    pub initializer: Keypair,
}

pub fn initialize_protocol(program_id: Pubkey) -> MegaSwapFaucetCtx {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // Go back to parent dir
    path.push("target");
    path.push("deploy");
    path.push("megaswap_faucet.so");
    let bytes = fs::read(&path).expect(&format!("Couldn't read {:?}", path));
    
    let  mut svm = LiteSVM::new();
    svm.add_program(program_id, &bytes);

    // Accounts needed for initializing the protocol
    let initializer = Keypair::new();
    svm.airdrop(&initializer.pubkey(), 5_000_000_000).unwrap();

    // Data for instruction.
    let dispense_limit = 1000_000u64;
    let mint_decimals = 6u8;
    let protocol_version = 1u8;
    let mut ix_data = vec![0u8];
    ix_data.extend_from_slice(&dispense_limit.to_le_bytes());
    ix_data.push(mint_decimals);
    ix_data.push(protocol_version);

    // ----- PDAs -----
    let (config_pda, config_bump) = Pubkey::find_program_address(
        &[b"config", &protocol_version.to_le_bytes()], &program_id,
    );

    let (mint_x, mint_x_bump) = Pubkey::find_program_address(
        &[b"mint_x", &config_pda.as_ref()], &program_id
    );
    let (mint_y, mint_y_bump) = Pubkey::find_program_address(
        &[b"mint_y", &config_pda.as_ref()], &program_id
    );
    let vault_x_ata = get_associated_token_address(&config_pda, &mint_x);
    let vault_y_ata = get_associated_token_address(&config_pda, &mint_y);

    let accounts = vec![
        AccountMeta::new(initializer.pubkey(), true),
        AccountMeta::new(config_pda, false),
        AccountMeta::new(mint_x, false),
        AccountMeta::new(mint_y, false),
        AccountMeta::new(vault_x_ata, false),
        AccountMeta::new(vault_y_ata, false),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(ASSOCIATED_TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
    ];
    let ix = Instruction::new_with_bytes(program_id, &ix_data, accounts);
    let tx = Transaction::new(
        &[&initializer],
        Message::new(&[ix], Some(&initializer.pubkey())),
        svm.latest_blockhash()
    );

    let tx_init = svm.send_transaction(tx);
    //println!("Test initializeng the protocol {:#?}", tx_init);
    let vault_x_bal = get_token_balance(&svm, &vault_y_ata);
    assert_eq!(vault_x_bal, dispense_limit);
    let vault_y_bal = get_token_balance(&svm, &vault_y_ata);
    assert_eq!(vault_y_bal, dispense_limit);
    assert_eq!(vault_x_bal, vault_y_bal);

    MegaSwapFaucetCtx {
        svm, initializer
    }
}
