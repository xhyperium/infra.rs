# types/canonical — Gate

| 字段 | 值 |
|---|---|
| 状态 | **交付门禁已定义**；package stable 仍 BLOCKED |
| 更新 | 2026-07-23 |
| 本轮证据 | 四域 focused fmt/test/doc/clippy/API/专项门禁已通过；最终全仓门禁与独立终审仍待执行 |

| # | 门禁 | 通过条件 |
|---|---|---|
| G1 | `cargo test -p canonical -p decimalx` | DTO/wire/golden/N-1/time/Envelope 全部通过 |
| G2 | `cargo check -p canonical --all-targets` | 全 targets 构建通过 |
| G3 | `cargo clippy -p canonical --all-targets -- -D warnings` | 零 warning |
| G4 | `cargo fmt -p canonical -- --check` | 格式无差异 |
| G5 | `node scripts/quality-gates/check-canonical-align.mjs` | workspace package、源码模式、fixture 存在性及脚本内 cargo 门禁通过 |
| G6 | Wire inventory | 12 个 committed 类型均有精确版本；coarse 查询保持兼容 |
| G7 | Strict JSON | committed DTO 拒绝未知/缺失字段；enum 拒绝未知 variant |
| G8 | Boundary | 无 canonical bytes / 通用 codec / 跨语言协议 / 自动版本路由声明或实现 |
| G9 | Version | `0.1.1 → 0.1.2` 已同步，待最终版本门禁 |
| G10 | Package stable | **BLOCKED / HUMAN_ONLY**；`publish = false` |

门禁通过只证明 L2 committed serde JSON DTO subset 达标，不得据此宣称整个 package stable。
