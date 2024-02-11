use anchor_lang::prelude::*;

use crate::AuthorizedUser;


pub fn authorize_user(
    ctx: Context<AuthorizeUser>,
    authorized: bool
) -> Result<()> {

    ctx.accounts.authorized_user.authorized = authorized;

    Ok(())
}

#[derive(Accounts)]
#[instruction(user_key: Pubkey)]
pub struct AuthorizeUser<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        init_if_needed,
        payer = signer,
        space = AuthorizedUser::SIZE,
        seeds = [b"authorized_users".as_ref(), user_key.key().as_ref()],
        bump
    )]
    pub authorized_user: Account<'info, AuthorizedUser>,
    pub system_program: Program<'info, System>,
}