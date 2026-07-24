// RSI auto-propose hook — L4 compound interest mechanism
// Trigger: PostToolUse
// GC health check (lightweight, non-blocking)

import { execSync } from "node:child_process";
import { existsSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const projectRoot = join(__dirname, "../..");

// GC health check (lightweight, non-blocking)
const gcScanScript = join(projectRoot, "scripts/harness/gc-scan.mjs");
if (existsSync(gcScanScript)) {
  try {
    const gc = execSync(`node ${gcScanScript} --json`, {
      encoding: "utf8",
      timeout: 10000,
      cwd: projectRoot,
    });
    const gcData = JSON.parse(gc);
    if (gcData.summary && gcData.summary.critical > 0) {
      console.warn(
        `[RSI Hook] GC Agent: ${gcData.summary.critical} critical finding(s)`,
      );
    }
  } catch (e) {
    // gc-scan failed — skip
  }
}
