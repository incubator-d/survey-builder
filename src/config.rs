/// Configuration de l'application
/// Contient toutes les constantes et seuils configurables

/// Seuils de score pour les recommandations
pub mod score_thresholds {
    /// Score excellent (vert)
    pub const EXCELLENT: f64 = 75.0;
    /// Score bon (jaune)
    pub const GOOD: f64 = 50.0;
    /// Score moyen (orange)
    pub const MEDIUM: f64 = 25.0;
    
    /// Seuil pour générer une recommandation globale de catégorie
    pub const CATEGORY_RECOMMENDATION_THRESHOLD: f64 = 50.0;
    /// Seuil pour priorité critique
    pub const CRITICAL_PRIORITY_THRESHOLD: f64 = 25.0;
    /// Seuil pour priorité élevée
    pub const HIGH_PRIORITY_THRESHOLD: f64 = 40.0;
}

/// Valeurs de réponse acceptées
pub mod answer_inputs {
    pub const OUI: &[&str] = &["oui", "o"];
    pub const NON: &[&str] = &["non", "n"];
    pub const JE_NE_SAIS_PAS: &[&str] = &["je ne sais pas", "jsp", "j"];
    pub const NON_APPLICABLE: &[&str] = &["non applicable", "na", "a"];
}

/// Configuration du scroll
pub mod scroll {
    /// Nombre de lignes à scroller par action
    pub const RECOMMENDATIONS_SCROLL_STEP: usize = 3;
}

/// Types d'audit disponibles
pub const AUDIT_TYPES: &[&str] = &["technique", "business", "marketing"];

