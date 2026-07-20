# Round 4 — §8 宏退役合同

> Verifier: 只读计划完备性检查（非实现验收）  
> Source Spec: `.agent/SSOT/testkit/xhyper-testkit-complete-spec.md`  
> Plan pack: `plan.md` · `tasks.md` · `gap-matrix.md` · `spec-inventory.md` · `residual-open.md` · `approval-packet.md` · `.worktree/testkit-todo.md`  
> 日期: 2026-07-14

## 检查项

| 规范 | 要点 | plan | tasks | inventory / residual / gap | 判定 |
|------|------|------|-------|----------------------------|------|
| §8.1 状态 | Deprecated → Removed | DEF-004；W3 | T-DEL-001 | I-DEL-1 | PASS |
| §8.1 迁移示例 | `xlib_test!` → `#[test]` / `#[tokio::test]` | plan/gap 有删因 | 无迁移文档任务专条 | — | PASS（弱；删除门槛更关键） |
| §8.1 删除门槛·workspace=0 | 调用点清零 | plan §0.6 扫描 | T-DEL-001「workspace 调用点=0」 | consumers 表 | PASS |
| §8.1 删除门槛·external=0 | external downstream 调用点=0 | plan 仅 workspace 扫描叙述 | **T-DEL-001 未单列 external** | residual 消费者表无 external crate 策略 | **FAIL** |
| §8.1 删除门槛·compile fixture | 更新 compile fixture | — | T-DEL-001「compile fixture」 | — | PASS |
| §8.1 删除门槛·active spec 不再要求该名 | 旧 spec 不得仍强制 xlib_test! | plan §0.8 Superseded | T-DOC-002；T-ARCH-007 | DEF-008 | PASS |
| §8.2 状态 | Immediate retirement | DEF-004 | T-DEL-002 | I-DEL-2 | PASS |
| §8.2 不提供兼容替代宏 | 明确 | Forbidden #5 禁空 mock 回流 | T-DEL-002 仅「调用点=0」；**未写「无替代空壳宏」** | — | **FAIL** |
| §8.2 迁移方案五条 | 手写 fake；消费方 impl trait；多实现→contract-testkit；调用记录 Arc\<Mutex\<Vec\<Call\>\>\>；复杂 expectation 先证明需求 | **完全无迁移矩阵** | 无 | 无 I-DEL-MOCK-MIG-* | **FAIL** |
| §8.2 禁另一空壳宏替代 | 明确 | Forbidden #5 | 无独立 AC | — | 并入上「无替代宏」FAIL |
| §8.3 状态 | Move and split | DEF-005/006 | T-DEL-004；T-CTC-* | I-DEL-4；I-CTC | PASS |
| §8.3 迁入 | contract-testkit | plan §5；W4 | T-CTC-001… | I-1.4–1.5 | PASS |
| §8.3 拆分 suite 名 | market_data_source / instrument_catalog / account_source / venue_time_source / execution_venue（_contract） | plan §5 优先级 | T-CTC-005…009 | I-CTC-1…5 | PASS |
| §8.3 与硬编码解绑 | 拆分时去掉 mock fixture 行为硬编码 | I-CTC-8 过粗 | T-CTC AC 未逐条 | 见 Round1 FAIL-3 | **FAIL**（交叉；本轮计 1） |
| §8.4 状态 | Immediate deletion；无迁移 API | DEF-004 | T-DEL-003 | I-DEL-3 | PASS |
| §8.4 消费方具体 builder | MacroPointFixture / OrderFixture / TickFixture 等示例方向 | **无** | 无 | 无 | PASS（弱：规范称无迁移 API，示例为指导性） |
| §8.4 命名规则 | 仅当真实字段+验证存在才允许称 builder | **完全无映射** | 无 | 无 | **FAIL** |

## PASS

- 四类退役对象（xlib_test! / mock! / provider 宏 / FixtureBuilder）在 I-DEL、DEF-004/005、W3/W4、PR-4/5 均有主线任务。
- provider「迁出并按 trait 拆 suite」与 contract-testkit 包路径一致。
- workspace 调用点清零、compile fixture、旧 spec Superseded / active 唯一有任务钩子。
- 消费者冻结扫描（plan §0.6）支持删除风险评估。

## FAIL

1. **§8.1 external downstream 调用点 = 0**  
   - **规范引用**: §8.1 删除门槛「external downstream 调用点 = 0」  
   - **缺失内容**: T-DEL-001 AC 仅「workspace 调用点=0；compile fixture」；inventory/residual 无 external（crates.io / 下游私有）扫描策略或「publish=false 故 N/A」的显式裁定。  
   - **建议补丁位置**: I-DEL-1 拆门槛子项；T-DEL-001 AC 增 external 或书面 `N/A (publish=false + 无外部承诺)` 并入 residual；approval-packet 可记 A3。

2. **§8.2 禁止兼容/空壳替代宏 + 迁移方案矩阵**  
   - **规范引用**: §8.2「不提供兼容替代宏」；五条迁移方案；「禁止用另一个生成空壳的宏替代」  
   - **缺失内容**: 仅有删除调用点；无迁移指南任务（README/CHANGELOG 章节）；无 AC 禁止 `mock2!`/空 derive 宏回流；五条迁移路径未进 inventory。  
   - **建议补丁位置**: `I-DEL-2-MIG-1…5`；`T-DEL-002`/`T-DEL-006` AC；`T-ARCH-010` CHANGELOG 写清「无替代宏」；Forbidden 保持并引用 I-*。

3. **§8.3 拆分时硬编码行为清除清单**  
   - **规范引用**: §8.3 迁到 contract-testkit 并拆分；结合 §2.2/§9.5 不得保留硬编码 mock 行为  
   - **缺失内容**: T-CTC-005…009 未要求「移除 stream 空 / server_time==0 / balance 空 / Pending / …」对照表；I-CTC-8 过粗。  
   - **建议补丁位置**: 同 Round1 FAIL-3；`T-DEL-004` AC「core 宏删除前 suite 已无硬编码默认」；负测证明 profile 驱动。

4. **§8.4 builder 命名规则**  
   - **规范引用**: §8.4「只有真实字段和验证存在时才允许命名为 builder」  
   - **缺失内容**: plan/tasks/inventory 完全无此治理规则；仅删 FixtureBuilder 类型本身。  
   - **建议补丁位置**: `I-DEL-3-RULE` 或 `I-FIXTURE-NAME`；`T-DEL-006`/`T-ARCH-008` README；NAMING 类门禁可选 warning。

## 本轮结论：FAIL

## fail_count: 4
