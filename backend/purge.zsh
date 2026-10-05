#!/usr/bin/env zsh
set -euo pipefail
zmodload zsh/datetime

# Check prerequisites before any system changes, including the JSON encoder.
for tool in sudo pagesize vm_stat sysctl awk date hostname whoami df jq dscacheutil killall tmutil; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    print -u2 -- "Required command not found: $tool"
    exit 1
  fi
done

# Authentication is performed by the caller with the TUI suspended. Never prompt
# from this captured-output backend, even if credentials expire during a run.
if ! sudo -n -v; then
  print -u2 -- "Administrator credentials are unavailable. Confirm the purge again to authenticate."
  exit 1
fi

# ── MEMORY STATS HELPER ─────────────────────────
get_memory_stats() {
  local page_size free_pages active_pages inactive_pages speculative_pages compressed_pages
  
  # Get page size (typically 16384 on Apple Silicon, 4096 on Intel)
  page_size=$(pagesize) || return 1
  [[ "$page_size" == <-> && "$page_size" -gt 0 ]] || return 1
  
  # Parse vm_stat output
  local vmstat
  vmstat=$(vm_stat) || return 1
  
  free_pages=$(echo "$vmstat" | awk '/Pages free:/ {gsub(/\./,"",$3); print $3}')
  active_pages=$(echo "$vmstat" | awk '/Pages active:/ {gsub(/\./,"",$3); print $3}')
  inactive_pages=$(echo "$vmstat" | awk '/Pages inactive:/ {gsub(/\./,"",$3); print $3}')
  speculative_pages=$(echo "$vmstat" | awk '/Pages speculative:/ {gsub(/\./,"",$3); print $3}')
  compressed_pages=$(echo "$vmstat" | awk '/Pages occupied by compressor:/ {gsub(/\./,"",$5); print $5}')
  
  # A missing reading is unknown, not an observed zero.
  local count
  for count in "$free_pages" "$active_pages" "$inactive_pages" "$speculative_pages" "$compressed_pages"; do
    [[ "$count" == <-> ]] || return 1
  done
  
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
  swap_info=$(sysctl vm.swapusage 2>/dev/null) || return 1
  
  if [[ -z "$swap_info" ]]; then
    return 1
  fi
  
  # Parse: vm.swapusage: total = 1024.00M  used = 512.00M  free = 512.00M
  local used_mb free_mb
  used_mb=$(echo "$swap_info" | awk '{for(i=1;i<=NF;i++) if($i=="used") {gsub(/M/,"",$(i+2)); print int($(i+2))}}')
  free_mb=$(echo "$swap_info" | awk '{for(i=1;i<=NF;i++) if($i=="free") {gsub(/M/,"",$(i+2)); print int($(i+2))}}')
  
  [[ "$used_mb" == <-> && "$free_mb" == <-> ]] || return 1
  
  echo "${used_mb}:${free_mb}"
}

