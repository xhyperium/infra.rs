# kafkax 独立审查报告

| 字段 | 值 |
|------|-----|
| 日期 | 2026-07-23 |
| 角色 | 独立审查者（非实现者） |
| 工作树 | `/home/workspace/infra.rs/.worktrees/feat/kafkax-spec-goal-close` |
| 对比基线 | `origin/main`（意图；本会话以 worktree 落盘文件静态审阅为主） |
| package | `kafkax` |
| Cargo.toml version | `0.3.4`（`publish = false`） |
| 审查范围 | `crates/adapters/storage/kafka` · `docs/ssot/*kafkax*` / adapters alignment · `.agents/ssot/adapters/storage/kafka` |

## Verdict

**Approve**

六条验收标准均满足；存在 1 条 **P2 非阻塞** 文档漂移 follow-up，不构成 Request changes / Block。

---

## 验收对照

| # | 标准 | 结论 | 摘要 |
|---|------|------|------|
| 1 | draft 十轮矩阵已落盘且 NO-GO/OOS 显式 | **PASS** | `evidence/kafkax-10pass-matrix.md` 含 R1–R10 + 条款表；GROUP/REB/EOS-N/STABLE 等 **NO-GO**；P2-* **OOS** |
| 2 | 代码 GAP 仅 rskafka 兼容（builder/timestamp/行为测试） | **PASS** | `KafkaConfigBuilder` + 校验测试；`KafkaMessage::timestamp` 消费侧透传；公共 API 行为测覆盖 connect 拒绝 / offset / PTC |
| 3 | 无私钥/密码入库 | **PASS** | default 无凭据；Debug 脱敏；配置/测试仅用占位 secret；live 走 env |
| 4 | 未假宣称 package stable / group / EOS | **PASS** | 多处显式 **NOT CLAIMED / NO-GO**；`Eos*` 为 deprecated 别名并文档声明非 EOS |
| 5 | 对齐文档 version 与 Cargo.toml 一致 | **PASS**（附 P2 漂移） | `kafkax-ssot-alignment` / goal / release / crate CHANGELOG / table 行均为 `0.3.4`；见 Blocking 外 follow-up |
| 6 | 公共 API 测试行为驱动而非仅 assert_type | **PASS** | `public_api_surface` 执行 build/commit/fail-closed/connect 拒绝；非 `size_of`/类型存在断言 |

---

## Blocking Issues

无。

---

## Non-blocking follow-ups（P2）

### F1. `adapters-ssot-alignment.md` 摘要行 version 过期

- **位置**：`docs/ssot/adapters-ssot-alignment.md:57`
- **现象**：`text` 摘要仍写 `redis/kafka/nats 0.3.2`，同文件表格（约 L63）已正确写 `kafkax` **`0.3.4`**，且 redis/nats 实际版本亦已前进。
- **影响**：读者若只扫摘要行会误判 kafkax 版本；表格与专项对齐文档正确，故不阻断。
- **建议**：合并前或 follow-up 将摘要行改为与各 crate `Cargo.toml` 一致（或删除易腐朽的版本枚举，只保留表格）。

### F2. `timestamp` 无独立字段语义断言

- **位置**：`src/message.rs` 字段存在；`src/consumer.rs:119` 已 `timestamp: Some(record.timestamp)`
- **现象**：离线测试构造 `timestamp: None` 但未断言非空/映射语义（需 broker 才能端到端验证 record 时间戳）
- **影响**：P2 可测性缺口；不否定字段交付与接线
- **建议**：可在 message 单测中断言 `Some(Utc::now())` round-trip 持有；broker 级 timestamp 保留在 conformance/live

---

## 分项证据

### 1. 十轮矩阵 + NO-GO/OOS

**文件**：`.agents/ssot/adapters/storage/kafka/evidence/kafkax-10pass-matrix.md`

- L8–L20：R1–R10 记录；R1 合法漂移到 `rskafka`；R10 Part2 **OOS**
- L44–L51 显式：
  - `GROUP` / `REB` / `RECON` / `EOS-N` / `SCHEMA` / `SCRAM` / `HA` / `STABLE` → **NO-GO**
  - `P2-*` → **OOS**
- L63–L65：收敛定义「可交付面闭合 + 其余显式 NO-GO/OOS」

