# SSOT 同步操作手册

> 从上游 `xhyper.rs` 镜像 SSOT 到本仓 `.agents/ssot/`。  
> **实现落地状态**不由本手册 rsync 决定——见 [SSOT_SYNC_REPORT.md](./SSOT_SYNC_REPORT.md) 与各 `*-ssot-alignment.md`。  
> 同步后必须：对照 `Cargo.toml` members、重读本仓 OOS/落地裁定（**禁止**用上游覆盖冲掉 #164 archgate OOS 等）。

## 前置条件

```bash
SRC=/home/workspace/xhyper.rs/.agent/SSOT
DST=/home/workspace/infra.rs/.agents/ssot

# 上游必须存在
test -d "$SRC" || { echo "ERROR: 上游 $SRC 不存在"; exit 1; }

# 目标会自动创建，但确保在 infra.rs 根目录执行
test -f Cargo.toml || { echo "ERROR: 请在 infra.rs 仓库根目录执行"; exit 1; }
```

## 域清单

| 域 | 目标路径 | 包含模块 |
|----|---------|---------|
| kernel | `.agents/ssot/kernel/` | — |
| testkit | `.agents/ssot/testkit/` | — |
| types | `.agents/ssot/types/` | decimal, canonical |
| infra | `.agents/ssot/` | bootstrap, configx, gate, observex, resiliencx, schedulex, testkitx, transport（已展平） |
| adapters | `.agents/ssot/adapters/` | binance, okx, redis, kafka, nats, postgres, taos, oss, clickhouse |
| contracts | `.agents/ssot/contracts/` | — |
| tools | `.agents/ssot/tools/` | evidence, goalctl, xtask, verifyctl（**本仓 SSOT**，不从外仓 rsync） |

## 全量同步

```bash
SRC=/home/workspace/xhyper.rs/.agent/SSOT
DST=/home/workspace/infra.rs/.agents/ssot

for domain in kernel testkit types adapters contracts; do
  echo "→ syncing $domain..."
  rsync -a --delete "$SRC/$domain/" "$DST/$domain/"
done
# infra（8 子域，源为 $SRC/infra/，目标已展平到 $DST/）
for sub in bootstrap configx gate observex resiliencx schedulex testkitx transport; do
  echo "→ syncing infra/$sub..."
  rsync -a --delete "$SRC/infra/$sub/" "$DST/$sub/"
done
# tools：路径本地化（.agent/SSOT → .agents/ssot）；verifyctl 等本仓扩展见 tools 节
find "$DST/tools" -type f \( -name '*.md' -o -name '*.sh' -o -name '*.json' -o -name '*.yaml' \) \
  -print0 | xargs -0 sed -i 's|\.agent/SSOT|.agents/ssot|g'
echo "✓ 全量同步完成（tools 需检查本仓扩展是否需从 git 恢复）"
```

## 按域同步

### kernel

```bash
rsync -a --delete "$SRC/kernel/" "$DST/kernel/"
```

### testkit

```bash
rsync -a --delete "$SRC/testkit/" "$DST/testkit/"
```

### types

```bash
rsync -a --delete "$SRC/types/" "$DST/types/"
```

### infra（8 子域，已展平）

```bash
for sub in bootstrap configx gate observex resiliencx schedulex testkitx transport; do
  rsync -a --delete "$SRC/infra/$sub/" "$DST/$sub/"
done
```

### adapters（9 子域）

```bash
rsync -a --delete "$SRC/adapters/" "$DST/adapters/"
```

### contracts

```bash
rsync -a --delete "$SRC/contracts/" "$DST/contracts/"
```

### tools（本仓 SSOT）

`tools/` 为本仓维护的 SSOT（`evidence` / `goalctl` / `xtask` / `verifyctl`），**不要**从外仓路径 rsync 覆盖。

维护与验收见 [tools-ssot-alignment.md](./tools-ssot-alignment.md)。

## 安全措施

```bash
# 同步前 dry-run 预览差异
rsync -an --delete "$SRC/kernel/" "$DST/kernel/" | head -20

# 同步前备份当前状态
cp -r "$DST" "$DST.bak.$(date +%Y%m%d-%H%M%S)"
```

## 验证

```bash
# 文件数对比（一键）
for domain in kernel testkit types adapters contracts; do
  src_n=$(find "$SRC/$domain" -type f 2>/dev/null | wc -l)
  dst_n=$(find "$DST/$domain" -type f 2>/dev/null | wc -l)
  if [ "$src_n" -eq "$dst_n" ]; then
    printf "  ✓ %-12s %4d files\n" "$domain" "$dst_n"
  else
    # tools 允许本仓扩展导致 dst > src
    printf "  ~ %-12s src=%d dst=%d\n" "$domain" "$src_n" "$dst_n"
  fi
done
# infra（8 子域，已展平；源在 $SRC/infra/，目标在 $DST/）
for sub in bootstrap configx gate observex resiliencx schedulex testkitx transport; do
  src_n=$(find "$SRC/infra/$sub" -type f 2>/dev/null | wc -l)
  dst_n=$(find "$DST/$sub" -type f 2>/dev/null | wc -l)
  if [ "$src_n" -eq "$dst_n" ]; then
    printf "  ✓ %-12s %4d files\n" "infra/$sub" "$dst_n"
  else
    printf "  ~ %-12s src=%d dst=%d\n" "infra/$sub" "$src_n" "$dst_n"
  fi
done

# 内容逐字节 diff（仅在文件数一致时有效）
diff -rq "$SRC/kernel/" "$DST/kernel/" 2>&1 | grep -v "Only in" || echo "  ✓ kernel 内容一致"
```

## 同步后步骤

1. **验证** — 运行上述验证命令，确认 0 diff
2. **检查** — 对照 [workspace-ssot-alignment.md](workspace-ssot-alignment.md) 确认新域是否需要 crate 落地
3. **提交** — 有变更时创建 PR：

```bash
git add .agents/ssot/
git commit -m "chore(ssot): sync from upstream ($(date +%Y-%m-%d))"
# 推送参考 CONTRIBUTING 流程（不可直推 main）
```

## 回滚

```bash
# 如备份目录存在，恢复
cp -r "$DST.bak."*/* "$DST/"
```

## 相关文档

- [SSOT_SYNC_REPORT.md](SSOT_SYNC_REPORT.md) — 最新同步完整性报告
- [workspace-ssot-alignment.md](workspace-ssot-alignment.md) — 各域落地状态总览
- `docs/ssot/*-ssot-alignment.md` — 各域详细对齐矩阵
