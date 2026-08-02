//! Client protocol account creation helpers.
use litesvm::LiteSVM;
use solana_transaction::Transaction;
use solana_keypair::Keypair;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_program::sysvar::rent;
use solana_instruction;
use spl_token::instruction as token_ix;
use spl_token::state::Mint;
use spl_token::state::Account as TokenAccount;
use solana_program::program_pack::Pack;
use solana_system_interface::instruction::create_account;

pub fn create_test_mint(
    svm: &mut LiteSVM, payer: &Keypair,
    mint_authority: &Pubkey, decimals: u8
) -> Pubkey {
    let mint = Keypair::new();

    let rent = rent::Rent::default();
    let mint_space = Mint::LEN;
    let lamports = rent.minimum_balance(mint_space);

    // Create mint account.
    let create_account_ix = create_account(
        &payer.pubkey(), &mint.pubkey(),
        lamports, mint_space as u64,
        &spl_token::ID
    );

    // Initialize mint
    let init_mint_ix = token_ix::initialize_mint(
        &spl_token::ID, &mint.pubkey(), mint_authority,
        None, decimals,
    ).unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[create_account_ix, init_mint_ix],
        Some(&payer.pubkey()),
        &[payer, &mint],
        svm.latest_blockhash(),
    );

    svm.send_transaction(tx).unwrap();

    mint.pubkey()
}

pub fn get_token_balance(svm: &LiteSVM, token_account_pubkey: &Pubkey) -> u64 {
    let acc_data = svm.get_account(token_account_pubkey)
        .expect("Account not found");
    let balance_bytes: [u8; 8] = acc_data.data[64..72].try_into()
        .expect("Failed to read amount bytes from account layout");

    u64::from_le_bytes(balance_bytes)
}

#[derive(Clone, Debug)]
pub struct ExtractedConfig {
    pub limit: u64,
    pub mint_x: Pubkey,
    pub x_decimal: u8,
    pub mint_y: Pubkey,
    pub y_decimal: u8,
    pub vault_x_ata: Pubkey,
    pub vault_y_ata: Pubkey,
}

pub fn get_config_data(svm: &LiteSVM, config_account_pubkey: &Pubkey) -> ExtractedConfig {
    let account = svm.get_account(config_account_pubkey)
        .expect("Account Not Found");
    let data = &account.data;

    let limit = u64::from_le_bytes(data[0..8].try_into().unwrap());
    let mint_x = Pubkey::new_from_array(data[8..40].try_into().unwrap());
    let x_decimal = data[40];
    let mint_y = Pubkey::new_from_array(data[41..73].try_into().unwrap());
    let y_decimal = data[73];
    let vault_x_ata = Pubkey::new_from_array(data[74..106].try_into().unwrap());
    let vault_y_ata = Pubkey::new_from_array(data[106..138].try_into().unwrap());

    ExtractedConfig {
        limit, mint_x, x_decimal,
        mint_y, y_decimal, vault_x_ata,
        vault_y_ata
    }
}
