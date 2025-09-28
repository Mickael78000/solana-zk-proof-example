# Solana Zero-Knowledge Proof Implementation

A comprehensive **Solana Zero-Knowledge Proof (ZK-SNARK) implementation** that demonstrates how to generate and verify Groth16 proofs on the Solana blockchain. This project leverages the BN254 (alt_bn128) elliptic curve and integrates with Solana's native cryptographic capabilities to enable privacy-preserving applications.

## 🏗️ Project Structure

```
├── Cargo.toml                      # Workspace configuration
├── dex-router/                     # OKX Labs DEX Router integration
├── integration-tests/              # TypeScript integration tests
│   ├── node_modules/              # Dependencies (Anchor, Solana Web3, etc.)
│   ├── package.json
│   ├── tests/
│   │   └── commit_reveal.ts       # Commit-reveal scheme tests
│   └── tsconfig.json
├── LICENSE
├── on-chain-program-example/       # Client application for proof generation
│   ├── Cargo.toml
│   ├── pk.bin                     # Proving key (generated)
│   ├── vk.bin                     # Verifying key (generated)
│   └── src/
│       ├── byte_utils.rs          # Endianness conversion utilities
│       ├── circuit.rs             # Example ZK circuit implementation
│       ├── errors.rs              # Custom error types
│       ├── lib.rs                 # Library exports
│       ├── main.rs                # Test suite and examples
│       ├── prove.rs               # Proof generation logic
│       ├── verify_lite.rs         # Lightweight verification
│       └── verify.rs              # Standard verification
├── proof-verify/                   # Solana on-chain program
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       └── lib.rs                 # On-chain proof verification
├── README.md
└── solana-commit-reveal/          # Commit-reveal scheme implementation
    └── commit-reveal-dapp/        # Anchor-based DApp
        ├── Anchor.toml
        ├── Cargo.toml
        ├── package.json
        ├── programs/              # Solana programs
        ├── tests/                 # Test suite
        └── yarn.lock
```

## 🚀 Key Components

### 1. **On-Chain Program Example**

A Rust-based client application that handles the generation and submission of zero-knowledge proofs to Solana.

**Key Features:**
- **Circuit Implementation** (`circuit.rs`): Defines a simple example circuit (`ExampleCircuit`) that implements the `ConstraintSynthesizer` trait for basic constraint generation
- **Proof Generation** (`prove.rs`): 
  - Implements trusted setup for generating proving and verifying keys
  - Creates proof packages in multiple formats (lite, prepared, and standard)
  - Saves keys to binary files for reuse
- **Verification Logic** (`verify.rs`, `verify_lite.rs`):
  - Converts arkworks-based proofs to Solana-compatible format
  - Implements endianness conversion for proper byte ordering
  - Includes both simple verification and prepared verifier structures
- **Testing Suite** (`main.rs`):
  - Off-chain verification tests
  - On-chain verification tests with local Solana validator
  - ALT_BN128 pairing tests to verify cryptographic operations

### 2. **Proof Verification Program**

A Solana on-chain program (smart contract) that verifies submitted Groth16 proofs.

**Capabilities:**
- Processes two types of instructions:
  1. `VerifyProof`: Standard proof verification
  2. `VerifyProofWithBalance`: Proof verification with additional balance checks
- Uses Solana's native ALT_BN128_PAIRING syscall
- Performs pairing checks to validate Groth16 proofs
- Updates on-chain state upon successful verification

### 3. **Integration Tests**

TypeScript-based test suite using Mocha and Chai for comprehensive testing of the ZK proof system, including commit-reveal schemes.

### 4. **Commit-Reveal DApp**

An Anchor-based decentralized application implementing commit-reveal schemes with zero-knowledge proof integration.

## 🔧 Technical Stack

**Cryptographic Foundation:**
- **Curve**: BN254 (alt_bn128) - optimized for pairing operations
- **Proof System**: Groth16 - succinct non-interactive zero-knowledge proofs
- **Libraries**: arkworks suite (ark-bn254, ark-groth16, ark-ec, ark-ff, ark-serialize)

**Key Technical Features:**
1. **Endianness Handling**: Custom byte conversion utilities ensure compatibility between arkworks' little-endian format and Solana's expected byte ordering
2. **Proof Negation**: The proof's `A` component is negated before submission to match Solana's pairing verification expectations
3. **Prepared Inputs**: Public inputs are pre-processed into G1 projective format for efficient verification
4. **Comprehensive Error Handling**: Detailed error types for debugging verification failures

## 🛠️ Prerequisites

