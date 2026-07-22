# transport maintenance evidence（2026-07-23）

Baseline：`3cd29a942710c0fb42f3f6bc05e3c31570acad47`；worktree：`feat/infra-2d9.8-transport-contracts`。

## 三轮审计索引

| 轮次 | 证据 | 结论 |
|---|---|---|
| Round 1 事实 | base `3cd29a9`；源码/测试/manifest/历史四向核对；初始 diff 为空 | 发现 HTTP/WS 分配前上限、URL Debug、SNI、pool lease、Retry-After 与文档追溯缺口 |
| Round 2 规格设计 | `goal/design/spec/matrix/test/gate` 与 active 双镜像；`cmp` exit 0 | 冻结 `HttpDriver` / `WsConnector` / 对象池 seam；M3/企业 PKI/业务 live 保持 NO-GO |
| Round 3 实现验证 | 下列逐 seam Red→Green 与最终命令/exit；独立 reviewer 另见 `review/` | 声明面实现完成；发布仍受独立审查、PR/CI/人工审批约束 |

## Round 3 Red → Green

| Seam | Red 命令/exit | 失败事实 | Green |
|---|---|---|---|
| HTTP chunk | `cargo test ... chunked_response_stops_at_first_cumulative_overflow` / 101 | 等待响应结束，500ms 超时 | 同命令 / 0 |
| WS inbound | `cargo test ... ws_inbound_limit_is_enforced_by_decoder_before_delivery` / 101 | 聚合后才报 `ws_frame` | 同命令 / 0，decoder 报 `ws_message` |
| URL Debug | `cargo test ... request_and_proxy_debug_redact_url_userinfo_and_all_query_values` / 101 | userinfo/query 原文泄漏 | 同命令 / 0；全部 query value fail-closed |
| Pool | `cargo test -p transportx --test pool_contracts` / 101 | `try_new`/lease API 不存在 | 同命令 / 0 |
| Retry-After | `cargo test ... retry_after_parser_supports_delay_seconds_and_http_date` / 101 | parser seam 不存在 | 同命令 / 0 |
| SNI | 独立 Red **NOT_RUN** | 源码审计证明字段被忽略；随后与 Debug/TLS 绿测一并修复 | `disabled_sni...` + `tls_defaults` / 0 |

## Final validation

| 命令 | exit | 结果 |
|---|---:|---|
| `cargo fmt --all --check` | 0 | PASS |
| `cargo test -p transportx --all-targets` | 0 | PASS（73 tests，bench/example 亦执行） |
| `cargo clippy -p transportx -p contracts --all-targets --all-features -- -D warnings` | 0 | PASS（首次因 manual saturating arithmetic exit 101，修正后绿） |
| `cargo doc -p transportx -p contracts --all-features --no-deps` | 0 | PASS |
| `cargo test -p binancex -p okxx --all-targets` | 0 | PASS；两条公网 live tests ignored |
| `check-crate-versions.mjs` / `check-workspace-deps.mjs` | 0 / 0 | PASS |
| `node scripts/quality-gates/check.mjs` | 0 | PASS，Harness 44/44；`STATUS.md` 已由生成器刷新 |
| workspace fmt/clippy/test/doc + `cargo deny check` | 0 | PASS；`cargo deny` 仅报告既有 skip 配置 warning |
| `node scripts/quality-gates/cov-gate-100.mjs -p transportx --filter crates/transport/src` | 0 | PASS，610/610 个 LCOV `DA` 行命中，100% |
| `cargo llvm-cov -p transportx --all-targets --fail-under-lines 100 --summary-only` | 1 | 97.94%；该统计把同一已命中源码行内的未命中 region 计为 missed line，作为加严诊断保留，不替代仓内 LCOV 行门禁 |
| 双镜像 `cmp` | 0 | PASS |

## Residual OPEN / NO-GO

- M3、企业 PKI/mTLS、WS 企业 TLS、公网/长稳与完整业务 live 均无证据，保持 NO-GO。
- 独立 Standards 与 Spec reviewer 均已 PASS；maintainer 审批与 GitHub CI 仍为发布门禁。
