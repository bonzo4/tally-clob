use std::borrow::BorrowMut;

use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Mint, Token, TokenAccount, Transfer};

use crate::{errors::TallyClobErrors, utils::has_unique_elements, FinalOrder, Market, MarketPortfolio, MarketStatus, Order, User};

pub fn bulk_sell_by_price(
    ctx: Context<BulkSellByPrice>,
    mut orders: Vec<Order>
) -> Result<()> {
    let mint = &ctx.accounts.mint;
    let mint_key = mint.key().to_string();
    let usdc_key = "5DUWZLh3zPKAAJKu7ftMJJrkBrKnq3zHPPmguzVkhSes".to_string(); // not actually usdc address for development

    require!(mint_key == usdc_key, TallyClobErrors::NotUSDC);

    let fee_account = &ctx.accounts.fee_usdc_account;
    let source = &ctx.accounts.from_usdc_account;
    let authority = &ctx.accounts.signer;

    require!(source.owner.to_string() == "7rTBUSkc8PHPW3VwGiPB4EbwHWxoSvVpMmbnAqRiGwWx", TallyClobErrors::NotAuthorized);
    require!(fee_account.owner.to_string() == "eQv1C2XUfsn1ynM65NghBikNsH4TDnTQn5aSZYZdH79",TallyClobErrors::NotAuthorized);
    require!(source.owner.to_string() == authority.key().to_string(), TallyClobErrors::NotAuthorized);

    let token_program = &ctx.accounts.token_program;
    let cpi_program = token_program.to_account_info();

    let orders: &mut Vec<Order> = orders.borrow_mut();

    // check orders
    // 1. check if there is less than 10 orders,
    require!(orders.len() <= ctx.accounts.market.sub_markets.len(), TallyClobErrors::BulkOrderTooBig);

    // 2. check if there are any duplicate choice_ids
    let sub_market_ids = orders.iter().map(|order| order.sub_market_id).collect::<Vec<u64>>();
    require!(has_unique_elements(sub_market_ids), TallyClobErrors::SameSubMarket);
    
    // 3. check if all the requested submarkets are in a buying period
    let market_periods = &ctx.accounts.market
        .get_buying_periods(orders)?;
    let mut is_selling_periods = market_periods.iter()
        .map(|market_period| [MarketStatus::Trading].contains(market_period));
    require!(is_selling_periods.all(|is_buying_period| !!is_buying_period), TallyClobErrors::NotSellingPeriod);
    
    // 4. calculate the prices
    let order_values = ctx.accounts.market.bulk_sell_values_by_price(orders)?;

    order_values.iter().for_each(|order| msg!("order_values: shares: {}, price: {}, fee: {}", order.shares_to_sell, order.sell_price, order.fee_price));

    // 5. check for slippage on the price per share
    let actual_prices_per_share = order_values.iter()
    .map(|values| values.sell_price / values.shares_to_sell).collect::<Vec<f64>>();
    actual_prices_per_share.iter().for_each(|pps|msg!("pps: {}", pps));

    // 6. Check if all prices are within the expected range
    let prices_in_range = orders.iter().enumerate().map(|(index, order)| {
        let top = order.requested_price_per_share * 1.05; // 1.05 as fixed-point
        let bottom = order.requested_price_per_share * 0.95; // 0.95 as fixed-point
        let within_limit = bottom < actual_prices_per_share[index] && actual_prices_per_share[index] < top;
    within_limit
    }).collect::<Vec<bool>>();

    // 7. Ensure all prices are within the expected range
    require!(prices_in_range.iter().all(|in_range| *in_range), TallyClobErrors::PriceEstimationOff);


    // prep order
    // 1. calculate sell values
    let final_orders = orders.iter()
        .enumerate()
        .map(|(index, order)| {
            let values = &order_values[index];
            FinalOrder {
                sub_market_id: order.sub_market_id, 
                choice_id: order.choice_id, 
                price: values.sell_price + values.fee_price, 
                shares: values.shares_to_sell,
                fee_price: values.fee_price
            }
        }).collect::<Vec<FinalOrder>>();

    final_orders.iter().for_each(|order|msg!("final_order: shares: {}, price: {}", order.price, order.shares));


    // check if there are enough shares
    ctx.accounts.market_portfolio
        .check_portfolio_shares(&final_orders)?;

    let total_price_after_fees = order_values.iter().map(|order| order.sell_price).sum();

    // Make order
    // 1. update market_portfolio
    ctx.accounts.market_portfolio.bulk_sell_from_portfolio(&final_orders)?;
    // 2. update market pots and prices
    ctx.accounts.market.adjust_markets_after_sell(&final_orders)?;
    // 3. update user portfolio
    ctx.accounts.user.add_to_balance(total_price_after_fees)?;

    //send fees
    let total_fee_amount = order_values.iter().map(|order|order.fee_price).sum::<f64>();

    let fee_cpi_accounts = Transfer {
        from: source.to_account_info().clone(),
        to: fee_account.to_account_info().clone(),
        authority: authority.to_account_info().clone()
    };

    transfer (
        CpiContext::new(cpi_program, fee_cpi_accounts),
        total_fee_amount as u64 * 10_u64.pow(6)
    )?;

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
    pub system_program: Program<'info, System>,
    #[account(mut)]
    pub from_usdc_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub fee_usdc_account: Account<'info, TokenAccount>,
    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
}