<!-- ssot:trace=domain_macro.design.001 -->
# domain_macro 设计决策

## ADR-DM-001：验证值对象与受控反序列化

受约束字段私有化；构造器、`TryFrom` 和反序列化共享同一验证器。公开字段不得绕过不变量。

## ADR-DM-002：来源身份与期间分离

供应商原始系列使用 `SourceSeriesId`，本仓指标使用 `IndicatorId`；`Period`、publication instant 和 vintage 互不替代。

## ADR-DM-003：十进制与显式单位

生产 wire 使用有限十进制/定点值；单位携带维度、币种、缩放、基期和变化口径。暂用 `f64` 的实现必须在边界验证有限性并记录误差。

## ADR-DM-004：聚合根维护修订与快照

只有 `RevisionChain::append` 和受控快照插入能修改历史；失败操作原子回滚。JSON 使用数组，不暴露结构体 HashMap key。

## ADR-DM-005：低基数错误观测

L0 不主动写日志；错误携带稳定 code 和脱敏字段路径，上层按 code 统计，不把来源原始文本作为指标标签。
