# gate-spec.md

> **Status: SUPERSEDED（2026-07-15）**  
> 生产方向与退役完成态见 [PLAN-GATE-RETIRE-001](../plan/xhyper-gate-retirement-complete-plan.md)（**Accepted**）与
> [ADR-016](../../../../../docs/architecture/adr/016-bootstrap-sole-composition-root.md)（**Accepted**）。  
> 物理路径 `crates/infra/gate` 与 package `xhyper-gate` **已删除**。  
> 组合根：`bootstrap` + typed `PlatformContext` / `AppContext` / `BootstrappedApp`。  
> **保留** `.agent/gates/`、`tools/archgate`、CI/release policy gates（非本 crate）。  
> 防回流：`cargo xtl no-new-gate`。  
> 执行计划包：[plan/](../plan)。

本文件保留为历史实现契约索引，**不再**描述 active package 验收标准。
下文为退役前契约摘要（只读考古）。

---

## 历史摘要（非 active）

曾定位：L0 模块注册门面与启动期能力发现（`Capability` + `Gate::register` / `resolve`）。

退役原因：字符串 Service Locator、无业务方法、与 bootstrap 双组合中心等（见 PLAN-GATE-RETIRE-001 §1）。

替代：ADR-016 bootstrap 唯一组合根 + typed fields。
