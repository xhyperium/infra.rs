# GATE-CONTRACTS-MAINT-003

状态：LOCAL VALIDATION PASS；PR/CI/HUMAN GATES PENDING

- G1 双镜像 `cmp`；G2 fmt；G3 contracts/contract-testkit test+clippy+doc。
- G4 bootstrap、observex、resiliencx、9 adapters 等生产消费者 check/test。
- G5 API ratchet只允许 additive；workspace deps/crate versions 通过。
- G6 coverage 与相关 quality gate；缺工具如实记录。
- G7 release/alignment 保持交易业务 live、全 conformance、原子性/E2E NO-GO。

任一 trait removal/signature change、backend 实现进入 contracts 或验证假通过：BLOCKED。
