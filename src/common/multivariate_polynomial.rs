use std::cmp::max;
use std::collections::HashMap;
use std::ops::{Add, Mul, Neg, Sub};

use serde::{Deserialize, Serialize};

use crate::common::field::{Field, FieldElement};
use crate::common::polynomial::Polynomial;

/// Represents a multivariate polynomial over a finite field.
///
/// The polynomial is stored sparsely as a dictionary (HashMap) mapping from
/// an exponent vector to a coefficient. For example, the term `c * x_0^2 * x_2^1`
/// in a 3-variable polynomial would be represented by the entry `{ [2, 0, 1]: c }`.
///
/// The length of the exponent vector determines the number of variables. The
/// implementation handles padding with zeros when performing arithmetic with
/// polynomials of different numbers of variables.
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct MPolynomial {
    /// The sparse representation of the polynomial.
    pub dictionary: HashMap<Vec<u128>, FieldElement>,
}

impl MPolynomial {
    /// Creates a new multivariate polynomial from a dictionary of exponents to coefficients.
    ///
    /// Any terms with a zero coefficient are removed to maintain a canonical representation.
    pub fn new(mut dictionary: HashMap<Vec<u128>, FieldElement>) -> Self {
        dictionary.retain(|_, v| !v.is_zero());
        Self { dictionary }
    }

    /// Returns the zero polynomial.
    pub fn zero() -> Self {
        Self::default()
    }

    /// Creates a constant polynomial from a field element.
    pub fn constant(element: FieldElement) -> Self {
        if element.is_zero() {
            Self::zero()
        } else {
            // An empty exponent vector represents a constant term.
            Self::new(HashMap::from([(vec![], element)]))
        }
    }

    /// Returns the number of variables in the polynomial.
    /// This is determined by the maximum length of any exponent vector.
    pub fn num_vars(&self) -> usize {
        self.dictionary
            .keys()
            .map(|exps| exps.len())
            .max()
            .unwrap_or(0)
    }

    /// Checks if the polynomial is the zero polynomial.
    pub fn is_zero(&self) -> bool {
        self.dictionary.is_empty()
    }

    /// Returns the field of the polynomial's coefficients.
    ///
    /// # Panics
    ///
    /// Panics if called on the zero polynomials, which have no coefficients.
    pub fn field(&self) -> Field {
        self.dictionary
            .values()
            .next()
            .expect("Cannot get field from a zero polynomial.")
            .field
    }

    /// Creates a list of polynomials, each representing a single variable.
    ///
    /// `variables(n, field)` returns `[x_0, x_1, ..., x_{n-1}]`.
    pub fn variables(num_vars: usize, field: Field) -> Vec<MPolynomial> {
        (0..num_vars)
            .map(|i| {
                let mut exponent = vec![0; num_vars];
                exponent[i] = 1;
                MPolynomial::new(HashMap::from([(exponent, field.one())]))
            })
            .collect()
    }

    /// Lifts a univariate polynomial into a multivariate one.
    ///
    /// The univariate polynomial `P(y)` is transformed into a multivariate
    /// polynomial `P(x_i)`, where `i` is the `variable_index`.
    pub fn lift(polynomial: &Polynomial, variable_index: usize) -> MPolynomial {
        let mut dictionary = HashMap::new();
        for (i, c) in polynomial.coeffs.iter().enumerate() {
            if !c.is_zero() {
                let mut exponent = vec![0; variable_index + 1];
                exponent[variable_index] = i as u128;
                dictionary.insert(exponent, *c);
            }
        }
        MPolynomial::new(dictionary)
    }

    /// Raises the polynomial to the power of `exponent` using exponentiation by squaring.
    pub fn pow(&self, exponent: u128) -> MPolynomial {
        if self.is_zero() {
            return MPolynomial::zero();
        }
        if exponent == 0 {
            return MPolynomial::constant(self.field().one());
        }

        let mut acc = MPolynomial::constant(self.field().one());
        let bit_len = 128 - exponent.leading_zeros();
        for i in (0..bit_len).rev() {
            acc = &acc * &acc;
            if (1 << i) & exponent != 0 {
                acc = &acc * self;
            }
        }
        acc
    }

