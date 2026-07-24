<!-- ssot:trace=japan_cb.gate.001 -->
# japan_cb — 离线门禁

- [ ] 脱敏 SDMX/CSV fixture 的合法、缺失、未知、坏数值、重复身份和乱序期间均有稳定结果；
- [ ] 解析结果保留来源系列身份、期间、单位、语言、修订和缺失原因；
- [ ] 测试只运行本地 fixture 和纯数据桩，不创建 provider 客户端或访问外部服务；
- [ ] `cargo fmt`、`cargo clippy`、`cargo test`、编码门禁和 SSOT 门禁均通过；
- [ ] 来源、统计代码、协议、认证、缓存、限流和许可在证据核验前保持 `UNKNOWN`。
