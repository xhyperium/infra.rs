#!/usr/bin/env node
/**
 * 校验 current-state SSOT 与 Cargo/文档事实一致。
 *
 * 用法：
 *   node scripts/quality-gates/check-ssot-current-state.mjs [--json] [--root <path>]
 */
import { spawnSync } from "node:child_process";
import {
  existsSync,
  readFileSync,
  readdirSync,
  realpathSync,
  statSync,
} from "node:fs";
import { dirname, isAbsolute, join, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

const scriptRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..", "..");
const expectedEvidenceSnapshot = "1b80898e0425cf7dc0f787c0f663154c24c8bb37";

const expectedPackages = new Map([
  ["binancex", "crates/adapters/exchange/binance/Cargo.toml"],
  ["okxx", "crates/adapters/exchange/okx/Cargo.toml"],
  ["clickhousex", "crates/adapters/storage/clickhouse/Cargo.toml"],
  ["kafkax", "crates/adapters/storage/kafka/Cargo.toml"],
  ["natsx", "crates/adapters/storage/nats/Cargo.toml"],
  ["ossx", "crates/adapters/storage/oss/Cargo.toml"],
  ["postgresx", "crates/adapters/storage/postgres/Cargo.toml"],
  ["redisx", "crates/adapters/storage/redis/Cargo.toml"],
  ["taosx", "crates/adapters/storage/taos/Cargo.toml"],
  ["bootstrap", "crates/bootstrap/Cargo.toml"],
  ["configx", "crates/configx/Cargo.toml"],
  ["contracts", "crates/contracts/Cargo.toml"],
  ["evidence", "crates/evidence/Cargo.toml"],
  ["kernel", "crates/kernel/Cargo.toml"],
  ["observex", "crates/observex/Cargo.toml"],
  ["resiliencx", "crates/resiliencx/Cargo.toml"],
  ["schedulex", "crates/schedulex/Cargo.toml"],
  ["contract-testkit", "crates/test-support/contracts/Cargo.toml"],
  ["testkit", "crates/testkit/Cargo.toml"],
  ["transportx", "crates/transport/Cargo.toml"],
  ["canonical", "crates/types/canonical/Cargo.toml"],
  ["decimalx", "crates/types/decimal/Cargo.toml"],
  ["goalctl", "tools/goalctl/Cargo.toml"],
  ["verifyctl", "tools/verifyctl/Cargo.toml"],
]);

const expectedDualSpecDirs = [
  "adapters/exchange/binance/spec",
  "adapters/exchange/okx/spec",
  "adapters/storage/clickhouse/spec",
  "adapters/storage/kafka/spec",
  "adapters/storage/nats/spec",
  "adapters/storage/oss/spec",
  "adapters/storage/postgres/spec",
  "adapters/storage/redis/spec",
  "adapters/storage/taos/spec",
  "bootstrap/spec",
  "configx/spec",
  "contracts/spec",
  "evidence/spec",
  "gate/spec",
  "kernel/spec",
  "observex/spec",
  "resiliencx/spec",
  "schedulex/spec",
  "testkit/spec",
  "testkitx/spec",
  "tools/goalctl/spec",
  "tools/verifyctl/spec",
  "tools/xtask/spec",
  "transport/spec",
  "types/canonical/spec",
  "types/decimal/spec",
];

function parseArgs(argv) {
  let json = false;
  let root = scriptRoot;
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--json") {
      json = true;
    } else if (arg === "--root") {
      index += 1;
      if (!argv[index]) throw new Error("--root 缺少路径参数");
      root = resolve(argv[index]);
    } else {
      throw new Error(`未知参数：${arg}`);
    }
  }
  return { json, root };
}

function pathInside(root, path) {
  const rel = relative(root, path);
  return rel !== ".." && !rel.startsWith(`..${sep}`) && !isAbsolute(rel);
}

function normalizedManifestPath(root, manifestPath) {
  const absolute = resolve(manifestPath);
  if (!pathInside(root, absolute)) return null;
  return relative(root, absolute).split(sep).join("/");
}

