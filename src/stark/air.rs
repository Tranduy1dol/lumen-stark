use ark_ff::PrimeField;
use ark_poly::multivariate::{SparsePolynomial, SparseTerm};

#[derive(Clone, Debug)]
pub struct BoundaryConstraint<F: PrimeField> {
    pub cycle: usize,
    pub register: usize,
    pub value: F,
}

#[derive(Clone)]
pub struct Air<F: PrimeField> {
    pub num_registers: usize,
    pub original_trace_length: usize,
    pub transition_constraints: Vec<SparsePolynomial<F, SparseTerm>>,
    pub boundary_constraints: Vec<BoundaryConstraint<F>>,
}
