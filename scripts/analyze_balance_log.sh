#!/usr/bin/env bash
set -euo pipefail

LOG_FILE="${1:-/tmp/lastconvoy.log}"

if [[ ! -f "$LOG_FILE" ]]; then
  echo "Log file not found: $LOG_FILE"
  echo "Run: cargo run"
  echo "Expected log path (from config.toml): /tmp/lastconvoy.log"
  exit 1
fi

awk '
  function as_num(v) { return v + 0.0 }

  {
    if (match($0, /^\[([0-9.]+)s\]/, t)) {
      time = as_num(t[1])
      if (time > run_seconds) run_seconds = time
    }
  }

  / BREACH_START / {
    breach_start_count++
  }

  / BREACH_RESOLVE / {
    breach_resolve_count++
    if (time <= 120.0) early_breaches++
  }

  / ORB_COLLECT / {
    if (match($0, /ORB_COLLECT ([A-Za-z]+)/, m)) {
      orb_collect[m[1]]++
      total_orb_collect++
    }
  }

  / BALANCE / {
    balance_count++

    if (match($0, /dps=([0-9.]+)/, m)) {
      dps = as_num(m[1])
      dps_sum += dps
      if (dps_min == 0 || dps < dps_min) dps_min = dps
      if (dps > dps_max) dps_max = dps
    }

    if (match($0, /large_ttk=([0-9.]+)s/, m)) {
      ttk = as_num(m[1])
      ttk_sum += ttk
      if (ttk_min == 0 || ttk < ttk_min) ttk_min = ttk
      if (ttk > ttk_max) ttk_max = ttk
    }

    if (match($0, /kills=([0-9]+)/, m)) {
      kills_last = m[1] + 0
    }

    if (match($0, /breaches=([0-9]+)/, m)) {
      breaches_last = m[1] + 0
    }

    if (match($0, /shields=([0-9]+)/, m)) {
      shields_last = m[1] + 0
    }
  }

  END {
    if (run_seconds <= 0) {
      print "No timestamped gameplay lines found in log."
      exit 1
    }

    run_minutes = run_seconds / 60.0
    breach_per_min = (run_minutes > 0 ? breach_resolve_count / run_minutes : 0)
    dps_avg = (balance_count > 0 ? dps_sum / balance_count : 0)
    ttk_avg = (balance_count > 0 ? ttk_sum / balance_count : 0)

    print "== Last Convoy Balance Report =="
    printf "Log file: %s\n", ARGV[1]
    printf "Run length: %.1f min (%.0f s)\n", run_minutes, run_seconds
    printf "Balance snapshots: %d\n", balance_count
    printf "Breach events: start=%d resolve=%d (%.2f / min)\n", breach_start_count, breach_resolve_count, breach_per_min
    printf "Early-run breaches (<=2 min): %d\n", early_breaches
    printf "Kills (last snapshot): %d | Breaches (last snapshot): %d | Shields (last snapshot): %d\n", kills_last, breaches_last, shields_last
    printf "DPS: min=%.2f avg=%.2f max=%.2f\n", dps_min, dps_avg, dps_max
    printf "Large TTK(s): min=%.2f avg=%.2f max=%.2f\n", ttk_min, ttk_avg, ttk_max

    print ""
    print "Orb collections:"
    if (total_orb_collect == 0) {
      print "- none"
    } else {
      for (k in orb_collect) {
        printf "- %s: %d\n", k, orb_collect[k]
      }
    }

    print ""
    print "Heuristic verdict:"

    issues = 0
    if (run_minutes < 8.0) {
      print "- CRITICAL: run ended very early (<8 min), likely overtuned pressure or survival instability."
      issues++
    } else if (run_minutes < 12.0) {
      print "- WARNING: run ended below target band (12-18 min)."
      issues++
    } else if (run_minutes > 22.0) {
      print "- WARNING: run exceeded expected band (>22 min), possible undertuned pressure."
      issues++
    } else {
      print "- OK: run length is near target band."
    }

    if (early_breaches > 6) {
      print "- WARNING: high early-run breach count (>6 in first 2 min); first minutes may be too punishing."
      issues++
    } else {
      print "- OK: early-run breach pressure is moderate."
    }

    if (ttk_max > 12.0) {
      print "- WARNING: high large enemy TTK peak (>12s) indicates potential bullet-sponge behavior."
      issues++
    } else {
      print "- OK: large enemy TTK remains in a readable range."
    }

    if (dps_max > 0 && dps_min > 0 && (dps_max / dps_min) < 1.30) {
      print "- WARNING: weak DPS growth (<1.30x); offense upgrades may feel low-impact."
      issues++
    } else {
      print "- OK: DPS growth suggests upgrades are having visible impact."
    }

    if (issues == 0) {
      print "- Overall: no major balance red flags from this run log."
    } else {
      printf "- Overall: %d potential balance issue(s) flagged for follow-up tuning.\n", issues
    }
  }
' "$LOG_FILE"
