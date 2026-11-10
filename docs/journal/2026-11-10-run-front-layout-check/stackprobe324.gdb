# Adapted from docs/journal/2026-11-09-run-fed-fallback-codegen/stackprobe.gdb: stops in
# mibz2::mtf::mtf_avx2 just after its prologue (the instruction after `mov %rcx,%rbx`), and
# records per call site the counts pointer (rbx) and the realigned frame base (rsp).
# PROBE_OFF (env) is that instruction's offset in the binary.
python
import gdb, os
off = int(os.environ["PROBE_OFF"], 16)
seen = {}
class BP(gdb.Breakpoint):
    def stop(self):
        f = gdb.newest_frame()
        rsp = int(gdb.parse_and_eval("$rsp")); counts = int(gdb.parse_and_eval("$rbx"))
        older = f.older(); caller = older.name() if older else "?"
        o2 = older.older() if older else None; c2 = o2.name() if o2 else "?"
        key = (caller, c2, counts, rsp)
        seen[key] = seen.get(key, 0) + 1
        return False
gdb.execute("set disable-randomization off"); gdb.execute("starti")
base = None
for l in gdb.execute("info proc mappings", to_string=True).splitlines():
    if l.strip().endswith("/ab"):
        base = int(l.split()[0], 16); break
BP("*%#x" % (base + off), internal=True)
gdb.execute("set pagination off")
gdb.execute("continue")
for (caller, c2, counts, rsp), n in sorted(seen.items(), key=lambda kv: (kv[0][2], kv[0][3])):
    rows = rsp + 0x40
    lo, hi = (rsp - counts) % 4096, (rsp + 0x1c0 - counts) % 4096
    # does counts' 1 KiB window [0, 0x400) mod 4K meet the frame [rsp, rsp+0x1c0) mod 4K?
    fr = [(rsp + i - counts) % 4096 for i in range(0, 0x1c0)]
    hit = sum(1 for x in fr if x < 0x400)
    print(f"STACK caller={caller} <- {c2} hits={n} counts={counts:#x} rsp={rsp:#x} frame-counts%4K={(rsp-counts)%4096:#x} rows-counts%4K={(rows-counts)%4096:#x} frame_bytes_in_counts_window={hit}")
end
