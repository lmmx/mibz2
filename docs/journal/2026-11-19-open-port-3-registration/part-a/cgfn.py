"""Sum a cachegrind out-file's events per function (demangled by name), and in total."""
import collections, json, subprocess, sys

def load(path):
    ev, fn, per = None, None, collections.defaultdict(lambda: None)
    tot = None
    with open(path) as f:
        for line in f:
            if line.startswith("events:"):
                ev = line.split()[1:]
            elif line.startswith("fn="):
                fn = line[3:].strip()
            elif line.startswith(("fl=", "fi=", "fe=", "desc:", "cmd:", "summary:")):
                if line.startswith("summary:"):
                    tot = [int(x) for x in line.split()[1:]]
            elif line[:1].isdigit() and fn is not None:
                xs = [int(x) for x in line.split()[1:]]
                xs += [0] * (len(ev) - len(xs))
                cur = per[fn]
                per[fn] = xs if cur is None else [a + b for a, b in zip(cur, xs)]
    return ev, dict(per), tot

def demangle(names):
    out = subprocess.run(["c++filt"], input="\n".join(names), capture_output=True, text=True)
    return out.stdout.splitlines()

if __name__ == "__main__":
    ev, per, tot = load(sys.argv[1])
    names = list(per)
    dm = demangle(names)
    rows = sorted(zip(dm, (per[n] for n in names)), key=lambda r: -r[1][0])
    for d, xs in rows[: int(sys.argv[2]) if len(sys.argv) > 2 else 40]:
        print(f"{xs[0]:>14,} {d[:160]}")
    print("total", dict(zip(ev, tot)))
