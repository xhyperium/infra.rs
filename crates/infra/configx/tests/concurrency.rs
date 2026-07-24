//! 并发读写 smoke：不证明公平性，只验证可恢复路径不 panic 且数据可见。

use configx::ConfigStore;
use std::sync::{Arc, mpsc};
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

#[test]
fn extend_pairs_does_not_expose_partial_commit() {
    struct BlockingPairs {
        step: u8,
        blocked: mpsc::Sender<()>,
        release: mpsc::Receiver<()>,
    }

    impl Iterator for BlockingPairs {
        type Item = (&'static str, &'static str);

        fn next(&mut self) -> Option<Self::Item> {
            match self.step {
                0 => {
                    self.step = 1;
                    Some(("new-a", "1"))
                }
                1 => {
                    self.step = 2;
                    self.blocked.send(()).expect("通知迭代器已阻塞");
                    self.release.recv().expect("等待继续收集批次");
                    Some(("new-b", "2"))
                }
                _ => None,
            }
        }
    }

    let store = Arc::new(ConfigStore::new());
    store.set("old", "value").unwrap();
    let (blocked_tx, blocked_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let writer_store = Arc::clone(&store);
    let writer = thread::spawn(move || {
        writer_store.extend_pairs(BlockingPairs {
            step: 0,
            blocked: blocked_tx,
            release: release_rx,
        })
    });

    blocked_rx.recv().expect("等待批次收集到中点");
    let during = store.try_snapshot().unwrap();
    assert_eq!(during.get("old"), Some("value"));
    assert_eq!(during.get("new-a"), None);
    assert_eq!(during.get("new-b"), None);
    release_tx.send(()).unwrap();
    writer.join().unwrap().unwrap();

    let after = store.try_snapshot().unwrap();
    assert_eq!(after.get("new-a"), Some("1"));
    assert_eq!(after.get("new-b"), Some("2"));
}
