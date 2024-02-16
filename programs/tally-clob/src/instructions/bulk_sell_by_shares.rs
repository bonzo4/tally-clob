use std::borrow::BorrowMut;

use anchor_lang::prelude::*;

use crate::{errors::TallyClobErrors, Market, MarketPortfolio, Order, User};

pub fn bulk_sell_by_shares(
    ctx: Context<BulkSellByShares>,
    mut orders: Vec<Order>
) -> Result<()> {
    
    let orders: &mut Vec<Order> = orders.borrow_mut();

    // check orders
    // 1. check if there is less than 10 orders,
    require!(orders.len() <= 10, TallyClobErrors::BulkOrderTooBig);
    // 2. check if all the requested submarkets are in a selling period
    ctx.accounts.market
        .check_selling_periods(orders)?;
    // 3. check if the requested prices are at least within
    let acutal_prices = ctx.accounts.market.get_order_prices(orders)?;
    let prices_in_range = orders.iter().enumerate().map(|(index, order)| {
        let top = acutal_prices[index] * 1.05;
        let bottom = acutal_prices[index] * 0.95;
        top < order.requested_price && order.requested_price < bottom
    }).collect::<Vec<bool>>();
    require!(prices_in_range.iter().all(|in_range| !!in_range), TallyClobErrors::PriceEstimationOff);
    // 4. check if there are enough shares to sell
    ctx.accounts.market_portfolio
        .check_portfolio_shares(orders)?;

    // Prep order 
    // 1. get total price based on market_status
    let order_prices = ctx.accounts.market
        .bulk_sell_price(orders)?;
    let total_price = order_prices.iter().sum();


    // Make order
    // 1. update market_portfolio
    ctx.accounts.market_portfolio.bulk_sell_from_portfolio(orders)?;
    // 2. update market pots and prices
    ctx.accounts.market.adjust_markets_after_buy(&orders, order_prices)?;
    // 3. update user portfolio
    ctx.accounts.user.add_to_balance(total_price)?;


    Ok(())
}

#[derive(Accounts)]
pub struct BulkSellByShares<'info> {
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