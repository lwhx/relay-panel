# relay-panel v0.4.x Roadmap

Planning document for the v0.4.x development line. Each version is one or more
PRs → a single tag → a deploy-verify cycle; we do not start the next version
until the current one passes its acceptance checks.

Status legend: ☐ planned · ◐ in progress · ☑ done (carry-over from v0.3.x).

---

## Guiding principles

1. **Protocol is three orthogonal fields**, never conflated:
   - **Business protocol** (what flows over the tunnel): `tcp` | `udp` | `tcp_udp`.
   - **Route mode** (forwarding topology): `direct` | `group` | `chain`.
   - **Transport** (how clients reach a listener), split into two views:
     - `public_transport`: what the user picks in the UI — `raw` | `ws` | `tls_simple`.
     - `node_transport`: what the node actually listens on, **derived by the
       panel and sent explicitly** — `raw` | `ws` | `tls_simple`.
   - The node NEVER guesses; the panel derives `node_transport` and sends it
     as an explicit field.
2. **SQLite stays the default.** No version may break the single-binary SQLite
   deployment. PostgreSQL (v0.4.3) and any future backend are opt-in.
3. **Each major version ships one tag.** Intra-version work is organized as
   internal PRs but the published artifact is a single `vX.Y.Z` tag. Only
   major versions get a full GitHub Release; **any fix that touches binaries,
   images, or the install script MUST also publish a Release** (a tag alone is
   not enough — the installer depends on Release assets).
4. **Forward-compat is governed by `config_protocol_version`, not the product
   version.** See the compatibility section below.

### Business WSS is REMOVED (v0.4.1 cleanup)

v0.4.0 shipped WSS (WebSocket Secure via reverse proxy) as a business ingress
transport. This has been **cancelled** — the reverse-proxy chain is too long
and error-prone, and TLS Simple (v0.4.1) covers the "encrypted TCP" need
better. v0.4.1 removes all WSS code:

- ❌ `PublicTransport::Wss` enum variant
- ❌ `wss → ws` derivation logic
- ❌ `wss-via-caddy` builtin tunnel profile
- ❌ `docs/WSS-REVERSE-PROXY.md` and all Caddy/Nginx WSS examples
- ❌ Node Edge Caddy automation (was v0.4.4 — now deleted entirely)
- ❌ WSS-related acceptance criteria and stress tests

**Control channel** (relay-node ↔ panel WebSocket/HTTP polling) is **NOT**
affected — that's a separate concern and stays as-is.

---

## Control channels (relay-node ↔ panel) — NOT changing

The relay-node ↔ panel communication stays exactly as it is:

- **WebSocket** (primary): real-time config push, heartbeat. relay-node connects
  to the panel's `/node/ws` endpoint.
- **HTTP polling** (fallback): `get_config` every N seconds. Used when WS is
  unavailable (panel restart, network issue).
- **config_protocol_version gate** (v0.4.0): both paths check the protocol
  version header and refuse on mismatch (426).

These control channels are orthogonal to the **business forwarding** transports
listed above. This roadmap only touches business forwarding.

---

## Compatibility strategy (v0.4.x = development line)

**v0.4.x is a development line. It does NOT guarantee that an old relay-node
works with a new panel.** However, compatibility is decided by
`config_protocol_version`, NOT the product version:

- Panel + node carry `CONFIG_PROTOCOL_VERSION: u32`.
- Node reports it in the `X-Config-Protocol-Version` header on get_config + WS.
- Panel refuses on mismatch: **426 Upgrade Required** + structured JSON
  (`{code, required, received, message}`). Node keeps cached config + backs off.
- Within the same protocol version, panel and node releases are interoperable
  even if the product version differs.

**v1.0** starts a fresh stable release sequence after the dev line is complete.

---

## Protocol → certificate responsibility