    /// Evaluates the polynomial at a given point (a slice of field elements).
    ///
    /// # Panics
    ///
    /// Panics if the point has fewer variables than the polynomial.
    pub fn evaluate(&self, point: &[FieldElement]) -> FieldElement {
        let field = if self.is_zero() {
            if point.is_empty() {
                // This is an ambiguous case. We need a field to return zero.
                // The caller should handle this. For now, we panic.
                panic!("Cannot determine field to evaluate zero polynomial at empty point.");
            }
            point[0].field
        } else {
            self.field()
        };

        let mut acc = field.zero();
        for (exponents, coeff) in &self.dictionary {
            let mut term = *coeff;
            for (i, &exp) in exponents.iter().enumerate() {
                assert!(
                    i < point.len(),
                    "Evaluation point has fewer variables than polynomial."
                );
                term = term * point[i].pow(exp);
            }
            acc = acc + term;
        }
        acc
    }

    /// Symbolically evaluates the polynomial, substituting some variables with
    /// univariate polynomials.
    ///
    /// # Panics
    ///
    /// Panics if the point has fewer variables than the polynomial.
    pub fn evaluate_symbolic(&self, point: &[Polynomial]) -> Polynomial {
        if self.is_zero() {
            return Polynomial::default();
        }

        let mut acc = Polynomial::default();
        for (exponents, coeff) in &self.dictionary {
            let mut term = Polynomial::new(vec![*coeff]);
            for (i, &exp) in exponents.iter().enumerate() {
                assert!(
                    i < point.len(),
                    "Symbolic evaluation point has fewer variables than polynomial."
                );
                // point[i] is a Polynomial. We need to raise it to the power `exp`.
                let p = point[i].pow(exp);
                term = &term * &p;
            }
            acc = &acc + &term;
        }
        acc
    }
}

// --- Operator Trait Implementations ---

impl Neg for MPolynomial {
    type Output = Self;
    fn neg(self) -> Self {
        // Delegate to the reference implementation.
        (&self).neg()
    }
}

impl Neg for &MPolynomial {
    type Output = MPolynomial;
    fn neg(self) -> MPolynomial {
        let new_dict = self
            .dictionary
            .iter()
            .map(|(k, v)| (k.clone(), -*v))
            .collect();
        MPolynomial::new(new_dict)
    }
}

impl Add for MPolynomial {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        // Delegate to the reference implementation.
        &self + &rhs
    }
}

impl<'b> Add<&'b MPolynomial> for &MPolynomial {
    type Output = MPolynomial;
    fn add(self, rhs: &'b MPolynomial) -> MPolynomial {
        if self.is_zero() {
            return rhs.clone();
        }
        if rhs.is_zero() {
            return self.clone();
        }

        let num_vars = max(self.num_vars(), rhs.num_vars());
        let mut new_dict = self.dictionary.clone();

        // Pad keys of the cloned `self` dictionary if necessary.
        if num_vars > self.num_vars() {
            new_dict = new_dict
                .into_iter()
                .map(|(mut k, v)| {
                    k.resize(num_vars, 0);
                    (k, v)
                })
                .collect();
        }

        // Add terms from `rhs`.
        for (k, v) in &rhs.dictionary {
            let mut padded_k = k.clone();
            padded_k.resize(num_vars, 0);
            new_dict
                .entry(padded_k)
                .and_modify(|c| *c = *c + *v)
                .or_insert(*v);
        }

        MPolynomial::new(new_dict)
    }
}

impl Sub for MPolynomial {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        // Delegate to the reference implementation.
        &self - &rhs
    }
}

impl<'b> Sub<&'b MPolynomial> for &MPolynomial {
    type Output = MPolynomial;
    fn sub(self, rhs: &'b MPolynomial) -> MPolynomial {
        self + &(-rhs)
    }
}

impl Mul for MPolynomial {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        // Delegate to the reference implementation.
        &self * &rhs
    }
}

impl<'b> Mul<&'b MPolynomial> for &MPolynomial {
    type Output = MPolynomial;
    fn mul(self, rhs: &'b MPolynomial) -> MPolynomial {
        if self.is_zero() || rhs.is_zero() {
            return MPolynomial::zero();
        }

        let num_vars = max(self.num_vars(), rhs.num_vars());
        let mut new_dict = HashMap::new();

        for (k0, v0) in &self.dictionary {
            for (k1, v1) in &rhs.dictionary {
                let mut exponent = vec![0; num_vars];
                for i in 0..k0.len() {
                    exponent[i] += k0[i];
                }
                for i in 0..k1.len() {
                    exponent[i] += k1[i];
                }

                let product = *v0 * *v1;
                if !product.is_zero() {
                    new_dict
                        .entry(exponent)
                        .and_modify(|c| *c = *c + product)
                        .or_insert(product);
                }
            }
        }
        MPolynomial::new(new_dict)
    }
}
