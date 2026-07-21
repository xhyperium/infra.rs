//! 全系统统一的时间获取、表示与比较语义。
//!
//! # 设计原则
//!
//! 墙钟和单调钟是不同语义，分别对应不同使用场景：
//!
//! - **墙钟**（[`Timestamp`] / [`Clock::now`]）：面向业务的绝对时间点，允许因 NTP
//!   或人工校时回退，不承诺非递减。
//! - **单调钟**（[`MonotonicInstant`] / [`Clock::monotonic`]）：仅用于测量间隔，
//!   绝对值无业务意义，不可持久化，不可跨进程比较。
//! - **domain**：单调采样点带 [`ClockDomain`]。同进程所有 [`SystemClock`] 共享
//!   进程级 domain 与原点；每个 `ManualClock`（testkit）有独立 domain。
//!   跨 domain 的 [`MonotonicInstant::checked_duration_since`] 返回 `None`，
//!   不得当作可靠间隔。
//!
//! 获取失败必须显式返回错误，禁止返回零值时间戳哨兵。

use std::time::Duration;

// ---------------------------------------------------------------------------
// Timestamp
// ---------------------------------------------------------------------------

/// Unix epoch 纳秒精度时间戳。
///
/// 内部以 `i64` 纳秒表示，可表示 epoch 前后的时间点。没有 [`Default`] 实现，
/// 也不提供人类时间格式化或 serde 支持——这些职责属于上层协议层。
///
/// # 不变式
///
/// - 墙钟允许回退，调用方不得假设非递减。
/// - `checked_*` 运算覆盖完整 `i64` 纳秒域。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp(i64);

impl Timestamp {
    /// 从纳秒值构造时间戳。仅用于 protocol 转换、fixture 和 testkit。
    pub const fn from_unix_nanos(nanos: i64) -> Self {
        Self(nanos)
    }

    /// 返回内部纳秒值。
    pub const fn as_unix_nanos(self) -> i64 {
        self.0
    }

    /// 安全加法，溢出时返回 `None`。
    ///
    /// 使用宽于 `i64` 的中间值避免误报溢出。
    pub fn checked_add(self, duration: Duration) -> Option<Self> {
        let nanos = i128::from(self.0);
        let dur_nanos: i128 = duration.as_nanos().try_into().ok()?;
        let result = nanos.checked_add(dur_nanos)?;
        let result_i64: i64 = result.try_into().ok()?;
        Some(Self(result_i64))
    }

    /// 安全减法，溢出时返回 `None`。
    ///
    /// 与 [`Timestamp::checked_add`] 一致：经 `i128` 中间值，避免 `std::time::Duration`
    /// 合法域内不可达的 `u128 → i128` 死分支。
    pub fn checked_sub(self, duration: Duration) -> Option<Self> {
        let nanos = i128::from(self.0);
        let dur_nanos: i128 = duration.as_nanos().try_into().ok()?;
        let result = nanos.checked_sub(dur_nanos)?;
        let result_i64: i64 = result.try_into().ok()?;
        Some(Self(result_i64))
    }

    /// 返回 `self` 和 `earlier` 之间的时间差。
    ///
    /// 若 `earlier > self` 则返回 `None`，禁止饱和为 `Duration::ZERO`。
    /// 使用宽于 `i64` 的中间值，使 `i64::MAX - i64::MIN`（= `u64::MAX` 纳秒）可表示。
    pub fn checked_duration_since(self, earlier: Self) -> Option<Duration> {
        if self.0 < earlier.0 {
            return None;
        }
        let diff = i128::from(self.0) - i128::from(earlier.0);
        // 非负且最大为 u64::MAX（i64 全量程跨度）
        let nanos_u64: u64 = diff.try_into().ok()?;
        Some(Duration::from_nanos(nanos_u64))
    }
}

// ---------------------------------------------------------------------------
// MonotonicInstant
// ---------------------------------------------------------------------------

/// 单调时钟 domain 标识。
///
/// 仅同一 domain 内的 [`MonotonicInstant`] 可比较间隔；跨 domain 比较返回 `None`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClockDomain(u64);

impl ClockDomain {
    /// 进程级系统单调域（所有 [`SystemClock`] 共享）。
    pub const PROCESS: Self = Self(1);

    /// 构造测试/自定义 domain（`id == 0` 或 `1` 保留给进程域时仍可用，但勿与
    /// [`Self::PROCESS`] 混用语义）。
    /// 测试/仿真入口。生产路径请使用 [`ClockDomain::PROCESS`]（infra-s9t.15）。
    pub const fn from_raw(id: u64) -> Self {
        Self(id)
    }

