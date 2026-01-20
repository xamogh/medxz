use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::{Algorithm, Argon2, Params, Version};
use axum::extract::rejection::JsonRejection;
use axum::extract::State;
use axum::http::header::AUTHORIZATION;
use axum::http::HeaderMap;
use axum::Json;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use thiserror::Error;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::ApiError;
use crate::state::AppState;

const ARGON2_PARAMS: Params = match Params::new(8192, 2, 1, None) {
    Ok(params) => params,
    Err(_) => Params::DEFAULT,
};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub organization_code: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct OrganizationInfo {
    pub id: Uuid,
    pub code: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub email: String,
    pub role: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub session_token: String,
    pub organization: OrganizationInfo,
    pub user: UserInfo,
}

#[derive(Debug, Serialize)]
pub struct MeResponse {
    pub organization: OrganizationInfo,
    pub user: UserInfo,
}

#[derive(Debug, Serialize)]
pub struct LogoutResponse {
    pub ok: bool,
}

#[derive(Debug, Error)]
pub enum PasswordHashError {
    #[error("failed to hash password: {message}")]
    Hash { message: String },
    #[error("invalid password hash format: {message}")]
    InvalidHashFormat { message: String },
}

pub fn hash_password(password: &str) -> Result<String, PasswordHashError> {
    let salt = SaltString::generate(&mut rand::thread_rng());
    let argon2 = argon2();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| PasswordHashError::Hash {
            message: e.to_string(),
        })?;
    Ok(hash.to_string())
}

fn verify_password(password_hash: &str, password: &str) -> Result<bool, PasswordHashError> {
    let argon2 = argon2();
    let parsed_hash =
        PasswordHash::new(password_hash).map_err(|e| PasswordHashError::InvalidHashFormat {
            message: e.to_string(),
        })?;

    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

fn argon2() -> Argon2<'static> {
    Argon2::new(Algorithm::Argon2id, Version::V0x13, ARGON2_PARAMS)
}

pub async fn login(
    State(state): State<AppState>,
    payload: Result<Json<LoginRequest>, JsonRejection>,
) -> Result<Json<LoginResponse>, ApiError> {
    let Json(req) = payload?;

    let organization_code = req.organization_code.trim().to_string();
    if organization_code.is_empty() {
        return Err(ApiError::bad_request("organization_code is required"));
    }

    let email = normalize_email(&req.email)?;
    let password = req.password;
    if password.is_empty() {
        return Err(ApiError::bad_request("password is required"));
    }

    let organization = sqlx::query_as::<_, OrganizationRow>(
        "SELECT id, code, name FROM organizations WHERE code = $1",
    )
    .bind(&organization_code)
    .fetch_optional(&state.pool)
    .await?;

    let Some(organization) = organization else {
        return Err(ApiError::not_found(format!(
            "unknown organization code: {organization_code}"
        )));
    };

    let user = sqlx::query_as::<_, UserRow>(
        "SELECT id, email, password_hash, role, is_active \
         FROM users \
         WHERE organization_id = $1 AND email = $2",
    )
    .bind(organization.id)
    .bind(&email)
    .fetch_optional(&state.pool)
    .await?;

    let Some(user) = user else {
        return Err(ApiError::not_found(format!(
            "no user with email {email} in this organization"
        )));
    };

    if !user.is_active {
        return Err(ApiError::forbidden(format!("user {email} is disabled")));
    }

    let password_ok = verify_password(&user.password_hash, &password)
        .map_err(|e| ApiError::internal(e.to_string()))?;
    if !password_ok {
        return Err(ApiError::unauthorized("incorrect password"));
    }

    let session_token = generate_session_token();
    let token_sha256 = sha256_bytes_from_session_token(&session_token)?;
    let session_id = Uuid::now_v7();
    let now = OffsetDateTime::now_utc();
    let expires_at = now + time::Duration::days(30);

    sqlx::query(
        "INSERT INTO sessions \
         (id, organization_id, user_id, token_sha256, created_at, expires_at, last_used_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $5)",
    )
    .bind(session_id)
    .bind(organization.id)
    .bind(user.id)
    .bind(token_sha256.as_slice())
    .bind(now)
    .bind(expires_at)
    .execute(&state.pool)
    .await?;

    Ok(Json(LoginResponse {
        session_token,
        organization: OrganizationInfo {
            id: organization.id,
            code: organization.code,
            name: organization.name,
        },
        user: UserInfo {
            id: user.id,
            email: user.email,
            role: user.role,
        },
    }))
}

