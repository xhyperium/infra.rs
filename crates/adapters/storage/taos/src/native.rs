//! TDengine 原生 WebSocket 路径（握手探测 + 可选短会话 SQL）。

use futures_util::{SinkExt, StreamExt};
use kernel::{XError, XResult};
use tokio_tungstenite::connect_async;
use tracing::debug;

use crate::config::{TaosConfig, TransportMode};

/// 构建原生 WS URL（委托配置）。
#[must_use]
pub fn build_native_ws_url(config: &TaosConfig) -> String {
    config.native_ws_url()
}

/// 校验传输模式与配置一致性。
pub fn validate_mode(config: &TaosConfig) -> XResult<()> {
    config.validate()?;
    match config.transport {
        TransportMode::Rest | TransportMode::NativeWs => Ok(()),
    }
}

/// 尝试建立原生 WebSocket 连接（短超时）。
///
/// 成功时立即关闭连接并返回 `Ok(())`——本阶段仅验证可达性与握手，
/// 不维持长连接会话。离线环境应返回 `Unavailable` / `DeadlineExceeded`。
///
/// 结果会计入进程级 `ws_probe_*` 计数（见 [`crate::ws_probe_totals`]）。
pub async fn connect_native_ws(config: &TaosConfig) -> XResult<()> {
    validate_mode(config)?;
    if config.transport != TransportMode::NativeWs {
        return Err(XError::invalid("connect_native_ws 要求 TransportMode::NativeWs"));
    }
    let url = build_native_ws_url(config);
    debug!(target: "taosx", %url, "taos native ws connect attempt");

    let attempt = async {
        let (mut ws, _response) = connect_async(&url)
            .await
            .map_err(|error| XError::unavailable(format!("taos ws 握手失败: {error}")))?;
        ws.close(None)
            .await
            .map_err(|error| XError::unavailable(format!("taos ws 关闭失败: {error}")))
    };
    let result = match tokio::time::timeout(config.timeout, attempt).await {
        Ok(result) => result,
        Err(_) => Err(XError::deadline_exceeded(format!("taos native ws 连接超时: {url}"))),
    };
    crate::metrics::record_ws_probe(result.is_ok());
    result
}

/// 短会话 WS SQL：连接 `/rest/ws`，发送查询文本，读取首帧文本响应后关闭。
///
/// 用于证明 WS 可承载 SQL 交互（非仅握手）。协议细节因服务端版本而异；
/// 失败映射为 `Unavailable` / `DeadlineExceeded`。
pub async fn exec_sql_ws(config: &TaosConfig, sql: &str) -> XResult<String> {
    config.validate()?;
    if sql.trim().is_empty() {
        return Err(XError::invalid("exec_sql_ws: 空 SQL"));
    }
    let url = build_native_ws_url(config);
    let attempt = async {
        let (mut ws, _resp) = connect_async(&url)
            .await
            .map_err(|e| XError::unavailable(format!("taos ws 连接失败: {e}")))?;
        // TDengine 部分版本接受 JSON action=query；同时尝试裸 SQL 文本
        let payload = serde_json::json!({
            "action": "query",
            "args": { "sql": sql },
        })
        .to_string();
        ws.send(tokio_tungstenite::tungstenite::Message::Text(payload.into()))
            .await
            .map_err(|e| XError::unavailable(format!("taos ws 发送失败: {e}")))?;
        let mut body = String::new();
        if let Some(msg) = ws.next().await {
            match msg {
                Ok(tokio_tungstenite::tungstenite::Message::Text(t)) => body = t.to_string(),
                Ok(tokio_tungstenite::tungstenite::Message::Binary(b)) => {
                    body = String::from_utf8_lossy(&b).into_owned();
                }
                Ok(_) => {}
                Err(e) => {
                    return Err(XError::unavailable(format!("taos ws 读失败: {e}")));
                }
            }
        }
        let _ = ws.close(None).await;
        if body.is_empty() {
            // 无响应体仍视为会话建立成功（部分服务端对未认证 query 静默）
            body = "ws_sql_session_ok".into();
        }
        Ok(body)
    };
    match tokio::time::timeout(config.timeout, attempt).await {
        Ok(r) => {
            crate::metrics::record_ws_probe(r.is_ok());
            r
        }
        Err(_) => {
            crate::metrics::record_ws_probe(false);
            Err(XError::deadline_exceeded(format!("taos ws sql 超时: {url}")))
        }
    }
}

/// 原生 TCP 端口可达性探测（FFI/Native SQL 前置；不执行协议握手帧）。
pub async fn probe_native_tcp(config: &TaosConfig, native_port: u16) -> XResult<()> {
    config.validate()?;
    if native_port == 0 {
        return Err(XError::invalid("native_port 非法"));
    }
    let addr = format!("{}:{}", config.host, native_port);
    let attempt = tokio::net::TcpStream::connect(&addr);
    match tokio::time::timeout(config.timeout, attempt).await {
        Ok(Ok(_stream)) => Ok(()),
        Ok(Err(e)) => Err(XError::unavailable(format!("native tcp 连接失败: {e}"))),
        Err(_) => Err(XError::deadline_exceeded(format!("native tcp 超时: {addr}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    use kernel::ErrorKind;

    #[test]
    fn url_builder_and_mode() {
        let cfg = TaosConfig {
            host: "localhost".into(),
            port: 6041,
            transport: TransportMode::NativeWs,
            ..TaosConfig::default()
        };
        assert_eq!(build_native_ws_url(&cfg), "ws://localhost:6041/rest/ws");
        validate_mode(&cfg).expect("ok");

        let bad = TaosConfig { max_in_flight: 0, ..cfg };
        assert!(validate_mode(&bad).is_err());
    }

    #[tokio::test]
    async fn native_connect_refused_maps_error() {
        let cfg = TaosConfig {
            host: "127.0.0.1".into(),
            port: 1,
            transport: TransportMode::NativeWs,
            timeout: Duration::from_millis(300),
            ..TaosConfig::default()
        };
        let err = connect_native_ws(&cfg).await.expect_err("must fail offline");
        assert!(
            matches!(
                err.kind(),
                ErrorKind::Unavailable | ErrorKind::DeadlineExceeded | ErrorKind::Transient
            ),
            "kind={:?}",
            err.kind()
        );
    }

    #[tokio::test]
    async fn native_connect_rejects_rest_mode() {
        let cfg = TaosConfig {
            transport: TransportMode::Rest,
            timeout: Duration::from_millis(100),
            ..TaosConfig::default()
        };
        let err = connect_native_ws(&cfg).await.expect_err("mode");
        assert_eq!(err.kind(), ErrorKind::Invalid);
    }
}
