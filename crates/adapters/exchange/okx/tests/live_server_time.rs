//! public 只读 `server_time`（infra-s9t.13）。
//!
//! `cargo test -p okxx --test live_server_time -- --ignored --nocapture`

use std::sync::Arc;

use contracts::{VenueAdapter, VenueTimeSource};
use okxx::OkxAdapter;
use transportx::ReqwestHttpDriver;

#[tokio::test]
#[ignore = "requires network; public OKX /api/v5/public/time"]
async fn live_okx_server_time_via_real_http_driver() {
    let http = Arc::new(ReqwestHttpDriver::new().expect("driver"));
    let a = OkxAdapter::new("okx-live", "https://www.okx.com").with_http(http);
    a.connect().await.expect("connect");
    let t = VenueAdapter::server_time(&a).await.expect("server_time");
    assert!(t > 1_600_000_000_000, "expect ms epoch, got {t}");
    let t2 = VenueTimeSource::server_time(&a).await.expect("VenueTimeSource");
    assert!(t2 > 1_600_000_000_000);
}
