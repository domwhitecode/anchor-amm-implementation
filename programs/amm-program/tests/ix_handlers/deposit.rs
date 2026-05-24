use anchor_lang::solana_program::system_program::ID as SYSTEM_PROGRAM_ID;
use anchor_lang::{InstructionData, ToAccountMetas};
use anchor_spl::associated_token::{self, ID as ASSOCIATED_TOKEN_PROGRAM_ID};
use anchor_spl::token::ID as TOKEN_PROGRAM_ID;
use litesvm::LiteSVM;
use litesvm_token::{CreateAssociatedTokenAccount, MintTo};
use solana_keypair::Keypair;
use solana_message::Instruction;
use solana_pubkey::Pubkey;
use solana_signer::Signer;

pub fn create_deposit_ix(
    svm: &mut LiteSVM,
    payer: &Keypair,
    mint_x: Pubkey,
    mint_y: Pubkey,
    config: Pubkey,
    mint_lp: Pubkey,
    vault_x: Pubkey,
    vault_y: Pubkey,
) -> Instruction {
    let user = payer.pubkey();

    let user_x = CreateAssociatedTokenAccount::new(svm, &payer, &mint_x)
        .owner(&user)
        .send()
        .unwrap();
    MintTo::new(svm, &payer, &mint_x, &user_x, 1_000_000_000)
        .send()
        .unwrap();

    let user_y = CreateAssociatedTokenAccount::new(svm, &payer, &mint_y)
        .owner(&user)
        .send()
        .unwrap();
    MintTo::new(svm, &payer, &mint_y, &user_y, 1_000_000_000)
        .send()
        .unwrap();

    let user_lp = associated_token::get_associated_token_address(&user, &mint_lp);

    Instruction::new_with_bytes(
        amm_program::id(),
        &amm_program::instruction::Deposit {
            amount: 100_000_000,
            max_x: 200_000_000,
            max_y: 200_000_000,
        }
        .data(),
        amm_program::accounts::Deposit {
            user,
            mint_x,
            mint_y,
            config,
            mint_lp,
            vault_x,
            vault_y,
            user_x,
            user_y,
            user_lp,
            token_program: TOKEN_PROGRAM_ID,
            associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
    )
}
