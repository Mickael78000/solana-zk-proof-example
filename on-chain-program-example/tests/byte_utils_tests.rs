#[cfg(test)]
mod byte_utils_tests {
    use solana_zk_client_example::byte_utils::*;
    use ark_bn254::Fr;
    use ark_ff::PrimeField;
    
    #[test]
    fn test_field_to_bytes_conversion() {
        let field = Fr::from(12345u64);
        let bytes = field_to_bytes(field);
        assert_eq!(bytes.len(), 32);
        
        // Vérifier la conversion inverse
        let field_back = bytes_to_field::<Fr>(&bytes).unwrap();
        assert_eq!(field, field_back);
    }
    
    #[test]
    fn test_bytes_to_field_zero() {
        let zero_bytes = [0u8; 32];
        let field = bytes_to_field::<Fr>(&zero_bytes).unwrap();
        assert_eq!(field, Fr::from(0u64));
    }
    
    #[test]
    fn test_endianness_conversion_64_bytes() {
        let input = [1u8; 64];
        let result: Result<[u8; 64], _> = convert_endianness(&input);
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert_eq!(output.len(), 64);
    }
    
    #[test]
    fn test_endianness_conversion_128_bytes() {
        let input = [255u8; 128];
        let result: Result<[u8; 128], _> = convert_endianness(&input);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_endianness_conversion_invalid_size() {
        let input = [0u8; 33]; // Non-multiple de 4
        let result: Result<[u8; 33], _> = convert_endianness(&input);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_endianness_roundtrip() {
        let original = [42u8; 64];
        let converted: [u8; 64] = convert_endianness(&original).unwrap();
        let back: [u8; 64] = convert_endianness(&converted).unwrap();
        assert_eq!(original, back);
    }
}