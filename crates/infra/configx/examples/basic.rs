//! 最小消费者路径：set → get，打印真实值。
//!
//! ```bash
//! cargo run -p configx --example basic
//! ```

use configx::ConfigStore;

fn main() {
    let store = ConfigStore::new();
    store.set("service.port", "8080").expect("healthy store set");

    let port = store.get("service.port").expect("key just set must be present");
    assert_eq!(port, "8080");
    println!("service.port={port}");

    // 缺失 key
    assert!(store.get("missing").is_none());
    println!("missing=None");
}
