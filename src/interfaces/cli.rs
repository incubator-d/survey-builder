use crate::audit::{AuditManager, QuestionGenerator};
use crate::models::*;
use crate::recommendations::RecommendationEngine;
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::io::{self, Write};
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "survey-audit")]
#[command(about = "Outil d'audit technique et business pour projets", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Crée un nouvel audit
    Create {
        /// Nom de l'audit
        name: String,
        /// Type d'audit (technique, business, marketing)
        #[arg(short, long, default_value = "technique")]
        r#type: String,
    },
    /// Liste tous les audits
    List,
    /// Affiche les détails d'un audit
    Show {
        /// ID de l'audit
        audit_id: String,
    },
    /// Répond à une question d'un audit
    Answer {
        /// ID de l'audit
        audit_id: String,
        /// ID de la question
        question_id: String,
        /// Réponse (oui, non, je-ne-sais-pas, non-applicable)
        answer: String,
        /// Pourcentage (0-100) si nécessaire
        #[arg(short, long)]
        percentage: Option<u8>,
        /// Commentaire
        #[arg(short, long)]
        comment: Option<String>,
    },
    /// Génère les recommandations pour un audit
    Recommend {
        /// ID de l'audit
        audit_id: String,
    },
    /// Affiche le résumé d'un audit
    Summary {
        /// ID de l'audit
        audit_id: String,
    },
    /// Mode interactif pour répondre aux questions
    Interactive {
        /// ID de l'audit
        audit_id: String,
    },
    /// Supprime un audit
    Delete {
        /// ID de l'audit
        audit_id: String,
    },
}

