//! 连接池操作微基准（harness = false）。
//!
//! 覆盖：获取/归还、健康检查、参数化 SQL、批量执行、COPY IN/OUT、
//! 事务提交、Repository save/find。
//!
//! 需要可连接的 Postgres（同 live 环境变量）。无环境时跳过，不产生失败。
//!
//! ```bash
//! cargo bench -p postgresx --bench pool_operations
//! ```

use contracts::Repository;
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use kernel::XError;
use postgresx::{PgRecord, PgRepository, PostgresConfig, PostgresPool};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::runtime::Runtime;

/// 从环境变量构建 Postgres 池；失败时打印 skip 并返回 None。
fn connect_or_skip() -> Option<(PostgresPool, Runtime)> {
    let rt = Runtime::new().expect("tokio runtime");
    let pool = rt.block_on(async {
        let cfg = match PostgresConfig::from_env() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("skip pool_operations bench: no config ({e})");
                return None;
            }
        };
        match PostgresPool::connect(&cfg).await {
            Ok(p) => Some(p),
            Err(e) => {
                eprintln!("skip pool_operations bench: connect failed ({e})");
                None
            }
        }
    })?;
    Some((pool, rt))
}

/// 设置基准所需的临时表并预填 1000 行。
fn ensure_bench_tables(rt: &Runtime, pool: &PostgresPool) {
    rt.block_on(async {
        let _ = pool
            .execute(
                "CREATE TEMPORARY TABLE IF NOT EXISTS bench_data (
                    id   SERIAL PRIMARY KEY,
                    val  INTEGER NOT NULL,
                    txt  TEXT,
                    bin  BYTEA
                ) ON COMMIT PRESERVE ROWS",
                &[],
            )
            .await;
        // 预填 1000 行
        let _ = pool.execute("DELETE FROM bench_data", &[]).await;
        for chunk in (0..1000).collect::<Vec<_>>().chunks(100) {
            let placeholders: Vec<String> = chunk
                .iter()
                .enumerate()
                .map(|(i, _)| {
                    let base = i * 4;
                    format!("(${}, ${}, ${}, ${})", base + 1, base + 2, base + 3, base + 4)
                })
                .collect();
            let sql = format!(
                "INSERT INTO bench_data (id, val, txt, bin) VALUES {}",
                placeholders.join(", ")
            );

            let mut typed: Vec<Box<dyn postgresx::ToSql + Sync>> = Vec::new();
            for &n in chunk {
                typed.push(Box::new(n));
                typed.push(Box::new(n));
                typed.push(Box::new(format!("text-row-{n:05}")));
                typed.push(Box::new(vec![0u8; 16usize]));
            }
            let refs: Vec<&(dyn postgresx::ToSql + Sync)> =
                typed.iter().map(|p| p.as_ref()).collect();
            let _ = pool.execute(&sql, &refs).await;
        }
    });
}

/// acquire + release（归还由 drop 完成）。
fn bench_acquire_release(c: &mut Criterion) {
    let Some((pool, rt)) = connect_or_skip() else {
        return;
    };

    c.bench_function("acquire_release", |b| {
        b.iter(|| {
            rt.block_on(async {
                let conn = pool.acquire().await.expect("acquire");
                black_box(conn);
            });
        });
    });

    pool.close();
}

/// 健康检查：`SELECT 1`。
fn bench_health_check(c: &mut Criterion) {
    let Some((pool, rt)) = connect_or_skip() else {
        return;
    };

    c.bench_function("health_check", |b| {
        b.iter(|| {
            rt.block_on(async {
                pool.health().await.expect("health");
            });
        });
    });

    pool.close();
}

/// 简单 `EXECUTE`（写一行）。
fn bench_execute_simple(c: &mut Criterion) {
    let Some((pool, rt)) = connect_or_skip() else {
        return;
    };
    ensure_bench_tables(&rt, &pool);

    let counter = Arc::new(AtomicU64::new(0));

    c.bench_function("execute_simple", |b| {
        b.iter(|| {
            let id = counter.fetch_add(1, Ordering::Relaxed);
            rt.block_on(async {
                let n = pool
                    .execute(
                        "INSERT INTO bench_data (val, txt) VALUES ($1, $2)",
                        &[&(id as i32), &format!("bench-{id}")],
                    )
                    .await
                    .expect("execute");
                black_box(n);
            });
        });
    });

    pool.close();
}

/// 查询恰好一行（参数化 `query_one`）。
fn bench_query_one(c: &mut Criterion) {
    let Some((pool, rt)) = connect_or_skip() else {
        return;
    };
    ensure_bench_tables(&rt, &pool);

    c.bench_function("query_one", |b| {
        b.iter(|| {
            rt.block_on(async {
                let row = pool
                    .query_one("SELECT val, txt FROM bench_data WHERE id = $1", &[&1i32])
                    .await
                    .expect("query_one");
                let v: i32 = row.get(0);
                let t: &str = row.get(1);
                black_box((v, t));
            });
        });
    });

    pool.close();
}

