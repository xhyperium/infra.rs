//! redisx **集成测试**：覆盖公开 API 面（需 live Redis）。
//!
//! ```bash
//! # 凭据来自 ZoneCNH secrets/env/dev.md，不回显密码
//! scripts/live/export-foundationx-env.sh --env dev -- \
//!   cargo test -p redisx --features pubsub --test integration_all_api -- --ignored --nocapture
//! ```

use std::time::Duration;

use contracts::{KeyValueStore, PubSub};
use kernel::ErrorKind;
use redisx::selfcheck::{CheckLevel, CheckStatus, RedisValidator};
use redisx::{RedisClient, RedisConfig, RedisLiveKv, RedisPool, TxCmd};

fn prefix() -> String {
    format!("redisx-it:{}:", std::process::id())
}

#[tokio::test]
#[ignore = "requires live Redis via FOUNDATIONX_REDISX_*"]
async fn it_config_pool_client_lifecycle() {
    let cfg = RedisConfig::from_env().expect("FOUNDATIONX_REDISX_* 或 REDIS_URL");
    let ep = cfg.display_endpoint();
    // CI service Redis 可无密码；有密码时必须脱敏
    if std::env::var("FOUNDATIONX_REDISX_PASSWORD").is_ok()
        || std::env::var("REDIS_URL").map(|u| u.contains('@') && u.contains(':')).unwrap_or(false)
    {
        assert!(
            ep.contains("***") || !ep.contains("://:***"),
            "password must not appear in endpoint: {ep}"
        );
        // 若配置了 password 字段，endpoint 应含 ***
        if ep.contains('@') {
            assert!(ep.contains("***"), "password must be redacted in endpoint: {ep}");
        }
    }
    assert!(!ep.is_empty());

    let pool = RedisPool::connect(cfg).await.expect("connect");
    assert!(pool.liveness());
    let rtt = pool.readiness().await.expect("readiness");
    assert!(rtt < Duration::from_secs(3));
    assert!(pool.stats().open >= 1, "lanes={}", pool.stats().open);
    assert_eq!(pool.command_lanes(), pool.stats().open);
    assert!(pool.metrics_snapshot().commands_ok >= 1);

    let client = pool.client().with_call_deadline(Duration::from_secs(10));
    assert!(client.has_call_deadline());
    assert!(!client.endpoint().is_empty());

    let key = format!("{}lifecycle", prefix());
    client.set(&key, b"v".to_vec(), Some(Duration::from_secs(60))).await.expect("set");
    assert_eq!(client.get(&key).await.expect("get"), Some(b"v".to_vec()));
    assert!(client.exists(&key).await.expect("exists"));
    assert!(client.ttl(&key).await.expect("ttl").is_some());
    assert!(client.expire(&key, Duration::from_secs(120)).await.expect("expire"));
    assert!(client.delete(&key).await.expect("del"));

    pool.close(Duration::from_secs(3)).await.expect("close");
    assert!(pool.is_closed());
    let err = pool.client().get("x").await.expect_err("closed");
    assert_eq!(err.kind(), ErrorKind::Unavailable);
    assert!(pool.metrics_snapshot().rejected_closed >= 1);
}

#[tokio::test]
#[ignore = "requires live Redis via FOUNDATIONX_REDISX_*"]
async fn it_kv_bytes_mset_mget_pipeline() {
    let client = RedisClient::connect_from_env().await.expect("connect");
    let p = prefix();
    let k1 = format!("{p}b1");
    let k2 = format!("{p}b2");
    let k3 = format!("{p}b3");
    let missing = format!("{p}missing");

    client.set_bytes(&k1, vec![0, 1, 255], Some(Duration::from_secs(90))).await.expect("set_bytes");
    assert_eq!(client.get_bytes(&k1).await.expect("get_bytes"), Some(vec![0, 1, 255]));

    client.mset(&[(&k2, b"two"), (&k3, b"three")]).await.expect("mset");
    let got = client.mget(&[&k2, &k3, &missing]).await.expect("mget");
    assert_eq!(got[0].as_deref(), Some(b"two".as_slice()));
    assert_eq!(got[1].as_deref(), Some(b"three".as_slice()));
    assert!(got[2].is_none());

    let p1 = format!("{p}p1");
    let p2 = format!("{p}p2");
    client
        .pipeline_set(&[(&p1, b"A".to_vec()), (&p2, b"B".to_vec())], Some(Duration::from_secs(60)))
        .await
        .expect("pipeline");
    assert_eq!(client.get(&p1).await.expect("g1"), Some(b"A".to_vec()));
    assert_eq!(client.get(&p2).await.expect("g2"), Some(b"B".to_vec()));

    for k in [&k1, &k2, &k3, &p1, &p2] {
        let _ = client.delete(k).await;
    }
}

