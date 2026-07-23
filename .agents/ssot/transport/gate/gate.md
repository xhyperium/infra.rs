# transport — Gate

> 状态：`0.1.4` 本地实现门禁已完成；外部发布门禁仍为 OPEN。

## 本地门禁

| 门禁 | 命令 | 状态 |
|---|---|---|
| Spec 镜像 | `cmp .agents/ssot/transport/spec/spec.md .agents/ssot/transport/spec/xhyper-transportx-complete-spec.md` | PASS |
| 测试 | `cargo test -p transportx --all-targets` | PASS |
| Clippy | `cargo clippy -p transportx --all-targets -- -D warnings` | PASS |
| Rustdoc | `RUSTDOCFLAGS='-D warnings' cargo doc -p transportx --no-deps` | PASS |
| 格式 | `cargo fmt --all --check` | PASS |
| 依赖集中管理 | `node scripts/quality-gates/check-workspace-deps.mjs` | PASS |
| 隐藏公开项 | `node scripts/quality-gates/check-public-api.mjs --require-tool` | PASS；transport 源码不得出现 `#[doc(hidden)] pub` |
| R-DEP-001 `httpdate` | `cargo tree -p transportx -i httpdate` + `cargo deny check` | PASS；见下方评估 |
| 固定代码证据 | [`manifest.json`](../../../../evidence/testkit/2026-07-23-infra-2d9.10/manifest.json) | 已绑定 |

这些结果覆盖当前 IMPLEMENTED CANDIDATE 声明面，不是生产 live 证据。

## R-DEP-001：`httpdate` 依赖门禁

- 用途：解析 RFC 9110 `Retry-After` 的 HTTP-date 语法；不承担通用日期时间建模。
- 锁定：`Cargo.lock` 为 `httpdate 1.0.3`；`cargo tree -p transportx -i httpdate`
  显示仅 `transportx` 直接使用。
- 许可与来源：发布 manifest 标注 `MIT OR Apache-2.0`，repository 为
  <https://github.com/pyfisch/httpdate>。本仓 `cargo deny check` 于 2026-07-23 退出码为 0，
  advisories、bans、licenses、sources 均为 ok；同时存在与 `httpdate` 无关的 deny
  skip 配置警告。
- 替代方案：手写 RFC HTTP-date 解析易在兼容格式、时区与边界值上出错；
  `chrono` / `time` 的依赖面与能力超出本用例。继续使用小而专用的 `httpdate`。
- 维护性：2026-07-23 核验上游仓库 `archived=false`、`disabled=false`、
  默认分支 `main`、最近 push 为 2024-12-22、open issues 为 4；GitHub release
  列表未包含 `1.0.3`，但 crates 发布不以 GitHub release tag 为必要条件。
  裁定为“低频稳定、非 archived”，不外推为活跃维护；需随
  `cargo deny` / Dependabot 持续复核。

## 外部门禁

| 门禁 | 当前状态 | 放行主体 |
|---|---|---|
| PR CI | OPEN | 远端 CI |
| 独立终审 | OPEN | 独立 reviewer |
| 人工批准 | OPEN | maintainer |
| Merge | OPEN | 仓库治理流程 |

## NO-GO

企业 PKI/mTLS、证书轮换、M3、真实业务 live 与 package stable 不在本轮证明范围，均为
**NO-GO**。外部门禁全部闭合前，不得将本候选标记为 released。
