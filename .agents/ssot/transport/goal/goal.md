# GOAL-TRANSPORT-MAINT-003

状态：IN PROGRESS（2026-07-23）

在不扩大 transportx 职责的前提下，关闭六个可验证安全缺口：HTTP 流式限额、WS 解码前限额、URL Debug 脱敏、未接线 SNI fail-closed、有界池 RAII 回收、RFC 9110 Retry-After。目标版本 `0.1.3`。

验收：公共 seam 红绿证据、双镜像一致、scoped test/clippy/doc、binancex/okxx 回归通过；M3、企业 PKI、完整业务 live 继续 NO-GO。
