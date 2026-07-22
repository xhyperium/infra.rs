//! PubSub 操作可调用性 smoke suite。

use crate::failure::{ContractFailure, ContractResult};
use crate::fixture::FixtureNamespace;
use bytes::Bytes;
use contracts::PubSub;

const C: &str = "PubSub";

/// 断言隔离 channel 的 subscribe 与 publish 操作均成功返回。
///
/// 本函数不轮询订阅流，也不证明必达、重放、顺序、确认、背压或投递次数。
pub async fn assert_pub_sub_smoke(
    pub_sub: &dyn PubSub,
    fixture: &FixtureNamespace,
) -> ContractResult {
    let channel = fixture.resource("pub_sub_smoke");
    let stream = pub_sub
        .sub_channel(&channel)
        .await
        .map_err(|error| ContractFailure::new(C, "subscribe", format!("订阅失败: {error}")))?;
    pub_sub
        .pub_message(&channel, Bytes::from_static(b"contract-testkit-pub-sub-v1"))
        .await
        .map_err(|error| ContractFailure::new(C, "publish", format!("发布失败: {error}")))?;
    let _ = stream;
    Ok(())
}
