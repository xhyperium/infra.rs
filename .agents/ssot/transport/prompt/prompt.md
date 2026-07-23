# PROMPT-TRANSPORT-MAINT-003

实现时只修改 transport 域及获准版本消费者；公共 seam 为 `HttpDriver`、`WsConnector`、`HttpClientPool`。逐 seam 先红后绿，不将 loopback 证据扩大为 M3/企业 PKI/业务 live；禁止提交、推送或修改其他 worktree。
