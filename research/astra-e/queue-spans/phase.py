"""Build a diagnostic copy with thread-CPU stamps bracketing the three construction phases."""
from pathlib import Path
import re
import sys

here = Path(__file__).resolve().parent
out = Path(sys.argv[1]).resolve()
out.mkdir(parents=True, exist_ok=True)
s = (here / "kernel.rs").read_text()
s = re.sub(r'#\[path = "([^"]+)"\]', lambda m: '#[path = "' + str((here/m[1]).resolve()) + '"]', s)
s = re.sub(r'include!\("([^"]+)"\)', lambda m: 'include!("' + str((here/m[1]).resolve()) + '")', s)
def changed(old, new):
    global s
    assert s.count(old) == 1, old
    s = s.replace(old, new)

changed("    pub page_cuts: usize,", "    pub page_cuts: usize,\n    pub phase: [u64; 3],")
changed("        let mut stats = Counts {", "        let begin = crate::cpu_ns();\n        let mut stats = Counts {")
changed("        stats.retained = m as usize;", "        let classified = crate::cpu_ns();\n        stats.retained = m as usize;")
changed("        stats.bands = self.bands.len();", "        let induced = crate::cpu_ns();\n        stats.bands = self.bands.len();")
changed("        (stats, mark, bits)", "        stats.phase = [classified - begin, induced - classified, crate::cpu_ns() - induced];\n        (stats, mark, bits)")
(out/"kernel.rs").write_text(s)
stage = (here/"stage.rs").read_text()
helpers = stage[stage.index("const ORDINALS:"):stage.index("struct Arm")] + stage[stage.index("fn cpu_ns"):stage.index("fn main")]
(out/"phase.rs").write_text('''use mibz2::stream::{Event, Scanner, SliceSource};
pub use mibz2::{bwt, runs, Kernels};
#[allow(dead_code, unexpected_cfgs)]
mod kernel;
''' + helpers + '''
fn lap<const MODE: u8>(k: &mut kernel::Kernel, b: &Block, out: &mut Vec<u8>) -> [u64;4] {
    let begin = cpu_ns();
    let stats = k.run::<true, MODE, false>(b.runs.records(), b.n, b.primary, 16, out);
    let total = cpu_ns() - begin;
    assert!(out == &b.reference);
    [stats.phase[0], stats.phase[1], stats.phase[2], total - stats.phase.iter().sum::<u64>()]
}
fn main() {
    let data = std::fs::read(std::env::args().nth(1).unwrap()).unwrap();
    let blocks = sample(&data, Kernels::detect());
    let mut arms: [kernel::Kernel;5] = Default::default();
    let mut outs: [Vec<u8>;5] = Default::default();
    println!("round,position,ordinal,mode,classify_and_queue_ns,induction_ns,project_edges_ns,emit_and_call_ns");
    for pass in 0..=20usize {
        for (i,b) in blocks.iter().enumerate() {
            for pos in 0..5 {
                let rot = pass.saturating_sub(1) + i;
                let mode = (rot + if rot / 5 % 2 == 0 {pos} else {4-pos}) % 5;
                let k = &mut arms[mode];
                let out = &mut outs[mode];
                let a = match mode {
                    0 => lap::<0>(k,b,out),
                    1 => lap::<1>(k,b,out),
                    2 => lap::<2>(k,b,out),
                    3 => lap::<3>(k,b,out),
                    _ => lap::<4>(k,b,out),
                };
                if pass > 0 {
                    println!("{},{pos},{},{mode},{},{},{},{}",pass-1,b.ordinal,a[0],a[1],a[2],a[3]);
                }
            }
        }
    }
}
''')
