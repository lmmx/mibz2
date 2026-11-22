"""Install the paired benchmark in a disposable checkout of the master base."""
import sys
from pathlib import Path
from shutil import copyfile

journal = Path(__file__).resolve().parent
master, fused, relay = (Path(path).resolve() for path in sys.argv[1:])
base = master / "src/open"
if (base / "trial.rs").exists():
    sys.exit("use a clean disposable master worktree")
copyfile(journal / "bench.rs", base / "trial.rs")

# Full decodes always go through the original OpenCells entry points. These
# test-only methods expose a prepared table so the clock can surround emit alone.
stage_template = """
impl STAGE for Kernel {
    fn prepare(&mut self,r:&Runs,n:usize,p:u32,out:&mut Vec<u8>)->Option<(usize,u32,u32,Option<u32>)> {
        let cap = OpenCells::cap(r.records().len(),n,p)?;
        let META=self.construct_capped::<true,true>(r.records(),n as u32,p,LIMIT,cap,out)?;
        self.walk(n,c.retained,mark,bits,PREV_ARG out);
        Some((c.retained,mark,bits,PREV))
    }
    fn emit_only(&self, (m,mark,bits,_prev):(usize,u32,u32,Option<u32>),n:usize,out:&mut Vec<u8>) {
        SPECIAL
        emit(&self.edges[..m],&self.extra,&self.spelling,&self.offsets,&self.inline16,&self.lens,n,mark,bits,out);
    }
}
"""

for name, root in [("control", master), ("xor", fused), ("relay6", relay), ("reference", master)]:
    source = root / "src/open.rs"
    kernel = source.read_text().split("#[cfg(test)]\nmod checks;")[0]
    dest = base / name
    is_reference = name == "reference"
    is_xor = name == "xor"
    special = ""
    if is_xor:
        special = """if let Some(prev)=_prev {
            return two_ended::emit(&self.edges[..m],&self.spelling,&self.offsets,
                &self.inline16,&self.lens,n,mark,prev,bits,out);
        }"""
    if name == "relay6":
        special = """if m>=relay::MIN_ROWS && self.extra.is_empty() && !self.relay_lengths.contains(&u16::MAX) {
            return relay::emit(&self.edges[..m],&self.spelling,&self.offsets,&self.inline16,
                &self.lens,&self.relay_lengths,n,mark,bits,out);
        }"""
    stage = (stage_template
             .replace("STAGE", "trial::Stage" if is_reference else "super::trial::Stage")
             .replace("META", "(c,mark,bits,prev)" if is_xor else "(c,mark,bits)")
             .replace("PREV_ARG", "prev," if is_xor else "")
             .replace("PREV", "prev" if is_xor else "None")
             .replace("SPECIAL", special))

    if is_reference:
        with source.open("a") as f:
            for module in ["control", "xor", "relay6", "trial"]:
                f.write(f"\n#[cfg(test)]\n#[allow(dead_code)]\nmod {module};\n")
            f.write("\n#[cfg(test)]\n" + stage)
        continue

    dest.mkdir()
    (dest / "mod.rs").write_text(kernel + stage)
    for path in (root / "src/open").glob("*.rs"):
        if f"mod {path.stem};" in kernel:
            body = path.read_text()
            if path.stem in ("two_ended", "relay"):
                body = body.split("#[cfg(test)]\nmod tests {")[0]
            (dest / path.name).write_text(body)

with (base / "relay6/mod.rs").open("a") as f:
    f.write("impl OpenCells { pub(super) fn length_bytes(&self) -> usize { self.open.relay_lengths.capacity() * 2 } }\n")
