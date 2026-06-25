//! Device group + rule deletion service.
//!
//! Extracted from `api/admin/{groups,rules}.rs`. Houses the group CRUD business
//! rules (admin-owned token generation, no-fields guard, 404-on-zero-rows) and
//! the rule deletion mutation. The handlers keep HTTP concerns + the
//! connection-manager side effects (`close_group` / `broadcast_all`) and audit
//! logging — those depend on `node_connections`, not the `Repository`.

use crate::db::error::DbError;
use crate::db::repo::ResourceScope;
use crate::db::Repository;
use crate::service::rules::group_type_to_str;
use relay_shared::models::DeviceGroup;
use relay_shared::protocol::GroupType;

#[derive(Debug)]
pub enum CreateGroupError {
    /// INSERT succeeded but the follow-up SELECT-by-token found nothing.
    FetchFailed,
    Database(DbError),
}

#[derive(Debug)]
pub enum UpdateGroupError {
    NotFound,
    NoFields,
    Database(DbError),
}

/// Create an admin-owned device group. Generates a fresh token, inserts, then
/// returns the persisted row (INSERT-then-SELECT-by-token; the token is a
/// freshly generated UUID so the SELECT is guaranteed to hit the new row).
///
/// v0.4.12 PR1: device groups are admin-managed shared infrastructure — the
/// caller passes the creating admin's id as `owner_uid` (the handler ignores
/// any client-supplied owner_uid).
pub async fn create_group(
    db: &dyn Repository,
    name: &str,
    group_type: &GroupType,
    owner_uid: i64,
    connect_host: &str,
    port_range: &str,
) -> Result<DeviceGroup, CreateGroupError> {
    let token = uuid::Uuid::new_v4().to_string();
    let group_type = group_type_to_str(group_type);
    db.insert_group(
        name,
        group_type,
        &token,
        owner_uid,
        connect_host,
        port_range,
    )
    .await
    .map_err(CreateGroupError::Database)?;

    match db.find_by_token_after_insert(&token).await {
        Ok(Some(g)) => Ok(g),
        Ok(None) => Err(CreateGroupError::FetchFailed),
        Err(e) => Err(CreateGroupError::Database(e)),
    }
}

/// Rotate a device group's node token. Generates a fresh UUID and persists it.
/// Returns `Ok(Some(new_token))` when a row changed, `Ok(None)` when the group
/// didn't exist (the handler maps that to 404). The connection teardown
/// (`close_group`) + broadcast stay in the handler.
pub async fn rotate_group_token(db: &dyn Repository, id: i64) -> Result<Option<String>, DbError> {
    // v0.4.12 PR1: admin-only. Scope All — an admin operates on any group.
    let new_token = uuid::Uuid::new_v4().to_string();
    match db
        .update_group_token(id, &ResourceScope::All, &new_token)
        .await?
    {
        0 => Ok(None),
        _ => Ok(Some(new_token)),
    }
}

/// Update an admin-owned device group. Enforces the no-fields guard and
/// 404-on-zero-rows. The token is NOT updatable here (rotation is a separate
/// endpoint).
pub async fn update_group(
    db: &dyn Repository,
    id: i64,
    name: Option<&str>,
    group_type: Option<&GroupType>,
    connect_host: Option<&str>,
    port_range: Option<&str>,
) -> Result<(), UpdateGroupError> {
    if name.is_none() && group_type.is_none() && connect_host.is_none() && port_range.is_none() {
        return Err(UpdateGroupError::NoFields);
    }

    match db
        .update_group_fields(
            id,
            &ResourceScope::All,
            name,
            group_type.map(group_type_to_str),
            connect_host,
            port_range,
        )
        .await
        .map_err(UpdateGroupError::Database)?
    {
        0 => Err(UpdateGroupError::NotFound),
        _ => Ok(()),
    }
}

/// Delete an admin-owned device group. Returns `Ok(true)` when a row was
/// deleted, `Ok(false)` when nothing existed at that id (the handler maps that
/// to 404 + no broadcast).
pub async fn delete_group(db: &dyn Repository, id: i64) -> Result<bool, DbError> {
    Ok(db.delete_group(id, &ResourceScope::All).await? > 0)
}

/// Delete a rule within `scope` (owner-scoped for regular users, All for
/// admins). Returns `Ok(true)` when a row was deleted, `Ok(false)` when nothing
/// matched (the handler maps that to 404 + no broadcast).
pub async fn delete_rule(
    db: &dyn Repository,
    id: i64,
    scope: &ResourceScope,
) -> Result<bool, DbError> {
    Ok(db.delete_rule(id, scope).await? > 0)
}
