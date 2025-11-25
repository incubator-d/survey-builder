//! État de l'application et logique UI
//! 
//! Contient `AppState` qui gère l'état de l'application, la navigation,
//! les interactions utilisateur et le rendu des vues.

use ratzilla::ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Row, Scrollbar, ScrollbarState, Table, Wrap},
    Frame,
};

use crate::domain::{AuditManager, QuestionGenerator, RecommendationEngine, *};
use crate::shared::{AUDIT_TYPES, answer_inputs, score_thresholds, scroll, calculate_global_score};
use serde::{Deserialize, Serialize};

#[cfg(target_arch = "wasm32")]
use web_sys::{window, HtmlAnchorElement, Url, HtmlInputElement};
#[cfg(target_arch = "wasm32")]
use js_sys;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

/// État principal de l'application
/// 
/// Gère toutes les données de l'application, la navigation entre les vues,
/// et les interactions utilisateur.
pub struct AppState {
    pub view: View,
    pub audit_manager: AuditManager,
    pub audits: Vec<Audit>,
    pub selected_audit_index: usize,
    pub current_question_index: usize,
    pub new_audit_name: String,
    pub new_audit_type_index: usize, // Index du type sélectionné
    pub new_project_name: String,
    pub new_product_name: String,
    pub answer_value: String,
    pub answer_percentage: String,
    pub answer_comment: String,
    pub input_mode: InputMode,
    pub summary_scroll: usize, // Index de scroll pour le résumé des catégories
    pub recommendations_scroll: usize, // Index de scroll pour les recommandations
    pub error_message: Option<String>, // Message d'erreur à afficher à l'utilisateur
    pub pending_delete_audit_id: Option<uuid::Uuid>, // ID de l'audit en attente de suppression
    pub import_file_input_id: Option<String>, // ID de l'input file pour l'import
    pub filter_unanswered: bool, // Filtre pour n'afficher que les questions sans réponse
}

/// Vues disponibles dans l'application
#[derive(Default, PartialEq, Debug)]
pub enum View {
    #[default]
    List,
    Detail,
    Create,
    Recommendations,
}

/// Modes de saisie pour les formulaires
#[derive(Default, PartialEq)]
pub enum InputMode {
    #[default]
    Normal,
    Name,
    Type,
    Project,
    Product,
    Comment,
}

impl AppState {
    pub fn load_audits(&mut self) {
        match self.audit_manager.list_audits() {
            Ok(audit_ids) => {
                self.audits.clear();
                let mut errors = Vec::new();
                for id in audit_ids {
                    match self.audit_manager.load_audit(&id) {
                        Ok(audit) => self.audits.push(audit),
                        Err(e) => {
                            errors.push(format!("Impossible de charger l'audit {}: {}", id, e));
                        }
                    }
                }
                // Afficher les erreurs si nécessaire
                if !errors.is_empty() {
                    self.error_message = Some(format!("{} erreur(s) lors du chargement", errors.len()));
                }
                // Trier les audits par complétion (du moins au plus)
                self.audits.sort_by(|a, b| {
                    let completion_a = a.completion_percentage();
                    let completion_b = b.completion_percentage();
                    completion_a.partial_cmp(&completion_b).unwrap_or(std::cmp::Ordering::Equal)
                });
                // Réinitialiser l'index de sélection si nécessaire
                self.validate_selected_index();
            }
            Err(e) => {
                self.error_message = Some(format!("Erreur lors du chargement des audits: {}", e));
            }
        }
    }

    /// Valide et corrige l'index de sélection si nécessaire
    pub fn validate_selected_index(&mut self) {
        if self.selected_audit_index >= self.audits.len() && !self.audits.is_empty() {
            self.selected_audit_index = 0;
        } else if self.audits.is_empty() {
            self.selected_audit_index = 0;
        }
    }

    pub fn handle_enter(&mut self) {
        match self.view {
            View::List => {
                if !self.audits.is_empty() && self.selected_audit_index < self.audits.len() {
                    self.view = View::Detail;
                    self.current_question_index = 0;
                }
            }
            View::Create => {
                if self.input_mode == InputMode::Normal {
                    self.input_mode = InputMode::Name;
                } else {
                    // Entrée pour passer au champ suivant ou créer l'audit
                    match self.input_mode {
                        InputMode::Name => {
                            if !self.new_audit_name.is_empty() {
                                self.input_mode = InputMode::Type;
                            }
                        }
                        InputMode::Type => {
                            self.input_mode = InputMode::Project;
                        }
                        InputMode::Project => {
                            self.input_mode = InputMode::Product;
                        }
                        InputMode::Product => {
                            if !self.new_audit_name.is_empty() {
                                self.create_audit();
                            }
                        }
                        _ => {}
                    }
                }
            }
            View::Detail => {
                if self.input_mode == InputMode::Comment {
                    // En mode commentaire, Entrée valide le commentaire et sort du mode
                    self.input_mode = InputMode::Normal;
                } else {
                    // Entrée passe à la question suivante
                    // Les réponses sont sauvegardées automatiquement quand on tape o/n/j/a
                    let visible_indices = self.get_visible_question_indices();
                    if self.current_question_index < visible_indices.len().saturating_sub(1) {
                        self.current_question_index += 1;
                    } else if self.filter_unanswered {
                        // Si on est en mode filtre et qu'on a répondu à toutes les questions visibles,
                        // on peut désactiver le filtre ou rester sur la dernière
                        // Pour l'instant, on reste sur la dernière question
                    }
                }
            }
            _ => {}
        }
    }

    pub fn handle_up(&mut self) {
        match self.view {
            View::List => {
                if self.selected_audit_index > 0 {
                    self.selected_audit_index -= 1;
                }
            }
            View::Detail => {
                let visible_indices = self.get_visible_question_indices();
                if self.current_question_index > 0 && !visible_indices.is_empty() {
                    self.current_question_index -= 1;
                } else if !visible_indices.is_empty() {
                    // Si on est à la première question visible, on reste dessus
                    self.current_question_index = 0;
                }
            }
            View::Recommendations => {
                // Scroller vers le haut
                if self.recommendations_scroll >= scroll::RECOMMENDATIONS_SCROLL_STEP {
                    self.recommendations_scroll -= scroll::RECOMMENDATIONS_SCROLL_STEP;
                } else {
                    self.recommendations_scroll = 0;
                }
            }
            View::Create => {
                if self.input_mode == InputMode::Type {
                    if self.new_audit_type_index > 0 {
                        self.new_audit_type_index -= 1;
                    } else {
                        self.new_audit_type_index = AUDIT_TYPES.len() - 1;
                    }
                }
            }
        }
    }

    pub fn get_current_audit_for_recommendations(&self) -> Option<&Audit> {
        // Même logique que render_recommendations
        if self.view == View::Detail {
            self.get_current_audit()
        } else if !self.audits.is_empty() && self.selected_audit_index < self.audits.len() {
            Some(&self.audits[self.selected_audit_index])
        } else {
            None
        }
    }

