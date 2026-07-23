# SSOT 路径裁决（2026-07-23）

## 问题

是否补齐 `.agents/ssot/adapters/storage/redisx/`？

## 裁决

**否。** Canonical 保持：

```text
.agents/ssot/adapters/storage/redis/   # SSOT 目录（storage×7 惯例）
crates/adapters/storage/redis/         # 实现路径
package name: redisx                   # Cargo package
```

## 理由

1. storage×7 目录均用产品短名（redis/kafka/…），package 用 `*x` 后缀。
2. 新增 `redisx/` 会造成双树漂移，违反「唯一 canonical」。
3. Draft 目标 crate 路径已写 `crates/adapters/storage/redis` → `redisx`。

## 禁止

- 复制 SSOT 到 `redisx/` 而不删 `redis/`
- 在文档中混用两套路径为「双 SSOT」
