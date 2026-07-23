# GOAL-SCHEDULEX-003

状态：IN PROGRESS

在不扩大为后台或分布式 scheduler 的前提下，使 schedulex 当前 registry + tick runner 声明面可预测、fail-closed、可审计。

## Acceptance Criteria

1. `Scheduler` registry 与 `JobRunner` tick seam 的职责、非目标和关系唯一自洽。
2. `add` 不能被未校验 JobId 或公开构造的非法 Schedule 绕过。
3. 同 tick 执行与 `list_meta` 按 Rust `str::cmp` 的 Job ID 字典序稳定；错误顺序同执行顺序。
4. 时间回退、跳跃、`every:<ms>` interval、重复 ID、失败原子性、cancel、Job Err 与 panic 语义由 public seam 测试固定。
5. 用户可见调度错误为简体中文。
6. Spec 双镜像、crate 文档、alignment、版本、API baseline 与发布证据一致。
7. scoped/full gate、独立 reviewer、PR CI 与人工审批全部通过后才可发布。

非目标：timer daemon、async、持久化、misfire 产品、完整 cron、分布式调度、package stable。
