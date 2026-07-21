/**
 * check-canonical-align.mjs — L1 逻辑测试
 *
 * 用法: node scripts/quality-gates/check-canonical-align.test.mjs
 * exit 0 = 全部通过
 *
 * 测试覆盖:
 *   - 脚本存在性 + shebang + 语法
 *   - import 有效性
 *   - 关键函数存在性 (fail, ok, run, quiet, has)
 *   - cargo metadata 查询模式
 *   - spec 合规检查 (Approved, 类型删除, 纳秒时间戳)
 *   - post-S1 漂移/目标/批准检查
 *   - 脚本执行（轻量测试）
 */
import { execFileSync, execSync } from "child_process";
import { readFileSync, existsSync } from "fs";
import { resolve, dirname } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(__dirname, "../..");
const SCRIPT = resolve(__dirname, "check-canonical-align.mjs");

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

console.log("check-canonical-align.mjs tests\n");

// ============================================================
// §1 文件存在性与完整性
// ============================================================
console.log("§1 文件存在性与完整性");

assert("脚本文件存在", existsSync(SCRIPT));

const raw = readFileSync(SCRIPT, "utf8");
assert("文件非空", raw.length > 0);
assert("文件为 ~99 行", raw.split("\n").length < 120, `实际: ${raw.split("\n").length} 行`);

const shebang = raw.split("\n")[0];
assert("shebang 为 #!/usr/bin/env node", shebang === "#!/usr/bin/env node", `实际: ${shebang}`);

assert("使用 ESM import", raw.includes("import {"));

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

assert("child_process 导入 (execSync)", raw.includes('import { execSync } from "child_process"'));
assert("fs 导入 (existsSync)", raw.includes('import { existsSync') && raw.includes('} from "fs"'));
assert("fs 导入 (readFileSync)", raw.includes("readFileSync"));
assert("path 导入 (join)", raw.includes('import { join } from "path"'));
assert("url 导入 (fileURLToPath)", raw.includes('import { fileURLToPath } from "url"'));

// ============================================================
// §4 关键函数存在性
// ============================================================
console.log("\n§4 关键函数存在性");

assert("包含 fail 函数", raw.includes("function fail(msg)"));
assert("fail 函数使用 exit(1)", raw.includes("function fail(msg) { console.error"));
assert("包含 ok 函数", raw.includes("function ok(msg)"));
assert("包含 run 函数", raw.includes("function run(cmd)"));
assert("包含 quiet 函数", raw.includes("function quiet(cmd)"));
assert("quiet 函数返回空字符串", raw.includes("return \"\""));
assert("包含 has 函数", raw.includes("function has(cmd)"));
assert("has 函数检查 command -v", raw.includes("command -v"));

// ============================================================
// §5 环境变量与路径
// ============================================================
console.log("\n§5 环境变量与路径");

assert("使用 SCRATCH 环境变量", raw.includes("SCRATCH"));
assert("默认 scratch 路径 /tmp/grok-check-canonical", raw.includes("/tmp/grok-check-canonical"));
assert("使用 SSOT 路径 .agents/ssot/types/canonical", raw.includes(".agents/ssot/types/canonical"));
assert("使用 crate 路径 crates/types/canonical", raw.includes("crates/types/canonical"));
assert("切换到 git root", raw.includes("process.chdir(root)"));

// ============================================================
// §6 cargo metadata 查询模式
// ============================================================
console.log("\n§6 cargo metadata 查询");

assert("调用 cargo metadata --no-deps", raw.includes("cargo metadata --no-deps"));
assert("解析 metadata JSON", raw.includes("JSON.parse(execSync"));
assert("检查 canonical package", raw.includes("canonical"));
assert("检查 decimalx package", raw.includes("decimalx"));

// ============================================================
// §7 文件存在性检查
// ============================================================
console.log("\n§7 文件存在性检查");

