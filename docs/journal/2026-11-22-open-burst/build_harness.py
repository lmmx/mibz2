"""Install the timing and parity tests into a disposable candidate worktree."""
import os
import subprocess
from pathlib import Path

study = Path(__file__).resolve().parent
os.chdir(os.environ["BENCH_TREE"])
base = "53f7ecf935d62ac7896eafaf55c5d249789614ca"


def read(path):
    return Path(path).read_text()


def write(path, body):
    Path(path).write_text(body)


def baseline(path):
    return subprocess.check_output(["git", "show", f"{base}:{path}"], text=True)


for name in ["reference", "control"]:
    directory = Path("src") / name
    directory.mkdir()
    write(directory / "mod.rs", baseline("src/open.rs").split("#[cfg(test)]\nmod checks;")[0])
    write(directory / "endpoint.rs", baseline("src/open/endpoint.rs").split("#[cfg(test)]\nmod tests {")[0])
    write(directory / "outq.rs", baseline("src/open/outq.rs"))

write("src/lib.rs", read("src/lib.rs") + """
#[cfg(test)] #[allow(dead_code)] mod reference;
#[cfg(test)] #[allow(dead_code)] mod control;
#[cfg(test)] mod timing;
#[cfg(test)] mod corpuscheck;
""")
write("src/open.rs", read("src/open.rs").replace("mod b5;", "pub(crate) mod b5;"))
write("src/open/b5.rs", read("src/open/b5.rs").replace("pub(super) fn plain", "pub(crate) fn plain"))

diet = baseline("docs/journal/2026-11-21-open-memory-diet/diet_harness.rs")
diet = diet[diet.index("type Block"):].replace("super::diet_ref", "crate::reference")
begin, arms = diet.split("fn decode(", 1)
arms = "fn decode(" + arms[:arms.index('        "scratch" =>')] + read(study / "modes.rs") + arms[arms.index('        "t1" =>'):]
timing = "use crate::runs::Runs;\nuse std::time::Instant;\n" + begin
for tag, mod in [("a", "control"), ("b", "open")]:
    timing += f"mod arm_{tag} {{\nuse super::*;\nuse crate::{mod}::{{Kernel, Counts, LIMIT}};\n"
    timing += arms.replace("ARM", mod) + "}\n"
write("src/timing.rs", timing)

admit = baseline("docs/journal/2026-11-21-open-dense-admission/admit_harness.rs")
corpus = "use crate::{runs::Runs, Kernels};\nuse crate::bwt::{InverseBwt, Plain};\nuse crate::stream::{Event, Scanner, SliceSource};\n"
corpus += admit[admit.index("pub(super) enum Blk"):admit.index("fn load_six")]
corpus += admit[admit.index("fn column("):admit.index("type Cand")]
corpus += read(study / "corpus.rs")
write("src/corpuscheck.rs", corpus)

# Build once without this feature for the headline. In a second binary, time
# each worker's decode_block call with its stream index in the real job queue.
write("Cargo.toml", read("Cargo.toml").replace("[features]", "[features]\nblock-clocks = []"))
worker = read("src/parallel.rs")
start = "        mark!(DecodeStart, place);"
end = "        mark!(DecodeEnd, place, out.len(), out.capacity());"
assert worker.count(start) == worker.count(end) == 1
worker = worker.replace(start, start + '\n        #[cfg(all(test, feature="block-clocks"))] let clock = std::time::Instant::now();')
worker = worker.replace(end, '#[cfg(all(test, feature="block-clocks"))] crate::corpuscheck::record(place, clock.elapsed());\n' + end)
write("src/parallel.rs", worker)
