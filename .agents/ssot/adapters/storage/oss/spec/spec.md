# ossx 实现规范

> 状态：当前 `0.3.1` 实现合同（Mock + aws-sdk-s3 真实驱动已落地；真测 `#[ignore]`，未达 M3）。**未宣称 package stable。**
> 权威顺序：`CONSTITUTION.md` → `docs/architecture/spec.md` → Approved ADR → 本文 → 代码。

## 1. 证据边界与范围

- **Evidence**：`crates/adapters/storage/oss/{Cargo.toml,src/lib.rs}` 提供：
  - `MockObjectStore`：内存 `HashMap` + `RwLock`；
  - `S3ObjectStore`：基于 `aws-sdk-s3` / `aws-config` 的真实驱动。
- **Inference**：生产部署仍须裁定 endpoint/凭证/重试与对象大小限制；代码存在 ≠ M3 生产证据。
- **Unknown**：分片上传、版本控制、加密、预签名 URL 策略尚未裁定。

目的：记录当前 object-store 适配器行为及生产化缺口。范围仅含 `ObjectStore` 实现。
非目标：把 ignored 真测当作 CI 已通过的生产证据。

## 2. 位置、依赖、版本

- 路径：`crates/adapters/storage/oss`（package `ossx`）；版本 `0.3.1`；无 features（真实驱动始终编译）。
- 普通依赖：`kernel`、`contracts`、`async-trait`、`bytes`、`anyhow`、`tokio`、`aws-config`、`aws-sdk-s3`。
- 当前依赖符合 R2。crate 独立版本化；每次更新必须恰为 `x.y.z → x.y.(z+1)`。

## 3. 当前公开 API 与行为

### 3.1 MockObjectStore

- `pub fn new() -> Self`；`Debug + Default`。
- `put_object` 写入或覆盖；`get_object` 克隆返回；缺失键返回 `XError::not_found(...)`。
- 数据仅在进程内，实例间不共享；无网络、持久化、鉴权、列表、删除或 TTL。

### 3.2 S3ObjectStore

- `new(bucket)`：`aws_config` 默认链路加载配置并创建 S3 client。
- `new_with_client(client, bucket)`：注入自定义 client（region / endpoint / 凭证）。
- `put_object` / `get_object`：SDK 错误统一映射为 `XError::Transient` 等。

## 4. 错误、并发、生命周期与信任边界

Mock 内部 `RwLock`；锁中毒时 `unwrap()` 会 panic。真实路径生命周期随 client/实例。
键和值未经验证；生产边界必须裁定凭据保密、路径/租户隔离、大小限制、传输加密。
重试/连接治理应委托既有基础设施，不在本 crate 重造。

**证据**：`s3_put_and_get` / `s3_get_missing_*` 均 `#[ignore]`；**不得**当作 CI 默认通过或 M3 证据。

## 5. 测试、验收与开放决策

Mock 单元测试覆盖 put/get、缺失、覆盖、键隔离和 trait object。真实测试 `#[ignore]`。命令：

```bash
cargo test -p ossx
cargo test -p ossx -- --ignored   # 需 AWS 凭证 + bucket；非 CI 默认
cargo check -p ossx --all-targets
cargo clippy -p ossx --all-targets -- -D warnings
```

验收标准：API/行为与第 3 节一致；依赖通过 R2；默认测试与 clippy 通过；生产能力不得由
Inference/Unknown 或 ignored 真测冒充。

## 6. 可追溯性

- `docs/architecture/spec.md` §2 R2、§4.3 `ObjectStore`、§4.5.1、§5、§8。
- `crates/adapters/storage/oss/{Cargo.toml,src/lib.rs,README.md}`。
