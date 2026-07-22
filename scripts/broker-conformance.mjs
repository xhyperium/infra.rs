#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import net from "node:net";
import process from "node:process";

const kafkaPort = parsePort(process.env.KAFKA_PORT ?? "29092", "KAFKA_PORT");
const natsPort = parsePort(process.env.NATS_PORT ?? "24222", "NATS_PORT");
const timeoutSeconds = parseBoundedInteger(
  process.env.BROKER_TEST_TIMEOUT_SECONDS ?? "120",
  "BROKER_TEST_TIMEOUT_SECONDS",
  1,
  1_800,
);
const runId = process.env.RUN_ID ?? `infra-broker-${process.env.USER ?? "agent"}-${process.pid}`;
const kafkaContainer = `${runId}-kafka`;
const natsContainer = `${runId}-nats`;
let failed = false;
let cleaned = false;

for (const [signal,exitCode] of [
  ["SIGINT", 130],
  ["SIGTERM", 143],
]) {
  process.once(signal, () => {
    failed = true;
    cleanup();
    process.exit(exitCode);
  });
}

function parseBoundedInteger(raw, name, minimum, maximum) {
  const value = Number.parseInt(raw, 10);
  if (!Number.isInteger(value) || value < minimum || value > maximum) {
    throw new Error(`${name} 必须是 ${minimum}..${maximum} 的整数`);
  }
  return value;
}

function parsePort(raw, name) {
  return parseBoundedInteger(raw, name, 1, 65_535);
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

function startContainers() {
  run(
    "docker",
    [
      "run",
      "--detach",
      "--rm",
      "--name",
      kafkaContainer,
      "--label",
      `infra.broker.run_id=${runId}`,
      "--publish",
      `127.0.0.1:${kafkaPort}:9092`,
      "--env",
      "KAFKA_NODE_ID=1",
      "--env",
      "KAFKA_PROCESS_ROLES=broker,controller",
      "--env",
      "KAFKA_LISTENERS=PLAINTEXT://:9092,CONTROLLER://:9093",
      "--env",
      `KAFKA_ADVERTISED_LISTENERS=PLAINTEXT://127.0.0.1:${kafkaPort}`,
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
    { capture: true },
  );
  run(
    "docker",
    [
      "run",
      "--detach",
      "--rm",
      "--name",
      natsContainer,
      "--label",
      `infra.broker.run_id=${runId}`,
      "--publish",
      `127.0.0.1:${natsPort}:4222`,
      "nats@sha256:b83efabe3e7def1e0a4a31ec6e078999bb17c80363f881df35edc70fcb6bb927",
      "-js",
    ],
    { capture: true },
  );
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
  for (let attempt = 1; attempt <= 90; attempt += 1) {
    if (await probePort(host, port)) {
      console.log(`${label} 已就绪：${host}:${port}`);
      return;
    }
    await new Promise((resolve) => setTimeout(resolve, 1_000));
  }
  throw new Error(`${label} 未在 90 秒内就绪：${host}:${port}`);
}

function runConformance() {
  const timeoutArgs = [
    "--foreground",
    "--signal=TERM",
    "--kill-after=10s",
    `${timeoutSeconds}s`,
  ];
  run(
    "timeout",
    [
      ...timeoutArgs,
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
        FOUNDATIONX_KAFKAX_BROKERS: `127.0.0.1:${kafkaPort}`,
      },
      timeoutMs: (timeoutSeconds + 15) * 1_000,
    },
  );
  run(
    "timeout",
    [
      ...timeoutArgs,
      "cargo",
      "test",
      "-p",
      "natsx",
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
        FOUNDATIONX_NATS_URL: `nats://127.0.0.1:${natsPort}`,
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
    console.error("broker conformance 失败，输出容器日志后清理");
    spawnSync("docker", ["logs", kafkaContainer], { stdio: "inherit", timeout: 30_000 });
    spawnSync("docker", ["logs", natsContainer], { stdio: "inherit", timeout: 30_000 });
  }
  console.log(`清理容器：${kafkaContainer} ${natsContainer}`);
  spawnSync("docker", ["rm", "-f", kafkaContainer, natsContainer], {
    stdio: "ignore",
    timeout: 30_000,
  });
}

try {
  startContainers();
  await waitForPort("127.0.0.1", kafkaPort, "Kafka");
  await waitForPort("127.0.0.1", natsPort, "NATS");
  runConformance();
  console.log("Kafka/NATS broker conformance 已通过");
} catch (error) {
  failed = true;
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
} finally {
  cleanup();
}
