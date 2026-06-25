# Repository Test Parity

> **Goal:** Every `Repository` trait method must have equivalent behavioural tests in both
> `sqlite_repo/tests.rs` (49 tests) and `pg_repo/tests.rs` (44 tests, `pg_`-prefixed).
>
> **Status:** WIP — this document serves as the working baseline for gap analysis and
> incremental remediation. Last updated: 2026-06-24 (v0.4.18 PR8).

---

## How to read this document

- **SQLite** = `crates/panel/src/db/sqlite_repo/tests.rs` — runs on `sqlite::memory:`, always
  executes.
- **PG** = `crates/panel/src/db/pg_repo/tests.rs` — requires a real PostgreSQL instance;
  gated on env var `TEST_PG_URL`. When absent the test function returns early (skip).
- **CI coverage:** GitHub Actions spins up a `postgres:16-alpine` service and sets
  `TEST_PG_URL=postgres://relaytest:relaytest@localhost:5432/relaytest`, so PG tests **do
  execute on every PR** — they are not blindly skipped.
- **Normalised name:** PG tests drop the `pg_` prefix before comparison.
- **Allowlist:** Tests that exist on only one backend for a documented, non-drift reason
  (e.g. migration helpers that are inherently backend-specific) are kept in
  `scripts/repo-test-parity-allowlist.txt`. The CI guard ignores them.

---

## 1. Test inventory (before remediation)

### 1a. SQLite tests (`sqlite_repo/tests.rs`) — 49 functions

| # | Test function |
|---|---------------|
| 1 | `shared_groups_lists_admin_inbound_for_user_without_rules` |
| 2 | `shared_groups_excludes_non_inbound_types` |
| 3 | `shared_groups_excludes_other_regular_users_groups` |
| 4 | `shared_groups_empty_for_admin` |
| 5 | `rule_targets_replace_and_list_enabled_in_order` |
| 6 | `user_find_by_username_distinguishes_banned` |
| 7 | `user_insert_returns_unique_violation_on_duplicate` |
| 8 | `user_update_password_and_find_password_by_id_round_trip` |
| 9 | `user_update_fields_only_touches_present_columns` |
| 10 | `user_is_admin_and_exists_by_id_distinguish_known_rows` |
| 11 | `user_reset_traffic_zeros_user_and_owned_rules_atomically` |
| 12 | `user_delete_non_admin_protects_admins` |
| 13 | `user_delete_cascade_clears_rules_groups_profiles_and_user` |
| 14 | `user_delete_cascade_refuses_admin_and_rolls_back` |
| 15 | `user_placeholder_password_methods_round_trip` |
| 16 | `rule_insert_quota_guarded_respects_max_rules` |
| 17 | `rule_insert_quota_guarded_surfaces_port_unique_violation` |
| 18 | `rule_insert_quota_guarded_tcp_udp_share_port` |
| 19 | `rule_insert_quota_guarded_port_scoped_by_group` |
| 20 | `rule_update_rule_fields_partial_update` |
| 21 | `rule_list_active_for_config_filters_banned_paused_overquota` |
| 22 | `group_insert_then_find_by_token_round_trip` |
| 23 | `group_update_token_returns_rows_affected` |
| 24 | `traffic_batch_applies_to_rule_and_user` |
| 25 | `traffic_batch_other_group_rule_yields_othergrouprule_and_rolls_back` |
| 26 | `traffic_batch_unknown_rule_is_unavailable_not_skipped` |
| 27 | `traffic_batch_single_entry_overflow_rejects_and_rolls_back` |
| 28 | `traffic_batch_duplicate_rule_ids_cumulative_overflow` |
| 29 | `traffic_batch_user_cumulative_overflow_across_rules` |
| 30 | `traffic_batch_exactly_i64_max_is_accepted` |
| 31 | `traffic_batch_duplicate_rule_ids_are_aggregated` |
| 32 | `kvs_set_get_delete_round_trip` |
| 33 | `kvs_scan_prefix_returns_only_matching_keys` |
| 34 | `find_profile_by_id_builtin_only_excludes_custom` |
| 35 | `migration_does_not_pause_cross_owner_shared_inbound_rules` |
| 36 | `migration_pauses_non_admin_owner_custom_profile_rule` |
| 37 | `migration_does_not_pause_valid_rules` |
| 38 | `list_active_for_config_excludes_cross_owner_rule` |
| 39 | `settings_get_returns_none_when_unseeded` |
| 40 | `settings_insert_if_absent_is_idempotent` |
| 41 | `settings_set_upserts_when_no_row` |
| 42 | `insert_user_from_plan_inherits_quota_and_handles_missing_plan` |
| 43 | `migration_creates_app_settings_table` |
| 44 | `find_auth_state_returns_all_three_or_none` |
| 45 | `change_own_password_bumps_version_and_clears_must_change` |
| 46 | `admin_reset_password_bumps_version_and_sets_must_change` |
| 47 | `ban_bumps_token_version` |
| 48 | `unban_does_not_bump_token_version` |
| 49 | `migration_adds_password_columns` |

