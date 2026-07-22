# infra.rs — 2026-07-21 Handoff

## 1. Beads ↔ GitHub Issues Sync

**Location:** `scripts/beads/`

| File | Lines | Purpose |
|------|------|---------|
| `gh-sync.mjs` | 700+ | Bidirectional sync engine |
| `gh-sync.test.mjs` | 61 assertions | Unit tests |
| `gh-sync-interactive-test.mjs` | 59 assertions | UI tests |
| `gh-sync-complex-test.mjs` | 11 assertions | Complex input tests |
| `gh-sync-stress-test.mjs` | 29 assertions | Stress tests |
| `gh-sync-i18n-test.mjs` | 20 assertions | Multi-language tests |

### Commands

```bash
make beads-test                       # 180 assertions (5 suites)
make beads-sync-dry                   # Preview sync changes
make beads-sync-interactive           # TTY conflict review
```

### Key Features

- Bidirectional beads ↔ GitHub Issues
- `--interactive` mode with side-by-side TTY review
- Footer-based idempotency (`<!-- beads-sync: ID | synced: TS -->`)
- Label management (beads labels → GitHub labels + priority/status/type)
- Hook integration: `session-review.mjs` fires incremental sync

### CI: `.github/workflows/beads-test.yml`

- 5 suites (180 assertions) parallel execution
- PR-triggered, path-filtered to `scripts/beads/**`
- `workflow_dispatch` for manual runs

---

## 2. Secrets Migration

**Location:** `scripts/sre/`

| File | Purpose |
|------|---------|
| `secrets-migrate.mjs` | Single-source MD table migration |
| `secrets-migrate-all.mjs` | Multi-source (MD tables + .env + Docker Compose) |
| `extract_all_secrets.sh` | Scope + complexity validation with strength report |
| `secrets-lint.yml` | CI workflow for migration script validation |

### Current State

```text
GitHub Secrets:      43 active (38 PASSWORD + 5 service tokens)
Source:              dev.md (57 password-only entries extracted)
Scope:               No DATABASE/USER entries (removed as non-sensitive)
Complexity:          WARNING (63.2% pass, 21 passwords need rotation)
```

### Key Commands

```bash
# Validate scope + complexity + strength report
scripts/sre/extract_all_secrets.sh dev

# Dry-run preview
node scripts/sre/secrets-migrate-all.mjs --source all --env dev --dry-run

# Live upload
node scripts/sre/secrets-migrate-all.mjs --source tables --env dev --live
```

---

## 3. Security Baseline

### Workflow Security Enforcement

**Location:** `.github/workflows/workflow-security.yml`

6 rules across 3 severity levels:

- HIGH: permissions block required, no `pull_request_target`, no self-hosted runners
- MEDIUM: explicit permissions (not `read-all` string)
- LOW: concurrency groups, path filters on PR triggers

**Ruleset:** `main-ai-first` (#19250230) — 3 required status checks:

1. Constitution Check
2. Template Validation
3. Workflow Security / 审计工作流安全基线

### Credential Complexity Baseline

**Location:** `docs/governance/credential-baseline.md`

- Minimum 24 chars, uppercase + lowercase + digits
- No 4+ consecutive same character
- No common patterns (password, admin123, etc.)
- API keys (hex) and tokens (JWT) exempted
- 15 passwords flagged for rotation (PG + TD per-db)

### Security Scan Results

```text
35 workflows scanned: 0 HIGH, 0 MEDIUM, 0 LOW
4 files fixed (permissions + concurrency blocks)
False positives eliminated (self-hosted, checkout@v3)
```

---

## 4. Layout Standard Update

**Location:** `crates/AGENTS.md` (v1.6.0)

Changed from 8-item to 7-item standard:

- Removed: `examples/`, `CHANGELOG.md`, `AGENTS.md`
- Added: `review/`, `releases/`
- All 21 crates: `review/README.md` + `releases/README.md` created

**Generator:** `scripts/docs/gen-crate-status.mjs` updated for 7-item matrix

---

## 5. Documentation

| Document | Purpose |
|----------|---------|
| `docs/governance/commit-template.md` | `.gitmessage` usage guide |
| `docs/governance/credential-baseline.md` | Password complexity + rotation tracking |
| `docs/governance/README.md` | Index updated with new entries |
| `docs/constitution/04-code-standards.md` | Updated for `.gitmessage` |
| `README.md` | Beads Sync Test badge + make targets |
| `AGENTS.md` | Updated commit convention checklist |
| `Makefile` | 12 new targets (beads-test, sync, verify) |

---

## 6. CI/CD Workflows Created

| Workflow | File | Trigger |
|----------|------|---------|
| Beads Sync Test Suite | `beads-test.yml` | PR + push |
| Secrets Lint | `secrets-lint.yml` | PR |
| Workflow Security | `workflow-security.yml` | PR |

---

## 7. Commits Summary

```text
docs:     credential-baseline.md, commit-template.md, README badges
feat:     beads sync engine, secrets migration, CI workflows
fix:      security permissions, false positives, import guards
test:     5 test suites (180 assertions), stress/i18n/complex coverage
ci:       beads-test.yml, secrets-lint.yml, workflow-security.yml
```

---

## Quick Reference

```bash
# Beads sync
make beads-test                # 180 assertions
make beads-sync-interactive    # TTY conflict review

# Secrets audit
scripts/sre/extract_all_secrets.sh dev

# Security scan
python3 -c "import yaml; ..."  # inline in workflow-security.yml
```

**PR:** [xhyperium/infra.rs#169](https://github.com/xhyperium/infra.rs/pull/169) (main)
**Badge:** `[![Beads Sync Test](...beads-test.yml/badge.svg)]` — 200 ✓
