use anchor_lang::prelude::*;

use crate::{vec_size, ChoicePortfolio, DISCRIMINATOR_SIZE, U64_SIZE};


#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct SubMarketPortfolio {
    pub sub_market_id: u64,
    pub choice_portfolio: Vec<ChoicePortfolio>,
    
}

impl SubMarketPortfolio {

    pub const SIZE: usize = DISCRIMINATOR_SIZE
    + U64_SIZE
    + vec_size(ChoicePortfolio::SIZE, 2);

    pub fn new(id: u64) -> SubMarketPortfolio {
        let choice_portfolio: Vec<ChoicePortfolio> = Vec::new();

        SubMarketPortfolio {
            sub_market_id: id,
            choice_portfolio,
        }
    }
         
    pub fn add_to_portfolio(&mut self, choice_id: &u64, shares: u128) -> Result<&Self> {
        self
            .get_choice_market_portfolio(choice_id)?
            .add_to_portfolio(shares)?;
        

        Ok(self)
    }

    pub fn sell_from_portfolio(&mut self, choice_id: &u64, shares: u128) -> Result<&Self> {
        self
            .get_choice_market_portfolio(choice_id)?
            .withdraw_from_portfolio(shares)?;

        Ok(self)
    }

    pub fn get_choice_shares(&mut self, choice_id: &u64) -> Result<u128> {
        let choice_shares = self
            .get_choice_market_portfolio(choice_id)?
            .shares;

        Ok(choice_shares)
    }


    pub fn get_choice_market_portfolio(&mut self, choice_id: &u64) -> Result<&mut ChoicePortfolio> {
        match self.choice_portfolio.binary_search_by_key(choice_id, |choice_portfolio| choice_portfolio.choice_id) {
            Ok(index) => Ok(&mut self.choice_portfolio[index]),
            Err(_) => {
                let new_choice_portfolio = &mut ChoicePortfolio::new(*choice_id);
                self.choice_portfolio.push(new_choice_portfolio.clone());
                Ok(self.choice_portfolio.last_mut().unwrap())
            },
        }
    }
    
}