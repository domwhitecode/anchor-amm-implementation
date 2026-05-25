
use {
    amm_program::{CONFIG_SEED, LP_SEED},
    anchor_spl::associated_token,
    litesvm::LiteSVM,
    litesvm_token::{
        get_spl_account,
        spl_token::state::{Account as TokenAccount, Mint as MintAccount},
        CreateMint,
    },
    solana_keypair::Keypair,
    solana_message::{Instruction, Message, VersionedMessage},
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
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

fn token_balance(svm: &LiteSVM, ata: &Pubkey) -> u64 {
    get_spl_account::<TokenAccount>(svm, ata).unwrap().amount
}

fn mint_supply(svm: &LiteSVM, mint: &Pubkey) -> u64 {
    get_spl_account::<MintAccount>(svm, mint).unwrap().supply
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
    send(&mut svm, &[instruction], &payer, &[&payer]).unwrap();

    assert_eq!(token_balance(&svm, &vault_x), 0);
    assert_eq!(token_balance(&svm, &vault_y), 0);
    assert_eq!(mint_supply(&svm, &mint_lp), 0);
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
    send(&mut svm, &[init_ix, deposit_ix], &payer, &[&payer]).unwrap();

    let user = payer.pubkey();
    let user_x = associated_token::get_associated_token_address(&user, &mint_x);
    let user_y = associated_token::get_associated_token_address(&user, &mint_y);
    let user_lp = associated_token::get_associated_token_address(&user, &mint_lp);

    // empty-pool deposit uses max_x / max_y verbatim, mints `amount` LP
    assert_eq!(token_balance(&svm, &vault_x), 200_000_000);
    assert_eq!(token_balance(&svm, &vault_y), 200_000_000);
    assert_eq!(token_balance(&svm, &user_lp), 100_000_000);
    assert_eq!(mint_supply(&svm, &mint_lp), 100_000_000);
    assert_eq!(token_balance(&svm, &user_x), 1_000_000_000 - 200_000_000);
    assert_eq!(token_balance(&svm, &user_y), 1_000_000_000 - 200_000_000);
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
    send(&mut svm, &[init_ix, deposit_ix, withdraw_ix], &payer, &[&payer]).unwrap();

    let user = payer.pubkey();
    let user_x = associated_token::get_associated_token_address(&user, &mint_x);
    let user_y = associated_token::get_associated_token_address(&user, &mint_y);
    let user_lp = associated_token::get_associated_token_address(&user, &mint_lp);

    // burning 10M of 100M LP returns 1/10 of each vault (= 20M of each)
    assert_eq!(token_balance(&svm, &vault_x), 180_000_000);
    assert_eq!(token_balance(&svm, &vault_y), 180_000_000);
    assert_eq!(token_balance(&svm, &user_lp), 90_000_000);
    assert_eq!(mint_supply(&svm, &mint_lp), 90_000_000);
    assert_eq!(token_balance(&svm, &user_x), 1_000_000_000 - 180_000_000);
    assert_eq!(token_balance(&svm, &user_y), 1_000_000_000 - 180_000_000);
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
    send(&mut svm, &[init_ix, deposit_ix, swap_ix], &payer, &[&payer]).unwrap();

    let user = payer.pubkey();
    let user_x = associated_token::get_associated_token_address(&user, &mint_x);
    let user_y = associated_token::get_associated_token_address(&user, &mint_y);

    let final_vault_x = token_balance(&svm, &vault_x);
    let final_vault_y = token_balance(&svm, &vault_y);
    let final_user_x = token_balance(&svm, &user_x);
    let final_user_y = token_balance(&svm, &user_y);

    // swapped exactly 10M X in
    assert_eq!(final_vault_x, 200_000_000 + 10_000_000);
    assert_eq!(final_user_x, 1_000_000_000 - 200_000_000 - 10_000_000);

    // got at least `min` Y out and the vault drained that same amount
    let y_received = final_user_y - (1_000_000_000 - 200_000_000);
    assert!(y_received >= 5_000_000, "received {y_received} Y, expected >= min 5_000_000");
    assert_eq!(final_vault_y, 200_000_000 - y_received);

    // constant-product invariant: fees stay in the pool, so k must not decrease
    let initial_k: u128 = 200_000_000u128 * 200_000_000u128;
    let final_k: u128 = final_vault_x as u128 * final_vault_y as u128;
    assert!(final_k >= initial_k, "k decreased: {initial_k} -> {final_k}");
}