# schema_codegen 设计入口

| 输入 | 实现 |
|---|---|
| protobuf | `src/protobuf.rs` |
| JSON Schema | `src/jsonschema.rs` |
| OpenAPI | `src/openapi.rs` |
| SQL DDL | `src/sql.rs` |

`src/main.rs` 只负责参数解析和把生成结果写到标准输出。
