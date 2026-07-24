#!/usr/bin/env node
/**
 * check-contract-compliance.mjs — 合同合规验证脚本
 *
 * 依据 CONTRACT_SPEC.md 定义的 L1–L4 验证规则，对域合同执行结构化合规校验。
 *
 * 用法:
 *   node scripts/quality-gates/check-contract-compliance.mjs
 *   node scripts/quality-gates/check-contract-compliance.mjs --help
 *   node scripts/quality-gates/check-contract-compliance.mjs --level L1
 *   node scripts/quality-gates/check-contract-compliance.mjs --level L1,L2,L3
 *   node scripts/quality-gates/check-contract-compliance.mjs --contract SPEC-KERNEL-002
 *   node scripts/quality-gates/check-contract-compliance.mjs --fail-level L2
 *   node scripts/quality-gates/check-contract-compliance.mjs --json
 *   node scripts/quality-gates/check-contract-compliance.mjs --summary
 *
 * 退出码: 0 通过；1 有违规；2 用法错误
 */
import { existsSync, readFileSync, readdirSync } from "fs";
import { join, relative, resolve } from "path";
import { fileURLToPath } from "url";
import { spawnSync } from "child_process";

const __dir = fileURLToPath(new URL(".", import.meta.url));
const ROOT = resolve(__dir, "..", "..");
const SSOT_DIR = join(ROOT, ".agents", "ssot");

const args = process.argv.slice(2);

// CLI
function printHelp() {
  console.log(`check-contract-compliance.mjs — 合同合规验证 (L1–L4)

Usage:
  node scripts/quality-gates/check-contract-compliance.mjs [options]

Options:
  -h, --help              Show this help
  --level <L1|L2|L3|L4>   Run specified level(s) (repeatable or comma-separated; default: L1,L2)
  --contract <spec-id>    Only check specified contract (e.g. SPEC-KERNEL-002)
  --fail-level <level>    Blocking level (default: L2)
  --json                  Output JSON report
  --summary               Output GitHub step summary markdown

Env:
  CONTRACT_COMPLIANCE_FAIL_LEVEL=L1|L2   Same as --fail-level
`);
}

function parseArgs(argv) {
  const out = {
    help: false, levels: [], contract: null,
    failLevel: process.env.CONTRACT_COMPLIANCE_FAIL_LEVEL || "L2",
    json: false, summary: false,
  };
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    switch (a) {
      case "-h": case "--help": out.help = true; break;
      case "--level": { const val = argv[++i]; out.levels.push(...val.split(",").map(s => s.trim()).filter(Boolean)); break; }
      case "--contract": out.contract = argv[++i]; break;
      case "--fail-level": out.failLevel = argv[++i]; break;
      case "--json": out.json = true; break;
      case "--summary": out.summary = true; break;
      default: console.error("error: unknown argument: " + a); printHelp(); process.exit(2);
    }
  }
  if (out.levels.length === 0) out.levels = ["L1", "L2"];
  return out;
}

// 合同发现
function discoverContracts() {
  const specs = [];
  const srcDirs = [
    join(SSOT_DIR, "kernel", "spec"),
    join(SSOT_DIR, "testkit", "spec"),
    join(SSOT_DIR, "contracts", "spec"),
    join(SSOT_DIR, "infra", "configx", "spec"),
    join(SSOT_DIR, "infra", "schedulex", "spec"),
    join(SSOT_DIR, "infra", "bootstrap", "spec"),
    join(SSOT_DIR, "infra", "evidence", "spec"),
    join(SSOT_DIR, "infra", "observex", "spec"),
    join(SSOT_DIR, "infra", "resiliencx", "spec"),
    join(SSOT_DIR, "infra", "transport", "spec"),
  ];
  for (const dir of srcDirs) {
    const specPath = join(dir, "spec.md");
    if (!existsSync(specPath)) continue;
    const content = readFileSync(specPath, "utf8");
    const pkgMatch = content.match(/Package\s*\/\s*lib\s*(?:\/\s*version)?\s*[|:]\s*`?(\w[\w-]*)`?\s*\/\s*`?(\w[\w-]*)`?/);
    const pathMatch = content.match(/(?:Path|Physical Path)\s*[|:]\s*(crates\/[^\s|\n]+)/);
    const specIdMatch = content.match(/Spec(?:\s+ID)?\s*[：:]\s*(SPEC-[\w-]+)/);
    const relFromSsoot = relative(SSOT_DIR, dir).split("/").filter(s => s !== "spec");
    const cratePath = "crates/" + relFromSsoot.join("/");
    const domain = relFromSsoot[relFromSsoot.length - 1];
    specs.push({
      specId: specIdMatch ? specIdMatch[1] : ("SPEC-" + domain.toUpperCase() + "-???"),
      domain, package: pkgMatch ? pkgMatch[1] : domain,
      lib: pkgMatch ? pkgMatch[2] : domain,
      path: pathMatch ? pathMatch[1] : cratePath,
      specPath: relative(ROOT, specPath),
      gateDir: join(dir, "..", "gate"),
    });
  }
  return specs;
}

