# R2 正确性

- PASS：Binance 4xx 业务体优先 ErrorKind；OKX cancel/place sCode；OrderRef 映射
- fixed 本轮：okx place sCode 入口测；空体 503 → Unavailable
- DEFER：HMAC 官方 KAT、query percent-encode、OKX place ts=0
