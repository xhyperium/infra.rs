# Maintainer checklist — 四包内部发布收尾

| 字段 | 值 |
|------|----|
| 日期 | 2026-07-21 |
| 范围 | `kernel` · `testkit` · `decimalx` · `canonical` |
| 代码基线 | `main` · `#159` + `#160` · tip `5acac34`（tag 锚点） |
| 内部 tag | **`v0.3.0-four-crates`** → `5acac34`（已推送 `origin`，2026-07-21） |
| 证据包 | [`2026-07-21-four-crates-internal-release.md`](./2026-07-21-four-crates-internal-release.md) |
| L5 权威 | 仍以 [`0.3.0-signoff.md`](./0.3.0-signoff.md) 为准（GO-with-Accepts · `@ZoneCNH`） |
| 本文件 | **收尾清单**；Agent **不得**代写 `Signed-off-by` |

---

## 0. 当前状态（已完成）

- [x] 四包在声明层级内的实现、测试、bench、examples、docs 合入 `main`（PR #159）
- [x] CI 全绿后 squash merge
- [x] 内部证据包落盘（**DRAFT · GO for declared tiers**）
- [x] Maintainer 收尾清单合入（PR #160）
- [x] 内部 annotated tag **`v0.3.0-four-crates`** 已推送 `origin`（指向 `5acac34`）
- [x] `publish = false` 保持；**未**批准 crates.io
- [x] 未宣称 workspace 整体 Production Ready
- [x] 发布后抽查：`cargo test` / clippy / fmt / public-api / examples / benches（2026-07-21 会话复验全绿）

---

## 1. 必做阅读（约 10 分钟）

1. 证据包：[`2026-07-21-four-crates-internal-release.md`](./2026-07-21-four-crates-internal-release.md)
2. 声明层级表（证据包 §1）与 Accept 风险（§5）
3. 确认红线：非 crates.io、非整体 PR、testkit ≠ 生产 runtime

---

## 2. 可选：在证据包上人工签字

若希望四包 tranche 有**独立** Maintainer 记录（非必须，因 `0.3.0-signoff` 已覆盖 L5）：

1. 打开 `2026-07-21-four-crates-internal-release.md` §6
2. **人工**填写（禁止 Agent 代写）：

   ```text
   Signed-off-by: @<your-handle> <YYYY-MM-DD>
   Verdict: GO for declared tiers only
   ```

3. 将文首状态从 `DRAFT · GO for declared tiers` 改为 `SIGNED · GO for declared tiers`
4. 单独 PR 合入；commit message 示例：

   ```text
   docs(releases): maintainer sign four-crates internal GO
   ```

**不要**：改成「Production Ready」营销表述；不要顺带 `publish = true`。

---

## 3. git tag（内部追溯，≠ crates.io）— **已完成**

| 项 | 值 |
|----|-----|
| Tag | `v0.3.0-four-crates` |
| 对象 | annotated → commit `5acac34` |
| 远程 | `origin` 已推送 |
| 并存 | 仓库另有 `v0.3.0` / `v0.3.18`；本 tag 为四包收口注记，**不**替代 crates.io |

校验：

```bash
git fetch --tags
git show v0.3.0-four-crates --no-patch
# 期望：tag 存在且 peels 到 5acac34（或 main 上等价 tip）
```

**不要**（仍有效）：

- 用 tag 暗示 crates.io 已发布
- `cargo publish`
- force-push `main` 或改写已推送 tag

---

## 4. 发布后本地验证（可选抽查）

```bash
cargo test -p kernel -p testkit -p decimalx -p canonical --all-targets
cargo run -p kernel --example basic
cargo run -p testkit --example basic
cargo run -p decimalx --example basic
cargo run -p canonical --example basic
node scripts/quality-gates/check-public-api.mjs
```

期望：exit 0；example 打印 `*-consumer: ok`。

---

## 5. 明确不做

| 项 | 原因 |
|----|------|
| crates.io publish | `publish = false`；本 tranche 未批准 |
| 改 README 为「Production Ready」 | 分层口径；见 0.3.0-signoff 红线 |
| 扩大到 contracts/adapters | 范围外；另开计划 |
| Agent 代签 / 代打生产 tag | 宪章与签核模板禁止 |

---

## 6. 完成后勾选

- [x] 证据包与 Accept 列表已可读且合入
- [ ] （**仍可选 · 仅人工**）证据包 §6 `Signed-off-by` 手签并合入 — Agent **未**代签；L5 权威继续用 `0.3.0-signoff.md`
- [x] 内部 tag `v0.3.0-four-crates` 已推送
- [x] 知悉口径：消费者按声明层级使用；testkit 仅 dev-dep；非 crates.io

---

## 7. 链接

| 文档 | 用途 |
|------|------|
| [four-crates-internal-release](./2026-07-21-four-crates-internal-release.md) | 四包证据 |
| [0.3.0-signoff](./0.3.0-signoff.md) | L5 权威 GO-with-Accepts |
| [PR #159](https://github.com/xhyperium/infra.rs/pull/159) | 实现合入 |
| [defer-disposition](../artifacts/defer-disposition.md) | Accept 风险 |
| [support-matrix](../../governance/support-matrix.md) | Linux + MSRV 1.85 |
