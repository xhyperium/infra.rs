# goalctl 运行态目录合同

```text
Document:  RUNTIME-STATE
Version:   1.0.0
Status:    ACTIVE（D05 DECIDED）
```

## 1. 默认根

```text
${XDG_STATE_HOME:-$HOME/.local/state}/xhyper-goalctl/<repo-identity>/
```

- `<repo-identity>`：来自 `RepositoryIdentity.repository_id` 的文件系统安全编码（替换 `/` 等）。
- 覆盖：`--state-dir <path>`（绝对或相对 cwd）；测试与 CI **必须** 使用显式 `--state-dir`，避免污染开发者 home。

## 2. 允许写入（运行态，可删除）

```text
<state-dir>/
  cache/           # 可重建索引缓存
  leases/          # Phase 2+ writer lease（Phase 1 可空）
  scratch/         # 临时文件
  logs/            # 可选本地日志（不得含密钥）
```

## 3. 禁止

| 路径 | 原因 |
|------|------|
| `./target/**` | 本仓禁止写死 target；且污染构建 |
| `.cargo/target/**` | Cargo 构建产物域；非业务 state |
| `../.cargo/target/**` | 历史路径；仍禁止作业务 state |
| `.config/goal/**` | monorepo 永久禁止控制面 |
| 仓库内未批准的隐藏控制面 | 防第二 SSOT |

## 4. 可提交制品 vs 运行态

| 类型 | 位置 |
|------|------|
| 规则 / schema / specs | `docs/goal/**`、`.agents/ssot/**` |
| Evidence（可审） | `evidence/**`、模块 `evidence/` |
| 运行态 | **仅** state-dir |

## 5. doctor 义务

- 打印 resolved state-dir（`--json` 时 `state_dir` 字段）
- `config_goal_present: false`
- 若 state-dir 落在禁止前缀内 → **POLICY** 失败

## 6. 权限

Phase 1 只读 MVA：state-dir 仅可选写 cache；无 lease 也可不创建目录。
不得因「目录不可写」而跳过 policy 检查。
