//! Utilitaires pour le calcul des scores et statistiques

use crate::domain::models::Audit;
use crate::domain::recommendations::RecommendationEngine;

/// Calcule le score global d'un audit
/// 
/// Le score global est le pourcentage de réponses positives sur toutes les réponses données.
/// 
/// # Arguments
/// * `audit` - L'audit pour lequel calculer le score
/// 
/// # Returns
/// * `Some(score)` - Le score en pourcentage (0-100) si des réponses existent
/// * `None` - Si aucune réponse n'a été donnée
pub fn calculate_global_score(audit: &Audit) -> Option<f64> {
    let summary = RecommendationEngine::generate_summary(audit);
    if summary.category_stats.is_empty() {
        return None;
    }
    
    let total_positive: usize = summary.category_stats.iter()
        .map(|s| s.positive_answers)
        .sum();
    let total_answered: usize = summary.category_stats.iter()
        .map(|s| s.answered)
        .sum();
    
    if total_answered > 0 {
        Some((total_positive as f64 / total_answered as f64) * 100.0)
    } else {
        None
    }
}

