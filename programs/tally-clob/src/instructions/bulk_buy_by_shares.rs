use std::borrow::BorrowMut;

use anchor_lang::prelude::*;

use crate::{errors::TallyClobErrors, utils::has_unique_elements, Market, MarketPortfolio, MarketStatus, Order, User};

pub fn bulk_buy_by_shares(
    ctx: Context<BulkBuyByShares>,
    mut orders: Vec<Order>
) -> Result<()> {

    let orders: &mut Vec<Order> = orders.borrow_mut();
    

    // check orders
    // 1. check if there is less than 10 orders,
    require!(orders.len() <= ctx.accounts.market.sub_markets.len(), TallyClobErrors::BulkOrderTooBig);

    // 2. check if there are any duplicate choice_ids
    let sub_market_ids = orders.iter().map(|order| order.sub_market_id).collect::<Vec<u64>>();
    require!(has_unique_elements(sub_market_ids), TallyClobErrors::SameSubMarket);

    // 3. check if all the requested submarkets are in a buying period
    let market_periods = ctx.accounts.market
        .get_buying_periods(orders)?;
    let mut is_buying_periods = market_periods.iter()
        .map(|market_period| [MarketStatus::FairLaunch, MarketStatus::Trading].contains(market_period));
    require!(is_buying_periods.all(|is_buying_period| !!is_buying_period), TallyClobErrors::NotBuyingPeriod);

    // 4. check if the requested prices are at least within 5%
    let acutal_prices = ctx.accounts.market.get_order_prices(orders)?;
    let prices_in_range = orders.iter().enumerate().map(|(index, order)| {
        if market_periods[index] == MarketStatus::FairLaunch 
            && order.requested_price == ctx.accounts.market.get_sub_market_default_price(&order.sub_market_id).unwrap() {
            return true;
        }
        let top = acutal_prices[index] * 1.05;
        let bottom = acutal_prices[index] * 0.95;
        bottom < order.requested_price && order.requested_price < top
    }).collect::<Vec<bool>>();
    require!(prices_in_range.iter().all(|in_range| !!in_range), TallyClobErrors::PriceEstimationOff);
    
    // Prep order 
    // 1. get total price based on market_status
    let order_prices = ctx.accounts.market
        .bulk_buy_price(orders, market_periods)?;
    let total_price = order_prices.iter().sum();
    // 2. check that the total price is less than the user balance
    require!(ctx.accounts.user.balance >= total_price, TallyClobErrors::BalanceTooLow);


    // Make order
    // 1. update user balance
    ctx.accounts.user.withdraw_from_balance(total_price)?;
    // 2. update market pots and prices
    ctx.accounts.market.adjust_markets_after_buy(orders, order_prices)?;
    // 3. update user portfolio
    ctx.accounts.market_portfolio.bulk_add_to_portfolio(orders)?;

    Ok(())
}



#[derive(Accounts)]
pub struct BulkBuyByShares<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub user: Account<'info, User>,
    #[account(mut)]
    pub market: Account<'info, Market>,
    #[account(
        init_if_needed,
        payer = signer,
        space = MarketPortfolio::SIZE,
        seeds = [b"market_portfolios".as_ref(), market.key().as_ref(), user.key().as_ref(), ],
        bump
    )]
    pub market_portfolio: Account<'info, MarketPortfolio>,
    pub system_program: Program<'info, System>
}