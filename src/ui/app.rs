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
                // Entrée passe à la question suivante
                // Les réponses sont sauvegardées automatiquement quand on tape o/n/j/a
                if let Some(audit) = self.get_current_audit() {
                    if self.current_question_index < audit.questions.len().saturating_sub(1) {
                        self.current_question_index += 1;
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
                if self.current_question_index > 0 {
                    self.current_question_index -= 1;
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

    pub fn handle_down(&mut self) {
        match self.view {
            View::List => {
                if self.selected_audit_index < self.audits.len().saturating_sub(1) {
                    self.selected_audit_index += 1;
                }
            }
            View::Detail => {
                if let Some(audit) = self.get_current_audit() {
                    if self.current_question_index < audit.questions.len().saturating_sub(1) {
                        self.current_question_index += 1;
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
                let audit_id = self.audits[index].id;
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
                let questions = QuestionGenerator::generate_technical_questions();
                for q in questions {
                    audit.add_question(q);
                }
            }
            AuditType::Business => {
                let questions = QuestionGenerator::generate_business_questions();
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
            if self.current_question_index >= audit.questions.len() {
                return;
            }
            audit.questions[self.current_question_index].id
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
        
        // Sauvegarder
        match self.audit_manager.save_audit(audit) {
            Ok(()) => {
                self.answer_value.clear();
                self.answer_percentage.clear();
                self.answer_comment.clear();
                
                // Passer à la question suivante
                // On utilise l'index sauvegardé pour éviter les problèmes de référence
                if saved_index < audit.questions.len().saturating_sub(1) {
                    self.current_question_index = saved_index + 1;
                } else {
                    // On est à la dernière question, on reste sur la dernière
                    self.current_question_index = audit.questions.len().saturating_sub(1);
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
            .style(Style::default().fg(Color::Yellow))
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
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
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

        // Footer avec message d'erreur si présent
        let footer_text = if let Some(ref error) = self.error_message {
            format!("⚠️  {} | n: Nouvel audit | ↑↓: Navigation | Entrée: Ouvrir | r: Résultats | d: Supprimer | q: Quitter", error)
        } else {
            "n: Nouvel audit | ↑↓: Navigation | Entrée: Ouvrir | r: Résultats | d: Supprimer | q: Quitter".to_string()
        };
        let help = Paragraph::new(footer_text)
            .style(Style::default().fg(if self.error_message.is_some() { Color::Red } else { Color::DarkGray }))
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

        if self.current_question_index >= audit.questions.len() {
            let msg = Paragraph::new("✓ Audit terminé ! Toutes les questions ont été répondues.")
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(msg, f.area());
            return;
        }

        let question = &audit.questions[self.current_question_index];
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
        let progress_text = format!(
            "Question {} / {} - Catégorie: {}",
            self.current_question_index + 1,
            audit.questions.len(),
            question.category
        );
        let progress_widget = Paragraph::new(progress_text)
            .style(Style::default().fg(Color::Green))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(progress_widget, chunks[1]);

        // Question
        let mut question_text = vec![
            Line::from(vec![Span::styled(
                &question.text,
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
        ];
        
        if let Some(desc) = &question.description {
            question_text.push(Line::from(vec![Span::styled(
                desc,
                Style::default().fg(Color::DarkGray),
            )]));
            question_text.push(Line::from(""));
        }

        if let Some(answer) = existing_answer {
            question_text.push(Line::from(vec![
                Span::styled("Réponse: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    format!("{}", answer.answer),
                    Style::default().fg(Color::Green),
                ),
            ]));
        }

        let question_widget = Paragraph::new(question_text)
            .wrap(Wrap { trim: true })
            .block(Block::default().borders(Borders::ALL).title("Question"));
        f.render_widget(question_widget, chunks[2]);

        // Footer avec instructions claires
        let help_text = if existing_answer.is_some() {
            "↑↓: Navigation | o/n/j/a: Répondre | r: Résultats | q: Retour"
        } else {
            "↑↓: Navigation | o: Oui | n: Non | j: Je ne sais pas | a: Non applicable | r: Résultats | q: Retour"
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
                Span::styled("Nom: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    if self.new_audit_name.is_empty() { "_" } else { &self.new_audit_name },
                    if self.input_mode == InputMode::Name {
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    },
                ),
            ]),
            Line::from({
                let mut type_spans = vec![Span::styled("Type: ", Style::default().fg(Color::Yellow))];
                for (i, &type_name) in AUDIT_TYPES.iter().enumerate() {
                    let is_selected = i == self.new_audit_type_index;
                    let is_active = self.input_mode == InputMode::Type;
                    let style = if is_selected && is_active {
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                    } else if is_selected {
                        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
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
                Span::styled("Projet: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    if self.new_project_name.is_empty() { "_" } else { &self.new_project_name },
                    if self.input_mode == InputMode::Project {
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    },
                ),
            ]),
            Line::from(vec![
                Span::styled("Produit: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    if self.new_product_name.is_empty() { "_" } else { &self.new_product_name },
                    if self.input_mode == InputMode::Product {
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
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
            let help = Paragraph::new("↑↓: Scroller les recommandations | q: Retour à la liste | r: Actualiser")
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
        };
        state.load_audits();
        state
    }
}