    /// 原始 ID。
    pub const fn as_raw(self) -> u64 {
        self.0
    }
}

/// 单调时钟采样点，仅用于测量时间间隔。
///
/// 封装为不透明类型，不暴露底层 ticks。不可持久化，不可跨进程比较，绝对值无
/// 业务意义。比较间隔前必须同 [`ClockDomain`]。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MonotonicInstant {
    elapsed: Duration,
    domain: ClockDomain,
}

impl PartialOrd for MonotonicInstant {
    /// 仅同 domain 可比较；跨 domain 返回 `None`（不可静默当可靠序）。
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.domain != other.domain {
            return None;
        }
        Some(self.elapsed.cmp(&other.elapsed))
    }
}

impl MonotonicInstant {
    /// 返回采样点所属 domain。
    pub const fn domain(self) -> ClockDomain {
        self.domain
    }

    /// 返回 `self` 和 `earlier` 之间的持续时间差。
    ///
    /// - 跨 domain：返回 `None`（不可静默当作可靠结果）
    /// - `earlier` 晚于 `self`：返回 `None`，禁止饱和为零
    pub fn checked_duration_since(self, earlier: Self) -> Option<Duration> {
        if self.domain != earlier.domain {
            return None;
        }
        self.elapsed.checked_sub(earlier.elapsed)
    }

    /// 从已流逝的时间构造 `MonotonicInstant`（默认进程 domain）。
    ///
    /// 仅供 [`Clock`] 实现和 `testkit::ManualClock` 使用。
    /// 调用位置约定：仅 `crates/kernel/src/clock.rs` 与 `crates/testkit/*`
    ///（本仓 **不** 用 archgate 机控；见 SSOT TIME-004 OOS / #164）。
    #[doc(hidden)]
    pub const fn from_clock_elapsed(elapsed: Duration) -> Self {
        Self { elapsed, domain: ClockDomain::PROCESS }
    }

    /// 从已流逝时间 + 显式 domain 构造。
    ///
    /// 仅供 [`Clock`] 实现和 `testkit::ManualClock` 使用。
    #[doc(hidden)]
    pub const fn from_clock_elapsed_in(elapsed: Duration, domain: ClockDomain) -> Self {
        Self { elapsed, domain }
    }
}

// ---------------------------------------------------------------------------
// ClockError
// ---------------------------------------------------------------------------

/// 时间源获取失败的错误。
#[non_exhaustive]
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ClockError {
    /// 系统时间早于 Unix epoch。
    #[error("系统时钟早于 Unix epoch")]
    BeforeUnixEpoch,

    /// 时钟值超出可表示的纳秒范围。
    #[error("时钟值超出可表示的纳秒范围")]
    Overflow,

    /// 时间源不可用。
    #[error("时间源不可用")]
    Unavailable,
}

// ---------------------------------------------------------------------------
// Clock trait
// ---------------------------------------------------------------------------

/// 时间源抽象 trait。
///
/// 所有时间源必须显式注入，禁止隐式全局 `Clock`。每个实现必须同时提供墙钟
/// 和单调钟；`monotonic` 不得有默认实现。
pub trait Clock: Send + Sync {
    /// 获取当前墙钟时间。允许因 NTP 或人工校时回退。
    ///
    /// # 错误
    ///
    /// 返回 [`ClockError`] 当时间源不可用、早于 epoch、或值超出可表示范围。
    fn now(&self) -> Result<Timestamp, ClockError>;

    /// 获取单调时钟采样点，仅用于测量间隔。
    fn monotonic(&self) -> MonotonicInstant;
}

// ---------------------------------------------------------------------------
// SystemClock
// ---------------------------------------------------------------------------

/// 基于 `std::time` 的真实系统时钟实现。
///
/// 这是生产环境中唯一的时钟实现。测试应使用 `testkit::ManualClock`。
/// 所有实例共享进程级单调原点与 [`ClockDomain::PROCESS`]。
#[derive(Debug, Clone)]
pub struct SystemClock;

impl SystemClock {
    /// 创建一个新的 `SystemClock`（共享进程单调原点）。
    pub fn new() -> Self {
        // 预热进程原点，避免首采样抖动影响 domain 语义说明
        let _ = process_monotonic_origin();
        Self
    }
}

impl Default for SystemClock {
    fn default() -> Self {
        Self::new()
    }
}

fn process_monotonic_origin() -> std::time::Instant {
    use std::sync::OnceLock;
    static ORIGIN: OnceLock<std::time::Instant> = OnceLock::new();
    *ORIGIN.get_or_init(std::time::Instant::now)
}

