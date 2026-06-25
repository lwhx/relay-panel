use super::PgRepository;
use crate::db::error::DbError;
use crate::db::repo::*;
use async_trait::async_trait;
use relay_shared::models::Statistic;

// ── StatisticsRepository ──

#[async_trait]
impl StatisticsRepository for PgRepository {
    async fn query_stats(
        &self,
        stat_type: Option<&str>,
        stat_key: Option<&str>,
        from: Option<&str>,
        to: Option<&str>,
    ) -> Result<Vec<Statistic>, DbError> {
        // Same COALESCE-optional-filter pattern as SQLite. PG's COALESCE works
        // identically. The sentinel timestamps are wide enough for any real
        // 'YYYY-MM-DD HH:MM:SS' value.
        let stats: Vec<Statistic> = sqlx::query_as(
            "SELECT * FROM statistics WHERE stat_type = COALESCE($1, stat_type) AND stat_key = COALESCE($2, stat_key) AND time >= COALESCE($3, '2000-01-01') AND time <= COALESCE($4, '2099-12-31') ORDER BY time",
        )
        .bind(stat_type)
        .bind(stat_key)
        .bind(from)
        .bind(to)
        .fetch_all(&self.pool)
        .await?;
        Ok(stats)
    }
}
