#!/usr/bin/env node
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
  chmodSync,
  cpSync,
  mkdtempSync,
  mkdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..", "..");
const gate = join(repoRoot, "scripts", "quality-gates", "check-ssot-current-state.mjs");
const expectedPackages = [
  ["binancex", "crates/adapters/exchange/binance/Cargo.toml"],
  ["okxx", "crates/adapters/exchange/okx/Cargo.toml"],
  ["clickhousex", "crates/adapters/storage/clickhouse/Cargo.toml"],
  ["kafkax", "crates/adapters/storage/kafka/Cargo.toml"],
  ["natsx", "crates/adapters/storage/nats/Cargo.toml"],
  ["ossx", "crates/adapters/storage/oss/Cargo.toml"],
  ["postgresx", "crates/adapters/storage/postgres/Cargo.toml"],
  ["redisx", "crates/adapters/storage/redis/Cargo.toml"],
  ["taosx", "crates/adapters/storage/taos/Cargo.toml"],
  ["bootstrap", "crates/infra/bootstrap/Cargo.toml"],
  ["configx", "crates/infra/configx/Cargo.toml"],
  ["contracts", "crates/contracts/Cargo.toml"],
  ["evidence", "crates/infra/evidence/Cargo.toml"],
  ["kernel", "crates/kernel/Cargo.toml"],
  ["observex", "crates/infra/observex/Cargo.toml"],
  ["resiliencx", "crates/infra/resiliencx/Cargo.toml"],
  ["schedulex", "crates/infra/schedulex/Cargo.toml"],
  ["contract-testkit", "crates/test-support/contracts/Cargo.toml"],
  ["testkit", "crates/testkit/Cargo.toml"],
  ["transportx", "crates/infra/transport/Cargo.toml"],
  ["canonical", "crates/types/canonical/Cargo.toml"],
  ["decimalx", "crates/types/decimal/Cargo.toml"],
  ["goalctl", "tools/goalctl/Cargo.toml"],
  ["verifyctl", "tools/verifyctl/Cargo.toml"],
  ["domainx", "crates/domainx/Cargo.toml"],
  ["domain_exchange", "crates/domain_exchange/Cargo.toml"],
  ["domain_market", "crates/domain_market/Cargo.toml"],
  ["exchange-binance", "crates/exchange/binance/Cargo.toml"],
  ["exchange-coinbase", "crates/exchange/coinbase/Cargo.toml"],
  ["exchange-coinglass", "crates/exchange/coinglass/Cargo.toml"],
  ["exchange-hyperliquid", "crates/exchange/hyperliquid/Cargo.toml"],
  ["exchange-okx", "crates/exchange/okx/Cargo.toml"],
  ["market_data", "crates/market_data/Cargo.toml"],
];

const tempRoots = [];
process.on("exit", () => {
  for (const root of tempRoots) rmSync(root, { recursive: true, force: true });
});

function makeFixture() {
  const root = mkdtempSync(join(tmpdir(), "ssot-current-state-"));
  tempRoots.push(root);
  writeFileSync(join(root, "Cargo.toml"), "[workspace]\nresolver = \"2\"\n");
  cpSync(join(repoRoot, "AGENTS.md"), join(root, "AGENTS.md"));
  cpSync(join(repoRoot, "CLAUDE.md"), join(root, "CLAUDE.md"));
  cpSync(join(repoRoot, ".agents", "ssot"), join(root, ".agents", "ssot"), { recursive: true });
  for (const file of [
    "crates/infra/configx/src/source.rs",
    "crates/infra/configx/src/layered.rs",
    "crates/infra/configx/src/watch.rs",
    "crates/infra/configx/src/secret.rs",
    "crates/infra/schedulex/src/runner.rs",
    "crates/adapters/exchange/binance/src/lib.rs",
    "crates/adapters/exchange/binance/src/adapter.rs",
    "crates/adapters/exchange/binance/tests/live_server_time.rs",
    "crates/adapters/exchange/okx/src/lib.rs",
    "crates/adapters/exchange/okx/src/adapter.rs",
    "crates/adapters/exchange/okx/tests/live_server_time.rs",
  ]) {
    const target = join(root, file);
    mkdirSync(dirname(target), { recursive: true });
    cpSync(join(repoRoot, file), target);
  }
  for (const file of [
    "workspace-ssot-alignment.md",
    "evidence-ssot-alignment.md",
    "tools-ssot-alignment.md",
    "configx-ssot-alignment.md",
    "schedulex-ssot-alignment.md",
    "adapters-ssot-alignment.md",
  ]) {
    const target = join(root, "docs", "ssot", file);
    mkdirSync(dirname(target), { recursive: true });
    cpSync(join(repoRoot, "docs", "ssot", file), target);
  }
  for (const file of [
    "README.md",
    "crate-inventory.md",
    "review-evidence.md",
    "review-workspace.md",
    "synthesis/go-nogo-synthesis.md",
  ]) {
    const target = join(root, "docs", "report", "2026-07-22", file);
    mkdirSync(dirname(target), { recursive: true });
    cpSync(join(repoRoot, "docs", "report", "2026-07-22", file), target);
  }
  const metadataPath = join(root, "metadata.json");
  writeMetadata(root, metadataPath, expectedPackages);
  const fakeCargo = join(root, "fake-cargo.sh");
  writeFileSync(fakeCargo, "#!/bin/sh\ncat \"$FAKE_METADATA_PATH\"\n");
  chmodSync(fakeCargo, 0o755);
  return { root, metadataPath, fakeCargo };
}

