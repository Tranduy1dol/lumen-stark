use std::vec;

use ark_ff::PrimeField;
use ark_poly::{
    DenseUVPolynomial, EvaluationDomain, Evaluations, GeneralEvaluationDomain,
    multivariate::{SparsePolynomial, SparseTerm},
    univariate::DensePolynomial,
};

use crate::stark::air::BoundaryConstraint;

fn interpolate_trace<F: PrimeField>(trace: &[Vec<F>]) -> Vec<DensePolynomial<F>> {
    let trace_length = trace.len();
    let domain = <GeneralEvaluationDomain<F> as EvaluationDomain<F>>::new(trace_length).unwrap();

    let num_registers = trace[0].len();
    let mut trace_polys = Vec::with_capacity(num_registers);

    for j in 0..num_registers {
        let column: Vec<F> = trace.iter().map(|row| row[j]).collect();
        let evals = Evaluations::from_vec_and_domain(column, domain);
        trace_polys.push(evals.interpolate());
    }

    trace_polys
}

fn boundary_quotients<F: PrimeField>(
    trace_polys: &[DensePolynomial<F>],
    boundary_constraints: &[BoundaryConstraint<F>],
    domain: &GeneralEvaluationDomain<F>,
) -> Vec<DensePolynomial<F>> {
    let mut polys = Vec::with_capacity(boundary_constraints.len());

    for constraint in boundary_constraints {
        let t_poly = &trace_polys[constraint.register];
        let numerator = t_poly - DensePolynomial::from_coefficients_vec(vec![constraint.value]);
        let omega_c = domain.element(constraint.cycle);
        let denominator = DensePolynomial::from_coefficients_vec(vec![-omega_c, F::one()]);
        let poly = numerator / denominator;

        polys.push(poly);
    }

    polys
}

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
    let t = domain.size();
    let omega = domain.group_gen();
    let w = trace_polys.len();

    let mut polys = Vec::with_capacity(transition_constraints.len());
    let mut shifted_polys = Vec::with_capacity(transition_constraints.len());

    for trace_poly in trace_polys {
        let mut shifted_coeffs = Vec::with_capacity(trace_poly.coeffs.len());
        for (i, coeff) in trace_poly.coeffs.iter().enumerate() {
            shifted_coeffs.push(*coeff * omega.pow(i));
        }
        shifted_polys.push(DensePolynomial::from_coefficients_vec(shifted_polys));
    }

    polys
}