/// 查询 100 行（`query` 返回 Vec<Row>）。
fn bench_query_100_rows(c: &mut Criterion) {
    let Some((pool, rt)) = connect_or_skip() else {
        return;
    };
    ensure_bench_tables(&rt, &pool);

    c.bench_function("query_100_rows", |b| {
        b.iter(|| {
            rt.block_on(async {
                let rows = pool
                    .query("SELECT id, val, txt FROM bench_data LIMIT 100", &[])
                    .await
                    .expect("query 100");
                black_box(&rows);
                assert!(!rows.is_empty());
            });
        });
    });

    pool.close();
}

/// 批量多语句执行：模拟 DDL / migration 场景，通过 acquire 获取连接
/// 后使用 [`PgConnection::batch_execute`]。
fn bench_execute_batch(c: &mut Criterion) {
    let Some((pool, rt)) = connect_or_skip() else {
        return;
    };

    let counter = AtomicU64::new(0);

    c.bench_function("execute_batch", |b| {
        b.iter(|| {
            let id = counter.fetch_add(1, Ordering::Relaxed);
            rt.block_on(async {
                let _ = pool
                    .execute(
                        &format!(
                            "CREATE TEMPORARY TABLE IF NOT EXISTS bench_batch_{id} \
                             (n INT) ON COMMIT DROP"
                        ),
                        &[],
                    )
                    .await
                    .expect("create");
                let mut conn = pool.acquire().await.expect("acquire");
                conn.batch_execute(&format!(
                    "INSERT INTO bench_batch_{id} VALUES (1),(2),(3); \
                     DROP TABLE bench_batch_{id}"
                ))
                .await
                .expect("batch_execute");
            });
        });
    });

    pool.close();
}

/// COPY IN + COPY OUT（有界载荷 1 KiB）。
fn bench_copy_in_out(c: &mut Criterion) {
    let Some((pool, rt)) = connect_or_skip() else {
        return;
    };

    rt.block_on(async {
        let _ = pool
            .execute(
                "CREATE TEMPORARY TABLE IF NOT EXISTS bench_copy (
                    id   SERIAL PRIMARY KEY,
                    data BYTEA NOT NULL
                ) ON COMMIT PRESERVE ROWS",
                &[],
            )
            .await;
    });

    let payload = vec![65u8; 1024]; // 1 KiB

    c.bench_function("copy_in", |b| {
        b.iter(|| {
            rt.block_on(async {
                let n = pool
                    .copy_in_bytes("COPY bench_copy (data) FROM STDIN BINARY", &payload)
                    .await
                    .expect("copy_in");
                black_box(n);
            });
        });
    });

    c.bench_function("copy_out", |b| {
        b.iter(|| {
            rt.block_on(async {
                let data = pool
                    .copy_out_bytes("COPY bench_copy (data) TO STDOUT BINARY", 65536)
                    .await
                    .expect("copy_out");
                black_box(data);
            });
        });
    });

    pool.close();
}

/// with_transaction：写两条 INSERT 后 commit。
fn bench_with_transaction_commit(c: &mut Criterion) {
    let Some((pool, rt)) = connect_or_skip() else {
        return;
    };
    ensure_bench_tables(&rt, &pool);

    let counter = Arc::new(AtomicU64::new(0));

    c.bench_function("with_transaction_commit", |b| {
        b.iter(|| {
            let base = counter.fetch_add(2, Ordering::Relaxed);
            rt.block_on(async {
                let vals = pool
                    .with_transaction(async |tx| {
                        tx.execute(
                            "INSERT INTO bench_data (val, txt) VALUES ($1, $2)",
                            &[&(base as i32), &format!("tx-a-{base}")],
                        )
                        .await?;
                        tx.execute(
                            "INSERT INTO bench_data (val, txt) VALUES ($1, $2)",
                            &[&((base + 1) as i32), &format!("tx-b-{}", base + 1)],
                        )
                        .await?;
                        Ok::<_, XError>((base, base + 1))
                    })
                    .await
                    .expect("with_transaction");
                black_box(vals);
            });
        });
    });

    pool.close();
}

/// Repository save + find 往返。
fn bench_repository_save_find(c: &mut Criterion) {
    let Some((pool, rt)) = connect_or_skip() else {
        return;
    };
    let repo = PgRepository::new(pool.clone());

    rt.block_on(async {
        repo.ensure_table().await.expect("ensure_table");
    });

    // 用固定 key 测 find（先存一条）
    let find_id = "bench-find-static";
    rt.block_on(async {
        let _ = repo.save(&PgRecord { id: find_id.to_string(), data: vec![2u8; 32] }).await;
    });

    let mut group = c.benchmark_group("repository");
    let counter = Arc::new(AtomicU64::new(0));

    group.bench_function("save", |b| {
        b.iter(|| {
            let id = format!("bench-{}", counter.fetch_add(1, Ordering::Relaxed));
            rt.block_on(async {
                let record = PgRecord { id, data: vec![1u8; 64] };
                repo.save(&record).await.expect("save");
            });
        });
    });

    group.bench_function("find", |b| {
        b.iter(|| {
            rt.block_on(async {
                let record = repo.find(find_id.to_string()).await.expect("find");
                black_box(record);
            });
        });
    });

    group.finish();
    pool.close();
}

criterion_group!(
    benches,
    bench_acquire_release,
    bench_health_check,
    bench_execute_simple,
    bench_query_one,
    bench_query_100_rows,
    bench_execute_batch,
    bench_copy_in_out,
    bench_with_transaction_commit,
    bench_repository_save_find
);
criterion_main!(benches);
