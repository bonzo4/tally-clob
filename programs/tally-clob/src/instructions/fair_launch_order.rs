use std::borrow::BorrowMut;

use anchor_lang::{context::Context, prelude::*};

use crate::{errors::TallyClobErrors, utils::has_unique_elements, ChoiceMarket, Market, MarketPortfolio, MarketStatus, Order, User};

pub fn fair_launch_order(
    ctx: Context<FairLaunchOrder>,
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
    let market_periods = &ctx.accounts.market
        .get_buying_periods(orders)?;
    let mut is_buying_periods = market_periods.iter()
        .map(|market_period| [MarketStatus::FairLaunch].contains(market_period));
    require!(is_buying_periods.all(|is_buying_period| !!is_buying_period), TallyClobErrors::NotBuyingPeriod);

    let total_price = orders.iter().map(|order|order.amount).sum();

    require!(ctx.accounts.user.balance >= total_price, TallyClobErrors::BalanceTooLow);

    ctx.accounts.user.balance -= total_price;

    orders.iter()
        .for_each(|order| {
            // let sub_market = ctx.accounts.market.get_sub_market(&order.sub_market_id).unwrap();
            // let choice = sub_market.get_choice(&order.choice_id).unwrap();
            let new_choices = ctx.accounts.market.get_sub_market(&order.sub_market_id).unwrap().choices.iter()
                .map(|choice| {
                    if order.choice_id == choice.id  {
                        return ChoiceMarket {
                        fair_launch_pot: choice.fair_launch_pot + order.amount,
                        ..choice.clone()
                        }
                    }
                    choice.clone()
                }).collect::<Vec<ChoiceMarket>>();

            let total_pot = new_choices.iter()
                .map(|choice| choice.fair_launch_pot)
                .sum::<f64>();
            let invariant = total_pot * total_pot;
            let pot_proportion = new_choices.iter()
                .map(|choice|{
                    choice.fair_launch_pot/ total_pot * 100.0
                })
                .collect::<Vec<f64>>();
            let k = (invariant/ (pot_proportion[0] * pot_proportion[1])).sqrt();

            ctx.accounts.market.get_sub_market(&order.sub_market_id).unwrap().get_choice(&order.choice_id).unwrap().fair_launch_pot += order.amount;
            ctx.accounts.market.get_sub_market(&order.sub_market_id).unwrap().get_choice(&order.choice_id).unwrap().usdc_pot += order.amount;
            ctx.accounts.market.get_sub_market(&order.sub_market_id).unwrap().get_choice(&order.choice_id).unwrap().minted_shares += order.amount;
            ctx.accounts.market.get_sub_market(&order.sub_market_id).unwrap().invariant = invariant;
            
            ctx.accounts.market.get_sub_market(&order.sub_market_id).unwrap().choices[0].pot_shares = k * pot_proportion[1];
            ctx.accounts.market.get_sub_market(&order.sub_market_id).unwrap().choices[1].pot_shares = k * pot_proportion[0];

            
            ctx.accounts.market_portfolio
                .add_to_portfolio(&order.sub_market_id, &order.choice_id, order.amount)
                .unwrap();
        });

    Ok(())
}

#[derive(Accounts)]
pub struct FairLaunchOrder<'info> {
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
}