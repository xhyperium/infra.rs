# 三、架构原则

## 3.1 模块边界

当前 workspace 以 `Cargo.toml` `[workspace.members]` + `cargo metadata` 为准（**24** 个 package，无 `infra-core`）。

```text
crates/
├── kernel/                         # L0 语义信任根（clock / lifecycle）· package: kernel
├── testkit/                        # T0 测试支持（仅 dev-dep）· package: testkit
├── test-support/contracts/         # T0 Fake + suite（仅 dev-dep）· package: contract-testkit
├── contracts/                      # adapter trait 出口 · package: contracts
├── types/
│   ├── decimal/                    # 十进制 / Money · package: decimalx
│   └── canonical/                  # 跨层纯 DTO · package: canonical
├── infra/                          # L1 平台平面
│   ├── bootstrap/                  # 组合根 · package: bootstrap
│   ├── configx/                    # 本地多源配置 · package: configx
│   ├── evidence/                   # 审计证据追加 · package: evidence
│   ├── observex/                   # instrumentation · package: observex
│   ├── resiliencx/                 # 重试 / 熔断 / 限流 / 舱壁 · package: resiliencx
│   ├── schedulex/                  # 任务 ID + 宿主 tick · package: schedulex
│   └── transport/                  # HTTP/WS · package: transportx
└── adapters/
    ├── exchange/{binance,okx}/     # package: binancex / okxx（交易 NO-GO）
    └── storage/{clickhouse,kafka,nats,oss,postgres,redis,taos}/
                                    # package: *x 后缀
tools/
├── goalctl/                        # package: goalctl
└── verifyctl/                      # package: verifyctl（非生产 verifier）
```

- 每个 crate 有单一明确的职责
- 依赖方向：上层依赖下层，禁止循环引用（`canonical` → `decimalx` → `kernel`）
- L0 / types 层不得依赖外部运行时或平台特定代码
- **包名以 `Cargo.toml` `[package].name` 为准**；禁止在入口文档中把 package 写成已废弃的 `xhyper-*` 前缀名（依赖键别名见根 `README.md`）
- 规格树 `.agents/ssot/` 与实现路径对齐；**规格 COMPLETE ≠ 本仓已 ship**（见 `.agents/ssot/SSOT.md` R7）

## 3.2 接口设计

- 公共 API 必须有文档注释（`///`）
- 文档注释中的代码示例必须可编译（doc-test）
- 破坏性变更必须经过 deprecation 周期

## 3.3 类型驱动设计

- **让非法状态不可表示**：用类型系统在编译期阻止错误
- 关键领域值（价格、数量、时间戳）必须创建 newtype 并在构造时校验
- 优先使用枚举替代字符串或哨兵值
- 量化领域专项见 [.agents/rules/quant-dev-spec.md](../../.agents/rules/quant-dev-spec.md)

## 3.4 错误处理

- 使用 `thiserror` 定义明确错误类型
- 错误链（`source()`）不可断裂
- `unwrap()` / `expect()` 仅在不可恢复或已证明不可能出错的场景使用

---

← [上一章：核心价值观](./02-values.md) · [索引](./README.md) · 下一章：[四、代码标准](./04-code-standards.md) →
