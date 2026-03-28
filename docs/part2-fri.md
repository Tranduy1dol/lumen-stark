# Part 2: FRI (Fast Reed-Solomon IOP of Proximity)

> *Corresponding tutorial: [Anatomy of a STARK, Part 3](https://aszepieniec.github.io/stark-anatomy/fri)*

FRI is the core protocol that lets us prove a polynomial has low degree, **without revealing the polynomial**. It's the cryptographic engine powering STARKs.

---

## Theory: Split-and-Fold

Given a polynomial `f(x)` of degree < `d`, split it into even and odd parts:

```
f(x) = f_even(x²) + x · f_odd(x²)
```

The verifier sends a random challenge `α`. The prover computes:

```
f'(x) = f_even(x) + α · f_odd(x)
```

This **halves** the degree! Repeat `log(d)` times until the polynomial is a constant.

### Why it works

If `f` was truly low-degree, each fold produces a valid lower-degree polynomial. If `f` was *not* low-degree, folding will produce inconsistencies that the verifier can catch by **querying** random evaluation points and checking consistency via Merkle proofs.

### Security

The verifier makes `k` random queries. Each query has a `1/|domain|` chance of missing an inconsistency, so `k` queries give security `≈ 2^(-k)`.

---

## Step 1: FRI Layer

Create `src/fri/layer.rs`:

### 1.1 Data structure

```rust
use ark_ff::PrimeField;
use ark_poly::univariate::DensePolynomial;
use ark_poly::{EvaluationDomain, GeneralEvaluationDomain, Polynomial};
use crate::crypto::merkle::MerkleTree;

/// One layer in the FRI protocol — stores evaluations over a coset and their Merkle commitment
#[derive(Clone)]
pub struct FriLayer<F: PrimeField> {
    /// Polynomial evaluations at points {coset · ω^i}
    pub evaluations: Vec<F>,
    /// Merkle tree commitment to the evaluations
    pub merkle_tree: MerkleTree<F>,
    /// Coset offset used for domain generation
    pub coset: F,
    /// Size of the evaluation domain
    pub domain_size: usize,
}
```

### 1.2 Constructor

```rust
impl<F: PrimeField> FriLayer<F> {
    /// Build a FRI layer by evaluating `poly` over the coset domain {coset · ω^i}
    pub fn from_poly(poly: &DensePolynomial<F>, coset: F, domain_size: usize) -> Self {
        // 1. Create evaluation domain of `domain_size` using GeneralEvaluationDomain
        // 2. Evaluate poly at each point (root * coset)
        // 3. Build MerkleTree from the evaluations
        todo!()
    }
}
```

### Key insight: Coset domains

We evaluate over a **coset** `{g·ω^i}` rather than the raw domain `{ω^i}` to avoid evaluating at roots of unity where the vanishing polynomial is zero (which would cause division-by-zero issues in the STARK protocol).

---

## Step 2: Polynomial Folding

In `src/fri/prover.rs`:

### 2.1 The fold operation

```rust
use ark_ff::PrimeField;
use ark_poly::univariate::DensePolynomial;
use ark_poly::DenseUVPolynomial;

/// Fold a polynomial using random challenge `r`:
///   f(x) = f_even(x²) + x · f_odd(x²)
///   folded(x) = f_even(x) + r · f_odd(x)
///
/// This halves the degree.
pub fn fold_polynomial<F: PrimeField>(
    poly: &DensePolynomial<F>,
    random_r: F,
) -> DensePolynomial<F> {
    // 1. Split coefficients into even-indexed and odd-indexed
    // 2. even_poly = DensePolynomial::from_coefficients_vec(even_coeffs)
    // 3. odd_poly = DensePolynomial::from_coefficients_vec(odd_coeffs)
    // 4. Return even_poly + r * odd_poly
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
    fn test_fold_polynomial() {
        // f(x) = 1 + 2x + 3x² + 4x³
        // even = 1 + 3x,  odd = 2 + 4x
        // fold with r=1: (1+3x) + 1*(2+4x) = 3 + 7x
        let poly = DensePolynomial::from_coefficients_vec(
            vec![Fq::from(1), Fq::from(2), Fq::from(3), Fq::from(4)]
        );
        let folded = fold_polynomial(&poly, Fq::from(1));
        let expected = DensePolynomial::from_coefficients_vec(
            vec![Fq::from(3), Fq::from(7)]
        );
        assert_eq!(folded, expected);
    }
}
```

---

## Step 3: FRI Prover

### 3.1 Proof structures

```rust
use crate::crypto::merkle::MerkleProof;

/// Authentication data for one query through all FRI layers
#[derive(Clone, Debug)]
pub struct Decommitment<F: PrimeField> {
    pub evaluations: Vec<F>,           // f(x_i) at each layer
    pub auth_paths: Vec<MerkleProof<F>>,
    pub sym_evaluations: Vec<F>,       // f(-x_i) at each layer (symmetric point)
    pub sym_auth_paths: Vec<MerkleProof<F>>,
}

/// Complete FRI proof
#[derive(Clone, Debug)]
pub struct FriProof<F: PrimeField> {
    pub domain_size: usize,
    pub coset: F,
    pub number_of_queries: usize,
    pub layers_root: Vec<F>,           // Merkle roots of each layer
    pub const_val: F,                  // Final constant value
    pub decommitment_list: Vec<Decommitment<F>>,
}
```

### 3.2 The two-phase prove

```rust
/// Generate a FRI proof
///
/// Phase 1 (Folding): Repeatedly fold the polynomial, committing each layer
/// Phase 2 (Query): Open random positions across all layers
pub fn generate_proof<F: PrimeField>(
    poly: DensePolynomial<F>,
    blowup_factor: usize,
    number_of_queries: usize,
) -> FriProof<F> {
    // compute domain_size = (poly.len() * blowup_factor).next_power_of_two()
    // set coset = F::GENERATOR
    // number_of_layers = domain_size.ilog2()

    // --- Phase 1: Folding ---
    // for each layer:
    //   1. Build FriLayer from current poly
    //   2. Push Merkle root into transcript
    //   3. Get random challenge from transcript
    //   4. Fold polynomial with challenge
    //   5. Square the coset, halve domain_size
    // Until poly is constant

    // --- Phase 2: Query ---
    // Get `number_of_queries` random indices from transcript
    // For each index, collect evaluations + Merkle proofs at that index
    // across all layers (both index and its symmetric partner)

    todo!()
}
```

---

## Step 4: FRI Verifier

Create `src/fri/verifier.rs`:

```rust
/// Verify a FRI proof
///
/// 1. Replay the Fiat-Shamir transcript to regenerate all challenges
/// 2. Regenerate query indices
/// 3. For each query, verify:
///    a. Merkle proofs are valid
///    b. Folding is consistent: f_next(x) = (r + w_i)·f(x_i)/(2·w_i) - (r - w_i)·f(-x_i)/(2·w_i)
///    c. Final value matches const_val
pub fn verify<F: PrimeField>(proof: FriProof<F>) -> Result<(), String> {
    // 1. Rebuild transcript: digest each Merkle root, squeeze random_r
    // 2. Digest const_val
    // 3. Generate query indices
    // 4. For each (index, decommitment): verify_query(...)
    todo!()
}

fn verify_query<F: PrimeField>(
    challenge: &usize,
    decommitment: &Decommitment<F>,
    random_r_list: &[F],
    domain_size: usize,
    const_val: F,
    coset: F,
) -> Result<(), String> {
    // For each layer i:
    //   1. Compute index and sym_index in current domain
    //   2. Verify Merkle proofs
    //   3. Compute folded value using the folding formula
    //   4. Check it matches the evaluation at layer i+1 (or const_val for last layer)
    //   5. Square coset, halve domain_size
    todo!()
}
```

### The folding verification formula

```
w_i = ω^index · coset     (the evaluation point)
q_fold = (r + w_i) · f(w_i) / (2·w_i) - (r - w_i) · f(-w_i) / (2·w_i)
```

This is algebraically equivalent to evaluating `f_even(w_i²) + r · f_odd(w_i²)` but uses only the evaluations `f(w_i)` and `f(-w_i)`, which is what the verifier has access to via Merkle proofs.

---

## Step 5: Wire up `src/fri/mod.rs`

```rust
pub mod layer;
pub mod prover;
pub mod verifier;
```

---

## Checkpoint: Full FRI Roundtrip

```rust
#[cfg(test)]
mod tests {
    use ark_poly::univariate::DensePolynomial;
    use ark_poly::DenseUVPolynomial;
    use crate::field::Fq;
    use crate::fri::prover::generate_proof;
    use crate::fri::verifier::verify;

    #[test]
    fn test_fri_roundtrip_degree_3() {
        let poly = DensePolynomial::from_coefficients_vec(
            vec![Fq::from(1), Fq::from(2), Fq::from(3), Fq::from(4)]
        );
        let proof = generate_proof(poly, 2, 2);  // blowup=2, queries=2
        assert!(verify(proof).is_ok());
    }

    #[test]
    fn test_fri_roundtrip_degree_5() {
        let poly = DensePolynomial::from_coefficients_vec(
            vec![Fq::from(1), Fq::from(2), Fq::from(3),
                 Fq::from(4), Fq::from(5), Fq::from(6)]
        );
        let proof = generate_proof(poly, 2, 2);
        assert!(verify(proof).is_ok());
    }

    #[test]
    fn test_fri_soundness() {
        let poly = DensePolynomial::from_coefficients_vec(
            vec![Fq::from(1), Fq::from(2), Fq::from(3), Fq::from(4)]
        );
        let mut proof = generate_proof(poly, 2, 2);
        proof.const_val -= Fq::from(1);  // tamper!
        assert!(verify(proof).is_err());
    }
}
```

---

## Next: [Part 3 — The STARK IOP](./part3-stark.md)
