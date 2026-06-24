use std::sync::Arc;

use axum::{extract::{Path, State}, http::StatusCode, Json};
use auth_lib::AuthClaim;
use base64::Engine;
use serde::Serialize;

use crate::rpcclient::dal::LoadPaperContentRequest;
use crate::{AppJotProvider, AppState};

#[derive(Serialize)]
pub struct LoadPaperContentResult {
    pub content: String, // base64 encoded
}

pub async fn load_paper_content(
    State(state): State<Arc<AppState>>,
    AuthClaim(_claim, _): AuthClaim<AppJotProvider>,
    Path(id): Path<i64>,
) -> Result<Json<LoadPaperContentResult>, (StatusCode, String)> {
    // 1. Check real_time service first — it holds the most up-to-date content
    //    in memory. This prevents stale reads when the user refreshes before
    //    the periodic persist (every 10s) has flushed to DAL.
    if let Some(b64) = fetch_realtime_content(&state, id).await {
        return Ok(Json(LoadPaperContentResult { content: b64 }));
    }

    // 2. Fall back to DAL.
    let mut client = state.dal.clone();

    let response = client
        .load_paper_content(LoadPaperContentRequest { id })
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let content = response.into_inner().content;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&content);

    Ok(Json(LoadPaperContentResult { content: b64 }))
}

/// Query the real_time service for the current document content.
/// Returns Some(base64_content) if the room is active or has pending content,
/// Returns None if real_time doesn't have the content (room not active).
async fn fetch_realtime_content(_state: &AppState, paper_id: i64) -> Option<String> {
    let realtime_addr = std::env::var("REALTIME_ADDR")
        .unwrap_or_else(|_| "http://127.0.0.1:3001".to_string());

    let url = format!("{}/api/doc/{}/content", realtime_addr, paper_id);

    let res = reqwest::Client::new()
        .get(&url)
        .timeout(std::time::Duration::from_secs(2))
        .send()
        .await
        .ok()?;

    if res.status().is_success() {
        let data: serde_json::Value = res.json().await.ok()?;
        let content = data.get("content")?.as_str()?;
        if !content.is_empty() {
            return Some(content.to_string());
        }
    }

    None
}
