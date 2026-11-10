"""The team path's kernels against the one-worker path's, counted: instructions by stage.

docs/journal/2026-11-10-team-kernel-gap-count.md. A count, never a clock: callgrind's Ir
(instructions executed), from profiles made with

    valgrind --tool=callgrind --dump-instr=yes --separate-threads=yes \\
        --callgrind-out-file=DIR/ARM.%p target/release/examples/ab child ARM THREADS FILE

one file a thread (`ARM.PID-NN`). This script reads them, with no valgrind tool of its own:

- **self cost** a function (its own instructions, inlined code included) and **call arcs**
  (caller, callee, calls, inclusive Ir), so that a library routine's cost (memcpy, memmove,
  memset, the allocator) goes to the stage of the function that called it, caller by
  caller, not to "other";
- **per-instruction execution counts** in the cooperative walk (`coop::traverse`): with
  `--dump-instr`, a cost line's Ir at one instruction address is the times it ran. The
  segment claims are the executions of its `lock xadd` (the counter's fetch_add), and the
  bytes it walked the executions of its byte stores, split by source line into the full
  phase's, the tail's and the first byte of each segment taken (`objdump -d -l` gives each
  instruction's text and line).

    python3 bench/team_gap_count.py report DIR/one DIR/team --binary AB --rows N --decoded D

(the files may be gzipped, PREFIX.PID-NN.gz, as the journal entry archives them)

prints the stage table for both arms (Ir a decoded byte, Ir a row), the caller split of
the libc routines, and the team arm's per-thread table. Stages are named by function
(`STAGES`); a library routine's arc takes its caller's stage (`LIBRARY`).
"""

import argparse
import collections
import glob
import gzip
import re
import subprocess
import sys

# (stage, regex on the demangled function name), first match wins
STAGES = [
    ("harness", r"^ab::|examples/ab|<ab::|ab::Hash|ab::Threads|ab::drain|ab::child|ab::decode"),
    ("MTF", r"mibz2::mtf::"),
    (
        "build",
        r"mibz2::bwt::build|build_one_pass|build_two_pass|FusedBuild|coop::build_fused|coop::check_inputs",
    ),
    (
        "walk",
        r"room_segments|mibz2::bwt::walk|coop::traverse|RoomWalk|coop::walk_room|coop::research::walk_one_caller|bwt::random_rows|walk_segments",
    ),
    ("assembly", r"SegScratch>::assemble|SegScratch::assemble"),
    ("RLE1", r"mibz2::rle::"),
    ("CRC", r"mibz2::crc::"),
    ("scanner", r"mibz2::stream::|mibz2::block::|mibz2::bits::|mibz2::read::scan|Scanner"),
    (
        "coordination",
        r"coop::Relay|coop::wait_until|Relay>::wait|Relay>::join|Relay>::set_walk_ready|coop::helper|coop::Pool|sched_yield|futex|park|Condvar|condvar|Parker|parking|pthread_cond|__lll_lock|lll_|__GI___lll|mibz2::parallel::.*wait",
    ),
]
STAGE_ORDER = [
    "scanner",
    "MTF",
    "build",
    "walk",
    "assembly",
    "RLE1",
    "CRC",
    "coordination",
    "harness",
    "other",
]
# library routines whose cost goes to the stage of the function calling them (the sort is
# the walk's sort of its segment starts)
LIBRARY = r"core::slice::sort::|memcpy|memmove|memset|__memcpy|__memmove|__memset|malloc|realloc|free|calloc|__rust_alloc|__rust_realloc|__rust_dealloc|__rdl_|_int_malloc|_int_free|_int_realloc|RawVec|raw_vec|alloc::alloc|finish_grow|reserve|do_reserve_and_handle|cfree|__libc_malloc|__libc_free|__libc_realloc|madvise|mmap|munmap|brk|sysmalloc|malloc_consolidate|unlink_chunk|tcache"


def stage_of(name):
    for st, rx in STAGES:
        if re.search(rx, name):
            return st
    return None


def is_library(name):
    return re.search(LIBRARY, name) is not None


