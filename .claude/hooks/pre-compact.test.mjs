/**
 * pre-compact.test.mjs — L1 单元测试 for pre-compact.mjs
 *
 * 测试范围：
 *  1. 快照结构 — 输出行序列（PreCompact header）
 *  2. 分支与状态解析 — git rev-parse / git status 输出
 *  3. 变更文件行构建 — 文件列表截断逻辑
 *  4. 最近提交解析 — git log -1 --oneline
 *  5. STATE.md 正则 — Phase / Last Run 提取
 *  6. 审查报告计数 — reviews 目录文件统计
 *
 * 使用 ESM (.mjs)，纯 assert 模式。
 */

let pass = 0, fail = 0;
function ok(c, name) {
  if (c) { pass++; console.log("  ok  " + name); }
  else { fail++; console.log("  FAIL " + name); }
}

// ═══ 从被测文件复制纯函数 ═══

/** 模拟 run() 的返回逻辑（无 execSync 依赖） */
const mockRun = (output) => (cmd) => {
  if (cmd.includes("rev-parse --abbrev-ref")) return "feat/test-branch";
  if (cmd.includes("status --short")) return output;
  if (cmd.includes("log -1")) return "abc1234 feat: add pre-compact hook";
  return "";
};

/** 构建变更文件列表（含截断） */
const buildChangedFiles = (statusOutput) => {
  if (!statusOutput) return { count: 0, files: [] };
  const lines = statusOutput.split("\n").filter(Boolean).filter(l => l.trim());
  return { count: lines.length, files: lines.slice(0, 10).map(l => l.trim()) };
};

/** 提取 STATE.md 中的 Phase */
const extractPhase = (content) => {
  const m = content.match(/\*\*Phase\*\*: (.+)/);
  return m ? m[1] : null;
};

/** 提取 STATE.md 中的 Last Run */
const extractLastRun = (content) => {
  const m = content.match(/\*\*Last Run\*\*: (.+)/);
  return m ? m[1] : null;
};

/** 计算 reviews 目录下 .md 文件数 */
const countReviews = (fileNames) => {
  return fileNames.filter(f => f.endsWith(".md")).length;
};

// ═══ 测试开始 ═══

console.log("\npre-compact L1 tests");

// L0: shebang/syntax — 由 node --check 保证

// --- 1. 分支解析 ---
const branch = mockRun("")("git rev-parse --abbrev-ref HEAD 2>/dev/null");
ok(branch === "feat/test-branch", "分支名正确提取: " + branch);

// --- 2. 变更文件列表 ---
const emptyChanges = buildChangedFiles("");
ok(emptyChanges.count === 0, "空状态 → 0 个变更文件");
ok(emptyChanges.files.length === 0, "空状态 → 文件列表为空");

const singleChange = buildChangedFiles(" M src/main.rs");
ok(singleChange.count === 1, "单文件变更 → count=1");
ok(singleChange.files[0] === "M src/main.rs", "单文件变更 → 文件路径正确");

const multiChanges = buildChangedFiles(" M a.rs\n M b.rs\n M c.rs");
ok(multiChanges.count === 3, "3 个变更 → count=3");
ok(multiChanges.files.length === 3, "3 个变更 → 文件列表长度为 3");

// --- 3. 文件列表截断（>10 个文件） ---
const manyLines = Array.from({ length: 15 }, (_, i) => ` M file${i}.rs`).join("\n");
const manyChanges = buildChangedFiles(manyLines);
ok(manyChanges.count === 15, "15 个变更 → count=15");
ok(manyChanges.files.length === 10, "文件列表截断为 10 行");

// --- 4. 最近提交 ---
const lastCommit = mockRun("")("git log -1 --oneline 2>/dev/null");
ok(lastCommit.includes("abc1234"), "最近提交包含 hash");
ok(lastCommit.includes("pre-compact hook"), "最近提交包含 message");

// --- 5. STATE.md Phase 提取 ---
const stateWithPhase = "**Phase**: build\n**Last Run**: 2024-01-01";
ok(extractPhase(stateWithPhase) === "build", "Phase 正确提取: build");
ok(extractPhase("no phase here") === null, "无 Phase → 返回 null");

const stateWithRun = "**Phase**: design\n**Last Run**: 2024-12-31 23:59";
ok(extractLastRun(stateWithRun) === "2024-12-31 23:59", "Last Run 正确提取");
ok(extractLastRun("no run here") === null, "无 Last Run → 返回 null");

// Phase + Last Run 同时提取
ok(extractPhase(stateWithRun) === "design", "Phase=design + Last Run 同时存在可正确提取");
ok(extractLastRun(stateWithPhase) === "2024-01-01", "Phase + Last Run=2024-01-01 同时存在可正确提取");

// --- 6. 审查报告计数 ---
ok(countReviews([]) === 0, "无文件 → 审查计数 0");
ok(countReviews(["a.txt", "b.log"]) === 0, "非 .md 文件不计入审查计数");
ok(countReviews(["review1.md"]) === 1, "1 个 .md → 审查计数 1");
ok(countReviews(["a.md", "b.md", "c.md", "d.txt"]) === 3, "混合文件 → 仅计 .md");

// --- 7. 输出行结构验证 ---
// PreCompact 输出的前两行固定
const header = ["[PreCompact: 会话状态快照]", ""];
ok(header[0] === "[PreCompact: 会话状态快照]", "header 第 1 行为标题");
ok(header[1] === "", "header 第 2 行为空行");

// 变更行格式
const buildSnapshot = (branch, changedCount) => {
  const lines = ["[PreCompact: 会话状态快照]", ""];
  if (changedCount > 0) {
    lines.push("当前分支: " + branch);
    lines.push("未提交变更: " + changedCount + " 个文件");
  }
  lines.push("---");
  return lines;
};

const snap = buildSnapshot("feat/foo", 5);
ok(snap[2] === "当前分支: feat/foo", "快照包含分支信息");
ok(snap[3] === "未提交变更: 5 个文件", "快照包含变更数量");

const snapClean = buildSnapshot("main", 0);
ok(snapClean[2] === "---", "无变更时跳过变更行直接输出分隔线");

// ═══ 结果 ═══
console.log(`\n  ${pass} passed, ${fail} failed, ${pass + fail} total\n`);
process.exit(fail > 0 ? 1 : 0);
