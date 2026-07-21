# archgate

只读架构门禁工具，扫描 workspace 元数据和源码中的已批准结构规则，并以文本或 JSON 报告违规。

## 用法

```bash
cargo run -p xhyper-archgate -- --json
```

## 边界

工具不修改源码、不替代人工架构评审，也不为未批准规则自动升级阻断级别。规则与例外的维护入口见 [`docs/README.md`](docs/README.md)。

## 非职责

- 不替代 `lint-deps` / `crate-standard` 的依赖与包结构门禁。
- 不修改业务 crate 源码。
- 不作为生产运行时依赖。

## 限制与安全

- 只读检查；失败以非零退出表达。
- 规则变更须先更新架构标准/ADR，再改本工具。

