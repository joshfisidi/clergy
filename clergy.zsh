#!/usr/bin/env zsh
set -euo pipefail

clergy_purge() {
  local MODE="tui"
  [[ "${1:-}" == "--json" ]] && MODE="json"

  local start_ts end_ts
  start_ts=$(date +%s)

  local host user
  host="$(hostname)"
  user="$(whoami)"

  local df_before df_after
  df_before="$(df -h /)"

  # ── ACTIONS ─────────────────────────────────────
  sudo dscacheutil -flushcache
  sudo killall -HUP mDNSResponder
  tmutil thinlocalsnapshots / 9999999999 4

  df_after="$(df -h /)"
  end_ts=$(date +%s)

  # ── JSON MODE ───────────────────────────────────
  if [[ "$MODE" == "json" ]]; then
    jq -n \
      --arg host "$host" \
      --arg user "$user" \
      --arg duration "$((end_ts-start_ts))" \
      --arg before "$df_before" \
      --arg after "$df_after" \
      '{
        host: $host,
        user: $user,
        duration_seconds: ($duration | tonumber),
        disk_before: $before,
        disk_after: $after,
        daemons_cleared: true
      }'
    return
  fi

  # ── TUI MODE (temporary / minimal) ──────────────
  echo "✝︎ CLERGY · PURGE COMPLETE ($((end_ts-start_ts))s)"
}
