//! decimalx —— `/types/` 十进制数值类型（ADR-006/007，spec §4.2）。
//!
//! 纯基础数值层，无业务逻辑。`Decimal` 族唯一定义点（ADR-007）。
//! 禁止 `f32`/`f64` 参与任何金额/数量运算；除法必须显式指定 [`RoundingStrategy`]。
//!
//! ## 契约要点（ADR-006）
//! - 双目加减/比较：scale 不一致时对齐到较大 scale（不足位补零，不隐式舍入）
//! - 除法：必须显式 `RoundingStrategy`；结果 scale = `max(lhs.scale, rhs.scale)`
//! - 缩位：[`Decimal::rescale`] / [`Decimal::checked_rescale`]
//! - 溢出：checked API 返回 `Err`；运算符在溢出时 panic（禁止静默回绕）
//!
//! ## 生产路径（P0）
//! - 推荐构造：[`Decimal::try_new`] / [`str::parse`]（`FromStr` 强制 [`MAX_SCALE`]）
//! - 推荐运算：`checked_*`；资金路径禁止依赖 panicking `+/-/*` / [`Decimal::rescale`]
//! - 字段仍 `pub`（兼容）；绕过校验的值须经 [`Decimal::validate`] 再进入生产逻辑
//! - wire：serde 字段 shape 为**当前事实**，**不**等于跨版本稳定协议（见 `docs/WIRE.md`）

use kernel::{XError, XResult};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::{Add, Mul, Sub};
use std::str::FromStr;

/// 生产 fallible API 强制的最大 scale（常见 NUMERIC(38,18) / 交易所精度）。
pub const MAX_SCALE: u8 = 18;

/// i128 可表示的最大 `10^exp` 指数。
pub const TECH_MAX_POW10_EXP: u32 = 38;

/// 十进制表示边界（生产 fallible 路径）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecimalLimits;

impl DecimalLimits {
    /// 与 [`MAX_SCALE`] 相同。
    pub const MAX_SCALE: u8 = MAX_SCALE;
    /// 与 [`TECH_MAX_POW10_EXP`] 相同。
    pub const TECH_MAX_POW10_EXP: u32 = TECH_MAX_POW10_EXP;
}

/// 十进制数：`mantissa × 10^(-scale)`（spec §4.2）。
///
/// 相等性与排序按**数值**（scale 对齐后）比较，而非按字段结构。
/// serde 仍按 `{mantissa, scale}` 字段序列化（未裁定 canonical wire format）。
/// 字段公开是兼容事实：生产应使用 [`Self::try_new`] / [`Self::validate`]。
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Decimal {
    /// 定点整数 mantissa。
    pub mantissa: i128,
    /// 小数位数；生产 fallible 路径限制 `≤ MAX_SCALE`。
    pub scale: u8,
}

impl Decimal {
    /// 未校验 scale 的构造（兼容保留）。生产请用 [`Self::try_new`]。
    pub const fn new(mantissa: i128, scale: u8) -> Self {
        Self { mantissa, scale }
    }

    /// 生产推荐构造：拒绝 `scale > MAX_SCALE`。
    pub fn try_new(mantissa: i128, scale: u8) -> XResult<Self> {
        if scale > MAX_SCALE {
            return Err(XError::invalid(format!(
                "decimal scale {scale} exceeds MAX_SCALE ({MAX_SCALE})"
            )));
        }
        Ok(Self { mantissa, scale })
    }

    /// 当前值是否满足生产 scale 上限。
    pub const fn is_within_limits(self) -> bool {
        self.scale <= MAX_SCALE
    }

    /// 校验 scale 上限；失败返回 `Err`。
    pub fn validate(self) -> XResult<Self> {
        Self::try_new(self.mantissa, self.scale)
    }

    /// 零值：`0 × 10^0`。
    pub const ZERO: Self = Self { mantissa: 0, scale: 0 };

    /// 运算结果检查：若 scale 越界，先 `normalize` 去掉尾随零再判定（保留可精确表示的乘积）。
    fn finish(self) -> XResult<Self> {
        if self.scale <= MAX_SCALE {
            return Ok(self);
        }
        let n = self.normalize();
        if n.scale <= MAX_SCALE {
            return Ok(n);
        }
        Err(XError::invalid(format!(
            "decimal scale {} exceeds MAX_SCALE ({MAX_SCALE})",
            self.scale
        )))
    }

    /// `10^exp`，溢出时返回 `None`。
    fn pow10(exp: u32) -> Option<i128> {
        10i128.checked_pow(exp)
    }

    /// 对齐到 `target` scale（`target >= self.scale` 时乘以 10 的幂）；禁止静默回绕。
    fn try_align_scale(self, target: u8) -> XResult<Decimal> {
        if target > MAX_SCALE {
            return Err(XError::invalid(format!(
                "decimal scale {target} exceeds MAX_SCALE ({MAX_SCALE})"
            )));
        }
        if self.scale >= target {
            return Ok(self);
        }
        let diff = u32::from(target - self.scale);
        let factor = Self::pow10(diff).ok_or_else(|| XError::invalid("decimal scale overflow"))?;
        let mantissa = self
            .mantissa
            .checked_mul(factor)
            .ok_or_else(|| XError::invalid("decimal overflow in scale alignment"))?;
        Ok(Decimal { mantissa, scale: target })
    }

