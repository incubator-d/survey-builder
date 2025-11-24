//! Application principale d'audit technique et business
//!
//! Cette application permet de créer et gérer des audits avec des questions,
//! de répondre aux questions, et de générer des recommandations basées sur les réponses.
//!
//! # Architecture
//!
//! - `AppState`: Gère l'état de l'application et la logique UI
//! - `AuditManager`: Gère la persistance des audits via le trait `Storage`
//! - `QuestionGenerator`: Génère les questions depuis des fichiers YAML
//! - `RecommendationEngine`: Génère des recommandations basées sur les réponses

// Modules organisés par couches
mod domain;
mod infrastructure;
mod shared;
mod ui;

use std::cell::RefCell;
use std::rc::Rc;

use ratzilla::ratatui::Terminal;
use ratzilla::{event::KeyCode, DomBackend, WebRenderer};

// Imports depuis les modules réorganisés
use ui::{AppState, InputMode, View};

// Variable globale pour stocker l'AppState (accessible depuis JavaScript)
static mut APP_STATE: Option<Rc<RefCell<AppState>>> = None;

#[wasm_bindgen::prelude::wasm_bindgen]
pub fn process_yaml_import(yaml_content: &str) -> Result<(), wasm_bindgen::JsValue> {
    unsafe {
        if let Some(ref app_state) = APP_STATE {
            let mut state = app_state.borrow_mut();
            state.process_yaml_import(yaml_content);
            Ok(())
        } else {
            Err(wasm_bindgen::JsValue::from_str("AppState non initialisé"))
        }
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() -> Result<(), wasm_bindgen::JsValue> {
    let app_state = Rc::new(RefCell::new(AppState::default()));

    // Stocker l'AppState dans la variable globale
    unsafe {
        APP_STATE = Some(app_state.clone());
    }

    let backend = DomBackend::new().map_err(|e| wasm_bindgen::JsValue::from_str(&e.to_string()))?;
    let terminal =
        Terminal::new(backend).map_err(|e| wasm_bindgen::JsValue::from_str(&e.to_string()))?;

    terminal.on_key_event({
        let app_state = app_state.clone();
        move |key_event| {
            let mut state = app_state.borrow_mut();

            // Gestion de la saisie de texte pour les formulaires et le mode commentaire (prioritaire)
            // En mode commentaire, TOUS les caractères sont de la saisie (y compris e, f, q, r)
            // Cette section doit être AVANT les raccourcis globaux pour intercepter tous les caractères
            if (state.view == View::Create && state.input_mode != InputMode::Normal)
                || (state.view == View::Detail && state.input_mode == InputMode::Comment) {
                match key_event.code {
                    // Pour le type, on utilise les flèches pour naviguer
                    KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right
                        if state.input_mode == InputMode::Type => {
                        match key_event.code {
                            KeyCode::Up | KeyCode::Left => {
                                state.handle_up();
                            }
                            KeyCode::Down | KeyCode::Right => {
                                state.handle_down();
                            }
                            _ => {}
                        }
                        return;
                    }
                    KeyCode::Char(c) => {
                        // En mode saisie, TOUS les caractères sont de la saisie
                        state.handle_char_input(c);
                        return;
                    }
                    KeyCode::Backspace => {
                        state.handle_backspace();
                        return;
                    }
                    KeyCode::Enter => {
                        state.handle_enter();
                        return;
                    }
                    KeyCode::Tab => {
                        // Tab ne fonctionne que dans la vue Create
                        if state.view == View::Create {
                            state.handle_tab();
                        }
                        return;
                    }
                    KeyCode::Esc => {
                        // Annuler le mode commentaire (le commentaire en cours est perdu)
                        if state.input_mode == InputMode::Comment {
                            state.answer_comment.clear();
                        }
                        state.input_mode = InputMode::Normal;
                        return;
                    }
                    _ => {
                        // Autres touches spéciales ignorées en mode saisie
                        return;
                    }
                }
            }

            // Gestion des réponses rapides dans la vue Detail (seulement si pas en mode commentaire)
            if state.view == View::Detail && state.input_mode != InputMode::Comment {
                match key_event.code {
                    KeyCode::Char('c') | KeyCode::Char('C') => {
                        // Entrer en mode commentaire
                        // Initialiser avec le commentaire existant s'il y en a un
                        if let Some(audit) = state.get_current_audit() {
                            // Calculer les indices visibles (même logique que get_visible_question_indices)
                            let visible_indices: Vec<usize> = if state.filter_unanswered {
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
                                (0..audit.questions.len()).collect()
                            };

                            if state.current_question_index < visible_indices.len() {
                                let real_idx = visible_indices[state.current_question_index];
                                if let Some(question) = audit.questions.get(real_idx) {
                                    if let Some(answer) = audit.get_answer(&question.id) {
                                        if let Some(ref comment) = answer.comment {
                                            state.answer_comment = comment.clone();
                                        } else {
                                            state.answer_comment.clear();
                                        }
                                    } else {
                                        state.answer_comment.clear();
                                    }
                                }
                            }
                        }
                        state.input_mode = InputMode::Comment;
                        return;
                    }
                    KeyCode::Char('o') | KeyCode::Char('O') => {
                        state.answer_value = "o".to_string();
                        state.save_answer();
                        return;
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') => {
                        state.answer_value = "n".to_string();
                        state.save_answer();
                        return;
                    }
                    KeyCode::Char('j') => {
                        state.answer_value = "j".to_string();
                        state.save_answer();
                        return;
                    }
                    KeyCode::Char('a') => {
                        state.answer_value = "a".to_string();
                        state.save_answer();
                        return;
                    }
                    _ => {}
                }
            }

            // Gestion de la confirmation de suppression (prioritaire)
            if state.pending_delete_audit_id.is_some() && state.view == View::List {
                match key_event.code {
                    KeyCode::Char('o') | KeyCode::Char('O') => {
                        state.confirm_delete();
                        return;
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                        state.cancel_delete();
                        return;
                    }
                    _ => {
                        // Ignorer les autres touches pendant la confirmation
                        return;
                    }
                }
            }

            // Gestion des raccourcis globaux (IGNORÉS en mode commentaire - sauf e, f, q, r déjà gérés)
            if state.input_mode != InputMode::Comment {
                match key_event.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        if state.view == View::Detail || state.view == View::Create {
                            if state.input_mode != InputMode::Normal {
                                state.input_mode = InputMode::Normal;
                            } else {
                                state.view = View::List;
                            }
                        } else if state.view == View::Recommendations {
                            state.view = View::List;
                        }
                    }
                    KeyCode::Char('n') => {
                        if state.view == View::List {
                            state.view = View::Create;
                            state.new_audit_name.clear();
                            state.new_audit_type_index = 0;
                            state.input_mode = InputMode::Name; // Positionner directement sur le premier champ
                        }
                    }
                    KeyCode::Char('r') => {
                        // Accéder aux recommandations depuis Detail ou List
                        // Seulement si toutes les questions sont répondues
                        if state.view == View::Detail || state.view == View::List {
                            if state.is_audit_complete() {
                                state.view = View::Recommendations;
                                state.summary_scroll = 0; // Réinitialiser le scroll
                                state.recommendations_scroll = 0; // Réinitialiser le scroll des recommandations
                            } else {
                                // Afficher un message d'erreur
                                if let Some(audit) = state.get_current_audit_for_recommendations() {
                                    let completion = audit.completion_percentage();
                                    state.error_message = Some(format!(
                                        "Vous devez répondre à toutes les questions avant d'accéder aux résultats (Complétion: {:.1}%)\n",
                                        completion
                                    ));
                                } else {
                                    state.error_message = Some("Aucun audit sélectionné".to_string());
                                }
                            }
                        }
                    }
                    KeyCode::Enter => {
                        state.handle_enter();
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        state.handle_up();
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        state.handle_down();
                    }
                    KeyCode::Left | KeyCode::Char('h') => {
                        state.handle_left();
                    }
                    KeyCode::Right | KeyCode::Char('l') => {
                        state.handle_right();
                    }
                    KeyCode::Char('d') => {
                        state.handle_delete();
                    }
                    KeyCode::Char('e') | KeyCode::Char('E') => {
                        // Exporter l'audit au format TXT
                        if state.view == View::Detail || state.view == View::List || state.view == View::Recommendations {
                            state.export_audit_to_txt();
                        }
                    }
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        // Exporter l'audit vierge au format YAML
                        if state.view == View::Detail || state.view == View::List {
                            state.export_audit_to_yaml();
                        }
                    }
                    KeyCode::Char('i') | KeyCode::Char('I') => {
                        // Importer un audit depuis un fichier YAML
                        if state.view == View::List {
                            state.import_audit_from_yaml();
                        }
                    }
                    KeyCode::Char('f') | KeyCode::Char('F') => {
                        // Activer/désactiver le filtre des questions non répondues
                        if state.view == View::Detail {
                            state.toggle_filter_unanswered();
                        }
                    }
                    KeyCode::Tab => {
                        state.handle_tab();
                    }
                    _ => {}
                }
            }
        }
    });

    terminal.draw_web(move |f| {
        let state = app_state.borrow();
        state.render(f);
    });

    Ok(())
}
