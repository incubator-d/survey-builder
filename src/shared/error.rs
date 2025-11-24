/// Gestion des erreurs de l'application
use std::fmt;

/// Type d'erreur de l'application
#[derive(Debug, Clone)]
pub enum AppError {
    /// Erreur de stockage
    Storage(String),
    /// Erreur de validation
    #[allow(dead_code)]
    Validation(String),
    /// Erreur de chargement
    Load(String),
    /// Erreur générique
    #[allow(dead_code)]
    Generic(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Storage(msg) => write!(f, "Erreur de stockage: {}", msg),
            AppError::Validation(msg) => write!(f, "Erreur de validation: {}", msg),
            AppError::Load(msg) => write!(f, "Erreur de chargement: {}", msg),
            AppError::Generic(msg) => write!(f, "Erreur: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

/// Résultat avec erreur d'application
pub type AppResult<T> = Result<T, AppError>;
