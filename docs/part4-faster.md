# Part 4: Speeding Things Up

> *Corresponding tutorial: [Anatomy of a STARK, Part 6](https://aszepieniec.github.io/stark-anatomy/faster)*

This part optimizes the naive implementation using NTT (Number Theoretic Transform) and preprocessing. With arkworks, you get most of these optimizations for free.

---

## What Arkworks Already Gives You

| Optimization | Naive Cost | Optimized Cost | Arkworks API |
|-------------|-----------|----------------|-------------|
| Polynomial evaluation on domain | O(n²) | O(n log n) | `domain.fft()` / `Evaluations` |
| Polynomial interpolation | O(n²) | O(n log n) | `Evaluations::interpolate()` |
| Polynomial multiplication | O(n²) | O(n log n) | via FFT internally |
| Coset evaluation | O(n²) | O(n log n) | `domain.fft()` with coset |

You've already been using these in Part 2 (FRI) and Part 3 (STARK) via `GeneralEvaluationDomain`.

---

## Step 1: NTT-Based Polynomial Operations

### 1.1 Fast evaluation on domains

```rust
use ark_poly::{EvaluationDomain, GeneralEvaluationDomain, Evaluations};
use ark_poly::univariate::DensePolynomial;

/// Evaluate polynomial on a domain using NTT (O(n log n) instead of O(n²))
fn fast_evaluate_domain<F: PrimeField>(
    poly: &DensePolynomial<F>,
    domain: &GeneralEvaluationDomain<F>,
) -> Vec<F> {
    // arkworks does this via FFT internally
    let evals = Evaluations::from_vec_and_domain(
        domain.fft(&poly.coeffs),
        *domain
    );
    evals.evals
}
```

### 1.2 Fast interpolation

```rust
/// Interpolate from evaluations to polynomial using IFFT (O(n log n))
fn fast_interpolate<F: PrimeField>(
    values: Vec<F>,
    domain: &GeneralEvaluationDomain<F>,
) -> DensePolynomial<F> {
    Evaluations::from_vec_and_domain(values, *domain).interpolate()
}
```

### 1.3 Coset FFT

For STARK, we need to evaluate on a **larger** coset domain (blowup):

```rust
/// Evaluate on a coset domain for FRI commitment
fn coset_evaluate<F: PrimeField>(
    poly: &DensePolynomial<F>,
    coset_domain: &GeneralEvaluationDomain<F>,
) -> Vec<F> {
    coset_domain.fft(&poly.coeffs)
}
```

---

## Step 2: Fast Zerofier Computation

### Sparse zerofiers

The transition zerofier `Z_T(x) = x^T - 1` has only 2 non-zero coefficients. Use `SparsePolynomial` for efficiency:

```rust
use ark_poly::univariate::SparsePolynomial;

/// Z(x) = x^n - 1 (vanishing poly for domain of size n)
fn fast_vanishing_poly<F: PrimeField>(n: usize) -> SparsePolynomial<F> {
    // domain.vanishing_polynomial() returns this
    SparsePolynomial::from_coefficients_vec(vec![
        (0, -F::one()),  // -1
        (n, F::one()),   //  x^n
    ])
}
```

### Division by vanishing polynomial

For computing quotient polynomials `p(x) / Z(x)`:

```rust
/// Fast division by vanishing polynomial
/// Uses the fact that Z(x) = x^n - 1 is sparse
fn divide_by_vanishing<F: PrimeField>(
    poly: &DensePolynomial<F>,
    domain: &GeneralEvaluationDomain<F>,
) -> DensePolynomial<F> {
    // Option 1: Evaluate poly on coset, evaluate Z on coset,
    //           divide pointwise, then IFFT back
    // Option 2: Direct coefficient manipulation (efficient for x^n - 1)
    todo!()
}
```

---

## Step 3: Preprocessing

### Precompute domain points

```rust
/// Cache frequently-used domain data
pub struct PreprocessedDomain<F: PrimeField> {
    /// The trace domain (size T)
    pub trace_domain: GeneralEvaluationDomain<F>,
    /// The evaluation domain (size T * blowup)
    pub eval_domain: GeneralEvaluationDomain<F>,
    /// Precomputed vanishing polynomial evaluations on eval domain
    pub vanishing_evals: Vec<F>,
}

impl<F: PrimeField> PreprocessedDomain<F> {
    pub fn new(trace_length: usize, blowup_factor: usize) -> Self {
        let trace_domain = GeneralEvaluationDomain::new(trace_length).unwrap();
        let eval_length = trace_length * blowup_factor;
        let eval_domain = GeneralEvaluationDomain::new(eval_length).unwrap();

        // Precompute vanishing poly evaluations on eval domain
        let vanishing = trace_domain.vanishing_polynomial();
        let vanishing_evals = eval_domain.elements()
            .map(|x| vanishing.evaluate(&x))
            .collect();

        Self { trace_domain, eval_domain, vanishing_evals }
    }
}
```

---

## Step 4: Optimized STARK Prover

Now update your STARK prover to use these fast operations:

```rust
/// Optimized STARK prover using NTT
pub fn prove_fast<F: PrimeField>(
    trace: Vec<Vec<F>>,
    air: &Air<F>,
    blowup_factor: usize,
) -> StarkProof<F> {
    let preprocessed = PreprocessedDomain::new(
        air.original_trace_length,
        blowup_factor,
    );

    // 1. Interpolate trace using IFFT (O(n log n))
    // 2. Evaluate trace polys on eval domain using FFT (O(n log n))
    // 3. Compute constraint polys pointwise on eval domain
    // 4. Divide by vanishing poly pointwise
    // 5. IFFT back to get quotient polynomials
    // 6. Random linear combination → composition poly
    // 7. FRI proof

    todo!()
}
```

---

## Checkpoint: Performance Comparison

```rust
#[cfg(test)]
mod tests {
    use std::time::Instant;

    #[test]
    fn bench_stark_naive_vs_fast() {
        // Create a larger trace (e.g., 2^10 steps)
        // Compare prove time between naive and fast implementations
        let trace_length = 1024;

        let start = Instant::now();
        // prove_naive(...)
        let naive_time = start.elapsed();

        let start = Instant::now();
        // prove_fast(...)
        let fast_time = start.elapsed();

        println!("Naive: {:?}, Fast: {:?}", naive_time, fast_time);
        // Fast should be significantly faster for large traces
    }
}
```

---

## Summary of Complexity

| Operation | Naive | With NTT (arkworks) |
|-----------|-------|-------------------|
| Trace interpolation | O(n²) | O(n log n) |
| Constraint evaluation | O(n²) | O(n log n) |
| Quotient computation | O(n²) | O(n log n) |
| FRI commitment | O(n²) | O(n log n) |
| **Total prover** | **O(n²)** | **O(n log n)** |

---

## What's Next?

You've completed the core STARK tutorial! Future directions:

1. **Folding schemes (Nova)**: Compose multiple STARK proofs incrementally
2. **Lattice-based**: Replace hash-based commitments with lattice assumptions
3. **StarkVM**: Define a VM instruction set as an AIR, prove arbitrary programs
4. **DEEP-ALI**: Add out-of-domain sampling for tighter soundness bounds
5. **Batch proofs**: Prove multiple computations in a single proof
