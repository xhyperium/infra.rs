# 安全策略

## 漏洞报告

发现安全漏洞时，**请勿在 GitHub Issue 中公开报告**。

### 报告流程

1. 私下发送邮件至维护者：在 GitHub 上联系 `@ZoneCNH` 或 `@liukongqiang5`
2. 邮件标题以 `[SECURITY]` 开头，描述以下内容：
   - 漏洞类型（如 RCE、XSS、SQL 注入、供应链投毒、密钥泄露等）
   - 受影响的 crate / 模块路径
   - 复现步骤或 PoC
   - 影响评估（影响的用户/数据范围）
3. 维护者在 **48 小时内**确认收到
4. 双方协商修复时间线，通常：
   - CRITICAL：7 天内发布修复版本
   - HIGH：14 天内发布修复版本
   - MEDIUM/LOW：下一个常规版本

### 报告时请勿

- 在公开 Issue / PR / Discussion 中披露漏洞细节
- 尝试在生产环境中利用漏洞
- 访问或修改他人数据

## 支持版本

| 版本范围 | 安全修复支持 |
|----------|-------------|
| `main` 最新 commit | ✅ 支持 |
| 最新 tag release | ✅ 支持 |
| 历史 release | ⚠️ 尽力而为 |

本项目处于 `0.x` 阶段，不保证向后兼容。建议始终使用最新版本。

## 安全基线

### 供应链安全

- `cargo deny check` 在每次 PR 和每周定期扫描中运行
- `cargo-audit` 漏洞数据库每周一自动更新
- 依赖禁止内联版本（`[workspace.dependencies]` 集中管理 + CI 门禁）
- Secret scanning + push protection 已在 GitHub 仓库级别启用

### 敏感信息

- `.env`、`.claude/*.local.json`、`.codex/config.toml` 等含凭据文件已 `.gitignore`
- `.claude/hooks/pre-tool-check.mjs` 在 Agent 工具调用层硬拦截 `.env` 写入
- 日志中禁止出现完整 token / 密码 / 私钥（见 `rust-dev-rules.md` 日志安全条款）
- 生产签核须 Maintainer 手写 `Signed-off-by`，Agent 不得代签

### AI Agent 安全约束

- AI 不可 approve / merge PR（宪章 §7.1）
- AI 不可直推 main、不可修改 CODEOWNERS
- AI 不可绕过强制门禁（`--no-verify`）
- `pr-auto-approve` 仅在用户明确要求时由第二身份执行，禁止 self-approve

## 致谢

感谢负责任披露的安全研究者。修复后的漏洞将在 CHANGELOG 中致谢（除非报告者要求匿名）。