// 工具
function collectRsFiles(dir, out) {
  if (!existsSync(dir)) return;
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const child = join(dir, entry.name);
    if (entry.isDirectory()) collectRsFiles(child, out);
    else if (entry.isFile() && entry.name.endsWith(".rs")) out.push(child);
  }
}
function grepInDir(dir, pattern) {
  const files = [];
  collectRsFiles(dir, files);
  for (const f of files) { if (pattern.test(readFileSync(f, "utf8"))) return true; }
  return false;
}
function levelRank(l) { return { L1: 1, L2: 2, L3: 3, L4: 4 }[l] || 99; }
function ok(level, ruleId, contractId, msg, detail) {
  return { ok: true, level, ruleId, contract: contractId, message: msg, detail: detail || "" };
}
function fail(level, ruleId, contractId, msg, detail) {
  return { ok: false, level, ruleId, contract: contractId, message: msg, detail: detail || "" };
}

// L1-SIG-001: 合同声明的公开项存在于 crate 根导出
function checkSig001(contract) {
  const specPath = join(ROOT, contract.specPath);
  if (!existsSync(specPath)) return [fail("L1", "L1-SIG-001", contract.specId, "spec 文件不存在: " + contract.specPath)];
  const specContent = readFileSync(specPath, "utf8");
  const results = [];
  const declared = [];
  const patterns = [["trait", /pub\s+trait\s+(\w+)/g], ["struct", /pub\s+struct\s+(\w+)/g], ["enum", /pub\s+enum\s+(\w+)/g], ["fn", /pub\s+fn\s+(\w+)/g]];
  for (const [kind, pat] of patterns) {
    for (const m of specContent.matchAll(pat)) declared.push({ name: m[1], kind });
  }
  if (declared.length === 0) return [ok("L1", "L1-SIG-001", contract.specId, "spec 中未声明公开项，跳过验证")];
  const srcDir = join(ROOT, contract.path, "src");
  if (!existsSync(srcDir)) return [fail("L1", "L1-SIG-001", contract.specId, "源码目录不存在: " + contract.path + "/src")];
  for (const item of declared) {
    if (!grepInDir(srcDir, new RegExp("pub\\s+" + item.kind + "\\s+" + item.name + "\\b"))) {
      results.push(fail("L1", "L1-SIG-001", contract.specId, item.kind + " `" + item.name + "` 在 spec 中声明但未在 " + contract.path + "/src/ 中找到"));
    }
  }
  if (results.length === 0) results.push(ok("L1", "L1-SIG-001", contract.specId, "已验证 " + declared.length + " 个公开项", declared.map(d => d.kind + " " + d.name).join(", ")));
  return results;
}

// L1-SIG-006: #[non_exhaustive] 标记存在
function checkSig006(contract) {
  const srcDir = join(ROOT, contract.path, "src");
  if (!existsSync(srcDir)) return [fail("L1", "L1-SIG-006", contract.specId, "源码目录不存在")];
  const results = [];
  const files = [];
  collectRsFiles(srcDir, files);
  for (const file of files) {
    const content = readFileSync(file, "utf8");
    const lines = content.split("\n");
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i].trim();
      const m = line.match(/^pub\s+(enum|struct)\s+(\w+)/);
      if (!m) continue;
      const kind = m[1], name = m[2];
      let hasNonExhaustive = false;
      for (let j = Math.max(0, i - 5); j < i; j++) { if (lines[j].trim() === "#[non_exhaustive]") { hasNonExhaustive = true; break; } }
      if (kind === "enum" && !hasNonExhaustive && !name.endsWith("Error")) {
        results.push(fail("L1", "L1-SIG-006", contract.specId, "公开 enum `" + name + "` 缺少 #[non_exhaustive]", relative(ROOT, file) + ":" + (i + 1)));
      }
    }
  }
  if (results.length === 0) results.push(ok("L1", "L1-SIG-006", contract.specId, "#[non_exhaustive] 标记合规"));
  return results;
}

