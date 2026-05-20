use std::sync::Arc;

use axum::{
    extract::State,
    http::{header, StatusCode, HeaderValue, HeaderMap},
    Json,
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use serde::{Deserialize, Serialize};

use crate::rpcclient::dal::{LoginByEmailRequest, RegisterRequest};
use crate::AppState;

const TOKEN_COOKIE_NAME: &str = "token";

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
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(body): Json<LoginBody>,
) -> Result<(StatusCode, HeaderMap, Json<LoginResult>), (StatusCode, String)> {
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

    let cookie = Cookie::build((TOKEN_COOKIE_NAME, inner.token.clone()))
        .http_only(true)
        .same_site(axum_extra::extract::cookie::SameSite::Lax)
        .path("/")
        .max_age(time::Duration::hours(24))
        .build();

    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie.to_string()).unwrap(),
    );

    Ok((
        StatusCode::OK,
        headers,
        Json(LoginResult {
            user_id: inner.user_id,
            username: inner.username,
        }),
    ))
}

#[derive(Serialize)]
pub struct MeResult {
    pub user_id: u64,
    pub username: String,
}

pub async fn me(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> Result<Json<MeResult>, (StatusCode, String)> {
    let token = jar
        .get(TOKEN_COOKIE_NAME)
        .map(|c| c.value().to_owned())
        .ok_or((StatusCode::UNAUTHORIZED, "未登录".to_string()))?;

    let claim = state
        .jot
        .decode(&token)
        .map_err(|e| {
            tracing::error!("Token decode error: {:?}", e);
            (StatusCode::UNAUTHORIZED, format!("token 无效或已过期: {}", e))
        })?;

    Ok(Json(MeResult {
        user_id: claim.get_subject(),
        username: claim.get_name().to_owned(),
    }))
}
