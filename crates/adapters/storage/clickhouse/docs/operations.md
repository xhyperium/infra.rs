# clickhousex 运维

## 健康检查

- **liveness**：进程存活即可
- **readiness**：调用池/客户端 `health`/`ping`（有 deadline）

## 故障

| 症状 | 处理 |
|------|------|
| connect 失败 | 检查 FOUNDATIONX_CLICKHOUSEX_* 与网络/认证 |
| DeadlineExceeded | 调高 timeout；查下游慢查询/背压 |
| Unavailable | 下游重启/鉴权；观察重连日志 |
| TLS/CA 握手失败 | 核对 SAN、信任链与 CA 文件；禁止降级远程 HTTP |

服务端错误正文可能含原始 SQL、payload 或认证细节。`clickhousex` 只读取最多
4096 字节用于解析固定数字错误码，并在对外错误中省略正文；排障应使用服务端的脱敏日志。

## 升级 / 回滚

1. 发布前跑 `cargo test -p clickhousex` 与 HTTPS conformance；真实 live 视环境运行
2. 升级：先滚动 canary 实例，观察错误率与延迟
3. 回滚：回退至上一 crate 版本；配置 schema 保持向后兼容（仅新增字段）

## 关闭

调用 `close()`：标记 closed 并关闭 Semaphore，拒绝新许可；不承诺取消或排空已取得许可的请求。
