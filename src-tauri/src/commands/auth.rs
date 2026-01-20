use crate::core::error::{AppError, AppResult};
use keyring::Entry;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct OrganizationInfo {
    pub id: String,
    pub code: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SessionInfo {
    pub organization: OrganizationInfo,
    pub user: UserInfo,
}

#[derive(Debug, Deserialize)]
struct LoginResponse {
    session_token: String,
    organization: OrganizationInfo,
    user: UserInfo,
}

#[derive(Debug, Deserialize)]
struct MeResponse {
    organization: OrganizationInfo,
    user: UserInfo,
}

#[derive(Debug, Deserialize)]
struct ServerErrorBody {
    code: String,
    message: String,
}

#[tauri::command(rename_all = "camelCase")]
#[specta::specta]
pub(crate) async fn login(
    server_url: String,
    organization_code: String,
    email: String,
    password: String,
) -> AppResult<SessionInfo> {
    let url = join_url(&server_url, "/v1/auth/login")?;

    let client = reqwest::Client::new();
    let response = client
        .post(url)
        .json(&serde_json::json!({
            "organization_code": organization_code,
            "email": email,
            "password": password,
        }))
        .send()
        .await
        .map_err(|e| AppError::Network {
            message: e.to_string(),
        })?;

    if !response.status().is_success() {
        return Err(parse_server_error(response).await);
    }

    let data: LoginResponse = response.json().await.map_err(|e| AppError::Network {
        message: format!("failed to decode server response: {e}"),
    })?;

    store_session_token(&data.session_token)?;

    Ok(SessionInfo {
        organization: data.organization,
        user: data.user,
    })
}

#[tauri::command(rename_all = "camelCase")]
#[specta::specta]
pub(crate) async fn me(server_url: String) -> AppResult<Option<SessionInfo>> {
    let Some(token) = load_session_token()? else {
        return Ok(None);
    };

    let url = join_url(&server_url, "/v1/auth/me")?;

    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| AppError::Network {
            message: e.to_string(),
        })?;

    if response.status() == StatusCode::UNAUTHORIZED {
        delete_session_token()?;
        return Ok(None);
    }

    if !response.status().is_success() {
        return Err(parse_server_error(response).await);
    }

    let data: MeResponse = response.json().await.map_err(|e| AppError::Network {
        message: format!("failed to decode server response: {e}"),
    })?;

    Ok(Some(SessionInfo {
        organization: data.organization,
        user: data.user,
    }))
}

#[tauri::command(rename_all = "camelCase")]
#[specta::specta]
pub(crate) async fn logout(server_url: String) -> AppResult<()> {
    let Some(token) = load_session_token()? else {
        return Ok(());
    };

    let url = join_url(&server_url, "/v1/auth/logout")?;

    let client = reqwest::Client::new();
    let response = client
        .post(url)
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| AppError::Network {
            message: e.to_string(),
        })?;

    if !response.status().is_success() {
        return Err(parse_server_error(response).await);
    }

    delete_session_token()?;
    Ok(())
}

fn join_url(base: &str, path: &str) -> AppResult<String> {
    let base = base.trim();
    if base.is_empty() {
        return Err(AppError::InvalidServerUrl {
            message: "server_url is required".into(),
        });
    }

    let base = base.trim_end_matches('/');
    Ok(format!("{base}{path}"))
}

fn session_entry() -> AppResult<Entry> {
    Entry::new("com.medxz.app", "session_token").map_err(|e| AppError::Keychain {
        message: e.to_string(),
    })
}

fn store_session_token(token: &str) -> AppResult<()> {
    session_entry()?
        .set_password(token)
        .map_err(|e| AppError::Keychain {
            message: e.to_string(),
        })?;
    Ok(())
}

fn load_session_token() -> AppResult<Option<String>> {
    let entry = session_entry()?;
    match entry.get_password() {
        Ok(value) => Ok(Some(value)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(AppError::Keychain {
            message: e.to_string(),
        }),
    }
}

fn delete_session_token() -> AppResult<()> {
    let entry = session_entry()?;
    match entry.delete_password() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(AppError::Keychain {
            message: e.to_string(),
        }),
    }
}

async fn parse_server_error(response: reqwest::Response) -> AppError {
    let status = response.status().as_u16();
    let body = response.json::<ServerErrorBody>().await;
    match body {
        Ok(body) => AppError::ServerError {
            status,
            code: body.code,
            message: body.message,
        },
        Err(e) => AppError::ServerError {
            status,
            code: "unknown".into(),
            message: format!("failed to decode server error response: {e}"),
        },
    }
}
