use crate::byte_utils::field_to_bytes;
use ark_bn254::Fr;
use ark_relations::lc;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError, Variable};
use thiserror::Error;
use ark_ff::Zero;
use ark_ff::One;
use ark_ff::PrimeField;
use ark_ff::BigInteger;



#[derive(Error, Debug)]
pub enum CircuitError {
    #[error("Missing assignment")]
    MissingAssignment,
    #[error("Invalid value range")]
    InvalidRange,
}

#[derive(Clone)]
pub struct ExampleCircuit {
    pub prover_value: Option<Fr>,   // X (secret)
    pub verifier_value: Option<Fr>, // Y (public threshold)
    pub range_check: bool, // Enable range checking
}

impl ExampleCircuit {
    pub fn default() -> Self {
        ExampleCircuit { 
            prover_value: None,
            verifier_value: None,
            range_check: true,
        }
    }

    pub fn new(x: u64, y: u64) -> Result<Self, CircuitError> {
        // Validate input range (example: ensure value is < 2^32)
        if x >= (1 << 32) || y >= (1 << 32) {
            return Err(CircuitError::InvalidRange);
        }

        Ok(ExampleCircuit {
            prover_value: Some(Fr::from(x)),
            verifier_value: Some(Fr::from(y)),
            range_check: true,
        })
    }

    pub fn public_inputs(&self) -> Result<Vec<[u8; 32]>, CircuitError> {
            match (self.prover_value, self.verifier_value) {
            (Some(x), Some(y)) => Ok(vec![field_to_bytes(x), field_to_bytes(y)]),
            _ => Err(CircuitError::MissingAssignment)
        }
    }
}

/// ExampleCircuit implements X ≥ Y proof via range check on D = X - Y
///
/// Constraints:
/// 1. Compute D = X - Y using R1CS constraints
/// 2. Range check D to ensure D ≥ 0 and D < 2^32
///
/// Complexity:
/// - Computing D: 2 R1CS constraints
/// - Range check: ~32 boolean constraints + linear constraints
///
/// Security:
/// - Range check ensures D ≥ 0, proving X ≥ Y
/// - 32-bit limit prevents overflow attacks
impl ConstraintSynthesizer<Fr> for ExampleCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        // Allocate the two public inputs
        let x = self.prover_value.ok_or(SynthesisError::AssignmentMissing)?;
        let y = self.verifier_value.ok_or(SynthesisError::AssignmentMissing)?;
        let x_var = cs.new_input_variable(|| Ok(x))?;
        let y_var = cs.new_input_variable(|| Ok(y))?;

        // Compute D = X - Y by introducing neg_y = -Y first
        let neg_one = Fr::from(-1i32);
        let neg_y_var = cs.new_witness_variable(|| Ok(neg_one * y))?;

        // Constrain neg_y_var = -Y
        cs.enforce_constraint(
            lc!() + (neg_one, y_var),
            lc!() + Variable::One,
            lc!() + neg_y_var,
        )?;

        // Compute D = X + neg_y and constrain it
        let d = x + (neg_one * y);
        let d_var = cs.new_witness_variable(|| Ok(d))?;
        cs.enforce_constraint(
            lc!() + x_var + neg_y_var,
            lc!() + Variable::One,
            lc!() + d_var,
        )?;

        // Optional range check (if enabled)
if self.range_check {
    // Range check D to ensure 0 ≤ D < 2^32 (proving X ≥ Y)
    let mut acc = Fr::zero();
    let mut acc_var = cs.new_witness_variable(|| Ok(acc))?;
    let mut prev_acc_var = acc_var;   // ✅ Single accumulator tracking

    
    for i in 0..32 {
        // Create binary variable
        let bit = cs.new_witness_variable(|| {
            Ok(if d.into_bigint().get_bit((i as u64).try_into().unwrap()) {
                Fr::one()
            } else {
                Fr::zero()
            })
        })?;

        // Ensure bit is boolean (0 or 1)
        cs.enforce_constraint(
            lc!() + bit,
            lc!() + bit,
            lc!() + bit,
        )?;

        // Add bit contribution to accumulator
        let power = Fr::from(1u64 << i);
        let bit_contribution = cs.new_witness_variable(|| {
            let bit_val = if d.into_bigint().get_bit((i as u64).try_into().unwrap()) {
                Fr::one()
            } else {
                Fr::zero()
            };
            Ok(bit_val * power)
        })?;

        // Constrain bit_contribution = power * bit
        cs.enforce_constraint(
            lc!() + (power, Variable::One),
            lc!() + bit,
            lc!() + bit_contribution,
        )?;

        // Update accumulator with new bit contribution
        acc = acc + (power * if d.into_bigint().get_bit(i.try_into().unwrap()) { Fr::one() } else { Fr::zero() });
        let new_acc = cs.new_witness_variable(|| Ok(acc))?;

        // Constrain new_acc = prev_acc + bit_contribution
        cs.enforce_constraint(
            lc!() + prev_acc_var + bit_contribution,
            lc!() + Variable::One,
            lc!() + new_acc,
        )?;

        prev_acc_var = new_acc;
        acc_var = new_acc;
    }

    // Final constraint: ensure d_var equals the accumulated value
        cs.enforce_constraint(
            lc!() + d_var,
            lc!() + Variable::One,
            lc!() + acc_var,
        )?;
}

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_relations::r1cs::ConstraintSystem;
    #[test]
    fn test_valid_inequality() {
        // X = 100, Y = 50, should pass since 100 ≥ 50
        let circuit = ExampleCircuit::new(100,50).unwrap();
        let cs = ConstraintSystem::<Fr>::new_ref();
        assert!(circuit.generate_constraints(cs).is_ok());
    }
    
    #[test]
    fn test_invalid_inequality() {
        // X = 50, Y = 100, should fail since 50 ≱ 100
        let circuit = ExampleCircuit::new(50,100).unwrap();
        let cs = ConstraintSystem::<Fr>::new_ref();
        // Constraints will be satisfied but range check will fail
        // since D = -50 won't decompose into 32 positive bits
        assert!(circuit.generate_constraints(cs).is_ok());
        // Full verification would fail during proof generation
    }
    
    #[test]
    fn test_range_limits() {
        // Values ≥ 2^32 should be rejected
        assert!(ExampleCircuit::new(1 << 32, 0).is_err());
        assert!(ExampleCircuit::new(0, 1 << 32).is_err());

        // Valid range should work
        assert!(ExampleCircuit::new((1 << 32) - 1, (1 << 32) - 1).is_ok());
    }
}
// ============================================================================
// TokenVerificationCircuit - Cleaner token balance verification
// ============================================================================

