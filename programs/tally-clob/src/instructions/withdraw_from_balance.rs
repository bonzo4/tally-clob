use anchor_lang::prelude::*;

use crate::{errors::TallyClobErrors, User};

pub fn withdraw_from_balance(ctx: Context<WithdrawFromBalance>, amount: f64) -> Result<()> {
    require!(amount > 0.0, TallyClobErrors::AmountToWithdrawTooLow);
    require!(amount <= ctx.accounts.user.balance, TallyClobErrors::AmountToWithdrawTooGreat);
    
    ctx.accounts.user.withdraw_from_balance(amount)?;

    msg!("Your new balance is: {}", ctx.accounts.user.balance);

    Ok(())
}

#[derive(Accounts)]
pub struct WithdrawFromBalance<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub user: Account<'info, User>,
}