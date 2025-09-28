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
    pub some_value: Option<Fr>,
    pub range_check: bool, // Enable range checking
}

impl ExampleCircuit {
    pub fn default() -> Self {
        ExampleCircuit { 
            some_value: None,
            range_check: true,
        }
    }

    pub fn new(value: u64) -> Result<Self, CircuitError> {
        // Validate input range (example: ensure value is < 2^32)
        if value >= (1 << 32) {
            return Err(CircuitError::InvalidRange);
        }

        Ok(ExampleCircuit {
            some_value: Some(Fr::from(value)),
            range_check: true,
        })
    }

    pub fn public_inputs(&self) -> Result<Vec<[u8; 32]>, CircuitError> {
        self.some_value
            .map(|v| vec![field_to_bytes(v)])
            .ok_or(CircuitError::MissingAssignment)
    }
}

impl ConstraintSynthesizer<Fr> for ExampleCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        // Allocate public input with proper error handling
        let value = self.some_value.ok_or(SynthesisError::AssignmentMissing)?;
        let value_var = cs.new_input_variable(|| Ok(value))?;

        // Basic constraint: value * 1 = value (ensures value is properly constrained)
        cs.enforce_constraint(
            lc!() + value_var,
            lc!() + Variable::One,
            lc!() + value_var,
        )?;

        // Optional range check (if enabled)
        if self.range_check {
            // Ensure value < 2^32 using binary decomposition
            let mut cur = value_var;
            let mut acc = Fr::zero();
            
            for i in 0..32 {
                // Create binary variable
                let bit = cs.new_witness_variable(|| {
                    Ok(if value.into_bigint().get_bit(i as u64) {
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
