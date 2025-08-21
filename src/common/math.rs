use std::ops::{Add, Div, Mul, Sub};

use crate::common::field::FieldElement;
use crate::common::polynomial::Polynomial;

/// Implements the Extended Euclidean Algorithm.
///
/// Given `x` and `y`, this function finds integers `s` and `t` such that `s*x + t*y = gcd(x, y)`.
/// It returns a tuple `(s, t, gcd)`.
///
/// We use `i128` to handle potential negative intermediate values for `s` and `t`,
/// which is the standard and robust way to implement this algorithm.
pub(super) fn extended_gcd(x: i128, y: i128) -> (i128, i128, i128) {
    let (mut old_r, mut r) = (x, y);
    let (mut old_s, mut s) = (1, 0);
    let (mut old_t, mut t) = (0, 1);

    while r != 0 {
        let quotient = old_r / r;
        (old_r, r) = (r, old_r - quotient * r);
        (old_s, s) = (s, old_s - quotient * s);
        (old_t, t) = (t, old_t - quotient * t);
    }

    (old_s, old_t, old_r)
}

/// Performs polynomial long division.
///
/// Given a numerator `N(x)` and a denominator `D(x)`, it returns a quotient `Q(x)`
/// and a remainder `R(x)` such that `N(x) = D(x) * Q(x) + R(x)` and `deg(R) < deg(D)`.
pub(super) fn poly_long_division(
    numerator: Polynomial,
    denominator: Polynomial,
) -> (Polynomial, Polynomial) {
    assert!(!denominator.is_zero(), "Denominator must not be zero.");

    if numerator.degree() < denominator.degree() {
        return (Polynomial::new(vec![]), numerator);
    }

    let field = denominator.coeffs[0].field;
    let mut remainder = numerator;
    let mut quotient_coeffs =
        vec![field.zero(); (remainder.degree() - denominator.degree() + 1) as usize];

    while !remainder.is_zero() && remainder.degree() >= denominator.degree() {
        let degree_diff = remainder.degree() - denominator.degree();
        let coeff = remainder
            .leading_coefficient()
            .unwrap()
            .div(denominator.leading_coefficient().unwrap());

        quotient_coeffs[degree_diff as usize] = coeff;

        // Create the monomial `coeff * x^degree_diff` to subtract.
        let mut monomial_coeffs = vec![field.zero(); degree_diff as usize];
        monomial_coeffs.push(coeff);
        let monomial = Polynomial::new(monomial_coeffs);

        let subtractee = denominator.clone().mul(monomial);
        remainder = remainder.sub(subtractee)
    }

    (Polynomial::new(quotient_coeffs), remainder)
}

/// Computes the unique polynomial of the smallest degree that passes through a given
/// set of points `(domain[i], values[i])` using Lagrange interpolation.
pub(super) fn interpolate_domain(
    domain: Vec<FieldElement>,
    values: Vec<FieldElement>,
) -> Polynomial {
    assert_eq!(
        domain.len(),
        values.len(),
        "Domain and values must have the same length."
    );
    if domain.is_empty() {
        return Polynomial::new(vec![]);
    }

    let field = domain[0].field;
    let x = Polynomial::new(vec![field.zero(), field.one()]);
    let mut sum = Polynomial::new(vec![]);

    for i in 0..domain.len() {
        // Skip if the value is zero, as it won't contribute to the sum.
        if values[i].is_zero() {
            continue;
        }

        // Start with the value y_i
        let mut lagrange_basis = Polynomial::new(vec![values[i]]);

        // Compute product of (x - x_j) / (x_i - x_j) for j != i
        for j in 0..domain.len() {
            if i == j {
                continue;
            }
            let numerator = x.clone().sub(Polynomial::new(vec![domain[j]]));
            let denominator_inv = domain[i].sub(domain[j]).inv();
            lagrange_basis = lagrange_basis
                .mul(numerator)
                .mul(Polynomial::new(vec![denominator_inv]));
        }
        sum = sum.add(lagrange_basis);
    }

    sum
}

/// Computes the polynomial that is zero on all points in the given domain.
/// This is done by computing the product of `(x - d)` for all `d` in the domain.
pub(super) fn zerofier_domain(domain: Vec<FieldElement>) -> Polynomial {
    let field = domain[0].field;
    let x = Polynomial::new(vec![field.zero(), field.one()]);
    let mut acc = Polynomial::new(vec![field.one()]);
    for d in domain {
        acc = acc.mul(x.clone().sub(Polynomial::new(vec![d])));
    }
    acc
}

/// Tests if a set of 2D points are collinear by interpolating a polynomial
/// through them and checking if its degree is at most 1.
pub(super) fn test_colinearity(points: Vec<(FieldElement, FieldElement)>) -> bool {
    let domain = points.iter().map(|p| p.0).collect::<Vec<_>>();
    let values = points.iter().map(|p| p.1).collect::<Vec<_>>();
    let polynomial = interpolate_domain(domain, values);
    polynomial.degree() <= 1
}
