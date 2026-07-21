//! testnet/public 只读 `server_time`（infra-s9t.13）。
//!
//! 默认 ignore；外网可用时：
//! `cargo test -p binancex --test live_server_time -- --ignored --nocapture`

use std::sync::Arc;

use binancex::BinanceAdapter;
use contracts::{VenueAdapter, VenueTimeSource};
use transportx::ReqwestHttpDriver;

#[tokio::test]
#[ignore = "requires network; public Binance /api/v3/time"]
async fn live_binance_server_time_via_real_http_driver() {
    let http = Arc::new(ReqwestHttpDriver::new().expect("driver"));
    let a = BinanceAdapter::new("binance-live", "https://api.binance.com").with_http(http);
    a.connect().await.expect("connect");
    let t = VenueAdapter::server_time(&a).await.expect("server_time");
    assert!(t > 1_600_000_000_000, "expect ms epoch, got {t}");
    let t2 = VenueTimeSource::server_time(&a).await.expect("VenueTimeSource");
    assert!(t2 > 1_600_000_000_000);
}
