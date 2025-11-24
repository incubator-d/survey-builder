//! Couche domaine - Logique métier de l'application
//! 
//! Contient les entités, les services métier et la logique de domaine
//! sans dépendance aux détails d'implémentation (UI, stockage, etc.)

pub mod models;
pub mod audit;
pub mod recommendations;

// Réexportations publiques
pub use models::*;
pub use audit::{AuditManager, QuestionGenerator};
pub use recommendations::RecommendationEngine;

