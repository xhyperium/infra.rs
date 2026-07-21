# AGENTS.md — schedulex

> 仓库级规则见 [`../../AGENTS.md`](../../AGENTS.md) 与 [`../../CONSTITUTION.md`](../../CONSTITUTION.md)。  
> 权威规范：active schedulex SSOT · [`.agents/ssot/infra/schedulex/spec/spec.md`](../../.agents/ssot/infra/schedulex/spec/spec.md)

## 身份

- **L1 任务 ID 登记表**（非 production scheduler；`publish = false`）
- package：`xhyper-schedulex` · lib：`schedulex` · path：`crates/schedulex`
- 稳定公开面仅 `Scheduler`：`new` / `Default` / `schedule` / `cancel` / `list`

## 本 crate 约束

- 生产依赖：**std-only**（`[dependencies]` 必须为空）
- `default = []`；禁止 feature 泄漏
- **禁止** 在生产源码引入 timer / Clock / Job / Run / tokio / async runtime / 持久化 / shutdown
- 重复 `schedule(id)` 必须幂等覆盖；`cancel` 必须返回此前是否存在；`list` 顺序未承诺
- 验证：`cargo test -p xhyper-schedulex` · `cargo clippy -p schedulex --all-targets -- -D warnings`
- 覆盖率：`cargo llvm-cov -p schedulex --fail-under-lines 100`
- 对齐矩阵：[`../../docs/ssot/schedulex-ssot-alignment.md`](../../docs/ssot/schedulex-ssot-alignment.md)

## 与 SSOT 镜像的关系

- `.agents/ssot/infra/schedulex` 是上游镜像布局；active 规范定义当前为 **registry only**
- **本 crate 是 infra.rs 落地**；完成声明以本仓 `cargo test` / llvm-cov 为准
- 不得把 registry 冒充 production timer scheduler

## 禁止占位

不得合并无行为 public placeholder，或静默扩大到 timer/Job 面而未更新 active SSOT。
