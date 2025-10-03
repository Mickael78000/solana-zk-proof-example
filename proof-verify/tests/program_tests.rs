#[cfg(test)]
mod program_tests {
    use solana_zk_example::*;
    use borsh::BorshDeserialize;
    
    #[test]
    fn test_verification_state_serialization() {
        let state = VerificationState {
            total_verifications: 10,
            last_amount: [1u8; 32],
            last_timestamp: 1234567890,
        };
        
        let serialized = borsh::to_vec(&state).unwrap();
        assert_eq!(serialized.len(), VerificationState::LEN);
        
        let deserialized = VerificationState::try_from_slice(&serialized).unwrap();
        assert_eq!(deserialized.total_verifications, 10);
        assert_eq!(deserialized.last_timestamp, 1234567890);
    }
    
    #[test]
    fn test_groth16_verifying_key_prepared() {
        let vk = Groth16VerifyingKeyPrepared {
            vk_alpha_g1: [0u8; 64],
            vk_beta_g2: [0u8; 128],
            vk_gamma_g2: [0u8; 128],
            vk_delta_g2: [0u8; 128],
        };
        
        let serialized = borsh::to_vec(&vk).unwrap();
        let deserialized = Groth16VerifyingKeyPrepared::try_from_slice(&serialized).unwrap();
        assert_eq!(vk, deserialized);
    }
    
    #[test]
    fn test_program_instruction_serialization() {
        let vk = Box::new(Groth16VerifyingKeyPrepared {
            vk_alpha_g1: [0u8; 64],
            vk_beta_g2: [0u8; 128],
            vk_gamma_g2: [0u8; 128],
            vk_delta_g2: [0u8; 128],
        });
        
        let verifier = Groth16VerifierPrepared::new(
            &[0u8; 64],
            &[0u8; 128],
            &[0u8; 64],
            &[0u8; 64],
            vk,
        ).unwrap();
        
        let instruction = ProgramInstruction::VerifyProof(verifier);
        let serialized = borsh::to_vec(&instruction).unwrap();
        assert!(serialized.len() > 0);
    }
    
    #[test]
    fn test_groth16_error_types() {
        assert_eq!(
            Groth16Error::InvalidG1Length.to_string(),
            "InvalidG1Length"
        );
        assert_eq!(
            Groth16Error::InvalidG2Length.to_string(),
            "InvalidG2Length"
        );
    }
}