clergy_purge() {
  local MODE="tui"
  [[ "${1:-}" == "--json" ]] && MODE="json"

  local start_ts end_ts
  start_ts=$(date +%s)
  local started_at finished_at
  started_at=$(date '+%Y-%m-%d %H:%M:%S %z')

  local host user
  host="$(hostname)"
  user="$(whoami)"

  # ── CAPTURE BEFORE STATS ──────────────────────
  local df_before df_after
  df_before="$(df -h /)"
  local disk_available_before disk_available_after
  disk_available_before=$(df -kP / | awk 'NR==2 {print $4}' || true)
  [[ "$disk_available_before" == <-> ]] || disk_available_before=null

  local snapshots_before snapshots_after
  snapshots_before=$(tmutil listlocalsnapshots / 2>/dev/null) || snapshots_before="__unavailable__"
  
  local mem_before swap_before
  local memory_before_available=true memory_after_available=true
  local swap_before_available=true swap_after_available=true
  mem_before=$(get_memory_stats) || { mem_before="0:0:0"; memory_before_available=false; }
  swap_before=$(get_swap_stats) || { swap_before="0:0"; swap_before_available=false; }

  # Capture each command's outcome. Stop after the first failure as before, but
  # retain the completed/failed/skipped steps in the final report.
  local -A action_status action_code action_ms action_output
  local previous_failed=false
  record_action() {
    local action_id="$1"
    shift
    action_status[$action_id]=skipped
    action_code[$action_id]=null
    action_ms[$action_id]=0
    action_output[$action_id]="Not run because an earlier action failed."
    [[ "$previous_failed" == false ]] || return 0

    local began="$EPOCHREALTIME" captured result_code
    if captured=$(sudo -n "$@" 2>&1); then
      result_code=0
      action_status[$action_id]=succeeded
    else
      result_code=$?
      action_status[$action_id]=failed
      previous_failed=true
    fi
    action_code[$action_id]=$result_code
    action_ms[$action_id]=$(printf '%.0f' "$(( (EPOCHREALTIME - began) * 1000 ))")
    # Bounded command diagnostics, never password input (all sudo calls use -n).
    action_output[$action_id]="${captured[1,2000]}"
    if (( ${#captured} > 2000 )); then
      action_output[$action_id]+=$'\n[output truncated at 2000 characters]'
    fi
  }

  record_action cache dscacheutil -flushcache
  record_action dns killall -HUP mDNSResponder
  record_action snapshots tmutil thinlocalsnapshots / 9999999999 4

  # ── CAPTURE AFTER STATS ───────────────────────
  df_after="$(df -h /)"
  disk_available_after=$(df -kP / | awk 'NR==2 {print $4}' || true)
  [[ "$disk_available_after" == <-> ]] || disk_available_after=null
  snapshots_after=$(tmutil listlocalsnapshots / 2>/dev/null) || snapshots_after="__unavailable__"
  
  local mem_after swap_after
  mem_after=$(get_memory_stats) || { mem_after="0:0:0"; memory_after_available=false; }
  swap_after=$(get_swap_stats) || { swap_after="0:0"; swap_after_available=false; }
  
  end_ts=$(date +%s)
  finished_at=$(date '+%Y-%m-%d %H:%M:%S %z')

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
      --arg started_at "$started_at" \
      --arg finished_at "$finished_at" \
      --argjson disk_available_before "$disk_available_before" \
      --argjson disk_available_after "$disk_available_after" \
      --arg snapshots_before "$snapshots_before" \
      --arg snapshots_after "$snapshots_after" \
      --arg cache_status "$action_status[cache]" \
      --argjson cache_code "$action_code[cache]" \
      --argjson cache_ms "$action_ms[cache]" \
      --arg cache_output "$action_output[cache]" \
      --arg dns_status "$action_status[dns]" \
      --argjson dns_code "$action_code[dns]" \
      --argjson dns_ms "$action_ms[dns]" \
      --arg dns_output "$action_output[dns]" \
      --arg snapshots_status "$action_status[snapshots]" \
      --argjson snapshots_code "$action_code[snapshots]" \
      --argjson snapshots_ms "$action_ms[snapshots]" \
      --arg snapshots_output "$action_output[snapshots]" \
      --argjson memory_before_available "$memory_before_available" \
      --argjson memory_after_available "$memory_after_available" \
      --argjson swap_before_available "$swap_before_available" \
      --argjson swap_after_available "$swap_after_available" \
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
      'def snapshot_names:
        if . == "__unavailable__" then null
        else split("\n") | map(select(startswith("com.apple.TimeMachine."))) | unique end;
      {
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
        dns_flushed: ($cache_status == "succeeded" and $dns_status == "succeeded"),
        snapshots_thinned: ($snapshots_status == "succeeded"),
        started_at: $started_at,
        finished_at: $finished_at,
        disk_available_before_kib: $disk_available_before,
        disk_available_after_kib: $disk_available_after,
        snapshots_before: ($snapshots_before | snapshot_names),
        snapshots_after: ($snapshots_after | snapshot_names),
        memory_before_available: $memory_before_available,
        memory_after_available: $memory_after_available,
        swap_before_available: $swap_before_available,
        swap_after_available: $swap_after_available,
        actions: [
          {label: "Directory cache", command: "sudo -n dscacheutil -flushcache",
           status: $cache_status, exit_code: $cache_code, duration_ms: $cache_ms, output: $cache_output},
          {label: "DNS responder", command: "sudo -n killall -HUP mDNSResponder",
           status: $dns_status, exit_code: $dns_code, duration_ms: $dns_ms, output: $dns_output},
          {label: "Time Machine snapshots", command: "sudo -n tmutil thinlocalsnapshots / 9999999999 4",
           status: $snapshots_status, exit_code: $snapshots_code, duration_ms: $snapshots_ms, output: $snapshots_output}
        ]
      }'
    return
  fi

  # ── TUI MODE (temporary) ────────────────────────
  echo "✝︎ CLERGY · PURGE FINISHED ($((end_ts-start_ts))s)"
  [[ "$previous_failed" == false ]]
}

clergy_purge "$@"
