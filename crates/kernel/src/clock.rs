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
    pub fn checked_sub(self, duration: Duration) -> Option<Self> {
        let nanos = i128::from(self.0);
        // duration.as_nanos() is u128, need to handle carefully for subtraction
        let dur_nanos: u128 = duration.as_nanos();
        if dur_nanos > i128::MAX as u128 {
            // duration is too large to represent as i128, subtraction would always overflow
            return None;
        }
        let dur_nanos_i128 = dur_nanos as i128;
        let result = nanos.checked_sub(dur_nanos_i128)?;
        let result_i64: i64 = result.try_into().ok()?;
        Some(Self(result_i64))
    }

    /// 返回 `self` 和 `earlier` 之间的时间差。
    ///
    /// 若 `earlier > self` 则返回 `None`，禁止饱和为 `Duration::ZERO`。
    pub fn checked_duration_since(self, earlier: Self) -> Option<Duration> {
        if self.0 < earlier.0 {
            return None;
        }
        let diff = self.0.checked_sub(earlier.0)?;
        // When self >= earlier, diff is non-negative and fits in u64
        // (i64::MAX nanoseconds is about 292 years, which fits)
        Some(Duration::from_nanos(diff as u64))
    }
}

// ---------------------------------------------------------------------------
// MonotonicInstant
// ---------------------------------------------------------------------------

/// 单调时钟采样点，仅用于测量时间间隔。
///
/// 封装为不透明类型，不暴露底层 ticks。不可持久化，不可跨进程比较，绝对值无
/// 业务意义。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MonotonicInstant(Duration);

impl MonotonicInstant {
    /// 返回 `self` 和 `earlier` 之间的持续时间差。
    ///
    /// 若 `earlier` 晚于 `self`（反向比较）则返回 `None`，禁止饱和为零。
    pub fn checked_duration_since(self, earlier: Self) -> Option<Duration> {
        self.0.checked_sub(earlier.0)
    }

    /// 从已流逝的时间构造 `MonotonicInstant`。
    ///
    /// 仅供 [`Clock`] 实现和 `testkit::ManualClock` 使用。`archgate` 限制
    /// 其调用位置只能出现在 `crates/kernel/src/clock.rs` 和 `crates/testkit/*`。
    #[doc(hidden)]
    pub const fn from_clock_elapsed(elapsed: Duration) -> Self {
        Self(elapsed)
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
    #[error("system clock is before Unix epoch")]
    BeforeUnixEpoch,

    /// 时钟值超出可表示的纳秒范围。
    #[error("clock value exceeds representable nanoseconds")]
    Overflow,

    /// 时间源不可用。
    #[error("time source unavailable")]
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
#[derive(Debug, Clone)]
pub struct SystemClock {
    origin: std::time::Instant,
}

impl SystemClock {
    /// 创建一个新的 `SystemClock`，以当前时刻作为单调钟原点。
    pub fn new() -> Self {
        Self { origin: std::time::Instant::now() }
    }
}

impl Default for SystemClock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock for SystemClock {
    fn now(&self) -> Result<Timestamp, ClockError> {
        use std::time::{SystemTime, UNIX_EPOCH};

        let ts = SystemTime::now();
        match ts.duration_since(UNIX_EPOCH) {
            Ok(d) => {
                let nanos: u128 = d.as_nanos();
                let nanos_i64: i64 = nanos.try_into().map_err(|_| ClockError::Overflow)?;
                Ok(Timestamp::from_unix_nanos(nanos_i64))
            }
            Err(_) => Err(ClockError::BeforeUnixEpoch),
        }
    }

    fn monotonic(&self) -> MonotonicInstant {
        MonotonicInstant::from_clock_elapsed(self.origin.elapsed())
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
        // Adding a duration larger than i128::MAX nanoseconds should overflow
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
    fn test_timestamp_checked_sub_huge_duration() {
        // Duration larger than i128::MAX nanoseconds triggers early return
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
        // a (2s) is earlier than b (5s); b - a = 3s (valid)
        assert_eq!(b.checked_duration_since(a), Some(Duration::from_secs(3)));
        // a - b would be negative, should return None
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
        let ts = clock.now().expect("wall clock should be available");
        // Sanity: timestamp should be well after Y2K
        assert!(ts.as_unix_nanos() > 0);
        // SystemTime nanos should fit in i64 for many decades
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
        let clock = SystemClock::default();
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
    }
}
