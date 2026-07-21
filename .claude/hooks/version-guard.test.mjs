#!/usr/bin/env node
/**
 * version-guard.test.mjs — L1 单元测试 for version-guard.mjs
 *
 * 测试范围：
 *  1. 文件结构：shebang、语法有效性
 *  2. TRACKED_FILES 列表内容
 *  3. SPEC_GLOB 模式匹配
 *  4. getCurrentVersion — 版本 JSON 解析
 *  5. 文件变更检测：追踪文件过滤逻辑
 *  6. 警告消息：manifest 变更/未变更 两种情况
 *  7. 版本文件读取的容错处理
 */

import { execFileSync } from "child_process";
import { readFileSync } from "fs";

// ── 从 version-guard.mjs 提取的可测试逻辑 ────────────────────

const TRACKED_FILES = [
  "STATUS.md", "README.md", "ARCHITECTURE.md", "module/README.md",
  "CLAUDE.md", "AGENTS.md", "CONSTITUTION.md",
  ".repo-contract.yaml", ".foundationx/repo-contract.json",
  ".foundationx/status/index.json", ".foundationx/blockers.json",
  "foundation-bom.yaml",
];

const SPEC_GLOB = /^module\/[^/]+\/SPEC\.md$/;

const MANIFEST_FILE = "release/manifest/latest.json";

/**
 * 过滤出被追踪的变更文件
 */
function filterTrackedChanged(allChangedFiles) {
  return allChangedFiles.filter((f) =>
    TRACKED_FILES.includes(f) || SPEC_GLOB.test(f)
  );
}

/**
 * 解析版本信息（模拟 getCurrentVersion 逻辑）
 */
function parseVersion(manifestJson) {
  try {
    const data = JSON.parse(manifestJson);
    return data.version || null;
  } catch {
    return null;
  }
}

/**
 * 解析 git diff --name-only HEAD 输出
 */
function parseChangedFiles(gitOutput) {
  if (!gitOutput) return [];
  return gitOutput.trim().split("\n").filter(Boolean);
}

/**
 * 检查 manifest 是否在变更列表中
 */
function isManifestChanged(changedFiles) {
  return changedFiles.includes(MANIFEST_FILE);
}

/**
 * 构造 version-guard 警告消息
 */
