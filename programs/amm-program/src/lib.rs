pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("8Wrtbhw9tKyTW7Z1oMyQrhivEoLZCPRxVim1ey9wvTiq");

// #[program]
// pub mod amm_program {
//     use super::*;

//     pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
//         initialize::handler(ctx)
//     }
// }
