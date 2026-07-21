#!/usr/bin/env node
/**
 * Local/CI helper: run kernel loom concurrency models.
 * Usage: node scripts/quality-gates/run-kernel-loom.mjs
 */
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import path from "node:path";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const r = spawnSync(
  "cargo",
  ["test", "-p", "kernel", "--test", "lifecycle_concurrency_loom", "--release"],
  {
    cwd: root,
    env: { ...process.env, RUSTFLAGS: "--cfg loom" },
    stdio: "inherit",
  },
);
process.exit(r.status ?? 1);
