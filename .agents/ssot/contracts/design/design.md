# DESIGN-CONTRACTS-MAINT-003

状态：APPROVED FOR IMPLEMENTATION（范围仅 maintenance Goal）

1. 保留全部 trait 方法与既有 helper；只新增准确命名入口或强化校验。
2. `LiveHandles::validate` 对有直接句柄的 kv/bus/tx/venue 做存在性校验；repo/account/venue_time 因类型中无对应句柄而明确 fail-closed。
3. `LiveContractProfile` 文档改为接线意图，`validate` 文档改为形状校验，不执行健康探测。
4. `bus_publish` 明示仅 producer call；准确命名入口不暗示 subscribe/ack/E2E。
5. `tx_kv_set` 明示 KV 与 TxContext 不绑定；准确命名入口描述实际顺序，不暗示原子事务。

不增加 registry、凭据、连接或具体后端类型。
