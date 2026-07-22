#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { mkdtempSync, rmSync, writeFileSync, writeSync } from "node:fs";
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

const timeoutSeconds = boundedInteger(
  process.env.CLICKHOUSE_HTTPS_TEST_TIMEOUT_SECONDS ?? "60",
  "CLICKHOUSE_HTTPS_TEST_TIMEOUT_SECONDS",
  1,
  600,
);
const certDir = mkdtempSync(path.join(tmpdir(), "infra-clickhouse-tls-"));
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
}

function generateCertificates() {
  const caKey = path.join(certDir, "ca.key");
  const caCert = path.join(certDir, "ca.crt");
  const badCaKey = path.join(certDir, "bad-ca.key");
  const badCaCert = path.join(certDir, "bad-ca.crt");
  const serverKey = path.join(certDir, "server.key");
  const serverCsr = path.join(certDir, "server.csr");
  const serverCert = path.join(certDir, "server.crt");
  const extension = path.join(certDir, "server.ext");
  writeFileSync(extension, "subjectAltName=DNS:localhost,IP:127.0.0.1\nextendedKeyUsage=serverAuth\n");
  run("openssl", ["req", "-x509", "-newkey", "rsa:2048", "-nodes", "-keyout", caKey, "-out", caCert, "-days", "2", "-subj", "/CN=infra-clickhouse-test-ca"], { capture: true });
  run("openssl", ["req", "-x509", "-newkey", "rsa:2048", "-nodes", "-keyout", badCaKey, "-out", badCaCert, "-days", "2", "-subj", "/CN=infra-clickhouse-wrong-ca"], { capture: true });
  run("openssl", ["req", "-newkey", "rsa:2048", "-nodes", "-keyout", serverKey, "-out", serverCsr, "-subj", "/CN=localhost"], { capture: true });
  run("openssl", ["x509", "-req", "-in", serverCsr, "-CA", caCert, "-CAkey", caKey, "-CAcreateserial", "-out", serverCert, "-days", "2", "-sha256", "-extfile", extension], { capture: true });
  return { caCert, badCaCert, serverCert, serverKey };
}

function cleanup() {
  if (cleaned) return;
  cleaned = true;
  try {
    rmSync(certDir, { recursive: true, force: true });
    log(`临时 TLS 证书已清理（result=${failed ? "failed" : "passed"}）`);
  } catch (error) {
    failed = true;
    process.exitCode = 1;
    logError(`临时 TLS 证书清理失败：${error instanceof Error ? error.message : String(error)}`);
  }
}

try {
  const certs = generateCertificates();
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
      "clickhousex",
      "--test",
      "https_conformance",
      "--",
      "--ignored",
      "--nocapture",
      "--test-threads=1",
    ],
    {
      env: {
        ...process.env,
        INFRA_CLICKHOUSE_TLS_CA_FILE: certs.caCert,
        INFRA_CLICKHOUSE_TLS_BAD_CA_FILE: certs.badCaCert,
        INFRA_CLICKHOUSE_TLS_CERT_FILE: certs.serverCert,
        INFRA_CLICKHOUSE_TLS_KEY_FILE: certs.serverKey,
      },
      timeoutMs: (timeoutSeconds + 15) * 1_000,
    },
  );
  log("ClickHouse HTTPS/CA conformance 已通过");
} catch (error) {
  failed = true;
  logError(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
} finally {
  cleanup();
}
