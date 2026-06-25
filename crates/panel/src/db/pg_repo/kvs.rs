use super::PgRepository;
use crate::db::error::DbError;
use crate::db::repo::*;
use async_trait::async_trait;

// ── KvsRepository ──

#[async_trait]
impl KvsRepository for PgRepository {
    async fn get(&self, key: &str) -> Result<Option<String>, DbError> {
        let row: Option<(String,)> = sqlx::query_as("SELECT value FROM kvs WHERE key = $1")
            .bind(key)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(|(v,)| v))
    }

    async fn set(&self, key: &str, value: &str) -> Result<(), DbError> {
        // PG upsert: ON CONFLICT (key) DO UPDATE. Equivalent to SQLite's
        // INSERT OR REPLACE (which is delete-then-insert; PG's update-in-place
        // is fine here because kvs has no FK dependents).
        sqlx::query(
            "INSERT INTO kvs (key, value) VALUES ($1, $2) \
             ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value",
        )
        .bind(key)
        .bind(value)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<u64, DbError> {
        let result = sqlx::query("DELETE FROM kvs WHERE key = $1")
            .bind(key)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }

    async fn scan_prefix(&self, prefix: &str) -> Result<Vec<(String, String)>, DbError> {
        // LIKE with the same '%' wildcard as SQLite. PG's LIKE is case-
        // sensitive by default (matching SQLite's behavior for ASCII), so the
        // node_status: prefix matches the same set of keys on both backends.
        let pattern = format!("{}%", prefix);
        let rows: Vec<(String, String)> =
            sqlx::query_as("SELECT key, value FROM kvs WHERE key LIKE $1")
                .bind(&pattern)
                .fetch_all(&self.pool)
                .await?;
        Ok(rows)
    }
}