交叉：

- `matrix/matrix.md` L14–L17：S-10 stable NO-GO；S-11 group 等 NO-GO；S-12 Part2 OOS；S-13 十轮矩阵 PASS
- `goal/goal.md` L9–L10 version `0.3.4`；L27–L30 NO-GO/OOS 列表
- `release/release.md` L8–L10：`publish=false`；SemVer package stable **未宣称**
- `review/review.md` L11：package stable **NOT CLAIMED**

### 2. 代码 GAP = rskafka 兼容面

| 能力 | 证据 |
|------|------|
| Builder | `src/config.rs:51–151` `KafkaConfigBuilder`；测试 `config.rs:457–475` loopback 通过 + 远程明文拒绝 |
| timestamp | `src/message.rs:48–49` 字段；`src/consumer.rs:112–119` 从 record 映射 |
| 行为测试 | `src/lib.rs:81–188` `default_exports_behavior_paths`：builder build、Memory/File offset commit、PTC fail-closed、`KafkaPool::connect` 拒绝 |

`lib.rs` 模块文档 L28–L33 权威边界与矩阵一致（驱动 rskafka；NO-GO/OOS 列表）。

### 3. 密钥 / 密码

| 检查 | 结果 | 证据 |
|------|------|------|
| default 无默认账号 | PASS | `config.rs:158–161` 注释「无默认账号」+ `None` 凭据 |
| Debug 脱敏 | PASS | `config.rs:172–187` username/password `***`；测试 `config.rs:356–370` 拒绝明文密码出现在 Debug |
| 测试占位 | PASS | 单测用 `"secret"` / `"super-secret-kafka"` 仅内存构造，非真实密钥文件 |
| live 凭据路径 | PASS | alignment / evidence 声明 env + scratch `0600`；仓库不入库 |
| Cargo / publish | PASS | `Cargo.toml:9` `publish = false` |

未发现私钥 PEM、生产密码或 `.env` 入库痕迹于 kafkax 源码与本轮 SSOT 证据文件。

### 4. 未假宣称 stable / group / EOS

| 声明点 | 表述 |
|--------|------|
| `lib.rs:8–9,31–33` | EventBus/Consumer **at-most-once**；group/native EOS **NO-GO** |
| `eos.rs:1–15` | 非原子 produce-then-checkpoint；**不提供** Kafka 原生事务 / EOS |
| `CHANGELOG.md:16–17,29–30` | 未宣称 package stable；native group EOS 仍 NO-GO |
| `releases/0.3.4.md:20–24` | 非宣称列表含 package stable / group / native EOS / Part2 |
| `spec/spec.md:2,15–20` | 未宣称 package stable；`ProduceThenCheckpoint*` 明确非 EOS；旧 `Eos*` 仅 deprecated 别名 |
| `README.md`（crate）L7–10,49–50 | AMO / 非 group；harness 不证明 group/rebalance/HA/native EOS |

### 5. version 对齐

| 位置 | version |
|------|---------|
| `crates/adapters/storage/kafka/Cargo.toml:4` | `0.3.4` |
| `CHANGELOG.md` `[0.3.4]` | 一致 |
| `docs/ssot/kafkax-ssot-alignment.md:9` | `0.3.4` |
| `goal/goal.md:10` | `0.3.4` |
| `release/release.md:6` | `0.3.4` |
| `adapters-ssot-alignment.md` 表格 kafkax 行 | `0.3.4` |
| `adapters-ssot-alignment.md:57` 摘要 | **过期**（见 F1） |

专项对齐文档与 crate 一致 → 标准 5 **PASS**；F1 不推翻结论。

### 6. 公共 API 行为测试

`src/lib.rs:80–188` `default_exports_behavior_paths`：

1. `KafkaConfigBuilder::build` 产出合法 loopback 配置并断言 `security_protocol` / `client_id`
2. `KafkaConfig::from_env` 无强制变量时回落 default
3. `ConsumerConfig::assign` + `with_start_offset` 字段行为
4. `encode_bus_id` / `parse_bus_id` 正反路径
5. `MemoryOffsetStore` commit → next-to-read `resolve_start_offset`
6. `ProduceThenCheckpointCoordinator::after_produce_result`：失败不推进；成功推进 checkpoint
7. `FileOffsetStore` 真实写盘 commit/read + 清理
8. `KafkaPool::connect` 对 `127.0.0.1:1` 短超时拒绝，断言 `ErrorKind` 集合

