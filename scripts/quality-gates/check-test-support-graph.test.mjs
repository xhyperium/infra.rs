#!/usr/bin/env node

import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import test from "node:test";
import { inspectTestSupportGraph } from "./check-test-support-graph.mjs";

const APP = "app 0.1.0 (path+file:///repo/crates/app)";
const TESTKIT = "testkit 0.1.0 (path+file:///repo/crates/testkit)";
const CONTRACT_TESTKIT =
  "contract-testkit 0.1.0 (path+file:///repo/crates/test-support/contracts)";
const LIB = "lib 0.1.0 (path+file:///repo/crates/lib)";

function metadata(kind) {
  return {
    workspace_members: [APP, TESTKIT, CONTRACT_TESTKIT],
    packages: [
      { id: APP, name: "app", manifest_path: "/repo/crates/app/Cargo.toml" },
      { id: TESTKIT, name: "testkit", manifest_path: "/repo/crates/testkit/Cargo.toml" },
      {
        id: CONTRACT_TESTKIT,
        name: "contract-testkit",
        manifest_path: "/repo/crates/test-support/contracts/Cargo.toml",
      },
    ],
    resolve: {
      nodes: [
        {
          id: APP,
          deps: [
            {
              name: "contract_testkit",
              pkg: CONTRACT_TESTKIT,
              dep_kinds: [{ kind, target: null }],
            },
          ],
        },
        { id: TESTKIT, deps: [] },
        { id: CONTRACT_TESTKIT, deps: [] },
      ],
    },
  };
}

function transitiveMetadata() {
  const value = metadata("dev");
  value.workspace_members.push(LIB);
  value.packages.push({ id: LIB, name: "lib", manifest_path: "/repo/crates/lib/Cargo.toml" });
  value.resolve.nodes[0].deps = [
    { name: "lib", pkg: LIB, dep_kinds: [{ kind: null, target: null }] },
  ];
  value.resolve.nodes.push({
    id: LIB,
    deps: [
      {
        name: "contract_testkit",
        pkg: CONTRACT_TESTKIT,
        dep_kinds: [{ kind: null, target: null }],
      },
    ],
  });
  return value;
}

test("dev-only 引用不进入生产图", () => {
  const dev = metadata("dev");
  const result = inspectTestSupportGraph(dev, dev);
  assert.equal(result.ok, true);
  assert.deepEqual(result.findings, []);
});

test("normal 直连 contract-testkit 必须失败", () => {
  const normal = metadata(null);
  const result = inspectTestSupportGraph(normal, normal);
  assert.equal(result.ok, false);
  assert.equal(result.findings.length, 1);
  assert.equal(result.findings[0].consumer, "app");
  assert.equal(result.findings[0].testSupportPackage, "contract-testkit");
  assert.equal(result.findings[0].dependencyKind, "normal");
  assert.equal(result.findings[0].activation, "default");
  assert.deepEqual(result.findings[0].dependencyPath, ["app", "contract-testkit"]);
});

test("传递 normal 路径必须报告完整依赖链", () => {
  const transitive = transitiveMetadata();
  const result = inspectTestSupportGraph(transitive, transitive);
  assert.equal(result.ok, false);
  assert.equal(result.findings.length, 2);
  const appFinding = result.findings.find((finding) => finding.consumer === "app");
  assert.deepEqual(appFinding.dependencyPath, ["app", "lib", "contract-testkit"]);
});

test("build dependency 必须失败并带结构化规则码", () => {
  const build = metadata("build");
  const result = inspectTestSupportGraph(build, build);
  assert.equal(result.ok, false);
  assert.equal(result.findings[0].dependencyKind, "build");
  assert.ok(result.findings[0].codes.includes("TESTKIT-GRAPH-003"));
});

test("仅 all-features 激活的生产泄漏必须失败", () => {
  const result = inspectTestSupportGraph(metadata("dev"), metadata(null));
  assert.equal(result.ok, false);
  assert.equal(result.findings.length, 1);
  assert.equal(result.findings[0].activation, "all-features-only");
  assert.ok(result.findings[0].codes.includes("TESTKIT-GRAPH-005"));
});

test("target-specific 泄漏保留 target 条件", () => {
  const targetSpecific = metadata(null);
  targetSpecific.resolve.nodes[0].deps[0].dep_kinds[0].target = "cfg(windows)";
  const result = inspectTestSupportGraph(targetSpecific, targetSpecific);
  assert.equal(result.findings[0].target, "cfg(windows)");
});

test("test-support package 之间的 normal 边不误报为生产 consumer", () => {
  const supportGraph = metadata("dev");
  supportGraph.resolve.nodes[1].deps = [
    {
      name: "contract_testkit",
      pkg: CONTRACT_TESTKIT,
      dep_kinds: [{ kind: null, target: null }],
    },
  ];
  const result = inspectTestSupportGraph(supportGraph, supportGraph);
  assert.equal(result.ok, true);
});

test("test-support inventory 缺包必须 fail closed", () => {
  const incomplete = metadata("dev");
  incomplete.workspace_members = incomplete.workspace_members.filter((id) => id !== TESTKIT);
  incomplete.packages = incomplete.packages.filter((pkg) => pkg.id !== TESTKIT);
  incomplete.resolve.nodes = incomplete.resolve.nodes.filter((node) => node.id !== TESTKIT);
  const result = inspectTestSupportGraph(incomplete, incomplete);
  assert.equal(result.ok, false);
  assert.equal(result.findings[0].codes[0], "TESTKIT-GRAPH-CONFIG");
});

test("metadata 缺少 resolve 必须 fail closed", () => {
  const incomplete = metadata("dev");
  incomplete.resolve = null;
  const result = inspectTestSupportGraph(incomplete, incomplete);
  assert.equal(result.ok, false);
  assert.ok(result.findings.some((finding) => finding.codes.includes("TESTKIT-GRAPH-CONFIG")));
});

test("workspace member 缺少 resolve node 必须 fail closed", () => {
  const incomplete = metadata("dev");
  incomplete.resolve.nodes = incomplete.resolve.nodes.filter((node) => node.id !== APP);
  const result = inspectTestSupportGraph(incomplete, incomplete);
  assert.equal(result.ok, false);
  assert.ok(result.findings.some((finding) => finding.codes.includes("TESTKIT-GRAPH-CONFIG")));
});

test("真实仓库 CLI 输出结构化 PASS", () => {
  const script = fileURLToPath(new URL("./check-test-support-graph.mjs", import.meta.url));
  const result = spawnSync(process.execPath, [script, "--json"], {
    cwd: fileURLToPath(new URL("../..", import.meta.url)),
    encoding: "utf8",
    timeout: 60_000,
  });
  assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
  const output = JSON.parse(result.stdout);
  assert.equal(output.ok, true);
  assert.deepEqual(output.findings, []);
});

test("cargo metadata 失败时 --json 仍输出结构化 FAIL", () => {
  const script = fileURLToPath(new URL("./check-test-support-graph.mjs", import.meta.url));
  const result = spawnSync(process.execPath, [script, "--json"], {
    cwd: fileURLToPath(new URL("../..", import.meta.url)),
    encoding: "utf8",
    env: { ...process.env, PATH: "" },
    timeout: 60_000,
  });
  assert.equal(result.status, 1);
  const output = JSON.parse(result.stdout);
  assert.equal(output.ok, false);
  assert.ok(output.findings[0].codes.includes("TESTKIT-GRAPH-EXEC"));
});
