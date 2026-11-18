"""Apply the admitted experiment to an already prepared disposable trace tree."""
from pathlib import Path
import sys
tree = Path(sys.argv[1])
here = Path(__file__).resolve().parent
p = tree/"src/lib.rs"
s = p.read_text()
needle = "let done = s.inverse.invert_runs(&s.runs, n, b.orig_ptr, &mut s.ll, k);"
assert s.count(needle)==1
s = s.replace(needle, """// Only the empty, owned worker slot. At most two extra marked letters.
        let direct = k.avx2 && out.is_empty() && n < 1<<20 &&
            128 + 13*(s.runs.records().len()+2) <= n+64;
        if direct {
            out.reserve(n+64);
            if s.inverse.invert_runs(&s.runs,n,b.orig_ptr,out,k) {
                return finish_slot(s,out,b.crc,k);
            }
            out.clear();
        }
        """ + needle)
# For the aliases at this lap, including induction's use of the final allocation.
s = s.replace("scratch.inverse.astra_ranges(scratch.ll.as_ptr() as usize);",
"""let n = scratch.ll.len();
    let direct = n == 0; // same valid block twice, fresh scratch per envelope
    scratch.inverse.astra_ranges(if direct { output.as_ptr() } else { scratch.ll.as_ptr() } as usize);
    println!("info direct {}", direct as u8);""")
p.write_text(s + (here/"direct.rs").read_text())
p = tree/"src/runs.rs"
p.write_text(p.read_text() + """
impl Runs {
    pub(crate) fn astra_directory(&mut self) -> &mut [u16] { &mut self.runs }
}
""")
