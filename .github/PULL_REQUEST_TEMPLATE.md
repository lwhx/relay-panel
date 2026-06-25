# Pull Request

Thanks for contributing to RelayPanel! Please fill out the sections below —
it helps reviewers (and future you) understand the change quickly.

## What

<!-- One-paragraph summary of the change. -->

## Why

<!-- What problem does this solve? Link the issue or describe the user-visible
     pain point. -->

## How to verify

<!-- Concrete steps to test the change end-to-end. List commands to run, UI
     steps to click, or features to exercise. -->

## Checklist

- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes
- [ ] `cargo test --workspace` passes (15+ tests)
- [ ] If touching `db/{sqlite,pg}_repo`: both backend implementations and tests are updated in parity
- [ ] `cd frontend && npm run lint` passes
- [ ] `cd frontend && npm run build` passes
- [ ] If release-relevant: `bash scripts/release-check.sh X.Y.Z` shows 0 FAIL
- [ ] Version bumped in all 6 places (per `docs/VERSIONS.md`) if releasing
- [ ] `CHANGELOG.md` updated under a new `## [X.Y.Z]` section if releasing
- [ ] No new `unwrap_or_default()` / `.ok()` silent-error patterns in Rust code
- [ ] No new `console.log` in frontend code
- [ ] If a UI change: added a screenshot / short description

## Risk / Breaking changes

<!-- Does this touch auth, schema migrations, API contracts, or config?
     What did you test to confirm the migration path? -->

## Related

<!-- Linked issues, PRs, or discussions. -->

---

> This template is required for all PRs to `main`. CI will not block on its
> presence, but the contributor is expected to complete it.