| Transport | Uses TLS? | Who handles the cert |
|---|---|---|
| `raw` (tcp/udp) | No | — |
| `ws` | No | — (plaintext) |
| `tls_simple` (ingress) | Yes, terminated at relay-node | relay-node is the TLS **server**: loads cert+key, presents cert, completes handshake. The **client** validates the cert (CA, expiry, SNI). relay-node does NOT validate the client's trust. |
| `tls_simple` (inter-node / chain, future) | Yes | relay-node is the TLS **client**: validates peer cert (CA, expiry, SNI, chain). |

---

## Delivered in v0.4.x vs deferred

| Item | Status / Deferred to | Reason |
|---|---|---|
| **Panel Caddy (admin-UI HTTPS)** | **Delivered in v0.4.5** | Compose-managed Caddy profile with domain, ACME, cert volumes, port pre-flight, and panel localhost binding. TLS Simple (node-side) remains separate. |
| Business WSS (WebSocket over TLS via reverse proxy) | **Never** | Cancelled. TLS Simple covers encrypted TCP. WS stays plaintext. |
| Node Edge Caddy automation | **Never** | Was for business WSS. Cancelled with WSS. |
| WSS direct (relay-node does TLS on a WS listener) | **Never** | Cancelled with WSS. |
| Multi-tenant full chain (per-uid rule isolation) | v1.0+ | Needs query audit. |
| Payments / plans / commercialization | v1.0+ | Product decision. |
| Caddy Layer-4 custom image | v0.5+ | Custom build needed. |
| relay-node ACME (auto cert issuance) | v0.5+ | Adds egress + state. |
| MySQL / MariaDB | v0.5+ | Three dialects too costly. |
| Auto online upgrade (in-place binary swap) | v0.5+ | Needs signed-update channel. |
| Real per-user speed_limit / ip_limit enforcement | v0.5+ | Placeholder fields, never enforced. |

---

## Release sequence (adjusted)

```
v0.4.0  config-flow + protocol gate                    ☑ DONE (published)
        NOTE: shipped WSS — to be removed in v0.4.1.
v0.4.1  TLS Simple + WSS removal                       ☑ DONE (published)
        (node rustls terminates TLS for tcp; strip all WSS code)
v0.4.2  Hotfix for v0.4.1 post-release findings        ☑ DONE (published)
        (start.sh unbound var, CertReloader recovery,
         Migration 18 transactional, wss-transport guard)
        Originally planned as "Panel Caddy" — that feature was deferred
        from v0.4.2 and later delivered in v0.4.5.
	v0.4.3  PostgreSQL (DB abstraction — stands alone)     ☑ DONE (published)
	v0.4.4  Hotfix release (PostgreSQL hardening + admin UI) ☑ DONE (published)
	v0.4.5  Current release (deployment hardening + web modes)
	```

Node Edge Caddy (was v0.4.4) is **deleted**. Wrap-up moved from v0.4.5 to
v0.4.4. **Panel Caddy (admin-UI HTTPS) was NOT implemented in v0.4.2**, but
v0.4.5 now ships the Compose-managed baseline (domain, ACME, cert volumes,
port pre-flight, and localhost panel binding).

v0.4.5 scope: deployment hardening. Web access modes (direct / reverse-proxy
/ Caddy Compose profile), port pre-flight, profile arrays in deploy.sh, and
reverse-proxy documentation. No new backend features.

### Numbering rule for the next version (v0.4.4)

The v0.4.4 slot is **not pre-committed** to any one scope. Decide its content
by what is actually ready when it is time to cut it:

- **If the next release is fix-driven** (deployment feedback, regression
  hotfix), it ships as **v0.4.4** and the **Wrap-up is deferred to v0.4.5**.
  This mirrors how v0.4.2 became a hotfix instead of its planned feature.
- **If the next release carries the Wrap-up batch** (SQLite backup/migration,
  tunnel health, cert countdown, stress-test SLOs, DR docs), it ships as
  **v0.4.4** and is the closing v0.4.x release.
- **Panel Caddy** shipped in **v0.4.5** as a Compose-managed baseline.
  Future enhancements can live in v0.5.x.

