use anchor_client::{
    solana_sdk::{
        commitment_config::CommitmentConfig,
        pubkey::Pubkey,
        signature::{Keypair, Signer},
    },
    Client, Cluster,
};
use anchor_client::solana_sdk::system_program;
use std::rc::Rc;
use sha2::Digest;

#[tokio::test]
async fn test_full_commit_reveal_flow() {
    // Setup client
    let payer = Rc::new(Keypair::new());
    let client = Client::new_with_options(
        Cluster::Localnet,
        payer,
        CommitmentConfig::processed(),
    );
    
    // Test commit-reveal flow
    let program = client.program(commit_reveal_dapp::ID).expect("Failed to get program");    
    // Generate test data
    let order_data = b"test order data";
    let secret = b"secret123";
    let mut hash_input = order_data.to_vec();
    hash_input.extend_from_slice(secret);
    
    let commitment_hash = sha2::Sha256::digest(&hash_input);
    
    // Test commitment
    let user = Keypair::new();
    let commitment_index = 0u64;
    // Derive state PDA
    let (state_pda, _) = Pubkey::find_program_address(
        &[b"state"],
        &program.id(),
    );
    
    let (commitment_pda, _) = Pubkey::find_program_address(
        &[
            b"commitment",
            user.pubkey().as_ref(),
            &commitment_index.to_le_bytes(),
        ],
        &program.id(),
    );
    
    // Commit order
    let sig = program
        .request()
        .accounts(commit_reveal_dapp::accounts::CommitOrder {
            commitment: commitment_pda,
            state: state_pda,
            user: user.pubkey(),
            system_program: system_program::ID,
        })
        .args(commit_reveal_dapp::instruction::CommitOrder {
            commitment_hash: commitment_hash.to_vec(),
            commitment_index,
            _zk_proof: None,
        })
        .signer(&user)
        .send()
        .expect("Commit should succeed");
    
    println!("Commit transaction: {}", sig);
    
    // Later reveal order
    let sig = program
        .request()
        .accounts(commit_reveal_dapp::accounts::RevealOrder {
            commitment: commitment_pda,
            state: state_pda,
            user: user.pubkey(),
        })
        .args(commit_reveal_dapp::instruction::RevealOrder {
            order_data: order_data.to_vec(),
            secret: secret.to_vec(),
        })
        .signer(&user)
        .send()
        .expect("Reveal should succeed");
    
    println!("Reveal transaction: {}", sig);
}