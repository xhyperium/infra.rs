#!/usr/bin/env node
/**
 * check-settings-hooks.mjs — 验证 .claude/settings.json 中的 hook 命令格式
 *
 * 检查所有 node 命令是否已包装 nice/timeout 限制。
 *
 * 阈值设计原则（平衡误报与漏报）：
 *   - nice 值: 10-19（低于 10 仍可能抢占正常进程）
 *   - timeout: 5-60s（低于 5s 可能误杀，高于 60s 等于无限制）
 *   - 空格/顺序/路径: 严格（格式问题，没有弹性空间）
 *
 * 用法: node scripts/quality-gates/check-settings-hooks.mjs
 * exit 0 = 全部通过, exit 1 = 存在违规
 */

import { readFileSync, existsSync } from "fs";
import { resolve, dirname } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(__dirname, "..", "..");
const SETTINGS_PATH = resolve(ROOT, ".claude", "settings.json");

// ═══════════════════════════════════════
// 可调阈值 — 唯一需要修改的集中点
// ═══════════════════════════════════════
const NICE_MIN  = 10;   // nice 最小值（含），≥10 已有显著降级效果
const NICE_MAX  = 19;   // nice 最大值（含），19 为系统上限
const TMO_MIN   = 5;    // timeout 最小值（秒），低于此值可能误杀合法脚本
const TMO_MAX   = 60;   // timeout 最大��（秒），高于此值等于无效限制

// ═══════════════════════════════════════
// 固定规范 — 不允许弹性
// ═══════════════════════════════════════
const HOOKS_PATH = "\\.claude/hooks/";
const HOOK_FILE  = "[a-z][a-z0-9_.-]*\\.(mjs|cjs)";

// 动态构建的正则（基于阈值）
function buildRegex(nice, tmo) {
  return new RegExp(
    `^nice -n ${nice} timeout ${tmo} node ${HOOKS_PATH}${HOOK_FILE}$`
  );
}

let pass = 0, fail = 0;

function ok(cond, name) {
  if (cond) { pass++; console.log(`  ok  ${name}`); }
  else      { fail++; console.log(`  FAIL ${name}`); }
}

console.log("\ncheck-settings-hooks tests\n");
console.log(`  阈值: nice ${NICE_MIN}-${NICE_MAX}  timeout ${TMO_MIN}-${TMO_MAX}s\n`);

// §1 文件存在
ok(existsSync(SETTINGS_PATH), "settings.json 存在");

let settings;
try {
  const raw = readFileSync(SETTINGS_PATH, "utf8");
  settings = JSON.parse(raw);
  ok(true, "settings.json 是有效 JSON");
} catch (e) {
  ok(false, `settings.json 解析失败: ${e.message}`);
  process.exit(1);
}

// §2 提取所有 command 字段
const commands = [];

function walk(obj, path) {
  if (Array.isArray(obj)) {
    for (let i = 0; i < obj.length; i++) walk(obj[i], `${path}[${i}]`);
  } else if (obj && typeof obj === "object") {
    if (obj.command && typeof obj.command === "string") commands.push({ path, command: obj.command });
    for (const key of Object.keys(obj)) walk(obj[key], `${path}.${key}`);
  }
}

walk(settings, "settings");
ok(commands.length > 0, `找到了 ${commands.length} 个 command 字段`);

// ═══ 命令解析工具 ═══

/**
 * 从 "nice -n 19 timeout 30 node .claude/hooks/x.mjs" 提取各部分
 * 返回 null 表示不是标准的 nice+timeout+node 命令
 */
function parseNiceTimeoutNode(cmd) {
  // 匹配: nice -n <N> timeout <T> node <path>
  const m = /^nice -n (\d+) timeout (\d+) node (.+)$/.exec(cmd);
  if (!m) return null;
  return {
    niceVal: parseInt(m[1], 10),
    tmoVal:  parseInt(m[2], 10),
    script:  m[3],
  };
}

