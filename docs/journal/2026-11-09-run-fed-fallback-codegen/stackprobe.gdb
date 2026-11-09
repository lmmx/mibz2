# gdb -q -batch -x stackprobe.gdb --args target/release/examples/run_fed_stage corpora/wikidata.bz2 2 3.6937
# Stops in mibz2::mtf::mtf_avx2 just after its prologue (offset 0x4b1eb in the aa409f6f build;
# 0x4accb in the fd60fffa build) and records, per call site, the counts pointer (rbx) and the
# realigned frame base (rsp); rows = rsp + 0x40, its 224-byte second-level list.
python
import gdb
seen = {}
class BP(gdb.Breakpoint):
    def stop(self):
        f = gdb.newest_frame()
        rsp = int(gdb.parse_and_eval("$rsp")); counts = int(gdb.parse_and_eval("$rbx"))
        older = f.older()
        caller = older.name() if older else "?"
        # the caller of the caller, to tell the entry sweep from the phase sweep
        o2 = older.older() if older else None
        c2 = o2.name() if o2 else "?"
        key = (caller, c2, counts, rsp)
        seen[key] = seen.get(key, 0) + 1
        return False
gdb.execute("set disable-randomization off"); gdb.execute("starti")
base = None
for l in gdb.execute("info proc mappings", to_string=True).splitlines():
    if "run_fed_stage" in l:
        base = int(l.split()[0], 16); break
BP("*%#x" % (base + 0x4b1eb), internal=True)
gdb.execute("set pagination off")
gdb.execute("continue")
for (caller, c2, counts, rsp), n in sorted(seen.items(), key=lambda kv: (kv[0][2], kv[0][3])):
    R = rsp
    rows = R + 0x40
    print(f"STACK caller={caller} <- {c2} hits={n} counts={counts:#x} rsp_entry={rsp:#x} rows={rows:#x} rows-counts={(rows - counts):#x} rows%4K={rows%4096:#x} counts%4K={counts%4096:#x}")
end