class Profile:
    """One callgrind file (one thread): self Ir by function, arcs, and Ir by instruction"""

    def __init__(self, path):
        self.path = path
        self.self_ir = collections.Counter()
        self.arcs = collections.Counter()  # (caller, callee) -> inclusive Ir
        self.calls = collections.Counter()  # (caller, callee) -> calls
        self.instr = collections.Counter()  # (fn, addr) -> Ir (= executions)
        self.instr_line = {}  # (fn, addr) -> (file, line)
        self.total = 0
        self.thread = None
        self.cmd = None
        self._parse()

    def _parse(self):
        names = {"fn": {}, "fl": {}, "ob": {}}
        positions = ["line"]
        fn = cfn = None
        fl = fi = None
        last = [0, 0]
        pending_call = None

        def name(kind, rest):
            m = re.match(r"\((\d+)\)(?: (.*))?$", rest)
            if not m:
                return rest
            ident, nm = m.group(1), m.group(2)
            if nm is not None:
                names[kind][ident] = nm
                return nm
            return names[kind][ident]

        def pos(tokens):
            out = []
            for i, t in enumerate(tokens):
                if t == "*":
                    v = last[i]
                elif t[0] in "+-":
                    v = last[i] + int(t, 0)
                else:
                    v = int(t, 0)
                out.append(v)
            return out

        opener = gzip.open if self.path.endswith(".gz") else open
        with opener(self.path, "rt") as f:
            for raw in f:
                line = raw.rstrip("\n")
                if not line or line[0] == "#":
                    continue
                c = line[0]
                if c.isdigit() or c in "+-*":
                    parts = line.split()
                    npos = len(positions)
                    p = pos(parts[:npos])
                    ir = int(parts[npos]) if len(parts) > npos else 0
                    last[:npos] = p
                    if pending_call is not None:
                        self.arcs[(fn, pending_call[0])] += ir
                        self.calls[(fn, pending_call[0])] += pending_call[1]
                        pending_call = None
                    else:
                        self.self_ir[fn] += ir
                        if positions[0] == "instr":
                            key = (fn, p[0])
                            self.instr[key] += ir
                            if npos > 1:
                                self.instr_line[key] = (fi or fl, p[1])
                    continue
                m = re.match(
                    r"^(positions|events|summary|totals|thread|cmd|pid|part|version|creator|desc): ?(.*)$",
                    line,
                )
                if m:
                    key, rest = m.group(1), m.group(2)
                    if key == "positions":
                        positions = rest.split()
                        last = [0] * len(positions)
                    elif key in ("summary", "totals"):
                        self.total = int(rest.split()[0])
                    elif key == "thread":
                        self.thread = int(rest.strip())
                    elif key == "cmd":
                        self.cmd = rest.strip()
                    continue
                key, _, rest = line.partition("=")
                if key == "positions":
                    positions = rest.split()
                    last = [0] * len(positions)
                elif key == "fn":
                    fn = name("fn", rest)
                elif key == "cfn":
                    cfn = name("fn", rest)
                elif key in ("fl",):
                    fl = name("fl", rest)
                    fi = None
                elif key in ("fi", "fe"):
                    fi = name("fl", rest)
                elif key in ("cfi", "cfl"):
                    name("fl", rest)
                elif key in ("ob", "cob"):
                    name("ob", rest)
                elif key == "calls":
                    pending_call = (cfn, int(rest.split()[0]))
                elif key == "totals" or key == "summary":
                    self.total = int(rest.split()[0])
                elif key == "thread":
                    self.thread = int(rest.strip())
                elif key == "cmd":
                    self.cmd = rest.strip()


def stages(profile):
    """Ir by stage: each function's self cost to its stage, each library routine's
    inclusive arc to its caller's stage; the rest "other"; and the library split by caller"""
    out = collections.Counter()
    lib_split = collections.Counter()  # (callee, caller stage) -> Ir
    for fn, ir in profile.self_ir.items():
        st = stage_of(fn) if fn else None
        if st and not is_library(fn):
            out[st] += ir
    for (caller, callee), ir in profile.arcs.items():
        if not callee or not is_library(callee):
            continue
        st = stage_of(caller) if caller and not is_library(caller) else None
        if st:
            out[st] += ir
            lib_split[(callee, st, caller)] += ir
    out["other"] = profile.total - sum(out.values())
    return out, lib_split


def disasm(binary, symbol_rx):
    """{addr: (text, file, line)} for the functions whose demangled name matches"""
    res = subprocess.run(
        ["objdump", "-d", "-l", "-C", "--no-show-raw-insn", binary],
        capture_output=True,
        text=True,
        check=True,
    ).stdout
    out, inside, fileline = {}, False, (None, None)
    for ln in res.splitlines():
        m = re.match(r"^[0-9a-f]+ <(.*)>:$", ln)
        if m:
            inside = re.search(symbol_rx, m.group(1)) is not None
            continue
        if not inside:
            continue
        m = re.match(r"^(/.*|[^\s].*):(\d+)( \(discriminator \d+\))?$", ln)
        if m:
            fileline = (m.group(1), int(m.group(2)))
            continue
        m = re.match(r"^\s+([0-9a-f]+):\s+(.*)$", ln)
        if m:
            out[int(m.group(1), 16)] = (m.group(2).strip(), fileline[0], fileline[1])
    return out


