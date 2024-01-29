use anchor_lang::prelude::*;

use crate::User;

pub fn init_wallet(ctx: Context<InitWallet>) -> Result<()> {
    ctx.accounts.user_wallet.balance = 0.0;
    Ok(())
}

#[derive(Accounts)]
pub struct InitWallet<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        init,
        payer = signer,
        space = User::SIZE, seeds = [b"users".as_ref(), signer.key().as_ref()], bump
    )]
    pub user_wallet: Account<'info, User>,
    pub system_program: Program<'info, System>,
}