Do not publish an empty version: if nothing of substance (binary / image /
install-script change) is ready, no tag is cut.

---

## v0.4.0 — DONE (published)

Config-flow + protocol gate. Shipped WSS as a business transport, but WSS is
**cancelled** and will be removed in v0.4.1. The protocol gate, three-field
split, tunnel profiles CRUD, and config_protocol_version mechanism remain.

---

## v0.4.1 — TLS Simple + WSS removal

**Goal:** (1) remove all business WSS code; (2) add TLS Simple ingress (node
terminates TLS directly via rustls, no WebSocket, no reverse proxy).

### Part A — WSS removal (cleanup)

- ☐ Delete `PublicTransport::Wss` variant + `from_db_str`/`to_db_str`/`derive_node_transport` arms.
- ☐ Delete `wss-via-caddy` builtin tunnel profile (seed Migration 18 adjusts
  the seed to remove it; existing DB rows with `transport='wss'` are left but
  ignored).
- ☐ Delete `docs/WSS-REVERSE-PROXY.md`.
- ☐ Frontend: remove WSS from `transportOptions`; remove `entryTransportWss`
  i18n keys; ws_path gating no longer checks `wss`.
- ☐ `is_public_transport_accepted`: remove `Wss` from the accepted set.
- ☐ Bump `CONFIG_PROTOCOL_VERSION` to 2 (the `Wss` variant removal is a wire
  change — old nodes that know `Wss` would fail on its absence, so the gate
  must refuse them).

### Part B — TLS Simple (new feature)

- ☐ `crates/node/Cargo.toml`: add `tokio-rustls`, `rustls`, `rustls-pemfile`.
- ☐ New `crates/node/src/forwarder/tls.rs`: `start_tls_listener` — accepts TCP,
  upgrades to TLS via `TlsAcceptor`, then pumps like tcp.rs.
- ☐ `manager.rs`: replace the "TlsSimple skipped" guard with an actual spawn
  arm `(Protocol::Tcp, NodeTransport::TlsSimple)`.
- ☐ Cert loading: env vars `TLS_CERT_PATH` + `TLS_KEY_PATH` (global, one cert
  per node). Panel does NOT manage cert files.
- ☐ Cert hot reload: mtime poll (5s); on change re-read + re-parse; on failure
  keep old cert + emit listener_error. Atomic swap via `Arc<RwLock<Arc<ServerConfig>>>`.
- ☐ Private-key permission check: refuse if mode allows group/other read (not 0600).
- ☐ Min TLS version: TLS 1.2 (1.0/1.1 rejected via `protocol_versions`).
- ☐ UDP + tls_simple rejected (frontend disables option + node guards).
- ☐ Panel: `is_public_transport_accepted` accepts `TlsSimple` (TCP only).
- ☐ Panel: `validate_protocol_transport` rejects `tls_simple` + non-TCP.
- ☐ Frontend: `tls_simple` enabled in `transportOptions` (TCP rules only).
- ☐ `docs/TLS-SIMPLE.md`: cert generation (self-signed + certbot), `openssl
  s_client` test, hot-reload behavior, permission requirements.

### Acceptance

- WSS option is gone from the UI; existing wss rules are ignored (not crashed on).
- `PublicTransport::Wss` no longer exists in code.
- TLS handshake succeeds → raw TCP forwards.
- Bad cert/key is visible on the panel (listener_error reported).
- Replacing cert file hot-reloads without node restart.
- Private key never in any API response or log.
- UDP + tls_simple rejected everywhere.
- TLS 1.0/1.1 refused.
- `CONFIG_PROTOCOL_VERSION` bumped to 2; v0.4.0 nodes are gated (426) until
  upgraded.

---

## v0.4.2 — DONE (published): Hotfix for v0.4.1

