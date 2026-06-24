pub mod auth;
pub mod compile;
pub mod exam_space;
pub mod graphql;
pub mod paper;

use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};

use crate::AppState;

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/me", get(auth::me))
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/exam-spaces", post(exam_space::create_exam_space))
        .route("/api/papers/{id}/content", get(paper::load_paper_content))
        .route("/api/papers/{id}/compile-pdf", post(compile::compile_pdf))
        .route("/api/graphql", post(graphql::graphql_handler))
        .with_state(state)
}

async fn health() -> &'static str {
    "ok"
}