#[tokio::test]
#[ignore = "requires live Redis via FOUNDATIONX_REDISX_*"]
async fn it_lua_and_dist_lock() {
    let client = RedisClient::connect_from_env().await.expect("connect");
    let p = prefix();
    let key = format!("{p}lua");
    client.set(&key, b"1".to_vec(), Some(Duration::from_secs(60))).await.expect("seed");
    let v =
        client.eval_script("return redis.call('GET', KEYS[1])", &[&key], &[]).await.expect("eval");
    match v {
        redis::Value::BulkString(b) => assert_eq!(b.as_slice(), b"1"),
        redis::Value::SimpleString(s) => assert_eq!(s.as_bytes(), b"1"),
        other => panic!("unexpected lua value {other:?}"),
    }

    let lock_key = format!("{p}lock");
    let lock = client.lock_acquire(&lock_key, Duration::from_secs(20)).await.expect("lock");
    assert!(lock.fence() >= 1);
    let conflict = client.lock_acquire(&lock_key, Duration::from_secs(5)).await;
    assert!(matches!(conflict, Err(e) if e.kind() == ErrorKind::Conflict));
    assert!(client.lock_extend(&lock, Duration::from_secs(30)).await.expect("extend"));
    assert!(client.lock_release(&lock).await.expect("release"));
    let lock2 = client.lock_acquire(&lock_key, Duration::from_secs(10)).await.expect("re");
    assert!(lock2.fence() > lock.fence());
    let _ = client.lock_release(&lock2).await;
    let _ = client.delete(&key).await;
}

#[tokio::test]
#[ignore = "requires live Redis via FOUNDATIONX_REDISX_*"]
async fn it_contract_trait_and_invalid_ttl() {
    let store: RedisLiveKv = RedisLiveKv::connect_from_env().await.expect("connect");
    let kv: &dyn KeyValueStore = &store;
    let key = format!("{}trait", prefix());
    kv.set(&key, b"tv".to_vec(), None).await.expect("set");
    assert_eq!(kv.get(&key).await.expect("get"), Some(b"tv".to_vec()));
    let _ = store.delete(&key).await;

    let err = store.set(&key, b"x".to_vec(), Some(Duration::ZERO)).await.expect_err("ttl0");
    assert_eq!(err.kind(), ErrorKind::Invalid);
}

#[tokio::test]
#[ignore = "requires live Redis via FOUNDATIONX_REDISX_*"]
async fn it_selfcheck_full() {
    let client = RedisClient::connect_from_env().await.expect("connect");
    let report = RedisValidator::new(client).run(CheckLevel::Full).await;
    assert_eq!(report.module, "redisx");
    assert_eq!(RedisValidator::static_catalog().len(), 11);
    for item in &report.items {
        if item.id == "redisx.full.cluster_slots" {
            assert_eq!(item.status, CheckStatus::Skipped, "{item:?}");
            continue;
        }
        #[cfg(not(feature = "pubsub"))]
        if item.id == "redisx.full.pubsub" {
            assert_eq!(item.status, CheckStatus::Skipped, "{item:?}");
            continue;
        }
        assert!(
            matches!(item.status, CheckStatus::Passed | CheckStatus::Degraded),
            "unexpected {item:?}"
        );
    }
    assert!(report.passed, "failed: {:?}", report.items);
}

#[tokio::test]
#[ignore = "requires live Redis + feature pubsub"]
async fn it_pubsub_roundtrip() {
    #[cfg(feature = "pubsub")]
    {
        use futures_util::StreamExt;
        use redisx::RedisPubSub;
        use tokio::time::timeout;

        let cfg = RedisConfig::from_env().expect("cfg");
        let channel = format!("redisx-it-{}-ch", std::process::id());
        let session = RedisPubSub::connect_config(cfg, [channel.clone()]).await.expect("sub");
        let mut stream = session.into_message_stream().expect("stream");

        // 独立 publish 会话
        let pub_cfg = RedisConfig::from_env().expect("cfg2");
        let publisher =
            RedisPubSub::connect_config(pub_cfg, Vec::<String>::new()).await.expect("publisher");
        let payload = b"hello-pubsub";
        publisher.publish(&channel, payload).await.expect("publish");

        let ok = timeout(Duration::from_secs(3), async {
            while let Some(msg) = stream.next().await {
                if msg.payload.as_ref() == payload {
                    return true;
                }
            }
            false
        })
        .await
        .expect("timeout waiting pubsub");
        assert!(ok, "message not received");
    }
    #[cfg(not(feature = "pubsub"))]
    {
        eprintln!("soft-skip: rebuild with --features pubsub for it_pubsub_roundtrip");
    }
}

