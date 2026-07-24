<!-- ssot:trace=yield_curve.design.001 -->
# yield_curve 设计决策

## D-YC-001：来源曲线与派生曲线分离

官方发布值保留原始来源身份；插值、拟合和外推结果必须使用新的 `derived` 身份，不能覆盖官方值。

## D-YC-002：期限是值对象

期限由数值和单位共同构成，禁止用自由文本或列名字符串参与排序和比较。

## D-YC-003：provider 独立适配

收益率曲线域不规定某个供应商的接口。provider 只有在官方契约、许可、fixture 和测试完成后才能加入 manifest 的实现路径。

## D-YC-004：市场惯例进入身份

币种、估值日、结算日、day-count、复利/支付频率、日历和报价口径必须进入 `Convention` 值对象；不同惯例不得共享 canonical tenor 或比较结果。`12M`/`1Y` 只有在同一 convention 明确等价时才可归一化。

## D-YC-005：kernel 路由

财政或市场来源只有在 provider 明确声明 `source identity → yield_curve` 映射后才能进入本域；普通宏观观测进入 `domain_macro`，行情/搜索/消息若无批准 bounded context 则留在 provider 离线边界。

## D-YC-006：运行恢复

曲线批次必须支持停止摄取、隔离坏批次、保留原始脱敏 fixture、按算法/schema 版本重放和比较恢复结果。Git 回滚不能替代数据批次回滚与重放证据。
