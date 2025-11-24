//! Couche infrastructure - Implémentations techniques
//! 
//! Contient les implémentations concrètes des abstractions définies dans le domaine
//! (stockage, persistance, etc.)

pub mod storage;
pub mod storage_trait;

// Réexportations publiques
pub use storage::WebStorage;
pub use storage_trait::Storage;

