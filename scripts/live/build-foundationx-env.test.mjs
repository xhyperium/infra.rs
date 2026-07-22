import test from "node:test";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
  chmodSync,
  existsSync,
  lstatSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
  statSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const HERE = dirname(fileURLToPath(import.meta.url));
const BUILDER = join(HERE, "build-foundationx-env.mjs");
const RUNNER = join(HERE, "run-foundationx-command.mjs");
const WRAPPER = join(HERE, "export-foundationx-env.sh");

function withTempDir(run) {
  const root = mkdtempSync(join(tmpdir(), "foundationx-live-test-"));
  try {
    return run(root);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}

function writeDevFixture(root, password = "dummy-dev-secret") {
  const secretsDir = join(root, "secrets");
  mkdirSync(secretsDir);
  writeFileSync(
    join(secretsDir, "dev.md"),
    `| Redis | 127.0.0.1 | 6379 | default | \`${password}\` |\n`,
    "utf-8",
  );
  return secretsDir;
}

function runBuilder(args) {
  return spawnSync(process.execPath, [BUILDER, ...args], { encoding: "utf-8" });
}

function runWrapper(args, env = process.env) {
  return spawnSync("bash", [WRAPPER, ...args], {
    encoding: "utf-8",
    env,
  });
}

test("dev 输出以 0600 独占创建且日志不包含值", () => {
  withTempDir((root) => {
    const secret = "dummy-dev-secret";
    const secretsDir = writeDevFixture(root, secret);
    const out = join(root, "dev.env");
    const result = runBuilder(["--env", "dev", "--secrets-dir", secretsDir, "--out", out]);

    assert.equal(result.status, 0, result.stderr);
    assert.equal(statSync(out).mode & 0o777, 0o600);
    assert.match(readFileSync(out, "utf-8"), /^FOUNDATIONX_REDISX_PASSWORD=/mu);
    assert.ok(!result.stdout.includes(secret));
    assert.ok(!result.stderr.includes(secret));

    const keys = runBuilder(["--env", "dev", "--secrets-dir", secretsDir, "--keys-only"]);
    assert.equal(keys.status, 0, keys.stderr);
    assert.match(keys.stdout, /FOUNDATIONX_REDISX_PASSWORD/u);
    assert.ok(!keys.stdout.includes(secret));
    assert.ok(!keys.stderr.includes(secret));
  });
});

test("prod 在读取任何文件前 fail-closed", () => {
  withTempDir((root) => {
    const out = join(root, "prod.env");
    const missingSecrets = join(root, "missing-secrets");
    const result = runBuilder([
      "--env",
      "prod",
      "--secrets-dir",
      missingSecrets,
      "--out",
      out,
    ]);
    assert.equal(result.status, 2);
    assert.match(result.stderr, /仅允许读取 dev 凭据/u);
    assert.doesNotMatch(result.stderr, /not found/u);
    assert.equal(existsSync(out), false);

    const keys = runBuilder([
      "--env",
      "prod",
      "--secrets-dir",
      missingSecrets,
      "--keys-only",
    ]);
    assert.equal(keys.status, 2);
    assert.match(keys.stderr, /仅允许读取 dev 凭据/u);
  });
});

test("拒绝覆盖既有文件、有效符号链接与断链符号链接", () => {
  withTempDir((root) => {
    const secretsDir = writeDevFixture(root);
    const existing = join(root, "existing.env");
    writeFileSync(existing, "保持原内容\n", "utf-8");
    const existingResult = runBuilder([
      "--env",
      "dev",
      "--secrets-dir",
      secretsDir,
      "--out",
      existing,
    ]);
    assert.equal(existingResult.status, 2);
    assert.equal(readFileSync(existing, "utf-8"), "保持原内容\n");

    const target = join(root, "target.env");
    writeFileSync(target, "目标原内容\n", "utf-8");
    const validLink = join(root, "valid-link.env");
    symlinkSync(target, validLink);
    const validResult = runBuilder([
      "--env",
      "dev",
      "--secrets-dir",
      secretsDir,
      "--out",
      validLink,
    ]);
    assert.equal(validResult.status, 2);
    assert.equal(lstatSync(validLink).isSymbolicLink(), true);
    assert.equal(readFileSync(target, "utf-8"), "目标原内容\n");

    const brokenLink = join(root, "broken-link.env");
    symlinkSync(join(root, "missing-target.env"), brokenLink);
    const brokenResult = runBuilder([
      "--env",
      "dev",
      "--secrets-dir",
      secretsDir,
      "--out",
      brokenLink,
    ]);
    assert.equal(brokenResult.status, 2);
    assert.equal(lstatSync(brokenLink).isSymbolicLink(), true);
  });
});

test("wrapper 将特殊字符按字面量注入并在成功后清理临时文件", () => {
  withTempDir((root) => {
    const runnerTmp = join(root, "runner-tmp");
    mkdirSync(runnerTmp);
    const sideEffect = join(root, "不应创建");
    const specialValue = `dummy dev value = ; $(touch ${sideEffect})`;
    const secretsDir = writeDevFixture(root, specialValue);
    const marker = join(root, "child-ok");
    const child = [
      "const fs = require('node:fs');",
      "const value = process.env.FOUNDATIONX_REDISX_PASSWORD;",
      "if (!value || !value.includes(' ') || !value.includes('=') || !value.includes(';') || !value.includes('$(')) process.exit(41);",
      "if (process.env.FOUNDATIONX_STALE_PROD !== undefined) process.exit(42);",
      "fs.writeFileSync(process.argv[1], 'received');",
    ].join("");
    const staleSecret = "stale-prod-sentinel";
    const env = {
      ...process.env,
      TMPDIR: runnerTmp,
      FOUNDATIONX_STALE_PROD: staleSecret,
    };
    delete env.FOUNDATIONX_REDISX_PASSWORD;

    const result = runWrapper(
      [
        "--env",
        "dev",
        "--secrets-dir",
        secretsDir,
        "--",
        process.execPath,
        "-e",
        child,
        marker,
      ],
      env,
    );

    assert.equal(result.status, 0, result.stderr);
    assert.equal(readFileSync(marker, "utf-8"), "received");
    assert.equal(existsSync(sideEffect), false);
    assert.deepEqual(readdirSync(runnerTmp), []);
    assert.ok(!result.stdout.includes(specialValue));
    assert.ok(!result.stderr.includes(specialValue));
    assert.ok(!result.stdout.includes(staleSecret));
    assert.ok(!result.stderr.includes(staleSecret));
  });
});

test("wrapper 传播子进程退出码，并在成功与失败路径清理", () => {
  withTempDir((root) => {
    const runnerTmp = join(root, "runner-tmp");
    mkdirSync(runnerTmp);
    const secretsDir = writeDevFixture(root);
    const env = { ...process.env, TMPDIR: runnerTmp };

    const childFailure = runWrapper(
      [
        "--env",
        "dev",
        "--secrets-dir",
        secretsDir,
        "--",
        process.execPath,
        "-e",
        "process.exit(37)",
      ],
      env,
    );
    assert.equal(childFailure.status, 37, childFailure.stderr);
    assert.deepEqual(readdirSync(runnerTmp), []);

    const buildFailure = runWrapper(
      [
        "--env",
        "dev",
        "--secrets-dir",
        join(root, "missing-secrets"),
        "--",
        process.execPath,
        "-e",
        "process.exit(0)",
      ],
      env,
    );
    assert.equal(buildFailure.status, 2);
    assert.deepEqual(readdirSync(runnerTmp), []);
  });
});

test("runner 拒绝非 FOUNDATIONX 键且不启动子进程", () => {
  withTempDir((root) => {
    const envFile = join(root, "malformed.env");
    const marker = join(root, "不应运行");
    writeFileSync(envFile, "PATH=/tmp/attacker\n", { encoding: "utf-8", mode: 0o600 });
    chmodSync(envFile, 0o600);
    const result = spawnSync(
      process.execPath,
      [
        RUNNER,
        "--env-file",
        envFile,
        "--",
        process.execPath,
        "-e",
        "require('node:fs').writeFileSync(process.argv[1], 'ran')",
        marker,
      ],
      { encoding: "utf-8" },
    );
    assert.equal(result.status, 2);
    assert.match(result.stderr, /不允许的键名/u);
    assert.equal(existsSync(marker), false);
  });
});

test("wrapper 拒绝 source 且不修改调用者 shell 选项", () => {
  const script = [
    "set +e +u",
    "set +o pipefail",
    'source "$1"',
    "status=$?",
    '[[ "$status" -eq 2 ]] || exit 10',
    '[[ "$-" != *e* ]] || exit 11',
    '[[ "$-" != *u* ]] || exit 12',
    "[[ $(set -o | awk '$1 == \"pipefail\" { print $2 }') == off ]] || exit 13",
  ].join("\n");
  const result = spawnSync("bash", ["-c", script, "bash", WRAPPER], {
    encoding: "utf-8",
  });
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stderr, /禁止 source/u);
});
