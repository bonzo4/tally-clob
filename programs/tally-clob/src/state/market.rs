use anchor_lang::prelude::*;

use crate::{errors::TallyClobErrors, BuyOrderValues, MarketStatus, SellOrderValues, SubMarket, U8_SIZE};

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
    pub requested_price_per_share: f64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, PartialEq)]
pub struct FinalOrder {
    pub sub_market_id: u64,
    pub choice_id: u64,
    pub price: f64,
    pub shares: u64
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
                .get_market_period()
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

    pub fn bulk_buy_values_by_price(
        &mut self,
        orders: &Vec<Order>,
        market_periods: &Vec<MarketStatus>
    ) -> Result<Vec<BuyOrderValues>> {
        let order_values = orders.iter()
            .enumerate()
            .map(|(index, order)| {
                let sub_market = self
                    .get_sub_market(&order.sub_market_id)
                    .unwrap();
                if market_periods[index] == MarketStatus::FairLaunch {
                    let share_price = self
                        .get_sub_market_default_price(&order.sub_market_id).unwrap();
                    let total_shares = order.amount  / share_price;
                    return BuyOrderValues {shares_to_buy: total_shares as u64, buy_price: order.amount, fee_price: 0.0}
                }
                let order_values = sub_market
                    .get_buy_values_by_price(&order.choice_id, order.amount)
                    .unwrap();
                    
                order_values
                }
            ).collect::<Vec<BuyOrderValues>>();

        Ok(order_values)
    }

    pub fn bulk_buy_values_by_shares(
        &mut self,
        orders: &Vec<Order>,
        market_periods: &Vec<MarketStatus>
    ) -> Result<Vec<BuyOrderValues>> {
        let order_values = orders.iter()
            .enumerate()
            .map(|(index, order)| {
            let sub_market = self
                .get_sub_market(&order.sub_market_id)
                .unwrap();
            if market_periods[index] == MarketStatus::FairLaunch {
                let share_price = self
                    .get_sub_market_default_price(&order.sub_market_id).unwrap();
                let total_price = order.amount * share_price;
                return BuyOrderValues {
                    shares_to_buy: order.amount as u64, 
                    buy_price: total_price, 
                    fee_price: 0.0
                }
            }
            let order_values = sub_market
                .get_buy_values_by_shares(&order.choice_id, order.amount as u64)
                .unwrap();
                
            order_values
            }
        ).collect::<Vec<BuyOrderValues>>();

    Ok(order_values)
    }

    pub fn bulk_sell_values_by_price(
        &mut self,
        orders: &Vec<Order>
    ) -> Result<Vec<SellOrderValues>> {
        let order_values = orders.iter()
            .map(|order| self
                .get_sub_market(&order.sub_market_id)
                .unwrap()
                .get_sell_values_by_price(&order.choice_id, order.amount)
                .unwrap()
            ).collect::<Vec<SellOrderValues>>();

        Ok(order_values)
    }

    pub fn bulk_sell_values_by_shares(
        &mut self,
        orders: &Vec<Order>
    ) -> Result<Vec<SellOrderValues>> {
        let order_values = orders.iter()
        .map(|order| self
            .get_sub_market(&order.sub_market_id)
            .unwrap()
            .get_sell_values_by_shares(&order.choice_id, order.amount as u64)
            .unwrap()
        ).collect::<Vec<SellOrderValues>>();

    Ok(order_values)
    }

    pub fn adjust_markets_after_buy(&mut self, final_orders: &Vec<FinalOrder>,) -> Result<()> {
        final_orders.iter()
        .for_each(|order| {
            self
                .get_sub_market(&order.sub_market_id)
                .unwrap()
                .add_to_pot(order.price)
                .unwrap()
                .add_to_choice_pot(&order.choice_id, order.price)
                .unwrap()
                .add_to_choice_shares(&order.choice_id, order.shares)
                .unwrap()
                .reprice_choices()
                .unwrap();
        });
        
        Ok(())
    }

    pub fn adjust_markets_after_sell(&mut self, final_orders: &Vec<FinalOrder>) -> Result<()> {
        final_orders.iter()
        .for_each(|order| {
            self
                .get_sub_market(&order.sub_market_id)
                .unwrap()
                .withdraw_from_pot(order.price)
                .unwrap()
                .withdraw_from_choice_pot(&order.choice_id, order.price)
                .unwrap()
                .delete_shares_from_choice_pot(&order.choice_id, order.shares)
                .unwrap()
                .reprice_choices()
                .unwrap();
        });
        
        Ok(())
    }

    pub fn get_sub_market_default_price(&mut self, sub_market_id: &u64) -> Result<f64> {
         Ok(1.0 / (self.get_sub_market(sub_market_id)?.choice_count as f64))
    }


    pub fn get_sub_market(&mut self, sub_market_id: &u64) -> Result<&mut SubMarket> {
        match self.sub_markets.binary_search_by_key(sub_market_id, |sub_market| sub_market.id) {
            Ok(index) => Ok(&mut self.sub_markets[index]),
            Err(_) => err!(TallyClobErrors::SubMarketNotFound),
        }
    }
    
}

