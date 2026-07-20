# GOAL-TYPES-CANONICAL-002 工作台账

> **性质**：追溯矩阵，**不是** package stable。  
> **Active SSOT**：[spec/spec.md](./spec/spec.md)（S1 Approved）  
> **Alignment**：[plan/alignment-matrix-infra-2026-07-21.md](./plan/alignment-matrix-infra-2026-07-21.md)  
> **DONE ≠** package stable / 全 wire Production Ready / 10x alone。

## 0. 图例

| 标签 | 含义 |
|------|------|
| **DONE** | agent-safe 完成且有证据 |
| **HUMAN_ONLY** | 须人类 |
| **DEFERRED** | 明确后置 |
| **POLICY** | 永久约束 |

## 1. agent-safe

| ID | 内容 | 状态 |
|----|------|------|
| SAFE-06 | active API 与源码 1:1 | **DONE** |
| SAFE-08 | 全 DTO RT + variants + Money 同一 | **DONE** |
| SAFE-09 | cancel/legacy ack fixtures | **DONE** |
| SAFE-12 | test/clippy/fmt | **DONE** |
| SAFE-14 | alignment 矩阵 | **DONE** |
| SAFE-INFRA-01 | workspace 落地 decimalx+canonical | **DONE** |
| SAFE-INFRA-02 | active/residual/pipeline 对齐 | **DONE** |
| SAFE-15 | 10x fail_rounds=0 | **DEFERRED**（本 clone 无 fresh `evidence/types-canonical-002/10x/`） |
| SAFE-16 | PR APPROVE readback | **HUMAN_ONLY**（历史 xhyper PR；本仓不伪造） |

## 2. HUMAN / POLICY

| ID | 项 | 状态 |
|----|-----|------|
| HUMAN-01 | Spec S1 Approved | **DONE**（2026-07-17 人审） |
| HUMAN-02 | package stable / crates.io | **HUMAN_ONLY** |
| POLICY-01 | 禁止 Encoding Core | **POLICY** |
| POLICY-03 | 禁止金融 f32/f64；Money 唯 decimalx | **POLICY** |
| POLICY-07 | R6：本改写 intentional；源与镜像双写 | **POLICY** |

## 3. DEFERRED

| ID | 项 |
|----|-----|
| DEFER-01 | OrderRef newtype 二期（OPEN-ID-002） |
| DEFER-02 | 全 DTO golden（OPEN-WIRE-002） |
| DEFER-03 | M3 产品树迁移 |
| DEFER-04 | 移除 serde |
| DEFER-05 | types/core 布局 RFC |
| DEFER-06 | 10x 重跑（SAFE-15） |

## 4. 更新

| UTC | 事件 |
|-----|------|
| 2026-07-17 | xhyper 战役台账 |
| 2026-07-21 | infra 对齐 + R6 双写落盘；SAFE-15/16 纠正为 DEFERRED/HUMAN |
