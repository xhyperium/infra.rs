#!/usr/bin/env node
/**
 * test-support 生产依赖图门禁。
 *
 * `testkit` 与 `contract-testkit` 只能通过 dev dependency 到达；normal/build 或
 * all-features 激活的生产路径均必须失败。
 */

import { spawnSync } from "node:child_process";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..", "..");

const TEST_SUPPORT_NAMES = new Set(["testkit", "contract-testkit"]);

function packageIndex(metadata) {
  return new Map((metadata.packages || []).map((pkg) => [pkg.id, pkg]));
}

function supportIds(metadata, packages) {
  const ids = new Set();
  for (const pkg of packages.values()) {
    const normalized = String(pkg.manifest_path || "").replaceAll("\\", "/");
    if (TEST_SUPPORT_NAMES.has(pkg.name) || normalized.includes("/crates/test-support/")) {
      ids.add(pkg.id);
    }
  }
  return ids;
}

function productionKind(depKind) {
  if (depKind?.kind === "dev") return null;
  return depKind?.kind === "build" ? "build" : "normal";
}

function inventoryFindings(metadataInputs) {
  const findings = [];
  for (const [label, metadata] of metadataInputs) {
    if (!metadata.resolve || !Array.isArray(metadata.resolve.nodes)) {
      findings.push({
        codes: ["TESTKIT-GRAPH-CONFIG"],
        consumer: "<workspace>",
        testSupportPackage: "<metadata>",
        dependencyKind: "config",
        target: null,
        activation: label,
        featurePath: label === "all-features" ? ["--all-features"] : [],
        dependencyPath: [],
        verdict: "FAIL",
        message: `${label} metadata 缺少完整 resolve.nodes`,
      });
      continue;
    }
    const nodeIds = new Set(metadata.resolve.nodes.map((node) => node.id));
    const missingNodes = (metadata.workspace_members || []).filter((id) => !nodeIds.has(id));
    if (missingNodes.length > 0) {
      findings.push({
        codes: ["TESTKIT-GRAPH-CONFIG"],
        consumer: "<workspace>",
        testSupportPackage: "<metadata>",
        dependencyKind: "config",
        target: null,
        activation: label,
        featurePath: label === "all-features" ? ["--all-features"] : [],
        dependencyPath: [],
        verdict: "FAIL",
        message: `${label} metadata 的 workspace member 缺少 resolve node: ${missingNodes.join(", ")}`,
      });
    }
  }
  for (const name of TEST_SUPPORT_NAMES) {
    const problems = new Set();
    for (const [label, metadata] of metadataInputs) {
      const packages = (metadata.packages || []).filter((pkg) => pkg.name === name);
      if (packages.length !== 1) {
        problems.add(`${label} metadata 中 ${name} 数量为 ${packages.length}，期望 1`);
        continue;
      }
      if (!(metadata.workspace_members || []).includes(packages[0].id)) {
        problems.add(`${label} metadata 中 ${name} 不是 workspace member`);
      }
    }
    if (problems.size > 0) {
      findings.push({
        codes: ["TESTKIT-GRAPH-CONFIG"],
        consumer: "<workspace>",
        testSupportPackage: name,
        dependencyKind: "config",
        target: null,
        activation: "metadata",
        featurePath: [],
        dependencyPath: [],
        verdict: "FAIL",
        message: [...problems].join("；"),
      });
    }
  }
  return findings;
}

function graphFindings(metadata, activation) {
  const packages = packageIndex(metadata);
  const support = supportIds(metadata, packages);
  const nodes = new Map((metadata.resolve?.nodes || []).map((node) => [node.id, node]));
  const findings = [];

  for (const memberId of metadata.workspace_members || []) {
    if (support.has(memberId)) continue;
    const consumer = packages.get(memberId);
    if (!consumer || !nodes.has(memberId)) continue;
    const queue = [{ id: memberId, path: [consumer.name], kind: "normal", target: null }];
    const visited = new Set();
    const emitted = new Set();
    while (queue.length > 0) {
      const current = queue.shift();
      const visitKey = `${current.id}|${current.kind}|${current.target}`;
      if (visited.has(visitKey)) continue;
      visited.add(visitKey);
      const node = nodes.get(current.id);
      if (!node) continue;
      for (const dep of node.deps || []) {
        const kinds = dep.dep_kinds?.length ? dep.dep_kinds : [{ kind: null, target: null }];
        for (const depKind of kinds) {
          const edgeKind = productionKind(depKind);
          if (!edgeKind) continue;
          const kind = current.kind === "build" || edgeKind === "build" ? "build" : "normal";
          const targetCondition = depKind?.target ?? current.target;
          const targetPackage = packages.get(dep.pkg);
          const targetName = targetPackage?.name || dep.pkg;
          const path = [...current.path, targetName];
          if (support.has(dep.pkg)) {
            const findingKey = `${targetName}|${kind}|${targetCondition}`;
            if (emitted.has(findingKey)) continue;
            emitted.add(findingKey);
            findings.push({
              codes: [
                targetName === "testkit" ? "TESTKIT-GRAPH-001" : "TESTKIT-GRAPH-002",
                ...(kind === "build" ? ["TESTKIT-GRAPH-003"] : []),
                "TESTKIT-GRAPH-004",
                ...(activation === "all-features-only" ? ["TESTKIT-GRAPH-005"] : []),
              ],
              consumer: consumer.name,
              testSupportPackage: targetName,
              dependencyKind: kind,
              target: targetCondition,
              activation,
              featurePath: activation === "all-features-only" ? ["--all-features"] : [],
              dependencyPath: path,
              verdict: "FAIL",
              message: `${consumer.name} 的 ${kind} 依赖图包含 test-support package ${targetName}`,
            });
            continue;
          }
          queue.push({ id: dep.pkg, path, kind, target: targetCondition });
        }
      }
    }
  }
  return findings;
}

