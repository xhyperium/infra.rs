<!-- ssot:trace=fred.gate.001 -->
# fred — 门禁

当前只执行 workspace 质量门禁、SSOT 结构/追溯/编码门禁和离线 fixture 测试。

晋级要求：

- 来源、许可、修订语义、权限和字段合同绑定官方/合同证据及日期；
- `SecretRef` 与公共配置分离，sentinel 仅覆盖脱敏输出表面，不表示外部访问合同；
- parser 覆盖成功、缺字段、未知字段、坏数值、缺失、重复身份和修订追加；
- 拒绝、挑战和权限未知返回稳定错误；配额、重试和访问方式均为 `UNKNOWN`，不得改变访问方式；
- provider 路径获批、是 Cargo member，并与 matrix、test、evidence 和 commit 一致。

未经批准不执行真实服务测试；跳过网络测试不是通过证据。
