use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Mint, Token, TokenAccount, Transfer};

use crate::{errors::TallyClobErrors, User};

pub fn withdraw_from_balance(ctx: Context<WithdrawFromBalance>, amount: f64) -> Result<()> {
    let mint = &ctx.accounts.mint;
    let mint_key = mint.key().to_string();
    let usdc_key = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string();
    require!(mint_key == usdc_key, TallyClobErrors::NotUSDC);

    let fee_amount = amount * 0.05;
    let new_amount = amount - fee_amount;
    
    require!(amount > 0.0, TallyClobErrors::AmountToWithdrawTooLow);
    require!(amount <= ctx.accounts.user.balance, TallyClobErrors::AmountToWithdrawTooGreat);
    
    ctx.accounts.user.withdraw_from_balance(amount)?;

    let destination = &ctx.accounts.to_usdc_account;
    let source = &ctx.accounts.from_usdc_account;
    let authority = &ctx.accounts.signer;

    require!(source.owner.to_string() == authority.key().to_string(), TallyClobErrors::NotAuthorized);

    let token_program = &ctx.accounts.token_program;

    let cpi_accounts = Transfer {
        from: source.to_account_info().clone(),
        to: destination.to_account_info().clone(),
        authority: authority.to_account_info().clone()
    };

    let cpi_program = token_program.to_account_info();

    transfer(
        CpiContext::new(cpi_program, cpi_accounts),
        (new_amount * 6.0) as u64 
    )?;


    Ok(())
}

#[derive(Accounts)]
#[instruction(user_key: Pubkey)]
pub struct WithdrawFromBalance<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"users".as_ref(), user_key.key().as_ref()], 
        bump = user.bump
    )]
    pub user: Account<'info, User>,
    #[account(mut)]
    pub from_usdc_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub to_usdc_account: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
}