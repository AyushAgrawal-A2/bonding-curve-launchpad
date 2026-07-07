use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use crate::{error::BondingCurveLaunchpadError, BondingCurve, BONDING_CURVE_SEED, TOKEN_DECIMALS};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        mint::decimals = TOKEN_DECIMALS,
        mint::authority = bonding_curve,
    )]
    pub launchpad_mint: Account<'info, Mint>,

    pub reserve_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = payer,
        associated_token::mint = reserve_mint,
        associated_token::authority = bonding_curve,
    )]
    pub reserve_vault: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = payer,
        space = 8 + BondingCurve::INIT_SPACE,
        seeds = [BONDING_CURVE_SEED, launchpad_mint.key().as_ref(), reserve_mint.key().as_ref()],
        bump
    )]
    pub bonding_curve: Account<'info, BondingCurve>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handle_initialize(
    ctx: Context<Initialize>,
    base_price: u64,
    slope: u64,
    scale: u64,
    supply_cap: u64,
) -> Result<()> {
    require!(
        base_price > 0 && slope > 0 && scale > 0 && supply_cap > 0,
        BondingCurveLaunchpadError::InvalidArguments
    );

    ctx.accounts.bonding_curve.set_inner(BondingCurve {
        base_price,
        slope,
        scale,
        supply_cap,
        supply: 0,
        reserve_amount: 0,
        bump: ctx.bumps.bonding_curve,
    });

    require!(
        ctx.accounts
            .bonding_curve
            .cost_between_supplies(0, supply_cap)
            .is_ok(),
        BondingCurveLaunchpadError::Overflow
    );

    Ok(())
}
