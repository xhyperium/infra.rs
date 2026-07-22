# adapters/storage/oss — Plan（infra.rs）

## 波次（已执行）

| 波次 | 内容 | PR |
|------|------|-----|
| 1 | 生产客户端默认路径 + from_env + unit/live ignore | #188 |
| 2 | docs usage/config/operations + pool 单测 + benches | #189 |
| 3 | bench 超时有界 + 公共 API 单测缺口 | #190 |
| 4 | live env 构建器 + SSOT landing/draft 入库 | #191 · #193 |
| 5 | HTTPS fail-closed、资源硬上界、multipart 完整性/orphan、总 deadline | infra-2d9.3.4 |

## 本文件角色

- 入口 plan；战役细节见 [infra-rs-landing.md](infra-rs-landing.md) 与 [infra-rs-draft-spec-goal.md](infra-rs-draft-spec-goal.md)
- 后续 DEFER 项不得混入已关闭 P0 波次

## 下一可选波次（OPEN）

- lifecycle / STS 临时凭证 / 流式 TB 对象与 checksum
- package stable / crates.io 发布流程
