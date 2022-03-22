use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::str::FromStr;

async fn create_tables(db: &SqlitePool) -> sqlx::Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS sites (
    name TEXT PRIMARY KEY,
    owner TEXT NOT NULL,
    address TEXT NOT NULL,
    expires INTEGER
)",
    )
    .execute(db)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
    name TEXT PRIMARY KEY,
    pubkey TEXT NOT NULL
)",
    )
    .execute(db)
    .await?;
    Ok(())
}

pub async fn try_connect_db(addr: impl AsRef<str>) -> sqlx::Result<SqlitePool> {
    let s = SqlitePoolOptions::new()
        .connect_with(SqliteConnectOptions::from_str(addr.as_ref())?.create_if_missing(true))
        .await?;
    create_tables(&s).await?;
    Ok(s)
}
