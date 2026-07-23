# RELEASE-TRANSPORTX-0.1.3

状态：CANDIDATE READY；RELEASE BLOCKED（本地门禁与 Standards/Spec 独立复审通过；仍需 GitHub CI 与人工审批）

候选内容仅为 `0.1.2 → 0.1.3` 安全维护：HTTP/WS 资源上限、Debug 脱敏、SNI fail-closed、RAII 池、Retry-After。无迁移脚本；公开 API 只允许 additive/兼容变更。

明确不宣称：M3 readiness、企业 PKI/mTLS、WS 企业 TLS、完整业务 live。禁止 tag/publish/deploy；提交与 PR 仅用于进入 CI/人工审批门禁。

验证详情：`../evidence/README.md`。仓内 `cov-gate-100.mjs` 已证明 610/610 个 LCOV `DA` 行命中；region-aware 加严诊断不替代该门禁，不得把任一覆盖率数字外推为生产可靠性。
