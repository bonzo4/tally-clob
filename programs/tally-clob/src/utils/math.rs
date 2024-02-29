use anchor_lang::prelude::*;

use crate::errors::TallyClobErrors;

use nrfind::find_root;

pub fn get_buy_quadratic_roots(pot_shares: Vec<u64>, invariant: u64) -> Result<(i64, i64)> {
    require!(pot_shares.len() == 2, TallyClobErrors::NotAValidOrder);

    let a = 1.0; // Since the quadratic term is x^2.
    let b = (pot_shares[0] + pot_shares[1]) as f64;
    let c = (pot_shares[0] * pot_shares[1] - invariant) as f64;

    let discriminant = b.powi(2) - 4.0 * a * c;
    require!(discriminant < 0.0, TallyClobErrors::NotAValidOrder);

    let root1 = (-b + discriminant.sqrt()) / (2.0 * a);
    let root2 = (-b - discriminant.sqrt()) / (2.0 * a);
    Ok((root1 as i64, root2 as i64)) // Two real roots.
}


// Assuming a function to expand and solve the cubic equation given as (a+x)(b+x)(c+x) = invariant
fn solve_cubic_equation_from_factors(vec: Vec<i32>, invariant: i32) -> Option<f64> {
    if vec.len() != 3 {
        return None; // Ensure vec contains exactly three elements.
    }

    let (a, b, c) = (vec[0] as f64, vec[1] as f64, vec[2] as f64);
    let invariant = invariant as f64;

    // Define the cubic equation based on expanded form (a+x)(b+x)(c+x) - invariant
    let f = move |x: f64| -> f64 {
        (a + x) * (b + x) * (c + x) - invariant
    };

    // Define the derivative of the cubic equation
    let fd = move |x: f64| -> f64 {
        (a + x) * (b + x) + (a + x) * (c + x) + (b + x) * (c + x)
    };

    // Attempt to find a root
    find_root(&f, &fd, 0.0, 0.0001, 100).ok()
}

// Assuming a function to solve the quartic equation given as (1+x)(2+x)(3+x)(4+x) = 5
fn solve_quartic_equation_from_factors(vec: Vec<i32>, invariant: i32) -> Option<f64> {
    if vec.len() != 4 {
        return None; // Ensure vec contains exactly four elements.
    }

    let (a, b, c, d) = (vec[0] as f64, vec[1] as f64, vec[2] as f64, vec[3] as f64);
    let invariant = invariant as f64;

    // Define the quartic equation based on expanded form (1+x)(2+x)(3+x)(4+x) = invariant
    let f = move |x: f64| -> f64 {
        (a + x) * (b + x) * (c + x) * (d + x) - invariant
    };

    // Approximate the derivative of the quartic equation (for the sake of example, not exact)
    let fd = move |x: f64| -> f64 {
        4.0*x.powi(3) + 6.0*(a + b + c + d)*x.powi(2) + 4.0*(a*(b + c + d) + b*(c + d) + c*d)*x + (a*b*c + a*b*d + a*c*d + b*c*d)
    };

    // Attempt to find a root
    find_root(&f, &fd, 0.0, 0.0001, 100).ok()
}