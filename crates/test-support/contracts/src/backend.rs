//! 真后端 profile：声明 live 连接意图与环境探测（无伪造 live 日志）。
//!
//! 当环境缺少凭据时，[`BackendProfile::probe`] 返回 `Unavailable`，调用方应
//! 回退到 Fake 路径并记录 honest failure。

use contracts::LiveContractProfile;
use kernel::{XError, XResult};
use std::collections::HashMap;
use std::env;

/// 真后端连接 profile（环境变量驱动）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendProfile {
    /// 逻辑名（如 `redis-ci`）。
    pub name: String,
    /// 对应契约 live 声明。
    pub contracts: LiveContractProfile,
    /// 必需环境变量键。
    pub required_env: Vec<String>,
    /// 可选 URL 环境变量键。
    pub url_env: Option<String>,
}

impl BackendProfile {
    /// Redis KV live profile。
    #[must_use]
    pub fn redis_kv() -> Self {
        Self {
            name: "redis-kv".into(),
            contracts: {
                let mut p = LiveContractProfile::none();
                p.kv = true;
                p
            },
            required_env: vec!["REDIS_URL".into()],
            url_env: Some("REDIS_URL".into()),
        }
    }

    /// Postgres Tx/Repo live profile。
    #[must_use]
    pub fn postgres_sql() -> Self {
        Self {
            name: "postgres-sql".into(),
            contracts: {
                let mut p = LiveContractProfile::none();
                p.repo = true;
                p.tx = true;
                p
            },
            required_env: vec!["DATABASE_URL".into()],
            url_env: Some("DATABASE_URL".into()),
        }
    }

    /// 全存储栈（需多 URL）。
    #[must_use]
    pub fn storage_stack() -> Self {
        Self {
            name: "storage-stack".into(),
            contracts: LiveContractProfile::storage_stack(),
            required_env: vec!["REDIS_URL".into(), "DATABASE_URL".into()],
            url_env: None,
        }
    }

    /// 使用自定义环境查找探测（可测；生产用 [`Self::probe`]）。
    pub fn probe_with<F>(&self, mut get: F) -> XResult<String>
    where
        F: FnMut(&str) -> Option<String>,
    {
        for key in &self.required_env {
            match get(key) {
                Some(v) if !v.trim().is_empty() => {}
                _ => {
                    return Err(XError::unavailable(format!(
                        "backend profile `{}` 缺少环境变量 {key}（live 不可用，请用 Fake）",
                        self.name
                    )));
                }
            }
        }
        if let Some(url_key) = &self.url_env {
            return get(url_key).ok_or_else(|| {
                XError::unavailable(format!("backend profile `{}` URL 缺失", self.name))
            });
        }
        Ok(self.name.clone())
    }

    /// 探测进程环境：全部必需 env 非空则 Ok(url 或 name)，否则 Unavailable。
    pub fn probe(&self) -> XResult<String> {
        self.probe_with(|k| env::var(k).ok())
    }

    /// 是否所有必需 env 已设置（不返回错误）。
    #[must_use]
    pub fn is_available(&self) -> bool {
        self.probe().is_ok()
    }
}

/// 批量探测；返回可用与不可用列表。
pub fn classify_backends(profiles: &[BackendProfile]) -> (Vec<String>, Vec<String>) {
    let mut ok = Vec::new();
    let mut bad = Vec::new();
    for p in profiles {
        if p.is_available() {
            ok.push(p.name.clone());
        } else {
            bad.push(p.name.clone());
        }
    }
    (ok, bad)
}

/// 用给定 map 做批量探测（测试与离线编排）。
pub fn classify_backends_with(
    profiles: &[BackendProfile],
    env_map: &HashMap<String, String>,
) -> (Vec<String>, Vec<String>) {
    let mut ok = Vec::new();
    let mut bad = Vec::new();
    for p in profiles {
        if p.probe_with(|k| env_map.get(k).cloned()).is_ok() {
            ok.push(p.name.clone());
        } else {
            bad.push(p.name.clone());
        }
    }
    (ok, bad)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redis_probe_missing_env() {
        let p = BackendProfile::redis_kv();
        let err = p.probe_with(|_| None).unwrap_err();
        assert_eq!(err.kind(), kernel::ErrorKind::Unavailable);
    }

    #[test]
    fn redis_probe_with_map() {
        let p = BackendProfile::redis_kv();
        let mut m: HashMap<String, String> = HashMap::new();
        m.insert("REDIS_URL".into(), "redis://127.0.0.1:6379".into());
        assert_eq!(p.probe_with(|k| m.get(k).cloned()).unwrap(), "redis://127.0.0.1:6379");
    }

    #[test]
    fn classify_backends_with_map() {
        let profiles = [BackendProfile::redis_kv(), BackendProfile::postgres_sql()];
        let empty: HashMap<String, String> = HashMap::new();
        let (ok, bad) = classify_backends_with(&profiles, &empty);
        assert!(ok.is_empty());
        assert_eq!(bad.len(), 2);

        let mut m: HashMap<String, String> = HashMap::new();
        m.insert("REDIS_URL".into(), "redis://x".into());
        m.insert("DATABASE_URL".into(), "postgres://x".into());
        let (ok2, bad2) = classify_backends_with(&profiles, &m);
        assert_eq!(ok2.len(), 2);
        assert!(bad2.is_empty());
        assert_eq!(BackendProfile::storage_stack().contracts.enabled_count(), 4);
    }
}
