# storage×7 SSOT 补齐 · 10 轮审查（2026-07-22）

范围：clickhousex / kafkax / natsx / ossx / postgresx / redisx / taosx

| Round | 焦点 | 发现 | 处置 |
|------:|------|------|------|
| 1 | 11 层是否存在 | 齐全 | 保留 |
| 2 | 层内容是否占位 | goal/design/… 均为 “布局占位” | **重写**为 P0 实质内容 |
| 3 | landing/draft | #193 已有 | 保留并交叉引用 |
| 4 | docs/ssot 分 package | 仅 adapters 总览 | **新增** 7 份 `*-ssot-alignment.md` |
| 5 | live/bench 合同 | test 层未写命令 | **写入** test.md / gate.md |
| 6 | DEFER 边界 | 易与 P0 混淆 | 每域 goal/review/matrix 显式 DEFER |
| 7 | 端口/协议 | taos 6041、ch 8123、nats conf | 写入 test/landing |
| 8 | scaffold 默认风险 | 需强调非默认 | design/prompt/gate 写死 |
| 9 | 外仓字面量 | 既有树 0 | 新文避免引入 |
| 10 | 索引联动 | workspace/adapters/README 需挂链 | 同步更新 |

## Verdict

storage×7 SSOT 从「布局占位」升级为「P0 可审计合同」；package stable 仍 OPEN。
