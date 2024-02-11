use std::ops::{AddAssign, SubAssign};

use anchor_lang::prelude::*;

use crate::{errors::TallyClobErrors, utils::clock, BOOL_SIZE, U64_SIZE};

use super::{vec_size, ChoiceMarket, DISCRIMINATOR_SIZE, F64_SIZE, I64_SIZE, U8_SIZE };

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct SubMarket {
    pub id: u64,
    pub total_pot: f64,
    pub choice_count: u8,
    pub choices: Vec<ChoiceMarket>,
    pub fair_launch_start: i64,
    pub fair_launch_end: i64,
    pub trading_start: i64,
    pub trading_end: i64,
    pub resolved: bool
}

impl SubMarket {
    pub const TITLE_MAX_LENGTH: usize = 100;
    pub const CHOICE_MAX_LENGTH: usize = 5;

    pub const SIZE: usize = DISCRIMINATOR_SIZE
        + U64_SIZE //
        + F64_SIZE // total_pot
        + U8_SIZE // choice_count
        + vec_size(ChoiceMarket::SIZE, SubMarket::CHOICE_MAX_LENGTH) // choices
        + (I64_SIZE * 4) // timestamps
        + BOOL_SIZE; //resolved



    pub fn get_buying_period(&self) -> Result<MarketStatus> {
        let now = clock::current_timestamp();
        require!(now > self.fair_launch_start, TallyClobErrors::MarketIntializing);

        let is_fair_launch = now > self.fair_launch_start 
        && now < self.fair_launch_end ;
        if is_fair_launch {return Ok(MarketStatus::FairLaunch)};

        let is_trading_period =  now > self.trading_start 
        && now < self.trading_end;
        if is_trading_period {return Ok(MarketStatus::Trading)};

        err!(TallyClobErrors::MarketClosed)
    }

    pub fn check_selling_period(&self) -> Result<MarketStatus> {
        let now = clock::current_timestamp();
        require!(now > self.fair_launch_start, TallyClobErrors::MarketIntializing);

        let is_fair_launch = now > self.fair_launch_start 
        && now < self.fair_launch_end ;
        require!(!is_fair_launch, TallyClobErrors::NotSellingPeriod);

        let is_trading_period =  now > self.trading_start 
        && now < self.trading_end;
        if is_trading_period {return Ok(MarketStatus::Trading)};

        err!(TallyClobErrors::MarketClosed)
    }

    pub fn get_buy_order_price(&mut self, choice_id: &u64, shares_to_buy: u64) -> Result<f64> {
        let total_pot = self.total_pot;
        let order_price = self
            .get_choice(choice_id)?
            .get_buy_order_price(total_pot, shares_to_buy)?;

        Ok(order_price)
    }

    pub fn get_sell_order_price(&mut self, choice_id: &u64, shares_to_sell: u64) -> Result<f64> {
        let total_pot = self.total_pot;
        let order_price = self
            .get_choice(choice_id)?
            .get_sell_order_price(total_pot, shares_to_sell)?;

        Ok(order_price)
    }

    pub fn get_buy_order_shares(&mut self, choice_id: &u64, buy_price: f64) -> Result<u64> {
        let total_pot = self.total_pot;
        let order_shares = self
            .get_choice(choice_id)?
            .get_buy_order_shares(total_pot, buy_price)?;

        Ok(order_shares)
    }

    pub fn get_sell_order_shares(&mut self, choice_id: &u64, sell_price: f64) -> Result<u64> {
        let total_pot = self.total_pot;
        let order_shares = self
            .get_choice(choice_id)?
            .get_sell_order_shares(total_pot, sell_price)?;

        Ok(order_shares)
    }

    pub fn reprice_choices(&mut self) -> Result<&mut Self> {

        for choice in self.choices.iter_mut() {
            choice.reprice(self.total_pot)?;
        }
        
       Ok(self)
    }

    pub fn add_to_pot(&mut self, amount: f64) -> Result<&mut Self> {
        require!(amount > 0.0, TallyClobErrors::AmountToAddTooLow);

        self
            .total_pot
            .add_assign(amount);


        Ok(self)
    }

    pub fn add_to_choice_pot(&mut self, choice_id: &u64, amount: f64) -> Result<&mut Self> {
        require!(amount > 0.0, TallyClobErrors::AmountToAddTooLow);

        self
        .get_choice(choice_id)?
            .add_to_pot(amount)?;

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

    pub fn withdraw_from_choice_pot(&mut self, choice_index: usize, amount: f64) -> Result<&mut Self> {
        require!(amount > 0.0, TallyClobErrors::AmountToWithdrawTooLow);
        require!(amount <= self.total_pot, TallyClobErrors::AmountToWithdrawTooGreat);

        self.choices
            .get_mut(choice_index)
            .ok_or(TallyClobErrors::ChoiceNotFound)?
            .withdraw_from_pot(amount)?;

        Ok(self)
    }


    pub fn get_choice(&mut self, choice_id: &u64) -> Result<&mut ChoiceMarket> {
        match self.choices.binary_search_by_key(choice_id, |choice_market| choice_market.id) {
            Ok(index) => Ok(&mut self.choices[index]),
            Err(_) => err!(TallyClobErrors::ChoiceNotFound),
        }
    }

}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, PartialEq, Eq)]
pub enum MarketStatus {
    Initializing,
    FairLaunch,
    Trading,
    Closed
}
