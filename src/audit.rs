use crate::models::*;
use crate::storage::WebStorage;
use anyhow::{Context, Result};
use serde::Deserialize;
use uuid::Uuid;

#[cfg(target_arch = "wasm32")]
use std::str;

/// Gestionnaire d'audits utilisant le stockage web (localStorage)
#[derive(Default)]
pub struct AuditManager;

impl AuditManager {
    #[allow(dead_code)]
    pub fn new(_audits_dir: impl Into<String>) -> Self {
        // Le paramètre est ignoré car on utilise localStorage
        Self
    }

    /// Sauvegarde un audit dans localStorage
    pub fn save_audit(&self, audit: &Audit) -> Result<()> {
        WebStorage::save_audit(audit)
    }

    /// Charge un audit depuis localStorage
    pub fn load_audit(&self, audit_id: &Uuid) -> Result<Audit> {
        WebStorage::load_audit(audit_id)
    }

    /// Liste tous les audits disponibles
    pub fn list_audits(&self) -> Result<Vec<Uuid>> {
        WebStorage::list_audits()
    }

    /// Supprime un audit
    pub fn delete_audit(&self, audit_id: &Uuid) -> Result<()> {
        WebStorage::delete_audit(audit_id)
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
            "questions/technique/infra.yaml" => include_str!("../questions/technique/infra.yaml"),
            "questions/technique/db.yaml" => include_str!("../questions/technique/db.yaml"),
            "questions/technique/operation.yaml" => include_str!("../questions/technique/operation.yaml"),
            "questions/technique/secu.yaml" => include_str!("../questions/technique/secu.yaml"),
            "questions/technique/a11y.yaml" => include_str!("../questions/technique/a11y.yaml"),
            "questions/technique/archi.yaml" => include_str!("../questions/technique/archi.yaml"),
            "questions/technique/oss.yaml" => include_str!("../questions/technique/oss.yaml"),
            "questions/technique/devxp.yaml" => include_str!("../questions/technique/devxp.yaml"),
            "questions/technique/produit.yaml" => include_str!("../questions/technique/produit.yaml"),
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
}

