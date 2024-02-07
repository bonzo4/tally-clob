use anchor_lang::prelude::*;

use crate::{state:: Market, SubMarket};

pub fn init_market(
    ctx: Context<InitMarket>,
    sub_markets: Vec<SubMarket>
) -> Result<()> {
    ctx.accounts.market.sub_markets = sub_markets;

    Ok(())
}

#[derive(Accounts)]
pub struct InitMarket<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        init_if_needed,
        payer = signer,
        space = Market::SIZE, 
        seeds = [b"markets".as_ref(), market.key().as_ref()], 
        bump
    )]
    pub market: Account<'info, Market>,
    pub system_program: Program<'info, System>,
}