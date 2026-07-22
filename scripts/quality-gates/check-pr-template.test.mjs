#!/usr/bin/env node
/**
 * check-pr-template.test.mjs — PR 模板校验脚本 L1 测试套件
 *
 * 覆盖:
 *   1. 完整模板通过验
 *   2. 缺失必填字段检测
 *   3. 部分填写模板检测
 *   4. 错误降级模式 (PR_TEMPLATE_ERROR_DOWNGRADE)
 *   5. 空 body 处理
 *   6. 边界情况: 纯空白、仅有标题、缺少勾选框
 */

import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import path from "node:path";

// ---------------------------------------------------------------------------
// 工具
// ---------------------------------------------------------------------------

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const SCRIPT = path.join(__dirname, "check-pr-template.mjs");

let passed = 0;
let failed = 0;

/**
 * 以指定环境变量运行校验脚本，返回 { status, stdout, stderr }
 */
function run(env) {
  const result = spawnSync(process.execPath, [SCRIPT], {
    env: { ...process.env, ...env },
    encoding: "utf-8",
    timeout: 10_000,
  });
  return {
    status: result.status ?? null,
    stdout: (result.stdout || "").trim(),
    stderr: (result.stderr || "").trim(),
    error: result.error || null,
  };
}

function assert(label, condition, detail = "") {
  if (condition) {
    passed++;
    console.log(`  ✓ ${label}`);
  } else {
    failed++;
    console.log(`  ✗ ${label}${detail ? ` — ${detail}` : ""}`);
  }
}

const DOWNGRADE_ALL = "all";

// ---------------------------------------------------------------------------
// 辅助: 构造 PR body
// ---------------------------------------------------------------------------

/** 最小合法 body (无 warning 区域) */
function minimalValidBody() {
  return `- [x] feat: test

Closes #42

## 变更摘要

变更说明内容。

## 宪章合规性

- [x] \`CONSTITUTION.md §X.Y\` 合规

## 验证方式

\`\`\`bash
cargo test
\`\`\`

## 审查聚焦

请关注模块边界处理。
`;
}

// ---------------------------------------------------------------------------
// 测试
// ---------------------------------------------------------------------------

console.log("\n=== 测试: 完整模板通过校验 ===\n");
{
  const { status, stdout } = run({ PR_BODY: minimalValidBody() });
  assert("exit code == 0", status === 0);
  assert("输出包含校验通过", stdout.includes("模板校验通过"));
}

console.log("\n=== 测试: 缺失必填字段 — 未勾选类型 ===\n");
{
  // 需要移除所有 - [x] (包含宪章合规性中的)，使 regex 无法在 body 中匹配到
  const body = minimalValidBody()
    .replace(/- \[x\]/g, "- [ ]");
  const { status, stdout } = run({ PR_BODY: body });
  assert("exit code != 0 (失败)", status !== 0 && status !== null);
  assert("输出包含变更类型错误", stdout.includes("请勾选至少一项") && stdout.includes("::error"));
}

console.log("\n=== 测试: 缺失必填字段 — 空摘要 ===\n");
{
  const body = minimalValidBody().replace("变更说明内容。", "");
  const { status, stdout } = run({ PR_BODY: body });
  assert("exit code != 0 (失败)", status !== 0 && status !== null);
  assert("输出包含摘要错误", stdout.includes("::error") && stdout.includes("变更摘要"));
}

console.log("\n=== 测试: 部分填写 — 仅有变更类型 ===\n");
{
  const body = `- [x] feat: test

Closes #1

## 变更摘要



## 宪章合规性

- [x] \`CONSTITUTION.md\` ok

## 验证方式

\`\`\`bash

\`\`\`

## 审查聚焦



`;
  const { status, stdout } = run({ PR_BODY: body });

  // 仅有 checkbox 勾选但摘要为空，应失败
  assert("exit code != 0 或摘要为空报错", status !== 0 && status !== null);
  assert("输出包含变更摘要错误", stdout.includes("变更摘要") && stdout.includes("::error"));
  // 验证方式和审查聚焦仅为 warning，不应阻止通过
  assert("输出包含验证方式提示", stdout.includes("验证方式") || stdout.includes("审查聚焦"));
}

