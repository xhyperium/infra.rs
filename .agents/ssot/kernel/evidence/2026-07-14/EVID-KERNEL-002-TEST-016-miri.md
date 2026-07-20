# EVID-KERNEL-002-TEST-016 — Miri

| 字段 | 值 |
|------|-----|
| Date | 2026-07-14 |
| Residual | **RES-TEST-016** |
| Status | **OPEN / DEFER**（**不得 CLOSED**） |
| Spec | SPEC-KERNEL-002 §11 / §18.3 miri |
| §18 | **仍 OPEN** |

## 工具可用性

| 项 | 状态 | 证据 |
|----|------|------|
| `cargo-miri` 二进制 | **PRESENT** | `/home/claude/.cargo/bin/cargo-miri` |
| `rust-toolchain.toml` | **stable** | `channel = "stable"`；components 仅 rustfmt + clippy |
| stable 工具链 miri 组件 | **ABSENT** | stable components 列表无 miri-preview |
| nightly 工具链 | **PRESENT** | `nightly-x86_64-unknown-linux-gnu` |
| nightly miri 组件 | **ABSENT** | `nightly` `components` 文件无 `miri-preview`；`bin/` 无 `miri` |
| 本会话 `cargo miri test -p kernel` | **SKIP** | 组件缺失 + Executor 无 Shell |

与既有备注一致：`EVID-KERNEL-002-CLK009-TEST004.md` 曾记「stable 工具链无 miri 组件」。

## 目标命令（未执行）

```bash
rustup component list 2>/dev/null | head
cargo miri test -p kernel 2>&1 | tee /tmp/kernel-miri.txt
```

`/tmp/kernel-miri.txt`：**未生成**。

## 相对门槛判定

| 门槛 | 实测 | 判定 |
|------|------|------|
| `cargo miri test -p kernel` 绿 | **未跑** | **DEFER OPEN** |

## DEFER 理由

1. **组件缺失**：虽有 `cargo-miri` shim，但当前 stable/nightly **均未**安装 miri sysroot 组件 → 实跑会失败。
2. **工具链策略**：仓库 pin stable；miri 通常需 `rustup +nightly component add miri`（或 CI nightly job），属环境/CI 决策，非本 residual 代码缺口 alone。
3. **无 shell**：本会话无法 `rustup component add miri` 或执行测试。

## 诚实结论

- **不得**宣称 RES-TEST-016 CLOSED。
- 状态保持 **OPEN / DEFER**。
- 关闭条件：nightly（或文档指定工具链）安装 miri → `cargo miri test -p kernel` 全绿 + 本 evidence 更新为 PASS。

## 建议复跑

```bash
rustup toolchain install nightly
rustup +nightly component add miri
cargo +nightly miri test -p kernel 2>&1 | tee /tmp/kernel-miri.txt
```
