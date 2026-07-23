# 远程 TLS Require live（2026-07-23）

## 现象

prod 主机 `sslmode=require` 在仅 webpki 公共根时握手失败：服务端证书自签（CN=`X-16v-64g`）。

## 闭合方式（0.3.7）

- `FOUNDATIONX_POSTGRESX_TLS_CA_FILE`：叠加自签 PEM
- `FOUNDATIONX_POSTGRESX_TLS_SERVER_NAME=X-16v-64g`：host=IP 时 hostaddr + SNI 分离

## 结果

`cargo test -p postgresx --test live_postgres -- --ignored` → **9 passed**

密钥与 PEM 仅存在于 scratch / secrets，**未入库**。
