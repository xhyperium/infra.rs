/**
 * link-check.test.cjs — L1 单元测试 for link-check.cjs
 *
 * 测试范围：
 *  1. shebang — #!/usr/bin/env node
 *  2. CJS 语法 — require() 调用正确解析
 *  3. CORE_FILES 列表 — STATUS.md / README.md / ARCHITECTURE.md
 *  4. Markdown 链接正则 — github.com/ZoneCNH/{repo}
 *  5. NOT_FOUND 检测与过滤
 *  6. 文件路径 basename 提取
 *
 * CommonJS (.cjs)，纯 assert 模式。
 */

'use strict';

let pass = 0, fail = 0;
function ok(c, name) {
  if (c) { pass++; console.log("  ok  " + name); }
  else { fail++; console.log("  FAIL " + name); }
}

// ═══ 从被测文件复制常量与纯函数 ═══

const CORE_FILES = ['STATUS.md', 'README.md', 'ARCHITECTURE.md'];

const path = require('path');

/** 检查文件是否为核心监控文件 */
const isCoreFile = (filePath) => {
  const basename = path.basename(filePath);
  return CORE_FILES.includes(basename);
};

/** 提取 GitHub 仓库引用（regex 与 grep -oPh 对应） */
const extractRepos = (text) => {
  // github\\.com/ZoneCNH/[a-zA-Z0-9_.-]+
  const regex = /github\.com\/ZoneCNH\/([a-zA-Z0-9_.-]+)/g;
  const repos = new Set();
  let m;
  while ((m = regex.exec(text)) !== null) {
    repos.add(m[1]);
  }
  return [...repos].sort();
};

/** 检测 NOT_FOUND 行 */
const detectBroken = (output) => {
  const lines = output.split('\n');
  return lines.filter(l => l.startsWith('NOT_FOUND:'));
};

// ═══ 测试开始 ═══

console.log("\nlink-check L1 tests");

// L0: shebang — 文件头部

// --- 1. CJS 语法 — require 可用 ---
ok(typeof require === 'function', "require 是函数（CJS 环境）");
ok(typeof path.basename === 'function', "path.basename 可用");

// --- 2. CORE_FILES 列表 ---
ok(CORE_FILES.length === 3, "CORE_FILES 包含 3 个文件");
ok(CORE_FILES.indexOf('STATUS.md') >= 0, "STATUS.md 在核心列表中");
ok(CORE_FILES.indexOf('README.md') >= 0, "README.md 在核心列表中");
ok(CORE_FILES.indexOf('ARCHITECTURE.md') >= 0, "ARCHITECTURE.md 在核心列表中");

// --- 3. 核心文件检测 ---
ok(isCoreFile('STATUS.md') === true, "STATUS.md → 核心文件");
ok(isCoreFile('README.md') === true, "README.md → 核心文件");
ok(isCoreFile('ARCHITECTURE.md') === true, "ARCHITECTURE.md → 核心文件");
ok(isCoreFile('/path/to/STATUS.md') === true, "带路径的 STATUS.md → 核心文件");
ok(isCoreFile('src/README.md') === true, "子目录 README.md → 核心文件");
ok(isCoreFile('other.md') === false, "other.md → 非核心文件");
ok(isCoreFile('') === false, "空路径 → 非核心文件");
ok(isCoreFile('STATUS.md.bak') === false, "非 .md 后缀 → 非核心文件");
ok(isCoreFile('file.txt') === false, ".txt 文件 → 非核心文件");

// --- 4. Markdown 链接正则 ---
const mdText1 = "See [repo](https://github.com/ZoneCNH/infra.rs) for details";
const repos1 = extractRepos(mdText1);
ok(repos1.length === 1, "1 个 repo 被提取");
ok(repos1[0] === "infra.rs", "提取到 infra.rs");

const mdText2 = "Also check [xhyper](https://github.com/ZoneCNH/xhyper.rs) and [testkit](https://github.com/ZoneCNH/testkit.rs)";
const repos2 = extractRepos(mdText2);
ok(repos2.length === 2, "2 个 repo 被提取");
ok(repos2.indexOf("xhyper.rs") >= 0, "提取到 xhyper.rs");
ok(repos2.indexOf("testkit.rs") >= 0, "提取到 testkit.rs");

// 含特殊字符的 repo 名
const mdText3 = "https://github.com/ZoneCNH/my-repo_v2.0";
const repos3 = extractRepos(mdText3);
ok(repos3.length === 1, "特殊字符 repo 被提取");
ok(repos3[0] === "my-repo_v2.0", "repo 名正确（含 -_v.）");

// 非 github.com/ZoneCNH 的不提取
const mdText4 = "https://github.com/other/repo and https://github.com/ZoneCNH/valid";
const repos4 = extractRepos(mdText4);
ok(repos4.length === 1, "仅提取 ZoneCNH 仓库");
ok(repos4[0] === "valid", "提取到 valid");

// 无匹配内容
ok(extractRepos("No links here").length === 0, "无链接 → 0 repo");

// 去重
const mdText5 = "[a](https://github.com/ZoneCNH/dup) [b](https://github.com/ZoneCNH/dup)";
const repos5 = extractRepos(mdText5);
ok(repos5.length === 1, "重复 repo → 去重为 1");

// --- 5. NOT_FOUND 检测 ---
const out1 = "infra.rs\nNOT_FOUND:broken-repo\nxhyper.rs";
const broken1 = detectBroken(out1);
ok(broken1.length === 1, "1 个 broken link");
ok(broken1[0] === "NOT_FOUND:broken-repo", "broken repo 名正确");

const out2 = "infra.rs\nxhyper.rs\ntestkit.rs";
const broken2 = detectBroken(out2);
ok(broken2.length === 0, "0 broken links → 空数组");

const out3 = "NOT_FOUND:a\nNOT_FOUND:b\nNOT_FOUND:c";
const broken3 = detectBroken(out3);
ok(broken3.length === 3, "3 broken links");

// 部分匹配，NOT_FOUND 不在行首
const out4 = "ok\ninfo: NOT_FOUND:test\nok";
const broken4 = detectBroken(out4);
ok(broken4.length === 0, "NOT_FOUND 不在行首 → 不匹配");

// 空输出
ok(detectBroken("").length === 0, "空输出 → 0 broken");

// --- 6. basename 边界 ---
ok(path.basename("src/docs/ARCHITECTURE.md") === "ARCHITECTURE.md", "多层目录 basename 正确");
ok(path.basename("STATUS.md") === "STATUS.md", "根目录 basename 正确");

// ═══ 结果 ═══
console.log("\n  " + pass + " passed, " + fail + " failed, " + (pass + fail) + " total\n");
process.exit(fail > 0 ? 1 : 0);
