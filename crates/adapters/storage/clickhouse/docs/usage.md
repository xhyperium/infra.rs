# clickhousex 用法

## 最小示例

生产默认入口：`connect` → 代表操作 → `close`。

配置通过 `FOUNDATIONX_CLICKHOUSEX_*` 环境变量注入；**禁止**把密钥写入仓库。

详见同目录 `config.md` 与 `operations.md`。

## 测试

```bash
# 单元（离线）
cargo test -p clickhousex

# live（需真实 ClickHouse + 已 export 环境变量）
cargo test -p clickhousex -- --ignored --nocapture
```
