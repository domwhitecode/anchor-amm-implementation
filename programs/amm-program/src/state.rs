use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Config {
    pub seed: u64, // Will be used to create different pools/configs
    pub authority: Option<Pubkey>, // If we want an authority 
    pub mint_x: Pubkey, // Token X
    pub mint_y: Pubkey, // Token Y
    pub fee: u16,  // 0-10,000 BPS
    pub locked: bool,  // is pool locked or not?
    pub config_bump: u8,  // bump seed for the config account 
    pub lp_bump: u8,  // bump seed for the lp token 
}