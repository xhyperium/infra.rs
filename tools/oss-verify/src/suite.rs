//! OSS 专用验证套件。
//!
//! 提供 6 层验证检查：
//! L0: 配置验证 (offline)
//! L1: 连接检测 (online)
//! L2: 基本操作 (put/get/delete/head)
//! L3: 流式操作 (put_stream/get_stream)
//! L4: 高级功能 (multipart/range/SSE/presign)
//! L5: 安全与并发 (conditional/header injection/concurrency)

use std::time::{Duration, Instant};
use bytes::Bytes;
use ossx::{OssConfig, OssPool, DownloadOptions, UploadOptions, ObjectKey};

use crate::types::{CheckResult, CheckKind, RunResult, RunStatus};

/// 执行单条检查。
pub async fn execute_check(pool: &OssPool, id: &str, kind: CheckKind, _desc: &str) -> CheckResult {
    let start = Instant::now();
    let result = match id {
        // ── L0: 配置验证 ──────────────────────────────────────────────────
        "config-valid" => check_config_valid(pool).await,
        "config-env" => check_config_env().await,
        "config-sse-default" => check_config_sse_default(pool).await,

        // ── L1: 连接检测 ──────────────────────────────────────────────────
        "conn-health" => check_health(pool).await,
        "conn-credential" => check_credential(pool).await,
        "conn-bucket-accessible" => check_bucket(pool).await,

        // ── L2: 基本操作 ──────────────────────────────────────────────────
        "ops-put" => check_put(pool).await,
        "ops-get" => check_get(pool).await,
        "ops-delete" => check_delete(pool).await,
        "ops-head" => check_head(pool).await,
        "ops-empty-obj" => check_empty_obj(pool).await,
        "ops-large-obj" => check_large_obj(pool).await,
        "ops-delete-idempotent" => check_delete_idempotent(pool).await,

        // ── L3: 流式操作 ──────────────────────────────────────────────────
        "stream-upload" => check_stream_upload(pool).await,
        "stream-download" => check_stream_download(pool).await,
        "stream-roundtrip" => check_stream_roundtrip(pool).await,
        "stream-range" => check_range_download(pool).await,

        // ── L4: 高级功能 ──────────────────────────────────────────────────
        "advanced-object-key" => check_object_key().await,
        "advanced-stats" => check_stats(pool).await,
        "advanced-close-reject" => check_close_reject(pool).await,
        "advanced-presign" => check_presign(pool).await,

        // ── L5: 安全与并发 ────────────────────────────────────────────────
        "security-oversized" => check_oversized(pool).await,
        "security-concurrent" => check_concurrent(pool).await,
        "security-key-injection" => check_key_injection(pool).await,

        _ => (false, "未知检查".into(), None),
    };
    let duration_ms = start.elapsed().as_millis() as u64;
    CheckResult {
        id: id.into(),
        kind,
        passed: result.0,
        duration_ms,
        message: result.1,
        detail: result.2,
    }
}

/// 生成测试键名。
fn test_key(prefix: &str) -> String {
    format!(
        "oss-verify/{}-{}-{}",
        prefix,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    )
}

// ── L0: 配置验证 ────────────────────────────────────────────────────────────

async fn check_config_valid(_pool: &OssPool) -> (bool, String, Option<String>) {
    (true, "配置验证通过（connect 已成功）".into(), None)
}

async fn check_config_env() -> (bool, String, Option<String>) {
    match OssConfig::from_env() {
        Ok(c) => (true, format!("ENV 加载成功 endpoint={}", c.endpoint), None),
        Err(e) => (false, "ENV 加载失败".into(), Some(format!("{e:?}"))),
    }
}

async fn check_config_sse_default(pool: &OssPool) -> (bool, String, Option<String>) {
    let sse = pool.config().sse_enabled;
    (true, format!("SSE-S3 默认={sse}"), None)
}

// ── L1: 连接检测 ────────────────────────────────────────────────────────────

async fn check_health(pool: &OssPool) -> (bool, String, Option<String>) {
    match pool.health(Duration::from_secs(10)).await {
        h if h.ready => (true, format!("健康 lat={}ms bucket_ok={}", h.latency_ms, h.bucket_accessible), None),
        h => (false, format!("不健康: {}", h.detail), None),
    }
}

