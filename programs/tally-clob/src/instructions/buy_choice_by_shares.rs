use anchor_lang::prelude::*;

use crate::{errors::TallyClobErrors, Market, MarketPortfolio, User};

pub fn buy_choice_by_shares(
    ctx: Context<BuyChoiceByShares>, 
    choice_index: usize, 
    shares: f64
) -> Result<()> {

    // get order price
    let order_price =  ctx.accounts.market.get_choice_price_by_shares(choice_index, shares)?;

    // check user balance
    require!(order_price <= ctx.accounts.user.balance, TallyClobErrors::BalanceTooLow);

    // subtract from user balance
    ctx.accounts.user.withdraw_from_balance(order_price)?;
    
    // add shares to user portfolio
    ctx.accounts.market_portfolio.add_to_portfolio(choice_index, shares)?;

    Ok(())
}

#[derive(Accounts)]
pub struct BuyChoiceByShares<'info> {
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


pub enum BuyType {
    Shares
}