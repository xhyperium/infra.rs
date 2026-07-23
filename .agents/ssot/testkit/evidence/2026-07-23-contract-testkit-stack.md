# contract-testkit 0.1.2 临时叠加证据

<!-- cspell:ignore RUSTUP -->

> 本文件只证明临时 stack 上的候选树；不是最终 PR、人工批准、发布或 live backend evidence。PR #256 合并后必须在最新 `main` 重放自有提交并重新生成最终证据。

| 字段 | 值 |
|------|-----|
| 日期 | 2026-07-23 |
| worktree | `.worktrees/feat/infra-2d9.9.1.1-contract-testkit-stack` |
| integration base | `2dedcc60013d4ddfe3d7278ea969ceadd5165a52` |
| verified candidate | `ddd53f37c5e26d55d4d23647b3e6e597e7425d19` |
| rustc / cargo | 1.97.0 / 1.97.0 |
| node | v24.14.0 |

## 原始门禁摘要

| 命令 | exit | 原始结果摘要 |
|------|------|--------------|
| `cargo test -p contract-testkit --test negative_implementations` | 0 | `15 passed; 0 failed` |
| `cargo test -p contract-testkit --test suite_self_tests` | 0 | `13 passed; 0 failed` |
| `cargo test --workspace --all-features --all-targets --quiet` | 0 | 全 workspace 测试与 bench target 完成；live tests 按显式条件 ignored/skipped |
| `node --test scripts/quality-gates/check-test-support-graph.test.mjs` | 0 | `tests 12` · `pass 12` · `fail 0` |
| `node scripts/quality-gates/check-test-support-graph.mjs --json` | 0 | `{"ok":true,"testSupportPackages":["contract-testkit","testkit"],"findings":[]}` |
| `RUSTUP_TOOLCHAIN=nightly node scripts/quality-gates/check-public-api.mjs -p contract-testkit --require-tool` | 0 | `OK` · baseline matches |
| `cargo clippy -p contract-testkit -p contracts --all-targets -- -D warnings` | 0 | `Finished dev profile` |
| `RUSTDOCFLAGS='-D warnings' cargo doc -p contract-testkit -p contracts --no-deps` | 0 | 文档生成成功 |
| `node scripts/quality-gates/check-workspace-deps.mjs` | 0 | `PASS` |
| `node scripts/quality-gates/check-crate-versions.mjs` | 0 | `PASS` |

图门禁负测明确覆盖 direct/transitive normal、build、all-features-only、target-specific、inventory 缺包、`resolve=null`、workspace member 缺 node，以及 cargo metadata 执行失败时的结构化 JSON FAIL。

兼容修复额外验证：AnalyticsSink 核心失败 case 保持 `sink`；TimeSeries fixture wrapper 通过自定义半开区间 backend；`FixtureNamespace::resource` 总长 63 字节成功、64 字节失败。

## 已知非通过项与边界

- Harness 健康检查为 45/46；唯一失败是 `STATUS.md crates 看板新鲜`。临时 stack 按 advisor 约束禁止修改 `STATUS.md`，最终重放后必须生成并复跑。
- 2026-07-23 验证期间 `origin/main` 已前进到 `5fe242c`（PR #257），并含部分 Batch2 suite；最终不得机械重放版本/重复 API，必须做 additive-only 冲突审计。
- 未运行 Sandbox/Real/Testnet；未声明 EventBus/PubSub 可移植 delivery/replay/order、ObjectStore 持久化或跨资源事务原子性。
