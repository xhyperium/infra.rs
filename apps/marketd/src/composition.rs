use async_trait::async_trait;
use kernel::{ShutdownGuard, ShutdownSignal};
use market_data::{
    ReplaySource,
    checkpoint::FileCheckpoint,
    model::{MarketEvent, SourceEvent},
    pipeline::{Delivery, DeliveryError, DeliveryPipeline},
    ports::{EventSink, MarketDataSource, PortError},
    quality::Quality,
};
use std::{collections::BTreeSet, path::PathBuf};
#[cfg(unix)]
use std::{future::Future, io};
use tokio::sync::watch;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureResult {
    pub deliveries: Vec<Delivery>,
    pub committed: u64,
    pub quality: Quality,
    pub emitted_sequences: Vec<u64>,
}

#[derive(Debug, Default)]
struct IdempotentSink {
    seen: BTreeSet<u64>,
    events: Vec<MarketEvent>,
}

#[async_trait]
impl EventSink for IdempotentSink {
    async fn emit(&mut self, event: &MarketEvent) -> Result<(), PortError> {
        if self.seen.insert(event.sequence) {
            self.events.push(event.clone());
        }
        Ok(())
    }
}

fn fixture_events() -> Vec<SourceEvent> {
    let event = |sequence| {
        SourceEvent::Data(MarketEvent {
            sequence,
            symbol: "BTCUSDT".into(),
            price: 100,
            size: 1,
            timestamp_ms: sequence as i64,
        })
    };
    vec![
        SourceEvent::Resubscribed { gap_possible: true },
        event(1),
        event(1),
        event(3),
        event(3),
        event(4),
    ]
}

/// Run the finite fixture with a caller-owned shutdown channel.
///
/// This proves local composition and checkpoint behavior only; provider
/// testnet, long-running replay, and real sink evidence remain external.
pub async fn run_fixture_with_shutdown(
    path: impl Into<PathBuf>,
    shutdown: &mut watch::Receiver<bool>,
) -> Result<FixtureResult, DeliveryError> {
    let source = ReplaySource::new(fixture_events());
    let pipeline =
        DeliveryPipeline::new(FileCheckpoint::new(path), IdempotentSink::default()).await?;
    run_replay(source, pipeline, shutdown).await
}

async fn run_replay(
    source: ReplaySource,
    mut pipeline: DeliveryPipeline<FileCheckpoint, IdempotentSink>,
    shutdown: &mut watch::Receiver<bool>,
) -> Result<FixtureResult, DeliveryError> {
    let deliveries = run_until_shutdown(source, &mut pipeline, shutdown).await?;
    let quality = pipeline.quality();
    let committed = pipeline.committed();
    let (_, sink) = pipeline.into_parts();
    Ok(FixtureResult {
        deliveries,
        committed,
        quality,
        emitted_sequences: sink.events.into_iter().map(|event| event.sequence).collect(),
    })
}

/// 将 OS 关停事件同时写入 async `watch` 通道与 kernel 生命周期信号。
///
/// - `watch`：供 `run_until_shutdown` 的 `tokio::select!` 使用（非阻塞）
/// - `ShutdownGuard::trigger`：通知 kernel 侧观察者（`is_triggered` / 阻塞 `wait`）
///
/// **禁止**在 async 运行时线程上调用 [`ShutdownSignal::wait`]（Condvar 阻塞）。
#[cfg(unix)]
async fn forward_shutdown_signal<F>(
    signal: F,
    sender: watch::Sender<bool>,
    guard: ShutdownGuard,
) -> io::Result<()>
where
    F: Future<Output = io::Result<()>>,
{
    signal.await?;
    // 先触发 kernel 生命周期，再唤醒 async 组合循环。
    guard.trigger();
    sender
        .send(true)
        .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "shutdown receiver dropped"))
}

/// Convert Unix `SIGTERM` or `SIGINT` into the same watch signal used by the
/// composition loop, and trigger the paired [`ShutdownGuard`].
/// Provider and sink work remains outside this adapter.
#[cfg(unix)]
pub async fn wait_for_shutdown_signal(
    sender: watch::Sender<bool>,
    guard: ShutdownGuard,
) -> io::Result<()> {
    use tokio::signal::unix::{SignalKind, signal};

    let mut terminate = signal(SignalKind::terminate())?;
    let mut interrupt = signal(SignalKind::interrupt())?;
    forward_shutdown_signal(
        async {
            tokio::select! {
                _ = terminate.recv() => Ok(()),
                _ = interrupt.recv() => Ok(()),
            }
        },
        sender,
        guard,
    )
    .await
}

/// Drive a source until it is exhausted or a shutdown signal is observed.
/// The current event is always committed before the next source read, so a
/// shutdown between reads leaves a restartable checkpoint.
pub async fn run_until_shutdown<R, C, S>(
    mut source: R,
    pipeline: &mut DeliveryPipeline<C, S>,
    shutdown: &mut watch::Receiver<bool>,
) -> Result<Vec<Delivery>, DeliveryError>
where
    R: MarketDataSource,
    C: market_data::ports::CheckpointStore,
    S: EventSink,
{
    if *shutdown.borrow() {
        return Ok(Vec::new());
    }
    let mut deliveries = Vec::new();
    loop {
        tokio::select! {
            changed = shutdown.changed() => {
                if changed.is_err() || *shutdown.borrow() {
                    break;
                }
            }
            event = source.next() => {
                match event? {
                    Some(event) => deliveries.push(pipeline.process(event).await?),
                    None => break,
                }
            }
        }
    }
    Ok(deliveries)
}

