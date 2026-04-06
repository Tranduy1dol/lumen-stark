use ark_ff::PrimeField;
use ark_poly::{EvaluationDomain, GeneralEvaluationDomain, Polynomial};

use crate::polynomial::domain;

pub struct PreprocessedDomain<F: PrimeField> {
    pub trace_domain: GeneralEvaluationDomain<F>,
    pub eval_domain: GeneralEvaluationDomain<F>,
    pub vanishing_evals: Vec<F>,
}

impl<F: PrimeField> PreprocessedDomain<F> {
    pub fn new(trace_length: usize, blowup_factor: usize) -> Self {
        let trace_domain = domain(trace_length);
        let eval_length = trace_length * blowup_factor;
        let eval_domain = domain(eval_length);

        // Precompute vanishing poly evaluations on eval domain
        let vanishing = trace_domain.vanishing_polynomial();
        let vanishing_evals = eval_domain
            .elements()
            .map(|x| vanishing.evaluate(&x))
            .collect();

        Self {
            trace_domain,
            eval_domain,
            vanishing_evals,
        }
    }
}