BYTE_STORE = re.compile(r"^mov\s+%(al|bl|cl|dl|sil|dil|bpl|spl|r\d+b),\s*.*\(")


def traverse_counts(profile, dis, lines):
    """Claims, and bytes walked by kind, in one thread's coop::traverse: `lines` maps a
    coop.rs line to its kind (full, tail, take)"""
    claims = 0
    walked = collections.Counter()
    for (fn, addr), ir in profile.instr.items():
        if not fn or "coop::traverse" not in fn or addr not in dis:
            continue
        text, file, line = dis[addr]
        if text.startswith("lock xadd"):
            claims += ir
        elif BYTE_STORE.match(text) and file and file.endswith("coop.rs") and line in lines:
            walked[lines[line]] += ir
    return claims, walked


def coop_lines(src):
    """coop.rs's byte-store lines in `traverse`, by kind: found by their text"""
    kinds = {}
    with open(src) as f:
        text = f.read().splitlines()
    for i, ln in enumerate(text, 1):
        s = ln.strip()
        if s == "unsafe { *base = q as u8 };":
            kinds[i] = "take"
        elif s == "unsafe { *base[k].wrapping_add(off) = q as u8 };":
            kinds[i] = "full"
        elif s == "*l.p = q as u8;":
            kinds[i] = "tail"
        elif s == "*tp.add(k * TILE + j) = q as u8;":
            # (traverse_staged's tile store, in its macro's body)
            kinds[i] = "full"
    return kinds


def load(prefix):
    # (callgrind leaves an empty PREFIX.PID beside the per-thread PREFIX.PID-NN files)
    files = sorted(f for f in glob.glob(prefix + ".*") if re.search(r"\.\d+-\d+(\.gz)?$", f))
    return [Profile(f) for f in files]


def report(args):
    arms = [("one worker", load(args.one)), ("team", load(args.team))]
    n, d = args.rows, args.decoded
    tables = {}
    for name, profs in arms:
        tot = collections.Counter()
        lib = collections.Counter()
        for p in profs:
            st, ls = stages(p)
            tot.update(st)
            lib.update(ls)
        tables[name] = (tot, lib, sum(p.total for p in profs), profs)
    print(f"denominators: D = {d:,} decoded bytes (after RLE1), N = M = {n:,} rows\n")
    print("| stage | one worker Ir | Ir/D | team Ir | Ir/D | team - one, Ir/D |")
    print("|---|--:|--:|--:|--:|--:|")
    for st in STAGE_ORDER:
        a, b = tables["one worker"][0][st], tables["team"][0][st]
        print(f"| {st} | {a:,} | {a / d:.2f} | {b:,} | {b / d:.2f} | {(b - a) / d:+.2f} |")
    a, b = tables["one worker"][2], tables["team"][2]
    print(f"| total | {a:,} | {a / d:.2f} | {b:,} | {b / d:.2f} | {(b - a) / d:+.2f} |")
    print("\nfunctions by stage (self Ir, all threads; Ir/D), the top of each:")
    for name, (tot, lib, total, profs) in tables.items():
        selfs = collections.Counter()
        for p in profs:
            selfs.update(p.self_ir)
        print(f"\n## {name}")
        for st in STAGE_ORDER[:-1]:
            fns = [
                (f, ir) for f, ir in selfs.items() if f and stage_of(f) == st and not is_library(f)
            ]
            fns.sort(key=lambda x: -x[1])
            items = ", ".join(f"{short(f)} {ir / d:.3f}" for f, ir in fns[:8] if ir / d >= 0.0005)
            print(f"- {st}: {items}")
        print("- library routines by caller (inclusive Ir/D):")
        for (callee, st, caller), ir in sorted(lib.items(), key=lambda x: -x[1]):
            if ir / d >= 0.005:
                print(f"  - {short(callee)} <- {short(caller)} [{st}]: {ir / d:.3f}")
        others = collections.Counter()
        for f, ir in selfs.items():
            if not f or stage_of(f) is None or is_library(f):
                others[f] += ir
        print(
            "- largest self costs not given a stage (library routines appear here by self cost; their arcs above):"
        )
        for f, ir in others.most_common(12):
            print(f"  - {short(f)}: {ir / d:.3f}")
    # the team's threads
    dis = disasm(args.binary, r"coop::traverse")
    lines = coop_lines(args.coop_src)
    print("\n## team arm, by thread")
    print(
        "| thread | total Ir | MTF | build | walk | assembly | RLE1+CRC | coordination | build shares | walk calls | walk claims | walked: take | full phase | tail |"
    )
    print("|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|")
    sums = collections.Counter()
    for p in tables["team"][3]:
        st, _ = stages(p)
        claims, walked = traverse_counts(p, dis, lines)
        shares = sum(
            c
            for (caller, callee), c in p.calls.items()
            if callee and "FusedBuild" in callee and "share" in callee
        )
        walks = sum(
            c for (caller, callee), c in p.calls.items() if callee and "coop::traverse" in callee
        )
        sums.update(walked)
        sums["claims"] += claims
        print(
            f"| {p.thread} | {p.total:,} | {st['MTF']:,} | {st['build']:,} | {st['walk']:,} | {st['assembly']:,} | "
            f"{st['RLE1'] + st['CRC']:,} | {st['coordination']:,} | {shares} | {walks} | {claims:,} | {walked['take']:,} | {walked['full']:,} | {walked['tail']:,} |"
        )
    print(
        f"\nall threads: claims {sums['claims']:,}, walked bytes take {sums['take']:,} + full {sums['full']:,} + tail {sums['tail']:,} = {sums['take'] + sums['full'] + sums['tail']:,}"
    )


