#[cfg(test)]
mod e2e_tests {
    use solana_zk_client_example::circuit::TokenVerificationCircuit;
    use solana_zk_client_example::prove::*;
    use solana_zk_client_example::verify::*;
    
    #[test]
    fn test_full_proof_workflow() {
        // 1. Créer le circuit
        let circuit = TokenVerificationCircuit::new(1000, 500).unwrap();
        let public_inputs = circuit.public_inputs().unwrap();
        
        // 2. Setup
        let (pk, vk) = setup(false, circuit.clone());
        
        // 3. Générer la preuve
        let result = generate_proof_package(&pk, &vk, circuit, &public_inputs);
        
        if let Err(e) = &result {
            eprintln!("Erreur lors de la génération de preuve: {:?}", e);
        }
        
        assert!(result.is_ok(), "La génération de preuve devrait réussir");
        
        let (_, _, proof_package) = result.unwrap();
        
        // 4. Vérifier la preuve
        let verification_result = verify_proof_package(&proof_package);
        assert!(verification_result.is_ok());
        assert!(verification_result.unwrap());
    }
    
        #[test]
    fn test_invalid_proof_workflow() {
        // Circuit avec balance insuffisante (500 < 1000)
        let circuit = TokenVerificationCircuit::new(500, 1000).unwrap();
        let public_inputs = circuit.public_inputs().unwrap();
        
        let (pk, vk) = setup(false, circuit.clone());
        
        // La génération de preuve devrait échouer ou paniquer
        // car D = 500 - 1000 = -500 ne peut pas être décomposé en bits positifs
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            generate_proof_package(&pk, &vk, circuit, &public_inputs)
        }));
        
        // Le test réussit si la génération de preuve échoue (panic ou erreur)
        assert!(result.is_err(), "La génération de preuve devrait échouer pour des valeurs invalides");
    }
}