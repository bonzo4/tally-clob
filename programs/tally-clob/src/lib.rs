use anchor_lang::prelude::*;
use instructions::*;
use state::*;

pub mod instructions;
pub mod errors;
pub mod state;
pub mod utils;

declare_id!("CPtwPtwjQhPbfZYsHiWskky7gRtBcRzFsh4HvsQ5tmXe");

#[program]
pub mod tally_clob {
    use crate::errors::TallyClobErrors;

    use self::utils::{is_owner, is_wallet_manager};

    use super::*;

    pub fn authorize_user(
        ctx: Context<AuthorizeUser>, 
        authorized: bool,
        user_key: Pubkey
    ) -> Result<()> {
        is_owner(ctx.accounts.signer.key())?;

        instructions::authorize_user(ctx, authorized)

    }

    pub fn init_market(
        ctx: Context<InitMarket>,
        sub_markets: Vec<SubMarket>,
        market_key: Pubkey
    ) -> Result<()> {
        require!(ctx.accounts.authorized_user.authorized, TallyClobErrors::NotAuthorized);

        instructions::init_market(
            ctx,
            sub_markets
        )
    }

    pub fn init_wallet(
        ctx: Context<InitWallet>,
        user_key: Pubkey
    ) -> Result<Pubkey> {
        is_wallet_manager(ctx.accounts.signer.key())?;
        let user_key = instructions::init_wallet(ctx)?;

        Ok(user_key)
    }

    pub fn add_to_balance(
        ctx: Context<AddToBalance>,
        amount: f64
    ) -> Result<()> {
        is_wallet_manager(ctx.accounts.signer.key())?;

        instructions::add_to_balance(ctx, amount)
    }

    pub fn withdraw_from_balance(
        ctx: Context<WithdrawFromBalance>,
        amount: f64
    ) -> Result<()> {
        is_wallet_manager(ctx.accounts.signer.key())?;

        instructions::withdraw_from_balance(ctx, amount)
    }

    pub fn bulk_buy_by_price(
        ctx: Context<BulkBuyByPrice>,
        orders: Vec<Order>
    ) -> Result<()> {
        is_wallet_manager(ctx.accounts.signer.key())?;
        instructions::bulk_buy_by_price(ctx, orders)
    }

    pub fn bulk_buy_by_shares(
        ctx: Context<BulkBuyByShares>,
        orders: Vec<Order>
    ) -> Result<()> {
        is_wallet_manager(ctx.accounts.signer.key())?;
        instructions::bulk_buy_by_shares(ctx, orders)
    }

    pub fn bulk_sell_by_price(
        ctx: Context<BulkSellByPrice>,
        orders: Vec<Order>
    ) -> Result<()> {
        is_wallet_manager(ctx.accounts.signer.key())?;
        instructions::bulk_sell_by_price(ctx, orders)
    }

    pub fn bulk_sell_by_shares(
        ctx: Context<BulkSellByShares>,
        orders: Vec<Order>
    ) -> Result<()> {
        is_wallet_manager(ctx.accounts.signer.key())?;
        instructions::bulk_sell_by_shares(ctx, orders)
    }

    pub fn resolve_market(
        ctx: Context<ResolveMarket>,
        sub_market_id: u64,
        choice_id: u64,
    ) -> Result<()> {
        require!(ctx.accounts.authorized_user.authorized, TallyClobErrors::NotAuthorized);

        instructions::resolve_market(ctx, sub_market_id, choice_id)
    }

    pub fn claim_winnings(
        ctx: Context<ClaimWinnings>,
        sub_market_id: u64,
        choice_id: u64
    ) -> Result<()> {

        instructions::claim_winnings(ctx, sub_market_id, choice_id)
    }

}

