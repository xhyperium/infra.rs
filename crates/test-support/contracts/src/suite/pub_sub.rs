//! PubSub 可移植核心 suite。

use crate::failure::{ContractFailure, ContractResult, ensure};
use bytes::Bytes;
use contracts::PubSub;

const C: &str = "PubSub";

/// 先建立订阅流，再发布一条消息，验证两个入口均可调用。
///
/// at-most-once 不等于必达；本 suite 不 poll 消息，也不假定 replay、顺序、唯一 ID
/// 或背压语义。更强的投递断言必须属于具体 adapter profile。
pub async fn assert_pub_sub_surface(
    pub_sub: &dyn PubSub,
    unique_channel: &str,
    payload: Bytes,
) -> ContractResult {
    ensure(C, "unique_channel", !unique_channel.is_empty(), "测试频道不得为空")?;
    let _stream = pub_sub
        .sub_channel(unique_channel)
        .await
        .map_err(|e| ContractFailure::new(C, "subscribe", format!("sub_channel 失败: {e}")))?;
    pub_sub
        .pub_message(unique_channel, payload)
        .await
        .map_err(|e| ContractFailure::new(C, "publish", format!("pub_message 失败: {e}")))
}
