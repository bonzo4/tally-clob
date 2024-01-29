use anchor_lang::prelude::*;

use crate::{errors::TallyClobErrors, utils::clock, Market, MarketPortfolio, User};

pub fn sell_choice_by_shares(
    ctx: Context<SellChoiceByShares>, 
    choice_index: usize, 
    shares: f64
) -> Result<()> {

    // check for correct selling period
    let now = clock::current_timestamp();

    let is_intializing = ctx.accounts.market.fair_launch_start > now;

    require!(!is_intializing, TallyClobErrors::MarketIntializing);

    let is_trading_period = ctx.accounts.market.trading_start < now 
        && ctx.accounts.market.trading_end > now;

    require!(is_trading_period, TallyClobErrors::NotSellingPeriod);

    // get order price
    let order_price =  ctx.accounts.market.get_sell_order_price(choice_index, shares)?;

    // check user balance
    require!(order_price <= ctx.accounts.user.balance, TallyClobErrors::BalanceTooLow);

    // subtract from user balance
    ctx.accounts.user.withdraw_from_balance(order_price)?;

    // adjust market prices
    ctx.accounts.market
        .add_to_pot(order_price)?
        .reprice_choices()?;
    
    // add shares to user portfolio
    ctx.accounts.market_portfolio.add_to_portfolio(choice_index, shares)?;

    Ok(())
}

#[derive(Accounts)]
pub struct SellChoiceByShares<'info> {
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
        mut,
        seeds = [market.key().as_ref(), user.key().as_ref()],
        bump
    )]
    pub market_portfolio: Account<'info, MarketPortfolio>
}
pub enum SellType {
    Shares
}