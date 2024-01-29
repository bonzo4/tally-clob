use anchor_lang::prelude::*;

use crate::{errors::TallyClobErrors, Market, MarketPortfolio, User};

pub fn sell_choice_by_shares(
    ctx: Context<SellChoiceByShares>, 
    choice_index: usize, 
    shares: f64
) -> Result<()> {

    let choice_portfolio = ctx.accounts
        .market_portfolio
        .choice_portfolio
        .get_mut(choice_index)
        .ok_or(TallyClobErrors::ChoicePortfolioNotFound)
        .unwrap();

    let _ = ctx.accounts.market
        .sell_order_by_shares(choice_index, shares);

    choice_portfolio.shares = choice_portfolio.shares - shares;

    Ok(())
}

#[derive(Accounts)]
pub struct SellChoiceByShares<'info> {
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
        mut,
        seeds = [market.key().as_ref(), user.key().as_ref()],
        bump
    )]
    pub market_portfolio: Account<'info, MarketPortfolio>
}
pub enum SellType {
    Shares
}