    /// Obtient les indices des questions non répondues pour l'audit actuel
    fn get_unanswered_question_indices(&self) -> Vec<usize> {
        if let Some(audit) = self.get_current_audit() {
            audit.questions
                .iter()
                .enumerate()
                .filter_map(|(idx, question)| {
                    if audit.get_answer(&question.id).is_none() {
                        Some(idx)
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Obtient la liste des indices de questions à afficher (filtrée ou non)
    fn get_visible_question_indices(&self) -> Vec<usize> {
        if let Some(audit) = self.get_current_audit() {
            if self.filter_unanswered {
                self.get_unanswered_question_indices()
            } else {
                (0..audit.questions.len()).collect()
            }
        } else {
            Vec::new()
        }
    }

    /// Obtient l'index réel de la question actuelle dans la liste complète
    fn get_real_question_index(&self) -> Option<usize> {
        let visible_indices = self.get_visible_question_indices();
        if self.current_question_index < visible_indices.len() {
            Some(visible_indices[self.current_question_index])
        } else {
            None
        }
    }

    /// Active ou désactive le filtre des questions non répondues
    pub fn toggle_filter_unanswered(&mut self) {
        if self.get_current_audit().is_some() {
            let old_filter = self.filter_unanswered;
            self.filter_unanswered = !self.filter_unanswered;
            
            // Si on active le filtre, on doit ajuster l'index pour pointer vers la première question non répondue
            if self.filter_unanswered && !old_filter {
                let unanswered_indices = self.get_unanswered_question_indices();
                if !unanswered_indices.is_empty() {
                    // Trouver l'index de la question actuelle dans la liste filtrée
                    if let Some(current_real_idx) = self.get_real_question_index() {
                        if let Some(pos) = unanswered_indices.iter().position(|&idx| idx == current_real_idx) {
                            self.current_question_index = pos;
                        } else {
                            // Si la question actuelle est déjà répondue, aller à la première non répondue
                            self.current_question_index = 0;
                        }
                    } else {
                        self.current_question_index = 0;
                    }
                } else {
                    // Toutes les questions sont répondues
                    self.current_question_index = 0;
                }
            } else if !self.filter_unanswered && old_filter {
                // Si on désactive le filtre, on doit ajuster l'index pour pointer vers la même question
                if let Some(real_idx) = self.get_real_question_index() {
                    self.current_question_index = real_idx;
                }
            }
        }
    }

    pub fn handle_down(&mut self) {
        match self.view {
            View::List => {
                if self.selected_audit_index < self.audits.len().saturating_sub(1) {
                    self.selected_audit_index += 1;
                }
            }
            View::Detail => {
                let visible_indices = self.get_visible_question_indices();
                if !visible_indices.is_empty() {
                    if self.current_question_index < visible_indices.len().saturating_sub(1) {
                        self.current_question_index += 1;
                    } else {
                        // On est à la dernière question visible, on reste dessus
                        self.current_question_index = visible_indices.len().saturating_sub(1);
                    }
                }
            }
            View::Recommendations => {
                // Scroller vers le bas
                self.recommendations_scroll += scroll::RECOMMENDATIONS_SCROLL_STEP;
            }
            View::Create => {
                if self.input_mode == InputMode::Type {
                    if self.new_audit_type_index < AUDIT_TYPES.len() - 1 {
                        self.new_audit_type_index += 1;
                    } else {
                        self.new_audit_type_index = 0;
                    }
                }
            }
        }
    }

    pub fn handle_delete(&mut self) {
        if self.view == View::List && !self.audits.is_empty() {
            let index = self.selected_audit_index;
            if index < self.audits.len() {
                // Si une suppression est déjà en attente, on annule
                if self.pending_delete_audit_id.is_some() {
                    self.cancel_delete();
                    return;
                }
                // Sinon, on demande confirmation
                let audit_id = self.audits[index].id;
                self.pending_delete_audit_id = Some(audit_id);
            }
        }
    }

    pub fn confirm_delete(&mut self) {
        if let Some(audit_id) = self.pending_delete_audit_id.take() {
            match self.audit_manager.delete_audit(&audit_id) {
                Ok(()) => {
                    self.load_audits();
                    // Ajuster l'index après suppression
                    if !self.audits.is_empty() {
                        if self.selected_audit_index >= self.audits.len() {
                            self.selected_audit_index = self.audits.len() - 1;
                        }
                    } else {
                        self.selected_audit_index = 0;
                    }
                }
                Err(e) => {
                    self.error_message = Some(format!("Erreur lors de la suppression: {}", e));
                }
            }
        }
    }

    pub fn cancel_delete(&mut self) {
        self.pending_delete_audit_id = None;
    }

    /// Exporte l'audit actuel ou sélectionné au format TXT
    pub fn export_audit_to_txt(&mut self) {
        let audit = self.get_current_audit_for_recommendations();
        if let Some(audit) = audit {
            let text = self.generate_audit_text(audit);
            self.download_text_file(&text, &format!("audit_{}_{}.txt", 
                audit.name.replace(" ", "_"), 
                audit.id.to_string().chars().take(8).collect::<String>()));
        } else {
            self.error_message = Some("Aucun audit sélectionné pour l'export".to_string());
        }
    }

    /// Génère le texte formaté de l'audit complet
    fn generate_audit_text(&self, audit: &Audit) -> String {
        let mut text = String::new();
        
        // En-tête
        text.push_str("═══════════════════════════════════════════════════════════════\n");
        text.push_str(&format!("RAPPORT D'AUDIT: {}\n", audit.name));
        text.push_str("═══════════════════════════════════════════════════════════════\n\n");
        
        // Informations générales
        text.push_str("INFORMATIONS GÉNÉRALES\n");
        text.push_str("───────────────────────────────────────────────────────────────\n");
        text.push_str(&format!("Type d'audit: {}\n", audit.audit_type));
        if let Some(ref project) = audit.project_name {
            text.push_str(&format!("Projet: {}\n", project));
        }
        if let Some(ref product) = audit.product_name {
            text.push_str(&format!("Produit: {}\n", product));
        }
        text.push_str(&format!("Date de création: {}\n", audit.created_at.format("%d/%m/%Y %H:%M:%S UTC")));
        text.push_str(&format!("Dernière mise à jour: {}\n", audit.updated_at.format("%d/%m/%Y %H:%M:%S UTC")));
        text.push_str(&format!("Complétion: {:.1}%\n", audit.completion_percentage()));
        
        // Score global
        if let Some(score) = calculate_global_score(audit) {
            text.push_str(&format!("Score global: {:.1}%\n", score));
        } else {
            text.push_str("Score global: N/A (aucune réponse)\n");
        }
        text.push_str("\n");
        
        // Résumé par catégorie
        let summary = RecommendationEngine::generate_summary(audit);
        text.push_str("RÉSUMÉ PAR CATÉGORIE\n");
        text.push_str("───────────────────────────────────────────────────────────────\n");
        
        // Trier les catégories par pourcentage positif (décroissant)
        let mut sorted_stats: Vec<_> = summary.category_stats.iter().collect();
        sorted_stats.sort_by(|a, b| {
            b.positive_percentage().partial_cmp(&a.positive_percentage())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        
        for stats in &sorted_stats {
            text.push_str(&format!("\nCatégorie: {}\n", stats.category));
            text.push_str(&format!("  Questions totales: {}\n", stats.total_questions));
            text.push_str(&format!("  Questions répondues: {}\n", stats.answered));
            text.push_str(&format!("  Réponses positives: {}\n", stats.positive_answers));
            text.push_str(&format!("  Réponses négatives: {}\n", stats.negative_answers));
            text.push_str(&format!("  Pourcentage positif: {:.1}%\n", stats.positive_percentage()));
            text.push_str(&format!("  Complétion: {:.1}%\n", stats.completion_percentage()));
        }
        text.push_str("\n");
        
        // Questions et réponses
        text.push_str("QUESTIONS ET RÉPONSES\n");
        text.push_str("───────────────────────────────────────────────────────────────\n");
        
        // Grouper les questions par catégorie
        let mut questions_by_category: std::collections::HashMap<Category, Vec<&Question>> = std::collections::HashMap::new();
        for question in &audit.questions {
            questions_by_category
                .entry(question.category.clone())
                .or_insert_with(Vec::new)
                .push(question);
        }
        
        // Trier les catégories
        let mut sorted_categories: Vec<_> = questions_by_category.keys().collect();
        sorted_categories.sort_by(|a, b| format!("{}", a).cmp(&format!("{}", b)));
        
        for category in sorted_categories {
            let questions = &questions_by_category[category];
            text.push_str(&format!("\n┌─ Catégorie: {} ─────────────────────────────────────────────────────┐\n", category));
            
            for (idx, question) in questions.iter().enumerate() {
                text.push_str(&format!("\nQuestion {}:\n", idx + 1));
                text.push_str(&format!("  {}\n", question.text));
                
                if let Some(ref desc) = question.description {
                    text.push_str(&format!("  Description: {}\n", desc));
                }
                
                if let Some(answer) = audit.get_answer(&question.id) {
                    text.push_str(&format!("  Réponse: {}\n", answer.answer));
                    if let Some(percentage) = answer.percentage {
                        text.push_str(&format!("  Pourcentage: {}%\n", percentage));
                    }
                    if let Some(ref comment) = answer.comment {
                        text.push_str(&format!("  Commentaire: {}\n", comment));
                    }
                    text.push_str(&format!("  Répondu le: {}\n", answer.answered_at.format("%d/%m/%Y %H:%M:%S UTC")));
                } else {
                    text.push_str("  Réponse: Non répondue\n");
                }
                
                if !question.links.is_empty() {
                    text.push_str("  Liens utiles:\n");
                    for link in &question.links {
                        text.push_str(&format!("    - {}\n", link));
                    }
                }
            }
            text.push_str("└────────────────────────────────────────────────────────────────────┘\n");
        }
        text.push_str("\n");
        
        // Recommandations
        let recommendations = RecommendationEngine::generate_recommendations(audit);
        if !recommendations.is_empty() {
            text.push_str("RECOMMANDATIONS\n");
            text.push_str("───────────────────────────────────────────────────────────────\n");
            
            // Trier par priorité (du plus élevé au plus faible)
            let mut sorted_recs = recommendations.clone();
            sorted_recs.sort_by(|a, b| b.priority.cmp(&a.priority));
            
            for (idx, rec) in sorted_recs.iter().enumerate() {
                text.push_str(&format!("\n{}. [{}] {}\n", idx + 1, rec.priority, rec.title));
                text.push_str(&format!("   Catégorie: {}\n", rec.category));
                text.push_str(&format!("   {}\n", rec.description));
            }
        } else {
            text.push_str("RECOMMANDATIONS\n");
            text.push_str("───────────────────────────────────────────────────────────────\n");
            text.push_str("✓ Aucune recommandation - Toutes les pratiques sont bonnes !\n");
        }
        
        text.push_str("\n═══════════════════════════════════════════════════════════════\n");
        text.push_str(&format!("Rapport généré le: {}\n", chrono::Utc::now().format("%d/%m/%Y %H:%M:%S UTC")));
        text.push_str("═══════════════════════════════════════════════════════════════\n");
        
        text
    }

    /// Exporte un audit vierge au format YAML
    pub fn export_audit_to_yaml(&mut self) {
        let audit = self.get_current_audit_for_recommendations();
        if let Some(audit) = audit {
            // Vérifier que l'audit est vierge (pas de réponses)
            if !audit.answers.is_empty() {
                self.error_message = Some("L'audit contient déjà des réponses. Exportez en TXT pour voir les résultats.\n".to_string());
                return;
            }
            
            let yaml = self.generate_audit_yaml(audit);
            self.download_text_file(&yaml, &format!("audit_{}_vierge.yaml", 
                audit.name.replace(" ", "_")));
        } else {
            self.error_message = Some("Aucun audit sélectionné pour l'export YAML".to_string());
        }
    }

    /// Génère le YAML d'un audit vierge prêt à être rempli
    fn generate_audit_yaml(&self, audit: &Audit) -> String {
        // Structures pour la sérialisation YAML
        #[derive(Serialize)]
        struct QuestionYamlExport {
            id: uuid::Uuid,
            text: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            description: Option<String>,
            category: String,
            requires_percentage: bool,
            #[serde(skip_serializing_if = "Option::is_none")]
            expected_answer: Option<String>,
            #[serde(skip_serializing_if = "Vec::is_empty")]
            links: Vec<String>,
            // Champs à remplir
            answer: Option<String>, // null par défaut, à remplir
            #[serde(skip_serializing_if = "Option::is_none")]
            percentage: Option<u8>,
            #[serde(skip_serializing_if = "Option::is_none")]
            comment: Option<String>,
        }
        
        #[derive(Serialize)]
        struct AuditYamlExport {
            audit: AuditInfoYamlExport,
            questions: Vec<QuestionYamlExport>,
        }
        
        #[derive(Serialize)]
        struct AuditInfoYamlExport {
            name: String,
            audit_type: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            project_name: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            product_name: Option<String>,
        }
        
        // Construire les questions
        let questions: Vec<QuestionYamlExport> = audit.questions.iter().map(|q| {
            QuestionYamlExport {
                id: q.id,
                text: q.text.clone(),
                description: q.description.clone(),
                category: format!("{}", q.category),
                requires_percentage: q.requires_percentage,
                expected_answer: q.expected_answer.as_ref().map(|a| format!("{}", a)),
                links: q.links.clone(),
                answer: None, // null par défaut, à remplir
                percentage: None,
                comment: None,
            }
        }).collect();
        
        let audit_info = AuditInfoYamlExport {
            name: audit.name.clone(),
            audit_type: format!("{}", audit.audit_type),
            project_name: audit.project_name.clone(),
            product_name: audit.product_name.clone(),
        };
        
        let yaml_data = AuditYamlExport {
            audit: audit_info,
            questions,
        };
        
        // Générer le YAML avec serde_yaml
        let mut yaml = String::new();
        yaml.push_str("# Audit vierge - Prêt à être rempli\n");
        yaml.push_str("# Format: remplissez les champs 'answer' pour chaque question\n");
        yaml.push_str("# Réponses possibles: Oui, Non, JeNeSaisPas, NonApplicable\n");
        yaml.push_str("# Pour les questions nécessitant un pourcentage, remplissez aussi 'percentage' (0-100)\n");
        yaml.push_str("# Vous pouvez aussi ajouter un 'comment' optionnel pour chaque réponse\n\n");
        
        // Utiliser serde_yaml pour générer le YAML proprement
        // serde_yaml devrait automatiquement échapper les chaînes avec des caractères spéciaux
        match serde_yaml::to_string(&yaml_data) {
            Ok(serde_yaml_str) => {
                // Post-traiter pour s'assurer que les chaînes avec des deux-points sont échappées
                let processed = self.fix_yaml_strings(&serde_yaml_str);
                yaml.push_str(&processed);
            }
            Err(e) => {
                // En cas d'erreur, utiliser un message d'erreur
                yaml.push_str(&format!("# Erreur lors de la génération YAML: {}\n", e));
            }
        }
        
        yaml
    }

    /// Pré-traite le YAML pour corriger les chaînes non échappées avant le parsing
    fn fix_yaml_for_parsing(&self, yaml: &str) -> String {
        let mut result = String::new();
        let lines: Vec<&str> = yaml.lines().collect();
        
        for line in lines.iter() {
            let trimmed = line.trim_start();
            let indent = line.len() - trimmed.len();
            
            // Chercher les lignes avec des clés de type texte qui ont des valeurs non échappées
            if trimmed.starts_with("text:") || trimmed.starts_with("description:") || 
               trimmed.starts_with("name:") || trimmed.starts_with("project_name:") || 
               trimmed.starts_with("product_name:") || trimmed.starts_with("expected_answer:") ||
               trimmed.starts_with("comment:") || trimmed.starts_with("answer:") {
                
                if let Some(colon_pos) = trimmed.find(':') {
                    let key_part = &trimmed[..colon_pos + 1];
                    let rest = trimmed[colon_pos + 1..].trim();
                    
                    // Si la valeur contient des deux-points et n'est pas déjà entre guillemets
                    if rest.contains(':') && !rest.starts_with('"') && !rest.starts_with('\'') && rest != "null" {
                        // Séparer la valeur du commentaire
                        let (value, comment) = if let Some(comment_pos) = rest.find(" #") {
                            (rest[..comment_pos].trim(), &rest[comment_pos..])
                        } else {
                            (rest, "")
                        };
                        
                        // Si la valeur contient des apostrophes, utiliser des guillemets doubles
                        let quoted_value = if value.contains('\'') {
                            format!("\"{}\"", value.replace('"', "\\\""))
                        } else {
                            format!("'{}'", value.replace('\'', "''"))
                        };
                        
                        result.push_str(&format!("{}{} {}{}\n", 
                            " ".repeat(indent), key_part, quoted_value, comment));
                    } else {
                        result.push_str(line);
                        result.push('\n');
                    }
                } else {
                    result.push_str(line);
                    result.push('\n');
                }
            } else {
                result.push_str(line);
                result.push('\n');
            }
        }
        
        result
    }

    /// Corrige les chaînes YAML qui contiennent des deux-points non échappés lors de la génération
    fn fix_yaml_strings(&self, yaml: &str) -> String {
        let mut result = String::new();
        let mut lines = yaml.lines().peekable();
        
        while let Some(line) = lines.next() {
            // Chercher les lignes avec des clés de type texte qui ont des valeurs non échappées
            let trimmed = line.trim_start();
            if trimmed.starts_with("text:") || trimmed.starts_with("description:") || 
               trimmed.starts_with("name:") || trimmed.starts_with("project_name:") || 
               trimmed.starts_with("product_name:") || trimmed.starts_with("expected_answer:") ||
               trimmed.starts_with("comment:") || trimmed.starts_with("answer:") {
                
                // Séparer la clé et la valeur
                if let Some(colon_pos) = trimmed.find(':') {
                    let key_part = &trimmed[..colon_pos + 1];
                    let value_part = trimmed[colon_pos + 1..].trim();
                    
                    // Si la valeur contient des deux-points et n'est pas déjà entre guillemets
                    if value_part.contains(':') && !value_part.starts_with('"') && !value_part.starts_with('\'') && value_part != "null" {
                        // Extraire le commentaire s'il y en a un
                        let (value, comment) = if let Some(comment_pos) = value_part.find(" #") {
                            (&value_part[..comment_pos], &value_part[comment_pos..])
                        } else {
                            (value_part, "")
                        };
                        
                        // Échapper les apostrophes simples dans la valeur
                        let escaped_value = value.replace('\'', "''");
                        let indent = line.len() - trimmed.len();
                        result.push_str(&format!("{}{} '{}'{}\n", 
                            " ".repeat(indent), key_part, escaped_value, comment));
                    } else {
                        result.push_str(line);
                        result.push('\n');
                    }
                } else {
                    result.push_str(line);
                    result.push('\n');
                }
            } else {
                result.push_str(line);
                result.push('\n');
            }
        }
        
        result
    }

    /// Importe un audit depuis un fichier YAML
    /// Note: Cette fonction crée un input file et le déclenche
    /// Le traitement du fichier doit être fait via JavaScript qui appelle process_yaml_import
    #[cfg(target_arch = "wasm32")]
    pub fn import_audit_from_yaml(&mut self) {
        if let Some(window) = window() {
            if let Some(document) = window.document() {
                // Créer un input file caché
                if let Ok(input) = document.create_element("input") {
                    if let Some(input_elem) = input.dyn_ref::<HtmlInputElement>() {
                        input_elem.set_type("file");
                        input_elem.set_accept(".yaml,.yml");
                        input_elem.style().set_property("display", "none").ok();
                        
                        // Générer un ID unique pour cet input
                        let input_id = format!("yaml_import_{}", uuid::Uuid::new_v4());
                        input_elem.set_id(&input_id);
                        self.import_file_input_id = Some(input_id.clone());
                        
                        // Ajouter au document et déclencher le clic
                        if let Some(body) = document.body() {
                            if body.append_child(&input).is_ok() {
                                input_elem.click();
                                // Le fichier sera traité via JavaScript qui lira le contenu
                                // et appellera process_yaml_import via wasm-bindgen
                            }
                        }
                    }
                }
            }
        }
    }

    /// Version pour les tests (non-wasm32) - ne fait rien
    #[cfg(not(target_arch = "wasm32"))]
    pub fn import_audit_from_yaml(&mut self) {
        // En mode test, on ne peut pas importer depuis un fichier
        self.error_message = Some("Import YAML non disponible en mode test".to_string());
    }

    /// Traite le contenu YAML importé
    pub fn process_yaml_import(&mut self, yaml_content: &str) {
        match self.parse_and_create_audit_from_yaml(yaml_content) {
            Ok(audit) => {
                match self.audit_manager.save_audit(&audit) {
                    Ok(()) => {
                        self.load_audits();
                        self.error_message = None;
                        // Optionnel: sélectionner le nouvel audit
                        if let Some(index) = self.audits.iter().position(|a| a.id == audit.id) {
                            self.selected_audit_index = index;
                        }
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Erreur lors de la sauvegarde: {}", e));
                    }
                }
            }
            Err(e) => {
                self.error_message = Some(format!("Erreur lors de l'import YAML: {}\n", e));
            }
        }
    }

    /// Parse le YAML et crée un audit avec les réponses
    fn parse_and_create_audit_from_yaml(&self, yaml_content: &str) -> Result<Audit, String> {
        #[derive(Debug, Deserialize)]
        struct QuestionYamlImport {
            id: uuid::Uuid,
            text: String,
            description: Option<String>,
            category: String,
            requires_percentage: bool,
            expected_answer: Option<String>,
            links: Option<Vec<String>>,
            answer: Option<String>,
            percentage: Option<u8>,
            comment: Option<String>,
        }
        
        #[derive(Debug, Deserialize)]
        struct AuditYamlImport {
            audit: AuditInfoYaml,
            questions: Vec<QuestionYamlImport>,
        }
        
        #[derive(Debug, Deserialize)]
        struct AuditInfoYaml {
            name: String,
            audit_type: String,
            project_name: Option<String>,
            product_name: Option<String>,
        }
        
        // Pré-traiter le YAML pour corriger les chaînes non échappées
        let fixed_yaml = self.fix_yaml_for_parsing(yaml_content);
        
        let yaml_data: AuditYamlImport = serde_yaml::from_str(&fixed_yaml)
            .map_err(|e| {
                // Améliorer le message d'erreur pour être plus utile
                let error_msg = format!("Erreur de parsing YAML: {}", e);
                // Si l'erreur mentionne une ligne, l'inclure dans le message
                if e.to_string().contains("line") {
                    format!("{} (Vérifiez la syntaxe YAML autour de cette ligne)", error_msg)
                } else {
                    error_msg
                }
            })?;
        
        // Créer l'audit
        let audit_type = match yaml_data.audit.audit_type.as_str() {
            "Technique" => AuditType::Technique,
            "Business" => AuditType::Business,
            "Marketing" => AuditType::Marketing,
            other => AuditType::Autre(other.to_string()),
        };
        
        let mut audit = Audit::new(yaml_data.audit.name, audit_type);
        audit.project_name = yaml_data.audit.project_name;
        audit.product_name = yaml_data.audit.product_name;
        
        // Ajouter les questions et réponses
        for q_yaml in yaml_data.questions {
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
                "Oui" => Answer::Oui,
                "Non" => Answer::Non,
                "JeNeSaisPas" => Answer::JeNeSaisPas,
                "NonApplicable" => Answer::NonApplicable,
                _ => Answer::Oui,
            });
            
            let question = Question {
                id: q_yaml.id,
                text: q_yaml.text,
                description: q_yaml.description,
                category,
                audit_type: audit.audit_type.clone(),
                requires_percentage: q_yaml.requires_percentage,
                expected_answer,
                links: q_yaml.links.unwrap_or_default(),
            };
            
            audit.add_question(question);
            
            // Si une réponse est fournie, l'ajouter
            if let Some(answer_str) = q_yaml.answer {
                // Normaliser la réponse (trim, casse insensible)
                let answer_str_trimmed = answer_str.trim();
                let answer_str_lower = answer_str_trimmed.to_lowercase();
                
                let answer = match answer_str_lower.as_str() {
                    "oui" | "o" | "yes" | "y" => Answer::Oui,
                    "non" | "n" | "no" => Answer::Non,
                    "jene sais pas" | "je ne sais pas" | "jsp" | "j" | "dontknow" | "dont know" | "unknown" => Answer::JeNeSaisPas,
                    "non applicable" | "na" | "a" | "n/a" | "not applicable" => Answer::NonApplicable,
                    _ => {
                        // Essayer de matcher avec la casse originale aussi
                        match answer_str_trimmed {
                            s if s == "Oui" || s == "OUI" => Answer::Oui,
                            s if s == "Non" || s == "NON" => Answer::Non,
                            s if s == "JeNeSaisPas" || s == "Je ne sais pas" => Answer::JeNeSaisPas,
                            s if s == "NonApplicable" || s == "Non applicable" => Answer::NonApplicable,
                            _ => {
                                return Err(format!("Réponse invalide: '{}'. Réponses acceptées: Oui, Non, JeNeSaisPas, NonApplicable", answer_str_trimmed));
                            }
                        }
                    }
                };
                
                let question_answer = QuestionAnswer {
                    question_id: q_yaml.id,
                    answer,
                    percentage: q_yaml.percentage,
                    comment: q_yaml.comment,
                    answered_at: chrono::Utc::now(),
                };
                
                audit.answer_question(question_answer);
            }
        }
        
        Ok(audit)
    }

    /// Télécharge un fichier texte dans le navigateur
    #[cfg(target_arch = "wasm32")]
    fn download_text_file(&self, content: &str, filename: &str) {
        if let Some(window) = window() {
            if let Some(document) = window.document() {
                // Créer un tableau de chaînes pour le blob
                let content_js = wasm_bindgen::JsValue::from_str(content);
                let content_array = js_sys::Array::new();
                content_array.push(&content_js);
                
                // Créer un blob avec le contenu
                let blob_options = {
                    let opts = web_sys::BlobPropertyBag::new();
                    opts.set_type("text/plain;charset=utf-8");
                    opts
                };
                
                if let Ok(blob) = web_sys::Blob::new_with_str_sequence_and_options(&content_array, &blob_options) {
                    // Créer une URL pour le blob
                    if let Ok(url) = Url::create_object_url_with_blob(&blob) {
                        // Créer un élément <a> pour le téléchargement
                        if let Ok(anchor) = document.create_element("a") {
                            if let Some(anchor_elem) = anchor.dyn_ref::<HtmlAnchorElement>() {
                                anchor_elem.set_href(&url);
                                anchor_elem.set_download(filename);
                                anchor_elem.style().set_property("display", "none").ok();
                                
                                // Ajouter au document, cliquer, puis retirer
                                if let Some(body) = document.body() {
                                    if body.append_child(&anchor).is_ok() {
                                        anchor_elem.click();
                                        let _ = body.remove_child(&anchor);
                                    }
                                }
                                
                                // Libérer l'URL du blob
                                let _ = Url::revoke_object_url(&url);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Version pour les tests (non-wasm32) - ne fait rien
    #[cfg(not(target_arch = "wasm32"))]
    fn download_text_file(&self, _content: &str, _filename: &str) {
        // En mode test, on ne télécharge pas vraiment de fichier
    }

    pub fn handle_tab(&mut self) {
        match self.view {
            View::Create => {
                self.input_mode = match self.input_mode {
                    InputMode::Name => InputMode::Type,
                    InputMode::Type => InputMode::Project,
                    InputMode::Project => InputMode::Product,
                    _ => InputMode::Name,
                };
            }
            _ => {}
        }
    }

    pub fn handle_left(&mut self) {
        match self.view {
            View::Create => {
                if self.input_mode == InputMode::Type {
                    if self.new_audit_type_index > 0 {
                        self.new_audit_type_index -= 1;
                    } else {
                        self.new_audit_type_index = AUDIT_TYPES.len() - 1;
                    }
                }
            }
            _ => {}
        }
    }

    pub fn handle_right(&mut self) {
        match self.view {
            View::Create => {
                if self.input_mode == InputMode::Type {
                    if self.new_audit_type_index < AUDIT_TYPES.len() - 1 {
                        self.new_audit_type_index += 1;
                    } else {
                        self.new_audit_type_index = 0;
                    }
                }
            }
            _ => {}
        }
    }

    pub fn handle_char_input(&mut self, c: char) {
        // Ignorer les caractères de contrôle
        if c.is_control() {
            return;
        }
        
        match self.input_mode {
            InputMode::Name => {
                self.new_audit_name.push(c);
            }
            InputMode::Type => {
                // Le type est sélectionné avec les flèches, pas de saisie de texte
            }
            InputMode::Project => {
                self.new_project_name.push(c);
            }
            InputMode::Product => {
                self.new_product_name.push(c);
            }
            InputMode::Comment => {
                self.answer_comment.push(c);
            }
            _ => {}
        }
    }

    pub fn handle_backspace(&mut self) {
        match self.input_mode {
            InputMode::Name => {
                self.new_audit_name.pop();
            }
            InputMode::Type => {
                // Le type est sélectionné avec les flèches, pas de backspace
            }
            InputMode::Project => {
                self.new_project_name.pop();
            }
            InputMode::Product => {
                self.new_product_name.pop();
            }
            InputMode::Comment => {
                self.answer_comment.pop();
            }
            _ => {}
        }
    }

    pub fn get_current_audit(&self) -> Option<&Audit> {
        if self.view == View::Detail && !self.audits.is_empty() {
            self.audits.get(self.selected_audit_index)
        } else {
            None
        }
    }

    /// Vérifie si l'audit actuel ou sélectionné est complété à 100%
    pub fn is_audit_complete(&self) -> bool {
        if let Some(audit) = self.get_current_audit_for_recommendations() {
            audit.completion_percentage() >= 100.0
        } else {
            false
        }
    }

    pub fn create_audit(&mut self) {
        // Validation: le nom ne doit pas être vide
        let name = self.new_audit_name.trim();
        if name.is_empty() {
            self.error_message = Some("Le nom de l'audit ne peut pas être vide".to_string());
            return;
        }

        let audit_type_str = AUDIT_TYPES[self.new_audit_type_index];
        let audit_type = match audit_type_str {
            "technique" => AuditType::Technique,
            "business" => AuditType::Business,
            "marketing" => AuditType::Marketing,
            _ => AuditType::Autre(audit_type_str.to_string()),
        };

        let mut audit = Audit::new(name.to_string(), audit_type.clone());
        audit.project_name = if self.new_project_name.is_empty() {
            None
        } else {
            Some(self.new_project_name.clone())
        };
        audit.product_name = if self.new_product_name.is_empty() {
            None
        } else {
            Some(self.new_product_name.clone())
        };

        // Ajouter les questions selon le type d'audit
        match audit.audit_type {
            AuditType::Technique => {
                let mut questions = QuestionGenerator::generate_technical_questions();
                // Trier les questions par catégorie pour les grouper
                questions.sort_by(|a, b| {
                    format!("{}", a.category).cmp(&format!("{}", b.category))
                });
                for q in questions {
                    audit.add_question(q);
                }
            }
            AuditType::Business => {
                let mut questions = QuestionGenerator::generate_business_questions();
                // Trier les questions par catégorie pour les grouper
                questions.sort_by(|a, b| {
                    format!("{}", a.category).cmp(&format!("{}", b.category))
                });
                for q in questions {
                    audit.add_question(q);
                }
            }
            AuditType::Marketing => {
                let mut questions = QuestionGenerator::generate_marketing_questions();
                // Trier les questions par catégorie pour les grouper
                questions.sort_by(|a, b| {
                    format!("{}", a.category).cmp(&format!("{}", b.category))
                });
                for q in questions {
                    audit.add_question(q);
                }
            }
            _ => {
                // Pas de questions par défaut pour les autres types
            }
        }

        match self.audit_manager.save_audit(&audit) {
            Ok(()) => {
                self.load_audits();
                self.view = View::List;
                self.new_audit_name.clear();
                self.new_audit_type_index = 0;
                self.new_project_name.clear();
                self.new_product_name.clear();
            }
            Err(e) => {
                self.error_message = Some(format!("Erreur lors de la création de l'audit: {}", e));
            }
        }
    }

    pub fn save_answer(&mut self) {
        let audit_index = self.selected_audit_index;
        if audit_index >= self.audits.len() {
            return;
        }

        // Récupérer l'ID de la question actuelle avant de modifier l'audit
        let current_question_id = {
            let audit = &self.audits[audit_index];
            // Utiliser l'index réel si on est en mode filtre
            let real_index = if self.view == View::Detail {
                self.get_real_question_index().unwrap_or(self.current_question_index)
            } else {
                self.current_question_index
            };
            if real_index >= audit.questions.len() {
                return;
            }
            audit.questions[real_index].id
        };
        
        // Parser la réponse en utilisant les constantes
        let answer = self.parse_answer_input(&self.answer_value);
        let answer = match answer {
            Some(a) => a,
            None => {
                self.error_message = Some("Réponse invalide".to_string());
                return;
            }
        };

        // Modifier l'audit directement dans le vecteur
        let audit = &mut self.audits[audit_index];
        
        // Trouver la question pour vérifier si elle nécessite un pourcentage
        let question = audit.questions.iter()
            .find(|q| q.id == current_question_id);
        
        let requires_percentage = question.map(|q| q.requires_percentage).unwrap_or(false);
        
        // Valider et parser le pourcentage si nécessaire
        // Si la question nécessite un pourcentage mais qu'il n'est pas fourni, on le met à None
        // (l'utilisateur pourra le mettre à jour plus tard)
        let percentage = if requires_percentage {
            if self.answer_percentage.is_empty() {
                None
            } else {
                match self.answer_percentage.parse::<u8>() {
                    Ok(p) if p <= 100 => Some(p),
                    _ => {
                        self.error_message = Some("Pourcentage invalide (doit être entre 0 et 100)".to_string());
                        return;
                    }
                }
            }
        } else {
            None
        };

        let comment = if self.answer_comment.is_empty() {
            None
        } else {
            Some(self.answer_comment.clone())
        };

        // Créer la réponse
        let question_answer = QuestionAnswer {
            question_id: current_question_id,
            answer,
            percentage,
            comment,
            answered_at: chrono::Utc::now(),
        };

        audit.answer_question(question_answer);

        // Sauver l'index actuel avant la sauvegarde
        let saved_index = self.current_question_index;
        let was_filtered = self.filter_unanswered;
        
        // Sauvegarder
        match self.audit_manager.save_audit(audit) {
            Ok(()) => {
                self.answer_value.clear();
                self.answer_percentage.clear();
                self.answer_comment.clear();
                
                // Passer à la question suivante
                if was_filtered {
                    // En mode filtre, on passe à la prochaine question non répondue
                    let unanswered_indices = self.get_unanswered_question_indices();
                    if !unanswered_indices.is_empty() {
                        // Trouver la position de la question actuelle dans la liste filtrée
                        if let Some(current_real_idx) = self.get_real_question_index() {
                            // Trouver la prochaine question non répondue après celle qu'on vient de répondre
                            if let Some(next_pos) = unanswered_indices.iter().position(|&idx| idx > current_real_idx) {
                                self.current_question_index = next_pos;
                            } else {
                                // Plus de questions non répondues après celle-ci
                                // On reste sur la dernière question non répondue ou on désactive le filtre
                                if saved_index < unanswered_indices.len() {
                                    self.current_question_index = unanswered_indices.len().saturating_sub(1);
                                } else {
                                    // Toutes les questions sont maintenant répondues
                                    self.current_question_index = 0;
                                }
                            }
                        } else {
                            // Si on ne trouve pas l'index réel, on reste à la position actuelle
                            self.current_question_index = saved_index;
                        }
                    } else {
                        // Toutes les questions sont maintenant répondues
                        self.current_question_index = 0;
                    }
                } else {
                    // Mode normal : passer à la question suivante
                    if saved_index < audit.questions.len().saturating_sub(1) {
                        self.current_question_index = saved_index + 1;
                    } else {
                        // On est à la dernière question, on reste sur la dernière
                        self.current_question_index = audit.questions.len().saturating_sub(1);
                    }
                }
            }
            Err(e) => {
                self.error_message = Some(format!("Erreur lors de la sauvegarde: {}", e));
            }
        }
    }

    /// Parse l'input utilisateur en Answer en utilisant les constantes
    fn parse_answer_input(&self, input: &str) -> Option<Answer> {
        let input_lower = input.to_lowercase();
        if answer_inputs::OUI.iter().any(|&s| input_lower == s) {
            Some(Answer::Oui)
        } else if answer_inputs::NON.iter().any(|&s| input_lower == s) {
            Some(Answer::Non)
        } else if answer_inputs::JE_NE_SAIS_PAS.iter().any(|&s| input_lower == s) {
            Some(Answer::JeNeSaisPas)
        } else if answer_inputs::NON_APPLICABLE.iter().any(|&s| input_lower == s) {
            Some(Answer::NonApplicable)
        } else {
            None
        }
    }

    pub fn render(&self, f: &mut Frame) {
        match self.view {
            View::List => self.render_list(f),
            View::Detail => self.render_detail(f),
            View::Create => self.render_create(f),
            View::Recommendations => self.render_recommendations(f),
        }
    }

    fn render_list(&self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(if self.error_message.is_some() { 5 } else { 3 }),
            ])
            .split(f.area());

        // Header
        let header = Paragraph::new("Survey Audit - Outil d'audit technique et business")
            .style(Style::default().fg(Color::Rgb(0, 255, 255)).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(header, chunks[0]);

        // Calculer le score global pour chaque audit
        let rows: Vec<Row> = self
            .audits
            .iter()
            .enumerate()
            .map(|(i, audit)| {
                let completion = audit.completion_percentage();
                
                // Calculer le score global (utilise la fonction utilitaire pour éviter la duplication)
                let global_score = calculate_global_score(audit);
                
                let score_text = match global_score {
                    Some(score) => format!("{:.1}%", score),
                    None => "N/A".to_string(),
                };
                
                let row_style = if i == self.selected_audit_index {
                    Style::default().fg(Color::Rgb(0, 255, 255)).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                
                // Appliquer une couleur au score selon les criticités (même système que les priorités)
                // Un score élevé = bon = Faible priorité (vert)
                // Un score faible = mauvais = Critique priorité (rouge)
                let score_color = match global_score {
                    Some(score) if score >= score_thresholds::EXCELLENT => Color::Rgb(0, 255, 0), // Vert vif (Faible)
                    Some(score) if score >= score_thresholds::GOOD => Color::Rgb(255, 255, 0), // Jaune vif (Moyenne)
                    Some(score) if score >= score_thresholds::MEDIUM => Color::Rgb(255, 165, 0), // Orange (Élevée)
                    Some(_) => Color::Rgb(255, 0, 0), // Rouge (Critique)
                    None => Color::DarkGray,
                };
                
                Row::new(vec![
                    Line::from(audit.name.clone()),
                    Line::from(format!("{}", audit.audit_type)),
                    Line::from(format!("{}", audit.questions.len())),
                    Line::from(format!("{:.1}%", completion)),
                    Line::from(Span::styled(score_text, Style::default().fg(score_color))),
                ])
                .style(row_style)
            })
            .collect();

        let widths = [
            Constraint::Percentage(30),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(25),
        ];
        
        // Créer une ligne de séparation avec des caractères ─ qui s'étendent
        // Utiliser un caractère répété pour créer une ligne visuelle
        let separator = Row::new(vec![
            Line::from("─".repeat(50)),
            Line::from("─".repeat(25)),
            Line::from("─".repeat(25)),
            Line::from("─".repeat(25)),
            Line::from("─".repeat(40)),
        ])
        .style(Style::default().fg(Color::DarkGray));
        
        // Insérer la ligne de séparation après le header dans les rows
        let mut all_rows = vec![separator];
        all_rows.extend(rows);
        
        let table = Table::new(all_rows, widths)
            .header(Row::new(vec!["Nom", "Type", "Questions", "Complétion", "Résultat global"])
                .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)))
            .block(Block::default().borders(Borders::ALL).title("Mes audits"))
            .column_spacing(1);
        
        f.render_widget(table, chunks[1]);

        // Footer avec message d'erreur si présent ou message de confirmation
        let footer_text = if self.pending_delete_audit_id.is_some() {
            if let Some(audit) = self.audits.get(self.selected_audit_index) {
                format!("⚠️  Confirmer la suppression de '{}' ? (o: Oui, n: Non)", audit.name)
            } else {
                "⚠️  Confirmer la suppression ? (o: Oui, n: Non)".to_string()
            }
        } else if let Some(ref error) = self.error_message {
            format!("⚠️  {} | n: Nouvel audit | ↑↓: Navigation | Entrée: Ouvrir | r: Résultats | e: Exporter TXT | y: Exporter YAML | i: Importer | d: Supprimer | q: Quitter", error)
        } else {
            "n: Nouvel audit | ↑↓: Navigation | Entrée: Ouvrir | r: Résultats | e: Exporter TXT | y: Exporter YAML | i: Importer | d: Supprimer | q: Quitter".to_string()
        };
        let help = Paragraph::new(footer_text)
            .style(
                if self.pending_delete_audit_id.is_some() {
                    Style::default().fg(Color::Rgb(255, 0, 255)).add_modifier(Modifier::BOLD)
                } else if self.error_message.is_some() {
                    Style::default().fg(Color::Red)
                } else {
                    Style::default().fg(Color::DarkGray)
                }
            )
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(help, chunks[2]);
    }

    fn render_detail(&self, f: &mut Frame) {
        let audit = match self.get_current_audit() {
            Some(a) => a,
            None => {
                let msg = Paragraph::new("Aucun audit sélectionné")
                    .alignment(Alignment::Center)
                    .block(Block::default().borders(Borders::ALL));
                f.render_widget(msg, f.area());
                return;
            }
        };

        if audit.questions.is_empty() {
            let msg = Paragraph::new("Aucune question disponible")
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(msg, f.area());
            return;
        }

        let visible_indices = self.get_visible_question_indices();
        
        if visible_indices.is_empty() {
            let msg = if self.filter_unanswered {
                Paragraph::new("✓ Toutes les questions ont été répondues ! Appuyez sur 'f' pour voir toutes les questions.")
                    .alignment(Alignment::Center)
                    .block(Block::default().borders(Borders::ALL))
            } else {
                Paragraph::new("✓ Audit terminé ! Toutes les questions ont été répondues.")
                    .alignment(Alignment::Center)
                    .block(Block::default().borders(Borders::ALL))
            };
            f.render_widget(msg, f.area());
            return;
        }

        // Ajuster l'index si nécessaire pour éviter les index hors limites
        let current_idx = if self.current_question_index >= visible_indices.len() {
            visible_indices.len().saturating_sub(1)
        } else {
            self.current_question_index
        };

        let real_index = visible_indices[current_idx];
        let question = &audit.questions[real_index];
        let existing_answer = audit.answers.iter().find(|a| a.question_id == question.id);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(f.area());

        // Header
        let header_text = format!("{} - Complétion: {:.1}%", audit.name, audit.completion_percentage());
        let header = Paragraph::new(header_text)
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(header, chunks[0]);

        // Progression
        let total_questions = if self.filter_unanswered {
            visible_indices.len()
        } else {
            audit.questions.len()
        };
        let filter_indicator = if self.filter_unanswered {
            format!(" [Filtre: {} non répondues]", visible_indices.len())
        } else {
            String::new()
        };
        let progress_text = format!(
            "Question {} / {}{} - Catégorie: {}",
            current_idx + 1,
            total_questions,
            filter_indicator,
            question.category
        );
        let progress_widget = Paragraph::new(progress_text)
            .style(Style::default().fg(Color::Green))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(progress_widget, chunks[1]);

        // Diviser la zone principale en 3 sections
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50),  // Section Question + Description (flexible)
                Constraint::Length(7),       // Section Réponse (fixe)
                Constraint::Min(4),          // Section Commentaire (minimum 4 lignes)
            ])
            .split(chunks[2]);

        // Section 1: Question + Description
        let mut question_lines = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                &question.text,
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
        ];
        
        if let Some(desc) = &question.description {
            question_lines.push(Line::from(vec![Span::styled(
                "Description:",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            )]));
            question_lines.push(Line::from(""));
            question_lines.push(Line::from(vec![Span::styled(
                desc,
                Style::default().fg(Color::DarkGray),
            )]));
        }

        let question_widget = Paragraph::new(question_lines)
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Question ")
                    .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            );
        f.render_widget(question_widget, main_chunks[0]);

        // Section 2: Réponse
        let mut answer_lines = vec![
            Line::from(""),
        ];

        if let Some(answer) = existing_answer {
            let answer_color = match answer.answer {
                Answer::Oui => Color::Green,
                Answer::Non => Color::Red,
                Answer::JeNeSaisPas => Color::Yellow,
                Answer::NonApplicable => Color::DarkGray,
            };
            answer_lines.push(Line::from(vec![
                Span::styled(
                    format!("{}", answer.answer),
                    Style::default().fg(answer_color).add_modifier(Modifier::BOLD),
                ),
            ]));
            if let Some(percentage) = answer.percentage {
                answer_lines.push(Line::from(vec![
                    Span::styled("Pourcentage: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{}%", percentage),
                        Style::default().fg(Color::Cyan),
                    ),
                ]));
            }
        } else {
            answer_lines.push(Line::from(vec![Span::styled(
                "Aucune réponse",
                Style::default().fg(Color::DarkGray),
            )]));
        }

        answer_lines.push(Line::from(""));
        if self.input_mode == InputMode::Comment {
            answer_lines.push(Line::from(vec![Span::styled(
                "Mode commentaire actif - Tapez votre commentaire puis Entrée",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            )]));
        } else {
            answer_lines.push(Line::from(vec![Span::styled(
                "Raccourcis: c=Commentaire | o=Oui | n=Non | j=Je ne sais pas | a=Non applicable",
                Style::default().fg(Color::DarkGray),
            )]));
        }

        let answer_widget = Paragraph::new(answer_lines)
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Réponse ")
                    .title_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
            );
        f.render_widget(answer_widget, main_chunks[1]);

        // Section 3: Commentaire
        let mut comment_lines = vec![
            Line::from(""),
        ];

        if self.input_mode == InputMode::Comment {
            // Afficher le commentaire en cours de saisie
            if self.answer_comment.is_empty() {
                comment_lines.push(Line::from(vec![
                    Span::styled("> ", Style::default().fg(Color::Yellow)),
                    Span::styled("_", Style::default().fg(Color::DarkGray)),
                ]));
            } else {
                // Afficher le commentaire ligne par ligne
                for line in self.answer_comment.lines() {
                    comment_lines.push(Line::from(vec![
                        Span::styled("> ", Style::default().fg(Color::Yellow)),
                        Span::styled(line, Style::default().fg(Color::White)),
                    ]));
                }
            }
        } else {
            // Afficher le commentaire sauvegardé ou le commentaire en cours (pas encore sauvegardé)
            let comment_to_show = if let Some(answer) = existing_answer {
                answer.comment.as_ref()
            } else {
                None
            };
            
            // Si pas de commentaire sauvegardé mais qu'on a un commentaire en cours, l'afficher
            let comment_to_show = if comment_to_show.is_some() {
                comment_to_show
            } else if !self.answer_comment.is_empty() {
                Some(&self.answer_comment)
            } else {
                None
            };
            
            if let Some(comment) = comment_to_show {
                for line in comment.lines() {
                    comment_lines.push(Line::from(vec![Span::styled(
                        line,
                        Style::default().fg(Color::White),
                    )]));
                }
            } else {
                comment_lines.push(Line::from(vec![Span::styled(
                    "Aucun commentaire",
                    Style::default().fg(Color::DarkGray),
                )]));
            }
        }

        let comment_widget = Paragraph::new(comment_lines)
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Commentaire ")
                    .title_style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))
            );
        f.render_widget(comment_widget, main_chunks[2]);

        // Footer avec instructions claires
        let filter_hint = if self.filter_unanswered {
            " | f: Voir toutes"
        } else {
            " | f: Filtrer non répondues"
        };
        let help_text = if self.input_mode == InputMode::Comment {
            format!("Mode commentaire | Entrée: Valider | Esc: Annuler | r: Résultats | e: Exporter{filter_hint} | q: Retour")
        } else if existing_answer.is_some() {
            format!("↑↓: Navigation | c: Commentaire | o/n/j/a: Répondre | r: Résultats | e: Exporter{filter_hint} | q: Retour")
        } else {
            format!("↑↓: Navigation | c: Commentaire | o: Oui | n: Non | j: Je ne sais pas | a: Non applicable | r: Résultats | e: Exporter{filter_hint} | q: Retour")
        };
        let help = Paragraph::new(help_text)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(help, chunks[3]);
    }

