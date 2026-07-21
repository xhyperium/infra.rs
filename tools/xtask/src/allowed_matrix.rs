//! 跨层允许依赖矩阵（spec §2 R1-R6）。

use crate::classify::Layer;
use std::collections::HashSet;

/// 跨层允许矩阵（仅 normal/build 依赖参与）。同层内部特例在 lint_deps::run 中单独校验。
pub fn allowed_targets(from: Layer) -> HashSet<Layer> {
    use Layer::*;
    match from {
        Kernel => HashSet::from([Kernel]),
        Types => HashSet::from([Kernel, Types]),
        Contract => HashSet::from([Kernel, Types]),
        Infra => HashSet::from([Kernel, Types, Contract, Infra]),
        Storage => HashSet::from([Kernel, Types, Contract, Infra, Storage]),
        Exchange => HashSet::from([Kernel, Types, Contract, Infra, Exchange]),
        Domain => HashSet::from([Kernel, Types, Domain]),
        Services => HashSet::from([Kernel, Types, Contract, Infra, Domain, Services]),
        Apps => HashSet::from([
            Kernel, Types, Contract, Infra, Storage, Exchange, Domain, Services, Apps,
        ]),
        XTask => HashSet::from([
            Kernel, Types, Contract, Infra, Storage, Exchange, Domain, XTask,
        ]),
        Legacy => HashSet::from([
            Kernel, Types, Contract, Infra, Storage, Exchange, Domain, Legacy,
        ]),
        TestSupport => HashSet::from([Kernel, Types, Contract, TestSupport]),
        Unknown => HashSet::new(),
    }
}
