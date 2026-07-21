#!/usr/bin/env node
/**
 * build-foundationx-env.mjs — 从 ZoneCNH secrets/env/*.md 生成 FOUNDATIONX_* .env
 *
 * 用途：本地 live 集成测试（`cargo test -p <adapter> -- --ignored`）。
 * 不打印密钥值；输出文件权限 0600。
 *
 * 用法:
 *   node scripts/live/build-foundationx-env.mjs --env dev --out /tmp/foundationx-live.env
 *   node scripts/live/build-foundationx-env.mjs --env dev --out -   # stdout keys only? no: writes env
 *
 * 优先级:
 *   1) secrets/env/{dev,prod}.md 表解析
 *   2) 若存在 /etc/nats/nats.conf，覆盖 NATS user/password（md 常过期）
 *   3) TDengine REST 默认端口 6041（非 native 6030）
 */

import { readFileSync, writeFileSync, existsSync, chmodSync } from "fs";
import { dirname, join, resolve } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = join(__dirname, "..", "..");
const DEFAULT_SECRETS = "/home/workspace/ZoneCNH/sre/secrets/env";
const NATS_CONF_CANDIDATES = ["/etc/nats/nats.conf", "/etc/nats-server.conf"];

function parseArgs(argv) {
  const out = { env: "dev", out: "", secretsDir: DEFAULT_SECRETS, dryKeys: false };
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a === "--env") out.env = argv[++i] || "dev";
    else if (a === "--out") out.out = argv[++i] || "";
    else if (a === "--secrets-dir") out.secretsDir = argv[++i] || DEFAULT_SECRETS;
    else if (a === "--keys-only") out.dryKeys = true;
    else if (a === "-h" || a === "--help") {
      console.log(`Usage: node scripts/live/build-foundationx-env.mjs --env dev --out <path.env>
  --secrets-dir  default: ${DEFAULT_SECRETS}
  --keys-only    print key names only (no values)`);
      process.exit(0);
    }
  }
  if (!out.out && !out.dryKeys) {
    console.error("error: --out <path> required (or --keys-only)");
    process.exit(2);
  }
  return out;
}

function set(env, k, v) {
  if (v == null || String(v).length === 0) return;
  env[k] = String(v);
}

