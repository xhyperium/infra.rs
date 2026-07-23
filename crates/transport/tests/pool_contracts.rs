//! HttpClientPool 公共配置与 RAII lease 合同。

use transportx::{HttpClientPool, PoolConfig, TransportError};

#[test]
fn invalid_pool_config_is_rejected() {
    let zero = HttpClientPool::<u32>::try_new(PoolConfig::new(0, 0)).expect_err("zero pool");
    assert!(matches!(zero, TransportError::ProtocolViolation(_)));

    let idle_over_size =
        HttpClientPool::<u32>::try_new(PoolConfig::new(1, 2)).expect_err("idle over size");
    assert!(matches!(idle_over_size, TransportError::ProtocolViolation(_)));
}

#[test]
fn lease_drop_returns_object_and_recovers_permit() {
    let pool = HttpClientPool::try_new(PoolConfig::new(1, 1)).expect("valid pool");
    {
        let mut lease = pool.checkout_lease_with(|| Ok(7_u32)).expect("lease");
        assert_eq!(lease.get(), Some(&7));
        *lease.get_mut().expect("lease 内对象存在") = 9;
        assert_eq!(pool.checked_out(), 1);
    }
    assert_eq!(pool.checked_out(), 0, "Drop 必须释放许可");
    assert_eq!(pool.idle_len(), 1, "Drop 必须归还对象");

    let lease = pool.checkout_lease_with(|| Ok(99)).expect("reused lease");
    assert_eq!(lease.get(), Some(&9), "必须复用上一 lease 归还的对象");
}

#[test]
fn taking_item_releases_permit_exactly_once_without_returning_idle() {
    let pool = HttpClientPool::try_new(PoolConfig::new(1, 1)).expect("valid pool");
    let lease = pool.checkout_lease_with(|| Ok(5_u32)).expect("lease");
    let item = lease.into_inner();
    assert_eq!(item, Some(5));
    assert_eq!(pool.checked_out(), 0);
    assert_eq!(pool.idle_len(), 0);
    let next = pool.checkout_lease_with(|| Ok(6_u32)).expect("permit released once");
    assert_eq!(next.get(), Some(&6));
}
