# adapters/storage/redis — Plan（infra.rs）

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

## 当前收敛与后续证据

| 项目 | 当前裁定 |
|------|----------|
| Pub/Sub 配置漂移 | 复用建池配置；Cluster/Sentinel 失败关闭 |
| 重试/原子性 | budget 下 ReadOnly + 无 TTL SET/MSET 幂等重试；相对 TTL SET/DEL/PEXPIRE 多试前拒绝；PUBLISH 不自动重试 |
| Cluster live | OPEN（另行受控环境验证） |
| Sentinel live/failover | OPEN（另行受控环境验证） |
| TLS/ACL live | OPEN（另行受控环境验证） |
| Streams / package stable | OPEN |
