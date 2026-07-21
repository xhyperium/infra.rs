# Alignment Matrix — infra.rs `resiliencx` 1:1

| 字段 | 值 |
|------|-----|
| Matrix ID | `ALIGN-INFRA-RESILIENCX-20260721` |
| Scope | `.agents/ssot/infra/resiliencx/**` ↔ `crates/resiliencx` |
| 更新 | 2026-07-21 |

## 图例

| 标签 | 含义 |
|------|------|
| **MATCH** | 文档 claim 与 live 一致 |
| **ADAPT** | 相对上游诚实适配，语义合同仍 MATCH |
| **OPEN** | residual |
| **POLICY** | 永久约束 |

## A. 公开 API

| Claim | Live | 状态 |
|-------|------|------|
| Package `xhyper-resiliencx` / lib `resiliencx` | Cargo.toml + workspace members | MATCH |
| `RetryConfig { max_attempts, base_delay_ms }` | lib.rs | MATCH |
| `retry_fn` §2 六类行为 | lib.rs + tests | MATCH |
| `Instrumentation` 注入 | 本 crate trait | ADAPT（无 contracts） |
| `record_retry` 仅在真正 retry 前 | lib.rs | MATCH |
| `base_delay_ms>0` → `thread::sleep` | lib.rs | MATCH（已知差距） |
| 无 circuit/limiter 实现 | 源码无 | MATCH |

## B. 依赖 / 禁令

| Claim | 状态 |
|-------|------|
| 仅 kernel 生产依赖 | MATCH |
| 无 observex | MATCH / POLICY |
| 不反向 transport/domain/app | MATCH |

## C. 测试 / 覆盖率

| Claim | 状态 |
|-------|------|
| §2 六类 + delay + downcast | MATCH |
| Lines Cover 100% | MATCH（llvm-cov 证据） |

## D. Residual honesty

| Claim | 状态 |
|-------|------|
| circuit / limiter / async wait / stable | OPEN / DEFER / HUMAN |