console.log("\n=== 测试: 错误降级 — PR_TEMPLATE_ERROR_DOWNGRADE=all ===\n");
{
  // 移除 checkbox 和摘要 → 两个硬错误，降级后全部变 warning
  const body = minimalValidBody()
    .replace("- [x]", "- [ ]")
    .replace("变更说明内容。", "");
  const { status, stdout } = run({
    PR_BODY: body,
    PR_TEMPLATE_ERROR_DOWNGRADE: DOWNGRADE_ALL,
  });
  assert("exit code == 0 (降级后通过)", status === 0);
  assert("输出包含降级标记", stdout.includes("误差降级"));
  assert("输出包含模板校验通过", stdout.includes("模板校验通过"));
}

console.log("\n=== 测试: 单字段降级 — 仅 summary ===\n");
{
  const body = minimalValidBody().replace("变更说明内容。", "");
  const { status, stdout } = run({
    PR_BODY: body,
    PR_TEMPLATE_ERROR_DOWNGRADE: "summary",
  });
  assert("exit code == 0 (summary 降级后通过)", status === 0);
  assert("输出包含降级标记", stdout.includes("误差降级"));
  assert("未勾选类型仍然报错 (未降级)", !stdout.includes("变更类型") || !stdout.includes("::error::变更类型"));
}

console.log("\n=== 测试: 单字段降级 — 仅 type ===\n");
{
  const body = minimalValidBody().replace("- [x]", "- [ ]");
  const { status, stdout } = run({
    PR_BODY: body,
    PR_TEMPLATE_ERROR_DOWNGRADE: "type",
  });
  // type 降级但 summary 仍有效 → 应通过
  assert("exit code == 0 (type 降级后通过)", status === 0);
  assert("输出包含降级标记或直接通过", stdout.includes("误差降级") || stdout.includes("模板校验通过"));
}

console.log("\n=== 测试: 两个字段都降级 ===\n");
{
  const body = minimalValidBody()
    .replace("- [x]", "- [ ]")
    .replace("变更说明内容。", "");
  const { status, stdout } = run({
    PR_BODY: body,
    PR_TEMPLATE_ERROR_DOWNGRADE: "type,summary",
  });
  assert("exit code == 0 (双降级后通过)", status === 0);
  assert("降级后通过", stdout.includes("模板校验通过") || stdout.includes("误差降级"));
}

console.log("\n=== 测试: 空 body ===\n");
{
  const { status, stdout } = run({ PR_BODY: "" });
  assert("exit code == 1", status === 1);
  assert("输出包含未检测到 PR 描述", stdout.includes("未检测到 PR 描述"));
}

console.log("\n=== 测试: 空白 body ===\n");
{
  const { status, stdout } = run({ PR_BODY: "   \n  \n  " });
  assert("exit code == 1", status === 1);
  assert("输出包含未检测到 PR 描述", stdout.includes("未检测到 PR 描述"));
}

console.log("\n=== 测试: 仅有标题、无勾选框，也无其他 - [x] ===\n");
{
  // body 中完全没有 - [x] 勾选，但摘要已填 → 仅类型缺失导致失败
  const body = `Closes #42

## 变更摘要

内容

## 宪章合规性

- [ ] 未勾选项

## 验证方式

\`\`\`bash
echo ok
\`\`\`

## 审查聚焦

关注模块边界
`;
  const { status, stdout } = run({ PR_BODY: body });
  assert("exit code != 0 (缺少类型勾选)", status !== 0 && status !== null);
  assert("输出包含变更类型错误", stdout.includes("::error") && stdout.includes("变更类型"));
}

