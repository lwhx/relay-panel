#!/usr/bin/env bash
# Local guardrail runner for mechanical refactors.
#
# Default behavior is intentionally safe for a fresh clone without frontend
# dependencies: run Rust checks, and run frontend checks only when node_modules
# already exists. Use --frontend (or --all) to require frontend checks, and
# --install-frontend to run npm ci first.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FRONTEND_DIR="$ROOT_DIR/frontend"

frontend_mode="auto" # auto | require | skip
install_frontend=0

usage() {
  cat <<'EOF'
Usage: scripts/refactor-check.sh [OPTIONS]

Runs the regression gates used while splitting large modules for v0.4.18.

Options:
  --backend-only       Run only Rust fmt/clippy/test gates.
  --frontend          Require frontend typecheck/lint/test/build gates.
  --all               Same as --frontend; Rust gates always run.
  --install-frontend  Run npm ci in frontend/ before frontend gates.
  -h, --help          Show this help.

Default:
  Rust gates always run. Frontend gates run only if frontend/node_modules exists;
  otherwise they are skipped with a clear message. This avoids mutating a local
  checkout unless explicitly requested with --install-frontend.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --backend-only)
      frontend_mode="skip"
      ;;
    --frontend|--all)
      frontend_mode="require"
      ;;
    --install-frontend)
      install_frontend=1
      frontend_mode="require"
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown option: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
  shift
done

run_step() {
  echo
  echo "==> $*"
  "$@"
}

cd "$ROOT_DIR"
run_step cargo fmt --check
run_step cargo clippy --workspace --all-targets -- -D warnings
run_step cargo test --workspace
run_step bash scripts/check-repo-test-parity.sh

if [[ "$frontend_mode" != "skip" ]]; then
  if [[ ! -d "$FRONTEND_DIR" ]]; then
    if [[ "$frontend_mode" == "require" ]]; then
      echo "frontend directory missing: $FRONTEND_DIR" >&2
      exit 1
    fi
    echo "frontend directory missing; skipping frontend gates"
    exit 0
  fi

  if [[ "$install_frontend" -eq 1 ]]; then
    cd "$FRONTEND_DIR"
    run_step npm ci --no-audit --no-fund
  elif [[ ! -d "$FRONTEND_DIR/node_modules" ]]; then
    if [[ "$frontend_mode" == "require" ]]; then
      echo "frontend/node_modules missing; run with --install-frontend or run npm ci first" >&2
      exit 1
    fi
    echo
    echo "==> frontend/node_modules missing; skipping frontend gates"
    echo "    Run: scripts/refactor-check.sh --install-frontend"
    exit 0
  fi

  cd "$FRONTEND_DIR"
  run_step npm run typecheck
  run_step npm run lint
  run_step npm run test
  run_step npm run build
fi
