# adapters/storage/taos — Plan（infra.rs）

## 波次（已执行）

| 波次 | 内容 | PR |
|------|------|-----|
| 1 | 生产客户端默认路径 + from_env + unit/live ignore | #188 |
| 2 | docs usage/config/operations + pool 单测 + benches | #189 |
| 3 | bench 超时有界 + 公共 API 单测缺口 | #190 |
| 4 | live env 构建器 + SSOT landing/draft 入库 | #191 · #193 |
| 5 | TLS/auth、Decimal NCHAR、资源与 close 硬上界；同步真实 REST/WS 边界 | infra-2d9.3.7 |

## 本文件角色

- 入口 plan；战役细节见 [infra-rs-landing.md](infra-rs-landing.md) 与 [infra-rs-draft-spec-goal.md](infra-rs-draft-spec-goal.md)
- 后续 DEFER 项不得混入已关闭 P0 波次

## 下一可选波次（OPEN）

- Native SQL / WS 认证长会话 / 全超表治理 / HA 集群
- 自动幂等重试与部分批次失败后的重复写裁定
- package stable / crates.io 发布流程
