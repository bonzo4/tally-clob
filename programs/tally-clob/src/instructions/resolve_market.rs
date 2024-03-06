use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Mint, Token, TokenAccount, Transfer};

use crate::{errors::TallyClobErrors, AuthorizedUser, ChoiceMarket, Market};

pub fn resolve_market(
    ctx: Context<ResolveMarket>,
    sub_market_id: u64,
    choice_id: u64
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

    ctx.accounts.market.get_sub_market(&sub_market_id)?.resolved;

    require!(!ctx.accounts.market.get_sub_market(&sub_market_id)?.resolved, TallyClobErrors::MarketAlreadyResolved);

    ctx.accounts.market.get_sub_market(&sub_market_id)?.resolved = true;

    ctx.accounts.market.get_sub_market(&sub_market_id)?.get_choice(&choice_id)?.winning_choice = true;

    let mut losing_choices = ctx.accounts.market.get_sub_market(&sub_market_id)?.choices.iter_mut()
        .filter(|choice| choice.id != choice_id)
        .collect::<Vec<&mut ChoiceMarket>>();
    

    let fee_price = losing_choices[0].usdc_pot / 10;

    losing_choices[0].usdc_pot -= fee_price;

    let token_program = &ctx.accounts.token_program;
    let cpi_program = token_program.to_account_info();


    let fee_cpi_accounts = Transfer {
        from: source.to_account_info().clone(),
        to: fee_account.to_account_info().clone(),
        authority: authority.to_account_info().clone()
    };

    transfer (
        CpiContext::new(cpi_program, fee_cpi_accounts),
        fee_price as u64
    )?;

    Ok(())
}

#[derive(Accounts)]
pub struct ResolveMarket<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub authorized_user: Account<'info, AuthorizedUser>,
    #[account(mut)]
    pub market: Account<'info, Market>,
    pub system_program: Program<'info, System>,
    #[account(mut)]
    pub from_usdc_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub fee_usdc_account: Account<'info, TokenAccount>,
    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
}