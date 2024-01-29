use anchor_lang::prelude::*;

use crate::Market;

pub fn resolve_market(
    _ctx: Context<ResolveMarket>,
) -> Result<()> {
    Ok(())
}

#[derive(Accounts)]
pub struct ResolveMarket<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub market: Account<'info, Market>,
}