配套：`config.rs` 远程明文 / SCRAM 拒绝 / CA 需 TLS / builder 行为；`message.rs` bus_id round-trip；`consumer.rs` start offset 矩阵。

**不是**仅 `assert_type` / `std::any::type_name` / 空 `let _: T = …` 表面测试。

---

## Risk

| 等级 | 项 | 说明 | 缓解 |
|------|----|------|------|
| **P2** | 摘要行 version 漂移（F1） | 多 adapter 对齐摘要易腐 | 改摘要或删版本枚举 |
| **P2** | timestamp 单测浅（F2） | 字段接线有、语义断言弱 | 补结构体持有断言 / conformance |
| **P2** | live / harness 本审查会话未复跑 | 证据依赖实现侧会话记录 | 合并前 CI/本地可再跑 `cargo test -p kafkax` |
| **P1**（存量边界，非本 PR 引入） | rskafka 无 group/rebalance/native EOS | 已诚实 NO-GO | 文档/门禁禁止假宣称；调用方勿当 librdkafka 替代 |
| **P0** | — | 无发现 | — |

**综合风险：P2**（可合并；follow-up 清理文档漂移即可）。

---

## 本会话执行情况

| 命令 / 动作 | 结果 |
|-------------|------|
| 阅读 `evidence/kafkax-10pass-matrix.md` | 完成 |
| 阅读 `lib.rs` / `config.rs` / `message.rs` / `consumer.rs` / `eos.rs` | 完成 |
| 阅读 `CHANGELOG.md` / `Cargo.toml` / alignment / SSOT matrix·goal·release·review | 完成 |
| `git diff --stat origin/main` | **本审查环境未执行 shell**（静态文件审阅） |
| `cargo test -p kafkax --lib` | **本审查环境未执行**（用户标可选）；以源码测试内容审查为准 |

若合并门禁需要独立复验，建议实现侧或 CI 再跑：

```bash
cargo test -p kafkax --all-targets
cargo clippy -p kafkax --all-targets -- -D warnings
```

---

## 结论

在「rskafka 可交付面 + 不可交付面显式 NO-GO/OOS」合同下，本分支 kafkax `0.3.4` 变更与 draft 十轮收敛一致，**未**夸大 package stable / group / EOS，密钥路径诚实，公共 API 测试为行为驱动。

**Verdict: Approve**

---

## Evidence 索引（file:line）

| 主题 | 路径 |
|------|------|
| 十轮矩阵 NO-GO/OOS | `.agents/ssot/adapters/storage/kafka/evidence/kafkax-10pass-matrix.md:8-51,63-65` |
| matrix S-10~S-13 | `.agents/ssot/adapters/storage/kafka/matrix/matrix.md:14-17` |
| crate 边界文档 | `crates/adapters/storage/kafka/src/lib.rs:1-33,80-188` |
| Builder + 脱敏 + 校验测试 | `crates/adapters/storage/kafka/src/config.rs:51-151,158-161,172-187,345-475` |
| timestamp 字段 | `crates/adapters/storage/kafka/src/message.rs:48-49` |
| timestamp 接线 | `crates/adapters/storage/kafka/src/consumer.rs:112-119` |
| PTC 非 EOS | `crates/adapters/storage/kafka/src/eos.rs:1-15` |
| version | `crates/adapters/storage/kafka/Cargo.toml:4,9` |
| CHANGELOG 0.3.4 | `crates/adapters/storage/kafka/CHANGELOG.md:3-17` |
| 专项对齐 | `docs/ssot/kafkax-ssot-alignment.md:9-27` |
| adapters 表行 | `docs/ssot/adapters-ssot-alignment.md:63` |
| adapters 摘要漂移 | `docs/ssot/adapters-ssot-alignment.md:57` |
| release 未 stable | `.agents/ssot/adapters/storage/kafka/release/release.md:6-10` |
| goal version/NO-GO | `.agents/ssot/adapters/storage/kafka/goal/goal.md:9-30` |
| spec 合同 | `.agents/ssot/adapters/storage/kafka/spec/spec.md:2,11-20` |
