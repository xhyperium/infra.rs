<!-- ssot:trace=yahoo.gate.001 -->
# yahoo — 门禁

当前只运行 workspace 质量门禁、SSOT 结构/追溯门禁和离线 fixture 测试。

晋级要求：来源、许可、访问权限、字段语义和更新策略绑定到可复核证据；实现路径是获批 Cargo member；所有失败状态、日志脱敏和 parser invariants 有真实测试；证据绑定候选 commit 与 fixture SHA-256。

未知权限、拒绝、挑战或配额响应必须终止调用并返回稳定错误。未经人工批准的网络测试不能作为通过证据。
