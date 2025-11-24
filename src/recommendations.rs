use crate::models::*;
use serde::Serialize;
use std::collections::HashMap;
use uuid::Uuid;

/// Générateur de recommandations basé sur les réponses d'un audit
#[allow(dead_code)]
pub struct RecommendationEngine;

impl RecommendationEngine {
    /// Génère des recommandations à partir des réponses d'un audit
    #[allow(dead_code)]
    pub fn generate_recommendations(audit: &Audit) -> Vec<Recommendation> {
        let mut recommendations = Vec::new();
        let mut category_scores: HashMap<Category, (u32, u32)> = HashMap::new(); // (réponses positives, total)

        // Analyse des réponses par catégorie
        for question in &audit.questions {
            let entry = category_scores.entry(question.category.clone()).or_insert((0, 0));
            entry.1 += 1;

            if let Some(answer) = audit.get_answer(&question.id) {
                // Si la réponse attendue est "Oui" mais qu'on a répondu "Non"
                if let Some(expected) = &question.expected_answer {
                    if expected == &Answer::Oui && answer.answer == Answer::Non {
                        // Créer une recommandation pour cette question
                        recommendations.push(Recommendation {
                            id: Uuid::new_v4(),
                            title: format!("Améliorer: {}", question.text),
                            description: format!(
                                "{}",
                                question.description.as_deref().unwrap_or("")
                            ),
                            category: question.category.clone(),
                            priority: Self::calculate_priority(&question.category, &answer.answer),
                            question_ids: vec![question.id],
                        });
                    } else if answer.answer == Answer::Oui {
                        entry.0 += 1;
                    }
                }
            }
        }

        // Recommandations globales par catégorie
        for (category, (positive, total)) in category_scores {
            if total > 0 {
                let score = (positive as f64 / total as f64) * 100.0;
                if score < 50.0 {
                    recommendations.push(Recommendation {
                        id: Uuid::new_v4(),
                        title: format!("Améliorer la catégorie {}", category),
                        description: format!(
                            "La catégorie {} a un score de {:.1}%. Il est recommandé de revoir les pratiques dans ce domaine.",
                            category, score
                        ),
                        category: category.clone(),
                        priority: if score < 25.0 {
                            Priority::Critique
                        } else if score < 40.0 {
                            Priority::Elevee
                        } else {
                            Priority::Moyenne
                        },
                        question_ids: audit
                            .questions
                            .iter()
                            .filter(|q| q.category == category)
                            .map(|q| q.id)
                            .collect(),
                    });
                }
            }
        }

        recommendations
    }

    /// Calcule la priorité d'une recommandation basée sur la catégorie et la réponse
    #[allow(dead_code)]
    fn calculate_priority(category: &Category, answer: &Answer) -> Priority {
        match (category, answer) {
            // Sécurité et infrastructure sont critiques
            (Category::Secu, Answer::Non) => Priority::Critique,
            (Category::Infra, Answer::Non) => Priority::Elevee,
            (Category::DB, Answer::Non) => Priority::Elevee,
            (Category::Operation, Answer::Non) => Priority::Elevee,
            // Autres catégories
            _ => Priority::Moyenne,
        }
    }

    /// Génère un rapport de synthèse de l'audit
    #[allow(dead_code)]
    pub fn generate_summary(audit: &Audit) -> AuditSummary {
        let mut category_stats: HashMap<Category, CategoryStats> = HashMap::new();

        for question in &audit.questions {
            let stats = category_stats
                .entry(question.category.clone())
                .or_insert_with(|| CategoryStats {
                    category: question.category.clone(),
                    total_questions: 0,
                    answered: 0,
                    positive_answers: 0,
                    negative_answers: 0,
                });

            stats.total_questions += 1;
            if let Some(answer) = audit.get_answer(&question.id) {
                stats.answered += 1;
                match answer.answer {
                    Answer::Oui => stats.positive_answers += 1,
                    Answer::Non => stats.negative_answers += 1,
                    _ => {}
                }
            }
        }

        AuditSummary {
            audit_id: audit.id,
            completion_percentage: audit.completion_percentage(),
            category_stats: category_stats.into_values().collect(),
            total_questions: audit.questions.len(),
            total_answered: audit.answers.len(),
        }
    }
}

/// Statistiques par catégorie
#[derive(Debug, Clone, Serialize)]
pub struct CategoryStats {
    pub category: Category,
    pub total_questions: usize,
    pub answered: usize,
    pub positive_answers: usize,
    pub negative_answers: usize,
}

impl CategoryStats {
    #[allow(dead_code)]
    pub fn completion_percentage(&self) -> f64 {
        if self.total_questions == 0 {
            return 0.0;
        }
        (self.answered as f64 / self.total_questions as f64) * 100.0
    }

    #[allow(dead_code)]
    pub fn positive_percentage(&self) -> f64 {
        if self.answered == 0 {
            return 0.0;
        }
        (self.positive_answers as f64 / self.answered as f64) * 100.0
    }
}

/// Résumé d'un audit
#[derive(Debug, Clone, Serialize)]
pub struct AuditSummary {
    pub audit_id: Uuid,
    pub completion_percentage: f64,
    pub category_stats: Vec<CategoryStats>,
    pub total_questions: usize,
    pub total_answered: usize,
}

