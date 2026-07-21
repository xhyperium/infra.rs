#!/usr/bin/env node
/**
 * secrets-migrate.mjs — 将 dev.md/prod.md 明文凭据迁移至 GitHub Secrets
 *
 * 解析 dev.md 和 prod.md 中的凭据表，映射为 FOUNDATIONX_* 环境变量命名空间，
 * 通过 gh secret set 批量上传至 GitHub Actions Secrets。
 *
 * 用法:
 *   node scripts/sre/secrets-migrate.mjs --env dev   --dry-run   预览 dev 凭据
 *   node scripts/sre/secrets-migrate.mjs --env dev   --live      上传 dev 凭据
 *   node scripts/sre/secrets-migrate.mjs --env prod  --dry-run   预览 prod 凭据
 *   node scripts/sre/secrets-migrate.mjs --env both  --dry-run   预览全部
 *   node scripts/sre/secrets-migrate.mjs --repo owner/name --env dev --live
 *
 * 命名规则:
 *   PostgreSQL  → FOUNDATIONX_POSTGRESX_{HOST|PORT|USER|PASSWORD|DATABASE|SSLMODE}
 *   TDengine     → FOUNDATIONX_TAOSX_{HOST|PORT|USER|PASSWORD|DATABASE|TLS}
 *   Redis       → FOUNDATIONX_REDISX_{ADDR|USERNAME|PASSWORD|DB|TLS}
 *   Kafka       → FOUNDATIONX_KAFKAX_{BROKERS|SASL_MECHANISM|SASL_USERNAME|SASL_PASSWORD|TLS}
 *   ClickHouse  → FOUNDATIONX_CLICKHOUSEX_{HOST|PORT|USER|PASSWORD|SSLMODE}
 *   NATS        → FOUNDATIONX_NATS_{URL|TOKEN}
 *   OSS         → FOUNDATIONX_OSSX_{ACCESS_KEY_ID|ACCESS_KEY_SECRET|BUCKET|REGION|ENDPOINT}
 *   FRED        → FOUNDATIONX_FRED_API_KEY
 *
 * SSOT: docs/sre/secrets-migration.md（待建）
 */

import { execSync } from "child_process";
import { readFileSync, existsSync } from "fs";
import { join, dirname } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = join(__dirname, "..", "..");
const SECRETS_DIR = "/home/workspace/ZoneCNH/sre/secrets/env";

// ====================== CLI Parsing ======================

function parseArgs() {
  const argv = process.argv.slice(2);
  return {
    env: argv.includes("--env") ? (argv[argv.indexOf("--env") + 1] || "dev") : "dev",
    live: argv.includes("--live"),
    dryRun: !argv.includes("--live"), // default dry-run
    repo: argv.includes("--repo") ? argv[argv.indexOf("--repo") + 1] : "",
    quiet: argv.includes("--quiet"),
  };
}

function die(msg, code = 1) { console.error(`❌ ${msg}`); process.exit(code); }
function info(msg) { console.log(`ℹ ${msg}`); }
function ok(msg) { console.log(`✅ ${msg}`); }
function warn(msg) { console.warn(`⚠ ${msg}`); }

function run(cmd, opts = {}) {
  try {
    return execSync(cmd, { encoding: "utf-8", timeout: 15000, stdio: ["pipe", "pipe", "pipe"], ...opts }).trim();
  } catch (e) {
    if (opts.required) die(`${opts.label || "command"} failed: ${cmd}\n${e.stderr || e.message}`);
    return "";
  }
}

// ====================== Credential Extraction ======================

/**
 * Parse markdown tables from the document, returning structured credential entries.
 * Each entry has: { service, db, params: { key: value } }
 */
