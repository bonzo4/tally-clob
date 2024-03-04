use anchor_lang::prelude::*;

use roots::Roots;
use roots::find_roots_quadratic;

use crate::{errors::TallyClobErrors, ChoiceMarket};

pub fn get_buy_price_by_shares(choices: Vec<ChoiceMarket>, choice_id: &u64, invariant: f64) -> Result<f64> {
    let choice_index = choices.binary_search_by_key(choice_id, |choice_market| choice_market.id)
        .map_err(|_| TallyClobErrors::ChoiceNotFound)?;

    // Find the index of the other choice without borrowing `choices` as mutable or immutable
    let other_choice_index = choices.iter().enumerate()
        .find(|&(idx, choice)| choice.id != *choice_id && idx != choice_index)
        .map(|(idx, _)| idx)
        .ok_or(TallyClobErrors::ChoiceNotFound)?;

    // Now, access both `choice` and `other_choice` using their indices
    let choice = &choices[choice_index];
    let other_choice = &choices[other_choice_index]; // No need for mutability if we're only reading

    // Calculate buy_price based on `choice` and `other_choice`
    let buy_price = -((invariant / choice.pot_shares) - other_choice.pot_shares);

    Ok(buy_price)
}


pub fn get_buy_price(pot_shares: Vec<f64>, invariant: f64) -> Result<f64> {

    msg!(&invariant.to_string());
    require!(pot_shares.len() == 2, TallyClobErrors::NotAValidOrder);

    let a: f64 = 1.0; // The coefficient of x^2 in the quadratic equation is always 1.
    let b: f64 = pot_shares[0] + pot_shares[1]; // Sum of pot_shares, with a negative sign for the coefficient of x.
    let c: f64 = pot_shares[0] * pot_shares[1] - invariant; // Product of pot_shares minus the invariant for the constant term.

    let mut root1 = 0.0;
    let mut root2 = 0.0;

    match find_roots_quadratic(a, b, c) {
        Roots::Two(roots) => {
            root1 = roots[0];
            root2 = roots[1];
        },
        _ => {}
    }
    if root1 > 0.0 {
        return Ok(root1);
    }
    if root2 > 0.0 {
        return Ok(root2);
    }

    err!(TallyClobErrors::NotAValidOrder)
}

pub fn get_sell_price(pot_shares: Vec<f64>, invariant: f64) -> Result<f64> {
    msg!(&pot_shares[0].to_string());
    msg!(&pot_shares[1].to_string());   
    msg!(&invariant.to_string());
    // (1-x)(2-x) = 3
    // 1 * 2 - 3 = -1
    let a: f64 = 1.0; // Since we're working directly with f64, no scaling is needed.
    let b: f64 = -(pot_shares[0] + pot_shares[1]); // Sum of pot_shares.
    let c: f64 = pot_shares[0] * pot_shares[1] - invariant; // Product of pot_shares minus the invariant.

    let mut root1 = 0.0;
    let mut root2 = 0.0;

    match find_roots_quadratic(a, b, c) {
        Roots::Two(roots) => {
            root1 = roots[0];
            root2 = roots[1];
        },
        _ => {}
    }
    
    if root1 > 0.0 && pot_shares[0] - root1 > 0.0 {
        return Ok(root1);
    }

    if root2 > 0.0 && pot_shares[1] - root2 > 0.0{
        return Ok(root2);
    }

    err!(TallyClobErrors::NotAValidOrder) // Return an error if neither root is valid.
}