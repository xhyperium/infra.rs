<!-- ssot:trace=bea.gate.001 -->
# bea — 离线门禁

- [ ] fixture 只包含脱敏观测与来源身份，不包含凭据、完整请求或原始响应秘密；
- [ ] 合法、缺失、未知、坏数值、重复身份和修订输入均返回稳定结果；
- [ ] `domain_macro` 映射保持单位、期间、频率和缺失原因，不宣称来源合同；
- [ ] `cargo fmt`、`cargo clippy`、`cargo test`、编码门禁和 SSOT 门禁均通过；
- [ ] 来源合同、访问方式、认证、配额、许可和真实测试在 `verified` 前必须保持 `UNKNOWN`。
