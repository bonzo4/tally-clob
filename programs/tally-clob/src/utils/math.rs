use anchor_lang::prelude::*;

use spl_math::precise_number::PreciseNumber;

use crate::errors::TallyClobErrors;

pub fn get_buy_price(pot_shares: Vec<u128>, invariant: u128) -> Result<u128> {

    require!(pot_shares.len() == 2, TallyClobErrors::NotAValidOrder);

    let pot_shares1 = PreciseNumber::new(pot_shares[0]).ok_or(TallyClobErrors::NotAValidOrder)?;
    let pot_shares2 = PreciseNumber::new(pot_shares[0]).ok_or(TallyClobErrors::NotAValidOrder)?;
    let invariant  = PreciseNumber::new(invariant).ok_or(TallyClobErrors::NotAValidOrder)?;

    let a = PreciseNumber::new(1).ok_or(TallyClobErrors::NotAValidOrder)?; // The coefficient of x^2 in the quadratic equation is always 1.
    let b= pot_shares1
                            .checked_add(&pot_shares2)
                            .ok_or(TallyClobErrors::NotAValidOrder)?; // Sum of pot_shares, with a negative sign for the coefficient of x.
    let c: PreciseNumber;
    if invariant.greater_than(&pot_shares1.checked_mul(&pot_shares2).ok_or(TallyClobErrors::NotAValidOrder)?) {
        c = invariant.checked_sub(&pot_shares1.checked_mul(&pot_shares2).ok_or(TallyClobErrors::NotAValidOrder)?).ok_or(TallyClobErrors::NotAValidOrder)?;
    } else {
        c = pot_shares1.checked_mul(&pot_shares2).ok_or(TallyClobErrors::NotAValidOrder)?.checked_sub(&invariant).ok_or(TallyClobErrors::NotAValidOrder)?;
    }

    let b_squared = b.checked_pow(2)
                                    .ok_or(TallyClobErrors::NotAValidOrder)?;

    let four_a_c= a.checked_mul(&PreciseNumber::new(4).ok_or(TallyClobErrors::NotAValidOrder)?)
                                    .ok_or(TallyClobErrors::NotAValidOrder)?
                                    .checked_mul(&c)
                                    .ok_or(TallyClobErrors::NotAValidOrder)?;


    let discriminant: PreciseNumber;

    if invariant.greater_than(&pot_shares1.checked_mul(&pot_shares2).ok_or(TallyClobErrors::NotAValidOrder)?) {
        discriminant = b_squared.checked_add(&four_a_c).ok_or(TallyClobErrors::NotAValidOrder)?;
    } else {
        discriminant = b_squared.checked_sub(&four_a_c).ok_or(TallyClobErrors::NotAValidOrder)?;
    }

    // Calculate the square root of the discriminant for the quadratic formula.
    let sqrt_discriminant = discriminant.sqrt().ok_or(TallyClobErrors::NotAValidOrder)?;

    let two_a = PreciseNumber::new(2)
                                .ok_or(TallyClobErrors::NotAValidOrder)?
                                .checked_mul(&a)
                                .ok_or(TallyClobErrors::NotAValidOrder)?;

    // let mut root1: PreciseNumber;
    // let mut root2: PreciseNumber;

    let valid_root = sqrt_discriminant
                                    .checked_sub(&b)
                                    .ok_or(TallyClobErrors::NotAValidOrder)?
                                    .checked_div(&two_a)
                                    .ok_or(TallyClobErrors::NotAValidOrder)?;

    Ok(valid_root.value.as_u128())

}

