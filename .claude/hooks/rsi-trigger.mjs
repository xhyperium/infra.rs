// RSI auto-propose hook — L4 compound interest mechanism
// Trigger: PostToolUse after audit runs or governance changes
// Reads audit output, auto-proposes RSI when FAIL detected

const { execSync } = await import('node:child_process');

const AUDIT_SCRIPT = 'python3 scripts/audit-status.py';
const RSI_TRIGGER = 'python3 docs/goal/tools/rsi-trigger.py';

try {
  const result = execSync(AUDIT_SCRIPT, { encoding: 'utf8', timeout: 30000 });
  const failCount = (result.match(/FAIL/g) || []).length;

  if (failCount > 0) {
    console.log(`[RSI Hook] Audit detected ${failCount} FAIL(s). Auto-proposing RSI...`);
    try {
      const proposal = execSync(`${RSI_TRIGGER} --propose --json`, {
        encoding: 'utf8',
        timeout: 15000,
        env: { ...process.env, RSI_AUTO: '1' }
      });
      console.log(`[RSI Hook] Proposal generated: ${proposal.trim()}`);
    } catch (e) {
      console.error(`[RSI Hook] RSI trigger failed: ${e.message}`);
    }
  } else {
    // silent pass — audit clean
  }
} catch (e) {
  // audit script not available or failed — skip silently
}

// GC health check (lightweight, non-blocking)
try {
  const gc = execSync('node scripts/gc-scan.mjs --json', {
    encoding: 'utf8',
    timeout: 10000
  });
  const gcData = JSON.parse(gc);
  if (gcData.summary.critical > 0) {
    console.warn(`[RSI Hook] GC Agent: ${gcData.summary.critical} critical finding(s)`);
  }
} catch (e) {
  // gc-scan not available — skip
}
