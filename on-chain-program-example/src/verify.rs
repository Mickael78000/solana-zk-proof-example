use ark_bn254::{Bn254, G1Projective};
use ark_groth16::{prepare_verifying_key, Groth16, Proof, VerifyingKey};
use thiserror::Error;
use crate::prove::ProofPackage;
use ark_ec::CurveGroup; 

#[derive(Error, Debug)]
pub enum VerificationError {
    #[error("Invalid proof format")]
    InvalidProof,
    #[error("Invalid public input")]
    InvalidPublicInput,
    #[error("Verification failed")]
    VerificationFailed,
}

pub fn verify(
    proof: &Proof<Bn254>,
    public_inputs: &G1Projective,
    vk: &VerifyingKey<Bn254>,
) -> Result<bool, VerificationError> {
    // Validate inputs
    if !is_valid_point(public_inputs) {
        return Err(VerificationError::InvalidPublicInput);
    }
    
    if !is_valid_proof(proof) {
        return Err(VerificationError::InvalidProof);
    }

    let pvk = prepare_verifying_key(vk);
    Groth16::<Bn254>::verify_proof_with_prepared_inputs(&pvk, proof, public_inputs)
        .map_err(|_| VerificationError::VerificationFailed)
}

pub fn verify_proof_package(proof_package: &ProofPackage) -> Result<bool, VerificationError> {
    if !is_valid_proof(&proof_package.proof) {
        return Err(VerificationError::InvalidProof);
    }

    if !is_valid_point(&proof_package.public_inputs) {
        return Err(VerificationError::InvalidPublicInput);
    }

    Groth16::<Bn254>::verify_proof_with_prepared_inputs(
        &proof_package.prepared_verifying_key,
        &proof_package.proof,
        &proof_package.public_inputs,
    )
    .map_err(|_| VerificationError::VerificationFailed)
}

fn is_valid_point(point: &G1Projective) -> bool {
    let affine = point.into_affine();
    affine.is_on_curve() && affine.is_in_correct_subgroup_assuming_on_curve()
}

fn is_valid_proof(proof: &Proof<Bn254>) -> bool {
    // Validate each component of the proof is on curve and in correct subgroup
    proof.a.is_on_curve() && 
    proof.a.is_in_correct_subgroup_assuming_on_curve() &&
    proof.b.is_on_curve() && 
    proof.b.is_in_correct_subgroup_assuming_on_curve() &&
    proof.c.is_on_curve() && 
    proof.c.is_in_correct_subgroup_assuming_on_curve()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bn254::{Fr, G1Affine, G2Affine};
    use ark_ec::AffineRepr;
    use ark_std::UniformRand;
    use ark_std::rand::thread_rng;

    fn generate_random_proof() -> Proof<Bn254> {
        let mut rng = thread_rng();
    
        Proof {
            a: G1Affine::rand(&mut rng).into(),
            b: G2Affine::rand(&mut rng).into(),
            c: G1Affine::rand(&mut rng).into(),
        }
    }

    fn generate_invalid_proof() -> Proof<Bn254> {
        // Generate a proof with points not on the curve
        Proof {
            a: G1Affine::zero().into(), // Invalid - zero point
            b: G2Affine::zero().into(), // Invalid - zero point
            c: G1Affine::zero().into(), // Invalid - zero point
        }
    }

    #[test]
    fn test_valid_point_check() {
        let mut rng = thread_rng();
        let valid_point = G1Affine::rand(&mut rng).into();
        assert!(is_valid_point(&valid_point));
    }

    #[test]
    fn test_invalid_point_check() {
        let invalid_point = G1Affine::zero().into();
        assert!(!is_valid_point(&invalid_point));
    }

    #[test]
    fn test_valid_proof_validation() {
        let proof = generate_random_proof();
        assert!(is_valid_proof(&proof));
    }

    #[test]
    fn test_invalid_proof_validation() {
        let proof = generate_invalid_proof();
        assert!(!is_valid_proof(&proof));
    }

    #[test]
    fn test_verify_with_invalid_proof() {
        let mut rng = thread_rng();
        let proof = generate_invalid_proof();
        let public_input = G1Affine::rand(&mut rng).into();
        let vk = VerifyingKey::<Bn254>::default();

        let result = verify(&proof, &public_input, &vk);
        assert!(matches!(result, Err(VerificationError::InvalidProof)));
    }

    #[test]
    fn test_verify_with_invalid_public_input() {
        let proof = generate_random_proof();
        let public_input = G1Affine::zero().into();
        let vk = VerifyingKey::<Bn254>::default();

        let result = verify(&proof, &public_input, &vk);
        assert!(matches!(result, Err(VerificationError::InvalidPublicInput)));
    }
}
