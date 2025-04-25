use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use warp::Filter;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

// ========================================================
// IdentityBlock: Represents a block that stores an identity registration.
#[derive(Serialize, Deserialize, Clone)]
struct IdentityBlock {
    index: u64,
    timestamp: String,
    identity_id: String,
    owner: String,
    previous_hash: String,
    hash: String,
    api_key: String, // Field for storing the API key for this identity.
}

impl IdentityBlock {
    fn new(index: u64, identity_id: String, owner: String, previous_hash: String, api_key: String) -> Self {
        let timestamp = Utc::now().to_rfc3339();
        let hash = IdentityBlock::calculate_hash(index, &timestamp, &identity_id, &owner, &previous_hash);
        IdentityBlock {
            index,
            timestamp,
            identity_id,
            owner,
            previous_hash,
            hash,
            api_key,
        }
    }

    fn calculate_hash(index: u64, timestamp: &str, identity_id: &str, owner: &str, previous_hash: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(index.to_string());
        hasher.update(timestamp);
        hasher.update(identity_id);
        hasher.update(owner);
        hasher.update(previous_hash);
        format!("{:x}", hasher.finalize())
    }
}

// ========================================================
// Blockchain: Holds the chain (ledger) and the file path for persistent storage.
#[derive(Clone)]
struct Blockchain {
    chain: Arc<Mutex<Vec<IdentityBlock>>>,
    storage_file: String,
}

impl Blockchain {
    /// Creates a new Blockchain instance.
    /// Loads the chain from disk if the storage file exists; otherwise, creates a genesis block.
    fn new(storage_file: String) -> Self {
        let chain = if Path::new(&storage_file).exists() {
            let data = fs::read_to_string(&storage_file).unwrap_or_default();
            serde_json::from_str(&data).unwrap_or_else(|_| {
                vec![IdentityBlock::new(
                    0,
                    "Genesis".to_string(),
                    "0".to_string(),
                    "0".to_string(),
                    "".to_string(),
                )]
            })
        } else {
            vec![IdentityBlock::new(
                0,
                "Genesis".to_string(),
                "0".to_string(),
                "0".to_string(),
                "".to_string(),
            )]
        };
        Blockchain {
            chain: Arc::new(Mutex::new(chain)),
            storage_file,
        }
    }

    /// Generates a random API key (a 32-character alphanumeric string).
    fn generate_api_key() -> String {
        thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect()
    }

    /// Registers a new identity block (generating a new API key) and persists the chain to disk.
    fn register_identity(&self, identity_id: String, owner: String) -> IdentityBlock {
        let new_block = {
            let mut chain = self.chain.lock().unwrap();
            let last_block = chain.last().unwrap().clone();
            let api_key = Blockchain::generate_api_key();
            let new_block = IdentityBlock::new(last_block.index + 1, identity_id, owner, last_block.hash, api_key);
            chain.push(new_block.clone());
            new_block
        };
        self.save();
        new_block
    }

    /// Returns a clone of the entire blockchain.
    fn get_chain(&self) -> Vec<IdentityBlock> {
        let chain = self.chain.lock().unwrap();
        chain.clone()
    }

    /// Persists the current chain to the storage file.
    fn save(&self) {
        let chain_snapshot = {
            let chain = self.chain.lock().unwrap();
            chain.clone()
        };
        let json = serde_json::to_string_pretty(&chain_snapshot)
            .expect("Serialization failed");
        fs::write(&self.storage_file, json)
            .expect("Unable to write file");
    }
}

// ========================================================
// API Request/Response Structures
#[derive(Deserialize)]
struct RegisterIdentityRequest {
    identity_id: String,
    owner: String,
}

#[derive(Deserialize)]
struct VerifyProofRequest {
    identity_id: String,
    proof: String,
}

#[derive(Deserialize)]
struct QuantumProofRequest {
    data: String,
}

#[derive(Serialize)]
struct QuantumProofResponse {
    proof: String,
}

#[derive(Deserialize)]
struct VerifyQuantumProofRequest {
    data: String,
    proof: String,
}

#[derive(Serialize)]
struct VerifyQuantumProofResponse {
    valid: bool,
}

#[derive(Deserialize)]
struct AuthRequest {
    api_key: String,
}

#[derive(Serialize)]
struct AuthResponse {
    valid: bool,
    message: String,
}

// ========================================================
// Dynamic Authorization Filter
//
// This filter checks if the provided "authorization" header value (the API key)
// exists in one of the identity blocks stored in the blockchain.
fn with_dynamic_auth(blockchain: Blockchain) -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    warp::header::<String>("authorization")
        .and_then(move |token: String| {
            let bc = blockchain.clone();
            async move {
                let chain = bc.get_chain();
                if chain.iter().any(|block| block.api_key == token) {
                    Ok(())
                } else {
                    Err(warp::reject::custom(Unauthorized))
                }
            }
        })
        .untuple_one()
}

