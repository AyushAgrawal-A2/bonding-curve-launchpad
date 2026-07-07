use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};

use crate::{error::BondingCurveLaunchpadError, BondingCurve, BONDING_CURVE_SEED};

#[derive(Accounts)]
pub struct Sell<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,

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
        mut,
        associated_token::mint = launchpad_mint,
        associated_token::authority = seller,
    )]
    pub seller_launchpad_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = reserve_mint,
        associated_token::authority = seller,
    )]
    pub seller_reserve_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handle_sell(ctx: Context<Sell>, num_tokens: u64, min_refund: u64) -> Result<()> {
    let refund = ctx.accounts.bonding_curve.refund_for_tokens(num_tokens)?;
    require!(
        refund >= min_refund,
        BondingCurveLaunchpadError::SlippageExceeded
    );
    token::burn(
        CpiContext::new(
            ctx.accounts.token_program.key(),
            token::Burn {
                mint: ctx.accounts.launchpad_mint.to_account_info(),
                from: ctx.accounts.seller_launchpad_ata.to_account_info(),
                authority: ctx.accounts.seller.to_account_info(),
            },
        ),
        num_tokens,
    )?;
    let launchpad_mint_address = ctx.accounts.launchpad_mint.key();
    let reserve_mint_address = ctx.accounts.reserve_mint.key();
    let seeds = [
        BONDING_CURVE_SEED,
        launchpad_mint_address.as_ref(),
        reserve_mint_address.as_ref(),
        &[ctx.accounts.bonding_curve.bump],
    ];
    token::transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.key(),
            token::TransferChecked {
                from: ctx.accounts.reserve_vault.to_account_info(),
                mint: ctx.accounts.reserve_mint.to_account_info(),
                to: ctx.accounts.seller_reserve_ata.to_account_info(),
                authority: ctx.accounts.bonding_curve.to_account_info(),
            },
            &[&seeds],
        ),
        refund,
        ctx.accounts.reserve_mint.decimals,
    )?;
    ctx.accounts.bonding_curve.apply_sell(num_tokens, refund)?;
    Ok(())
}
