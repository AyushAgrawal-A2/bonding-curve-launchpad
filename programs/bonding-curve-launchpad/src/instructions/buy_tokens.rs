use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount},
};

use crate::{error::BondingCurveLaunchpadError, BondingCurve, BONDING_CURVE_SEED};

#[derive(Accounts)]
pub struct BuyTokens<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(
        mut,
        mint::authority = bonding_curve,
    )]
    pub launchpad_mint: Account<'info, Mint>,

    pub reserve_mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = reserve_mint,
        associated_token::authority = bonding_curve,
    )]
    pub reserve_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [BONDING_CURVE_SEED, launchpad_mint.key().as_ref(), reserve_mint.key().as_ref()],
        bump = bonding_curve.bump
    )]
    pub bonding_curve: Account<'info, BondingCurve>,

    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint = launchpad_mint,
        associated_token::authority = buyer,
    )]
    pub buyer_launchpad_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = reserve_mint,
        associated_token::authority = buyer,
    )]
    pub buyer_reserve_ata: Account<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handle_buy_tokens(ctx: Context<BuyTokens>, num_tokens: u64, max_cost: u64) -> Result<()> {
    let cost = ctx.accounts.bonding_curve.cost_to_buy_tokens(num_tokens)?;
    require!(
        cost <= max_cost,
        BondingCurveLaunchpadError::SlippageExceeded
    );
    token::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.key(),
            token::TransferChecked {
                from: ctx.accounts.buyer_reserve_ata.to_account_info(),
                mint: ctx.accounts.reserve_mint.to_account_info(),
                to: ctx.accounts.reserve_vault.to_account_info(),
                authority: ctx.accounts.buyer.to_account_info(),
            },
        ),
        cost,
        ctx.accounts.reserve_mint.decimals,
    )?;
    let launchpad_mint_address = ctx.accounts.launchpad_mint.key();
    let reserve_mint_address = ctx.accounts.reserve_mint.key();
    let seeds = [
        BONDING_CURVE_SEED,
        launchpad_mint_address.as_ref(),
        reserve_mint_address.as_ref(),
        &[ctx.accounts.bonding_curve.bump],
    ];
    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.key(),
            token::MintTo {
                mint: ctx.accounts.launchpad_mint.to_account_info(),
                to: ctx.accounts.buyer_launchpad_ata.to_account_info(),
                authority: ctx.accounts.bonding_curve.to_account_info(),
            },
            &[&seeds],
        ),
        num_tokens,
    )?;
    ctx.accounts.bonding_curve.apply_buy(num_tokens, cost)?;
    Ok(())
}
