#!/usr/bin/env bash
# 为单个 live 子进程注入 dev FOUNDATIONX_*；不修改调用者 shell 环境。
if [[ "${BASH_SOURCE[0]}" != "$0" ]]; then
  echo "错误: 禁止 source；请直接执行本脚本并传入子进程命令" >&2
  return 2
fi

set -euo pipefail

usage() {
  cat <<'EOF'
用法:
  scripts/live/export-foundationx-env.sh --env dev [选项] -- <command> [args...]

选项:
  --secrets-dir <dir>  dev.md 所在目录
  --nats-conf <path>   可选的 dev NATS 配置文件；默认不读取宿主配置

示例:
  scripts/live/export-foundationx-env.sh --env dev -- \
    cargo test -p redisx --test live_kv -- --ignored
EOF
}

env_name="dev"
secrets_dir=""
nats_conf=""
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
    --nats-conf)
      [[ $# -ge 2 ]] || { echo "错误: --nats-conf 缺少参数" >&2; exit 2; }
      nats_conf=$2
      shift 2
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

if [[ "$env_name" != "dev" ]]; then
  echo "错误: 仅允许读取 dev 凭据；其他环境已拒绝" >&2
  exit 2
fi
if [[ $# -eq 0 ]]; then
  echo "错误: -- 后必须提供子进程命令" >&2
  exit 2
fi

umask 077
script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)
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
tmp_dir=$(mktemp -d "${tmp_root%/}/foundationx-live.XXXXXXXXXX")
env_file="$tmp_dir/foundationx.env"

builder_args=(--env "$env_name" --out "$env_file")
if [[ -n "$secrets_dir" ]]; then
  builder_args+=(--secrets-dir "$secrets_dir")
fi
if [[ -n "$nats_conf" ]]; then
  builder_args+=(--nats-conf "$nats_conf")
fi

node "$script_dir/build-foundationx-env.mjs" "${builder_args[@]}"
set +e
node "$script_dir/run-foundationx-command.mjs" --env-file "$env_file" -- "$@"
status=$?
set -e
exit "$status"
