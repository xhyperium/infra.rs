# `.agents/ssot/` — Agent 操作说明

> 本目录是 **infra.rs 本仓域规格 SSOT**（R6）。  
> 实现代码在 `crates/` / `tools/`；对齐矩阵在 `docs/ssot/`。

## 1. 何时读这里

| 场景 | 读什么 |
|------|--------|
| 改 adapter / tools 规格 | 对应域 `spec/` + `plan/infra-rs-landing.md` |
| 对照 draft 战役合同 | `plan/infra-rs-draft-*.md`（#188 入库快照） |
| 判断是否可宣称 ship | **不要**只看本树 COMPLETE；读 `docs/ssot/*-ssot-alignment.md` + members |
| 新增域 | 先改本文件与根 `SSOT.md` 清单，再补 11 层 |

## 2. 标准 11 层（域叶节点）

```text
goal/ spec/ design/ plan/ tasks/ prompt/ test/ review/ release/ retrospective/
matrix/ gate/ evidence/   + README.md
```

- **Code 不在本树**：实现路径写在 README / landing 的 Code 列
- 禁止在 SSOT 写 `src/`、`Cargo.toml`、`*.rs` 实现副本

## 3. 本仓域树

| 路径 | 角色 |
|------|------|
| `kernel/` `testkit/` `types/` | L0 / test-support / types |
| `{bootstrap,configx,gate,observex,resiliencx,schedulex,testkitx,transport}/` | infra 面（gate 等可仅规格） |
| `adapters/{exchange,storage}/…` | 九 adapter 域（保留 `adapters/` 层级） |
| `contracts/` | trait 出口规格 |
| `tools/{evidence,goalctl,xtask,verifyctl}/` | 工具域（保留 `tools/` 层级） |

## 4. 落地状态速查（2026-07-22）

| 域 | 本仓状态（摘要） |
|----|------------------|
| storage×7 | 生产默认客户端 P0 + live/bench（#188–#190） |
| exchange×2 | scaffold + server_time |
| evidence / goalctl / verifyctl | members 已落地 |
| xtask / gate | 规格可有；crate **未**宣称落地 |

权威细节：`docs/ssot/workspace-ssot-alignment.md`。

## 5. 变更规则

1. **worktree + PR** 修改本树（禁止 main 直接改）
2. 改规格后同步 `docs/ssot/*-ssot-alignment.md` 若影响落地判定
3. 从外仓 rsync 时 **禁止**冲掉本仓 OOS / landing / draft 入库文件
4. 外仓名字面量（`xhyper` + `.rs`）不得进入本树

## 6. 验证

```bash
test -f .agents/ssot/SSOT.md
test -f .agents/ssot/AGENTS.md
test -f .agents/ssot/adapters/README.md
test -f .agents/ssot/tools/README.md
# 叶域 11 层
test -d .agents/ssot/adapters/storage/redis/spec
test -f .agents/ssot/tools/goalctl/plan/infra-rs-landing.md
test -f .agents/ssot/tools/verifyctl/plan/infra-rs-draft-spec.md
```
