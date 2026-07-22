# ossx 用法

## 最小示例

生产默认入口：`connect` → 代表操作 → `close`。

配置通过 `FOUNDATIONX_OSSX_*` 环境变量注入；**禁止**把密钥写入仓库。

生产远程 endpoint 仅允许 HTTPS。对象、缓冲、错误体、in-flight、multipart part/count 与重试
预算均有硬上界；需要大于 512 MiB 的逻辑对象时，当前 Bytes API 不适合作为流式上传面。

详见同目录 `config.md` 与 `operations.md`。

## 测试

```bash
# 单元（离线）
cargo test -p ossx

# live（需真实 OSS + 已 export 环境变量）
cargo test -p ossx -- --ignored --nocapture
```
