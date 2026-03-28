# Part 0: Introduction & Project Setup

> *Corresponding tutorial: [Anatomy of a STARK, Part 0](https://aszepieniec.github.io/stark-anatomy/)*

## What Are STARKs?

STARK = **S**calable **T**ransparent **AR**gument of **K**nowledge

- **Scalable**: Prover time is quasilinear `O(n log n)`, verifier time is polylogarithmic `O(log² n)`
- **Transparent**: No trusted setup — all randomness is public (contrast with SNARKs using pairing-based trusted setup)
- **Argument of Knowledge**: The prover convinces the verifier that they *know* a witness satisfying some computation

### The STARK Pipeline

```
Computation → Algebraic Intermediate Representation (AIR)
           → Polynomial IOPs (interpolation + constraints)
           → Cryptographic Compilation (FRI + Merkle trees)
           → Non-interactive Proof (Fiat-Shamir)
```

---

## Step 1: Project Setup

### 1.1 Update `Cargo.toml`

Replace the current dependencies with arkworks:

```toml
[package]
name = "lumen-stark"
version = "0.1.0"
edition = "2024"

[dependencies]
ark-ff = "0.5"
ark-poly = "0.5"
ark-serialize = "0.5"
ark-std = "0.5"
sha2 = "0.10"
anyhow = "1"
```

### 1.2 Create the folder structure

```
src/
├── lib.rs
├── main.rs
├── field/
│   ├── mod.rs
│   └── goldilocks.rs
├── crypto/
│   ├── mod.rs
│   ├── hasher.rs
│   ├── merkle.rs
│   └── transcript.rs
├── fri/
│   ├── mod.rs
│   ├── layer.rs
│   ├── prover.rs
│   └── verifier.rs
└── stark/
    ├── mod.rs
    ├── air.rs
    ├── prover.rs
    └── verifier.rs
```

### 1.3 Wire up `src/lib.rs`

```rust
pub mod field;
pub mod crypto;
pub mod fri;
pub mod stark;
```

Create empty `mod.rs` files in each subdirectory to make it compile.

---

## Step 2: The Goldilocks Field

### 2.1 Why Goldilocks?

The Goldilocks prime `p = 2^64 - 2^32 + 1` is STARK-friendly because:
- It fits in a `u64`, so arithmetic is fast
- `p - 1 = 2^32 × (2^32 - 1)` has a large power-of-two factor, giving us roots of unity up to order `2^32`
- It's widely used in production STARK systems (Plonky2, Polygon zkEVM)

### 2.2 Define the field

Create `src/field/goldilocks.rs`:

```rust
use ark_ff::{Fp64, MontBackend, MontConfig};

#[derive(MontConfig)]
#[modulus = "18446744069414584321"]  // 2^64 - 2^32 + 1
#[generator = "7"]                   // primitive root
pub struct GoldilocksConfig;

/// The Goldilocks prime field: p = 2^64 - 2^32 + 1
pub type Fq = Fp64<MontBackend<GoldilocksConfig, 1>>;
```

> **What arkworks gives you for free**: All field arithmetic (`+`, `-`, `*`, `/`, `.inverse()`, `.pow()`), serialization, `FftField` trait (roots of unity, NTT), `PrimeField` trait.

### 2.3 Export from `src/field/mod.rs`

```rust
pub mod goldilocks;
pub use goldilocks::Fq;
```

---

## Checkpoint

After this step, you should be able to:

```bash
cargo build
```

Write a quick smoke test in `src/main.rs`:

```rust
use ark_ff::{Field, PrimeField};
use lumen_stark::field::Fq;

fn main() {
    let a = Fq::from(42u64);
    let b = Fq::from(7u64);
    println!("a + b = {}", a + b);           // 49
    println!("a * b = {}", a * b);           // 294
    println!("a / b = {}", a / b);           // 6 (since 6*7 = 42)
    println!("a^{-1} = {}", a.inverse().unwrap());
    println!("Field modulus: {}", Fq::MODULUS);
}
```

```bash
cargo run
```

---

## Key Arkworks Types to Know

| Type | Crate | Purpose |
|------|-------|---------|
| `Fp64<MontBackend<C, 1>>` | `ark-ff` | 64-bit prime field element |
| `PrimeField` trait | `ark-ff` | Field arithmetic, serialization, modulus |
| `FftField` trait | `ark-ff` | Roots of unity, NTT support |
| `DensePolynomial<F>` | `ark-poly` | Univariate polynomial (coefficient form) |
| `SparsePolynomial<F>` | `ark-poly` | Univariate polynomial (sparse terms) |
| `SparsePolynomial<F, SparseTerm>` | `ark-poly` | Multivariate polynomial (sparse terms) |
| `GeneralEvaluationDomain<F>` | `ark-poly` | FFT domains for polynomial evaluation |

---

## Next: [Part 1 — Basic Tools](./part1-basic-tools.md)
