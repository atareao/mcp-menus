use anyhow::Result;
use sqlx::{Row, SqlitePool};

use crate::models::{ListMenusParams, MenuEntry, SetMenuParams};

pub async fn create_pool(db_path: &str) -> Result<SqlitePool> {
    let pool = SqlitePool::connect_with(
        sqlx::sqlite::SqliteConnectOptions::new()
            .filename(db_path)
            .create_if_missing(true)
            .busy_timeout(std::time::Duration::from_secs(5))
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal),
    )
    .await?;
    Ok(pool)
}

pub async fn init_db(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS menu_entries (
            date TEXT PRIMARY KEY,
            comida_id TEXT,
            comida_name TEXT,
            cena_id TEXT,
            cena_name TEXT
        )
        "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn set_menu(pool: &SqlitePool, params: SetMenuParams) -> Result<MenuEntry> {
    sqlx::query(
        r#"
        INSERT INTO menu_entries (date, comida_id, comida_name, cena_id, cena_name)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT(date) DO UPDATE SET
            comida_id = COALESCE($2, menu_entries.comida_id),
            comida_name = COALESCE($3, menu_entries.comida_name),
            cena_id = COALESCE($4, menu_entries.cena_id),
            cena_name = COALESCE($5, menu_entries.cena_name)
        "#,
    )
    .bind(&params.date)
    .bind(&params.comida_id)
    .bind(&params.comida_name)
    .bind(&params.cena_id)
    .bind(&params.cena_name)
    .execute(pool)
    .await?;

    get_menu(pool, &params.date).await
}

pub async fn get_menu(pool: &SqlitePool, date: &str) -> Result<MenuEntry> {
    let row = sqlx::query(
        r#"
        SELECT date, comida_id, comida_name, cena_id, cena_name
        FROM menu_entries
        WHERE date = $1
        "#,
    )
    .bind(date)
    .fetch_one(pool)
    .await?;

    Ok(MenuEntry {
        date: row.get("date"),
        comida_id: row.get("comida_id"),
        comida_name: row.get("comida_name"),
        cena_id: row.get("cena_id"),
        cena_name: row.get("cena_name"),
    })
}

pub async fn list_menus(pool: &SqlitePool, params: ListMenusParams) -> Result<Vec<MenuEntry>> {
    let start = params
        .start_date
        .unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%d").to_string());

    let end = params.end_date.unwrap_or_else(|| {
        (chrono::Utc::now() + chrono::Duration::days(6))
            .format("%Y-%m-%d")
            .to_string()
    });

    let rows = sqlx::query(
        r#"
        SELECT date, comida_id, comida_name, cena_id, cena_name
        FROM menu_entries
        WHERE date >= $1 AND date <= $2
        ORDER BY date ASC
        "#,
    )
    .bind(&start)
    .bind(&end)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .iter()
        .map(|row| MenuEntry {
            date: row.get("date"),
            comida_id: row.get("comida_id"),
            comida_name: row.get("comida_name"),
            cena_id: row.get("cena_id"),
            cena_name: row.get("cena_name"),
        })
        .collect())
}

pub async fn clear_day(pool: &SqlitePool, date: &str) -> Result<bool> {
    let result = sqlx::query("DELETE FROM menu_entries WHERE date = $1")
        .bind(date)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}