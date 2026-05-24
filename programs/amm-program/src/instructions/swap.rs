use anchor_lang::prelude::*;
use anchor_spl:: {
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount, Transfer, revoke, transfer },
};
use constant_product_curve::{ConstantProduct, CurveError, LiquidityPair, SwapResult};

use crate::{CONFIG_SEED, LP_SEED, error::AmmError, state::Config };

#[derive(Accounts)]
pub struct Swap<'info> {
    pub user: Signer<'info>,
    pub mint_x: Account<'info, Mint>,
    pub mint_y: Account<'info, Mint>,


    #[account(
        has_one = mint_x,
        has_one = mint_y,
        seeds = [CONFIG_SEED, config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump,
    )]
    pub config: Account<'info, Config>,

    // mint lp will be used to get the current state of the curve
    // need to calculate what the withdraw amount will be on the deposit
    #[account(
        seeds = [LP_SEED, config.key().as_ref()],
        bump = config.lp_bump
    )]
    pub mint_lp: Account<'info, Mint>,

    // the vaults are mutable since we are swapping in and out 
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

    // depost/withdraws will come from these accounts
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

    // program accounts
    pub token_program: Program<'info, Token>,
    pub system_account: Program<'info, System>,
    pub associated_token_account: Program<'info, AssociatedToken>
}

impl<'info> Swap<'info> {
    pub fn swap(
        &mut self,
        is_x: bool,
        amount: u64,
        min: u64
    ) -> Result<()> {

        // validate
        require!(amount > 0, AmmError::InvalidAmount);

        // initialize
        let mut curve: ConstantProduct = ConstantProduct::init(
            self.vault_x.amount,
            self.vault_y.amount,
            self.mint_lp.supply, 
            self.config.fee,
            Some(6),
        ).map_err(|_| AmmError::CPMMInitFail)?;

        let p: LiquidityPair = match is_x {
            true => LiquidityPair::X,
            false => LiquidityPair::Y,
        };

        let swap_result: constant_product_curve::SwapResult = curve
            .swap(p, amount, min)
            .map_err(|_| AmmError::SlippageExceeded)?;
        
        self.deposit_tokens(is_x, swap_result.deposit)?;
        self.withdraw_tokens(is_x, swap_result.withdraw)

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


    pub fn withdraw_tokens(&self, is_x: bool, amount: u64) -> Result<()> {
        // if is x is true we actually want to be withdrawing y 
        // since this is a swap 
        let (from , to) = match is_x {
            true => (
                self.vault_y.to_account_info(),
                self.user_y.to_account_info(),
            ),
            false => (
                self.vault_x.to_account_info(),
                self.user_x.to_account_info(),
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
}