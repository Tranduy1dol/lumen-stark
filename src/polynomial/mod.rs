use ark_ff::{FftField, PrimeField};
use ark_poly::{
    DenseUVPolynomial, EvaluationDomain, Evaluations, GeneralEvaluationDomain, Polynomial,
    univariate::{DensePolynomial, SparsePolynomial},
};

pub fn poly_pow<F: PrimeField>(poly: &DensePolynomial<F>, exp: usize) -> DensePolynomial<F> {
    match exp {
        0 => DensePolynomial::from_coefficients_vec(vec![F::one()]),
        1 => poly.clone(),
        _ => {
            let mut result = poly.clone();
            for _ in 1..exp {
                result = &result * poly;
            }
            result
        }
    }
}

pub fn shift_poly<F: PrimeField>(poly: &DensePolynomial<F>, factor: F) -> DensePolynomial<F> {
    let mut new_coeffs = Vec::new();
    let mut power = F::one();

    for coeff in &poly.coeffs {
        new_coeffs.push(*coeff * power);
        power *= factor;
    }

    DensePolynomial::from_coefficients_vec(new_coeffs)
}

pub fn domain<F: FftField>(size: usize) -> GeneralEvaluationDomain<F> {
    GeneralEvaluationDomain::new(size).expect("failed to create evaluation domain")
}

pub fn fast_evaluate_domain<F: PrimeField>(
    poly: &DensePolynomial<F>,
    domain: &GeneralEvaluationDomain<F>,
) -> Vec<F> {
    let eval = Evaluations::from_vec_and_domain(domain.fft(&poly.coeffs), *domain);
    eval.evals
}

pub fn fast_interpolate<F: PrimeField>(
    values: Vec<F>,
    domain: &GeneralEvaluationDomain<F>,
) -> DensePolynomial<F> {
    Evaluations::from_vec_and_domain(values, *domain).interpolate()
}

pub fn coset_evaluate<F: PrimeField>(
    poly: &DensePolynomial<F>,
    coset_domain: &GeneralEvaluationDomain<F>,
) -> Vec<F> {
    coset_domain.fft(&poly.coeffs)
}

pub fn fast_vanishing_poly<F: PrimeField>(n: usize) -> SparsePolynomial<F> {
    SparsePolynomial::from_coefficients_vec(vec![(0, -F::one()), (n, F::one())])
}

pub fn divide_by_vanishing<F: PrimeField>(
    poly: &DensePolynomial<F>,
    domain: &GeneralEvaluationDomain<F>,
) -> DensePolynomial<F> {
    let n = domain.size();
    let deg = poly.degree();

    let mut q_coeffs = vec![F::zero(); deg - n + 1];
    let mut coeffs = poly.coeffs.clone();

    for i in (0..=deg - n).rev() {
        q_coeffs[i] = coeffs[i + n];
        coeffs[i] += q_coeffs[i];
    }

    DensePolynomial::from_coefficients_vec(q_coeffs)
}
