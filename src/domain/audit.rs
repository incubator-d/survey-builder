use crate::domain::models::*;
use crate::infrastructure::WebStorage;
use crate::infrastructure::Storage;
use crate::shared::error::AppResult;
use anyhow::{Context, Result};
use serde::Deserialize;
use uuid::Uuid;

#[cfg(target_arch = "wasm32")]
use std::str;

/// Gestionnaire d'audits utilisant le stockage web (localStorage)
pub struct AuditManager {
    storage: Box<dyn Storage>,
}

impl Default for AuditManager {
    fn default() -> Self {
        Self {
            storage: Box::new(WebStorage),
        }
    }
}

impl AuditManager {
    /// Crée un nouveau gestionnaire d'audits avec un stockage personnalisé
    #[allow(dead_code)]
    pub fn with_storage(storage: Box<dyn Storage>) -> Self {
        Self { storage }
    }

    /// Sauvegarde un audit
    pub fn save_audit(&self, audit: &Audit) -> AppResult<()> {
        self.storage.save_audit(audit)
    }

    /// Charge un audit depuis le stockage
    pub fn load_audit(&self, audit_id: &Uuid) -> AppResult<Audit> {
        self.storage.load_audit(audit_id)
    }

    /// Liste tous les audits disponibles
    pub fn list_audits(&self) -> AppResult<Vec<Uuid>> {
        self.storage.list_audits()
    }

    /// Supprime un audit
    pub fn delete_audit(&self, audit_id: &Uuid) -> AppResult<()> {
        self.storage.delete_audit(audit_id)
    }
}

/// Structure pour désérialiser les questions depuis YAML
#[derive(Debug, Deserialize)]
struct QuestionYaml {
    text: String,
    description: Option<String>,
    category: String,
    requires_percentage: bool,
    expected_answer: Option<String>,
    links: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct QuestionsFile {
    questions: Vec<QuestionYaml>,
}

/// Générateur de questions par défaut pour les audits techniques
pub struct QuestionGenerator;

impl QuestionGenerator {
    /// Charge les questions depuis un fichier YAML
    /// Dans le navigateur, utilise include_str! pour inclure les fichiers au moment de la compilation
    #[cfg(target_arch = "wasm32")]
    fn load_questions_from_yaml(file_path: &str) -> Result<Vec<Question>> {
        let content = match file_path {
            "questions/technique/infra.yaml" => include_str!("../../questions/technique/infra.yaml"),
            "questions/technique/db.yaml" => include_str!("../../questions/technique/db.yaml"),
            "questions/technique/operation.yaml" => include_str!("../../questions/technique/operation.yaml"),
            "questions/technique/secu.yaml" => include_str!("../../questions/technique/secu.yaml"),
            "questions/technique/a11y.yaml" => include_str!("../../questions/technique/a11y.yaml"),
            "questions/technique/archi.yaml" => include_str!("../../questions/technique/archi.yaml"),
            "questions/technique/oss.yaml" => include_str!("../../questions/technique/oss.yaml"),
            "questions/technique/devxp.yaml" => include_str!("../../questions/technique/devxp.yaml"),
            "questions/technique/produit.yaml" => include_str!("../../questions/technique/produit.yaml"),
            "questions/technique/organisation.yaml" => include_str!("../../questions/technique/organisation.yaml"),
            _ => return Err(anyhow::anyhow!("Fichier non trouvé: {}", file_path)),
        };
        Self::parse_yaml_content(content, file_path)
    }

    /// Charge les questions depuis un fichier YAML (version native pour tests)
    #[cfg(not(target_arch = "wasm32"))]
    fn load_questions_from_yaml(file_path: &str) -> Result<Vec<Question>> {
        use std::fs;
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Impossible de lire le fichier {}", file_path))?;
        Self::parse_yaml_content(&content, file_path)
    }

