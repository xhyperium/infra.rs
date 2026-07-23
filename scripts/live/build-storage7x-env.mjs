#!/usr/bin/env node
/**
 * build-storage7x-env.mjs — 从 ZoneCNH secrets/env/{dev,prod}.md 生成 STORAGE7X_* .env
 *
 * 覆盖 7 个 storage 域：clickhouse / kafka / nats / oss / postgres / redis / taos。
 * 用途：本次 storage7x live 集成测试（`cargo test -p <adapter> -- --ignored`）。
 * 不打印密钥值；输出文件权限 0600。
 *
 * 用法:
 *   node scripts/live/build-storage7x-env.mjs --env dev --out <私有目录>/storage7x.env
 *   node scripts/live/build-storage7x-env.mjs --env prod --out <私有目录>/storage7x.env
 *   node scripts/live/build-storage7x-env.mjs --env dev --keys-only
 *   node scripts/live/build-storage7x-env.mjs --env dev --dry-run
 *
 * 与 build-foundationx-env.mjs 的差异（本文件新增，不修改 foundationx 既有文件）:
 *   - 新增 --env prod 校验分支：dev/prod 均解析各自 secrets/env/<env>.md，
 *     除 dev/prod 外的任何取值仍在读取文件前 fail-closed。
 *   - 字段范围收窄为本次 7 个 storage 域，不解析 FRED / Grafana / RabbitMQ / Qdrant 等无关字段。
 *   - dev.md 与 prod.md 表格列结构不同（prod 缺少 dev 版本的独立"数据库"列），
 *     因此分别为两个环境维护正则，而非用同一套正则跨结构匹配。
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
import { resolve } from "path";

const DEFAULT_SECRETS = "/home/workspace/ZoneCNH/sre/secrets/env";
const ALLOWED_ENVS = new Set(["dev", "prod"]);

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
    dryKeys: false,
    dryRun: false,
    help: false,
  };
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a === "--env") out.env = requireValue(argv, i++, a);
    else if (a === "--out") out.out = requireValue(argv, i++, a);
    else if (a === "--secrets-dir") out.secretsDir = requireValue(argv, i++, a);
    else if (a === "--keys-only") out.dryKeys = true;
    else if (a === "--dry-run") out.dryRun = true;
    else if (a === "-h" || a === "--help") out.help = true;
    else throw new Error(`未知参数: ${a}`);
  }
  if (!ALLOWED_ENVS.has(out.env)) {
    throw new Error("仅允许 --env dev 或 --env prod；其他取值已拒绝");
  }
  if (out.help) return out;
  if (!out.out && !out.dryKeys && !out.dryRun) {
    throw new Error("必须提供 --out <path>（或使用 --keys-only / --dry-run）");
  }
  if (out.out === "-") {
    throw new Error("禁止将凭据写入 stdout；请提供安全的文件路径");
  }
  return out;
}

function printHelp() {
  console.log(`用法: node scripts/live/build-storage7x-env.mjs --env dev|prod --out <path.env>
  --secrets-dir  默认: ${DEFAULT_SECRETS}
  --keys-only    仅打印键名，不打印值，不写文件
  --dry-run      同 --keys-only：仅打印将写入的键名，不写文件`);
}

function set(env, k, v) {
  if (v == null || String(v).length === 0) return;
  const value = String(v);
  if (/[\r\n\0]/u.test(value)) {
    throw new Error(`变量 ${k} 包含禁止的控制字符`);
  }
  env[k] = value;
}

/** dev.md：本机 127.0.0.1 部署，各服务表格含独立"数据库/用户名"列。 */
function parseDevDoc(text) {
  const env = {};

  // PostgreSQL admin row
  {
    const m = text.match(
      /\|\s*PostgreSQL\s*\|\s*127\.0\.0\.1\s*\|\s*5432\s*\|\s*postgres\s*\|\s*`([^`]+)`/,
    );
    if (m) {
      set(env, "STORAGE7X_POSTGRESX_HOST", "127.0.0.1");
      set(env, "STORAGE7X_POSTGRESX_PORT", "5432");
      set(env, "STORAGE7X_POSTGRESX_USER", "postgres");
      set(env, "STORAGE7X_POSTGRESX_PASSWORD", m[1]);
      set(env, "STORAGE7X_POSTGRESX_DATABASE", "postgres");
      set(env, "STORAGE7X_POSTGRESX_SSLMODE", "disable");
    }
  }

  // TDengine (taos) root — REST port 6041
  {
    const m = text.match(
      /\|\s*TDengine\s*\|\s*127\.0\.0\.1\s*\|\s*6030\s*\/\s*6041\s*\|\s*root\s*\|\s*`([^`]+)`/,
    );
    if (m) {
      set(env, "STORAGE7X_TAOSX_HOST", "127.0.0.1");
      set(env, "STORAGE7X_TAOSX_PORT", "6041");
      set(env, "STORAGE7X_TAOSX_USER", "root");
      set(env, "STORAGE7X_TAOSX_PASSWORD", m[1]);
      set(env, "STORAGE7X_TAOSX_TLS", "false");
    }
  }

  // Redis
  {
    const m = text.match(
      /\|\s*Redis\s*\|\s*127\.0\.0\.1\s*\|\s*6379\s*\|\s*default\s*\|\s*`([^`]+)`/,
    );
    if (m) {
      set(env, "STORAGE7X_REDISX_ADDR", "127.0.0.1:6379");
      set(env, "STORAGE7X_REDISX_USERNAME", "default");
      set(env, "STORAGE7X_REDISX_PASSWORD", m[1]);
      set(env, "STORAGE7X_REDISX_DB", "0");
      set(env, "STORAGE7X_REDISX_TLS", "false");
    }
  }

  // Kafka SASL
  {
    const m = text.match(
      /\|\s*Kafka\s*\|\s*127\.0\.0\.1\s*\|\s*9092\s*\|\s*admin\s*\|\s*`([^`]+)`/,
    );
    if (m) {
      set(env, "STORAGE7X_KAFKAX_BROKERS", "127.0.0.1:9092");
      set(env, "STORAGE7X_KAFKAX_SASL_MECHANISM", "PLAIN");
      set(env, "STORAGE7X_KAFKAX_SASL_USERNAME", "admin");
      set(env, "STORAGE7X_KAFKAX_SASL_PASSWORD", m[1]);
      set(env, "STORAGE7X_KAFKAX_TLS", "false");
    }
  }

  // ClickHouse HTTP (8123) — “其他服务凭据”表格行
  {
    const m = text.match(
      /\|\s*ClickHouse\s*\|\s*127\.0\.0\.1\s*\|\s*9000\s*\/\s*8123\s*\|\s*default\s*\|\s*`([^`]+)`/,
    );
    if (m) {
      set(env, "STORAGE7X_CLICKHOUSEX_HOST", "127.0.0.1");
      set(env, "STORAGE7X_CLICKHOUSEX_HTTP_PORT", "8123");
      set(env, "STORAGE7X_CLICKHOUSEX_PORT", "8123");
      set(env, "STORAGE7X_CLICKHOUSEX_USER", "default");
      set(env, "STORAGE7X_CLICKHOUSEX_PASSWORD", m[1]);
      set(env, "STORAGE7X_CLICKHOUSEX_SSLMODE", "disable");
    }
  }

  // NATS — 独立小节的"认证"行
  {
    const auth = text.match(/\*\*认证\*\*:\s*用户名\s*`([^`]+)`\s*，密码\s*`([^`]+)`/);
    if (auth) {
      set(env, "STORAGE7X_NATS_URL", "nats://127.0.0.1:4222");
      set(env, "STORAGE7X_NATS_USER", auth[1]);
      set(env, "STORAGE7X_NATS_PASSWORD", auth[2]);
      set(env, "STORAGE7X_NATSX_URL", "nats://127.0.0.1:4222");
      set(env, "STORAGE7X_NATSX_USER", auth[1]);
      set(env, "STORAGE7X_NATSX_PASSWORD", auth[2]);
    }
  }

  // 阿里云 OSS (polarisx)
  {
    const ak = text.match(/\*\*AccessKey ID\*\*\s*\|\s*`([^`]+)`/);
    const sk = text.match(/\*\*AccessKey Secret\*\*\s*\|\s*`([^`]+)`/);
    const bucket = text.match(/\*\*Bucket\*\*\s*\|\s*`([^`]+)`/);
    const ep = text.match(/外网访问\s*\|\s*`([^`]+)`/);
    if (ak) set(env, "STORAGE7X_OSSX_ACCESS_KEY_ID", ak[1]);
    if (sk) set(env, "STORAGE7X_OSSX_ACCESS_KEY_SECRET", sk[1]);
    if (bucket) set(env, "STORAGE7X_OSSX_BUCKET", bucket[1]);
    if (ep) {
      const host = ep[1];
      set(
        env,
        "STORAGE7X_OSSX_ENDPOINT",
        host.startsWith("http") ? host : `https://${host}`,
      );
    }
    set(env, "STORAGE7X_OSSX_REGION", "ap-northeast-1");
  }

  return env;
}