function read(root, rel) {
  return readFileSync(join(root, rel), "utf8");
}

function collectDualSpecDirs(root) {
  const ssotRoot = join(root, ".agents", "ssot");
  const found = [];
  const visit = (directory) => {
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      const path = join(directory, entry.name);
      if (entry.isDirectory()) {
        if (entry.name === "spec") found.push(path);
        else visit(path);
      }
    }
  };
  visit(ssotRoot);
  return found.filter((directory) => {
    const names = readdirSync(directory);
    return names.includes("spec.md") || names.some((name) => /^xhyper-.*-complete-spec\.md$/u.test(name));
  });
}

function runGate(root) {
  const checks = [];
  const check = (id, ok, message) => checks.push({ id, ok: Boolean(ok), message });

  check("root", existsSync(join(root, "Cargo.toml")), "仓库根 Cargo.toml 存在性");
  if (!checks.at(-1).ok) return checks;

  const cargo = process.env.CARGO || "cargo";
  const metadataRun = spawnSync(
    cargo,
    ["metadata", "--no-deps", "--format-version", "1", "--manifest-path", join(root, "Cargo.toml")],
    { cwd: root, encoding: "utf8" },
  );
  if (metadataRun.error || metadataRun.status !== 0) {
    const detail = metadataRun.error?.message || metadataRun.stderr?.trim() || `退出码 ${metadataRun.status}`;
    check("cargo-metadata", false, `cargo metadata 执行失败：${detail}`);
    return checks;
  }

  let metadata;
  try {
    metadata = JSON.parse(metadataRun.stdout);
  } catch (error) {
    check("cargo-metadata", false, `cargo metadata 输出不是合法 JSON：${error.message}`);
    return checks;
  }
  if (!Array.isArray(metadata.packages)) {
    check("cargo-metadata", false, "cargo metadata 缺少 packages 数组");
    return checks;
  }

  const actualPackages = new Map();
  const metadataErrors = [];
  for (const pkg of metadata.packages) {
    if (typeof pkg?.name !== "string" || typeof pkg?.manifest_path !== "string") {
      metadataErrors.push("存在缺少 name/manifest_path 的 package");
      continue;
    }
    if (actualPackages.has(pkg.name)) metadataErrors.push(`package 名重复：${pkg.name}`);
    const manifest = normalizedManifestPath(root, pkg.manifest_path);
    if (manifest === null) metadataErrors.push(`manifest 越出仓库根：${pkg.name}`);
    actualPackages.set(pkg.name, manifest);
  }
  if (actualPackages.size !== expectedPackages.size) {
    metadataErrors.push(`package 数量应为 ${expectedPackages.size}，实际为 ${actualPackages.size}`);
  }
  for (const [name, expectedPath] of expectedPackages) {
    if (!actualPackages.has(name)) metadataErrors.push(`缺少 package：${name}`);
    else if (actualPackages.get(name) !== expectedPath) {
      metadataErrors.push(`package ${name} 路径应为 ${expectedPath}，实际为 ${actualPackages.get(name)}`);
    }
  }
  for (const name of actualPackages.keys()) {
    if (!expectedPackages.has(name)) metadataErrors.push(`出现未冻结 package：${name}`);
  }
  check(
    "cargo-packages",
    metadataErrors.length === 0,
    metadataErrors.length === 0 ? "24 个 package 名称与路径一致" : metadataErrors.join("；"),
  );

  const canonicalEvidence = ".agents/ssot/evidence/spec/spec.md";
  const canonicalEvidenceMirror = ".agents/ssot/evidence/spec/xhyper-evidence-complete-spec.md";
  const evidenceRedirect = ".agents/ssot/tools/evidence/README.md";
  const evidenceLegacyPointer = ".agents/ssot/tools/evidence/evidence-spec.md";
  const evidenceErrors = [];
  for (const path of [canonicalEvidence, canonicalEvidenceMirror, evidenceRedirect, evidenceLegacyPointer]) {
    if (!existsSync(join(root, path))) evidenceErrors.push(`缺少 ${path}`);
  }
  if (evidenceErrors.length === 0) {
    if (!readFileSync(join(root, canonicalEvidence)).equals(readFileSync(join(root, canonicalEvidenceMirror)))) {
      evidenceErrors.push("canonical evidence 双镜像不一致");
    }
    const redirect = read(root, evidenceRedirect);
    if (!redirect.includes("历史重定向")) evidenceErrors.push("tools/evidence 未标记为历史重定向");
    if (!redirect.includes(canonicalEvidence)) evidenceErrors.push("tools/evidence 未指向 canonical evidence spec");
    if (/\]\(spec\//u.test(redirect) || redirect.includes(".agents/ssot/tools/evidence/spec/spec.md")) {
      evidenceErrors.push("tools/evidence 仍声明本地 active spec");
    }
    const legacyPointer = read(root, evidenceLegacyPointer);
    if (!legacyPointer.includes(canonicalEvidence) || !legacyPointer.includes("不是第二个 active spec")) {
      evidenceErrors.push("tools/evidence 历史指针未唯一指向 canonical spec");
    }
    if (/SSOT 入口[^\n]*\]\(spec\//u.test(legacyPointer)) {
      evidenceErrors.push("tools/evidence 历史指针仍自称本地 SSOT 入口");
    }
    const canonical = read(root, canonicalEvidence);
    if (!canonical.includes(expectedEvidenceSnapshot)) {
      evidenceErrors.push(`evidence implementation snapshot 未冻结为 ${expectedEvidenceSnapshot}`);
    }
  }
  check(
    "evidence-authority",
    evidenceErrors.length === 0,
    evidenceErrors.length === 0 ? "evidence canonical 入口唯一" : evidenceErrors.join("；"),
  );

  const currentStateRules = [
    {
      path: ".agents/ssot/tools/goalctl/README.md",
      required: ["workspace member", "`goalctl`", "`0.2.0`", "最小"],
      forbidden: [/无\s*`?tools\/goalctl`?\s*workspace member/u],
    },
    {
      path: ".agents/ssot/tools/verifyctl/README.md",
      required: ["workspace member", "`verifyctl`", "`0.1.0`", "非生产 verifier"],
      forbidden: [/本仓尚未创建/u, /本仓无 member/u, /以下应失败直至落地/u],
    },
    {
      path: ".agents/ssot/tools/verifyctl/release/release.md",
      required: ["`verifyctl`", "`0.1.0`", "BLOCKED", "Production Ready"],
      forbidden: [/本仓无\s*`?tools\/verifyctl`?\s*crate/u],
    },
    {
      path: ".agents/ssot/tools/README.md",
      required: [".agents/ssot/evidence/", "历史重定向", "非生产 verifier"],
      forbidden: [/\.agents\/ssot\/tools\/evidence\/spec\/spec\.md/u, /最小生产 CLI/u],
    },
    {
      path: ".agents/ssot/SSOT.md",
      required: [".agents/ssot/evidence/", "签名 REST", "公共 WS", "NO-GO"],
      forbidden: [/exchange[^\n]*(?:scaffold|mock HTTP)[^\n]*server.?time/iu],
    },
    {
      path: ".agents/ssot/AGENTS.md",
      required: ["签名 REST", "公共 WS", "NO-GO"],
      forbidden: [/exchange[^\n]*(?:scaffold|mock HTTP)[^\n]*server.?time/iu],
    },
    {
      path: ".agents/ssot/adapters/README.md",
      required: ["签名 REST", "公共 WS", "交易 **NO-GO**"],
      forbidden: [/(?:binance|okx)[^\n]*(?:scaffold|mock HTTP)[^\n]*server.?time/iu],
    },
    {
      path: "AGENTS.md",
      required: [".agents/ssot/evidence/", "历史重定向", "Memory/Env/File source", "JobRunner::tick", "交易 **NO-GO**", "verifyctl 非生产 verifier"],
      forbidden: [/tools\/.*含 evidence/u, /非多源热更新/u, /registry only/iu, /最小生产 CLI members/u],
    },
    {
      path: "CLAUDE.md",
      required: [".agents/ssot/evidence/", "历史重定向", "本地多源", "JobRunner::tick", "交易 **NO-GO**", "verifyctl 非生产 verifier"],
      forbidden: [/tools\/\{evidence,/u, /exchange 生产默认 REST\+WS/iu],
    },
    {
      path: ".agents/ssot/kernel/design/design.md",
      required: ["path: crates/evidence", ".agents/ssot/evidence/"],
      forbidden: [/path: tools\/evidence/u],
    },
    {
      path: ".agents/ssot/kernel/design/DESIGN-KERNEL-002.md",
      required: ["path: crates/evidence", ".agents/ssot/evidence/"],
      forbidden: [/path: tools\/evidence/u],
    },
    {
      path: ".agents/ssot/tools/xtask/spec/spec.md",
      required: ["`crates/evidence/`", "gate 为 OOS"],
      forbidden: [/`tools\/evidence\/`/u, /`crates\/infra\/gate\/`/u],
    },
    {
      path: "docs/ssot/workspace-ssot-alignment.md",
      required: ["24", "MemorySource", "JobRunner::tick", "交易 NO-GO"],
      forbidden: [/非多源热更新/u, /registry only/iu],
    },
    {
      path: "docs/ssot/evidence-ssot-alignment.md",
      required: [".agents/ssot/evidence/spec/spec.md", "历史入口", "不得维护第二份 active spec"],
      forbidden: [/SSOT 镜像\s*\|\s*`\.agents\/ssot\/tools\/evidence/iu],
    },
    {
      path: "docs/ssot/tools-ssot-alignment.md",
      required: [".agents/ssot/evidence/", "历史重定向", "非生产 verifier"],
      forbidden: [/SSOT 路径\s*\|[^\n]*\.agents\/ssot\/tools\/evidence/iu, /最小生产 CLI/iu],
    },
    {
      path: "docs/ssot/configx-ssot-alignment.md",
      required: ["MemorySource", "EnvSource", "FileSource", "LayeredConfig", "ConfigWatch", "SecretString"],
      forbidden: [/尚未实现多源/u, /多源加载\s*\/\s*热更新\s*NOT IMPLEMENTED/iu, /仅内存字符串 KV/u],
    },
    {
      path: "docs/ssot/schedulex-ssot-alignment.md",
      required: ["JobRunner::tick", "宿主驱动", "分布式调度"],
      forbidden: [/registry only/iu],
    },
    {
      path: "docs/ssot/adapters-ssot-alignment.md",
      required: ["签名 REST", "公共 WS", "交易 NO-GO"],
      forbidden: [/exchange[^\n]*(?:scaffold|mock HTTP)[^\n]*server.?time/iu],
    },
    {
      path: "docs/report/2026-07-22/README.md",
      required: ["历史快照", "不是 current-state SSOT", "crate-inventory.md", "review-workspace.md"],
      forbidden: [],
    },
    {
      path: "docs/report/2026-07-22/crate-inventory.md",
      required: ["本地多源", "JobRunner::tick", ".agents/ssot/evidence/", "签名 REST + 公共 WS", "交易 NO-GO"],
      forbidden: [/L1 内存合同/u, /L1 registry/u, /scaffold\s*\+\s*server_time/iu, /evidence SSOT 物理位置[^\n]*tools\/evidence/iu],
    },
    {
      path: "docs/report/2026-07-22/review-evidence.md",
      required: [".agents/ssot/evidence/spec/spec.md", "canonical", "历史重定向"],
      forbidden: [/\| SSOT \| `\.agents\/ssot\/tools\/evidence\/?`/u],
    },
    {
      path: "docs/report/2026-07-22/review-workspace.md",
      required: ["最小 CLI", "verifyctl **非生产 verifier**"],
      forbidden: [/最小生产 CLI/u],
    },
    {
      path: "docs/report/2026-07-22/synthesis/go-nogo-synthesis.md",
      required: ["签名 REST + 公共 WS", "NO-GO 交易", "私有 WS", "重连"],
      forbidden: [/scaffold\s*\+\s*(?:公共\s*)?server_time/iu],
    },
  ];
  const currentStateErrors = [];
  for (const rule of currentStateRules) {
    if (!existsSync(join(root, rule.path))) {
      currentStateErrors.push(`缺少 ${rule.path}`);
      continue;
    }
    const content = read(root, rule.path);
    for (const marker of rule.required) {
      if (!content.includes(marker)) currentStateErrors.push(`${rule.path} 缺少当前事实标记：${marker}`);
    }
    for (const pattern of rule.forbidden) {
      if (pattern.test(content)) currentStateErrors.push(`${rule.path} 命中陈旧声明：${pattern.source}`);
    }
  }
  check(
    "current-state-docs",
    currentStateErrors.length === 0,
    currentStateErrors.length === 0 ? "current-state 文档无已知陈旧否定" : currentStateErrors.join("；"),
  );

  const domainMarkers = [
    {
      path: ".agents/ssot/configx/spec/spec.md",
      markers: ["MemorySource", "EnvSource", "FileSource", "LayeredConfig", "ConfigWatch", "SecretString", "远端配置中心"],
    },
    {
      path: ".agents/ssot/schedulex/spec/spec.md",
      markers: ["JobRunner::tick", "宿主驱动", "分布式调度"],
    },
    {
      path: ".agents/ssot/adapters/exchange/binance/spec/spec.md",
      markers: ["签名 REST", "公共 WS", "NO-GO", "精度", "限流", "时钟", "私有 WS", "重连"],
    },
    {
      path: ".agents/ssot/adapters/exchange/okx/spec/spec.md",
      markers: ["签名 REST", "公共 WS", "NO-GO", "精度", "限流", "时钟", "私有 WS", "重连"],
    },
  ];
  const domainErrors = [];
  for (const { path, markers } of domainMarkers) {
    if (!existsSync(join(root, path))) {
      domainErrors.push(`缺少 ${path}`);
      continue;
    }
    const content = read(root, path);
    for (const marker of markers) {
      if (!content.includes(marker)) domainErrors.push(`${path} 缺少边界标记：${marker}`);
    }
  }
  check(
    "domain-boundaries",
    domainErrors.length === 0,
    domainErrors.length === 0 ? "四域实现与 OPEN/NO-GO 边界已显式" : domainErrors.join("；"),
  );

  const implementationSurfaceRules = [
    ["crates/configx/src/source.rs", ["pub struct MemorySource", "pub struct EnvSource", "pub struct FileSource"]],
    ["crates/configx/src/layered.rs", ["pub struct LayeredConfig", "#[cfg(test)]"]],
    ["crates/configx/src/watch.rs", ["pub struct ConfigWatch", "#[cfg(test)]"]],
    ["crates/configx/src/secret.rs", ["pub struct SecretString", "#[cfg(test)]"]],
    ["crates/schedulex/src/runner.rs", ["pub struct JobRunner", "pub fn tick", "#[cfg(test)]"]],
    ["crates/adapters/exchange/binance/src/lib.rs", ["BinanceAdapter::with_api_key", "BinanceAdapter::with_ws"]],
    ["crates/adapters/exchange/binance/src/adapter.rs", ["async fn signed_place_cancel_query_assert_path_headers_and_parse"]],
    ["crates/adapters/exchange/binance/tests/live_server_time.rs", ["#[ignore", "live_binance_server_time"]],
    ["crates/adapters/exchange/okx/src/lib.rs", ["OkxAdapter::with_api_key", "OkxAdapter::with_ws"]],
    ["crates/adapters/exchange/okx/src/adapter.rs", ["async fn signed_place_cancel_query_protocol"]],
    ["crates/adapters/exchange/okx/tests/live_server_time.rs", ["#[ignore", "live_okx_server_time"]],
  ];
  const implementationErrors = [];
  for (const [path, markers] of implementationSurfaceRules) {
    if (!existsSync(join(root, path))) {
      implementationErrors.push(`缺少实现/测试证据：${path}`);
      continue;
    }
    const content = read(root, path);
    for (const marker of markers) {
      if (!content.includes(marker)) implementationErrors.push(`${path} 缺少实现/测试标记：${marker}`);
    }
  }
  check(
    "implementation-surfaces",
    implementationErrors.length === 0,
    implementationErrors.length === 0 ? "四域 source/test 表面与 current-state 声明一致" : implementationErrors.join("；"),
  );

  const dualSpecErrors = [];
  let dualSpecCount = 0;
  try {
    const dualSpecDirs = collectDualSpecDirs(root);
    const actualDirs = new Set(
      dualSpecDirs.map((directory) =>
        relative(join(root, ".agents", "ssot"), directory).split(sep).join("/"),
      ),
    );
    for (const expected of expectedDualSpecDirs) {
      if (!actualDirs.has(expected)) dualSpecErrors.push(`缺少冻结的 dual spec 域：${expected}`);
    }
    for (const actual of actualDirs) {
      if (!expectedDualSpecDirs.includes(actual)) dualSpecErrors.push(`出现未声明的 dual spec 域：${actual}`);
    }
    for (const directory of dualSpecDirs) {
      const names = readdirSync(directory);
      const spec = join(directory, "spec.md");
      const mirrors = names.filter((name) => /^xhyper-.*-complete-spec\.md$/u.test(name));
      const rel = relative(root, directory).split(sep).join("/");
      if (!existsSync(spec)) {
        dualSpecErrors.push(`${rel} 缺少 spec.md`);
        continue;
      }
      if (mirrors.length !== 1) {
        dualSpecErrors.push(`${rel} complete spec 数量应为 1，实际为 ${mirrors.length}`);
        continue;
      }
      if (!readFileSync(spec).equals(readFileSync(join(directory, mirrors[0])))) {
        dualSpecErrors.push(`${rel}/spec.md 与 ${mirrors[0]} 不同构`);
      }
      dualSpecCount += 1;
    }
  } catch (error) {
    dualSpecErrors.push(`扫描双镜像失败：${error.message}`);
  }
  if (dualSpecCount !== expectedDualSpecDirs.length) {
    dualSpecErrors.push(`active spec/complete spec 对数应为 ${expectedDualSpecDirs.length}，实际为 ${dualSpecCount}`);
  }
  check(
    "dual-specs",
    dualSpecErrors.length === 0,
    dualSpecErrors.length === 0 ? `${dualSpecCount} 对 active spec/complete spec 同构` : dualSpecErrors.join("；"),
  );

  return checks;
}

function output(result, json) {
  if (json) {
    process.stdout.write(`${JSON.stringify(result)}\n`);
    return;
  }
  for (const item of result.checks) {
    process.stdout.write(`${item.ok ? "PASS" : "FAIL"}: ${item.id} — ${item.message}\n`);
  }
  process.stdout.write(`结果: ${result.ok ? "PASS" : "FAIL"}\n`);
}

let options = { json: process.argv.includes("--json"), root: scriptRoot };
let result;
try {
  options = parseArgs(process.argv.slice(2));
  const root = existsSync(options.root) ? realpathSync(options.root) : options.root;
  if (!existsSync(root) || !statSync(root).isDirectory()) throw new Error(`仓库根不存在或不是目录：${root}`);
  const checks = runGate(root);
  result = { gate: "ssot-current-state", ok: checks.every((item) => item.ok), checks };
} catch (error) {
  result = {
    gate: "ssot-current-state",
    ok: false,
    checks: [{ id: "internal", ok: false, message: `门禁执行失败：${error.message}` }],
  };
}
output(result, options.json);
process.exit(result.ok ? 0 : 1);
