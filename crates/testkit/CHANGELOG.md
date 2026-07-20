# Changelog — testkit

本文件记录 testkit 的用户可见变更，遵循 [Keep a Changelog](https://keepachangelog.com/)。

## [Unreleased]

### 新增

- 将 `xhyper-testkit` 0.1.1（ManualClock V2）移植到 **infra.rs** workspace
- 合同 / 并发 / property 测试与 unit tests 一并落地

## [0.1.1] - 2026-07-14（上游 xhyper.rs ship；infra 移植保留版本号）

### 新增

- `ManualClock` V2：`Mutex` 状态模型；墙钟/单调钟 checked 控制 API；wall fault；`snapshot`；无 `Default` / `Clone`
- 合同/并发/property 测试
- crate 属性：`forbid(unsafe_code)`、`deny(missing_docs)`、`deny(unreachable_pub)`；`publish = false`

### 移除（相对历史草案）

- `xlib_test!`、`mock!`、`FixtureBuilder`、`provider_capability_contract_tests!`
