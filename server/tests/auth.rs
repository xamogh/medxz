use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use serde_json::json;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn login_me_logout_happy_path() {
    let Some(test_db) = TestDb::new().await else {
        return;
    };
    test_db
        .seed_org_and_user(
            "acme-dental",
            "Acme Dental",
            "front@desk.com",
            "pw123",
            "front_desk",
        )
        .await;

    let app = test_db.router();

    let login_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "organization_code": "acme-dental",
                        "email": "front@desk.com",
                        "password": "pw123"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(login_response.status(), StatusCode::OK);
    let login_body = body_json(login_response).await;
    let token = login_body
        .get("session_token")
        .and_then(|v| v.as_str())
        .expect("session_token must be present")
        .to_string();

    let me_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/auth/me")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(me_response.status(), StatusCode::OK);
    let me_body = body_json(me_response).await;
    assert_eq!(me_body["user"]["email"], "front@desk.com");
    assert_eq!(me_body["organization"]["code"], "acme-dental");

    let logout_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/logout")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(logout_response.status(), StatusCode::OK);

    let me_after_logout = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/auth/me")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(me_after_logout.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn login_returns_descriptive_errors() {
    let Some(test_db) = TestDb::new().await else {
        return;
    };
    test_db
        .seed_org_and_user("acme", "Acme", "front@desk.com", "pw123", "front_desk")
        .await;
    let app = test_db.router();

    let missing_org = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "organization_code": " ",
                        "email": "front@desk.com",
                        "password": "pw123"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(missing_org.status(), StatusCode::BAD_REQUEST);

    let unknown_org = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "organization_code": "nope",
                        "email": "front@desk.com",
                        "password": "pw123"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(unknown_org.status(), StatusCode::NOT_FOUND);
    let unknown_org_body = body_json(unknown_org).await;
    assert!(unknown_org_body["message"]
        .as_str()
        .unwrap_or_default()
        .contains("unknown organization code"));

    let unknown_user = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "organization_code": "acme",
                        "email": "nope@desk.com",
                        "password": "pw123"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(unknown_user.status(), StatusCode::NOT_FOUND);

    let wrong_password = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "organization_code": "acme",
                        "email": "front@desk.com",
                        "password": "wrong"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(wrong_password.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn me_requires_bearer_token() {
    let Some(test_db) = TestDb::new().await else {
        return;
    };
    test_db
        .seed_org_and_user("acme", "Acme", "front@desk.com", "pw123", "front_desk")
        .await;
    let app = test_db.router();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/auth/me")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

struct TestDb {
    pool: PgPool,
}

impl TestDb {
    async fn new() -> Option<Self> {
        let database_url = match std::env::var("DATABASE_URL") {
            Ok(v) => v,
            Err(_) => {
                eprintln!("DATABASE_URL is not set; skipping Postgres-backed auth tests");
                return None;
            }
        };
        let schema = format!("test_{}", Uuid::now_v7().as_simple());

        let admin_pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .expect("failed to connect to DATABASE_URL");

        sqlx::query(&format!("CREATE SCHEMA \"{schema}\""))
            .execute(&admin_pool)
            .await
            .expect("failed to create test schema");

        let schema_clone = schema.clone();
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .after_connect(move |conn, _meta| {
                let schema = schema_clone.clone();
                Box::pin(async move {
                    sqlx::query(&format!("SET search_path TO \"{schema}\""))
                        .execute(conn)
                        .await?;
                    Ok(())
                })
            })
            .connect(&database_url)
            .await
            .expect("failed to connect to DATABASE_URL");

        medxz_server::db::migrate(&pool)
            .await
            .expect("failed to run migrations");

        Some(Self { pool })
    }

    fn router(&self) -> axum::Router {
        medxz_server::app::router(medxz_server::state::AppState {
            pool: self.pool.clone(),
        })
    }

    async fn seed_org_and_user(
        &self,
        org_code: &str,
        org_name: &str,
        email: &str,
        password: &str,
        role: &str,
    ) {
        let org_id = Uuid::now_v7();
        sqlx::query("INSERT INTO organizations (id, code, name) VALUES ($1, $2, $3)")
            .bind(org_id)
            .bind(org_code)
            .bind(org_name)
            .execute(&self.pool)
            .await
            .expect("failed to seed organization");

        let password_hash =
            medxz_server::auth::hash_password(password).expect("hash_password failed");
        let user_id = Uuid::now_v7();
        sqlx::query(
            "INSERT INTO users (id, organization_id, email, password_hash, role) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(user_id)
        .bind(org_id)
        .bind(email.trim().to_ascii_lowercase())
        .bind(password_hash)
        .bind(role)
        .execute(&self.pool)
        .await
        .expect("failed to seed user");
    }
}

async fn body_json(response: axum::response::Response) -> serde_json::Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(bytes.as_ref()).unwrap()
}
