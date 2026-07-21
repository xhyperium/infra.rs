//! Repository Fake。

use async_trait::async_trait;
use contracts::Repository;
use kernel::{XError, XResult};
use std::collections::HashMap;
use std::sync::Mutex;

/// 简单内存 [`Repository`]（`Id: Eq + Hash + Clone + Send + Sync`，`T: Clone + Send + Sync`）。
///
/// `save` 要求调用方提供 `id_of` 提取函数（构造时注入），避免强制 `T` 携带 id 字段。
pub struct FakeRepository<T, Id> {
    inner: Mutex<HashMap<Id, T>>,
    id_of: Box<dyn Fn(&T) -> Id + Send + Sync>,
}

impl<T, Id> FakeRepository<T, Id>
where
    T: Clone + Send + Sync + 'static,
    Id: Eq + std::hash::Hash + Clone + Send + Sync + 'static,
{
    /// 以 `id_of` 提取主键。
    pub fn new<F>(id_of: F) -> Self
    where
        F: Fn(&T) -> Id + Send + Sync + 'static,
    {
        Self { inner: Mutex::new(HashMap::new()), id_of: Box::new(id_of) }
    }

    /// 当前实体数。
    pub fn len(&self) -> XResult<usize> {
        let g = self.inner.lock().map_err(|_| XError::internal("repository lock 中毒"))?;
        Ok(g.len())
    }

    /// 是否为空。
    pub fn is_empty(&self) -> XResult<bool> {
        Ok(self.len()? == 0)
    }
}

#[async_trait]
impl<T, Id> Repository<T, Id> for FakeRepository<T, Id>
where
    T: Clone + Send + Sync + 'static,
    Id: Eq + std::hash::Hash + Clone + Send + Sync + 'static,
{
    async fn find(&self, id: Id) -> XResult<Option<T>> {
        let g = self.inner.lock().map_err(|_| XError::internal("repository lock 中毒"))?;
        Ok(g.get(&id).cloned())
    }

    async fn save(&self, entity: &T) -> XResult<()> {
        let id = (self.id_of)(entity);
        let mut g = self.inner.lock().map_err(|_| XError::internal("repository lock 中毒"))?;
        g.insert(id, entity.clone());
        Ok(())
    }
}
