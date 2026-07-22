//! ObjectStore 精确 roundtrip 合同 suite。

use crate::failure::{ContractFailure, ContractResult, ensure};
use crate::fixture::FixtureNamespace;
use bytes::Bytes;
use contracts::ObjectStore;

const C: &str = "ObjectStore";

/// 断言同一隔离 key 的 put/get 返回精确 payload。
///
/// 本 suite 不声明持久化时长、覆盖、删除、列表或跨进程一致性语义。
pub async fn assert_object_store(
    store: &dyn ObjectStore,
    fixture: &FixtureNamespace,
) -> ContractResult {
    let key = fixture.resource("object_store_roundtrip");
    let expected = Bytes::from_static(b"contract-testkit-object-v1");
    store
        .put_object(&key, expected.clone())
        .await
        .map_err(|error| ContractFailure::new(C, "put", format!("put_object 失败: {error}")))?;
    let actual = store
        .get_object(&key)
        .await
        .map_err(|error| ContractFailure::new(C, "get", format!("get_object 失败: {error}")))?;
    ensure(
        C,
        "roundtrip_payload",
        actual == expected,
        format!("roundtrip payload 不一致: {actual:?}"),
    )
}
