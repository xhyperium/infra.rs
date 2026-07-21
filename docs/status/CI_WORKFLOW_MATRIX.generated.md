# CI 工作流矩阵（自动生成）

> **生成方式**：`node scripts/gen-docs-status.mjs`
> **生成日期**：2026-07-21
> **源目录**：`.github/workflows/`
> **勿手改**：本文件由脚本覆盖；叙事性说明见 [CI_STATUS_REPORT.md](CI_STATUS_REPORT.md) / [CONFIG_SUMMARY.md](CONFIG_SUMMARY.md)。

## 工作流一览

| 文件 | name | 触发（启发式） | Jobs |
|------|------|----------------|------|
| `canonical-coverage.yml` | Canonical Coverage | push, pull_request | `line-coverage` |
| `ci-rust.yml` | CI（Rust） | push, pull_request | `check-rust`, `build`, `test`, `msrv`, `coverage` |
| `constitution.yml` | Constitution | push, pull_request, workflow_dispatch | `constitution` |
| `contracts-coverage.yml` | Contracts Coverage | push, pull_request | `line-coverage` |
| `decimal-coverage.yml` | Decimal Coverage | push, pull_request | `line-coverage` |
| `evidence-coverage.yml` | Evidence Coverage | push, pull_request | `line-coverage` |
| `kernel-coverage.yml` | Kernel Coverage | push, pull_request | `coverage` |
| `kernel-miri.yml` | Kernel Miri | schedule, workflow_dispatch | `miri` |
| `kernel-mutants.yml` | Kernel Mutants | schedule, workflow_dispatch | `mutants` |
| `observex-coverage.yml` | Observex Coverage | push, pull_request | `line-coverage` |
| `pr-template-check.yml` | PR Template Check | pull_request | `template-check` |
| `quality.yml` | 质量 | push, pull_request | `check-rust`, `fmt`, `clippy`, `doc` |
| `resiliencx-coverage.yml` | Resiliencx Coverage | push, pull_request | `line-coverage` |
| `schedulex-coverage.yml` | Schedulex Coverage | push, pull_request | `line-coverage` |
| `security.yml` | 安全 | push, pull_request, schedule, workflow_dispatch | `check-rust`, `deny`, `audit` |
| `testkit-coverage.yml` | Testkit Coverage | push, pull_request | `line-coverage` |
| `testkit-miri.yml` | Testkit Miri | schedule, workflow_dispatch | `miri` |
| `testkit-mutants.yml` | Testkit Mutants | schedule, workflow_dispatch | `mutants` |
| `validation.yml` | 校验 | push, pull_request | `yaml-lint`, `toml-lint`, `markdown-lint`, `spellcheck`, `link-check`, `harness` |

## 统计

| 指标 | 值 |
|------|-----|
| 工作流文件数 | 19 |
| Job 总数（解析） | 33 |

## 维护

```bash
node scripts/gen-docs-status.mjs          # 重新生成
node scripts/gen-docs-status.mjs --check  # CI/本地一致性检查
```
