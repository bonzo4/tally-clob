use anchor_lang::prelude::*;

use crate::{errors::TallyClobErrors, utils::clock, Market, MarketPortfolio, User};

pub fn buy_choice_by_shares(
    ctx: Context<BuyChoiceByShares>, 
    choice_index: usize, 
    shares: f64
) -> Result<()> {

    // check for correct buying period
    let now = clock::current_timestamp();

    let is_intializing = ctx.accounts.market.fair_launch_start > now;

    require!(!is_intializing, TallyClobErrors::MarketIntializing);

    let is_fair_launch = ctx.accounts.market.fair_launch_start < now 
        && ctx.accounts.market.fair_launch_end > now;

    let is_trading_period = ctx.accounts.market.trading_start < now 
        && ctx.accounts.market.trading_end > now;

    require!(is_fair_launch || is_trading_period, TallyClobErrors::NotSellingPeriod);

    // check for balance and requested shares

    // get order price
    let order_price =  ctx.accounts.market.get_buy_order_price(choice_index, shares)?;

    // subtract from user balance
    ctx.accounts.user.withdraw_from_balance(order_price)?;

    // adjust market prices
    ctx.accounts.market
        .add_to_choice_pot(choice_index, order_price)?
        .add_to_pot(order_price)?
        .reprice_choices()?;
    
    // add shares to user portfolio
    ctx.accounts.market_portfolio.add_to_portfolio(choice_index, shares)?;

    Ok(())
}

#[derive(Accounts)]
pub struct BuyChoiceByShares<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut, 
        seeds = [b"users", signer.key().as_ref()], 
        bump = user.bump
    )]
    pub user: Account<'info, User>,
    #[account(mut)]
    pub market: Account<'info, Market>,
    #[account(
        init_if_needed,
        payer = signer,
        space = MarketPortfolio::SIZE,
        seeds = [market.key().as_ref(), user.key().as_ref()],
        bump
    )]
    pub market_portfolio: Account<'info, MarketPortfolio>,
    pub system_program: Program<'info, System>
}   


pub enum BuyType {
    Shares
}