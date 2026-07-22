#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { writeSync } from "node:fs";
import net from "node:net";
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

let redisPort = process.env.REDIS_PORT
  ? parseInteger(process.env.REDIS_PORT, "REDIS_PORT", 1, 65_535)
  : 0;
let natsPort = process.env.NATS_PORT
  ? parseInteger(process.env.NATS_PORT, "NATS_PORT", 1, 65_535)
  : 0;
const timeoutSeconds = parseInteger(
  process.env.STORAGE_TEST_TIMEOUT_SECONDS ?? "120",
  "STORAGE_TEST_TIMEOUT_SECONDS",
  1,
  1_800,
);
const runId = process.env.RUN_ID ?? `infra-storage-${process.env.USER ?? "agent"}-${process.pid}`;
const redisContainer = `${runId}-redis`;
const natsContainer = `${runId}-nats`;
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

function parseInteger(raw, name, minimum, maximum) {
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

function startContainers() {
  run(
    "docker",
    [
      "run",
      "--detach",
      "--rm",
      "--name",
      redisContainer,
      "--label",
      `infra.storage.run_id=${runId}`,
      "--publish",
      `127.0.0.1:${redisPort}:6379`,
      "redis@sha256:bb186d083732f669da90be8b0f975a37812b15e913465bb14d845db72a4e3e08",
    ],
    { capture: true, timeoutMs: 120_000 },
  );
  if (redisPort === 0) {
    redisPort = publishedPort(redisContainer, 6379);
  }
  run(
    "docker",
    [
      "run",
      "--detach",
      "--rm",
      "--name",
      natsContainer,
      "--label",
      `infra.storage.run_id=${runId}`,
      "--publish",
      `127.0.0.1:${natsPort}:4222`,
      "nats@sha256:b83efabe3e7def1e0a4a31ec6e078999bb17c80363f881df35edc70fcb6bb927",
    ],
    { capture: true, timeoutMs: 120_000 },
  );
  if (natsPort === 0) {
    natsPort = publishedPort(natsContainer, 4222);
  }
}

function publishedPort(container, containerPort) {
  const output = run("docker", ["port", container, `${containerPort}/tcp`], {
    capture: true,
  });
  const match = output.match(/:(\d+)$/u);
  if (!match) {
    throw new Error(`无法解析 ${container}:${containerPort} 的宿主端口：${output}`);
  }
  return parseInteger(match[1], "Docker 动态端口", 1, 65_535);
}

function probePort(host, port) {
  return new Promise((resolve) => {
    const socket = net.createConnection({ host, port });
    const timer = setTimeout(() => {
      socket.destroy();
      resolve(false);
    }, 1_000);
    socket.once("connect", () => {
      clearTimeout(timer);
      socket.destroy();
      resolve(true);
    });
    socket.once("error", () => {
      clearTimeout(timer);
      socket.destroy();
      resolve(false);
    });
  });
}

async function waitForPort(host, port, label) {
  for (let attempt = 1; attempt <= 60; attempt += 1) {
    if (await probePort(host, port)) {
      log(`${label} 已就绪：${host}:${port}`);
      return;
    }
    await new Promise((resolve) => setTimeout(resolve, 1_000));
  }
  throw new Error(`${label} 未在 60 秒内就绪：${host}:${port}`);
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
      "bootstrap",
      "--test",
      "storage_composition_e2e",
      "--",
      "--ignored",
      "--nocapture",
      "--test-threads=1",
    ],
    {
      env: {
        ...process.env,
        INFRA_BOOTSTRAP_E2E_REDIS_URL: `redis://127.0.0.1:${redisPort}`,
        INFRA_BOOTSTRAP_E2E_NATS_URL: `nats://127.0.0.1:${natsPort}`,
      },
      timeoutMs: (timeoutSeconds + 15) * 1_000,
    },
  );
}

function cleanup() {
  if (cleaned) {
    return;
  }
  cleaned = true;
  if (failed) {
    logError("storage composition conformance 失败，输出容器日志后清理");
    spawnSync("docker", ["logs", redisContainer], { stdio: "inherit", timeout: 30_000 });
    spawnSync("docker", ["logs", natsContainer], { stdio: "inherit", timeout: 30_000 });
  }
  log(`清理容器：${redisContainer} ${natsContainer}`);
  const removal = spawnSync("docker", ["rm", "-f", redisContainer, natsContainer], {
    encoding: "utf8",
    stdio: "pipe",
    timeout: 30_000,
  });
  if (removal.error || removal.status !== 0) {
    failed = true;
    process.exitCode = 1;
    logError(`storage 容器清理失败：${removal.error?.message ?? removal.stderr.trim()}`);
  }
}

try {
  startContainers();
  await waitForPort("127.0.0.1", redisPort, "Redis");
  await waitForPort("127.0.0.1", natsPort, "NATS");
  runConformance();
  log("bootstrap 正式 storage contracts E2E 已通过");
} catch (error) {
  failed = true;
  logError(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
} finally {
  cleanup();
}
