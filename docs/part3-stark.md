# Part 3: The STARK IOP

> *Corresponding tutorial: [Anatomy of a STARK, Part 4](https://aszepieniec.github.io/stark-anatomy/stark)*

Now we use FRI to build a complete STARK — proving correctness of arbitrary computations.

---

## Theory: From Computation to Proof

### The Pipeline

```
Execution Trace  →  Interpolation  →  Constraints  →  Quotient Polys  →  FRI
     (table)        (polynomials)     (AIR rules)     (low-degree)     (proof)
```

### Arithmetic Intermediate Representation (AIR)

A computation is encoded as an **execution trace** — a table where:
- Each **row** is a step of the computation
- Each **column** (register) holds a value at that step

The correctness of the computation is expressed via two types of constraints:

1. **Boundary constraints**: "Register `j` at step `i` equals `v`"  
   Example: `x₀ = 3` (initial input), `x_{T-1} = output`

2. **Transition constraints**: "The relationship between row `i` and row `i+1`"  
   Example: `x_{i+1} = x_i²` (repeated squaring)

### Key Insight: Polynomials Encode Low-Degree

If the trace satisfies all constraints, then certain **quotient polynomials** have low degree. FRI proves these quotients are low-degree, which proves the trace is valid.

---

## Step 1: AIR Definition

Create `src/stark/air.rs`:

### 1.1 Boundary constraints

```rust
use ark_ff::PrimeField;

/// A single boundary constraint: register `register` at cycle `cycle` must equal `value`
#[derive(Clone, Debug)]
pub struct BoundaryConstraint<F: PrimeField> {
    pub cycle: usize,      // which row (step) of the trace
    pub register: usize,   // which column (register)
    pub value: F,           // expected value
}
```

### 1.2 Transition constraints

Transition constraints are multivariate polynomials over `2w` variables, where `w` is the number of registers (columns). Variables `x_0..x_{w-1}` represent current state, `x_w..x_{2w-1}` represent next state.

```rust
use ark_poly::polynomial::multivariate::{SparsePolynomial, SparseTerm};

/// The full AIR specification
#[derive(Clone)]
pub structAir<F: PrimeField> {
    /// Number of registers (columns in the trace)
    pub num_registers: usize,
    /// Original trace length (number of rows)
    pub original_trace_length: usize,
    /// Transition constraints: polynomials that must be zero for valid transitions
    /// Variables: x_0..x_{w-1} = current state, x_w..x_{2w-1} = next state
    pub transition_constraints: Vec<SparsePolynomial<F, SparseTerm>>,
    /// Boundary constraints
    pub boundary_constraints: Vec<BoundaryConstraint<F>>,
}
```

### 1.3 Example AIR: Repeated Squaring

To prove knowledge of `x` such that applying `x → x²` repeatedly for `T` steps gives a known output:

```rust
/// Build AIR for: x_{i+1} = x_i²
/// This uses 1 register, transition: x_next - x_curr² = 0
fn repeated_squaring_air<F: PrimeField>(
    trace_length: usize,
    input: F,
    output: F,
) -> Air<F> {
    // transition: x_1 - x_0² = 0
    // Variables: x_0 = current, x_1 = next
    let transition = SparsePolynomial::from_coefficients_vec(
        2,  // 2 variables: x_0 (current), x_1 (next)
        vec![
            (F::one(), SparseTerm::new(vec![(1, 1)])),     // + x_1
            (-F::one(), SparseTerm::new(vec![(0, 2)])),    // - x_0²
        ],
    );

    Air {
        num_registers: 1,
        original_trace_length: trace_length,
        transition_constraints: vec![transition],
        boundary_constraints: vec![
            BoundaryConstraint { cycle: 0, register: 0, value: input },
            BoundaryConstraint { cycle: trace_length - 1, register: 0, value: output },
        ],
    }
}
```

---

## Step 2: STARK Prover

Create `src/stark/prover.rs`:

### 2.1 Trace interpolation

The prover first converts the execution trace into polynomials:

```rust
use ark_ff::PrimeField;
use ark_poly::univariate::DensePolynomial;
use ark_poly::{EvaluationDomain, GeneralEvaluationDomain, Evaluations};

/// Interpolate each trace column into a polynomial
///
/// trace[i][j] = value of register j at step i
/// Returns one polynomial per register
fn interpolate_trace<F: PrimeField>(
    trace: &[Vec<F>],
) -> Vec<DensePolynomial<F>> {
    let trace_length = trace.len();
    let domain = GeneralEvaluationDomain::<F>::new(trace_length).unwrap();

    let num_registers = trace[0].len();
    let mut trace_polys = Vec::with_capacity(num_registers);

    for j in 0..num_registers {
        let column: Vec<F> = trace.iter().map(|row| row[j]).collect();
        // Use IFFT to interpolate: Evaluations → DensePolynomial
        let evals = Evaluations::from_vec_and_domain(column, domain);
        trace_polys.push(evals.interpolate());
    }

    trace_polys
}
```

### 2.2 Boundary quotients

For each boundary constraint `(cycle c, register j, value v)`:

```
boundary_quotient_j(x) = (trace_poly_j(x) - v) / (x - ω^c)
```

If the trace satisfies the constraint, this quotient is a polynomial (no remainder).

```rust
/// Compute boundary quotient polynomials
fn boundary_quotients<F: PrimeField>(
    trace_polys: &[DensePolynomial<F>],
    boundary_constraints: &[BoundaryConstraint<F>],
    domain: &GeneralEvaluationDomain<F>,
) -> Vec<DensePolynomial<F>> {
    // For each constraint:
    // 1. Get trace_poly for the register
    // 2. Subtract the expected value: p(x) - v
    // 3. Divide by (x - ω^cycle)
    // This should divide evenly if the constraint is satisfied
    todo!()
}
```

### 2.3 Transition quotients

For each transition constraint polynomial `C(x_cur, x_next)`:

```
transition_quotient(x) = C(trace(x), trace(ω·x)) / Z_T(x)
```

where `Z_T(x)` is the zerofier over the trace domain excluding the last point:
`Z_T(x) = (x^T - 1) / (x - ω^{T-1})`

```rust
/// Compute transition quotient polynomials
fn transition_quotients<F: PrimeField>(
    trace_polys: &[DensePolynomial<F>],
    transition_constraints: &[SparsePolynomial<F, SparseTerm>],
    domain: &GeneralEvaluationDomain<F>,
) -> Vec<DensePolynomial<F>> {
    // For each transition constraint:
    // 1. Substitute trace_poly(x) for current-state variables
    // 2. Substitute trace_poly(ω·x) for next-state variables
    //    (this is trace_poly with a "shift": multiply evaluation point by ω)
    // 3. The result should be zero on all trace points except possibly the last
    // 4. Divide by the transition zerofier
    todo!()
}
```

### 2.4 Composition polynomial

Combine all quotients into one via random linear combination:

```rust
/// The main STARK prove function
pub fn prove<F: PrimeField>(
    trace: Vec<Vec<F>>,
    air: &Air<F>,
) -> StarkProof<F> {
    // 1. Interpolate trace → trace_polys
    // 2. Compute boundary quotients
    // 3. Compute transition quotients
    // 4. Get random weights from Fiat-Shamir transcript
    // 5. Combine: composition = Σ weight_i · quotient_i
    // 6. Run FRI on the composition polynomial
    // 7. Package everything into StarkProof
    todo!()
}
```

### 2.5 Proof structure

```rust
use crate::fri::prover::FriProof;

#[derive(Clone, Debug)]
pub struct StarkProof<F: PrimeField> {
    /// FRI proof for the composition polynomial
    pub fri_proof: FriProof<F>,
    /// Trace polynomial evaluations at out-of-domain point (for DEEP method)
    pub trace_evaluations: Vec<F>,
    /// Merkle roots of trace column commitments
    pub trace_roots: Vec<F>,
}
```

---

## Step 3: STARK Verifier

Create `src/stark/verifier.rs`:

```rust
/// Verify a STARK proof
pub fn verify<F: PrimeField>(
    proof: &StarkProof<F>,
    air: &Air<F>,
) -> Result<(), String> {
    // 1. Replay Fiat-Shamir to get same random weights
    // 2. Verify trace commitments (Merkle roots)
    // 3. Verify boundary constraint evaluations
    // 4. Verify transition constraint evaluations
    // 5. Verify the composition polynomial matches the weighted sum
    // 6. Verify FRI proof
    todo!()
}
```

---

## Step 4: Wire up `src/stark/mod.rs`

```rust
pub mod air;
pub mod prover;
pub mod verifier;
```

---

## Checkpoint: End-to-End STARK

```rust
#[cfg(test)]
mod tests {
    use crate::field::Fq;
    use crate::stark::air::{Air, BoundaryConstraint, repeated_squaring_air};
    use crate::stark::prover::prove;
    use crate::stark::verifier::verify;

    #[test]
    fn test_stark_repeated_squaring() {
        // Prove: starting from x=3, apply x→x² four times
        let input = Fq::from(3);
        let trace_length = 4;

        // Build the execution trace
        let mut trace = vec![vec![input]];
        for i in 1..trace_length {
            let prev = trace[i-1][0];
            trace.push(vec![prev * prev]);
        }
        // trace = [[3], [9], [81], [6561]]

        let output = trace[trace_length - 1][0];
        let air = repeated_squaring_air(trace_length, input, output);

        let proof = prove(trace, &air);
        assert!(verify(&proof, &air).is_ok());
    }

    #[test]
    fn test_stark_soundness() {
        let input = Fq::from(3);
        let trace_length = 4;

        let mut trace = vec![vec![input]];
        for i in 1..trace_length {
            let prev = trace[i-1][0];
            trace.push(vec![prev * prev]);
        }

        let output = trace[trace_length - 1][0];
        let air = repeated_squaring_air(trace_length, input, output);

        // Tamper with the trace
        trace[2][0] += Fq::from(1);

        let proof = prove(trace, &air);
        assert!(verify(&proof, &air).is_err());
    }
}
```

---

## Next: [Part 4 — Speeding Things Up](./part4-faster.md)
