use anchor_lang::prelude::*;

use crate::{errors::TallyClobErrors, AuthorizedUser, Market};

pub fn resolve_market(
    ctx: Context<ResolveMarket>,
    sub_market_id: u64,
    choice_id: u64
) -> Result<()> {

    ctx.accounts.market.get_sub_market(&sub_market_id)?.resolved;

    require!(!ctx.accounts.market.get_sub_market(&sub_market_id)?.resolved, TallyClobErrors::MarketAlreadyResolved);

    ctx.accounts.market.get_sub_market(&sub_market_id)?.resolved = true;

    ctx.accounts.market.get_sub_market(&sub_market_id)?.get_choice(&choice_id)?.winning_choice = true;

    Ok(())
}

#[derive(Accounts)]
pub struct ResolveMarket<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub authorized_user: Account<'info, AuthorizedUser>,
    #[account(mut)]
    pub market: Account<'info, Market>,
    pub system_program: Program<'info, System>,
}