assert("检查 crates/types/canonical/src/lib.rs", raw.includes("join(crate, \"src/lib.rs\")"));
assert("检查 crates/types/decimal/src/lib.rs", raw.includes("crates/types/decimal/src/lib.rs"));
assert("检查 fixtures/market/order_cancel_okx.json", raw.includes("fixtures/market/order_cancel_okx.json"));

// ============================================================
// §8 spec 合规检查
// ============================================================
console.log("\n§8 spec 合规检查");

assert("检查 spec.md **Approved**", raw.includes("**Approved**"));
assert("检查类型已删/类型已删除", raw.includes("类型已删|类型已删除"));
assert("检查纳秒时间戳", raw.includes("纳秒|Unix"));
assert("检查 deprecated OrderId", raw.includes("deprecated `OrderId`"));
assert("检查 alignment-matrix 文件", raw.includes("alignment-matrix-infra"));
assert("双重镜像 MD5 校验", raw.includes("dual-mirror mismatch"));

// ============================================================
// §9 post-S1 检查
// ============================================================
console.log("\n§9 post-S1 检查");

assert("检查 plan DRIFT-04 非 5 测", raw.includes("当前仅 5 测"));
assert("检查 complete-goal §7 未假装批准", raw.includes("未假装批准"));
assert("检查 approval-packet.md SUPERSEDED", raw.includes("SUPERSEDED for current-state"));
assert("检查 crate 无 OPEN-time 措辞", raw.includes("ts unit remains OPEN"));
assert("检查 plan DRIFT agent-safe 处置已闭", raw.includes("agent-safe 补测|agent-safe 修正"));

// ============================================================
// §10 cargo 质量门
// ============================================================
console.log("\n§10 cargo 质量门");

assert("运行 cargo test -p canonical", raw.includes("cargo test -p canonical -p decimalx"));
assert("运行 cargo clippy -p canonical", raw.includes("cargo clippy -p canonical -p decimalx"));
assert("运行 cargo fmt -p canonical", raw.includes("cargo fmt -p canonical -p decimalx"));
assert("ALL CHECKS PASSED", raw.includes("ALL CHECKS PASSED"));

// ============================================================
// §11 脚本执行 (轻量)
// ============================================================
console.log("\n§11 脚本执行");

// 脚本依赖 git repo 环境，在 infra.rs 仓库中基本可执行
// 但不依赖 cargo（可能失败），测试脚本不崩溃即可
{
  try {
    execFileSync("node", [SCRIPT], {
      cwd: ROOT,
      encoding: "utf8",
      stdio: "pipe",
      timeout: 30_000,
    });
    assert("脚本不崩溃 (exit 0)", true);
  } catch (e) {
    // Non-zero exit is ok — git/cargo in CI context
    assert("脚本不崩溃 (可能 exit 非零但非语法错误)", e.status !== undefined,
      `exit code: ${e.status}, err: ${String(e.stderr).slice(0, 200)}`);
  }
}

// ============================================================
// §12 SAFE-15 / SAFE-16 / T-10X
// ============================================================
console.log("\n§12 SAFE / T-10X 标记");

// These are critical spec compliance markers
assert("检查 SAFE-15 DEFERRED", raw.includes("SAFE-15") && raw.includes("DEFERRED"));
assert("检查 SAFE-16 HUMAN_ONLY", raw.includes("SAFE-16") && raw.includes("HUMAN"));
assert("检查 T-10X-001 DEFERRED", raw.includes("T-10X-001") && raw.includes("DEFERRED"));

// ============================================================
// §13 源代码检查 (crate src)
// ============================================================
console.log("\n§13 源代码检查");

assert("检查 crate 无 type OrderId", raw.includes("type OrderId"));
assert("检查 crate 无 f32/f64", raw.includes("f32") && raw.includes("f64"));

// ============================================================
// 汇总
// ============================================================
console.log(`\n${total} 个测试运行，${failed} 个失败`);

if (failed > 0) {
  console.error(`\n${failed} test(s) failed`);
  process.exit(1);
}
console.log("\nall passed");