    fn render_create(&self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(f.area());

        let header = Paragraph::new("Créer un nouvel audit")
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(header, chunks[0]);

        let form_text = vec![
            Line::from(vec![
                Span::styled("Nom: ", Style::default().fg(Color::Rgb(0, 255, 255)).add_modifier(Modifier::BOLD)),
                Span::styled(
                    if self.new_audit_name.is_empty() { "_" } else { &self.new_audit_name },
                    if self.input_mode == InputMode::Name {
                        Style::default().fg(Color::Rgb(255, 165, 0)).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    },
                ),
            ]),
            Line::from({
                let mut type_spans = vec![Span::styled("Type: ", Style::default().fg(Color::Rgb(0, 255, 255)).add_modifier(Modifier::BOLD))];
                for (i, &type_name) in AUDIT_TYPES.iter().enumerate() {
                    let is_selected = i == self.new_audit_type_index;
                    let is_active = self.input_mode == InputMode::Type;
                    let style = if is_selected && is_active {
                        Style::default().fg(Color::Rgb(255, 165, 0)).add_modifier(Modifier::BOLD)
                    } else if is_selected {
                        Style::default().fg(Color::Rgb(255, 165, 0)).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    };
                    if i > 0 {
                        type_spans.push(Span::styled(" | ", Style::default()));
                    }
                    type_spans.push(Span::styled(type_name, style));
                }
                Line::from(type_spans)
            }),
            Line::from(vec![
                Span::styled("Projet: ", Style::default().fg(Color::Rgb(0, 255, 255)).add_modifier(Modifier::BOLD)),
                Span::styled(
                    if self.new_project_name.is_empty() { "_" } else { &self.new_project_name },
                    if self.input_mode == InputMode::Project {
                        Style::default().fg(Color::Rgb(255, 165, 0)).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    },
                ),
            ]),
            Line::from(vec![
                Span::styled("Produit: ", Style::default().fg(Color::Rgb(0, 255, 255)).add_modifier(Modifier::BOLD)),
                Span::styled(
                    if self.new_product_name.is_empty() { "_" } else { &self.new_product_name },
                    if self.input_mode == InputMode::Product {
                        Style::default().fg(Color::Rgb(255, 165, 0)).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    },
                ),
            ]),
        ];

        let form = Paragraph::new(form_text)
            .block(Block::default().borders(Borders::ALL).title("Formulaire"));
        f.render_widget(form, chunks[1]);

        let help_text = match self.input_mode {
            InputMode::Normal => "Entrée: Commencer la saisie | q: Annuler",
            InputMode::Type => "←→ ou ↑↓: Changer de type | Tab: Champ suivant | Entrée: Passer au suivant | q/Esc: Retour",
            _ => "Tapez pour saisir | Tab: Champ suivant | Entrée: Passer au suivant/Créer | q/Esc: Retour",
        };
        let help = Paragraph::new(help_text)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(help, chunks[2]);
    }

