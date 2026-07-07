pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("EWHqZk9odbYcFb9rxHsTSBKxZbjJp8YYeZmmyWPKeE1m");

#[program]
pub mod bonding_curve_launchpad {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        base_price: u64,
        slope: u64,
        scale: u64,
        supply_cap: u64,
    ) -> Result<()> {
        crate::instructions::initialize::handle_initialize(
            ctx, base_price, slope, scale, supply_cap,
        )
    }

    pub fn buy_tokens(ctx: Context<BuyTokens>, num_tokens: u64, max_cost: u64) -> Result<()> {
        crate::instructions::buy_tokens::handle_buy_tokens(ctx, num_tokens, max_cost)
    }

    pub fn buy_with_budget(
        ctx: Context<BuyWithBudget>,
        budget: u64,
        min_tokens: u64,
    ) -> Result<()> {
        crate::instructions::buy_with_budget::handle_buy_with_budget(ctx, budget, min_tokens)
    }

    pub fn sell(ctx: Context<Sell>, num_tokens: u64, min_refund: u64) -> Result<()> {
        crate::instructions::sell::handle_sell(ctx, num_tokens, min_refund)
    }
}
