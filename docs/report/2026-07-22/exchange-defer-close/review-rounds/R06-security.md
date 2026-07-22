# R6 安全

- secret 不进 Debug / query string 明文（仅 signature）
- OKX passphrase 在头（协议要求）
- 无硬编码生产密钥
- residual: passphrase 头可观测 — 协议必需，记 DEFER 文档化
