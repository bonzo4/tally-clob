use anchor_lang::prelude::*;

use crate::{state:: Market, AuthorizedUser, InitSubMarket, SubMarket};

pub fn init_market(
    ctx: Context<InitMarket>,
    init_sub_markets: Vec<InitSubMarket>
) -> Result<()> {
    let sub_markets = init_sub_markets.iter()
        .map(|init_sub_market| SubMarket::new(init_sub_market))
        .collect::<Vec<SubMarket>>();
    ctx.accounts.market.sub_markets = sub_markets;

    Ok(())
}

#[derive(Accounts)]
#[instruction(init_sub_markets: Vec<InitSubMarket>, market_key: Pubkey)]
pub struct InitMarket<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub authorized_user: Account<'info, AuthorizedUser>,
    #[account(
        init,
        payer = signer,
        space = Market::SIZE, 
        seeds = [b"markets".as_ref(), market_key.key().as_ref()], 
        bump
    )]
    pub market: Account<'info, Market>,
    pub system_program: Program<'info, System>,
}