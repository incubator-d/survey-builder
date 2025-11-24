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

use ratzilla::{event::KeyCode, DomBackend, WebRenderer};
use ratzilla::ratatui::Terminal;

// Imports depuis les modules réorganisés
use ui::{AppState, InputMode, View};

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() -> Result<(), wasm_bindgen::JsValue> {
    let app_state = Rc::new(RefCell::new(AppState::default()));
    let backend = DomBackend::new().map_err(|e| wasm_bindgen::JsValue::from_str(&e.to_string()))?;
    let terminal = Terminal::new(backend).map_err(|e| wasm_bindgen::JsValue::from_str(&e.to_string()))?;

    terminal.on_key_event({
        let app_state = app_state.clone();
        move |key_event| {
            let mut state = app_state.borrow_mut();
            
            // Gestion de la saisie de texte pour les formulaires (prioritaire)
            if state.view == View::Create && state.input_mode != InputMode::Normal {
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
                        state.handle_tab();
                        return;
                    }
                    KeyCode::Esc => {
                        state.input_mode = InputMode::Normal;
                        return;
                    }
                    _ => {
                        // Autres touches spéciales ignorées en mode saisie
                        return;
                    }
                }
            }
            
            // Gestion des réponses rapides dans la vue Detail
            if state.view == View::Detail {
                match key_event.code {
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
            
            // Gestion des raccourcis globaux (seulement si pas en mode saisie)
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
                        state.input_mode = InputMode::Normal;
                    }
                }
                KeyCode::Char('r') => {
                    // Accéder aux recommandations depuis Detail ou List
                    if state.view == View::Detail || state.view == View::List {
                        state.view = View::Recommendations;
                        state.summary_scroll = 0; // Réinitialiser le scroll
                        state.recommendations_scroll = 0; // Réinitialiser le scroll des recommandations
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
                KeyCode::Tab => {
                    state.handle_tab();
                }
                _ => {}
            }
        }
    });

    terminal.draw_web(move |f| {
        let state = app_state.borrow();
        state.render(f);
    });

    Ok(())
}