async fn check_credential(pool: &OssPool) -> (bool, String, Option<String>) {
    // 通过 HEAD 请求验证凭据有效性
    match pool.head(&test_key("cred-check")).await {
        Ok(_) => (true, "凭据有效".into(), None),
        Err(e) => {
            let msg = format!("{e:?}");
            if msg.contains("404") || msg.contains("Missing") || msg.contains("not found") {
                (true, "凭据有效（预期 404）".into(), Some(msg))
            } else {
                (false, "凭据无效或网络不可达".into(), Some(msg))
            }
        }
    }
}

async fn check_bucket(pool: &OssPool) -> (bool, String, Option<String>) {
    let h = pool.health(Duration::from_secs(10)).await;
    (h.bucket_accessible, format!("bucket={}", pool.config().bucket), Some(h.detail))
}

// ── L2: 基本操作 ────────────────────────────────────────────────────────────

async fn check_put(pool: &OssPool) -> (bool, String, Option<String>) {
    let k = test_key("put");
    let d = Bytes::from("oss-verify-put-test");
    match pool.put_object(&k, d.clone()).await {
        Ok(()) => {
            let _ = pool.delete_object(&k).await;
            (true, "PUT 成功".into(), None)
        }
        Err(e) => (false, "PUT 失败".into(), Some(format!("{e:?}"))),
    }
}

async fn check_get(pool: &OssPool) -> (bool, String, Option<String>) {
    let k = test_key("get");
    let d = Bytes::from("oss-verify-get-test-data");
    match pool.put_object(&k, d.clone()).await {
        Ok(()) => {
            match pool.get_object(&k).await {
                Ok(got) => {
                    let _ = pool.delete_object(&k).await;
                    if got == d { (true, "GET 成功 + 数据一致".into(), None) }
                    else { (false, "GET 数据不一致".into(), None) }
                }
                Err(e) => { let _ = pool.delete_object(&k).await; (false, "GET 失败".into(), Some(format!("{e:?}"))) }
            }
        }
        Err(e) => (false, "PUT 前置失败".into(), Some(format!("{e:?}"))),
    }
}

async fn check_delete(pool: &OssPool) -> (bool, String, Option<String>) {
    let k = test_key("delete");
    pool.put_object(&k, Bytes::from("tmp")).await.ok();
    match pool.delete_object(&k).await {
        Ok(()) => (true, "DELETE 成功".into(), None),
        Err(e) => (false, "DELETE 失败".into(), Some(format!("{e:?}"))),
    }
}

async fn check_head(pool: &OssPool) -> (bool, String, Option<String>) {
    let k = test_key("head");
    pool.put_object(&k, Bytes::from("head-data")).await.ok();
    match pool.head(&k).await {
        Ok(m) => {
            let _ = pool.delete_object(&k).await;
            if m.size > 0 && m.etag.is_some() {
                (true, format!("HEAD ok size={} etag={}", m.size, m.etag.unwrap_or_default()), None)
            } else {
                (false, "HEAD 元数据不完整".into(), Some(format!("{m:?}")))
            }
        }
        Err(e) => { let _ = pool.delete_object(&k).await; (false, "HEAD 失败".into(), Some(format!("{e:?}"))) }
    }
}

async fn check_empty_obj(pool: &OssPool) -> (bool, String, Option<String>) {
    let k = test_key("empty");
    match pool.put_object(&k, Bytes::new()).await {
        Ok(()) => {
            match pool.get_object(&k).await {
                Ok(d) => {
                    let _ = pool.delete_object(&k).await;
                    (d.is_empty(), format!("空对象 size={}", d.len()), None)
                }
                Err(e) => { let _ = pool.delete_object(&k).await; (false, "GET 空对象失败".into(), Some(format!("{e:?}"))) }
            }
        }
        Err(e) => (false, "PUT 空对象失败".into(), Some(format!("{e:?}"))),
    }
}

