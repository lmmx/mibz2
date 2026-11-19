"""Materialize an isolated direct-row constructor, exact check and crossover driver."""
from pathlib import Path
import re
import sys

here = Path(__file__).resolve().parent
base = here.parent
out = Path(sys.argv[1]).resolve()
out.mkdir(parents=True, exist_ok=True)


def edit(s, old, new):
    assert s.count(old) == 1, old
    return s.replace(old, new)


def locate(s, exceptions=()):
    def resolve(m):
        name = m[1]
        return m[0] if name in exceptions else m[0].replace(name, str((base / name).resolve()))
    return re.sub(r'(?:#\[path = "|include!\(")([^"]+)"', resolve, s)


kernel = locate((base / "kernel.rs").read_text())
anchor = "impl Kernel {"
kernel = edit(kernel, anchor, (here / "append.rs").read_text() + "\n" + anchor)
kernel = edit(kernel, "} else if QUEUE == 1 {\n                        self.queues[a].extend",
              "} else if QUEUE >= 5 {\n                        append_rows::<QUEUE>(&mut self.queues[a], m, w);\n                    } else if QUEUE == 1 {\n                        self.queues[a].extend")
kernel = edit(kernel, "} else if QUEUE == 1 {\n                        let src",
              "} else if QUEUE == 1 || QUEUE >= 5 {\n                        let src")
(out / "kernel.rs").write_text(kernel)
(out / "candidate.rs").write_text((base / "candidate.rs").read_text())
check = (base / "check.rs").read_text()
old = "            check::<3>(kernel, &rec, col.len(), p as u32, limit, &reference);"
check = edit(check, old, "\n".join(old.replace("<3>", f"<{mode}>") for mode in [5, 6, 7]))
check = edit(check, "all four modes", "u32 rows, raw spans, rescan and all three direct append modes")
(out / "check.rs").write_text(check)
stage = locate((base / "stage.rs").read_text(), ["candidate.rs"])
start = stage.index("        arm::<bwt::LazyCells>")
end = stage.index("    ];", start)
stage = stage[:start] + """        arm::<preserved::Candidate>("preserved", kernels),
        arm::<spans::Variant<1>>("rows", kernels),
        arm::<spans::Variant<5>>("single", kernels),
        arm::<spans::Variant<6>>("four", kernels),
        arm::<spans::Variant<7>>("hybrid", kernels),
""" + stage[end:]
(out / "stage.rs").write_text(stage)
