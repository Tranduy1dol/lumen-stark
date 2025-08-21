use std::cmp::max;
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

use crate::common::field::FieldElement;
use crate::common::math;

/// Represents a univariate polynomial over a finite field.
///
/// The polynomial is stored in coefficient form, i.e., `coeffs[i]` is the
/// coefficient of `x^i`. The vector of coefficients is kept in a canonical
/// form by trimming trailing zeros.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Polynomial {
    /// The vector of coefficients, where `coeffs[i]` is the coefficient of `x^i`.
    pub coeffs: Vec<FieldElement>,
}

impl Polynomial {
    /// Creates a new polynomial from a vector of coefficients.
    ///
    /// Trailing zero coefficients are removed to maintain a canonical representation.
    pub fn new(coeffs: Vec<FieldElement>) -> Self {
        let mut coeffs = coeffs;
        // Trim trailing zeros to maintain a canonical representation.
        while let Some(c) = coeffs.last() {
            if c.is_zero() {
                coeffs.pop();
            } else {
                break;
            }
        }
        Self { coeffs }
    }

    /// Returns the degree of the polynomial.
    ///
    /// The degree of the zero polynomials is defined as -1.
    pub fn degree(&self) -> i128 {
        self.coeffs.len() as i128 - 1
    }

    /// Checks if the polynomial is the zero polynomial.
    pub fn is_zero(&self) -> bool {
        self.coeffs.is_empty()
    }

    /// Returns the leading coefficient of the polynomial.
    ///
    /// Returns `None` if the polynomial is the zero polynomial.
    pub fn leading_coefficient(&self) -> Option<FieldElement> {
        self.coeffs.last().copied()
    }

    /// Returns the field to which the polynomial's coefficients belong.
    ///
    /// # Panics
    ///
    /// Panics if called on the zero polynomials, which have no coefficients.
    pub fn field(&self) -> FieldElement {
        assert!(!self.is_zero(), "Zero polynomial has no field.");
        self.coeffs[0]
    }

    /// Raises the polynomial to the power of `exponent`.
    pub fn pow(&self, exponent: u128) -> Polynomial {
        if self.is_zero() {
            return Polynomial::new(vec![]);
        }
        if exponent == 0 {
            return Polynomial::new(vec![self.field().field.one()]);
        };

        let mut acc = Polynomial::new(vec![self.field().field.one()]);
        let bit_len = 128 - exponent.leading_zeros();
        for i in (0..bit_len).rev() {
            acc = &acc * &acc;
            if (1 << i) & exponent != 0 {
                acc = &acc * self;
            }
        }

        acc
    }

    /// Evaluates the polynomial at a given point using Horner's method.
    pub fn evaluate(&self, point: FieldElement) -> FieldElement {
        let mut x_i = point.field.one();
        let mut value = point.field.zero();
        for c in &self.coeffs {
            value = value + *c * x_i;
            x_i = x_i * point;
        }

        value
    }

    /// Evaluates the polynomial on a given domain of points.
    pub fn evaluate_domain(&self, domain: Vec<FieldElement>) -> Vec<FieldElement> {
        domain.iter().map(|e| self.evaluate(*e)).collect()
    }

    /// Scales the polynomial by a factor.
    ///
    /// This operation transforms a polynomial `P(x)` into `P(factor * x)`.
    /// It achieves this by multiplying the i-th coefficient by `factor^i`.
    pub fn scale(&self, factor: FieldElement) -> Polynomial {
        let coeffs = self
            .coeffs
            .iter()
            .enumerate()
            .map(|(i, c)| *c * factor.pow(i as u128))
            .collect();
        Polynomial::new(coeffs)
    }

    /// Computes the unique polynomial of the smallest degree that passes through a given
    /// set of points using Lagrange interpolation.
    pub fn interpolate(domain: Vec<FieldElement>, values: Vec<FieldElement>) -> Self {
        math::interpolate_domain(domain, values)
    }

    /// Computes the polynomial that is zero on all points in the given domain.
    /// This is done by computing the product of `(x - d)` for all `d` in the domain.
    pub fn zerofier(domain: Vec<FieldElement>) -> Self {
        math::zerofier_domain(domain)
    }
}

impl Neg for Polynomial {
    type Output = Self;
    fn neg(self) -> Self {
        Polynomial::new(self.coeffs.iter().map(|c| -*c).collect())
    }
}

impl Neg for &Polynomial {
    type Output = Polynomial;
    fn neg(self) -> Polynomial {
        Polynomial::new(self.coeffs.iter().map(|c| -*c).collect())
    }
}

impl Add for Polynomial {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        &self + &rhs
    }
}

impl<'b> Add<&'b Polynomial> for &Polynomial {
    type Output = Polynomial;
    fn add(self, rhs: &'b Polynomial) -> Polynomial {
        if self.is_zero() {
            return rhs.clone();
        }
        if rhs.is_zero() {
            return self.clone();
        }

        let field = self.field().field;
        let new_len = max(self.coeffs.len(), rhs.coeffs.len());
        let mut new_coeffs = vec![field.zero(); new_len];

        for (i, c) in self.coeffs.iter().enumerate() {
            new_coeffs[i] = new_coeffs[i] + *c;
        }
        for (i, c) in rhs.coeffs.iter().enumerate() {
            new_coeffs[i] = new_coeffs[i] + *c;
        }

        Polynomial::new(new_coeffs)
    }
}

impl Sub for Polynomial {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        &self - &rhs
    }
}

impl<'b> Sub<&'b Polynomial> for &Polynomial {
    type Output = Polynomial;
    fn sub(self, rhs: &'b Polynomial) -> Polynomial {
        self + &(-rhs)
    }
}

impl Mul for Polynomial {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        &self * &rhs
    }
}

impl<'b> Mul<&'b Polynomial> for &Polynomial {
    type Output = Polynomial;
    fn mul(self, rhs: &'b Polynomial) -> Polynomial {
        if self.is_zero() || rhs.is_zero() {
            return Polynomial::new(vec![]);
        }

        let field = self.field().field;
        let new_len = self.coeffs.len() + rhs.coeffs.len() - 1;
        let mut new_coeffs = vec![field.zero(); new_len];

        for i in 0..self.coeffs.len() {
            if self.coeffs[i].is_zero() {
                continue; // Optimization for sparse polynomials
            }
            for j in 0..rhs.coeffs.len() {
                new_coeffs[i + j] = new_coeffs[i + j] + self.coeffs[i] * rhs.coeffs[j];
            }
        }

        Polynomial::new(new_coeffs)
    }
}

impl Div for Polynomial {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        let (quo, rem) = math::poly_long_division(self, rhs);
        assert!(
            rem.is_zero(),
            "Cannot perform polynomial division because remainder is not zero"
        );
        quo
    }
}

impl Rem for Polynomial {
    type Output = Self;
    fn rem(self, rhs: Self) -> Self {
        let (_, rem) = math::poly_long_division(self, rhs);
        rem
    }
}
