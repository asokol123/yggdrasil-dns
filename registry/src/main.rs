mod db;
use axum::{
    extract::Extension,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use clap::Parser;
use p256::ecdsa::{signature::Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    str::FromStr,
    sync::Arc,
    time::{SystemTime, SystemTimeError, UNIX_EPOCH},
};

#[derive(Debug, Parser)]
struct Args {
    #[clap(short, long)]
    bind_addr: String,
    #[clap(short, long)]
    db_addr: String,
}

fn check_nonce(body: impl AsRef<[u8]>) -> bool {
    hex::encode(Sha256::digest(body.as_ref())).starts_with("0000")
}

fn current_timestamp() -> u64 {
    let now = std::time::SystemTime::now();
    now.duration_since(std::time::UNIX_EPOCH)
        .expect("Time is before unix epoch")
        .as_secs()
}

fn check_timestamp(ts: u64) -> bool {
    current_timestamp().wrapping_sub(ts) < 30
}

struct State {
    db: sqlx::SqlitePool,
}

fn internal_error(e: impl std::fmt::Debug) -> (StatusCode, Json<String>) {
    let resp = format!("Internal error: {:?}", e);
    tracing::error!("Internal error: {:?}", e);
    (StatusCode::INTERNAL_SERVER_ERROR, Json(resp))
}

fn bad_request(e: impl std::fmt::Debug) -> (StatusCode, Json<String>) {
    let resp = format!("Bad request: {:?}", e);
    tracing::info!("Got invalid request: {:?}", e);
    (StatusCode::BAD_REQUEST, Json(resp))
}

#[derive(Deserialize)]
struct RegisterRequest {
    name: String,
    pubkey: String,
    timestamp: u64,
}

async fn register(
    body: String,
    Extension(state): Extension<Arc<State>>,
) -> Result<(), (StatusCode, Json<String>)> {
    if !check_nonce(&body) {
        return Err(bad_request("bad nonce"));
    }

    let req: RegisterRequest = serde_json::from_str(&body).map_err(bad_request)?;
    if !check_timestamp(req.timestamp) {
        return Err(bad_request("bad timestamp"));
    }

    let mut tx = state.db.begin().await.map_err(internal_error)?;
    if sqlx::query("SELECT * FROM users WHERE name = $1")
        .bind(&req.name)
        .fetch_optional(&mut tx)
        .await
        .map_err(internal_error)?
        .is_some()
    {
        return Err((
            StatusCode::CONFLICT,
            Json(format!("User {} already exists", &req.name)),
        ));
    }

    sqlx::query("INSERT INTO users (name, pubkey) VALUES ($1, $2)")
        .bind(&req.name)
        .bind(&req.pubkey)
        .execute(&mut tx)
        .await
        .map_err(internal_error)?;

    tx.commit().await.map_err(internal_error)?;

    Ok(())
}

#[derive(Deserialize)]
struct SetSiteRequest {
    site: String,
    address: String,
    expires: i64,
    owner: String,
    signature: String,
    timestamp: u64,
}

fn check_signature(
    req: &SetSiteRequest,
    pubkey: &p256::ecdsa::VerifyingKey,
) -> Result<bool, (StatusCode, Json<String>)> {
    let message = req.owner.clone() + &req.site + &req.timestamp.to_string();
    let signature = hex::decode(&req.signature).map_err(bad_request)?;
    Ok(pubkey
        .verify(
            message.as_bytes(),
            &p256::ecdsa::Signature::from_der(&signature).map_err(bad_request)?,
        )
        .is_ok())
}

async fn set_site(
    body: String,
    Extension(state): Extension<Arc<State>>,
) -> Result<(), (StatusCode, Json<String>)> {
    if !check_nonce(&body) {
        return Err(bad_request("bad nonce"));
    }

    let req: SetSiteRequest = serde_json::from_str(&body).map_err(bad_request)?;
    if !check_timestamp(req.timestamp) {
        return Err(bad_request("bad timestamp"));
    }

    let mut tx = state.db.begin().await.map_err(internal_error)?;
    let pubkey = sqlx::query_as::<_, (String,)>("SELECT pubkey FROM users WHERE name = $1")
        .bind(&req.owner)
        .fetch_optional(&mut tx)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(format!("No user with name {}", &req.owner)),
            )
        })?
        .0;
    let pubkey = VerifyingKey::from_str(&pubkey).map_err(internal_error)?;
    if !check_signature(&req, &pubkey)? {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(format!("Invalid signature for user {}", &req.owner)),
        ));
    }

    if let Some(owner) = sqlx::query_as::<_, (String,)>("SELECT owner FROM sites WHERE name = $1")
        .bind(&req.site)
        .fetch_optional(&mut tx)
        .await
        .map_err(internal_error)?
    {
        let owner = owner.0;
        if owner != req.owner {
            return Err((
                StatusCode::FORBIDDEN,
                Json(format!(
                    "Site {} is owned by {}, not {}",
                    &req.site, &req.owner, &owner
                )),
            ));
        }
        sqlx::query("UPDATE sites SET address = $1, expires = $2 WHERE name = $3")
            .bind(&req.address)
            .bind(&req.expires)
            .bind(&req.site)
            .execute(&mut tx)
            .await
            .map_err(internal_error)?;
    } else {
        sqlx::query("INSERT INTO sites (name, owner, address, expires) VALUES ($1, $2, $3, $4)")
            .bind(&req.site)
            .bind(&req.owner)
            .bind(&req.address)
            .bind(&req.expires)
            .execute(&mut tx)
            .await
            .map_err(internal_error)?;
    }

    tx.commit().await.map_err(internal_error)?;

    Ok(())
}

#[derive(Deserialize)]
struct GetSiteRequest {
    name: String,
    timestamp: u64,
}

#[derive(Serialize)]
struct GetSiteResponse {
    address: String,
}

fn get_current_timestamp() -> Result<i64, SystemTimeError> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64)
}

#[derive(sqlx::FromRow)]
struct Site {
    address: String,
    expires: i64,
}

async fn get_site(
    body: String,
    Extension(state): Extension<Arc<State>>,
) -> Result<Json<GetSiteResponse>, (StatusCode, Json<String>)> {
    if !check_nonce(&body) {
        return Err(bad_request("bad nonce"));
    }

    let req: GetSiteRequest = serde_json::from_str(&body).map_err(bad_request)?;
    if !check_timestamp(req.timestamp) {
        return Err(bad_request("bad timestamp"));
    }

    let site: Site = sqlx::query_as("SELECT address, expires FROM sites WHERE name = $1")
        .bind(&req.name)
        .fetch_optional(&state.db)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(format!("No site with name {}", &req.name)),
            )
        })?;

    if site.expires < get_current_timestamp().map_err(internal_error)? {
        return Err((
            StatusCode::GONE,
            Json(format!(
                "Site {} already expired, please update it if you are owner",
                &req.name
            )),
        ));
    }

    Ok(Json(GetSiteResponse {
        address: site.address,
    }))
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    tracing_subscriber::fmt::init();

    let state = Arc::new(State {
        db: db::try_connect_db(&args.db_addr).await.unwrap(),
    });

    let app = Router::new()
        .route("/register", post(register))
        .route("/set_site", post(set_site))
        .route("/get_site", get(get_site))
        .layer(Extension(state));

    axum::Server::bind(&args.bind_addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
