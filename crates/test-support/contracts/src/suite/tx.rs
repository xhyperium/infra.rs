//! TxRunner / TxContext 合同 suite。

use crate::failure::{ContractFailure, ContractResult, ensure};
use contracts::{TxRunner, run_tx_commit_on_ok};
use kernel::{ErrorKind, XError};

const C: &str = "TxRunner";

/// 断言 Ok→commit、Err→rollback，以及 `dyn TxRunner` 对象安全 begin_tx。
pub async fn assert_tx_runner(runner: &dyn TxRunner) -> ContractResult {
    let n = run_tx_commit_on_ok(runner, |_ctx| async move { Ok::<_, XError>(11u32) })
        .await
        .map_err(|e| ContractFailure::new(C, "commit_path", format!("Ok 路径失败: {e}")))?;
    ensure(C, "commit_value", n == 11, format!("期望 11，得到 {n}"))?;

    let err = match run_tx_commit_on_ok(runner, |_ctx| async move {
        Err::<(), _>(XError::invalid("业务校验失败"))
    })
    .await
    {
        Err(e) => e,
        Ok(()) => {
            return Err(ContractFailure::new(C, "rollback_path", "期望业务 Err，得到 Ok"));
        }
    };
    ensure(
        C,
        "rollback_kind",
        err.kind() == ErrorKind::Invalid,
        format!("期望 Invalid，得到 {:?}", err.kind()),
    )?;
    ensure(
        C,
        "rollback_context",
        err.context().contains("业务校验失败"),
        format!("错误上下文丢失: {}", err.context()),
    )?;

    let mut ctx = runner
        .begin_tx()
        .await
        .map_err(|e| ContractFailure::new(C, "begin_tx", format!("begin_tx 失败: {e}")))?;
    ctx.commit()
        .await
        .map_err(|e| ContractFailure::new(C, "direct_commit", format!("commit 失败: {e}")))?;
    Ok(())
}
