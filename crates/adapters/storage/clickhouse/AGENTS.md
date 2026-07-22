# clickhousex

- 默认实现 `contracts::AnalyticsSink` 于真实 HTTP 客户端（`ClickHousePool`）
- 内存实现仅 `feature = "scaffold"` 的 `ClickHouseAdapter`
- 禁止将密钥写入源码或提交；使用 `FOUNDATIONX_CLICKHOUSEX_*` 环境变量
- 远程 HTTP 必须 fail-closed；HTTPS 使用 rustls roots 与可选 PEM CA
- 本地 TLS 协议实验不等于真实 ClickHouse 集群 TLS 证据
- 未验证集群 / 原生协议前禁止宣称 package stable