    fn render_recommendations(&self, f: &mut Frame) {
        // Récupérer l'audit actuel ou sélectionné
        let audit = self.get_current_audit_for_recommendations();

        if let Some(audit) = audit {
            // Générer les recommandations et le résumé (une seule fois par frame)
            let recommendations = RecommendationEngine::generate_recommendations(audit);
            let summary = RecommendationEngine::generate_summary(audit);
            
            // S'assurer que le scroll est à 0 (on affiche tout maintenant)
            // Note: on ne peut pas modifier self ici, mais on n'utilise plus summary_scroll

            // Trier les catégories par pourcentage positif (décroissant)
            // Utiliser un tri stable avec un critère secondaire (nom de catégorie) pour éviter les changements
            let mut sorted_stats: Vec<_> = summary.category_stats.iter().collect();
            sorted_stats.sort_by(|a, b| {
                // Tri principal par pourcentage (décroissant)
                let percent_cmp = b.positive_percentage().partial_cmp(&a.positive_percentage()).unwrap_or(std::cmp::Ordering::Equal);
                if percent_cmp != std::cmp::Ordering::Equal {
                    percent_cmp
                } else {
                    // Tri secondaire par nom de catégorie (alphabétique) pour stabilité
                    format!("{}", a.category).cmp(&format!("{}", b.category))
                }
            });

            // Calculer la hauteur nécessaire pour l'histogramme
            // Le Block avec bordures prend 2 lignes (haut et bas)
            // Le titre prend 1 ligne, la ligne vide 1 ligne
            // Chaque catégorie prend 1 ligne
            let num_categories = sorted_stats.len();
            // +4 pour: titre (1) + ligne vide (1) + bordures haut/bas du Block (2)
            let histogram_height = (num_categories + 4) as u16;
            let min_histogram_height = 9; // Hauteur minimale
            let histogram_height = histogram_height.max(min_histogram_height);

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length(histogram_height), // Hauteur exacte pour éviter l'espace supplémentaire
                    Constraint::Min(0),
                    Constraint::Length(3),
                ])
                .split(f.area());

