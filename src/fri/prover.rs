use std::ops::Mul;

use ark_ff::PrimeField;
use ark_poly::{DenseUVPolynomial, Polynomial, univariate::DensePolynomial};

use crate::{
    crypto::{merkle::MerkleProof, transcript::Transcript},
    fri::layer::FriLayer,
};

/// Authentication data for one query through all FRI layers
#[derive(Clone, Debug, Default)]
pub struct Decommitment<F: PrimeField> {
    pub evaluations: Vec<F>, // f(x_i) at each layer
    pub auth_paths: Vec<MerkleProof<F>>,
    pub sym_evaluations: Vec<F>, // f(-x_i) at each layer (symmetric point)
    pub sym_auth_paths: Vec<MerkleProof<F>>,
}

#[derive(Clone, Debug)]
pub struct FriProof<F: PrimeField> {
    pub domain_size: usize,
    pub coset: F,
    pub number_of_queries: usize,
    pub layers_root: Vec<F>, // Merkle roots of each layer
    pub const_val: F,        // Final constant value
    pub decommitment_list: Vec<Decommitment<F>>,
}

pub fn generate_proof<F: PrimeField>(
    poly: DensePolynomial<F>,
    blowup_factor: usize,
    number_of_queries: usize,
) -> FriProof<F> {
    let domain_size = (poly.degree() + 1).next_power_of_two() * blowup_factor;
    let coset = F::GENERATOR;
    let mut transcript = Transcript::new(F::zero());

    let mut layers = Vec::new();
    let mut layers_root = Vec::new();
    let mut curr_poly = poly.clone();
    let mut curr_coset = coset;
    let mut curr_domain_size = domain_size;

    while curr_poly.degree() != 0 {
        let layer = FriLayer::from_poly(&curr_poly, curr_coset, curr_domain_size);
        let root = layer.merkle_tree.root();

        layers.push(layer);
        layers_root.push(root);
        transcript.digest(root);

        let random_r = transcript.generate_a_challenge();
        curr_poly = fold_polynomial(&curr_poly, random_r);
        curr_coset = curr_coset.square();
        curr_domain_size /= 2;
    }

    let const_val = curr_poly.coeffs[0];
    transcript.digest(const_val);

    let query_indices = transcript.generate_challenge_list_usize(number_of_queries, domain_size);
    let mut decommitment_list = Vec::new();
    query_indices.into_iter().for_each(|query_idx| {
        let mut decommitment = Decommitment::default();
        let mut curr_idx = query_idx;

        layers.iter().for_each(|layer| {
            let sym_idx = (curr_idx + layer.domain_size / 2) % layer.domain_size;
            decommitment.evaluations.push(layer.evaluations[curr_idx]);
            decommitment
                .auth_paths
                .push(layer.merkle_tree.generate_proof(curr_idx));
            decommitment
                .sym_evaluations
                .push(layer.evaluations[sym_idx]);
            decommitment
                .sym_auth_paths
                .push(layer.merkle_tree.generate_proof(sym_idx));

            curr_idx %= layer.domain_size / 2;
        });

        decommitment_list.push(decommitment);
    });

    FriProof {
        domain_size,
        coset,
        number_of_queries,
        layers_root,
        const_val,
        decommitment_list,
    }
}

pub fn fold_polynomial<F: PrimeField>(poly: &DensePolynomial<F>, r: F) -> DensePolynomial<F> {
    let coeffs = poly.coeffs.clone();
    let even_coeffs = coeffs.iter().step_by(2).cloned().collect::<Vec<_>>();
    let odd_coeffs = coeffs
        .iter()
        .skip(1)
        .step_by(2)
        .cloned()
        .collect::<Vec<_>>();

    let even_poly = DensePolynomial::from_coefficients_vec(even_coeffs);
    let odd_poly = DensePolynomial::from_coefficients_vec(odd_coeffs);

    even_poly + odd_poly.mul(r)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::Fq;

    #[test]
    fn test_fold_polynomial() {
        // f(x) = 1 + 2x + 3x² + 4x³
        // even = 1 + 3x,  odd = 2 + 4x
        // fold with r=1: (1+3x) + 1*(2+4x) = 3 + 7x
        let poly = DensePolynomial::from_coefficients_vec(vec![
            Fq::from(1),
            Fq::from(2),
            Fq::from(3),
            Fq::from(4),
        ]);
        let folded = fold_polynomial(&poly, Fq::from(1));
        let expected = DensePolynomial::from_coefficients_vec(vec![Fq::from(3), Fq::from(7)]);
        assert_eq!(folded, expected);
    }
}