console.log("\n=== 测试: 模板内容为空字符串 (仅标题) ===\n");
{
  const body = `## 变更摘要


## 宪章合规性

- [ ] \`foo\`

## 验证方式

\`\`\`

\`\`\`

## 审查聚焦


`;
  const { status, stdout } = run({ PR_BODY: body });
  // 无勾选 checkbox + 空摘要 → 双重错误
  assert("exit code != 0", status !== 0 && status !== null);
  assert("输出包含变更类型和摘要错误", stdout.includes("::error"));
}

console.log("\n=== 测试: 缺失 Closes (仅 warning，不阻塞) ===\n");
{
  const body = minimalValidBody().replace("Closes #42", "");
  const { status, stdout } = run({ PR_BODY: body });
  assert("exit code == 0 (Closes 仅为 warning)", status === 0);
  assert("输出包含关联 Issue 警告", stdout.includes("关联 Issue") && stdout.includes("::warning"));
}

console.log("\n=== 测试: 宪章合规性未勾选 (仅 warning) ===\n");
{
  const body = minimalValidBody().replace("- [x] `CONSTITUTION.md", "- [ ] `CONSTITUTION.md");
  const { status, stdout } = run({ PR_BODY: body });
  assert("exit code == 0 (未勾选仅为 warning)", status === 0);
  assert("输出包含未勾选警告", stdout.includes("未勾选") && stdout.includes("::warning"));
}

console.log("\n=== 测试: 验证方式代码块为空 (仅 warning) ===\n");
{
  const body = minimalValidBody().replace("cargo test", "");
  const { status, stdout } = run({ PR_BODY: body });
  assert("exit code == 0 (空验证仅为 warning)", status === 0);
}

console.log("\n=== 测试: 审查聚焦为空 (仅 warning) ===\n");
{
  const body = minimalValidBody().replace("请关注模块边界处理。", "");
  const { status, stdout } = run({ PR_BODY: body });
  assert("exit code == 0 (空聚焦仅为 warning)", status === 0);
}

console.log("\n=== 测试: 无 body 环境变量 ===\n");
{
  const { status, stdout } = run({});
  // PR_BODY 未设置时默认为 ""
  assert("exit code == 1 (未设置 PR_BODY)", status === 1);
  assert("输出包含未检测到", stdout.includes("未检测到 PR 描述"));
}

console.log("\n=== 测试: 摘要含 HTML 注释 (注释被剥离) ===\n");
{
  const body = `- [x] feat: test

Closes #42

## 变更摘要

<!-- 这是一段 HTML 注释，应被忽略 -->
实际摘要内容。

## 宪章合规性

- [x] ok

## 验证方式

\`\`\`bash
echo ok
\`\`\`

## 审查聚焦

关注点
`;
  const { status, stdout } = run({ PR_BODY: body });
  assert("exit code == 0 (HTML 注释被剥离，有实际内容)", status === 0);
  assert("输出包含校验通过", stdout.includes("模板校验通过"));
}

console.log("\n=== 测试: 摘要仅含 HTML 注释 (无实际内容) ===\n");
{
  const body = `- [x] feat: test

Closes #42

## 变更摘要

<!-- 仅注释无正文 -->

## 宪章合规性

- [x] ok

## 验证方式

\`\`\`bash
echo ok
\`\`\`

## 审查聚焦

关注点
`;
  const { status, stdout } = run({ PR_BODY: body });
  assert("exit code != 0 (仅注释被视为空)", status !== 0 && status !== null);
  assert("输出包含摘要错误", stdout.includes("::error") && stdout.includes("变更摘要"));
}

// ---------------------------------------------------------------------------
// 汇总
// ---------------------------------------------------------------------------

console.log(`\n${"─".repeat(40)}`);
const total = passed + failed;
console.log(`结果: ${passed}/${total} 通过`);
if (failed > 0) {
  console.log(`${"─".repeat(40)}`);
  process.exit(1);
}
console.log(`${"─".repeat(40)}`);
process.exit(0);
