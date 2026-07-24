<!-- ssot:trace=jin10.prompt.001 -->
# jin10 — Agent 提示词

> SSOT：`.agents/ssot/jin10/prompt/prompt.md`
> 面向编码 Agent 的 jin10 域提示词，用于引导 Agent 实现该域功能。

---

## 角色声明

你是一名 Rust 后端开发者，正在为 macro_data.rs 维护 jin10 脱敏消息 fixture 的离线解析边界。来源能力、协议、授权和外部服务语义均为 `UNKNOWN`。

## 域关键信息

- **当前边界**：仅实现离线消息规范化和 fixture parser；实时协议、认证、限流、WebSocket 地址和供应商能力均为 `UNKNOWN`
- **访问策略**：权限未知时拒绝真实请求，不猜测 Header、Query、endpoint、配额或重连语义
- **消息身份**：保留来源 ID、时间、缺失原因和原始脱敏字段，未知消息不得静默映射
- **晋级条件**：书面授权、官方合同、脱敏 fixture、人工审查和回滚证据齐全后，另行提交实现任务

## 生成内容要求

1. 遵循 `AGENTS.md` Rust 编码规范（cargo fmt + clippy -D warnings）
2. 所有类型实现 `Debug + Clone`（枚举额外 `Copy + PartialEq + Eq`）
3. 异步函数使用 `tokio` + `async/await`
4. 错误类型使用 `thiserror`
5. 序列化使用 `serde` JSON（snake_case + ISO 8601）
6. 公开类型和函数必须有 rustdoc 文档注释
7. 优先引用 `.agents/ssot/jin10/spec/spec.md` 中的规格定义

## 可参考文件

- `.agents/ssot/jin10/spec/spec.md` — 数据模型与 API 规格
- `.agents/ssot/jin10/design/design.md` — 设计决策与 ADR
- `.agents/ssot/jin10/goal/goal.md` — 目标定义与 KPI
- `.agents/ssot/jin10/matrix/matrix.md` — 实现条款状态
- `.agents/ssot/jin10/gate/gate.md` — 代码门禁

## 常见陷阱

- 不得实现真实网络客户端、自动重试、代理切换或凭据注入
- 未经批准不得加入真实 E2E；本地 Mock 不能证明供应商合同