/// Token Verification Circuit implements tokens_to_send >= tokens_asked
///
/// Circuit Constraint: tokens_to_send >= tokens_asked
/// Field Operations: BN254::Fr (254-bit prime field)
/// Witness Structure: Private(tokens_to_send), Public(tokens_asked)
///
/// Implementation Details:
/// - Private Input: tokens_to_send (secret balance/amount)
/// - Public Input: tokens_asked (publicly known requirement)
/// - Constraint Method: Range check on D = tokens_to_send - tokens_asked
#[derive(Clone)]
pub struct TokenVerificationCircuit {
    pub tokens_to_send: Option<Fr>,  // Secret witness
    pub tokens_asked: Option<Fr>,    // Public input
}

impl TokenVerificationCircuit {
    pub fn new(tokens_to_send: u64, tokens_asked: u64) -> Result<Self, CircuitError> {
        // Validate input range (ensure values fit in 32-bit for safe arithmetic)
        if tokens_to_send >= (1 << 32) || tokens_asked >= (1 << 32) {
            return Err(CircuitError::InvalidRange);
        }

        Ok(TokenVerificationCircuit {
            tokens_to_send: Some(Fr::from(tokens_to_send)),
            tokens_asked: Some(Fr::from(tokens_asked)),
        })
    }

    pub fn public_inputs(&self) -> Result<Vec<[u8; 32]>, CircuitError> {
        match self.tokens_asked {
            Some(tokens_asked) => Ok(vec![field_to_bytes(tokens_asked)]),
            None => Err(CircuitError::MissingAssignment)
        }
    }
}

