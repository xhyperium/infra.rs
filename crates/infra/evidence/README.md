# evidence（crate 名 `xhyper-evidence`）

L1 **审计证据追加面**：trait + 进程内实现，供 bootstrap 注入。

| 项 | 值 |
|----|-----|
| package | `xhyper-evidence` |
| lib | `evidence` |
| version | `0.1.0` |
| deps | **std-only** |

## 公开面

- `EvidenceAppender::append_named` → `AppendReceipt { name, seq }`
- `EvidenceError::{DurabilityFailure, Unavailable}`
- `InMemoryEvidenceAppender`（`fail_next` / `close` 测试钩子）

## 非目标

远程存储、签名链、完整 monorepo wire 协议。

## 验证

```bash
cargo test -p evidence --all-targets
cargo clippy -p evidence --all-targets -- -D warnings
node scripts/cov-gate-100.mjs -p evidence --filter crates/infra/evidence/src
```

## 生产误用红线

| 禁止 | 原因 |
|------|------|
| `InMemoryEvidenceAppender` 作合规审计 | 仅内存；进程退出即失 |
| 宣称远程/签名证据链完成 | SSOT DEFER |

示例：`cargo run -p evidence --example append_memory`

## 持久化（infra-s9t.7）

- `FileEvidenceAppender::open(path)`：本地文件行追加（`seq\\tname`）+ flush。
- `InMemoryEvidenceAppender`：**仅**开发/测试；禁止单独作为合规审计后端。
