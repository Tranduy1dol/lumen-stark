# STARK Tutorial — Building STARKs with Arkworks in Rust

A step-by-step guide to understanding and implementing STARKs (Scalable Transparent ARguments of Knowledge) from scratch, using the [arkworks](https://arkworks.rs/) library for finite field and polynomial arithmetic.

**Based on**: [Anatomy of a STARK](https://aszepieniec.github.io/stark-anatomy/) by Alan Szepieniec

## Prerequisites

- Rust (edition 2024)
- Basic understanding of modular arithmetic and polynomials
- Familiarity with hash functions and Merkle trees

## Tutorial Structure

| Part | Topic | What You'll Build | Code Location |
|------|-------|--------------------|--------------|
| 0 | [Introduction](./part0-introduction.md) | Project setup, arkworks orientation | `Cargo.toml`, `src/field/` |
| 1 | [Basic Tools](./part1-basic-tools.md) | Field, polynomials, Merkle tree, Fiat-Shamir | `src/field/`, `src/crypto/` |
| 2 | [FRI](./part2-fri.md) | FRI low-degree test (prover + verifier) | `src/fri/` |
| 3 | [The STARK IOP](./part3-stark.md) | STARK prover & verifier with AIR constraints | `src/stark/` |
| 4 | [Speeding Things Up](./part4-faster.md) | NTT optimization, preprocessing | Performance improvements |

## How to Use This Guide

Each part contains:
1. **Theory** — key concepts explained with equations
2. **Architecture** — what structs/traits to define and why
3. **Step-by-step instructions** — what to implement, with code signatures
4. **Checkpoints** — tests to run to verify your implementation
5. **References** — links to the original tutorial and arkworks docs

> **Convention**: Code signatures are provided but implementations are left for you to write. Each checkpoint has a `cargo test` command so you can verify your work.

## Quick Start

```bash
# Clone and enter the project
cd lumen-stark

# After implementing each part, run its tests:
cargo test --lib field     # Part 0
cargo test --lib crypto    # Part 1
cargo test --lib fri       # Part 2
cargo test --lib stark     # Part 3
```

## Future Roadmap

After completing this tutorial:
- **Folding schemes**: Integrate with Nova, lattice-based proof systems
- **StarkVM**: Build a virtual machine whose execution is proven via STARKs
