# infra.rs draft SPEC_GOAL 入库（只读快照）

> **来源**：`.cargo/draft/ossx_SPEC_GOAL.md`（原 gitignored 草稿；现入库 SSOT）
> **入库原因**：draft 战役合同进入本仓 SSOT，供实现对照；**≠** package stable
> **本仓落地**：见 `docs/ssot/adapters-ssot-alignment.md` 与 PR #188–#191

---

# ossx 生产级开发库：GOAL / SPEC

> Draft v1.0｜`crates/adapters/storage/oss` → `ossx`｜基线 HEAD `6b5f8bb`（2026-07-21）

## 1. GOAL

将内存 `OssAdapter` 升级为面向 S3/兼容对象存储（包含阿里云 OSS 兼容策略需单独验证）的生产级异步开发库。提供显式 `OssPool`（共享 SDK client/HTTP connection pool）、流式上传下载、multipart、范围读、元数据/条件请求、并发与带宽背压、校验和、重试、凭据轮换、可观测性和故障验证。

### 边界

- 实现现有 `contracts::ObjectStore`，但它只适合有大小上限的小对象。
- 大对象必须使用 streaming API；禁止把未知大小对象全部读入内存。
- 不承诺不同 S3-compatible 服务完全一致；维护后端能力矩阵。
- 不默认提供目录语义、文件锁或原子 rename；对象 key 不是文件路径。

### 验收目标

10k 小对象并发请求有界；单个 TB 级逻辑对象可通过流式 multipart（实际测试规模按预算）而不随对象大小增长内存；取消能 abort multipart；凭据刷新不中断健康请求；24h soak 无 HTTP connection/task/FD 泄漏。

## 2. SPEC

### 2.1 当前差距

当前实现把完整 `Bytes` 放入 HashMap，无真实网络、bucket/region、流式、multipart、etag/checksum、重试、安全、生命周期和观测。

### 2.2 backend 与 feature

首选一种经过评审的后端：AWS SDK for Rust（S3 完整能力）或 Apache `object_store`（多云统一）。禁止同时把两者都暴露为稳定 API。features：`default=[runtime-tokio,tls-rustls,s3]`，可选 `aliyun-oss`、`path-style`、`kms`、`metrics`、`test-util`、`scaffold`。阿里 OSS 的签名、endpoint、multipart/checksum 必须真实验证，不能仅因 S3-compatible 就宣称支持。

### 2.3 API

```rust
pub struct OssConfig;
pub struct OssConfigBuilder;
#[derive(Clone)] pub struct OssPool;
#[derive(Clone)] pub struct OssClient;
pub struct ObjectKey;                  // 验证过、无 bucket 混入
pub struct UploadOptions;
pub struct DownloadOptions;
pub struct ObjectMeta { /* size, etag, version, checksum, content-type */ }
pub struct ByteStream;                 // Stream<Item=XResult<Bytes>>

impl OssPool {
    pub async fn connect(config: OssConfig) -> XResult<Self>;
    pub fn client(&self) -> OssClient;
    pub async fn put_stream(&self, key: &ObjectKey, body: ByteStream, opts: UploadOptions) -> XResult<ObjectMeta>;
    pub async fn get_stream(&self, key: &ObjectKey, opts: DownloadOptions) -> XResult<(ObjectMeta, ByteStream)>;
    pub async fn head(&self, key: &ObjectKey) -> XResult<ObjectMeta>;
    pub async fn delete(&self, key: &ObjectKey, condition: Option<VersionMatch>) -> XResult<()>;
    pub async fn health(&self) -> XResult<OssHealth>;
    pub fn stats(&self) -> OssPoolStats;
    pub async fn close(&self, deadline: Duration) -> XResult<()>;
}
```

配置必须包含 provider、endpoint、region、bucket、addressing style、TLS、credential provider、connect/read/operation timeout、max in-flight、HTTP max idle/connection、multipart threshold/part size/concurrency、stream buffer bytes、retry budget、checksum、SSE。

### 2.4 连接池语义

SDK client 构造昂贵且内部持有 HTTP connection pool/身份缓存；`OssPool` 必须在应用启动构造一次并通过 clone 共享。禁止每请求创建 SDK client。外围 Pool 提供 semaphore、带宽/字节预算、multipart registry、health/stats/close。它不是 checkout 物理连接的队列，`client()` clone 应是 O(1) 共享句柄。

