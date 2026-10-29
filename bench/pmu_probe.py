"""What a machine's performance counters can measure here (diagnostic only; lmmx/mibz2#76).

The i9's counter study (the cooperative-inversion journal, §10, on the research branches)
rests on events a GitHub-hosted runner may not expose: an earlier CI run found no PMU at
all (docs/journal/2026-10-27-walk-sweep.md). This records, per machine:

- the CPU (vendor, model, family and model numbers), its topology (CPUs, physical cores,
  threads a core), its caches (level, size, which CPUs share each), the CPU quota
  (cgroup `cpu.max`), the kernel and the hypervisor if any;
- perf: its version, `perf_event_paranoid`, the PMUs the kernel registered
  (`/sys/bus/event_source/devices`);
- each candidate event, counted on its own over a short busy loop: its count, or
  `<not supported>` / `<not counted>` / perf's error. An event that fails per process is
  tried machine-wide (`-a`), as uncore events need. The candidates are the generic
  hardware events, those the i9's study used, and their AMD counterparts;
- the raw Skylake-client offcore-response pair (prefetch-inclusive L3 hit and miss) only
  on a Skylake-client-derived model, since its encoding is that family's.

It measures nothing about mibz2 itself. It writes pmu-probe.json and prints a Markdown
report (to the job summary on CI).

  python3 bench/pmu_probe.py [--perf "sudo perf"] [--out pmu-probe.json]
"""

import argparse
import json
import os
import platform
import re
import shlex
import shutil
import subprocess
import sys

GENERIC = [
    "cycles",
    "instructions",
    "ref-cycles",
    "cache-references",
    "cache-misses",
    "branch-misses",
    "stalled-cycles-backend",
]
# the i9's study (Skylake-family names)
INTEL = [
    "mem_load_retired.l3_hit",
    "mem_load_retired.l3_miss",
    "offcore_requests.all_data_rd",
    "offcore_requests_outstanding.all_data_rd",
    "l1d_pend_miss.fb_full",
    "sw_prefetch_access.t0",
    "longest_lat_cache.reference",
    "longest_lat_cache.miss",
    "uncore_imc/data_reads/",
]
# AMD Zen 3 / Zen 4 counterparts: L3 lookups (the L3 PMU, machine-wide) and data-cache
# fills by source, demand and prefetch
AMD = [
    "l3_lookup_state.all_coherent_accesses_to_l3",
    "l3_lookup_state.l3_hit",
    "l3_lookup_state.l3_miss",
    "ls_dmnd_fills_from_sys.mem_io_local",
    "ls_any_fills_from_sys.mem_io_local",
    "ls_sw_pf_dc_fills.mem_io_local",
    "ls_hw_pf_dc_fills.mem_io_local",
]
# the i9's prefetch-inclusive L3 pair, by encoding (Intel's Skylake-client tables:
# ALL_DATA_RD 0x491 by L3_HIT or L3_MISS, any snoop); meaningful on that family only
SKYLAKE_CLIENT_MODELS = {0x4E, 0x5E, 0x8E, 0x9E, 0xA5, 0xA6}
RAW = {
    "ocr_all_data_rd_l3_hit": "cpu/event=0xb7,umask=0x1,offcore_rsp=0x3F803C0491,name=ocr_all_data_rd_l3_hit/",
    "ocr_all_data_rd_l3_miss": "cpu/event=0xb7,umask=0x1,offcore_rsp=0x3FBC000491,name=ocr_all_data_rd_l3_miss/",
}
BUSY = [sys.executable, "-c", "sum(i * i for i in range(2_000_000))"]


def read(path, default=""):
    try:
        return open(path).read().strip()
    except OSError:
        return default


def run(argv):
    try:
        return subprocess.run(argv, capture_output=True, text=True, timeout=60)
    except (OSError, subprocess.TimeoutExpired) as e:
        return subprocess.CompletedProcess(argv, 127, "", str(e))


def cpu():
    info = {}
    text = read("/proc/cpuinfo")
    for key, name in [
        ("vendor_id", "vendor"),
        ("model name", "model_name"),
        ("cpu family", "family"),
        ("model", "model"),
        ("stepping", "stepping"),
        ("flags", "flags"),
    ]:
        m = re.search(rf"^{re.escape(key)}\s*:\s*(.*)$", text, re.M)
        info[name] = m.group(1).strip() if m else ""
    info["hypervisor"] = "hypervisor" in info.pop("flags").split()
    info["cpus"] = os.cpu_count()
    cores, siblings = set(), set()
    for d in sorted(os.listdir("/sys/devices/system/cpu")):
        if re.fullmatch(r"cpu\d+", d):
            base = f"/sys/devices/system/cpu/{d}/topology"
            cores.add((read(f"{base}/physical_package_id"), read(f"{base}/core_id")))
            siblings.add(read(f"{base}/thread_siblings_list"))
    info["physical_cores"] = len(cores)
    info["thread_siblings"] = sorted(siblings)
    caches = []
    base = "/sys/devices/system/cpu/cpu0/cache"
    for d in sorted(os.listdir(base)) if os.path.isdir(base) else []:
        if d.startswith("index"):
            caches.append(
                {
                    "level": read(f"{base}/{d}/level"),
                    "type": read(f"{base}/{d}/type"),
                    "size": read(f"{base}/{d}/size"),
                    "shared_cpu_list": read(f"{base}/{d}/shared_cpu_list"),
                }
            )
    info["caches"] = caches
    info["cpu_max"] = read("/sys/fs/cgroup/cpu.max", "(no cgroup v2 cpu.max)")
    info["kernel"] = platform.release()
    virt = run(["systemd-detect-virt"])
    info["virtualisation"] = virt.stdout.strip() or "none"
    return info


