# Residual Open — PLAN-TYPES-DECIMALX-002-agent-safe-v1

本文件登记**不得**在本战役标 DONE 的项。

## HUMAN_ONLY

| ID | 项 | 原因 | 失败条件 |
|----|-----|------|----------|
| T-HUM-001 | `MAX_SCALE` / `DecimalLimits` 取值批准 | 需 DB/provider/Display 统计与人审 | 写死 38 或未批强制 |
| T-HUM-002 | 字段私有化 | 高影响；需 consumer=0 + 兼容 | 未迁移即 `pub`→private |
| T-HUM-003 | 独立 `DecimalError` / 错误升格 | 公开签名与下游映射 | 静默改 kernel 映射语义 |
| T-HUM-004 | serde/text/DB/protocol wire stable | 独立合同；非 derive 即 stable | 文档宣称跨版本 stable |
| T-HUM-005 | SPEC Approved / GOAL Achieved | 仅人审 | AI 独断改 Status |

## DEFERRED

| ID | 项 | 前置 |
|----|-----|------|
| T-DEF-001 | 除法显式 `target_scale` | 真实 consumer use case |
| T-DEF-002 | 删除/弃用 panicking operators | inventory + 资金路径迁移 + 兼容政策 |
| T-DEF-003 | 全 i128/u8 + differential oracle | 资源与 oracle 设计 |

## POLICY

| ID | 项 |
|----|-----|
| T-POL-001 | 禁止迁 `crates/types/numeric`；禁止 `decimalx → canonical`；禁止默认 `Money<U>` |

## 明确非本战役

- 汇率、跨币种、tick/step、会计/手续费政策
- BigInt 后端替换
- 生产 Secret/Ruleset/真 publish
- 在 `main` 上直接开发
