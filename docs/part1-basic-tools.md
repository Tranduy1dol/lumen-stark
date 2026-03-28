# Part 1: Basic Tools

> *Corresponding tutorial: [Anatomy of a STARK, Part 2](https://aszepieniec.github.io/stark-anatomy/basic-tools)*

This part builds the cryptographic primitives needed by FRI and STARK: polynomial utilities, Merkle trees, and the Fiat-Shamir transcript.

---

## Step 1: Polynomial Utilities

Arkworks provides all polynomial types out of the box. Here's what you'll use:

### 1.1 Univariate Polynomials

```rust
use ark_poly::univariate::DensePolynomial;
use ark_poly::{DenseUVPolynomial, Polynomial};
use crate::field::Fq;

// Create from coefficients: 3 + 2x + x^2
let poly = DensePolynomial::from_coefficients_vec(vec![
    Fq::from(3), Fq::from(2), Fq::from(1),
]);

// Evaluate at a point
let val = poly.evaluate(&Fq::from(5)); // 3 + 10 + 25 = 38

// Arithmetic: +, -, * all work via operator overloading
```

### 1.2 Multivariate Polynomials

```rust
use ark_poly::polynomial::multivariate::{SparsePolynomial, SparseTerm, Term};

// f(x_0, x_1) = 2·x_0² + 3·x_0·x_1 + 5
let poly = SparsePolynomial::from_coefficients_vec(
    2,  // number of variables
    vec![
        (Fq::from(2), SparseTerm::new(vec![(0, 2)])),         // 2·x_0²
        (Fq::from(3), SparseTerm::new(vec![(0, 1), (1, 1)])), // 3·x_0·x_1
        (Fq::from(5), SparseTerm::new(vec![])),                // constant 5
    ],
);
```

### 1.3 Evaluation Domains (for later FRI/NTT)

```rust
use ark_poly::{EvaluationDomain, GeneralEvaluationDomain};

// Domain of size 8 (roots of unity)
let domain = GeneralEvaluationDomain::<Fq>::new(8).unwrap();

// Evaluate poly at all domain points
let evals: Vec<Fq> = domain.elements().map(|w| poly.evaluate(&w)).collect();

// Vanishing polynomial: Z(x) = x^n - 1 (zero on all domain points)
let vanishing = domain.vanishing_polynomial();
```

### 1.4 Polynomial helper module

Create `src/polynomial/mod.rs` with utility functions you'll need:

```rust
use ark_ff::PrimeField;
use ark_poly::univariate::DensePolynomial;
use ark_poly::{DenseUVPolynomial, Polynomial, EvaluationDomain, GeneralEvaluationDomain};

/// Evaluate a polynomial over a coset domain: {coset * ω^i} for i in 0..domain_size
pub fn evaluate_on_coset<F: PrimeField>(
    poly: &DensePolynomial<F>,
    coset: F,
    domain_size: usize,
) -> Vec<F> {
    let domain = GeneralEvaluationDomain::<F>::new(domain_size).unwrap();
    domain.elements().map(|w| poly.evaluate(&(coset * w))).collect()
}

/// Compute the zerofier polynomial for a given set of points
/// Z(x) = ∏(x - d) for d in domain
pub fn zerofier<F: PrimeField>(domain: &[F]) -> DensePolynomial<F> {
    // TODO: implement
    todo!()
}

/// Lagrange interpolation: given points (domain[i], values[i]), find the unique
/// polynomial of minimal degree passing through all points
pub fn interpolate<F: PrimeField>(
    domain: &[F],
    values: &[F],
) -> DensePolynomial<F> {
    // TODO: implement (or use IFFT for power-of-two domains)
    todo!()
}
```

---

## Step 2: SHA-256 Hasher

Create `src/crypto/hasher.rs`:

```rust
use ark_ff::PrimeField;
use sha2::{Sha256, Digest};

/// Hash a single field element to a field element via SHA-256
pub fn hash<F: PrimeField>(data: &F) -> F {
    // TODO: implement
    // 1. Create SHA-256 hasher
    // 2. Update with the string representation of data
    // 3. Finalize and convert back to field element via F::from_le_bytes_mod_order
    todo!()
}

/// Hash a slice of field elements
pub fn hash_slice<F: PrimeField>(data: &[F]) -> F {
    // TODO: implement
    todo!()
}
```

### Checkpoint

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::Fq;

    #[test]
    fn test_hash_deterministic() {
        let a = Fq::from(42u64);
        assert_eq!(hash(&a), hash(&a)); // Same input → same output
    }

    #[test]
    fn test_hash_different_inputs() {
        let a = Fq::from(42u64);
        let b = Fq::from(43u64);
        assert_ne!(hash(&a), hash(&b)); // Different inputs → different outputs
    }
}
```

---

## Step 3: Merkle Tree

Create `src/crypto/merkle.rs`:

### 3.1 Data structures

```rust
use ark_ff::PrimeField;