/// 将 epoch 起的 `Duration` 转为 [`Timestamp`]（纯函数，便于测 Overflow）。
fn timestamp_from_unix_duration(d: Duration) -> Result<Timestamp, ClockError> {
    let nanos: u128 = d.as_nanos();
    let nanos_i64: i64 = nanos.try_into().map_err(|_| ClockError::Overflow)?;
    Ok(Timestamp::from_unix_nanos(nanos_i64))
}

/// 将 `std::time::SystemTime` 转为 [`Timestamp`]（纯函数，便于测 BeforeUnixEpoch / Overflow）。
fn timestamp_from_system_time(ts: std::time::SystemTime) -> Result<Timestamp, ClockError> {
    use std::time::UNIX_EPOCH;
    match ts.duration_since(UNIX_EPOCH) {
        Ok(d) => timestamp_from_unix_duration(d),
        Err(_) => Err(ClockError::BeforeUnixEpoch),
    }
}

impl Clock for SystemClock {
    fn now(&self) -> Result<Timestamp, ClockError> {
        timestamp_from_system_time(std::time::SystemTime::now())
    }

    fn monotonic(&self) -> MonotonicInstant {
        MonotonicInstant::from_clock_elapsed_in(
            process_monotonic_origin().elapsed(),
            ClockDomain::PROCESS,
        )
    }
}

