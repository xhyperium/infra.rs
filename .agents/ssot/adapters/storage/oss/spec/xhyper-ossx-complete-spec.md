# ossx 实现规范

> 状态：当前 `0.3.3` 实现合同。默认路径为 reqwest + OSS V1 真实客户端；live 测试仍
> `#[ignore]`，**未宣称 package stable**。

## 1. 范围与证据边界

- `OssClient` 实现 `contracts::ObjectStore`，并提供 delete 与 multipart 扩展。
- `OssAdapter` 仅在 `scaffold` feature 下提供进程内测试替身。
- 代码和离线测试存在不等于目标云生产证据；live 未运行时不得宣称 Aliyun OSS 全面就绪。
- lifecycle、STS 临时凭证与 package stable 证据保持 **OPEN**。

## 2. 配置与传输安全

- 必填：`FOUNDATIONX_OSSX_{ENDPOINT,BUCKET,ACCESS_KEY_ID,ACCESS_KEY_SECRET}`；region 可选。
- 远程 endpoint 仅允许 HTTPS；HTTP 只允许 loopback 开发端点。
- endpoint 禁止 userinfo、path、query、fragment；bucket 仅允许小写字母、数字和连字符。
- `Debug` 对 AccessKeyId 局部脱敏、对 AccessKeySecret 完全脱敏。
- timeout、deadline、并发和字节限制可通过 builder 或 `FOUNDATIONX_OSSX_*` 资源变量配置；
  零值、超硬上界和非法 Unicode/数字均 fail-closed。

## 3. 资源硬上界

| 维度 | 默认值 | 配置硬上界 |
|------|--------|------------|
| in-flight 请求 | 64 | 1024 |
| 对象大小 | 512 MiB | 5 GiB（当前 Bytes API 还受缓冲上限约束） |
| 单次内存缓冲 | 512 MiB | 512 MiB |
| 错误响应体 | 64 KiB | 1 MiB |
| multipart part | 调用方显式设置 | 512 MiB；非末片至少 100 KiB |
| multipart part 数 | — | 10000 |
| retry attempts | 3 | 10 |
| object key | — | 1023 UTF-8 字节 |

响应体按 chunk 读取并在追加前检查上限；未知或虚假 `Content-Length` 不得绕过限制。所有网络
请求先获取 Semaphore 许可，排队受 acquire timeout 约束。

## 4. deadline 与重试

- 单请求 timeout 与含全部尝试/退避的 operation deadline 分离；deadline 到期返回
  `DeadlineExceeded`，不继续放大重试。
- `put_object_multipart` 的 initiate、全部 part 与 complete 共享同一个剩余 operation deadline，
  禁止每片重新获得完整预算。
- GET、同 key/同 body PUT、DELETE、UploadPart 与 Abort 可在有界预算内重试。
- Initiate 与 Complete 遇到响应不确定时可能产生 orphan 或“已完成但响应丢失”；因此外围只
  发起一次尝试，禁止自动重放。
- 401/403、Invalid、Missing、Cancelled 与最终 DeadlineExceeded 不自动重试。

## 5. multipart 完整性、取消与 orphan

- `part_number` 必须在 `1..=10000`；Complete parts 非空、无重复，并按序写入 XML。
- ETag 先校验长度/控制字符，再进行 XML text escaping；UploadId 有长度与字符边界，拒绝实体
  注入形式。
- 高层上传任一 part/complete 失败时必须尝试 abort。abort 也失败时返回 `Conflict`，错误上下文
  明确包含 `orphan_risk=true`，不得静默吞掉。
- `close()` 关闭 Semaphore，等待者与后续操作得到 `Cancelled`。
- 高层 multipart 在获得 UploadId 后建立 RAII guard；future 被 drop 时同步写入有界进程内 orphan
  registry。调用方可从 `multipart_orphan_audits()` 取回 key/UploadId，调用 `abort_multipart()`
  补偿；成功后记录自动移除。详细记录硬上限 1024，key 硬上限 1023 字节，溢出计数单独
  可见，Debug 不输出标识。
- 在服务端成功但客户端尚未取得 UploadId 的极端响应丢失场景只能返回 unknown risk；进程
  崩溃后的清理仍依赖服务端 lifecycle，因此 lifecycle 能力保持 OPEN。

## 6. 测试与验收

```bash
cargo test -p ossx --all-targets
cargo clippy -p ossx --all-targets -- -D warnings
cargo fmt --all --check
cmp .agents/ssot/adapters/storage/oss/spec/spec.md \
    .agents/ssot/adapters/storage/oss/spec/xhyper-ossx-complete-spec.md
```

live 仅在人工提供真凭据时运行：

```bash
cargo test -p ossx --test live_object_store -- --ignored --nocapture
```

默认 CI 不读取凭据、不访问生产环境。STS、lifecycle、流式 TB 对象、checksum 与 package stable
不属于本版本完成声明。

## 7. 三轮加固（0.3.3）对抗证据

- 401/403 经完整重试链路不重试（`tests/oss_conformance.rs`，loopback）。
- operation deadline 超时不残留请求；无界资源配置在校验阶段拒绝。
- **未宣称** package stable / 多区域 HA。
