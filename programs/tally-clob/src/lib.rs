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

declare_id!("6tRBcUULjwksbwqp2DBKo84WAHVQUXYKhBAZ6Q2r1Lca");

#[program]
pub mod tally_wallets {
    use self::utils::is_authorized;

    use super::*;

    pub fn init_market(
        ctx: Context<InitMarket>,
        title: String,
        choice_count: u8,
        choices: Vec<ChoiceMarket>,
        fair_launch_start: i64,
        fair_launch_end: i64,
        trading_start: i64,
        trading_end: i64
    ) -> Result<()> {
        is_authorized(ctx.accounts.signer.key())?;

        instructions::init_market(
            ctx,
            title,
            choice_count,
            choices,
            fair_launch_start,
            fair_launch_end,
            trading_start,
            trading_end
        )
    }

    pub fn init_wallet(
        ctx: Context<InitWallet>,
    ) -> Result<()> {
        instructions::init_wallet(ctx)
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

    pub fn buy_choice_by_shares(
        ctx: Context<BuyChoiceByShares>,
        choice_index: usize,
        shares: f64
    ) -> Result<()> {
        instructions::buy_choice_by_shares(ctx, choice_index, shares)
    }

    pub fn sell_choice_by_shares(
        ctx: Context<SellChoiceByShares>,
        choice_index: usize,
        shares: f64
    ) -> Result<()> {
        instructions::sell_choice_by_shares(ctx, choice_index, shares)
    }

    pub fn resolve_market(
        ctx: Context<ResolveMarket>
    ) -> Result<()> {
        is_authorized(ctx.accounts.signer.key())?;

        instructions::resolve_market(ctx)
    }

}

