# REST API 规格

> Binance REST API 端点矩阵与速率限制
> 来源: 草案 spec.md §2 + arch1.md §4
> 审查: R10 (安全) + R3 (infra.rs 映射)

## 端点矩阵

| 端点功能 | Spot | UM Futures | CM Futures | Options |
|---------|------|-----------|-----------|---------|
| 交易对信息 | GET /api/v3/exchangeInfo | GET /fapi/v1/exchangeInfo | GET /dapi/v1/exchangeInfo | GET /eapi/v1/exchangeInfo |
| 深度快照 | GET /api/v3/depth?symbol=X&limit=N | GET /fapi/v1/depth?symbol=X&limit=N | GET /dapi/v1/depth?symbol=X&limit=N | GET /eapi/v1/depth?symbol=X&limit=N |
| K 线 | GET /api/v3/klines?symbol=X&interval=1m&limit=500 | GET /fapi/v1/klines?symbol=X&interval=1m&limit=500 | GET /fapi/v1/klines?symbol=X&interval=1m&limit=500 | GET /eapi/v1/klines?symbol=X&interval=1m&limit=500 |
| 历史成交 | GET /api/v3/trades?symbol=X&limit=1000 | GET /fapi/v1/aggTrades?symbol=X&limit=1000 | GET /dapi/v1/aggTrades?symbol=X&limit=1000 | GET /eapi/v1/trades?symbol=X&limit=1000 |
| 监听密钥 | POST /api/v3/userDataStream | POST /fapi/v1/listenKey | POST /dapi/v1/listenKey | -- |

## 速率限制

| 产品线 | 权重限制 (/min) | 硬限制 (/min) | 订单限制 (/10s) |
|--------|:---:|:---:|:---:|
| Spot | 6,000 | 1,200 | 100 |
| UM Futures | 2,400 | 600 | 50 |
| CM Futures | 2,400 | 600 | 50 |
| Options | 400 | 300 | 50 |

### 端点权重

| 端点 | Spot | UM Futures | CM Futures | Options |
|------|:---:|:---:|:---:|:---:|
| exchangeInfo | 10 | 1 | 1 | 1 |
| depth (100) | 5 | 5 | 5 | 5 |
| depth (500) | 25 | 25 | 25 | 25 |
| klines | 1 | 1 | 1 | 1 |
| trades | 1 | 1 | 1 | 1 |
| listenKey | 1 | 1 | 1 | -- |

## 响应格式

- Content-Type: `application/json`
- 所有价格/数量字段以**字符串**形式返回（避免浮点精度损失）
- 时间戳: 毫秒 Unix 时间戳

## 错误码

| HTTP | 含义 | 处理 |
|------|------|------|
| 200 | 成功 | 正常解析 |
| 429 | 速率限制 | Retry-After 退避，触发限流器 |
| 418 | IP 被封 | 记录错误，等待自动恢复（2-5 分钟） |
| 4xx | 请求错误 | AdapterError::InvalidRequest |
| 5xx | 服务端错误 | 指数退避重试 |
| timeout | 网络超时 | AdapterError::Network |

## 签名端点（如添加私有功能）

- 算法: HMAC-SHA256
- 头部: `X-MBX-APIKEY: {api_key}`
- 查询参数: `...&timestamp={ts}&signature={hex}` (签名在末尾)
- 详见: `security/signing.md`
