#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import net from "node:net";
import process from "node:process";

const timeoutSeconds = boundedInteger(
  process.env.KAFKA_BROKER_TEST_TIMEOUT_SECONDS ?? "120",
  "KAFKA_BROKER_TEST_TIMEOUT_SECONDS",
  1,
  1_800,
);
const runId = process.env.RUN_ID ?? `infra-kafka-broker-${process.env.USER ?? "agent"}-${process.pid}`;
const container = `${runId}-kafka`;
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
  console.log(`执行：${[command, ...args].join(" ")}`);
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

function reservePort() {
  return new Promise((resolve, reject) => {
    const server = net.createServer();
    server.once("error", reject);
    server.listen(0, "127.0.0.1", () => {
      const address = server.address();
      const port = typeof address === "object" && address ? address.port : 0;
      server.close((error) => (error ? reject(error) : resolve(port)));
    });
  });
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
  for (let attempt = 1; attempt <= 90; attempt += 1) {
    if (await probePort(port)) {
      console.log(`Kafka broker 已就绪：127.0.0.1:${port}`);
      return;
    }
    await new Promise((resolve) => setTimeout(resolve, 1_000));
  }
  throw new Error(`Kafka broker 未在 90 秒内就绪：127.0.0.1:${port}`);
}

function startKafka(port) {
  run(
    "docker",
    [
      "run",
      "--detach",
      "--name",
      container,
      "--label",
      `infra.kafka_broker.run_id=${runId}`,
      "--publish",
      `127.0.0.1:${port}:9092`,
      "--env",
      "KAFKA_NODE_ID=1",
      "--env",
      "KAFKA_PROCESS_ROLES=broker,controller",
      "--env",
      "KAFKA_LISTENERS=PLAINTEXT://:9092,CONTROLLER://:9093",
      "--env",
      `KAFKA_ADVERTISED_LISTENERS=PLAINTEXT://127.0.0.1:${port}`,
      "--env",
      "KAFKA_CONTROLLER_LISTENER_NAMES=CONTROLLER",
      "--env",
      "KAFKA_LISTENER_SECURITY_PROTOCOL_MAP=CONTROLLER:PLAINTEXT,PLAINTEXT:PLAINTEXT",
      "--env",
      "KAFKA_CONTROLLER_QUORUM_VOTERS=1@localhost:9093",
      "--env",
      "KAFKA_OFFSETS_TOPIC_REPLICATION_FACTOR=1",
      "--env",
      "KAFKA_TRANSACTION_STATE_LOG_REPLICATION_FACTOR=1",
      "--env",
      "KAFKA_TRANSACTION_STATE_LOG_MIN_ISR=1",
      "apache/kafka@sha256:22c4bea38875408e8f9fe52aca8e3a6ee67f9aa0090db59af99a2f6647558db5",
    ],
    { capture: true, timeoutMs: 120_000 },
  );
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
      "kafkax",
      "--test",
      "broker_conformance",
      "--",
      "--ignored",
      "--nocapture",
      "--test-threads=1",
    ],
    {
      env: {
        ...process.env,
        RUSTC_WRAPPER: "",
        FOUNDATIONX_KAFKAX_BROKERS: `127.0.0.1:${port}`,
      },
      timeoutMs: (timeoutSeconds + 15) * 1_000,
    },
  );
}

function cleanup() {
  if (cleaned) return;
  cleaned = true;
  if (failed) {
    console.error("Kafka broker conformance 失败，输出末尾容器日志后清理");
    const logs = spawnSync("docker", ["logs", "--tail", "200", container], {
      encoding: "utf8",
      stdio: "pipe",
      timeout: 30_000,
    });
    process.stderr.write(`${logs.stdout ?? ""}${logs.stderr ?? ""}`);
  }
  console.log(`清理 Kafka 容器：${container}`);
  const removal = spawnSync("docker", ["rm", "-f", container], {
    encoding: "utf8",
    stdio: "pipe",
    timeout: 30_000,
  });
  if (removal.error || removal.status !== 0) {
    failed = true;
    process.exitCode = 1;
    console.error(`Kafka 容器清理失败：${removal.error?.message ?? removal.stderr.trim()}`);
  }
}

try {
  const port = await reservePort();
  startKafka(port);
  await waitForPort(port);
  runConformance(port);
  console.log("Kafka broker AMO/ALO/重复窗口 conformance 已通过");
} catch (error) {
  failed = true;
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
} finally {
  cleanup();
}
