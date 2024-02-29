use std::ops::{AddAssign, SubAssign};

use anchor_lang::prelude::*;

use crate::{errors::TallyClobErrors, sub_market, SubMarket, BOOL_SIZE, U64_SIZE};

use super::DISCRIMINATOR_SIZE;

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, PartialEq)]
pub struct BuyOrderValues {
    pub shares_to_buy: u64,
    pub buy_price: u64,
    pub fee_price: u64
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, PartialEq)]
pub struct SellOrderValues {
    pub shares_to_sell: u64,
    pub sell_price: u64,
    pub fee_price: u64
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, PartialEq)]
pub struct ChoiceMarket {
    pub id: u64,
    pub pot_shares: u64,
    pub minted_shares: u64,
    pub winning_choice: bool
}


impl ChoiceMarket {

    pub const SIZE: usize = DISCRIMINATOR_SIZE
    + (U64_SIZE * 3)
    + BOOL_SIZE;

    pub fn new(choice_id: &u64, choice_count: u8) -> Self {
        ChoiceMarket {
            id: *choice_id,
            pot_shares: 0,
            minted_shares: 0,
            winning_choice: false
        }
    }

    pub fn get_buy_order_values(
        &mut self, 
        k: u64,
        buy_price: Option<u64>, 
        shares_to_buy: Option<u64>
    ) -> Result<BuyOrderValues> {
    
        require!(!(buy_price.is_none() && shares_to_buy.is_none()), TallyClobErrors::NotAValidOrder);
    
        let mut cumulative_shares = 0;
        let mut cumulative_price = 0;
        let mut choice_pot = self.total_pot;
        let mut price = self.get_price(market_pot, choice_pot)?;

        let buy_price_threshold = buy_price.map(|bp| bp * 995 / 1000); 
    
        if let Some(buy_price_threshold) = buy_price_threshold {
            
            while cumulative_price + price < buy_price_threshold {
                market_pot += price;
                choice_pot += price;
                
                cumulative_shares += 1;
                cumulative_price += price;
    
                price = self.get_price(choice_pot, market_pot)?;
            }
            let fee_price = cumulative_price / 200; // 0.005 as fixed-point
    
            return Ok(BuyOrderValues { buy_price: cumulative_price, fee_price, shares_to_buy: cumulative_shares });
        } 
    
        if let Some(shares_to_buy) = shares_to_buy {
            while cumulative_shares < shares_to_buy {
                market_pot += price;
                choice_pot += price; 
                
                cumulative_shares += 1;
                cumulative_price += price;
    
                price = self.get_price(choice_pot, market_pot)?;
            }
            let fee_price = cumulative_price / 200; // 0.005 as fixed-point
    
            return Ok(BuyOrderValues { buy_price: cumulative_price, fee_price, shares_to_buy });
        }
    
        err!(TallyClobErrors::NotAValidOrder)
    }
    

    pub fn get_sell_order_values(
        &mut self, 
        mut market_pot: u64, 
        sell_price: Option<u64>, 
        shares_to_sell: Option<u64>
    ) -> Result<SellOrderValues> {
        require!(!(sell_price.is_none() && shares_to_sell.is_none()), TallyClobErrors::NotAValidOrder);
    
        let mut cumulative_shares = 0;
        let mut cumulative_price = 0;
        let mut choice_pot = self.total_pot;
        let mut price = self.get_price(market_pot, choice_pot)?;
    
        let sell_price_threshold = sell_price.map(|sp| sp * 1005 / 1000);
    
        if let Some(sell_price_threshold) = sell_price_threshold {
            while cumulative_price + price < sell_price_threshold {
                market_pot -= price;
                choice_pot -= price;
                
                cumulative_shares += 1;
                cumulative_price += price;
    
                price = self.get_price(choice_pot, market_pot)?;
            }
            let fee_price = cumulative_price / 200; // 0.005 as fixed-point
    
            return Ok(SellOrderValues { sell_price: cumulative_price, fee_price, shares_to_sell: cumulative_shares });
        } 
    
        if let Some(shares_to_sell) = shares_to_sell {
            while cumulative_shares < shares_to_sell {
                market_pot -= price;
                choice_pot -= price; 
                
                cumulative_shares += 1;
                cumulative_price += price;
    
                price = self.get_price(choice_pot, market_pot)?;
            }
            let fee_price = cumulative_price / 200; // 0.005 as fixed-point
    
            return Ok(SellOrderValues { sell_price: cumulative_price, fee_price, shares_to_sell });
        }
    
        err!(TallyClobErrors::NotAValidOrder)
    }
    
    

    pub fn get_price(&mut self, market_pot: u64, choice_pot: u64) -> Result<u64> {
        if market_pot == 0 {
            return Ok(1_000_000); // 1 USDC in fixed-point representation
        }
    
        // Calculate new price using fixed-point arithmetic without additional scaling
        let price = choice_pot / market_pot;
    
        Ok(price)
    }
    
    pub fn add_to_pot(&mut self, amount: u64) -> Result<&mut Self> {

        require!(amount > 0, TallyClobErrors::AmountToAddTooLow);

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

    pub fn withdraw_from_pot(&mut self, amount: u64) -> Result<&mut Self>  {
        require!(amount > 0, TallyClobErrors::AmountToWithdrawTooLow);
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