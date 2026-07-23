# 存储适配器配置规范

本文档定义 infra.rs 7 个存储适配器的环境变量配置与本地开发数据库规范。

**SSOT 配置来源**：`/home/workspace/ZoneCNH/sre/secrets/env/`（`dev.md` 开发环境 · `prod.md` 生产环境）

---

## §1 环境变量前缀

所有存储适配器通过 `configx` crate 加载，使用 `FOUNDATIONX_` 前缀：

| 适配器 | 环境变量前缀 |
|--------|--------------|
| clickhousex | `FOUNDATIONX_CLICKHOUSEX_*` |
| kafkax | `FOUNDATIONX_KAFKAX_*` |
| natsx | `FOUNDATIONX_NATS_*` |
| ossx | `FOUNDATIONX_OSSX_*`（服务名 `polarisx`） |
| postgresx | `FOUNDATIONX_POSTGRESX_*` |
| redisx | `FOUNDATIONX_REDISX_*` |
| taosx | `FOUNDATIONX_TAOSX_*` |

---

## §2 各适配器配置字段

### postgresx

| 字段 | 说明 | 本地默认 |
|------|------|----------|
| `FOUNDATIONX_POSTGRESX_HOST` | 主机 | `127.0.0.1` |
| `FOUNDATIONX_POSTGRESX_PORT` | 端口 | `5432` |
| `FOUNDATIONX_POSTGRESX_DATABASE` | 数据库名 | 按需创建 |
| `FOUNDATIONX_POSTGRESX_USER` | 用户 | 按需创建 |
| `FOUNDATIONX_POSTGRESX_PASSWORD` | 密码 | 按需设置 |
| `FOUNDATIONX_POSTGRESX_SSLMODE` | SSL 模式 | `disable`（本地） |

### taosx

| 字段 | 说明 | 本地默认 |
|------|------|----------|
| `FOUNDATIONX_TAOSX_HOST` | 主机 | `127.0.0.1` |
| `FOUNDATIONX_TAOSX_PORT` | 端口 | `6030` |
| `FOUNDATIONX_TAOSX_DATABASE` | 数据库名 | 按需创建 |
| `FOUNDATIONX_TAOSX_USER` | 用户 | `root` |
| `FOUNDATIONX_TAOSX_PASSWORD` | 密码 | `taosdata` |
| `FOUNDATIONX_TAOSX_TLS` | TLS 开关 | `false`（本地） |

### redisx

| 字段 | 说明 | 本地默认 |
|------|------|----------|
| `FOUNDATIONX_REDISX_ADDR` | 地址（host:port） | `127.0.0.1:6379` |
| `FOUNDATIONX_REDISX_USERNAME` | 用户名 | 按需设置 |
| `FOUNDATIONX_REDISX_PASSWORD` | 密码 | 按需设置 |
| `FOUNDATIONX_REDISX_DB` | 数据库编号 | `0` |
| `FOUNDATIONX_REDISX_TLS` | TLS 开关 | `false`（本地） |

### kafkax

| 字段 | 说明 | 本地默认 |
|------|------|----------|
| `FOUNDATIONX_KAFKAX_BROKERS` | Broker 列表 | `127.0.0.1:9092` |
| `FOUNDATIONX_KAFKAX_SASL_MECHANISM` | 认证机制 | `PLAIN` |
| `FOUNDATIONX_KAFKAX_SASL_USERNAME` | 用户名 | 按需设置 |
| `FOUNDATIONX_KAFKAX_SASL_PASSWORD` | 密码 | 按需设置 |
| `FOUNDATIONX_KAFKAX_TLS` | TLS 开关 | `false`（本地） |

### clickhousex

| 字段 | 说明 | 本地默认 |
|------|------|----------|
| 连接协议 | Native（9000）/ HTTP（8123） | Native |
| 默认用户 | `default` | 无密码（本地） |
| 认证 | 用户/密码 | 按需设置 |

### natsx

| 字段 | 说明 | 本地默认 |
|------|------|----------|
| 连接端口 | 客户端 4222（`nats://`），监控 8222 | `127.0.0.1:4222` |
| 认证 | 用户/密码 | 按需设置 |
| JetStream | 本地启用 | 是 |
| `max_payload` | 最大消息体 | 1 MB |
| `max_connections` | 最大连接数 | 1024 |

### ossx（阿里云 OSS）

| 字段 | 说明 |
|------|------|
| AccessKey ID | 阿里云访问密钥 |
| AccessKey Secret | 阿里云密钥 |
| Bucket | `x-go`（dev）/ `polarisx`（prod） |
| Region | `ap-northeast-1` |
| Protocol | HTTPS |

> OSS 为远端云服务，无本地模拟。dev 和 prod 指向不同的 Bucket。

---

## §3 本地数据库创建规范

### 原则

- **权限隔离**：每个适配器使用独立数据库/用户，避免跨服务权限泄露
- **密码管理**：本地开发密码存入 `sre/secrets/env/dev.md`（已 gitignore），不提交到代码仓
- **连接隔离**：本地所有服务绑定 `127.0.0.1`，不接受外网连接

### PostgreSQL 创建示例

```sql
-- 以 postgres 超级用户登录后执行
CREATE ROLE foundationx_dev WITH LOGIN PASSWORD '<dev-password>';
CREATE DATABASE foundationx_db OWNER foundationx_dev;
GRANT ALL PRIVILEGES ON DATABASE foundationx_db TO foundationx_dev;
```

### Redis 密码设置

```bash
# 在 redis.conf 或通过 redis-cli 设置
CONFIG SET requirepass "<dev-password>"
```

### TDengine 创建示例

```sql
-- 以 root 登录后执行
CREATE DATABASE foundationx_ts KEEP 365;
CREATE USER foundationx_dev PASS '<dev-password>';
GRANT ALL ON foundationx_ts TO foundationx_dev;
```

### Kafka 本地开发

本地未安装 Kafka。开发依赖可：
1. 使用 `docker-compose` 启动单节点 Kafka + Zookeeper
2. 或安装 `kafka-server` 包并配置 SASL PLAIN 认证

### NATS 本地启动

```bash
# 以配置文件启动（含 JetStream）
nats-server -c nats-local.conf --jetstream --user foundationx_dev --pass <dev-password>
```

### ClickHouse 本地启动

```bash
# 启动服务后创建数据库
clickhouse-client --query "CREATE DATABASE IF NOT EXISTS foundationx_db"
```

---

## §4 变更日志

| 日期 | 变更 |
|------|------|
| 2026-07-23 | 初始版本：7 存储适配器配置字段、本地数据库创建指南、SSOT 来源引用 |