pub async fn run() -> Result<()> {
    let cli = Cli::parse();
    let manager = AuditManager::new("./audits");

    match cli.command {
        Commands::Create { name, r#type } => {
            let audit_type = match r#type.to_lowercase().as_str() {
                "technique" => AuditType::Technique,
                "business" => AuditType::Business,
                "marketing" => AuditType::Marketing,
                other => AuditType::Autre(other.to_string()),
            };

            let mut audit = Audit::new(name, audit_type.clone());
            
            // Ajoute les questions par défaut selon le type
            match audit_type {
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
                    println!("Aucune question par défaut pour ce type d'audit. Vous pouvez les ajouter manuellement.");
                }
            }

            manager.save_audit(&audit)?;
            println!("✅ Audit créé avec succès!");
            println!("ID: {}", audit.id);
            println!("Type: {}", audit.audit_type);
            println!("Questions: {}", audit.questions.len());
        }

        Commands::List => {
            let audits = manager.list_audits()?;
            if audits.is_empty() {
                println!("Aucun audit trouvé.");
            } else {
                println!("Audits disponibles:");
                for audit_id in audits {
                    match manager.load_audit(&audit_id) {
                        Ok(audit) => {
                            let completion = audit.completion_percentage();
                            println!(
                                "  {} - {} ({}) - {:.1}% complété",
                                audit.id, audit.name, audit.audit_type, completion
                            );
                        }
                        Err(e) => {
                            eprintln!("  {} - Erreur: {}", audit_id, e);
                        }
                    }
                }
            }
        }

        Commands::Show { audit_id } => {
            let uuid = Uuid::parse_str(&audit_id)
                .context("ID d'audit invalide")?;
            let audit = manager.load_audit(&uuid)?;
            
            println!("📋 Audit: {}", audit.name);
            println!("Type: {}", audit.audit_type);
            println!("Créé le: {}", audit.created_at.format("%Y-%m-%d %H:%M:%S"));
            println!("Complétion: {:.1}%", audit.completion_percentage());
            println!("\nQuestions:");
            
            // Groupe par catégorie
            let mut categories: Vec<_> = audit.questions.iter()
                .map(|q| &q.category)
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();
            categories.sort_by_key(|c| format!("{}", c));

            for category in categories {
                println!("\n## {}", category);
                for question in audit.questions_by_category(category) {
                    let status = if audit.is_answered(&question.id) {
                        if let Some(answer) = audit.get_answer(&question.id) {
                            format!("✅ {}", answer.answer)
                        } else {
                            "❓".to_string()
                        }
                    } else {
                        "⏳".to_string()
                    };
                    println!("  {} [{}] {}", status, question.id, question.text);
                    if let Some(desc) = &question.description {
                        println!("     {}", desc);
                    }
                }
            }
        }

        Commands::Answer {
            audit_id,
            question_id,
            answer,
            percentage,
            comment,
        } => {
            let audit_uuid = Uuid::parse_str(&audit_id)
                .context("ID d'audit invalide")?;
            let question_uuid = Uuid::parse_str(&question_id)
                .context("ID de question invalide")?;

            let mut audit = manager.load_audit(&audit_uuid)?;
            
            let parsed_answer = match answer.to_lowercase().as_str() {
                "oui" | "o" | "yes" | "y" => Answer::Oui,
                "non" | "n" | "no" => Answer::Non,
                "je-ne-sais-pas" | "jsp" | "dont-know" | "?" => Answer::JeNeSaisPas,
                "non-applicable" | "na" | "n/a" => Answer::NonApplicable,
                _ => return Err(anyhow::anyhow!("Réponse invalide. Utilisez: oui, non, je-ne-sais-pas, non-applicable")),
            };

            let question_answer = QuestionAnswer {
                question_id: question_uuid,
                answer: parsed_answer,
                percentage,
                comment,
                answered_at: chrono::Utc::now(),
            };

            audit.answer_question(question_answer);
            manager.save_audit(&audit)?;
            println!("✅ Réponse enregistrée!");
        }

        Commands::Recommend { audit_id } => {
            let uuid = Uuid::parse_str(&audit_id)
                .context("ID d'audit invalide")?;
            let audit = manager.load_audit(&uuid)?;
            
            let recommendations = RecommendationEngine::generate_recommendations(&audit);
            
            if recommendations.is_empty() {
                println!("✅ Aucune recommandation. L'audit semble bon!");
            } else {
                println!("📊 Recommandations ({}):\n", recommendations.len());
                
                // Trie par priorité
                let mut sorted = recommendations;
                sorted.sort_by(|a, b| b.priority.cmp(&a.priority));
                
                for rec in sorted {
                    let emoji = match rec.priority {
                        Priority::Critique => "🔴",
                        Priority::Elevee => "🟠",
                        Priority::Moyenne => "🟡",
                        Priority::Faible => "🟢",
                    };
                    println!("{} [{}] {}", emoji, rec.priority, rec.title);
                    println!("   {}", rec.description);
                    println!("   Catégorie: {}\n", rec.category);
                }
            }
        }

        Commands::Summary { audit_id } => {
            let uuid = Uuid::parse_str(&audit_id)
                .context("ID d'audit invalide")?;
            let audit = manager.load_audit(&uuid)?;
            
            let summary = RecommendationEngine::generate_summary(&audit);
            
            println!("📊 Résumé de l'audit");
            println!("Complétion: {:.1}%", summary.completion_percentage);
            println!("Questions totales: {}", summary.total_questions);
            println!("Questions répondues: {}", summary.total_answered);
            println!("\nPar catégorie:");
            
            for stats in summary.category_stats {
                println!("\n{}", stats.category);
                println!("  Questions: {}/{} ({:.1}% complété)", 
                    stats.answered, stats.total_questions, stats.completion_percentage());
                println!("  Réponses positives: {} ({:.1}%)", 
                    stats.positive_answers, stats.positive_percentage());
            }
        }

        Commands::Interactive { audit_id } => {
            let uuid = Uuid::parse_str(&audit_id)
                .context("ID d'audit invalide")?;
            let mut audit = manager.load_audit(&uuid)?;
            
            println!("🎯 Mode interactif pour l'audit: {}\n", audit.name);
            
            // Collecte les catégories et questions d'abord pour éviter les problèmes d'emprunt
            let mut categories: Vec<_> = audit.questions.iter()
                .map(|q| q.category.clone())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();
            categories.sort_by_key(|c| format!("{}", c));

            // Collecte les IDs de questions par catégorie
            let mut questions_by_cat: std::collections::HashMap<Category, Vec<Uuid>> = std::collections::HashMap::new();
            for question in &audit.questions {
                questions_by_cat
                    .entry(question.category.clone())
                    .or_insert_with(Vec::new)
                    .push(question.id);
            }

            for category in categories {
                println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!("📁 Catégorie: {}", category);
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

                if let Some(question_ids) = questions_by_cat.get(&category) {
                    for question_id in question_ids {
                        // Trouve la question correspondante
                        let question = audit.questions.iter()
                            .find(|q| q.id == *question_id)
                            .expect("Question introuvable");

                        if audit.is_answered(question_id) {
                            if let Some(answer) = audit.get_answer(question_id) {
                                println!("✅ [Déjà répondu: {}] {}", answer.answer, question.text);
                                continue;
                            }
                        }

                        println!("❓ {}", question.text);
                        if let Some(desc) = &question.description {
                            println!("   {}", desc);
                        }
                        if !question.links.is_empty() {
                            println!("   Liens:");
                            for link in &question.links {
                                println!("     - {}", link);
                            }
                        }

                        print!("\nRéponse (oui/non/je-ne-sais-pas/non-applicable): ");
                        io::stdout().flush()?;
                        let mut input = String::new();
                        io::stdin().read_line(&mut input)?;
                        let answer_str = input.trim().to_lowercase();

                        let answer = match answer_str.as_str() {
                            "oui" | "o" | "yes" | "y" => Answer::Oui,
                            "non" | "n" | "no" => Answer::Non,
                            "je-ne-sais-pas" | "jsp" | "dont-know" | "?" => Answer::JeNeSaisPas,
                            "non-applicable" | "na" | "n/a" => Answer::NonApplicable,
                            _ => {
                                println!("⚠️  Réponse invalide, passage à la question suivante.");
                                continue;
                            }
                        };

                        let mut percentage = None;
                        if question.requires_percentage {
                            print!("Pourcentage (0-100): ");
                            io::stdout().flush()?;
                            let mut pct_input = String::new();
                            io::stdin().read_line(&mut pct_input)?;
                            if let Ok(pct) = pct_input.trim().parse::<u8>() {
                                percentage = Some(pct);
                            }
                        }

                        print!("Commentaire (optionnel, Entrée pour passer): ");
                        io::stdout().flush()?;
                        let mut comment_input = String::new();
                        io::stdin().read_line(&mut comment_input)?;
                        let comment = comment_input.trim();
                        let comment = if comment.is_empty() { None } else { Some(comment.to_string()) };

                        let question_answer = QuestionAnswer {
                            question_id: *question_id,
                            answer,
                            percentage,
                            comment,
                            answered_at: chrono::Utc::now(),
                        };

                        audit.answer_question(question_answer);
                        println!("✅ Réponse enregistrée!\n");
                    }
                }
            }

            manager.save_audit(&audit)?;
            println!("\n🎉 Audit complété! Complétion: {:.1}%", audit.completion_percentage());
        }

        Commands::Delete { audit_id } => {
            let uuid = Uuid::parse_str(&audit_id)
                .context("ID d'audit invalide")?;
            manager.delete_audit(&uuid)?;
            println!("✅ Audit supprimé!");
        }
    }

    Ok(())
}

