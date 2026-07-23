//! redis crate 错误 → [`kernel::XError`] 映射。

use kernel::{XError, XResult};

/// 将 `redis::RedisError` 映射为可分类的 [`XError`]。
///
/// 分类遵循 draft §2.7：LOADING/TRYAGAIN/IO → Transient；认证/无节点 → Unavailable；
/// 客户端配置 → Invalid；未分类 → Internal。
pub fn map_redis_error(err: redis::RedisError) -> XError {
    use redis::ErrorKind as Rk;

    let msg = err.to_string();
    let mapped = match err.kind() {
        Rk::InvalidClientConfig | Rk::ClientError => {
            XError::invalid(format!("redis 客户端参数非法: {msg}"))
        }
        Rk::AuthenticationFailed => XError::unavailable(format!("redis 认证失败: {msg}")),
        Rk::BusyLoadingError | Rk::TryAgain | Rk::IoError => {
            XError::transient(format!("redis 可恢复故障: {msg}"))
        }
        Rk::ClusterDown
        | Rk::MasterDown
        | Rk::ReadOnly
        | Rk::ClusterConnectionNotFound
        | Rk::EmptySentinelList
        | Rk::MasterNameNotFoundBySentinel
        | Rk::NoValidReplicasFoundBySentinel => {
            XError::unavailable(format!("redis 节点不可用: {msg}"))
        }
        Rk::Moved | Rk::Ask | Rk::CrossSlot => {
            // P0 单机路径不应见到；归为 Transient 以便上层有限重试后升级
            XError::transient(format!("redis 集群重定向/跨 slot: {msg}"))
        }
        Rk::ExecAbortError => XError::conflict(format!("redis 执行中止: {msg}")),
        Rk::NoScriptError => XError::missing(format!("redis 脚本不存在: {msg}")),
        Rk::TypeError => XError::internal(format!("redis 类型/协议错误: {msg}")),
        Rk::ResponseError => {
            let upper = msg.to_ascii_uppercase();
            if upper.contains("LOADING") || upper.contains("TRYAGAIN") {
                XError::transient(format!("redis 响应可恢复: {msg}"))
            } else if upper.contains("NOAUTH") || upper.contains("WRONGPASS") {
                XError::unavailable(format!("redis 认证失败: {msg}"))
            } else if upper.contains("READONLY") {
                XError::unavailable(format!("redis 只读: {msg}"))
            } else {
                XError::internal(format!("redis 响应错误: {msg}"))
            }
        }
        Rk::ExtensionError | Rk::NotBusy => {
            let upper = msg.to_ascii_uppercase();
            if upper.contains("NOAUTH") || upper.contains("WRONGPASS") {
                XError::unavailable(format!("redis 认证失败: {msg}"))
            } else if upper.contains("LOADING") || upper.contains("TRYAGAIN") {
                XError::transient(format!("redis 可恢复故障: {msg}"))
            } else {
                XError::internal(format!("redis 扩展错误: {msg}"))
            }
        }
        _ => XError::internal(format!("redis 未分类错误: {msg}")),
    };

    mapped.with_source(err)
}

/// 将 redis 结果映射为 [`XResult`]。
#[inline]
pub fn map_redis_result<T>(result: redis::RedisResult<T>) -> XResult<T> {
    result.map_err(map_redis_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::ErrorKind;

    #[test]
    fn maps_busy_loading_to_transient() {
        let err = redis::RedisError::from((redis::ErrorKind::BusyLoadingError, "loading"));
        let x = map_redis_error(err);
        assert_eq!(x.kind(), ErrorKind::Transient);
    }

    #[test]
    fn maps_auth_to_unavailable() {
        let err = redis::RedisError::from((redis::ErrorKind::AuthenticationFailed, "auth"));
        let x = map_redis_error(err);
        assert_eq!(x.kind(), ErrorKind::Unavailable);
    }

    #[test]
    fn maps_invalid_config_to_invalid() {
        let err = redis::RedisError::from((redis::ErrorKind::InvalidClientConfig, "bad cfg"));
        let x = map_redis_error(err);
        assert_eq!(x.kind(), ErrorKind::Invalid);
    }

    #[test]
    fn maps_cluster_down_to_unavailable() {
        let err = redis::RedisError::from((redis::ErrorKind::ClusterDown, "cluster down"));
        assert_eq!(map_redis_error(err).kind(), ErrorKind::Unavailable);
    }

    #[test]
    fn maps_moved_to_transient() {
        let err = redis::RedisError::from((redis::ErrorKind::Moved, "MOVED 3999 127.0.0.1:7001"));
        assert_eq!(map_redis_error(err).kind(), ErrorKind::Transient);
    }

    #[test]
    fn maps_exec_abort_to_conflict() {
        let err = redis::RedisError::from((redis::ErrorKind::ExecAbortError, "EXECABORT"));
        assert_eq!(map_redis_error(err).kind(), ErrorKind::Conflict);
    }

    #[test]
    fn maps_no_script_to_missing() {
        let err = redis::RedisError::from((redis::ErrorKind::NoScriptError, "NOSCRIPT"));
        assert_eq!(map_redis_error(err).kind(), ErrorKind::Missing);
    }

    #[test]
    fn maps_response_loading_text_to_transient() {
        let err =
            redis::RedisError::from((redis::ErrorKind::ResponseError, "LOADING Redis is loading"));
        assert_eq!(map_redis_error(err).kind(), ErrorKind::Transient);
    }

    #[test]
    fn maps_response_noauth_to_unavailable() {
        let err = redis::RedisError::from((
            redis::ErrorKind::ResponseError,
            "NOAUTH Authentication required",
        ));
        assert_eq!(map_redis_error(err).kind(), ErrorKind::Unavailable);
    }

    #[test]
    fn map_redis_result_preserves_ok_and_err_paths() {
        assert_eq!(map_redis_result(Ok(7usize)).expect("ok"), 7);
        let err = redis::RedisError::from((redis::ErrorKind::IoError, "io"));
        let mapped = map_redis_result::<()>(Err(err)).expect_err("err");
        assert_eq!(mapped.kind(), ErrorKind::Transient);
    }
}