function writeMetadata(root, path, packages) {
  const metadata = {
    packages: packages.map(([name, manifest]) => ({ name, manifest_path: join(root, manifest) })),
  };
  writeFileSync(path, JSON.stringify(metadata));
}

function runFixture(fixture, { json = true } = {}) {
  const args = [gate, "--root", fixture.root];
  if (json) args.push("--json");
  return spawnSync(process.execPath, args, {
    encoding: "utf8",
    env: { ...process.env, CARGO: fixture.fakeCargo, FAKE_METADATA_PATH: fixture.metadataPath },
  });
}

function mutate(file, update) {
  const current = readFileSync(file, "utf8");
  writeFileSync(file, update(current));
}

function expectFailure(name, mutateFixture, expectedCheck) {
  const fixture = makeFixture();
  mutateFixture(fixture);
  const run = runFixture(fixture);
  assert.equal(run.status, 1, `${name}: 应 fail-closed；stdout=${run.stdout}`);
  assert.equal(run.stderr, "", `${name}: JSON 模式不得写追加 stderr`);
  const result = JSON.parse(run.stdout);
  assert.equal(result.ok, false, `${name}: JSON ok 应为 false`);
  const failed = result.checks.filter((item) => !item.ok);
  assert.ok(failed.some((item) => item.id === expectedCheck), `${name}: 未命中 ${expectedCheck}`);
}

{
  const fixture = makeFixture();
  const run = runFixture(fixture);
  assert.equal(run.status, 0, `正常路径失败：${run.stdout}\n${run.stderr}`);
  assert.equal(run.stderr, "", "JSON 模式不得写追加 stderr");
  const result = JSON.parse(run.stdout);
  assert.equal(result.ok, true);
  assert.equal(result.gate, "ssot-current-state");
  assert.equal(result.checks.find((item) => item.id === "cargo-packages")?.ok, true);
}

expectFailure(
  "package 路径漂移",
  ({ root, metadataPath }) => {
    const changed = expectedPackages.map(([name, path]) => [name, name === "goalctl" ? "tools/renamed/Cargo.toml" : path]);
    writeMetadata(root, metadataPath, changed);
  },
  "cargo-packages",
);

expectFailure(
  "evidence 历史入口重新自称 active",
  ({ root }) => {
    writeFileSync(
      join(root, ".agents/ssot/tools/evidence/README.md"),
      "# evidence\n\n当前 active spec：[spec/spec.md](spec/spec.md)\n",
    );
  },
  "evidence-authority",
);

expectFailure(
  "evidence 历史指针恢复本地 SSOT",
  ({ root }) => {
    writeFileSync(
      join(root, ".agents/ssot/tools/evidence/evidence-spec.md"),
      "# evidence\n\n> **SSOT 入口**：[spec/spec.md](spec/spec.md)\n",
    );
  },
  "evidence-authority",
);

expectFailure(
  "evidence snapshot 回退无效 commit",
  ({ root }) => {
    for (const name of ["spec.md", "xhyper-evidence-complete-spec.md"]) {
      mutate(join(root, ".agents/ssot/infra/evidence/spec", name), (text) =>
        text.replace("1b80898e0425cf7dc0f787c0f663154c24c8bb37", "b0934baa"),
      );
    }
  },
  "evidence-authority",
);

expectFailure(
  "goalctl member 陈旧否定",
  ({ root }) => {
    mutate(join(root, ".agents/ssot/tools/goalctl/README.md"), (text) =>
      text.replace("workspace member", "无 `tools/goalctl` workspace member"),
    );
  },
  "current-state-docs",
);

expectFailure(
  "verifyctl crate 陈旧否定",
  ({ root }) => {
    mutate(join(root, ".agents/ssot/tools/verifyctl/README.md"), (text) =>
      `${text}\n> 本仓尚未创建 verifyctl。\n`,
    );
  },
  "current-state-docs",
);

expectFailure(
  "verifyctl release 陈旧否定",
  ({ root }) => {
    writeFileSync(
      join(root, ".agents/ssot/tools/verifyctl/release/release.md"),
      "# Release\n\n> 本仓无 `tools/verifyctl` crate。\n",
    );
  },
  "current-state-docs",
);

expectFailure(
  "tools evidence 恢复第二 active 入口",
  ({ root }) => {
    mutate(join(root, ".agents/ssot/tools/README.md"), (text) =>
      `${text}\n当前 spec：.agents/ssot/tools/evidence/spec/spec.md\n`,
    );
  },
  "current-state-docs",
);

