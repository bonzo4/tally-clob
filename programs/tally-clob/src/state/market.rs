use std::ops::{AddAssign, SubAssign};

use anchor_lang::prelude::*;

use crate::errors::TallyClobErrors;

use super::{vec_size, ChoiceMarket, CHAR_SIZE, DISCRIMINATOR_SIZE, ENUM_SIZE, F64_SIZE, I64_SIZE, U8_SIZE };

#[account]
pub struct Market {
    pub bump: u8,
    pub title: String,
    pub total_pot: f64,
    pub choice_count: u8,
    pub choices: Vec<ChoiceMarket>,
    pub market_status: MarketStatus,
    pub start_time: i64,
    pub fair_launch_start: i64,
    pub fair_launch_end: i64,
    pub trading_start: i64,
    pub trading_end: i64,
}

impl Market {
    pub const TITLE_MAX_LENGTH: usize = 100;
    pub const CHOICE_MAX_LENGTH: usize = 5;

    pub const SIZE: usize = DISCRIMINATOR_SIZE
    + U8_SIZE // bump
    + vec_size(CHAR_SIZE, Market::TITLE_MAX_LENGTH) // title
    + F64_SIZE // total_pot
    + U8_SIZE // choice_count
    + vec_size(ChoiceMarket::SIZE, Market::CHOICE_MAX_LENGTH) // choices
    + ENUM_SIZE // status
    + (I64_SIZE * 5); // timestamps

    pub fn buy_order_by_shares(&mut self, choice_index: usize, amount: f64) -> Result<f64> {
        
        let order_price = self.
            get_choice_price_by_shares(choice_index, amount)?;

    
        self
            .add_to_pot(order_price)?
            .reprice_choices()?;

        Ok(order_price)
    }

    pub fn get_choice_price_by_shares(&mut self, choice_index: usize, amount: f64) -> Result<f64> {
        let order_price = self.choices
            .get_mut(choice_index)
            .ok_or(TallyClobErrors::ChoiceNotFound)
            .unwrap()
            .get_buy_price_by_shares(self.total_pot, amount)?;

        Ok(order_price)

    }

    pub fn sell_order_by_shares(&mut self, choice_index: usize, amount: f64) -> Result<f64> {
        
        let order_price = self.choices
            .get_mut(choice_index)
            .ok_or(TallyClobErrors::ChoiceNotFound)
            .unwrap()
            .sell_order_by_shares(
                self.total_pot, 
                amount
            ).unwrap();

    
        let _ = self.add_to_pot(order_price);
        let _ = self.reprice_choices();

        Ok(order_price)
    }


    pub fn reprice_choices(&mut self) -> Result<&mut Self> {

        for choice in self.choices.iter_mut() {
            choice.reprice(self.total_pot)?;
        }
        
       Ok(self)
    }

    pub fn get_new_price(&self, choice: &ChoiceMarket) -> Result<f64> {
        let new_price = choice.total_pot / self.total_pot;

        Ok(new_price)
    }

    pub fn add_to_pot(&mut self, amount: f64) -> Result<&mut Self> {
        require!(amount > 0.0, TallyClobErrors::AmountToAddTooLow);

        self
            .total_pot
            .add_assign(amount);


        Ok(self)
    }

    pub fn withdraw_from_pot(&mut self, amount: f64) -> Result<&mut Self> {
        require!(amount > 0.0, TallyClobErrors::AmountToWithdrawTooLow);
        require!(amount <= self.total_pot, TallyClobErrors::AmountToWithdrawTooGreat);

        self
            .total_pot
            .sub_assign(amount);
        Ok(self)
    }
}


#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, PartialEq, Eq)]
pub enum MarketStatus {
    Intializing,
    FairLaunch,
    Settlement,
    Trading,
    Resolved,
    Closed
}