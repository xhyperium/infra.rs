# Round 5 — §9 Contract Testkit 规范

> Verifier: 只读计划完备性检查（非实现验收）  
> Source Spec: `.agents/ssot/testkit/xhyper-testkit-complete-spec.md`  
> Plan pack: `plan.md` · `tasks.md` · `gap-matrix.md` · `spec-inventory.md` · `residual-open.md` · `approval-packet.md` · `.worktree/testkit-todo.md`  
> 日期: 2026-07-14

## 检查项

| 规范 | 要点 | plan | tasks | inventory / residual / gap | 判定 |
|------|------|------|-------|----------------------------|------|
| §9.1 原则·正 | 多实现对同一 contract 可观察语义一致 | plan §5；W4 | T-CTC-017「§9 原则」 | I-CTC-11 负测侧证 | PASS（弱：原则正文未入库） |
| §9.1 原则·反 | **不是**验证 mock 预设默认值 | DEF-005 相关 | 无显式 AC「禁默认值合同」 | I-CTC-8 部分 | **FAIL** |
| §9.2 一 suite 一 trait | 示例 KV / MarketDataSource | plan §5 | T-CTC-004…009 按模块 | I-CTC-1…6 | PASS |
| §9.2 禁依赖具体 adapter crate | Suite 不得依赖 binance/okx/redis… | plan 允许 deps 白名单无 adapter | T-CTC-002 白名单；**无「禁 adapter dep」AC 句** | I-DONE-24.4「无 adapter dep」；I-CTC 无专条 | **FAIL** |
| §9.3 Fake contract 验证 bullet | trait object；配置 I/O；错误注入；生命周期；调用记录；无外部 IO | T-CTC-015 仅「分层文档」 | **无 Fake 分项 I-*** | 无 | **FAIL** |
| §9.3 Sandbox contract 验证 bullet | 本地真实服务；真实协议；可重复 fixture；隔离 namespace；cleanup | 同上 | 无 | 无 | **FAIL** |
| §9.3 Real/Testnet 验证 bullet | 远端；网络；凭据；延迟；限流；API 漂移 | 同上 | 无 | 无 | **FAIL** |
| §9.3 三类不得混默认断言 | 明确 | T-CTC-015「禁止混断言」 | 有文档任务 | 无分项 checklist | PASS（弱；依赖 FAIL 分层 bullet 补齐） |
| §9.4 普通函数优先 | `assert_*_contract(factory, profile)` | plan §5 | T-CTC-004「普通函数优先」 | — | PASS |
| §9.4 薄宏仅入口 | 断言在函数；宏只生成测试 | plan §5 | T-CTC-010 | I-CTC-10 隐藏 dep | PASS |
| §9.5 禁硬编码列表 | server_time==0；stream 空；balance 空；order Pending | I-CTC-8「server_time==0 等」 | T-CTC 未逐条 | 全表未入库 | **FAIL** |
| §9.5 Profile 形状示例 | VenueTimeProfile；StreamExpectation 等 | **无** | 无类型级 AC（示例可最小实现） | 无 | PASS（弱：示例非强制类型名） |
| §9.5 最小 profile / 禁复杂 DSL | 首版只真实需要的最小 profile | **无** | 无 | 无 | **FAIL** |
| §9.6 ContractFailure 字段 | `contract` / `case` / `detail: String` | — | T-CTC-003「ContractFailure / ContractResult §9.6」 | I-CTC-9 仅名「ContractFailure」 | **FAIL** |
| §9.6 返回类型 | `Result<(), ContractFailure>`；禁大量 unwrap | T-CTC-003 | 未写禁 unwrap | 无 | **FAIL**（可与字段合并计数为 ContractResult 合同） |
| §9.7 隐藏依赖禁止 | 宏/函数外部 crate 必须在 contract-testkit Cargo.toml；禁调用方偶发提供 tokio/canonical/contracts/futures_util | plan §5；DEF-005 | T-CTC-002；T-CTC-010 | I-CTC-10 | PASS |
| §9 + §13.9 交叉 | reference fake 过 + broken fake 必须失败 | plan §5 | T-CTC-011/012 | I-CTC-11 | PASS |
| §3.2/§9 suite 集 | event_bus 可选 | T-CTC-016 DEFER | residual DEFER | I-CTC-7 | PASS（DEFER 已登记，≠ SKIP=PASS） |
| §4.2 交叉 | suite 文件与自测布局 | 见 Round2 | — | — | 本轮不重复计 FAIL |

