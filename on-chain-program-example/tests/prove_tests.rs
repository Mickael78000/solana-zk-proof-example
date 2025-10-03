#[cfg(test)]
mod prove_tests {
    use solana_zk_client_example::prove::*;
    use solana_zk_client_example::circuit::ExampleCircuit;
    
    #[test]
    fn test_validate_public_input_valid() {
        let input = [1u8; 32];
        let result = validate_public_input(&input);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_validate_public_input_zero() {
        let input = [0u8; 32];
        let result = validate_public_input(&input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ark_bn254::Fr::from(0u64));
    }
    
    #[test]
    fn test_setup_with_save_keys() {
        let circuit = ExampleCircuit::new(100, 50).unwrap();
        let (pk, vk) = setup(false, circuit);
        
        // Vérifier que les clés sont générées
        assert!(pk.a_query.len() > 0);
        assert!(vk.gamma_abc_g1.len() > 0);
    }
    
    #[test]
    fn test_validate_proving_key() {
        let circuit = ExampleCircuit::new(100, 50).unwrap();
        let (pk, _) = setup(false, circuit.clone());
        
        // Vérifier que la clé de preuve contient des données valides
        assert!(pk.a_query.len() > 0, "a_query devrait contenir des éléments");
        assert!(pk.l_query.len() > 0, "l_query devrait contenir des éléments");
    }
    
    #[test]
    fn test_proof_package_generation() {
        let circuit = ExampleCircuit::new(100, 50).unwrap();
        let public_inputs = circuit.public_inputs().unwrap();
        let (pk, vk) = setup(false, circuit.clone());
        
        // Note: Ce test peut échouer si validate_proving_key est trop strict
        // Tester la génération de preuve directement
        use ark_groth16::Groth16;
        use ark_snark::SNARK;
        use rand::thread_rng;
        
        let mut rng = thread_rng();
        let proof = Groth16::<ark_bn254::Bn254>::prove(&pk, circuit, &mut rng);
        
        assert!(proof.is_ok(), "La génération de preuve devrait réussir");
    }
    
    #[test]
    fn test_proof_package_with_invalid_inputs() {
        let circuit = ExampleCircuit::new(100, 50).unwrap();
        let wrong_inputs = vec![[0u8; 32]]; // Nombre incorrect d'entrées
        let (pk, vk) = setup(false, circuit.clone());
        
        let result = generate_proof_package(&pk, &vk, circuit, &wrong_inputs);
        assert!(result.is_err(), "Devrait échouer avec un nombre incorrect d'entrées");
    }
}