use std::borrow::BorrowMut;

use anchor_lang::{context::Context, prelude::*};
use anchor_spl::token::{transfer, Mint, Token, TokenAccount, Transfer};

use crate::{errors::TallyClobErrors, utils::has_unique_elements, FinalOrder, Market, MarketPortfolio, MarketStatus, Order, User};

pub fn bulk_buy_by_price(
    ctx: Context<BulkBuyByPrice>,
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
    let mut is_buying_periods = market_periods.iter()
        .map(|market_period| [MarketStatus::Trading].contains(market_period));
    require!(is_buying_periods.all(|is_buying_period| !!is_buying_period), TallyClobErrors::NotBuyingPeriod);

    // 4. calculate the prices
    let order_values = ctx.accounts.market.bulk_buy_values_by_price(orders)?;

    order_values.iter().for_each(|order| msg!("order_values: shares: {}, price: {}, fee: {}", order.shares_to_buy, order.buy_price, order.fee_price));

    // 5. check for slippage on the price per share
    let actual_prices_per_share = order_values.iter()
        .map(|values| values.buy_price as f64 / values.shares_to_buy as f64).collect::<Vec<f64>>();
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


    // 6. check if user has enough balance
    let total_price = order_values.iter().map(|values|values.buy_price + values.fee_price).sum::<u128>();
    require!(ctx.accounts.user.balance >= total_price, TallyClobErrors::BalanceTooLow);

    // prep order
    let final_orders = orders.iter()
        .enumerate()
        .map(|(index, order)| {
            let values = &order_values[index];
            FinalOrder {
                sub_market_id: order.sub_market_id, 
                choice_id: order.choice_id, 
                price: values.buy_price, 
                shares: values.shares_to_buy,
                fee_price: values.fee_price
            }
        }).collect::<Vec<FinalOrder>>();
    
    final_orders.iter().for_each(|order|msg!("final_order: shares: {}, price: {}", order.shares, order.price));

    // Make order
    // 1. update user balance
    ctx.accounts.user.withdraw_from_balance(total_price)?;
    // 2. update market pots and prices
    ctx.accounts.market.adjust_markets_after_buy(&final_orders)?;
    // 3. update user portfolio
    ctx.accounts.market_portfolio.bulk_add_to_portfolio(&final_orders)?;

    //send fees
    let total_fee_amount = order_values.iter().map(|order|order.fee_price).sum::<u128>();

    let fee_cpi_accounts = Transfer {
        from: source.to_account_info().clone(),
        to: fee_account.to_account_info().clone(),
        authority: authority.to_account_info().clone()
    };
    
    transfer (
        CpiContext::new(cpi_program, fee_cpi_accounts),
        total_fee_amount as u64
    )?;

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
    pub system_program: Program<'info, System>,
    #[account(mut )]
    pub from_usdc_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub fee_usdc_account: Account<'info, TokenAccount>,
    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
}