#!/usr/bin/env node
/**
 * check-canonical-align.mjs — canonical 域本地对齐检查
 *
 * 职责: 验证 canonical 与 decimalx 的本地 crate 实现一致性。
 * 不依赖上游 `.agents/ssot/` 镜像内容 — 所有检查以本仓源码为准。
 *
 * 用法:
 *   node scripts/quality-gates/check-canonical-align.mjs
 */
import { execSync } from "child_process";
import { existsSync } from "fs";
import { join } from "path";

const root = execSync("git rev-parse --show-toplevel", { encoding: "utf8" }).trim();
const branch = execSync("git rev-parse --abbrev-ref HEAD", { encoding: "utf8" }).trim();
const crate = "crates/types/canonical";

process.chdir(root);

// helpers
function fail(msg) { console.error("FAIL: " + msg); process.exit(1); }
function ok(msg) { console.log("OK: " + msg); }
function run(cmd) { execSync(cmd, { encoding: "utf8", stdio: "pipe" }); }
function quiet(cmd) { try { return execSync(cmd, { encoding: "utf8", stdio: "pipe" }).trim(); } catch { return ""; } }
function has(cmd) { try { execSync("command -v " + cmd, { encoding: "utf8", stdio: "pipe" }); return true; } catch { return false; } }

console.log("=== check-canonical-align branch=" + branch + " ===");

// 验证 workspace packages 存在
let names;
try {
    const meta = JSON.parse(execSync("cargo metadata --no-deps --format-version 1", { encoding: "utf8" }));
    names = meta.packages.map(p => p.name);
} catch { fail("cargo metadata failed"); }
if (!names.includes("canonical") || !names.includes("decimalx"))
    fail("missing packages: " + JSON.stringify(names));
console.log("Packages: " + JSON.stringify(names));

// 文件存在检查
if (!existsSync(join(crate, "src/lib.rs"))) fail("canonical crate 不存在");
if (!existsSync("crates/types/decimal/src/lib.rs")) fail("decimalx crate 不存在");
if (!existsSync("fixtures/market/order_cancel_okx.json")) fail("缺少测试 fixture");
ok("crate 文件结构");

// 源码模式检查
const rg = has("rg") ? "rg" : "grep -rn";

// 禁止在 crate 源码中使用已弃用的 OrderId 类型别名
if (quiet(`${rg} 'type OrderId' ${crate}/src`))
    fail("crate 源码中仍存在 type OrderId");

// 禁止 f32/f64 — 金融精度要求使用十进制
if (quiet(`${rg} '\\\\bf32\\\\b|\\\\bf64\\\\b' ${crate}/src --glob '*.rs'`))
    fail("crate 源码中发现 f32/f64");

// 禁止 OPEN-time 占位文案
if (quiet(`${rg} 'until_can_time_approved|ts unit remains OPEN' ${crate}/src`))
    fail("crate 源码中仍有 OPEN-time 占位文案");

ok("源码模式检查");

// cargo test
try { run("cargo test -p canonical -p decimalx 2>&1"); ok("cargo test"); }
catch { fail("测试失败"); }

// cargo clippy
try { run("cargo clippy -p canonical -p decimalx --all-targets -- -D warnings 2>&1"); ok("cargo clippy"); }
catch { fail("clippy 失败"); }

// cargo fmt
try { run("cargo fmt -p canonical -p decimalx -- --check"); ok("cargo fmt"); }
catch { fail("fmt 失败"); }

console.log("=== ALL CHECKS PASSED ===");