#[tokio::test]
#[ignore = "requires live Redis via FOUNDATIONX_REDISX_*"]
async fn it_connect_from_url_compat() {
    // 仅验证 from_env 路径已覆盖 URL；这里测 builder 显式连接
    let cfg = RedisConfig::from_env().expect("cfg");
    let pool = RedisPool::connect(cfg).await.expect("connect");
    pool.ping().await.expect("ping");
    let _ = pool.close(Duration::from_secs(2)).await;
}

#[tokio::test]
#[ignore = "requires live Redis via FOUNDATIONX_REDISX_*"]
async fn it_retry_budget_get_set_live() {
    use resiliencx::RetryBudget;

    let client = RedisClient::connect_from_env()
        .await
        .expect("connect")
        .with_retry_budget(RetryBudget::new(8), 3);
    assert!(client.has_retry_budget());
    let key = format!("{}budget", prefix());
    client.set(&key, b"budget-v".to_vec(), None).await.expect("set with client budget");
    assert_eq!(
        client.get(&key).await.expect("get with client budget").as_deref(),
        Some(b"budget-v".as_slice())
    );

    let budget = RetryBudget::new(4);
    client
        .set_with_budget(&key, b"explicit".to_vec(), None, &budget, 2)
        .await
        .expect("set_with_budget");
    assert_eq!(
        client.get_with_budget(&key, &budget, 2).await.expect("get_with_budget").as_deref(),
        Some(b"explicit".as_slice())
    );
    // 相对 TTL + multi attempt → fail-closed before driver
    let err = client
        .set_with_budget(&key, b"ttl".to_vec(), Some(Duration::from_secs(30)), &budget, 3)
        .await
        .expect_err("relative TTL multi-attempt");
    assert_eq!(err.kind(), ErrorKind::Invalid);
    let _ = client.delete(&key).await;
}

#[tokio::test]
#[ignore = "requires live Redis + feature pubsub"]
async fn it_result_message_stream_and_facade() {
    #[cfg(feature = "pubsub")]
    {
        use futures_util::StreamExt;
        use redisx::{RedisPubSub, RedisPubSubFacade};
        use tokio::time::timeout;

        let cfg = RedisConfig::from_env().expect("cfg");
        let channel = format!("redisx-it-{}-result", std::process::id());

        // result stream：消息后可继续等到断开 Err（本测只收一条 Ok）
        let session =
            RedisPubSub::connect_config(cfg.clone(), [channel.clone()]).await.expect("sub");
        let mut stream = session.into_result_message_stream().expect("result stream");

        let publisher =
            RedisPubSub::connect_config(cfg.clone(), Vec::<String>::new()).await.expect("pub");
        let payload = b"result-stream-payload";
        publisher.publish(&channel, payload).await.expect("publish");

        let msg = timeout(Duration::from_secs(3), stream.next())
            .await
            .expect("timeout")
            .expect("stream ended")
            .expect("Ok message");
        assert_eq!(msg.payload.as_ref(), payload);

        // Facade：pub_message + sub_channel
        let facade = RedisPubSubFacade::connect(cfg).await.expect("facade");
        let ch2 = format!("redisx-it-{}-facade", std::process::id());
        let mut sub = facade.sub_channel(&ch2).await.expect("facade sub");
        facade.pub_message(&ch2, bytes::Bytes::from_static(b"facade-hi")).await.expect("pub");
        let got = timeout(Duration::from_secs(3), sub.next())
            .await
            .expect("facade timeout")
            .expect("facade msg");
        assert_eq!(got.payload.as_ref(), b"facade-hi");
    }
    #[cfg(not(feature = "pubsub"))]
    {
        eprintln!("soft-skip: rebuild with --features pubsub for result stream/facade");
    }
}

#[tokio::test]
#[ignore = "requires live Redis via FOUNDATIONX_REDISX_*"]
async fn it_selfcheck_json_report_and_cancel() {
    use redisx::selfcheck::{RedisSelfCheckConfig, ValidationContext};

    let client = RedisClient::connect_from_env().await.expect("connect");
    let v = RedisValidator::new(client.clone());
    let json = v.run_json(CheckLevel::Basic).await.expect("json");
    assert!(json.contains("\"module\": \"redisx\"") || json.contains("\"module\":\"redisx\""));
    assert!(json.contains("redisx.basic.ping"));
    let report: serde_json::Value = serde_json::from_str(&json).expect("parse json");
    assert_eq!(report["module"], "redisx");

    // cancel 路径：不 panic、项均为 Skipped
    let ctx = ValidationContext::new(RedisSelfCheckConfig::default());
    ctx.cancel.cancel();
    let cancelled = v.run_with_context(&ctx, CheckLevel::ReadWrite).await;
    assert!(cancelled.passed);
    assert!(cancelled.items.iter().all(|i| i.status == CheckStatus::Skipped));
}

