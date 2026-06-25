use crate::api::AppState;
use crate::service::password::{
    hash_password, validate_password, verify_password, PasswordValidationError,
};
use crate::service::users::validate_username;
use axum::{extract::State, Json};
use jsonwebtoken::{encode, EncodingKey, Header};
use once_cell::sync::Lazy;
use relay_shared::models::User;
use relay_shared::protocol::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Simple in-memory login rate limiter. Tracks per-username attempt counts
/// within a 1-minute window. Limits brute-force attacks without needing a
/// Redis/external store. Reset on process restart (acceptable: the attacker
/// has to re-establish state each time).
struct LoginAttempt {
    count: u32,
    window_start: Instant,
}

static LOGIN_LIMITER: Lazy<Mutex<HashMap<String, LoginAttempt>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

const MAX_LOGIN_ATTEMPTS: u32 = 5;
const LOGIN_WINDOW: Duration = Duration::from_secs(60);

/// Returns true if the login attempt should be blocked (too many attempts).
/// Resets the window if it has elapsed. Also prunes expired entries when the
/// map grows beyond CAP to prevent unbounded memory growth from crafted
/// usernames that never succeed (and thus never get remove()d).
fn check_rate_limit(username: &str) -> bool {
    const CAP: usize = 10_000;
    let now = Instant::now();
    let mut map = LOGIN_LIMITER.lock().unwrap();
    match map.get_mut(username) {
        Some(entry) if now.duration_since(entry.window_start) < LOGIN_WINDOW => {
            entry.count += 1;
            entry.count > MAX_LOGIN_ATTEMPTS
        }
        _ => {
            // New window (first attempt, or window expired)
            map.insert(
                username.to_string(),
                LoginAttempt {
                    count: 1,
                    window_start: now,
                },
            );
            // GC: drop entries whose window has expired to bound memory.
            if map.len() > CAP {
                map.retain(|_, v| now.duration_since(v.window_start) < LOGIN_WINDOW);
            }
            false
        }
    }
}

/// Clear the rate-limit counter for a username on successful login.
fn clear_rate_limit(username: &str) {
    LOGIN_LIMITER.lock().unwrap().remove(username);
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: i64, // user id
    admin: bool,
    // v0.4.10 PR4: session-version counter copied from users.token_version at
    // sign time. The auth middleware rejects a token whose token_version != the
    // current DB value, so bumping the DB column instantly revokes this token.
    token_version: i64,
    exp: usize,
}

