use std::cmp::max;
use std::collections::HashMap;

use crate::common::field::{Field, FieldElement};
use crate::common::polynomial::Polynomial;

pub struct MPolynomial {
    pub dictionary: HashMap<Vec<u128>, FieldElement>,
}

impl MPolynomial {
    pub fn new(dictionary: HashMap<Vec<u128>, FieldElement>) -> Self {
        Self { dictionary }
    }

    pub fn zero() -> Self {
        Self::new(HashMap::new())
    }

    pub fn num_vars(&self) -> usize {
        self.dictionary
            .keys()
            .map(|exps| exps.len())
            .max()
            .unwrap_or(0)
    }

    pub fn add(&self, other: MPolynomial) -> MPolynomial {
        let mut dictionary: HashMap<Vec<u128>, FieldElement> = HashMap::new();
        let num_vars = max(self.num_vars(), other.num_vars());

        for (key, value) in self.dictionary.clone() {
            let mut pad = key.clone();
            pad.resize(num_vars, 0);
            dictionary.insert(pad, value);
        }

        for (key, value) in other.dictionary {
            let mut pad = key.clone();
            pad.resize(num_vars, 0);
            dictionary
                .entry(pad)
                .and_modify(|coeff| {
                    coeff.add(value);
                })
                .or_insert(value);
        }

        MPolynomial::new(dictionary)
    }

    pub fn mul(&self, other: MPolynomial) -> MPolynomial {
        let mut dictionary: HashMap<Vec<u128>, FieldElement> = HashMap::new();
        let num_vars = max(self.num_vars(), other.num_vars());

        for (key, value) in self.dictionary.clone() {
            for (key2, value2) in other.dictionary.clone() {
                let mut exponent = vec![0u128; num_vars];
                for k in 0..key.len() {
                    exponent[k] += key[k];
                }
                for k in 0..key2.len() {
                    exponent[k] += key2[k];
                }
                dictionary
                    .entry(exponent)
                    .and_modify(|coeff| {
                        coeff.add(value.mul(value2));
                    })
                    .or_insert(value.mul(value2));
            }
        }

        MPolynomial::new(dictionary)
    }

    pub fn neg(&self) -> MPolynomial {
        let mut dictionary: HashMap<Vec<u128>, FieldElement> = HashMap::new();
        for (key, value) in self.dictionary.clone() {
            dictionary.insert(key, value.neg());
        }
        MPolynomial::new(dictionary)
    }

    pub fn sub(&self, other: MPolynomial) -> MPolynomial {
        self.add(other.neg())
    }

    pub fn is_zero(&self) -> bool {
        if self.dictionary.is_empty() {
            return true;
        } else {
            for value in self.dictionary.values() {
                if !value.is_zero() {
                    return false;
                }
            }
        }
        true
    }

    pub fn xor(&self, exponent: u128) -> MPolynomial {
        if self.is_zero() {
            return MPolynomial::zero();
        }

        let field = self.dictionary.values().last().unwrap().field;
        let num_vars = self.num_vars();
        let exponent_vec = vec![exponent; num_vars];

        let mut acc: MPolynomial = MPolynomial::new(HashMap::from([(exponent_vec, field.one())]));

        let bit_len = 128 - exponent.leading_zeros();
        for i in (0..bit_len).rev() {
            acc = acc.mul(MPolynomial::new(acc.dictionary.clone()));
            if (1 << i) & exponent != 0 {
                acc = acc.mul(MPolynomial::new(self.dictionary.clone()));
            }
        }

        acc
    }

    pub fn constant(element: FieldElement) -> MPolynomial {
        MPolynomial::new(HashMap::from([(vec![0], element)]))
    }

    pub fn variable(num_vars: u128, field: Field) -> MPolynomial {
        let mut variable = HashMap::new();
        for i in 0..num_vars {
            let mut exponent = vec![0u128; i as usize];
            exponent.push(1);
            exponent.extend(vec![0; (num_vars - i - 1) as usize]);

            variable.insert(exponent, field.one());
        }
        MPolynomial::new(variable)
    }

    pub fn lift(polynomial: Polynomial, variable_index: u128) -> MPolynomial {
        if polynomial.is_zero() {
            return MPolynomial::zero();
        }

        let field = polynomial.coeffs[0].field;
        let variables = MPolynomial::variable(variable_index + 1, field);
        let exponent = variables.dictionary.iter().last().unwrap();
        let x = MPolynomial::new(HashMap::from([(
            exponent.0.to_owned(),
            exponent.1.to_owned(),
        )]));

        let mut acc = MPolynomial::zero();
        for i in 0..polynomial.coeffs.len() {
            acc = acc.add(MPolynomial::constant(polynomial.coeffs[i]).mul(x.xor(i as u128)))
        }

        acc
    }

    pub fn evaluate(&self, point: Vec<FieldElement>) -> FieldElement {
        let mut acc = point[0].field.zero();
        for (key, value) in self.dictionary.clone() {
            let mut prod = value;
            for i in 0..key.len() {
                prod = prod.mul(point[i].xor(key[i]));
            }
            acc = acc.add(prod);
        }
        acc
    }

    pub fn evaluate_symbolic(&self, point: Vec<FieldElement>) -> Polynomial {
        let mut acc = Polynomial::default();
        for (key, value) in self.dictionary.clone() {
            let mut prod = Polynomial::new(vec![value]);
            for i in 0..key.len() {
                prod = prod.mul(Polynomial::new(vec![point[i].xor(key[i])]));
            }
            acc = acc.add(prod);
        }

        acc
    }
}
