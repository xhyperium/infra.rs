//! TDengine 原生 WebSocket 路径（URL 构建 + 真实短超时连接尝试）。

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
