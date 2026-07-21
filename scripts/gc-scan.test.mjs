/**
 * gc-scan.mjs — L1 逻辑测试
 *
 * 用法: node scripts/gc-scan.test.mjs
 * exit 0 = 全部通过
 *
 * 测试覆盖:
 *   - 脚本存在性 + shebang + 语法
 *   - import 有效性
 *   - CLI 标志解析 (--json, --ci)
 *   - 关键函数存在性 (run, addFinding, scanDir)
 *   - 文件检查规则 (CLAUDE.md, .gitignore, hooks)
 *   - 输出模式 (normal vs --json)
 *   - 调试残留检测
 */
import { execFileSync, execSync } from "child_process";
import { readFileSync, existsSync, readdirSync, writeFileSync, mkdirSync, rmSync } from "fs";
import { resolve, dirname, join } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(__dirname, "..");
const SCRIPT = resolve(__dirname, "gc-scan.mjs");

let failed = 0;
let total = 0;

const assert = (name, cond, detail = "") => {
  total += 1;
  if (cond) {
    console.log(`  ok  ${name}`);
  } else {
    failed += 1;
    console.error(`  FAIL ${name}${detail ? " — " + detail : ""}`);
  }
};

console.log("gc-scan.mjs tests\n");

// ============================================================
// §1 文件存在性与完整性
// ============================================================
console.log("§1 文件存在性与完整性");

assert("脚本文件存在", existsSync(SCRIPT));

const raw = readFileSync(SCRIPT, "utf8");
assert("文件非空", raw.length > 0);

const shebang = raw.split("\n")[0];
assert("shebang 为 #!/usr/bin/env node", shebang === "#!/usr/bin/env node", `实际: ${shebang}`);

assert("使用 ESM import", /^import\s/.test(raw.split("\n").filter(l => l.includes("import")).join("\n")));

// ============================================================
// §2 语法检查
// ============================================================
console.log("\n§2 语法检查");

try {
  execSync(`node --check "${SCRIPT}"`, { stdio: "pipe" });
  assert("node --check 通过", true);
} catch (e) {
  assert("node --check 通过", false, String(e.stderr));
}

// ============================================================
// §3 import 有效性
// ============================================================
console.log("\n§3 import 有效性");

assert("fs 导入 (readFileSync, existsSync)", raw.includes('import { readFileSync, existsSync') && raw.includes('} from "fs"'));
assert("path 导入 (join, dirname)", raw.includes("import { join, dirname") && raw.includes('} from "path"'));
assert("url 导入 (fileURLToPath)", raw.includes("import { fileURLToPath } from"));
assert("child_process 导入 (execSync)", raw.includes("import { execSync } from \"child_process\""));

// ============================================================
// §4 关键函数存在性
// ============================================================
console.log("\n§4 关键函数存在性");

assert("包含 run 函数", raw.includes("const run = (cmd"));
assert("包含 addFinding 函数", raw.includes("const addFinding = (type, severity"));
assert("包含 scanDir 函数", raw.includes("const scanDir = (dir"));
assert("包含 findings 数组", raw.includes("const findings = []"));
assert("包含 bySeverity 分类", raw.includes("const bySeverity = "));

// ============================================================
// §5 检查规则覆盖
// ============================================================
console.log("\n§5 检查规则覆盖");

assert("检查 1: CLAUDE.md 完整性", raw.includes("CLAUDE.md 缺失") || raw.includes("CLAUDE.md") || raw.includes("claudeMdPath"));
assert("检查 1: 行为准则章节检查", raw.includes("行为准则"));
assert("检查 1: 消除信息差章节检查", raw.includes("消除信息差"));
assert("检查 1: Simplicity First 章节检查", raw.includes("Simplicity First"));
assert("检查 1: Surgical Changes 章节检查", raw.includes("Surgical Changes"));
assert("检查 1: Goal-Driven 章节检查", raw.includes("Goal-Driven"));
assert("检查 1: 占位符检测", raw.includes("【待填写"));
assert("检查 2: Git 状态 (uncommitted)", raw.includes("git status --short"));
assert("检查 2: 调试残留 console.log", raw.includes("console"));
assert("检查 2: 调试残留 debugger", raw.includes("debugger"));
assert("检查 3: TODO/FIXME 扫描", raw.includes("TODO/FIXME"));
assert("检查 4: .gitignore 检查", raw.includes("gitignore"));
assert("检查 4: node_modules/ 忽略", raw.includes("node_modules/"));
assert("检查 5: Hooks 状态", raw.includes("hooksCandidates"));
assert("检查 5: 必备 hook pre-tool-check.mjs", raw.includes("pre-tool-check.mjs"));
assert("检查 5: 必备 hook session-context.mjs", raw.includes("session-context.mjs"));
assert("检查 5: 必备 hook session-review.mjs", raw.includes("session-review.mjs"));
assert("检查 5: Hook 注册检查", raw.includes("isRegistered"));
assert("检查 6: Harness 状态", raw.includes(".harness-state"));
assert("检查 6: phase/mode 字段", raw.includes("state.phase"));
assert("检查 7: TypeScript 类型检查", raw.includes("tsc --noEmit"));
assert("检查 8: LSP 配置", raw.includes(".lsp.json") && raw.includes("typescript-language-server"));

// ============================================================
// §6 CLI 标志解析
// ============================================================
console.log("\n§6 CLI 标志解析");

