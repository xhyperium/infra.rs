# transport — Design

> 状态：`0.1.4` IMPLEMENTED CANDIDATE 的已实施设计；本地固定代码证据由
> manifest 绑定，PR CI、独立终审、人工批准与 merge 均为 OPEN。

权威行为合同为 [`spec/spec.md`](../spec/spec.md)。本设计只解释三轮收敛，不扩展公开面。

## R1：安全边界

- 请求与代理 Debug 共用 URL 脱敏语义：只保留 scheme、host 与显式 port；path、query
  名和值、userinfo、fragment 全部隐藏；不能安全解析时固定输出
  `<invalid-url-redacted>`。
- 敏感 header 与 body 内容不进入 Debug，body 只显示长度。
- `TlsConfig::default()` 使用系统根并启用 SNI；reqwest 当前不能兑现 `sni=false`，
  因而构造阶段返回错误，禁止静默降级。
- 五个历史 `#[doc(hidden)] pub` 测试钩子已迁入 crate 私有单测。全仓消费者扫描为零，
  `cargo-public-api` baseline 从未收录这些隐藏项，crate 亦为 `publish = false`；本轮按
  PATCH 内部边界修复处理，并由 public API 源码门禁阻止隐藏公开项回流。

## R2：资源边界

- 请求体在 I/O 前校验；响应有 `Content-Length` 时先校验，未知长度按 chunk 累计并在
  首次越界时中止。
- WS 出站 frame 在发送前校验；同一上限下沉到 tungstenite 的 frame/message decoder，
  使单帧及碎片聚合超限都在交付前失败。
- 上限为零是显式逃生口，不是安全默认值。

## R3：协议与生命周期

- 429 映射为 `RateLimited`；`Retry-After` 同时解析 delay-seconds 与 HTTP-date，过去时间
  钳制为零，非法值为 `None`。
- `PoolConfig::validate` 与 `try_new` 拒绝无效容量；兼容 `new` 也执行同一校验并在无效
  配置时 fail-fast；新代码使用 `try_new` 与 `HttpClientLease`。
- pool 许可由单一状态维护：lease Drop 自动归还，`into_inner` 只释放许可；锁中毒从已持有
  状态恢复；factory error 或 panic unwind 由回滚守卫精确释放一次许可。

## R-DEP-001：`httpdate` 评估

`httpdate` 只用于 RFC 9110 `Retry-After` 的 HTTP-date 解析。手写解析器容易遗漏
兼容格式、时区与边界值；`chrono` / `time` 的依赖面和能力超出本用例，
因此保留专用小依赖。

当前可验证事实如下：

- `Cargo.lock` 锁定 `httpdate 1.0.3`，`cargo tree -p transportx -i httpdate` 显示仅
  `transportx` 直接使用；
- 发布 manifest 标注 `MIT OR Apache-2.0`，repository 为
  <https://github.com/pyfisch/httpdate>；
- 2026-07-23 核验上游仓库 `archived=false`、`disabled=false`、默认分支 `main`、
  最近 push 为 2024-12-22、open issues 为 4；GitHub release 列表未包含 `1.0.3`，
  crates 发布不以 GitHub release tag 为必要条件；
- 本仓 `cargo deny check` 于 2026-07-23 退出码为 0，advisories、bans、licenses、
  sources 均为 ok，但报告了与 `httpdate` 无关的 deny skip 配置警告。

维护性只裁定为“低频稳定、非 archived”，不宣称活跃维护。后续继续通过
`cargo deny` / Dependabot 复核漏洞、许可与版本状态。

## 边界与权衡

transportx 只封装客户端 HTTP/WS、TLS/代理配置与进程内池，不实现 resiliencx 策略、
业务认证、服务发现或跨进程治理。企业 PKI/mTLS、M3 与 live 矩阵继续 **NO-GO**。

实现入口见 [`crates/transport`](../../../../crates/transport/README.md)，追溯见
[`matrix/matrix.md`](../matrix/matrix.md)。
