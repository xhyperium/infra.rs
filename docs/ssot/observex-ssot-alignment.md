# observex SSOT 对齐与本仓落地状态

| 字段 | 值 |
|------|-----|
| 策略 | **B — 本仓移植 observex 0.1.0 最小面** |
| 日期 | 2026-07-21 |
| 规范 | `.agents/ssot/observex/spec/spec.md` |
| package | `xhyper-observex` 0.1.0 · lib `observex` |
| 契约面 | `xhyper-contracts` · lib `contracts`（**Instrumentation**） |
| 跟进 | L3 Instrumentation 真入口（`infra-s9t.3` / #172）；OTEL 仍 DEFER |

## 结论摘要

| 问题 | 状态 |
|------|------|
| 上游/镜像 COMPLETE 叙事 | **禁止**单独当作本仓交付证明 |
| 本仓 `crates/observex` | **已落地**（`TracingInstrumentation` + tracing 三方法） |
| 本仓 Instrumentation 契约 | `contracts::Instrumentation`；本 crate 为实现 **contracts L3 子集**（Instrumentation）的非 scaffold 入口之一 |
| OTEL exporter / flush / shutdown | **DEFER**（禁止宣称 OTEL 栈完成） |
| 完整 VenueAdapter 等 contracts | **DEFER**（非 observex 职责） |
| resiliencx / bootstrap 注入链 | **PASS**（bootstrap 默认 `TracingInstrumentation`） |
| LCOV 行覆盖率 100% | **PASS** |

## 本仓可观察事实

```text
crates/contracts/               EXISTS（xhyper-contracts + Instrumentation）
crates/observex/                EXISTS
Cargo.toml members              含 contracts + observex + adapters
package names                   contracts / observex
lib names                       contracts / observex
publish                         false
observex prod deps              xhyper-kernel, xhyper-contracts, tracing
features.default                []
```

## 验证命令

```bash
cargo test -p contracts -p observex
cargo clippy -p contracts -p observex --all-targets -- -D warnings
cargo fmt --all --check
node scripts/quality-gates/cov-gate-100.mjs -p observex --filter crates/observex/src
node scripts/quality-gates/cov-gate-100.mjs -p contracts --filter crates/contracts/src
```

CI：`.github/workflows/observex-coverage.yml` · `contracts-coverage.yml`

## Clause matrix（本仓证据）

> `PASS` = 本仓源码 + 可运行测试；`GAP` = 0.1.0 必选缺口；`DEFER` = 显式范围外。

### §1 定位 / 职责

| ID | 条款 | 状态 | 证据 |
|----|------|------|------|
| 1.1 | L1 tracing/metrics 封装，实现 Instrumentation | PASS | `crates/observex` + `impl Instrumentation` |
| 1.2 | 当前仅 tracing info，无 OTEL exporter | PASS | 源码仅 `tracing::info!`；alignment 声明 |
| 1.3 | 公开名 TracingInstrumentation（ADR 写 Observex*） | PASS | 类型 + 别名 `ObservexInstrumentation` |
| 1.4 | 非目标：业务审计 / 策略 / 全局组装 | PASS | README 非职责 |

### §2 依赖与版本

| ID | 条款 | 状态 | 证据 |
|----|------|------|------|
| 2.1 | 路径 L1 / version 0.1.0 | PASS | Cargo.toml |
| 2.2 | 依赖 kernel + contracts + tracing | PASS | observex Cargo.toml |
| 2.3 | 无 feature | PASS | `default = []` |
| 2.4 | kernel 仅信封 | PASS | `use kernel as _kernel` + allow unused |
| 2.5 | 完整上游 contracts（VenueAdapter/PubSub…） | DEFER | 本仓 contracts 含 adapter + Instrumentation；非 xhyper 全平面 |

### §3 公开 API

| ID | 条款 | 状态 | 证据 |
|----|------|------|------|
| 3.1 | TracingInstrumentation 零字段 Debug/Default/Clone/Copy | PASS | derive + unit tests |
| 3.2 | `new()` | PASS | `const fn new` |
| 3.3 | record_retry / open / close | PASS | impl + tracing 字段测试 |
| 3.4 | 无 OTEL/flush/shutdown API | PASS | 源码无此类 API |

### §4 行为与不变量

| ID | 条款 | 状态 | 证据 |
|----|------|------|------|
| 4.1 | 实现 contracts trait | PASS | `impl Instrumentation for TracingInstrumentation` |
| 4.2 | 三方法写对应 info 事件 | PASS | tracing 捕获测试 |
| 4.3 | 无 subscriber 不 panic | PASS | unit 无 subscriber 调用 |
| 4.4 | 同步、无显式 I/O/锁 | PASS | 方法体仅为 tracing 宏 |
| 4.5 | 基数/敏感性强制 | DEFER | 规范为推论；0.1.0 未强制 |
| 4.6 | OTEL 导出合同 | DEFER | 未实现 |

### §5 并发与生命周期

| ID | 条款 | 状态 | 证据 |
|----|------|------|------|
| 5.1 | Copy + Send + Sync / trait object | PASS | Copy + `dyn Instrumentation` 测试 |
| 5.2 | 无返回错误 / 无 flush 生命周期 | PASS | 方法签名 `()` |
| 5.3 | 未来失败隔离策略 | DEFER | 无 exporter |

### §6 测试合同

| ID | 条款 | 状态 | 证据 |
|----|------|------|------|
| 6.1 | 三方法不 panic | PASS | unit |
| 6.2 | Default/new/trait object/Clone-Copy | PASS | unit |
| 6.3 | 捕获 tracing 字段 | PASS | unit `tracing_fields_*` |
| 6.4 | cargo test / clippy / fmt | PASS | 本仓日志 |
| 6.5 | LCOV 100% | PASS | cov-gate-100 |
| 6.6 | resiliencx 无 observex 依赖图 | PASS | `resiliencx` dep 仅 kernel+contracts |
| 6.7 | exporter/flush 测试 | DEFER | API 未批 |

### §7 验收清单

| ID | 条款 | 状态 | 说明 |
|----|------|------|------|
| 7.1 | 依赖/API/测试与源码一致 | PASS | 本仓 |
| 7.2 | ADR 命名 | PASS | 别名 `ObservexInstrumentation` |
| 7.3 | 不把 tracing 宣称为 OTEL 完成 | PASS | 文档明确 |
| 7.4 | bootstrap 注入链 | PASS | bootstrap dep observex；默认 TracingInstrumentation |
| 7.5 | 版本步进规则 | PASS | 初始 0.1.0 |

## Core 0.1.0 必选 GAP 计数

```text
core 0.1.0 GAP = 0
```

## 未做（follow-up / DEFER）

- OpenTelemetry SDK / exporter / metric 名称 / 采样 / 缓冲 / flush / shutdown
- evidence 全量 wire 协议接入 bootstrap
- `op` 受控集合强制校验

## 与镜像文档的关系

- `.agents/ssot/observex/**`：只读镜像；禁止改 COMPLETE 叙事冒充同步
- 实现 SSOT 以 **源码 + 本仓测试/LCOV** 为准（SSOT.md R6/R7）
