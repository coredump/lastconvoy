#!/usr/bin/env bash
set -euo pipefail

LOG_FILE="/tmp/lastconvoy.log"
SHOW_MODE="all"   # all|last|single
TARGET_RUN=""
SHOW_AGGREGATE=1

while [[ $# -gt 0 ]]; do
  case "$1" in
    --last)
      SHOW_MODE="last"
      shift
      ;;
    --run)
      if [[ $# -lt 2 ]]; then
        echo "Missing value for --run"
        exit 1
      fi
      TARGET_RUN="$2"
      SHOW_MODE="single"
      shift 2
      ;;
    --no-aggregate)
      SHOW_AGGREGATE=0
      shift
      ;;
    --help|-h)
      cat <<'USAGE'
Usage: scripts/analyze_balance_log.sh [options] [log_file]

Options:
  --last           Show only the latest detected run
  --run N          Show only run number N (1-based)
  --no-aggregate   Omit overall aggregate summary
  -h, --help       Show this help

Default:
  Shows all detected runs + overall aggregate summary.
USAGE
      exit 0
      ;;
    -* )
      echo "Unknown option: $1"
      exit 1
      ;;
    *)
      LOG_FILE="$1"
      shift
      ;;
  esac
done

if [[ ! -f "$LOG_FILE" ]]; then
  echo "Log file not found: $LOG_FILE"
  echo "Run: cargo run"
  echo "Expected log path (from config.toml): /tmp/lastconvoy.log"
  exit 1
fi

