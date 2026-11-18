"""Prepare diagnostic-only hooks in an independent checkout of the exact 0.5.6 pin."""
from pathlib import Path
import subprocess
import sys

tree = Path(sys.argv[1])
here = Path(__file__).resolve().parent
pin = "1edb29fdc3d42dfddfc3d60e46456b611323760c"
assert subprocess.check_output(["git", "-C", tree, "rev-parse", "HEAD"], text=True).strip() == pin

def edit(file, pairs=(), more=""):
    path = tree / "src" / file
    text = subprocess.check_output(["git", "-C", tree, "show", f"{pin}:src/{file}"], text=True)
    for before, after in pairs:
        assert text.count(before) == 1, (file, before)
        text = text.replace(before, after)
    path.write_text(text + "\n" + more)

def mark(before, phase):
    return before, f"crate::astra_mark({phase});\n        {before}"

edit("lib.rs", [
    mark("if let Err(kind) = rle::unrle", 9),
    mark("let computed = crc::crc32(&out[start..], k);", 10),
], (here / "probe.rs").read_text())
edit("cells.rs", [
    mark("let result = self\n            .first_stage", 3),
    mark("self.induce_variant(budget, out)", 4),
    mark("self.write_owned(word, len, n, out);", 5),
    mark("self.state.fallback.invert_runs(r, n, p, out, k)", 8),
])
edit("cells_owned.rs", [
    mark("let tail = owned_scan(", 6),
    mark("owned_copies(base, dst, pos, ro, tail, nr, p);", 7),
])
edit("cells_lazy.rs", more="""
impl CellsVariant<LAZY> {
    pub(crate) fn astra_ranges(&self, out: usize) {
        use crate::{region, vector};
        let s = &self.0.state;
        let k = &s.kernel;
        let l = k.lay;
        let b = k.buf.as_ptr() as usize;
        vector("cellbuf", &k.buf);
        vector("positions", &k.pos);
        if s.took {
            region("tops", b + l.tops, l.ends[0] - l.tops);
            region("arena", b + l.arena, l.rules - l.arena);
            region("rules", b + l.rules, k.rule_end - l.rules);
            region("entries", out + BPAD, 4 * k.letters);
            region("front_bytes", out + k.lz.tby, k.letters);
            region("bottom", out + l.bots, k.lz.bend - l.bots);
        }
        s.fallback.astra_ranges();
        println!("info cells {} letters {} peak_pairs {} rules {}", s.took as u8,
            k.letters, k.lz.peak, (k.rule_end - l.rules) / 16);
    }
}
""")
edit("bwt.rs", more="""
impl Segmented {
    pub(crate) fn astra_ranges(&self) {
        use crate::vector;
        vector("tvector", &self.tt.0);
        vector("segments", &self.seg.bufs);
        vector("segments", &self.seg.bitmap);
        vector("segments", &self.seg.starts);
        vector("segments", &self.seg.next);
        for b in &self.seg.bufs { vector("segments", b); }
    }
}
""")
edit("runs.rs", [
    # Two diagnostic stores expose the addresses of stack arrays.
    # Record through globals once; read those globals after the measured block.
    ("let mut cnt = [0u64; 256];", """let mut cnt = [0u64; 256];
    ASTRA_ROWS.store(tail as usize, std::sync::atomic::Ordering::Relaxed);
    ASTRA_COUNT.store(cnt.as_ptr() as usize, std::sync::atomic::Ordering::Relaxed);""")
], more="""
use std::sync::atomic::{AtomicUsize, Ordering};
static ASTRA_ROWS: AtomicUsize = AtomicUsize::new(0);
static ASTRA_COUNT: AtomicUsize = AtomicUsize::new(0);
impl Runs {
    pub(crate) fn astra_ranges(&self) {
        use crate::{region, vector};
        vector("runs", &self.runs);
        vector("mtf_bits", &self.lit);
        vector("mtf_bits", &self.one);
        region("run_counts", self.counts.as_ptr() as usize, 1024);
        region("run_counts", self.per.as_ptr() as usize, 1024);
        region("mtf_rows", ASTRA_ROWS.load(Ordering::Relaxed), 224);
        region("mtf_counts", ASTRA_COUNT.load(Ordering::Relaxed), 2048);
        println!("info records {}", self.runs.len());
    }
}
""")
(tree / "examples" / "astra_trace.rs").write_text("""fn main() {
    let a: Vec<_> = std::env::args().collect();
    mibz2::astra_block(&a[1], &a[2], &a[3]);
}
""")
