use ark_ff::{PrimeField, Zero};
use ark_poly::{
    DenseMVPolynomial, DenseUVPolynomial, EvaluationDomain, Evaluations, GeneralEvaluationDomain,
    multivariate::{SparsePolynomial, SparseTerm, Term},
    univariate::DensePolynomial,
};

use crate::polynomial::{domain, poly_pow, shift_poly};

pub(super) fn interpolate_trace<F: PrimeField>(trace: &[Vec<F>]) -> Vec<DensePolynomial<F>> {
    let trace_length = trace.len();
    let domain = domain(trace_length);

    let num_registers = trace[0].len();
    let mut trace_polys = Vec::with_capacity(num_registers);

    for j in 0..num_registers {
        let column: Vec<F> = trace.iter().map(|row| row[j]).collect();
        let evals = Evaluations::from_vec_and_domain(column, domain);
        trace_polys.push(evals.interpolate());
    }

    trace_polys
}

pub(super) fn boundary_quotients<F: PrimeField>(
    trace_polys: &[DensePolynomial<F>],
    boundary_constraints: &[super::air::BoundaryConstraint<F>],
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

pub(super) fn transition_quotients<F: PrimeField>(
    trace_polys: &[DensePolynomial<F>],
    transition_constraints: &[SparsePolynomial<F, SparseTerm>],
    domain: &GeneralEvaluationDomain<F>,
) -> Vec<DensePolynomial<F>> {
    let t = domain.size();
    let omega = domain.group_gen();
    let w = trace_polys.len();

    let mut polys = Vec::with_capacity(transition_constraints.len());
    let mut shifted_polys = Vec::with_capacity(transition_constraints.len());

    for trace_poly in trace_polys {
        let shifted = shift_poly(trace_poly, omega);
        shifted_polys.push(shifted);
    }

    let vanishing: DensePolynomial<F> = domain.vanishing_polynomial().into();
    let last_point = domain.element(t - 1);
    let exclude_last = DensePolynomial::from_coefficients_vec(vec![-last_point, F::one()]);
    let transition_zerofier = vanishing / exclude_last;

    for constraint in transition_constraints {
        let mut numerator = DensePolynomial::<F>::zero();
        for (coeff, term) in constraint.terms() {
            let mut mononial = DensePolynomial::from_coefficients_vec(vec![*coeff]);
            for (var_index, power) in term.vars().iter().zip(term.powers()) {
                let p = if *var_index < w {
                    poly_pow(&trace_polys[*var_index], power)
                } else {
                    poly_pow(&shifted_polys[*var_index - w], power)
                };
                mononial = mononial * p;
            }
            numerator = numerator + mononial;
        }

        let poly = numerator / transition_zerofier.clone();
        polys.push(poly);
    }

    polys
}
