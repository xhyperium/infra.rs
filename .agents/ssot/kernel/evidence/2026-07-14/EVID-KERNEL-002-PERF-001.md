# EVID-KERNEL-002-PERF-001 — Cow::Borrowed static context

| 字段 | 值 |
|------|-----|
| Residual | **RES-PERF-001** |
| Date | 2026-07-14 |
| Status | **CLOSED (DEFER accepted)** |
| Spec | SPEC-KERNEL-002 §13.1 Error |
| API 变更 | **未实施**（避免半破坏性公共面改动） |

## §13.1 要求

```text
- 静态 context 应允许 Cow::Borrowed；
- 动态 context 可分配；
```

## 现状（源码）

路径：`crates/kernel/src/error.rs`

```rust
// 字段已是 Cow
context: Cow<'static, str>,

// 构造路径恒 Owned
fn ctx(context: impl Into<String>) -> Cow<'static, str> {
    Cow::Owned(context.into())
}

// 全部公共构造器签名
pub fn invalid(context: impl Into<String>) -> Self { ... }
// missing / conflict / transient / … 同形
```

公开 API 快照（`.architecture/api/kernel-public-api.txt`）：

```text
pub fn kernel::error::XError::invalid(impl core::convert::Into<alloc::string::String>) -> Self
// 其余构造器同：Into<String>
```

**事实**：即使调用方传入 `&'static str` 字面量，也经 `Into<String>` → `Cow::Owned`，**无法**得到 `Cow::Borrowed`。存储类型已具备 Borrowed 能力，入口 API 未暴露。

## 候选非破坏路径评估

| 方案 | 效果 | 破坏面 |
|------|------|--------|
| A. `impl Into<Cow<'static, str>>` | 静态字面量可 Borrowed；`String`/`format!` 可 Owned | **半破坏**：非 `'static` 的 `&str`（函数参数、`.as_str()`）**不再**满足 `Into<Cow<'static, str>>`，而今天经 `Into<String>` **可编译** |
| B. 自定义 `IntoContext` trait：`&'static str`→Borrowed，`String`→Owned，`&str`→Owned | 尽量保留调用形态 | 新 trait 进入公共面；与 `Into` 重叠；`&'static str` vs `&str` 重叠需仔细排序；**仍改变 cargo-public-api 签名**（KERNEL-API-001 快照 diff） |
| C. 并行 API `invalid_static(&'static str)` / `invalid_owned(String)` | 零成本路径清晰 | additive 但增面；主路径仍 Owned；与「单一构造器」风格不一致 |
| D. 内部 magic 在 `Into<String>` 下 Borrowed | **不可能** — `Into<String>` 已丢失静态生命周期 | — |

## 下游影响（方案 A 示意）

工作区大量 `XError::invalid(format!(...))` / `to_string()` → 仍 OK。  
风险在于未来或边界代码：

```rust
fn map_err(msg: &str) -> XError {
    XError::invalid(msg) // 今日 OK；改 Into<Cow<'static, str>> 后编译失败
}
```

`publish = false` 降低 crates.io 爆炸半径，但 workspace 内 + API 快照门禁仍视签名变更为 **API 事件**。

## DEFER 理由（正式采纳）

1. **无干净零成本且零破坏路径**（无 specialization / 不改签名则无法 Borrowed）。
2. 任务纪律：**DO NOT break public API without strong non-breaking path**。
3. 收益为微优化（静态字面量少一次分配）；kernel §13 明确「不追求极端微优化」，且错误路径本非热环主路径。
4. 字段已是 `Cow<'static, str>`，未来 additive（方案 C）或人审批准的签名迁移仍可做。

## 关闭条件（本 DEFER 满足）

- [x] 读 error 构造路径并记录现状  
- [x] 评估非破坏式 `Into<Cow>` / 替代方案  
- [x] 书面 **DEFER accepted** + 再开启条件  

**再开启条件**（未来可把 residual 重开为实现项）：

1. 人审批准公共签名变更（或方案 C additive），并更新 `kernel-public-api.txt`；  
2. 全 workspace `cargo check` / 下游编译绿；  
3. 可选：基准或分配计数证明静态路径零分配。

## Verdict

- **RES-PERF-001 → CLOSED (DEFER accepted)**。  
- **不得**计为 §13.1 已实现 Borrowed；计为 **有意推迟的已知差距**。  
- 未 bump 版本；未改 public API。