### 1b. PG tests (`pg_repo/tests.rs`) — 44 functions

| # | Test function | Normalised |
|---|---------------|------------|
| 1 | `pg_user_find_by_username_distinguishes_banned` | `user_find_by_username_distinguishes_banned` |
| 2 | `pg_user_insert_returns_unique_violation_on_duplicate` | `user_insert_returns_unique_violation_on_duplicate` |
| 3 | `pg_user_update_password_and_find_password_by_id_round_trip` | `user_update_password_and_find_password_by_id_round_trip` |
| 4 | `pg_user_update_fields_only_touches_present_columns` | `user_update_fields_only_touches_present_columns` |
| 5 | `pg_user_reset_traffic_zeros_user_and_owned_rules` | `user_reset_traffic_zeros_user_and_owned_rules` |
| 6 | `pg_rule_targets_replace_and_list_enabled_in_order` | `rule_targets_replace_and_list_enabled_in_order` |
| 7 | `pg_user_delete_non_admin_protects_admins` | `user_delete_non_admin_protects_admins` |
| 8 | `pg_delete_user_cascade_removes_rules_groups_profiles_and_user` | `delete_user_cascade_removes_rules_groups_profiles_and_user` |
| 9 | `pg_apply_schema_seeds_baseline_version` | `apply_schema_seeds_baseline_version` |
| 10 | `pg_delete_user_cascade_refuses_admin_and_rolls_back` | `delete_user_cascade_refuses_admin_and_rolls_back` |
| 11 | `pg_user_placeholder_password_methods_round_trip` | `user_placeholder_password_methods_round_trip` |
| 12 | `pg_rule_insert_quota_guarded_respects_max_rules` | `rule_insert_quota_guarded_respects_max_rules` |
| 13 | `pg_rule_insert_quota_guarded_surfaces_port_unique_violation` | `rule_insert_quota_guarded_surfaces_port_unique_violation` |
| 14 | `pg_rule_insert_quota_guarded_tcp_udp_share_port` | `rule_insert_quota_guarded_tcp_udp_share_port` |
| 15 | `pg_rule_insert_quota_guarded_port_scoped_by_group` | `rule_insert_quota_guarded_port_scoped_by_group` |
| 16 | `pg_rule_update_switch_to_direct_clears_device_group_out` | `rule_update_switch_to_direct_clears_device_group_out` |
| 17 | `pg_rule_list_active_for_config_filters_banned_paused_overquota` | `rule_list_active_for_config_filters_banned_paused_overquota` |
| 18 | `pg_group_insert_then_find_by_token_round_trip` | `group_insert_then_find_by_token_round_trip` |
| 19 | `pg_group_update_token_returns_rows_affected` | `group_update_token_returns_rows_affected` |
| 20 | `pg_traffic_batch_applies_to_rule_and_user` | `traffic_batch_applies_to_rule_and_user` |
| 21 | `pg_traffic_batch_other_group_rule_yields_othergrouprule_and_rolls_back` | `traffic_batch_other_group_rule_yields_othergrouprule_and_rolls_back` |
| 22 | `pg_traffic_batch_unknown_rule_is_unavailable_not_skipped` | `traffic_batch_unknown_rule_is_unavailable_not_skipped` |
| 23 | `pg_traffic_batch_single_entry_overflow` | `traffic_batch_single_entry_overflow` |
| 24 | `pg_traffic_batch_duplicate_rule_ids_cumulative_overflow` | `traffic_batch_duplicate_rule_ids_cumulative_overflow` |
| 25 | `pg_traffic_batch_user_cumulative_overflow_across_rules` | `traffic_batch_user_cumulative_overflow_across_rules` |
| 26 | `pg_traffic_batch_exactly_i64_max_is_accepted` | `traffic_batch_exactly_i64_max_is_accepted` |
| 27 | `pg_traffic_batch_duplicate_rule_ids_are_aggregated` | `traffic_batch_duplicate_rule_ids_are_aggregated` |
| 28 | `pg_kvs_set_get_delete_round_trip` | `kvs_set_get_delete_round_trip` |
| 29 | `pg_kvs_scan_prefix_returns_only_matching_keys` | `kvs_scan_prefix_returns_only_matching_keys` |
| 30 | `pg_find_profile_by_id_builtin_only_excludes_custom` | `find_profile_by_id_builtin_only_excludes_custom` |
| 31 | `pg_migration_pauses_cross_owner_rules` | `migration_pauses_cross_owner_rules` |
| 32 | `pg_migration_pauses_non_admin_owner_custom_profile_rule` | `migration_pauses_non_admin_owner_custom_profile_rule` |
| 33 | `pg_migration_does_not_pause_valid_rules` | `migration_does_not_pause_valid_rules` |
| 34 | `pg_list_active_for_config_returns_cross_owner_for_shared_inbound` | `list_active_for_config_returns_cross_owner_for_shared_inbound` |
| 35 | `pg_shared_groups_admin_inbound_only` | `shared_groups_admin_inbound_only` |
| 36 | `pg_settings_get_returns_none_when_unseeded` | `settings_get_returns_none_when_unseeded` |
| 37 | `pg_settings_insert_if_absent_is_idempotent` | `settings_insert_if_absent_is_idempotent` |
| 38 | `pg_settings_set_upserts_when_no_row` | `settings_set_upserts_when_no_row` |
| 39 | `pg_insert_user_from_plan_inherits_quota_and_handles_missing_plan` | `insert_user_from_plan_inherits_quota_and_handles_missing_plan` |
| 40 | `pg_find_auth_state_returns_all_three_or_none` | `find_auth_state_returns_all_three_or_none` |
| 41 | `pg_change_own_password_bumps_version_and_clears_must_change` | `change_own_password_bumps_version_and_clears_must_change` |
| 42 | `pg_admin_reset_password_bumps_version_and_sets_must_change` | `admin_reset_password_bumps_version_and_sets_must_change` |
| 43 | `pg_ban_bumps_token_version` | `ban_bumps_token_version` |
| 44 | `pg_unban_does_not_bump_token_version` | `unban_does_not_bump_token_version` |

