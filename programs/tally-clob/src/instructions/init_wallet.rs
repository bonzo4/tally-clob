use anchor_lang::prelude::*;

use crate::User;

pub fn init_wallet(ctx: Context<InitWallet>) -> Result<Pubkey> {
    ctx.accounts.user.balance = 0.0;
    Ok(ctx.accounts.user.key())
}

#[derive(Accounts)]
pub struct InitWallet<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        init_if_needed,
        payer = signer,
        space = User::SIZE, seeds = [b"users".as_ref(), signer.key().as_ref()], bump
    )]
    pub user: Account<'info, User>,
    pub system_program: Program<'info, System>,
}