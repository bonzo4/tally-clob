use anchor_lang::prelude::*;

use crate::{BOOL_SIZE, F64_SIZE, U64_SIZE};

use super::DISCRIMINATOR_SIZE;

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, PartialEq)]
pub struct BuyOrderValues {
    pub shares_to_buy: f64,
    pub buy_price: f64,
    pub fee_price: f64
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, PartialEq)]
pub struct SellOrderValues {
    pub shares_to_sell: f64,
    pub sell_price: f64,
    pub fee_price: f64
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, PartialEq)]
pub struct ChoiceMarket {
    pub id: u64,
    pub usdc_pot: f64,
    pub pot_shares: f64,
    pub minted_shares: f64,
    pub fair_launch_pot: f64,
    pub winning_choice: bool
}


impl ChoiceMarket {

    pub const SIZE: usize = DISCRIMINATOR_SIZE
    + U64_SIZE
    + (F64_SIZE * 4)
    + BOOL_SIZE;

    pub fn new(choice_id: &u64) -> Self {
        ChoiceMarket {
            id: *choice_id,
            pot_shares: 100.0,
            usdc_pot: 50.0,
            minted_shares: 0.0,
            fair_launch_pot: 50.0,
            winning_choice: false
        }
    }
}