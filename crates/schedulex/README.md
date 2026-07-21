# schedulex

L1 **任务 ID 登记表**（active schedulex SSOT：无真实定时器）。

| 项 | 值 |
|----|-----|
| package | `xhyper-schedulex` |
| lib | `schedulex` |
| path | `crates/schedulex` |
| version | `0.1.0` |
| publish | `false`（internal only） |
| deps | **std-only**（无生产依赖） |

> **不是** production timer scheduler。  
> “登记任务 ID” ≠ 定时触发 / 执行 Job。

规范镜像：[`../../.agents/ssot/infra/schedulex/spec/spec.md`](../../.agents/ssot/infra/schedulex/spec/spec.md)  
对齐说明：[`../../docs/schedulex-ssot-alignment.md`](../../docs/schedulex-ssot-alignment.md)

## 公开面

```rust
use schedulex::Scheduler;

let mut s = Scheduler::new(); // 或 Default
s.schedule("job-1");          // 重复 ID 幂等覆盖
assert!(s.cancel("job-1"));   // 返回此前是否存在
let _ids = s.list();          // 顺序未承诺
```

## 未实现（active SSOT §3）

Clock / timer / async runtime / Job·Run / Once·FixedDelay·FixedRate·cron /
misfire / 并发 lease / timeout·cancellation / shutdown / 持久化 / 分布式调度。

## 验证

```bash
cargo test -p xhyper-schedulex --all-targets
cargo clippy -p xhyper-schedulex --all-targets -- -D warnings
cargo fmt --all --check
cargo llvm-cov -p xhyper-schedulex --fail-under-lines 100 --summary-only
```
