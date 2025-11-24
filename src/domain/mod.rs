//! Couche domaine - Logique métier de l'application
//!
//! Contient les entités, les services métier et la logique de domaine
//! sans dépendance aux détails d'implémentation (UI, stockage, etc.)

pub mod audit;
pub mod models;
pub mod recommendations;

// Réexportations publiques
pub use audit::{AuditManager, QuestionGenerator};
pub use models::*;
pub use recommendations::RecommendationEngine;