            // Header
            let header_text = format!("Résultats: {} - Complétion: {:.1}%", audit.name, summary.completion_percentage);
            let header = Paragraph::new(header_text)
                .style(Style::default().fg(Color::Cyan))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(header, chunks[0]);

            // Préparer les lignes pour l'histogramme (une seule colonne)
            let mut histogram_lines = vec![
                Line::from(vec![
                    Span::styled("Résultats par catégorie (% positif)", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(""),
            ];

            // Largeur max des barres
            // Réserver de l'espace pour :
            // - nom catégorie : format!("{:12} ") = 13 caractères (12 + espace)
            // - pourcentage positif : format!(" {:4.0}%") = 6 caractères (espace + 4 chiffres + %)
            // - complétion : format!(" ({:3.0}%)") = 7 caractères (espace + parenthèse + 3 chiffres + % + parenthèse)
            // Total = 13 + 6 + 7 = 26 caractères + marge de sécurité pour les bordures
            let max_bar_width = (chunks[1].width.saturating_sub(28)).max(10) as usize;

            // Afficher TOUTES les catégories
            for stats in &sorted_stats {
                let positive = stats.positive_percentage();
                let completion = stats.completion_percentage();
                let category_name = format!("{}", stats.category);
                let category_display = if category_name.len() > 9 {
                    format!("{}...", &category_name[..9])
                } else {
                    category_name
                };
                
                // Couleur selon le pourcentage - Échelle rouge à vert avec couleurs vives
                let color = if positive >= score_thresholds::EXCELLENT {
                    Color::Rgb(0, 255, 0) // Vert vif
                } else if positive >= score_thresholds::GOOD {
                    Color::Rgb(255, 255, 0) // Jaune vif
                } else if positive >= score_thresholds::MEDIUM {
                    Color::Rgb(255, 165, 0) // Orange vif
                } else {
                    Color::Rgb(255, 0, 0) // Rouge vif
                };
                
                // Calculer la largeur de la barre
                let bar_width = ((positive / 100.0) * max_bar_width as f64) as usize;
                let bar = "▒".repeat(bar_width); // Utiliser un caractère à 50% de densité pour éviter la fatigue visuelle
                let empty = "░".repeat(max_bar_width.saturating_sub(bar_width));
                
                histogram_lines.push(Line::from(vec![
                    Span::styled(
                        format!("{:12} ", category_display),
                        Style::default().fg(Color::White),
                    ),
                    Span::styled(
                        bar,
                        Style::default().fg(color),
                    ),
                    Span::styled(
                        empty,
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(
                        format!(" {:4.0}%", positive),
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!(" ({:3.0}%)", completion),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]));
            }
            
            let histogram_widget = Paragraph::new(histogram_lines)
                .block(Block::default().borders(Borders::ALL).title("Histogramme des résultats"));
            f.render_widget(histogram_widget, chunks[1]);

            // Recommandations - Trier par priorité (du plus élevé au plus faible)
            let mut sorted_recommendations = recommendations.clone();
            sorted_recommendations.sort_by(|a, b| b.priority.cmp(&a.priority)); // Ordre décroissant (Critique > Élevée > Moyenne > Faible)
            
            // Construire les lignes pour le Paragraph avec wrapping
            let mut rec_lines = Vec::new();
            if sorted_recommendations.is_empty() {
                rec_lines.push(Line::from(vec![
                    Span::styled(
                        "✓ Aucune recommandation - Toutes les pratiques sont bonnes !",
                        Style::default().fg(Color::Green),
                    ),
                ]));
            } else {
                for (index, rec) in sorted_recommendations.iter().enumerate() {
                    let priority_color = match rec.priority {
                        Priority::Critique => Color::Rgb(255, 0, 0), // Rouge
                        Priority::Elevee => Color::Rgb(255, 165, 0), // Orange
                        Priority::Moyenne => Color::Rgb(255, 255, 0), // Jaune vif (au lieu de kaki)
                        Priority::Faible => Color::Rgb(0, 255, 0), // Vert vif
                    };

                    // Titre avec index et priorité
                    rec_lines.push(Line::from(vec![
                        Span::styled(
                            format!("{}. ", index + 1),
                            Style::default().fg(Color::Cyan),
                        ),
                        Span::styled(
                            format!("[{}] ", rec.priority),
                            Style::default().fg(priority_color).add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            &rec.title,
                            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                        ),
                    ]));
                    
                    // Description avec catégorie
                    rec_lines.push(Line::from(vec![
                        Span::styled(
                            format!("  Catégorie: {} | ", rec.category),
                            Style::default().fg(Color::DarkGray),
                        ),
                        Span::styled(
                            &rec.description,
                            Style::default().fg(Color::DarkGray),
                        ),
                    ]));
                    
                    rec_lines.push(Line::from(""));
                }
            }

            let recommendations_title = format!("Recommandations ({})", sorted_recommendations.len());
            
            // Calculer le nombre de lignes visibles (soustraire les bordures : 2)
            let visible_height = chunks[2].height.saturating_sub(2) as usize;
            let total_lines = rec_lines.len();
            
            // Calculer le scroll offset
            let max_scroll = if total_lines > visible_height {
                total_lines.saturating_sub(visible_height)
            } else {
                0
            };
            let scroll_offset = if self.recommendations_scroll > max_scroll {
                max_scroll
            } else {
                self.recommendations_scroll
            };
            
            // Prendre seulement les lignes visibles
            let visible_lines: Vec<Line> = rec_lines.iter()
                .skip(scroll_offset)
                .take(visible_height)
                .cloned()
                .collect();
            
            // Utiliser un Paragraph avec wrapping au lieu d'une List
            let recommendations_paragraph = Paragraph::new(visible_lines)
                .wrap(Wrap { trim: true })
                .block(Block::default().borders(Borders::ALL).title(recommendations_title));
            
            f.render_widget(recommendations_paragraph, chunks[2]);
            
            // Afficher la scrollbar si nécessaire
            if total_lines > visible_height {
                let scrollable_range = max_scroll + 1;
                let mut scrollbar_state = ScrollbarState::new(scrollable_range)
                    .position(scroll_offset.min(max_scroll));
                let scrollbar = Scrollbar::default()
                    .orientation(ratzilla::ratatui::widgets::ScrollbarOrientation::VerticalRight)
                    .begin_symbol(Some("↑"))
                    .end_symbol(Some("↓"))
                    .track_symbol(Some("│"))
                    .thumb_symbol("█");
                f.render_stateful_widget(scrollbar, chunks[2], &mut scrollbar_state);
            }

            // Footer
            let help = Paragraph::new("↑↓: Scroller les recommandations | e: Exporter | q: Retour à la liste | r: Actualiser")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(help, chunks[3]);
        } else {
            let msg = Paragraph::new("Aucun audit sélectionné. Retournez à la liste (q) et sélectionnez un audit.")
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(msg, f.area());
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        let mut state = Self {
            view: View::List,
            audit_manager: AuditManager::default(),
            audits: Vec::new(),
            selected_audit_index: 0,
            current_question_index: 0,
            new_audit_name: String::new(),
            new_audit_type_index: 0,
            new_project_name: String::new(),
            new_product_name: String::new(),
            answer_value: String::new(),
            answer_percentage: String::new(),
            answer_comment: String::new(),
            input_mode: InputMode::Normal,
            summary_scroll: 0,
            recommendations_scroll: 0,
            error_message: None,
            pending_delete_audit_id: None,
            import_file_input_id: None,
            filter_unanswered: false,
        };
        state.load_audits();
        state
    }
}

