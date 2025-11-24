use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Type d'audit (Technique, Business, Marketing, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuditType {
    Technique,
    Business,
    Marketing,
    Autre(String),
}

impl std::fmt::Display for AuditType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditType::Technique => write!(f, "Technique"),
            AuditType::Business => write!(f, "Business"),
            AuditType::Marketing => write!(f, "Marketing"),
            AuditType::Autre(s) => write!(f, "{}", s),
        }
    }
}

/// Catégories d'audit (Infra, Opération, DB, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Category {
    Infra,
    Operation,
    DB,
    A11y, // Accessibilité
    Archi, // Architecture
    Intra,
    OSS, // Open Source
    DevXP, // Developer Experience
    Produit,
    Secu, // Sécurité
    Bizdev,
    Autre(String),
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Category::Infra => write!(f, "Infra"),
            Category::Operation => write!(f, "Opération"),
            Category::DB => write!(f, "DB"),
            Category::A11y => write!(f, "A11y"),
            Category::Archi => write!(f, "Archi"),
            Category::Intra => write!(f, "Intra"),
            Category::OSS => write!(f, "OSS"),
            Category::DevXP => write!(f, "Dev XP"),
            Category::Produit => write!(f, "Produit"),
            Category::Secu => write!(f, "Sécu"),
            Category::Bizdev => write!(f, "Bizdev"),
            Category::Autre(s) => write!(f, "{}", s),
        }
    }
}

/// Réponse possible à une question
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Answer {
    Oui,
    Non,
    JeNeSaisPas,
    NonApplicable,
}

impl std::fmt::Display for Answer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Answer::Oui => write!(f, "Oui"),
            Answer::Non => write!(f, "Non"),
            Answer::JeNeSaisPas => write!(f, "Je ne sais pas"),
            Answer::NonApplicable => write!(f, "Non applicable"),
        }
    }
}

/// Réponse à une question avec commentaire et pourcentage optionnel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionAnswer {
    pub question_id: Uuid,
    pub answer: Answer,
    pub percentage: Option<u8>, // 0-100, utilisé pour certaines questions
    pub comment: Option<String>,
    pub answered_at: DateTime<Utc>,
}

/// Question d'audit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    pub id: Uuid,
    pub text: String,
    pub description: Option<String>, // Description détaillée de la question
    pub category: Category,
    pub audit_type: AuditType,
    pub requires_percentage: bool, // Si la question nécessite un pourcentage
    pub expected_answer: Option<Answer>, // Réponse généralement attendue (Oui par défaut)
    pub links: Vec<String>, // Liens utiles pour la question
}

/// Recommandation générée à partir des réponses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub category: Category,
    pub priority: Priority,
    pub question_ids: Vec<Uuid>, // Questions qui ont déclenché cette recommandation
}

/// Priorité d'une recommandation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Faible,
    Moyenne,
    Elevee,
    Critique,
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Priority::Faible => write!(f, "Faible"),
            Priority::Moyenne => write!(f, "Moyenne"),
            Priority::Elevee => write!(f, "Élevée"),
            Priority::Critique => write!(f, "Critique"),
        }
    }
}

/// Audit complet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Audit {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub audit_type: AuditType,
    pub project_name: Option<String>,
    pub product_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub questions: Vec<Question>,
    pub answers: Vec<QuestionAnswer>,
    pub recommendations: Vec<Recommendation>,
}

