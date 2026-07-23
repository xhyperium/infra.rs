//! ObjectStore 可移植核心 suite。

use crate::failure::{ContractFailure, ContractResult, ensure};
use crate::fixture::FixtureNamespace;
use bytes::Bytes;
use contracts::ObjectStore;

const C: &str = "ObjectStore";

/// 对调用方提供的唯一 key 执行 put/get，并断言字节精确往返。
///
/// 本 suite 不假定 missing、overwrite、metadata、清理、持久化时长或跨进程一致性；
/// 真实后端调用方负责在 suite 返回后删除测试对象。
///
/// # Errors
///
/// key 为空、后端调用失败或读回 payload 不一致时返回 [`ContractFailure`]。
pub async fn assert_object_store(
    store: &dyn ObjectStore,
    unique_key: &str,
    payload: Bytes,
) -> ContractResult {
    ensure(C, "unique_key", !unique_key.is_empty(), "测试 key 不得为空")?;
    store
        .put_object(unique_key, payload.clone())
        .await
        .map_err(|error| ContractFailure::new(C, "put", format!("put_object 失败: {error}")))?;
    let actual = store
        .get_object(unique_key)
        .await
        .map_err(|error| ContractFailure::new(C, "get", format!("get_object 失败: {error}")))?;
    ensure(C, "roundtrip", actual == payload, format!("roundtrip payload 不一致: {actual:?}"))
}

/// 使用确定性 fixture 派生 key 并运行 [`assert_object_store`]。
///
/// # Errors
///
/// 资源名派生失败或核心 suite 失败时返回 [`ContractFailure`]。
pub async fn assert_object_store_with_fixture(
    store: &dyn ObjectStore,
    fixture: &FixtureNamespace,
) -> ContractResult {
    let key = fixture.resource("object_store_roundtrip")?;
    assert_object_store(store, &key, Bytes::from_static(b"contract-testkit-object-v1")).await
}
