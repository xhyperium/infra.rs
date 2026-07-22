# MATRIX-TRANSPORT-MAINT-003

| Requirement | Public seam | Test evidence | Release condition |
|---|---|---|---|
| TR-HTTP-1 chunk 累计限额 | `HttpDriver::execute` | chunked/无长度 loopback 超限 | 越界不返回 body |
| TR-WS-1 解码前限额 | `WsConnector::connect` | loopback 入站超限 | 不交付超限 payload |
| TR-DBG-1 URL 脱敏 | `Debug` | userinfo/query/非法 URL | 敏感值零泄漏 |
| TR-TLS-1 SNI | `ReqwestHttpDriver::with_tls` | `sni=false` | 明确 ProtocolViolation |
| TR-POOL-1 bounded RAII | `HttpClientPool` | 无效配置、lease drop | 许可自动回收 |
| TR-RATE-1 RFC 9110 | Retry-After parser | seconds/date/past/invalid | 确定性 duration |

M3、企业 PKI、完整业务 live 不在 PASS 矩阵，固定 NO-GO。
