//! PubSub 操作可调用性 smoke suite。

use crate::failure::{ContractFailure, ContractResult};
use crate::fixture::FixtureNamespace;
use bytes::Bytes;
use contracts::PubSub;

const C: &str = "PubSub";

/// 断言调用方提供的唯一 channel 的 subscribe 与 publish 操作均成功返回。
///
/// 本函数不轮询订阅流，也不证明必达、重放、顺序、确认、背压或投递次数。
///
/// # Errors
///
/// channel 为空，或 subscribe/publish 调用失败时返回 [`ContractFailure`]。
pub async fn assert_pub_sub_surface(
    pub_sub: &dyn PubSub,
    unique_channel: &str,
    payload: Bytes,
) -> ContractResult {
    if unique_channel.is_empty() {
        return Err(ContractFailure::new(C, "unique_channel", "测试频道不得为空"));
    }
    let stream = pub_sub
        .sub_channel(unique_channel)
        .await
        .map_err(|error| ContractFailure::new(C, "subscribe", format!("订阅失败: {error}")))?;
    pub_sub
        .pub_message(unique_channel, payload)
        .await
        .map_err(|error| ContractFailure::new(C, "publish", format!("发布失败: {error}")))?;
    let _ = stream;
    Ok(())
}

/// 使用确定性 fixture 运行 [`assert_pub_sub_surface`]。
///
/// # Errors
///
/// 资源名派生失败或 surface suite 失败时返回 [`ContractFailure`]。
pub async fn assert_pub_sub_smoke(
    pub_sub: &dyn PubSub,
    fixture: &FixtureNamespace,
) -> ContractResult {
    let channel = fixture.resource("pub_sub_smoke")?;
    assert_pub_sub_surface(pub_sub, &channel, Bytes::from_static(b"contract-testkit-pub-sub-v1"))
        .await
}
