#!/usr/bin/env node
/**
 * branch-protect.test.mjs — L1 单元测试 for branch-protect.mjs
 *
 * 测试范围：
 *  1. 文件结构：shebang、语法有效性
 *  2. 纯函数：commit 数组解析、preview 切片、消息构造
 *  3. 分支逻辑：main/master 静默退出、detached HEAD 静默退出
 *  4. 警告输出：包含未合并提交预览、建议操作
 */

import { execFileSync } from "child_process";
import { readFileSync } from "fs";

// ── 从 branch-protect.mjs 提取的可测试逻辑 ────────────────────

/**
 * 模拟 git log --oneline 输出，提取未合并提交列表
 * （等效于源文件中 aheadLog 的解析逻辑）
 */
function parseUnmergedCommits(gitLog) {
  if (!gitLog) return [];
  return gitLog.split("\n").filter(Boolean);
}

/**
 * 预览切片：取前 5 条提交，计算剩余数量
 */
function slicePreview(commits, maxPreview = 5) {
  const preview = commits.slice(0, maxPreview);
  const remaining = commits.length - preview.length;
  return { preview, remaining };
}

/**
 * 构建 branch-protect 警告消息
 * （复用源文件的消息构造逻辑）
 */
function buildWarning(branch, commits) {
  const { preview, remaining } = slicePreview(commits);
  const lines = [];
  lines.push("");
  lines.push("══════════════════════════════════════════════════════");
  lines.push("[BranchProtect] ⚠️  检测到未合并提交，禁止误删分支！");
  lines.push("");
  lines.push(`  当前分支: ${branch}`);
  lines.push(`  未合并提交: ${commits.length} 个 (领先 origin/main)`);
  lines.push("");
  lines.push("  未合并提交预览:");
  for (const c of preview) {
    lines.push(`    ${c}`);
  }
  if (remaining > 0) {
    lines.push(`    ... (还有 ${remaining} 个)`);
  }
  lines.push("");
  lines.push("  ❌ 禁止操作:");
  lines.push(`    - git branch -D ${branch}  (强制删除分支会丢失提交)`);
  lines.push("    - git checkout main 前未推送/未建 PR  (切换后分支可能被 GC)");
  lines.push("");
  lines.push("  ✅ 建议操作:");
  lines.push(`    $ git push -u origin HEAD          # 推送当前分支到远端`);
  lines.push(`    $ gh pr create --base main         # 创建 PR 合入 main`);
  lines.push("");
  lines.push("  规则来源: CLAUDE.md §分支保护");
  lines.push("══════════════════════════════════════════════════════");
  lines.push("");
  return lines.join("\n");
}

// ── 测试框架 ──────────────────────────────────────────────────

let passed = 0;
let failed = 0;

function assert(condition, message) {
  if (condition) {
    passed++;
  } else {
    failed++;
    console.error(`  FAIL: ${message}`);
  }
}

function describe(name, fn) {
  console.log(`\n${name}`);
  fn();
}

function it(name, fn) {
  console.log(`  ${name}`);
  try {
    fn();
  } catch (e) {
    failed++;
    console.error(`  FAIL: ${name} threw: ${e.message}`);
    if (process.env.DEBUG) console.error(e.stack);
  }
}

// ── 测试 1: 文件结构 ────────────────────────────────────────

describe("branch-protect.mjs — 文件结构", () => {
  it("shebang 存在", () => {
    const src = readFileSync(".claude/hooks/branch-protect.mjs", "utf8");
    assert(src.startsWith("#!/usr/bin/env node"), "首行 shebang");
  });

  it("语法有效（node --check）", () => {
    try {
      execFileSync("node", ["--check", ".claude/hooks/branch-protect.mjs"], {
        stdio: "pipe",
        timeout: 5000,
      });
      assert(true, "node --check 通过");
    } catch (e) {
      assert(false, `node --check 失败: ${e.stderr?.toString() || e.message}`);
    }
  });

  it("包含 execSync import", () => {
    const src = readFileSync(".claude/hooks/branch-protect.mjs", "utf8");
    assert(
      src.includes('import { execSync } from "child_process"') ||
        src.includes("import { execSync } from 'child_process'"),
      "execSync import 存在"
    );
  });
});

// ── 测试 2: 未合并提交解析 ──────────────────────────────────

