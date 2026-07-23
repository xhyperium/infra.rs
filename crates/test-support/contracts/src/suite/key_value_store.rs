//! KeyValueStore 合同 suite。

use crate::failure::{ContractFailure, ContractResult, ensure};
use crate::fixture::FixtureNamespace;
use contracts::KeyValueStore;
use std::time::Duration;

const C: &str = "KeyValueStore";

/// 断言实现满足最小 KV 合同：miss → set → hit。
///
/// # Errors
///
/// 后端调用失败或 roundtrip 不满足最小合同时返回 [`ContractFailure`]。
pub async fn assert_key_value_store(store: &dyn KeyValueStore) -> ContractResult {
    assert_key_value_store_keys(store, "missing", "k").await
}

/// 使用确定性 fixture 隔离 miss/hit key 后运行最小 KV 合同。
///
/// # Errors
///
/// 资源名派生失败、后端调用失败或 roundtrip 不满足最小合同时返回 [`ContractFailure`]。
pub async fn assert_key_value_store_isolated(
    store: &dyn KeyValueStore,
    fixture: &FixtureNamespace,
) -> ContractResult {
    let missing_key = fixture.resource("key_value_missing")?;
    let hit_key = fixture.resource("key_value_hit")?;
    assert_key_value_store_keys(store, &missing_key, &hit_key).await
}

async fn assert_key_value_store_keys(
    store: &dyn KeyValueStore,
    missing_key: &str,
    hit_key: &str,
) -> ContractResult {
    match store.get(missing_key).await {
        Ok(None) => {}
        Ok(Some(_)) => {
            return Err(ContractFailure::new(C, "get_missing", "期望 None，得到 Some"));
        }
        Err(e) => {
            return Err(ContractFailure::new(C, "get_missing", format!("get 失败: {e}")));
        }
    }

    store
        .set(hit_key, b"hello".to_vec(), Some(Duration::from_secs(30)))
        .await
        .map_err(|e| ContractFailure::new(C, "set", format!("set 失败: {e}")))?;

    let v = store
        .get(hit_key)
        .await
        .map_err(|e| ContractFailure::new(C, "get_hit", format!("get 失败: {e}")))?
        .ok_or_else(|| ContractFailure::new(C, "get_hit", "期望 Some"))?;

    ensure(C, "get_hit_value", v == b"hello", format!("期望 b\"hello\"，得到 {v:?}"))
}
