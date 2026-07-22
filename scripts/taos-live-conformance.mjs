#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { randomBytes } from "node:crypto";
import process from "node:process";

const images = {
  x64: "docker.io/tdengine/tdengine:3.3.6.13@sha256:aaad66b4b7fb0e732d053b6a62b8087f59d72fa0644214cc653c7a06c47d3137",
  arm64:
    "docker.io/tdengine/tdengine:3.3.6.13@sha256:5a6df8870404f87e20d77643c3b0ecb490c2bfee20872f25ff62b4f10abf24eb",
};

const image = images[process.arch];
if (!image) {
  throw new Error(`taos live conformance 不支持架构 ${process.arch}`);
}

const timeoutSeconds = boundedInteger(
  process.env.TAOS_LIVE_TEST_TIMEOUT_SECONDS ?? "180",
  "TAOS_LIVE_TEST_TIMEOUT_SECONDS",
  30,
  600,
);
const container = `infra-taos-${process.pid}-${Date.now()}`;
const password = `Ta0s!${randomBytes(8).toString("hex")}`;
const cleanHostEnv = Object.fromEntries(
  Object.entries(process.env).filter(([name]) => !name.startsWith("FOUNDATIONX_TAOSX_")),
);
let failed = false;

function boundedInteger(raw, name, minimum, maximum) {
  const value = Number.parseInt(raw, 10);
  if (!Number.isInteger(value) || value < minimum || value > maximum) {
    throw new Error(`${name} 必须为 ${minimum}..${maximum} 的整数`);
  }
  return value;
}

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    cwd: process.cwd(),
    env: options.env ?? process.env,
    encoding: "utf8",
    stdio: options.capture ? "pipe" : "inherit",
    timeout: options.timeoutMs ?? 30_000,
  });
  if (result.error) {
    throw new Error(`${command} 执行失败`, { cause: result.error });
  }
  if (result.status !== 0) {
    const detail = options.capture ? `: ${(result.stderr || result.stdout).trim()}` : "";
    throw new Error(`${command} 退出码 ${result.status}${detail}`);
  }
  return result.stdout?.trim() ?? "";
}

async function waitReady(port) {
  const deadline = Date.now() + 60_000;
  const authorization = `Basic ${Buffer.from(`root:${password}`, "utf8").toString("base64")}`;
  while (Date.now() < deadline) {
    try {
      const response = await fetch(`http://127.0.0.1:${port}/rest/sql`, {
        method: "POST",
        headers: { authorization, "content-type": "text/plain; charset=utf-8" },
        body: "SELECT SERVER_VERSION()",
        signal: AbortSignal.timeout(2_000),
      });
      if (response.ok) {
        const payload = await response.json();
        if (payload && payload.code === 0) return;
      }
    } catch {
      // 容器尚未 ready；在总 deadline 内继续。
    }
    await new Promise((resolve) => setTimeout(resolve, 500));
  }
  throw new Error("TDengine 容器 60 秒内未 ready");
}

function cleanup() {
  const result = spawnSync("docker", ["rm", "-f", container], {
    encoding: "utf8",
    stdio: "pipe",
    timeout: 30_000,
  });
  if (result.error || (result.status !== 0 && !result.stderr.includes("No such container"))) {
    failed = true;
    process.stderr.write("taos conformance 容器清理失败\n");
  }
}

for (const [signal, code] of [
  ["SIGINT", 130],
  ["SIGTERM", 143],
]) {
  process.once(signal, () => {
    cleanup();
    process.exit(code);
  });
}

try {
  run(
    "docker",
    [
      "run",
      "-d",
      "--rm",
      "--name",
      container,
      "-p",
      "127.0.0.1::6041",
      "-e",
      "TAOS_ROOT_PASSWORD",
      image,
    ],
    {
      capture: true,
      env: { ...process.env, TAOS_ROOT_PASSWORD: password },
      timeoutMs: timeoutSeconds * 1_000,
    },
  );
  const binding = run("docker", ["port", container, "6041/tcp"], { capture: true });
  const port = binding.slice(binding.lastIndexOf(":") + 1);
  if (!/^\d+$/.test(port)) throw new Error("无法解析 TDengine 动态 REST 端口");
  await waitReady(port);

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
      "taosx",
      "--test",
      "live_smoke",
      "--",
      "--ignored",
      "--test-threads=1",
    ],
    {
      env: {
        ...cleanHostEnv,
        FOUNDATIONX_TAOSX_HOST: "127.0.0.1",
        FOUNDATIONX_TAOSX_PORT: port,
        FOUNDATIONX_TAOSX_USER: "root",
        FOUNDATIONX_TAOSX_PASSWORD: password,
        FOUNDATIONX_TAOSX_DATABASE: `infra_taos_${process.pid}`,
        FOUNDATIONX_TAOSX_TLS: "false",
        FOUNDATIONX_TAOSX_TRANSPORT: "rest",
        FOUNDATIONX_TAOSX_PRECISION: "ms",
        FOUNDATIONX_TAOSX_TIMEOUT_MS: "10000",
        FOUNDATIONX_TAOSX_MAX_IN_FLIGHT: "8",
        FOUNDATIONX_TAOSX_ACQUIRE_TIMEOUT_MS: "5000",
        FOUNDATIONX_TAOSX_BATCH_MAX_ROWS: "100",
        FOUNDATIONX_TAOSX_BATCH_MAX_BYTES: "1048576",
        FOUNDATIONX_TAOSX_MAX_RESPONSE_BYTES: "8388608",
        FOUNDATIONX_TAOSX_MAX_QUERY_ROWS: "10000",
        FOUNDATIONX_TAOSX_CLOSE_TIMEOUT_MS: "5000",
      },
      timeoutMs: (timeoutSeconds + 15) * 1_000,
    },
  );
  process.stdout.write(`taos REST/Decimal live conformance 通过（image=${image}）\n`);
} catch (error) {
  failed = true;
  process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
  const logs = spawnSync("docker", ["logs", container], {
    encoding: "utf8",
    stdio: "pipe",
    timeout: 10_000,
  });
  const rawDiagnostic = `${logs.stdout ?? ""}${logs.stderr ?? ""}`;
  const diagnostic = rawDiagnostic
    .slice(-16 * 1024)
    .replaceAll(password, "***")
    .replaceAll("cm9vdDp0YW9zZGF0YQ==", "***");
  if (diagnostic.trim()) process.stderr.write(`${diagnostic.trim()}\n`);
} finally {
  cleanup();
}

if (failed) process.exitCode = 1;
