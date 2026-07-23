#!/usr/bin/env node
/**
 * kafkax 生产测试矩阵 runner：隔离单节点 broker + prod_reliability + 可选故障注入/soak。
 *
 * 用法：
 *   node scripts/kafka-prod-matrix.mjs
 *   node scripts/kafka-prod-matrix.mjs --fault-restart
 *   KAFKAX_SOAK_SECONDS=30 node scripts/kafka-prod-matrix.mjs --soak
 *
 * 不证明 HA / group rebalance / native EOS / 24h 门禁。
 */

import { spawnSync } from "node:child_process";
import net from "node:net";
import process from "node:process";

const timeoutSeconds = boundedInteger(
  process.env.KAFKA_PROD_MATRIX_TIMEOUT_SECONDS ?? "300",
  "KAFKA_PROD_MATRIX_TIMEOUT_SECONDS",
  30,
  3_600,
);
const runId =
  process.env.RUN_ID ?? `infra-kafka-prod-${process.env.USER ?? "agent"}-${process.pid}`;
const container = `${runId}-kafka`;
const wantFault = process.argv.includes("--fault-restart");
const wantSoak = process.argv.includes("--soak");
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
  const printable = [command, ...args].join(" ");
  console.log(`执行：${printable.length > 200 ? printable.slice(0, 200) + "…" : printable}`);
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
      `infra.kafka_prod.run_id=${runId}`,
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
      // 允许较大消息（1MiB 测试）
      "--env",
      "KAFKA_MESSAGE_MAX_BYTES=2000000",
      "--env",
      "KAFKA_REPLICA_FETCH_MAX_BYTES=2000000",
      "apache/kafka@sha256:22c4bea38875408e8f9fe52aca8e3a6ee67f9aa0090db59af99a2f6647558db5",
    ],
    { capture: true, timeoutMs: 120_000 },
  );
}

function cargoTest(filters, env, seconds) {
  // cargo 测试过滤器是**子串匹配**，不是 shell/regex `|`。
  const list = Array.isArray(filters) ? filters : filters ? [filters] : [null];
  for (const filter of list) {
    const args = [
      "--foreground",
      "--signal=TERM",
      "--kill-after=10s",
      `${seconds}s`,
      "cargo",
      "test",
      "-p",
      "kafkax",
      "--test",
      "prod_reliability",
      "--",
      "--ignored",
      "--nocapture",
      "--test-threads=1",
    ];
    if (filter) {
      args.push(filter);
    }
    run("timeout", args, {
      env: {
        ...process.env,
        RUSTC_WRAPPER: "",
        ...env,
      },
      timeoutMs: (seconds + 20) * 1_000,
    });
  }
}

function cleanup() {
  if (cleaned) return;
  cleaned = true;
  if (failed) {
    console.error("prod matrix 失败，输出容器日志后清理");
    const logs = spawnSync("docker", ["logs", "--tail", "200", container], {
      encoding: "utf8",
      stdio: "pipe",
      timeout: 30_000,
    });
    process.stderr.write(`${logs.stdout ?? ""}${logs.stderr ?? ""}`);
  }
  console.log(`清理 Kafka 容器：${container}`);
  spawnSync("docker", ["rm", "-f", container], {
    encoding: "utf8",
    stdio: "pipe",
    timeout: 30_000,
  });
}

try {
  const port = await reservePort();
  const brokers = `127.0.0.1:${port}`;
  startKafka(port);
  await waitForPort(port);

  // 主矩阵：一次跑全部 ignored（soak/fault 测试在未设 env 时早退成功）
  cargoTest(null, { FOUNDATIONX_KAFKAX_BROKERS: brokers }, timeoutSeconds);
  console.log("prod_reliability 主场景 PASS");

  if (wantFault) {
    console.log("故障注入：停止 broker…");
    run("docker", ["stop", container], { capture: true, timeoutMs: 60_000 });
    cargoTest(
      "fault_broker_down_connect_fails",
      {
        FOUNDATIONX_KAFKAX_BROKERS: brokers,
        KAFKAX_EXPECT_BROKER_DOWN: "1",
      },
      60,
    );
    console.log("故障注入：重启 broker…");
    // KRaft 单容器 stop/start 可能卡 DUPLICATE_BROKER_REGISTRATION → 重建容器
    run("docker", ["rm", "-f", container], { capture: true, timeoutMs: 60_000 });
    cleaned = false; // allow final cleanup of new container name collision
    startKafka(port);
    await waitForPort(port);
    cargoTest(
      "observability_stats_increment_on_publish",
      { FOUNDATIONX_KAFKAX_BROKERS: brokers },
      120,
    );
    console.log("故障注入：停机可失败 + 重建可恢复 PASS");
  }

  if (wantSoak) {
    const soak = process.env.KAFKAX_SOAK_SECONDS ?? "30";
    console.log(`有界 soak：KAFKAX_SOAK_SECONDS=${soak}`);
    cargoTest(
      "optional_bounded_soak_loop",
      {
        FOUNDATIONX_KAFKAX_BROKERS: brokers,
        KAFKAX_SOAK_SECONDS: soak,
      },
      Math.max(timeoutSeconds, Number(soak) + 60),
    );
    console.log("有界 soak PASS");
  }

  console.log("Kafka 生产测试矩阵（隔离 broker）已通过");
  console.log("说明：不包含 24h soak 默认门禁、group rebalance、native EOS、HA。");
} catch (error) {
  failed = true;
  console.error(error?.stack || error);
  process.exitCode = 1;
} finally {
  cleanup();
}
