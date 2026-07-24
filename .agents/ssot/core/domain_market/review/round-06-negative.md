# Round 6 — 负路径与错误面

**结论**: ready

## 证据
- DX-VAL 负数量 / 不平衡 / 错误价格字段 / 时间倒置 / 坏 GTD
- DM-BOOK 乱序 bid/ask、倒置 update id、缺失 ID 不假定连续
- DM-TIME 秒级时间戳拒绝、received 早于 event 拒绝
- DE-REST Unsupported Display；未连接 InvalidRequest

## 问题
- 无阻断
