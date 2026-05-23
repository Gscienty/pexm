use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};
use auth_lib::AuthClaim;
use serde::{Deserialize, Serialize};

use crate::rpcclient::dal::CreateExamSpaceRequest;
use crate::{AppJotProvider, AppState};

#[derive(Deserialize)]
pub struct CreateSpaceBody {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Serialize)]
pub struct CreateSpaceResult {
    pub id: i64,
    pub token: String,
}

pub async fn create_exam_space(
    State(state): State<Arc<AppState>>,
    AuthClaim(claim, _): AuthClaim<AppJotProvider>,
    Json(body): Json<CreateSpaceBody>,
) -> Result<(StatusCode, Json<CreateSpaceResult>), (StatusCode, String)> {
    let mut client = state.dal.clone();

    let response = client
        .create_exam_space(CreateExamSpaceRequest {
            name: body.name,
            description: body.description.unwrap_or_default(),
            user_id: claim.get_subject() as i64,
        })
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let inner = response.into_inner();
    Ok((StatusCode::CREATED, Json(CreateSpaceResult { id: inner.id, token: inner.token })))
}