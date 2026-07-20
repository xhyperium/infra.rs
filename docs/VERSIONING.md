# VERSIONING.md — 统一版本管理规则

> 本文件定义 `infra.rs` 项目全部可交付物的版本策略。
> SSOT 源：本文件。冲突时以此为准。

---

## 版本体系

```
项目版本 (Project)          CONSTITUTION 版本 (Governance)
├── Cargo.toml              ├── CONSTITUTION.md
├── CHANGELOG.md            └── .agents/ssot/SSOT.md
└── crates/infra-core/      └── docs/*
                             crates/AGENTS.md (独立)
```

| 版本类型   | 载体                  | 格式                        | 示例     | 更新时机       |
| ---------- | --------------------- | --------------------------- | -------- | -------------- |
| 项目版本   | `Cargo.toml`          | SemVer                      | `0.2.0`  | 每次 release   |
| 宪章版本   | `CONSTITUTION.md`     | vX.Y.Z                      | `v1.5.0` | 每次修订       |
| Crate 版本 | `crates/*/Cargo.toml` | SemVer (`workspace = true`) | `0.2.0`  | 跟随 workspace |
| 工具版本   | `Cargo.lock`          | 锁定版本                    | —        | `cargo update` |
| 文档附版   | `docs/*.md`           | 参考宪章版本                | —        | 宪章修订时同步 |

## 版本同步规则

### R-V1: SemVer 语义

- **MAJOR** (X): 破坏性变更
- **MINOR** (Y): 向后兼容的新功能
- **PATCH** (Z): 向后兼容的修复

### R-V2: 发布触发

| 事件                         | 项目版本 | 宪章版本   |
| ---------------------------- | -------- | ---------- |
| 新增 crate 或破坏性 API 变更 | MAJOR    | 不变       |
| 新增功能、新增宪章章节       | MINOR    | MINOR      |
| 修复、文档更新               | PATCH    | PATCH      |
| 宪章仅修订                   | 不变     | 视修订级别 |

### R-V3: 版本一致性检查

CI 中验证：

1. `Cargo.toml` 版本 ≤ `CHANGELOG.md` 最新版本
2. `CONSTITUTION.md` 最新版本号在项目 CHANGELOG 中可追溯
3. 所有 `workspace = true` 的 crate 版本与 workspace 一致

### R-V4: 版本记录

- `CHANGELOG.md`：项目发布版本（SemVer）
- `CONSTITUTION.md §8.3`：宪章版本历史表
- 其他文档：文件底部版本表（格式：`| vX.Y.Z | YYYY-MM-DD | 修订 |`）

---

## 当前版本快照 (2026-07-21)

| 载体                          | 版本     |
| ----------------------------- | -------- |
| `Cargo.toml` (项目)           | `0.3.0`  |
| `CHANGELOG.md` (最新)         | `0.3.0`  |
| `CONSTITUTION.md` (宪章)      | `v1.4.0` |
| `.agents/ssot/SSOT.md`        | `v1.1.0` |
| `infra-core` (crate)          | `0.1.0`  |
| `crates/AGENTS.md`            | `v1.0.0` |
| `crates/infra-core/AGENTS.md` | `v1.0.0` |

版本号已统一，下次发布按 R-V2 规则递增。

---

## 版本号自治原则

| 域                      | 自治度     | 说明                       |
| ----------------------- | ---------- | -------------------------- |
| 项目版本 (Cargo.toml)   | 完全控制   | 跟随 SemVer                |
| 宪章版本 (CONSTITUTION) | 完全控制   | 独立于项目版本             |
| Crate 版本 (workspace)  | 跟随项目   | `version.workspace = true` |
| 文档附版                | 跟随宪章   | 同宪章版本号               |
| 工具版本 (Cargo.lock)   | Cargo 管理 | 入库，CI 验证              |
