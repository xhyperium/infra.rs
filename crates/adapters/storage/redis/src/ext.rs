//! Redis 扩展：pipeline、Lua script、带 fencing 的分布式锁。

use std::time::Duration;

use kernel::{XError, XResult};
use redis::{AsyncCommands, Script};

use crate::client::RedisClient;
use crate::error_map::map_redis_result;
use crate::pool::RedisBackend;

/// 持有锁时的所有权令牌 + fencing 序号。
///
/// - `token`：随机 ownership，用于 compare-and-delete / compare-and-expire
/// - `fence`：单调 fencing token（`INCR` 于 `fence:{key}`），关键写应带此序号防脑裂
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedisLock {
    key: String,
    token: String,
    fence: u64,
}

impl RedisLock {
    /// 锁键。
    #[must_use]
    pub fn key(&self) -> &str {
        &self.key
    }

    /// 所有权 token（不记录到常规日志）。
    #[must_use]
    pub fn token(&self) -> &str {
        &self.token
    }

    /// fencing 序号（关键写应携带并校验单调递增）。
    #[must_use]
    pub fn fence(&self) -> u64 {
        self.fence
    }
}

/// 释放锁 Lua：仅 owner 可 DEL。
const RELEASE_LUA: &str = r#"
if redis.call('GET', KEYS[1]) == ARGV[1] then
  return redis.call('DEL', KEYS[1])
else
  return 0
end
"#;

/// 续租 Lua：仅 owner 可 PEXPIRE。
const EXTEND_LUA: &str = r#"
if redis.call('GET', KEYS[1]) == ARGV[1] then
  return redis.call('PEXPIRE', KEYS[1], ARGV[2])
else
  return 0
end
"#;

impl RedisClient {
    /// 执行 Lua 脚本（固定脚本体 + KEYS/ARGV；禁止拼接不可信输入进脚本源）。
    ///
    /// 驱动侧 `Script` 会缓存 SHA 并优先 `EVALSHA`（固定脚本体 → 稳定 SHA）。
    pub async fn eval_script(
        &self,
        script: &str,
        keys: &[&str],
        args: &[&[u8]],
    ) -> XResult<redis::Value> {
        if script.trim().is_empty() {
            return Err(XError::invalid("redis Lua 脚本不能为空"));
        }
        let keys: Vec<String> = keys.iter().map(|k| (*k).to_owned()).collect();
        let args: Vec<Vec<u8>> = args.iter().map(|a| a.to_vec()).collect();
        let script_body = script.to_owned();
        self.with_pool_conn(move |mut conn: RedisBackend| async move {
            let s = Script::new(&script_body);
            let mut inv = s.prepare_invoke();
            for k in &keys {
                inv.key(k);
            }
            for a in &args {
                inv.arg(a.as_slice());
            }
            map_redis_result(inv.invoke_async(&mut conn).await)
        })
        .await
    }

    /// 先 `SCRIPT LOAD` 再按 SHA 调用（显式固定 SHA 路径；禁止动态拼接脚本）。
    ///
    /// 返回 `(sha, value)`；`sha` 可缓存复用 [`Self::eval_sha`]。
    pub async fn script_load_and_eval(
        &self,
        script: &str,
        keys: &[&str],
        args: &[&[u8]],
    ) -> XResult<(String, redis::Value)> {
        if script.trim().is_empty() {
            return Err(XError::invalid("redis Lua 脚本不能为空"));
        }
        let script_body = script.to_owned();
        let sha: String = self
            .with_pool_conn({
                let body = script_body.clone();
                move |mut conn: RedisBackend| async move {
                    let sha: String = map_redis_result(
                        redis::cmd("SCRIPT").arg("LOAD").arg(body).query_async(&mut conn).await,
                    )?;
                    Ok(sha)
                }
            })
            .await?;
        let value = self.eval_sha(&sha, keys, args).await?;
        Ok((sha, value))
    }

