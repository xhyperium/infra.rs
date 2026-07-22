# taosx 用法

## 最小示例

生产默认入口：`connect` → 代表操作 → `close`。

配置通过 `FOUNDATIONX_TAOSX_*` 环境变量注入；**禁止**把密钥写入仓库。

详见同目录 `config.md` 与 `operations.md`。

## 测试

```bash
# 单元（离线）
cargo test -p taosx

# 隔离 live（固定 digest、动态 loopback；不使用 prod）
node scripts/taos-live-conformance.mjs
```
