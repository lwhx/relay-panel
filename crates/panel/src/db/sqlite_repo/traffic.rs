use super::SqliteRepository;
use crate::db::error::DbError;
use crate::db::repo::*;
use async_trait::async_trait;
use relay_shared::protocol::TrafficEntry;

// ── TrafficRepository ──
//
// Atomicity + security contract (v0.4.9 hardened):
//   - whole batch is one transaction (deferred BEGIN; SQLite serialises writers)
//   - rule NOT available to this node (missing OR foreign-group): ABORT +
//     rollback the entire batch, return Ok(vec![Unavailable]). The caller maps
//     that to a uniform 403 with a generic message. There is deliberately NO
//     distinction between "missing" and "foreign" — that distinction was a
//     rule-id existence oracle (a node could enumerate ids and tell from the
//     response whether an id exists in another group). The real reason is
//     logged server-side only, never returned to the node.
//   - overflow (per-entry, per-rule cumulative, per-user cumulative, or
//     existing value + delta): ABORT + rollback, return Ok(vec![Overflow]).
//     The caller maps that to a uniform 400.
//   - duplicate rule_ids in one batch are aggregated (summed) first, so the
//     overflow check sees the true batch delta and each distinct rule gets one
//     UPDATE.
//   - any UPDATE failure: ABORT + rollback, return Err(DbError).
//   - only after COMMIT succeeds do we return Ok(vec![Ok]).
#[async_trait]
impl TrafficRepository for SqliteRepository {
    async fn apply_traffic_batch(
        &self,
        group_id: i64,
        entries: &[TrafficEntry],
    ) -> Result<Vec<TrafficEntryResult>, DbError> {
        let mut tx = self.pool.begin().await?;

        // ── Pass 1: validate u64→i64 per entry (a single entry's upload or
        // download alone can exceed i64::MAX; reject before any DB read). ──
        // Aggregate duplicate rule_ids INTO ONE delta first so the per-rule
        // overflow check below sees the true batch total, not a per-row slice.
        // (Rule keyed by id; we resolve owner/uid in pass 2.)
        let mut rule_delta: std::collections::HashMap<i64, (u64, u64)> =
            std::collections::HashMap::new();
        for entry in entries {
            if entry.upload > i64::MAX as u64 || entry.download > i64::MAX as u64 {
                let _ = tx.rollback().await;
                return Ok(vec![TrafficEntryResult::Overflow]);
            }
            let e = rule_delta.entry(entry.rule_id).or_insert((0, 0));
            e.0 = match e.0.checked_add(entry.upload) {
                Some(v) => v,
                None => {
                    let _ = tx.rollback().await;
                    return Ok(vec![TrafficEntryResult::Overflow]);
                }
            };
            e.1 = match e.1.checked_add(entry.download) {
                Some(v) => v,
                None => {
                    let _ = tx.rollback().await;
                    return Ok(vec![TrafficEntryResult::Overflow]);
                }
            };
        }

        // ── Pass 2: ownership + existing-value resolution.
        // SINGLE query per distinct rule_id: id+uid gated by device_group_in.
        // A miss means "not available to this node" (missing OR foreign) — we
        // do NOT run a second "does this id exist elsewhere?" query; that was
        // the rule-id existence oracle. The reason is logged, not returned.
        struct Resolved {
            rule_id: i64,
            uid: i64,
            delta_up: u64,
            delta_down: u64,
        }
        let mut resolved: Vec<Resolved> = Vec::with_capacity(rule_delta.len());
        // Track the per-USER aggregate delta (a user may own several rules in
        // this batch) for the cumulative overflow check.
        let mut user_delta: std::collections::HashMap<i64, i64> = std::collections::HashMap::new();
        for (rule_id, (dup, ddown)) in &rule_delta {
            // The rule's own delta must fit in i64 (upload+download summed).
            let rule_delta_sum = match dup.checked_add(*ddown) {
                Some(v) if v <= i64::MAX as u64 => v as i64,
                _ => {
                    let _ = tx.rollback().await;
                    return Ok(vec![TrafficEntryResult::Overflow]);
                }
            };
            let row: Option<(i64, i64, i64, i64)> = sqlx::query_as(
                "SELECT fr.id, fr.uid, fr.traffic_used, u.traffic_used \
                 FROM forward_rules fr \
                 JOIN users u ON u.id = fr.uid \
                 WHERE fr.id = ? AND fr.device_group_in = ?",
            )
            .bind(rule_id)
            .bind(group_id)
            .fetch_optional(&mut *tx)
            .await?;
            let Some((rid, uid, rule_used, user_used)) = row else {
                // Not available: missing OR foreign. Log the id (server-side
                // only) and roll the whole batch back with a uniform 403.
                tracing::warn!(
                    "traffic_batch: rule {} not available to group {} \
                     (missing or foreign) — rejecting batch",
                    rule_id,
                    group_id
                );
                let _ = tx.rollback().await;
                return Ok(vec![TrafficEntryResult::Unavailable]);
            };
            // Per-rule cumulative overflow: existing + this batch's delta.
            if rule_used.checked_add(rule_delta_sum).is_none() {
                let _ = tx.rollback().await;
                return Ok(vec![TrafficEntryResult::Overflow]);
            }
            // Per-user cumulative: existing user total + (running user delta).
            let cur_user_delta = *user_delta.get(&uid).unwrap_or(&0);
            let new_user_delta = match cur_user_delta.checked_add(rule_delta_sum) {
                Some(v) => v,
                None => {
                    let _ = tx.rollback().await;
                    return Ok(vec![TrafficEntryResult::Overflow]);
                }
            };
            if user_used.checked_add(new_user_delta).is_none() {
                let _ = tx.rollback().await;
                return Ok(vec![TrafficEntryResult::Overflow]);
            }
            user_delta.insert(uid, new_user_delta);
            resolved.push(Resolved {
                rule_id: rid,
                uid,
                delta_up: *dup,
                delta_down: *ddown,
            });
        }

        // ── Pass 3: apply writes for every resolved rule. We resolved against
        // the SAME tx, so a concurrent DELETE between passes still produces a
        // 0-rows-affected UPDATE (not an error). Duplicate rule_ids are already
        // aggregated, so each distinct rule gets ONE UPDATE (fewer SQL round
        // trips + no double-counting). ──
        for r in &resolved {
            let up = r.delta_up as i64;
            let down = r.delta_down as i64;
            sqlx::query(
                "UPDATE forward_rules SET traffic_used = traffic_used + ? + ? WHERE id = ?",
            )
            .bind(up)
            .bind(down)
            .bind(r.rule_id)
            .execute(&mut *tx)
            .await?;
            sqlx::query("UPDATE users SET traffic_used = traffic_used + ? + ? WHERE id = ?")
                .bind(up)
                .bind(down)
                .bind(r.uid)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        Ok(vec![TrafficEntryResult::Ok])
    }
}