    /// `EVALSHA`：仅接受已加载的 SHA；NoScript → [`kernel::ErrorKind::Missing`]。
    pub async fn eval_sha(
        &self,
        sha: &str,
        keys: &[&str],
        args: &[&[u8]],
    ) -> XResult<redis::Value> {
        if sha.trim().is_empty() {
            return Err(XError::invalid("EVALSHA sha 不能为空"));
        }
        let sha = sha.to_owned();
        let keys: Vec<String> = keys.iter().map(|k| (*k).to_owned()).collect();
        let args: Vec<Vec<u8>> = args.iter().map(|a| a.to_vec()).collect();
        self.with_pool_conn(move |mut conn: RedisBackend| async move {
            let mut cmd = redis::cmd("EVALSHA");
            cmd.arg(&sha).arg(keys.len());
            for k in &keys {
                cmd.arg(k);
            }
            for a in &args {
                cmd.arg(a.as_slice());
            }
            map_redis_result(cmd.query_async(&mut conn).await)
        })
        .await
    }

    /// 管道批量 `SET`（可选统一 TTL）；单次网络往返。
    ///
    /// 跨 slot（Cluster）不承诺原子性。
    pub async fn pipeline_set(
        &self,
        items: &[(&str, Vec<u8>)],
        ttl: Option<Duration>,
    ) -> XResult<()> {
        if items.is_empty() {
            return Ok(());
        }
        if let Some(t) = ttl {
            if t.is_zero() || t.as_millis() == 0 {
                return Err(XError::invalid("pipeline TTL 不能为 0 或亚毫秒"));
            }
        }
        let owned: Vec<(String, Vec<u8>)> =
            items.iter().map(|(k, v)| ((*k).to_owned(), v.clone())).collect();
        let ttl_ms = match ttl {
            None => None,
            Some(t) => Some(u64::try_from(t.as_millis()).map_err(|_| XError::invalid("TTL 过大"))?),
        };
        self.with_pool_conn(move |mut conn: RedisBackend| async move {
            let mut pipe = redis::pipe();
            pipe.atomic();
            for (k, v) in &owned {
                if let Some(ms) = ttl_ms {
                    pipe.cmd("PSETEX").arg(k).arg(ms).arg(v.as_slice()).ignore();
                } else {
                    pipe.set(k.as_str(), v.as_slice()).ignore();
                }
            }
            let _: () = map_redis_result(pipe.query_async(&mut conn).await)?;
            Ok(())
        })
        .await
    }

    /// 获取分布式锁：`SET key token NX PX ttl` + fencing `INCR fence:{key}`。
    ///
    /// 竞争失败返回 [`XError::conflict`]。**不**提供「锁即正确」业务保证；关键写须携带
    /// [`RedisLock::fence`]。
    pub async fn lock_acquire(&self, key: &str, ttl: Duration) -> XResult<RedisLock> {
        if key.is_empty() {
            return Err(XError::invalid("锁 key 不能为空"));
        }
        if ttl.is_zero() || ttl.as_millis() == 0 {
            return Err(XError::invalid("锁 TTL 不能为 0 或亚毫秒"));
        }
        let ms = u64::try_from(ttl.as_millis()).map_err(|_| XError::invalid("锁 TTL 过大"))?;
        let token = random_token();
        let key_owned = key.to_owned();
        let fence_key = format!("fence:{key}");

        let acquired = self
            .with_pool_conn({
                let key = key_owned.clone();
                let token = token.clone();
                move |mut conn: RedisBackend| async move {
                    // SET key token NX PX ms → Some("OK") / None
                    let r: Option<String> = map_redis_result(
                        redis::cmd("SET")
                            .arg(&key)
                            .arg(&token)
                            .arg("NX")
                            .arg("PX")
                            .arg(ms)
                            .query_async(&mut conn)
                            .await,
                    )?;
                    Ok(r.is_some())
                }
            })
            .await?;

        if !acquired {
            return Err(XError::conflict(format!("redis 锁竞争失败: {key_owned}")));
        }

        let fence: i64 = self
            .with_pool_conn(move |mut conn: RedisBackend| async move {
                let n: i64 = map_redis_result(conn.incr(fence_key, 1i64).await)?;
                Ok(n)
            })
            .await?;

        if fence < 0 {
            return Err(XError::internal(format!("redis fencing 异常: {fence}")));
        }

        Ok(RedisLock { key: key_owned, token, fence: fence as u64 })
    }

