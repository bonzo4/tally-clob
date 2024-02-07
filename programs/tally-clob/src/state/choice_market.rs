use std::ops::{AddAssign, SubAssign};

use anchor_lang::prelude::*;

use crate::errors::TallyClobErrors;

use super::{DISCRIMINATOR_SIZE, F64_SIZE};

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct ChoiceMarket {
    pub id: u64,
    pub total_pot: f64,
    pub shares: f64,
    pub price: f64,
    pub winning_choice: bool
}

impl ChoiceMarket {

    pub const SIZE: usize = DISCRIMINATOR_SIZE
    + (F64_SIZE * 3);


    pub fn get_buy_order_price(
        &mut self, market_pot: f64, shares_to_buy: f64,
    ) -> Result<f64> {

        pub fn total_buy_price(
            market: &mut ChoiceMarket, market_pot: f64, shares_to_buy: f64, price: f64, shares: f64, total_price: f64
        ) -> Result<f64> {

            if shares >= shares_to_buy {
                return Ok(price)
            }

            let shares_to_inc = if (shares_to_buy - shares) > 1.00 {1.00} else {shares_to_buy - shares};
            let old_price = price * shares_to_inc;
            let new_price = (market.total_pot + old_price) / (market_pot + old_price);

            Ok(total_buy_price(
                market, market_pot, shares_to_buy, new_price, shares + shares_to_inc, total_price + price, 
            )?)
        }

        Ok(total_buy_price(
            self, market_pot, shares_to_buy, self.price, 0.0, 0.0,  
        )?)
    }

    pub fn get_sell_order_price(
        &self, 
        market_pot: f64, 
        shares_to_sell: f64
    ) -> Result<f64> {

        pub fn total_sell_price(market: &ChoiceMarket, market_pot: f64, shares_to_sell: f64, total_withdrawn: f64, price: f64) -> Result<f64> {
            if shares_to_sell <= 0.0 {
                return Ok(price)
            }
    
            let shares_to_dec = if shares_to_sell < 1.00 {1.00} else {shares_to_sell};
            let old_price = price * shares_to_dec;
            let new_price = (market.total_pot + old_price) / (market_pot + old_price);
    
            Ok(total_sell_price(
                    market, market_pot, shares_to_sell - shares_to_dec, total_withdrawn, new_price
                )?)
        }

        Ok(total_sell_price(
            self, market_pot, shares_to_sell, 0.0, self.price
        )?)
    }

    pub fn get_buy_order_shares(
        &mut self, market_pot: f64, buy_price: f64
    ) -> Result<f64> {
        
        let mut shares = 0.0;
        let mut total_price  = 0.0;
        
        while total_price < buy_price {
            
            let new_price = (self.total_pot + self.price) / (market_pot + self.price);
            total_price += new_price;
            shares += 1.0;
        }

        Ok(shares)

    }

    pub fn get_sell_order_shares(
        &self, market_pot: f64, sell_price: f64
    ) -> Result<f64> {
        
        let mut shares = 0.0;
        let mut total_price = 0.0;

        while total_price < sell_price {
            let new_price = (self.total_pot - self.price) / (market_pot - self.price);
            total_price += new_price;
            shares += 1.0;
        }

        Ok(shares)
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