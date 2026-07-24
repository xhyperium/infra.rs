# transport — Plan

> 状态：三轮实现计划与本地 workspace 门禁已运行，固定代码证据由
> manifest 绑定；PR CI、独立终审、人工批准与 merge 均为 OPEN。

## 执行顺序

| 阶段 | 输入 | 产出 | 状态 |
|---|---|---|---|
| R1 安全审计 | Debug 与 TLS 构造路径 | URL/代理 fail-closed 脱敏、SNI 拒绝测试 | 已完成 |
| R2 资源设计与实现 | HTTP/WS 默认上限 | chunk 累计中止、WS decoder 前置限制 | 已完成 |
| R3 生命周期审计与实现 | 429 与 pool 许可状态机 | HTTP-date、RAII、poison/factory unwind 恢复 | 已完成 |
| 本地验收 | 当前 spec、实现与测试 | test/clippy/doc/fmt/dependency gate | 已完成 |
| 外部交付 | 候选 diff 与本地证据 | PR CI、独立终审、人工批准、merge | OPEN |

## 本地验收命令

```bash
cmp .agents/ssot/infra/transport/spec/spec.md \
  .agents/ssot/infra/transport/spec/xhyper-transportx-complete-spec.md
cargo test -p transportx --all-targets
cargo clippy -p transportx --all-targets -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc -p transportx --no-deps
cargo fmt --all --check
node scripts/quality-gates/check-workspace-deps.mjs
```

以上本地门禁已完成。复验失败时回到对应轮次修正，不得通过缩小合同或删除测试放行。

## 交付顺序

候选只能按“PR CI → 独立 reviewer → 人工批准 → merge”收敛；任一门禁仍为 OPEN 时均不得
宣称 released、package stable 或 M3。企业 PKI/mTLS 与真实业务 live 不属于本计划，
保持 **NO-GO**。

任务切片见 [`tasks/tasks.md`](../tasks/tasks.md)，门禁状态见 [`gate/gate.md`](../gate/gate.md)。