function parseDoc(filePath) {
  if (!existsSync(filePath)) throw new Error(`secrets file not found: ${filePath}`);
  const text = readFileSync(filePath, "utf-8");
  const env = {};

  // PostgreSQL admin row
  {
    const m = text.match(
      /\|\s*PostgreSQL\s*\|\s*127\.0\.0\.1\s*\|\s*5432\s*\|\s*postgres\s*\|\s*`([^`]+)`/,
    );
    if (m) {
      set(env, "FOUNDATIONX_POSTGRESX_HOST", "127.0.0.1");
      set(env, "FOUNDATIONX_POSTGRESX_PORT", "5432");
      set(env, "FOUNDATIONX_POSTGRESX_USER", "postgres");
      set(env, "FOUNDATIONX_POSTGRESX_PASSWORD", m[1]);
      set(env, "FOUNDATIONX_POSTGRESX_DATABASE", "postgres");
      set(env, "FOUNDATIONX_POSTGRESX_SSLMODE", "disable");
    }
  }

  // TDengine root — REST port 6041
  {
    const m = text.match(
      /\|\s*TDengine\s*\|\s*127\.0\.0\.1\s*\|\s*6030\s*\/\s*6041\s*\|\s*root\s*\|\s*`([^`]+)`/,
    );
    if (m) {
      set(env, "FOUNDATIONX_TAOSX_HOST", "127.0.0.1");
      set(env, "FOUNDATIONX_TAOSX_PORT", "6041");
      set(env, "FOUNDATIONX_TAOSX_USER", "root");
      set(env, "FOUNDATIONX_TAOSX_PASSWORD", m[1]);
      set(env, "FOUNDATIONX_TAOSX_TLS", "false");
    }
  }

  // Redis
  {
    const m = text.match(
      /\|\s*Redis\s*\|\s*127\.0\.0\.1\s*\|\s*6379\s*\|\s*default\s*\|\s*`([^`]+)`/,
    );
    if (m) {
      set(env, "FOUNDATIONX_REDISX_ADDR", "127.0.0.1:6379");
      set(env, "FOUNDATIONX_REDISX_USERNAME", "default");
      set(env, "FOUNDATIONX_REDISX_PASSWORD", m[1]);
      set(env, "FOUNDATIONX_REDISX_DB", "0");
      set(env, "FOUNDATIONX_REDISX_TLS", "false");
    }
  }

  // Kafka SASL
  {
    const m = text.match(
      /\|\s*Kafka\s*\|\s*127\.0\.0\.1\s*\|\s*9092\s*\|\s*admin\s*\|\s*`([^`]+)`/,
    );
    if (m) {
      set(env, "FOUNDATIONX_KAFKAX_BROKERS", "127.0.0.1:9092");
      set(env, "FOUNDATIONX_KAFKAX_SASL_MECHANISM", "PLAIN");
      set(env, "FOUNDATIONX_KAFKAX_SASL_USERNAME", "admin");
      set(env, "FOUNDATIONX_KAFKAX_SASL_PASSWORD", m[1]);
      set(env, "FOUNDATIONX_KAFKAX_TLS", "false");
    }
  }

  // ClickHouse HTTP (8123)
  {
    const m =
      text.match(/user\s*`default`\s*\/\s*password\s*`([^`]+)`/) ||
      text.match(
        /\|\s*ClickHouse\s*\|\s*127\.0\.0\.1\s*\|\s*9000\s*\/\s*8123\s*\|\s*default\s*\|\s*`([^`]+)`/,
      );
    if (m) {
      set(env, "FOUNDATIONX_CLICKHOUSEX_HOST", "127.0.0.1");
      set(env, "FOUNDATIONX_CLICKHOUSEX_HTTP_PORT", "8123");
      set(env, "FOUNDATIONX_CLICKHOUSEX_PORT", "8123");
      set(env, "FOUNDATIONX_CLICKHOUSEX_USER", "default");
      set(env, "FOUNDATIONX_CLICKHOUSEX_PASSWORD", m[1]);
      set(env, "FOUNDATIONX_CLICKHOUSEX_SSLMODE", "disable");
    }
  }

  // NATS from markdown (may be stale — overridden by conf below)
  {
    const auth = text.match(/\*\*认证\*\*:\s*用户名\s*`([^`]+)`\s*，密码\s*`([^`]+)`/);
    if (auth) {
      set(env, "FOUNDATIONX_NATS_URL", "nats://127.0.0.1:4222");
      set(env, "FOUNDATIONX_NATS_USER", auth[1]);
      set(env, "FOUNDATIONX_NATS_PASSWORD", auth[2]);
      set(env, "FOUNDATIONX_NATSX_URL", "nats://127.0.0.1:4222");
      set(env, "FOUNDATIONX_NATSX_USER", auth[1]);
      set(env, "FOUNDATIONX_NATSX_PASSWORD", auth[2]);
    }
  }

  // OSS
  {
    const ak = text.match(/\*\*AccessKey ID\*\*\s*\|\s*`([^`]+)`/);
    const sk = text.match(/\*\*AccessKey Secret\*\*\s*\|\s*`([^`]+)`/);
    const bucket = text.match(/\*\*Bucket\*\*\s*\|\s*`([^`]+)`/);
    const ep = text.match(/外网访问\s*\|\s*`([^`]+)`/);
    if (ak) set(env, "FOUNDATIONX_OSSX_ACCESS_KEY_ID", ak[1]);
    if (sk) set(env, "FOUNDATIONX_OSSX_ACCESS_KEY_SECRET", sk[1]);
    if (bucket) set(env, "FOUNDATIONX_OSSX_BUCKET", bucket[1]);
    if (ep) {
      const host = ep[1];
      set(
        env,
        "FOUNDATIONX_OSSX_ENDPOINT",
        host.startsWith("http") ? host : `https://${host}`,
      );
    }
    set(env, "FOUNDATIONX_OSSX_REGION", "ap-northeast-1");
  }

  return env;
}

/** 本机 nats.conf 覆盖（dev.md 常与实际不一致） */
function overlayNatsConf(env) {
  for (const p of NATS_CONF_CANDIDATES) {
    if (!existsSync(p)) continue;
    const conf = readFileSync(p, "utf-8");
    const user = conf.match(/^\s*user:\s*(\S+)/m);
    const password = conf.match(/^\s*password:\s*(\S+)/m);
    if (user && password) {
      set(env, "FOUNDATIONX_NATS_URL", "nats://127.0.0.1:4222");
      set(env, "FOUNDATIONX_NATS_USER", user[1]);
      set(env, "FOUNDATIONX_NATS_PASSWORD", password[1]);
      set(env, "FOUNDATIONX_NATSX_URL", "nats://127.0.0.1:4222");
      set(env, "FOUNDATIONX_NATSX_USER", user[1]);
      set(env, "FOUNDATIONX_NATSX_PASSWORD", password[1]);
      env.__nats_overlay = p;
      return;
    }
  }
}

function main() {
  const opts = parseArgs(process.argv.slice(2));
  const md = join(opts.secretsDir, `${opts.env}.md`);
  const env = parseDoc(md);
  overlayNatsConf(env);
  const overlay = env.__nats_overlay;
  delete env.__nats_overlay;

  const keys = Object.keys(env).sort();
  if (opts.dryKeys) {
    for (const k of keys) console.log(k);
    console.error(`# ${keys.length} keys from ${md}${overlay ? ` + ${overlay}` : ""}`);
    return;
  }

  const body = keys.map((k) => `${k}=${env[k]}`).join("\n") + "\n";
  const outPath = resolve(opts.out);
  writeFileSync(outPath, body, { encoding: "utf-8" });
  try {
    chmodSync(outPath, 0o600);
  } catch {
    /* ignore on platforms without chmod */
  }
  console.error(
    `wrote ${keys.length} keys -> ${outPath} (source=${md}${overlay ? ` nats=${overlay}` : ""})`,
  );
}

main();