// ---------------------------------------------------------------------------
// 单元测试
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Timestamp 算术 ---------------------------------------------------

    #[test]
    fn test_timestamp_i64_bounds() {
        let min = Timestamp::from_unix_nanos(i64::MIN);
        let max = Timestamp::from_unix_nanos(i64::MAX);
        assert_eq!(min.as_unix_nanos(), i64::MIN);
        assert_eq!(max.as_unix_nanos(), i64::MAX);
    }

    #[test]
    fn test_timestamp_checked_add_normal() {
        let t = Timestamp::from_unix_nanos(100);
        let result = t.checked_add(Duration::from_nanos(50)).unwrap();
        assert_eq!(result.as_unix_nanos(), 150);
    }

    #[test]
    fn test_timestamp_checked_add_overflow() {
        let t = Timestamp::from_unix_nanos(i64::MAX);
        assert!(t.checked_add(Duration::from_nanos(1)).is_none());
    }

    #[test]
    fn test_timestamp_checked_add_negative_to_positive() {
        let t = Timestamp::from_unix_nanos(-100);
        let result = t.checked_add(Duration::from_nanos(200)).unwrap();
        assert_eq!(result.as_unix_nanos(), 100);
    }

    #[test]
    fn test_timestamp_checked_add_huge_duration() {
        // 极大 Duration 应溢出为 None，不得 panic
        let t = Timestamp::from_unix_nanos(i64::MAX);
        let huge = Duration::from_secs(u64::MAX);
        assert!(t.checked_add(huge).is_none());
    }

    #[test]
    fn test_timestamp_checked_sub_normal() {
        let t = Timestamp::from_unix_nanos(100);
        let result = t.checked_sub(Duration::from_nanos(50)).unwrap();
        assert_eq!(result.as_unix_nanos(), 50);
    }

    #[test]
    fn test_timestamp_checked_sub_overflow() {
        let t = Timestamp::from_unix_nanos(i64::MIN);
        assert!(t.checked_sub(Duration::from_nanos(1)).is_none());
    }

    #[test]
    fn test_timestamp_checked_sub_saturates_at_i64_bounds() {
        // u64::MAX seconds (≈ 1.8e19 s ≈ 1.8e28 ns) < i128::MAX (≈ 1.7e38 ns)
        // 因此 Duration::MAX 无法触发 i128 溢出守卫。本测试验证实际可达的
        // checked_sub 路径：Duration 转换为 i64 后溢出 → 返回 None。
        let t = Timestamp::from_unix_nanos(42);
        let huge = Duration::from_secs(u64::MAX);
        assert!(t.checked_sub(huge).is_none());
    }

    #[test]
    fn test_timestamp_checked_duration_since_equal() {
        let t = Timestamp::from_unix_nanos(42);
        let d = t.checked_duration_since(t).unwrap();
        assert_eq!(d, Duration::ZERO);
    }

    #[test]
    fn test_timestamp_checked_duration_since_normal() {
        let later = Timestamp::from_unix_nanos(100);
        let earlier = Timestamp::from_unix_nanos(30);
        let d = later.checked_duration_since(earlier).unwrap();
        assert_eq!(d, Duration::from_nanos(70));
    }

    #[test]
    fn test_timestamp_checked_duration_since_reverse() {
        let earlier = Timestamp::from_unix_nanos(10);
        let later = Timestamp::from_unix_nanos(100);
        assert!(earlier.checked_duration_since(later).is_none());
    }

    #[test]
    fn test_timestamp_ordering() {
        let a = Timestamp::from_unix_nanos(1);
        let b = Timestamp::from_unix_nanos(2);
        let c = Timestamp::from_unix_nanos(1);
        assert!(a < b);
        assert!(a <= c);
        assert_eq!(a, c);
    }

    // -- MonotonicInstant -------------------------------------------------

    #[test]
    fn test_monotonic_duration_since_normal() {
        let a = MonotonicInstant::from_clock_elapsed(Duration::from_secs(2));
        let b = MonotonicInstant::from_clock_elapsed(Duration::from_secs(5));
        let d = b.checked_duration_since(a).unwrap();
        assert_eq!(d, Duration::from_secs(3));
    }

    #[test]
    fn test_monotonic_duration_since_reverse_returns_none() {
        let a = MonotonicInstant::from_clock_elapsed(Duration::from_secs(2));
        let b = MonotonicInstant::from_clock_elapsed(Duration::from_secs(5));
        // a 早于 b：b - a = 3s（合法）
        assert_eq!(b.checked_duration_since(a), Some(Duration::from_secs(3)));
        // a - b 为反向差，应返回 None
        assert!(a.checked_duration_since(b).is_none());
    }

    #[test]
    fn test_monotonic_ordering() {
        let a = MonotonicInstant::from_clock_elapsed(Duration::from_secs(1));
        let b = MonotonicInstant::from_clock_elapsed(Duration::from_secs(2));
        assert!(a < b);
    }

    // -- SystemClock ------------------------------------------------------

    #[test]
    fn test_system_clock_returns_timestamp() {
        let clock = SystemClock::new();
        let ts = clock.now().expect("墙钟应可用");
        // 时间戳应为正（远晚于 epoch）
        assert!(ts.as_unix_nanos() > 0);
    }

    #[test]
    fn test_system_clock_monotonic_non_decreasing() {
        let clock = SystemClock::new();
        let a = clock.monotonic();
        let b = clock.monotonic();
        assert!(b >= a);
    }

    #[test]
    fn test_system_clock_default() {
        let clock = SystemClock::new();
        let ts = clock.now().expect("wall clock should be available");
        assert!(ts.as_unix_nanos() > 0);
    }

    // -- Clock trait 无 monotonic 默认实现 --------------------------------
    // 此测试为编译时检查：如果 Clock trait 的 monotonic 有默认实现，
    // 下面这个实现不需要提供 monotonic 也能编译。它需要提供。
    //
    // 实际验证由 compile_fail 测试完成；此处只是 sanity。

    struct DummyClock;

    impl Clock for DummyClock {
        fn now(&self) -> Result<Timestamp, ClockError> {
            Ok(Timestamp::from_unix_nanos(0))
        }

        fn monotonic(&self) -> MonotonicInstant {
            MonotonicInstant::from_clock_elapsed(Duration::ZERO)
        }
    }

    #[test]
    fn test_dummy_clock() {
        let c = DummyClock;
        let ts = c.now().unwrap();
        assert_eq!(ts.as_unix_nanos(), 0);
        let mono = c.monotonic();
        assert_eq!(mono.checked_duration_since(mono), Some(Duration::ZERO));
    }

    /// SystemClock 墙钟映射：epoch 前 → BeforeUnixEpoch；超 i64 纳秒 → Overflow。
    #[test]
    fn system_time_mapping_before_epoch_and_overflow() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let before = UNIX_EPOCH.checked_sub(Duration::from_secs(1)).expect("epoch - 1s");
        assert_eq!(timestamp_from_system_time(before), Err(ClockError::BeforeUnixEpoch));

        // ~year 2286：纳秒 > i64::MAX → Overflow
        let far = Duration::from_secs(10_000_000_000);
        assert_eq!(timestamp_from_unix_duration(far), Err(ClockError::Overflow));

        let ok = timestamp_from_system_time(SystemTime::now()).expect("now after epoch");
        assert!(ok.as_unix_nanos() > 0);
    }

    // -- ControlledClock（§11.1 双通道测试替身；from_clock_elapsed 仅允许本文件） --

    use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};

    /// 可控双通道 Clock：墙钟可回退，单调通道独立推进。
    struct ControlledClock {
        wall_nanos: AtomicI64,
        monotonic_nanos: AtomicU64,
    }

    impl ControlledClock {
        fn new(wall_nanos: i64, monotonic_nanos: u64) -> Self {
            Self {
                wall_nanos: AtomicI64::new(wall_nanos),
                monotonic_nanos: AtomicU64::new(monotonic_nanos),
            }
        }

        fn set_wall(&self, wall_nanos: i64) {
            self.wall_nanos.store(wall_nanos, Ordering::Relaxed);
        }

        fn set_monotonic_nanos(&self, monotonic_nanos: u64) {
            self.monotonic_nanos.store(monotonic_nanos, Ordering::Relaxed);
        }
    }

    impl Clock for ControlledClock {
        fn now(&self) -> Result<Timestamp, ClockError> {
            Ok(Timestamp::from_unix_nanos(self.wall_nanos.load(Ordering::Relaxed)))
        }

        fn monotonic(&self) -> MonotonicInstant {
            // 允许：KERNEL-TIME-004 allowlist 含 crates/kernel/src/clock.rs
            MonotonicInstant::from_clock_elapsed(Duration::from_nanos(
                self.monotonic_nanos.load(Ordering::Relaxed),
            ))
        }
    }

    /// 墙钟回退不得牵连单调通道间隔语义。
    #[test]
    fn wall_clock_regression_does_not_regress_monotonic_time() {
        let c = ControlledClock::new(1_000, 10);
        let wall_before = c.now().unwrap();
        let mono_before = c.monotonic();

        c.set_wall(500);
        c.set_monotonic_nanos(15);

        let wall_after = c.now().unwrap();
        let mono_after = c.monotonic();
        assert!(wall_after < wall_before, "墙钟回退是合法状态变化");
        assert_eq!(mono_after.checked_duration_since(mono_before), Some(Duration::from_nanos(5)));
    }

    /// 单调 elapsed 反向差为 None（§11.1 / 属性测试同源，放在 allowlist 路径内）。
    #[test]
    fn mono_elapsed_reverse_matrix_samples() {
        for (ms_a, ms_b) in [(0u64, 1), (10, 100), (1_000_000, 1_000_001)] {
            let a = MonotonicInstant::from_clock_elapsed(Duration::from_millis(ms_a));
            let b = MonotonicInstant::from_clock_elapsed(Duration::from_millis(ms_b));
            assert!(a.checked_duration_since(b).is_none());
            assert_eq!(b.checked_duration_since(a), Some(Duration::from_millis(ms_b - ms_a)));
        }
    }
    #[test]
    fn system_clocks_share_process_domain() {
        let a = SystemClock::new();
        let b = SystemClock::new();
        let ia = a.monotonic();
        let ib = b.monotonic();
        assert_eq!(ia.domain(), ClockDomain::PROCESS);
        assert_eq!(ib.domain(), ClockDomain::PROCESS);
        let _ = ia.checked_duration_since(ia);
        let _ = ib.checked_duration_since(ia);
    }

    #[test]
    fn cross_domain_duration_is_none() {
        let a = MonotonicInstant::from_clock_elapsed_in(
            Duration::from_millis(10),
            ClockDomain::PROCESS,
        );
        let b = MonotonicInstant::from_clock_elapsed_in(
            Duration::from_millis(5),
            ClockDomain::from_raw(99),
        );
        assert!(a.checked_duration_since(b).is_none());
        assert!(b.checked_duration_since(a).is_none());
    }

    #[test]
    fn clock_error_display_is_chinese() {
        assert!(ClockError::BeforeUnixEpoch.to_string().contains("系统"));
        assert!(ClockError::Overflow.to_string().contains("纳秒"));
        assert!(ClockError::Unavailable.to_string().contains("不可用"));
    }

    #[test]
    fn clock_domain_raw_and_partial_ord() {
        let d = ClockDomain::from_raw(42);
        assert_eq!(d.as_raw(), 42);
        let a =
            MonotonicInstant::from_clock_elapsed_in(Duration::from_millis(1), ClockDomain::PROCESS);
        let b =
            MonotonicInstant::from_clock_elapsed_in(Duration::from_millis(2), ClockDomain::PROCESS);
        assert!(a < b);
        let c = MonotonicInstant::from_clock_elapsed_in(
            Duration::from_millis(1),
            ClockDomain::from_raw(7),
        );
        assert!(a.partial_cmp(&c).is_none());
        let _ = SystemClock::new();
        // 覆盖 Default 实现（非 unit-struct default lint 的 ::default() 调用）
        let _clk: SystemClock = Default::default();
        let _ = _clk.monotonic().domain();
    }
}
