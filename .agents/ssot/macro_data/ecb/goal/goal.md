<!-- ssot:trace=ecb.goal.001 -->
# ecb — 离线目标

当前 `draft`/`not_started`；来源、认证、端点和许可均为 `UNKNOWN`。

- 解析脱敏 SDMX fixture，保留维度、期间、单位、修订和缺失原因；
- 对未知维度、坏数值、重复身份和排序冲突返回稳定错误；
- 获批前不声称认证方式，不执行请求，不进入 `macrox` L0。
