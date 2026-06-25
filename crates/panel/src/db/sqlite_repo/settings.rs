use super::SqliteRepository;
use crate::db::error::DbError;
use crate::db::repo::*;
use async_trait::async_trait;
use relay_shared::models::Plan;

// ── PlanRepository ──

#[async_trait]
impl PlanRepository for SqliteRepository {
    async fn list_plans(&self) -> Result<Vec<Plan>, DbError> {
        let plans: Vec<Plan> = sqlx::query_as("SELECT * FROM plans ORDER BY id")
            .fetch_all(&self.pool)
            .await?;
        Ok(plans)
    }

    async fn find_plan_name_by_id(&self, id: i64) -> Result<Option<String>, DbError> {
        let row: Option<(String,)> = sqlx::query_as("SELECT name FROM plans WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(|(n,)| n))
    }
}

// ── Helpers ──

/// Parse the `registration_allowed_plan_ids` TEXT column as a JSON `Vec<i64>`.
/// Falls back to `[default_plan_id]` on parse failure (dirty data).
fn parse_allowed_plan_ids(raw: &str, fallback: i64) -> Vec<i64> {
    serde_json::from_str::<Vec<i64>>(raw).unwrap_or_else(|_| vec![fallback])
}

/// Serialize a `Vec<i64>` to a JSON string.
fn serialize_allowed_plan_ids(ids: &[i64]) -> String {
    serde_json::to_string(ids).unwrap_or_else(|_| "[1]".to_string())
}

#[async_trait]
impl SettingsRepository for SqliteRepository {
    async fn get_registration_settings(&self) -> Result<Option<RegistrationSettings>, DbError> {
        let row: Option<(bool, i64, String)> = sqlx::query_as(
            "SELECT registration_enabled, default_registration_plan_id, \
             registration_allowed_plan_ids FROM app_settings WHERE id = 1",
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|(enabled, plan_id, raw_allowed)| {
            let allowed = parse_allowed_plan_ids(&raw_allowed, plan_id);
            RegistrationSettings {
                registration_enabled: enabled,
                default_registration_plan_id: plan_id,
                allowed_plan_ids: allowed,
            }
        }))
    }

    async fn insert_settings_if_absent(
        &self,
        enabled: bool,
        default_plan_id: i64,
        allowed_plan_ids: &[i64],
    ) -> Result<(), DbError> {
        let allowed_json = serialize_allowed_plan_ids(allowed_plan_ids);
        sqlx::query(
            "INSERT OR IGNORE INTO app_settings (id, registration_enabled, \
             default_registration_plan_id, registration_allowed_plan_ids) \
             VALUES (1, ?, ?, ?)",
        )
        .bind(enabled)
        .bind(default_plan_id)
        .bind(&allowed_json)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn set_registration_settings(
        &self,
        enabled: bool,
        default_plan_id: i64,
        allowed_plan_ids: &[i64],
    ) -> Result<(), DbError> {
        let allowed_json = serialize_allowed_plan_ids(allowed_plan_ids);
        sqlx::query(
            "INSERT INTO app_settings (id, registration_enabled, \
             default_registration_plan_id, registration_allowed_plan_ids) \
             VALUES (1, ?, ?, ?) \
             ON CONFLICT(id) DO UPDATE SET \
                 registration_enabled = excluded.registration_enabled, \
                 default_registration_plan_id = excluded.default_registration_plan_id, \
                 registration_allowed_plan_ids = excluded.registration_allowed_plan_ids",
        )
        .bind(enabled)
        .bind(default_plan_id)
        .bind(&allowed_json)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
