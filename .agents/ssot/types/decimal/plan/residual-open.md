# decimalx 开放项

> 本文件只登记 active spec 之外或尚无足够证据关闭的 residual。已经落地的字段私有化、
> `DecimalError` / `DecimalErrorKind`、`MAX_SCALE = 18` 与 default-off panicking ops 不再列为开放项。

## HUMAN_ONLY / POLICY

| ID | 开放项 | 关闭条件 |
|---|---|---|
| DEC-RES-001 | 跨语言精确 wire / package stable 声明 | 人审批准独立协议、兼容策略、迁移计划与语言间一致性测试 |
| DEC-RES-002 | serde v1 未来破坏性演进 | 明确 schema 升级、向后兼容窗口、golden 与迁移证据 |
| DEC-RES-003 | Goal Achieved / Spec Approved / release 裁决 | 独立 reviewer 与发布流程基于完整证据裁决 |

## DEFERRED

| ID | 开放项 | 原因 / 前置 |
|---|---|---|
| DEC-RES-004 | JSON `i128` 跨语言精确承载 | JavaScript 等 consumer 的 JSON number 不能覆盖全部 i128；需裁定十进制字符串、分段整数或二进制编码 |
| DEC-RES-005 | 除法显式 `target_scale` | 当前合同固定结果 scale 为两侧最大值；等待真实 consumer use case |
| DEC-RES-006 | 移除 `new` / `rescale` / `panicking-ops` | 需 consumer inventory、deprecation 与兼容政策；当前只隔离出生产路径 |
| DEC-RES-007 | 全 i128 differential / exhaustive oracle | 需可复验 oracle 与资源预算；当前以 property + 定向边界覆盖 |
| DEC-RES-008 | BigInt 或更宽中间值 | 当前正式合同是 i128 中间值溢出返回 Err；替换后端需独立设计 |

## 非开放项

- `Display → FromStr` 覆盖全部可表示 Decimal：本轮生产合同，必须实现并验证，不能延期。
- `DecimalError → XError` 保留 source chain：本轮宪章合同，必须实现并验证，不能延期。
- `MAX_SCALE = 18`：当前已裁定生产边界，不再等待“取值批准”。
- Decimal/Currency/Money 字段私有：当前已实现，不再登记字段私有化迁移。

## 明确不在 decimalx 范围

- 汇率、跨币种运算、tick/step、会计与手续费政策；
- SQL/交易所 schema 映射；
- 生产 Secret、发布凭据和真实 crates.io publish；
- 通过 serde derive 或 schema 常量推导跨语言 stable。