// L1-SIG-007: spec 版本与 Cargo.toml 一致
function checkSig007(contract) {
  const specPath = join(ROOT, contract.specPath);
  const cargoPath = join(ROOT, contract.path, "Cargo.toml");
  if (!existsSync(specPath)) return [fail("L1", "L1-SIG-007", contract.specId, "spec 文件不存在: " + contract.specPath)];
  if (!existsSync(cargoPath)) return [fail("L1", "L1-SIG-007", contract.specId, "Cargo.toml 不存在: " + contract.path + "/Cargo.toml")];
  const specContent = readFileSync(specPath, "utf8");
  const cargoContent = readFileSync(cargoPath, "utf8");
  const specVer = specContent.match(/(?:(?:Current )?[Vv]ersion)\s*[|：:]\s*`?(\d+\.\d+\.\d+)`?/);
  const cargoVer = cargoContent.match(/^version\s*=\s*"(\d+\.\d+\.\d+)"/m);
  if (!specVer) return [ok("L1", "L1-SIG-007", contract.specId, "spec 中未声明版本号，跳过验证")];
  if (!cargoVer) return [fail("L1", "L1-SIG-007", contract.specId, "Cargo.toml 中未找到 version 字段")];
  if (specVer[1] !== cargoVer[1]) return [fail("L1", "L1-SIG-007", contract.specId, "版本不一致: spec=" + specVer[1] + ", Cargo.toml=" + cargoVer[1])];
  return [ok("L1", "L1-SIG-007", contract.specId, "版本一致: " + specVer[1])];
}

// L2-BEH-002: ErrorKind 覆盖率
function checkBeh002(contract) {
  const testFiles = [];
  for (const dir of [join(ROOT, contract.path, "tests"), join(ROOT, contract.path, "src")]) {
    if (existsSync(dir)) collectRsFiles(dir, testFiles);
  }
  if (testFiles.length === 0) return [ok("L2", "L2-BEH-002", contract.specId, "无测试文件，跳过 ErrorKind 覆盖率检查")];
  const allKinds = ["Invalid", "Missing", "Conflict", "Transient", "Unavailable", "Cancelled", "DeadlineExceeded", "Invariant", "Internal"];
  const covered = new Set();
  for (const f of testFiles) {
    const content = readFileSync(f, "utf8");
    for (const k of allKinds) { if (content.includes("ErrorKind::" + k)) covered.add(k); }
  }
  const missing = allKinds.filter(k => !covered.has(k));
  if (missing.length > 0) return [fail("L2", "L2-BEH-002", contract.specId, "ErrorKind 覆盖率 " + covered.size + "/" + allKinds.length + "，缺少: [" + missing.join(", ") + "]")];
  return [ok("L2", "L2-BEH-002", contract.specId, "ErrorKind 覆盖率 " + covered.size + "/" + allKinds.length + " (100%)")];
}

// L2-BEH-005: cargo check
function checkBeh005(contract) {
  const r = spawnSync("cargo", ["check", "-p", contract.package, "--all-features"], { encoding: "utf8", cwd: ROOT, maxBuffer: 32 * 1024 * 1024 });
  if (r.status !== 0) return [fail("L2", "L2-BEH-005", contract.specId, "cargo check 失败: " + contract.package, (r.stderr || r.stdout || "").split("\n").slice(-5).join("\n"))];
  return [ok("L2", "L2-BEH-005", contract.specId, "cargo check 通过: " + contract.package)];
}

// L3-NF-004: cargo-deny
function checkNf004() {
  const denyConfig = join(ROOT, "deny.toml");
  if (!existsSync(denyConfig)) return [ok("L3", "L3-NF-004", "workspace", "无 deny.toml，跳过 cargo-deny 检查")];
  const r = spawnSync("cargo", ["deny", "check"], { encoding: "utf8", cwd: ROOT, maxBuffer: 32 * 1024 * 1024 });
  if (r.status !== 0) return [fail("L3", "L3-NF-004", "workspace", "cargo-deny check 失败", (r.stderr || r.stdout || "").split("\n").slice(0, 20).join("\n"))];
  return [ok("L3", "L3-NF-004", "workspace", "cargo-deny check 通过")];
}

