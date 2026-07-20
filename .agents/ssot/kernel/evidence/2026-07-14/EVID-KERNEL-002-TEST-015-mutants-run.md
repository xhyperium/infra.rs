# EVID-KERNEL-002-TEST-015 — cargo-mutants 实测

| 字段 | 值 |
|------|-----|
| Date | 2026-07-14 |
| Tool | cargo-mutants **27.1.0** |
| Tree | clean worktree @ `origin/main` (`22057f73`) under `/tmp/kernel-mutants-src` |
| Package | `-p kernel` |
| Flags | `--timeout 90 --jobs 2` |
| Target dir | `/tmp/kernel-mutants-target` |

## 结果

```text
Found 64 mutants to test
ok       Unmutated baseline in 21s build + 1s test
TIMEOUT  lifecycle.rs wait body → ()
TIMEOUT  lifecycle.rs wait delete !
64 mutants tested in 3m: 31 caught, 31 unviable, 2 timeouts
```

| 类别 | 数量 |
|------|------|
| caught | **31** |
| unviable | 31 |
| timeout | **2**（`ShutdownSignal::wait` 语义变异导致挂起） |
| **missed** | **0** |

### 分数（诚实）

- **missed = 0** → 无存活变异
- 可执行变异击杀：`caught / (caught + missed) = 31/31 = 100%`
- timeout：视为测试检测到行为异常（死锁/无限等），**不计 missed**；若严格只计 CAUGHT，有效检出率仍为 100% 相对 missed

**判定：RES-TEST-015 CLOSED (measured · missed=0 · ≥90% 目标满足)**

## 复现

```bash
git worktree add --detach /tmp/kernel-mutants-src origin/main
cd /tmp/kernel-mutants-src
export CARGO_TARGET_DIR=/tmp/kernel-mutants-target
printf '%s\n' '[build]' 'target-dir = "/tmp/kernel-mutants-target"' > .cargo/config.toml
cargo mutants -p kernel --timeout 90 --jobs 2
```

## 产物

- `mutants/summary.txt`
- `mutants/outcomes.json`
- `mutants/{caught,timeout,unviable,missed}.txt`
