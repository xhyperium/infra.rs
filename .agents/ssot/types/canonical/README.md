# types/canonical — 本仓 SSOT 入口

| 项 | 当前事实 |
|---|---|
| 实现 | `crates/types/canonical` |
| package / lib / version | `canonical` / `canonical` / `0.1.2` |
| Active Spec | [spec/spec.md](spec/spec.md) |
| 声明边界 | L2 strict serde JSON committed subset（v1–v1.3） |

## 当前合同

- 12 个 committed DTO/枚举按 v1/v1.1/v1.2/v1.3 登记；精确查询见 `WireVersion`。
- committed 类型拒绝未知/缺失字段与未知枚举，并由 golden/N-1/拒绝样例覆盖。
- `Envelope<T>` 是运输包装；消费方必须显式验证版本，不自动路由。
- `ts` 为 Unix epoch 纳秒；无损 ns→ms 使用 `unix_millis_from_ns_exact`。
- 不宣称 canonical bytes、通用 codec、跨语言协议、package stable 或业务有效性。

## 管线入口

[design](design/design.md) · [test](test/test.md) · [gate](gate/gate.md) ·
[matrix](matrix/matrix.md) · [wire matrix](plan/wire-commitment-matrix.md) ·
[residual](plan/residual-open.md)

同目录 `spec/xhyper-canonical-complete-spec.md` 是 active spec 的机械镜像，必须逐字同构；`20260717/` 与其他 campaign 文件才是历史来源，不继承 PASS。