    /// 释放锁（compare-and-delete）；非 owner 返回 `Ok(false)`。
    pub async fn lock_release(&self, lock: &RedisLock) -> XResult<bool> {
        let v =
            self.eval_script(RELEASE_LUA, &[lock.key.as_str()], &[lock.token.as_bytes()]).await?;
        Ok(redis_value_as_i64(&v)? > 0)
    }

    /// 续租（compare-and-expire）；非 owner 返回 `Ok(false)`。
    pub async fn lock_extend(&self, lock: &RedisLock, ttl: Duration) -> XResult<bool> {
        if ttl.is_zero() || ttl.as_millis() == 0 {
            return Err(XError::invalid("续租 TTL 不能为 0 或亚毫秒"));
        }
        let ms = u64::try_from(ttl.as_millis()).map_err(|_| XError::invalid("续租 TTL 过大"))?;
        let ms_bytes = ms.to_string().into_bytes();
        let v = self
            .eval_script(
                EXTEND_LUA,
                &[lock.key.as_str()],
                &[lock.token.as_bytes(), ms_bytes.as_slice()],
            )
            .await?;
        Ok(redis_value_as_i64(&v)? > 0)
    }
}

fn random_token() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    // 进程内唯一性足够：时间 + 地址扰动（无外部依赖）
    format!("lk-{nanos:x}-{:x}", std::process::id())
}

fn redis_value_as_i64(v: &redis::Value) -> XResult<i64> {
    match v {
        redis::Value::Int(n) => Ok(*n),
        redis::Value::Okay => Ok(1),
        redis::Value::Nil => Ok(0),
        redis::Value::BulkString(b) => std::str::from_utf8(b)
            .ok()
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| XError::internal("redis Lua 返回值无法解析为整数")),
        redis::Value::SimpleString(s) => {
            s.parse().map_err(|_| XError::internal(format!("redis Lua status 无法解析: {s}")))
        }
        _ => Err(XError::internal("redis Lua 返回了非整数类型")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pool::RedisPool;
    use kernel::ErrorKind;
    use std::sync::Arc;
    use std::sync::atomic::AtomicUsize;

    #[tokio::test]
    async fn lock_ttl_zero_rejects() {
        let client = RedisPool::test_probe(Arc::new(AtomicUsize::new(0))).client();
        let err = client.lock_acquire("k", Duration::ZERO).await.expect_err("ttl0");
        assert_eq!(err.kind(), ErrorKind::Invalid);
    }

    #[tokio::test]
    async fn empty_script_rejects() {
        let client = RedisPool::test_probe(Arc::new(AtomicUsize::new(0))).client();
        let err = client.eval_script("  ", &[], &[]).await.expect_err("empty");
        assert_eq!(err.kind(), ErrorKind::Invalid);
    }

    #[tokio::test]
    async fn pipeline_empty_ok() {
        let client = RedisPool::test_probe(Arc::new(AtomicUsize::new(0))).client();
        client.pipeline_set(&[], None).await.expect("empty");
    }

    #[test]
    fn lock_accessors() {
        let lock = RedisLock { key: "k".into(), token: "t".into(), fence: 7 };
        assert_eq!(lock.key(), "k");
        assert_eq!(lock.token(), "t");
        assert_eq!(lock.fence(), 7);
    }
}
