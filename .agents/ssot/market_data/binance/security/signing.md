# 安全与签名规格

> Binance API 安全基线：密钥管理、签名、限流、TLS
> 审查来源: R10 (安全与限流审查) P0 发现

## P0 已知缺陷

| 缺陷 | 位置 | 严重度 | 审查 |
|------|------|:---:|------|
| secret_key 静默丢失 | BinanceAdapter::from_config | P0 | R10 |
| Debug 派生泄漏密钥 | BinanceConfig + BinanceAdapter | P0 | R10 |
| HMAC-SHA256 签名未实现 | 计划遗漏 | P0 | R10 |

## 密钥管理

### 存储

```rust
// ✗ 当前实现 (有安全缺陷)
pub struct BinanceConfig {
    pub api_key: Option<String>,      // Debug 泄漏
    pub secret_key: Option<String>,   // Debug 泄漏 + from_config 丢失
}

// ✓ 建议实现
use secrecy::{SecretString, ExposeSecret};

pub struct BinanceConfig {
    pub api_key: SecretString,
    pub secret_key: SecretString,
}

impl fmt::Debug for BinanceConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BinanceConfig")
            .field("api_key", &"***REDACTED***")
            .field("secret_key", &"***REDACTED***")
            .finish()
    }
}
```

### 来源

- 环境变量: `BINANCE_API_KEY`, `BINANCE_SECRET_KEY`
- 配置文件: `configx::MemoryConfigStore`（设置中标记 `secret` 字段）
- **禁止**: 硬编码、日志输出、调试打印

## HMAC-SHA256 签名

### 算法

```
签名 = hex(HMAC-SHA256(secret_key, query_string))
```

其中 `query_string` 为排序后的 URL 查询参数（不含 `?`）。

### Rust 实现

```rust
use sha2::Sha256;
use hmac::{Hmac, Mac};

type HmacSha256 = Hmac<Sha256>;

fn sign(query_string: &str, secret_key: &SecretString) -> String {
    let mut mac = HmacSha256::new_from_slice(
        secret_key.expose_secret().as_bytes()
    ).expect("HMAC key 初始化失败");
    mac.update(query_string.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}
```

### 使用

- 所有私有端点请求的 `timestamp` + `signature` 尾部参数
- `recvWindow` 默认 5000ms（可配置）
- 示例: `symbol=BTCUSDT&side=BUY&type=LIMIT&...&timestamp=1234567890000&signature=abc123...`

## 速率限制合规

| 限制类型 | 规则 | 处理 |
|---------|------|------|
| IP 权重 | 限制/分钟 | TokenBucket 实例每 ProductLine |
| 硬限 | 每分钟原始请求数 | 达到时返回 429 |
| 429 响应 | Retry-After 头部 | 退避 = max(retry_after_ms, 1s) |
| 418 响应 | IP 被封 | 记录，等待 2-5 分钟自动恢复 |

## TLS

- reqwest: 默认启用 TLS (rustls)，证书验证启用
- tokio-tungstenite: `rustls-tls-native-roots` feature
- 生产环境: 严禁 `danger_accept_invalid_certs(true)`

## 新增安全门禁

| Gate ID | 名称 | 描述 |
|---------|------|------|
| BN-SEC-001 | from_config 密钥传递 | from_config 不丢弃 secret_key |
| BN-SEC-002 | 日志不泄漏密钥 | BinanceConfig/Adapter 手动 Debug 脱敏 |
| BN-SEC-003 | HMAC-SHA256 签名 | 私有端点签名验证 |
| BN-SEC-004 | SecretString 密钥 | 使用 secrecy crate |
| BN-SEC-005 | 限流合规 | 每市场 TokenBucket + 429/418 处理 |
| BN-SEC-006 | TLS 强制 | 生产环境禁止跳过证书验证 |
| BN-SEC-007 | 时间戳防重放 | recvWindow 默认 5000ms |
| BN-SEC-008 | 配置无硬编码凭据 | 凭据仅从 env/configx 获取 |