- **Rust** (latest stable version)
- **Solana CLI** (v1.14.0+)
- **Node.js** (v16+) and **npm**/**yarn**
- **Anchor Framework** (v0.27.0+)

## 🚀 Quick Start

### 1. Clone the Repository

```bash
git clone <repository-url>
cd solana-zk-proof-implementation
```

### 2. Build the Rust Components

```bash
# Build all workspace components
cargo build --release

# Build specific components
cd on-chain-program-example
cargo build --release

cd ../proof-verify
cargo build --release
```

### 3. Run Off-Chain Tests

```bash
cd on-chain-program-example
cargo run
```

This will:
- Generate proving and verifying keys
- Create example proofs
- Test off-chain verification
- Demonstrate proof format conversion

### 4. Deploy and Test On-Chain

```bash
# Start local Solana validator
solana-test-validator

# Deploy the proof verification program
cd proof-verify
anchor deploy

# Run integration tests
cd ../integration-tests
npm install
npm test
```

### 5. Test Commit-Reveal DApp

```bash
cd solana-commit-reveal/commit-reveal-dapp
yarn install
anchor test
```

## 📝 Usage Examples

### Generating a Proof

```rust
use on_chain_program_example::{ExampleCircuit, create_proof_package};

// Create circuit with secret value
let circuit = ExampleCircuit { value: Some(Fr::from(42)) };

// Generate proof
let proof_package = create_proof_package(&circuit)?;

// Convert to Solana format
let solana_proof = proof_package.to_solana_format()?;
```

### On-Chain Verification

```rust
// In your Solana program
use proof_verify::{verify_groth16_proof, Groth16VerifierPrepared};

pub fn verify_proof(
    ctx: Context<VerifyProof>,
    verifier: Groth16VerifierPrepared,
) -> Result<()> {
    verify_groth16_proof(&verifier)?;
    // Update program state on successful verification
    Ok(())
}
```

## 🎯 Use Cases

This project serves as a foundation for building privacy-preserving applications on Solana:

- **Private Transactions**: Hide transaction amounts and participants
- **Identity Verification**: Prove identity without revealing personal information
- **Compliance Proofs**: Demonstrate regulatory compliance without exposing sensitive data
- **Private Voting Systems**: Enable secret ballot voting with verifiable results
- **Confidential DeFi Operations**: Private trading, lending, and liquidity provision
- **Commit-Reveal Schemes**: Secure auctions and fair randomness generation

## 🧪 Testing

The project includes comprehensive testing at multiple levels:

```bash
# Unit tests for circuit logic
cd on-chain-program-example
cargo test

# Integration tests with Solana validator
cd integration-tests
npm test

# Anchor program tests
cd solana-commit-reveal/commit-reveal-dapp
anchor test
```

## 🔍 Architecture Details

### Proof Generation Flow

1. **Circuit Definition**: Define constraints using arkworks' R1CS framework
2. **Trusted Setup**: Generate proving and verifying keys (one-time process)
3. **Proof Creation**: Generate proof for specific witness values
4. **Format Conversion**: Convert arkworks proof to Solana-compatible format
5. **Submission**: Submit proof to Solana program for verification

### On-Chain Verification Flow

1. **Proof Reception**: Receive proof data through program instruction
2. **Format Validation**: Ensure proof components are properly formatted
3. **Pairing Operations**: Use Solana's ALT_BN128_PAIRING syscall for verification
4. **State Updates**: Update program state based on verification result

## 📚 Documentation

- **Circuit Development**: See `on-chain-program-example/src/circuit.rs` for example implementations
- **Proof Generation**: Check `on-chain-program-example/src/prove.rs` for key generation and proving
- **On-Chain Integration**: Review `proof-verify/src/lib.rs` for Solana program patterns
- **Testing Patterns**: Examine `integration-tests/tests/` for comprehensive test examples

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## ⚠️ Security Considerations

- **Trusted Setup**: The proving and verifying keys must be generated through a trusted setup ceremony for production use
- **Circuit Review**: All constraint systems should be thoroughly audited before deployment
- **Key Management**: Protect proving keys and ensure verifying keys are properly validated
- **Gas Optimization**: Monitor Solana compute unit usage for complex circuits

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🔗 Related Projects

- **OKX Labs DEX Router**: Smart order routing integration
- **Solana ZK Token SDK**: Additional privacy primitives
- **Arkworks**: Cryptographic library ecosystem

## 📬 Support

For questions, issues, or contributions:
- Open an issue on GitHub
- Join the Solana developer community
- Consult the Solana and arkworks documentation

---

**Built with ❤️ for the Solana ecosystem**