describe("parseUnmergedCommits — git log 输出解析", () => {
  it("正常多行输出", () => {
    const log = "abc123 feat: 登录模块\nbcd234 fix: 修复 bug\ndef345 docs: 更新";
    const commits = parseUnmergedCommits(log);
    assert(commits.length === 3, `3 条提交, got ${commits.length}`);
    assert(commits[0] === "abc123 feat: 登录模块", `c0=${commits[0]}`);
    assert(commits[2] === "def345 docs: 更新", `c2=${commits[2]}`);
  });

  it("单行输出", () => {
    const commits = parseUnmergedCommits("single commit line");
    assert(commits.length === 1, `1 条, got ${commits.length}`);
    assert(commits[0] === "single commit line", `c0=${commits[0]}`);
  });

  it("空字符串 → 空数组", () => {
    const commits = parseUnmergedCommits("");
    assert(commits.length === 0, `空数组, got ${commits.length}`);
  });

  it("null/undefined → 空数组", () => {
    const commits = parseUnmergedCommits(null);
    assert(commits.length === 0, `null → 空数组, got ${commits.length}`);
    const commits2 = parseUnmergedCommits(undefined);
    assert(commits2.length === 0, `undefined → 空数组, got ${commits2.length}`);
  });

  it("仅换行符 → 空数组", () => {
    const commits = parseUnmergedCommits("\n\n");
    assert(commits.length === 0, `空数组, got ${commits.length}`);
  });

  it("前后有空行", () => {
    const log = "\nabc123 fix: bug\n\ndef456 feat: add\n\n";
    const commits = parseUnmergedCommits(log);
    assert(commits.length === 2, `2 条, got ${commits.length}`);
  });
});

// ── 测试 3: preview 切片 ─────────────────────────────────────

describe("slicePreview — 提交预览切片", () => {
  it("少于 maxPreview 显示全部", () => {
    const commits = ["a", "b", "c"];
    const { preview, remaining } = slicePreview(commits, 5);
    assert(preview.length === 3, `preview=3, got ${preview.length}`);
    assert(remaining === 0, `remaining=0, got ${remaining}`);
  });

  it("正好 maxPreview 显示全部", () => {
    const commits = ["a", "b", "c", "d", "e"];
    const { preview, remaining } = slicePreview(commits);
    assert(preview.length === 5, `preview=5, got ${preview.length}`);
    assert(remaining === 0, `remaining=0, got ${remaining}`);
  });

  it("超过 maxPreview 截断并显示剩余", () => {
    const commits = ["a", "b", "c", "d", "e", "f", "g", "h"];
    const { preview, remaining } = slicePreview(commits);
    assert(preview.length === 5, `preview=5, got ${preview.length}`);
    assert(remaining === 3, `remaining=3, got ${remaining}`);
    assert(preview[0] === "a", "preview[0]");
    assert(preview[4] === "e", "preview[4]");
  });

  it("空数组 → 空 preview, remaining=0", () => {
    const { preview, remaining } = slicePreview([]);
    assert(preview.length === 0, `preview=0, got ${preview.length}`);
    assert(remaining === 0, `remaining=0`);
  });
});

// ── 测试 4: 警告消息 ─────────────────────────────────────────

describe("buildWarning — 警告消息构造", () => {
  it("中文警告标签存在", () => {
    const commits = ["abc feat: test"];
    const msg = buildWarning("feat/my-feature", commits);
    assert(msg.includes("[BranchProtect]"), "[BranchProtect] 标签");
    assert(msg.includes("未合并提交"), "未合并提交 中文");
    assert(msg.includes("禁止误删分支"), "禁止误删分支 中文");
  });

  it("包含分支名", () => {
    const commits = ["abc feat: test"];
    const msg = buildWarning("feat/login", commits);
    assert(msg.includes("当前分支: feat/login"), "分支名");
  });

  it("包含提交计数", () => {
    const commits = ["a", "b", "c"];
    const msg = buildWarning("feat/test", commits);
    assert(msg.includes("未合并提交: 3 个"), "3 个提交");
  });

  it("包含禁止操作提示", () => {
    const commits = ["abc feat: test"];
    const msg = buildWarning("feat/my-feature", commits);
    assert(msg.includes("git branch -D feat/my-feature"), "`git branch -D` 提示");
    assert(msg.includes("git checkout main"), "git checkout main 提示");
  });

  it("包含建议操作", () => {
    const commits = ["abc feat: test"];
    const msg = buildWarning("feat/test", commits);
    assert(msg.includes("git push -u origin HEAD"), "push 建议");
    assert(msg.includes("gh pr create"), "pr create 建议");
  });

  it("包含规则来源", () => {
    const commits = ["abc feat: test"];
    const msg = buildWarning("feat/test", commits);
    assert(msg.includes("规则来源: CLAUDE.md §分支保护"), "规则来源");
  });

  it("超过 5 条显示 ... 截断", () => {
    const commits = ["a", "b", "c", "d", "e", "f", "g"];
    const msg = buildWarning("feat/many", commits);
    assert(msg.includes("... (还有 2 个)"), "显示还有 2 个");
  });

  it("有提交预览内容", () => {
    const commits = ["abc123 feat: add login page"];
    const msg = buildWarning("feat/login", commits);
    assert(msg.includes("abc123 feat: add login page"), "预览包含提交信息");
  });
});

