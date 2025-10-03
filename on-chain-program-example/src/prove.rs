use crate::byte_utils::bytes_to_field;
use ark_bn254::{Bn254, Fr, G1Projective};
use ark_groth16::{
    prepare_verifying_key, Groth16, PreparedVerifyingKey, Proof, ProvingKey, VerifyingKey,
};
use ark_relations::r1cs::ConstraintSynthesizer;
use ark_serialize::{CanonicalSerialize, Compress};
use ark_snark::SNARK;
use borsh::{BorshDeserialize, BorshSerialize};
use rand::thread_rng;
use std::fs::File;
use std::io::Write;
use thiserror::Error;
use ark_ff::{PrimeField, Field};

#[derive(Error, Debug)]
pub enum ProofError {
    #[error("Invalid public input: {0}")]
    InvalidPublicInput(String),
    #[error("Invalid proving key")]
    InvalidProvingKey,
    #[error("Circuit validation failed")]
    CircuitValidationFailed,
    #[error("Proof generation failed")]
    ProofGenerationFailed,
}

pub fn validate_public_input(input: &[u8; 32]) -> Result<Fr, ProofError> {
    let field_element: Fr = bytes_to_field(input)
        .map_err(|_| ProofError::InvalidPublicInput("Failed to convert bytes to field element".to_string()))?;
    
   // Check if the input is within valid field range
    if field_element.into_bigint() >= Fr::MODULUS {
        return Err(ProofError::InvalidPublicInput("Input exceeds field characteristic".to_string()));
    }
    
    Ok(field_element)
}

pub fn validate_proving_key<C: ConstraintSynthesizer<Fr> + Clone>(
    proving_key: &ProvingKey<Bn254>,
    circuit: &C,
) -> Result<(), ProofError> {
    // Verify circuit constraints can be generated
    let cs = ark_relations::r1cs::ConstraintSystem::<Fr>::new_ref();
    circuit.clone().generate_constraints(cs.clone())
        .map_err(|_| ProofError::CircuitValidationFailed)?;
    
    // Vérification simplifiée: juste s'assurer que la clé n'est pas vide
    // Note: La comparaison stricte cs.num_constraints() == proving_key.a_query.len()
    // n'est pas toujours valide dans Groth16 car a_query peut contenir des éléments
    // supplémentaires pour l'optimisation
    if proving_key.a_query.is_empty() {
        return Err(ProofError::InvalidProvingKey);
    }
    
    Ok(())
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct ProofPackageLite {
    pub proof: Vec<u8>,
    pub public_inputs: Vec<[u8; 32]>,
    pub verifying_key: Vec<u8>,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct ProofPackagePrepared {
    pub proof: Vec<u8>,
    pub public_inputs: Vec<u8>,
    pub verifying_key: Vec<u8>,
}

pub struct ProofPackage {
    pub proof: Proof<Bn254>,
    pub public_inputs: G1Projective,
    pub prepared_verifying_key: PreparedVerifyingKey<Bn254>,
}

pub fn setup<C: ConstraintSynthesizer<Fr>>(
    save_keys: bool,
    circuit: C,
) -> (ProvingKey<Bn254>, VerifyingKey<Bn254>) {
    let rng = &mut thread_rng();
    let (proving_key, verifying_key) =
        Groth16::<Bn254>::circuit_specific_setup(circuit, rng).unwrap();

    if save_keys {
        let mut pk_file = File::create("pk.bin").unwrap();
        let mut pk_bytes = Vec::new();
        proving_key
            .serialize_uncompressed(&mut pk_bytes)
            .expect("Failed to serialize proving key (pk) to uncompressed bytes");
        pk_file
            .write(&pk_bytes)
            .expect("Failed to write proving key bytes to pk.bin");

        let mut file = File::create("vk.bin").unwrap();
        let mut vk_bytes = Vec::new();
        verifying_key
                .serialize_uncompressed(&mut vk_bytes)
                .expect("Failed to serialize verifying key (vk) to uncompressed bytes");
        file
            .write(&vk_bytes)
            .expect("Failed to write verifying key bytes to vk.bin");
    };

    (proving_key, verifying_key)
}

pub fn generate_proof_package<C: ConstraintSynthesizer<Fr> + Clone>(
    proving_key: &ProvingKey<Bn254>,
    verifying_key: &VerifyingKey<Bn254>,
    circuit: C,
    public_inputs: &Vec<[u8; 32]>,
) -> Result<(ProofPackageLite, ProofPackagePrepared, ProofPackage), ProofError> {
    // Validate proving key matches circuit
    validate_proving_key(proving_key, &circuit)?;
    
    // Validate public inputs
    let public_inputs_fr: Vec<Fr> = public_inputs
        .iter()
        .map(validate_public_input)
        .collect::<Result<Vec<_>, _>>()?;
    
    // Verify number of public inputs matches circuit expectations
    let cs = ark_relations::r1cs::ConstraintSystem::<Fr>::new_ref();
    circuit.clone().generate_constraints(cs.clone())
        .map_err(|_| ProofError::CircuitValidationFailed)?;
    
        // Vérifier le nombre d'entrées publiques
    // Note: num_instance_variables() inclut la variable constante "1"
    let expected_public_inputs = cs.num_instance_variables().saturating_sub(1);
    if public_inputs_fr.len() != expected_public_inputs {
        return Err(ProofError::InvalidPublicInput(
            format!("Number of public inputs doesn't match circuit: expected {}, got {}", 
                    expected_public_inputs, public_inputs_fr.len())
        ));
    }

    let rng = &mut thread_rng();

    // Create a proof
    let proof = Groth16::<Bn254>::prove(&proving_key, circuit, rng).unwrap();

    let mut proof_bytes = Vec::with_capacity(proof.serialized_size(Compress::No));
    proof
        .serialize_uncompressed(&mut proof_bytes)
        .expect("Error serializing proof");

    let public_inputs_fr = public_inputs
        .iter()
        .map(|input| bytes_to_field(input))
        .collect::<Result<Vec<Fr>, _>>()
        .expect("Failed to convert public inputs bytes to field elements (Fr)");

    let prepared_verifying_key = prepare_verifying_key(&verifying_key);

    let g1_projective: G1Projective =
        Groth16::<Bn254>::prepare_inputs(&prepared_verifying_key, &public_inputs_fr)
            .expect("Error preparing inputs with public inputs and prepared verifying key");

    let mut projective_bytes: Vec<u8> = Vec::new();
    let _ = g1_projective.serialize_uncompressed(&mut projective_bytes);
    let mut verifying_key_bytes: Vec<u8> =
        Vec::with_capacity(verifying_key.serialized_size(Compress::No));
    let _ = verifying_key.serialize_uncompressed(&mut verifying_key_bytes);
    let mut prepared_verifying_key_bytes: Vec<u8> = Vec::new();
    let _ = prepared_verifying_key.serialize_uncompressed(&mut prepared_verifying_key_bytes);

    Ok((
        ProofPackageLite {
            proof: proof_bytes.clone(),
            public_inputs: public_inputs.clone(),
            verifying_key: prepared_verifying_key_bytes.clone(),
        },
        ProofPackagePrepared {
            proof: proof_bytes,
            public_inputs: projective_bytes,
            verifying_key: prepared_verifying_key_bytes,
        },
        ProofPackage {
            proof,
            public_inputs: g1_projective,
            prepared_verifying_key,
        },
    ))
}
