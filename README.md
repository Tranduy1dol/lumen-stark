# stark

A STARK (Scalable Transparent ARgument of Knowledge) implementation following [STARK Anatomy](https://aszepieniec.github.io/stark-anatomy/).

## Features

- FRI commitment scheme
- Algebraic Intermediate Representation (AIR)
- STARK prover and verifier (in development)

## Project Structure

```
lumen-stark/
├── Cargo.toml
├── docs/                          # Step-by-step tutorial guides
│   ├── README.md                  # Overview
│   ├── part0-introduction.md      # Setup + Goldilocks field
│   ├── part1-basic-tools.md       # Hasher, Merkle, transcript
│   ├── part2-fri.md               # FRI protocol
│   ├── part3-stark.md             # STARK protocol
│   └── part4-faster.md            # NTT optimization
├── src/
│   ├── lib.rs
│   ├── main.rs
│   ├── field/
│   │   ├── mod.rs
│   │   └── goldilocks.rs          # Goldilocks Fq via MontConfig
│   ├── polynomial/
│   │   └── mod.rs                 # Utility fns (zerofier, interpolate, coset eval)
│   ├── crypto/
│   │   ├── mod.rs
│   │   ├── hasher.rs              # SHA-256 → field element
│   │   ├── merkle.rs              # MerkleTree<F> + MerkleProof<F>
│   │   └── transcript.rs          # Fiat-Shamir (absorb/squeeze)
│   ├── fri/
│   │   ├── mod.rs
│   │   ├── layer.rs               # FriLayer<F>
│   │   ├── prover.rs              # fold_polynomial + generate_proof
│   │   └── verifier.rs            # verify
│   └── stark/
│       ├── mod.rs
│       ├── air.rs                 # BoundaryConstraint, AIR, transition polys
│       ├── prover.rs              # trace → quotients → FRI → proof
│       └── verifier.rs            # verify proof + AIR
└── tests/
    ├── field_tests.rs
    ├── fri_tests.rs
    └── stark_tests.rs
```

## License

This project is licensed under the MIT License.