    /// 去掉尾随零，得到值相等的规范表示（用于 Hash）。
    fn canonical(self) -> (i128, u8) {
        if self.mantissa == 0 {
            return (0, 0);
        }
        let mut m = self.mantissa;
        let mut s = self.scale;
        while s > 0 && m % 10 == 0 {
            m /= 10;
            s -= 1;
        }
        (m, s)
    }

    /// 数值比较（ADR-006：对齐到较大 scale）。对齐溢出时按量级判定。
    pub fn cmp_value(self, other: Decimal) -> Ordering {
        if self.mantissa == 0 && other.mantissa == 0 {
            return Ordering::Equal;
        }
        if self.scale == other.scale {
            return self.mantissa.cmp(&other.mantissa);
        }
        if self.scale < other.scale {
            let diff = u32::from(other.scale - self.scale);
            match Self::pow10(diff).and_then(|f| self.mantissa.checked_mul(f)) {
                Some(aligned) => aligned.cmp(&other.mantissa),
                None => {
                    // |self| 在 other 的 scale 下无法装入 i128 → |self| 更大
                    if self.mantissa > 0 { Ordering::Greater } else { Ordering::Less }
                }
            }
        } else {
            other.cmp_value(self).reverse()
        }
    }

    /// 比较：scale 对齐后值相同。
    pub fn eq_value(self, other: Decimal) -> bool {
        self.cmp_value(other) == Ordering::Equal
    }

    /// 加减法：scale 对齐到较大值；溢出返回 `Err`（ADR-006 `checked_add`）。
    pub fn checked_add(self, rhs: Decimal) -> XResult<Decimal> {
        let scale = self.scale.max(rhs.scale);
        let a = self.try_align_scale(scale)?;
        let b = rhs.try_align_scale(scale)?;
        let mantissa = a
            .mantissa
            .checked_add(b.mantissa)
            .ok_or_else(|| XError::invalid("decimal overflow in addition"))?;
        Decimal { mantissa, scale }.finish()
    }

    /// 减法：scale 对齐到较大值；溢出返回 `Err`（ADR-006 `checked_sub`）。
    pub fn checked_sub(self, rhs: Decimal) -> XResult<Decimal> {
        let scale = self.scale.max(rhs.scale);
        let a = self.try_align_scale(scale)?;
        let b = rhs.try_align_scale(scale)?;
        let mantissa = a
            .mantissa
            .checked_sub(b.mantissa)
            .ok_or_else(|| XError::invalid("decimal overflow in subtraction"))?;
        Decimal { mantissa, scale }.finish()
    }

    /// 乘法：mantissa 相乘、scale 相加；溢出返回 `Err`。
    pub fn checked_mul(self, rhs: Decimal) -> XResult<Decimal> {
        let mantissa = self
            .mantissa
            .checked_mul(rhs.mantissa)
            .ok_or_else(|| XError::invalid("decimal overflow in multiplication"))?;
        let scale = self
            .scale
            .checked_add(rhs.scale)
            .ok_or_else(|| XError::invalid("decimal scale overflow in multiplication"))?;
        Decimal { mantissa, scale }.finish()
    }

    /// 除法：必须显式指定舍入策略（ADR-006 `checked_div`）。
    ///
    /// 结果 scale = `max(self.scale, other.scale)`。
    /// 数值：`round(m1 * 10^(s_r - s1 + s2) / m2)`，等价于对齐双方后再按目标 scale 定点除。
    pub fn checked_div(self, other: Decimal, strategy: RoundingStrategy) -> XResult<Decimal> {
        if other.mantissa == 0 {
            return Err(XError::invalid("division by zero"));
        }
        let target_scale = self.scale.max(other.scale);
        if target_scale > MAX_SCALE {
            return Err(XError::invalid(format!(
                "decimal scale {target_scale} exceeds MAX_SCALE ({MAX_SCALE})"
            )));
        }
        // exp = (target_scale - self.scale) + other.scale ≥ 0
        let exp = u32::from(target_scale - self.scale) + u32::from(other.scale);
        let factor = Self::pow10(exp).ok_or_else(|| XError::invalid("decimal scale overflow"))?;
        let numerator = self
            .mantissa
            .checked_mul(factor)
            .ok_or_else(|| XError::invalid("decimal overflow in division"))?;
        let denominator = other.mantissa;
        // i128::MIN / -1 会溢出：必须 checked，禁止 bare `/`/`%` panic
        let q = numerator
            .checked_div(denominator)
            .ok_or_else(|| XError::invalid("decimal overflow in division"))?;
        let r = numerator
            .checked_rem(denominator)
            .ok_or_else(|| XError::invalid("decimal overflow in division"))?;
        let rounded = apply_rounding(q, r, denominator, strategy)?;
        Decimal { mantissa: rounded, scale: target_scale }.finish()
    }

    /// 除法（与 [`Self::checked_div`] 相同；保留既有公开名）。
    pub fn div(self, other: Decimal, strategy: RoundingStrategy) -> XResult<Decimal> {
        self.checked_div(other, strategy)
    }

    /// 显式缩位/扩位到目标 scale（ADR-006）。
    ///
    /// 生产路径请优先使用 [`Self::checked_rescale`]。
    ///
    /// # Panics
    ///
    /// 当 scale 对齐或舍入导致溢出时 panic（内部 `expect`）。禁止用本方法掩盖溢出。
    pub fn rescale(self, target_scale: u8, strategy: RoundingStrategy) -> Decimal {
        self.checked_rescale(target_scale, strategy).expect("decimal rescale overflow")
    }