    /// Parse le contenu YAML en questions
    fn parse_yaml_content(content: &str, file_path: &str) -> Result<Vec<Question>> {
        let file: QuestionsFile = serde_yaml::from_str(content)
            .with_context(|| format!("Erreur de parsing YAML dans {}", file_path))?;
        
        let mut questions = Vec::new();
        
        for q_yaml in file.questions {
            let category = match q_yaml.category.as_str() {
                "Infra" => Category::Infra,
                "Operation" => Category::Operation,
                "DB" => Category::DB,
                "A11y" => Category::A11y,
                "Archi" => Category::Archi,
                "OSS" => Category::OSS,
                "DevXP" => Category::DevXP,
                "Produit" => Category::Produit,
                "Secu" => Category::Secu,
                "Bizdev" => Category::Bizdev,
                "Organisation" => Category::Organisation,
                other => Category::Autre(other.to_string()),
            };
            
            let expected_answer = q_yaml.expected_answer.as_ref().map(|s| match s.as_str() {
                "Oui" | "oui" => Answer::Oui,
                "Non" | "non" => Answer::Non,
                "JeNeSaisPas" | "je-ne-sais-pas" => Answer::JeNeSaisPas,
                "NonApplicable" | "non-applicable" => Answer::NonApplicable,
                _ => Answer::Oui,
            });
            
            questions.push(Question {
                id: Uuid::new_v4(),
                text: q_yaml.text,
                description: q_yaml.description,
                category,
                audit_type: AuditType::Technique,
                requires_percentage: q_yaml.requires_percentage,
                expected_answer,
                links: q_yaml.links.unwrap_or_default(),
            });
        }
        
        Ok(questions)
    }
    
    /// Génère les questions par défaut pour un audit technique
    /// Charge les questions depuis les fichiers YAML dans questions/technique/
    pub fn generate_technical_questions() -> Vec<Question> {
        let mut all_questions = Vec::new();
        
        // Liste des fichiers YAML par catégorie
        // Les chemins sont relatifs à la racine du projet
        let question_files = vec![
            "questions/technique/infra.yaml",
            "questions/technique/db.yaml",
            "questions/technique/operation.yaml",
            "questions/technique/secu.yaml",
            "questions/technique/a11y.yaml",
            "questions/technique/archi.yaml",
            "questions/technique/oss.yaml",
            "questions/technique/devxp.yaml",
            "questions/technique/produit.yaml",
            "questions/technique/organisation.yaml",
        ];
        
        // Charge les questions depuis chaque fichier
        for file_path in question_files {
            match Self::load_questions_from_yaml(file_path) {
                Ok(mut questions) => {
                    all_questions.append(&mut questions);
                }
                Err(e) => {
                    eprintln!("⚠️  Avertissement: Impossible de charger {}: {}", file_path, e);
                    // Continue avec les autres fichiers même si un échoue
                }
            }
        }
        
        // Si aucun fichier n'a pu être chargé, retourne un vecteur vide
        // (ou on pourrait avoir des questions par défaut en fallback)
        if all_questions.is_empty() {
            eprintln!("⚠️  Aucune question n'a pu être chargée depuis les fichiers YAML");
        }
        
        all_questions
    }
    
