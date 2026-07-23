#!/usr/bin/env bash
# oss-verify CI 报告过滤脚本
# 用法:
#   oss-verify run -o report.json
#   bash scripts/ci/oss-verify-filter.sh report.json [--summary|--failures|--layers|--latency|--slack]

set -euo pipefail

REPORT="${1:-/home/workspace/data/ossx/verify-report.json}"
ACTION="${2:---summary}"

if [[ ! -f "$REPORT" ]]; then
    echo "报告文件不存在: $REPORT"
    exit 2
fi

case "$ACTION" in
    --exit-code)
        # CI 门禁：PASS → exit 0, FAIL → exit 1, PARTIAL → exit 1
        STATUS=$(jq -r '.status' "$REPORT")
        if [[ "$STATUS" == "PASS" ]]; then
            echo "PASS"
            exit 0
        else
            echo "FAIL ($STATUS)"
            exit 1
        fi
        ;;

    --summary)
        # 一行摘要
        jq -r '"\(.status) | \(.passed)/\(.total) passed | \(.duration_ms)ms | \(.summary)"' "$REPORT"
        ;;

    --status)
        # 仅状态码
        jq -r '.status' "$REPORT"
        ;;

    --failures)
        # 列出所有失败项
        FAILED=$(jq -r '[.checks[] | select(.passed == false)] | length' "$REPORT")
        if [[ "$FAILED" == "0" ]]; then
            echo "无失败项"
        else
            echo "=== 失败项 ($FAILED) ==="
            jq -r '.checks[] | select(.passed == false) | "  [\(.kind)] \(.id): \(.message)\n    └─ \(.detail // "无详情")"' "$REPORT"
        fi
        ;;

    --layers)
        # 按层分组统计
        echo "=== 分层统计 ==="
        jq -r '
          ["L0:配置","L1:连接","L2:基本操作","L3:流式","L4:高级","L5:安全并发"] as $labels |
          [.checks | group_by(
            if .kind == "config" then 0
            elif .kind == "connectivity" then 1
            elif .kind == "object_ops" then 2
            elif .kind == "streaming" then 3
            elif .kind == "advanced" then 4
            else 5 end
          )[] | {
            layer: (if .[0].kind == "config" then 0
                    elif .[0].kind == "connectivity" then 1
                    elif .[0].kind == "object_ops" then 2
                    elif .[0].kind == "streaming" then 3
                    elif .[0].kind == "advanced" then 4
                    else 5 end),
            name: $labels[(if .[0].kind == "config" then 0
                    elif .[0].kind == "connectivity" then 1
                    elif .[0].kind == "object_ops" then 2
                    elif .[0].kind == "streaming" then 3
                    elif .[0].kind == "advanced" then 4
                    else 5 end)],
            total: length,
            passed: (map(select(.passed == true)) | length),
            failed: (map(select(.passed == false)) | length),
            max_latency_ms: (map(.duration_ms) | max)
          }] | sort_by(.layer) | .[] |
          "  \(.name)  \(.passed)/\(.total)  max_lat=\(.max_latency_ms)ms\(if .failed > 0 then "  ❌ \(.failed) FAILED" else "  ✅" end)"
        ' "$REPORT"
        ;;

    --latency)
        # 延迟排序
        echo "=== 延迟排名 (TOP 10) ==="
        jq -r '[.checks[] | {id, kind, duration_ms}] | sort_by(-.duration_ms) | .[:10][] | "  \(.duration_ms)ms  [\(.kind)] \(.id)"' "$REPORT"
        ;;

    --slack)
        # Slack Markdown 通知
        STATUS=$(jq -r '.status' "$REPORT")
        PASSED=$(jq -r '.passed' "$REPORT")
        TOTAL=$(jq -r '.total' "$REPORT")
        DURATION=$(jq -r '.duration_ms' "$REPORT")
        ICON="✅"
        [[ "$STATUS" != "PASS" ]] && ICON="❌"
        cat <<EOF
${ICON} *oss-verify* ${STATUS} — ${PASSED}/${TOTAL} passed (${DURATION}ms)

\`\`\`
$(jq -r '[.checks[] | "\(if .passed then "✅" else "❌" end) \(.id)  \(.duration_ms)ms  \(.message)"] | join("\n")' "$REPORT")
\`\`\`
EOF
        ;;

    --github-summary)
        # GitHub Actions Step Summary (https://docs.github.com/en/actions/writing-workflows/choosing-what-your-workflow-does/workflow-commands-for-github-actions)
        if [[ -n "${GITHUB_STEP_SUMMARY:-}" ]]; then
            STATUS=$(jq -r '.status' "$REPORT")
            PASSED=$(jq -r '.passed' "$REPORT")
            TOTAL=$(jq -r '.total' "$REPORT")
            DURATION=$(jq -r '.duration_ms' "$REPORT")
            {
                echo "## OSS Verify: ${STATUS}"
                echo ""
                echo "| 指标 | 值 |"
                echo "|------|-----|"
                echo "| 状态 | ${STATUS} |"
                echo "| 通过 | ${PASSED}/${TOTAL} |"
                echo "| 耗时 | ${DURATION}ms |"
                echo ""
                echo "| 检查 | 类别 | 耗时 | 结果 |"
                echo "|------|------|------|------|"
                jq -r '.checks[] | "| \(.id) | \(.kind) | \(.duration_ms)ms | \(if .passed then "✅" else "❌" end) |"' "$REPORT"
                echo ""
                echo "**摘要**: $(jq -r '.summary' "$REPORT")"
            } >> "$GITHUB_STEP_SUMMARY"
        else
            echo "GITHUB_STEP_SUMMARY 未设置（非 GitHub Actions 环境）"
        fi
        ;;

    --json-failures)
        # 仅输出失败项的 JSON
        jq '[.checks[] | select(.passed == false)]' "$REPORT"
        ;;

    --json-passed)
        # 仅输出通过项的 JSON
        jq '[.checks[] | select(.passed == true)]' "$REPORT"
        ;;

    --metrics)
        # 供 Prometheus/Grafana 消费的指标
        STATUS_CODE=$(jq -r 'if .status == "PASS" then 1 else 0 end' "$REPORT")
        jq -r '
"# HELP oss_verify_passed Total passed checks",
"# TYPE oss_verify_passed gauge",
"oss_verify_passed \(.passed)",
"# HELP oss_verify_failed Total failed checks",
"# TYPE oss_verify_failed gauge",
"oss_verify_failed \(.failed)",
"# HELP oss_verify_duration_ms Total verification duration",
"# TYPE oss_verify_duration_ms gauge",
"oss_verify_duration_ms \(.duration_ms)"
' "$REPORT"
        echo "# HELP oss_verify_status 1=pass 0=fail"
        echo "# TYPE oss_verify_status gauge"
        echo "oss_verify_status $STATUS_CODE"
        ;;

    *)
        echo "用法: $0 <report.json> [action]"
        echo ""
        echo "Actions:"
        echo "  --exit-code        CI 门禁 (PASS→0, 其他→1)"
        echo "  --summary          一行摘要"
        echo "  --status           仅状态码"
        echo "  --failures         列出失败项"
        echo "  --layers           按层分组统计"
        echo "  --latency          延迟 TOP 10"
        echo "  --slack            Slack Markdown 通知"
        echo "  --github-summary   GitHub Actions Step Summary"
        echo "  --json-failures    失败项 JSON"
        echo "  --json-passed      通过项 JSON"
        echo "  --metrics          Prometheus 指标"
        exit 1
        ;;
esac
