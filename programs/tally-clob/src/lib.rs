use anchor_lang::prelude::*;
use instructions::*;
use state::*;

// wallet - proxyWallet
// - SPL Token Account - TUSDC and Choice Token
// - SPL Token - Amount

pub mod instructions;
pub mod errors;
pub mod state;
pub mod utils;

declare_id!("CPtwPtwjQhPbfZYsHiWskky7gRtBcRzFsh4HvsQ5tmXe");

#[program]
pub mod tally_clob {
    use self::utils::is_authorized;

    use super::*;

    pub fn init_market(
        ctx: Context<InitMarket>,
        sub_markets: Vec<SubMarket>
    ) -> Result<()> {
        is_authorized(ctx.accounts.signer.key())?;

        instructions::init_market(
            ctx,
            sub_markets
        )
    }

    pub fn init_wallet(
        ctx: Context<InitWallet>,
    ) -> Result<Pubkey> {
        let user_key = instructions::init_wallet(ctx)?;

        Ok(user_key)
    }

    pub fn add_to_balance(
        ctx: Context<AddToBalance>,
        amount: f64
    ) -> Result<()> {
        is_authorized(ctx.accounts.signer.key())?;

        instructions::add_to_balance(ctx, amount)
    }

    pub fn withdraw_from_balance(
        ctx: Context<WithdrawFromBalance>,
        amount: f64
    ) -> Result<()> {
        is_authorized(ctx.accounts.signer.key())?;

        instructions::withdraw_from_balance(ctx, amount)
    }

    pub fn bulk_buy_by_price(
        ctx: Context<BulkBuyByPrice>,
        orders: Vec<Order>
    ) -> Result<()> {
        instructions::bulk_buy_by_price(ctx, orders)
    }

    pub fn bulk_buy_by_shares(
        ctx: Context<BulkBuyByShares>,
        orders: Vec<Order>
    ) -> Result<()> {
        instructions::bulk_buy_by_shares(ctx, orders)
    }

    pub fn bulk_sell_by_price(
        ctx: Context<BulkSellByPrice>,
        orders: Vec<Order>
    ) -> Result<()> {
        instructions::bulk_sell_by_price(ctx, orders)
    }

    pub fn bulk_sell_by_shares(
        ctx: Context<BulkSellByShares>,
        orders: Vec<Order>
    ) -> Result<()> {
        instructions::bulk_sell_by_shares(ctx, orders)
    }

    pub fn resolve_market(
        ctx: Context<ResolveMarket>
    ) -> Result<()> {
        is_authorized(ctx.accounts.signer.key())?;

        instructions::resolve_market(ctx)
    }

}