## PASS

- contract-testkit 包路径、依赖白名单方向、按 trait 拆 suite、薄宏、reference/broken 负测、Binance/OKX 迁移、隐藏依赖禁止均有 W4 任务与 I-CTC 主干。
- event_bus 以 DEFER 登记并要求两消费者准入，符合「不得把 DEFER 当实现完成」纪律。
- 普通函数 + Profile 优先的方向在 plan §5 与 T-CTC-004/010 清晰。

## FAIL

1. **§9.1 反原则：不是验证 mock 默认值**  
   - **规范引用**: §9.1「它不是：验证某个 mock 返回预设默认值」  
   - **缺失内容**: 无 I-*；T-CTC-017 仅笼统「§9 原则」；与硬编码默认断言清除无强制联动 AC。  
   - **建议补丁位置**: `I-CTC-PRINCIPLE`；T-CTC-017/012 AC；负测「仅返回默认空/0/Pending 的 fake 不得使 suite 无 profile 地全绿」。

2. **§9.2 Suite 禁止依赖具体 adapter crate**  
   - **规范引用**: §9.2「Suite 不得依赖具体 adapter crate」  
   - **缺失内容**: T-CTC-002 只列允许依赖，未显式「禁止 adapters/*」；I-CTC 无专条（仅 I-DONE-24.4 总勾）。机器验收任务缺失。  
   - **建议补丁位置**: `I-CTC-NO-ADAPTER`；T-CTC-002 AC + `cargo tree -p contract-testkit` / archgate；T-GATE 可增规则。

3. **§9.3 Fake / Sandbox / Real 三层验证 bullet 矩阵**  
   - **规范引用**: §9.3 三小节各自 5–6 条验证点 + 「三类不得用同一组默认断言混为一谈」  
   - **缺失内容**: 仅 T-CTC-015 高层「fake/sandbox/real 分层文档；禁止混断言」；inventory **零**分项 ID。实现者无法按 bullet 勾选 suite 覆盖范围。  
   - **建议补丁位置**: `I-CTC-FAKE-1…6`、`I-CTC-SANDBOX-1…5`、`I-CTC-REAL-1…6`；T-CTC-015 改为「文档 + inventory 勾选」；首批 suite 可声明仅 Fake 层并 DEFER Sandbox/Real（须 residual accepted，**不得 SKIP=PASS**）。

4. **§9.5 硬编码禁令全表 + 最小 profile / 禁复杂 DSL**  
   - **规范引用**: §9.5 禁 `server_time==0` / `stream 必须为空` / `balance 必须为空` / `order 必须 Pending`；「首个版本只实现真实需要的最小 profile，不预建复杂 DSL」  
   - **缺失内容**: I-CTC-8 过粗；无「最小 profile」「禁复杂 DSL」I-* 或 Task AC。  
   - **建议补丁位置**: 拆 `I-CTC-HARDCODE-*`；`I-CTC-PROFILE-MIN`；T-CTC-004…009 AC；code review checklist。

5. **§9.6 ContractFailure / ContractResult 字段与失败可定位性**  
   - **规范引用**: §9.6 `ContractFailure { contract, case, detail }`；`Result<(), ContractFailure>`；「不得大量 unwrap() 造成无法定位的失败」  
   - **缺失内容**: I-CTC-9 仅符号名；T-CTC-003 未列字段、未禁 unwrap 作为 AC。  
   - **建议补丁位置**: 扩展 I-CTC-9 字段表；T-CTC-003 AC；suite 自测断言 failure 含 case 名。

## 本轮结论：FAIL

## fail_count: 5

> 计数说明：§9.3 三层（Fake/Sandbox/Real）在检查项表中分列，但 findings 合并为 **1 条 FAIL（分层 bullet 矩阵）**，避免同一缺口重复计次；硬编码与最小 profile 合并为 1 条；ContractFailure 字段与 unwrap 合并为 1 条。若按检查项表「空映射行」粗计会更高——本轮采用「可独立修复的补丁单元」计 fail_count=5。
