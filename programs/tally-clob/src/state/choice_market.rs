use std::ops::{AddAssign, SubAssign};

use anchor_lang::prelude::*;

use crate::{errors::TallyClobErrors, BOOL_SIZE, U64_SIZE};

use super::{DISCRIMINATOR_SIZE, F64_SIZE};

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, PartialEq)]
pub struct BuyOrderValues {
    pub shares_to_buy: u64,
    pub buy_price: f64,
    pub fee_price: f64
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, PartialEq)]
pub struct SellOrderValues {
    pub shares_to_sell: u64,
    pub sell_price: f64,
    pub fee_price: f64
}

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
    + U64_SIZE
    + (F64_SIZE * 2)
    + BOOL_SIZE;


    pub fn get_buy_order_values(
        &mut self, 
        mut market_pot: f64, 
        buy_price: f64, 
        shares_to_buy: u64
    ) -> Result<BuyOrderValues> {

        let mut cumulative_shares = 0;
        let mut cumulative_price  = 0.0;
        let mut price = self.price;
        let mut choice_pot = self.total_pot;


        if shares_to_buy == 0 {
            while cumulative_price + price < buy_price * 0.995 {
            
                market_pot += price;
                choice_pot += price; 
                
                cumulative_shares += 1;
                cumulative_price += price;
    
                price = self.get_new_price(market_pot, choice_pot)?;
                // msg!("price: {}", price)
            }

            let fee_price = cumulative_price * 0.005;
            
            return Ok(BuyOrderValues {buy_price: cumulative_price, fee_price, shares_to_buy: cumulative_shares})
        } 
        if buy_price == 0.0 {
            while cumulative_shares < shares_to_buy {
                market_pot += price;
                choice_pot += price; 
                
                cumulative_shares += 1;
                cumulative_price += price;
    
                price = self.get_new_price(market_pot, choice_pot)?;
                // msg!("price: {}", price)
            }
            let fee_price = cumulative_price * 0.005;

            return Ok(BuyOrderValues{buy_price: cumulative_price, fee_price, shares_to_buy})
        } 

        err!(TallyClobErrors::NotAValidOrder)
    }

    pub fn get_sell_order_values(
        &mut self, 
        mut market_pot: f64, 
        sell_price: f64, 
        shares_to_sell: u64
    ) -> Result<SellOrderValues> {

        let mut cumulative_shares = 0;
        let mut cumulative_price  = 0.0;
        let mut price = self.price;
        let mut choice_pot = self.total_pot;

        if shares_to_sell == 0 {
            while cumulative_price + price < sell_price * 0.995 {
            
                market_pot -= price;
                choice_pot -= price; 

                cumulative_shares += 1;
                cumulative_price += price;
    
                price = self.get_new_price(market_pot, choice_pot)?;
                // msg!("price: {}", price)
            }

            let fee_price = cumulative_price * 0.005;
            
            return Ok(SellOrderValues {sell_price: cumulative_price - fee_price, fee_price, shares_to_sell: cumulative_shares})
        }
        if sell_price == 0.0 {
            while cumulative_shares < shares_to_sell {
                market_pot -= price;
                choice_pot -= price; 

                cumulative_shares += 1;
                cumulative_price += price;

                price = self.get_new_price(market_pot, choice_pot)?;
                // msg!("price: {}", price)
            }

            let fee_price = cumulative_price * 0.005;
            return Ok(SellOrderValues{sell_price: cumulative_price - fee_price, fee_price, shares_to_sell})
        }

        err!(TallyClobErrors::NotAValidOrder)
        
    }

    pub fn get_new_price(&mut self, market_pot: f64, choice_pot: f64) -> Result<f64> {
        if market_pot == 0.0 {return Ok(0.99)}
        let new_price = choice_pot / market_pot;
        if new_price > 0.99 {return Ok(0.99)}
        if new_price < 0.01 {return Ok(0.01)}
        
        Ok(new_price)
    }
   

    pub fn reprice(&mut self, market_pot: f64) -> Result<&mut Self> {
        let new_price = self.get_new_price(market_pot, self.total_pot)?;
        self.price = new_price;
        
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