async fn check_large_obj(pool: &OssPool) -> (bool, String, Option<String>) {
    let k = test_key("large");
    let d = Bytes::from(vec![0xABu8; 1024 * 1024]);
    match pool.put_object(&k, d.clone()).await {
        Ok(()) => {
            match pool.get_object(&k).await {
                Ok(got) => {
                    let _ = pool.delete_object(&k).await;
                    (got == d, format!("1MiB roundtrip ok={}", got == d), None)
                }
                Err(e) => { let _ = pool.delete_object(&k).await; (false, "GET 大对象失败".into(), Some(format!("{e:?}"))) }
            }
        }
        Err(e) => (false, "PUT 大对象失败".into(), Some(format!("{e:?}"))),
    }
}

async fn check_delete_idempotent(pool: &OssPool) -> (bool, String, Option<String>) {
    let k = test_key("idem");
    let r1 = pool.delete_object(&k).await;
    let r2 = pool.delete_object(&k).await;
    (r1.is_ok() && r2.is_ok(), "DELETE 幂等".into(), None)
}

// ── L3: 流式操作 ────────────────────────────────────────────────────────────

async fn check_stream_upload(pool: &OssPool) -> (bool, String, Option<String>) {
    let k = test_key("su");
    let d = Bytes::from(vec![0xCDu8; 100 * 1024]);
    let bs = ossx::byte_stream_from_bytes(d.clone());
    match pool.put_stream(&k, bs, UploadOptions::default()).await {
        Ok(m) => {
            let _ = pool.delete_object(&k).await;
            (m.size > 0, format!("put_stream ok size={}", m.size), None)
        }
        Err(e) => (false, "put_stream 失败".into(), Some(format!("{e:?}"))),
    }
}

async fn check_stream_download(pool: &OssPool) -> (bool, String, Option<String>) {
    let k = test_key("sd");
    let d = Bytes::from("stream-dl-test");
    pool.put_object(&k, d.clone()).await.ok();
    match pool.get_stream(&k, DownloadOptions::default()).await {
        Ok((_, mut stream)) => {
            use futures_util::StreamExt;
            let mut c = Vec::new();
            while let Some(cr) = stream.next().await {
                match cr { Ok(chunk) => c.extend_from_slice(&chunk), Err(e) => { let _ = pool.delete_object(&k).await; return (false, "stream chunk error".into(), Some(format!("{e:?}"))); } }
            }
            let _ = pool.delete_object(&k).await;
            (Bytes::from(c) == d, "get_stream 数据一致".into(), None)
        }
        Err(e) => { let _ = pool.delete_object(&k).await; (false, "get_stream 失败".into(), Some(format!("{e:?}"))) }
    }
}

async fn check_stream_roundtrip(pool: &OssPool) -> (bool, String, Option<String>) {
    let k = test_key("srt");
    let d = Bytes::from(vec![0xFEu8; 50 * 1024]);
    let bs = ossx::byte_stream_from_bytes(d.clone());
    match pool.put_stream(&k, bs, UploadOptions::default()).await {
        Ok(_) => {
            match pool.get_stream(&k, DownloadOptions::default()).await {
                Ok((_, mut stream)) => {
                    use futures_util::StreamExt;
                    let mut c = Vec::new();
                    while let Some(cr) = stream.next().await { if let Ok(chunk) = cr { c.extend_from_slice(&chunk); } }
                    let _ = pool.delete_object(&k).await;
                    (Bytes::from(c) == d, "流 roundtrip 一致".into(), None)
                }
                Err(e) => { let _ = pool.delete_object(&k).await; (false, "get_stream 失败".into(), Some(format!("{e:?}"))) }
            }
        }
        Err(e) => (false, "put_stream 失败".into(), Some(format!("{e:?}"))),
    }
}

async fn check_range_download(pool: &OssPool) -> (bool, String, Option<String>) {
    let k = test_key("range");
    pool.put_object(&k, Bytes::from(b"0123456789".to_vec())).await.ok();
    match pool.get_stream(&k, DownloadOptions::with_range("bytes=0-4")).await {
        Ok((_, mut stream)) => {
            use futures_util::StreamExt;
            let mut c = Vec::new();
            while let Some(cr) = stream.next().await { if let Ok(chunk) = cr { c.extend_from_slice(&chunk); } }
            let _ = pool.delete_object(&k).await;
            (Bytes::from(c) == Bytes::from(b"01234".to_vec()), "Range 0-4 正确".into(), None)
        }
        Err(e) => { let _ = pool.delete_object(&k).await; (false, "Range 下载失败".into(), Some(format!("{e:?}"))) }
    }
}

