/// Trait pour l'abstraction du stockage
use crate::domain::models::Audit;
use crate::shared::error::AppResult;
use uuid::Uuid;

/// Trait pour le stockage des audits
/// Permet de changer facilement de backend (localStorage, fichier, base de données, etc.)
pub trait Storage: Send + Sync {
    /// Sauvegarde un audit
    fn save_audit(&self, audit: &Audit) -> AppResult<()>;

    /// Charge un audit par son ID
    fn load_audit(&self, audit_id: &Uuid) -> AppResult<Audit>;

    /// Liste tous les IDs d'audits disponibles
    fn list_audits(&self) -> AppResult<Vec<Uuid>>;

    /// Supprime un audit
    fn delete_audit(&self, audit_id: &Uuid) -> AppResult<()>;
}
