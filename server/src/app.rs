use axum::{
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;

use crate::{auth, state::AppState};

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/v1/auth/login", post(auth::login))
        .route("/v1/auth/me", get(auth::me))
        .route("/v1/auth/logout", post(auth::logout))
        .with_state(state)
}

async fn healthz() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}
