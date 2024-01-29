use std::ops::{AddAssign, SubAssign};

use anchor_lang::prelude::*;

use crate::errors::TallyClobErrors;

use super::{DISCRIMINATOR_SIZE, F64_SIZE};

#[account]
pub struct ChoiceMarket {
    pub total_pot: f64,
    pub shares: f64,
    pub price: f64,
}

impl ChoiceMarket {

    pub const SIZE: usize = DISCRIMINATOR_SIZE
    + (F64_SIZE * 3);

    pub fn buy_order_by_shares(
        &mut self, 
        market_pot: f64, 
        shares_to_buy: f64
    ) -> Result<f64> {

        let new_liquidity = self
        .get_buy_price_by_shares(market_pot, shares_to_buy)
        .unwrap();

        self.add_to_pot(new_liquidity)?;
        
        Ok(new_liquidity)
    }

    pub fn sell_order_by_shares(
        &mut self, 
        market_pot: f64, 
        shares_to_sell: f64
    ) -> Result<f64> {
        let withdraw_liqudity = self
            .sell_price_by_shares(
                market_pot, 
                shares_to_sell,
                0.0,
                self.price
            ).unwrap();
        
        let _ = self
            .withdraw_from_pot(withdraw_liqudity)
            .unwrap();

        Ok(withdraw_liqudity)
    }

    pub fn get_buy_price_by_shares(
        &mut self,
        market_pot: f64,
        shares_to_buy: f64,
    ) -> Result<f64> {

        pub fn total_buy_price(market: &mut ChoiceMarket, market_pot: f64, shares_to_buy: f64, shares: f64, total_price: f64, price: f64) -> Result<f64> {

            if shares >= shares_to_buy {
                return Ok(price)
            }

            let shares_to_inc = if (shares_to_buy - shares) > 1.00 {1.00} else {shares_to_buy - shares};
    
            let new_price = (market.total_pot + (price * shares_to_inc))/ market_pot;

            Ok(total_buy_price(
                market,
                market_pot, 
                shares_to_buy, 
                shares + shares_to_inc, 
                total_price + price, 
                new_price
            )?)
        }

        Ok(total_buy_price(self, market_pot, shares_to_buy, self.price, 0.0,  0.0)?)

    }

    pub fn sell_price_by_shares(
        &self, 
        market_pot: f64, 
        shares_to_sell: f64,
        total_withdrawn: f64,
        mut price: f64
    ) -> Result<f64> {

        if shares_to_sell <= 0.0 {
            return Ok(price)
        }

        let shares_to_dec = if shares_to_sell < 1.00 {1.00} else {shares_to_sell};

        price = (self.total_pot - (price * shares_to_dec)) / market_pot;

        Ok(self
            .sell_price_by_shares(
                market_pot,
                shares_to_sell - shares_to_dec,
                total_withdrawn, 
                price
            ).unwrap())
    }
   

    pub fn reprice(&mut self, market_pot: f64) -> Result<&mut Self> {
        self.total_pot = if market_pot == 0.0 {self.total_pot / market_pot} else {0.0};
        
        Ok(self)
    }
    
    pub fn add_to_pot(&mut self, amount: f64) -> Result<&mut Self> {

        require!(amount > 0.0, TallyClobErrors::AmountToAddTooLow);

        self
            .total_pot
            .add_assign(amount);
        
        Ok(self)
    }

    pub fn withdraw_from_pot(&mut self, amount: f64) -> Result<&mut Self>  {
        require!(amount > 0.0, TallyClobErrors::AmountToWithdrawTooLow);
        require!(amount <= self.total_pot, TallyClobErrors::AmountToWithdrawTooGreat);

        self
            .total_pot
            .sub_assign(amount);
        Ok(self)
    }
}