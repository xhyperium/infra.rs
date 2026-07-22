#!/usr/bin/env node
/**
 * build-foundationx-env.mjs — 从 ZoneCNH secrets/env/*.md 生成 FOUNDATIONX_* .env
 *
 * 用途：本地 live 集成测试（`cargo test -p <adapter> -- --ignored`）。
 * 不打印密钥值；输出文件权限 0600。
 *
 * 用法:
 *   node scripts/live/build-foundationx-env.mjs --env dev --out <私有目录>/foundationx.env
 *   node scripts/live/build-foundationx-env.mjs --env dev --keys-only
 *
 * 优先级:
 *   1) secrets/env/dev.md 表解析（其他环境 fail-closed）
 *   2) 显式传入 --nats-conf 时，覆盖 NATS user/password
 *   3) TDengine REST 默认端口 6041（非 native 6030）
 */

import {
  closeSync,
  constants,
  existsSync,
  fchmodSync,
  fsyncSync,
  openSync,
  readFileSync,
  unlinkSync,
  writeFileSync,
} from "fs";
import { dirname, join, resolve } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const DEFAULT_SECRETS = "/home/workspace/ZoneCNH/sre/secrets/env";

function requireValue(argv, index, option) {
  const value = argv[index + 1];
  if (value == null || value.length === 0 || value.startsWith("--")) {
    throw new Error(`${option} 缺少参数`);
  }
  return value;
}

function parseArgs(argv) {
  const out = {
    env: "dev",
    out: "",
    secretsDir: DEFAULT_SECRETS,
    natsConf: "",
    dryKeys: false,
    help: false,
  };
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a === "--env") out.env = requireValue(argv, i++, a);
    else if (a === "--out") out.out = requireValue(argv, i++, a);
    else if (a === "--secrets-dir") out.secretsDir = requireValue(argv, i++, a);
    else if (a === "--nats-conf") out.natsConf = requireValue(argv, i++, a);
    else if (a === "--keys-only") out.dryKeys = true;
    else if (a === "-h" || a === "--help") out.help = true;
    else throw new Error(`未知参数: ${a}`);
  }
  if (out.env !== "dev") {
    throw new Error("仅允许读取 dev 凭据；其他环境已拒绝");
  }
  if (out.help) return out;
  if (!out.out && !out.dryKeys) {
    throw new Error("必须提供 --out <path>（或使用 --keys-only）");
  }
  if (out.out === "-") {
    throw new Error("禁止将凭据写入 stdout；请提供安全的文件路径");
  }
  return out;
}

function printHelp() {
  console.log(`用法: node scripts/live/build-foundationx-env.mjs --env dev --out <path.env>
  --secrets-dir  默认: ${DEFAULT_SECRETS}
  --nats-conf    可选的 dev NATS 配置文件；默认不读取宿主配置
  --keys-only    仅打印键名，不打印值`);
}

function set(env, k, v) {
  if (v == null || String(v).length === 0) return;
  const value = String(v);
  if (/[\r\n\0]/u.test(value)) {
    throw new Error(`变量 ${k} 包含禁止的控制字符`);
  }
  env[k] = value;
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

/** 显式 dev nats.conf 覆盖；默认不读取宿主配置。 */
function overlayNatsConf(env, confPath) {
  if (!confPath) return false;
  const conf = readFileSync(confPath, "utf-8");
  const user = conf.match(/^\s*user:\s*(\S+)/m);
  const password = conf.match(/^\s*password:\s*(\S+)/m);
  if (!user || !password) {
    throw new Error("指定的 NATS 配置缺少 user/password");
  }
  set(env, "FOUNDATIONX_NATS_URL", "nats://127.0.0.1:4222");
  set(env, "FOUNDATIONX_NATS_USER", user[1]);
  set(env, "FOUNDATIONX_NATS_PASSWORD", password[1]);
  set(env, "FOUNDATIONX_NATSX_URL", "nats://127.0.0.1:4222");
  set(env, "FOUNDATIONX_NATSX_USER", user[1]);
  set(env, "FOUNDATIONX_NATSX_PASSWORD", password[1]);
  return true;
}

function writeExclusive(outPath, body) {
  const flags =
    constants.O_WRONLY |
    constants.O_CREAT |
    constants.O_EXCL |
    constants.O_NOFOLLOW;
  let fd;
  let failure;
  try {
    fd = openSync(outPath, flags, 0o600);
    fchmodSync(fd, 0o600);
    writeFileSync(fd, body, { encoding: "utf-8" });
    fsyncSync(fd);
  } catch (error) {
    failure = error;
  } finally {
    if (fd !== undefined) {
      try {
        closeSync(fd);
      } catch (error) {
        failure ??= error;
      }
    }
  }

  if (failure) {
    if (fd !== undefined) {
      try {
        unlinkSync(outPath);
      } catch {
        // 仅清理由本次独占创建的路径；清理失败时保留原始错误。
      }
    }
    if (failure.code === "EEXIST" || failure.code === "ELOOP") {
      throw new Error("输出路径已存在或为符号链接，拒绝覆盖");
    }
    throw failure;
  }
}

function main() {
  const opts = parseArgs(process.argv.slice(2));
  if (opts.help) {
    printHelp();
    return;
  }
  const md = join(opts.secretsDir, `${opts.env}.md`);
  const env = parseDoc(md);
  const overlay = overlayNatsConf(env, opts.natsConf);

  const keys = Object.keys(env).sort();
  if (opts.dryKeys) {
    for (const k of keys) console.log(k);
    console.error(`# ${keys.length} keys from ${md}${overlay ? " + explicit nats config" : ""}`);
    return;
  }

  const body = keys.map((k) => `${k}=${env[k]}`).join("\n") + "\n";
  const outPath = resolve(opts.out);
  writeExclusive(outPath, body);
  console.error(
    `已写入 ${keys.length} 个键 -> ${outPath}（来源=${md}${overlay ? "，含显式 NATS 配置" : ""}）`,
  );
}

try {
  main();
} catch (error) {
  console.error(`错误: ${error instanceof Error ? error.message : "未知错误"}`);
  process.exitCode = 2;
}