#[tokio::test]
#[ignore = "requires live Redis via FOUNDATIONX_REDISX_*"]
async fn it_data_structures_hash_list_set_zset() {
    let client = RedisClient::connect_from_env().await.expect("connect");
    let p = prefix();
    let h = format!("{p}hash");
    let l = format!("{p}list");
    let s = format!("{p}set");
    let z = format!("{p}zset");

    assert!(client.hset(&h, "f1", b"v1".to_vec()).await.expect("hset"));
    assert_eq!(client.hget(&h, "f1").await.expect("hget").as_deref(), Some(b"v1".as_slice()));
    let all = client.hgetall(&h).await.expect("hgetall");
    assert!(all.iter().any(|(k, v)| k == "f1" && v.as_slice() == b"v1"));
    assert_eq!(client.hdel(&h, &["f1"]).await.expect("hdel"), 1);

    assert!(client.lpush(&l, b"a".to_vec()).await.expect("lpush") >= 1);
    assert!(client.rpush(&l, b"b".to_vec()).await.expect("rpush") >= 2);
    let range = client.lrange(&l, 0, -1).await.expect("lrange");
    assert!(range.len() >= 2);
    let _ = client.lpop(&l).await.expect("lpop");

    assert_eq!(client.sadd(&s, b"m1".to_vec()).await.expect("sadd"), 1);
    assert!(client.sismember(&s, b"m1").await.expect("sismember"));
    assert_eq!(client.srem(&s, b"m1").await.expect("srem"), 1);

    assert_eq!(client.zadd(&z, b"zm".to_vec(), 3.5).await.expect("zadd"), 1);
    assert_eq!(client.zscore(&z, b"zm").await.expect("zscore"), Some(3.5));
    assert_eq!(client.zrem(&z, b"zm").await.expect("zrem"), 1);

    for k in [&h, &l, &s, &z] {
        let _ = client.delete(k).await;
    }
}

#[tokio::test]
#[ignore = "requires live Redis via FOUNDATIONX_REDISX_*"]
async fn it_streams_and_multi_exec() {
    let client = RedisClient::connect_from_env().await.expect("connect");
    let p = prefix();
    let stream = format!("{p}stream");
    let k1 = format!("{p}tx1");
    let k2 = format!("{p}tx2");

    let id = client.xadd(&stream, &[("sym", b"BTC"), ("px", b"1")]).await.expect("xadd");
    assert!(!id.is_empty());
    assert!(client.xlen(&stream).await.expect("xlen") >= 1);
    let range = client.xrange(&stream, "-", "+", Some(10)).await.expect("xrange");
    assert!(range.iter().any(|e| e.id == id));
    let read = client.xread(&stream, "0-0", Some(10)).await.expect("xread");
    assert!(!read.is_empty());
    assert_eq!(client.xdel(&stream, &[&id]).await.expect("xdel"), 1);

    let vals = client
        .multi_exec(&[
            TxCmd::Set { key: k1.clone(), value: b"a".to_vec() },
            TxCmd::Set { key: k2.clone(), value: b"b".to_vec() },
            TxCmd::Incr { key: format!("{p}cnt") },
        ])
        .await
        .expect("multi_exec");
    assert_eq!(vals.len(), 3);
    assert_eq!(client.get(&k1).await.expect("g1").as_deref(), Some(b"a".as_slice()));
    client.multi_set(&[(&k1, b"A2"), (&k2, b"B2")]).await.expect("multi_set");
    assert_eq!(client.get(&k2).await.expect("g2").as_deref(), Some(b"B2".as_slice()));

    // SCRIPT LOAD + EVALSHA
    let (sha, v) = client
        .script_load_and_eval("return ARGV[1]", &[], &[b"sha-ok"])
        .await
        .expect("script load");
    assert!(!sha.is_empty());
    match v {
        redis::Value::BulkString(b) => assert_eq!(b.as_slice(), b"sha-ok"),
        other => panic!("unexpected {other:?}"),
    }
    let v2 = client.eval_sha(&sha, &[], &[b"again"]).await.expect("eval_sha");
    match v2 {
        redis::Value::BulkString(b) => assert_eq!(b.as_slice(), b"again"),
        other => panic!("unexpected {other:?}"),
    }

    for k in [&stream, &k1, &k2, &format!("{p}cnt")] {
        let _ = client.delete(k).await;
    }
}

