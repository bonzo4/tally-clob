use std::borrow::BorrowMut;

use anchor_lang::prelude::*;

use crate::{errors::TallyClobErrors, Market, MarketPortfolio, Order, OrderData, User};

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
    // 3. check if there are enough shares to sell
    ctx.accounts.market_portfolio
        .check_portfolio_shares(orders)?;

    // Prep order 
    // 1. get total price based on market_status
    let order_prices = ctx.accounts.market
        .bulk_sell_price(orders)?;
    let total_price = order_prices.iter().sum();

    // 2. check if esitamted price is too far off from acutal price
    let estimated_prices = orders.iter()
        .map(|order| order.estimated_amount)
        .collect::<Vec<f64>>();
    let total_estimated_price = estimated_prices.iter().sum::<f64>();
    let bottom = total_price * 0.99; let top = total_price * 1.01; 
    let within_range = bottom < total_estimated_price && total_estimated_price < top;
    require!(within_range, TallyClobErrors::PriceEstimationOff);

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
#[instruction(order_data: OrderData)]
pub struct BulkSellByShares<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut, 
        seeds = [b"users",  order_data.user_key.key().as_ref()], 
        bump = user.bump
    )]
    pub user: Account<'info, User>,
    #[account(
        mut,
        seeds = [b"markets".as_ref(), order_data.market_key.key().as_ref()],
        bump = market.bump
    )]
    pub market: Account<'info, Market>,
    #[account(
        mut,
        seeds = [order_data.user_key.key().as_ref(), order_data.market_key.key().as_ref()],
        bump
    )]
    pub market_portfolio: Account<'info, MarketPortfolio>,
    pub system_program: Program<'info, System>
}