def count(perf, event, system):
    """one event over the busy loop: (outcome, value)"""
    argv = [*perf, "stat", "-x,", "-e", RAW.get(event, event)]
    argv += ["-a"] if system else []
    r = run([*argv, "--", *BUSY])
    if r.returncode:
        return "error", r.stderr.strip().splitlines()[-1][:160] if r.stderr.strip() else "failed"
    for line in r.stderr.splitlines():
        f = line.split(",")
        if len(f) >= 3 and (f[2] == event or f[2].endswith(f"/{event}/") or event in f[2]):
            if f[0].startswith("<"):
                return f[0].strip("<>"), None
            try:
                return "counted", float(f[0])
            except ValueError:
                return "unparsed", f[0]
    return "absent", None


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--perf", default="sudo perf")
    ap.add_argument("--out", default="pmu-probe.json")
    a = ap.parse_args()
    perf = shlex.split(a.perf)

    c = cpu()
    v = run([*perf, "--version"])
    perf_ok = v.returncode == 0 and v.stdout.startswith("perf version")
    devices = (
        sorted(os.listdir("/sys/bus/event_source/devices"))
        if os.path.isdir("/sys/bus/event_source/devices")
        else []
    )
    report = {
        "cpu": c,
        "perf": v.stdout.strip() if perf_ok else (v.stderr.strip() or v.stdout.strip())[:300],
        "perf_available": perf_ok,
        "perf_event_paranoid": read("/proc/sys/kernel/perf_event_paranoid", "?"),
        "pmus": devices,
        "core_pmu": any(d in devices for d in ("cpu", "cpu_core", "cpu_atom")),
        "events": {},
    }
    candidates = GENERIC + (
        INTEL if "Intel" in c["vendor"] else AMD if "AMD" in c["vendor"] else []
    )
    try:
        model = int(c["model"])
    except ValueError:
        model = -1
    if "Intel" in c["vendor"] and c["family"] == "6" and model in SKYLAKE_CLIENT_MODELS:
        candidates += list(RAW)
    if perf_ok:
        for e in candidates:
            outcome, value = count(perf, e, system=False)
            mode = "per process"
            if outcome != "counted":
                outcome2, value2 = count(perf, e, system=True)
                if outcome2 == "counted":
                    outcome, value, mode = outcome2, value2, "machine-wide"
            report["events"][e] = {"outcome": outcome, "value": value, "mode": mode}
    json.dump(report, open(a.out, "w"), indent=1)

    p = print
    p(f"## PMU probe: {c['model_name']}")
    p()
    p(
        f"- **CPU:** {c['vendor']}, family {c['family']} model {c['model']} stepping {c['stepping']}; {c['cpus']} CPUs on {c['physical_cores']} physical cores (siblings {', '.join(c['thread_siblings'][:4])}{'…' if len(c['thread_siblings']) > 4 else ''})"
    )
    p(
        f"- **Caches (CPU 0's):** "
        + "; ".join(
            f"L{x['level']} {x['type'].lower()} {x['size']} shared by {x['shared_cpu_list']}"
            for x in c["caches"]
        )
    )
    p(
        f"- **Quota:** cgroup cpu.max `{c['cpu_max']}`; virtualisation {c['virtualisation']}; hypervisor flag {'set' if c['hypervisor'] else 'clear'}; kernel {c['kernel']}"
    )
    p(
        f"- **perf:** {report['perf'] if perf_ok else 'unavailable: ' + report['perf']}; perf_event_paranoid {report['perf_event_paranoid']}"
    )
    p(
        f"- **PMUs the kernel registered:** {', '.join(devices) or 'none'}; a core PMU {'is' if report['core_pmu'] else 'is **not**'} among them"
    )
    p()
    if report["events"]:
        p("| event | outcome | count over the busy loop | mode |")
        p("|---|---|--:|---|")
        for e, r in report["events"].items():
            val = "" if r["value"] is None or isinstance(r["value"], str) else f"{r['value']:.0f}"
            p(f"| `{e}` | {r['outcome']} | {val} | {r['mode']} |")
    else:
        p("No events tried: perf is not available here.")
    p()
    counted = [e for e, r in report["events"].items() if r["outcome"] == "counted"]
    p(
        f"**{len(counted)} of {len(report['events'])} candidate events counted.** Diagnostic only: nothing here gates a pull request."
    )


if __name__ == "__main__":
    if not shutil.which("python3"):
        sys.exit("pmu-probe: needs python3")
    main()
