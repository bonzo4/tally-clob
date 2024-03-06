use anchor_lang::prelude::*;

use crate::{errors::TallyClobErrors, utils::{clock, get_buy_price, get_sell_price}, BuyOrderValues, FinalOrder, SellOrderValues, U128_SIZE};

use super::{vec_size, ChoiceMarket, DISCRIMINATOR_SIZE, U64_SIZE, I64_SIZE ,BOOL_SIZE};

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct InitSubMarket {
    pub id: u64,
    pub choice_ids: Vec<u64>,
    pub fair_launch_start: i64,
    pub fair_launch_end: i64,
    pub trading_start: i64,
    pub trading_end: i64,
    pub init_pot: u128
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct SubMarket {
    pub id: u64,
    pub invariant: u128,
    pub choices: Vec<ChoiceMarket>,
    pub fair_launch_start: i64,
    pub fair_launch_end: i64,
    pub trading_start: i64,
    pub trading_end: i64,
    pub resolved: bool
}

impl SubMarket {

    pub const SIZE: usize = DISCRIMINATOR_SIZE
        + U64_SIZE // id
        + U128_SIZE // invaraint
        + vec_size(ChoiceMarket::SIZE, 2) // choices
        + (I64_SIZE * 4) // timestamps
        + BOOL_SIZE; //resolved


    pub fn new(init_sub_market: &InitSubMarket) -> Self {
        let choices = init_sub_market.choice_ids.iter()
            .map(|choice_id| ChoiceMarket::new(
                choice_id, init_sub_market.init_pot))
            .collect::<Vec<ChoiceMarket>>();
        SubMarket {
            id: init_sub_market.id,
            invariant: init_sub_market.init_pot.pow(2),
            choices,
            fair_launch_start: init_sub_market.fair_launch_start,
            fair_launch_end: init_sub_market.fair_launch_end,
            trading_start: init_sub_market.trading_start,
            trading_end: init_sub_market.trading_end,
            resolved: false
        }
    }

    pub fn get_market_period(&self) -> Result<MarketStatus> {
        if self.resolved {return Ok(MarketStatus::Closed)}
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

    pub fn calculate_shares_to_buy(&mut self, mut choices: Vec<ChoiceMarket>, choice_id: &u64, price: u128,) -> Result<u128> {
        let choice = match choices.binary_search_by_key(choice_id, |choice_market| choice_market.id) {
            Ok(index) => Ok(&mut self.choices[index]),
            Err(_) => err!(TallyClobErrors::ChoiceNotFound),
        }?;
        
        let losing_choices = choices.iter_mut()
            .filter(|choice| &choice.id != choice_id)
            .collect::<Vec<&mut ChoiceMarket>>();

        
        let old_pot_shares = choice.pot_shares + price;
        let invariant = self.invariant;
        let new_pot_shares = invariant / losing_choices[0].pot_shares;
        msg!(&old_pot_shares.to_string());
        msg!(&new_pot_shares.to_string());
        Ok(old_pot_shares - new_pot_shares)
    }

    pub fn get_buy_values_by_price(&mut self, choice_id: &u64, buy_price: u128) -> Result<BuyOrderValues> {

        let fee_price = buy_price / 200;
        let new_buy_price = buy_price - fee_price;

        // yes: 104.975, no: 104.975
        let new_choices = self.choices
            .iter()
            .map(|choice|ChoiceMarket {
                pot_shares: choice.pot_shares + new_buy_price,
                ..choice.clone()
            })
            .collect::<Vec<ChoiceMarket>>();

        let shares_to_buy = self.calculate_shares_to_buy(new_choices, choice_id, new_buy_price)?;

        Ok(BuyOrderValues {
            fee_price,
            buy_price: new_buy_price,
            shares_to_buy
        })
    }

    pub fn get_buy_values_by_shares(&self, choice_id: &u64, shares_to_buy: u128) -> Result<BuyOrderValues> {
        let pot_shares = self.choices
            .iter()
            .map(|choice|{
                if choice.id == *choice_id {
                    choice.pot_shares - shares_to_buy
                } else {
                    choice.pot_shares
                }
            })
            .collect::<Vec<u128>>();


        let buy_price = get_buy_price(pot_shares, self.invariant)?;
        let fee_price = buy_price / 200; 

        Ok(BuyOrderValues{
            shares_to_buy,
            buy_price,
            fee_price
        })
    }

    pub fn calculate_shares_to_sell(&mut self, choices: Vec<ChoiceMarket>, choice_id: &u64) -> Result<u128> {
        let choice = match choices.binary_search_by_key(choice_id, |choice_market| choice_market.id) {
            Ok(index) => Ok(&mut self.choices[index]),
            Err(_) => err!(TallyClobErrors::ChoiceNotFound),
        }?;

        let total_pot_shares = choices.iter()
            .map(|choice|choice.pot_shares)
            .sum::<u128>();
        let old_pot_shares = choice.pot_shares;
        let invariant = self.invariant;
        let new_pot_shares = invariant / (total_pot_shares  - choice.pot_shares);
        Ok(new_pot_shares - old_pot_shares)
     }

    pub fn get_sell_values_by_price(&mut self, choice_id: &u64, sell_price: u128) -> Result<SellOrderValues> {
        let fee_price = sell_price / 200;

        let new_choices = self.choices
            .iter()
            .map(|choice|ChoiceMarket {
                pot_shares: choice.pot_shares - sell_price,
                ..choice.clone()
            })
            .collect::<Vec<ChoiceMarket>>();

        let shares_to_sell = self.calculate_shares_to_sell(new_choices, choice_id)?;

        Ok(SellOrderValues {
            shares_to_sell,
            sell_price: sell_price - fee_price,
            fee_price
        })

    }

    pub fn get_sell_values_by_shares(&self, choice_id: &u64, shares_to_sell: u128) -> Result<SellOrderValues> {
        let pot_shares = self.choices
            .iter()
            .map(|choice|{if choice.id == *choice_id {choice.pot_shares + shares_to_sell} else {choice.pot_shares}})
            .collect::<Vec<u128>>();

        let sell_price = get_sell_price(pot_shares, self.invariant)?;
        msg!(&sell_price.to_string());
        let fee_price = sell_price / 200;

        // if sell_price < 5.0 {
        //     let difference = 5.0 - sell_price;
        //     let reduction_factor = 0.1 * difference;
        //     sell_price = sell_price * (1.0 - reduction_factor);
        // }

        Ok(SellOrderValues {
            shares_to_sell,
            sell_price: sell_price - fee_price,
            fee_price
        })
    }


    pub fn adjust_markets_after_buy(&mut self, final_order: &FinalOrder) -> Result<()> {
        
        let new_choices = self.choices
            .iter()
            .map(| choice|{
                msg!(&choice.pot_shares.to_string());
                ChoiceMarket {
                pot_shares: choice.pot_shares + final_order.price,
                ..choice.clone()
                }
            }
            )
            .collect::<Vec<ChoiceMarket>>();

        let shares_to_buy = self.calculate_shares_to_buy(new_choices, &final_order.choice_id, final_order.price)?;
        msg!(&shares_to_buy.to_string());

        require!(shares_to_buy == final_order.shares, TallyClobErrors::SharesNotEqual);
        

        self.choices
            .iter_mut()
            .for_each(|choice|{
                choice.pot_shares += final_order.price;
            });

        self.get_choice(&final_order.choice_id)?.usdc_pot += final_order.price;
        self.get_choice(&final_order.choice_id)?.pot_shares -= shares_to_buy;
        self.get_choice(&final_order.choice_id)?.minted_shares += shares_to_buy;

        Ok(())
    }

    pub fn adjust_markets_after_sell(&mut self, final_order: &FinalOrder) -> Result<()> {
        
        // let new_choices = self.choices
        //     .iter()
        //     .map(|choice|ChoiceMarket {
        //         pot_shares: choice.pot_shares - final_order.price + final_order.fee_price,
        //         ..choice.clone()
        //     })
        //     .collect::<Vec<ChoiceMarket>>();

        // let shares_to_sell = self.calculate_shares_to_sell(new_choices, &final_order.choice_id)?;

        // require!(shares_to_sell == final_order.shares, TallyClobErrors::SharesNotEqual);

        self.choices
            .iter_mut()
            .for_each(|choice|{
                choice.pot_shares -= final_order.price + final_order.fee_price;
            });

        self.get_choice(&final_order.choice_id)?.usdc_pot -= final_order.price + final_order.fee_price;
        self.get_choice(&final_order.choice_id)?.pot_shares += final_order.shares;
        self.get_choice(&final_order.choice_id)?.minted_shares -= final_order.shares;

        Ok(())
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
