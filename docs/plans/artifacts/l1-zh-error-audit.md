# L1 用户可见错误中文化抽查（infra-s9t.12）

| Crate | Display 语言（抽查） | 裁定 |
|-------|----------------------|------|
| kernel | 中文/混合 | 维持 |
| configx | 英文 key 上下文 | P2 跟进或豁免 |
| evidence | 英文 enum Display | P2 |
| transportx | 英文 thiserror + 中文 PayloadTooLarge | 部分 |
| bootstrap | 中文 panic / 英文 BootstrapError | 混合 |
| resiliencx | 经 kernel XError | 依赖 kernel |

**结论（本轮）**：记录债；**不阻断** L1 Internal 签字。豁免至下一次治理冲刺。