    /// 显式缩位/扩位；溢出/非法返回 `Err`。
    pub fn checked_rescale(self, target_scale: u8, strategy: RoundingStrategy) -> XResult<Decimal> {
        if target_scale > MAX_SCALE {
            return Err(XError::invalid(format!(
                "decimal scale {target_scale} exceeds MAX_SCALE ({MAX_SCALE})"
            )));
        }
        if target_scale == self.scale {
            return self.finish();
        }
        if target_scale > self.scale {
            return self.try_align_scale(target_scale);
        }
        let diff = u32::from(self.scale - target_scale);
        let factor = Self::pow10(diff).ok_or_else(|| XError::invalid("decimal scale overflow"))?;
        // factor 恒为正幂，但统一走 checked 路径，避免任何 i128 除法 panic
        let q = self
            .mantissa
            .checked_div(factor)
            .ok_or_else(|| XError::invalid("decimal overflow in rescale"))?;
        let r = self
            .mantissa
            .checked_rem(factor)
            .ok_or_else(|| XError::invalid("decimal overflow in rescale"))?;
        let rounded = apply_rounding(q, r, factor, strategy)?;
        Decimal { mantissa: rounded, scale: target_scale }.finish()
    }

    /// 去掉尾随小数零，使 scale 最小且值不变。
    pub fn normalize(self) -> Decimal {
        let mut mantissa = self.mantissa;
        let mut scale = self.scale;
        while scale > 0 && mantissa % 10 == 0 {
            mantissa /= 10;
            scale -= 1;
        }
        Decimal { mantissa, scale }
    }
}

impl PartialEq for Decimal {
    fn eq(&self, other: &Self) -> bool {
        self.eq_value(*other)
    }
}

impl Eq for Decimal {}

impl PartialOrd for Decimal {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Decimal {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cmp_value(*other)
    }
}

impl Hash for Decimal {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let (m, s) = self.canonical();
        m.hash(state);
        s.hash(state);
    }
}

impl FromStr for Decimal {
    type Err = XError;

    /// 解析十进制字符串（如 `"100"` / `"100.5"` / `"-1.25"`）。
    ///
    /// 禁止 `NaN` / `Inf` 等非有限表示；非法输入返回 [`XError::invalid`]。
    fn from_str(s: &str) -> XResult<Self> {
        let s = s.trim();
        if s.is_empty() {
            return Err(XError::invalid("empty decimal string"));
        }

        // 禁止 NaN / Inf（大小写不敏感）
        let lower = s.to_ascii_lowercase();
        if matches!(
            lower.as_str(),
            "nan" | "inf" | "+inf" | "-inf" | "infinity" | "+infinity" | "-infinity"
        ) {
            return Err(XError::invalid("NaN/Inf not allowed for Decimal"));
        }

        let (negative, body) = if let Some(rest) = s.strip_prefix('-') {
            (true, rest)
        } else if let Some(rest) = s.strip_prefix('+') {
            (false, rest)
        } else {
            (false, s)
        };

        if body.is_empty() {
            return Err(XError::invalid(format!("invalid decimal: {s}")));
        }

        // 至多一个小数点
        if body.bytes().filter(|&b| b == b'.').count() > 1 {
            return Err(XError::invalid(format!("invalid decimal: {s}")));
        }

        let (int_part, frac_part) = match body.split_once('.') {
            Some((i, f)) => (i, f),
            None => (body, ""),
        };

        // 允许 "5." / ".5"；拒绝单独 "."
        if int_part.is_empty() && frac_part.is_empty() {
            return Err(XError::invalid(format!("invalid decimal: {s}")));
        }
        if !int_part.is_empty() && !int_part.bytes().all(|b| b.is_ascii_digit()) {
            return Err(XError::invalid(format!("invalid decimal: {s}")));
        }
        if !frac_part.bytes().all(|b| b.is_ascii_digit()) {
            return Err(XError::invalid(format!("invalid decimal: {s}")));
        }

        // 生产 FromStr：scale 受 MAX_SCALE 约束
        if frac_part.len() > MAX_SCALE as usize {
            return Err(XError::invalid(format!(
                "decimal scale {} exceeds MAX_SCALE ({MAX_SCALE})",
                frac_part.len()
            )));
        }
        let scale = frac_part.len() as u8;

        // 拼接整数与小数位再解析为 i128
        let digits = if int_part.is_empty() {
            frac_part.to_string()
        } else if frac_part.is_empty() {
            int_part.to_string()
        } else {
            format!("{int_part}{frac_part}")
        };

        // 全零或空（如 "." 已拒绝）→ 0
        let abs_mantissa: i128 = if digits.is_empty() || digits.bytes().all(|b| b == b'0') {
            0
        } else {
            digits
                .parse::<i128>()
                .map_err(|_| XError::invalid(format!("decimal mantissa overflow: {s}")))?
        };

        let mantissa = if negative && abs_mantissa != 0 { -abs_mantissa } else { abs_mantissa };

        Decimal { mantissa, scale }.finish()
    }
}

impl fmt::Display for Decimal {
    /// 规范化展示：去掉无意义尾随小数零；纯整数不带小数点。
    ///
    /// 例：`100.0` → `"100"`，`10.50` → `"10.5"`，`0.00` → `"0"`。
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let n = self.normalize();
        if n.scale == 0 {
            return write!(f, "{}", n.mantissa);
        }

