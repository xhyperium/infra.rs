# PROMPT-SCHEDULEX-003

继续条件：只在独立 worktree 修改 schedulex 域；禁止触碰父任务四域。

实现目标：保持 std-only 和现有 public interface，使 JobRunner 在非法输入、同 tick 顺序、时间回退、错误、panic、替换与取消方面具备明确且可测语义。

停止条件：出现 breaking API、Cargo/依赖变化早于前序 PR 合并、引入后台/分布式能力、并发 writer 或无法解释的门禁失败。
