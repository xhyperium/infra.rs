# postgresx 实现规范

状态：当前 `0.3.2` 实现合同（Mock + sqlx 真实驱动已落地；真测 `#[ignore]`，未达 M3）。**未宣称 package stable。**

## 0. 权威与定位

按 Constitution → XLib spec → 已批准 ADR → 本文 → 代码裁定。**Evidence** 为直接证据，
**Inference** 只定义最低验收，**Unknown** 需评审。`postgresx` 位于
`crates/adapters/storage/postgres`，实现 `Repository` 与 `TxRunner`：
- Mock：`MockRepository` / `MockTxRunner`；
- 真实：`PgRepository` / `PgTxRunner`（`sqlx` + `PgPool`）。

非目标：在本版本承诺迁移工具链、隔离级别合同、查询 DSL，或把 ignored 真测当作 CI 生产证据。

## 1. Cargo 与版本

版本 `0.3.2`（package `postgresx`）。

| 项目 | 当前事实 |
| --- | --- |
| 普通依赖 | `kernel`、`contracts`、`async-trait`、`anyhow`、`sqlx` |
| features | 无（真实驱动始终编译） |
| dev-dependency | `tokio` |

依赖符合 R2 且无同层依赖。版本更新仅允许 `x.y.z → x.y.(z+1)`。

## 2. 当前公开 API 与行为

### 2.1 本地 trait

- `Identifiable<Id>::id(&self) -> Id`：mock 仓储按 key 存取用；**不是** contracts 跨层契约。
- `PgEntity`：真实仓储用；提供 `table_name` / `id_column` / `upsert_sql` / `bind_args`。
  属于 postgresx 实现细节，不污染 contracts（R4 Additive Only）。

### 2.2 Mock

- `MockRepository<T, Id>` 持有 `Mutex<HashMap<Id, T>>`；`find` 克隆返回，缺失 `Ok(None)`；
  `save` 按实体 id 插入或覆盖。
- `MockTxRunner`：`run_tx` 仅 await 传入 future 并原样返回，不 begin/commit/rollback。

### 2.3 真实驱动

- `PgRepository::new(pool)` / `pool()`：
  - `find`：`SELECT * FROM {table} WHERE {id_col} = $1` + `FromRow`；
  - `save`：`PgEntity::upsert_sql` + `bind_args`。
- `PgTxRunner::new(pool)`：
  - `TxRunner::run_tx`：仅提供事务边界（begin/commit/rollback），**不**把 tx 句柄传给闭包；
  - `run_tx_with`：闭包接收 `&mut Transaction`，用于同一事务内真实 SQL。

两处 mock 锁访问均 `unwrap()`，锁中毒会 panic。

## 3. 差距、错误与信任边界

- **证据**：`sqlx` 与真实实现已引入。
- **证据**：真实测试（save/find、缺失、commit、rollback）均 `#[ignore = "需要 postgres 服务（设置 DATABASE_URL）"]`；
  **不得**当作 CI 默认通过或 M3 生产证据。
- **证据**：现行 `TxRunner::run_tx` 签名不传事务句柄；真实事务内 SQL 须用 `run_tx_with`。
- **未知**：实体映射规范、迁移所有权、连接/事务超时、隔离与重试、嵌套事务、错误分类细粒度。
- 数据库输入是信任边界；生产实现必须参数化查询、最小权限、保护 DSN。

## 4. 测试、验收与追溯

Mock 测试覆盖保存/查找、缺失、覆盖、Repository trait object、future 返回等。
真实测试 `#[ignore]`。运行：

```text
cargo test -p postgresx
cargo test -p postgresx -- --ignored   # 需 Postgres；非 CI 默认
cargo clippy -p postgresx --all-targets -- -D warnings
cargo fmt -- --check
cargo run -p xtask -- lint-deps
```

验收要求：不把 `MockTxRunner` 描述为真实事务；不把 ignored 真测宣称为生产就绪；
API/Cargo/测试一致；contracts 变更先评审；版本更新遵守精确 patch 规则。

追溯：XLib spec §§2 R2/R6、4.3、4.5、5；
`crates/contracts/src/lib.rs`；`crates/adapters/storage/postgres/{Cargo.toml,src/lib.rs,README.md}`。
