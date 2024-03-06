use anchor_lang::prelude::*;

use crate::{BOOL_SIZE, U128_SIZE, U64_SIZE};

use super::DISCRIMINATOR_SIZE;

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, PartialEq)]
pub struct BuyOrderValues {
    pub shares_to_buy: u128,
    pub buy_price: u128,
    pub fee_price: u128
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, PartialEq)]
pub struct SellOrderValues {
    pub shares_to_sell: u128,
    pub sell_price: u128,
    pub fee_price: u128
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, PartialEq)]
pub struct ChoiceMarket {
    pub id: u64,
    pub usdc_pot: u128,
    pub pot_shares: u128,
    pub minted_shares: u128,
    pub fair_launch_pot: u128,
    pub winning_choice: bool
}


impl ChoiceMarket {

    pub const SIZE: usize = DISCRIMINATOR_SIZE
    + U64_SIZE
    + (U128_SIZE * 4)
    + BOOL_SIZE;

    pub fn new(choice_id: &u64, init_pot: u128) -> Self {
        ChoiceMarket {
            id: *choice_id,
            pot_shares: init_pot,
            usdc_pot: init_pot / 2,
            minted_shares: 0,
            fair_launch_pot: init_pot / 2,
            winning_choice: false
        }
    }
}