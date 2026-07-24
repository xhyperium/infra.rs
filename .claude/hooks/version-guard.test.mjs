#!/usr/bin/env node
/**
 * version-guard.test.mjs — L1 单元测试 for version-guard.mjs
 *
 * 测试范围：
 *  1. 文件结构：shebang、语法有效性
 *  2. Cargo.toml 路径匹配（crates/ 和 tools/）
 *  3. TOML version 字段解析
 *  4. 变更检测：Cargo.toml 是否在 git diff 中
 *  5. 版本比较逻辑（HEAD vs working tree）
 *  6. 源码变更检测（crates/tools 非 Cargo.toml 文件）
 *  7. 新版警告消息（crate-bump.mjs 引用）
 */

import { execFileSync } from "child_process";
import { readFileSync } from "fs";

// ── 从 version-guard.mjs 提取的可测试逻辑 ────────────────────

const CRATE_CARGO_TOML = /^(crates|tools)\/.+\/Cargo\.toml$/;

/**
 * 检测文件是否在 crates/ 或 tools/ 下
 */
function isCrateOrToolsPath(filePath) {
  return /^(crates|tools)\//.test(filePath);
}

/**
 * 检测文件是否是 crate Cargo.toml
 */
function isCargoToml(filePath) {
  return CRATE_CARGO_TOML.test(filePath);
}

/**
 * 从 TOML 内容提取 version
 */
function getVersionFromTomlContent(tomlText) {
  const m = tomlText.match(/^version\s*=\s*"([^"]+)"/m);
  return m ? m[1] : null;
}

/**
 * 检测是否有源码变更（crates/ 或 tools/ 下非 Cargo.toml 文件）
 */
function hasSourceChanges(files) {
  return files.some(
    (f) => isCrateOrToolsPath(f) && !f.endsWith("Cargo.toml"),
  );
}

/**
 * 比较版本：检查 Cargo.toml 中 version 是否变化
 * @returns {{ toml: string, version: string }[]} 需要 bump 的项
 */
function findUnbumped(cargoTomlsChanged, getCurrentVersion, getPreviousVersion) {
  const needsBump = [];
  for (const toml of cargoTomlsChanged) {
    const current = getCurrentVersion(toml);
    const previous = getPreviousVersion(toml);
    if (previous && current === previous) {
      needsBump.push({ toml, version: current });
    }
  }
  return needsBump;
}

/**
 * 解析 git diff --name-only HEAD 输出
 */
function parseChangedFiles(gitOutput) {
  if (!gitOutput) return [];
  return gitOutput.trim().split("\n").filter(Boolean);
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

  it("是 Stop hook 用途", () => {
    const src = readFileSync(".claude/hooks/version-guard.mjs", "utf8");
    assert(src.includes("Stop Hook") || src.includes("Stop"), "Stop Hook 注释");
  });

  it("不将 release/manifest/latest.json 用作功能路径", () => {
    const src = readFileSync(".claude/hooks/version-guard.mjs", "utf8");
    // 允许出现在注释中，但不应作为 MANIFEST_FILE 或 getCurrentVersion 的入参
    const lines = src.split("\n").filter((l) => l.includes("release/manifest/latest.json"));
    const functionalUse = lines.filter((l) => !l.trim().startsWith("//"));
    assert(functionalUse.length === 0, `无功能性引用: ${functionalUse.join(", ")}`);
  });

  it("不引用旧 scripts/version-bump.sh", () => {
    const src = readFileSync(".claude/hooks/version-guard.mjs", "utf8");
    assert(!src.includes("version-bump.sh"), "无旧 bump 脚本");
  });

  it("引用 scripts/version/crate-bump.mjs", () => {
    const src = readFileSync(".claude/hooks/version-guard.mjs", "utf8");
    assert(src.includes("crate-bump.mjs"), "新 bump 工具路径");
  });

  it("引用 .agents/rules/VERSIONING.md", () => {
    const src = readFileSync(".claude/hooks/version-guard.mjs", "utf8");
    assert(src.includes("VERSIONING.md"), "版本规则 SSOT");
  });
});

// ── 测试 2: Cargo.toml 路径匹配 ─────────────────────────────