pub async fn run() -> Result<(), DeliveryError> {
    let path = std::env::var_os("MARKETD_CHECKPOINT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("marketd.checkpoint"));
    // kernel 生命周期：组合根持有 guard；观察端可 clone 给注入方。
    // async 循环仍走 watch；OS 信号时同时 trigger guard（见 wait_for_shutdown_signal）。
    let (kernel_guard, kernel_signal) = ShutdownSignal::new();
    let (shutdown_sender, mut shutdown) = watch::channel(false);
    #[cfg(unix)]
    let signal_task = tokio::spawn(wait_for_shutdown_signal(shutdown_sender, kernel_guard));
    #[cfg(not(unix))]
    {
        let _shutdown_sender = shutdown_sender;
        let _kernel_guard = kernel_guard;
    }
    let result = run_fixture_with_shutdown(path, &mut shutdown).await;
    #[cfg(unix)]
    signal_task.abort();
    let result = result?;
    // 自然观察点：fixture 正常耗尽时通常未触发；OS 信号路径会置位。
    let _shutdown_via_kernel = kernel_signal.is_triggered();
    println!(
        "marketd fixture committed={} emitted={:?} quality={:?} kernel_shutdown={}",
        result.committed, result.emitted_sequences, result.quality, _shutdown_via_kernel
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use market_data::checkpoint::MemoryCheckpoint;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    #[tokio::test]
    async fn fixture_replay_reports_gap_and_idempotent_sequences() {
        let directory = tempfile::tempdir().unwrap();
        let (_sender, mut shutdown) = watch::channel(false);
        let result = run_fixture_with_shutdown(directory.path().join("checkpoint"), &mut shutdown)
            .await
            .unwrap();
        assert_eq!(result.committed, 4);
        assert_eq!(result.emitted_sequences, vec![1, 3, 4]);
        assert_eq!(result.quality.reconnects, 1);
        assert_eq!(result.quality.duplicates, 2);
        assert_eq!(result.quality.gaps, 1);
        assert_eq!(result.quality.dropped, 1);
    }

    #[tokio::test]
    async fn restart_reopens_checkpoint_without_reemitting_fixture_events() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("checkpoint");

        let (_sender, mut shutdown) = watch::channel(false);
        let first = run_fixture_with_shutdown(&path, &mut shutdown).await.unwrap();
        assert_eq!(first.committed, 4);
        assert_eq!(first.emitted_sequences, vec![1, 3, 4]);

        let (_sender, mut shutdown) = watch::channel(false);
        let restarted = run_fixture_with_shutdown(path, &mut shutdown).await.unwrap();
        assert_eq!(restarted.committed, 4);
        assert!(restarted.emitted_sequences.is_empty());
        assert_eq!(restarted.quality.duplicates, 5);
    }

    struct OneThenPending {
        event: Option<SourceEvent>,
        reads: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl MarketDataSource for OneThenPending {
        async fn next(&mut self) -> Result<Option<SourceEvent>, PortError> {
            match self.event.take() {
                Some(event) => {
                    self.reads.fetch_add(1, Ordering::SeqCst);
                    Ok(Some(event))
                }
                None => std::future::pending().await,
            }
        }
    }

    struct SignalAfterEmit {
        sender: watch::Sender<bool>,
    }

    #[async_trait]
    impl EventSink for SignalAfterEmit {
        async fn emit(&mut self, _: &MarketEvent) -> Result<(), PortError> {
            self.sender
                .send(true)
                .map_err(|_| PortError::Unavailable("shutdown receiver dropped".into()))
        }
    }

    fn event(sequence: u64) -> SourceEvent {
        SourceEvent::Data(MarketEvent {
            sequence,
            symbol: "BTCUSDT".into(),
            price: 100,
            size: 1,
            timestamp_ms: sequence as i64,
        })
    }

    #[tokio::test]
    async fn shutdown_after_emit_keeps_checkpoint_before_next_read() {
        let reads = Arc::new(AtomicUsize::new(0));
        let source = OneThenPending { event: Some(event(1)), reads: Arc::clone(&reads) };
        let (sender, mut shutdown) = watch::channel(false);
        let mut pipeline =
            DeliveryPipeline::new(MemoryCheckpoint::default(), SignalAfterEmit { sender })
                .await
                .unwrap();

        let deliveries = run_until_shutdown(source, &mut pipeline, &mut shutdown).await.unwrap();
        assert_eq!(deliveries, vec![Delivery::Emitted(1)]);
        assert_eq!(pipeline.committed(), 1);
        assert_eq!(reads.load(Ordering::SeqCst), 1);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn signal_forwarder_updates_watch_channel() {
        let (guard, signal) = ShutdownSignal::new();
        let (sender, receiver) = watch::channel(false);
        assert!(!signal.is_triggered());
        forward_shutdown_signal(async { Ok(()) }, sender, guard).await.unwrap();
        assert!(*receiver.borrow());
        assert!(signal.is_triggered(), "composition path must trigger kernel ShutdownGuard");
    }

    /// 组合路径契约：`ShutdownGuard::trigger` 使配对 `ShutdownSignal::is_triggered` 为 true。
    /// 不调用阻塞的 `wait()`（async 运行时禁用）。
    #[test]
    fn kernel_shutdown_guard_sets_is_triggered() {
        let (guard, signal) = ShutdownSignal::new();
        let observed = signal.clone();
        assert!(!signal.is_triggered());
        assert!(!observed.is_triggered());
        guard.trigger();
        assert!(signal.is_triggered());
        assert!(observed.is_triggered());
    }
}
