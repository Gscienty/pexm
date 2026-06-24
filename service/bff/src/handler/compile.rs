use std::sync::Arc;

use axum::{extract::State, http::StatusCode, body::Body, response::Response};
use axum::extract::Path;
use auth_lib::AuthClaim;
use base64::Engine;

use crate::rpcclient::clsi::CompilePdfRequest;
use crate::{AppJotProvider, AppState};

/// POST /api/papers/:token/compile-pdf
///
/// Accepts a paper token, fetches the yjs content from real_time or DAL,
/// sends it to the clsi service for latex compilation, and returns the PDF binary.
pub async fn compile_pdf(
    State(state): State<Arc<AppState>>,
    AuthClaim(_claim, _): AuthClaim<AppJotProvider>,
    Path(token): Path<String>,
) -> Result<Response, (StatusCode, String)> {
    let paper_id: i64 = token.parse()
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid paper token".to_string()))?;

    // 1. Fetch yjs content from real_time or DAL.
    let yjs_content = fetch_paper_yjs_content(&state, paper_id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    if yjs_content.is_empty() {
        return Err((StatusCode::NOT_FOUND, "Paper content not found".to_string()));
    }

    // 2. Call clsi service.
    let mut client = state.clsi.clone();
    let response = client
        .compile_pdf(CompilePdfRequest { yjs_content })
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("clsi error: {}", e)))?;

    let result = response.into_inner();

    if !result.success {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, result.error));
    }

    // 3. Return PDF binary.
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/pdf")
        .body(Body::from(result.pdf_data))
        .unwrap())
}

/// Fetch the yjs binary content for a paper.
/// Tries real_time first, then falls back to DAL.
async fn fetch_paper_yjs_content(state: &AppState, paper_id: i64) -> Result<Vec<u8>, String> {
    // Try real_time first.
    if let Some(content) = fetch_realtime_content(paper_id).await {
        return Ok(content);
    }

    // Fall back to DAL.
    let mut client = state.dal.clone();
    let response = client
        .load_paper_content(crate::rpcclient::dal::LoadPaperContentRequest { id: paper_id })
        .await
        .map_err(|e| format!("DAL error: {}", e))?;

    Ok(response.into_inner().content)
}

async fn fetch_realtime_content(paper_id: i64) -> Option<Vec<u8>> {
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
        let content_b64 = data.get("content")?.as_str()?;
        if !content_b64.is_empty() {
            return base64::engine::general_purpose::STANDARD
                .decode(content_b64)
                .ok();
        }
    }

    None
}