---

## 2. Gap analysis (normalised, before remediation)

### 2a. Only in SQLite (12 tests)

| # | Test | Plan |
|---|------|------|
| 1 | `shared_groups_lists_admin_inbound_for_user_without_rules` | → port to PG as `pg_shared_groups_lists_admin_inbound_for_user_without_rules` |
| 2 | `shared_groups_excludes_non_inbound_types` | → port to PG as `pg_shared_groups_excludes_non_inbound_types` |
| 3 | `shared_groups_excludes_other_regular_users_groups` | → port to PG as `pg_shared_groups_excludes_other_regular_users_groups` |
| 4 | `shared_groups_empty_for_admin` | → port to PG as `pg_shared_groups_empty_for_admin` |
| 5 | `user_is_admin_and_exists_by_id_distinguish_known_rows` | → port to PG as `pg_user_is_admin_and_exists_by_id_distinguish_known_rows` |
| 6 | `user_reset_traffic_zeros_user_and_owned_rules_atomically` | → port to PG as `pg_user_reset_traffic_zeros_user_and_owned_rules_atomically` |
| 7 | `user_delete_cascade_clears_rules_groups_profiles_and_user` | → port to PG as `pg_user_delete_cascade_clears_rules_groups_profiles_and_user` |
| 8 | `rule_update_rule_fields_partial_update` | → port to PG as `pg_rule_update_rule_fields_partial_update` |
| 9 | `traffic_batch_single_entry_overflow_rejects_and_rolls_back` | → port to PG as `pg_traffic_batch_single_entry_overflow_rejects_and_rolls_back` |
| 10 | `migration_does_not_pause_cross_owner_shared_inbound_rules` | → port to PG as `pg_migration_does_not_pause_cross_owner_shared_inbound_rules` |
| 11 | `migration_creates_app_settings_table` | → **ALLOWLIST** — SQLite-only migration helper (PG has no in-schema migrations) |
| 12 | `migration_adds_password_columns` | → **ALLOWLIST** — SQLite-only migration helper |

