use std::sync::Arc;

use axum::{
    extract::State,
    http::{header, StatusCode, HeaderValue, HeaderMap},
    Json,
};
use axum_extra::extract::cookie::Cookie;
use auth_lib::AuthClaim;
use serde::{Deserialize, Serialize};

use crate::rpcclient::dal::{LoginByEmailRequest, RegisterRequest};
use crate::AppJotProvider;
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

#[derive(Serialize)]
pub struct RegisterError {
    pub error: String,
}

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RegisterBody>,
) -> Result<(StatusCode, Json<RegisterResult>), (StatusCode, Json<RegisterError>)> {
    let mut client = state.dal.clone();
    let response = client
        .register(RegisterRequest {
            username: body.username,
            email: body.email,
            password: body.password,
        })
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(RegisterError { error: e.to_string() })))?;

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

#[derive(Serialize)]
pub struct LoginError {
    pub error: String,
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(body): Json<LoginBody>,
) -> Result<(StatusCode, HeaderMap, Json<LoginResult>), (StatusCode, Json<LoginError>)> {
    let mut client = state.dal.clone();
    let response = client
        .login_by_email(LoginByEmailRequest {
            email: body.email,
            password: body.password,
        })
        .await
        .map_err(|e| {
            if e.code() == tonic::Code::Unauthenticated {
                (StatusCode::UNAUTHORIZED, Json(LoginError { error: "用户名或密码错误".into() }))
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(LoginError { error: e.to_string() }))
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

pub async fn logout() -> (StatusCode, HeaderMap) {
    let cookie = Cookie::build((TOKEN_COOKIE_NAME, ""))
        .http_only(true)
        .same_site(axum_extra::extract::cookie::SameSite::Lax)
        .path("/")
        .max_age(time::Duration::ZERO)
        .build();

    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie.to_string()).unwrap(),
    );

    (StatusCode::OK, headers)
}

#[derive(Serialize)]
pub struct MeResult {
    pub user_id: u64,
    pub username: String,
}

pub async fn me(
    AuthClaim(claim, _): AuthClaim<AppJotProvider>,
) -> Result<Json<MeResult>, (StatusCode, String)> {
    Ok(Json(MeResult {
        user_id: claim.get_subject(),
        username: claim.get_name().to_owned(),
    }))
}
