use anchor_lang::prelude::*;
use anchor_spl:: {
    associated_token::AssociatedToken,
    token::{burn, transfer, Burn, Mint, Token, TokenAccount, Transfer },
};
use constant_product_curve::{ConstantProduct, CurveError, XYAmounts};

use crate::{CONFIG_SEED, LP_SEED, error::AmmError, state::Config};

#[derive(Accounts)]
pub struct Withdraw<'info> {
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


    // mutable because we will be burning lp tokens 
    #[account(
        mut,
        seeds = [LP_SEED, config.key().as_ref()],
        bump = config.lp_bump
    )]
    pub mint_lp: Account<'info, Mint>,


    // the vaults are mutable since we are withdrawing tokens out of both
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
    

    // the user_* are where the withdraws are going
    // both account authority will be the user
    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = user,
    )]
    pub user_x: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = user,
    )]
    pub user_y: Account<'info, TokenAccount>,


    // lp token account we'll withdraw from
    #[account(
        mut,
        associated_token::mint = mint_lp,
        associated_token::authority = user,
    )]
    pub user_lp: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>
}

impl<'info> Withdraw<'info> {

    pub fn deposit(
        &mut self,
        amount: u64,
        min_x: u64,
        min_y: u64
    ) -> Result<()> {
        require_neq!(amount, 0, AmmError::InvalidAmount);


        // otherwise we'll want to use this function to calculate the amounts
        let amounts: XYAmounts = ConstantProduct::xy_withdraw_amounts_from_l(
            self.vault_x.amount, 
            self.vault_y.amount, 
            self.mint_lp.supply, 
            amount,
            6,
        ).map_err(|_| AmmError::XYCalculationFailed)?;
        
        require!(
            amounts.x >= min_x && amounts.y >= min_y,
            AmmError::SlippageExceeded
        );    
        
        // burn lp tokens first then withdraw
        self.burn_lp_tokens(amount)?;
        // withdraw x tokens
        self.withdraw_tokens(true, amounts.x)?;
        // withdraw y tokens
        self.withdraw_tokens(false, amounts.y)

    }

    // Helper functions 
    pub fn withdraw_tokens(&self, is_x: bool, amount: u64) -> Result<()> {
        let (from , to) = match is_x {
            true => (
                self.vault_x.to_account_info(),
                self.user_x.to_account_info(),
            ),
            false => (
                self.vault_y.to_account_info(),
                self.user_y.to_account_info(),
            )
        };

        let cpi_program: Pubkey = self.token_program.key();
        let cpi_accounts: Transfer<'_> = Transfer {
            from,
            to,
            authority: self.user.to_account_info(),
        };
        // from is vault whos auth is config which is a pda 
        let signer_seeds: &[&[&[u8]]] = &[&[
            CONFIG_SEED,
            &self.config.seed.to_le_bytes(),
            &[self.config.config_bump],
        ]];
        let ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        transfer(ctx, amount)
    }

    pub fn burn_lp_tokens(&self, amount: u64) -> Result<()> {
        let cpi_program: Pubkey = self.token_program.key();

        let cpi_accounts: Burn<'_> = Burn {
            mint: self.mint_lp.to_account_info(),
            from: self.user_lp.to_account_info(),
            authority: self.user.to_account_info(),
        };
        // no need for the signer seeds in withdraw since from is user_lp whose
        // auth is the user, therefore user will sign the tx 
        let ctx = 
            CpiContext::new(cpi_program, cpi_accounts,);

        burn(ctx, amount)
    }
}