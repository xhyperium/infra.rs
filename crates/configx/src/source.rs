//! 配置源抽象：内存 / 环境变量 / KEY=VALUE 文件。

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use kernel::{XError, XResult};

/// 可加载配置条目的源。
///
/// 实现应返回完整快照；调用方负责合并与覆盖策略。
pub trait ConfigSource: Send + Sync {
    /// 加载当前键值映射。
    fn load(&self) -> XResult<HashMap<String, String>>;
}

/// 内存配置源。
#[derive(Debug, Clone, Default)]
pub struct MemorySource {
    entries: HashMap<String, String>,
}

impl MemorySource {
    /// 空源。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 由键值对构造。
    #[must_use]
    pub fn from_pairs<I, K, V>(pairs: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        let mut entries = HashMap::new();
        for (k, v) in pairs {
            entries.insert(k.into(), v.into());
        }
        Self { entries }
    }
}

impl ConfigSource for MemorySource {
    fn load(&self) -> XResult<HashMap<String, String>> {
        Ok(self.entries.clone())
    }
}

/// 环境变量配置源。
///
/// 仅收集以 `prefix` 开头的变量；写入映射时**剥离前缀**。
/// 例如 `prefix = "APP_"`，`APP_HOST=h` → key `"HOST"`。
#[derive(Debug, Clone)]
pub struct EnvSource {
    prefix: String,
}

impl EnvSource {
    /// 构造；`prefix` 为空则不加载任何变量（避免误吞整表环境）。
    #[must_use]
    pub fn new(prefix: impl Into<String>) -> Self {
        Self { prefix: prefix.into() }
    }

    /// 前缀。
    #[must_use]
    pub fn prefix(&self) -> &str {
        &self.prefix
    }
}

impl ConfigSource for EnvSource {
    fn load(&self) -> XResult<HashMap<String, String>> {
        self.load_from_iter(env::vars())
    }
}

impl EnvSource {
    /// 从任意键值迭代器加载（测试与注入用；生产路径用 [`ConfigSource::load`]）。
    pub fn load_from_iter<I, K, V>(&self, vars: I) -> XResult<HashMap<String, String>>
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: Into<String>,
    {
        if self.prefix.is_empty() {
            return Ok(HashMap::new());
        }
        let mut out = HashMap::new();
        for (k, v) in vars {
            let k = k.as_ref();
            if let Some(stripped) = k.strip_prefix(&self.prefix) {
                if !stripped.is_empty() {
                    out.insert(stripped.to_string(), v.into());
                }
            }
        }
        Ok(out)
    }
}

/// 简单文件配置源：`KEY=VALUE` 行（`#` 注释、空行忽略）。
///
/// 值两侧空白会被 trim；支持可选引号包裹（单/双引号各去一层）。
#[derive(Debug, Clone)]
pub struct FileSource {
    path: PathBuf,
}

impl FileSource {
    /// 指定文件路径。
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// 路径。
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl ConfigSource for FileSource {
    fn load(&self) -> XResult<HashMap<String, String>> {
        let text = fs::read_to_string(&self.path).map_err(|e| {
            XError::invalid(format!("read config file {}: {e}", self.path.display()))
        })?;
        parse_key_value_file(&text)
    }
}

/// 解析 `KEY=VALUE` 文本。
pub fn parse_key_value_file(text: &str) -> XResult<HashMap<String, String>> {
    let mut out = HashMap::new();
    for (lineno, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((k, v)) = line.split_once('=') else {
            return Err(XError::invalid(format!(
                "config file line {}: expected KEY=VALUE, got `{raw}`",
                lineno + 1
            )));
        };
        let key = k.trim();
        if key.is_empty() {
            return Err(XError::invalid(format!("config file line {}: empty key", lineno + 1)));
        }
        let mut val = v.trim().to_string();
        if (val.starts_with('"') && val.ends_with('"') && val.len() >= 2)
            || (val.starts_with('\'') && val.ends_with('\'') && val.len() >= 2)
        {
            val = val[1..val.len() - 1].to_string();
        }
        out.insert(key.to_string(), val);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn memory_and_parse() {
        let m = MemorySource::from_pairs([("a", "1"), ("b", "2")]);
        let map = m.load().unwrap();
        assert_eq!(map.get("a").map(String::as_str), Some("1"));
        let parsed = parse_key_value_file("# c\nHOST=h\nPORT = \"8080\"\n\n").unwrap();
        assert_eq!(parsed.get("HOST").map(String::as_str), Some("h"));
        assert_eq!(parsed.get("PORT").map(String::as_str), Some("8080"));
        assert!(parse_key_value_file("nope").is_err());
        assert!(parse_key_value_file("=v").is_err());
    }

    #[test]
    fn file_source_roundtrip() {
        let dir = std::env::temp_dir().join(format!("cfg-src-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("app.conf");
        {
            let mut f = std::fs::File::create(&path).unwrap();
            writeln!(f, "A=1\nB=two").unwrap();
        }
        let src = FileSource::new(&path);
        assert_eq!(src.path(), path.as_path());
        let map = src.load().unwrap();
        assert_eq!(map.get("A").map(String::as_str), Some("1"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn env_source_prefix() {
        let prefix = "APP_";
        let src = EnvSource::new(prefix);
        assert_eq!(src.prefix(), prefix);
        let map = src
            .load_from_iter([
                ("APP_HOST", "h"),
                ("APP_PORT", "80"),
                ("OTHER", "x"),
                ("APP_", "empty-suffix-skip"),
            ])
            .unwrap();
        assert_eq!(map.get("HOST").map(String::as_str), Some("h"));
        assert_eq!(map.get("PORT").map(String::as_str), Some("80"));
        assert!(!map.contains_key("OTHER"));
        assert!(!map.contains_key(""));
        let empty = EnvSource::new("").load().unwrap();
        assert!(empty.is_empty());
        // 真实环境变量路径可调用（不依赖特定键）
        let _ = EnvSource::new("UNLIKELY_PREFIX_XYZ_").load().unwrap();
    }
}
