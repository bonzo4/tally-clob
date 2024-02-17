use std::borrow::BorrowMut;

use anchor_lang::prelude::*;

use crate::{errors::TallyClobErrors, utils::has_unique_elements, Market, MarketPortfolio, MarketStatus, Order, User};

pub fn bulk_buy_by_price(
    ctx: Context<BulkBuyByPrice>,
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
        top < order.requested_price && order.requested_price < bottom
    }).collect::<Vec<bool>>();
    require!(prices_in_range.iter().all(|in_range| !!in_range), TallyClobErrors::PriceEstimationOff);
    
    // 5. check if user has enough balance
    let order_prices = orders.iter()
        .enumerate()
        .map(|(_index, order)| {
            order.amount
        }
    ).collect::<Vec<f64>>();
    let total_price = order_prices.iter()
        .sum();
    require!(ctx.accounts.user.balance >= total_price, TallyClobErrors::BalanceTooLow);

    // Prep order 
    // 1. get total shares based on market_status
    let order_shares = ctx.accounts.market
        .bulk_buy_shares(orders, market_periods)?;

    // Make order
    // 1. update user balance
    ctx.accounts.user.withdraw_from_balance(total_price)?;
    // 2. update market pots and prices
    let orders_with_shares = orders.iter()
        .enumerate()
        .map(|(order_index, order)| {
            let mut order_with_share = order.clone();
            order_with_share.amount = order_shares[order_index] as f64;
            order_with_share
        }).collect::<Vec<Order>>();
    ctx.accounts.market.adjust_markets_after_buy(&orders_with_shares, order_prices)?;
    // 3. update user portfolio
    ctx.accounts.market_portfolio.bulk_add_to_portfolio(&orders_with_shares)?;

    Ok(())
}

#[derive(Accounts)]
pub struct BulkBuyByPrice<'info> {
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