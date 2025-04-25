#[cfg(test)]
mod integration_tests {
    use layer0_core::Blockchain; // Ensure your workspace is set up properly.
    use zk_verification::proof_gen::{generate_proof, verify_proof};
    use identity::create_identity_contract;

    #[test]
    fn test_blockchain_and_proof() {
        // Test blockchain: create a new block.
        let blockchain = Blockchain::new();
        let new_block = blockchain.add_block("Test block data".to_string());
        assert!(new_block.index > 0);

        // Test dummy zk-STARK proof generation and verification.
        let data = "Test Data";
        let proof = generate_proof(data);
        assert!(verify_proof(&proof, data));

        // Test smart contract function.
        assert_eq!(create_identity_contract(), "Identity Contract Deployed");
    }
}
