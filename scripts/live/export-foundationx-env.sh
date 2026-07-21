#!/usr/bin/env bash
# 从本地 secrets/env 文件加载 FOUNDATIONX_*（不打印值、不嵌入密钥）。
#
# 用法:
#   source scripts/live/export-foundationx-env.sh /path/to/file.env
#   source scripts/live/export-foundationx-env.sh /path/to/secrets/dir
#   scripts/live/export-foundationx-env.sh --list   # 仅打印所需变量名
#
# 文件格式（.env.example 风格）:
#   KEY=VALUE
#   # 注释行忽略
#   KEY="quoted value"
#
# ---- 常用变量清单（本脚本不提供默认 secret）----
# Redis:
#   FOUNDATIONX_REDIS_URL / REDIS_URL
# OSS (ossx):
#   FOUNDATIONX_OSSX_ENDPOINT=https://oss-ap-northeast-1.aliyuncs.com
#   FOUNDATIONX_OSSX_BUCKET=
#   FOUNDATIONX_OSSX_ACCESS_KEY_ID=
#   FOUNDATIONX_OSSX_ACCESS_KEY_SECRET=
#   FOUNDATIONX_OSSX_REGION=ap-northeast-1
# Postgres / 其他 adapter: 以各 crate README 为准
#
set -euo pipefail

if [[ "${1:-}" == "--list" || "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  cat <<'EOF'
Required / known FOUNDATIONX_* keys (values must come from your secret store):

  FOUNDATIONX_OSSX_ENDPOINT
  FOUNDATIONX_OSSX_BUCKET
  FOUNDATIONX_OSSX_ACCESS_KEY_ID
  FOUNDATIONX_OSSX_ACCESS_KEY_SECRET
  FOUNDATIONX_OSSX_REGION          # optional

  FOUNDATIONX_REDIS_URL            # or REDIS_URL
  FOUNDATIONX_POSTGRES_URL
  FOUNDATIONX_KAFKA_BROKERS
  FOUNDATIONX_NATS_URL
  FOUNDATIONX_CLICKHOUSE_URL

Usage:
  source scripts/live/export-foundationx-env.sh /path/to/file.env
  source scripts/live/export-foundationx-env.sh /path/to/dir
EOF
  return 0 2>/dev/null || exit 0
fi

if [[ $# -lt 1 ]]; then
  echo "usage: source $0 <env-file-or-dir> | $0 --list" >&2
  return 2 2>/dev/null || exit 2
fi

target=$1
load_file() {
  local f=$1
  [[ -f "$f" ]] || return 0
  # 仅接受 KEY=VALUE 行；跳过注释
  while IFS= read -r line || [[ -n "$line" ]]; do
    [[ "$line" =~ ^[[:space:]]*# ]] && continue
    [[ "$line" =~ ^[[:space:]]*$ ]] && continue
    if [[ "$line" =~ ^([A-Za-z_][A-Za-z0-9_]*)=(.*)$ ]]; then
      key="${BASH_REMATCH[1]}"
      val="${BASH_REMATCH[2]}"
      # 去引号
      val="${val%\"}"; val="${val#\"}"
      val="${val%\'}"; val="${val#\'}"
      export "$key=$val"
    fi
  done < "$f"
  echo "loaded env keys from $(basename "$f")" >&2
}

if [[ -d "$target" ]]; then
  for f in "$target"/*.env "$target"/*; do
    [[ -f "$f" ]] || continue
    case "$f" in
      *.env|*.md) load_file "$f" ;;
    esac
  done
else
  load_file "$target"
fi

# 从 ZoneCNH secrets/env/*.md 生成 .env（推荐 live 测试入口）:
#   node scripts/live/build-foundationx-env.mjs --env dev --out /tmp/foundationx-live.env
#   set -a; source /tmp/foundationx-live.env; set +a
#   cargo test -p redisx --test live_kv -- --ignored
