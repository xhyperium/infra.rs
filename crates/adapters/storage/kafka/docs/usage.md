# kafkax 用法

## 最小示例

生产默认入口：`connect` → 代表操作 → `close`。

配置通过 `FOUNDATIONX_KAFKAX_*` 环境变量注入；**禁止**把密钥写入仓库。

详见同目录 `config.md` 与 `operations.md`。

## 测试

```bash
# 单元（离线）
cargo test -p kafkax

# 可复现单节点 broker 语义
node scripts/broker-conformance.mjs

# live（需真实 Kafka + 已 export 环境变量）
cargo test -p kafkax -- --ignored --nocapture
```