        let neg = n.mantissa < 0;
        let abs = n.mantissa.unsigned_abs();
        let divisor = 10u128.pow(n.scale as u32);
        let int_part = abs / divisor;
        let frac_part = abs % divisor;
        if neg {
            write!(f, "-{int_part}.{:0width$}", frac_part, width = n.scale as usize)
        } else {
            write!(f, "{int_part}.{:0width$}", frac_part, width = n.scale as usize)
        }
    }
}

/// 加法运算符：内部走 [`Decimal::checked_add`]。
///
/// **生产资金路径请用 [`Decimal::checked_add`]**；本运算符仅保留兼容/测试便利。
///
/// # Panics
///
/// scale 对齐或 mantissa 加法溢出时 panic（含 `MAX_SCALE` 越界）。
impl Add for Decimal {
    type Output = Decimal;
    fn add(self, other: Decimal) -> Decimal {
        self.checked_add(other).expect("decimal add overflow")
    }
}

/// 减法运算符：内部走 [`Decimal::checked_sub`]。
///
/// **生产资金路径请用 [`Decimal::checked_sub`]**；本运算符仅保留兼容/测试便利。
///
/// # Panics
///
/// scale 对齐或 mantissa 减法溢出时 panic（含 `MAX_SCALE` 越界）。
impl Sub for Decimal {
    type Output = Decimal;
    fn sub(self, other: Decimal) -> Decimal {
        self.checked_sub(other).expect("decimal sub overflow")
    }
}

/// 乘法运算符：内部走 [`Decimal::checked_mul`]。
///
/// **生产资金路径请用 [`Decimal::checked_mul`]**；本运算符仅保留兼容/测试便利。
///
/// # Panics
///
/// mantissa 相乘、scale 相加溢出或结果 `scale > MAX_SCALE` 时 panic。
impl Mul for Decimal {
    type Output = Decimal;
    fn mul(self, other: Decimal) -> Decimal {
        self.checked_mul(other).expect("decimal mul overflow")
    }
}

/// 舍入策略（ADR-006）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoundingStrategy {
    /// 向 −∞ 取整
    Floor,
    /// 向 +∞ 取整
    Ceiling,
    /// 四舍五入（恰好一半时远离零）
    HalfUp,
    /// 五舍六入（恰好一半时趋向零）
    HalfDown,
    /// 银行家舍入（恰好一半时趋向偶数）
    HalfEven,
}

/// 对向零截断的 `(q, r)` 按策略舍入；`r` 为余数（与 Rust `%` 同号于被除数）。
///
/// 中点判定用 `2·|r| ? |d|`，避免 `abs_d/2` 在奇数分母上把非中点当成中点。
fn apply_rounding(q: i128, r: i128, d: i128, strategy: RoundingStrategy) -> XResult<i128> {
    if r == 0 {
        return Ok(q);
    }
    // 真值符号为负 ⟺ 被除数与除数异号。Rust 中 r 与被除数同号。
    let neg = (r < 0) ^ (d < 0);

    let abs_r = r.unsigned_abs();
    let abs_d = d.unsigned_abs();
    // 比较 2·|r| 与 |d|；防止 u128 乘法溢出（|r| > u128::MAX/2）
    let cmp_half = if abs_r > u128::MAX / 2 { Ordering::Greater } else { (abs_r * 2).cmp(&abs_d) };

    let round_away = match strategy {
        RoundingStrategy::Floor => neg,
        RoundingStrategy::Ceiling => !neg,
        RoundingStrategy::HalfUp => matches!(cmp_half, Ordering::Greater | Ordering::Equal),
        RoundingStrategy::HalfDown => matches!(cmp_half, Ordering::Greater),
        RoundingStrategy::HalfEven => match cmp_half {
            Ordering::Greater => true,
            Ordering::Equal => q % 2 != 0,
            Ordering::Less => false,
        },
    };

    if !round_away {
        return Ok(q);
    }
    if neg {
        q.checked_sub(1).ok_or_else(|| XError::invalid("decimal overflow in rounding"))
    } else {
        q.checked_add(1).ok_or_else(|| XError::invalid("decimal overflow in rounding"))
    }
}

/// 价格（newtype，spec §4.2）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Price(pub Decimal);

/// 数量（newtype，spec §4.2）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Qty(pub Decimal);

/// 比率（newtype，spec §4.2）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Ratio(pub Decimal);

/// ISO 4217 风格币种标识（3 字节大写 ASCII）。字段公开是兼容事实；生产用 try_new/parse。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Currency(pub [u8; 3]);

impl Currency {
    /// 将内部 3 字节解释为 UTF-8；非法字节时返回空串。
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.0).unwrap_or("")
    }

    /// 生产构造：三字节均须为大写 ASCII 字母。
    pub fn try_new(bytes: [u8; 3]) -> XResult<Self> {
        if !bytes.iter().all(|c| c.is_ascii_uppercase()) {
            return Err(XError::invalid("currency must be 3 uppercase ASCII letters"));
        }
        Ok(Self(bytes))
    }

    /// 当前字节是否全部为大写 ASCII。
    pub fn is_valid(self) -> bool {
        self.0.iter().all(|c| c.is_ascii_uppercase())
    }

    /// 校验后返回自身，否则 `Err`。
    pub fn validate(self) -> XResult<Self> {
        Self::try_new(self.0)
    }
}