pub async fn me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<MeResponse>, ApiError> {
    let ctx = authenticate(&headers, &state.pool).await?;
    Ok(Json(MeResponse {
        organization: OrganizationInfo {
            id: ctx.organization_id,
            code: ctx.organization_code,
            name: ctx.organization_name,
        },
        user: UserInfo {
            id: ctx.user_id,
            email: ctx.user_email,
            role: ctx.user_role,
        },
    }))
}

pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<LogoutResponse>, ApiError> {
    let ctx = authenticate(&headers, &state.pool).await?;
    sqlx::query("UPDATE sessions SET revoked_at = now() WHERE id = $1")
        .bind(ctx.session_id)
        .execute(&state.pool)
        .await?;
    Ok(Json(LogoutResponse { ok: true }))
}

fn normalize_email(input: &str) -> Result<String, ApiError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(ApiError::bad_request("email is required"));
    }
    Ok(trimmed.to_ascii_lowercase())
}

fn generate_session_token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

fn sha256_bytes_from_session_token(token: &str) -> Result<Vec<u8>, ApiError> {
    let decoded = URL_SAFE_NO_PAD
        .decode(token.as_bytes())
        .map_err(|e| ApiError::unauthorized(format!("invalid session token encoding: {e}")))?;
    Ok(Sha256::digest(decoded).to_vec())
}

#[derive(Debug)]
struct AuthContext {
    session_id: Uuid,
    organization_id: Uuid,
    organization_code: String,
    organization_name: String,
    user_id: Uuid,
    user_email: String,
    user_role: String,
}

async fn authenticate(headers: &HeaderMap, pool: &PgPool) -> Result<AuthContext, ApiError> {
    let authorization = headers
        .get(AUTHORIZATION)
        .ok_or_else(|| ApiError::unauthorized("missing Authorization header"))?;
    let authorization = authorization
        .to_str()
        .map_err(|_| ApiError::unauthorized("invalid Authorization header"))?;

    let token = authorization
        .strip_prefix("Bearer ")
        .or_else(|| authorization.strip_prefix("bearer "))
        .ok_or_else(|| ApiError::unauthorized("Authorization must be a Bearer token"))?;

    if token.trim().is_empty() {
        return Err(ApiError::unauthorized("empty Bearer token"));
    }

    let token_sha256 = sha256_bytes_from_session_token(token)?;

    let row = sqlx::query_as::<_, SessionRow>(
        "SELECT \
            s.id AS session_id, \
            s.organization_id AS organization_id, \
            s.user_id AS user_id, \
            u.email AS user_email, \
            u.role AS user_role, \
            o.code AS organization_code, \
            o.name AS organization_name \
         FROM sessions s \
         JOIN users u ON u.id = s.user_id \
         JOIN organizations o ON o.id = s.organization_id \
         WHERE s.token_sha256 = $1 \
           AND s.revoked_at IS NULL \
           AND s.expires_at > now()",
    )
    .bind(token_sha256.as_slice())
    .fetch_one(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => ApiError::unauthorized("invalid or expired session token"),
        other => ApiError::from(other),
    })?;

    sqlx::query("UPDATE sessions SET last_used_at = now() WHERE id = $1")
        .bind(row.session_id)
        .execute(pool)
        .await?;

    Ok(AuthContext {
        session_id: row.session_id,
        organization_id: row.organization_id,
        organization_code: row.organization_code,
        organization_name: row.organization_name,
        user_id: row.user_id,
        user_email: row.user_email,
        user_role: row.user_role,
    })
}

#[derive(Debug, sqlx::FromRow)]
struct OrganizationRow {
    id: Uuid,
    code: String,
    name: String,
}

#[derive(Debug, sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    email: String,
    password_hash: String,
    role: String,
    is_active: bool,
}

#[derive(Debug, sqlx::FromRow)]
struct SessionRow {
    session_id: Uuid,
    organization_id: Uuid,
    user_id: Uuid,
    user_email: String,
    user_role: String,
    organization_code: String,
    organization_name: String,
}
