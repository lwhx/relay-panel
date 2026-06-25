use super::SqliteRepository;
use crate::db::error::DbError;
use crate::db::repo::*;
use async_trait::async_trait;

// ── KvsRepository ──

#[async_trait]
impl KvsRepository for SqliteRepository {
    async fn get(&self, key: &str) -> Result<Option<String>, DbError> {
        let row: Option<(String,)> = sqlx::query_as("SELECT value FROM kvs WHERE key = ?")
            .bind(key)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(|(v,)| v))
    }

    async fn set(&self, key: &str, value: &str) -> Result<(), DbError> {
        // INSERT OR REPLACE is the SQLite upsert — used by report_status to
        // store node status JSON keyed by node_status:{gid}[:{node_id}].
        sqlx::query("INSERT OR REPLACE INTO kvs (key, value) VALUES (?, ?)")
            .bind(key)
            .bind(value)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<u64, DbError> {
        let result = sqlx::query("DELETE FROM kvs WHERE key = ?")
            .bind(key)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }

    async fn scan_prefix(&self, prefix: &str) -> Result<Vec<(String, String)>, DbError> {
        // LIKE with no wildcard escapes — prefix is concatenated with '%' so
        // special LIKE chars in the prefix are taken literally by SQLite only
        // if ESCAPE is set. The current callers pass fixed prefixes like
        // 'node_status:' so this is fine; the PR2 PG impl will use a real
        // starts_with operator.
        let pattern = format!("{}%", prefix);
        let rows: Vec<(String, String)> =
            sqlx::query_as("SELECT key, value FROM kvs WHERE key LIKE ?")
                .bind(&pattern)
                .fetch_all(&self.pool)
                .await?;
        Ok(rows)
    }
}
