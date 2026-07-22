# Gate — kernel（SPEC-KERNEL-002）

| 字段 | 当前值 |
|------|--------|
| Contract | Approved / Active |
| Package candidate | `kernel 0.3.1` |
| Distribution | `publish = false` |
| Maturity | L1 Internal Ready；L4 仅限已证支持面 |
| Current delivery verdict | PATCH bump 已同步；`fc201d7` focused gate PASS；R1 修复候选待重验 |
| Production certification | 未声明 |

本文件只记录当前交付门禁，不继承历史战役 PASS。行为变更已按 patch-default 从 `0.3.0` 升至 `0.3.1`，且本 PR 不再重复 bump。

## 1. 阻断门禁

| Gate | 当前状态 | 通过条件 |
|------|----------|----------|
| Spec / design / test 一致 | `fc201d7` PASS；最终候选待重验 | 三份合同使用相同签名和语义 |
| `ClockDomain` | `fc201d7` PASS；最终候选待重验 | process domain、共享 origin、跨 domain `None` |
| 隐藏构造 seam | `fc201d7` PASS；最终候选待重验 | 两个 `#[doc(hidden)]` seam 存在且调用边界明确 |
| `wait_timeout` 返回面 | `fc201d7` PASS；最终候选待重验 | `Result<bool, WaitTimeoutError>` |
| deadline overflow | `fc201d7` PASS；最终候选待重验 | `Duration::MAX` 精确匹配 typed error |
| 常规 timeout / trigger | `fc201d7` PASS；最终候选待重验 | `Ok(false)` / `Ok(true)` 行为正确 |
| 根公开 API | `fc201d7` PASS；最终候选待重验 | 导出 `WaitTimeoutError`，API 基线同步 |
| Doctest | `fc201d7` PASS；最终候选待重验 | rustdoc `compile_fail` 全部通过 |
| Loom | `fc201d7` PASS；最终候选待重验 | 核心 wait/trigger 模型通过 |
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

执行者应把实际命令、commit、工具链和输出写入本轮 evidence，再将对应门禁改为 PASS 或 FAIL。

## 3. 非本仓门禁

archgate 与 `.architecture/**` 对 infra.rs 为 OOS。本 gate 不要求 crates.io publish，也不以历史 tag、release 或 registry stable 声明作为完成条件。

coverage、Miri 与 mutation 以当前 CI 实际接线为准。旧制品、未运行或 SKIP 均不得写成 fresh PASS。

## 4. 完成判定

全部阻断门禁通过、版本和公开 API 基线同步、且证据绑定当前 commit 后，当前交付才可判定完成。

完成只证明本轮 L1 内部合同与列出的 L4 支持面，不构成 production-certified、crates.io 可发布或全平台生产认证。
