# CI 工作流矩阵（自动生成）

> **生成方式**：`node scripts/docs/gen-docs-status.mjs`
> **生成日期**：2026-07-21
> **源目录**：`.github/workflows/`
> **勿手改**：本文件由脚本覆盖；叙事性说明见 [CI_STATUS_REPORT.md](CI_STATUS_REPORT.md) / [CONFIG_SUMMARY.md](CONFIG_SUMMARY.md)。

## 工作流一览

| 文件 | name | 触发（启发式） | Jobs |
|------|------|----------------|------|
| `beads-e2e.yml` | Beads E2E | pull_request, workflow_dispatch | `beads-smoke` |
| `benchmark.yml` | Benchmark | workflow_dispatch | `bench` |
| `canonical-coverage.yml` | Canonical Coverage | pull_request, workflow_dispatch | `line-coverage` |
| `ci-rust-org.yml` | CI Rust（组织复用） | push, pull_request, workflow_dispatch | `org-rust` |
| `ci-rust.yml` | CI（Rust） | pull_request, workflow_dispatch | `check-rust`, `build`, `test`, `msrv`, `coverage` |
| `ci-summary.yml` | CI Summary | (see workflow) | `summary` |
| `configx-coverage.yml` | ConfigX Coverage | pull_request, workflow_dispatch | `coverage` |
| `constitution.yml` | Constitution | pull_request, workflow_dispatch | `constitution` |
| `contracts-coverage.yml` | Contracts Coverage | pull_request, workflow_dispatch | `line-coverage` |
| `contracts-live.yml` | Contracts Mock Verification | workflow_dispatch | `mock-backend` |
| `decimal-coverage.yml` | Decimal Coverage | pull_request, workflow_dispatch | `line-coverage` |
| `decimal-miri.yml` | Decimal Miri | schedule, workflow_dispatch | `miri` |
| `decimal-mutants.yml` | Decimal Mutants | schedule, workflow_dispatch | `mutants` |
| `evidence-coverage.yml` | Evidence Coverage | pull_request, workflow_dispatch | `line-coverage` |
| `kernel-coverage.yml` | Kernel Coverage | pull_request, workflow_dispatch | `coverage` |
| `kernel-loom.yml` | kernel-loom | pull_request, workflow_dispatch | `loom` |
| `kernel-miri.yml` | Kernel Miri | schedule, workflow_dispatch | `miri` |
| `kernel-mutants.yml` | Kernel Mutants | schedule, workflow_dispatch | `mutants` |
| `observex-coverage.yml` | Observex Coverage | pull_request, workflow_dispatch | `line-coverage` |
| `pr-template-check.yml` | PR Template Check | pull_request | `template-check` |
| `public-api.yml` | 公开 API | pull_request, workflow_dispatch | `public-api` |
| `quality.yml` | 质量 | pull_request, workflow_dispatch | `check-rust`, `fmt`, `clippy`, `doc` |
| `release.yml` | Release | workflow_dispatch | `release-build`, `release-test`, `release-clippy`, `release-doc` |
| `resiliencx-coverage.yml` | Resiliencx Coverage | pull_request, workflow_dispatch | `line-coverage` |
| `schedulex-coverage.yml` | Schedulex Coverage | pull_request, workflow_dispatch | `line-coverage` |
| `security.yml` | 安全 | pull_request, schedule, workflow_dispatch | `check-rust`, `deny`, `audit` |
| `self-test.yml` | 模块自验证 | pull_request | `scripts-lint`, `hooks-lint`, `scripts-test` |
| `testkit-coverage.yml` | Testkit Coverage | pull_request, workflow_dispatch | `line-coverage` |
| `testkit-miri.yml` | Testkit Miri | schedule, workflow_dispatch | `miri` |
| `testkit-mutants.yml` | Testkit Mutants | schedule, workflow_dispatch | `mutants` |
| `validation.yml` | 校验 | pull_request, workflow_dispatch | `yaml-lint`, `toml-lint`, `markdown-lint`, `spellcheck`, `link-check`, `harness`, `canonical-align`, `decimal-panicking-ops` |

## 统计

| 指标 | 值 |
|------|-----|
| 工作流文件数 | 31 |
| Job 总数（解析） | 52 |

## 维护

```bash
node scripts/docs/gen-docs-status.mjs          # 重新生成
node scripts/docs/gen-docs-status.mjs --check  # CI/本地一致性检查
```
