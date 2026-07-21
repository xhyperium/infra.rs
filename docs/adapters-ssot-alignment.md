# adapters SSOT 对齐与本仓落地状态

| 字段 | 值 |
|------|-----|
| 域 | `adapters/`（exchange + storage） |
| 镜像 | `.agents/ssot/adapters/**`（R6 只读；**禁止**改镜像冒充本仓完成） |
| 本仓路径 | `crates/adapters/{exchange,storage}/<name>` |
| 审计日期 | 2026-07-21 |
| 结论 | **SSOT 镜像已本地化注册**；**9 adapter + contracts 已 workspace 注册**；标准布局/`publish=false` 已补齐；**未**宣称业务实现 / package stable / ship |

## 结论摘要

| 问题 | 状态 |
|------|------|
| 上游镜像 COMPLETE / Spec Approved 叙事 | 描述的是 **xhyper monorepo 战役**；**禁止**单独当作本仓交付证明 |
| 镜像同步 | **完整**：与 `xhyper.rs/.agent/SSOT/adapters/` `diff -rq` = 0（144 文件 / 1.1M） |
| 本仓 adapter crates | **scaffold 已注册**（#42）；`Error` + `Result` 骨架；**无**业务实现 |
| `crates/AGENTS.md` 标准七项布局 | **已齐**（README / AGENTS / CHANGELOG / examples / docs / tests） |
| `publish = false` | **已**在 adapter + contracts `Cargo.toml` 显式关闭 |
| package stable / crates.io | **未**宣称 |
| contracts trait crate | **已** workspace member（#43 `xhyper-contracts`）；trait scaffold，非业务实现 |

## 镜像目录（只读）

保留上游 `adapters/` 层级（与 `infra/` 同理，勿展平到 `.agents/ssot/` 根）。

```text
.agents/ssot/adapters/
├── exchange/
│   ├── binance/     # → crates/adapters/exchange/binance · package binancex
│   └── okx/         # → crates/adapters/exchange/okx     · package okxx
└── storage/
    ├── clickhouse/  # → crates/adapters/storage/clickhouse · package clickhousex
    ├── kafka/       # → crates/adapters/storage/kafka      · package kafkax
    ├── nats/        # → crates/adapters/storage/nats       · package natsx
    ├── oss/         # → crates/adapters/storage/oss        · package ossx
    ├── postgres/    # → crates/adapters/storage/postgres   · package postgresx
    ├── redis/       # → crates/adapters/storage/redis      · package redisx
    └── taos/        # → crates/adapters/storage/taos       · package taosx
```

每个域目录对齐 kernel 11 层布局：`goal/spec/design/plan/tasks/prompt/test/review/release/retrospective` + `matrix/gate/evidence` + `README.md`。

双镜像：`spec/spec.md` ≡ `spec/xhyper-*-complete-spec.md`（`cmp` 同构；镜像内已满足）。

## 本仓可观察事实

```text
Cargo.toml members              含 crates/contracts + 9 个 crates/adapters/**
package 命名                    xhyper-contracts；binancex / okxx / redisx / …（adapter 无 xhyper- 前缀）
lib 入口                        adapters: Error/Result；contracts: ExchangeAdapter/StorageAdapter
生产依赖                        adapters: thiserror；contracts: serde + thiserror
实现深度                        scaffold（adapters 无 client；contracts 仅 trait）
标准布局七项                    已齐
publish                         false（显式）
version                         workspace 0.3.0
```

| 镜像路径 | 本仓路径 | package | 本仓状态 |
|----------|----------|---------|----------|
| `.agents/ssot/adapters/exchange/binance` | `crates/adapters/exchange/binance` | `binancex` | scaffold |
| `.agents/ssot/adapters/exchange/okx` | `crates/adapters/exchange/okx` | `okxx` | scaffold |
| `.agents/ssot/adapters/storage/clickhouse` | `crates/adapters/storage/clickhouse` | `clickhousex` | scaffold |
| `.agents/ssot/adapters/storage/kafka` | `crates/adapters/storage/kafka` | `kafkax` | scaffold |
| `.agents/ssot/adapters/storage/nats` | `crates/adapters/storage/nats` | `natsx` | scaffold |
| `.agents/ssot/adapters/storage/oss` | `crates/adapters/storage/oss` | `ossx` | scaffold |
| `.agents/ssot/adapters/storage/postgres` | `crates/adapters/storage/postgres` | `postgresx` | scaffold |
| `.agents/ssot/adapters/storage/redis` | `crates/adapters/storage/redis` | `redisx` | scaffold |
| `.agents/ssot/adapters/storage/taos` | `crates/adapters/storage/taos` | `taosx` | scaffold |