function buildVersionWarning(trackedChanged, currentVersion, lastBumpInfo, manifestChanged) {
  const lines = [];
  lines.push("");
  lines.push("══════════════════════════════════════════════════════");
  lines.push("[VersionGuard] 📋 本次会话修改了追踪文件，检查版本号...");
  lines.push("");
  lines.push(`  当前版本: ${currentVersion || "(未找到)"}`);
  lines.push(`  上次 bump: ${lastBumpInfo || "(无记录)"}`);
  lines.push(`  manifest 已变更: ${manifestChanged ? "是 ✅" : "否 ⚠️"}`);
  lines.push("");
  lines.push("  修改的追踪文件:");
  for (const f of trackedChanged) {
    lines.push(`    - ${f}`);
  }
  lines.push("");

  if (!manifestChanged) {
    lines.push("  ⚠️  警告: 追踪文件已修改但 release/manifest/latest.json 版本号未变！");
    lines.push("");
    lines.push("  CLAUDE.md 规则: 每次更新迭代，版本号都要 +1");
    lines.push("");
    lines.push("  自动 bump:");
    lines.push("    $ ./scripts/version-bump.sh              # patch bump");
    lines.push("    $ ./scripts/version-bump.sh --level minor # minor bump");
    lines.push("    $ ./scripts/version-bump.sh --dry-run      # 预览");
    lines.push("");
    lines.push("  Bump 级别选择:");
    lines.push("    PATCH: 错字修复、链接更新、说明澄清");
    lines.push("    MINOR: 新增模块/章节、架构描述变更");
    lines.push("    MAJOR: 治理体系重构、顶层架构重写");
  } else {
    lines.push("  ✅ manifest 版本号已变更，版本递增规则已满足。");
  }

  lines.push("══════════════════════════════════════════════════════");

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

describe("version-guard.mjs — 文件结构", () => {
  it("shebang 存在", () => {
    const src = readFileSync(".claude/hooks/version-guard.mjs", "utf8");
    assert(src.startsWith("#!/usr/bin/env node"), "首行 shebang");
  });

  it("语法有效（node --check）", () => {
    try {
      execFileSync("node", ["--check", ".claude/hooks/version-guard.mjs"], {
        stdio: "pipe",
        timeout: 5000,
      });
      assert(true, "node --check 通过");
    } catch (e) {
      assert(false, `node --check 失败: ${e.stderr?.toString() || e.message}`);
    }
  });

  it("包含 MANIFEST_FILE 路径", () => {
    const src = readFileSync(".claude/hooks/version-guard.mjs", "utf8");
    assert(src.includes("release/manifest/latest.json"), "MANIFEST_FILE 存在");
  });

  it("是 Stop hook 用途", () => {
    const src = readFileSync(".claude/hooks/version-guard.mjs", "utf8");
    assert(src.includes("Stop Hook") || src.includes("Stop"), "Stop Hook 注释");
  });
});

// ── 测试 2: TRACKED_FILES 列表 ──────────────────────────────

describe("TRACKED_FILES — 追踪文件列表", () => {
  it("包含核心文档", () => {
    assert(TRACKED_FILES.includes("STATUS.md"), "STATUS.md");
    assert(TRACKED_FILES.includes("README.md"), "README.md");
    assert(TRACKED_FILES.includes("ARCHITECTURE.md"), "ARCHITECTURE.md");
  });

  it("包含治理文档", () => {
    assert(TRACKED_FILES.includes("CLAUDE.md"), "CLAUDE.md");
    assert(TRACKED_FILES.includes("AGENTS.md"), "AGENTS.md");
    assert(TRACKED_FILES.includes("CONSTITUTION.md"), "CONSTITUTION.md");
  });

  it("包含 foundation 文件", () => {
    assert(
      TRACKED_FILES.includes(".repo-contract.yaml"),
      ".repo-contract.yaml"
    );
    assert(
      TRACKED_FILES.includes(".foundationx/repo-contract.json"),
      ".foundationx/repo-contract.json"
    );
    assert(
      TRACKED_FILES.includes(".foundationx/status/index.json"),
      ".foundationx/status/index.json"
    );
    assert(
      TRACKED_FILES.includes(".foundationx/blockers.json"),
      ".foundationx/blockers.json"
    );
  });

  it("包含 module/README.md", () => {
    assert(TRACKED_FILES.includes("module/README.md"), "module/README.md");
  });

  it("包含 foundation-bom.yaml", () => {
    assert(TRACKED_FILES.includes("foundation-bom.yaml"), "foundation-bom.yaml");
  });

  it("TRACKED_FILES 数量为 12", () => {
    assert(TRACKED_FILES.length === 12, `12 个追踪文件, got ${TRACKED_FILES.length}`);
  });
});

// ── 测试 3: SPEC_GLOB 模式匹配 ──────────────────────────────

describe("SPEC_GLOB — module/*/SPEC.md 匹配", () => {
  it("module/kernel/SPEC.md 匹配", () => {
    assert(SPEC_GLOB.test("module/kernel/SPEC.md"), "kernel/SPEC.md");
  });

  it("module/testkit/SPEC.md 匹配", () => {
    assert(SPEC_GLOB.test("module/testkit/SPEC.md"), "testkit/SPEC.md");
  });

  it("module/configx/SPEC.md 匹配", () => {
    assert(SPEC_GLOB.test("module/configx/SPEC.md"), "configx/SPEC.md");
  });

  it("module/deep/sub/SPEC.md 不匹配（非直接目录）", () => {
    assert(!SPEC_GLOB.test("module/deep/sub/SPEC.md"), "嵌套目录不匹配");
  });

  it("module/SPEC.md 不匹配（缺少 Module 目录）", () => {
    assert(!SPEC_GLOB.test("module/SPEC.md"), "缺少目录层级");
  });

  it("SPEC.md 不匹配", () => {
    assert(!SPEC_GLOB.test("SPEC.md"), "直接 SPEC.md 不匹配");
  });

  it("module/testkit/README.md 不匹配", () => {
    assert(!SPEC_GLOB.test("module/testkit/README.md"), "非 SPEC.md 文件");
  });

  it("other/module/test/SPEC.md 不匹配", () => {
    assert(!SPEC_GLOB.test("other/module/test/SPEC.md"), "不以 module/ 开头");
  });

  it("module/testkit/spec.md 不匹配（大小写）", () => {
    assert(!SPEC_GLOB.test("module/testkit/spec.md"), "小写 spec.md 不匹配");
  });
});

// ── 测试 4: 版本 JSON 解析 ──────────────────────────────────

describe("parseVersion — 版本 JSON 解析", () => {
  it("标准版本格式", () => {
    const v = parseVersion('{"version":"2.3.1","name":"infra.rs"}');
    assert(v === "2.3.1", `version=2.3.1, got ${v}`);
  });

  it("无 version 字段", () => {
    const v = parseVersion('{"name":"infra.rs","description":"test"}');
    assert(v === null, `无 version → null, got ${v}`);
  });

  it("无效 JSON", () => {
    const v = parseVersion("not json");
    assert(v === null, "无效 JSON → null");
  });

  it("空字符串", () => {
    const v = parseVersion("");
    assert(v === null, "空字符串 → null");
  });

  it("version 为空字符串", () => {
    const v = parseVersion('{"version":""}');
    assert(v === null, `空 version → null, got ${v}`);
  });

  it("version 为 0.0.0", () => {
    const v = parseVersion('{"version":"0.0.0"}');
    assert(v === "0.0.0", "version=0.0.0");
  });

  it("version 为语义化版本", () => {
    const v = parseVersion('{"version":"1.0.0-alpha.1"}');
    assert(v === "1.0.0-alpha.1", "semver pre-release");
  });
});

// ── 测试 5: 文件变更检测 ────────────────────────────────────

describe("parseChangedFiles — git diff 输出解析", () => {
  it("多行输出", () => {
    const files = parseChangedFiles("STATUS.md\nREADME.md\ncrates/kernel/src/lib.rs");
    assert(files.length === 3, `3 个文件, got ${files.length}`);
    assert(files[0] === "STATUS.md", "STATUS.md");
    assert(files[1] === "README.md", "README.md");
    assert(files[2] === "crates/kernel/src/lib.rs", "lib.rs");
  });

  it("单行输出", () => {
    const files = parseChangedFiles("STATUS.md");
    assert(files.length === 1, "1 个文件");
    assert(files[0] === "STATUS.md", "STATUS.md");
  });

  it("空字符串 → 空数组", () => {
    const files = parseChangedFiles("");
    assert(files.length === 0, "空数组");
  });

  it("前后有空行", () => {
    const files = parseChangedFiles("\nSTATUS.md\n\nREADME.md\n");
    assert(files.length === 2, `2 个文件, got ${files.length}`);
  });

  it("git 无变更输出", () => {
    const files = parseChangedFiles(null);
    assert(files.length === 0, "null → 空数组");
  });
});

describe("filterTrackedChanged — 追踪文件过滤", () => {
  it("全追踪文件", () => {
    const all = ["STATUS.md", "README.md"];
    const result = filterTrackedChanged(all);
    assert(result.length === 2, `2 个被追踪, got ${result.length}`);
  });

  it("部分追踪", () => {
    const all = ["STATUS.md", "src/lib.rs", "tests/test.mjs"];
    const result = filterTrackedChanged(all);
    assert(result.length === 1, "仅 STATUS.md 被追踪");
    assert(result[0] === "STATUS.md", "STATUS.md");
  });

  it("全部非追踪", () => {
    const all = ["src/lib.rs", "tests/test.mjs"];
    const result = filterTrackedChanged(all);
    assert(result.length === 0, "0 个追踪文件");
  });

  it("空变更列表", () => {
    const result = filterTrackedChanged([]);
    assert(result.length === 0, "空列表");
  });

  it("SPEC.md 匹配", () => {
    const all = ["module/kernel/SPEC.md", "src/lib.rs"];
    const result = filterTrackedChanged(all);
    assert(result.length === 1, "1 个追踪");
    assert(result[0] === "module/kernel/SPEC.md", "module/kernel/SPEC.md");
  });

  it("CLAUDE.md 和 AGENTS.md 都被追踪", () => {
    const all = ["CLAUDE.md", "AGENTS.md", "src/main.rs"];
    const result = filterTrackedChanged(all);
    assert(result.length === 2, "2 个追踪");
    assert(result.includes("CLAUDE.md"), "CLAUDE.md");
    assert(result.includes("AGENTS.md"), "AGENTS.md");
  });

  it("foundation 文件被追踪", () => {
    const all = [
      ".foundationx/status/index.json",
      ".foundationx/blockers.json",
      "src/lib.rs",
    ];
    const result = filterTrackedChanged(all);
    assert(result.length === 2, "2 个 foundation 文件");
  });
});

describe("isManifestChanged — manifest 变更检测", () => {
  it("manifest 在变更列表中", () => {
    assert(
      isManifestChanged(["STATUS.md", "release/manifest/latest.json"]),
      "manifest 已变更"
    );
  });

  it("manifest 不在变更列表中", () => {
    assert(
      !isManifestChanged(["STATUS.md", "README.md"]),
      "manifest 未变更"
    );
  });

  it("空变更列表", () => {
    assert(!isManifestChanged([]), "空列表 → manifest 未变更");
  });
});

// ── 测试 6: 警告消息 ─────────────────────────────────────────

describe("buildVersionWarning — 警告消息（manifest 未变更）", () => {
  it("包含 [VersionGuard] 标签", () => {
    const msg = buildVersionWarning(["STATUS.md"], "1.0.0", "", false);
    assert(msg.includes("[VersionGuard]"), "[VersionGuard] 标签");
  });

  it("包含当前版本", () => {
    const msg = buildVersionWarning(["STATUS.md"], "2.3.1", "", false);
    assert(msg.includes("2.3.1"), "当前版本 2.3.1");
  });

  it("版本为空时显示 (未找到)", () => {
    const msg = buildVersionWarning(["STATUS.md"], null, "", false);
    assert(msg.includes("(未找到)"), "(未找到) 占位");
  });

  it("包含修改的追踪文件", () => {
    const msg = buildVersionWarning(["STATUS.md", "CLAUDE.md"], "1.0", "", false);
    assert(msg.includes("STATUS.md"), "STATUS.md");
    assert(msg.includes("CLAUDE.md"), "CLAUDE.md");
  });

  it("包含上次 bump 信息", () => {
    const msg = buildVersionWarning(
      ["STATUS.md"],
      "1.0",
      "abc123 bump: v1.0.0",
      false
    );
    assert(msg.includes("abc123 bump: v1.0.0"), "上次 bump 信息");
  });

  it("上次 bump 为空时显示 (无记录)", () => {
    const msg = buildVersionWarning(["STATUS.md"], "1.0", "", false);
    assert(msg.includes("(无记录)"), "(无记录) 占位");
  });

  it("manifest 未变更 → 显示警告", () => {
    const msg = buildVersionWarning(["STATUS.md"], "1.0", "", false);
    assert(
      msg.includes("版本号未变"),
      "版本号未变 警告"
    );
    assert(
      msg.includes("CLAUDE.md 规则: 每次更新迭代，版本号都要 +1"),
      "CLAUDE.md 规则"
    );
    assert(msg.includes("scripts/version-bump.sh"), "bump 脚本引用");
  });

  it("包含 Bump 级别选择说明", () => {
    const msg = buildVersionWarning(["STATUS.md"], "1.0", "", false);
    assert(msg.includes("PATCH:"), "PATCH 级别");
    assert(msg.includes("MINOR:"), "MINOR 级别");
    assert(msg.includes("MAJOR:"), "MAJOR 级别");
  });
});

describe("buildVersionWarning — 警告消息（manifest 已变更）", () => {
  it("manifest 已变更 → 显示满足", () => {
    const msg = buildVersionWarning(["STATUS.md"], "1.0", "abc bump", true);
    assert(
      msg.includes("版本递增规则已满足"),
      "版本递增规则已满足"
    );
    assert(msg.includes("✅"), "通过标记");
  });

  it("manifest 已变更 → 不显示 Bump 级别选择", () => {
    const msg = buildVersionWarning(["STATUS.md"], "1.0", "abc bump", true);
    assert(!msg.includes("PATCH:"), "不显示 PATCH 级别");
    assert(!msg.includes("MINOR:"), "不显示 MINOR 级别");
  });

  it("manifest 已变更 → 不显示警告 ⚠️", () => {
    const msg = buildVersionWarning(["STATUS.md"], "1.0", "abc bump", true);
    assert(!msg.includes("版本号未变"), "无不匹配警告");
  });
});

// ── 测试 7: 完整流程模拟 ────────────────────────────────────

describe("version-guard — 完整流程模拟", () => {
  it("流程: 无变更文件 → 静默退出", () => {
    const changedFiles = parseChangedFiles("");
    const trackedChanged = filterTrackedChanged(changedFiles);
    assert(trackedChanged.length === 0, "无追踪文件变更 → 静默退出");
  });

  it("流程: 变更但非追踪文件 → 静默退出", () => {
    const changedFiles = parseChangedFiles("src/lib.rs\ntests/test.mjs");
    const trackedChanged = filterTrackedChanged(changedFiles);
    assert(trackedChanged.length === 0, "非追踪文件 → 静默退出");
  });

  it("流程: 追踪文件变更 + manifest 未变更 → 输出警告", () => {
    const changedFiles = parseChangedFiles("STATUS.md\nREADME.md\nsrc/lib.rs");
    const trackedChanged = filterTrackedChanged(changedFiles);
    assert(trackedChanged.length === 2, "2 个追踪文件变更");

    const currentVersion = parseVersion('{"version":"1.0.0"}');
    const manifestChanged = isManifestChanged(changedFiles);
    assert(!manifestChanged, "manifest 未变更");

    const msg = buildVersionWarning(trackedChanged, currentVersion, "abc bump", manifestChanged);
    assert(msg.includes("[VersionGuard]"), "输出警告");
    assert(msg.includes("版本号未变"), "警告内容");
  });

  it("流程: 追踪文件变更 + manifest 已变更 → 通过", () => {
    const changedFiles = parseChangedFiles(
      "STATUS.md\nrelease/manifest/latest.json"
    );
    const trackedChanged = filterTrackedChanged(changedFiles);
    assert(trackedChanged.length === 1, "STATUS.md 追踪");

    const manifestChanged = isManifestChanged(changedFiles);
    assert(manifestChanged, "manifest 已变更");

    const msg = buildVersionWarning(
      trackedChanged,
      "2.0.0",
      "def bump v2",
      manifestChanged
    );
    assert(msg.includes("版本递增规则已满足"), "规则满足");
  });

  it("流程: module/SPEC.md 变更 + manifest 未变更 → 警告", () => {
    const changedFiles = parseChangedFiles("module/kernel/SPEC.md\nsrc/lib.rs");
    const trackedChanged = filterTrackedChanged(changedFiles);
    assert(trackedChanged.length === 1, "SPEC.md 被追踪");
    assert(trackedChanged[0] === "module/kernel/SPEC.md", "kernel SPEC.md");

    const manifestChanged = isManifestChanged(changedFiles);
    assert(!manifestChanged, "manifest 未变更");
  });

  it("流程: 多个 SPEC.md 变更", () => {
    const all = [
      "module/kernel/SPEC.md",
      "module/testkit/SPEC.md",
      "src/lib.rs",
    ];
    const trackedChanged = filterTrackedChanged(all);
    assert(trackedChanged.length === 2, "2 个 SPEC.md 被追踪");
  });

  it("流程: foundation 文件变更检测", () => {
    const all = [".foundationx/status/index.json", "src/main.rs"];
    const trackedChanged = filterTrackedChanged(all);
    assert(trackedChanged.length === 1, "foundation 文件被追踪");
  });
});

// ── 测试 8: 容错处理 ────────────────────────────────────────

describe("version-guard — 容错处理", () => {
  it("manifest 文件不存在 → version=null", () => {
    // 不存在的话 getCurrentVersion 返回 null
    // L1 测试：验证返回 null 时的行为
    const currentVersion = parseVersion("not valid json");
    assert(currentVersion === null, "无效 JSON → null");
  });

  it("git diff 执行失败 → 空数组", () => {
    // getChangedFiles 的 catch 返回空数组
    const files = parseChangedFiles("");
    assert(files.length === 0, "失败 → 空数组");
  });

  it("git log 获取失败 → 显示 (无记录)", () => {
    const msg = buildVersionWarning(["STATUS.md"], "1.0", "", false);
    assert(msg.includes("(无记录)"), "无 bump 记录 → 占位符");
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