impl std::str::FromStr for Currency {
    type Err = XError;

    fn from_str(s: &str) -> XResult<Self> {
        let b = s.as_bytes();
        if b.len() != 3 {
            return Err(XError::invalid("currency must be 3 uppercase ASCII letters"));
        }
        let mut arr = [0u8; 3];
        arr.copy_from_slice(b);
        Self::try_new(arr)
    }
}

/// 金额（spec §4.2）。生产请用 [`Money::try_new`]。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Money {
    /// 金额数值。
    pub amount: Decimal,
    /// 币种。
    pub currency: Currency,
}

impl Money {
    /// 生产构造：同时校验 amount scale 与 currency 合法性。
    pub fn try_new(amount: Decimal, currency: Currency) -> XResult<Self> {
        let amount = amount.validate()?;
        let currency = currency.validate()?;
        Ok(Self { amount, currency })
    }

    /// 校验 amount/currency 后返回自身。
    pub fn validate(self) -> XResult<Self> {
        Self::try_new(self.amount, self.currency)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;

    fn hash_of(d: Decimal) -> u64 {
        let mut h = DefaultHasher::new();
        d.hash(&mut h);
        h.finish()
    }

    // ── 加减与 scale 对齐 ──────────────────────────────────────────

    #[test]
    fn decimal_add_aligns_scale() {
        let a = Decimal::new(1, 0);
        let b = Decimal::new(25, 2);
        let c = a + b;
        assert_eq!(c.scale, 2);
        assert_eq!(c.mantissa, 125);
    }

    #[test]
    fn decimal_sub() {
        let a = Decimal::new(10, 1);
        let b = Decimal::new(1, 0);
        let c = a - b;
        assert_eq!(c.mantissa, 0);
        assert!(c.eq_value(Decimal::ZERO));
    }

    #[test]
    fn decimal_mul_scales_add() {
        let a = Decimal::new(2, 1);
        let b = Decimal::new(3, 0);
        let c = a * b;
        assert_eq!(c.scale, 1);
        assert_eq!(c.mantissa, 6);
    }

    #[test]
    fn checked_add_sub_match_operators() {
        let a = Decimal::new(1, 0);
        let b = Decimal::new(25, 2);
        assert_eq!(a.checked_add(b).unwrap(), a + b);
        assert_eq!(a.checked_sub(b).unwrap(), a - b);
    }

    // ── 相等 / 排序（数值语义，非结构字段） ────────────────────────

    #[test]
    fn equality_aligns_scale() {
        // 1.0 == 1.00 == 1
        assert_eq!(Decimal::new(1, 0), Decimal::new(10, 1));
        assert_eq!(Decimal::new(10, 1), Decimal::new(100, 2));
        assert_eq!(Decimal::new(0, 0), Decimal::new(0, 5));
        assert_ne!(Decimal::new(1, 0), Decimal::new(1, 1));
    }

    #[test]
    fn ordering_aligns_scale() {
        assert!(Decimal::new(1, 0) < Decimal::new(15, 1)); // 1 < 1.5
        assert!(Decimal::new(20, 1) > Decimal::new(1, 0)); // 2.0 > 1
        assert!(Decimal::new(-5, 1) > Decimal::new(-1, 0)); // -0.5 > -1
        assert!(Decimal::new(-20, 1) < Decimal::new(-1, 0)); // -2.0 < -1
    }

    #[test]
    fn hash_consistent_with_value_eq() {
        let a = Decimal::new(1, 0);
        let b = Decimal::new(100, 2);
        assert_eq!(a, b);
        assert_eq!(hash_of(a), hash_of(b));
        assert_eq!(hash_of(Decimal::new(0, 0)), hash_of(Decimal::new(0, 7)));
    }

    // ── 除法 scale 合同 ────────────────────────────────────────────

    #[test]
    fn div_uses_rhs_scale_in_numerator() {
        // 1.00 / 0.50 = 2.00（旧实现把除数 mantissa 当整数，得到 0.02）
        let a = Decimal::new(100, 2);
        let b = Decimal::new(50, 2);
        let c = a.checked_div(b, RoundingStrategy::HalfUp).unwrap();
        assert_eq!(c, Decimal::new(200, 2));
        assert_eq!(c.scale, 2);
    }

    #[test]
    fn div_mixed_scales() {
        // 10 / 2.5 = 4.0
        let a = Decimal::new(10, 0);
        let b = Decimal::new(25, 1);
        let c = a.div(b, RoundingStrategy::HalfUp).unwrap();
        assert_eq!(c, Decimal::new(40, 1));
    }

    #[test]
    fn decimal_div_half_up_non_midpoint() {
        // 10/3 = 3.333… → scale 0 时 HalfUp 应为 3（非中点）
        let a = Decimal::new(10, 0);
        let b = Decimal::new(3, 0);
        let c = a.div(b, RoundingStrategy::HalfUp).unwrap();
        assert_eq!(c.mantissa, 3);
        assert_eq!(c.scale, 0);
    }

    #[test]
    fn decimal_div_floor() {
        let a = Decimal::new(10, 0);
        let b = Decimal::new(3, 0);
        let c = a.div(b, RoundingStrategy::Floor).unwrap();
        assert_eq!(c.mantissa, 3);
    }

    #[test]
    fn decimal_div_by_zero_errors() {
        let a = Decimal::new(1, 0);
        let b = Decimal::ZERO;
        let err = a.div(b, RoundingStrategy::Floor).unwrap_err();
        assert!(err.to_string().contains("division by zero"));
    }

    #[test]
    fn checked_div_i128_min_over_neg_one_errors_not_panic() {
        // i128::MIN / -1 在 bare `/` 上会 "attempt to divide with overflow"；
        // checked 路径必须返回 Err，与 checked_mul(MIN, -1) 一致。
        let a = Decimal::new(i128::MIN, 0);
        let b = Decimal::new(-1, 0);
        for strategy in [
            RoundingStrategy::Floor,
            RoundingStrategy::Ceiling,
            RoundingStrategy::HalfUp,
            RoundingStrategy::HalfDown,
            RoundingStrategy::HalfEven,
        ] {
            let err = a.checked_div(b, strategy).expect_err("must not panic");
            assert!(err.to_string().contains("overflow"), "strategy={strategy:?}: {err}");
            // div 别名同一路径
            assert!(a.div(b, strategy).is_err());
        }
        // 对照：乘法路径对同一边界也返回 Err
        assert!(a.checked_mul(b).is_err());
    }

    // ── Half* 中点与奇数分母 ───────────────────────────────────────

    #[test]
    fn half_up_exact_half_away_from_zero() {
        // 15/2 = 7.5 → HalfUp → 8
        let c = Decimal::new(15, 0).div(Decimal::new(2, 0), RoundingStrategy::HalfUp).unwrap();
        assert_eq!(c.mantissa, 8);
        // -15/2 = -7.5 → HalfUp → -8
        let c = Decimal::new(-15, 0).div(Decimal::new(2, 0), RoundingStrategy::HalfUp).unwrap();
        assert_eq!(c.mantissa, -8);
    }

    #[test]
    fn half_down_exact_half_toward_zero() {
        let c = Decimal::new(15, 0).div(Decimal::new(2, 0), RoundingStrategy::HalfDown).unwrap();
        assert_eq!(c.mantissa, 7);
        let c = Decimal::new(-15, 0).div(Decimal::new(2, 0), RoundingStrategy::HalfDown).unwrap();
        assert_eq!(c.mantissa, -7);
    }

    #[test]
    fn half_even_ties_to_even() {
        // 15/2 = 7.5 → 向偶数 8
        let c = Decimal::new(15, 0).div(Decimal::new(2, 0), RoundingStrategy::HalfEven).unwrap();
        assert_eq!(c.mantissa, 8);
        // 13/2 = 6.5 → 向偶数 6
        let c = Decimal::new(13, 0).div(Decimal::new(2, 0), RoundingStrategy::HalfEven).unwrap();
        assert_eq!(c.mantissa, 6);
        // -13/2 = -6.5 → -6（偶数）
        let c = Decimal::new(-13, 0).div(Decimal::new(2, 0), RoundingStrategy::HalfEven).unwrap();
        assert_eq!(c.mantissa, -6);
    }

    #[test]
    fn half_up_odd_denominator_not_false_midpoint() {
        // 旧 bug：half = abs_d/2 使 1/3 被当成 ≥ half
        // 10/3 = 3.333… HalfUp → 3；Ceiling → 4
        let a = Decimal::new(10, 0);
        let b = Decimal::new(3, 0);
        assert_eq!(a.div(b, RoundingStrategy::HalfUp).unwrap().mantissa, 3);
        assert_eq!(a.div(b, RoundingStrategy::Ceiling).unwrap().mantissa, 4);
        // 2/5 = 0.4 < 0.5 → HalfUp 保持 0
        assert_eq!(
            Decimal::new(2, 0).div(Decimal::new(5, 0), RoundingStrategy::HalfUp).unwrap().mantissa,
            0
        );
        // 3/5 = 0.6 > 0.5 → HalfUp → 1
        assert_eq!(
            Decimal::new(3, 0).div(Decimal::new(5, 0), RoundingStrategy::HalfUp).unwrap().mantissa,
            1
        );
    }

    #[test]
    fn floor_ceiling_negative() {
        // -10/3 = -3.333… Floor → -4, Ceiling → -3
        let a = Decimal::new(-10, 0);
        let b = Decimal::new(3, 0);
        assert_eq!(a.div(b, RoundingStrategy::Floor).unwrap().mantissa, -4);
        assert_eq!(a.div(b, RoundingStrategy::Ceiling).unwrap().mantissa, -3);
    }

    // ── rescale ────────────────────────────────────────────────────

    #[test]
    fn rescale_expand_and_shrink() {
        let d = Decimal::new(125, 2); // 1.25
        let up = d.rescale(4, RoundingStrategy::HalfUp);
        assert_eq!(up.mantissa, 12500);
        assert_eq!(up.scale, 4);

        // 1.25 → scale 1 HalfUp → 1.3
        let down = d.rescale(1, RoundingStrategy::HalfUp);
        assert_eq!(down.mantissa, 13);
        assert_eq!(down.scale, 1);

        // 1.25 → scale 1 HalfEven → 1.2（2 为偶数）
        let down_e = d.rescale(1, RoundingStrategy::HalfEven);
        assert_eq!(down_e.mantissa, 12);
    }

    #[test]
    fn rescale_half_up_midpoint() {
        // 1.25 → scale 1 HalfUp → 1.3；1.35 → 1.4
        assert_eq!(Decimal::new(125, 2).rescale(1, RoundingStrategy::HalfUp).mantissa, 13);
        assert_eq!(Decimal::new(135, 2).rescale(1, RoundingStrategy::HalfUp).mantissa, 14);
    }

    // ── 溢出 ──────────────────────────────────────────────────────

    #[test]
    fn align_scale_overflow_errors() {
        let d = Decimal::new(i128::MAX, 0);
        let err = d.try_align_scale(1).unwrap_err();
        assert!(err.to_string().contains("overflow"));
    }

    #[test]
    fn checked_add_overflow_errors() {
        let a = Decimal::new(i128::MAX, 0);
        let b = Decimal::new(1, 0);
        assert!(a.checked_add(b).is_err());
    }

    #[test]
    fn checked_mul_overflow_errors() {
        let a = Decimal::new(i128::MAX, 0);
        let b = Decimal::new(2, 0);
        assert!(a.checked_mul(b).is_err());
    }

    #[test]
    fn checked_mul_scale_overflow_errors() {
        let a = Decimal::new(1, 200);
        let b = Decimal::new(1, 200);
        assert!(a.checked_mul(b).is_err());
    }

    #[test]
    fn no_silent_wrap_on_align() {
        // 旧实现 wrapping_mul 会静默回绕得到错误值
        let d = Decimal::new(i128::MAX / 2 + 1, 0);
        assert!(d.try_align_scale(1).is_err());
    }

    #[test]
    fn cmp_value_overflow_safe_magnitude() {
        // 对齐会溢出时仍应正确比较量级
        let huge = Decimal::new(i128::MAX, 0);
        let small = Decimal::new(1, 40); // 1e-40
        assert!(huge > small);
        assert!(Decimal::new(i128::MIN, 0) < small);
    }

    // ── serde round-trip ───────────────────────────────────────────

    #[test]
    fn serde_roundtrip_struct_fields() {
        let d = Decimal::new(-12345, 3);
        let json = serde_json::to_string(&d).unwrap();
        let back: Decimal = serde_json::from_str(&json).unwrap();
        assert_eq!(back.mantissa, -12345);
        assert_eq!(back.scale, 3);
        assert_eq!(back, d);
    }

    #[test]
    fn serde_roundtrip_money() {
        let m = Money { amount: Decimal::new(999, 2), currency: "USD".parse().unwrap() };
        let json = serde_json::to_string(&m).unwrap();
        let back: Money = serde_json::from_str(&json).unwrap();
        assert_eq!(back, m);
    }

    // ── Currency ───────────────────────────────────────────────────

    #[test]
    fn currency_parse_valid() {
        let c: Currency = "USD".parse().unwrap();
        assert_eq!(c.as_str(), "USD");
    }

    #[test]
    fn currency_parse_invalid() {
        let r: Result<Currency, _> = "us".parse();
        assert!(r.is_err());
        let r: Result<Currency, _> = "USDD".parse();
        assert!(r.is_err());
        let r: Result<Currency, _> = "usd".parse();
        assert!(r.is_err());
    }

    #[test]
    fn no_f32_f64_in_api() {
        let d = Decimal::new(1, 0);
        let _ = (d.mantissa, d.scale);
    }

    #[test]
    fn from_str_basic() {
        let d: Decimal = "100".parse().unwrap();
        assert_eq!(d, Decimal::new(100, 0));

        let d: Decimal = "100.5".parse().unwrap();
        assert_eq!(d, Decimal::new(1005, 1));

        let d: Decimal = "0".parse().unwrap();
        assert_eq!(d, Decimal::ZERO);

        let d: Decimal = "-1.25".parse().unwrap();
        assert_eq!(d, Decimal::new(-125, 2));

        let d: Decimal = "100.0".parse().unwrap();
        assert_eq!(d, Decimal::new(1000, 1));
        assert!(d.eq_value(Decimal::new(100, 0)));
    }

    #[test]
    fn from_str_rejects_nan_inf_and_garbage() {
        for bad in ["", "nan", "NaN", "inf", "-Inf", "infinity", "1.2.3", "abc", "1e2", "--1", "."]
        {
            let r: Result<Decimal, _> = bad.parse();
            assert!(r.is_err(), "expected err for {bad:?}");
        }
    }

    #[test]
    fn display_strips_trailing_zeros() {
        assert_eq!(Decimal::new(1000, 1).to_string(), "100");
        assert_eq!(Decimal::new(1500, 1).to_string(), "150");
        assert_eq!(Decimal::new(1050, 2).to_string(), "10.5");
        assert_eq!(Decimal::new(0, 2).to_string(), "0");
        assert_eq!(Decimal::new(-125, 2).to_string(), "-1.25");
        assert_eq!(Decimal::new(10, 0).to_string(), "10");
    }

    #[test]
    fn parse_display_roundtrip() {
        for s in ["0", "100", "100.5", "-1.25", "10.5", "999.99", "0.1"] {
            let d: Decimal = s.parse().unwrap();
            let out = d.to_string();
            let back: Decimal = out.parse().unwrap();
            assert!(d.eq_value(back), "roundtrip value mismatch: {s} -> {out} -> {back:?}");
            // Display 已规范化，再次 Display 应稳定
            assert_eq!(back.to_string(), out);
        }
    }

    #[test]
    fn ledger_style_sum_display() {
        // 与 ledger 测试期望对齐： "100.0" + "50.0" → "150"
        let a: Decimal = "100.0".parse().unwrap();
        let b: Decimal = "50.0".parse().unwrap();
        assert_eq!((a + b).to_string(), "150");
        let c: Decimal = "10.0".parse().unwrap();
        assert_eq!(c.to_string(), "10");
    }

    // ── M0 边界补强（agent-safe；不宣称全 i128 空间） ──────────────

    #[test]
    fn from_str_leading_trailing_dot_and_trim() {
        let d: Decimal = ".5".parse().unwrap();
        assert_eq!(d, Decimal::new(5, 1));
        let d: Decimal = "5.".parse().unwrap();
        assert_eq!(d, Decimal::new(5, 0));
        let d: Decimal = "  -0.10  ".parse().unwrap();
        assert_eq!(d, Decimal::new(-10, 2));
        assert!(d.eq_value(Decimal::new(-1, 1)));
        let d: Decimal = "+3.0".parse().unwrap();
        assert_eq!(d, Decimal::new(30, 1));
    }

    #[test]
    fn checked_sub_overflow_errors() {
        let a = Decimal::new(i128::MIN, 0);
        let b = Decimal::new(1, 0);
        assert!(a.checked_sub(b).is_err());
    }

    #[test]
    fn checked_rescale_matches_rescale_when_ok() {
        let d = Decimal::new(199, 2); // 1.99
        let c = d.checked_rescale(1, RoundingStrategy::HalfUp).unwrap();
        assert_eq!(c, d.rescale(1, RoundingStrategy::HalfUp));
        assert_eq!(c.mantissa, 20);
        assert_eq!(c.scale, 1);
    }

    #[test]
    fn currency_public_field_invalid_as_str_empty() {
        // 公开字段可绕过 FromStr；as_str 对非 UTF-8 返回空串（当前兼容事实）
        let bad = Currency([0xff, 0xfe, 0xfd]);
        assert_eq!(bad.as_str(), "");
    }

    #[test]
    fn public_new_accepts_any_u8_scale_baseline() {
        // MAX_SCALE 未批准：new 接受任意 u8 是当前事实，非长期不变量
        let d = Decimal::new(1, u8::MAX);
        assert_eq!(d.scale, u8::MAX);
        assert_eq!(d.mantissa, 1);
    }

    #[test]
    fn hash_consistent_negative_trailing_zeros() {
        let a = Decimal::new(-100, 2); // -1.00
        let b = Decimal::new(-10, 1); // -1.0
        let c = Decimal::new(-1, 0); // -1
        assert_eq!(a, b);
        assert_eq!(b, c);
        assert_eq!(hash_of(a), hash_of(b));
        assert_eq!(hash_of(b), hash_of(c));
    }

    #[test]
    fn serde_shape_is_struct_fields_not_string() {
        let d = Decimal::new(42, 1);
        let json = serde_json::to_string(&d).unwrap();
        assert!(
            json.contains("mantissa") && json.contains("scale"),
            "current wire shape is struct fields: {json}"
        );
        assert!(!json.starts_with('"'), "not a bare decimal string: {json}");
    }

    #[test]
    fn try_new_enforces_max_scale() {
        assert_eq!(MAX_SCALE, 18);
        assert!(Decimal::try_new(1, MAX_SCALE).is_ok());
        assert!(Decimal::try_new(1, MAX_SCALE + 1).is_err());
        assert!(!Decimal::new(1, MAX_SCALE + 1).is_within_limits());
    }

    #[test]
    fn from_str_rejects_scale_above_max() {
        let s = format!("0.{}", "1".repeat((MAX_SCALE as usize) + 1));
        assert!(s.parse::<Decimal>().is_err());
        let s_ok = format!("0.{}", "1".repeat(MAX_SCALE as usize));
        assert_eq!(s_ok.parse::<Decimal>().unwrap().scale, MAX_SCALE);
    }

    #[test]
    fn checked_mul_result_respects_max_scale() {
        assert!(Decimal::new(1, 10).checked_mul(Decimal::new(1, 10)).is_err());
    }

    #[test]
    fn checked_mul_normalizes_before_max_scale_reject() {
        // 1.0 * 1e-18 → 原始 scale 19，normalize 后为 scale 18，应接受
        let a = Decimal::try_new(10, 1).unwrap();
        let b = Decimal::try_new(1, MAX_SCALE).unwrap();
        let p = a.checked_mul(b).expect("exact product within MAX_SCALE after normalize");
        assert_eq!(p, Decimal::try_new(1, MAX_SCALE).unwrap());
        // 不可约 scale 20 仍拒绝
        assert!(
            Decimal::try_new(3, 10).unwrap().checked_mul(Decimal::try_new(1, 10).unwrap()).is_err()
        );
    }

    #[test]
    fn checked_rescale_rejects_target_above_max() {
        assert!(
            Decimal::new(1, 0).checked_rescale(MAX_SCALE + 1, RoundingStrategy::HalfUp).is_err()
        );
    }

    #[test]
    fn currency_try_new_and_money_try_new() {
        let c = Currency::try_new(*b"USD").unwrap();
        assert!(c.is_valid());
        assert!(Currency::try_new(*b"usd").is_err());
        assert!(Money::try_new(Decimal::try_new(100, 2).unwrap(), c).is_ok());
        assert!(Money::try_new(Decimal::new(1, MAX_SCALE + 1), c).is_err());
    }
}
