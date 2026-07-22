# RETROSPECTIVE-SCHEDULEX-003

状态：PRELIMINARY

已确认的流程问题：active spec 已承认 JobRunner，但 crate AGENTS/README 仍禁止 Job/Run，导致治理与源码双重真相；仅依赖 HashMap 行为的“deterministic”声明缺少机器语义。

本轮改进：先冻结 public interface，再以真实 Red 证明缺口；为并发父任务建立独立 worktree；把版本与共享 manifest 延后到前序 PR 合并后处理。

最终复盘须补充 reviewer/CI 逃逸缺陷、返工次数和合并清理证据。