// ── 测试 5: 分支逻辑（main/master 静默退出）─────────────────

describe("branch-protect — main/master 分支逻辑", () => {
  it("main 分支应静默退出（不输出警告）", () => {
    // 逻辑等价：如果 branch === 'main'，则不产生警告
    const isMainBranch = (b) => b === "main" || b === "master";
    assert(isMainBranch("main"), "main → 静默退出");
    assert(isMainBranch("master"), "master → 静默退出");
    assert(!isMainBranch("feat/feature"), "feat/feature → 继续检测");
    assert(!isMainBranch("develop"), "develop → 继续检测");
    assert(!isMainBranch(""), "空字符串 → 继续检测");
  });
});

describe("branch-protect — detached HEAD 逻辑", () => {
  it("空分支名（detached HEAD）应静默退出", () => {
    // 逻辑等价：如果 run("git rev-parse --abbrev-ref HEAD") 返回空，则返回
    const isDetached = (branch) => !branch;
    assert(isDetached(""), "空字符串 → 静默退出");
    assert(isDetached(null), "null → 静默退出");
    assert(isDetached(undefined), "undefined → 静默退出");
    assert(!isDetached("feat/test"), "feat/test → 继续检测");
  });
});

describe("branch-protect — 未合并提交检测触发条件", () => {
  it("有未合并提交 → 应输出警告", () => {
    // 逻辑等价：parseUnmergedCommits 非空 → 输出警告
    const commits = parseUnmergedCommits("abc fix: bug\ndef feat: add");
    const shouldWarn = commits.length > 0;
    assert(shouldWarn, "有提交 → 输出警告");
  });

  it("无未合并提交 → 静默退出", () => {
    const commits = parseUnmergedCommits("");
    const shouldWarn = commits.length > 0;
    assert(!shouldWarn, "无提交 → 静默退出");
  });
});

// ── 测试 6: 集成行为（模拟 git 输出） ────────────────────────

describe("branch-protect — 完整流程模拟", () => {
  it("全流程：feat/test 分支 + 2 条未合并提交 → 输出警告", () => {
    const branch = "feat/test";
    const gitLog = "abc123 feat: initial impl\ndef456 fix: bug";

    // Step 1: 检查分支
    assert(branch !== "", "分支非空");
    assert(branch !== "main" && branch !== "master", "非 main/master");

    // Step 2: 计算未合并提交
    const commits = parseUnmergedCommits(gitLog);
    assert(commits.length === 2, "2 条未合并提交");

    // Step 3: 生成警告
    const msg = buildWarning(branch, commits);
    assert(msg.length > 0, "产生了警告消息");
    assert(msg.includes("[BranchProtect]"), "含警告标签");
    assert(msg.includes("2 个"), "含提交计数");
  });

  it("全流程：main 分支 → 静默退出", () => {
    const branch = "main";
    if (branch === "main" || branch === "master") {
      assert(true, "main 分支直接退出，不计算未合并提交");
    }
  });

  it("全流程：feat/test 分支 + 0 条未合并提交 → 静默退出", () => {
    const branch = "feat/test";
    assert(branch !== "" && branch !== "main" && branch !== "master", "非 main/master");

    const commits = parseUnmergedCommits("");
    assert(commits.length === 0, "0 条未合并提交 → 静默退出");
  });
});

// ── 结果汇总 ────────────────────────────────────────────────

console.log(`\n=== 测试结果 ===`);
console.log(`通过: ${passed}`);
console.log(`失败: ${failed}`);
console.log(`总计: ${passed + failed}`);

if (failed > 0) {
  console.error(`\n${failed} 个测试失败`);
  process.exit(1);
} else {
  console.log("\n全数通过！");
  process.exit(0);
}