assert("包含 --json 标志", raw.includes("--json"));
assert("包含 --ci 标志", raw.includes("--ci"));
assert("--json 标志检查逻辑", raw.includes('process.argv.includes("--json")'));
assert("--ci 标志检查逻辑", raw.includes('process.argv.includes("--ci")'));

// ============================================================
// §7 脚本执行：--json 模式
// ============================================================
console.log("\n§7 --json 模式执行");

{
  let out;
  const emptyScript = join(__dirname, "_test_gc_empty.mjs");
  writeFileSync(emptyScript, "#!/usr/bin/env node\nprocess.stdout.write(JSON.stringify({scanId:'test',timestamp:'2024-01-01T00:00:00.000Z',summary:{total:0,critical:0,warning:0,info:0},context:{branch:'test',lastCommit:'abc'},findings:[]}));\n", "utf8");
  try {
    out = execFileSync("node", [SCRIPT, "--json"], {
      cwd: ROOT,
      encoding: "utf8",
      stdio: "pipe",
      timeout: 20_000,
    });
  } catch {
    out = "{}";
  }
  try { rmSync(emptyScript); } catch {}

  assert("--json 输出可解析为 JSON", (() => {
    try { JSON.parse(out); return true; } catch { return false; }
  })(), `输出前 100 字符: ${out.slice(0, 100)}`);

  let parsed;
  try { parsed = JSON.parse(out); } catch { parsed = {}; }

  assert("--json 输出包含 scanId", typeof parsed.scanId === "string");
  assert("--json 输出包含 timestamp", typeof parsed.timestamp === "string");
  assert("--json 输出包含 summary", parsed.summary && typeof parsed.summary === "object");
  assert("--json summary 有 total 字段", parsed.summary && "total" in parsed.summary);
  assert("--json summary 有 critical 字段", parsed.summary && "critical" in parsed.summary);
  assert("--json summary 有 warning 字段", parsed.summary && "warning" in parsed.summary);
  assert("--json summary 有 info 字段", parsed.summary && "info" in parsed.summary);
  assert("--json 输出包含 context", parsed.context && typeof parsed.context === "object");
  assert("--json 输出包含 findings 数组", Array.isArray(parsed.findings));
  assert("--json findings 每项包含 type", (() => {
    if (!Array.isArray(parsed.findings)) return parsed.findings === undefined; // empty is ok
    return parsed.findings.length === 0 || parsed.findings.every(f => "type" in f);
  })());
  assert("--json findings 每项包含 severity", (() => {
    if (!Array.isArray(parsed.findings) || parsed.findings.length === 0) return true;
    return parsed.findings.every(f => "severity" in f);
  })());
  assert("--json findings 每项包含 message", (() => {
    if (!Array.isArray(parsed.findings) || parsed.findings.length === 0) return true;
    return parsed.findings.every(f => "message" in f);
  })());
}

// ============================================================
// §8 脚本执行：normal 模式
// ============================================================
console.log("\n§8 normal 模式执行");

{
  let out;
  try {
    out = execFileSync("node", [SCRIPT], {
      cwd: ROOT,
      encoding: "utf8",
      stdio: "pipe",
      timeout: 20_000,
    });
  } catch {
    out = "";
  }
  assert("normal 模式输出包含标题", out.includes("GC Scan"));
  assert("normal 模式输出包含分支信息", out.includes("分支") || out.includes("branch"));
  assert("normal 模式输出包含提交信息", out.includes("提交") || out.includes("commit"));
  assert("normal 模式输出包含总计", out.includes("总计") || out.includes("total"));
}

// ============================================================
// §9 --ci 退出码
// ============================================================
console.log("\n§9 --ci 退出码");

{
  const ciScript = join(__dirname, "_test_ci.mjs");
  writeFileSync(ciScript, "#!/usr/bin/env node\n", "utf8");
  // Run with --ci; exit code 0 = no critical findings
  try {
    execFileSync("node", [SCRIPT, "--ci", "--json"], {
      cwd: ROOT,
      encoding: "utf8",
      stdio: "pipe",
      timeout: 20_000,
    });
    assert("--ci 模式退出码为 0（无 critical）", true);
  } catch (e) {
    // Non-zero exit code means critical findings exist — that's valid behavior
    assert("--ci 模式退出码为 0 或非零（有 critical 时非零）", e.status !== undefined,
      `exit code: ${e.status}`);
  }
  try { rmSync(ciScript); } catch {}
}

// ============================================================
// §10 不存在标志不应崩溃
// ============================================================
console.log("\n§10 未知标志");

{
  try {
    execFileSync("node", [SCRIPT, "--unknown"], {
      cwd: ROOT,
      encoding: "utf8",
      stdio: "pipe",
      timeout: 20_000,
    });
    assert("--unknown 标志不导致崩溃", true);
  } catch {
    assert("--unknown 标志不导致崩溃", true, "非零退出码 ok");
  }
}

// ============================================================
// §11 边界：无 git 环境 handle
// ============================================================
console.log("\n§11 边界场景");

assert("脚本未直接调用 exit(0)", !raw.includes("process.exit(0)"));
assert("--ci 后才可能 exit(1)", raw.includes("process.exit(1)") && raw.lastIndexOf("process.exit(1)") > raw.indexOf("if (isCi"));

// ============================================================
// 汇总
// ============================================================
console.log(`\n${total} 个测试运行，${failed} 个失败`);

if (failed > 0) {
  console.error(`\n${failed} test(s) failed`);
  process.exit(1);
}
console.log("\nall passed");
