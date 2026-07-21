# clickhousex

- 默认实现 `contracts::AnalyticsSink` 于真实 HTTP 客户端（`ClickHousePool`）
- 内存实现仅 `feature = "scaffold"` 的 `ClickHouseAdapter`
- 禁止将密钥写入源码或提交；使用 `FOUNDATIONX_CLICKHOUSEX_*` 环境变量
- 未验证集群 / 原生协议前禁止宣称 package stable