/**
 * 分析 default 与 all-features 两份 cargo metadata。
 *
 * @param {object} defaultMetadata
 * @param {object} allFeaturesMetadata
 * @returns {{ok: boolean, testSupportPackages: string[], findings: object[]}}
 */
export function inspectTestSupportGraph(defaultMetadata, allFeaturesMetadata) {
  const configFindings = inventoryFindings([
    ["default", defaultMetadata],
    ["all-features", allFeaturesMetadata],
  ]);
  const defaultFindings = graphFindings(defaultMetadata, "default");
  const defaultKeys = new Set(
    defaultFindings.map(
      (finding) =>
        `${finding.consumer}|${finding.testSupportPackage}|${finding.dependencyKind}|${finding.target}`,
    ),
  );
  const allOnly = graphFindings(allFeaturesMetadata, "all-features-only").filter(
    (finding) =>
      !defaultKeys.has(
        `${finding.consumer}|${finding.testSupportPackage}|${finding.dependencyKind}|${finding.target}`,
      ),
  );
  const findings = [...configFindings, ...defaultFindings, ...allOnly];
  const packages = packageIndex(allFeaturesMetadata);
  const names = [...supportIds(allFeaturesMetadata, packages)]
    .map((id) => packages.get(id)?.name || id)
    .sort();
  return {
    ok: findings.length === 0,
    checkedActivations: ["default", "all-features"],
    testSupportPackages: names,
    findings,
  };
}

function loadMetadata(allFeatures) {
  const args = ["metadata", "--locked", "--format-version", "1"];
  if (allFeatures) args.push("--all-features");
  const result = spawnSync("cargo", args, {
    cwd: ROOT,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
  if (result.status !== 0) {
    throw new Error(
      `cargo metadata${allFeatures ? " --all-features" : ""} 失败: ${(result.stderr || result.stdout || "").trim()}`,
    );
  }
  return JSON.parse(result.stdout);
}

function main() {
  const jsonOutput = process.argv.includes("--json");
  if (process.argv.includes("--help") || process.argv.includes("-h")) {
    console.log(
      "用法: node scripts/quality-gates/check-test-support-graph.mjs [--json]\n" +
        "验证 default/all-features 的 normal/build 图不包含 testkit 或 contract-testkit。",
    );
    return 0;
  }
  try {
    const result = inspectTestSupportGraph(loadMetadata(false), loadMetadata(true));
    if (jsonOutput) {
      console.log(JSON.stringify(result, null, 2));
    } else {
      console.log("check-test-support-graph — test-support 生产依赖图门禁");
      console.log(`  test-support: ${result.testSupportPackages.join(", ")}`);
      for (const finding of result.findings) {
        console.log(
          `  ERROR [${finding.codes.join(",")}] ${finding.message}; path=${finding.dependencyPath.join(" -> ")}`,
        );
      }
      console.log(result.ok ? "PASS" : "FAIL");
    }
    return result.ok ? 0 : 1;
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    if (jsonOutput) {
      console.log(
        JSON.stringify(
          {
            ok: false,
            checkedActivations: ["default", "all-features"],
            testSupportPackages: [],
            findings: [
              {
                codes: ["TESTKIT-GRAPH-EXEC"],
                consumer: "<workspace>",
                testSupportPackage: "<metadata>",
                dependencyKind: "config",
                target: null,
                activation: "metadata",
                featurePath: [],
                dependencyPath: [],
                verdict: "FAIL",
                message,
              },
            ],
          },
          null,
          2,
        ),
      );
    } else {
      console.error(`FAIL: ${message}`);
    }
    return 1;
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  process.exitCode = main();
}
