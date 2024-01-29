use anchor_lang::prelude::*;

use crate::{errors::TallyClobErrors, User};

pub fn add_to_balance(ctx: Context<AddToBalance>, amount: f64) -> Result<()> {
    require!(amount > 0.0, TallyClobErrors::AmountToAddTooLow);

    ctx.accounts.user.add_to_balance(amount)?;

    msg!("Your new balance is: {}", ctx.accounts.user.balance);

    Ok(())
}

#[derive(Accounts)]
pub struct AddToBalance<'info> {
    pub signer: Signer<'info>,
    #[account(mut)]
    pub user: Account<'info, User>,
}