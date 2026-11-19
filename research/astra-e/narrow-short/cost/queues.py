"""Use existing dense row queues to recover the two per-byte row counters."""
from pathlib import Path
import shutil, sys

def build(src, dest):
    shutil.copytree(src,dest,dirs_exist_ok=True)
    path=dest/"research/astra-e/narrow-short/constructor_ahead.rs"
    s=path.read_text()
    def change(old,new):
        nonlocal s
        assert s.count(old)==1,old
        s=s.replace(old,new)
    change("                thin[a] += w;","                if !QUEUE { thin[a] += w; }")
    change("                    to: all[a],","                    to: if QUEUE { all[a] + self.queues[a].len() as u32 } else { all[a] },")
    change("            all[a] += w;","            if !QUEUE || !keep { all[a] += w; }")
    change("        assert!(m > 0 && all.iter()", """        if QUEUE {
            // The ordered u32 payload already counts retained rows. While scanning,
            // all[a] counted omitted thick rows only. Reconstitute both histograms.
            for a in 0..256 {
                thin[a] = self.queues[a].len() as u32;
                all[a] += thin[a];
            }
        }
        assert!(m > 0 && all.iter()""")
    path.write_text(s)

if __name__=="__main__":
    build(Path(sys.argv[1]),Path(sys.argv[2]))
