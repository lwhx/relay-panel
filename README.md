# RelayPanel

**English** | [中文](README.zh-CN.md)

[![CI](https://github.com/MoeShinX/relay-panel/actions/workflows/ci.yml/badge.svg)](https://github.com/MoeShinX/relay-panel/actions/workflows/ci.yml)
[![Debian Compat](https://github.com/MoeShinX/relay-panel/actions/workflows/debian-compat.yml/badge.svg)](https://github.com/MoeShinX/relay-panel/actions/workflows/debian-compat.yml)

A self-hosted **TCP/UDP forwarding management panel** built in Rust. Manage
port-forwarding rules, device groups, traffic accounting, and live node status
through a web UI — lightweight: one ~7 MB panel binary + a ~4 MB node binary
per forwarding host (Docker images ~140 MB).

**Target OS:** Linux (Debian 11 / 12 / 13) · **Deploy:** Docker Compose only ·
**Version:** `1.0.1`

---

## Architecture

```
 ┌─────────────┐    WebSocket (config push) + HTTP (status/traffic)   ┌──────────────┐
 │  Browser    +<-----+                                          +---<+ relay-node  │
 │  (React UI) │       │                                          │    │ (Tokio TCP/ │
 └─────────────┘       │   ┌──────────────────┐                    │    │  UDP engine)│
                       +--->│   relay-panel    │+<-------------------+    └──────────────┘
                           │  (Axum + SQLite) │              │
                           │  serves UI + API │              v
                           └──────────────────┘        forwards traffic
                                       ^               to real targets
                                       │
                              ┌────────┴────────┐
                              │  SQLite (data)  │
                              └─────────────────┘
```

- **Panel** — Axum HTTP server: serves the React SPA + REST API, persists state
  in SQLite. JWT auth, bcrypt passwords.
- **Node** — runs on each forwarding host. Opens TCP/UDP listeners, forwards
  traffic, reports status + traffic back. Outbound-only (no NAT traversal).
- **Config delivery** — WebSocket for real-time push (25 s heartbeat) + HTTP
  poll every 10 s as fallback. WS failure never stops forwarding.
- **Auth** — every node request carries `Authorization: Bearer <NODE_TOKEN>`
  (never in the query string, so it can't leak into access/proxy logs).
- **Accounting** — traffic is attributed to `rule_id` (not the listen port),
  then propagated to the rule and its owning user.

## Repository layout

```
relay-panel/
├── crates/
│   ├── shared/             # protocol types + DB models (panel + node)
│   ├── panel/              # the Axum panel binary
│   └── node/               # the Tokio forwarding node binary
├── frontend/               # React + TypeScript + antd SPA
├── docs/                   # user-facing docs (DEPLOYMENT, NODE, VERSIONS…)
├── scripts/                # install / release-check helpers
├── tests/e2e_test.py       # automated TCP+UDP forwarding test
├── install.sh              # one-line panel installer
├── deploy.sh               # panel deployer (pulls GHCR images + compose up)
├── docker-compose.yaml     # source-build compose
├── docker-compose.release.yaml  # pre-built-image compose
└── Caddyfile               # Caddy TLS reverse proxy (Compose profile)
```

## Quick start

**Production (one command — installs deps, clones, starts the panel):**

```bash
curl -fsSL https://raw.githubusercontent.com/MoeShinX/relay-panel/main/install.sh | bash
```

Full deployment guide (secrets, upgrades, reverse proxy, troubleshooting):
**[docs/DEPLOYMENT.md](docs/DEPLOYMENT.md)** ·
Reverse proxy guide: **[docs/REVERSE-PROXY.md](docs/REVERSE-PROXY.md)** ·
Forwarding node setup: **[docs/NODE.md](docs/NODE.md)**

> v0.4.15 added node-level GeoIP (a country flag next to each node). As of
> v0.4.16 it is **enabled by default**; v0.4.19 switched to built-in primary
> (ipinfo.io Lite) + fallback (ipwho.is) providers. To opt out set
> `GEOIP_ENABLED=false` — see
> [GeoIP setup](docs/DEPLOYMENT.md#geoip--node-region-resolution-optional-enabled-by-default-since-v0416).

> **Default login `admin` / `admin123` — the first login forces a password
> change, so pick a strong one.**
> See the [security checklist](docs/DEPLOYMENT.md#deploy-with-docker-compose).

**Local dev:**

```bash
cargo build && cargo run -p relay-panel &   # API on :18888
cd frontend && npm install && npm run dev   # UI on :5173 (proxies /api → :18888)
python3 tests/e2e_test.py                   # end-to-end TCP+UDP forwarding test
```

## Update

**Update an existing install** (pull new images + restart containers):

```bash
cd /opt/relay-panel && git pull --quiet && ./deploy.sh
```

> ⚠️ **We strongly recommend backing up your data before updating.** Copy your
> `.env` and your database (the `data/` directory for SQLite, or a `pg_dump` for
> PostgreSQL) somewhere safe first, so you can roll back if an upgrade goes wrong.

> Forwarding nodes update from the panel: **Device Groups → Copy Install Command**,
> paste it on the node (same command installs and upgrades). See
> [docs/NODE.md](docs/NODE.md#update).

## Tech stack

| Layer    | Choice                              |
|----------|--------------------------------------|
| Backend  | Rust, Axum 0.8, Tokio, sqlx, SQLite  |
| Auth     | JWT (jsonwebtoken), bcrypt           |
| Forward  | Tokio async TCP (`io::copy`) + UDP   |
| Frontend | React 19, TypeScript, antd 6, Vite   |
| Deploy   | Docker multi-stage, docker-compose   |

## Status

MVP, verified end-to-end. WebSocket real-time config + HTTP poll fallback,
per-rule traffic accounting, editable users/traffic reset, and live node status
(CPU/mem/conns/version). Pre-release — enforced
per-user quotas are future work.

## License & Disclaimer

AGPL-3.0 — see [LICENSE](LICENSE). Open-source traffic-forwarding tool for
**personal study and research only**; use lawfully and at your own risk. Full
**[Disclaimer](docs/DISCLAIMER.md)**.
