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

  {
    if (match($0, /^\[([0-9.]+)s\]/, t)) {
      time = as_num(t[1])
      if (time > run_seconds) run_seconds = time
    }
  }

  / BREACH_START / { breach_start_count++ }
  / BREACH_RESOLVE / {
    breach_resolve_count++
    if (time <= 120.0) early_breaches++
  }
  / PLAYER_HIT / { player_hit_count++ }
  / PLAYER_DMG / { player_hit_count++ } # backward-compat
  / EXPLOSIVE_TRIGGER / { explosive_trigger_count++ }
  / EXPLOSIVE_SHIELD / { explosive_trigger_count++ } # backward-compat

  / ORB_PICKUP / {
    total_orb_pickups++
    if (match($0, /type=([A-Za-z0-9_]+)/, m)) {
      orb_pickups[m[1]]++
    }
  }
  / ORB_COLLECT / { # backward-compat with older format
    total_orb_pickups++
    if (match($0, /ORB_COLLECT ([A-Za-z0-9_>-]+)/, m)) {
      orb_pickups[m[1]]++
    }
  }

  / BALANCE_SNAPSHOT / || / BALANCE / {
    snapshot_count++

    dps = kv_num("dps")
    if (dps >= 0) {
      dps_sum += dps
      if (dps_min == 0 || dps < dps_min) dps_min = dps
      if (dps > dps_max) dps_max = dps
    }

    ttk = kv_num("ttk_large_s")
    if (ttk < 0 && match($0, /large_ttk=([0-9.]+)s/, m)) ttk = as_num(m[1])
    if (ttk >= 0) {
      ttk_sum += ttk
      if (ttk_min == 0 || ttk < ttk_min) ttk_min = ttk
      if (ttk > ttk_max) ttk_max = ttk
    }

    p = kv_num("pressure_bpm")
    if (p >= 0) {
      pressure_sum += p
      pressure_count++
    }

    k = kv_int("kills"); if (k >= 0) kills_last = k
    b = kv_int("breaches"); if (b >= 0) breaches_last = b
    s = kv_int("shields"); if (s >= 0) shields_last = s

    a = kv_int("buffs_active")
    if (a >= 0) {
      buffs_active_sum += a
    } else {
      # backward-compat with bool flags in older BALANCE lines
      active = 0
      if ($0 ~ /dmg=true/) active++
      if ($0 ~ /rate=true/) active++
      if ($0 ~ /burst=true/) active++
      if ($0 ~ /pierce=true/) active++
      if ($0 ~ /stagger=true/) active++
      buffs_active_sum += active
    }

    if (kv_num("buff_damage_s") > 0) buff_live["DamageBuff"]++
    if (kv_num("buff_rate_s") > 0) buff_live["FireRateBuff"]++
    if (kv_num("buff_burst_s") > 0) buff_live["BurstBuff"]++
    if (kv_num("buff_pierce_s") > 0) buff_live["PierceBuff"]++
    if (kv_num("buff_stagger_s") > 0) buff_live["StaggerBuff"]++
  }

  END {
    if (run_seconds <= 0) {
      print "No timestamped gameplay lines found in log."
      exit 1
    }

    run_minutes = run_seconds / 60.0
    breach_per_min = (run_minutes > 0 ? breach_resolve_count / run_minutes : 0)
    dps_avg = (snapshot_count > 0 ? dps_sum / snapshot_count : 0)
    ttk_avg = (snapshot_count > 0 ? ttk_sum / snapshot_count : 0)
    pressure_avg = (pressure_count > 0 ? pressure_sum / pressure_count : breach_per_min)
    buffs_active_avg = (snapshot_count > 0 ? buffs_active_sum / snapshot_count : 0)

    print "== Last Convoy Lane-Pressure Report =="
    printf "Log file: %s\n", ARGV[1]
    printf "Run length: %.1f min (%.0f s)\n", run_minutes, run_seconds
    printf "Snapshots: %d\n", snapshot_count

    print ""
    print "[Pressure]"
    printf "Breach flow: start=%d resolve=%d (%.2f/min)\n", breach_start_count, breach_resolve_count, breach_per_min
    printf "Pressure snapshot avg: %.2f breaches/min\n", pressure_avg
    printf "Early breaches (<=2 min): %d\n", early_breaches

    print ""
    print "[Survival]"
    printf "Player hits: %d | Shields (last snapshot): %d\n", player_hit_count, shields_last
    printf "Explosive triggers: %d\n", explosive_trigger_count
    printf "Kills/Breaches (last snapshot): %d / %d\n", kills_last, breaches_last

    print ""
    print "[Lethality]"
    printf "DPS: min=%.2f avg=%.2f max=%.2f\n", dps_min, dps_avg, dps_max
    printf "Large TTK(s): min=%.2f avg=%.2f max=%.2f\n", ttk_min, ttk_avg, ttk_max
    printf "Active offense buffs per snapshot (avg): %.2f\n", buffs_active_avg

    print ""
    print "[Buff cadence]"
    buffs[1] = "DamageBuff"
    buffs[2] = "FireRateBuff"
    buffs[3] = "BurstBuff"
    buffs[4] = "PierceBuff"
    buffs[5] = "StaggerBuff"
    for (i = 1; i <= 5; i++) {
      bname = buffs[i]
      pickups = (bname in orb_pickups ? orb_pickups[bname] : 0)
      live_pct = pct((bname in buff_live ? buff_live[bname] : 0), snapshot_count)
      printf "%s: pickups=%d uptime_by_snapshot=%.0f%%\n", bname, pickups, live_pct
    }
    printf "Other pickups: Shield=%d Explosive=%d Drone=%d DroneRemote=%d\n",
      ("Shield" in orb_pickups ? orb_pickups["Shield"] : 0),
      ("Explosive" in orb_pickups ? orb_pickups["Explosive"] : 0),
      ("Drone" in orb_pickups ? orb_pickups["Drone"] : 0),
      ("DroneRemote" in orb_pickups ? orb_pickups["DroneRemote"] : 0)

    print ""
    print "[Heuristic flags]"
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

    if (ttk_max > 12.0) {
      print "- WARNING: large enemy TTK peak >12s (possible sponge feel)."
      issues++
    } else {
      print "- OK: large enemy TTK stays readable."
    }

    if (buffs_active_avg < 0.8) {
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
  }
' "$LOG_FILE"
