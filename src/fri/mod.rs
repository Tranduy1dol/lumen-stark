pub mod layer;
pub mod prover;
pub mod verifier;

#[cfg(test)]
mod tests {
    use ark_ff::Zero;
    use ark_poly::{DenseUVPolynomial, univariate::DensePolynomial};

    use crate::crypto::transcript::Transcript;
    use crate::field::Fq;
    use crate::fri::{prover::generate_proof, verifier::verify};

    #[test]
    fn test_fri_roundtrip_degree_3() {
        let poly = DensePolynomial::from_coefficients_vec(vec![
            Fq::from(1),
            Fq::from(2),
            Fq::from(3),
            Fq::from(4),
        ]);
        let mut prover_transcript = Transcript::new(Fq::zero());
        let proof = generate_proof(poly, 2, 2, &mut prover_transcript);

        let mut verifier_transcript = Transcript::new(Fq::zero());
        assert!(verify(&proof, &mut verifier_transcript).is_ok());
    }

    #[test]
    fn test_fri_roundtrip_degree_5() {
        let poly = DensePolynomial::from_coefficients_vec(vec![
            Fq::from(1),
            Fq::from(2),
            Fq::from(3),
            Fq::from(4),
            Fq::from(5),
            Fq::from(6),
        ]);
        let mut prover_transcript = Transcript::new(Fq::zero());
        let proof = generate_proof(poly, 2, 2, &mut prover_transcript);

        let mut verifier_transcript = Transcript::new(Fq::zero());
        assert!(verify(&proof, &mut verifier_transcript).is_ok());
    }

    #[test]
    fn test_fri_soundness() {
        let poly = DensePolynomial::from_coefficients_vec(vec![
            Fq::from(1),
            Fq::from(2),
            Fq::from(3),
            Fq::from(4),
        ]);
        let mut prover_transcript = Transcript::new(Fq::zero());
        let mut proof = generate_proof(poly, 2, 2, &mut prover_transcript);
        proof.const_val -= Fq::from(1); // tamper!

        let mut verifier_transcript = Transcript::new(Fq::zero());
        assert!(verify(&proof, &mut verifier_transcript).is_err());
    }
}
