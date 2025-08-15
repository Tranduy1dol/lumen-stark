use crate::common::math;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct FieldElement {
    pub value: u128,
    pub field: Field,
}

impl FieldElement {
    pub fn new(value: u128, field: Field) -> Self {
        Self {
            value: value % field.0,
            field,
        }
    }

    pub fn add(&self, other: FieldElement) -> FieldElement {
        self.field.add(*self, other)
    }

    pub fn mul(&self, other: FieldElement) -> FieldElement {
        self.field.mul(*self, other)
    }

    pub fn div(&self, other: FieldElement) -> FieldElement {
        self.field.divide(*self, other)
    }

    pub fn sub(&self, other: FieldElement) -> FieldElement {
        self.field.sub(*self, other)
    }

    pub fn neg(&self) -> FieldElement {
        self.field.negate(*self)
    }

    pub fn inv(&self) -> FieldElement {
        self.field.inverse(*self)
    }

    pub fn xor(&self, exponent: u128) -> FieldElement {
        let mut acc = FieldElement::new(1, self.field);
        let val = FieldElement::new(self.value, self.field);

        let bit_len = 128 - exponent.leading_zeros();
        for i in (0..bit_len).rev() {
            acc = acc.mul(acc);
            if (1 << i) & exponent != 0 {
                acc = acc.mul(val);
            }
        }
        acc
    }

    pub fn eq(&self, other: FieldElement) -> bool {
        self.value == other.value
    }

    pub fn neq(&self, other: FieldElement) -> bool {
        self.value != other.value
    }

    pub fn is_zero(&self) -> bool {
        self.value == 0
    }

    pub fn str(&self) -> String {
        self.value.to_string()
    }

    pub fn bytes(&self) -> Vec<u8> {
        self.value.to_le_bytes().to_vec()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Field(u128);

impl Field {
    pub fn new(field: u128) -> Self {
        Self(field)
    }

    pub fn zero(&self) -> FieldElement {
        FieldElement::new(0, *self)
    }

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
        let (a, _, _) = math::extended_gcd(operand.value, self.0);
        FieldElement::new(a, *self)
    }

    pub fn divide(&self, left: FieldElement, right: FieldElement) -> FieldElement {
        assert_ne!(right.value, 0, "Division by zero");

        let (a, _, _) = math::extended_gcd(right.value, self.0);
        FieldElement::new((left.value * a) % self.0, *self)
    }

    pub fn generator(&self) -> FieldElement {
        assert_eq!(self.0, 1 + 407 * (1 << 119), "Do not know generator for other fields beyond 1+407*2^119");
        FieldElement::new(85408008396924667383611388730472331217, *self)
    }

    pub fn primitive_nth_root(&self, n: u128) -> FieldElement {
        assert_eq!(self.0, 1 + 407 * (1 << 119), "Do not know primitive nth root for other fields beyond 1+407*2^119");
        assert!(n <= 1 << 119 && (n & (n - 1) == 0), "Field does not have nth root of unity where n > 2^119 or not power of two.");
        
        let mut root = FieldElement::new(85408008396924667383611388730472331217, *self);
        let mut order = 1u128 << 119;
        
        while order != n {
            root = root.mul(root);
            order /= 2;
        }
        
        root
    }
    
    pub fn sample(&self, byte_arrays: Vec<u8>) -> FieldElement {
        let mut acc = 0;
        for b in byte_arrays {
            acc = (acc << 8) | b as u128;
        }
        FieldElement::new(acc, *self)   
    }
}
