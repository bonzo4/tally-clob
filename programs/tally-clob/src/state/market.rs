use anchor_lang::prelude::*;

use crate::{errors::TallyClobErrors, MarketStatus, SubMarket, U8_SIZE};

use super::{vec_size, DISCRIMINATOR_SIZE};

#[account]
pub struct Market {
    pub bump: u8,
    pub sub_markets: Vec<SubMarket>
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, PartialEq)]
pub struct Order {
    pub amount: f64,
    pub sub_market_id: u64,
    pub choice_id: u64,
    pub requested_price: f64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, PartialEq)]
pub struct OrderData {
    pub market_key: Pubkey,
    pub user_key: Pubkey,
}

impl Market {
    pub const MARKET_MAX_LENGTH: usize = 10;

    pub const SIZE: usize = DISCRIMINATOR_SIZE
        + U8_SIZE
        + vec_size(SubMarket::SIZE, Market::MARKET_MAX_LENGTH); //sub_markets

    pub fn get_buying_periods(&mut self, orders: &Vec<Order>) -> Result<Vec<MarketStatus>> {
        let market_periods = orders.iter()
        .map(|order| order.sub_market_id)
        .collect::<Vec<u64>>()
        .iter()
        .map(|sub_market_id| self
                .get_sub_market(sub_market_id)
                .unwrap()
                .get_buying_period()
                .unwrap()
        ).collect();
        
        Ok(market_periods)
    }

    pub fn get_order_prices(&mut self, orders: &Vec<Order>) -> Result<Vec<f64>> {
        let order_prices = orders.iter()
            .map(|order| self.get_sub_market(&order.sub_market_id).unwrap().get_choice(&order.choice_id).unwrap().price)
            .collect::<Vec<f64>>();

        Ok(order_prices)
    }

    pub fn check_selling_periods(&mut self, orders: &Vec<Order>) -> Result<()> {
        orders.iter()
            .map(|order| order.sub_market_id)
            .collect::<Vec<u64>>()
            .iter()
            .for_each(|sub_market_id| {
                self
                    .get_sub_market(sub_market_id)
                    .unwrap()
                    .check_selling_period()
                    .unwrap();
            });
        
        Ok(())
    }

    pub fn bulk_buy_price(&mut self, orders: &Vec<Order>, market_periods: Vec<MarketStatus>) -> Result<Vec<f64>> {
        let order_prices = orders.iter()
            .enumerate()
            .map(|(order_index, order)| {
                if market_periods[order_index] == MarketStatus::FairLaunch {
                    return order.amount * 0.5;
                } 
                let sub_market_id = &order.sub_market_id;
                self.get_sub_market(sub_market_id).unwrap().get_buy_order_price(&order.choice_id, order.amount as u64).unwrap()
                
            })
            .collect();
        
        Ok(order_prices)
    }

    pub fn bulk_sell_price(&mut self, orders: &Vec<Order>) -> Result<Vec<f64>> {
        let order_prices =  orders.iter()
        .map(|order| {
            self.get_sub_market(&order.sub_market_id)
            .unwrap()
            .get_sell_order_price(&order.choice_id, order.amount as u64)
            .unwrap()
        }).collect();
        
        Ok(order_prices)
    }

    pub fn bulk_buy_shares(&mut self, orders: &Vec<Order>, market_periods: Vec<MarketStatus>) -> Result<Vec<u64>> {
        let order_shares = orders.iter()
            .enumerate()
            .map(|(order_index, order)| {
                
                if market_periods[order_index] == MarketStatus::FairLaunch {return order.amount as u64 * 4}
                else {
                    return self
                    .get_sub_market(&order.sub_market_id)
                    .unwrap()
                    .get_buy_order_shares(&order.choice_id, order.amount)
                    .unwrap();
                }
            }).collect();

            Ok(order_shares)
    }

    pub fn bulk_sell_shares(&mut self, orders: &Vec<Order>) -> Result<Vec<u64>> {
        let order_shares = orders.iter()
            .map(|order| {
                self
                    .get_sub_market(&order.sub_market_id)
                    .unwrap()
                    .get_sell_order_shares(&order.choice_id, order.amount)
                    .unwrap()
            }).collect();

        Ok(order_shares)
    }

    pub fn adjust_markets_after_buy(&mut self, orders: &Vec<Order>, order_prices: Vec<f64>) -> Result<()> {
        orders.iter()
        .enumerate()
        .for_each(|(order_index,order)| {
            let order_price = order_prices[order_index];
            self
                .get_sub_market(&order.sub_market_id)
                .unwrap()
                .add_to_pot(order_price)
                .unwrap()
                .add_to_choice_pot(&order.choice_id, order_price)
                .unwrap()
                .reprice_choices()
                .unwrap();
        });
        
        Ok(())
    }


    pub fn get_sub_market(&mut self, sub_market_id: &u64) -> Result<&mut SubMarket> {
        match self.sub_markets.binary_search_by_key(sub_market_id, |sub_market| sub_market.id) {
            Ok(index) => Ok(&mut self.sub_markets[index]),
            Err(_) => err!(TallyClobErrors::SubMarketNotFound),
        }
    }
    
}

