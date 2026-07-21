# SSOT 同步操作手册

> 从上游 `xhyper.rs` 镜像 SSOT 到本仓 `.agents/ssot/`。
> 来源：`/home/workspace/xhyper.rs/.agent/SSOT/`

## 前置条件

```bash
test -d /home/workspace/xhyper.rs/.agent/SSOT || { echo "上游 SSOT 目录不存在"; exit 1; }
```

## 全量同步（一键）

```bash
SRC=/home/workspace/xhyper.rs/.agent/SSOT
DST=/home/workspace/infra.rs/.agents/ssot

rsync -a --delete "$SRC/kernel/"    "$DST/kernel/"
rsync -a --delete "$SRC/testkit/"   "$DST/testkit/"
rsync -a --delete "$SRC/types/"     "$DST/types/"
rsync -a --delete "$SRC/infra/"     "$DST/infra/"
rsync -a --delete "$SRC/adapters/"  "$DST/adapters/"
rsync -a --delete "$SRC/contracts/" "$DST/contracts/"
```

## 按域同步

### kernel

```bash
rsync -a --delete /home/workspace/xhyper.rs/.agent/SSOT/kernel/ \
  .agents/ssot/kernel/
```

### testkit

```bash
rsync -a --delete /home/workspace/xhyper.rs/.agent/SSOT/testkit/ \
  .agents/ssot/testkit/
```

### types (decimal + canonical)

```bash
rsync -a --delete /home/workspace/xhyper.rs/.agent/SSOT/types/ \
  .agents/ssot/types/
```

### infra (8 域)

```bash
rsync -a --delete /home/workspace/xhyper.rs/.agent/SSOT/infra/ \
  .agents/ssot/infra/
```

包含: bootstrap, configx, gate, observex, resiliencx, schedulex, testkitx, transport

### adapters (9 域)

```bash
rsync -a --delete /home/workspace/xhyper.rs/.agent/SSOT/adapters/ \
  .agents/ssot/adapters/
```

包含: binance, okx, redis, kafka, nats, postgres, taos, oss, clickhouse

### contracts

```bash
rsync -a --delete /home/workspace/xhyper.rs/.agent/SSOT/contracts/ \
  .agents/ssot/contracts/
```

## 验证

```bash
# 文件数对比
for d in kernel testkit types infra adapters contracts; do
  src=$(find /home/workspace/xhyper.rs/.agent/SSOT/$d -type f | wc -l)
  dst=$(find .agents/ssot/$d -type f | wc -l)
  printf "%-12s src=%-4d dst=%-4d %s\n" "$d" $src $dst \
    $(test $src -eq $dst && echo "✓" || echo "✗ MISMATCH")
done

# 内容 diff
for d in kernel testkit types infra adapters contracts; do
  diff -rq /home/workspace/xhyper.rs/.agent/SSOT/$d .agents/ssot/$d \
    2>/dev/null || echo "  $d: DIFFERENCES FOUND"
done
```

## 注意事项

- **删除感知**：`--delete` 确保上游删除的文件也在本地删除
- **保留层级**：使用 `$SRC/<domain>/` → `$DST/<domain>/`，保持目录深度一致
- **镜像 ≠ 实现**：同步成功后需检查本仓 `crates/` 是否已落地对应实现
- **提交**：同步后有差异须创建 PR，附验证输出

## 相关文档

- [SSOT_SYNC_REPORT.md](SSOT_SYNC_REPORT.md) — 最新同步完整性报告
- [workspace-ssot-alignment.md](workspace-ssot-alignment.md) — 各域落地状态总览
- `docs/*-ssot-alignment.md` — 各域详细对齐矩阵
