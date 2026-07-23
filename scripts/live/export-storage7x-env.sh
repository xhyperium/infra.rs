#!/usr/bin/env bash
# 为单个 live 子进程注入 dev/prod STORAGE7X_*；不修改调用者 shell 环境。
# 覆盖 7 个 storage 域：clickhouse / kafka / nats / oss / postgres / redis / taos。
if [[ "${BASH_SOURCE[0]}" != "$0" ]]; then
  echo "错误: 禁止 source；请直接执行本脚本并传入子进程命令" >&2
  return 2
fi

set -euo pipefail

usage() {
  cat <<'EOF'
用法:
  scripts/live/export-storage7x-env.sh --env dev|prod [选项] -- <command> [args...]

选项:
  --secrets-dir <dir>  dev.md / prod.md 所在目录
  --dry-run            仅打印将要注入的环境变量键名，不写文件、不执行子进程命令

示例:
  scripts/live/export-storage7x-env.sh --env dev -- \
    cargo test -p redisx --test live_kv -- --ignored
EOF
}

env_name="dev"
secrets_dir=""
dry_run=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --env)
      [[ $# -ge 2 ]] || { echo "错误: --env 缺少参数" >&2; exit 2; }
      env_name=$2
      shift 2
      ;;
    --secrets-dir)
      [[ $# -ge 2 ]] || { echo "错误: --secrets-dir 缺少参数" >&2; exit 2; }
      secrets_dir=$2
      shift 2
      ;;
    --dry-run)
      dry_run=1
      shift
      ;;
    --)
      shift
      break
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "错误: 未知参数: $1" >&2
      exit 2
      ;;
  esac
done

if [[ "$env_name" != "dev" && "$env_name" != "prod" ]]; then
  echo "错误: 仅允许 --env dev 或 --env prod；其他取值已拒绝" >&2
  exit 2
fi

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)

if [[ "$dry_run" -eq 1 ]]; then
  dry_args=(--env "$env_name" --dry-run)
  if [[ -n "$secrets_dir" ]]; then
    dry_args+=(--secrets-dir "$secrets_dir")
  fi
  exec node "$script_dir/build-storage7x-env.mjs" "${dry_args[@]}"
fi

if [[ $# -eq 0 ]]; then
  echo "错误: -- 后必须提供子进程命令（或改用 --dry-run）" >&2
  exit 2
fi

umask 077
tmp_dir=""
env_file=""

cleanup() {
  local status=$?
  trap - EXIT HUP INT TERM
  if [[ -n "$env_file" ]]; then
    rm -f -- "$env_file"
  fi
  if [[ -n "$tmp_dir" ]]; then
    rmdir -- "$tmp_dir" 2>/dev/null || true
  fi
  return "$status"
}

trap cleanup EXIT
trap 'exit 129' HUP
trap 'exit 130' INT
trap 'exit 143' TERM

tmp_root=${TMPDIR:-/tmp}
tmp_dir=$(mktemp -d "${tmp_root%/}/storage7x-live.XXXXXXXXXX")
env_file="$tmp_dir/storage7x.env"

builder_args=(--env "$env_name" --out "$env_file")
if [[ -n "$secrets_dir" ]]; then
  builder_args+=(--secrets-dir "$secrets_dir")
fi

node "$script_dir/build-storage7x-env.mjs" "${builder_args[@]}"
set +e
node "$script_dir/run-storage7x-command.mjs" --env-file "$env_file" -- "$@"
status=$?
set -e
exit "$status"
