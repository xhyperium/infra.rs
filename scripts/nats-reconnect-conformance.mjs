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

const timeoutSeconds = boundedInteger(
  process.env.NATS_RECONNECT_TEST_TIMEOUT_SECONDS ?? "150",
  "NATS_RECONNECT_TEST_TIMEOUT_SECONDS",
  1,
  1_800,
);
const runs = boundedInteger(
  process.env.NATS_RECONNECT_TEST_RUNS ?? "3",
  "NATS_RECONNECT_TEST_RUNS",
  1,
  20,
);
const runId = process.env.RUN_ID ?? `infra-nats-reconnect-${process.env.USER ?? "agent"}-${process.pid}`;
const container = `${runId}-nats`;
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

function startContainer() {
  run(
    "docker",
    [
      "run",
      "--detach",
      "--name",
      container,
      "--label",
      `infra.nats_reconnect.run_id=${runId}`,
      "--publish",
      "127.0.0.1:0:4222",
      "--entrypoint",
      "/bin/sh",
      "nats@sha256:b83efabe3e7def1e0a4a31ec6e078999bb17c80363f881df35edc70fcb6bb927",
      "-c",
      [
        "while :; do",
        "nats-server --config /etc/nats/nats-server.conf &",
        "child=$!;",
        "printf '%s\\n' \"$child\" > /tmp/nats-server.pid;",
        "wait \"$child\";",
        "sleep 1;",
        "done",
      ].join(" "),
    ],
    { capture: true, timeoutMs: 120_000 },
  );
  const output = run("docker", ["port", container, "4222/tcp"], { capture: true });
  const match = output.match(/:(\d+)$/u);
  if (!match) {
    throw new Error(`无法解析 NATS 动态端口：${output}`);
  }
  return boundedInteger(match[1], "Docker 动态端口", 1, 65_535);
}

function probePort(port) {
  return new Promise((resolve) => {
    const socket = net.createConnection({ host: "127.0.0.1", port });
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

async function waitForPort(port) {
  for (let attempt = 1; attempt <= 60; attempt += 1) {
    if (await probePort(port)) {
      log(`NATS 已就绪：127.0.0.1:${port}`);
      return;
    }
    await new Promise((resolve) => setTimeout(resolve, 1_000));
  }
  throw new Error(`NATS 未在 60 秒内就绪：127.0.0.1:${port}`);
}

function runConformance(port) {
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
      "natsx",
      "--test",
      "reconnect_conformance",
      "--",
      "--ignored",
      "--nocapture",
      "--test-threads=1",
    ],
    {
      env: {
        ...process.env,
        INFRA_NATS_RECONNECT_URL: `nats://127.0.0.1:${port}`,
        INFRA_NATS_RECONNECT_CONTAINER: container,
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
    logError("NATS reconnect conformance 失败，输出容器日志后清理");
    spawnSync("docker", ["logs", container], { stdio: "inherit", timeout: 30_000 });
  }
  log(`清理容器：${container}`);
  const removal = spawnSync("docker", ["rm", "-f", container], {
    encoding: "utf8",
    stdio: "pipe",
    timeout: 30_000,
  });
  if (removal.error || removal.status !== 0) {
    failed = true;
    process.exitCode = 1;
    logError(`NATS 容器清理失败：${removal.error?.message ?? removal.stderr.trim()}`);
  }
}

try {
  const port = startContainer();
  await waitForPort(port);
  for (let iteration = 1; iteration <= runs; iteration += 1) {
    log(`NATS 重连 conformance 轮次：${iteration}/${runs}`);
    runConformance(port);
  }
  log(`NATS 重连与慢消费者 conformance 已连续通过 ${runs} 轮`);
} catch (error) {
  failed = true;
  logError(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
} finally {
  cleanup();
}
