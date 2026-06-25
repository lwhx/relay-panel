use super::SqliteRepository;
use crate::db::error::DbError;
use crate::db::repo::*;
use async_trait::async_trait;
use relay_shared::models::Statistic;

// ── StatisticsRepository ──

#[async_trait]
impl StatisticsRepository for SqliteRepository {
    async fn query_stats(
        &self,
        stat_type: Option<&str>,
        stat_key: Option<&str>,
        from: Option<&str>,
        to: Option<&str>,
    ) -> Result<Vec<Statistic>, DbError> {
        // COALESCE treats a NULL filter as "match anything" — the canonical
        // optional-filter pattern in SQL. '2000-01-01' / '2099-12-31' are
        // sentinels wide enough to cover any realistic timestamp string.
        let stats: Vec<Statistic> = sqlx::query_as(
            "SELECT * FROM statistics WHERE stat_type = COALESCE(?, stat_type) AND stat_key = COALESCE(?, stat_key) AND time >= COALESCE(?, '2000-01-01') AND time <= COALESCE(?, '2099-12-31') ORDER BY time",
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
