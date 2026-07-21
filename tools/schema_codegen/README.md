# schema_codegen

从 protobuf、JSON Schema、OpenAPI 或 SQL DDL 输入生成 Rust 源码的内部命令行工具。

## 用法

```bash
cargo run -p xhyper-schema-codegen -- protobuf --input schemas/example.proto
cargo run -p xhyper-schema-codegen -- jsonschema --input schemas/example.json
cargo run -p xhyper-schema-codegen -- openapi --input schemas/openapi.json
cargo run -p xhyper-schema-codegen -- sql --input schemas/example.sql
```

生成结果写到标准输出；工具不直接改写仓库文件。输入格式与生成器入口见 [`docs/README.md`](docs/README.md)。

## 非职责

- 不手写绕过生成的协议真相源。
- 不在生成物中嵌入密钥或环境私密。
- 不替代契约评审与兼容性测试。

## 限制与安全

- 生成代码变更须可审阅；禁止不可复现的本地脏生成合入。
- 输入 schema 须版本化。

