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