pub fn get_sell_price(pot_shares: Vec<u128>, invariant: u128) -> Result<u128> {

    require!(pot_shares.len() == 2, TallyClobErrors::NotAValidOrder);

    let pot_shares1 = PreciseNumber::new(pot_shares[0]).ok_or(TallyClobErrors::NotAValidOrder)?;
    let pot_shares2 = PreciseNumber::new(pot_shares[0]).ok_or(TallyClobErrors::NotAValidOrder)?;
    let invariant  = PreciseNumber::new(invariant).ok_or(TallyClobErrors::NotAValidOrder)?;

    let a = PreciseNumber::new(1).ok_or(TallyClobErrors::NotAValidOrder)?; // The coefficient of x^2 in the quadratic equation is always 1.
    let b= pot_shares1
                            .checked_add(&pot_shares2)
                            .ok_or(TallyClobErrors::NotAValidOrder)?; // Sum of pot_shares, with a negative sign for the coefficient of x.
    let c = pot_shares1
                            .checked_mul(&pot_shares2)
                            .ok_or(TallyClobErrors::NotAValidOrder)?
                            .checked_sub(&invariant)
                            .ok_or(TallyClobErrors::NotAValidOrder)?; // Product of pot_shares minus the invariant for the constant term.

    let b_squared =  b.checked_pow(2)
                                    .ok_or(TallyClobErrors::NotAValidOrder)?;

    let four_a_c= a.checked_mul(&PreciseNumber::new(4).ok_or(TallyClobErrors::NotAValidOrder)?)
                                        .ok_or(TallyClobErrors::NotAValidOrder)?
                                        .checked_mul(&c)
                                        .ok_or(TallyClobErrors::NotAValidOrder)?;

    let discriminant = b_squared.checked_sub(&four_a_c).ok_or(TallyClobErrors::NotAValidOrder)?;

    // Calculate the square root of the discriminant for the quadratic formula.
    let sqrt_discriminant = discriminant.sqrt().ok_or(TallyClobErrors::NotAValidOrder)?;

    let two_a = PreciseNumber::new(2)
                                .ok_or(TallyClobErrors::NotAValidOrder)?
                                .checked_mul(&a)
                                .ok_or(TallyClobErrors::NotAValidOrder)?;

    let b_minus_discriminant_sqrt = b
                                                            .checked_sub(&sqrt_discriminant);

    if b_minus_discriminant_sqrt.is_some() {
        let root = b_minus_discriminant_sqrt
                                            .ok_or(TallyClobErrors::NotAValidOrder)?
                                            .checked_div(&two_a)
                                            .ok_or(TallyClobErrors::NotAValidOrder)?;
        return Ok(root.value.as_u128())
    }

    let b_plus_discriminant_sqrt = b
                                                    .checked_sub(&sqrt_discriminant)
                                                    .ok_or(TallyClobErrors::NotAValidOrder)?;

    let root = b_plus_discriminant_sqrt.checked_div(&two_a)
                                .ok_or(TallyClobErrors::NotAValidOrder)?;

    let root_value = root.value.as_u128();

    if root_value < pot_shares[0] && root_value < pot_shares[1] {
        return Ok(root_value)
    }

    err!(TallyClobErrors::NotAValidOrder) // Return an error if neither root is valid.
}

// pub fn get_sell_price(pot_shares: Vec<f64>, invariant: f64) -> Result<f64> {

//     let mut pot_shares1 = Float::new(128);
//     pot_shares1.assign(pot_shares[0]);
//     let mut pot_shares2 = Float::new(128);
//     pot_shares2.assign(pot_shares[1]);
//     let mut invariant_prec = Float::new(128);
//     invariant_prec.assign(invariant);


//     // (1-x)(2-x) = 3
//     // c = 1 * 2 - 3 = -1
//     let a: Float = Float::with_val(128, 1.0); // Since we're working directly with f64, no scaling is needed.
//     let b = -(pot_shares1 + pot_shares[1]); // Sum of pot_shares.
//     let c = pot_shares1 * pot_shares2 - invariant_prec; // Product of pot_shares minus the invariant.


//     let discriminant: Float = b.pow(2) - 4.0 * a * c; // Calculate the discriminant of the quadratic equation.

//     require!(discriminant >= 0.0, TallyClobErrors::NotAValidOrder); // Ensure discriminant is non-negative for real roots.

//     // Calculate the square root of the discriminant for the quadratic formula.
//     let sqrt_discriminant = discriminant.sqrt();
//     let root1: Float = (-b + sqrt_discriminant) / (2.0 * a); // Calculate the first root.
//     let root2: Float = (-b - sqrt_discriminant) / (2.0 * a); // Calculate the second root.

//     // Ensure the roots are valid based on the application's logic.
//     // Adjust the conditions to reflect the specifics of a sell operation.
//     // msg!(&(pot_shares[0] - root1).to_string());
//     if root1 > 0.0 && pot_shares[0] - root1 > 0.0 {
//         return Ok(root1.to_f64());
//     }
//     // msg!(&(pot_shares[1] - root2).to_string());
//     if root2 > 0.0 && pot_shares[1] - root2 > 0.0{
//         return Ok(root1.to_f64());
//     }

//     err!(TallyClobErrors::NotAValidOrder) // Return an error if neither root is valid.
// }