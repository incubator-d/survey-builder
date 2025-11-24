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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::*;
    use uuid::Uuid;

    #[test]
    fn test_calculate_global_score_empty_audit() {
        let audit = Audit::new("Test".to_string(), AuditType::Technique);
        let score = calculate_global_score(&audit);
        assert_eq!(score, None);
    }

    #[test]
    fn test_calculate_global_score_with_answers() {
        let mut audit = Audit::new("Test".to_string(), AuditType::Technique);
        
        // Ajouter des questions
        let question1 = Question {
            id: Uuid::new_v4(),
            text: "Question 1".to_string(),
            description: None,
            category: Category::Infra,
            audit_type: AuditType::Technique,
            requires_percentage: false,
            expected_answer: None,
            links: Vec::new(),
        };
        let question2 = Question {
            id: Uuid::new_v4(),
            text: "Question 2".to_string(),
            description: None,
            category: Category::Infra,
            audit_type: AuditType::Technique,
            requires_percentage: false,
            expected_answer: None,
            links: Vec::new(),
        };
        
        audit.add_question(question1.clone());
        audit.add_question(question2.clone());
        
        // Répondre "Oui" à la première, "Non" à la seconde
        audit.answer_question(QuestionAnswer {
            question_id: question1.id,
            answer: Answer::Oui,
            percentage: None,
            comment: None,
            answered_at: chrono::Utc::now(),
        });
        audit.answer_question(QuestionAnswer {
            question_id: question2.id,
            answer: Answer::Non,
            percentage: None,
            comment: None,
            answered_at: chrono::Utc::now(),
        });
        
        let score = calculate_global_score(&audit);
        assert!(score.is_some());
        // 1 positif sur 2 réponses = 50%
        assert_eq!(score.unwrap(), 50.0);
    }

    #[test]
    fn test_calculate_global_score_all_positive() {
        let mut audit = Audit::new("Test".to_string(), AuditType::Technique);
        
        let question = Question {
            id: Uuid::new_v4(),
            text: "Question".to_string(),
            description: None,
            category: Category::Infra,
            audit_type: AuditType::Technique,
            requires_percentage: false,
            expected_answer: None,
            links: Vec::new(),
        };
        
        audit.add_question(question.clone());
        audit.answer_question(QuestionAnswer {
            question_id: question.id,
            answer: Answer::Oui,
            percentage: None,
            comment: None,
            answered_at: chrono::Utc::now(),
        });
        
        let score = calculate_global_score(&audit);
        assert!(score.is_some());
        assert_eq!(score.unwrap(), 100.0);
    }

    #[test]
    fn test_calculate_global_score_all_negative() {
        let mut audit = Audit::new("Test".to_string(), AuditType::Technique);
        
        let question = Question {
            id: Uuid::new_v4(),
            text: "Question".to_string(),
            description: None,
            category: Category::Infra,
            audit_type: AuditType::Technique,
            requires_percentage: false,
            expected_answer: None,
            links: Vec::new(),
        };
        
        audit.add_question(question.clone());
        audit.answer_question(QuestionAnswer {
            question_id: question.id,
            answer: Answer::Non,
            percentage: None,
            comment: None,
            answered_at: chrono::Utc::now(),
        });
        
        let score = calculate_global_score(&audit);
        assert!(score.is_some());
        assert_eq!(score.unwrap(), 0.0);
    }
}

