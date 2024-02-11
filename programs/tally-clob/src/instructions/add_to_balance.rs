use anchor_lang::prelude::*;

use crate::{errors::TallyClobErrors, User};

pub fn add_to_balance(ctx: Context<AddToBalance>, amount: f64) -> Result<()> {
    require!(amount > 0.0, TallyClobErrors::AmountToAddTooLow);

    ctx.accounts.user.add_to_balance(amount)?;

    msg!("Your new balance is: {}", ctx.accounts.user.balance);

    Ok(())
}

#[derive(Accounts)]
#[instruction(user_key: Pubkey)]
pub struct AddToBalance<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"users".as_ref(), user_key.key().as_ref()], 
        bump = user.bump
    )]
    pub user: Account<'info, User>,
    pub system_program: Program<'info, System>,
}