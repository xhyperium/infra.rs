# CONTRACT_SPEC.md — 合同合规验证规则

> 本文件是合同合规验证的**可执行规范**。
> 定义如何对每份合同执行 L1–L4 合规校验、证据格式、门禁流程与误报处理。
>
> 数据模型以 [`CONTRACT.md`](./CONTRACT.md) 为准；验证规则以本文件为准。
> 验证脚本：`scripts/quality-gates/check-contract-compliance.mjs`
> CI 工作流：`.github/workflows/contract-compliance.yml`

---

## 1. 验证架构

```text
L1 签名验证 ──→ L2 行为验证 ──→ L3 非功能验证 ──→ L4 生产证据
    (自动)         (自动)         (手动触发)        (手动触发)
    < 1 min       < 5 min       < 10 min         无限制
```

- L1–L2 在 PR CI 中自动执行
- L3–L4 通过 `workflow_dispatch` 手动触发
- 每个级别独立报告，任意级别失败阻塞交付

---

## 2. L1 签名合规验证（7 规则）

| 规则 ID | 规则 | 门禁方式 |
|---------|------|----------|
| L1-SIG-001 | 合同声明的所有公开 trait/struct/fn/类型必须存在于 crate 源码中 | 交叉验证（spec.md vs src/） |
| L1-SIG-002 | 合同声明的 trait bound (`Send + Sync` 等) 必须在 crate 源码中成立 | 源码扫描 |
| L1-SIG-003 | 合同声明的依赖不超出合同许可范围 | `cargo tree` 差分 |
| L1-SIG-004 | 合同声明的 `ErrorKind` 子集与方法实际 `XResult` 完全一致 | API 扫描 |
| L1-SIG-005 | 公开项数量不得在未声明时减少（Additive Only 域） | API diff |
| L1-SIG-006 | `#[non_exhaustive]` 标记不得删除 | 源码扫描 |
| L1-SIG-007 | 域 `spec.md` 引用的版本与 `Cargo.toml` 一致 | 字符串匹配 |

### 验证命令

```bash
node scripts/quality-gates/check-contract-compliance.mjs --level L1 --fail-level L1
```

### 通过标准

- 所有声明的公开项在源码中存在
- 所有公开 enum 标记 `#[non_exhaustive]`
- 所有合同 spec 版本与 Cargo.toml 版本一致

---

## 3. L2 行为合规验证（6 规则）

| 规则 ID | 规则 | 门禁方式 |
|---------|------|----------|
| L2-BEH-001 | 每个声明的前置条件违反必须有对应测试 | 覆盖率 ≥ 声明前置条件数 |
| L2-BEH-002 | 每个声明的 ErrorKind 路径必须有对应测试 | ErrorKind 覆盖率 100% |
| L2-BEH-003 | 每个声明的不变量必须有 proptest 或 loom 覆盖 | 检查 test 文件存在性 |
| L2-BEH-004 | 每个声明的事务/状态转换矩阵必须全覆盖 | 检查二元测试矩阵完整 |
| L2-BEH-005 | conformance crate 全部编译通过 | `cargo check` 全通过 |
| L2-BEH-006 | 后置条件违反不得静默（必须返回 Err） | 结构化测试 |

### 覆盖率要求

| 合同层级 | 错误路径 | Proptest | Loom |
|----------|----------|----------|------|
| L0 | 100% ErrorKind | 是 | 是 |
| L1 | 100% ErrorKind | 是 | 按需 |
| L2 | 100% ErrorKind | 按需 | 按需 |
| L3 | 100% ErrorKind | 按需 | 按需 |

### 验证命令

```bash
# L2 全量
node scripts/quality-gates/check-contract-compliance.mjs --level L2 --fail-level L2

# 专项
cargo test -p kernel --all-targets
cargo test -p contract-testkit --all-targets
RUSTFLAGS='--cfg loom' cargo test -p kernel --test lifecycle_concurrency_loom --release
```

### 通过标准

- ErrorKind 覆盖率 100%
- `cargo check` 全通过

---

## 4. L3 非功能合规验证（6 规则）

| 规则 ID | 规则 | 门禁方式 |
|---------|------|----------|
| L3-NF-001 | 合同声明的性能边界不可打破 | benchmark 回归检测 |
| L3-NF-002 | 合同声明的并发安全保证不可打破 | stress test + loom |
| L3-NF-003 | 合同声明的资源上限不可打破 | resource profiler |
| L3-NF-004 | 无已知 CVE | `cargo deny check` |
| L3-NF-005 | unsafe 代码块有安全证明（L0 禁止） | 审计报告 |
| L3-NF-006 | 无 deadlock / livelock 风险 | loom / 静态分析 |

### 验证命令

```bash
node scripts/quality-gates/check-contract-compliance.mjs --level L3 --fail-level L3
cargo deny check
```

### 通过标准

- `cargo-deny check` 零 CVE
- L0 零 unsafe；L1–L3 有审计签名

---

## 5. L4 生产合规证据（4 规则）

| 规则 ID | 规则 | 证据要求 |
|---------|------|----------|
| L4-PROD-001 | 与真实后端集成测试至少一轮受控执行 | live test 日志 |
| L4-PROD-002 | 错误分类与合同声明的 ErrorKind 一致 | 错误采样报告 |
| L4-PROD-003 | latencia p99 在合同声明范围内 | 监控数据 |
| L4-PROD-004 | 运行时间 ≥ 合同声明的最短证明时间 | 运行时长记录 |

### 验证命令

```bash
node scripts/quality-gates/check-contract-compliance.mjs --level L4 --fail-level L4
```

---

## 6. CI 集成

```yaml
# .github/workflows/contract-compliance.yml

PR 触发（自动）          workflow_dispatch（手动）
├── guard               ├── L1
├── l1-signature        ├── L2
├── l2-behavior         ├── L3
└── report              ├── L4
                        └── report
```

### 失败响应

| 层级 | PR 响应 | 手动触发响应 |
|------|---------|-------------|
| L1 | 阻断 | 阻断 |
| L2 | 阻断 | 阻断 |
| L3 | 不触发 | 阻断 |
| L4 | 不触发 | 阻断 |

---

## 7. 误报与例外

```yaml
# {domain}/gate/exemptions.yaml
exemptions:
  - rule_id: L3-NF-001
    contract_id: SPEC-REDIS-001
    reason: benchmark 受 CI runner 性能影响
    adr: docs/architecture/adr/012-redis-bench-ci-noise.md
    expires: 2026-09-01
    approved_by: "@maintainer"
```

### 规则

- 每次例外必须有 ADR
- 例外有截止日期，到期自动恢复为 FAIL
- 例外不得全局禁用规则
- 不得以"测试不稳定"/"没有时间"/"其他 crate 也是如此"为由申请例外

---

## 8. 版本

| 版本 | 日期 | 修订 |
|------|------|------|
| v1.0.0 | 2026-07-24 | 初始合规验证规则定义 |
