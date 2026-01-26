#!/usr/bin/env zsh
set -euo pipefail

# Require admin privileges upfront
if ! sudo -v; then
  echo "Administrator privileges are required."
  exit 1
fi

# ── MEMORY STATS HELPER ─────────────────────────
get_memory_stats() {
  local page_size free_pages active_pages inactive_pages speculative_pages compressed_pages
  
  # Get page size (typically 16384 on Apple Silicon, 4096 on Intel)
  page_size=$(pagesize)
  
  # Parse vm_stat output
  local vmstat
  vmstat=$(vm_stat)
  
  free_pages=$(echo "$vmstat" | awk '/Pages free:/ {gsub(/\./,"",$3); print $3}')
  active_pages=$(echo "$vmstat" | awk '/Pages active:/ {gsub(/\./,"",$3); print $3}')
  inactive_pages=$(echo "$vmstat" | awk '/Pages inactive:/ {gsub(/\./,"",$3); print $3}')
  speculative_pages=$(echo "$vmstat" | awk '/Pages speculative:/ {gsub(/\./,"",$3); print $3}')
  compressed_pages=$(echo "$vmstat" | awk '/Pages occupied by compressor:/ {gsub(/\./,"",$5); print $5}')
  
  # Default to 0 if not found
  free_pages=${free_pages:-0}
  active_pages=${active_pages:-0}
  inactive_pages=${inactive_pages:-0}
  speculative_pages=${speculative_pages:-0}
  compressed_pages=${compressed_pages:-0}
  
  # Calculate in MB
  local bytes_per_mb=$((1024 * 1024))
  local free_mb=$(( (free_pages * page_size) / bytes_per_mb ))
  local used_mb=$(( ((active_pages + inactive_pages + speculative_pages) * page_size) / bytes_per_mb ))
  local compressed_mb=$(( (compressed_pages * page_size) / bytes_per_mb ))
  
  echo "${used_mb}:${free_mb}:${compressed_mb}"
}

# ── SWAP STATS HELPER ───────────────────────────
get_swap_stats() {
  local swap_info
  swap_info=$(sysctl vm.swapusage 2>/dev/null || echo "")
  
  if [[ -z "$swap_info" ]]; then
    echo "0:0"
    return
  fi
  
  # Parse: vm.swapusage: total = 1024.00M  used = 512.00M  free = 512.00M
  local used_mb free_mb
  used_mb=$(echo "$swap_info" | awk '{for(i=1;i<=NF;i++) if($i=="used") {gsub(/M/,"",$(i+2)); print int($(i+2))}}')
  free_mb=$(echo "$swap_info" | awk '{for(i=1;i<=NF;i++) if($i=="free") {gsub(/M/,"",$(i+2)); print int($(i+2))}}')
  
  used_mb=${used_mb:-0}
  free_mb=${free_mb:-0}
  
  echo "${used_mb}:${free_mb}"
}

clergy_purge() {
  local MODE="tui"
  [[ "${1:-}" == "--json" ]] && MODE="json"

  local start_ts end_ts
  start_ts=$(date +%s)

  local host user
  host="$(hostname)"
  user="$(whoami)"

  # ── CAPTURE BEFORE STATS ──────────────────────
  local df_before df_after
  df_before="$(df -h /)"
  
  local mem_before swap_before
  mem_before=$(get_memory_stats)
  swap_before=$(get_swap_stats)

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

  # ── CAPTURE AFTER STATS ───────────────────────
  df_after="$(df -h /)"
  
  local mem_after swap_after
  mem_after=$(get_memory_stats)
  swap_after=$(get_swap_stats)
  
  end_ts=$(date +%s)

  # ── PARSE STATS ───────────────────────────────
  local mem_before_used mem_before_free mem_before_compressed
  local mem_after_used mem_after_free mem_after_compressed
  local swap_before_used swap_before_free
  local swap_after_used swap_after_free
  
  IFS=':' read -r mem_before_used mem_before_free mem_before_compressed <<< "$mem_before"
  IFS=':' read -r mem_after_used mem_after_free mem_after_compressed <<< "$mem_after"
  IFS=':' read -r swap_before_used swap_before_free <<< "$swap_before"
  IFS=':' read -r swap_after_used swap_after_free <<< "$swap_after"

  # ── JSON MODE ───────────────────────────────────
  if [[ "$MODE" == "json" ]]; then
    jq -n \
      --arg host "$host" \
      --arg user "$user" \
      --arg start "$(date -r "$start_ts" '+%H:%M:%S')" \
      --arg duration "$((end_ts-start_ts))" \
      --arg before "$df_before" \
      --arg after "$df_after" \
      --argjson mem_before_used "$mem_before_used" \
      --argjson mem_before_free "$mem_before_free" \
      --argjson mem_before_compressed "$mem_before_compressed" \
      --argjson mem_after_used "$mem_after_used" \
      --argjson mem_after_free "$mem_after_free" \
      --argjson mem_after_compressed "$mem_after_compressed" \
      --argjson swap_before_used "$swap_before_used" \
      --argjson swap_before_free "$swap_before_free" \
      --argjson swap_after_used "$swap_after_used" \
      --argjson swap_after_free "$swap_after_free" \
      '{
        host: $host,
        user: $user,
        start_time: $start,
        duration_seconds: ($duration | tonumber),
        disk_before: $before,
        disk_after: $after,
        memory_before: {
          used_mb: $mem_before_used,
          free_mb: $mem_before_free,
          compressed_mb: $mem_before_compressed
        },
        memory_after: {
          used_mb: $mem_after_used,
          free_mb: $mem_after_free,
          compressed_mb: $mem_after_compressed
        },
        swap_before: {
          used_mb: $swap_before_used,
          free_mb: $swap_before_free
        },
        swap_after: {
          used_mb: $swap_after_used,
          free_mb: $swap_after_free
        },
        dns_flushed: true,
        snapshots_thinned: true
      }'
    return
  fi

  # ── TUI MODE (temporary) ────────────────────────
  echo "✝︎ CLERGY · PURGE COMPLETE ($((end_ts-start_ts))s)"
}

clergy_purge "$@"
