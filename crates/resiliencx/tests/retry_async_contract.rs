//! async 重试合同：驱动 shipped [`retry_async`] + [`AsyncWait`]。

use kernel::{ErrorKind, XError};
use resiliencx::{
    AsyncWait, Backoff, NoWait, NoopInstrumentation, RecordingWait, RetryConfig, retry_async,
    retry_downcast, retry_ok,
};

#[tokio::test]
async fn retry_async_succeeds_after_transient_with_recording_wait() {
    let cfg = RetryConfig {
        max_attempts: 4,
        base_delay_ms: 7,
        backoff: Backoff::Constant,
        jitter_bps: 0,
    };
    let instr = NoopInstrumentation;
    let wait = RecordingWait::new();
    let mut n = 0u32;
    let boxed = retry_async(&cfg, &instr, "async.op", &wait, || {
        n += 1;
        let attempt = n;
        async move {
            if attempt < 3 {
                return Err(XError::transient(format!("t{attempt}")));
            }
            Ok(retry_ok(attempt))
        }
    })
    .await
    .expect("ok");
    let got: u32 = retry_downcast(boxed).unwrap();
    assert_eq!(got, 3);
    assert_eq!(wait.delays(), vec![7, 7]);
}

#[tokio::test]
async fn retry_async_non_retryable_stops_immediately() {
    let cfg = RetryConfig::fixed(5, 10);
    let instr = NoopInstrumentation;
    let wait = RecordingWait::new();
    let err = retry_async(&cfg, &instr, "op", &wait, || async { Err(XError::invalid("nope")) })
        .await
        .expect_err("invalid");
    assert_eq!(err.kind(), ErrorKind::Invalid);
    assert!(wait.delays().is_empty());
}

#[tokio::test]
async fn retry_async_zero_attempts_invalid() {
    let cfg = RetryConfig { max_attempts: 0, ..RetryConfig::default() };
    let err =
        retry_async(&cfg, &NoopInstrumentation, "op", &NoWait, || async { Ok(retry_ok(1u8)) })
            .await
            .expect_err("zero");
    assert_eq!(err.kind(), ErrorKind::Invalid);
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn retry_async_tokio_sleep_wait_short_delay() {
    use resiliencx::TokioSleepWait;
    let cfg = RetryConfig {
        max_attempts: 2,
        base_delay_ms: 1,
        backoff: Backoff::Constant,
        jitter_bps: 0,
    };
    let mut n = 0u32;
    let boxed = retry_async(&cfg, &NoopInstrumentation, "tokio.op", &TokioSleepWait, || {
        n += 1;
        let attempt = n;
        async move {
            if attempt == 1 {
                return Err(XError::transient("once"));
            }
            Ok(retry_ok(attempt))
        }
    })
    .await
    .expect("ok");
    assert_eq!(retry_downcast::<u32>(boxed).unwrap(), 2);
}

#[tokio::test]
async fn async_wait_nowait_is_noop() {
    // 对象安全：&dyn AsyncWait
    let w: &dyn AsyncWait = &NoWait;
    w.wait_ms(1000).await;
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn tokio_sleep_wait_zero_is_noop_and_debug() {
    use resiliencx::TokioSleepWait;
    let w = TokioSleepWait;
    let _ = format!("{w:?}");
    AsyncWait::wait_ms(&w, 0).await;
}
