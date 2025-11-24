use crate::domain::models::*;
use crate::shared::error::{AppError, AppResult};
use crate::infrastructure::storage_trait::Storage;
use anyhow::{Context, Result};
use uuid::Uuid;
use web_sys::window;

const STORAGE_KEY_PREFIX: &str = "survey_audit_";
const STORAGE_INDEX_KEY: &str = "survey_audit_index";

/// Gestionnaire de stockage utilisant localStorage (compatible Vercel)
pub struct WebStorage;

impl Storage for WebStorage {
    fn save_audit(&self, audit: &Audit) -> AppResult<()> {
        Self::save_audit_impl(audit)
            .map_err(|e| AppError::Storage(e.to_string()))
    }

    fn load_audit(&self, audit_id: &Uuid) -> AppResult<Audit> {
        Self::load_audit_impl(audit_id)
            .map_err(|e| AppError::Load(e.to_string()))
    }

    fn list_audits(&self) -> AppResult<Vec<Uuid>> {
        Self::list_audits_impl()
            .map_err(|e| AppError::Load(e.to_string()))
    }

    fn delete_audit(&self, audit_id: &Uuid) -> AppResult<()> {
        Self::delete_audit_impl(audit_id)
            .map_err(|e| AppError::Storage(e.to_string()))
    }
}

impl WebStorage {
    /// Obtient le localStorage
    fn get_storage() -> Result<web_sys::Storage> {
        let window = window().context("Impossible d'accéder à la fenêtre")?;
        window
            .local_storage()
            .map_err(|e| anyhow::anyhow!("Erreur localStorage: {:?}", e))?
            .context("localStorage non disponible")
    }

    /// Sauvegarde un audit dans localStorage (implémentation interne)
    fn save_audit_impl(audit: &Audit) -> Result<()> {
        let storage = Self::get_storage()?;
        let key = format!("{}{}", STORAGE_KEY_PREFIX, audit.id);
        let json = serde_json::to_string(audit)
            .context("Erreur lors de la sérialisation de l'audit")?;
        
        storage
            .set_item(&key, &json)
            .map_err(|e| anyhow::anyhow!("Erreur lors de l'écriture: {:?}", e))?;

        // Mettre à jour l'index des audits
        Self::update_index(&storage, audit.id, true)?;

        Ok(())
    }

    /// Charge un audit depuis localStorage (implémentation interne)
    fn load_audit_impl(audit_id: &Uuid) -> Result<Audit> {
        let storage = Self::get_storage()?;
        let key = format!("{}{}", STORAGE_KEY_PREFIX, audit_id);
        
        let json = storage
            .get_item(&key)
            .map_err(|e| anyhow::anyhow!("Erreur lors de la lecture: {:?}", e))?
            .context(format!("Audit {} introuvable", audit_id))?;

        let audit: Audit = serde_json::from_str(&json)
            .context("Erreur lors de la désérialisation de l'audit")?;
        
        Ok(audit)
    }

    /// Liste tous les IDs d'audits disponibles (implémentation interne)
    fn list_audits_impl() -> Result<Vec<Uuid>> {
        let storage = Self::get_storage()?;
        
        let index_json = storage
            .get_item(STORAGE_INDEX_KEY)
            .map_err(|e| anyhow::anyhow!("Erreur lors de la lecture de l'index: {:?}", e))?
            .unwrap_or_else(|| "[]".to_string());

        let audit_ids: Vec<String> = serde_json::from_str(&index_json)
            .unwrap_or_else(|_| Vec::new());

        let mut uuids = Vec::new();
        for id_str in audit_ids {
            if let Ok(uuid) = Uuid::parse_str(&id_str) {
                uuids.push(uuid);
            }
        }

        Ok(uuids)
    }

    /// Supprime un audit (implémentation interne)
    fn delete_audit_impl(audit_id: &Uuid) -> Result<()> {
        let storage = Self::get_storage()?;
        let key = format!("{}{}", STORAGE_KEY_PREFIX, audit_id);
        
        storage
            .remove_item(&key)
            .map_err(|e| anyhow::anyhow!("Erreur lors de la suppression: {:?}", e))?;

        // Mettre à jour l'index
        Self::update_index(&storage, *audit_id, false)?;

        Ok(())
    }

    /// Met à jour l'index des audits
    fn update_index(
        storage: &web_sys::Storage,
        audit_id: Uuid,
        add: bool,
    ) -> Result<()> {
        let index_json = storage
            .get_item(STORAGE_INDEX_KEY)
            .map_err(|e| anyhow::anyhow!("Erreur lors de la lecture de l'index: {:?}", e))?
            .unwrap_or_else(|| "[]".to_string());

        let mut audit_ids: Vec<String> = serde_json::from_str(&index_json)
            .unwrap_or_else(|_| Vec::new());

        let id_str = audit_id.to_string();
        
        if add {
            if !audit_ids.contains(&id_str) {
                audit_ids.push(id_str);
            }
        } else {
            audit_ids.retain(|id| id != &id_str);
        }

        let updated_index = serde_json::to_string(&audit_ids)
            .context("Erreur lors de la sérialisation de l'index")?;

        storage
            .set_item(STORAGE_INDEX_KEY, &updated_index)
            .map_err(|e| anyhow::anyhow!("Erreur lors de l'écriture de l'index: {:?}", e))?;

        Ok(())
    }
}

