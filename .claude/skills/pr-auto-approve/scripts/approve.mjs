#!/usr/bin/env node

import { execFileSync } from "child_process";

const TOKEN = process.env.LIUKONGQIANG5_APPROVE_TOKEN || "";
const EXPECTED_LOGIN = process.env.PR_AUTO_APPROVE_EXPECTED_LOGIN || "liukongqiang5";
const API_BASE = process.env.PR_AUTO_APPROVE_API || "https://api.github.com";
const FALLBACK_REPO = "xhyperium/infra.rs";

function resolveRepo() {
  if (process.env.PR_AUTO_APPROVE_REPO) {
    return process.env.PR_AUTO_APPROVE_REPO;
  }

  try {
    const result = execFileSync(
      "gh",
      ["repo", "view", "--json", "nameWithOwner", "--jq", ".nameWithOwner"],
      { encoding: "utf-8", stdio: ["pipe", "pipe", "pipe"] }
    ).trim();
    if (result) return result;
  } catch {
    // fall through
  }

  try {
    const remote = execFileSync("git", ["remote", "get-url", "origin"], {
      encoding: "utf-8",
      stdio: ["pipe", "pipe", "pipe"],
    }).trim();
    const match = remote.match(/github\.com[:\/]([^\/]+)\/([^\/]+?)(?:\.git)?$/);
    if (match) {
      return `${match[1]}/${match[2]}`;
    }
  } catch {
    // fall through
  }

  return FALLBACK_REPO;
}

async function api(method, path, body = null) {
  const opts = {
    method,
    headers: {
      Authorization: `Bearer ${TOKEN}`,
      Accept: "application/vnd.github+json",
      "X-GitHub-Api-Version": "2022-11-28",
      "Content-Type": "application/json",
    },
  };
  if (body) {
    opts.body = JSON.stringify(body);
  }
  const res = await fetch(`${API_BASE}${path}`, opts);
  const data = res.headers.get("content-length") === "0" ? null : await res.json();
  return { status: res.status, ok: res.ok, headers: res.headers, data };
}

async function main() {
  const prNumber = process.argv[2];
  if (!prNumber) {
    console.error("Usage: node .claude/skills/pr-auto-approve/scripts/approve.mjs <pr-number> [review-body]");
    process.exit(1);
  }

  if (!TOKEN) {
    console.error("Error: LIUKONGQIANG5_APPROVE_TOKEN is not set");
    process.exit(2);
  }

  const userRes = await api("GET", "/user");
  if (!userRes.ok) {
    console.error("Error: Failed to verify token identity");
    process.exit(2);
  }

  const login = userRes.data.login;
  if (login !== EXPECTED_LOGIN) {
    console.error(`Error: Token login "${login}" does not match expected login "${EXPECTED_LOGIN}"`);
    process.exit(2);
  }

  const repo = resolveRepo();

  const prRes = await api("GET", `/repos/${repo}/pulls/${prNumber}`);
  if (!prRes.ok) {
    console.error(`Error: Failed to fetch PR #${prNumber}: ${prRes.status}`);
    process.exit(3);
  }

  const pr = prRes.data;

  if (pr.state !== "open") {
    console.error(`Error: PR #${prNumber} is not open (state: ${pr.state})`);
    process.exit(4);
  }

  if (pr.user && pr.user.login === login) {
    console.error(`Error: PR #${prNumber} author "${pr.user.login}" matches token login "${login}" (self-approve not allowed)`);
    process.exit(4);
  }

  const reviewsRes = await api("GET", `/repos/${repo}/pulls/${prNumber}/reviews`);
  if (reviewsRes.ok && Array.isArray(reviewsRes.data)) {
    const alreadyApproved = reviewsRes.data.some(
      (r) => r.user && r.user.login === login && r.state === "APPROVED"
    );
    if (alreadyApproved) {
      process.exit(0);
    }
  }

  const reviewBody = process.argv[3] || "";
  const approveBody = { event: "APPROVE" };
  if (reviewBody) {
    approveBody.body = reviewBody;
  }

  const approveRes = await api("POST", `/repos/${repo}/pulls/${prNumber}/reviews`, approveBody);
  if (!approveRes.ok) {
    console.error(`Error: Failed to approve PR #${prNumber}: ${approveRes.status}`);
    if (approveRes.data) {
      console.error(JSON.stringify(approveRes.data, null, 2));
    }
    process.exit(3);
  }

  process.exit(0);
}

main();
