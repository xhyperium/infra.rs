# `.agents/ssot/infra/` — L1 平台规格平面

与 **`crates/infra/`** 对齐的 SSOT 域归组（2026-07-24 · SSOT v2.3.0）。

## 成员

| 域 | 权威路径 | 本仓实现 |
|----|----------|----------|
| bootstrap | `infra/bootstrap/` | `crates/infra/bootstrap` |
| configx | `infra/configx/` | `crates/infra/configx` |
| evidence | `infra/evidence/` | `crates/infra/evidence`（canonical current-state） |
| gate | `infra/gate/` | 仅规格；本仓 **未 member** |
| observex | `infra/observex/` | `crates/infra/observex` |
| resiliencx | `infra/resiliencx/` | `crates/infra/resiliencx` |
| schedulex | `infra/schedulex/` | `crates/infra/schedulex` |
| testkitx | `infra/testkitx/` | 仅规格；**非** `crates/testkit` |
| transport | `infra/transport/` | `crates/infra/transport`（package `transportx`） |

## 重定向

旧根路径 `.agents/ssot/{bootstrap,configx,evidence,gate,observex,resiliencx,schedulex,testkitx,transport}/` 仅保留 README 重定向（R5）；**勿**在旧路径新增 active spec。

## 规则

见 [SSOT.md](../SSOT.md) R6 / R7。规格 COMPLETE ≠ 本仓 ship。
