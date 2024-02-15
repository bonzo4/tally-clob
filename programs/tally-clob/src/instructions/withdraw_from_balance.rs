use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Mint, Token, TokenAccount, Transfer};

use crate::{errors::TallyClobErrors, User};

pub fn withdraw_from_balance(ctx: Context<WithdrawFromBalance>, amount: f64) -> Result<()> {
    let mint = &ctx.accounts.mint;
    let mint_key = mint.key().to_string();
    let usdc_key = "5DUWZLh3zPKAAJKu7ftMJJrkBrKnq3zHPPmguzVkhSes".to_string(); // not actually usdc address for development
    require!(mint_key == usdc_key, TallyClobErrors::NotUSDC);
    
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

    let fee_amount = amount * 0.05;
    let new_amount = amount - fee_amount;

    let cpi_program = token_program.to_account_info();

    let decimals:u64 = 10_u64.pow(mint.decimals as u32);

    transfer(
        CpiContext::new(cpi_program, cpi_accounts),
        (new_amount * decimals as f64) as u64 
    )?;

    Ok(())
}

#[derive(Accounts)]
pub struct WithdrawFromBalance<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub user: Account<'info, User>,
    #[account(mut )]
    pub from_usdc_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub to_usdc_account: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
}