def walker_counts(profile, dis, lines):
    """One thread's walk, by walker (`traverse` and `traverse_staged`): calls, claims, bytes
    walked by kind, and the staged walker's flush fragments (its calls to memcpy, empty
    ones included) and their inclusive Ir"""
    out = {}
    for variant, rx in (
        ("old", r"coop::traverse(?!_staged)"),
        ("staged", r"coop::traverse_staged"),
    ):
        claims = 0
        walked = collections.Counter()
        for (fn, addr), ir in profile.instr.items():
            if not fn or not re.search(rx, fn) or addr not in dis:
                continue
            text, file, line = dis[addr]
            if text.startswith("lock xadd"):
                claims += ir
            elif BYTE_STORE.match(text) and file and file.endswith("coop.rs") and line in lines:
                walked[lines[line]] += ir
        calls = sum(
            c for (caller, callee), c in profile.calls.items() if callee and re.search(rx, callee)
        )
        frag = sum(
            c
            for (caller, callee), c in profile.calls.items()
            if caller and re.search(rx, caller) and callee and "memcpy" in callee
        )
        frag_ir = sum(
            ir
            for (caller, callee), ir in profile.arcs.items()
            if caller and re.search(rx, caller) and callee and "memcpy" in callee
        )
        out[variant] = dict(
            calls=calls, claims=claims, walked=walked, fragments=frag, flush_ir=frag_ir
        )
    return out


