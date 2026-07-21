# 公开 API Baseline（Public API Snapshots）

> **DEFER-5**：核心 crate 公开 API 文本快照，供 semver / 意外破坏检测。

## 覆盖 crate

| 文件 | package | 角色 |
|------|---------|------|
| `kernel.txt` | `kernel` | L0 语义信任根 |
| `testkit.txt` | `testkit` | 测试时钟 |
| `decimalx.txt` | `decimalx` | 十进制数值 |
| `canonical.txt` | `canonical` | 跨层 DTO |
| `contracts.txt` | `contracts` | trait 出口（Additive Only） |

## 生成与比对

```bash
# 比对（本地 / CI）
node scripts/quality-gates/check-public-api.mjs

# 仅某包
node scripts/quality-gates/check-public-api.mjs -p kernel

# 接受当前公开面并重写 baseline（须在 PR 说明 breaking / additive）
node scripts/quality-gates/check-public-api.mjs --update
# 等价：REGEN=1 node scripts/quality-gates/check-public-api.mjs

# 允许 diff 存在（本地探查；CI 需 PR label `api-breaking`）
node scripts/quality-gates/check-public-api.mjs --allow-breaking
```

工具：[`cargo-public-api`](https://github.com/cargo-public-api/cargo-public-api)（`--simplified`）。

- **已安装工具**：与 `docs/api-baselines/*.txt` 做 live diff。  
- **未安装工具**：打印 notice，**仍要求** baseline 文件存在且非空，并可选 `cargo doc`；不比对 live API。  
- **CI**：`taiki-e/install-action` 安装工具，并以 `--require-tool` 运行。

## 更新规则

1. **Additive（新增 pub 项）**：通常 MINOR；更新 baseline 并在 PR 说明。  
2. **Breaking（删除或改签名）**：MAJOR 或显式 breaking；PR 打 label **`api-breaking`**，更新 baseline，CHANGELOG 写迁移。  
3. Baseline 必须在 **Linux x86_64** 官方矩阵上生成（见 [`../governance/support-matrix.md`](../governance/support-matrix.md)）。

## 工作流

- `.github/workflows/public-api.yml`：对上述 crate 路径变更的 PR 运行比对。  
- 带 `api-breaking` label 的 PR 允许 diff 非空（仍须提交更新后的 baseline 才能在后续 PR 保持绿）。
