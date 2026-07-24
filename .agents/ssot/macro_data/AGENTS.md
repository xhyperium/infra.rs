# `.agents/ssot/` — Agent 操作说明（macro_data.rs）

> 本目录是 **macro_data.rs 本仓域规格 SSOT**。
> 实现代码在 `crates/`；`docs/ssot/` 保存经 allowlist 管理的治理文档源，不复制域规格。

## 1. 何时读这里

| 场景 | 读什么 |
|------|--------|
| 改域规格 | 对应域 `spec/` |
| 新增数据源 | 先更新 `manifest.json` 与根 `SSOT.md` 清单，再补 13 层 |
| 判断落地状态 | 参考各域 `matrix/` + `gate/` |

## 2. 标准 13 层（域叶节点）

```text
goal/ spec/ design/ plan/ tasks/ prompt/ test/ review/ release/ retrospective/
matrix/ gate/ evidence/   + README.md
```

实际结构由 `.agents/ssot/macro_data/manifest.json` 维护：11 个域、13 个目录层、每域 14 个文件。`spec_status` 与 `implementation_status` 必须分离；计划路径不是落地证据。

- **Code 不在本树**：实现路径只作为追溯引用；实际状态以 manifest 与 Cargo workspace 为准
- 禁止在 SSOT 写 `src/`、`Cargo.toml`、`*.rs` 实现副本

## 3. 本仓域树

> `domain_macro` 领域规格已移至 `core/domain_macro/`，详见 `core/AGENTS.md`。

| 路径 | 角色 |
|------|------|
| `yield_curve/` | 来源无关的收益率曲线统一契约 |
| `manifest.json` | 机器可读的域结构与状态清单 |
| `{bea,eastmoney,ecb,fred,japan_cb,jin10,treasury,uk_cb,yahoo}/` | 宏观经济数据源适配器 |

## 4. 落地状态速查

当前已建立 11 个域、13 个标准层和可机器校验的追溯矩阵：`domain_macro` 与 `yield_curve` 是来源无关的 kernel 规格域，9 个数据源适配器仍处于 `not_started`；详细状态以 `manifest.json` 为准。

## 5. 变更规则

1. **worktree + PR** 修改本树（禁止 main 直接改）
2. 改规格后运行 `node scripts/quality-gates/check-ssot.mjs`；`docs/ssot/` 只维护治理文档源

## 6. 验证

```bash
node scripts/quality-gates/check-ssot.test.mjs
node scripts/quality-gates/check-ssot.mjs
```