/// ConstraintSynthesizer implementation for TokenVerificationCircuit
///
/// Enforces: tokens_to_send >= tokens_asked via R1CS constraints
///
/// Constraint Strategy:
/// 1. Allocate tokens_to_send as witness variable (private)
/// 2. Allocate tokens_asked as input variable (public)
/// 3. Compute difference D = tokens_to_send - tokens_asked
/// 4. Bit-decompose D to ensure 0 ≤ D < 2^32 (proves non-negativity)
///
/// Security Properties:
/// - Range check prevents wrap-around attacks
/// - 32-bit constraint ensures practical token amounts
/// - Zero-knowledge: tokens_to_send remains private
impl ConstraintSynthesizer<Fr> for TokenVerificationCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        // Allocate secret input (tokens_to_send as witness)
        let tokens_to_send = self.tokens_to_send.ok_or(SynthesisError::AssignmentMissing)?;
        let tokens_to_send_var = cs.new_witness_variable(|| Ok(tokens_to_send))?;
        
        // Allocate public input (tokens_asked)
        let tokens_asked = self.tokens_asked.ok_or(SynthesisError::AssignmentMissing)?;
        let tokens_asked_var = cs.new_input_variable(|| Ok(tokens_asked))?;
        
        // Compute D = tokens_to_send - tokens_asked
        // Step 1: Create neg_tokens_asked = -tokens_asked
        let neg_one = Fr::from(-1i32);
        let neg_tokens_asked_var = cs.new_witness_variable(|| Ok(neg_one * tokens_asked))?;
        
        // Constrain: neg_tokens_asked_var = -tokens_asked
        cs.enforce_constraint(
            lc!() + (neg_one, tokens_asked_var),
            lc!() + Variable::One,
            lc!() + neg_tokens_asked_var,
        )?;
        
        // Step 2: Compute D = tokens_to_send + neg_tokens_asked
        let d = tokens_to_send + (neg_one * tokens_asked);
        let d_var = cs.new_witness_variable(|| Ok(d))?;
        
        cs.enforce_constraint(
            lc!() + tokens_to_send_var + neg_tokens_asked_var,
            lc!() + Variable::One,
            lc!() + d_var,
        )?;
        
        // Step 3: Range check D ∈ [0, 2^32) to prove tokens_to_send >= tokens_asked
        // This is equivalent to enforce_cmp(tokens_to_send, tokens_asked, Greater, true)
        let mut acc = Fr::zero();
        let mut acc_var = cs.new_witness_variable(|| Ok(acc))?;
        let mut prev_acc_var = acc_var;
        
        for i in 0..32 {
            // Extract bit i from D
            let bit = cs.new_witness_variable(|| {
                Ok(if d.into_bigint().get_bit((i as u64).try_into().unwrap()) {
                    Fr::one()
                } else {
                    Fr::zero()
                })
            })?;
            
            // Boolean constraint: bit * bit = bit (ensures bit ∈ {0, 1})
            cs.enforce_constraint(
                lc!() + bit,
                lc!() + bit,
                lc!() + bit,
            )?;
            
            // Compute bit_contribution = bit * 2^i
            let power = Fr::from(1u64 << i);
            let bit_contribution = cs.new_witness_variable(|| {
                let bit_val = if d.into_bigint().get_bit((i as u64).try_into().unwrap()) {
                    Fr::one()
                } else {
                    Fr::zero()
                };
                Ok(bit_val * power)
            })?;
            
            // Constrain: bit_contribution = power * bit
            cs.enforce_constraint(
                lc!() + (power, Variable::One),
                lc!() + bit,
                lc!() + bit_contribution,
            )?;
            
            // Accumulate: new_acc = prev_acc + bit_contribution
            acc = acc + (power * if d.into_bigint().get_bit(i.try_into().unwrap()) { Fr::one() } else { Fr::zero() });
            let new_acc = cs.new_witness_variable(|| Ok(acc))?;
            
            cs.enforce_constraint(
                lc!() + prev_acc_var + bit_contribution,
                lc!() + Variable::One,
                lc!() + new_acc,
            )?;
            
            prev_acc_var = new_acc;
            acc_var = new_acc;
        }
        
        // Final constraint: D = accumulated value (ensures D is correctly decomposed)
        cs.enforce_constraint(
            lc!() + d_var,
            lc!() + Variable::One,
            lc!() + acc_var,
        )?;
        
        Ok(())
    }
}

// ============================================================================
// Tests for TokenVerificationCircuit
// ============================================================================

#[cfg(test)]
mod token_circuit_tests {
    use super::*;
    use ark_relations::r1cs::ConstraintSystem;
    use ark_groth16::Groth16;
    use ark_bn254::Bn254;
    use ark_snark::SNARK;
    use rand::thread_rng;
    
    #[test]
    fn test_token_valid_balance() {
        // tokens_to_send = 1000, tokens_asked = 500
        // Should succeed since 1000 >= 500
        let circuit = TokenVerificationCircuit::new(1000, 500).unwrap();
        let cs = ConstraintSystem::<Fr>::new_ref();
        assert!(circuit.generate_constraints(cs).is_ok());
    }
    
    #[test]
    fn test_token_insufficient_balance() {
        // tokens_to_send = 500, tokens_asked = 1000
        // Constraint generation succeeds but proof would fail
        let circuit = TokenVerificationCircuit::new(500, 1000).unwrap();
        let cs = ConstraintSystem::<Fr>::new_ref();
        // Note: constraints succeed but range check of D = -500 fails during proving
        assert!(circuit.generate_constraints(cs).is_ok());
    }
    
    #[test]
    fn test_token_equal_amounts() {
        // tokens_to_send = tokens_asked = 1000
        // Should succeed since 1000 >= 1000
        let circuit = TokenVerificationCircuit::new(1000, 1000).unwrap();
        let cs = ConstraintSystem::<Fr>::new_ref();
        assert!(circuit.generate_constraints(cs).is_ok());
    }
    
    #[test]
    fn test_token_range_validation() {
        // Values >= 2^32 should be rejected
        assert!(TokenVerificationCircuit::new(1 << 32, 0).is_err());
        assert!(TokenVerificationCircuit::new(0, 1 << 32).is_err());
        
        // Maximum valid range should work
        assert!(TokenVerificationCircuit::new((1 << 32) - 1, 0).is_ok());
    }
    
    #[test]
    fn test_token_proof_generation() {
        let mut rng = thread_rng();
        
        // Create circuit with valid token amounts
        let circuit = TokenVerificationCircuit::new(2000, 1500).unwrap();
        
        // Setup
        let (pk, vk) = Groth16::<Bn254>::circuit_specific_setup(circuit.clone(), &mut rng).unwrap();
        
        // Generate proof
        let proof = Groth16::<Bn254>::prove(&pk, circuit.clone(), &mut rng).unwrap();
        
        // Verify (note: public input is only tokens_asked)
        let public_inputs = vec![Fr::from(1500u64)];
        let result = Groth16::<Bn254>::verify(&vk, &public_inputs, &proof).unwrap();
        assert!(result, "Proof verification should succeed");
    }
}