// §3 逐条校验
for (const { path, command } of commands) {
  const cmd = command.trim();
  const label = path.split(".").pop();

  // §3a: 裸 node 命令
  if (cmd.startsWith("node ")) {
    ok(false, `裸 node 命令: ${path} → \`${cmd}\``);
    continue;
  }
  if (cmd.startsWith("node")) {
    ok(false, `node 命令前缀不标准: ${path} → \`${cmd}\``);
    continue;
  }

  // §3b: 首尾空格（命令原串 ≠ trim 后）
  if (command !== command.trim()) {
    ok(false, `首尾空格: ${path} → \`${cmd}\``);
    continue;
  }

  // §3c: 连续空格
  if (cmd.includes("  ")) {
    ok(false, `连续空格: ${path} → \`${cmd}\``);
    continue;
  }

  // §3d: 解析各部分
  const parsed = parseNiceTimeoutNode(cmd);
  if (!parsed) {
    // 不是 nice+timeout+node 格式
    if (cmd.includes("node ") && (cmd.includes("nice") || cmd.includes("timeout"))) {
      ok(false, `nice/timeout 格式异常: ${path} → \`${cmd}\``);
      continue;
    }
    // 非 node 命令（如 bd prime）
    ok(true, `非 node 命令: ${label} → \`${cmd.substring(0, 45)}...\``);
    continue;
  }

  // §3e: 顺序校验 (nice → timeout → node)
  // parseNiceTimeoutNode 已保证顺序（正则锚定），此处二次确认
  const ni = cmd.indexOf("nice");
  const ti = cmd.indexOf("timeout");
  const noi = cmd.indexOf("node ");
  if (ni < 0 || ti < 0 || noi < 0 || ni > ti || ti > noi) {
    ok(false, `顺序错误 (应为 nice→timeout→node): ${path} → \`${cmd}\``);
    continue;
  }

  // §3f: nice 值在阈值范围内
  const { niceVal, tmoVal, script } = parsed;
  if (niceVal < NICE_MIN || niceVal > NICE_MAX) {
    ok(false, `nice 值 ${niceVal} 不在 [${NICE_MIN},${NICE_MAX}]: ${path} → \`${cmd}\``);
    continue;
  }

  // §3g: timeout 值在阈值范围内
  if (tmoVal < TMO_MIN || tmoVal > TMO_MAX) {
    ok(false, `timeout 值 ${tmoVal}s 不在 [${TMO_MIN},${TMO_MAX}]: ${path} → \`${cmd}\``);
    continue;
  }

  // §3h: 脚本路径在 .claude/hooks/ 下
  if (!script.startsWith(".claude/hooks/")) {
    ok(false, `脚本路径不在 hooks 目录: ${path} → \`${script}\``);
    continue;
  }

  // §3i: 脚本扩展名为 .mjs 或 .cjs
  if (!/\.(mjs|cjs)$/.test(script)) {
    ok(false, `脚本扩展名非 .mjs/.cjs: ${path} → \`${script}\``);
    continue;
  }

  // 全部通过
  ok(true, `通过 (nice=${niceVal}, tmo=${tmoVal}s): ${label} → \`${script}\``);
}

// §4 裸 node 命令计数
const bareNodeCommands = commands.filter(({ command }) => {
  const c = command.trim();
  return c.startsWith("node ") || c.startsWith("node.");
});
ok(bareNodeCommands.length === 0, `裸 node 命令: ${bareNodeCommands.length} (应为 0)`);

// §5 合规命令统计
const compliant = commands.filter(({ command }) => {
  const p = parseNiceTimeoutNode(command.trim());
  if (!p) return false;
  return p.niceVal >= NICE_MIN && p.niceVal <= NICE_MAX
      && p.tmoVal  >= TMO_MIN  && p.tmoVal  <= TMO_MAX
      && p.script.startsWith(".claude/hooks/")
      && /\.(mjs|cjs)$/.test(p.script);
});
ok(compliant.length > 0, `合规命令: ${compliant.length} 条`);

// §6 非预期 node 引用
const unexpectedNode = commands.filter(({ command }) => {
  const c = command.trim();
  return !c.startsWith("nice -n ") && c.includes("node ") && !c.startsWith("node ");
});
ok(unexpectedNode.length === 0, `非预期 node 引用: ${unexpectedNode.length}`);

// 汇总
console.log(`\n${pass} passed, ${fail} failed, ${pass + fail} total\n`);

if (fail > 0) {
  console.log(`期望格式: nice -n <${NICE_MIN}-${NICE_MAX}> timeout <${TMO_MIN}-${TMO_MAX}> node .claude/hooks/<name>.mjs`);
  process.exit(1);
}
console.log("All settings commands pass √\n");
process.exit(0);
