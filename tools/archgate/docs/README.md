# archgate 维护入口

- 扫描实现与单元测试：`src/main.rs`、`src/kernel_rules.rs`
- 时间调用例外：根目录 `.architecture/exceptions.toml`
- CI 入口：`.github/workflows/ci.yml` 的 `architecture-drift` job；kernel loom 见 `kernel-loom` job

### KERNEL-*（SPEC-KERNEL-002 §12.2）

| 规则 | 含义 |
|------|------|
| KERNEL-DEP-001/002 | workspace 依赖数=0；生产外部仅 thiserror |
| KERNEL-FEATURE-001 | 仅 `default = []` |
| KERNEL-API-001 | 用固定版本 `cargo-public-api` 从当前 `xhyper-kernel` 源码生成规范化 public API，并与 candidate snapshot 比较；工具缺失/版本不符/生成失败 fail closed；禁 Component/serde 面 |
| KERNEL-API-002 | 以冻结 baseline + FNV 指纹为基准：removal/signature change 一律拒绝；addition 须在 `kernel-api-rfc.toml` 逐行登记 **Approved** RFC；不得回退到仅比较两份人工快照 |
| KERNEL-TIME-001/002/003/004 | SystemTime / Instant::now / from_unix_nanos / from_clock_elapsed 边界 + allowlist |
| KERNEL-ERR-001/002 | `XError::internal` 棘轮；禁止字符串分类 |
| KERNEL-SERDE/ASYNC/UNSAFE-001 | kernel 无 serde/tokio/unsafe 用法 |
| KERNEL-LIFECYCLE-001 | loom 测试资产存在（执行由 CI `kernel-loom`） |

规则变更须引用已批准的架构标准或决策，不从当前代码反推批准状态。

控制面文件级 consumer/enforcement 与生产 **NO-GO** 对齐视图：  
`docs/report/2026-7-17/architecture-control-plane-alignment.md`（**16/16** KERNEL-* 含 API 源码生成与 PUBLISH-001；`policies/public_api.toml` 等仍为 advisory，非本工具解析对象）。
