# INFRA-008 — cargo-semver-checks 使用说明（TARGET INTERFACE）

状态：`PARTIAL — tool+baseline wired / NOT WP ACCEPTED`
工作包：INFRA-008 Public API / Additive Only 基线
相关：G8 blocker evidence、`contracts` additive-only 纪律
工具 pin：`cargo-semver-checks 0.48.0`

> 本文件是 **AI 可执行脚手架文档**，不是完整 DoD 关闭证明。
> `semver-check` 对 `contracts`/`kernel`/`decimalx`/`canonical` 的 check-release **通过 ≠ INFRA-008 ACCEPTED**。
> 仍缺：负向 fixture、CI required pin、全 workspace 覆盖。

## 1. 目标

对发布面 crate（尤其 `contracts`、稳定 types）检测：

- 删除/改签名的 public 方法
- supertrait / bounds / 关联类型破坏
- 非 additive 的 breaking 变更

在 adapter 实现前于 CI/本地 **fail-closed** 阻断。

## 2. 工具固定

推荐 pin（本机已验证）：

```bash
# 安装时请禁用失败的 RUSTC_WRAPPER/sccache（若遇权限错误）
env -u RUSTC_WRAPPER cargo install --locked cargo-semver-checks --version 0.48.0
cargo semver-checks --version   # 期望：cargo-semver-checks 0.48.0
```

Baseline tags（本地已建；推送可选）：

```text
contracts-v0.1.0
kernel-v0.1.0
decimalx-v0.1.0
canonical-v0.1.0
```

xtask 入口：

```bash
cargo run -p xhyper-xtask -- semver-check --json
cargo run -p xhyper-xtask -- semver-check
```

| 条件 | `status` | exit |
|------|----------|------|
| 未安装工具 | `TOOL_MISSING` | ≠ 0 |
| 有工具、无 `<crate>-v*` tag | `BASELINE_MISSING` | ≠ 0 |
| 有工具+tag，check-release 失败 | `BREAKING_OR_ERROR` | ≠ 0 |
| 有工具+tag，检查通过 | `PASS` | 0 |

## 3. Baseline 策略（草案）

1. **优先**：git tag `<crate>-vX.Y.Z`（与仓库 release 约定一致）
2. **次选**：crates.io 已发布版本（仅 publish=true 的 crate）
3. **禁止**：无 baseline 时用“当前 HEAD 对比当前 HEAD”冒充 PASS
4. `contracts` 发布后 **additive-only**：可增方法/类型，不可破既有签名

示例（工具可用且存在 baseline 后手工执行）：

```bash
# 以最近 contracts tag 为 baseline（示例，tag 名以仓库实际为准）
cargo semver-checks check-release -p contracts

# 或指定 baseline 版本
cargo semver-checks check-release -p contracts --baseline-version 0.1.0
```

## 4. Fixture / 负向自测（已入库脚本）

| 路径 | 用途 |
|------|------|
| `fixtures/semver/README.md` | 协议说明 |
| `scripts/infrastructure/semver-negative-selftest.sh` | 临时仓：additive 必绿、删 pub 方法必红 |

```bash
bash scripts/infrastructure/semver-negative-selftest.sh
```

CI：`scripts/ci-optional-ssot-checks.sh`（optional workflow，**非** required check）在工具可用时运行负向自测。

完整 DoD（全 crate 覆盖 + required CI pin）落地前，INFRA-008 **不得** ACCEPTED。

## 5. 与 Evidence / Gate

- 运行结果应沉淀为 `*.evidence.json`（INFRA-003），含 commit / toolchain / command / exit_code
- G8 历史 blocker 见 `evidence/ci/g8-semver-blocker-2026-07-13.md`
- runner 中 `cargo-semver-checks` SKIP **不计** PASS（INFRA-041）

## 6. 非目标

- 不在本切片安装全局工具或修改 CI required checks
- 不创建平行 `infragate` crate
- AI **不得**在无工具输出时手写 PASS Evidence