def staged(args):
    """The staged walk's count: several arms on one input, each its stage table (Ir a
    decoded byte), its inclusive walk and inversion, and each team arm's walkers by thread"""
    d, n = args.decoded, args.rows
    dis = disasm(args.binary, r"coop::traverse")
    lines = coop_lines(args.coop_src)
    arms = []
    for spec in args.arm:
        name, prefix = spec.split("=", 1)
        profs = load(prefix)
        if not profs:
            sys.exit(f"no profiles at {prefix}")
        tot = collections.Counter()
        for p in profs:
            st, _ = stages(p)
            tot.update(st)
        arms.append((name, profs, tot, sum(p.total for p in profs)))
    print(f"denominators: D = {d:,} decoded bytes, N = {n:,} rows\n")
    print("| stage | " + " | ".join(f"{a[0]} Ir/D" for a in arms) + " |")
    print("|---|" + "--:|" * len(arms))
    for st in STAGE_ORDER:
        print(f"| {st} | " + " | ".join(f"{a[2][st] / d:.3f}" for a in arms) + " |")
    print(
        "| **inversion (build + walk + assembly)** | "
        + " | ".join(f"{(a[2]['build'] + a[2]['walk'] + a[2]['assembly']) / d:.3f}" for a in arms)
        + " |"
    )
    print("| **whole process** | " + " | ".join(f"{a[3] / d:.3f}" for a in arms) + " |")
    print("\nraw Ir:")
    for name, profs, tot, total in arms:
        print(f"- {name}: total {total:,}; " + ", ".join(f"{st} {tot[st]:,}" for st in STAGE_ORDER))
    print("\nwalk functions (self Ir/D; library arcs from them, inclusive Ir/D):")
    for name, profs, tot, total in arms:
        selfs = collections.Counter()
        arcs = collections.Counter()
        for p in profs:
            selfs.update(p.self_ir)
            for (caller, callee), ir in p.arcs.items():
                if caller and callee and stage_of(caller) == "walk" and is_library(callee):
                    arcs[(short(callee), short(caller))] += ir
        fns = sorted(
            (
                (f, ir)
                for f, ir in selfs.items()
                if f and stage_of(f) == "walk" and not is_library(f)
            ),
            key=lambda x: -x[1],
        )
        print(
            f"- {name}: "
            + ", ".join(f"{short(f)} {ir / d:.3f}" for f, ir in fns if ir / d >= 0.0005)
        )
        for (callee, caller), ir in arcs.most_common():
            if ir / d >= 0.0005:
                print(f"  - {callee} <- {caller}: {ir / d:.3f}")
    for name, profs, tot, total in arms:
        rows = []
        for p in profs:
            st, _ = stages(p)
            w = walker_counts(p, dis, lines)
            shares = sum(
                c
                for (caller, callee), c in p.calls.items()
                if callee and "FusedBuild" in callee and "share" in callee
            )
            if not any(w[v]["calls"] for v in w) and not shares:
                continue
            rows.append((p, st, w, shares))
        if not rows:
            continue
        print(f"\n## {name}, by thread (threads that built or walked)")
        print(
            "| thread | total Ir | build | walk | build shares | walker | walk calls | claims | take | full phase | tail | fragments | flush Ir |"
        )
        print("|---|--:|--:|--:|--:|---|--:|--:|--:|--:|--:|--:|--:|")
        sums = collections.Counter()
        for p, st, w, shares in rows:
            for v in ("old", "staged"):
                c = w[v]
                if not c["calls"]:
                    continue
                wk = c["walked"]
                print(
                    f"| {p.thread} | {p.total:,} | {st['build']:,} | {st['walk']:,} | {shares} | {v} | {c['calls']} | {c['claims']:,} | {wk['take']:,} | {wk['full']:,} | {wk['tail']:,} | {c['fragments']:,} | {c['flush_ir']:,} |"
                )
                for key in ("calls", "claims", "fragments", "flush_ir"):
                    sums[v + key] += c[key]
                for key in ("take", "full", "tail"):
                    sums[v + key] += wk[key]
        for v in ("old", "staged"):
            if sums[v + "calls"]:
                m = sums[v + "take"] + sums[v + "full"] + sums[v + "tail"]
                print(
                    f"\n{v}: calls {sums[v + 'calls']:,}, claims {sums[v + 'claims']:,}, walked take {sums[v + 'take']:,} + full {sums[v + 'full']:,} + tail {sums[v + 'tail']:,} = M {m:,}; tail share {sums[v + 'tail'] / max(m, 1):.4f}; fragments {sums[v + 'fragments']:,}; flush {sums[v + 'flush_ir'] / d:.3f} Ir/D"
                )


def short(f):
    f = re.sub(r"<[^<>]*>", "", f or "?") if f and len(f) > 90 else (f or "?")
    return f.replace("mibz2::", "")


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    r = sub.add_parser("report")
    r.add_argument(
        "one", help="the one-worker arm's callgrind file prefix (ARM files: PREFIX.PID-NN)"
    )
    r.add_argument("team", help="the team arm's prefix")
    r.add_argument("--binary", required=True, help="the profiled ab binary (for objdump)")
    r.add_argument("--coop-src", default="src/coop.rs")
    r.add_argument("--rows", type=int, required=True)
    r.add_argument("--decoded", type=int, required=True)
    g = sub.add_parser("staged", help="the staged walk's arms on one input")
    g.add_argument("--arm", action="append", required=True, help="NAME=PREFIX, repeated")
    g.add_argument("--binary", required=True)
    g.add_argument("--coop-src", default="src/coop.rs")
    g.add_argument("--rows", type=int, required=True)
    g.add_argument("--decoded", type=int, required=True)
    args = ap.parse_args()
    if args.cmd == "report":
        report(args)
    elif args.cmd == "staged":
        staged(args)


if __name__ == "__main__":
    sys.exit(main())
