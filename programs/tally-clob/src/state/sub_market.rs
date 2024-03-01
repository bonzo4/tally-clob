use std::ops::{AddAssign, SubAssign};

use anchor_lang::prelude::*;
use solana_program::sysvar::fees;

use crate::{errors::TallyClobErrors, utils::{clock, get_buy_price, get_buy_quadratic_roots, get_sell_price}, BuyOrderValues, SellOrderValues};

use super::{vec_size, ChoiceMarket, DISCRIMINATOR_SIZE, U64_SIZE, I64_SIZE, U8_SIZE ,BOOL_SIZE};

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct InitSubMarket {
    pub id: u64,
    pub choice_count: u8,
    pub choice_ids: Vec<u64>,
    pub fair_launch_start: i64,
    pub fair_launch_end: i64,
    pub trading_start: i64,
    pub trading_end: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct SubMarket {
    pub id: u64,
    pub total_pot: u64,
    pub invariant: u64,
    pub choice_count: u8,
    pub choices: [ChoiceMarket; 2],
    pub fair_launch_start: i64,
    pub fair_launch_end: i64,
    pub trading_start: i64,
    pub trading_end: i64,
    pub resolved: bool
}

impl SubMarket {
    pub const CHOICE_MAX_LENGTH: usize = 5;

    pub const SIZE: usize = DISCRIMINATOR_SIZE
        + U64_SIZE // id
        + U64_SIZE // total_pot
        + U8_SIZE // choice_count
        + vec_size(ChoiceMarket::SIZE, 2) // choices
        + (I64_SIZE * 4) // timestamps
        + BOOL_SIZE; //resolved


    pub fn new(init_sub_market: &InitSubMarket) -> Self {
        let choices = init_sub_market.choice_ids.iter()
            .map(|choice_id| ChoiceMarket::new(
                choice_id))
            .collect::<Vec<ChoiceMarket>>();
        SubMarket {
            id: init_sub_market.id,
            total_pot: 0,
            invariant: 0,
            choice_count: init_sub_market.choice_count,
            choices: [
                ChoiceMarket::new(&init_sub_market.choice_ids[0]), 
                ChoiceMarket::new(&init_sub_market.choice_ids[1])
                    ],
            fair_launch_start: init_sub_market.fair_launch_start,
            fair_launch_end: init_sub_market.fair_launch_end,
            trading_start: init_sub_market.trading_start,
            trading_end: init_sub_market.trading_end,
            resolved: false
        }
    }

    pub fn get_market_period(&self) -> Result<MarketStatus> {
        let now = clock::current_timestamp();
        if now < self.fair_launch_start {return Ok(MarketStatus::Initializing)};
        // require!(now > self.fair_launch_start, TallyClobErrors::MarketIntializing);

        let is_fair_launch = now > self.fair_launch_start 
        && now < self.fair_launch_end ;
        if is_fair_launch {return Ok(MarketStatus::FairLaunch)};

        let is_trading_period =  now > self.trading_start 
        && now < self.trading_end;
        if is_trading_period {return Ok(MarketStatus::Trading)};

        Ok(MarketStatus::Closed)
    }

    pub fn fair_launch_order(&mut self, choice_id: &u64, buy_price: u64) -> Result<BuyOrderValues> {
        
    }

    pub fn get_buy_values_by_price(&mut self, choice_id: &u64, buy_price: u64) -> Result<BuyOrderValues> {

        let fee_price = buy_price / 20;

        let new_choices = self.choices
            .iter()
            .map(|choice|ChoiceMarket {
                id: choice.id, 
                pot_shares: choice.pot_shares + buy_price - fee_price,
                minted_shares: choice.minted_shares,    
                winning_choice: choice.winning_choice
            })
            .collect::<Vec<ChoiceMarket>>();

        let choice = match new_choices.binary_search_by_key(choice_id, |choice_market| choice_market.id) {
            Ok(index) => Ok(&mut self.choices[index]),
            Err(_) => err!(TallyClobErrors::ChoiceNotFound),
        }?;

        let total_pot_shares = new_choices.iter()
            .map(|choice|choice.pot_shares)
            .sum::<u64>();
        let old_pot_shares = choice.pot_shares;
        let new_pot_shares = self.invariant / (total_pot_shares - choice.pot_shares);
        let shares_to_buy = old_pot_shares - new_pot_shares;

        Ok(BuyOrderValues {
            fee_price,
            buy_price,
            shares_to_buy
        })
    }

    pub fn get_buy_values_by_shares(&mut self, choice_id: &u64, shares_to_buy: u64) -> Result<BuyOrderValues> {
        let pot_shares = self.choices
            .iter()
            .map(|choice|{if choice.id == *choice_id {choice.pot_shares - shares_to_buy} else {choice.pot_shares}})
            .collect::<Vec<u64>>();

        let buy_price = get_buy_price(pot_shares, self.invariant)?;
        let fee_price = buy_price / 20;

        Ok(BuyOrderValues{
            shares_to_buy,
            buy_price,
            fee_price
        })
    }

    pub fn get_sell_values_by_price(&mut self, choice_id: &u64, sell_price: u64) -> Result<SellOrderValues> {
        let fee_price = sell_price / 20;

        let new_choices = self.choices
            .iter()
            .map(|choice|ChoiceMarket {
                id: choice.id, 
                pot_shares: choice.pot_shares - sell_price - fee_price,
                minted_shares: choice.minted_shares,    
                winning_choice: choice.winning_choice
            })
            .collect::<Vec<ChoiceMarket>>();

        let choice = match new_choices.binary_search_by_key(choice_id, |choice_market| choice_market.id) {
            Ok(index) => Ok(&mut self.choices[index]),
            Err(_) => err!(TallyClobErrors::ChoiceNotFound),
        }?;

        let total_pot_shares = self.choices.iter()
            .map(|choice|choice.pot_shares)
            .sum::<u64>();
        let old_pot_shares = choice.pot_shares;
        let new_pot_shares = self.invariant / (total_pot_shares - choice.pot_shares);
        let shares_to_buy = old_pot_shares - new_pot_shares;

    }

    pub fn get_sell_values_by_shares(&mut self, choice_id: &u64, shares_to_sell: u64) -> Result<SellOrderValues> {
        let pot_shares = self.choices
            .iter()
            .map(|choice|{if choice.id == *choice_id {choice.pot_shares + shares_to_sell} else {choice.pot_shares}})
            .collect::<Vec<u64>>();

        let sell_price = get_sell_price(pot_shares, self.invariant);
        let fee_price = sell_price / 20;

        Ok(SellOrderValues {
            shares_to_sell,
            sell_price,
            fee_price
        })
    }

    // pub fn withdraw_collateral(&mut self, choice_id: &u64, shares_to_buy: u64) -> Result<u64> {
    //     let choice = self.get_choice(choice_id)?;
    //     choice.pot_shares.add_assign(shares_to_buy);
        

    // }


    // pub fn get_buy_values_by_shares(&mut self, choice_id: &u64, shares_to_buy: u64) -> Result<BuyOrderValues> {
    //     let market_pot = self.total_pot;
    //     Ok(self
    //         .get_choice(choice_id)?
    //         .get_buy_order_values(market_pot, None, Some(shares_to_buy))?)
    // }

    // pub fn get_buy_values_by_price(&mut self, choice_id: &u64, buy_price: u64) -> Result<BuyOrderValues> {
    //     let market_pot = self.total_pot;
    //     Ok(self
    //         .get_choice(choice_id)?
    //         .get_buy_order_values(market_pot, Some(buy_price), None)?)
    // }

    // pub fn get_sell_values_by_shares(&mut self, choice_id: &u64, shares_to_sell: u64) -> Result<SellOrderValues> {
    //     let market_pot = self.total_pot;
    //     Ok(self
    //         .get_choice(choice_id)?
    //         .get_sell_order_values(market_pot, None, Some(shares_to_sell))?)
    // }

    // pub fn get_sell_values_by_price(&mut self, choice_id: &u64, sell_price: u64) -> Result<SellOrderValues> {
    //     let market_pot = self.total_pot;
    //     Ok(self
    //         .get_choice(choice_id)?
    //         .get_sell_order_values(market_pot, Some(sell_price), None)?)
    // }

    pub fn add_to_pot(&mut self, amount: u64) -> Result<&mut Self> {
        require!(amount > 0, TallyClobErrors::AmountToAddTooLow);

        self
            .total_pot
            .add_assign(amount);


        Ok(self)
    }

    pub fn add_to_choice_pot(&mut self, choice_id: &u64, amount: u64) -> Result<&mut Self> {
        require!(amount > 0, TallyClobErrors::AmountToAddTooLow);

        self
        .get_choice(choice_id)?
            .add_to_pot(amount)?;

        Ok(self)
    }

    pub fn add_to_choice_shares(&mut self, choice_id: &u64, amount: u64) -> Result<&mut Self> {
        require!(amount > 0, TallyClobErrors::AmountToAddTooLow);

        self.get_choice(choice_id)?
            .add_to_shares(amount)?;

        Ok(self)
    }

    pub fn withdraw_from_pot(&mut self, amount: u64) -> Result<&mut Self> {
        require!(amount > 0, TallyClobErrors::AmountToWithdrawTooLow);
        require!(amount <= self.total_pot, TallyClobErrors::AmountToWithdrawTooGreat);

        self
            .total_pot
            .sub_assign(amount);
        Ok(self)
    }

    pub fn withdraw_from_choice_pot(&mut self, choice_id: &u64, amount: u64) -> Result<&mut Self> {
        require!(amount > 0, TallyClobErrors::AmountToWithdrawTooLow);
        require!(amount <= self.total_pot, TallyClobErrors::AmountToWithdrawTooGreat);

        self
            .get_choice(choice_id)?
            .withdraw_from_pot(amount)?;

        Ok(self)
    }

    pub fn delete_shares_from_choice_pot(&mut self, choice_id: &u64, amount: u64) -> Result<&mut Self> {
        require!(amount > 0, TallyClobErrors::AmountToWithdrawTooLow);

        self
            .get_choice(choice_id)?
            .delete_shares(amount)?;

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
