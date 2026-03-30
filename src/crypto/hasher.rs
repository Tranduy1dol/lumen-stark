use ark_ff::PrimeField;
use sha2::{Digest, Sha256};

pub fn hash<F: PrimeField>(data: &F) -> F {
    let mut hasher = Sha256::new();
    hasher.update(data.to_string());
    let h = hasher.finalize();
    F::from_le_bytes_mod_order(&h)
}

pub fn hash_slice<F: PrimeField>(data: &[F]) -> F {
    let mut hasher = Sha256::new();
    data.iter().for_each(|d| hasher.update(d.to_string()));
    let h = hasher.finalize();
    F::from_le_bytes_mod_order(&h)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::Fq;

    #[test]
    fn test_hash_deterministic() {
        let a = Fq::from(42u64);
        assert_eq!(hash(&a), hash(&a));
    }

    #[test]
    fn test_hash_different_inputs() {
        let a = Fq::from(42u64);
        let b = Fq::from(43u64);
        assert_ne!(hash(&a), hash(&b));
    }
}