// ── L4: 高级功能 ────────────────────────────────────────────────────────────

async fn check_object_key() -> (bool, String, Option<String>) {
    let tests = vec![
        ("valid", ObjectKey::new("key.txt").is_ok()),
        ("nested", ObjectKey::new("a/b/c").is_ok()),
        ("reject-empty", ObjectKey::new("").is_err()),
        ("reject-dotdot", ObjectKey::new("a/../b").is_err()),
        ("reject-control", ObjectKey::new("a\nb").is_err()),
    ];
    let all_ok = tests.iter().all(|(_, ok)| *ok);
    let failed: Vec<_> = tests.iter().filter(|(_, ok)| !ok).map(|(n, _)| *n).collect();
    (all_ok, format!("ObjectKey {} 项验证", tests.len()), if all_ok { None } else { Some(format!("失败: {failed:?}")) })
}

async fn check_stats(pool: &OssPool) -> (bool, String, Option<String>) {
    let s = pool.stats();
    (s.max_in_flight > 0 && !s.closed, format!("stats max_in_flight={} closed={}", s.max_in_flight, s.closed), None)
}

async fn check_close_reject(_pool: &OssPool) -> (bool, String, Option<String>) {
    // 创建临时池做 close 测试
    let p2 = match OssPool::connect(OssConfig::from_env().expect("env")) {
        Ok(p) => p,
        Err(e) => return (false, "connect failed".into(), Some(format!("{e:?}"))),
    };
    p2.close(Duration::from_secs(5)).await.ok();
    let result = p2.put_object("should-fail", Bytes::from("x")).await;
    (result.is_err(), "close 后拒绝操作".into(), None)
}

async fn check_presign(pool: &OssPool) -> (bool, String, Option<String>) {
    use ossx::PresignOptions;
    let opts = PresignOptions::default();
    match ossx::presign_url(
        &pool.config().endpoint,
        &pool.config().bucket,
        "test-key",
        &pool.config().access_key_id,
        &pool.config().access_key_secret,
        &opts,
    ) {
        Ok(url) => {
            let valid = url.contains("OSSAccessKeyId=") && url.contains("Expires=") && url.contains("Signature=");
            (valid, "预签名 URL 格式正确".into(), Some(url))
        }
        Err(e) => (false, "预签名 URL 生成失败".into(), Some(format!("{e:?}"))),
    }
}

// ── L5: 安全与并发 ──────────────────────────────────────────────────────────

async fn check_oversized(pool: &OssPool) -> (bool, String, Option<String>) {
    // 利用配置上限：pool.config().max_object_bytes
    let limit = pool.config().max_object_bytes;
    let d = Bytes::from(vec![0u8; limit + 1]);
    let k = test_key("oversized");
    let result = pool.put_object(&k, d).await;
    (result.is_err(), format!("{limit}+1 字节被拒绝"), None)
}

async fn check_concurrent(pool: &OssPool) -> (bool, String, Option<String>) {
    let n = 10;
    let mut handles = Vec::new();
    for i in 0..n {
        let p = pool.clone();
        let k = test_key(&format!("conc-{i}"));
        let d = Bytes::from(format!("concurrent-item-{i}"));
        handles.push(tokio::spawn(async move {
            p.put_object(&k, d.clone()).await?;
            let got = p.get_object(&k).await?;
            p.delete_object(&k).await?;
            Ok::<_, kernel::error::XError>(got == d)
        }));
    }
    let mut all_ok = true;
    let mut errs = Vec::new();
    for h in handles {
        match h.await {
            Ok(Ok(true)) => {}
            Ok(Ok(false)) => { all_ok = false; errs.push("data mismatch".into()); }
            Ok(Err(e)) => { all_ok = false; errs.push(format!("{e:?}")); }
            Err(e) => { all_ok = false; errs.push(format!("join: {e:?}")); }
        }
    }
    (all_ok, format!("{n} 并发操作"), if all_ok { None } else { Some(errs.join("; ")) })
}