#[tokio::test]
#[ignore = "requires live Redis via FOUNDATIONX_REDISX_*"]
async fn it_config_hardenings_and_topology_probe() {
    // 硬化配置面可构造；真实 Cluster/Sentinel/TLS 仅当 env 提供
    let base = RedisConfig::from_env().expect("env");
    let mut b = RedisConfig::builder()
        .addr(base.addr())
        .db(base.db())
        .tls(base.tls())
        .mode(base.mode())
        .warmup_count(1)
        .command_lanes(8)
        .max_cluster_redirects(8)
        .blocking_timeout(Duration::from_secs(2))
        .reconnect_max_delay(Duration::from_secs(2))
        .tcp_keepalive(Duration::from_secs(30))
        .connect_timeout(base.connect_timeout())
        .command_timeout(base.command_timeout())
        .acquire_timeout(base.acquire_timeout())
        .password_from_provider(|| {
            std::env::var("FOUNDATIONX_REDISX_PASSWORD").ok().filter(|s| !s.is_empty())
        });
    if let Ok(u) = std::env::var("FOUNDATIONX_REDISX_USERNAME") {
        if !u.is_empty() {
            b = b.username(u);
        }
    }
    let hardened = b.build().expect("hardened cfg");
    assert_eq!(hardened.warmup_count(), 1);
    assert_eq!(hardened.command_lanes(), 8);
    assert!(hardened.tcp_keepalive().is_some());
    let pool = RedisPool::connect(hardened).await.expect("connect hardened");
    assert!(pool.liveness());
    pool.ping().await.expect("ping");
    assert_eq!(pool.stats().open, 8);
    let _ = pool.close(Duration::from_secs(2)).await;

    // 拓扑探测：无 Cluster/Sentinel/TLS env 时诚实 soft-skip（不伪绿）
    let mode = std::env::var("FOUNDATIONX_REDISX_MODE").unwrap_or_else(|_| "standalone".into());
    let tls = std::env::var("FOUNDATIONX_REDISX_TLS").unwrap_or_else(|_| "false".into());
    if mode.eq_ignore_ascii_case("cluster") {
        let cfg = RedisConfig::from_env().expect("cluster env");
        assert_eq!(cfg.mode(), redisx::RedisMode::Cluster);
        let p = RedisPool::connect(cfg).await.expect("cluster live");
        p.ping().await.expect("cluster ping");
        let _ = p.close(Duration::from_secs(2)).await;
    } else if mode.eq_ignore_ascii_case("sentinel") {
        let cfg = RedisConfig::from_env().expect("sentinel env");
        assert_eq!(cfg.mode(), redisx::RedisMode::Sentinel);
        let p = RedisPool::connect(cfg).await.expect("sentinel live");
        p.ping().await.expect("sentinel ping");
        let _ = p.close(Duration::from_secs(2)).await;
    } else {
        eprintln!("topology soft-skip: MODE={mode} TLS={tls} (no Cluster/Sentinel live env)");
    }
    if tls.eq_ignore_ascii_case("true") || tls == "1" {
        let cfg = RedisConfig::from_env().expect("tls env");
        assert!(cfg.tls());
        let p = RedisPool::connect(cfg).await.expect("tls live");
        p.ping().await.expect("tls ping");
        let _ = p.close(Duration::from_secs(2)).await;
    } else {
        eprintln!("tls soft-skip: FOUNDATIONX_REDISX_TLS={tls}");
    }
}

#[tokio::test]
#[ignore = "requires live Redis via FOUNDATIONX_REDISX_*"]
async fn it_blpop_timeout_none() {
    let client = RedisClient::connect_from_env().await.expect("connect");
    let key = format!("{}blpop-empty", prefix());
    let _ = client.delete(&key).await;
    // 1s 阻塞等待，空 list → None（或 deadline，均非 panic）
    let got = client.blpop(&key, Duration::from_secs(1)).await;
    match got {
        Ok(None) => {}
        Ok(Some(_)) => panic!("unexpected element"),
        Err(e) => {
            // 部分环境把阻塞截止映射为 DeadlineExceeded
            assert!(
                matches!(e.kind(), ErrorKind::DeadlineExceeded | ErrorKind::Transient),
                "kind={:?}",
                e.kind()
            );
        }
    }
}
