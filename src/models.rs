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
    pub fn new(name: String, audit_type: AuditType) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
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
    pub fn questions_by_category(&self, category: &Category) -> Vec<&Question> {
        self.questions
            .iter()
            .filter(|q| &q.category == category)
            .collect()
    }
}


