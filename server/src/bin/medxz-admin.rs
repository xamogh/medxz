#![forbid(unsafe_code)]

use std::collections::HashMap;

use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    if let Err(err) = run().await {
        eprintln!("{err}");
        if err.should_print_usage() {
            eprintln!();
            eprintln!("{}", usage());
        }
        std::process::exit(err.exit_code());
    }
}

#[derive(Debug, Error)]
enum CliError {
    #[error("missing command")]
    MissingCommand,

    #[error("unknown command: {0}")]
    UnknownCommand(String),

    #[error("unexpected argument: {0}")]
    UnexpectedArgument(String),

    #[error("missing value for flag --{0}")]
    MissingFlagValue(String),

    #[error("missing required flag --{0}")]
    MissingRequiredFlag(&'static str),

    #[error("unknown organization code: {0}")]
    UnknownOrganizationCode(String),

    #[error("user already exists: {0}")]
    UserAlreadyExists(String),

    #[error(transparent)]
    Db(#[from] medxz_server::db::DbError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    PasswordHash(#[from] medxz_server::auth::PasswordHashError),
}

impl CliError {
    fn exit_code(&self) -> i32 {
        match self {
            CliError::MissingCommand
            | CliError::UnknownCommand(_)
            | CliError::UnexpectedArgument(_)
            | CliError::MissingFlagValue(_)
            | CliError::MissingRequiredFlag(_) => 2,
            CliError::UnknownOrganizationCode(_)
            | CliError::UserAlreadyExists(_)
            | CliError::Db(_)
            | CliError::Sqlx(_)
            | CliError::PasswordHash(_) => 1,
        }
    }

    fn should_print_usage(&self) -> bool {
        matches!(
            self,
            CliError::MissingCommand
                | CliError::UnknownCommand(_)
                | CliError::UnexpectedArgument(_)
                | CliError::MissingFlagValue(_)
                | CliError::MissingRequiredFlag(_)
        )
    }
}

async fn run() -> Result<(), CliError> {
    let mut args = std::env::args().skip(1);
    let command = args.next().ok_or(CliError::MissingCommand)?;

    if command == "help" || command == "--help" || command == "-h" {
        println!("{}", usage());
        return Ok(());
    }

    let opts = parse_opts(args)?;

    if command == "bootstrap" {
        bootstrap(opts).await?;
        return Ok(());
    }
    if command == "create-organization" {
        create_organization(opts).await?;
        return Ok(());
    }
    if command == "create-user" {
        create_user(opts).await?;
        return Ok(());
    }

    Err(CliError::UnknownCommand(command))
}

async fn bootstrap(opts: HashMap<String, String>) -> Result<(), CliError> {
    let pool = connect().await?;
    let org_code = required(&opts, "org-code")?;
    let org_name = required(&opts, "org-name")?;

    let org_id = ensure_organization(&pool, org_code, org_name).await?;

    let email = required(&opts, "email")?;
    let password = required(&opts, "password")?;
    let role = opts.get("role").map(String::as_str).unwrap_or("front_desk");

    let user_id = ensure_user(&pool, org_id, email, password, role).await?;

    println!("Bootstrapped:");
    println!("- organization_code={org_code} organization_id={org_id}");
    println!("- email={email} user_id={user_id} role={role}");
    Ok(())
}

async fn create_organization(opts: HashMap<String, String>) -> Result<(), CliError> {
    let pool = connect().await?;
    let org_code = required(&opts, "org-code")?;
    let org_name = required(&opts, "org-name")?;
    let org_id = ensure_organization(&pool, org_code, org_name).await?;
    println!("organization_code={org_code} organization_id={org_id}");
    Ok(())
}

async fn create_user(opts: HashMap<String, String>) -> Result<(), CliError> {
    let pool = connect().await?;
    let org_code = required(&opts, "org-code")?;
    let email = required(&opts, "email")?;
    let password = required(&opts, "password")?;
    let role = opts.get("role").map(String::as_str).unwrap_or("front_desk");

    let org_id: Option<Uuid> = sqlx::query_scalar("SELECT id FROM organizations WHERE code = $1")
        .bind(org_code)
        .fetch_optional(&pool)
        .await?;

    let org_id = org_id.ok_or_else(|| CliError::UnknownOrganizationCode(org_code.to_string()))?;

    let user_id = ensure_user(&pool, org_id, email, password, role).await?;
    println!("email={email} user_id={user_id} role={role}");
    Ok(())
}

async fn connect() -> Result<PgPool, CliError> {
    Ok(medxz_server::db::connect_from_env_and_migrate().await?)
}

async fn ensure_organization(pool: &PgPool, code: &str, name: &str) -> Result<Uuid, CliError> {
    let existing: Option<Uuid> = sqlx::query_scalar("SELECT id FROM organizations WHERE code = $1")
        .bind(code)
        .fetch_optional(pool)
        .await?;

    if let Some(id) = existing {
        return Ok(id);
    }

    let id = Uuid::now_v7();
    sqlx::query("INSERT INTO organizations (id, code, name) VALUES ($1, $2, $3)")
        .bind(id)
        .bind(code)
        .bind(name)
        .execute(pool)
        .await?;
    Ok(id)
}

async fn ensure_user(
    pool: &PgPool,
    organization_id: Uuid,
    email: &str,
    password: &str,
    role: &str,
) -> Result<Uuid, CliError> {
    let email = email.trim().to_ascii_lowercase();
    let existing: Option<Uuid> =
        sqlx::query_scalar("SELECT id FROM users WHERE organization_id = $1 AND email = $2")
            .bind(organization_id)
            .bind(&email)
            .fetch_optional(pool)
            .await?;

    if existing.is_some() {
        return Err(CliError::UserAlreadyExists(email));
    }

    let password_hash = medxz_server::auth::hash_password(password)?;

    let id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO users (id, organization_id, email, password_hash, role) \
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(id)
    .bind(organization_id)
    .bind(&email)
    .bind(password_hash)
    .bind(role)
    .execute(pool)
    .await?;
    Ok(id)
}

fn parse_opts(args: impl Iterator<Item = String>) -> Result<HashMap<String, String>, CliError> {
    let mut opts = HashMap::new();
    let mut it = args;
    while let Some(arg) = it.next() {
        if !arg.starts_with("--") {
            return Err(CliError::UnexpectedArgument(arg));
        }
        let key = arg.trim_start_matches("--").to_string();
        let value = match it.next() {
            Some(v) => v,
            None => return Err(CliError::MissingFlagValue(key)),
        };
        opts.insert(key, value);
    }
    Ok(opts)
}

fn required<'a>(opts: &'a HashMap<String, String>, key: &'static str) -> Result<&'a str, CliError> {
    opts.get(key)
        .map(String::as_str)
        .ok_or(CliError::MissingRequiredFlag(key))
}

fn usage() -> &'static str {
    "Usage:\n  medxz-admin bootstrap --org-code <code> --org-name <name> --email <email> --password <password> [--role <role>]\n  medxz-admin create-organization --org-code <code> --org-name <name>\n  medxz-admin create-user --org-code <code> --email <email> --password <password> [--role <role>]"
}