/// A proof of membership in a Merkle tree
#[derive(Debug, Clone)]
pub struct MerkleProof<F: PrimeField> {
    pub index: usize,
    pub leaf_val: F,
    pub auth_path: Vec<F>,  // sibling hashes from leaf to root
    pub root: F,
}

/// Binary Merkle tree with field element leaves
#[derive(Debug, Clone)]
pub struct MerkleTree<F: PrimeField> {
    internal_nodes: Vec<Vec<F>>,  // level 0 = hashed leaves, last level = root
    pub leaves: Vec<F>,
    depth: usize,
}
```

### 3.2 Methods to implement

```rust
impl<F: PrimeField> MerkleTree<F> {
    /// Build the tree from leaf values
    /// 1. Hash each leaf → level 0
    /// 2. Pair-hash adjacent nodes → level 1, 2, ...
    /// 3. Until single root
    pub fn new(leaves: Vec<F>) -> Self { todo!() }

    /// Return the Merkle root
    pub fn root(&self) -> F { todo!() }

    /// Generate an authentication path for leaf at `index`
    pub fn generate_proof(&self, index: usize) -> MerkleProof<F> { todo!() }
}

/// Verify a Merkle proof against its embedded root
pub fn verify_merkle_proof<F: PrimeField>(proof: &MerkleProof<F>) -> bool {
    todo!()
}
```

### Checkpoint

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::Fq;

    #[test]
    fn test_merkle_commit_and_verify() {
        let leaves: Vec<Fq> = (0..8).map(|i| Fq::from(i as u64)).collect();
        let tree = MerkleTree::new(leaves.clone());
        let root = tree.root();

        for i in 0..8 {
            let proof = tree.generate_proof(i);
            assert!(verify_merkle_proof(&proof));
            assert_eq!(proof.root, root);
        }
    }

    #[test]
    fn test_merkle_tampered_proof_fails() {
        let leaves: Vec<Fq> = (0..4).map(|i| Fq::from(i as u64)).collect();
        let tree = MerkleTree::new(leaves);

        let mut proof = tree.generate_proof(0);
        proof.leaf_val += Fq::from(1u64);  // tamper
        assert!(!verify_merkle_proof(&proof));
    }
}
```

---

## Step 4: Fiat-Shamir Transcript

Create `src/crypto/transcript.rs`:

The Fiat-Shamir transform converts an interactive proof into a non-interactive one by deriving
the verifier's random challenges from a hash of the transcript so far.

### 4.1 Architecture

```rust
use ark_ff::PrimeField;
use sha2::{Sha256, Digest};

/// A Fiat-Shamir transcript that absorbs field elements and squeezes challenges
pub struct Transcript<F: PrimeField> {
    hasher: Sha256,
    _phantom: std::marker::PhantomData<F>,
}
```

### 4.2 Methods to implement

```rust
impl<F: PrimeField> Transcript<F> {
    /// Create a new transcript, optionally seeded with an initial value
    pub fn new(seed: F) -> Self { todo!() }

    /// Absorb a field element into the transcript
    pub fn digest(&mut self, value: F) { todo!() }

    /// Squeeze a field element challenge from the transcript
    /// This hashes the current state and returns a field element
    pub fn generate_a_challenge(&mut self) -> F { todo!() }

    /// Generate a list of usize challenges (for query indices)
    pub fn generate_challenge_list_usize(&mut self, count: usize) -> Vec<usize> { todo!() }
}
```

### Key insight

The transcript must be **deterministic**: given the same sequence of `digest` calls, it must produce the same challenges. This is what makes the proof non-interactive — the verifier can replay the transcript and derive the same challenges.

### Checkpoint

```rust
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
```

---

## Step 5: Wire up `src/crypto/mod.rs`

```rust
pub mod hasher;
pub mod merkle;
pub mod transcript;
```

---

## Summary

After completing Part 1, you have:

| Component | File | Arkworks types used |
|-----------|------|--------------------|
| Polynomial helpers | `src/polynomial/mod.rs` | `DensePolynomial<F>`, `SparsePolynomial<F, SparseTerm>`, `GeneralEvaluationDomain<F>` |
| SHA-256 hasher | `src/crypto/hasher.rs` | `PrimeField::from_le_bytes_mod_order` |
| Merkle tree | `src/crypto/merkle.rs` | `PrimeField` |
| Fiat-Shamir transcript | `src/crypto/transcript.rs` | `PrimeField` |

```bash
cargo test --lib crypto
cargo test --lib polynomial
```

---

## Next: [Part 2 — FRI](./part2-fri.md)
