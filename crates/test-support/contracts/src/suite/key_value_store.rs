//! KeyValueStore 合同 suite。

use crate::failure::{ContractResult, ensure};
use contracts::KeyValueStore;
use std::time::Duration;

const C: &str = "KeyValueStore";

/// 断言实现满足最小 KV 合同：miss → set → hit。
pub async fn assert_key_value_store(store: &dyn KeyValueStore) -> ContractResult {
    match store.get("missing").await {
        Ok(None) => {}
        Ok(Some(_)) => {
            return Err(crate::failure::ContractFailure::new(
                C,
                "get_missing",
                "期望 None，得到 Some",
            ));
        }
        Err(e) => {
            return Err(crate::failure::ContractFailure::new(
                C,
                "get_missing",
                format!("get 失败: {e}"),
            ));
        }
    }

    store
        .set("k", b"hello".to_vec(), Some(Duration::from_secs(30)))
        .await
        .map_err(|e| crate::failure::ContractFailure::new(C, "set", format!("set 失败: {e}")))?;

    let v = store
        .get("k")
        .await
        .map_err(|e| crate::failure::ContractFailure::new(C, "get_hit", format!("get 失败: {e}")))?
        .ok_or_else(|| crate::failure::ContractFailure::new(C, "get_hit", "期望 Some"))?;

    ensure(C, "get_hit_value", v == b"hello", format!("期望 b\"hello\"，得到 {v:?}"))
}