/** prod.md：远端部署，管理员/其他服务表格无独立"数据库"列，以 IP 地址列开头。 */
function parseProdDoc(text) {
  const env = {};

  // PostgreSQL admin row
  {
    const m = text.match(
      /\|\s*PostgreSQL\s*\|\s*[\d.]+\s*\|\s*5432\s*\|\s*postgres\s*\|\s*`([^`]+)`/,
    );
    if (m) {
      const host = text.match(/\|\s*PostgreSQL\s*\|\s*([\d.]+)\s*\|\s*5432/);
      set(env, "STORAGE7X_POSTGRESX_HOST", host ? host[1] : "");
      set(env, "STORAGE7X_POSTGRESX_PORT", "5432");
      set(env, "STORAGE7X_POSTGRESX_USER", "postgres");
      set(env, "STORAGE7X_POSTGRESX_PASSWORD", m[1]);
      set(env, "STORAGE7X_POSTGRESX_DATABASE", "postgres");
      set(env, "STORAGE7X_POSTGRESX_SSLMODE", "disable");
    }
  }

  // TDengine (taos) root — REST port 6041
  {
    const m = text.match(
      /\|\s*TDengine\s*\|\s*[\d.]+\s*\|\s*6030\s*\/\s*6041\s*\|\s*root\s*\|\s*`([^`]+)`/,
    );
    if (m) {
      const host = text.match(/\|\s*TDengine\s*\|\s*([\d.]+)\s*\|\s*6030\s*\/\s*6041/);
      set(env, "STORAGE7X_TAOSX_HOST", host ? host[1] : "");
      set(env, "STORAGE7X_TAOSX_PORT", "6041");
      set(env, "STORAGE7X_TAOSX_USER", "root");
      set(env, "STORAGE7X_TAOSX_PASSWORD", m[1]);
      set(env, "STORAGE7X_TAOSX_TLS", "false");
    }
  }

  // Redis — “其他服务凭据”表格行
  {
    const m = text.match(
      /\|\s*Redis\s*\|\s*([\d.]+)\s*\|\s*6379\s*\|\s*default\s*\|\s*`([^`]+)`/,
    );
    if (m) {
      set(env, "STORAGE7X_REDISX_ADDR", `${m[1]}:6379`);
      set(env, "STORAGE7X_REDISX_USERNAME", "default");
      set(env, "STORAGE7X_REDISX_PASSWORD", m[2]);
      set(env, "STORAGE7X_REDISX_DB", "0");
      set(env, "STORAGE7X_REDISX_TLS", "false");
    }
  }

  // Kafka SASL
  {
    const m = text.match(
      /\|\s*Kafka\s*\|\s*([\d.]+)\s*\|\s*9092\s*\|\s*admin\s*\|\s*`([^`]+)`/,
    );
    if (m) {
      set(env, "STORAGE7X_KAFKAX_BROKERS", `${m[1]}:9092`);
      set(env, "STORAGE7X_KAFKAX_SASL_MECHANISM", "PLAIN");
      set(env, "STORAGE7X_KAFKAX_SASL_USERNAME", "admin");
      set(env, "STORAGE7X_KAFKAX_SASL_PASSWORD", m[2]);
      set(env, "STORAGE7X_KAFKAX_TLS", "false");
    }
  }

  // ClickHouse — prod 文档中标注仅 localhost，沿用 127.0.0.1
  {
    const m = text.match(
      /\|\s*ClickHouse\s*\|\s*127\.0\.0\.1\s*\|\s*9000\s*\/\s*8123\s*\|\s*default\s*\|\s*`([^`]+)`/,
    );
    if (m) {
      set(env, "STORAGE7X_CLICKHOUSEX_HOST", "127.0.0.1");
      set(env, "STORAGE7X_CLICKHOUSEX_HTTP_PORT", "8123");
      set(env, "STORAGE7X_CLICKHOUSEX_PORT", "8123");
      set(env, "STORAGE7X_CLICKHOUSEX_USER", "default");
      set(env, "STORAGE7X_CLICKHOUSEX_PASSWORD", m[1]);
      set(env, "STORAGE7X_CLICKHOUSEX_SSLMODE", "disable");
    }
  }

  // NATS — “其他服务凭据”表格行（prod 用户名可能与 dev 不同，以文档实际值为准）
  {
    const m = text.match(
      /\|\s*NATS\s*\|\s*([\d.]+)\s*\|\s*4222\s*\|\s*([^\s|]+)\s*\|\s*`([^`]+)`/,
    );
    if (m) {
      const url = `nats://${m[1]}:4222`;
      set(env, "STORAGE7X_NATS_URL", url);
      set(env, "STORAGE7X_NATS_USER", m[2]);
      set(env, "STORAGE7X_NATS_PASSWORD", m[3]);
      set(env, "STORAGE7X_NATSX_URL", url);
      set(env, "STORAGE7X_NATSX_USER", m[2]);
      set(env, "STORAGE7X_NATSX_PASSWORD", m[3]);
    }
  }

  // 阿里云 OSS (polarisx)
  {
    const ak = text.match(/\*\*AccessKey ID\*\*\s*\|\s*`([^`]+)`/);
    const sk = text.match(/\*\*AccessKey Secret\*\*\s*\|\s*`([^`]+)`/);
    const bucket = text.match(/\*\*Bucket\*\*\s*\|\s*`([^`]+)`/);
    const ep = text.match(/外网访问\s*\|\s*`([^`]+)`/);
    if (ak) set(env, "STORAGE7X_OSSX_ACCESS_KEY_ID", ak[1]);
    if (sk) set(env, "STORAGE7X_OSSX_ACCESS_KEY_SECRET", sk[1]);
    if (bucket) set(env, "STORAGE7X_OSSX_BUCKET", bucket[1]);
    if (ep) {
      const host = ep[1];
      set(
        env,
        "STORAGE7X_OSSX_ENDPOINT",
        host.startsWith("http") ? host : `https://${host}`,
      );
    }
    set(env, "STORAGE7X_OSSX_REGION", "ap-northeast-1");
  }

  return env;
}

function parseDoc(env, filePath) {
  if (!existsSync(filePath)) throw new Error(`secrets file not found: ${filePath}`);
  const text = readFileSync(filePath, "utf-8");
  return env === "prod" ? parseProdDoc(text) : parseDevDoc(text);
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
  const md = resolve(opts.secretsDir, `${opts.env}.md`);
  const env = parseDoc(opts.env, md);

  const keys = Object.keys(env).sort();
  if (opts.dryKeys || opts.dryRun) {
    for (const k of keys) console.log(k);
    console.error(`# ${keys.length} keys from ${md}（env=${opts.env}，dry-run 未写入文件）`);
    return;
  }

  const body = keys.map((k) => `${k}=${env[k]}`).join("\n") + "\n";
  const outPath = resolve(opts.out);
  writeExclusive(outPath, body);
  console.error(`已写入 ${keys.length} 个键 -> ${outPath}（来源=${md}，env=${opts.env}）`);
}

try {
  main();
} catch (error) {
  console.error(`错误: ${error instanceof Error ? error.message : "未知错误"}`);
  process.exitCode = 2;
}
