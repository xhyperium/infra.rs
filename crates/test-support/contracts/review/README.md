# contract-testkit 审查入口

本目录记录 crate 级审查边界，不替代 PR 的独立审查结论。

0.1.2 审查至少确认：

- 14 个 contracts trait 均有 reference 路径；15 个 broken case 会被精确 suite 拒绝；
- `EventBus` / `PubSub` 只做操作 smoke，不虚构交付、重放、顺序或次数保证；
- AnalyticsSink / Instrumentation observed suite 的观察函数仅是 test-support seam；
- `FixtureNamespace` 确定、显式且不读取时间、随机数或环境变量；
- normal/build production graph 在 default 与 all-features 下均不含 test-support package；
- Fake、自测或环境探测通过不升级为真实 backend readiness。

验证命令见 [crate README](../README.md)。
