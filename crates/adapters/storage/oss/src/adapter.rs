//! OSS 内存 scaffold：`ObjectStore`。

use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use bytes::Bytes;
use contracts::ObjectStore;
use kernel::{XError, XResult};

pub struct OssAdapter {
    name: String,
    endpoint: String,
    objects: Mutex<HashMap<String, Bytes>>,
}

impl OssAdapter {
    pub fn new(name: impl Into<String>, endpoint: impl Into<String>) -> Self {
        Self { name: name.into(), endpoint: endpoint.into(), objects: Mutex::new(HashMap::new()) }
    }

    pub fn local() -> Self {
        Self::new("oss-local", "s3://localhost/bucket")
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    fn lock(&self) -> XResult<std::sync::MutexGuard<'_, HashMap<String, Bytes>>> {
        self.objects.lock().map_err(|e| XError::internal(format!("objects lock poisoned: {e}")))
    }
}

#[async_trait]
impl ObjectStore for OssAdapter {
    async fn put_object(&self, key: &str, data: Bytes) -> XResult<()> {
        self.lock()?.insert(key.to_string(), data);
        Ok(())
    }

    async fn get_object(&self, key: &str) -> XResult<Bytes> {
        self.lock()?
            .get(key)
            .cloned()
            .ok_or_else(|| XError::missing(format!("object not found: {key}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn put_get() {
        let a = OssAdapter::local();
        a.put_object("k", Bytes::from_static(b"v")).await.expect("put");
        assert_eq!(a.get_object("k").await.expect("get"), Bytes::from_static(b"v"));
    }

    #[tokio::test]
    async fn missing() {
        let a = OssAdapter::local();
        assert!(a.get_object("nope").await.is_err());
    }
}
