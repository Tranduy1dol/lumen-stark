use std::marker::PhantomData;

use ark_ff::PrimeField;
use sha2::{Digest, Sha256};

pub struct Transcript<F: PrimeField> {
    hasher: Sha256,
    _phantom: PhantomData<F>,
}

impl<F: PrimeField> Transcript<F> {
    pub fn new(seed: F) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(seed.to_string());

        Self {
            hasher,
            _phantom: PhantomData,
        }
    }

    pub fn digest(&mut self, value: F) {
        self.hasher.update(value.to_string());
    }

    pub fn generate_a_challenge(&mut self) -> F {
        let value = self.hasher.clone().finalize();
        let f = F::from_be_bytes_mod_order(&value);
        self.hasher.update(f.to_string());
        f
    }

    pub fn generate_challenge_list_usize(&mut self, count: usize, domain: usize) -> Vec<usize> {
        (0..count)
            .map(|_| {
                let challenge = self.generate_a_challenge();
                let big_int = challenge.into_bigint();
                let val = big_int.as_ref()[0] as usize;
                val % domain
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::Fq;

    #[test]
    fn test_transcript_deterministic() {
        let mut t1 = Transcript::<Fq>::new(Fq::from(0u64));
        let mut t2 = Transcript::<Fq>::new(Fq::from(0u64));

        t1.digest(Fq::from(42u64));
        t2.digest(Fq::from(42u64));

        assert_eq!(t1.generate_a_challenge(), t2.generate_a_challenge());
    }

    #[test]
    fn test_transcript_different_inputs() {
        let mut t1 = Transcript::<Fq>::new(Fq::from(0u64));
        let mut t2 = Transcript::<Fq>::new(Fq::from(0u64));

        t1.digest(Fq::from(42u64));
        t2.digest(Fq::from(43u64));

        assert_ne!(t1.generate_a_challenge(), t2.generate_a_challenge());
    }
}
