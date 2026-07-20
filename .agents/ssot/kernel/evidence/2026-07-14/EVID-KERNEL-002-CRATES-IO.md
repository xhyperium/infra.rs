# EVID-KERNEL-002-CRATES-IO — crates.io 发布

| 字段 | 值 |
|------|-----|
| Date | 2026-07-14 |
| crates.io name | **`xhyper-kernel`** |
| version | **0.1.1** |
| lib name | `kernel`（`use kernel::` 不变） |
| dry-run | **PASS** |
| publish | **PASS** 2026-07-14T09:27:38Z |

## 为何不叫 `kernel`

`cargo publish --dry-run` 报告 **`kernel@0.1.1 already exists on crates.io index`**。  
改用 **`xhyper-kernel`**，workspace path dep 使用 `package = "xhyper-kernel"`。


## 真发布结果（2026-07-14）

```text
Uploaded xhyper-kernel v0.1.1 to registry crates-io
Published xhyper-kernel v0.1.1 at registry crates-io
```

- URL: https://crates.io/crates/xhyper-kernel/0.1.1
- docs: https://docs.rs/xhyper-kernel/0.1.1 （索引同步后）