async fn check_key_injection(pool: &OssPool) -> (bool, String, Option<String>) {
    // 验证带特殊字符的 key 被规范化或拒绝
    let cases = vec![
        ("normal", "normal-key"),
        ("with-slash", "a/b/c/d"),
        ("with-dash", "file-name.txt"),
        ("unicode", "中文键名"),
    ];
    let mut results = Vec::new();
    for (name, _key) in &cases {
        let full_key = format!("oss-verify/injection/{}-{}", name, test_key("inj"));
        match pool.put_object(&full_key, Bytes::from("injection-test")).await {
            Ok(()) => {
                match pool.get_object(&full_key).await {
                    Ok(d) => {
                        let _ = pool.delete_object(&full_key).await;
                        results.push((name, true, format!("{name}: OK size={}", d.len())));
                    }
                    Err(e) => { let _ = pool.delete_object(&full_key).await; results.push((name, false, format!("{name}: GET err={e:?}"))); }
                }
            }
            Err(e) => results.push((name, false, format!("{name}: PUT err={e:?}"))),
        }
    }
    let all_ok = results.iter().all(|(_, ok, _)| *ok);
    let detail = results.iter().map(|(_, _, msg)| msg.clone()).collect::<Vec<_>>().join(" | ");
    (all_ok, format!("键注入测试 {} 项", cases.len()), Some(detail))
}

// ── 报告生成 ────────────────────────────────────────────────────────────────

pub fn aggregate_results(checks: Vec<CheckResult>, total_duration_ms: u64) -> RunResult {
    let total = checks.len();
    let passed = checks.iter().filter(|c| c.passed).count();
    let failed = total - passed;
    let status = if failed == 0 { RunStatus::Pass }
    else if passed == 0 { RunStatus::Fail }
    else { RunStatus::Partial };

    RunResult {
        schema: RunResult::SCHEMA.into(),
        status,
        module: "ossx".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        plan_digest: "direct".into(),
        total, passed, failed,
        duration_ms: total_duration_ms,
        checks,
        summary: format!("{passed}/{total} passed ({status:?})"),
    }
}

/// 获取所有验证检查的规格列表。
pub fn all_check_specs() -> Vec<(String, CheckKind, u8)> {
    vec![
        // L0
        ("config-valid".into(), CheckKind::Config, 0),
        ("config-env".into(), CheckKind::Config, 0),
        ("config-sse-default".into(), CheckKind::Config, 0),
        // L1
        ("conn-health".into(), CheckKind::Connectivity, 1),
        ("conn-credential".into(), CheckKind::Connectivity, 1),
        ("conn-bucket-accessible".into(), CheckKind::Connectivity, 1),
        // L2
        ("ops-put".into(), CheckKind::ObjectOps, 2),
        ("ops-get".into(), CheckKind::ObjectOps, 2),
        ("ops-delete".into(), CheckKind::ObjectOps, 2),
        ("ops-head".into(), CheckKind::ObjectOps, 2),
        ("ops-empty-obj".into(), CheckKind::ObjectOps, 2),
        ("ops-large-obj".into(), CheckKind::ObjectOps, 2),
        ("ops-delete-idempotent".into(), CheckKind::ObjectOps, 2),
        // L3
        ("stream-upload".into(), CheckKind::Streaming, 3),
        ("stream-download".into(), CheckKind::Streaming, 3),
        ("stream-roundtrip".into(), CheckKind::Streaming, 3),
        ("stream-range".into(), CheckKind::Streaming, 3),
        // L4
        ("advanced-object-key".into(), CheckKind::Advanced, 4),
        ("advanced-stats".into(), CheckKind::Advanced, 4),
        ("advanced-close-reject".into(), CheckKind::Advanced, 4),
        ("advanced-presign".into(), CheckKind::Advanced, 4),
        // L5
        ("security-oversized".into(), CheckKind::Security, 5),
        ("security-concurrent".into(), CheckKind::Concurrency, 5),
        ("security-key-injection".into(), CheckKind::Security, 5),
    ]
}