// L3-NF-005: unsafe 代码扫描
function checkNf005(contract) {
  const srcDir = join(ROOT, contract.path, "src");
  if (!existsSync(srcDir)) return [ok("L3", "L3-NF-005", contract.specId, "源码目录不存在")];
  const files = [];
  collectRsFiles(srcDir, files);
  const unsafeBlocks = [];
  for (const file of files) {
    const lines = readFileSync(file, "utf8").split("\n");
    for (let i = 0; i < lines.length; i++) { if (/^\s*unsafe\s*[{\s]/.test(lines[i])) unsafeBlocks.push(relative(ROOT, file) + ":" + (i + 1)); }
  }
  if (contract.domain === "kernel" && unsafeBlocks.length > 0) return [fail("L3", "L3-NF-005", contract.specId, "L0 kernel 禁止 unsafe 代码", unsafeBlocks.join("\n"))];
  if (unsafeBlocks.length > 0) return [ok("L3", "L3-NF-005", contract.specId, "发现 " + unsafeBlocks.length + " 处 unsafe 块", unsafeBlocks.join("\n"))];
  return [ok("L3", "L3-NF-005", contract.specId, "无 unsafe 代码")];
}

// 主逻辑
function main() {
  const opts = parseArgs(args);
  if (opts.help) { printHelp(); return 0; }
  const contracts = discoverContracts();
  if (contracts.length === 0) { console.error("error: 未发现任何合同"); return 1; }
  const targetContracts = opts.contract ? contracts.filter(c => c.specId === opts.contract) : contracts;
  if (targetContracts.length === 0) { console.error("error: 未找到合同: " + opts.contract); return 1; }

  const allResults = [];
  const levels = opts.levels;
  const failRank = levelRank(opts.failLevel);

  for (const contract of targetContracts) {
    if (levels.includes("L1")) { allResults.push(...checkSig001(contract)); allResults.push(...checkSig006(contract)); allResults.push(...checkSig007(contract)); }
    if (levels.includes("L2")) { allResults.push(...checkBeh002(contract)); allResults.push(...checkBeh005(contract)); }
    if (levels.includes("L3")) { allResults.push(...checkNf005(contract)); }
  }
  if (levels.includes("L3")) allResults.push(...checkNf004());

  const failures = allResults.filter(r => !r.ok);
  const blocking = failures.filter(r => levelRank(r.level) <= failRank);

  if (opts.json) {
    console.log(JSON.stringify({
      timestamp: new Date().toISOString(),
      summary: { total: allResults.length, passed: allResults.filter(r => r.ok).length, failed: failures.length, blocking: blocking.length, levels, failLevel: opts.failLevel },
      contracts: targetContracts.map(c => c.specId), results: allResults,
    }, null, 2));
  } else if (opts.summary) {
    process.stdout.write("## 合同合规验证报告\n\n| 层级 | 通过 | 失败 |\n|------|------|------|\n");
    for (const level of levels) { const lr = allResults.filter(r => r.level === level); process.stdout.write("| " + level + " | " + lr.filter(r => r.ok).length + " | " + lr.filter(r => !r.ok).length + " |\n"); }
    if (failures.length > 0) { process.stdout.write("\n### 失败明细\n\n"); for (const f of failures) { process.stdout.write("- **" + f.ruleId + "** [" + f.contract + "]: " + f.message + "\n"); } }
  } else {
    process.stdout.write("=== 合同合规验证报告 ===\n\n");
    for (const level of levels) {
      const lr = allResults.filter(r => r.level === level);
      process.stdout.write("[" + level + "] " + lr.filter(r => r.ok).length + " pass, " + lr.filter(r => !r.ok).length + " fail\n");
      for (const r of lr) { process.stdout.write((r.ok ? "  ✓" : "  ✗") + " " + r.ruleId + " [" + r.contract + "] " + r.message + "\n"); }
      process.stdout.write("\n");
    }
    if (blocking.length > 0) process.stdout.write("=== FAIL (" + blocking.length + " 项阻塞级违规) ===\n");
    else if (failures.length > 0) process.stdout.write("=== PASS with warnings (" + failures.length + " 项高于 " + opts.failLevel + " 级别的违规) ===\n");
    else process.stdout.write("=== PASS ===\n");
  }
  return blocking.length > 0 ? 1 : 0;
}
process.exit(main());
