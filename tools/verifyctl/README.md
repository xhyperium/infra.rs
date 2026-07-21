# verifyctl

最小验证 **plan / execute / report**。

## CLI

```bash
# dry：全部检查变为 `true`
VERIFYCTL_DRY=1 cargo run -p verifyctl -- plan \
  --contract contract.json \
  --changed tools/verifyctl/src/lib.rs \
  -o plan.json

cargo run -p verifyctl -- execute plan.json -o run.json
cargo run -p verifyctl -- report run.json
```

## 计划内容

默认检查：`fmt` / `clippy` / `test`；变更含 `docs/` 或 `*.md` 时追加 `docs`（`cargo doc`）。

`VERIFYCTL_DRY=1`：所有命令替换为 `true`（集成测）。

## Evidence（可选 feature）

```bash
cargo run -p verifyctl --features with-evidence -- ...
VERIFYCTL_EVIDENCE=/tmp/ev.log  # append_named 事件
```
