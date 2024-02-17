use std::ops::{AddAssign, SubAssign};

use anchor_lang::prelude::*;

use crate::{errors::TallyClobErrors, BOOL_SIZE, U64_SIZE};

use super::{DISCRIMINATOR_SIZE, F64_SIZE};

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct ChoiceMarket {
    pub id: u64,
    pub total_pot: f64,
    pub shares: u64,
    pub price: f64,
    pub winning_choice: bool
}

impl ChoiceMarket {

    pub const SIZE: usize = DISCRIMINATOR_SIZE
    + U64_SIZE
    + (F64_SIZE * 3)
    + BOOL_SIZE;


    pub fn get_buy_order_price(
        &mut self, market_pot: f64, shares_to_buy: u64,
    ) -> Result<f64> {

        let mut shares = 0;
        let mut total_price = 0.0;

        while shares < shares_to_buy {
            let new_price = (self.total_pot + self.price) / (market_pot + self.price);
            total_price += new_price;
            shares += 1
        }

        Ok(total_price)

    }

    pub fn get_sell_order_price(
        &self, 
        market_pot: f64, 
        shares_to_sell: u64
    ) -> Result<f64> {

        let mut shares = 0;
        let mut total_price = 0.0;

        while shares < shares_to_sell {
            let new_price = (self.total_pot - self.price) / (market_pot - self.price);
            total_price += new_price;
            shares += 1
        }

        Ok(total_price)

    }

    pub fn get_buy_order_shares(
        &mut self, market_pot: f64, buy_price: f64
    ) -> Result<u64> {
        
        let mut shares = 0;
        let mut total_price  = 0.0;
        
        while total_price < buy_price {
            
            let new_price = (self.total_pot + self.price) / (market_pot + self.price);
            total_price += new_price;
            shares += 1;
        }

        Ok(shares)

    }

    pub fn get_sell_order_shares(
        &self, market_pot: f64, sell_price: f64
    ) -> Result<u64> {
        
        let mut shares = 0;
        let mut total_price = 0.0;

        while total_price < sell_price {
            let new_price = (self.total_pot - self.price) / (market_pot - self.price);
            total_price += new_price;
            shares += 1;
        }

        Ok(shares)
    }
   

    pub fn reprice(&mut self, market_pot: f64) -> Result<&mut Self> {
        self.price = if market_pot != 0.0 {self.total_pot / market_pot} else {1.0};
        
        Ok(self)
    }
    
    pub fn add_to_pot(&mut self, amount: f64) -> Result<&mut Self> {

        require!(amount > 0.0, TallyClobErrors::AmountToAddTooLow);

        self
            .total_pot
            .add_assign(amount);
        
        Ok(self)
    }

    pub fn add_to_shares(&mut self, amount: u64) -> Result<&mut Self> {
        require!(amount > 0, TallyClobErrors::AmountToAddTooLow);

        self
            .shares
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

    pub fn delete_shares(&mut self, amount: u64) -> Result<&mut Self> {
        require!(amount > 0, TallyClobErrors::AmountToWithdrawTooLow);
        require!(amount <= self.shares, TallyClobErrors::AmountToWithdrawTooGreat);

        self
            .shares
            .sub_assign(amount);

        Ok(self)
    }
}