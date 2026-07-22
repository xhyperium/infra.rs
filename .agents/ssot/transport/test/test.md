# TEST-TRANSPORT-MAINT-003

策略：仅通过冻结的公共 seam 测可观察行为；网络边界使用本地 loopback，时间使用显式 `SystemTime`，不 mock crate 内部函数。

红灯必须来自：chunked 超限仍先聚合、tungstenite 未配置入站上限、URL 泄漏、SNI 被忽略、无 lease API/无配置校验、HTTP-date 未解析。每个红灯记录命令与退出码，绿灯复用同一测试。

最终：transportx all-targets test/clippy/doc、binancex/okxx 回归、coverage/API/依赖与版本门禁。
