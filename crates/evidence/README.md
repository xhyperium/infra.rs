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
node scripts/cov-gate-100.mjs -p evidence --filter crates/evidence/src
```
