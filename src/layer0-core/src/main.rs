use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::{Arc, Mutex};
use warp::Filter;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use sled::{Db, IVec};

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
    api_key: String,
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
// Blockchain: Holds the in‑memory chain and a sled::Db handle
#[derive(Clone)]
struct Blockchain {
    chain: Arc<Mutex<Vec<IdentityBlock>>>,
    db: Arc<Db>,
}

impl Blockchain {
    /// Opens (or creates) a sled database at `path`.
    /// Loads existing blocks (or writes a genesis block if empty).
    fn new(path: &str) -> Self {
        let db = sled::open(path).expect("opening sled database");
        let mut blocks: Vec<IdentityBlock> = Vec::new();

        // Iterate existing entries in key order
        for item in db.iter() {
            let (_, raw) = item.expect("reading sled entry");
            let blk: IdentityBlock = serde_json::from_slice(&raw)
                .expect("deserializing block from sled");
            blocks.push(blk);
        }

        // If no blocks, create & persist a genesis block
        if blocks.is_empty() {
            let genesis = IdentityBlock::new(
                0,
                "Genesis".to_string(),
                "0".to_string(),
                "0".to_string(),
                "".to_string(),
            );
            let key = genesis.index.to_be_bytes();
            let val = serde_json::to_vec(&genesis).unwrap();
            db.insert(key, val).expect("writing genesis to sled");
            blocks.push(genesis);
        }

        Blockchain {
            chain: Arc::new(Mutex::new(blocks)),
            db: Arc::new(db),
        }
    }

    /// Generates a random 32‑char alphanumeric API key.
    fn generate_api_key() -> String {
        thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect()
    }

    /// Appends a new block both in‑memory and in sled.
    fn register_identity(&self, identity_id: String, owner: String) -> IdentityBlock {
        let new_blk = {
            let mut chain = self.chain.lock().unwrap();
            let last = chain.last().unwrap().clone();
            let api_key = Blockchain::generate_api_key();
            let block = IdentityBlock::new(
                last.index + 1,
                identity_id,
                owner,
                last.hash.clone(),
                api_key,
            );

            let key = block.index.to_be_bytes();
            let val = serde_json::to_vec(&block).unwrap();
            self.db.insert(key, val).expect("writing new block to sled");

            chain.push(block.clone());
            block
        };
        new_blk
    }

    /// Returns a clone of the current in‑memory chain.
    fn get_chain(&self) -> Vec<IdentityBlock> {
        let chain = self.chain.lock().unwrap();
        chain.clone()
    }
}

// ========================================================
// API Request/Response Structs
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
fn with_dynamic_auth(blockchain: Blockchain) 
    -> impl Filter<Extract=(),Error=warp::Rejection> + Clone 
{
    warp::header::<String>("authorization")
        .and_then(move |token: String| {
            let bc = blockchain.clone();
            async move {
                let chain = bc.get_chain();
                if chain.iter().any(|blk| blk.api_key == token) {
                    Ok(())
                } else {
                    Err(warp::reject::custom(Unauthorized))
                }
            }
        })
        .untuple_one()
}

#[derive(Debug)]
struct Unauthorized;
impl warp::reject::Reject for Unauthorized {}

// ========================================================
// Authorization Handler
async fn authorize_handler(req: AuthRequest, blockchain: Blockchain)
    -> Result<impl warp::Reply, warp::Rejection>
{
    let valid = blockchain.get_chain()
        .iter()
        .any(|blk| blk.api_key == req.api_key);
    let resp = AuthResponse {
        valid,
        message: if valid {
            "API key is valid".to_string()
        } else {
            "Invalid API key".to_string()
        },
    };
    let status = if valid {
        warp::http::StatusCode::OK
    } else {
        warp::http::StatusCode::UNAUTHORIZED
    };
    Ok(warp::reply::with_status(warp::reply::json(&resp), status))
}

// ========================================================
// Dummy zk‑STARK interface
mod zk_verification_interface {
    pub fn generate_proof(data: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }
    pub fn verify_proof(proof: &str, data: &str) -> bool {
        generate_proof(data) == proof
    }
}

async fn generate_quantum_proof_handler(req: QuantumProofRequest)
    -> Result<impl warp::Reply, warp::Rejection>
{
    let proof = zk_verification_interface::generate_proof(&req.data);
    Ok(warp::reply::json(&QuantumProofResponse { proof }))
}

async fn verify_quantum_proof_handler(req: VerifyQuantumProofRequest)
    -> Result<impl warp::Reply, warp::Rejection>
{
    let valid = zk_verification_interface::verify_proof(&req.proof, &req.data);
    Ok(warp::reply::json(&VerifyQuantumProofResponse { valid }))
}

// ========================================================
// Main: wire up all routes exactly as before
#[tokio::main]
async fn main() {
    // Open (or create) sled DB at "./ddxid_chain"
    let blockchain = Blockchain::new("ddxid_chain");

    let bc_filter = warp::any().map({
        let bc = blockchain.clone();
        move || bc.clone()
    });
    let bc_auth = blockchain.clone();

    // GET /chain
    let get_chain = warp::path!("chain")
        .and(warp::get())
        .and(bc_filter.clone())
        .and_then(|bc: Blockchain| async move {
            Ok::<_, warp::Rejection>(warp::reply::json(&bc.get_chain()))
        });

    // POST /register_identity
    let register_identity = warp::path!("register_identity")
        .and(warp::post())
        .and(warp::body::json())
        .and(bc_filter.clone())
        .and_then(|req: RegisterIdentityRequest, bc: Blockchain| async move {
            Ok::<_, warp::Rejection>(warp::reply::json(
                &bc.register_identity(req.identity_id, req.owner),
            ))
        });

    // POST /authorize
    let authorize = warp::path!("authorize")
        .and(warp::post())
        .and(warp::body::json())
        .and(bc_filter.clone())
        .and_then(authorize_handler);

    // Protected POST /verify_proof
    let verify_proof = warp::path!("verify_proof")
        .and(warp::post())
        .and(with_dynamic_auth(bc_auth.clone()))
        .and(warp::body::json())
        .and_then(|req: VerifyProofRequest| async move {
            let valid = zk_verification_interface::verify_proof(&req.proof, &req.identity_id);
            Ok::<_, warp::Rejection>(warp::reply::json(&serde_json::json!({ "valid": valid })))
        });

    // Protected POST /generate_quantum_proof
    let generate_quantum_proof = warp::path!("generate_quantum_proof")
        .and(warp::post())
        .and(with_dynamic_auth(bc_auth.clone()))
        .and(warp::body::json())
        .and_then(generate_quantum_proof_handler);

    // Protected POST /verify_quantum_proof
    let verify_quantum_proof = warp::path!("verify_quantum_proof")
        .and(warp::post())
        .and(with_dynamic_auth(bc_auth.clone()))
        .and(warp::body::json())
        .and_then(verify_quantum_proof_handler);

    let routes = get_chain
        .or(register_identity)
        .or(authorize)
        .or(verify_proof)
        .or(generate_quantum_proof)
        .or(verify_quantum_proof);

    println!("dxid Layer0 Node running on 0.0.0.0:3030 with sled-backed storage");
    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
}
