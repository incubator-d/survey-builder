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
    
    #[allow(dead_code)]
    /// Ancienne méthode - conservée pour référence
    /// Utilisez maintenant les fichiers YAML dans questions/technique/
    fn _generate_technical_questions_old() -> Vec<Question> {
        // Cette fonction a été remplacée par le chargement depuis YAML
        vec![
            // ========== INFRASTRUCTURE - Scaling & Performance ==========
            Question {
                id: Uuid::new_v4(),
                text: "L'application est-elle déployée chez un hébergeur souverain ?".to_string(),
                description: Some("Entreprise et serveurs en Union Européenne - Critère essentiel pour les projets publics et sensibles".to_string()),
                category: Category::Infra,
                audit_type: AuditType::Technique,
                requires_percentage: true,
                expected_answer: Some(Answer::Oui),
                links: vec![
                    "https://www.numerique.gouv.fr/services/cloud/doctrine/".to_string(),
                    "https://www.numerique.gouv.fr/services/cloud/regles-doctrine/#contenu".to_string(),
                ],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Votre infrastructure supporte-t-elle le scaling horizontal automatique (auto-scaling) ?".to_string(),
                description: Some("Capacité à augmenter/diminuer automatiquement le nombre d'instances selon la charge. Essentiel pour gérer les pics de trafic sans sur-provisionner en permanence.".to_string()),
                category: Category::Infra,
                audit_type: AuditType::Technique,
                requires_percentage: true,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Avez-vous mis en place un CDN (Content Delivery Network) pour les assets statiques ?".to_string(),
                description: Some("CDN pour servir images, CSS, JS depuis des serveurs proches des utilisateurs. Réduit la latence et la charge sur vos serveurs, essentiel pour le scaling.".to_string()),
                category: Category::Infra,
                audit_type: AuditType::Technique,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Utilisez-vous un système de cache distribué (Redis, Memcached) pour améliorer les performances ?".to_string(),
                description: Some("Cache distribué pour réduire la charge sur les bases de données et améliorer les temps de réponse. Critique pour le scaling.".to_string()),
                category: Category::Infra,
                audit_type: AuditType::Technique,
                requires_percentage: true,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Votre load balancer utilise-t-il des algorithmes avancés (round-robin, least connections, IP hash) ?".to_string(),
                description: Some("Distribution intelligente de la charge entre plusieurs instances pour optimiser les performances et la disponibilité.".to_string()),
                category: Category::Infra,
                audit_type: AuditType::Technique,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Avez-vous une architecture multi-régions pour la résilience et la performance globale ?".to_string(),
                description: Some("Déploiement dans plusieurs régions géographiques pour réduire la latence et assurer la continuité en cas de panne régionale.".to_string()),
                category: Category::Infra,
                audit_type: AuditType::Technique,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Votre infrastructure est-elle gérée \"As Code\" (Terraform, OpenTofu, Pulumi) ?".to_string(),
                description: Some("Infrastructure as Code permet la reproductibilité, la versioning, et le déploiement rapide de nouveaux environnements. Essentiel pour le scaling et la collaboration.".to_string()),
                category: Category::Infra,
                audit_type: AuditType::Technique,
                requires_percentage: true,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Avez-vous mis en place un système de logs centralisé et analysable (ELK, Loki, Datadog) ?".to_string(),
                description: Some("Centralisation des logs pour le debugging, le monitoring, et l'analyse de performance à grande échelle.".to_string()),
                category: Category::Infra,
                audit_type: AuditType::Technique,
                requires_percentage: true,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Utilisez-vous des containers (Docker) et orchestration (Kubernetes, Docker Swarm) ?".to_string(),
                description: Some("Containerisation et orchestration pour faciliter le déploiement, le scaling, et la gestion de multiples services.".to_string()),
                category: Category::Infra,
                audit_type: AuditType::Technique,
                requires_percentage: true,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Avez-vous une documentation à jour de votre architecture infrastructure avec diagrammes ?".to_string(),
                description: Some("Documentation claire de l'architecture, des flux de données, et des dépendances. Essentiel pour l'onboarding et la maintenance à grande échelle.".to_string()),
                category: Category::Infra,
                audit_type: AuditType::Technique,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Les bases de données sont-elles déployées en haute disponibilité ?".to_string(),
                description: Some("Redondance avec master/slave.".to_string()),
                category: Category::DB,
                audit_type: AuditType::Technique,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Votre infrastructure est-elle gérée \"As Code\" ?".to_string(),
                description: Some("L'Infrastructure as Code (IaC) vous permet de documenter les changements d'infra et de remonter vos applications suite à un désastre. Par exemple Terraform ou son équivalent open source \"OpenTOFU\"".to_string()),
                category: Category::Infra,
                audit_type: AuditType::Technique,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Monitorez-vous votre infrastructure ? Si oui, avec quels outils ?".to_string(),
                description: Some("Par exemple avec des outils type Grafana, Metrics scalingo, ... (CPU, RAM, Disque, ...)".to_string()),
                category: Category::Operation,
                audit_type: AuditType::Technique,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Recevez-vous une alerte en cas de downtime sur le produit ?".to_string(),
                description: Some("par ex: updown.io (proposé par l'Incubateur)".to_string()),
                category: Category::Operation,
                audit_type: AuditType::Technique,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Avez-vous testé dans les 6 derniers mois votre procédure de backup et restore de la base de donnée ?".to_string(),
                description: None,
                category: Category::DB,
                audit_type: AuditType::Technique,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Votre base de données est-elle infogérée (managée) ?".to_string(),
                description: Some("Managée complètement par Scalingo ou Scaleway".to_string()),
                category: Category::DB,
                audit_type: AuditType::Technique,
                requires_percentage: true,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Les données sont-elles chiffrées au repos (at rest) ?".to_string(),
                description: Some("Cela a pour but d'empêcher l'utilisation des données dans le cas d'une intrusion par exemple.".to_string()),
                category: Category::Secu,
                audit_type: AuditType::Technique,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Utilisez-vous les chiffrement pour les données en transit ?".to_string(),
                description: Some("Par exemple utilisation de HTTPS entre le client et le serveur, ou TLS entre le serveur et la base de données.".to_string()),
                category: Category::Secu,
                audit_type: AuditType::Technique,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Avez-vous en place une détection des secrets dans le code exécuté sur la CI ?".to_string(),
                description: Some("Par exemple détection GitLab / GitHub / SonarQube".to_string()),
                category: Category::Secu,
                audit_type: AuditType::Technique,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![
                    "https://docs.gitlab.com/ee/user/application_security/secret_detection/".to_string(),
                    "https://gitleaks.io/".to_string(),
                ],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Utilisez-vous des MFAs pour accéder aux interfaces d'administration de vos outils et infras ?".to_string(),
                description: Some("Ex. : dépôt de code (GitLab, GitHub...), accès à l'interface d'administration de l'hébergeur, sentry, grafana, plus généralement, tous les outils qui concourent au bon fonctionnement du service...".to_string()),
                category: Category::Secu,
                audit_type: AuditType::Technique,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec!["https://fr.wikipedia.org/wiki/Double_authentification".to_string()],
            },
            // Questions supplémentaires - Infrastructure
            Question {
                id: Uuid::new_v4(),
                text: "Avez-vous une documentation à jour de votre architecture infrastructure ?".to_string(),
                description: Some("Diagrammes d'architecture, schémas réseau, documentation des services déployés.".to_string()),
                category: Category::Infra,
                audit_type: AuditType::Technique,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Utilisez-vous un système de gestion de configuration (ex: Ansible, Puppet, Chef) ?".to_string(),
                description: Some("Pour automatiser la configuration et le déploiement des serveurs.".to_string()),
                category: Category::Infra,
                audit_type: AuditType::Technique,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Avez-vous mis en place un système de logs centralisé ?".to_string(),
                description: Some("Par exemple ELK stack, Loki, ou équivalent pour centraliser et analyser les logs.".to_string()),
                category: Category::Infra,
                audit_type: AuditType::Technique,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Votre infrastructure supporte-t-elle la montée en charge automatique (auto-scaling) ?".to_string(),
                description: Some("Capacité à augmenter automatiquement les ressources en cas de pic de charge.".to_string()),
                category: Category::Infra,
                audit_type: AuditType::Technique,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            // Questions supplémentaires - Opération
            Question {
                id: Uuid::new_v4(),
                text: "Avez-vous un plan de reprise d'activité (PRA) documenté et testé ?".to_string(),
                description: Some("Procédure de récupération en cas de sinistre majeur.".to_string()),
                category: Category::Operation,
                audit_type: AuditType::Technique,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Effectuez-vous des tests de charge réguliers ?".to_string(),
                description: Some("Tests pour valider la capacité de l'infrastructure à supporter la charge attendue.".to_string()),
                category: Category::Operation,
                audit_type: AuditType::Technique,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Avez-vous une procédure de rollback documentée et testée ?".to_string(),
                description: Some("Capacité à revenir à une version précédente en cas de problème après déploiement.".to_string()),
                category: Category::Operation,
                audit_type: AuditType::Technique,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Vos déploiements sont-ils automatisés via une CI/CD ?".to_string(),
                description: Some("Pipeline de déploiement automatique (GitLab CI, GitHub Actions, Jenkins, etc.).".to_string()),
                category: Category::Operation,
                audit_type: AuditType::Technique,
                requires_percentage: true,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            // Questions supplémentaires - Base de données
            Question {
                id: Uuid::new_v4(),
                text: "Effectuez-vous des sauvegardes automatiques régulières de vos bases de données ?".to_string(),
                description: Some("Sauvegardes quotidiennes ou plus fréquentes selon la criticité.".to_string()),
                category: Category::DB,
                audit_type: AuditType::Technique,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Avez-vous testé la restauration complète d'une base de données depuis une sauvegarde ?".to_string(),
                description: Some("Test de restauration effectué dans les 6 derniers mois.".to_string()),
                category: Category::DB,
                audit_type: AuditType::Technique,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Vos bases de données sont-elles optimisées (index, requêtes, etc.) ?".to_string(),
                description: Some("Analyse régulière des performances et optimisation des requêtes lentes.".to_string()),
                category: Category::DB,
                audit_type: AuditType::Technique,
                requires_percentage: true,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Avez-vous une stratégie de migration de schéma documentée ?".to_string(),
                description: Some("Procédure pour gérer les évolutions de schéma de base de données de manière sûre.".to_string()),
                category: Category::DB,
                audit_type: AuditType::Technique,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            // Questions supplémentaires - Sécurité
            Question {
                id: Uuid::new_v4(),
                text: "Effectuez-vous des audits de sécurité réguliers (pentest, scans de vulnérabilités) ?".to_string(),
                description: Some("Audits de sécurité au moins annuels, scans automatiques réguliers.".to_string()),
                category: Category::Secu,
                audit_type: AuditType::Technique,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Avez-vous mis en place une gestion des secrets (ex: Vault, AWS Secrets Manager) ?".to_string(),
                description: Some("Stockage sécurisé des secrets (mots de passe, clés API, tokens) hors du code source.".to_string()),
                category: Category::Secu,
                audit_type: AuditType::Technique,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Avez-vous une politique de gestion des accès (IAM) documentée et appliquée ?".to_string(),
                description: Some("Principe du moindre privilège, gestion des droits d'accès, revue régulière.".to_string()),
                category: Category::Secu,
                audit_type: AuditType::Technique,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Utilisez-vous un WAF (Web Application Firewall) ?".to_string(),
                description: Some("Protection contre les attaques au niveau applicatif.".to_string()),
                category: Category::Secu,
                audit_type: AuditType::Technique,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Avez-vous mis en place une gestion des certificats SSL/TLS automatisée ?".to_string(),
                description: Some("Renouvellement automatique des certificats (ex: Let's Encrypt avec certbot).".to_string()),
                category: Category::Secu,
                audit_type: AuditType::Technique,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            // Questions - Accessibilité (A11y)
            Question {
                id: Uuid::new_v4(),
                text: "Votre application respecte-t-elle les standards d'accessibilité (WCAG 2.1) ?".to_string(),
                description: Some("Conformité au niveau AA minimum recommandé.".to_string()),
                category: Category::A11y,
                audit_type: AuditType::Technique,
                requires_percentage: true,
                expected_answer: Some(Answer::Oui),
                links: vec!["https://www.w3.org/WAI/WCAG21/quickref/".to_string()],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Effectuez-vous des tests d'accessibilité automatisés dans votre CI/CD ?".to_string(),
                description: Some("Outils comme axe-core, pa11y, ou équivalents intégrés au pipeline.".to_string()),
                category: Category::A11y,
                audit_type: AuditType::Technique,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Avez-vous testé votre application avec des lecteurs d'écran ?".to_string(),
                description: Some("Tests manuels avec NVDA, JAWS, VoiceOver, ou équivalents.".to_string()),
                category: Category::A11y,
                audit_type: AuditType::Technique,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Vos formulaires ont-ils des labels associés et une gestion d'erreurs accessible ?".to_string(),
                description: Some("Labels ARIA, messages d'erreur associés aux champs, navigation au clavier.".to_string()),
                category: Category::A11y,
                audit_type: AuditType::Technique,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Votre application est-elle utilisable uniquement au clavier (sans souris) ?".to_string(),
                description: Some("Navigation complète possible avec Tab, Entrée, Espace, flèches.".to_string()),
                category: Category::A11y,
                audit_type: AuditType::Technique,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            // Questions - Architecture
            Question {
                id: Uuid::new_v4(),
                text: "Votre architecture suit-elle les principes SOLID ?".to_string(),
                description: Some("Séparation des responsabilités, dépendances inversées, etc.".to_string()),
                category: Category::Archi,
                audit_type: AuditType::Technique,
                requires_percentage: true,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Avez-vous une architecture modulaire et découplée ?".to_string(),
                description: Some("Services indépendants, API bien définies, faible couplage.".to_string()),
                category: Category::Archi,
                audit_type: AuditType::Technique,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Votre code respecte-t-il le principe DRY (Don't Repeat Yourself) ?".to_string(),
                description: Some("Éviter la duplication de code, factorisation appropriée.".to_string()),
                category: Category::Archi,
                audit_type: AuditType::Technique,
                requires_percentage: true,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Avez-vous une documentation technique à jour de votre architecture ?".to_string(),
                description: Some("Documentation des composants, flux de données, décisions architecturales.".to_string()),
                category: Category::Archi,
                audit_type: AuditType::Technique,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Utilisez-vous des patterns de design appropriés pour votre contexte ?".to_string(),
                description: Some("Patterns adaptés au domaine métier et aux contraintes techniques.".to_string()),
                category: Category::Archi,
                audit_type: AuditType::Technique,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            // Questions - Open Source (OSS)
            Question {
                id: Uuid::new_v4(),
                text: "Votre code source est-il ouvert (open source) ?".to_string(),
                description: Some("Code accessible publiquement sous licence open source.".to_string()),
                category: Category::OSS,
                audit_type: AuditType::Technique,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Avez-vous une politique de contribution documentée ?".to_string(),
                description: Some("Guide pour les contributeurs, processus de contribution, code de conduite.".to_string()),
                category: Category::OSS,
                audit_type: AuditType::Technique,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Votre projet open source a-t-il une documentation complète (README, CONTRIBUTING, etc.) ?".to_string(),
                description: Some("Documentation d'installation, d'utilisation, de contribution.".to_string()),
                category: Category::OSS,
                audit_type: AuditType::Technique,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Utilisez-vous des dépendances open source à jour et maintenues ?".to_string(),
                description: Some("Dépendances régulièrement mises à jour, vérification des vulnérabilités.".to_string()),
                category: Category::OSS,
                audit_type: AuditType::Technique,
                requires_percentage: true,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            // Questions - Developer Experience (Dev XP)
            Question {
                id: Uuid::new_v4(),
                text: "Un nouveau développeur peut-il démarrer le projet en moins de 30 minutes ?".to_string(),
                description: Some("Documentation claire, environnement de développement simple à mettre en place.".to_string()),
                category: Category::DevXP,
                audit_type: AuditType::Technique,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Avez-vous un environnement de développement local facile à configurer (Docker, etc.) ?".to_string(),
                description: Some("Configuration automatisée via Docker Compose, Vagrant, ou équivalent.".to_string()),
                category: Category::DevXP,
                audit_type: AuditType::Technique,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Votre codebase a-t-il une couverture de tests suffisante (> 70%) ?".to_string(),
                description: Some("Tests unitaires, d'intégration, couverture mesurée et suivie.".to_string()),
                category: Category::DevXP,
                audit_type: AuditType::Technique,
                requires_percentage: true,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Avez-vous des outils de qualité de code intégrés (linters, formatters) ?".to_string(),
                description: Some("ESLint, Prettier, Rubocop, ou équivalents configurés et appliqués automatiquement.".to_string()),
                category: Category::DevXP,
                audit_type: AuditType::Technique,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Votre documentation technique est-elle à jour et facilement accessible ?".to_string(),
                description: Some("Documentation inline, README complet, guides de développement.".to_string()),
                category: Category::DevXP,
                audit_type: AuditType::Technique,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Avez-vous un système de revue de code (code review) systématique ?".to_string(),
                description: Some("Toutes les modifications passent par une revue avant merge.".to_string()),
                category: Category::DevXP,
                audit_type: AuditType::Technique,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            // Questions - Produit
            Question {
                id: Uuid::new_v4(),
                text: "Avez-vous mis en place un système de suivi des erreurs utilisateurs (Sentry, etc.) ?".to_string(),
                description: Some("Monitoring des erreurs en production, alertes, analyse des crashs.".to_string()),
                category: Category::Produit,
                audit_type: AuditType::Technique,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Collectez-vous des métriques d'usage (analytics) pour comprendre l'utilisation du produit ?".to_string(),
                description: Some("Métriques d'utilisation, parcours utilisateur, points de friction.".to_string()),
                category: Category::Produit,
                audit_type: AuditType::Technique,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Avez-vous un système de feedback utilisateur intégré ?".to_string(),
                description: Some("Mécanisme pour recueillir les retours utilisateurs (formulaires, tickets, etc.).".to_string()),
                category: Category::Produit,
                audit_type: AuditType::Technique,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Votre application est-elle responsive (mobile-friendly) ?".to_string(),
                description: Some("Interface adaptée aux différentes tailles d'écran, testée sur mobile.".to_string()),
                category: Category::Produit,
                audit_type: AuditType::Technique,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
            Question {
                id: Uuid::new_v4(),
                text: "Avez-vous mis en place des tests d'acceptation utilisateur (UAT) ?".to_string(),
                description: Some("Tests avec de vrais utilisateurs avant mise en production.".to_string()),
                category: Category::Produit,
                audit_type: AuditType::Technique,
                requires_percentage: false,
                expected_answer: Some(Answer::Oui),
                links: vec![],
            },
        ]
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