// Custom rejection type for unauthorized access.
#[derive(Debug)]
struct Unauthorized;
impl warp::reject::Reject for Unauthorized {}

// ========================================================
// Authorization Handler
//
// This handler receives a JSON body containing an API key and checks its validity.
async fn authorize_handler(req: AuthRequest, blockchain: Blockchain) -> Result<impl warp::Reply, warp::Rejection> {
    let chain = blockchain.get_chain();
    if chain.iter().any(|block| block.api_key == req.api_key) {
        // Both branches now return a WithStatus<Json>
        Ok(warp::reply::with_status(
            warp::reply::json(&AuthResponse { valid: true, message: "API key is valid".to_string() }),
            warp::http::StatusCode::OK,
        ))
    } else {
        Ok(warp::reply::with_status(
            warp::reply::json(&AuthResponse { valid: false, message: "Invalid API key".to_string() }),
            warp::http::StatusCode::UNAUTHORIZED,
        ))
    }
}

// ========================================================
// Quantumproof Handlers
async fn generate_quantum_proof_handler(req: QuantumProofRequest) -> Result<impl warp::Reply, warp::Rejection> {
    let proof = zk_verification_interface::generate_proof(&req.data);
    Ok(warp::reply::json(&QuantumProofResponse { proof }))
}

async fn verify_quantum_proof_handler(req: VerifyQuantumProofRequest) -> Result<impl warp::Reply, warp::Rejection> {
    let valid = zk_verification_interface::verify_proof(&req.proof, &req.data);
    Ok(warp::reply::json(&VerifyQuantumProofResponse { valid }))
}

// ========================================================
// Dummy zk-STARK Functions (for illustration)
//
// In a real implementation, replace these with a production-ready zk-STARK library.
mod zk_verification_interface {
    pub fn generate_proof(identity_id: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(identity_id);
        format!("{:x}", hasher.finalize())
    }

    pub fn verify_proof(proof: &str, identity_id: &str) -> bool {
        let generated = generate_proof(identity_id);
        generated == proof
    }
}

// ========================================================
// Main Function: Set up and run the HTTP API using Warp.
#[tokio::main]
async fn main() {
    let storage_file = "ddxid_chain.json".to_string();
    let blockchain = Blockchain::new(storage_file);

    // Create a filter for public routes that clones the blockchain.
    let blockchain_filter = warp::any().map({
        let bc = blockchain.clone();
        move || bc.clone()
    });

    // Create a separate clone for use in the dynamic auth filter.
    let blockchain_for_auth = blockchain.clone();

    // GET /chain: Retrieve the full blockchain ledger.
    let get_chain = warp::path!("chain")
        .and(warp::get())
        .and(blockchain_filter.clone())
        .and_then(|bc: Blockchain| async move {
            Ok::<_, warp::Rejection>(warp::reply::json(&bc.get_chain()))
        });

    // POST /register_identity: Public endpoint to register a new identity.
    let register_identity = warp::path!("register_identity")
        .and(warp::post())
        .and(warp::body::json())
        .and(blockchain_filter.clone())
        .and_then(|req: RegisterIdentityRequest, bc: Blockchain| async move {
            Ok::<_, warp::Rejection>(warp::reply::json(&bc.register_identity(req.identity_id, req.owner)))
        });

    // POST /authorize: Endpoint to verify an API key; expects a JSON body with { "api_key": "..." }.
    let authorize = warp::path!("authorize")
        .and(warp::post())
        .and(warp::body::json())
        .and(blockchain_filter.clone())
        .and_then(authorize_handler);

    // Protected endpoint: POST /verify_proof.
    let verify_proof = warp::path!("verify_proof")
        .and(warp::post())
        .and(with_dynamic_auth(blockchain_for_auth.clone()))
        .and(warp::body::json())
        .and_then(|req: VerifyProofRequest| async move {
            let valid = zk_verification_interface::verify_proof(&req.proof, &req.identity_id);
            Ok::<_, warp::Rejection>(warp::reply::json(&serde_json::json!({ "valid": valid })))
        });

    // Protected endpoint: POST /generate_quantum_proof.
    let generate_quantum_proof = warp::path!("generate_quantum_proof")
        .and(warp::post())
        .and(with_dynamic_auth(blockchain_for_auth.clone()))
        .and(warp::body::json())
        .and_then(generate_quantum_proof_handler);

    // Protected endpoint: POST /verify_quantum_proof.
    let verify_quantum_proof = warp::path!("verify_quantum_proof")
        .and(warp::post())
        .and(with_dynamic_auth(blockchain_for_auth.clone()))
        .and(warp::body::json())
        .and_then(verify_quantum_proof_handler);

    // Combine all routes.
    let routes = get_chain
        .or(register_identity)
        .or(authorize)
        .or(verify_proof)
        .or(generate_quantum_proof)
        .or(verify_quantum_proof);

    println!("ddxid Layer0 Node running on 0.0.0.0:3030");
    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
}
