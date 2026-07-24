#!/usr/bin/env node
// Version Guard — Stop Hook
//
// 检查本次会话是否修改了 crates/tools 源码但未递增版本号。
// 输出警告到 stderr，不阻塞。
//
// 规则来源：.agents/rules/VERSIONING.md R-C2 — 每更一次 PATCH +1
// 版本源：crates/**/Cargo.toml 与 tools/**/Cargo.toml 显式独立版本
//
// 不再使用 release/manifest/latest.json（旧模型，已移除）。

import { execSync } from "child_process";
import { readFileSync, existsSync } from "fs";
import { join, dirname } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const projectRoot = join(__dirname, "../..");

const CRATE_CARGO_TOML = /^(crates|tools)\/.+\/Cargo\.toml$/;

function getChangedFiles() {
  try {
    const output = execSync("git diff --name-only HEAD", {
      encoding: "utf8",
      cwd: projectRoot,
    }).trim();
    return output.split("\n").filter(Boolean);
  } catch {
    return [];
  }
}

function getVersionFromToml(tomlPath) {
  const abs = join(projectRoot, tomlPath);
  if (!existsSync(abs)) return null;
  try {
    const text = readFileSync(abs, "utf8");
    const m = text.match(/^version\s*=\s*"([^"]+)"/m);
    return m ? m[1] : null;
  } catch {
    return null;
  }
}

function getVersionBeforeChange(tomlPath) {
  try {
    const output = execSync(`git show HEAD:"${tomlPath}"`, {
      encoding: "utf8",
      cwd: projectRoot,
    }).trim();
    const m = output.match(/^version\s*=\s*"([^"]+)"/m);
    return m ? m[1] : null;
  } catch {
    // 新文件，无 HEAD 版本
    return null;
  }
}

function main() {
  const changedFiles = getChangedFiles();

  if (changedFiles.length === 0) {
    return;
  }

  // 只关注 crates/ 和 tools/ 下的 Cargo.toml
  const cargoTomlsChanged = changedFiles.filter((f) => CRATE_CARGO_TOML.test(f));

  if (cargoTomlsChanged.length === 0) {
    // 没有 crate/tools Cargo.toml 变更，无需版本检查
    return;
  }

  // 检查源码变更 — 任一 crates/ 或 tools/ 下非 Cargo.toml 文件变更
  const sourceChanged = changedFiles.some(
    (f) => /^(crates|tools)\//.test(f) && !f.endsWith("Cargo.toml"),
  );

  const needsBump = [];

  for (const toml of cargoTomlsChanged) {
    const current = getVersionFromToml(toml);
    const previous = getVersionBeforeChange(toml);

    if (previous && current === previous) {
      needsBump.push({ toml, version: current });
    }
  }

  // 如果只有 Cargo.toml 改了且版本已增，跳过
  if (needsBump.length === 0 && !sourceChanged) {
    return;
  }

  // 如果 Cargo.toml 改了且版本已增，但还有源码变更 — 检查源码所在 crate
  if (needsBump.length > 0 || sourceChanged) {
    const lines = [];
    lines.push("");
    lines.push("\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550");
    lines.push("[VersionGuard] 检查 crates/tools 版本号...");
    lines.push("");

    if (needsBump.length > 0) {
      lines.push("  以下 crate 的 Cargo.toml 已修改但版本号未变：");
      for (const { toml, version } of needsBump) {
        lines.push(`    - ${toml}  (current: ${version})`);
      }
    }

    if (sourceChanged && needsBump.length === 0) {
      lines.push("  源码已变更但所有受影响的 Cargo.toml 不在变更列表中。");
      lines.push("  请确认是否需要 bump 对应 crate 版本：");
    }

    lines.push("");
    lines.push("  规则: .agents/rules/VERSIONING.md R-C2 — 每更一次 PATCH +1");
    lines.push("");
    lines.push("  自动 bump:");
    lines.push("    $ node scripts/version/crate-bump.mjs <package-name>");
    lines.push("    $ node scripts/version/crate-bump.mjs crates/infra/configx");
    lines.push("    $ node scripts/version/crate-bump.mjs <package-name> --dry-run  # 预览");
    lines.push("");

    if (needsBump.length > 0) {
      lines.push("  \u26a0\ufe0f  警告: Cargo.toml 已修改但版本号未递增！");
    }

    if (sourceChanged && needsBump.length === 0) {
      lines.push("  \u2139\ufe0f  提示: 仅有源码变更，版本更新需手动确认");
    }

    lines.push("\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550");

    console.error(lines.join("\n"));
  }
}

main();
