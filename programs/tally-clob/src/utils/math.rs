use anchor_lang::prelude::*;

use crate::errors::TallyClobErrors;

pub fn get_buy_price(pot_shares: Vec<u64>, invariant: u64) -> Result<u64> {
    require!(pot_shares.len() == 2, TallyClobErrors::NotAValidOrder);

    let a = 1.0; // Since the quadratic term is x^2.
    let b = (pot_shares[0] + pot_shares[1]) as f64;
    let c = (pot_shares[0] * pot_shares[1] - invariant) as f64;

    let discriminant = b.powi(2) - 4.0 * a * c;
    require!(discriminant < 0.0, TallyClobErrors::NotAValidOrder);

    let root1 = ((-b + discriminant.sqrt()) / (2.0 * a)) as u64;
    let root2 = ((-b - discriminant.sqrt()) / (2.0 * a)) as u64;
    
    if root1 > 0 && pot_shares[0] - root1 > 0 {
        return Ok(root1)
    }
    if root2 > 0 && pot_shares[1] - root1 > 0 {
        return Ok(root2)
    }

    err!(TallyClobErrors::NotAValidOrder)
}

pub fn get_sell_price(pot_shares: Vec<u64>, invariant: u64) -> Result<u64> {
    require!(pot_shares.len() == 2, TallyClobErrors::NotAValidOrder);

    let a = 1.0; // Since the quadratic term is x^2.
    // Negate the sum of pot_shares for the `b` coefficient.
    let b = -(pot_shares[0] as i64 + pot_shares[1] as i64) as f64;
    // Add invariant for the `c` coefficient.
    let c = (pot_shares[0] * pot_shares[1] + invariant) as f64;

    let discriminant = b.powi(2) - 4.0 * a * c;
    // Check for non-negative discriminant for real roots.
    require!(discriminant >= 0.0, TallyClobErrors::NotAValidOrder);

    let root1 = ((-b + discriminant.sqrt()) / (2.0 * a)) as u64;
    let root2 = ((-b - discriminant.sqrt()) / (2.0 * a)) as u64;
    if root1 > 0 && pot_shares[0] - root1 > 0 {
        return Ok(root1)
    }
    if root2 > 0 && pot_shares[1] - root1 > 0 {
        return Ok(root2)
    }

    err!(TallyClobErrors::NotAValidOrder)
}