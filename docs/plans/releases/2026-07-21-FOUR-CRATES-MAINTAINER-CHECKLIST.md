# Maintainer checklist — 四包内部发布收尾

| 字段 | 值 |
|------|----|
| 日期 | 2026-07-21 |
| 范围 | `kernel` · `testkit` · `decimalx` · `canonical` |
| 代码基线 | `main` · `#159` · `8fbd5ef` |
| 证据包 | [`2026-07-21-four-crates-internal-release.md`](./2026-07-21-four-crates-internal-release.md) |
| L5 权威 | 仍以 [`0.3.0-signoff.md`](./0.3.0-signoff.md) 为准（GO-with-Accepts · `@ZoneCNH`） |
| 本文件 | **人工执行清单**；Agent 不得代签 / 不得擅自打生产 tag |

---

## 0. 当前状态（已完成，无需 Maintainer 重复）

- [x] 四包在声明层级内的实现、测试、bench、examples、docs 合入 `main`（PR #159）
- [x] CI 全绿后 squash merge
- [x] 内部证据包落盘（**DRAFT · GO for declared tiers**）
- [x] `publish = false` 保持；**未**批准 crates.io
- [x] 未宣称 workspace 整体 Production Ready

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

## 3. 可选：git tag（内部追溯，≠ crates.io）

仅在 Maintainer 确认需要版本锚点时执行。

```bash
# 在干净 main 上
git fetch origin main
git checkout main && git pull --ff-only origin main
git rev-parse HEAD   # 期望含 #159（8fbd5ef 或其后的 main tip）

# 推荐 annotated tag 名称（示例，按仓库惯例二选一）
# A) 与 workspace 0.3.0 对齐的「四包收口」注记 tag
git tag -a v0.3.0-four-crates -m "Internal GO: kernel L1+L4, testkit L1 test-support, decimalx L1, canonical L2 wire subset (#159)"

# B) 若仓库另有统一 v0.3.0 tag 策略，用 changelog 注记代替新 tag，避免重复
git push origin v0.3.0-four-crates   # 仅 A 且已确认后
```

**不要**：

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

## 6. 完成后勾选（Maintainer）

- [ ] 已读证据包与 Accept 列表
- [ ] （可选）证据包 §6 已手签并合入
- [ ] （可选）内部 tag 已推送或明确跳过
- [ ] 知悉：消费者按声明层级使用；testkit 仅 dev-dep

---

## 7. 链接

| 文档 | 用途 |
|------|------|
| [four-crates-internal-release](./2026-07-21-four-crates-internal-release.md) | 四包证据 |
| [0.3.0-signoff](./0.3.0-signoff.md) | L5 权威 GO-with-Accepts |
| [PR #159](https://github.com/xhyperium/infra.rs/pull/159) | 实现合入 |
| [defer-disposition](../artifacts/defer-disposition.md) | Accept 风险 |
| [support-matrix](../../governance/support-matrix.md) | Linux + MSRV 1.85 |
