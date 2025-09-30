use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, 
    entrypoint, 
    entrypoint::ProgramResult, 
    instruction::Instruction,
    msg, 
    program::invoke, 
    program_error::ProgramError, 
    pubkey::Pubkey,
    clock::Clock,
    sysvar::Sysvar,
};
use thiserror::Error;

// BN254 alt_bn128 pairing syscall program ID
pub const ALT_BN128_PAIRING: Pubkey = Pubkey::new_from_array([
    2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
]);

// Program entrypoint
entrypoint!(process_instruction);

#[derive(BorshSerialize, BorshDeserialize)]
pub enum ProgramInstruction {
    VerifyProof(Groth16VerifierPrepared),
    VerifyProofWithBalance {
        proof_data: Groth16VerifierPrepared,
        required_balance: u64,
        account_to_check: Pubkey,
    },
}

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = ProgramInstruction::try_from_slice(instruction_data)?;
    match instruction {
        ProgramInstruction::VerifyProof(proof_package) => verify_proof(accounts, proof_package),
        ProgramInstruction::VerifyProofWithBalance {
            proof_data,
            required_balance,
            account_to_check,
        } => verify_proof_with_balance(accounts, proof_data, required_balance, account_to_check),
    }
}

fn verify_proof(accounts: &[AccountInfo], mut groth16_verifier_prepared: Groth16VerifierPrepared) -> ProgramResult {
    let result = groth16_verifier_prepared.verify(accounts).map_err(|e| {
        msg!("Verification error: {:?}", e);
        ProgramError::InvalidAccountData
    })?;

    if result {
        msg!("Proof is valid! Inputs verified.");
        
        // Update state if a state account is provided
        if accounts.is_empty() {
            return Err(ProgramError::NotEnoughAccountKeys);
        }
        update_on_chain_state_with_amount(
            &accounts[0], 
            &groth16_verifier_prepared.prepared_public_inputs
        )?;
        Ok(())
    } else {
        msg!("Proof is invalid!");
        Err(ProgramError::InvalidAccountData)
    }
}

fn verify_proof_with_balance(
    accounts: &[AccountInfo],
    mut groth16_verifier_prepared: Groth16VerifierPrepared,
    required_balance: u64,
    account_to_check: Pubkey,
) -> ProgramResult {
    let result = groth16_verifier_prepared.verify(accounts).map_err(|e| {
        msg!("Verification error: {:?}", e);
        ProgramError::InvalidAccountData
    })?;

    if result {
        msg!("Proof is valid! Inputs verified.");

        let account_to_check_info = accounts.iter().find(|a| a.key == &account_to_check)
            .ok_or(ProgramError::InvalidAccountData)?;

        let account_balance_rc = account_to_check_info.lamports.clone();
        let account_balance: u64 = **account_balance_rc.borrow();

        if account_balance >= required_balance {
            msg!("Account balance is sufficient.");
            
            // Update state if a state account is provided (first account)
            if !accounts.is_empty() {
                update_on_chain_state_with_amount(
                    &accounts[0], 
                    &groth16_verifier_prepared.prepared_public_inputs
                )?;
            } else {
                update_on_chain_state()?;
            }
            Ok(())
        } else {
            msg!("Account balance is insufficient.");
            Err(ProgramError::InsufficientFunds)
        }
    } else {
        msg!("Proof is invalid!");
        Err(ProgramError::InvalidAccountData)
    }
}

fn update_on_chain_state() -> ProgramResult {
    msg!("Updating on-chain state...");
    Ok(())
}

#[derive(PartialEq, Eq, Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct Groth16VerifyingKeyPrepared {
    pub vk_alpha_g1: [u8; 64],
    pub vk_beta_g2: [u8; 128],
    pub vk_gamma_g2: [u8; 128],
    pub vk_delta_g2: [u8; 128],
}

/// Verification State - Tracks proof verification history
#[derive(BorshSerialize, BorshDeserialize)]
pub struct VerificationState {
    pub total_verifications: u64,
    pub last_amount: [u8; 32],  // Last verified tokens_asked value
    pub last_timestamp: i64,
}

impl VerificationState {
    pub const LEN: usize = 8 + 32 + 8;
}

fn update_on_chain_state_with_amount(
    account: &AccountInfo,
    amount: &[u8; 64],
) -> ProgramResult {
    let mut data = account.try_borrow_mut_data()?;
    let mut state = VerificationState::try_from_slice(&data)?;
    
    state.total_verifications += 1;
    state.last_amount.copy_from_slice(&amount[..32]);
    state.last_timestamp = Clock::get()?.unix_timestamp;
    
    state.serialize(&mut &mut data[..])?;
    
    msg!("Verification #{}: Amount verified", state.total_verifications);
    Ok(())
}

