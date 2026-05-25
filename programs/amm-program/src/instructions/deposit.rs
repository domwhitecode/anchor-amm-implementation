use anchor_lang::prelude::*;
use anchor_spl:: {
    associated_token::AssociatedToken,
    token::{Mint, MintTo, Token, TokenAccount, Transfer, mint_to, transfer},
};
use constant_product_curve::{ConstantProduct, XYAmounts};

use crate::{CONFIG_SEED, LP_SEED, error::AmmError, state::Config};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,  // will be the liquidity provider
    pub mint_x: Account<'info, Mint>,
    pub mint_y: Account<'info, Mint>,

    // no need to init since already initialized earlier 
    // just use the same keys
    #[account(
        has_one = mint_x,
        has_one = mint_y,
        seeds = [CONFIG_SEED, config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump,
    )]
    pub config: Account<'info, Config>,


    // mutable because we will be minting lp tokens 
    #[account(
        mut,
        seeds = [LP_SEED, config.key().as_ref()],
        bump = config.lp_bump
    )]
    pub mint_lp: Account<'info, Mint>,


    // the vaults are mutable since we are depositing tokens in both as
    // liquidity providers
    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = config,
    )]
    pub vault_x: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = config,
    )]
    pub vault_y: Box<Account<'info, TokenAccount>>,
    

    // the users are where the deposit is coming from
    // both account authority will be the users 
    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = user,
    )]
    pub user_x: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = user,
    )]
    pub user_y: Box<Account<'info, TokenAccount>>,


    // need to have an associated token account 
    // for the user to store lp tokens
    // use init_if_needed because not sure if its users first time
    // depositing 
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint_lp,
        associated_token::authority = user,
    )]
    pub user_lp: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>
}

impl<'info> Deposit<'info> {

    pub fn deposit(
        &mut self,
        amount: u64,
        max_x: u64,
        max_y: u64
    ) -> Result<()> {

        // Not really necessary because if the config is locked the tx should fail
        // anyway 
        require!(!self.config.locked, AmmError::PoolLocked);
        require_neq!(amount, 0, AmmError::InvalidAmount);

        let (x, y) = 
            // if all of these are true then we are the ones setting 
            // up the initial state of the curve
            if self.mint_lp.supply == 0 && self.vault_x.amount == 0 && self.vault_y.amount == 0 {
                (max_x, max_y)
            } else {
                // otherwise we'll want to use this function to calculate the amounts
                let amounts: XYAmounts = ConstantProduct::xy_deposit_amounts_from_l(
                    self.vault_x.amount, 
                    self.vault_y.amount, 
                    self.mint_lp.supply, 
                    amount,
                    6,
                ).map_err(|_| AmmError::XYCalculationFailed)?;
                
                require!(
                    amounts.x <= max_x && amounts.y <= max_y,
                    AmmError::SlippageExceeded
                );
                
                (amounts.x, amounts.y)
            };
            
        // deposit x tokens
        self.deposit_tokens(true, x)?;
        // deposit y tokens
        self.deposit_tokens(false, y)?;
        // mint lp tokens
        self.mint_lp_tokens(amount)

    }

    // Helper functions 
    pub fn deposit_tokens(&self, is_x: bool, amount: u64) -> Result<()> {
        let (from , to) = match is_x {
            true => (
                self.user_x.to_account_info(),
                self.vault_x.to_account_info(),
            ),
            false => (
                self.user_y.to_account_info(),
                self.vault_y.to_account_info(),
            )
        };

        let cpi_program: Pubkey = self.token_program.key();

        let cpi_accounts: Transfer<'_> = Transfer {
            from,
            to,
            authority: self.user.to_account_info(),
        };

        let ctx = CpiContext::new(cpi_program, cpi_accounts);

        transfer(ctx, amount)
    }

    pub fn mint_lp_tokens(&self, amount: u64) -> Result<()> {
        let cpi_program: Pubkey = self.token_program.key();

        let cpi_accounts: MintTo<'_> = MintTo {
            mint: self.mint_lp.to_account_info(),
            to: self.user_lp.to_account_info(),
            authority: self.config.to_account_info(),
        };

        // we need to use signer seeds here because the config
        // is the authority, is a pda, and needs to sign the transaction
        let signer_seeds: &[&[&[u8]]] = &[&[
            CONFIG_SEED,
            &self.config.seed.to_le_bytes(),
            &[self.config.config_bump],
        ]];

        let ctx = 
            CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        mint_to(ctx, amount)
    }
}