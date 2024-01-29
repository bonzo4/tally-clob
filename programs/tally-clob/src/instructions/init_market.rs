use anchor_lang::prelude::*;

use crate::state::{ChoiceMarket, Market, MarketStatus};

pub fn init_market(
    ctx: Context<InitMarket>,
    title: String,
    choice_count: u8,
    choices: Vec<ChoiceMarket>,
    fair_launch_start: i64,
    fair_launch_end: i64,
    trading_start: i64,
    trading_end: i64,
) -> Result<()> {

    ctx.accounts.market.title = title;
    ctx.accounts.market.total_pot = 0.0;
    ctx.accounts.market.choice_count = choice_count;
    ctx.accounts.market.choices = choices;
    ctx.accounts.market.market_status = MarketStatus::Intializing;
    ctx.accounts.market.fair_launch_start = fair_launch_start;
    ctx.accounts.market.fair_launch_end = fair_launch_end;
    ctx.accounts.market.trading_start = trading_start;
    ctx.accounts.market.trading_end = trading_end;

    Ok(())
}

#[derive(Accounts)]
pub struct InitMarket<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        init,
        payer = signer,
        space = Market::SIZE, 
        seeds = [b"markets".as_ref(), signer.key().as_ref()], 
        bump
    )]
    pub market: Account<'info, Market>,
    pub system_program: Program<'info, System>,
}