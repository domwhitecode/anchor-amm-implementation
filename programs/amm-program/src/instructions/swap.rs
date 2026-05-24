use anchor_lang::prelude::*;
use anchor_spl:: {
    associated_token::AssociatedToken,
    token::{transfer, Token, TokenAccount, Transfer },
};
use constant_product_curve::{ConstantProduct, LiquidityPair};

use crate::{CONFIG_SEED, LP_SEED, error::AmmError, state::Config};