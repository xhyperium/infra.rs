#!/usr/bin/env node
/**
 * PostToolUse Hook: Markdown link checker
 *
 * Runs after Edit/Write on .md files. Scans STATUS.md, README.md, ARCHITECTURE.md
 * for broken GitHub repo links. Uses concurrent gh API calls via xargs -P.
 * Exits 0 always (warn only, never blocks).
 *
 * Pure shell: uses grep + gh api + xargs. Zero npm dependencies beyond gh CLI.
 */

'use strict';

const { execSync } = require('child_process');
const path = require('path');

const FILE_PATH = process.env.FILE_PATH || '';
const CORE_FILES = ['STATUS.md', 'README.md', 'ARCHITECTURE.md'];
const basename = path.basename(FILE_PATH);

if (!CORE_FILES.includes(basename)) {
  process.exit(0);
}

console.log(`[LinkCheck] Scanning ${basename} for broken links...`);

try {
  // Extract unique repos, then check in parallel with xargs
  const cmd = `bash -c '
    repos=$(grep -oPh "github\\\\.com/ZoneCNH/[a-zA-Z0-9_.-]+" STATUS.md README.md ARCHITECTURE.md | sed "s|github.com/ZoneCNH/||" | sort -u)
    echo "$repos" | xargs -P 10 -I {} sh -c "gh api repos/ZoneCNH/{} -q .name 2>/dev/null || echo NOT_FOUND:{}"
  '`;

  // 内部超时 < settings 外层 timeout(30s)，确保本脚本 catch 能先跑完并 exit 0（warn-only）
  const output = execSync(cmd, { encoding: 'utf8', timeout: 25000, maxBuffer: 1024 * 1024 }).trim();

  const broken = output.split('\n').filter(l => l.startsWith('NOT_FOUND:'));
  if (broken.length > 0) {
    console.warn(`[LinkCheck] BROKEN LINKS FOUND:\n${broken.join('\n')}`);
  } else {
    console.log('[LinkCheck] All repo links resolve correctly.');
  }
} catch (e) {
  console.warn(`[LinkCheck] Scan failed: ${e.message}`);
}

process.exit(0);
