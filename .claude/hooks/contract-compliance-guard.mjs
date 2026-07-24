#!/usr/bin/env node
// Contract Compliance Guard — Stop Hook
//
// 会话收尾时，检测本会话是否修改了 Rust 源码或合同规格，
// 若是则运行 L1 合同合规验证并输出警告。
//
// 默认不阻塞会话（exit 0），可通过环境变量启用阻断模式：
//   CONTRACT_COMPLIANCE_BLOCKING=1  →  违规时 exit 1
//
// 跳过条件：
//   CONTRACT_COMPLIANCE_SKIP=1  →  静默退出，不运行任何检查
//
// 规则来源：.agents/ssot/CONTRACT_SPEC.md · SSOT.md R4
//
// ============================================================================

import { execSync } from "child_process";
import { existsSync } from "fs";
import { join, dirname } from "path";
import { fileURLToPath } from "url";

const __dir = dirname(fileURLToPath(import.meta.url));
const ROOT = join(__dir, "..", "..");

function run(cmd) {
  try {
    return execSync(cmd, {
      encoding: "utf8",
      cwd: ROOT,
      stdio: ["pipe", "pipe", "pipe"],
      timeout: 8000,
    }).trim();
  } catch {
    return "";
  }
}

function main() {
  // 跳过检查
  if (process.env.CONTRACT_COMPLIANCE_SKIP === "1") {
    return;
  }

  const blocking = process.env.CONTRACT_COMPLIANCE_BLOCKING === "1";

  // 1. 检测本会话是否修改了 Rust 源码或 SSOT 规格
  const rustChanged = run(
    "git diff --name-only HEAD~1 2>/dev/null; git diff --cached --name-only"
  );
  const allChanged = rustChanged
    ? rustChanged.split("\n").filter(Boolean)
    : [];

  const relevantPatterns = [/\.rs$/, /Cargo\.toml$/, /\.agents\/ssot\//, /spec\.md$/];
  const hasRelevantChanges = allChanged.some((f) =>
    relevantPatterns.some((p) => p.test(f))
  );

  // 也检查工作区未提交的变更
  const wsChanged = run("git diff --name-only").split("\n").filter(Boolean);
  wsChanged.push(...run("git diff --cached --name-only").split("\n").filter(Boolean));

  const hasWorkspaceChanges = new Set([...allChanged, ...wsChanged]).size > allChanged.length
    ? wsChanged.some((f) => relevantPatterns.some((p) => p.test(f)))
    : false;

  const shouldCheck = hasRelevantChanges || hasWorkspaceChanges;

  if (!shouldCheck) {
    // 无相关文件变更，静默退出
    return;
  }

  // 2. 检查合规脚本是否存在
  const scriptPath = join(ROOT, "scripts", "quality-gates", "check-contract-compliance.mjs");
  if (!existsSync(scriptPath)) {
    console.error("");
    console.error("══════════════════════════════════════════════════════");
    console.error("[ContractCompliance] ⚠️  合规脚本缺失");
    console.error("");
    console.error(`  期望路径: scripts/quality-gates/check-contract-compliance.mjs`);
    console.error("  跳过合同合规检查。");
    console.error("══════════════════════════════════════════════════════");
    console.error("");
    return; // 缺失脚本不阻塞
  }

  // 3. 运行 L1 合同合规验证
  let result;
  try {
    const output = execSync(
      `node ${scriptPath} --level L1 --fail-level L1 --json`,
      {
        encoding: "utf8",
        cwd: ROOT,
        stdio: ["pipe", "pipe", "pipe"],
        timeout: 30000,
      }
    );
    result = JSON.parse(output);
  } catch (err) {
    // 脚本异常（退出码 ≠ 0 表示有违规）
    if (err.stdout) {
      try {
        result = JSON.parse(err.stdout.toString());
      } catch {
        result = null;
      }
    }
    if (!result) {
      console.error("");
      console.error("══════════════════════════════════════════════════════");
      console.error("[ContractCompliance] ⚠️  合规检查执行失败");
      console.error("");
      if (err.stderr) {
        const stderr = err.stderr.toString().trim();
        if (stderr) console.error(`  ${stderr}`);
      }
      console.error("══════════════════════════════════════════════════════");
      console.error("");
      return; // 脚本故障不阻塞
    }
  }

  if (!result || !result.summary) return;

  const { total, passed, failed } = result.summary;

  // 全部通过，静默退出
  if (failed === 0) return;

  // 4. 输出违规警告
  const lines = [];
  lines.push("");
  lines.push("══════════════════════════════════════════════════════");
  if (blocking) {
    lines.push("[ContractCompliance] 🛑  合同合规验证失败（阻断模式）");
  } else {
    lines.push("[ContractCompliance] ⚠️  合同合规验证发现违规");
  }
  lines.push("");
  lines.push(`  检查项: ${total}  |  通过: ${passed}  |  违规: ${failed}`);
  lines.push("");

  // 列出违规明细
  const violations = (result.results || []).filter((r) => !r.ok);
  const maxShow = 10;
  for (const v of violations.slice(0, maxShow)) {
    lines.push(`  ✗ ${v.ruleId} [${v.contract}] ${v.message}`);
  }
  if (violations.length > maxShow) {
    lines.push(`  ... (还有 ${violations.length - maxShow} 项违规)`);
  }

  lines.push("");
  lines.push("  📋 查看完整报告:");
  lines.push(`    $ node scripts/quality-gates/check-contract-compliance.mjs --level L1`);
  lines.push("");
  lines.push("  规则来源: .agents/ssot/CONTRACT_SPEC.md");
  lines.push("══════════════════════════════════════════════════════");
  lines.push("");

  console.error(lines.join("\n"));

  if (blocking) {
    process.exit(1);
  }
}

main();
process.exit(0);
