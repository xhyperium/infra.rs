# transport — Prompt

基于 [active spec](../spec/spec.md) 维护 `0.1.4` 候选：资源上限必须在分配/聚合边界 fail-closed；URL Debug 移除 userinfo 并遮蔽 query/fragment；Retry-After 支持 delay-seconds/HTTP-date；SNI false 显式拒绝；pool Err/unwind/poison 与 RAII 许可守恒。

本地固定代码证据由跨域 `manifest.json` 绑定。下一步只做独立终审、PR CI、
人工批准与 merge，当前均为 OPEN。企业 PKI/mTLS、M3、自动重连与真实业务 live
均为 NO-GO。
