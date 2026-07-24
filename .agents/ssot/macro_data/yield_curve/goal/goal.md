<!-- ssot:trace=yield_curve.goal.001 -->
# yield_curve 目标

- G1：以来源身份、观测日、期限、单位、曲线类型和 vintage 唯一标识一个观测。
- G2：区分官方发布的 par yield、zero rate、discount rate 与本地插值结果。
- G3：保留原始精度、缺失原因、发布日期和修订信息。
- G4：在来源授权、fixture、测试和回滚证据齐全前保持 `draft`。

非目标：把不同算法或不同来源的 10Y 数值静默合并；用收益率曲线模型替代来源发布值。