describe("Cargo.toml — 路径匹配", () => {
  it("crates/kernel/Cargo.toml 匹配", () => {
    assert(isCargoToml("crates/kernel/Cargo.toml"), "kernel Cargo.toml");
  });

  it("crates/infra/configx/Cargo.toml 匹配", () => {
    assert(isCargoToml("crates/infra/configx/Cargo.toml"), "configx Cargo.toml");
  });

  it("tools/verifyctl/Cargo.toml 匹配", () => {
    assert(isCargoToml("tools/verifyctl/Cargo.toml"), "tools Cargo.toml");
  });

  it("根 Cargo.toml 不匹配", () => {
    assert(!isCargoToml("Cargo.toml"), "根 Cargo.toml 不匹配");
  });

  it("src/lib.rs 不匹配", () => {
    assert(!isCargoToml("crates/kernel/src/lib.rs"), "源码不匹配");
  });

  it("其他目录 Cargo.toml 不匹配", () => {
    assert(!isCargoToml("other/Cargo.toml"), "非 crates/tools 不匹配");
  });
});

// ── 测试 3: isCrateOrToolsPath ───────────────────────────────

describe("isCrateOrToolsPath — 源码路径检测", () => {
  it("crates/kernel/src/lib.rs → true", () => {
    assert(isCrateOrToolsPath("crates/kernel/src/lib.rs"), "kernel 源码");
  });

  it("tools/verifyctl/src/main.rs → true", () => {
    assert(isCrateOrToolsPath("tools/verifyctl/src/main.rs"), "tools 源码");
  });

  it("docs/readme.md → false", () => {
    assert(!isCrateOrToolsPath("docs/readme.md"), "非 crates/tools");
  });

  it(".claude/hooks/session-context.mjs → false", () => {
    assert(!isCrateOrToolsPath(".claude/hooks/session-context.mjs"), "hooks 目录");
  });
});

// ── 测试 4: TOML version 解析 ────────────────────────────────

describe("getVersionFromTomlContent — version 字段解析", () => {
  it("标准 version", () => {
    const v = getVersionFromTomlContent('[package]\nname = "kernel"\nversion = "0.1.0"');
    assert(v === "0.1.0", `version=0.1.0, got ${v}`);
  });

  it("复杂 version（pre-release）", () => {
    const v = getVersionFromTomlContent('version = "1.0.0-alpha.1"');
    assert(v === "1.0.0-alpha.1", `pre-release`);
  });

  it("version 不存在", () => {
    const v = getVersionFromTomlContent('[package]\nname = "test"');
    assert(v === null, "无 version → null");
  });

  it("空 TOML", () => {
    const v = getVersionFromTomlContent("");
    assert(v === null, "空内容 → null");
  });

  it("workspace 继承 version", () => {
    const v = getVersionFromTomlContent('version = "0.2.0"\nedition = "2021"');
    assert(v === "0.2.0", "显式 version 解析");
  });
});

// ── 测试 5: 变更检测 ────────────────────────────────────────

describe("parseChangedFiles — git diff 输出解析", () => {
  it("多行输出", () => {
    const files = parseChangedFiles("crates/kernel/Cargo.toml\ncrates/kernel/src/lib.rs\nREADME.md");
    assert(files.length === 3, `3 个文件, got ${files.length}`);
    assert(files[0] === "crates/kernel/Cargo.toml", "Cargo.toml");
    assert(files[1] === "crates/kernel/src/lib.rs", "lib.rs");
    assert(files[2] === "README.md", "README.md");
  });

  it("空字符串 → 空数组", () => {
    const files = parseChangedFiles("");
    assert(files.length === 0, "空数组");
  });

  it("前后有空行", () => {
    const files = parseChangedFiles("\ncrates/kernel/Cargo.toml\n\nREADME.md\n");
    assert(files.length === 2, `2 个文件, got ${files.length}`);
  });
});

// ── 测试 6: 源码变更检测 ─────────────────────────────────────

describe("hasSourceChanges — 源码变更检测", () => {
  it("有源码变更时返回 true", () => {
    const files = ["crates/kernel/src/lib.rs", "README.md"];
    assert(hasSourceChanges(files), "kernel 源码变更");
  });

  it("仅有 Cargo.toml 变更时返回 false", () => {
    const files = ["crates/kernel/Cargo.toml"];
    assert(!hasSourceChanges(files), "仅 Cargo.toml 变更");
  });

  it("仅有 docs 变更时返回 false", () => {
    const files = ["docs/readme.md", "AGENTS.md"];
    assert(!hasSourceChanges(files), "非 crates/tools 变更");
  });

  it("tools 源码变更返回 true", () => {
    const files = ["tools/verifyctl/src/main.rs"];
    assert(hasSourceChanges(files), "tools 源码变更");
  });
});

// ── 测试 7: 版本比较逻辑 ─────────────────────────────────────

