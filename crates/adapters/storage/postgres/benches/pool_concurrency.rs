//! 连接池并发微基准（harness = false）。
//!
//! 覆盖：并行 acquire 吞吐、混合读写、池饱和恢复。
//!
//! 需要可连接的 Postgres（同 live 环境变量）。无环境时跳过。
//!
//! ```bash
//! cargo bench -p postgresx --bench pool_concurrency
//! ```

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use postgresx::{PostgresConfig, PostgresPool};
use std::time::Duration;
use tokio::runtime::Runtime;

/// 从环境变量构建 Postgres 池；失败时打印 skip 并返回 None。
fn connect_or_skip(max_pool_size: usize) -> Option<(PostgresPool, Runtime)> {
    let rt = Runtime::new().expect("tokio runtime");
    let pool = rt.block_on(async {
        let mut cfg = match PostgresConfig::from_env() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("skip pool_concurrency bench: no config ({e})");
                return None;
            }
        };
        cfg.max_pool_size = max_pool_size;
        cfg.acquire_timeout = Duration::from_secs(10);
        cfg.operation_timeout = Duration::from_secs(10);
        match PostgresPool::connect(&cfg).await {
            Ok(p) => Some(p),
            Err(e) => {
                eprintln!("skip pool_concurrency bench: connect failed ({e})");
                None
            }
        }
    })?;
    Some((pool, rt))
}

/// 建立临时表。
fn ensure_bench_tables(rt: &Runtime, pool: &PostgresPool) {
    rt.block_on(async {
        let _ = pool
            .execute(
                "CREATE TEMPORARY TABLE IF NOT EXISTS bench_concurrent (
                    id   SERIAL PRIMARY KEY,
                    val  INTEGER NOT NULL,
                    txt  TEXT
                ) ON COMMIT PRESERVE ROWS",
                &[],
            )
            .await;
        let _ = pool.execute("DELETE FROM bench_concurrent", &[]).await;
    });
}

/// 并行 acquire 吞吐量：N 个任务同时借/还连接。
fn bench_parallel_acquires(c: &mut Criterion) {
    let Some((pool, rt)) = connect_or_skip(16) else {
        return;
    };

    for &concurrency in &[1, 4, 8, 16, 32] {
        c.bench_function(&format!("parallel_acquires_c{concurrency}"), |b| {
            let pool = pool.clone();
            b.iter(|| {
                rt.block_on(async {
                    let mut handles = Vec::with_capacity(concurrency);
                    for _ in 0..concurrency {
                        let pool = pool.clone();
                        handles.push(tokio::task::spawn(async move {
                            let conn = pool.acquire().await.expect("acquire");
                            // 立即 drop 归还
                            black_box(conn);
                        }));
                    }
                    for h in handles {
                        h.await.expect("join");
                    }
                });
            });
        });
    }

    pool.close();
}

/// 混合读写：并发下各自执行 SELECT / INSERT。
fn bench_mixed_read_write(c: &mut Criterion) {
    let Some((pool, rt)) = connect_or_skip(16) else {
        return;
    };
    ensure_bench_tables(&rt, &pool);

    // 预填数据供读取
    rt.block_on(async {
        for n in 0..500 {
            let _ = pool
                .execute(
                    "INSERT INTO bench_concurrent (val, txt) VALUES ($1, $2)",
                    &[&n, &format!("row-{n}")],
                )
                .await;
        }
    });

    for &concurrency in &[4, 8, 16] {
        c.bench_function(&format!("mixed_read_write_c{concurrency}"), |b| {
            let pool = pool.clone();
            b.iter(|| {
                rt.block_on(async {
                    let mut handles = Vec::with_capacity(concurrency);
                    for i in 0..concurrency {
                        let pool = pool.clone();
                        handles.push(tokio::task::spawn(async move {
                            if i % 2 == 0 {
                                // 写任务
                                let n = pool
                                    .execute(
                                        "INSERT INTO bench_concurrent (val, txt) VALUES ($1, $2)",
                                        &[&(i as i32 + 10000), &format!("w-{i}")],
                                    )
                                    .await
                                    .expect("insert");
                                black_box(n);
                            } else {
                                // 读任务
                                let rows = pool
                                    .query("SELECT id, val FROM bench_concurrent LIMIT 10", &[])
                                    .await
                                    .expect("query");
                                black_box(rows);
                            }
                        }));
                    }
                    for h in handles {
                        h.await.expect("join");
                    }
                });
            });
        });
    }

    pool.close();
}

/// 池饱和恢复：瞬间堆积超过池大小的 acquire 请求，测量全部返回的时间。
fn bench_saturation_recovery(c: &mut Criterion) {
    let pool_size = 4;
    let Some((pool, rt)) = connect_or_skip(pool_size) else {
        return;
    };

    // 预占用全部连接
    rt.block_on(async {
        let mut holders = Vec::new();
        for _ in 0..pool_size {
            let conn = pool.acquire().await.expect("acquire for saturation");
            holders.push(conn);
        }

        let pool_clone = pool.clone();
        let num_over = pool_size * 3; // 3x 超额请求

        let start = std::time::Instant::now();
        let mut handles = Vec::with_capacity(num_over);
        for _ in 0..num_over {
            let p = pool_clone.clone();
            handles.push(tokio::task::spawn(async move {
                p.acquire().await.expect("acquire after release")
            }));
        }

        // 释放占用，让等待队列前进
        drop(holders);

        for h in handles {
            h.await.expect("join");
        }
        let elapsed = start.elapsed();
        eprintln!(
            "saturation_recovery: {num_over} acquires after releasing {pool_size} \
             pre-occupied, total={elapsed:?}"
        );
    });

    // 用较粗粒度测量：创建/释放占满池 + 批量请求
    c.bench_function("saturation_recovery", |b| {
        b.iter(|| {
            rt.block_on(async {
                // 占用全池
                let mut holders = Vec::new();
                for _ in 0..pool_size {
                    holders.push(pool.acquire().await.expect("acquire"));
                }

                let num_over = pool_size * 2;
                let mut handles = Vec::with_capacity(num_over);
                for _ in 0..num_over {
                    let p = pool.clone();
                    handles.push(tokio::task::spawn(
                        async move { p.acquire().await.expect("acquire") },
                    ));
                }

                drop(holders); // 释放

                for h in handles {
                    h.await.expect("join");
                }
            });
        });
    });

    pool.close();
}

criterion_group!(
    benches,
    bench_parallel_acquires,
    bench_mixed_read_write,
    bench_saturation_recovery
);
criterion_main!(benches);
