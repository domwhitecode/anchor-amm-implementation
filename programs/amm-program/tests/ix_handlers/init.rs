use anchor_lang::solana_program::system_program::ID as SYSTEM_PROGRAM_ID;
use anchor_lang::{InstructionData, ToAccountMetas};
use anchor_spl::associated_token::ID as ASSOCIATED_TOKEN_PROGRAM_ID;
use anchor_spl::token::ID as TOKEN_PROGRAM_ID;
use litesvm::LiteSVM;
use solana_keypair::Keypair;
use solana_message::Instruction;
use solana_pubkey::Pubkey;
use solana_signer::Signer;

pub fn create_initialise_ix(
    mut _svm: &mut LiteSVM,
    payer: &Keypair,
    mint_x: Pubkey,
    mint_y: Pubkey,
    config: Pubkey,
    mint_lp: Pubkey,
    vault_x: Pubkey,
    vault_y: Pubkey,
) -> Instruction {
    let maker = payer.pubkey();

    let (_derived_config, config_bump) = 
        Pubkey::find_program_address(
            &[amm_program::CONFIG_SEED, &123u64.to_le_bytes()],
            &amm_program::id(),
        );
    let (_derived_mint_lp, mint_lp_bump) =
        Pubkey::find_program_address(
            &[amm_program::LP_SEED, config.as_ref()], 
            &amm_program::id()
        );

    Instruction::new_with_bytes(
        amm_program::id(),
        &amm_program::instruction::Initialize {
            seed: 123,
            fee: 30,
            authority: Some(maker),
            config_bump,
            mint_lp_bump,
        }
        .data(),
        amm_program::accounts::Initialize {
            initializer: maker,
            mint_x,
            mint_y,
            mint_lp,
            vault_x,
            vault_y,
            config,
            token_program: TOKEN_PROGRAM_ID,
            associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
    )
}
