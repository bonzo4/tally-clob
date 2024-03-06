use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Mint, Token, TokenAccount, Transfer};

use crate::{errors::TallyClobErrors, User};

pub fn withdraw_from_balance(ctx: Context<WithdrawFromBalance>, amount: u128) -> Result<()> {
    let mint = &ctx.accounts.mint;
    let mint_key = mint.key().to_string();
    let usdc_key = "5DUWZLh3zPKAAJKu7ftMJJrkBrKnq3zHPPmguzVkhSes".to_string(); // not actually usdc address for development
    require!(mint_key == usdc_key, TallyClobErrors::NotUSDC);
    
    require!(amount > 0, TallyClobErrors::AmountToWithdrawTooLow);
    require!(amount <= ctx.accounts.user.balance, TallyClobErrors::AmountToWithdrawTooGreat);
    
    ctx.accounts.user.withdraw_from_balance(amount)?;

    let destination = &ctx.accounts.to_usdc_account;
    let source = &ctx.accounts.from_usdc_account;
    let fee_account = &ctx.accounts.fee_usdc_account;
    let authority = &ctx.accounts.signer;

    require!(source.owner.to_string() == "7rTBUSkc8PHPW3VwGiPB4EbwHWxoSvVpMmbnAqRiGwWx", TallyClobErrors::NotAuthorized);
    require!(fee_account.owner.to_string() == "eQv1C2XUfsn1ynM65NghBikNsH4TDnTQn5aSZYZdH79",TallyClobErrors::NotAuthorized);
    require!(source.owner.to_string() == authority.key().to_string(), TallyClobErrors::NotAuthorized);

    let token_program = &ctx.accounts.token_program;
    let cpi_program = &token_program.to_account_info();

    let fee_amount = amount / 20;
    let new_amount = (amount as i64 - fee_amount as i64) as u64;

    // transfer fees
    let fee_cpi_accounts = Transfer {
        from: source.to_account_info().clone(),
        to: fee_account.to_account_info().clone(),
        authority: authority.to_account_info().clone()
    };

    transfer (
        CpiContext::new(cpi_program.clone(), fee_cpi_accounts),
        fee_amount as u64
    )?;

    // transfer amount
    let cpi_accounts = Transfer {
        from: source.to_account_info().clone(),
        to: destination.to_account_info().clone(),
        authority: authority.to_account_info().clone()
    };

    transfer (
        CpiContext::new(cpi_program.clone(), cpi_accounts),
        new_amount
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
    #[account(mut)]
    pub fee_usdc_account: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
}