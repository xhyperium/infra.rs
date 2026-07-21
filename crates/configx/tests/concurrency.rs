//! 并发读写 smoke：不证明公平性，只验证可恢复路径不 panic 且数据可见。

use configx::ConfigStore;
use std::sync::Arc;
use std::thread;

#[test]
fn concurrent_set_get_no_panic() {
    let store = Arc::new(ConfigStore::new());
    let n_threads = 8usize;
    let iters = 100usize;

    let handles: Vec<_> = (0..n_threads)
        .map(|t| {
            let s = Arc::clone(&store);
            thread::spawn(move || {
                for i in 0..iters {
                    let key = format!("t{t}");
                    s.set(&key, format!("{i}")).expect("set");
                    let _ = s.get(&key);
                    let _ = s.get("other");
                }
            })
        })
        .collect();

    for h in handles {
        h.join().expect("join");
    }

    for t in 0..n_threads {
        let v = store.get(&format!("t{t}")).unwrap_or_else(|| panic!("missing key t{t}"));
        // 最后一次写入为 iters-1
        assert_eq!(v, format!("{}", iters - 1));
    }
}