expectFailure(
  "evidence alignment 回退旧权威路径",
  ({ root }) => {
    writeFileSync(
      join(root, "docs/ssot/evidence-ssot-alignment.md"),
      "# evidence\n\n| SSOT 镜像 | `.agents/ssot/tools/evidence/spec/` |\n",
    );
  },
  "current-state-docs",
);

expectFailure(
  "同日 crate inventory 回退旧状态",
  ({ root }) => {
    mutate(join(root, "docs/report/2026-07-22/crate-inventory.md"), (text) =>
      `${text}\nconfigx：L1 内存合同；schedulex：L1 registry；exchange：scaffold + server_time。\n`,
    );
  },
  "current-state-docs",
);

expectFailure(
  "workspace review 恢复最小生产 CLI",
  ({ root }) => {
    mutate(join(root, "docs/report/2026-07-22/review-workspace.md"), (text) =>
      `${text}\nTools 是最小生产 CLI。\n`,
    );
  },
  "current-state-docs",
);

expectFailure(
  "adapters 索引退回 exchange scaffold",
  ({ root }) => {
    mutate(join(root, ".agents/ssot/adapters/README.md"), (text) =>
      text.replace("签名 REST + 公共 WS 解析/注入", "scaffold + mock HTTP + server_time"),
    );
  },
  "current-state-docs",
);

expectFailure(
  "根治理说明退回 configx 仅内存",
  ({ root }) => {
    mutate(join(root, "AGENTS.md"), (text) => `${text}\nconfigx 非多源热更新。\n`);
  },
  "current-state-docs",
);

expectFailure(
  "kernel 设计恢复旧 evidence 物理路径",
  ({ root }) => {
    mutate(join(root, ".agents/ssot/kernel/design/design.md"), (text) =>
      text.replace("path: crates/infra/evidence", "path: tools/evidence"),
    );
  },
  "current-state-docs",
);

expectFailure(
  "source 表面删除后规格不可继续通过",
  ({ root }) => {
    mutate(join(root, "crates/infra/configx/src/source.rs"), (text) =>
      text.replace("pub struct FileSource", "struct RemovedFileSource"),
    );
  },
  "implementation-surfaces",
);

expectFailure(
  "exchange 退回 scaffold/server_time",
  ({ root }) => {
    mutate(join(root, ".agents/ssot/SSOT.md"), (text) =>
      text.replace("签名 REST + 公共 WS", "scaffold + mock HTTP + 只读 server_time"),
    );
  },
  "current-state-docs",
);

expectFailure(
  "configx 退回仅内存声明",
  ({ root }) => {
    mutate(join(root, "docs/ssot/configx-ssot-alignment.md"), (text) =>
      `${text}\n当前仅内存字符串 KV，尚未实现多源。\n`,
    );
  },
  "current-state-docs",
);

expectFailure(
  "schedulex 退回 registry only",
  ({ root }) => {
    mutate(join(root, "docs/ssot/schedulex-ssot-alignment.md"), (text) => `${text}\n状态：registry only。\n`);
  },
  "current-state-docs",
);

expectFailure(
  "dual spec 漂移",
  ({ root }) => {
    const path = join(root, ".agents/ssot/infra/configx/spec/xhyper-configx-complete-spec.md");
    mutate(path, (text) => `${text}\n漂移\n`);
  },
  "dual-specs",
);

expectFailure(
  "dual spec 整对删除",
  ({ root }) => {
    rmSync(join(root, ".agents/ssot/infra/testkitx/spec"), { recursive: true, force: true });
  },
  "dual-specs",
);

expectFailure(
  "新增第 47 对 dual spec — 46 域之外新增未声明域",
  ({ root }) => {
    const directory = join(root, ".agents/ssot/extra/spec");
    mkdirSync(directory, { recursive: true });
    writeFileSync(join(directory, "spec.md"), "# extra\n");
    writeFileSync(join(directory, "xhyper-extra-complete-spec.md"), "# extra\n");
  },
  "dual-specs",
);

expectFailure(
  "dual spec 域替换保持 46 域数量不变但引入未声明域",
  ({ root }) => {
    rmSync(join(root, ".agents/ssot/infra/observex/spec"), { recursive: true, force: true });
    const directory = join(root, ".agents/ssot/replacement/spec");
    mkdirSync(directory, { recursive: true });
    writeFileSync(join(directory, "spec.md"), "# replacement\n");
    writeFileSync(join(directory, "xhyper-replacement-complete-spec.md"), "# replacement\n");
  },
  "dual-specs",
);

{
  const fixture = makeFixture();
  mutate(join(fixture.root, ".agents/ssot/SSOT.md"), (text) =>
    text.replace("签名 REST + 公共 WS", "scaffold + mock HTTP + 只读 server_time"),
  );
  const run = runFixture(fixture, { json: false });
  assert.equal(run.status, 1);
  assert.match(run.stdout, /FAIL/u);
  assert.match(run.stdout, /陈旧声明|缺少当前事实标记/u, "人类模式错误应为中文");
}

process.stdout.write("PASS: check-ssot-current-state 隔离测试全部通过\n");
