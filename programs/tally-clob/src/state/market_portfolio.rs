use anchor_lang::prelude::*;

use crate::{errors::TallyClobErrors, vec_size, Market, Order, SubMarketPortfolio, DISCRIMINATOR_SIZE, U8_SIZE};

#[account]
pub struct MarketPortfolio {
    pub bump: u8,
    pub sub_market_portfolio: Vec<SubMarketPortfolio>,
}

impl MarketPortfolio {

    pub const SIZE: usize = DISCRIMINATOR_SIZE
    + U8_SIZE
    + vec_size(SubMarketPortfolio::SIZE, Market::MARKET_MAX_LENGTH);

    pub fn check_portfolio_shares(&mut self, orders: &Vec<Order>) -> Result<&Self> {
        for order in orders.iter() {
            let portfolio_shares = self
                    .get_choice_shares(
                        &order.sub_market_id, 
                        &order.choice_id
                    ).unwrap();

            require!(portfolio_shares >= order.requested_amount as u64, TallyClobErrors::NotEnoughSharesToSell);
        }

        Ok(self)
    }
    
    pub fn bulk_add_to_portfolio(&mut self, orders: &Vec<Order>) ->Result<&Self> {
        orders.iter()
            .for_each(|order| {
                self.add_to_portfolio(
                    &order.sub_market_id, 
                    &order.choice_id, 
                    order.requested_amount as u64
                ).unwrap();
            });

        Ok(self)
    }

    pub fn bulk_sell_from_portfolio(&mut self, orders: &Vec<Order>) -> Result<&Self> {
        orders.iter()
            .for_each(|order| {
                self
                    .sell_from_portfolio(
                        &order.sub_market_id, 
                        &order.choice_id, 
                        order.requested_amount as u64
                    ).unwrap();
            });

        Ok(self)
    }

    pub fn add_to_portfolio(&mut self, sub_market_id: &u64, choice_id: &u64, shares: u64) -> Result<&Self> {
        match self
            .get_sub_market_portfolio(sub_market_id) {
                Ok (sub_market_portfolio) => {
                    sub_market_portfolio
                        .get_choice_market_portfolio(choice_id)?
                        .add_to_portfolio(shares)?;
                }
                Err(_) => {
                    let new_sub_portfolio = &mut SubMarketPortfolio::new(*sub_market_id);
                    new_sub_portfolio.add_to_portfolio(choice_id, shares)?;
                    self.sub_market_portfolio.push(new_sub_portfolio.clone());             
                }
            }

        Ok(self)
    }
    
    pub fn sell_from_portfolio(&mut self, sub_market_id: &u64, choice_id: &u64, shares: u64) -> Result<&Self> {
        self
            .get_sub_market_portfolio(sub_market_id)?
            .sell_from_portfolio(choice_id, shares)?;

        Ok(self)
    }

    pub fn get_choice_shares(&mut self, sub_market_id: &u64, choice_id: &u64) -> Result<u64> {
        let choice_shares = self
            .get_sub_market_portfolio(sub_market_id)?
            .get_choice_shares(choice_id)?;

        Ok(choice_shares)
    }


    pub fn get_sub_market_portfolio(&mut self, sub_market_id: &u64) -> Result<&mut SubMarketPortfolio> {
        match self.sub_market_portfolio.binary_search_by_key(sub_market_id, |sub_market| sub_market.sub_market_id) {
            Ok(index) => Ok(&mut self.sub_market_portfolio[index]),
            Err(_) => err!(TallyClobErrors::SubMarketPortfolioNotFound),
        }
    }
    
}