并发限制至少有 request permits、multipart part permits、buffered bytes 三维；只限制请求数不足以防止大对象 OOM。

### 2.5 ObjectStore 合同

- `put_object` 支持小对象，超过 `max_buffered_object_size` 返回 `Invalid` 并引导 stream API。
- `get_object` 先检查 Content-Length；超过上限不 collect。
- 不存在映射 `Missing`；空对象返回空 `Bytes`，不是 Missing。
- key 不做不透明路径规范化，不删除 `..` 或重复 `/`；而是按明确 key 规则验证，防止不同调用产生意外同名。

### 2.6 multipart 与数据完整性

- 达阈值自动 multipart；part size 满足后端限制，part 数不得超上限。
- 任一 part 最终失败或调用取消必须尝试 AbortMultipartUpload；abort 失败进入 orphan 指标/补偿清单。
- 完成前核对所有 part 编号/etag；结果返回 version/etag/checksum。
- ETag 不应被通用地当作 MD5。使用服务支持的 checksum；下载可选边流边验证。
- 未知长度流使用有界缓冲；不通过先 collect 计算长度。
- 条件写支持 If-Match/If-None-Match/version；冲突返回 `Conflict`。

### 2.7 重试与 deadline

总 deadline 包含排队、凭据获取、连接、每 part、complete。GET/HEAD 和尚未发送 body 的操作可受控重试；PUT/multipart part 是否重试依据 body 可重放性与 operation id。stream body 不可重放时禁止自动从头重试。SDK 与外围重试预算合并，避免乘法重试。使用指数 full jitter，并遵守服务 retry-after。

### 2.8 错误

非法 key/config/range/size → `Invalid`；NoSuchKey/Bucket → `Missing`；precondition/version → `Conflict`；throttle/5xx/连接抖动 → `Transient`；endpoint/DNS/credential provider 不可用 → `Unavailable`；取消 → `Cancelled`；排队/传输超时 → `DeadlineExceeded`；multipart 状态破坏/校验和不一致 → `Invariant`（数据校验失败也可专门映射，需固定）；未知 → `Internal`。

### 2.9 安全

默认 TLS/hostname 校验；凭据通过标准 provider chain/短期角色，禁止静态 secret Debug。支持 SSE-S3/SSE-KMS/SSE-C（后者 secret 特别处理）；bucket policy/role 最小权限。默认阻止 public ACL。endpoint allowlist 防 SSRF；presign 为独立 API，限制 method、key prefix、有效期和 headers，并禁止日志记录签名 URL。

### 2.10 可观测性

指标：requests/latency/errors/retries/throttle、in-flight/waiters/buffer bytes、upload/download bytes、multipart active/parts/abort/orphan、checksum failure、credential refresh。标签为 provider、operation、outcome、logical bucket；禁止 key、URL、upload id、credential。Tracing 不记录 body/presigned query。

readiness 做有 deadline 的 HeadBucket/受限探针；避免写探针。liveness 仅本地。diagnostics 返回脱敏 region/endpoint 与 pool 状态。

### 2.11 测试

key/range/size/错误/脱敏/重试单元；ObjectStore 合同；真实 S3-compatible（至少 MinIO）与目标云服务 smoke；TLS/path style/versioning/SSE；multipart part 失败、complete 不确定、abort、取消、慢流、range；凭据过期/刷新、throttle/5xx/network；内存上限属性测试；并发/带宽/1KiB~多 GiB 基准；24h soak。云测试使用临时 bucket/最小权限并自动清理。

### 2.12 里程碑与 DoD

P0 shared client Pool + 小对象/stream/TLS/telemetry；P1 multipart/checksum/range/conditional；P2 credential rotation、故障/soak；P3 阿里 OSS/其他后端能力验证。

DoD：默认无 scaffold；大对象路径全程有界；取消/失败 orphan 可发现；真实后端能力矩阵、API/示例/迁移/运维手册、安全/SBOM/许可证/基准/故障门禁齐全。

## 3. 参考依据

- [infra.rs](https://github.com/xhyperium/infra.rs)
- [AWS SDK for Rust：应复用 client，client 可能维护 HTTP 连接池与身份缓存](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/best-practices.html)
- [Apache `object_store` 异步统一接口](https://docs.rs/object_store/latest/object_store/)

