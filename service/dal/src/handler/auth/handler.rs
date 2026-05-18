use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use argon2::{
    Argon2,
    password_hash::{PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use auth_lib::claim::Claim;
use auth_lib::jot::Jot;
use tonic::{Request, Response, Status};
use tracing::info;

use crate::dal::{LoginByEmailRequest, LoginByEmailResponse, RegisterRequest, RegisterResponse};
use crate::db::DbPool;

static JTI_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Clone)]
pub struct AuthHandler {
    db: DbPool,
    jot: Arc<Jot>,
}

impl AuthHandler {
    pub fn new(db: DbPool, jot: Arc<Jot>) -> Self {
        Self { db, jot }
    }

    pub async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        let req = request.into_inner();
        let username = req.username.trim().to_string();
        let email = req.email.trim().to_string();
        let password = req.password;

        if username.is_empty() {
            return Err(Status::invalid_argument("username is required"));
        }
        if email.is_empty() {
            return Err(Status::invalid_argument("email is required"));
        }
        if !email.contains('@') {
            return Err(Status::invalid_argument("invalid email format"));
        }
        if password.len() < 6 {
            return Err(Status::invalid_argument("password must be at least 6 characters"));
        }

        let salt = SaltString::generate(&mut OsRng);
        let hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| Status::internal(e.to_string()))?
            .to_string();

        let user_id = self
            .db
            .create_user(&username, &email, &hash)
            .await
            .map_err(|e| Status::already_exists(e))?;

        info!("User registered: {} <{}> (id={})", username, email, user_id);

        Ok(Response::new(RegisterResponse { user_id }))
    }

    pub async fn login_by_email(
        &self,
        request: Request<LoginByEmailRequest>,
    ) -> Result<Response<LoginByEmailResponse>, Status> {
        let req = request.into_inner();
        let email = req.email.trim().to_string();
        let password = req.password;

        let user = self
            .db
            .find_user_by_email(&email)
            .await
            .map_err(|e| Status::internal(e))?
            .ok_or_else(|| Status::unauthenticated("user not found"))?;

        let parsed_hash = argon2::PasswordHash::new(&user.password_hash)
            .map_err(|e| Status::internal(e.to_string()))?;
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| Status::unauthenticated("invalid password"))?;

        let token = self.generate_token(user.id, &user.username)?;

        info!("User logged in: {} <{}> (id={})", user.username, email, user.id);

        Ok(Response::new(LoginByEmailResponse {
            user_id: user.id,
            username: user.username,
            token,
        }))
    }

    fn generate_token(&self, user_id: i64, username: &str) -> Result<String, Status> {
        let mut claim = Claim::new(
            user_id as u64,
            username.to_string(),
            JTI_COUNTER.fetch_add(1, Ordering::Relaxed),
        );
        claim.set_issuer("dal");
        claim.set_expiration_time(86400);
        self.jot.encode(&claim).map_err(|e| Status::internal(e.to_string()))
    }
}