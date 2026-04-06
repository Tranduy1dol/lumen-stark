use ark_ff::PrimeField;
use ark_poly::{
    EvaluationDomain, GeneralEvaluationDomain, Polynomial, univariate::DensePolynomial,
};

use crate::{crypto::merkle::MerkleTree, polynomial::domain};

pub struct FriLayer<F: PrimeField> {
    pub evaluations: Vec<F>,
    pub merkle_tree: MerkleTree<F>,
    pub domain_size: usize,
}

impl<F: PrimeField> FriLayer<F> {
    pub fn from_poly(poly: &DensePolynomial<F>, coset: F, domain_size: usize) -> Self {
        let domain: GeneralEvaluationDomain<F> = domain(domain_size);
        let evaluations = domain
            .elements()
            .map(|root| {
                let cur1 = root * coset;
                poly.evaluate(&cur1)
            })
            .collect::<Vec<_>>();
        let merkle_tree = MerkleTree::new(evaluations.clone());

        Self {
            evaluations,
            merkle_tree,
            domain_size,
        }
    }
}
