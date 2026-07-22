# evidence 公开 API

**角色**：审计证据追加

## 公开消费面

`EvidenceAppender` / `InMemoryEvidenceAppender` / `AppendReceipt` / `EvidenceError`。

## 最小用法

```rust
use evidence::{EvidenceAppender, InMemoryEvidenceAppender};

let a = InMemoryEvidenceAppender::new();
let r = a.append_named("boot").unwrap();
assert_eq!(r.seq, 1);
```

## 2026-07-22 dual-bar surface

Public helpers added for STATUS 100% structure + declared-surface hardening; see crate root docs and ssot alignment. **Not** Production Ready / L5.
