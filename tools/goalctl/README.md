# goalctl

最小 **Goal → Contract** 编译器（library + CLI）。

## CLI

```bash
cargo run -p goalctl -- doctor
cargo run -p goalctl -- validate path/to/goal.yaml
cargo run -p goalctl -- compile path/to/goal.yaml -o contract.json
```

## Goal 字段

| 字段 | 说明 |
|------|------|
| `id` | 必填 |
| `outcome` | 必填、非空 |
| `risk` | `R0`…`R4` |
| `acceptance[]` | 每项必有 `id` + `statement` |
| `invariants` / `forbidden` / `not_in_scope` / `touches` | 可选列表 |

## Fail-closed

- 空 `outcome`
- 缺失 / 重复 acceptance `id`
- 主观词 lint（如 better / hopefully / 更好）

## Digest

对 **不含 digest 字段** 的 canonical JSON（对象 key 排序）做 sha256 hex，写回 `digest`。

夹具见 `tests/fixtures/`。
