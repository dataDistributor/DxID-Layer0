use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;

/// Type of identity: either an individual or an entity.
#[derive(Debug, Clone)]
pub enum IdentityType {
    Individual,
    Entity,
}

/// Structure holding identity information.
#[derive(Debug, Clone)]
pub struct Identity {
    pub id: String,         // A unique identifier (e.g., UUID or hash)
    pub owner: String,      // Owner's public key (or blockchain address)
    pub identity_type: IdentityType,
    pub metadata: String,   // Optional extra info
}

/// Global storage for identities (for demo purposes only).
pub static IDENTITY_STORAGE: Lazy<Mutex<HashMap<String, Identity>>> = Lazy::new(|| Mutex::new(HashMap::new()));

/// Register a new individual identity.
pub fn register_individual(id: String, owner: String, metadata: String) -> Result<(), &'static str> {
    let mut storage = IDENTITY_STORAGE.lock().unwrap();
    if storage.contains_key(&id) {
        return Err("Identity already exists");
    }
    storage.insert(
        id.clone(),
        Identity {
            id,
            owner,
            identity_type: IdentityType::Individual,
            metadata,
        },
    );
    Ok(())
}

/// Register a new entity (business/organization) identity.
pub fn register_entity(id: String, owner: String, metadata: String) -> Result<(), &'static str> {
    let mut storage = IDENTITY_STORAGE.lock().unwrap();
    if storage.contains_key(&id) {
        return Err("Entity already exists");
    }
    storage.insert(
        id.clone(),
        Identity {
            id,
            owner,
            identity_type: IdentityType::Entity,
            metadata,
        },
    );
    Ok(())
}

/// Delete an identity.
pub fn delete_identity(id: &str) -> Result<(), &'static str> {
    let mut storage = IDENTITY_STORAGE.lock().unwrap();
    if storage.remove(id).is_some() {
        Ok(())
    } else {
        Err("Identity does not exist")
    }
}

/// Transfer identity ownership.
pub fn transfer_identity(id: &str, new_owner: String) -> Result<(), &'static str> {
    let mut storage = IDENTITY_STORAGE.lock().unwrap();
    if let Some(identity) = storage.get_mut(id) {
        identity.owner = new_owner;
        Ok(())
    } else {
        Err("Identity does not exist")
    }
}

/// Verify if an identity exists.
pub fn verify_identity(id: &str) -> bool {
    let storage = IDENTITY_STORAGE.lock().unwrap();
    storage.contains_key(id)
}