#[derive(PartialEq, Eq, Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct Groth16VerifierPrepared {
    proof_a: [u8; 64],
    proof_b: [u8; 128],
    proof_c: [u8; 64],
    pub prepared_public_inputs: [u8; 64],  // Made public for state updates
    verifying_key: Box<Groth16VerifyingKeyPrepared>,
}

impl Groth16VerifierPrepared {
    pub fn new(
        proof_a: &[u8],
        proof_b: &[u8],
        proof_c: &[u8],
        prepared_public_inputs: &[u8],  // ✅ Just the parameter name
        verifying_key: Box<Groth16VerifyingKeyPrepared>,
    ) -> Result<Self, Groth16Error> {
        if proof_a.len() != 64 {
            return Err(Groth16Error::InvalidG1Length);
        }
        if proof_b.len() != 128 {
            return Err(Groth16Error::InvalidG2Length);
        }
        if proof_c.len() != 64 {
            return Err(Groth16Error::InvalidG1Length);
        }
        if prepared_public_inputs.len() != 64 {
            return Err(Groth16Error::InvalidPublicInputsLength);
        }

        let mut proof_a_arr = [0u8; 64];
        proof_a_arr.copy_from_slice(proof_a);
        let mut proof_b_arr = [0u8; 128];
        proof_b_arr.copy_from_slice(proof_b);
        let mut proof_c_arr = [0u8; 64];
        proof_c_arr.copy_from_slice(proof_c);
        let mut prepared_public_inputs_arr = [0u8; 64];
        prepared_public_inputs_arr.copy_from_slice(prepared_public_inputs);

        Ok(Groth16VerifierPrepared {
            proof_a: proof_a_arr,
            proof_b: proof_b_arr,
            proof_c: proof_c_arr,
            prepared_public_inputs: prepared_public_inputs_arr,
            verifying_key,
        })
    }

    pub fn verify(&mut self, accounts: &[AccountInfo]) -> Result<bool, Groth16Error> {
        let pairing_input = [
            self.proof_a.as_slice(),
            self.proof_b.as_slice(),
            self.prepared_public_inputs.as_slice(),
            self.verifying_key.vk_gamma_g2.as_slice(),
            self.proof_c.as_slice(),
            self.verifying_key.vk_delta_g2.as_slice(),
            self.verifying_key.vk_alpha_g1.as_slice(),
            self.verifying_key.vk_beta_g2.as_slice(),
        ]
        .concat();

        let ix = Instruction {
            program_id: ALT_BN128_PAIRING,
            accounts: vec![],
            data: pairing_input,
        };

        invoke(&ix, accounts).map_err(|_| Groth16Error::PairingVerificationError)?;

        Ok(true)
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum Groth16Error {
    #[error("Incompatible Verifying Key with number of public inputs")]
    IncompatibleVerifyingKeyWithNrPublicInputs,
    #[error("ProofVerificationFailed")]
    ProofVerificationFailed,
    #[error("PairingVerificationError")]
    PairingVerificationError,
    #[error("PreparingInputsG1AdditionFailed")]
    PreparingInputsG1AdditionFailed,
    #[error("PreparingInputsG1MulFailed")]
    PreparingInputsG1MulFailed,
    #[error("InvalidG1Length")]
    InvalidG1Length,
    #[error("InvalidG2Length")]
    InvalidG2Length,
    #[error("InvalidPublicInputsLength")]
    InvalidPublicInputsLength,
    #[error("DecompressingG1Failed")]
    DecompressingG1Failed,
    #[error("DecompressingG2Failed")]
    DecompressingG2Failed,
    #[error("PublicInputGreaterThenFieldSize")]
    PublicInputGreaterThenFieldSize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_groth16_verifier_prepared_creation() {
        let proof_a = [0u8; 64];
        let proof_b = [0u8; 128];
        let proof_c = [0u8; 64];
        let prepared_public_inputs = [0u8; 64];
        let verifying_key = Box::new(Groth16VerifyingKeyPrepared {
            vk_alpha_g1: [0u8; 64],
            vk_beta_g2: [0u8; 128],
            vk_gamma_g2: [0u8; 128],
            vk_delta_g2: [0u8; 128],
        });

        let result = Groth16VerifierPrepared::new(
            &proof_a,
            &proof_b,
            &proof_c,
            &prepared_public_inputs,
            verifying_key,
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_proof_lengths() {
        let short_proof_a = [0u8; 63];
        let proof_b = [0u8; 128];
        let proof_c = [0u8; 64];
        let prepared_public_inputs = [0u8; 64];
        let verifying_key = Box::new(Groth16VerifyingKeyPrepared {
            vk_alpha_g1: [0u8; 64],
            vk_beta_g2: [0u8; 128],
            vk_gamma_g2: [0u8; 128],
            vk_delta_g2: [0u8; 128],
        });

        let result = Groth16VerifierPrepared::new(
            &short_proof_a,
            &proof_b,
            &proof_c,
            &prepared_public_inputs,
            verifying_key,
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Groth16Error::InvalidG1Length);
    }
}
