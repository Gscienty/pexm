use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use base64::Engine;

use crate::AppState;

pub async fn get_paper_content(
    Path(id): Path<i64>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let room_id = id as u64;

    // 1. Check real_time in-memory (active room or pending content).
    if let Some(content) = state.room_set.get_content(room_id).await {
        let b64 = base64::engine::general_purpose::STANDARD.encode(&content);
        return (StatusCode::OK, axum::Json(serde_json::json!({ "content": b64 }))).into_response();
    }

    // 2. Not in real_time — return 404 so the caller falls back to DAL.
    (StatusCode::NOT_FOUND, axum::Json(serde_json::json!({ "content": "" }))).into_response()
}
