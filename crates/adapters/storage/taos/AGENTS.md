# taosx

- 默认实现 `contracts::TimeSeriesStore` 于 REST 客户端（`TaosPool`）
- 内存实现仅 `feature = "scaffold"` 的 `TaosAdapter`
- `Tick.ts` 为纳秒；写入前按库 `precision` 换算
- 禁止将密钥写入源码或提交；使用 `FOUNDATIONX_TAOSX_*` 环境变量
- 未验证 native/TMQ 前禁止宣称 package stable