/// Pre-computed dummy bcrypt hash used when the username does not exist.
/// Verifying against this eliminates the timing side-channel that would
/// otherwise reveal whether a username is registered (~300 ms bcrypt vs ~1 ms
/// early return).
static DUMMY_HASH: &str = "$2b$12$AAAAAAAAAAAAAAAAAAAAAOYEUtP4bEYKnMmFJEPW9HTZLX9R5gO4iSq";

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Json<ApiResponse<LoginResponse>> {
    // Rate limit: max 5 attempts per username per 60s.
    if check_rate_limit(&req.username) {
        return Json(ApiResponse {
            code: 429,
            message: "Too many login attempts. Please wait a minute and try again.".into(),
            data: None,
        });
    }

    let user: Option<User> = match state.db.find_by_username_not_banned(&req.username).await {
        Ok(u) => u,
        Err(e) => {
            tracing::error!("login: db lookup failed for {:?}: {}", req.username, e);
            None
        }
    };

    // Always perform a bcrypt verification to prevent timing attacks that
    // reveal whether a username exists. When the user is None we verify
    // against a pre-computed dummy hash so the CPU cost is identical.
    let (verified, user) = match user {
        Some(u) => (verify_password(&req.password, &u.password), Some(u)),
        None => {
            let _ = verify_password(&req.password, DUMMY_HASH);
            (false, None)
        }
    };

    if verified {
        if let Some(user) = user {
            clear_rate_limit(&req.username);
            let claims = Claims {
                sub: user.id,
                admin: user.admin,
                token_version: user.token_version,
                exp: chrono::Utc::now().timestamp_millis() as usize / 1000 + 86400,
            };
            let token = encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret(state.config.jwt_secret.as_bytes()),
            )
            .unwrap_or_default();

            return Json(ApiResponse::success(LoginResponse {
                token,
                admin: user.admin,
            }));
        }
    }

    Json(ApiResponse {
        code: 401,
        message: "Invalid credentials".into(),
        data: None,
    })
}

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Json<ApiResponse<()>> {
    // v0.4.10 PR3: registration toggle now lives in app_settings (admin-managed),
    // NOT the REGISTRATION_ENABLED env var. The env var only seeds the row on
    // first boot; afterwards only the admin PUT can change it. A missing row
    // (unseeded) is treated as "disabled" (safe default).
    let settings =
        match crate::service::settings::get_registration_settings(state.db.as_ref()).await {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("register: registration settings lookup failed: {}", e);
                return Json(ApiResponse {
                    code: 500,
                    message: "database error".into(),
                    data: None,
                });
            }
        };
    let enabled = settings.registration_enabled;
    if !enabled {
        return Json(ApiResponse {
            code: 403,
            message: "Registration is disabled. Ask an admin to create your account.".into(),
            data: None,
        });
    }

    // v0.4.21 PR2: resolve plan_id from request, falling back to the default.
    // Validate it is in the allowed list.
    let selected_plan_id = req.plan_id.unwrap_or(settings.default_registration_plan_id);
    if !settings.allowed_plan_ids.contains(&selected_plan_id) {
        return Json(ApiResponse {
            code: 400,
            message: "Selected plan is not available for registration.".into(),
            data: None,
        });
    }

    // Validate username: non-empty, ≤64 chars, ASCII alphanumeric + underscore.
    // Prevents table rendering breakage and DB bloat from absurd inputs.
    if !validate_username(&req.username) {
        return Json(ApiResponse {
            code: 400,
            message: "Username must be 1-64 chars, ASCII letters/digits/underscore only".into(),
            data: None,
        });
    }

    // v0.4.10 PR3: password length validation. bcrypt truncates at 72 bytes,
    // so anything longer is silently weakened; anything shorter than 8 is
    // trivially brute-forced. len() is UTF-8 bytes (matches bcrypt's boundary).
    if let Err(e) = validate_password(&req.password) {
        return Json(ApiResponse {
            code: 400,
            message: match e {
                PasswordValidationError::TooShort => "Password must be at least 8 characters",
                PasswordValidationError::TooLong => {
                    "Password must be at most 72 bytes (bcrypt limit)"
                }
            }
            .into(),
            data: None,
        });
    }

    let hashed = match hash_password(&req.password) {
        Ok(h) => h,
        Err(e) => {
            return Json(ApiResponse {
                code: 500,
                message: format!("Failed to hash password: {}", e),
                data: None,
            });
        }
    };

    // v0.4.10 PR3: insert_user_from_plan atomically copies the plan's quota
    // fields (max_rules/traffic_limit/speed_limit/ip_limit) via INSERT...SELECT,
    // closing the "validate plan then plan changes" race. The default plan_id
    // comes from app_settings. Match order matters:
    //   Ok(1)                    → registered
    //   Ok(0)                    → plan missing (deleted out from under us) → 500
    //   Err(UniqueViolation)     → concurrent same-username register → 409
    //   Err(other)               → 500
    let plan_id = selected_plan_id;
    match state
        .db
        .insert_user_from_plan(&req.username, &hashed, plan_id)
        .await
    {
        Ok(1) => Json(ApiResponse::success(())),
        Ok(0) => {
            tracing::error!(
                "register: default plan {} is missing; no user created",
                plan_id
            );
            Json(ApiResponse {
                code: 500,
                message: "Registration is misconfigured (default plan missing). \
                          Contact an administrator."
                    .into(),
                data: None,
            })
        }
        Ok(_) => Json(ApiResponse {
            // Should not happen for a single-row insert; defensive.
            code: 500,
            message: "database error".into(),
            data: None,
        }),
        Err(crate::db::error::DbError::UniqueViolation) => Json(ApiResponse {
            code: 409,
            message: "Username already exists".into(),
            data: None,
        }),
        Err(e) => {
            tracing::error!("register: insert failed for {:?}: {}", req.username, e);
            Json(ApiResponse {
                code: 500,
                message: "database error".into(),
                data: None,
            })
        }
    }
}

/// v0.4.10 PR3 / v0.4.21 PR2: public registration-status probe.
/// Unauthenticated (used by the login page to decide whether to show
/// the "create account" link and the registration page to render a plan
/// selector). Returns enabled flag, default_plan_id, and the list of
/// allowed plans (filtered from the full plans table).
///
/// A DB error is surfaced as 500 (NOT masqueraded as "disabled"), so a panel
/// outage doesn't make users think registration is closed.
pub async fn registration_status(
    State(state): State<AppState>,
) -> Json<ApiResponse<RegistrationStatus>> {
    let settings =
        match crate::service::settings::get_registration_settings(state.db.as_ref()).await {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("registration_status: settings lookup failed: {}", e);
                return Json(ApiResponse {
                    code: 500,
                    message: "database error".into(),
                    data: None,
                });
            }
        };

    let allowed_set: std::collections::HashSet<i64> =
        settings.allowed_plan_ids.iter().copied().collect();

    let all_plans: Vec<relay_shared::models::Plan> = match state.db.list_plans().await {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("registration_status: list_plans failed: {}", e);
            return Json(ApiResponse {
                code: 500,
                message: "database error".into(),
                data: None,
            });
        }
    };

    let plans: Vec<relay_shared::models::Plan> = all_plans
        .into_iter()
        .filter(|p| allowed_set.contains(&p.id))
        .collect();

    Json(ApiResponse::success(RegistrationStatus {
        enabled: settings.registration_enabled,
        default_plan_id: settings.default_registration_plan_id,
        plans,
        default_password_change_required: default_password_change_required(state.db.as_ref()).await,
    }))
}

/// v0.4.22: check whether the default admin (id=1) still has
/// must_change_password set. Used by the login page to decide whether
/// to show the security reminder banner.
async fn default_password_change_required(db: &dyn crate::db::repo::Repository) -> bool {
    match db.find_auth_state_by_id(1).await {
        Ok(Some((_banned, _version, must_change))) => must_change,
        // User doesn't exist (fresh DB before seed) or DB error → no banner.
        _ => false,
    }
}
