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
    pub shares: f64,
    pub fee_price: f64
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


    pub fn bulk_buy_values_by_price(
        &mut self,
        orders: &Vec<Order>,
    ) -> Result<Vec<BuyOrderValues>> {
        let order_values = orders.iter()
            .map(|order| self
                    .get_sub_market(&order.sub_market_id)
                    .unwrap()
                    .get_buy_values_by_price(&order.choice_id, order.amount)
                    .unwrap()
            ).collect::<Vec<BuyOrderValues>>();

        Ok(order_values)
    }

    pub fn bulk_buy_values_by_shares(
        &mut self,
        orders: &Vec<Order>,
    ) -> Result<Vec<BuyOrderValues>> {
        let order_values = orders.iter()
            .map(|order| 
            self
                .get_sub_market(&order.sub_market_id)
                .unwrap()
                .get_buy_values_by_shares(&order.choice_id, order.amount)
                .unwrap()
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
            .get_sell_values_by_shares(&order.choice_id, order.amount)
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
                .adjust_markets_after_buy(&order)
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
                .adjust_markets_after_sell(&order)
                .unwrap()
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