impl Audit {
    /// Crée un nouvel audit avec validation
    /// 
    /// # Arguments
    /// * `name` - Nom de l'audit (ne doit pas être vide)
    /// * `audit_type` - Type d'audit
    /// 
    /// # Panics
    /// Panique si le nom est vide (devrait être validé avant l'appel)
    pub fn new(name: String, audit_type: AuditType) -> Self {
        // Validation: le nom ne doit pas être vide
        assert!(!name.trim().is_empty(), "Le nom de l'audit ne peut pas être vide");
        
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.trim().to_string(),
            description: None,
            audit_type,
            project_name: None,
            product_name: None,
            created_at: now,
            updated_at: now,
            questions: Vec::new(),
            answers: Vec::new(),
            recommendations: Vec::new(),
        }
    }

    /// Ajoute une question à l'audit
    pub fn add_question(&mut self, question: Question) {
        self.questions.push(question);
        self.updated_at = Utc::now();
    }

    /// Enregistre une réponse à une question
    pub fn answer_question(&mut self, answer: QuestionAnswer) {
        // Retire l'ancienne réponse si elle existe
        self.answers.retain(|a| a.question_id != answer.question_id);
        self.answers.push(answer);
        self.updated_at = Utc::now();
    }

    /// Obtient la réponse à une question
    pub fn get_answer(&self, question_id: &Uuid) -> Option<&QuestionAnswer> {
        self.answers.iter().find(|a| a.question_id == *question_id)
    }

    /// Vérifie si une question a été répondue
    #[allow(dead_code)]
    pub fn is_answered(&self, question_id: &Uuid) -> bool {
        self.answers.iter().any(|a| a.question_id == *question_id)
    }

    /// Calcule le pourcentage de complétion de l'audit
    pub fn completion_percentage(&self) -> f64 {
        if self.questions.is_empty() {
            return 0.0;
        }
        let answered = self.answers.len() as f64;
        let total = self.questions.len() as f64;
        (answered / total) * 100.0
    }

    /// Obtient les questions par catégorie
    #[allow(dead_code)]
    pub fn questions_by_category(&self, category: &Category) -> Vec<&Question> {
        self.questions
            .iter()
            .filter(|q| &q.category == category)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_new() {
        let audit = Audit::new("Test Audit".to_string(), AuditType::Technique);
        assert_eq!(audit.name, "Test Audit");
        assert_eq!(audit.audit_type, AuditType::Technique);
        assert_eq!(audit.questions.len(), 0);
        assert_eq!(audit.answers.len(), 0);
    }

    #[test]
    fn test_audit_add_question() {
        let mut audit = Audit::new("Test".to_string(), AuditType::Technique);
        let question = Question {
            id: Uuid::new_v4(),
            text: "Test question".to_string(),
            description: None,
            category: Category::Infra,
            audit_type: AuditType::Technique,
            requires_percentage: false,
            expected_answer: None,
            links: Vec::new(),
        };
        
        audit.add_question(question.clone());
        assert_eq!(audit.questions.len(), 1);
        assert_eq!(audit.questions[0].text, "Test question");
    }

    #[test]
    fn test_audit_answer_question() {
        let mut audit = Audit::new("Test".to_string(), AuditType::Technique);
        let question_id = Uuid::new_v4();
        let question = Question {
            id: question_id,
            text: "Test question".to_string(),
            description: None,
            category: Category::Infra,
            audit_type: AuditType::Technique,
            requires_percentage: false,
            expected_answer: None,
            links: Vec::new(),
        };
        
        audit.add_question(question);
        
        let answer = QuestionAnswer {
            question_id,
            answer: Answer::Oui,
            percentage: None,
            comment: None,
            answered_at: Utc::now(),
        };
        
        audit.answer_question(answer);
        assert_eq!(audit.answers.len(), 1);
        assert!(audit.is_answered(&question_id));
    }

    #[test]
    fn test_audit_get_answer() {
        let mut audit = Audit::new("Test".to_string(), AuditType::Technique);
        let question_id = Uuid::new_v4();
        let question = Question {
            id: question_id,
            text: "Test question".to_string(),
            description: None,
            category: Category::Infra,
            audit_type: AuditType::Technique,
            requires_percentage: false,
            expected_answer: None,
            links: Vec::new(),
        };
        
        audit.add_question(question);
        
        let answer = QuestionAnswer {
            question_id,
            answer: Answer::Non,
            percentage: None,
            comment: Some("Test comment".to_string()),
            answered_at: Utc::now(),
        };
        
        audit.answer_question(answer.clone());
        
        let retrieved = audit.get_answer(&question_id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().answer, Answer::Non);
        assert_eq!(retrieved.unwrap().comment.as_ref().unwrap(), "Test comment");
    }

    #[test]
    fn test_audit_completion_percentage() {
        let mut audit = Audit::new("Test".to_string(), AuditType::Technique);
        
        // Audit vide
        assert_eq!(audit.completion_percentage(), 0.0);
        
        // Ajouter 3 questions
        for i in 0..3 {
            let question = Question {
                id: Uuid::new_v4(),
                text: format!("Question {}", i),
                description: None,
                category: Category::Infra,
                audit_type: AuditType::Technique,
                requires_percentage: false,
                expected_answer: None,
                links: Vec::new(),
            };
            audit.add_question(question);
        }
        
        // 0% complété
        assert_eq!(audit.completion_percentage(), 0.0);
        
        // Répondre à 1 question sur 3
        let answer = QuestionAnswer {
            question_id: audit.questions[0].id,
            answer: Answer::Oui,
            percentage: None,
            comment: None,
            answered_at: Utc::now(),
        };
        audit.answer_question(answer);
        
        // ~33.3% complété
        let completion = audit.completion_percentage();
        assert!(completion > 33.0 && completion < 34.0);
        
        // Répondre à toutes les questions
        let question_ids: Vec<Uuid> = audit.questions[1..].iter().map(|q| q.id).collect();
        for question_id in question_ids {
            let answer = QuestionAnswer {
                question_id,
                answer: Answer::Oui,
                percentage: None,
                comment: None,
                answered_at: Utc::now(),
            };
            audit.answer_question(answer);
        }
        
        // 100% complété
        assert_eq!(audit.completion_percentage(), 100.0);
    }

    #[test]
    fn test_audit_questions_by_category() {
        let mut audit = Audit::new("Test".to_string(), AuditType::Technique);
        
        // Ajouter des questions de différentes catégories
        let infra_question = Question {
            id: Uuid::new_v4(),
            text: "Infra question".to_string(),
            description: None,
            category: Category::Infra,
            audit_type: AuditType::Technique,
            requires_percentage: false,
            expected_answer: None,
            links: Vec::new(),
        };
        
        let secu_question = Question {
            id: Uuid::new_v4(),
            text: "Secu question".to_string(),
            description: None,
            category: Category::Secu,
            audit_type: AuditType::Technique,
            requires_percentage: false,
            expected_answer: None,
            links: Vec::new(),
        };
        
        audit.add_question(infra_question);
        audit.add_question(secu_question);
        
        let infra_questions = audit.questions_by_category(&Category::Infra);
        assert_eq!(infra_questions.len(), 1);
        assert_eq!(infra_questions[0].text, "Infra question");
        
        let secu_questions = audit.questions_by_category(&Category::Secu);
        assert_eq!(secu_questions.len(), 1);
        assert_eq!(secu_questions[0].text, "Secu question");
    }

    #[test]
    fn test_audit_replace_answer() {
        let mut audit = Audit::new("Test".to_string(), AuditType::Technique);
        let question_id = Uuid::new_v4();
        let question = Question {
            id: question_id,
            text: "Test question".to_string(),
            description: None,
            category: Category::Infra,
            audit_type: AuditType::Technique,
            requires_percentage: false,
            expected_answer: None,
            links: Vec::new(),
        };
        
        audit.add_question(question);
        
        // Première réponse
        let answer1 = QuestionAnswer {
            question_id,
            answer: Answer::Oui,
            percentage: None,
            comment: None,
            answered_at: Utc::now(),
        };
        audit.answer_question(answer1);
        assert_eq!(audit.answers.len(), 1);
        assert_eq!(audit.get_answer(&question_id).unwrap().answer, Answer::Oui);
        
        // Remplacer par une nouvelle réponse
        let answer2 = QuestionAnswer {
            question_id,
            answer: Answer::Non,
            percentage: None,
            comment: None,
            answered_at: Utc::now(),
        };
        audit.answer_question(answer2);
        assert_eq!(audit.answers.len(), 1); // Toujours une seule réponse
        assert_eq!(audit.get_answer(&question_id).unwrap().answer, Answer::Non);
    }

    #[test]
    fn test_audit_type_display() {
        assert_eq!(format!("{}", AuditType::Technique), "Technique");
        assert_eq!(format!("{}", AuditType::Business), "Business");
        assert_eq!(format!("{}", AuditType::Marketing), "Marketing");
        assert_eq!(format!("{}", AuditType::Autre("Custom".to_string())), "Custom");
    }

    #[test]
    fn test_category_display() {
        assert_eq!(format!("{}", Category::Infra), "Infra");
        assert_eq!(format!("{}", Category::Operation), "Opération");
        assert_eq!(format!("{}", Category::DB), "DB");
        assert_eq!(format!("{}", Category::Secu), "Sécu");
        assert_eq!(format!("{}", Category::Autre("Custom".to_string())), "Custom");
    }

    #[test]
    fn test_answer_display() {
        assert_eq!(format!("{}", Answer::Oui), "Oui");
        assert_eq!(format!("{}", Answer::Non), "Non");
        assert_eq!(format!("{}", Answer::JeNeSaisPas), "Je ne sais pas");
        assert_eq!(format!("{}", Answer::NonApplicable), "Non applicable");
    }

    #[test]
    fn test_priority_display() {
        assert_eq!(format!("{}", Priority::Faible), "Faible");
        assert_eq!(format!("{}", Priority::Moyenne), "Moyenne");
        assert_eq!(format!("{}", Priority::Elevee), "Élevée");
        assert_eq!(format!("{}", Priority::Critique), "Critique");
    }

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::Critique > Priority::Elevee);
        assert!(Priority::Elevee > Priority::Moyenne);
        assert!(Priority::Moyenne > Priority::Faible);
    }
}


