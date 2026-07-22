# PLAN-SCHEDULEX-003

1. Round 1：冻结 baseline，审计治理冲突、公开非法状态、顺序、时间与错误面。
2. Round 2：批准 registry + explicit tick 设计，写 active 双镜像与 NO-GO。
3. Round 3：逐 public seam Red→Green；同步 crate 文档；运行 scoped gate；独立 review。
4. 等待前序 contracts PR 人工合并后 rebase，统一版本、lock、STATUS 与 contract-testkit。
5. 全仓门禁、PR CI、人工审批后才允许合并与清理。

回退点：schedulex 实现提交与后续版本/contract-testkit 提交分离；任一阶段可独立撤回。
