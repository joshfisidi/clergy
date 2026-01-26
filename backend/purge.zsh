#!/usr/bin/env zsh
set -euo pipefail

# Require admin privileges upfront
if ! sudo -v; then
  echo "Administrator privileges are required."
  exit 1
fi

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

  # ── SILENCE NOISY OUTPUT IF JSON MODE ───────────
  if [[ "$MODE" == "json" ]]; then
    exec 3>&1
    exec 1>/dev/null
  fi

  # ── ACTIONS ─────────────────────────────────────
  sudo dscacheutil -flushcache
  sudo killall -HUP mDNSResponder
  tmutil thinlocalsnapshots / 9999999999 4

  # ── RESTORE STDOUT FOR JSON ─────────────────────
  if [[ "$MODE" == "json" ]]; then
    exec 1>&3
    exec 3>&-
  fi

  df_after="$(df -h /)"
  end_ts=$(date +%s)

  # ── JSON MODE ───────────────────────────────────
  if [[ "$MODE" == "json" ]]; then
    jq -n \
      --arg host "$host" \
      --arg user "$user" \
      --arg start "$(date -r "$start_ts" '+%H:%M:%S')" \
      --arg duration "$((end_ts-start_ts))" \
      --arg before "$df_before" \
      --arg after "$df_after" \
      '{
        host: $host,
        user: $user,
        start_time: $start,
        duration_seconds: ($duration | tonumber),
        disk_before: $before,
        disk_after: $after,
        dns_flushed: true,
        snapshots_thinned: true
      }'
    return
  fi

  # ── TUI MODE (temporary) ────────────────────────
  echo "✝︎ CLERGY · PURGE COMPLETE ($((end_ts-start_ts))s)"
}

clergy_purge "$@"
