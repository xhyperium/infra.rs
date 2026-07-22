#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { randomBytes } from "node:crypto";
import { chmodSync, mkdtempSync, rmSync, writeFileSync, writeSync } from "node:fs";
import net from "node:net";
import { tmpdir } from "node:os";
import path from "node:path";
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

function redactSensitive(message) {
  let redacted = String(message ?? "");
  for (const value of [password, keystorePassword]) {
    if (value) redacted = redacted.split(value).join("<redacted>");
  }
  return redacted;
}

const timeoutSeconds = boundedInteger(
  process.env.KAFKA_TLS_TEST_TIMEOUT_SECONDS ?? "180",
  "KAFKA_TLS_TEST_TIMEOUT_SECONDS",
  1,
  1_800,
);
const runId = process.env.RUN_ID ?? `infra-kafka-tls-${process.env.USER ?? "agent"}-${process.pid}`;
const container = `${runId}-kafka`;
const secretsDir = mkdtempSync(path.join(tmpdir(), "infra-kafka-tls-"));
const username = "infra-test";
const password = randomBytes(24).toString("hex");
const keystorePassword = randomBytes(24).toString("hex");
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
  log(options.sensitive ? `执行：${command}（参数已脱敏）` : `执行：${[command, ...args].join(" ")}`);
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
    const detail = options.capture ? `\n${redactSensitive(result.stderr || result.stdout)}` : "";
    throw new Error(`${command} 退出码 ${result.status}${detail}`);
  }
  return result.stdout?.trim() ?? "";
}

