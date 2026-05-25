
use {
    amm_program::{CONFIG_SEED, LP_SEED}, anchor_spl::{associated_token, token::accessor::mint}, litesvm::LiteSVM, litesvm_token::CreateMint, solana_keypair::Keypair, solana_message::{Instruction, Message, VersionedMessage}, solana_pubkey::Pubkey, solana_signer::Signer, solana_transaction::versioned::VersionedTransaction
};

mod ix_handlers;
use ix_handlers::*;

fn send(
    svm: &mut LiteSVM,
    ixs: &[Instruction],
    payer: &Keypair,
    signers: &[&Keypair],
) -> litesvm::types::TransactionResult {
    svm.expire_blockhash();
    let blockhash = svm.latest_blockhash();
    let msg = 
        Message::new_with_blockhash(ixs, Some(&payer.pubkey()), &blockhash);
    let tx = 
        VersionedTransaction::try_new(VersionedMessage::Legacy(msg), signers).unwrap();
    svm.send_transaction(tx)
}

fn setup() -> (
    LiteSVM,
    Keypair,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
) {
    let program_id = amm_program::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/amm_program.so");
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();

    // lets create two mints to represent xy tokens
    let mint_x = CreateMint::new(&mut svm, &payer)
        .decimals(6)
        .authority(&payer.pubkey())
        .send()
        .unwrap();
    let mint_y = CreateMint::new(&mut svm, &payer)
        .decimals(6)
        .authority(&payer.pubkey())
        .send()
        .unwrap();
    
    // derive config and mint lp address 
    let config = Pubkey::find_program_address(
        &[CONFIG_SEED, &123u64.to_le_bytes()],
        &amm_program::id()
    ).0;
    let mint_lp = Pubkey::find_program_address(
        &[LP_SEED, config.as_ref()],
        &amm_program::id()
    ).0;

    let vault_x = associated_token::get_associated_token_address(&config, &mint_x);
    let vault_y = associated_token::get_associated_token_address(&config, &mint_y);

    (
        svm,
        payer,
        mint_x,
        mint_y,
        config,
        mint_lp,
        vault_x,
        vault_y
    )

}

#[test]
fn test_initialize() {
    let (mut svm, payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y) = setup();

    let instruction = create_initialise_ix(
        &mut svm, &payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y
    );
    let res = send(&mut svm, &[instruction], &payer, &[&payer]);
    assert!(res.is_ok());
}

#[test]
fn test_deposit() {
    let (mut svm, payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y) = setup();

    let init_ix = create_initialise_ix(
        &mut svm, &payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y
    );
    let deposit_ix = create_deposit_ix(
        &mut svm, &payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y
    );

    let res = send(&mut svm, &[init_ix, deposit_ix], &payer, &[&payer]);
    assert!(res.is_ok());
}


#[test]
fn test_withdraw() {
    let (mut svm, payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y) = setup();

    let init_ix = create_initialise_ix(
        &mut svm, &payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y
    );
    let deposit_ix = create_deposit_ix(
        &mut svm, &payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y
    );
    let withdraw_ix = create_withdraw_ix(
        &payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y
    );

    let res = send(&mut svm, &[init_ix, deposit_ix, withdraw_ix], &payer, &[&payer]);
    assert!(res.is_ok());
}


#[test]
fn test_swap() {
    let (mut svm, payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y) = setup();

    let init_ix = create_initialise_ix(
        &mut svm, &payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y
    );
    let deposit_ix = create_deposit_ix(
        &mut svm, &payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y
    );
    let swap_ix = create_swap_ix(
        &payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y
    );

    let res = send(&mut svm, &[init_ix, deposit_ix, swap_ix], &payer, &[&payer]);
    assert!(res.is_ok());
}