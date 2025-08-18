use std::cmp::max;

use crate::common::field::FieldElement;
use crate::common::math::poly_long_division;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Polynomial {
    pub coeffs: Vec<FieldElement>,
}

impl Polynomial {
    pub fn new(coeffs: Vec<FieldElement>) -> Self {
        Polynomial { coeffs }
    }

    pub fn degree(&self) -> i128 {
        if self.coeffs.is_empty() {
            return -1;
        }

        let zero = self.coeffs[0].field.zero();
        if self.coeffs == vec![zero; self.coeffs.len()] {
            return 0;
        }

        let mut max_index = 0;
        for i in 0..self.coeffs.len() {
            if self.coeffs[i] != zero {
                max_index = i;
            }
        }

        max_index as i128
    }

    pub fn neg(&self) -> Polynomial {
        Polynomial::new(self.coeffs.iter().map(|x| x.neg()).collect())
    }

    pub fn add(&self, other: Polynomial) -> Polynomial {
        if self.degree() == -1 {
            return other;
        } else if other.degree() == -1 {
            return Polynomial::new(self.coeffs.clone());
        };

        let field = self.coeffs[0].field;
        let mut coeffs = vec![field.zero(); max(self.coeffs.len(), other.coeffs.len())];

        for i in 0..self.coeffs.len() {
            coeffs[i] = coeffs[i].add(self.coeffs[i]);
        }
        for i in 0..other.coeffs.len() {
            coeffs[i] = coeffs[i].add(other.coeffs[i]);
        }

        Polynomial::new(coeffs)
    }

    pub fn sub(&self, other: Polynomial) -> Polynomial {
        self.add(other.neg())
    }

    pub fn mul(&self, other: Polynomial) -> Polynomial {
        if self.coeffs.is_empty() || other.coeffs.is_empty() {
            return Polynomial::new(vec![]);
        }

        let zero = self.coeffs[0].field.zero();
        let mut coeffs = vec![zero; self.coeffs.len() + other.coeffs.len() - 1];

        for i in 0..self.coeffs.len() {
            if self.coeffs[i].is_zero() {
                continue;
            }

            for j in 0..other.coeffs.len() {
                coeffs[i + j] = coeffs[i + j].add(self.coeffs[i].mul(other.coeffs[j]));
            }
        }

        Polynomial::new(coeffs)
    }

    pub fn eq(&self, other: Polynomial) -> bool {
        self.coeffs == other.coeffs
    }

    pub fn ne(&self, other: Polynomial) -> bool {
        !self.eq(other)
    }

    pub fn is_zero(&self) -> bool {
        self.degree() == -1
    }

    pub fn leading_coeff(&self) -> FieldElement {
        self.coeffs[self.degree() as usize]
    }

    pub fn true_div(&self, other: Polynomial) -> anyhow::Result<Polynomial> {
        let (quo, rem) = poly_long_division(Polynomial::new(self.coeffs.clone()), other)?;
        assert!(
            rem.is_zero(),
            "cannot perform polynomial division because remainder is not zero"
        );

        Ok(quo)
    }

    pub fn mod_(&self, other: Polynomial) -> anyhow::Result<Polynomial> {
        let (_, rem) = poly_long_division(Polynomial::new(self.coeffs.clone()), other)?;
        Ok(rem)
    }

    pub fn xor(&self, exponent: u128) -> Polynomial {
        if self.is_zero() {
            return Polynomial::default();
        }
        if exponent == 0 {
            return Polynomial::new(vec![self.coeffs[0].field.one()]);
        };

        let mut acc = Polynomial::new(vec![self.coeffs[0].field.one()]);
        let bit_len = 128 - exponent.leading_zeros();
        for i in (0..bit_len).rev() {
            acc = acc.mul(Polynomial::new(acc.coeffs.clone()));
            if (1 << i) & exponent != 0 {
                acc = acc.mul(Polynomial::new(self.coeffs.clone()));
            }
        }

        acc
    }

    pub fn evaluate(&self, point: FieldElement) -> FieldElement {
        let mut x_i = point.field.one();
        let mut value = point.field.zero();
        for c in self.coeffs.clone() {
            value = value.add(c.mul(x_i));
            x_i = x_i.mul(point);
        }

        value
    }

    pub fn evaluate_domain(&self, domain: Vec<FieldElement>) -> Vec<FieldElement> {
        domain.iter().map(|e| self.evaluate(*e)).collect()
    }

    pub fn scale(&self, factor: FieldElement) -> Polynomial {
        let mut coeffs: Vec<FieldElement> = Vec::new();
        for i in 0..self.coeffs.len() {
            coeffs.push(self.coeffs[i].mul(factor.xor(i as u128)));
        }

        Polynomial::new(coeffs)
    }
}
