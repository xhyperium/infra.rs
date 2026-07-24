<!-- ssot:trace=jin10.gate.001 -->
# jin10 — 门禁

当前状态为 `draft`/`not_started`，仅允许离线 fixture 和 parser 质量检查。

## 通用门禁

`cargo fmt --all --check`、`cargo build --workspace`、`cargo clippy --workspace --all-features --all-targets -- -D warnings`、`cargo test --workspace`、`cargo deny check`（工具可用时）和 `node scripts/quality-gates/check-ssot-current-state.mjs` 必须按 CI 结果记录。

## 域专项门禁

- 消息 identity、时间、单位和缺失语义在 fixture 中稳定可重放。
- 拒绝、挑战、配额和未知权限返回稳定错误，不进行访问方式改变。
- secret、完整 URL、原始响应和用户输入不得进入日志、错误或序列化。
- 只有获批 Cargo member 和 commit 绑定证据才能把状态推进到实现。

“跳过网络测试”不是成功证据；未经人工批准不得运行真实服务测试。
