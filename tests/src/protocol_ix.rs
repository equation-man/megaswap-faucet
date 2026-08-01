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
    get_associated_token_address,
};

use crate::helpers::{
    get_token_balance, get_config_data,
};

pub struct MegaSwapFaucetCtx {
    pub svm: LiteSVM,
    // Protocol's authority
    pub initializer: Keypair,
    pub protocol_version: u8,
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
    let protocol_seed_amount = 500_000_000_000u64;
    let dispense_limit = 1_000_000u64;
    let mint_decimals = 6u8;
    let x_decimals = 6u8;
    let y_decimals = 6u8;
    let protocol_version = 1u8;
    let mut ix_data = vec![0u8];
    ix_data.extend_from_slice(&dispense_limit.to_le_bytes());
    ix_data.extend_from_slice(&protocol_seed_amount.to_le_bytes());
    ix_data.push(mint_decimals);
    ix_data.push(x_decimals);
    ix_data.push(y_decimals);
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
    //println!("Test initialize transaction: {:#?}", tx_init);
    let vault_x_bal = get_token_balance(&svm, &vault_x_ata);
    let vault_y_bal = get_token_balance(&svm, &vault_y_ata);
    // Getting config data.
    let config_disp_bal = get_config_data(&svm, &config_pda);

    assert_eq!(vault_x_bal, protocol_seed_amount, "Vault x wallet: Expected {}, but got instead: {}", protocol_seed_amount, vault_x_bal);
    assert_eq!(vault_y_bal, protocol_seed_amount, "Vault y wallet: Expected {}, but got instead: {}", protocol_seed_amount, vault_y_bal);
    // Confirms the vaults are seeded appropriately with funds
    assert_eq!(vault_x_bal, vault_y_bal, "Expected: Vault x {} == Vault y {}", vault_x_bal, vault_y_bal);
    // Confirmed the PDA is appropriately initialized
    assert_eq!(config_disp_bal, dispense_limit, "Expected dispense limit {}, got {}", dispense_limit, config_disp_bal);

    MegaSwapFaucetCtx {
        svm, initializer, protocol_version
    }
}

pub fn dispense_tokens(ctx: &mut MegaSwapFaucetCtx, program_id: Pubkey) {
    // Dispense date.
    let dispense_amount = 1_000_000u64;
    let mut ix_data = vec![1u8];
    ix_data.extend_from_slice(&dispense_amount.to_le_bytes());
    // accounts; config, destination_wallet
    let trader_wallet = Keypair::new();
    ctx.svm.airdrop(&trader_wallet.pubkey(), 5_000_000_000).unwrap();
    let (config_pda, config_bump) = Pubkey::find_program_address(
        &[b"config", &ctx.protocol_version.to_le_bytes()],
        &program_id
    );

    let accounts = vec![
        AccountMeta::new(trader_wallet.pubkey(), true),
        AccountMeta::new(config_pda, false),
    ];

    let ix = Instruction::new_with_bytes(program_id, &ix_data, accounts);
    let tx = Transaction::new(
        &[&trader_wallet],
        Message::new(&[ix], Some(&trader_wallet.pubkey())),
        ctx.svm.latest_blockhash()
    );

    let tx_disp = ctx.svm.send_transaction(tx);
    println!("Testing dispense transaction: {:#?}", tx_disp);

    //let trader_wallet_bal = get_token_balance(&ctx.svm, &trader_wallet.pubkey());
    //println!("The trader's wallet balance is {}", trader_wallet_bal);
}