### 2b. Only in PG (7 tests)

| # | Test | Plan |
|---|------|------|
| 1 | `apply_schema_seeds_baseline_version` | → **ALLOWLIST** — PG-only schema-seed guard (SQLite bootstraps via SCHEMA_SQL, no seed table) |
| 2 | `delete_user_cascade_removes_rules_groups_profiles_and_user` | → port to SQLite as `delete_user_cascade_removes_rules_groups_profiles_and_user` |
| 3 | `rule_update_switch_to_direct_clears_device_group_out` | → port to SQLite as `rule_update_switch_to_direct_clears_device_group_out` |
| 4 | `migration_pauses_cross_owner_rules` | → port to SQLite as `migration_pauses_cross_owner_rules` |
| 5 | `list_active_for_config_returns_cross_owner_for_shared_inbound` | → port to SQLite as `list_active_for_config_returns_cross_owner_for_shared_inbound` |
| 6 | `shared_groups_admin_inbound_only` | → port to SQLite as `shared_groups_admin_inbound_only` |
| 7 | `traffic_batch_single_entry_overflow` | → port to SQLite as `traffic_batch_single_entry_overflow` |

### 2c. Already in parity (37 pairs)

`user_find_by_username_distinguishes_banned`, `user_insert_returns_unique_violation_on_duplicate`,
`user_update_password_and_find_password_by_id_round_trip`, `user_update_fields_only_touches_present_columns`,
`user_reset_traffic_zeros_user_and_owned_rules`, `rule_targets_replace_and_list_enabled_in_order`,
`user_delete_non_admin_protects_admins`, `delete_user_cascade_refuses_admin_and_rolls_back`,
`user_placeholder_password_methods_round_trip`, `rule_insert_quota_guarded_respects_max_rules`,
`rule_insert_quota_guarded_surfaces_port_unique_violation`, `rule_insert_quota_guarded_tcp_udp_share_port`,
`rule_insert_quota_guarded_port_scoped_by_group`, `rule_list_active_for_config_filters_banned_paused_overquota`,
`group_insert_then_find_by_token_round_trip`, `group_update_token_returns_rows_affected`,
`traffic_batch_applies_to_rule_and_user`, `traffic_batch_other_group_rule_yields_othergrouprule_and_rolls_back`,
`traffic_batch_unknown_rule_is_unavailable_not_skipped`, `traffic_batch_duplicate_rule_ids_cumulative_overflow`,
`traffic_batch_user_cumulative_overflow_across_rules`, `traffic_batch_exactly_i64_max_is_accepted`,
`traffic_batch_duplicate_rule_ids_are_aggregated`, `kvs_set_get_delete_round_trip`,
`kvs_scan_prefix_returns_only_matching_keys`, `find_profile_by_id_builtin_only_excludes_custom`,
`migration_pauses_non_admin_owner_custom_profile_rule`, `migration_does_not_pause_valid_rules`,
`settings_get_returns_none_when_unseeded`, `settings_insert_if_absent_is_idempotent`,
`settings_set_upserts_when_no_row`, `insert_user_from_plan_inherits_quota_and_handles_missing_plan`,
`find_auth_state_returns_all_three_or_none`, `change_own_password_bumps_version_and_clears_must_change`,
`admin_reset_password_bumps_version_and_sets_must_change`, `ban_bumps_token_version`,
`unban_does_not_bump_token_version`

