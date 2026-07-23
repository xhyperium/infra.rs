# observex Round 02 发布记录

日期：2026-07-23
状态：候选变更，尚未提交或发布

> 后续版本说明：root 已在第 3 轮发布准备阶段完成 PATCH bump 至 `0.1.2`；本记录仍保留 Round 2
> 当时“版本由 root 处理”的历史范围。

## 行为变更

- `op` 按“过滤 control → trim → UTF-8 有界”语义清理，并覆盖长尾随空白。
- `ExportingInstrumentation` 隔离 exporter 的 unwind panic；`panic=abort` 不可捕获。
- 新增 `ExportingInstrumentationStats`，诊断失败调用、unwind panic 与交付状态未知事件。
- `TelemetryExporter` 固定为必须快速返回、不等待外部 I/O 的同步合同。
- `ExportError` 使用 `thiserror`，Display 全部为简体中文。

## 兼容与边界

- 新增 API 为 additive；版本更新由 root 统一处理。
- 没有 OpenTelemetry SDK/OTLP、远端 worker、timeout 或阻塞隔离。
- 本记录不是发布签名，不包含 commit/tag/checksum。
