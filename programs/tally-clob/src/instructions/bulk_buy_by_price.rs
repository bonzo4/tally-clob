use std::borrow::BorrowMut;

use anchor_lang::prelude::*;

use crate::{errors::TallyClobErrors, Order, Market, MarketPortfolio, User};

pub fn bulk_buy_by_price(
    ctx: Context<BulkBuyByPrice>,
    mut orders: Vec<Order>
) -> Result<()> {

    let orders: &mut Vec<Order> = orders.borrow_mut();

    // check orders
    // 1. check if there is less than 10 orders,
    require!(orders.len() <= 10, TallyClobErrors::BulkOrderTooBig);
    // 2. check if all the requested submarkets are in a buying period
    let market_periods = ctx.accounts.market
        .get_buying_periods(orders)?;
    // 3. check if user has enough balance
    let order_prices = orders.iter()
        .map(|order| order.amount)
        .collect::<Vec<f64>>();
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
    ctx.accounts.market.adjust_markets_after_buy(&orders, order_prices)?;
    // 3. update user portfolio
    let orders_with_shares = orders.iter()
        .enumerate()
        .map(|(order_index, order)| {
            let mut order_with_share = order.clone();
            order_with_share.amount = order_shares[order_index];
            order_with_share
        }).collect::<Vec<Order>>();
    ctx.accounts.market_portfolio.bulk_add_to_portfolio(&orders_with_shares)?;

    Ok(())
}

#[derive(Accounts)]
pub struct BulkBuyByPrice<'info> {
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