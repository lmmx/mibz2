"""Insert clocks around the original record expansion and first complete transform."""
import subprocess
from pathlib import Path

d = Path(__file__).resolve().parent
subprocess.run(["python3", str(d/"stage.py")], check=True)
tmp = d / "target"
instrumented = tmp / "instrumented.rs"
source = (d/"candidate.rs").read_text().replace("impl Solver {", """
thread_local! {
    pub(super) static FIRST: std::cell::Cell<(u128,u64,u32)> = const { std::cell::Cell::new((0,0,0)) };
}
impl Solver {""")
source = source.replace("        self.a.runs.clear();", """
        let cpu_start = crate::cpu_ns();
        let start = std::time::Instant::now();
        self.a.runs.clear();""")
source = source.replace("        for _ in 0..8 {", "        for level in 0..8 {").replace(
    "            p = q;", """            p = q;
            if level == 0 {
                let wall = start.elapsed().as_nanos();
                let cpu = crate::cpu_ns()-cpu_start;
                FIRST.with(|cell| cell.set((wall,cpu,next)));
            }""")
assert "FIRST.with(" in source
instrumented.write_text(source)
stage = (tmp/"stage.rs").read_text().replace(str(d/"candidate.rs"), str(instrumented))
stage = stage.replace("elapsed_ns,thread_cpu_ns", "elapsed_ns,thread_cpu_ns,first_ns,first_cpu_ns,token_rows")
stage = stage.replace("                if let Some(elapsed) = elapsed {","""                if let Some(elapsed) = elapsed {
                    let f = if arm.name == "phrases" { phrases::FIRST.with(|c| c.get()) } else { (0,0,0) };""")
stage = stage.replace('"{},{},{},{},{},{},{},{},{}"','"{},{},{},{},{},{},{},{},{},{},{},{}"')
stage = stage.replace('elapsed, cpu.unwrap()','elapsed, cpu.unwrap(), f.0, f.1, f.2')
(tmp/"phase.rs").write_text(stage)
