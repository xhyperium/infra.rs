# DESIGN-TRANSPORT-MAINT-003

状态：APPROVED FOR IMPLEMENTATION（范围仅 maintenance Goal）

1. `ReqwestHttpDriver::execute` 使用 `Response::chunk` 逐块累计并用 `saturating_add` 将溢出钳制到上界；越界即返回。
2. `connect_async_with_config` 注入 tungstenite `WebSocketConfig` 的 frame/message 上限，保留应用层防御检查。
3. URL Debug 统一走 fail-closed formatter：可解析时删除 userinfo、全部 query value 变 `***`；不可解析只输出占位，禁止凭 key 黑名单猜测敏感性。
4. `TlsConfig.sni=false` 在 client builder 前拒绝，避免虚假配置成功。
5. Pool 增加配置验证与借用 `HttpClientLease`；Drop 归还，旧手动接口兼容保留。
6. `parse_retry_after_at(value, now)` 提供确定性时间 seam，解析整数秒或 HTTP-date。

新增直接依赖 `httpdate` 的理由：RFC 9110 HTTP-date 含多种历史日期格式，手写解析容易产生时区、闰年与兼容错误；替代方案“仅支持整数秒”不满足本轮合同，“自研日期解析”因正确性风险拒绝。当前直接复用 lockfile 已存在的 `1.0.3` 版本，无新 feature，维护状态由 `cargo deny` 与 workspace 锁定门禁持续监控。

不设计 mTLS 身份、WS 自定义 TLS connector、异步池或业务 retry。
