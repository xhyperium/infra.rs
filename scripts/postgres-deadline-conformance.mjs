#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { randomBytes } from "node:crypto";
import { writeSync } from "node:fs";
import process from "node:process";

const writeBackoff = new Int32Array(new SharedArrayBuffer(4));

function writeAll(fd, message) {
  const buffer = Buffer.from(`${message}\n`, "utf8");
  let offset = 0;
  while (offset < buffer.length) {
    try {
      const written = writeSync(fd, buffer, offset, buffer.length - offset);
      if (written === 0) throw new Error("同步日志写入未取得进展");
      offset += written;
    } catch (error) {
      const code = error && typeof error === "object" && "code" in error ? error.code : undefined;
      if (code === "EINTR") continue;
      if (code === "EAGAIN" || code === "EWOULDBLOCK") {
        Atomics.wait(writeBackoff, 0, 0, 1);
        continue;
      }
      throw error;
    }
  }
}

function log(message) {
  writeAll(process.stdout.fd, message);
}

function logError(message) {
  writeAll(process.stderr.fd, message);
}

const image =
  "postgres@sha256:742f40ea20b9ff2ff31db5458d127452988a2164df9e17441e191f3b72252193";
const timeoutSeconds = boundedInteger(
  process.env.POSTGRES_DEADLINE_TEST_TIMEOUT_SECONDS ?? "120",
  "POSTGRES_DEADLINE_TEST_TIMEOUT_SECONDS",
  1,
  600,
);
const runId = process.env.RUN_ID ?? `infra-postgres-${process.env.USER ?? "agent"}-${process.pid}`;
const container = `${runId}-postgres`;
const password = randomBytes(24).toString("hex");
let port = 0;
let failed = false;
let cleaned = false;

for (const [signal, exitCode] of [
  ["SIGINT", 130],
  ["SIGTERM", 143],
]) {
  process.once(signal, () => {
    failed = true;
    cleanup();
    process.exit(exitCode);
  });
}

function boundedInteger(raw, name, minimum, maximum) {
  const value = Number.parseInt(raw, 10);
  if (!Number.isInteger(value) || value < minimum || value > maximum) {
    throw new Error(`${name} 必须是 ${minimum}..${maximum} 的整数`);
  }
  return value;
}

function run(command, args, options = {}) {
  log(`执行：${[command, ...args].join(" ")}`);
  const result = spawnSync(command, args, {
    cwd: process.cwd(),
    env: options.env ?? process.env,
    encoding: "utf8",
    stdio: options.capture ? "pipe" : "inherit",
    timeout: options.timeoutMs ?? 30_000,
  });
  if (result.error) {
    throw new Error(`执行 ${command} 失败`, { cause: result.error });
  }
  if (result.status !== 0) {
    const detail = options.capture ? `\n${result.stderr || result.stdout}` : "";
    throw new Error(`${command} 退出码 ${result.status}${detail}`);
  }
  return result.stdout?.trim() ?? "";
}

function startPostgres() {
  run(
    "docker",
    [
      "run",
      "--detach",
      "--name",
      container,
      "--label",
      `infra.storage.run_id=${runId}`,
      "--env",
      "POSTGRES_PASSWORD",
      "--publish",
      "127.0.0.1::5432",
      image,
    ],
    { capture: true, timeoutMs: 120_000, env: { ...process.env, POSTGRES_PASSWORD: password } },
  );
  const published = run("docker", ["port", container, "5432/tcp"], { capture: true });
  const match = published.match(/:(\d+)$/u);
  if (!match) {
    throw new Error(`无法解析 PostgreSQL 动态端口：${published}`);
  }
  port = boundedInteger(match[1], "Docker 动态端口", 1, 65_535);
}

function waitUntilReady() {
  for (let attempt = 1; attempt <= 60; attempt += 1) {
    const result = spawnSync("docker", ["exec", container, "pg_isready", "-U", "postgres"], {
      stdio: "ignore",
      timeout: 2_000,
    });
    if (result.status === 0) {
      log(`PostgreSQL 已就绪：127.0.0.1:${port}`);
      return;
    }
    Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 1_000);
  }
  throw new Error("PostgreSQL 未在 60 秒内就绪");
}

function runConformance() {
  run(
    "timeout",
    [
      "--foreground",
      "--signal=TERM",
      "--kill-after=10s",
      `${timeoutSeconds}s`,
      "cargo",
      "test",
      "-p",
      "postgresx",
      "--test",
      "deadline_conformance",
      "--",
      "--ignored",
      "--nocapture",
      "--test-threads=1",
    ],
    {
      env: {
        ...process.env,
        INFRA_POSTGRES_TEST_PORT: String(port),
        INFRA_POSTGRES_TEST_PASSWORD: password,
      },
      timeoutMs: (timeoutSeconds + 15) * 1_000,
    },
  );
}

function cleanup() {
  if (cleaned) return;
  cleaned = true;
  if (failed) {
    spawnSync("docker", ["logs", container], { stdio: "inherit", timeout: 30_000 });
  }
  const removal = spawnSync("docker", ["rm", "-f", container], {
    encoding: "utf8",
    stdio: "pipe",
    timeout: 30_000,
  });
  if (removal.error || removal.status !== 0) {
    failed = true;
    process.exitCode = 1;
    logError(`PostgreSQL 容器清理失败：${removal.error?.message ?? removal.stderr.trim()}`);
  } else {
    log(`PostgreSQL 容器已清理（result=${failed ? "failed" : "passed"}）`);
  }
}

try {
  startPostgres();
  waitUntilReady();
  runConformance();
  log("PostgreSQL 截止时间与连接隔离 conformance 已通过");
} catch (error) {
  failed = true;
  logError(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
} finally {
  cleanup();
}
