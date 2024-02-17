use std::borrow::BorrowMut;

use anchor_lang::prelude::*;

use crate::{errors::TallyClobErrors, utils::has_unique_elements, Market, MarketPortfolio, MarketStatus, Order, User};

pub fn bulk_sell_by_price(
    ctx: Context<BulkSellByPrice>,
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
    let mut is_selling_periods = market_periods.iter()
        .map(|market_period| [MarketStatus::Trading].contains(market_period));
    require!(is_selling_periods.all(|is_buying_period| !!is_buying_period), TallyClobErrors::NotSellingPeriod);
    
    // 4. check if the requested prices are at least within
    let acutal_prices = ctx.accounts.market.get_order_prices(orders)?;
    let prices_in_range = orders.iter().enumerate().map(|(index, order)| {
        let top = acutal_prices[index] * 1.05;
        let bottom = acutal_prices[index] * 0.95;
        top < order.requested_price && order.requested_price < bottom
    }).collect::<Vec<bool>>();
    require!(prices_in_range.iter().all(|in_range| !!in_range), TallyClobErrors::PriceEstimationOff);
    // 5. check if there are enough shares to sell
    let order_shares = ctx.accounts.market.bulk_sell_shares(orders)?;
    let orders_with_shares = orders.iter()
        .enumerate()
        .map(|(order_index, order)| {
            let mut order_with_share = order.clone();
            order_with_share.amount = order_shares[order_index] as f64;
            order_with_share
        }).collect::<Vec<Order>>();
    ctx.accounts.market_portfolio
        .check_portfolio_shares(&orders_with_shares)?;

    // Prep order 
    // 1. get total price
    let order_prices = orders.iter()
        .map(|order| order.amount)
        .collect::<Vec<f64>>();
    let total_price = order_prices.iter().sum();

    // Make order
    // 1. update market_portfolio
    ctx.accounts.market_portfolio.bulk_sell_from_portfolio(&orders_with_shares)?;
    // 2. update market pots and prices
    ctx.accounts.market.adjust_markets_after_buy(&orders, order_prices)?;
    // 3. update user portfolio
    ctx.accounts.user.add_to_balance(total_price)?;


    Ok(())
}

#[derive(Accounts)]
pub struct BulkSellByPrice<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub user: Account<'info, User>,
    #[account(mut)]
    pub market: Account<'info, Market>,
    #[account(mut)]
    pub market_portfolio: Account<'info, MarketPortfolio>,
    pub system_program: Program<'info, System>
}