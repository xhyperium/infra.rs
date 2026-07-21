//! `kafkax` — kafka 存储适配器。
//!
//! 实现 `StorageAdapter` trait。scaffold 使用进程内 HashMap 模拟 KV，
//! **非**真实 kafka 客户端。

use std::collections::HashMap;
use std::sync::Mutex;

use crate::{AdapterState, Error, Result, StorageAdapter};

/// kafka 存储适配器（内存 scaffold）。
pub struct KafkaAdapter {
    name: String,
    state: AdapterState,
    endpoint: String,
    store: Mutex<HashMap<String, Vec<u8>>>,
}

impl KafkaAdapter {
    /// 创建适配器。
    pub fn new(name: impl Into<String>, endpoint: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            state: AdapterState::Uninitialized,
            endpoint: endpoint.into(),
            store: Mutex::new(HashMap::new()),
        }
    }

    /// 默认本地 endpoint。
    pub fn local() -> Self {
        Self::new("kafka-local", "localhost:9092")
    }

    /// 配置的 endpoint（scaffold 观测用）。
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    fn require_connected(&self) -> Result<()> {
        if self.state != AdapterState::Connected {
            return Err(Error::NotConnected);
        }
        Ok(())
    }
}

impl StorageAdapter for KafkaAdapter {
    fn name(&self) -> &str {
        &self.name
    }

    fn connect(&mut self) -> Result<()> {
        if self.state == AdapterState::Connected {
            return Err(Error::AlreadyConnected);
        }
        self.state = AdapterState::Connected;
        Ok(())
    }

    fn disconnect(&mut self) -> Result<()> {
        if self.state != AdapterState::Connected {
            return Err(Error::NotConnected);
        }
        self.state = AdapterState::Disconnected;
        Ok(())
    }

    fn state(&self) -> AdapterState {
        self.state
    }

    fn write(&self, key: &str, value: &[u8]) -> Result<()> {
        self.require_connected()?;
        let mut guard =
            self.store.lock().map_err(|e| Error::Internal(format!("store lock poisoned: {e}")))?;
        guard.insert(key.to_string(), value.to_vec());
        Ok(())
    }

    fn read(&self, key: &str) -> Result<Option<Vec<u8>>> {
        self.require_connected()?;
        let guard =
            self.store.lock().map_err(|e| Error::Internal(format!("store lock poisoned: {e}")))?;
        Ok(guard.get(key).cloned())
    }

    fn delete(&self, key: &str) -> Result<()> {
        self.require_connected()?;
        let mut guard =
            self.store.lock().map_err(|e| Error::Internal(format!("store lock poisoned: {e}")))?;
        guard.remove(key);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StorageAdapter;

    #[test]
    fn connect_disconnect() {
        let mut a = KafkaAdapter::local();
        assert_eq!(a.state(), AdapterState::Uninitialized);
        a.connect().expect("connect");
        assert_eq!(a.state(), AdapterState::Connected);
        a.disconnect().expect("disconnect");
        assert_eq!(a.state(), AdapterState::Disconnected);
    }

    #[test]
    fn double_connect_fails() {
        let mut a = KafkaAdapter::local();
        a.connect().expect("connect");
        assert!(a.connect().is_err());
    }

    #[test]
    fn ops_require_connect() {
        let a = KafkaAdapter::local();
        assert!(a.write("k", b"v").is_err());
        assert!(a.read("k").is_err());
        assert!(a.delete("k").is_err());
    }

    #[test]
    fn write_read_delete_roundtrip() {
        let mut a = KafkaAdapter::local();
        a.connect().expect("connect");
        a.write("k1", b"hello").expect("write");
        assert_eq!(a.read("k1").expect("read"), Some(b"hello".to_vec()));
        a.delete("k1").expect("delete");
        assert_eq!(a.read("k1").expect("read after del"), None);
    }

    #[test]
    fn name_and_endpoint() {
        let a = KafkaAdapter::local();
        assert_eq!(a.name(), "kafka-local");
        assert_eq!(a.endpoint(), "localhost:9092");
    }
}