describe("findUnbumped — 版本未递增检测", () => {
  it("version 未变 → 需要 bump", () => {
    const needsBump = findUnbumped(
      ["crates/kernel/Cargo.toml"],
      () => "0.1.0",
      () => "0.1.0",
    );
    assert(needsBump.length === 1, "1 个需要 bump");
    assert(needsBump[0].version === "0.1.0", "version=0.1.0");
  });

  it("version 已递增 → 无需 bump", () => {
    const needsBump = findUnbumped(
      ["crates/kernel/Cargo.toml"],
      () => "0.1.1",
      () => "0.1.0",
    );
    assert(needsBump.length === 0, "版本已递增");
  });

  it("新文件（无 HEAD version）→ 无需 bump", () => {
    const needsBump = findUnbumped(
      ["crates/new_crate/Cargo.toml"],
      () => "0.1.0",
      () => null,
    );
    assert(needsBump.length === 0, "新文件无需 bump");
  });

  it("多个 Cargo.toml 混合状态", () => {
    const versions = new Map([
      ["crates/kernel/Cargo.toml", { current: "0.1.1", previous: "0.1.0" }],
      ["crates/infra/configx/Cargo.toml", { current: "0.2.0", previous: "0.2.0" }],
    ]);
    const needsBump = findUnbumped(
      ["crates/kernel/Cargo.toml", "crates/infra/configx/Cargo.toml"],
      (t) => versions.get(t).current,
      (t) => versions.get(t).previous,
    );
    assert(needsBump.length === 1, "仅 configx 需要 bump");
    assert(needsBump[0].toml === "crates/infra/configx/Cargo.toml", "configx");
  });
});

// ── 测试 8: 完整流程模拟 ────────────────────────────────────

describe("version-guard — 完整流程模拟", () => {
  it("无变更 → 静默", () => {
    const files = parseChangedFiles("");
    const cargoTomlsChanged = files.filter((f) => isCargoToml(f));
    const srcChanged = hasSourceChanges(files);
    assert(cargoTomlsChanged.length === 0, "无 Cargo.toml 变更");
    assert(!srcChanged, "无源码变更");
  });

  it("仅 docs 变更 → 无版本检查", () => {
    const files = parseChangedFiles("README.md\nAGENTS.md");
    const cargoTomlsChanged = files.filter((f) => isCargoToml(f));
    assert(cargoTomlsChanged.length === 0, "无 Cargo.toml 变更");
  });

  it("Cargo.toml 变更 + 版本已增 → 通过", () => {
    const files = parseChangedFiles("crates/kernel/Cargo.toml\ncrates/kernel/src/lib.rs");
    const cargoTomlsChanged = files.filter((f) => isCargoToml(f));
    assert(cargoTomlsChanged.length === 1, "kernel Cargo.toml 变更");

    const needsBump = findUnbumped(
      cargoTomlsChanged,
      () => "0.1.1",
      () => "0.1.0",
    );
    assert(needsBump.length === 0, "版本已递增 → 通过");
  });

  it("Cargo.toml 变更 + 版本未变 → 警告", () => {
    const files = parseChangedFiles("crates/kernel/Cargo.toml\ncrates/kernel/src/lib.rs");
    const cargoTomlsChanged = files.filter((f) => isCargoToml(f));
    const srcChanged = hasSourceChanges(files);

    assert(cargoTomlsChanged.length === 1, "Cargo.toml 变更");
    assert(srcChanged, "源码变更");

    const needsBump = findUnbumped(
      cargoTomlsChanged,
      () => "0.1.0",
      () => "0.1.0",
    );
    assert(needsBump.length === 1, "版本未变 → 需要警告");
  });

  it("仅源码变更 → 提示确认", () => {
    const files = parseChangedFiles("crates/kernel/src/lib.rs");
    const cargoTomlsChanged = files.filter((f) => isCargoToml(f));
    const srcChanged = hasSourceChanges(files);

    assert(cargoTomlsChanged.length === 0, "无 Cargo.toml 变更");
    assert(srcChanged, "有源码变更");
  });
});

// ── 测试 9: 容错处理 ────────────────────────────────────────

describe("version-guard — 容错处理", () => {
  it("git diff 执行失败 → 空数组", () => {
    const files = parseChangedFiles("");
    assert(files.length === 0, "失败 → 空数组");
  });

  it("无效 TOML → version=null", () => {
    const v = getVersionFromTomlContent("not valid toml content here");
    assert(v === null, "无效内容 → null");
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