验证（本仓权威命令）：

```bash
# 镜像完整性（相对上游）
diff -rq /home/workspace/xhyper.rs/.agent/SSOT/adapters .agents/ssot/adapters

# 双镜像
for d in exchange/binance exchange/okx \
  storage/clickhouse storage/kafka storage/nats storage/oss \
  storage/postgres storage/redis storage/taos; do
  cmp .agents/ssot/adapters/$d/spec/spec.md \
      .agents/ssot/adapters/$d/spec/xhyper-*-complete-spec.md
done

# scaffold 可构建
cargo check -p binancex -p okxx -p redisx -p kafkax -p natsx \
  -p postgresx -p taosx -p ossx -p clickhousex
cargo test --workspace --all-targets
```

## 对齐矩阵（本仓证据，非镜像勾选）

| ID | 条款 | 状态 | 本仓证据 |
|----|------|------|----------|
| A-1 | 镜像路径保留 `adapters/` 层级 | PASS | `.agents/ssot/adapters/{exchange,storage}/` |
| A-2 | 与上游 `diff -rq` 无差异 | PASS | 同步报告 2026-07-21 |
| A-3 | 9 域均有 README + 11 层 + dual-spec | PASS | 镜像树；`cmp` 全 OK |
| A-4 | 本仓 crate 路径与 SSOT Code 列一致 | PASS | `crates/adapters/...` 与镜像 README Code 列同构 |
| A-5 | workspace members 已注册 9 package | PASS | 根 `Cargo.toml` |
| A-6 | scaffold 可 `cargo check` / `cargo test` | PASS | 0 业务测试；编译通过 |
| A-7 | 标准七项布局 | PASS | 9 adapters + contracts 均已补齐 |
| A-8 | `publish = false` | PASS | 各 `Cargo.toml` 显式关闭 |
| A-9 | 实现真实 I/O / adapter 业务 | OPEN | 无 client；contracts 仅 trait |
| A-10 | package stable / Spec Approved 本仓宣称 | OPEN | **禁止**用镜像 COMPLETE 代替 |
| A-11 | contracts workspace 注册 | PASS | #43 `crates/contracts` → `xhyper-contracts` |

## 与镜像文档的关系

- `.agents/ssot/adapters/**`：只读镜像；禁止本地改 CLOSED/COMPLETE 叙事冒充同步
- 镜像内验证命令仍写 `.agent/SSOT/...`（上游 monorepo 路径）；**本仓**以 `.agents/ssot/adapters/...` 与 `crates/adapters/...` 为准
- 实现 SSOT 以 **源码 + 本仓测试输出** 为准
- 详见 `.agents/ssot/SSOT.md` R6 / R7 与根 `AGENTS.md`

## 依赖与边界（本仓意图）

```text
adapters/*  →  (future) contracts / types / kernel
               禁止  kernel/types 反向依赖 adapters
               禁止  把 adapter 当 L0
```

- 当前 scaffold **仅**依赖 `thiserror`
- 未来引入 `canonical` / 外部 SDK 时，须在对应对齐文档与 PR 中显式声明
- 真实交易 / 生产 I/O **默认禁止**，直至 Spec + 证据批准（镜像 binance 规范亦声明 mock 边界）

## `crates/contracts` 评估结论

| 问题 | 结论 |
|------|------|
| 是否应并入 workspace？ | **已并入**（#43），无需再次注册 |
| package | `xhyper-contracts` · path `crates/contracts` |
| 角色 | adapter trait 出口；**不是**上游 `.agents/ssot` 独立域 |
| 实现深度 | trait + 共享类型 scaffold |
| 标准布局 / publish | 本 PR 补齐 |
| 风险 | `Ticker` 使用 `f64` 金额字段 — 与 decimalx 禁令冲突；**禁止**宣称 stable 直至收口 |

## 未做（follow-up / OPEN）

1. 按域实现 adapter（mock → 集成测 `#[ignore]` → M3）— 一域一战役，禁止混做
2. adapters 依赖 `xhyper-contracts` 并实现 trait
3. contracts：`Ticker` 等金额字段迁离 `f64`（改 decimalx / canonical）
4. package 命名是否统一 `xhyper-*` 前缀 — 需 Lead 裁决；当前保留 #42 / #43 命名

## 相关索引

- 总览：[workspace-ssot-alignment.md](./workspace-ssot-alignment.md)
- 同步报告：[SSOT_SYNC_REPORT.md](./SSOT_SYNC_REPORT.md)
- 规则：[.agents/ssot/SSOT.md](../.agents/ssot/SSOT.md)
- 初始化 commit：`b586260` feat: initialize adapter crates (#42)
