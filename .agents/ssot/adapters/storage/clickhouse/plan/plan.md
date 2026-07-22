# adapters/storage/clickhouse — Plan（infra.rs）

## 波次（已执行）

| 波次 | 内容 | PR |
|------|------|-----|
| 1 | 生产客户端默认路径 + from_env + unit/live ignore | #188 |
| 2 | docs usage/config/operations + pool 单测 + benches | #189 |
| 3 | bench 超时有界 + 公共 API 单测缺口 | #190 |
| 4 | live env 构建器 + SSOT landing/draft 入库 | #191 · #193 |

## 本文件角色

- 入口 plan；战役细节见 [infra-rs-landing.md](infra-rs-landing.md) 与 [infra-rs-draft-spec-goal.md](infra-rs-draft-spec-goal.md)
- 后续 DEFER 项不得混入已关闭 P0 波次

## 下一可选波次（OPEN）

- native 9000 protocol / cluster / ReplicatedMergeTree 运维面
- package stable / crates.io 发布流程
