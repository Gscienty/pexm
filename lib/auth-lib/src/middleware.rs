use std::sync::Arc;

use axum::{
    extract::FromRequestParts,
    http::request::Parts,
};
use axum_extra::extract::cookie::CookieJar;

use crate::{claim::Claim, jot::Jot};

const TOKEN_COOKIE_NAME: &str = "token";

/// Wrapper for Arc<Jot>.
/// Each service should implement `FromRequestParts<S> for JotProvider`
/// via a local newtype, then use `AuthClaim<LocalJotProvider>`.
pub struct JotProvider(pub Arc<Jot>);

impl std::ops::Deref for JotProvider {
    type Target = Jot;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// 从请求中提取已认证的 Claim，作为 axum extractor 使用。
///
/// `P` 是 Jot 的提供者类型，需要实现 `FromRequestParts<S>`
/// 并返回能 deref 到 `Jot` 的类型。
///
/// 用法：
/// ```rust
/// // 在服务中定义 local newtype
/// pub struct AppJotProvider(auth_lib::JotProvider);
/// impl FromRequestParts<Arc<AppState>> for AppJotProvider { ... }
/// impl std::ops::Deref for AppJotProvider { type Target = Jot; ... }
///
/// // 在 handler 中使用
/// async fn handler(claim: AuthClaim<AppJotProvider>) -> impl IntoResponse {
///     let user_id = claim.0.get_subject();
/// }
/// ```
pub struct AuthClaim<P>(pub Claim, pub std::marker::PhantomData<P>);

impl<S, P> FromRequestParts<S> for AuthClaim<P>
where
    S: Send + Sync,
    P: FromRequestParts<S>,
    P: std::ops::Deref<Target = Jot>,
{
    type Rejection = axum::http::StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_request_parts(parts, state)
            .await
            .map_err(|_| axum::http::StatusCode::UNAUTHORIZED)?;

        let token = jar
            .get(TOKEN_COOKIE_NAME)
            .map(|c| c.value().to_owned())
            .ok_or(axum::http::StatusCode::UNAUTHORIZED)?;

        let jot = P::from_request_parts(parts, state)
            .await
            .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

        let claim = jot
            .decode(&token)
            .map_err(|e| {
                tracing::warn!("Auth failed: {:?}", e);
                axum::http::StatusCode::UNAUTHORIZED
            })?;

        Ok(AuthClaim(claim, std::marker::PhantomData))
    }
}