awk -v mode="$SHOW_MODE" -v target_run="$TARGET_RUN" -v show_aggregate="$SHOW_AGGREGATE" -v log_path="$LOG_FILE" '
  function as_num(v) { return v + 0.0 }
  function kv_num(key,   m) {
    if (match($0, key "=([0-9.]+)", m)) return as_num(m[1])
    return -1
  }
  function kv_int(key,   m) {
    if (match($0, key "=([0-9]+)", m)) return m[1] + 0
    return -1
  }
  function pct(part, whole) {
    if (whole <= 0) return 0
    return 100.0 * part / whole
  }
  function run_key(r, k) { return r SUBSEP k }
  function start_run(origin) {
    run_count++
    cur = run_count
    run_origin[cur] = origin
    first_time[cur] = -1
    last_time[cur] = -1
    ended[cur] = 0
    issues_by_run[cur] = 0
  }
  function ensure_run() {
    if (cur == 0) start_run("implicit")
  }
  function update_time(t) {
    if (first_time[cur] < 0) first_time[cur] = t
    last_time[cur] = t
  }
  function run_seconds(r) {
    if (last_time[r] < 0) return 0
    return last_time[r]
  }
  function has_data(r) {
    return snapshot_count[r] > 0 || breach_start_count[r] > 0 || breach_resolve_count[r] > 0 || player_hit_count[r] > 0 || run_seconds(r) > 0
  }
  function include_run(r) {
    if (mode == "last") return r == run_count
    if (mode == "single") return r == (target_run + 0)
    return 1
  }
  function print_heuristics(r, run_minutes, early_breaches, ttk_max_v, buffs_avg,   issues) {
    issues = 0

    if (run_minutes < 8.0) {
      print "- CRITICAL: run ended very early (<8 min)."
      issues++
    } else if (run_minutes < 12.0) {
      print "- WARNING: run ended below target band (12-18 min)."
      issues++
    } else if (run_minutes > 22.0) {
      print "- WARNING: run exceeded expected band (>22 min), likely undertuned pressure."
      issues++
    } else {
      print "- OK: run length within expected band."
    }

    if (early_breaches > 6) {
      print "- WARNING: high early breach pressure (>6 in first 2 min)."
      issues++
    } else {
      print "- OK: early breach pressure is moderate."
    }

    if (ttk_max_v > 12.0) {
      print "- WARNING: large enemy TTK peak >12s (possible sponge feel)."
      issues++
    } else {
      print "- OK: large enemy TTK stays readable."
    }

    if (buffs_avg < 0.8) {
      print "- WARNING: low offense buff uptime (<0.8 active buffs on average)."
      issues++
    } else {
      print "- OK: offense buff uptime looks healthy."
    }

    if (issues == 0) {
      print "- Overall: no major balance red flags."
    } else {
      printf "- Overall: %d potential balance issue(s) flagged.\n", issues
    }

    return issues
  }
  function print_run(r,   run_sec, run_min, breach_pm, dps_avg, ttk_avg, pressure_avg, buffs_avg, pickups, live_pct, i, bname, dps_min_v, dps_max_v, ttk_min_v, ttk_max_v, issues) {
    run_sec = run_seconds(r)
    run_min = run_sec / 60.0
    breach_pm = (run_min > 0 ? breach_resolve_count[r] / run_min : 0)
    dps_avg = (dps_count[r] > 0 ? dps_sum[r] / dps_count[r] : 0)
    ttk_avg = (ttk_count[r] > 0 ? ttk_sum[r] / ttk_count[r] : 0)
    pressure_avg = (pressure_count[r] > 0 ? pressure_sum[r] / pressure_count[r] : breach_pm)
    buffs_avg = (snapshot_count[r] > 0 ? buffs_active_sum[r] / snapshot_count[r] : 0)

    dps_min_v = (dps_count[r] > 0 ? dps_min[r] : 0)
    dps_max_v = (dps_count[r] > 0 ? dps_max[r] : 0)
    ttk_min_v = (ttk_count[r] > 0 ? ttk_min[r] : 0)
    ttk_max_v = (ttk_count[r] > 0 ? ttk_max[r] : 0)

    printf "\n--- Run #%d", r
    meta = ""
    if (run_id[r] > 0) meta = meta sprintf("run_id=%d", run_id[r])
    if (run_source[r] != "") {
      if (meta != "") meta = meta ", "
      meta = meta sprintf("source=%s", run_source[r])
    }
    if (meta != "") printf " (%s", meta
    else printf " (origin=%s", run_origin[r]
    if (ended[r]) printf ", complete)\n"
    else printf ", incomplete)\n"

    printf "Run length: %.1f min (%.0f s)\n", run_min, run_sec
    printf "Snapshots: %d\n", snapshot_count[r]

    print ""
    print "[Pressure]"
    printf "Breach flow: start=%d resolve=%d (%.2f/min)\n", breach_start_count[r], breach_resolve_count[r], breach_pm
    printf "Pressure snapshot avg: %.2f breaches/min\n", pressure_avg
    printf "Early breaches (<=2 min): %d\n", early_breaches[r]

    print ""
    print "[Survival]"
    printf "Player hits: %d | Shields (last snapshot): %d\n", player_hit_count[r], shields_last[r]
    printf "Explosive triggers: %d\n", explosive_trigger_count[r]
    printf "Kills/Breaches (last snapshot): %d / %d\n", kills_last[r], breaches_last[r]

    print ""
    print "[Lethality]"
    printf "DPS: min=%.2f avg=%.2f max=%.2f\n", dps_min_v, dps_avg, dps_max_v
    printf "Large TTK(s): min=%.2f avg=%.2f max=%.2f\n", ttk_min_v, ttk_avg, ttk_max_v
    printf "Active offense buffs per snapshot (avg): %.2f\n", buffs_avg

    print ""
    print "[Buff cadence]"
    buffs[1] = "DamageBuff"
    buffs[2] = "FireRateBuff"
    buffs[3] = "BurstBuff"
    buffs[4] = "PierceBuff"
    buffs[5] = "StaggerBuff"
    for (i = 1; i <= 5; i++) {
      bname = buffs[i]
      pickups = (run_key(r, bname) in orb_pickups ? orb_pickups[run_key(r, bname)] : 0)
      live_pct = pct((run_key(r, bname) in buff_live ? buff_live[run_key(r, bname)] : 0), snapshot_count[r])
      printf "%s: pickups=%d uptime_by_snapshot=%.0f%%\n", bname, pickups, live_pct
    }
    printf "Other pickups: Shield=%d Explosive=%d Drone=%d DroneRemote=%d\n",
      (run_key(r, "Shield") in orb_pickups ? orb_pickups[run_key(r, "Shield")] : 0),
      (run_key(r, "Explosive") in orb_pickups ? orb_pickups[run_key(r, "Explosive")] : 0),
      (run_key(r, "Drone") in orb_pickups ? orb_pickups[run_key(r, "Drone")] : 0),
      (run_key(r, "DroneRemote") in orb_pickups ? orb_pickups[run_key(r, "DroneRemote")] : 0)

    print ""
    print "[Heuristic flags]"
    issues = print_heuristics(r, run_min, early_breaches[r], ttk_max_v, buffs_avg)
    issues_by_run[r] = issues
  }

  {
    has_time = 0
    t = -1
    if (match($0, /^\[([0-9.]+)s\]/, mtime)) {
      has_time = 1
      t = as_num(mtime[1])
    }

    if ($0 ~ / RUN_START /) {
      start_run("marker")
      if (match($0, /run_id=([0-9]+)/, mid)) run_id[cur] = mid[1] + 0
      if (match($0, /source=([A-Za-z0-9_]+)/, msrc)) run_source[cur] = msrc[1]
      if (has_time) update_time(t)
      next
    }

    ensure_run()

    if (has_time && last_time[cur] >= 0 && t < last_time[cur] - 0.0001) {
      start_run("time_reset")
      if (has_time) update_time(t)
    } else if (has_time) {
      update_time(t)
    }

    if ($0 ~ / RUN_END /) {
      ended[cur] = 1
      if (match($0, /reason=([A-Za-z0-9_]+)/, mre)) run_end_reason[cur] = mre[1]
      next
    }

    if ($0 ~ / BREACH_START /) breach_start_count[cur]++
    if ($0 ~ / BREACH_RESOLVE /) {
      breach_resolve_count[cur]++
      if (has_time && t <= 120.0) early_breaches[cur]++
    }
    if ($0 ~ / PLAYER_HIT / || $0 ~ / PLAYER_DMG /) player_hit_count[cur]++
    if ($0 ~ / EXPLOSIVE_TRIGGER / || $0 ~ / EXPLOSIVE_SHIELD /) explosive_trigger_count[cur]++

    if ($0 ~ / ORB_PICKUP /) {
      total_orb_pickups[cur]++
      if (match($0, /type=([A-Za-z0-9_]+)/, mp)) orb_pickups[run_key(cur, mp[1])]++
    }
    if ($0 ~ / ORB_COLLECT /) {
      total_orb_pickups[cur]++
      if (match($0, /ORB_COLLECT ([A-Za-z0-9_>-]+)/, mc)) orb_pickups[run_key(cur, mc[1])]++
    }

    if ($0 ~ / BALANCE_SNAPSHOT / || $0 ~ / BALANCE /) {
      snapshot_count[cur]++

      dps = kv_num("dps")
      if (dps >= 0) {
        dps_sum[cur] += dps
        dps_count[cur]++
        if (!(cur in dps_min) || dps < dps_min[cur]) dps_min[cur] = dps
        if (!(cur in dps_max) || dps > dps_max[cur]) dps_max[cur] = dps
      }

      ttk = kv_num("ttk_large_s")
      if (ttk < 0 && match($0, /large_ttk=([0-9.]+)s/, mt)) ttk = as_num(mt[1])
      if (ttk >= 0) {
        ttk_sum[cur] += ttk
        ttk_count[cur]++
        if (!(cur in ttk_min) || ttk < ttk_min[cur]) ttk_min[cur] = ttk
        if (!(cur in ttk_max) || ttk > ttk_max[cur]) ttk_max[cur] = ttk
      }

      p = kv_num("pressure_bpm")
      if (p >= 0) {
        pressure_sum[cur] += p
        pressure_count[cur]++
      }

      k = kv_int("kills"); if (k >= 0) kills_last[cur] = k
      b = kv_int("breaches"); if (b >= 0) breaches_last[cur] = b
      s = kv_int("shields"); if (s >= 0) shields_last[cur] = s

      a = kv_int("buffs_active")
      if (a >= 0) {
        buffs_active_sum[cur] += a
      } else {
        active = 0
        if ($0 ~ /dmg=true/) active++
        if ($0 ~ /rate=true/) active++
        if ($0 ~ /burst=true/) active++
        if ($0 ~ /pierce=true/) active++
        if ($0 ~ /stagger=true/) active++
        buffs_active_sum[cur] += active
      }

      if (kv_num("buff_damage_s") > 0) buff_live[run_key(cur, "DamageBuff")]++
      if (kv_num("buff_rate_s") > 0) buff_live[run_key(cur, "FireRateBuff")]++
      if (kv_num("buff_burst_s") > 0) buff_live[run_key(cur, "BurstBuff")]++
      if (kv_num("buff_pierce_s") > 0) buff_live[run_key(cur, "PierceBuff")]++
      if (kv_num("buff_stagger_s") > 0) buff_live[run_key(cur, "StaggerBuff")]++
    }
  }

  END {
    if (run_count == 0) {
      print "No timestamped gameplay lines found in log."
      exit 1
    }

    if (mode == "single") {
      if (target_run !~ /^[0-9]+$/ || target_run + 0 < 1 || target_run + 0 > run_count) {
        printf "Requested run %s is out of range (1..%d).\n", target_run, run_count
        exit 1
      }
    }

    print "== Last Convoy Lane-Pressure Multi-Run Report =="
    printf "Log file: %s\n", log_path
    printf "Runs detected: %d\n", run_count

    reported = 0
    agg_seconds = 0
    agg_snapshots = 0
    agg_breach_start = 0
    agg_breach_resolve = 0
    agg_early_breaches = 0
    agg_player_hits = 0
    agg_explosive = 0
    agg_kills_last = 0
    agg_breaches_last = 0
    agg_shields_last = 0
    agg_dps_sum = 0
    agg_dps_count = 0
    agg_ttk_sum = 0
    agg_ttk_count = 0
    agg_pressure_sum = 0
    agg_pressure_count = 0
    agg_buffs_active_sum = 0
    agg_issues = 0

    for (r = 1; r <= run_count; r++) {
      if (!include_run(r)) continue
      if (!has_data(r)) continue

      print_run(r)
      reported++

      agg_seconds += run_seconds(r)
      agg_snapshots += snapshot_count[r]
      agg_breach_start += breach_start_count[r]
      agg_breach_resolve += breach_resolve_count[r]
      agg_early_breaches += early_breaches[r]
      agg_player_hits += player_hit_count[r]
      agg_explosive += explosive_trigger_count[r]
      agg_kills_last += kills_last[r]
      agg_breaches_last += breaches_last[r]
      agg_shields_last += shields_last[r]
      agg_dps_sum += dps_sum[r]
      agg_dps_count += dps_count[r]
      agg_ttk_sum += ttk_sum[r]
      agg_ttk_count += ttk_count[r]
      agg_pressure_sum += pressure_sum[r]
      agg_pressure_count += pressure_count[r]
      agg_buffs_active_sum += buffs_active_sum[r]
      agg_issues += issues_by_run[r]
    }

    if (reported == 0) {
      print "\nNo runs matched selection or contained gameplay data."
      exit 1
    }

    printf "\nRuns reported: %d\n", reported

    if (!show_aggregate) exit 0

    agg_minutes = agg_seconds / 60.0
    agg_breach_pm = (agg_minutes > 0 ? agg_breach_resolve / agg_minutes : 0)
    agg_dps_avg = (agg_dps_count > 0 ? agg_dps_sum / agg_dps_count : 0)
    agg_ttk_avg = (agg_ttk_count > 0 ? agg_ttk_sum / agg_ttk_count : 0)
    agg_pressure_avg = (agg_pressure_count > 0 ? agg_pressure_sum / agg_pressure_count : agg_breach_pm)
    agg_buffs_avg = (agg_snapshots > 0 ? agg_buffs_active_sum / agg_snapshots : 0)

    print "\n=== Overall Aggregate ==="
    printf "Total runtime: %.1f min (%.0f s)\n", agg_minutes, agg_seconds
    printf "Total snapshots: %d\n", agg_snapshots

    print ""
    print "[Pressure]"
    printf "Breach flow: start=%d resolve=%d (%.2f/min)\n", agg_breach_start, agg_breach_resolve, agg_breach_pm
    printf "Pressure snapshot avg: %.2f breaches/min\n", agg_pressure_avg
    printf "Early breaches (<=2 min): %d\n", agg_early_breaches

    print ""
    print "[Survival]"
    printf "Player hits: %d\n", agg_player_hits
    printf "Explosive triggers: %d\n", agg_explosive
    printf "Kills/Breaches (sum of run last snapshots): %d / %d\n", agg_kills_last, agg_breaches_last
    printf "Shields (sum of run last snapshots): %d\n", agg_shields_last

    print ""
    print "[Lethality]"
    printf "DPS avg: %.2f\n", agg_dps_avg
    printf "Large TTK avg: %.2f s\n", agg_ttk_avg
    printf "Active offense buffs per snapshot (avg): %.2f\n", agg_buffs_avg

    print ""
    print "[Heuristic tally]"
    printf "Potential issue flags across reported runs: %d\n", agg_issues
  }
' "$LOG_FILE"