**Actual scope:** hotfix release closing post-release review findings on v0.4.1.
The v0.4.1 binaries themselves are usable; this release fixed a startup crash
in the generated `start.sh`, a `CertReloader` that could never recover from an
initial failure, a non-transactional migration, and a wss-transport guard gap.
It also deleted the `PublicTransport::Wss` enum variant entirely (v0.4.1 had
kept it parseable).

See `CHANGELOG.md` § [0.4.2] for the authoritative list.

### Panel Caddy — delivered in v0.4.5 as Compose-managed Caddy

This work was **originally planned as "Panel Caddy" (HTTPS for the admin
UI)** for v0.4.2, but was postponed until v0.4.5. It is more than "add a
Caddy container" — the delivered v0.4.5 baseline includes a complete story for:

- domain + email configuration,
- ACME auto-issuance and renewal,
- 80/443 port occupancy / conflicts with an existing Nginx or Caddy,
- Caddy data-volume persistence,
- no-domain / IP-only deployments,
- switching between an external reverse proxy and the built-in one,
- install / upgrade / uninstall / recovery.

That was treated as deployment hardening for v0.4.5. Users may still put their
own Nginx / Caddy in front of the panel instead. **Note:** TLS Simple (v0.4.1)
is node-side business forwarding encryption — it is a different concern from
panel admin-UI HTTPS and does not substitute for panel admin HTTPS.

---

## v0.4.3 — PostgreSQL

**Goal:** SQLite default; PostgreSQL opt-in. Stands alone.

### Work items

- ☐ Layer DB init/migration (dialect split).
- ☐ Rewrite SQLite-isms: `datetime('now')`→`NOW()`, `INSERT OR IGNORE`→`ON CONFLICT`,
  `AUTOINCREMENT`→`SERIAL`, `?`→`$1`, error codes, isolation levels.
- ☐ Config: `DATABASE_URL=sqlite:...` or `DATABASE_URL=postgres://...`.
- ☐ Optional PostgreSQL Compose profile.
- ☐ Same API + business tests on both backends.

### Acceptance

- Identical behaviour on SQLite and PG.
- Concurrent traffic reporting, rollback, quota atomicity all pass on PG.

---

## Wrap-up (v0.4.5 — deployment hardening; remaining items → v0.5.x)

**Goal:** make v0.4.x ready for public recommendation.

> **v0.4.4 shipped** as a hotfix release (PostgreSQL hardening + admin UI).
> **v0.4.5** covers deployment hardening: web access modes, Caddy Compose
> profile, port pre-flight, and reverse-proxy documentation. The remaining
> Wrap-up items below are deferred to v0.5.x.

> **Note (2026-06-20):** the four items below were originally listed here as
> ☑ done, but they were actually delivered as part of **v0.4.3** (PostgreSQL +
> `deploy.sh` database-mode selection). They are kept here only as a reference
> of what is already in place; they are **not** Wrap-up scope. The remaining
> unchecked items are the real Wrap-up work.

### Work items

- *(done in v0.4.3)* DATABASE_URL as canonical env var (DATABASE_PATH legacy
  fallback).
- *(done in v0.4.3)* deploy.sh database mode selection (SQLite / embedded PG /
  external PG).
- *(done in v0.4.3)* docker-compose postgres profile + healthcheck.
- *(done in v0.4.3)* External PG connectivity pre-flight with real psql query
  (fail-fast, no fallback).
- ☐ SQLite export / backup / restore.
- ☐ SQLite → PostgreSQL migration tool.
- ☐ Tunnel health: online status, handshake errors, latency.
- ☐ Certificate expiry countdown (for tls_simple rules).
- ☐ Distinguish listener-failure vs tunnel-failure in UI.
- ☐ Stress tests with defined SLOs: machine spec, duration, CPU/RAM ceiling,
  failure-rate threshold. Transports tested: raw tcp/udp, ws, tls_simple.
- ☐ Full upgrade / rollback / disaster-recovery docs.
- ☐ Pre-release version-consistency + image verification.

### Acceptance

- All pass on both SQLite and PG.
- A v0.3.x deployment upgrades to the final v0.4.x with zero data loss.
