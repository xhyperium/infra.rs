//! ObjectStore 可移植核心 suite。

use crate::failure::{ContractFailure, ContractResult, ensure};
use bytes::Bytes;
use contracts::ObjectStore;

const C: &str = "ObjectStore";

/// 对调用方提供的唯一 key 执行 put/get，并断言字节精确往返。
///
/// 本 suite 不假定 missing、overwrite、metadata 或清理语义；真实后端调用方负责
/// 在 suite 返回后删除测试对象。
pub async fn assert_object_store(
    store: &dyn ObjectStore,
    unique_key: &str,
    payload: Bytes,
) -> ContractResult {
    ensure(C, "unique_key", !unique_key.is_empty(), "测试 key 不得为空")?;
    store
        .put_object(unique_key, payload.clone())
        .await
        .map_err(|e| ContractFailure::new(C, "put", format!("put_object 失败: {e}")))?;
    let got = store
        .get_object(unique_key)
        .await
        .map_err(|e| ContractFailure::new(C, "get", format!("get_object 失败: {e}")))?;
    ensure(C, "roundtrip", got == payload, "对象字节往返不一致")
}