function parseDoc(filePath) {
  if (!existsSync(filePath)) die(`File not found: ${filePath}`);
  const text = readFileSync(filePath, "utf-8");
  const entries = [];

  // PostgreSQL per-database credentials (table rows)
  const pgPattern = /\|\s*(market_\w+|macro_\w+|engineer|jin10|eastmoney|coinglass|xdotcom)\s*\|\s*127\.0\.0\.1\s*\|\s*5432\s*\|\s*(\w+)\s*\|\s*`([^`]+)`/g;
  let m;
  while ((m = pgPattern.exec(text))) {
    entries.push({
      service: "postgres",
      db: m[1],
      params: {
        PASSWORD: m[3],
        USER: m[2],
        HOST: "127.0.0.1",
        PORT: "5432",
        DATABASE: m[1],
        SSLMODE: "disable",
      },
    });
  }

  // PostgreSQL admin
  const pgAdmin = text.match(/\|\s*PostgreSQL\s*\|\s*127\.0\.0\.1\s*\|\s*5432\s*\|\s*postgres\s*\|\s*`([^`]+)`/);
  if (pgAdmin) {
    entries.push({
      service: "postgres_admin",
      db: "admin",
      params: { USER: "postgres", PASSWORD: pgAdmin[1], HOST: "127.0.0.1", PORT: "5432" },
    });
  }

  // TDengine per-database credentials
  const tdPattern = /\|\s*(market_\w+|macro_\w+|macro_coinglass)\s*\|\s*127\.0\.0\.1\s*\|\s*6030\s*\/\s*6041\s*\|\s*(\w+)\s*\|\s*`([^`]+)`/g;
  while ((m = tdPattern.exec(text))) {
    entries.push({
      service: "taos",
      db: m[1],
      params: {
        PASSWORD: m[3],
        USER: m[2],
        HOST: "127.0.0.1",
        PORT: "6030",
        DATABASE: m[1],
        TLS: "false",
      },
    });
  }

  // TDengine root
  const taosRoot = text.match(/\|\s*TDengine\s*\|\s*127\.0\.0\.1\s*\|\s*6030\s*\/\s*6041\s*\|\s*root\s*\|\s*`([^`]+)`/);
  if (taosRoot) {
    entries.push({
      service: "taos_admin",
      db: "admin",
      params: { USER: "root", PASSWORD: taosRoot[1], HOST: "127.0.0.1", PORT: "6030" },
    });
  }

  // Redis
  const redis = text.match(/\|\s*Redis\s*\|\s*127\.0\.0\.1\s*\|\s*6379\s*\|\s*default\s*\|\s*`([^`]+)`/);
  if (redis) {
    entries.push({
      service: "redis",
      db: "default",
      params: { ADDR: "127.0.0.1:6379", USERNAME: "default", PASSWORD: redis[1], DB: "0", TLS: "false" },
    });
  }

  // Kafka SASL
  const kafka = text.match(/\|\s*Kafka\s*\|\s*127\.0\.0\.1\s*\|\s*9092\s*\|\s*admin\s*\|\s*`([^`]+)`/);
  if (kafka) {
    entries.push({
      service: "kafka",
      db: "default",
      params: { BROKERS: "127.0.0.1:9092", SASL_MECHANISM: "PLAIN", SASL_USERNAME: "admin", SASL_PASSWORD: kafka[1], TLS: "false" },
    });
  }

  // ClickHouse
  const ch = text.match(/user\s*`default`\s*\/\s*password\s*`([^`]+)`/) || text.match(/\|\s*ClickHouse\s*\|\s*127\.0\.0\.1\s*\|\s*9000\s*\/\s*8123\s*\|\s*default\s*\|\s*`([^`]+)`/);
  if (ch) {
    entries.push({
      service: "clickhouse",
      db: "default",
      params: { USER: "default", PASSWORD: ch[1], HOST: "127.0.0.1", PORT: "9000", SSLMODE: "disable" },
    });
  }

  // NATS — extract password from 认证 line: 用户名 `admin`，密码 `xxx`
  const natsPwd = text.match(/\*\*认证\*\*:.*?密码\s*`([^`]+)`/);
  if (natsPwd) {
    entries.push({
      service: "nats",
      db: "default",
      params: { URL: "nats://127.0.0.1:4222", TOKEN: natsPwd[1] },
    });
  }

  return entries;
}

// ====================== Secret Naming ======================

function toSecretName(service, param, db) {
  const prefix = {
    postgres: "FOUNDATIONX_POSTGRESX",
    postgres_admin: "FOUNDATIONX_POSTGRESX_ADMIN",
    taos: "FOUNDATIONX_TAOSX",
    taos_admin: "FOUNDATIONX_TAOSX_ADMIN",
    redis: "FOUNDATIONX_REDISX",
    kafka: "FOUNDATIONX_KAFKAX",
    clickhouse: "FOUNDATIONX_CLICKHOUSEX",
    nats: "FOUNDATIONX_NATS",
  }[service] || `FOUNDATIONX_${service.toUpperCase()}X`;

  // Per-database secrets get db suffix
  if (db && db !== "admin" && db !== "default") {
    return `${prefix}_${db.toUpperCase()}_${param}`;
  }
  return `${prefix}_${param}`;
}

// ====================== Migration ======================

function migrate(entries, opts) {
  let created = 0, skipped = 0, errors = 0;
  const repoFlag = opts.repo ? ` --repo "${opts.repo}"` : "";

  for (const entry of entries) {
    for (const [param, value] of Object.entries(entry.params)) {
      const name = toSecretName(entry.service, param, entry.db);
      const label = `${entry.service}${entry.db ? "/" + entry.db : ""}`;

      if (!value || value.trim() === "") {
        warn(`[${label}] ${name}: empty value, skipped`);
        skipped++;
        continue;
      }

      if (opts.dryRun) {
        info(`[${label}] ${name} = ${mask(value)}  [DRY-RUN]`);
        skipped++;
        continue;
      }

      try {
        run(`gh secret set "${name}" --body "${value.replace(/"/g, '\\"')}"${repoFlag}`, { required: true, label: `gh secret set ${name}` });
        ok(`[${label}] ${name} set`);
        created++;
      } catch {
        errors++;
      }
    }
  }

  return { created, skipped, errors };
}

