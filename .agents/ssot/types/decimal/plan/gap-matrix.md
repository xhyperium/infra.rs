# Gap Matrix — PLAN-TYPES-DECIMALX-002-agent-safe-v1

证据标签：`[KNOWN]` / `[INFERRED]`；处置：`AGENT_SAFE` / `HUMAN_ONLY` / `DEFERRED` / `POLICY`。

| GAP ID | 主题 | 当前事实 | 目标/Draft | 风险 | 处置 | Task |
|--------|------|----------|------------|------|------|------|
| GAP-001 | Active vs Draft SSOT | Active=`decimalx-spec.md`；Draft 候选在 `20260717/` | 不得用 Draft 覆盖 Active | 误升 Approved | AGENT_SAFE | T-DOC-001 |
| GAP-002 | Active 候选链接 | Active 曾指向 `.agent/draft/...`（已迁） | 指向 `20260717/` | 死链/双 SSOT | AGENT_SAFE | T-DOC-003 |
| GAP-003 | Consumer inventory | 多 domain/adapter/tools 依赖 | M0 固定基线 | 迁移成本未知 | AGENT_SAFE | T-M0-001 |
| GAP-004 | 边界测试 | 35 unit + 12 proptest | 补 parse/Display/cmp/Currency/Hash 边 | 回归缺口 | AGENT_SAFE | T-M0-002 |
| GAP-005 | Panic 文档 | operators/`rescale` panic 事实存在 | `# Panics` + 生产用 checked | 误用 | AGENT_SAFE | T-M2-001 |
| GAP-006 | README 生产路径 | 未强调 checked 主路径 | 文档对齐 ADR-006 | 误导 | AGENT_SAFE | T-DOC-004 |
| GAP-007 | MAX_SCALE | 无批准上限；`new` 接受任意 u8 | DEC-LIMIT-001 | 不可表示值 | HUMAN_ONLY | T-HUM-001 |
| GAP-008 | 字段私有化 | `pub` mantissa/scale/Currency/Money | 先 fallible API 再收紧 | 破坏性 | HUMAN_ONLY | T-HUM-002 |
| GAP-009 | 错误枚举 | 全 `XError::Invalid` | DEC-ERR-001 | 签名/下游 | HUMAN_ONLY | T-HUM-003 |
| GAP-010 | 除法 target scale | 固定 max scale | DEC-DIV-001 OPEN | 假 API | DEFERRED | T-DEF-001 |
| GAP-011 | Wire stable | serde 字段 shape 未批 | DEC-WIRE-001 OPEN | 假 stable | HUMAN_ONLY | T-HUM-004 |
| GAP-012 | Panic API 删除 | consumer 仍可能用 operators | 先 inventory+迁移 | 破坏 | DEFERRED | T-DEF-002 |
| GAP-013 | Eq 全空间 | 有 unit+proptest，非全 i128 | DEC-EQ-002 | 过度宣称 | DEFERRED | T-DEF-003 |
| GAP-014 | 路径 numeric | REJECTED | 禁止回流 | 架构 | POLICY | T-POL-001 |
| GAP-015 | Goal/Spec 晋级 | Draft | 人审 | 伪 Achieved | HUMAN_ONLY | T-HUM-005 |
| GAP-016 | Float 路径 | 本 crate 无 f32/f64 金融路径；下游需持续扫 | DEC-FLOAT-001 | 回归 | AGENT_SAFE（扫描） | T-M0-003 |
| GAP-017 | 10x / approve | 战役收口 | fail_rounds=0；tip APPROVE 或 HAR | 假绿 | AGENT_SAFE* | T-VER-* |

\* T-VER-003 依赖 GitHub/token/PR tip；失败记 `HUMAN_ACTION_REQUIRED`，不伪造。
