# Round 9 — 公共 API 面

**结论**: ready

## 证据
- domainx 导出 ValidationError + validate_*
- domain_market 导出 BookError/TimeError + book/time helpers
- domain_exchange AdapterError::Unsupported 新增（non_exhaustive 兼容）
- clippy -D warnings workspace 通过；adapters 仍编译

## 问题
- 无