function mask(value) {
  if (!value) return "(empty)";
  if (value.length <= 6) return "*".repeat(value.length);
  return value.slice(0, 2) + "*".repeat(value.length - 4) + value.slice(-2);
}

// ====================== Summary ======================

function printSummary(env, entries, results) {
  console.log(`\n=== ${env.toUpperCase()} Secrets Migration ===`);
  console.log(`Entries: ${entries.length} | Created: ${results.created} | Skipped: ${results.skipped} | Errors: ${results.errors}`);
  console.log("");
  console.log("CI Workflow Usage:");
  console.log("  env:");
  for (const entry of entries.slice(0, 5)) {
    for (const param of Object.keys(entry.params).slice(0, 2)) {
      const name = toSecretName(entry.service, param, entry.db);
      console.log(`    ${name}: \${{ secrets.${name} }}`);
    }
  }
  console.log("    # ... (${entries.length} total entries)");
}

// ====================== Main ======================

function main() {
  const opts = parseArgs();

  if (!opts.live && !opts.dryRun) {
    info("Default: --dry-run. Use --live to upload secrets.");
  }

  let entries = [];
  const envs = opts.env === "both" ? ["dev", "prod"] : [opts.env];

  for (const env of envs) {
    const filePath = join(SECRETS_DIR, `${env}.md`);
    info(`Parsing ${filePath}...`);
    const envEntries = parseDoc(filePath);
    // Tag with environment
    envEntries.forEach((e) => { e.env = env; });
    entries = entries.concat(envEntries);
  }

  info(`Found ${entries.length} credential entries`);
  const results = migrate(entries, opts);
  printSummary(opts.env, entries, results);

  if (results.errors > 0) die(`${results.errors} errors during migration`, 2);
}

main();

export { parseDoc, toSecretName, mask, SECRETS_DIR };