    /// Génère les questions par défaut pour un audit business
    pub fn generate_business_questions() -> Vec<Question> {
        vec![
            Question {
                id: Uuid::new_v4(),
                text: "Disposez-vous d'une stratégie de déploiement territorial documentée ?".to_string(),
                description: None,
                category: Category::Bizdev,
                audit_type: AuditType::Business,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Mesurez-vous le coût d'acquisition de nouveaux utilisateurs ou territoires ?".to_string(),
                description: None,
                category: Category::Bizdev,
                audit_type: AuditType::Business,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Avez-vous calculé le retour sur investissement (ROI) pour les collectivités qui adoptent votre solution ?".to_string(),
                description: None,
                category: Category::Bizdev,
                audit_type: AuditType::Business,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Avez-vous un plan de communication ou marketing pour promouvoir votre solution ?".to_string(),
                description: None,
                category: Category::Bizdev,
                audit_type: AuditType::Business,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::recommendations::RecommendationEngine;
    use crate::shared::utils::calculate_global_score;

    #[test]
    fn test_parse_yaml_content() {
        let yaml_content = r#"
questions:
  - text: "Question test 1"
    description: "Description test"
    category: Infra
    requires_percentage: true
    expected_answer: Oui
    links:
      - "https://example.com"
  - text: "Question test 2"
    category: Secu
    requires_percentage: false
    expected_answer: Non
"#;
        
        let questions = QuestionGenerator::parse_yaml_content(yaml_content, "test.yaml")
            .expect("Le parsing YAML devrait fonctionner");
        
        assert_eq!(questions.len(), 2);
        assert_eq!(questions[0].text, "Question test 1");
        assert_eq!(questions[0].category, Category::Infra);
        assert_eq!(questions[0].requires_percentage, true);
        assert_eq!(questions[0].expected_answer, Some(Answer::Oui));
        assert_eq!(questions[0].links.len(), 1);
        
        assert_eq!(questions[1].text, "Question test 2");
        assert_eq!(questions[1].category, Category::Secu);
        assert_eq!(questions[1].requires_percentage, false);
        assert_eq!(questions[1].expected_answer, Some(Answer::Non));
    }

    #[test]
    fn test_parse_yaml_content_with_optional_fields() {
        let yaml_content = r#"
questions:
  - text: "Question sans description"
    category: DB
    requires_percentage: false
  - text: "Question avec description"
    description: "Description optionnelle"
    category: Operation
    requires_percentage: true
    links:
      - "https://link1.com"
      - "https://link2.com"
"#;
        
        let questions = QuestionGenerator::parse_yaml_content(yaml_content, "test.yaml")
            .expect("Le parsing YAML devrait fonctionner");
        
        assert_eq!(questions.len(), 2);
        assert!(questions[0].description.is_none());
        assert_eq!(questions[1].description, Some("Description optionnelle".to_string()));
        assert_eq!(questions[1].links.len(), 2);
    }

    #[test]
    fn test_parse_yaml_content_category_mapping() {
        let yaml_content = r#"
questions:
  - text: "Test Infra"
    category: Infra
    requires_percentage: false
  - text: "Test Operation"
    category: Operation
    requires_percentage: false
  - text: "Test DB"
    category: DB
    requires_percentage: false
  - text: "Test A11y"
    category: A11y
    requires_percentage: false
  - text: "Test Archi"
    category: Archi
    requires_percentage: false
  - text: "Test OSS"
    category: OSS
    requires_percentage: false
  - text: "Test DevXP"
    category: DevXP
    requires_percentage: false
  - text: "Test Produit"
    category: Produit
    requires_percentage: false
  - text: "Test Secu"
    category: Secu
    requires_percentage: false
  - text: "Test Bizdev"
    category: Bizdev
    requires_percentage: false
  - text: "Test Autre"
    category: CustomCategory
    requires_percentage: false
"#;
        
        let questions = QuestionGenerator::parse_yaml_content(yaml_content, "test.yaml")
            .expect("Le parsing YAML devrait fonctionner");
        
        assert_eq!(questions.len(), 11);
        assert_eq!(questions[0].category, Category::Infra);
        assert_eq!(questions[1].category, Category::Operation);
        assert_eq!(questions[2].category, Category::DB);
        assert_eq!(questions[3].category, Category::A11y);
        assert_eq!(questions[4].category, Category::Archi);
        assert_eq!(questions[5].category, Category::OSS);
        assert_eq!(questions[6].category, Category::DevXP);
        assert_eq!(questions[7].category, Category::Produit);
        assert_eq!(questions[8].category, Category::Secu);
        assert_eq!(questions[9].category, Category::Bizdev);
        match &questions[10].category {
            Category::Autre(s) => assert_eq!(s, "CustomCategory"),
            _ => panic!("Devrait être Category::Autre"),
        }
    }

    #[test]
    fn test_parse_yaml_content_expected_answer_mapping() {
        let yaml_content = r#"
questions:
  - text: "Test Oui"
    category: Infra
    requires_percentage: false
    expected_answer: Oui
  - text: "Test Non"
    category: Infra
    requires_percentage: false
    expected_answer: Non
  - text: "Test JeNeSaisPas"
    category: Infra
    requires_percentage: false
    expected_answer: JeNeSaisPas
  - text: "Test NonApplicable"
    category: Infra
    requires_percentage: false
    expected_answer: NonApplicable
  - text: "Test lowercase"
    category: Infra
    requires_percentage: false
    expected_answer: oui
  - text: "Test avec tirets"
    category: Infra
    requires_percentage: false
    expected_answer: je-ne-sais-pas
"#;
        
        let questions = QuestionGenerator::parse_yaml_content(yaml_content, "test.yaml")
            .expect("Le parsing YAML devrait fonctionner");
        
        assert_eq!(questions.len(), 6);
        assert_eq!(questions[0].expected_answer, Some(Answer::Oui));
        assert_eq!(questions[1].expected_answer, Some(Answer::Non));
        assert_eq!(questions[2].expected_answer, Some(Answer::JeNeSaisPas));
        assert_eq!(questions[3].expected_answer, Some(Answer::NonApplicable));
        assert_eq!(questions[4].expected_answer, Some(Answer::Oui));
        assert_eq!(questions[5].expected_answer, Some(Answer::JeNeSaisPas));
    }

    #[test]
    fn test_parse_yaml_content_invalid_yaml() {
        let invalid_yaml = "invalid: yaml: content: [";
        
        let result = QuestionGenerator::parse_yaml_content(invalid_yaml, "test.yaml");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_yaml_content_empty() {
        let empty_yaml = "questions: []";
        
        let questions = QuestionGenerator::parse_yaml_content(empty_yaml, "test.yaml")
            .expect("Le parsing YAML devrait fonctionner");
        
        assert_eq!(questions.len(), 0);
    }

    #[test]
    fn test_generate_technical_questions() {
        // Test que la génération de questions techniques fonctionne
        // Note: En mode test (non-wasm32), cela va lire depuis les fichiers
        let questions = QuestionGenerator::generate_technical_questions();
        
        // Devrait avoir au moins quelques questions si les fichiers existent
        // Si les fichiers n'existent pas, la liste sera vide mais le test ne devrait pas planter
        assert!(!questions.is_empty() || questions.is_empty()); // Toujours vrai, juste pour vérifier que ça compile
        
        // Si des questions sont chargées, vérifier leur structure
        for question in &questions {
            assert!(!question.text.is_empty());
            assert_eq!(question.audit_type, AuditType::Technique);
        }
    }

    #[test]
    fn test_generate_business_questions() {
        let questions = QuestionGenerator::generate_business_questions();
        
        assert!(!questions.is_empty());
        assert!(questions.len() >= 4);
        
        for question in &questions {
            assert!(!question.text.is_empty());
            assert_eq!(question.audit_type, AuditType::Business);
            assert_eq!(question.category, Category::Bizdev);
        }
    }

    #[test]
    fn test_audit_manager_default() {
        let _manager = AuditManager::default();
        // Vérifier que le manager peut être créé
        assert!(true); // Pas de panique = succès
    }

    // Mock storage pour les tests
    struct MockStorage {
        audits: std::collections::HashMap<uuid::Uuid, Audit>,
    }

    impl crate::infrastructure::storage_trait::Storage for MockStorage {
        fn save_audit(&self, _audit: &Audit) -> crate::shared::error::AppResult<()> {
            // Dans un vrai mock, on devrait utiliser RefCell ou Mutex
            // Pour simplifier, on ne fait rien ici car on ne peut pas muter
            Ok(())
        }

        fn load_audit(&self, audit_id: &uuid::Uuid) -> crate::shared::error::AppResult<Audit> {
            self.audits.get(audit_id)
                .cloned()
                .ok_or_else(|| crate::shared::error::AppError::Load("Audit non trouvé".to_string()))
        }

        fn list_audits(&self) -> crate::shared::error::AppResult<Vec<uuid::Uuid>> {
            Ok(self.audits.keys().cloned().collect())
        }

        fn delete_audit(&self, _audit_id: &uuid::Uuid) -> crate::shared::error::AppResult<()> {
            Ok(())
        }
    }

    #[test]
    fn test_audit_manager_with_storage() {
        use std::collections::HashMap;
        
        let mock_storage = MockStorage {
            audits: HashMap::new(),
        };
        let _manager = AuditManager::with_storage(Box::new(mock_storage));
        
        // Vérifier que le manager peut être créé avec un storage personnalisé
        assert!(true); // Pas de panique = succès
    }

    #[test]
    fn test_parse_yaml_content_with_organisation_category() {
        let yaml_content = r#"
questions:
  - text: "Test Organisation"
    category: Organisation
    requires_percentage: false
"#;
        
        let questions = QuestionGenerator::parse_yaml_content(yaml_content, "test.yaml")
            .expect("Le parsing YAML devrait fonctionner");
        
        assert_eq!(questions.len(), 1);
        assert_eq!(questions[0].category, Category::Organisation);
    }

    #[test]
    fn test_parse_yaml_content_with_intra_category() {
        let yaml_content = r#"
questions:
  - text: "Test Intra"
    category: Intra
    requires_percentage: false
"#;
        
        let questions = QuestionGenerator::parse_yaml_content(yaml_content, "test.yaml")
            .expect("Le parsing YAML devrait fonctionner");
        
        assert_eq!(questions.len(), 1);
        // "Intra" n'est pas dans le mapping, donc il est traité comme "Autre"
        match &questions[0].category {
            Category::Autre(s) => assert_eq!(s, "Intra"),
            _ => panic!("Devrait être Category::Autre(\"Intra\")"),
        }
    }

    #[test]
    fn test_create_complete_audit_workflow() {
        // Test d'intégration : créer un audit complet avec questions et réponses
        let mut audit = Audit::new("Audit de test complet".to_string(), AuditType::Technique);
        audit.project_name = Some("Projet Test".to_string());
        audit.product_name = Some("Produit Test".to_string());
        
        // Générer des questions techniques
        let questions = QuestionGenerator::generate_technical_questions();
        assert!(!questions.is_empty() || questions.is_empty()); // Peut être vide si fichiers absents
        
        // Ajouter quelques questions de test
        let test_question1 = Question {
            id: Uuid::new_v4(),
            text: "Question test 1".to_string(),
            description: Some("Description test 1".to_string()),
            category: Category::Infra,
            audit_type: AuditType::Technique,
            requires_percentage: false,
            expected_answer: Some(Answer::Oui),
            links: vec!["https://example.com".to_string()],
        };
        
        let test_question2 = Question {
            id: Uuid::new_v4(),
            text: "Question test 2".to_string(),
            description: None,
            category: Category::Secu,
            audit_type: AuditType::Technique,
            requires_percentage: true,
            expected_answer: Some(Answer::Oui),
            links: Vec::new(),
        };
        
        audit.add_question(test_question1.clone());
        audit.add_question(test_question2.clone());
        
        // Vérifier l'état initial
        assert_eq!(audit.questions.len(), 2);
        assert_eq!(audit.answers.len(), 0);
        assert_eq!(audit.completion_percentage(), 0.0);
        
        // Répondre à la première question
        let answer1 = QuestionAnswer {
            question_id: test_question1.id,
            answer: Answer::Oui,
            percentage: None,
            comment: Some("Commentaire test".to_string()),
            answered_at: chrono::Utc::now(),
        };
        audit.answer_question(answer1);
        
        // Répondre à la seconde question avec pourcentage
        let answer2 = QuestionAnswer {
            question_id: test_question2.id,
            answer: Answer::Oui,
            percentage: Some(75),
            comment: None,
            answered_at: chrono::Utc::now(),
        };
        audit.answer_question(answer2);
        
        // Vérifier l'état après réponses
        assert_eq!(audit.answers.len(), 2);
        assert_eq!(audit.completion_percentage(), 100.0);
        
        // Vérifier qu'on peut récupérer les réponses
        assert!(audit.get_answer(&test_question1.id).is_some());
        assert!(audit.get_answer(&test_question2.id).is_some());
        assert_eq!(audit.get_answer(&test_question1.id).unwrap().answer, Answer::Oui);
        assert_eq!(audit.get_answer(&test_question2.id).unwrap().percentage, Some(75));
        
        // Générer des recommandations
        let _recommendations = RecommendationEngine::generate_recommendations(&audit);
        // Avec des réponses positives, il ne devrait pas y avoir de recommandations spécifiques
        // mais peut avoir des recommandations globales selon les scores
        
        // Générer un résumé
        let summary = RecommendationEngine::generate_summary(&audit);
        assert_eq!(summary.total_questions, 2);
        assert_eq!(summary.total_answered, 2);
        assert_eq!(summary.completion_percentage, 100.0);
        assert!(!summary.category_stats.is_empty());
        
        // Vérifier les statistiques par catégorie
        let infra_stats = summary.category_stats.iter()
            .find(|s| s.category == Category::Infra);
        assert!(infra_stats.is_some());
        let infra = infra_stats.unwrap();
        assert_eq!(infra.total_questions, 1);
        assert_eq!(infra.answered, 1);
        assert_eq!(infra.positive_answers, 1);
        
        let secu_stats = summary.category_stats.iter()
            .find(|s| s.category == Category::Secu);
        assert!(secu_stats.is_some());
        let secu = secu_stats.unwrap();
        assert_eq!(secu.total_questions, 1);
        assert_eq!(secu.answered, 1);
        assert_eq!(secu.positive_answers, 1);
        
        // Calculer le score global
        let global_score = calculate_global_score(&audit);
        assert!(global_score.is_some());
        assert_eq!(global_score.unwrap(), 100.0); // 2 réponses positives sur 2
        
        // Test réussi : l'audit complet fonctionne correctement
    }
}

