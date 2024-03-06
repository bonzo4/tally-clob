use anchor_lang::prelude::*;

use crate::{errors::TallyClobErrors, User};

pub fn add_to_unreedeemable(ctx: Context<AddToUnreedeemable>, amount: u128) -> Result<()> {
    require!(amount > 0, TallyClobErrors::AmountToAddTooLow);

    ctx.accounts.user.add_to_unreedeemable(amount)?;

    msg!("Your new balance is: {}", ctx.accounts.user.balance);

    Ok(())
}

#[derive(Accounts)]
pub struct AddToUnreedeemable<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub user: Account<'info, User>,
    pub system_program: Program<'info, System>,
}