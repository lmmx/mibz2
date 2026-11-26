"""Derive a three-arm phase comparison from the pinned checked stage harness."""
import subprocess
from pathlib import Path

here = Path(__file__).resolve().parent
root = here.parents[3]
source = subprocess.check_output(["git", "show", "0f25dee737432feafb03b8b14dc10afbf5c141ad:examples/astra_e.rs"], text=True)
lo,hi = source.index("#[allow(dead_code)]"), source.index("const ORDINALS")
source = source[:lo] + f"""#[allow(dead_code)]
#[path = "{root}/docs/journal/2026-11-18-open-return/open_return.rs"]
mod open_return;
#[path = "{here}/candidate.rs"]
mod phrases;
""" + source[hi:]
lo,hi = source.index("    let mut arms = ["), source.index("    let count = arms.len();")
source = source[:lo] + """    let mut arms = [
        arm::<bwt::LazyCells>("lazy", kernels),
        arm::<open_return::Open<16,true,true>>("opend16", kernels),
        arm::<phrases::Candidate>("phrases", kernels),
    ];
""" + source[hi:]
clock = subprocess.check_output(["git","show","cec3a58635582885d9062c92e65a363c5fdff599:research/astra-e/stage2/clock.inc.rs"],text=True)
source = source.replace("fn main() {", clock + "\nfn main() {")
source = source.replace("let begin = (pass > 0).then(Instant::now);","let cpu_start = (pass > 0).then(cpu_ns);\n                let begin = (pass > 0).then(Instant::now);")
source = source.replace("let elapsed = begin.map(|start| start.elapsed().as_nanos());","let elapsed = begin.map(|start| start.elapsed().as_nanos());\n                let cpu = cpu_start.map(|t| cpu_ns()-t);")
source = source.replace("records,arm,elapsed_ns", "records,arm,elapsed_ns,thread_cpu_ns")
source = source.replace('"{},{},{},{},{},{},{},{}"', '"{},{},{},{},{},{},{},{},{}"').replace('                        elapsed\n','                        elapsed, cpu.unwrap()\n')
assert "cpu.unwrap()" in source
target = here / "target"
target.mkdir(exist_ok=True)
(target / "stage.rs").write_text(source)
