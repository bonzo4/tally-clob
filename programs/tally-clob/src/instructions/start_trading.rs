use anchor_lang::prelude::*;

use crate::{errors::TallyClobErrors, utils::current_timestamp, AuthorizedUser, Market};

pub fn start_trading(
    ctx: Context<StartTrading>,
    sub_market_id: u64,
) -> Result<()> {

    let now = current_timestamp();

    ctx.accounts.market.get_sub_market(&sub_market_id)?.fair_launch_end = now;
    ctx.accounts.market.get_sub_market(&sub_market_id)?.trading_start = now;

    Ok(())
}

#[derive(Accounts)]
pub struct StartTrading<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub authorized_user: Account<'info, AuthorizedUser>,
    #[account(mut)]
    pub market: Account<'info, Market>,
    pub system_program: Program<'info, System>,
}