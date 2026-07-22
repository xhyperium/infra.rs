# Gate — kernel（SPEC-KERNEL-002）

| 字段 | 当前值 |
|------|--------|
| Contract | Approved / Active |
| Package candidate | `kernel 0.3.1` |
| Distribution | `publish = false` |
| Maturity | L1 Internal Ready；L4 仅限已证支持面 |
| Current delivery verdict | R3 内容候选 `387a1dc` 全量门禁 PASS；独立终审待闭合 |
| Production certification | 未声明 |

本文件只记录当前交付门禁，不继承历史战役 PASS。行为变更已按 patch-default 从 `0.3.0` 升至 `0.3.1`，且本 PR 不再重复 bump。

## 1. 阻断门禁

| Gate | 当前状态 | 通过条件 |
|------|----------|----------|
| Spec / design / test 一致 | `387a1dc` PASS；独立终审待闭合 | 三份合同使用相同签名和语义 |
| `ClockDomain` | `387a1dc` PASS；独立终审待闭合 | process domain、共享 origin、跨 domain `None` |
| 隐藏构造 seam | `387a1dc` PASS；独立终审待闭合 | 两个 `#[doc(hidden)]` seam 存在且调用边界明确 |
| `wait_timeout` 返回面 | `387a1dc` PASS；独立终审待闭合 | `Result<bool, WaitTimeoutError>` |
| deadline overflow | `387a1dc` PASS；独立终审待闭合 | 未触发时 `Duration::MAX` 精确匹配 typed error |
| 常规 timeout / trigger | `387a1dc` PASS；独立终审待闭合 | `Ok(false)` / `Ok(true)` 正确，且已触发完成优先于 timeout 校验 |
| 根公开 API | `387a1dc` PASS；独立终审待闭合 | 导出 `WaitTimeoutError`，API 基线同步 |
| Doctest | `387a1dc` PASS；独立终审待闭合 | rustdoc `compile_fail` 全部通过 |
| Loom | `387a1dc` PASS；独立终审待闭合 | 核心 wait/trigger 模型通过 |
| 版本 | PASS | `0.3.0 → 0.3.1`，仅 bump 一次 |

任何 deadline overflow 被返回为 `Ok(false)` 都必须 FAIL。任何历史 evidence 被当作本轮 PASS 也必须 FAIL。

## 2. 必跑命令

```bash
cargo fmt --all -- --check
cargo clippy -p kernel --all-targets -- -D warnings
cargo test -p kernel --all-targets
cargo test -p kernel --doc
RUSTFLAGS='--cfg loom' cargo test -p kernel --test lifecycle_concurrency_loom --release
node scripts/quality-gates/check-public-api.mjs
node scripts/quality-gates/check-workspace-deps.mjs
```

执行者应把实际命令、commit、工具链、退出码和关键输出摘要写入本轮 evidence，再将对应门禁改为 PASS 或 FAIL。完整 stdout 可由 CI artifact 或执行会话承载，但若未入库必须明确说明，且不得把非持久会话当作仓库证据；入库 evidence 必须足以复算候选与重放命令。

## 3. 非本仓门禁

archgate 与 `.architecture/**` 对 infra.rs 为 OOS。本 gate 不要求 crates.io publish，也不以历史 tag、release 或 registry stable 声明作为完成条件。

coverage、Miri 与 mutation 以当前 CI 实际接线为准。旧制品、未运行或 SKIP 均不得写成 fresh PASS。

## 4. 完成判定

全部阻断门禁通过、版本和公开 API 基线同步、且证据绑定当前 commit 后，当前交付才可判定完成。

完成只证明本轮 L1 内部合同与列出的 L4 支持面，不构成 production-certified、crates.io 可发布或全平台生产认证。
