use anchor_lang::prelude::*;

use crate::{errors::TallyClobErrors, Market, MarketPortfolio, User};

pub fn claim_winnings(
    ctx: Context<ClaimWinnings>,
    sub_market_id: u64,
    choice_id: u64
) -> Result<()> {

    let resolved = ctx.accounts.market.get_sub_market(&sub_market_id)?.resolved;

    let winning_choice = ctx.accounts.market.get_sub_market(&sub_market_id)?.get_choice(&choice_id)?.winning_choice;

    let choice_market_portfolio = ctx.accounts.market_portfolio.get_sub_market_portfolio(&sub_market_id)?.get_choice_market_portfolio(&choice_id)?;

    // check if market is resolved
    require!(resolved, TallyClobErrors::MarketNotResolved);

    // check if user has a winning choice
    require!(winning_choice, TallyClobErrors::NotWinningChoice);

    // check if user's shares have already been claimed
    require!(!choice_market_portfolio.claimed, TallyClobErrors::AlreadyClaimed);

    let total_pot = ctx.accounts.market.get_sub_market(&sub_market_id)?.choices.iter()
        .map(|choice| {
            msg!(&choice.usdc_pot.to_string());
            choice.usdc_pot
        })
        .sum::<u128>();
    

    let winning_shares = ctx.accounts.market.get_sub_market(&sub_market_id)?.get_choice(&choice_id)?.minted_shares;

    let pot_portion = choice_market_portfolio.shares / winning_shares;

    let total_winnings = pot_portion * total_pot;

    // withdraw from shares
    choice_market_portfolio.withdraw_from_portfolio(choice_market_portfolio.shares)?;

    choice_market_portfolio.claimed = true;

    // add to balance 
    ctx.accounts.user.add_to_balance(total_winnings)?;

    // err!(TallyClobErrors::NotAValidOrder)

    Ok(())
}



#[derive(Accounts)]
pub struct ClaimWinnings<'info> {
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