function generateCertificates() {
  const caKey = path.join(secretsDir, "ca.key");
  const caCert = path.join(secretsDir, "ca.crt");
  const badCaKey = path.join(secretsDir, "bad-ca.key");
  const badCaCert = path.join(secretsDir, "bad-ca.crt");
  const serverKey = path.join(secretsDir, "server.key");
  const serverCsr = path.join(secretsDir, "server.csr");
  const serverCert = path.join(secretsDir, "server.crt");
  const extension = path.join(secretsDir, "server.ext");
  const keystore = path.join(secretsDir, "server.p12");
  const truststore = path.join(secretsDir, "truststore.p12");

  writeFileSync(extension, "subjectAltName=DNS:localhost,IP:127.0.0.1\nextendedKeyUsage=serverAuth\n");
  run("openssl", ["req", "-x509", "-newkey", "rsa:2048", "-nodes", "-keyout", caKey, "-out", caCert, "-days", "2", "-subj", "/CN=infra-kafka-test-ca"], { capture: true });
  run("openssl", ["req", "-x509", "-newkey", "rsa:2048", "-nodes", "-keyout", badCaKey, "-out", badCaCert, "-days", "2", "-subj", "/CN=infra-kafka-wrong-ca"], { capture: true });
  run("openssl", ["req", "-newkey", "rsa:2048", "-nodes", "-keyout", serverKey, "-out", serverCsr, "-subj", "/CN=localhost"], { capture: true });
  run("openssl", ["x509", "-req", "-in", serverCsr, "-CA", caCert, "-CAkey", caKey, "-CAcreateserial", "-out", serverCert, "-days", "2", "-sha256", "-extfile", extension], { capture: true });
  run(
    "openssl",
    ["pkcs12", "-export", "-in", serverCert, "-inkey", serverKey, "-certfile", caCert, "-name", "kafka", "-out", keystore, "-passout", "env:INFRA_KEYSTORE_PASSWORD"],
    { capture: true, sensitive: true, env: { ...process.env, INFRA_KEYSTORE_PASSWORD: keystorePassword } },
  );
  run(
    "keytool",
    [
      "-importcert",
      "-noprompt",
      "-alias",
      "infra-test-ca",
      "-file",
      caCert,
      "-keystore",
      truststore,
      "-storetype",
      "PKCS12",
      "-storepass:env",
      "INFRA_KEYSTORE_PASSWORD",
    ],
    {
      capture: true,
      sensitive: true,
      env: { ...process.env, INFRA_KEYSTORE_PASSWORD: keystorePassword },
    },
  );

  writeFileSync(path.join(secretsDir, "key-credentials"), keystorePassword);
  writeFileSync(path.join(secretsDir, "keystore-credentials"), keystorePassword);
  writeFileSync(path.join(secretsDir, "truststore-credentials"), keystorePassword);
  writeFileSync(
    path.join(secretsDir, "kafka_server_jaas.conf"),
    `KafkaServer {\n  org.apache.kafka.common.security.plain.PlainLoginModule required\n  username="${username}"\n  password="${password}"\n  user_${username}="${password}";\n};\n`,
  );
  for (const file of [caCert, badCaCert, serverCsr, serverCert, extension, truststore]) {
    chmodSync(file, 0o644);
  }
  for (const file of [
    caKey,
    badCaKey,
    serverKey,
    keystore,
    path.join(secretsDir, "key-credentials"),
    path.join(secretsDir, "keystore-credentials"),
    path.join(secretsDir, "truststore-credentials"),
    path.join(secretsDir, "kafka_server_jaas.conf"),
  ]) {
    chmodSync(file, 0o600);
  }
  return { caCert, badCaCert };
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
  for (let attempt = 1; attempt <= 120; attempt += 1) {
    if (await probePort(port)) {
      log(`Kafka TLS 端口已就绪：127.0.0.1:${port}`);
      return;
    }
    await new Promise((resolve) => setTimeout(resolve, 1_000));
  }
  throw new Error(`Kafka TLS 未在 120 秒内就绪：127.0.0.1:${port}`);
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
      `infra.kafka_tls.run_id=${runId}`,
      "--publish",
      `127.0.0.1:${port}:9092`,
      "--volume",
      `${secretsDir}:/etc/kafka/secrets:ro`,
      "--env",
      "KAFKA_NODE_ID=1",
      "--env",
      "KAFKA_PROCESS_ROLES=broker,controller",
      "--env",
      "KAFKA_LISTENERS=SASL_SSL://:9092,CONTROLLER://:9093",
      "--env",
      `KAFKA_ADVERTISED_LISTENERS=SASL_SSL://localhost:${port}`,
      "--env",
      "KAFKA_CONTROLLER_LISTENER_NAMES=CONTROLLER",
      "--env",
      "KAFKA_LISTENER_SECURITY_PROTOCOL_MAP=CONTROLLER:PLAINTEXT,SASL_SSL:SASL_SSL",
      "--env",
      "KAFKA_CONTROLLER_QUORUM_VOTERS=1@localhost:9093",
      "--env",
      "KAFKA_INTER_BROKER_LISTENER_NAME=SASL_SSL",
      "--env",
      "KAFKA_SASL_ENABLED_MECHANISMS=PLAIN",
      "--env",
      "KAFKA_SASL_MECHANISM_INTER_BROKER_PROTOCOL=PLAIN",
      "--env",
      "KAFKA_SSL_KEYSTORE_FILENAME=server.p12",
      "--env",
      "KAFKA_SSL_KEY_CREDENTIALS=key-credentials",
      "--env",
      "KAFKA_SSL_KEYSTORE_CREDENTIALS=keystore-credentials",
      "--env",
      "KAFKA_SSL_KEYSTORE_TYPE=PKCS12",
      "--env",
      "KAFKA_SSL_TRUSTSTORE_FILENAME=truststore.p12",
      "--env",
      "KAFKA_SSL_TRUSTSTORE_CREDENTIALS=truststore-credentials",
      "--env",
      "KAFKA_SSL_TRUSTSTORE_TYPE=PKCS12",
      "--env",
      "KAFKA_SSL_CLIENT_AUTH=requested",
      "--env",
      "KAFKA_OPTS=-Djava.security.auth.login.config=/etc/kafka/secrets/kafka_server_jaas.conf",
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

function runConformance(port, certificates) {
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
      "tls_sasl_conformance",
      "--",
      "--ignored",
      "--nocapture",
      "--test-threads=1",
    ],
    {
      env: {
        ...process.env,
        INFRA_KAFKA_TLS_BROKER: `localhost:${port}`,
        INFRA_KAFKA_TLS_USERNAME: username,
        INFRA_KAFKA_TLS_PASSWORD: password,
        INFRA_KAFKA_TLS_CA_FILE: certificates.caCert,
        INFRA_KAFKA_TLS_BAD_CA_FILE: certificates.badCaCert,
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
    logError("Kafka TLS+SASL conformance 失败，输出脱敏容器日志后清理");
    const logs = spawnSync("docker", ["logs", "--tail", "200", container], {
      encoding: "utf8",
      stdio: "pipe",
      timeout: 30_000,
    });
    logError(redactSensitive(`${logs.stdout ?? ""}${logs.stderr ?? ""}`));
  }
  log(`清理容器与临时证书：${container}`);
  const removal = spawnSync("docker", ["rm", "-f", container], {
    encoding: "utf8",
    stdio: "pipe",
    timeout: 30_000,
  });
  if (removal.error || removal.status !== 0) {
    failed = true;
    process.exitCode = 1;
    logError(`Kafka 容器清理失败：${removal.error?.message ?? removal.stderr.trim()}`);
  }
  try {
    rmSync(secretsDir, { recursive: true, force: true });
  } catch (error) {
    failed = true;
    process.exitCode = 1;
    logError(`Kafka 临时证书清理失败：${error instanceof Error ? error.message : String(error)}`);
  }
}

try {
  const certificates = generateCertificates();
  const port = await reservePort();
  startKafka(port);
  await waitForPort(port);
  runConformance(port, certificates);
  log("Kafka TLS+SASL/PLAIN conformance 已通过");
} catch (error) {
  failed = true;
  logError(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
} finally {
  cleanup();
}
