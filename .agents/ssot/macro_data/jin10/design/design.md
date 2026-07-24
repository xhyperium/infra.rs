<!-- ssot:trace=jin10.design.001 -->
# jin10 — 离线设计

当前为 `draft`/`not_started`，权限、协议、认证、限流、缓存和再分发合同均为 `UNKNOWN`。本设计只定义离线 fixture 的边界，不设计或指导真实网络客户端。

## 设计边界

1. 输入适配器只读取脱敏 fixture，保留来源事件 ID、时间、语言、缺失原因和原始字段。
2. 解析器使用纯函数完成 JSON 结构、字段长度、时间顺序、重复身份和未知事件校验。
3. 权限未知时返回稳定的 `access_denied` 语义；不得猜测 endpoint、Header、Query、WebSocket 地址、配额、重试或缓存策略。
4. secret sentinel 只用于离线安全测试，禁止进入 Debug、Display、Serialize、URL、错误、tracing 和原始响应。

## 未来变更门

若供应商提供可复核合同，必须先完成来源、授权、许可、字段、限流、脱敏、停止条件和回滚审查，再单独提交网络适配器设计。获批前不得创建 crate、加入依赖、访问外部服务或提交联网验收。
