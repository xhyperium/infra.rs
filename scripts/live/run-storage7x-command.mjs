#!/usr/bin/env node
/**
 * 读取 builder 生成的私有 env 文件，并为单个子进程注入 STORAGE7X_*。
 * 本脚本不使用 shell 求值，不打印变量值，也不负责持久化凭据文件。
 */

import {
  closeSync,
  constants,
  fstatSync,
  openSync,
  readFileSync,
} from "fs";
import { spawnSync } from "child_process";

function fail(message) {
  throw new Error(message);
}

function parseArgs(argv) {
  if (argv[0] !== "--env-file" || !argv[1]) {
    fail("用法: run-storage7x-command.mjs --env-file <path> -- <command> [args...]");
  }
  if (argv[2] !== "--" || !argv[3]) {
    fail("必须使用 -- 分隔 env 文件参数与子进程命令");
  }
  return { envFile: argv[1], command: argv[3], args: argv.slice(4) };
}

function parseEnvironment(filePath) {
  let fd;
  let text;
  try {
    fd = openSync(filePath, constants.O_RDONLY | constants.O_NOFOLLOW);
    const stat = fstatSync(fd);
    if (!stat.isFile()) {
      fail("env 文件必须是普通文件，不能是符号链接");
    }
    if ((stat.mode & 0o777) !== 0o600) {
      fail("env 文件权限必须恰好为 0600");
    }
    text = readFileSync(fd, "utf-8");
  } finally {
    if (fd !== undefined) closeSync(fd);
  }
  if (text.includes("\r") || text.includes("\0")) {
    fail("env 文件包含禁止的控制字符");
  }

  const parsed = {};
  const lines = text.split("\n");
  for (let index = 0; index < lines.length; index++) {
    const line = lines[index];
    if (line.length === 0) continue;
    const separator = line.indexOf("=");
    if (separator <= 0) fail(`env 文件第 ${index + 1} 行格式无效`);
    const key = line.slice(0, separator);
    const value = line.slice(separator + 1);
    if (!/^STORAGE7X_[A-Z0-9_]+$/u.test(key)) {
      fail(`env 文件第 ${index + 1} 行包含不允许的键名`);
    }
    if (Object.hasOwn(parsed, key)) {
      fail(`env 文件包含重复键: ${key}`);
    }
    parsed[key] = value;
  }
  return parsed;
}

function main() {
  const opts = parseArgs(process.argv.slice(2));
  const injected = parseEnvironment(opts.envFile);
  const childEnv = Object.fromEntries(
    Object.entries(process.env).filter(([key]) => !key.startsWith("STORAGE7X_")),
  );
  const result = spawnSync(opts.command, opts.args, {
    env: { ...childEnv, ...injected },
    shell: false,
    stdio: "inherit",
  });
  if (result.error) throw result.error;
  if (result.signal) {
    process.kill(process.pid, result.signal);
    return;
  }
  process.exitCode = result.status ?? 1;
}

try {
  main();
} catch (error) {
  console.error(`错误: ${error instanceof Error ? error.message : "未知错误"}`);
  process.exitCode = 2;
}