---

## 3. Allowlist

The following 3 tests are allowed to exist on only one backend. Each has a documented reason.

| Test | Backend | Reason |
|------|---------|--------|
| `migration_creates_app_settings_table` | SQLite | SQLite-only migration helper (PG initialises via `pg_schema.rs`, no in-schema migrations) |
| `migration_adds_password_columns` | SQLite | SQLite-only migration helper (PG schema already includes password columns from initial state) |
| `apply_schema_seeds_baseline_version` | PG | PG-only schema-seed guard (SQLite bootstraps via `SCHEMA_SQL` without a seed-version table) |

The allowlist file lives at `scripts/repo-test-parity-allowlist.txt`.

---

## 4. CI guard

The script `scripts/check-repo-test-parity.sh` is invoked:
- In `.github/workflows/ci.yml` after `cargo test --workspace`.
- In `scripts/refactor-check.sh` after `cargo test --workspace`.

**CI behaviour:** GitHub Actions runs PG tests against a real `postgres:16-alpine` service
(`TEST_PG_URL` is set). The parity check runs after tests pass, ensuring that any gap
introduced in a PR is caught before merge. The allowlist is consulted so
backend-inherent differences don't produce false alarms.

**Smoke test:** Deleting one test from either `tests.rs` must cause
`check-repo-test-parity.sh` to exit non-zero and print the missing test name.

---

## 5. High-risk methods with confirmed coverage

| Method | Risk | SQLite test | PG test |
|--------|------|-------------|---------|
| `insert_user_from_plan` | SQLite binds `plan_id` twice (positional `?`), PG reuses `$3` | ✅ `insert_user_from_plan_inherits_quota_and_handles_missing_plan` | ✅ `pg_insert_user_from_plan_inherits_quota_and_handles_missing_plan` |
| `increment_user_traffic` | Called inside traffic batch tx; checked_add overflow | ✅ Covered by traffic_batch_* overflow tests | ✅ Covered by pg_traffic_batch_* overflow tests |
| `apply_traffic_batch` | Multi-rule atomic increment; ownership check + write in same tx | ✅ 7 dedicated tests | ✅ 7 dedicated tests |
| `count_by_uid` / `max_rules_for_uid` | Quota enforcement called by `insert_quota_guarded` | ✅ Covered indirectly via rule_insert_quota_guarded_* | ✅ Covered indirectly via pg_rule_insert_quota_guarded_* |
| `delete_rule` (Owner scope) | Must reject rules owned by other users | 🆕 Added in PR8 | 🆕 Added in PR8 |
| `find_rule_by_id` (Owner scope) | Must return None for other users' rules | 🆕 Added in PR8 | 🆕 Added in PR8 |
| `update_group_fields` (Owner scope) | Must reject other users' groups | 🆕 Added in PR8 | 🆕 Added in PR8 |
| `delete_group` (Owner scope) | Must reject other users' groups | 🆕 Added in PR8 | 🆕 Added in PR8 |

---

## 6. How to run PG tests locally

```bash
# Start a throwaway PostgreSQL (Docker required)
docker run -d --name relay-pg-test \
  -e POSTGRES_USER=relaytest \
  -e POSTGRES_PASSWORD=relaytest \
  -e POSTGRES_DB=relaytest \
  -p 5432:5432 \
  postgres:16-alpine

# Wait for PG to accept connections
until docker exec relay-pg-test pg_isready -U relaytest; do sleep 1; done

# Run with PG backend enabled
TEST_PG_URL=postgres://relaytest:relaytest@localhost:5432/relaytest \
  cargo test --workspace

# Tear down
docker rm -f relay-pg-test
```

When `TEST_PG_URL` is unset, PG tests return early (`return;`) so a plain `cargo test
--workspace` still passes — but only SQLite tests actually exercise the backend.
