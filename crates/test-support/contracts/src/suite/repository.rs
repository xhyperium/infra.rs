//! Repository 合同 suite。

use crate::failure::{ContractFailure, ContractResult, ensure};
use contracts::Repository;
use std::fmt::Debug;

const C: &str = "Repository";

/// 断言 find miss → save → find hit → upsert。
pub async fn assert_repository<T, Id>(
    repo: &dyn Repository<T, Id>,
    sample: T,
    id: Id,
    mutate: impl FnOnce(&mut T),
    same: impl Fn(&T, &T) -> bool,
) -> ContractResult
where
    T: Clone + Send + Sync + Debug,
    Id: Clone + Send + Sync,
{
    match repo.find(id.clone()).await {
        Ok(None) => {}
        Ok(Some(v)) => {
            return Err(ContractFailure::new(
                C,
                "find_missing",
                format!("期望 None，得到 Some({v:?})"),
            ));
        }
        Err(e) => {
            return Err(ContractFailure::new(C, "find_missing", format!("find 失败: {e}")));
        }
    }

    repo.save(&sample)
        .await
        .map_err(|e| ContractFailure::new(C, "save", format!("save 失败: {e}")))?;

    let got = repo
        .find(id.clone())
        .await
        .map_err(|e| ContractFailure::new(C, "find_hit", format!("find 失败: {e}")))?
        .ok_or_else(|| ContractFailure::new(C, "find_hit", "期望 Some"))?;
    ensure(C, "find_hit_eq", same(&got, &sample), format!("got={got:?} sample={sample:?}"))?;

    let mut updated = sample.clone();
    mutate(&mut updated);
    repo.save(&updated)
        .await
        .map_err(|e| ContractFailure::new(C, "upsert", format!("upsert 失败: {e}")))?;
    let got2 = repo
        .find(id)
        .await
        .map_err(|e| ContractFailure::new(C, "find_upsert", format!("find 失败: {e}")))?
        .ok_or_else(|| ContractFailure::new(C, "find_upsert", "期望 Some"))?;
    ensure(C, "upsert_eq", same(&got2, &updated), format!("got={got2:?} updated={updated:?}"))
}
