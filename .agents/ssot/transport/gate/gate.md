# GATE-TRANSPORT-MAINT-003

状态：LOCAL VALIDATION PASS；PR/CI/HUMAN GATES PENDING

- G1 双镜像 `cmp`。
- G2 `cargo fmt --all --check`。
- G3 transportx test/clippy/doc。
- G4 binancex/okxx tests。
- G5 workspace deps、crate versions、相关 quality gate。
- G6 coverage 与 API ratchet；工具不存在时记录 NOT_RUN 并使用仓内替代。
- G7 Release 文档必须保留 M3/企业 PKI/业务 live NO-GO。

任一安全行为测试失败或 API 出现 breaking removal/change：BLOCKED。
