use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::rpcclient::dal::{LoginByEmailRequest, RegisterRequest};
use crate::AppState;

#[derive(Deserialize)]
pub struct RegisterBody {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct RegisterResult {
    pub user_id: i64,
}

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RegisterBody>,
) -> Result<(StatusCode, Json<RegisterResult>), (StatusCode, String)> {
    let mut client = state.dal.clone();
    let response = client
        .register(RegisterRequest {
            username: body.username,
            email: body.email,
            password: body.password,
        })
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let inner = response.into_inner();
    Ok((StatusCode::CREATED, Json(RegisterResult { user_id: inner.user_id })))
}

#[derive(Deserialize)]
pub struct LoginBody {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResult {
    pub user_id: i64,
    pub username: String,
    pub token: String,
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(body): Json<LoginBody>,
) -> Result<Json<LoginResult>, (StatusCode, String)> {
    let mut client = state.dal.clone();
    let response = client
        .login_by_email(LoginByEmailRequest {
            email: body.email,
            password: body.password,
        })
        .await
        .map_err(|e| {
            if e.code() == tonic::Code::Unauthenticated {
                (StatusCode::UNAUTHORIZED, e.to_string())
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            }
        })?;

    let inner = response.into_inner();
    Ok(Json(LoginResult {
        user_id: inner.user_id,
        username: inner.username,
        token: inner.token,
    }))
}