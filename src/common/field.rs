use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Sub};

use serde::{Deserialize, Serialize};

use crate::common::math;

/// Represents an element in a finite field.
///
/// A `FieldElement` is defined by a `value` and the `field` (a prime modulus) it belongs to.
/// All arithmetic operations are performed modulo the field's prime.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldElement {
    pub value: u128,
    pub field: Field,
}

impl FieldElement {
    /// Creates a new `FieldElement`, ensuring the value is within the field by taking the modulus.
    pub fn new(value: u128, field: Field) -> Self {
        Self {
            value: value % field.0,
            field,
        }
    }

    /// Computes the multiplicative inverse of the element.
    ///
    /// # Panics
    ///
    /// Panics if the element is zero, as it has no inverse.
    pub fn inv(&self) -> FieldElement {
        self.field.inverse(*self)
    }

    /// Performs modular exponentiation.
    pub fn pow(&self, exponent: u128) -> FieldElement {
        let mut acc = FieldElement::new(1, self.field);
        let val = FieldElement::new(self.value, self.field);

        let bit_len = 128 - exponent.leading_zeros();
        for i in (0..bit_len).rev() {
            acc = acc * acc;
            if (1 << i) & exponent != 0 {
                acc = acc * val;
            }
        }
        acc
    }

    pub fn eq(&self, other: FieldElement) -> bool {
        self.value == other.value
    }

    pub fn is_zero(&self) -> bool {
        self.value == 0
    }

    /// Serializes the field element to its little-endian byte representation.
    /// Useful for cryptographic hashing.
    pub fn bytes(&self) -> Vec<u8> {
        self.value.to_le_bytes().to_vec()
    }
}

impl Add for FieldElement {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        self.field.add(self, rhs)
    }
}

impl Sub for FieldElement {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        self.field.sub(self, rhs)
    }
}

impl Mul for FieldElement {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        self.field.mul(self, rhs)
    }
}

impl Div for FieldElement {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        self.field.divide(self, rhs)
    }
}

impl Neg for FieldElement {
    type Output = Self;
    fn neg(self) -> Self::Output {
        self.field.negate(self)
    }
}

impl fmt::Display for FieldElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

/// Represents a prime finite field.
///
/// The field is defined by its order (modulus), which must be a prime number.
/// This struct provides methods for performing arithmetic on `FieldElement`s.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Field(u128);

impl Field {
    /// Creates a new `Field` with the given prime modulus.
    pub fn new(field: u128) -> Self {
        Self(field)
    }

    /// Returns the additive identity (0) in this field.
    pub fn zero(&self) -> FieldElement {
        FieldElement::new(0, *self)
    }

    /// Returns the multiplicative identity (1) in this field.
    pub fn one(&self) -> FieldElement {
        FieldElement::new(1, *self)
    }

    pub fn add(&self, left: FieldElement, right: FieldElement) -> FieldElement {
        FieldElement::new((left.value + right.value) % self.0, *self)
    }

    pub fn mul(&self, left: FieldElement, right: FieldElement) -> FieldElement {
        FieldElement::new((left.value * right.value) % self.0, *self)
    }

    pub fn sub(&self, left: FieldElement, right: FieldElement) -> FieldElement {
        FieldElement::new((self.0 + left.value - right.value) % self.0, *self)
    }

    pub fn negate(&self, operand: FieldElement) -> FieldElement {
        FieldElement::new(self.0 - operand.value, *self)
    }

    pub fn inverse(&self, operand: FieldElement) -> FieldElement {
        assert!(
            !operand.is_zero(),
            "Multiplicative inverse of 0 is undefined."
        );
        // Use the extended Euclidean algorithm to find the modular inverse.
        let (a, _, g) = math::extended_gcd(operand.value as i128, self.0 as i128);
        assert_eq!(g, 1, "Element is not invertible.");

        // The result `a` can be negative, so we map it to a positive value in the field.
        let p = self.0 as i128;
        let modular_inverse = (a % p + p) % p;

        FieldElement::new(modular_inverse as u128, *self)
    }

    pub fn divide(&self, left: FieldElement, right: FieldElement) -> FieldElement {
        assert_ne!(right.value, 0, "Division by zero");
        self.mul(left, self.inverse(right))
    }

    /// Returns a known generator for the field.
    ///
    /// A generator is an element `g` such that the set `{g^0, g^1, ...}` covers
    /// all non-zero elements of the field.
    ///
    /// # Panics
    ///
    /// Panics if the field is not the specific one for which the generator is known.
    pub fn generator(&self) -> FieldElement {
        assert_eq!(
            self.0,
            1 + 407 * (1 << 119),
            "Do not know generator for other fields beyond 1+407*2^119"
        );
        FieldElement::new(85408008396924667383611388730472331217, *self)
    }

    /// Returns a primitive n-th root of unity.
    pub fn primitive_nth_root(&self, n: u128) -> FieldElement {
        assert_eq!(
            self.0,
            1 + 407 * (1 << 119),
            "Do not know primitive nth root for other fields beyond 1+407*2^119"
        );
        assert!(
            n <= 1 << 119 && (n & (n - 1) == 0),
            "Field does not have nth root of unity where n > 2^119 or not power of two."
        );

        let mut root = FieldElement::new(85408008396924667383611388730472331217, *self);
        let mut order = 1u128 << 119;

        while order != n {
            root = root * root;
            order /= 2;
        }

        root
    }

    /// Samples a `FieldElement` from a byte array by interpreting the bytes as a
    /// big-endian integer and taking the modulus.
    pub fn sample(&self, byte_arrays: Vec<u8>) -> FieldElement {
        let mut acc = 0;
        for b in byte_arrays {
            acc = (acc << 8) | b as u128;
        }
        FieldElement::new(acc, *self)
    }
}
