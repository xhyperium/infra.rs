# R6 安全

- PASS：secret Debug 脱敏；不进 URL
- fixed 本轮：transport 脱敏 OK-ACCESS-* / passphrase（HttpRequest Debug）
- DEFER：pub 字段可被直接读（调用方责任）
