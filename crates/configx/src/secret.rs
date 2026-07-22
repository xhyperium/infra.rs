//! 密钥字符串与配置存储辅助。

use std::fmt;

use kernel::XResult;

use crate::ConfigStore;

/// 敏感字符串：[`Debug`] 显示为 `***`，须通过 [`expose`](Self::expose) 显式读取。
#[derive(Clone, PartialEq, Eq)]
pub struct SecretString {
    inner: String,
}

impl SecretString {
    /// 构造。
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self { inner: value.into() }
    }

    /// 显式暴露明文（调用方负责最小化使用范围）。
    #[must_use]
    pub fn expose(&self) -> &str {
        &self.inner
    }

    /// 是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// 字节长度（不含脱敏）。
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SecretString(***)")
    }
}

impl fmt::Display for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("***")
    }
}

impl From<String> for SecretString {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for SecretString {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

/// 密钥 key 统一前缀（`set_secret` 写入时附加）。
pub const SECRET_KEY_PREFIX: &str = "secret:";

/// 将密钥写入 store（key 自动加 [`SECRET_KEY_PREFIX`] 若尚未带前缀）。
pub fn set_secret(store: &ConfigStore, key: impl AsRef<str>, secret: &SecretString) -> XResult<()> {
    let raw = key.as_ref();
    let full = if raw.starts_with(SECRET_KEY_PREFIX) {
        raw.to_string()
    } else {
        format!("{SECRET_KEY_PREFIX}{raw}")
    };
    store.set(full, secret.expose())
}

/// 读取密钥；缺失返回 `None`。
///
/// 接受裸 key 或已带前缀的 key。
#[must_use]
pub fn get_secret(store: &ConfigStore, key: &str) -> Option<SecretString> {
    let full = if key.starts_with(SECRET_KEY_PREFIX) {
        key.to_string()
    } else {
        format!("{SECRET_KEY_PREFIX}{key}")
    };
    store.get(&full).map(SecretString::new)
}

/// 判断 store key 是否为密钥标记。
#[must_use]
pub fn is_secret_key(key: &str) -> bool {
    key.starts_with(SECRET_KEY_PREFIX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_redacts() {
        let s = SecretString::new("top-secret");
        assert_eq!(s.expose(), "top-secret");
        assert!(format!("{s:?}").contains("***"));
        assert!(!format!("{s:?}").contains("top-secret"));
        assert_eq!(format!("{s}"), "***");
        assert_eq!(s.len(), 10);
        assert!(!s.is_empty());
        let from: SecretString = "x".into();
        assert_eq!(from.expose(), "x");
        let from_s: SecretString = String::from("y").into();
        assert_eq!(from_s.expose(), "y");
    }

    #[test]
    fn store_helpers() {
        let store = ConfigStore::new();
        set_secret(&store, "db_password", &SecretString::new("p@ss")).unwrap();
        assert!(store.contains_key("secret:db_password"));
        let got = get_secret(&store, "db_password").unwrap();
        assert_eq!(got.expose(), "p@ss");
        assert!(is_secret_key("secret:db_password"));
        assert!(!is_secret_key("db_password"));
        let again = get_secret(&store, "secret:db_password").unwrap();
        assert_eq!(again.expose(), "p@ss");
    }
}
