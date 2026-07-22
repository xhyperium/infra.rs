# AGENTS — contracts

- Package: contracts · lib: contracts
- 只放 trait/type；Additive Only
- 依赖: kernel + canonical + async-trait/bytes/futures-core/thiserror
- `thiserror` 仅用于公开类型化错误派生，遵循组织 Rust P0 与 DDR-002

## contract-testkit

- Fake/suite 在 `crates/test-support/contracts`（package `contract-testkit`）
- 本 crate **禁止** unit 测试依赖 contract-testkit（双版本陷阱）
- integration tests / examples 可用 dev-dep
