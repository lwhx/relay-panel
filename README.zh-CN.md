# RelayPanel

[English](README.md) | **中文**

[![CI](https://github.com/MoeShinX/relay-panel/actions/workflows/ci.yml/badge.svg)](https://github.com/MoeShinX/relay-panel/actions/workflows/ci.yml)
[![Debian Compat](https://github.com/MoeShinX/relay-panel/actions/workflows/debian-compat.yml/badge.svg)](https://github.com/MoeShinX/relay-panel/actions/workflows/debian-compat.yml)

自托管的 **TCP/UDP 端口转发管理面板**，用 Rust 编写。通过 Web UI 管理端口转发规则、
设备分组、流量统计和实时节点状态 —— 轻量：单个约 7 MB 的 panel 二进制 + 约 4 MB 的 node 二进制（Docker 镜像约 140 MB）。

**目标系统：** Linux（Debian 11 / 12 / 13） · **部署：** 仅 Docker Compose ·
**当前版本：** `1.0.1`

---

## 架构

```
 ┌─────────────┐    WebSocket（配置推送）+ HTTP（状态/流量上报）   ┌──────────────┐
 │  浏览器     │◄──────┐                                          ┌───►│ relay-node  │
 │  (React UI) │       │                                          │    │ (Tokio TCP/ │
 └─────────────┘       │   ┌──────────────────┐                    │    │  UDP 引擎)  │
                       └──►│   relay-panel    │◄────────────────────┘    └──────────────┘
                           │ (Axum + SQLite)  │              │
                           │ 提供 UI + API    │              ▼
                           └──────────────────┘       转发流量到真实目标
                                       ▲
                                       │
                              ┌────────┴────────┐
                              │  SQLite (数据)  │
                              └─────────────────┘
```

- **Panel** — Axum HTTP 服务器：提供 React SPA + REST API，用 SQLite 持久化状态。JWT 鉴权，bcrypt 密码哈希。
- **Node** — 运行在每个转发主机上。开启 TCP/UDP 监听器转发流量，回报状态与流量。仅需对外访问（无 NAT 穿透）。
- **配置下发** — WebSocket 实时推送（25 秒心跳）+ HTTP 每 10 秒轮询兜底。WS 失败绝不中断转发。
- **鉴权** — Node 每个请求带 `Authorization: Bearer <NODE_TOKEN>`（绝不放查询字符串，避免泄露到访问/代理日志）。
- **计费** — 流量按 `rule_id` 归因（不是监听端口），再同步累加到规则和所属用户。

## 仓库结构

```
relay-panel/
├── crates/
│   ├── shared/             # 协议类型 + 数据库模型（panel + node 共享）
│   ├── panel/              # Axum panel 二进制
│   └── node/               # Tokio 转发节点二进制
├── frontend/               # React + TypeScript + antd 单页应用
├── docs/                   # 用户文档（DEPLOYMENT、NODE、VERSIONS…）
├── scripts/                # 安装 / release-check 辅助脚本
├── tests/e2e_test.py       # TCP + UDP 转发自动化测试
├── install.sh              # 一行 panel 安装器
├── deploy.sh               # panel 部署器（拉取 GHCR 镜像 + compose up）
├── docker-compose.yaml     # 源码构建 compose
├── docker-compose.release.yaml  # 预构建镜像 compose
└── Caddyfile               # Caddy TLS 反向代理（Compose profile）
```

## 快速开始

**生产部署（一条命令 —— 自动装依赖、克隆、启动 panel）：**

```bash
curl -fsSL https://raw.githubusercontent.com/MoeShinX/relay-panel/main/install.sh | bash
```

完整部署指南（密钥、升级、反向代理、故障排查）：
**[docs/DEPLOYMENT.md](docs/DEPLOYMENT.md)** ·
反向代理指南：**[docs/REVERSE-PROXY.md](docs/REVERSE-PROXY.md)** ·
转发节点安装：**[docs/NODE.zh-CN.md](docs/NODE.zh-CN.md)**

> v0.4.15 新增节点级 GeoIP（在每个节点旁显示国家/地区旗标）。自 v0.4.16 起
> **默认开启**；v0.4.19 切换为内置主源（ipinfo.io Lite）+ 备用源（ipwho.is）。
> 如需关闭请设置 `GEOIP_ENABLED=false` —— 详见
> [GeoIP 配置](docs/DEPLOYMENT.md#geoip--node-region-resolution-optional-enabled-by-default-since-v0416)。

> **默认账号 `admin` / `admin123`，首次登录会强制要求修改密码，请设置强口令。**
> 详见 [安全检查清单](docs/DEPLOYMENT.md#deploy-with-docker-compose)。

**本地开发：**

```bash
cargo build && cargo run -p relay-panel &   # API 在 :18888
cd frontend && npm install && npm run dev   # UI 在 :5173（代理 /api → :18888）
python3 tests/e2e_test.py                   # 端到端 TCP+UDP 转发测试
```

## 更新

**更新已有部署**（拉取新镜像 + 重启容器）：

```bash
cd /opt/relay-panel && git pull --quiet && ./deploy.sh
```

> ⚠️ **强烈建议您更新前备份数据。** 请先把 `.env` 和数据库（SQLite 为 `data/`
> 目录，PostgreSQL 用 `pg_dump`）复制到安全位置，以便升级出问题时回滚。

> 转发节点更新：在面板「设备分组 → 复制对接命令」，粘贴到节点服务器执行
> （同一条命令兼顾安装与升级）。详见 [docs/NODE.zh-CN.md](docs/NODE.zh-CN.md#更新)。

## 技术栈

| 层级     | 选型                                |
|----------|--------------------------------------|
| 后端     | Rust, Axum 0.8, Tokio, sqlx, SQLite  |
| 鉴权     | JWT (jsonwebtoken), bcrypt           |
| 转发     | Tokio 异步 TCP (`io::copy`) + UDP    |
| 前端     | React 19, TypeScript, antd 6, Vite   |
| 部署     | Docker 多阶段构建，docker-compose    |

## 项目状态

MVP 已完成并通过端到端验证。WebSocket 实时配置 + HTTP 轮询兜底、按规则流量计费、
用户编辑/流量重置、实时节点状态（CPU/内存/连接数/版本）。预发布阶段 —— 强制按用户配额是后续工作。

## 许可证与免责声明

AGPL-3.0 —— 详见 [LICENSE](LICENSE)。开源流量转发工具，**仅供个人学习与研究使用**；
请在合法合规前提下使用，风险自负。完整 **[免责声明](docs/DISCLAIMER.md)**。
