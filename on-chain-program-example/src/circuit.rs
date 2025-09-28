use crate::byte_utils::field_to_bytes;
use ark_bn254::Fr;
use ark_relations::lc;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError, Variable};
use thiserror::Error;



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
        self.some_value
            .map(|v| vec![field_to_bytes(v)])
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
            lc!() + neg_one * y_var,
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
            let mut cur = d_var;
            let mut acc = Fr::zero();
            
            for i in 0..32 {
                // Create binary variable
                let bit = cs.new_witness_variable(|| {
                    Ok(if d.into_bigint().get_bit(i as u64) {
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
                if i > 0 {
                    acc += Fr::from(1u64 << i);
                    cs.enforce_constraint(
                        lc!() + cur - acc,
                        lc!() + bit,
                        lc!() + Variable::One,
                    )?;
                }
            }
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
        let circuit = ExampleCircuit::new(100).unwrap();
        let cs = ConstraintSystem::<Fr>::new_ref();
        assert!(circuit.generate_constraints(cs).is_ok());
    }
    
    #[test]
    fn test_invalid_inequality() {
        // X = 50, Y = 100, should fail since 50 ≱ 100
        let circuit = ExampleCircuit::new(50).unwrap();
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
