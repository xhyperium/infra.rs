# natsx 用法

## 最小示例

生产默认入口：`connect` → 代表操作 → `close`。

配置通过 `FOUNDATIONX_NATS_*` 环境变量注入；**禁止**把密钥写入仓库。

详见同目录 `config.md` 与 `operations.md`。

## 测试

```bash
# 单元（离线）
cargo test -p natsx

# 可复现 Core/JetStream 单节点语义
node scripts/broker-conformance.mjs

# live（需真实 NATS + 已 export 环境变量）
cargo test -p natsx -- --ignored --nocapture
```
