use crate::audit::{AuditManager, QuestionGenerator};
use crate::models::*;
use crate::recommendations::RecommendationEngine;
use actix_web::{web, HttpResponse, Result as ActixResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
        }
    }
}

#[derive(Deserialize)]
pub struct CreateAuditRequest {
    pub name: String,
    pub audit_type: String,
    pub project_name: Option<String>,
    pub product_name: Option<String>,
}

#[derive(Deserialize)]
pub struct AnswerQuestionRequest {
    pub answer: String,
    pub percentage: Option<u8>,
    pub comment: Option<String>,
}

pub struct AppState {
    pub audit_manager: Arc<AuditManager>,
}

/// Liste tous les audits
pub async fn list_audits(state: web::Data<AppState>) -> ActixResult<HttpResponse> {
    match state.audit_manager.list_audits() {
        Ok(audit_ids) => {
            let mut audits = Vec::new();
            for id in audit_ids {
                if let Ok(audit) = state.audit_manager.load_audit(&id) {
                    audits.push(audit);
                }
            }
            Ok(HttpResponse::Ok().json(ApiResponse::ok(audits)))
        }
        Err(e) => Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::err(
            e.to_string(),
        ))),
    }
}

/// Crée un nouvel audit
pub async fn create_audit(
    state: web::Data<AppState>,
    req: web::Json<CreateAuditRequest>,
) -> ActixResult<HttpResponse> {
    let audit_type = match req.audit_type.to_lowercase().as_str() {
        "technique" => AuditType::Technique,
        "business" => AuditType::Business,
        "marketing" => AuditType::Marketing,
        other => AuditType::Autre(other.to_string()),
    };

    let mut audit = Audit::new(req.name.clone(), audit_type.clone());
    audit.project_name = req.project_name.clone();
    audit.product_name = req.product_name.clone();

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
        AuditType::Marketing => {
            let questions = QuestionGenerator::generate_marketing_questions();
            for q in questions {
                audit.add_question(q);
            }
        }
        _ => {}
    }

    match state.audit_manager.save_audit(&audit) {
        Ok(_) => Ok(HttpResponse::Ok().json(ApiResponse::ok(audit))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::err(
            e.to_string(),
        ))),
    }
}

/// Récupère un audit par son ID
pub async fn get_audit(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> ActixResult<HttpResponse> {
    let audit_id = match Uuid::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(e) => {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::err(
                format!("ID d'audit invalide: {}", e),
            )));
        }
    };

    match state.audit_manager.load_audit(&audit_id) {
        Ok(audit) => Ok(HttpResponse::Ok().json(ApiResponse::ok(audit))),
        Err(e) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::err(
            e.to_string(),
        ))),
    }
}

/// Répond à une question
pub async fn answer_question(
    state: web::Data<AppState>,
    path: web::Path<(String, String)>,
    req: web::Json<AnswerQuestionRequest>,
) -> ActixResult<HttpResponse> {
    let (audit_id_str, question_id_str) = path.into_inner();

    let audit_id = match Uuid::parse_str(&audit_id_str) {
        Ok(id) => id,
        Err(e) => {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::err(
                format!("ID d'audit invalide: {}", e),
            )));
        }
    };

    let question_id = match Uuid::parse_str(&question_id_str) {
        Ok(id) => id,
        Err(e) => {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::err(
                format!("ID de question invalide: {}", e),
            )));
        }
    };

    let answer = match req.answer.to_lowercase().as_str() {
        "oui" | "o" | "yes" | "y" => Answer::Oui,
        "non" | "n" | "no" => Answer::Non,
        "je-ne-sais-pas" | "jsp" | "dont-know" | "?" => Answer::JeNeSaisPas,
        "non-applicable" | "na" | "n/a" => Answer::NonApplicable,
        _ => {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::err(
                "Réponse invalide".to_string(),
            )));
        }
    };

    let mut audit = match state.audit_manager.load_audit(&audit_id) {
        Ok(a) => a,
        Err(e) => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::<()>::err(
                e.to_string(),
            )));
        }
    };

    let question_answer = QuestionAnswer {
        question_id,
        answer,
        percentage: req.percentage,
        comment: req.comment.clone(),
        answered_at: chrono::Utc::now(),
    };

    audit.answer_question(question_answer);

    match state.audit_manager.save_audit(&audit) {
        Ok(_) => Ok(HttpResponse::Ok().json(ApiResponse::ok(audit))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::err(
            e.to_string(),
        ))),
    }
}

/// Génère les recommandations pour un audit
pub async fn get_recommendations(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> ActixResult<HttpResponse> {
    let audit_id = match Uuid::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(e) => {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::err(
                format!("ID d'audit invalide: {}", e),
            )));
        }
    };

    let audit = match state.audit_manager.load_audit(&audit_id) {
        Ok(a) => a,
        Err(e) => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::<()>::err(
                e.to_string(),
            )));
        }
    };

    let recommendations = RecommendationEngine::generate_recommendations(&audit);
    Ok(HttpResponse::Ok().json(ApiResponse::ok(recommendations)))
}

/// Récupère le résumé d'un audit
pub async fn get_summary(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> ActixResult<HttpResponse> {
    let audit_id = match Uuid::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(e) => {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::err(
                format!("ID d'audit invalide: {}", e),
            )));
        }
    };

    let audit = match state.audit_manager.load_audit(&audit_id) {
        Ok(a) => a,
        Err(e) => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::<()>::err(
                e.to_string(),
            )));
        }
    };

    let summary = RecommendationEngine::generate_summary(&audit);
    Ok(HttpResponse::Ok().json(ApiResponse::ok(summary)))
}

/// Supprime un audit
pub async fn delete_audit(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> ActixResult<HttpResponse> {
    let audit_id = match Uuid::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(e) => {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::err(
                format!("ID d'audit invalide: {}", e),
            )));
        }
    };

    match state.audit_manager.delete_audit(&audit_id) {
        Ok(_) => Ok(HttpResponse::Ok().json(ApiResponse::ok(()) as ApiResponse<()>)),
        Err(e) => Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::err(
            e.to_string(),
        ))),
    }
}

pub fn config_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/audits", web::get().to(list_audits))
            .route("/audits", web::post().to(create_audit))
            .route("/audits/{id}", web::get().to(get_audit))
            .route("/audits/{id}", web::delete().to(delete_audit))
            .route("/audits/{id}/questions/{question_id}/answer", web::post().to(answer_question))
            .route("/audits/{id}/recommendations", web::get().to(get_recommendations))
            .route("/audits/{id}/summary", web::get().to(get_summary)),
    );
}

pub async fn run_server(port: u16) -> anyhow::Result<()> {
    use actix_web::{App, HttpServer};
    use actix_files::Files;

    let audit_manager = Arc::new(AuditManager::new("./audits"));
    let app_state = web::Data::new(AppState { audit_manager });

    println!("🚀 Serveur web démarré sur http://localhost:{}", port);

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .configure(config_routes)
            .service(Files::new("/", "./static").index_file("index.html"))
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await?;

    Ok(())
}

