/*!
# zk-verification Library

This library provides a dummy implementation of a zk-STARK proof system for identity verification.
It uses the SHA-256 hash function to generate a proof from an identity ID as a placeholder for a real
zk-STARK implementation.

## Functions

- `generate_proof(identity_id: &str) -> String`: Generates a dummy proof based on the provided identity ID.
- `verify_proof(proof: &str, identity_id: &str) -> bool`: Verifies that a given proof matches the proof generated
  from the identity ID.
*/

use sha2::{Digest, Sha256};

/// Generates a zk-STARK proof for the given identity ID.
///
/// # Arguments
///
/// * `identity_id` - A string slice representing the unique identity identifier.
///
/// # Returns
///
/// A hexadecimal string representing the generated proof.
///
/// # Example
///
/// ```
/// let proof = zk_verification::generate_proof("User123");
/// println!("Proof: {}", proof);
/// ```
pub fn generate_proof(identity_id: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(identity_id);
    format!("{:x}", hasher.finalize())
}

/// Verifies the provided proof against the identity ID.
///
/// # Arguments
///
/// * `proof` - A string slice that holds the proof to be verified.
/// * `identity_id` - A string slice representing the identity identifier used to generate the proof.
///
/// # Returns
///
/// A boolean indicating whether the provided proof is valid.
///
/// # Example
///
/// ```
/// let proof = zk_verification::generate_proof("User123");
/// assert!(zk_verification::verify_proof(&proof, "User123"));
/// ```
pub fn verify_proof(proof: &str, identity_id: &str) -> bool {
    let generated_proof = generate_proof(identity_id);
    generated_proof == proof
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dummy_proof() {
        let identity_id = "User123";
        let proof = generate_proof(identity_id);
        assert!(verify_proof(&proof, identity_id));
    }

    #[test]
    fn test_invalid_proof() {
        let identity_id = "User123";
        let wrong_proof = "invalidproof";
        assert!(!verify_proof(wrong_proof, identity_id));
    }
}
