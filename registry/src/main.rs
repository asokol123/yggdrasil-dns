mod db;
use std::{
    sync::Arc,
    time::{SystemTime, SystemTimeError, UNIX_EPOCH},
};

use axum::{
    extract::Extension,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Debug, Parser)]
struct Args {
    #[clap(short, long)]
    bind_addr: String,
    #[clap(short, long)]
    db_addr: String,
}

fn check_nonce(_body: impl AsRef<str>) -> bool {
    // TODO: check nonce
    true
}

fn check_timestamp(_ts: u64) -> bool {
    // TODO: check timestamp
    true
}

struct State {
    db: sqlx::SqlitePool,
}

fn internal_error(e: impl std::fmt::Debug) -> StatusCode {
    tracing::error!("Internal error: {:?}", e);
    StatusCode::INTERNAL_SERVER_ERROR
}

fn bad_request(e: impl std::fmt::Debug) -> StatusCode {
    tracing::info!("Got invalid request: {:?}", e);
    StatusCode::BAD_REQUEST
}

#[derive(Deserialize)]
struct RegisterRequest {
    name: String,
    pubkey: String,
    timestamp: u64,
}

async fn register(body: String, Extension(state): Extension<Arc<State>>) -> Result<(), StatusCode> {
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
        return Err(StatusCode::CONFLICT);
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

fn check_signature(req: &SetSiteRequest) -> bool {
    // TODO
    !req.signature.is_empty()
}

async fn set_site(body: String, Extension(state): Extension<Arc<State>>) -> Result<(), StatusCode> {
    if !check_nonce(&body) {
        return Err(bad_request("bad nonce"));
    }

    let req: SetSiteRequest = serde_json::from_str(&body).map_err(bad_request)?;
    if !check_timestamp(req.timestamp) {
        return Err(bad_request("bad timestamp"));
    }

    let mut tx = state.db.begin().await.map_err(internal_error)?;
    let pubkey: (String,) = sqlx::query_as("SELECT pubkey FROM users WHERE name = $1")
        .bind(&req.owner)
        .fetch_optional(&mut tx)
        .await
        .map_err(internal_error)?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let _pubkey = pubkey.0;
    if !check_signature(&req) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    if let Some(owner) = sqlx::query_as::<_, (String,)>("SELECT owner FROM sites WHERE name = $1")
        .bind(&req.site)
        .fetch_optional(&mut tx)
        .await
        .map_err(internal_error)?
    {
        let owner = owner.0;
        if owner != req.owner {
            return Err(StatusCode::UNAUTHORIZED);
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
) -> Result<Json<GetSiteResponse>, StatusCode> {
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
        .ok_or(StatusCode::NOT_FOUND)?;

    if site.expires < get_current_timestamp().map_err(internal_error)? {
        return Err(StatusCode::GONE);
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
