from pathlib import Path
import subprocess as sp, sys, os

root = Path(os.environ["BENCH_TREE"])


def src(rev, file):
    return sp.check_output(["git", "show", rev + ":" + file], cwd=root, text=True)


def put(file, s):
    p = root / "src" / file
    p.parent.mkdir(exist_ok=True)
    p.write_text(s)


h = src(
    "8ff1187af6a920f7d57fd338680db3a5633a3ac5",
    "docs/journal/2026-11-21-open-dense-admission/admit_harness.rs",
)
check = (
    "use crate::{runs::Runs, Kernels};\nuse crate::bwt::{InverseBwt, Plain};\nuse crate::stream::{Event,Scanner,SliceSource};\n"
    + h[h.index("use std::alloc") : h.index("fn load_six")]
    + h[h.index("fn column(") : h.index("type Cand")]
)
block = """
{let mut k=crate::ARM::OpenCells::default();let mut out=Vec::with_capacity(900_016);
 let base=live();let mut high=0; for b in &blocks {dec(&mut k,b,&mut out);high=high.max(crate::ARM::open_scratch(&k));}
 println!("FULL ARM {} {} {}", crate::ARM::open_scratch(&k),high,live()-base);}
"""
decl = ""
checks = ""
for kv in sys.argv[1:]:
    k, rev = kv.split("=")
    op = src(rev, "src/open.rs").split("#[cfg(test)]\nmod checks;")[0]
    mem = (
        src(rev, "src/open/b5.rs")
        .split("fn scratch(k:")[1]
        .split("/// Worst-case scratch")[0]
    )
    put(
        k + "/mod.rs",
        op
        + "\nfn scratch(k:"
        + mem
        + "\npub(crate) fn open_scratch(k: &OpenCells) -> usize {scratch(&k.open)}\n",
    )
    put(
        k + "/endpoint.rs",
        src(rev, "src/open/endpoint.rs").split("#[cfg(test)]\nmod tests {")[0],
    )
    put(k + "/outq.rs", src(rev, "src/open/outq.rs"))
    decl += f"#[cfg(test)] #[allow(dead_code)] mod {k};\n"
    checks += block.replace("ARM", k)
put(
    "lib.rs",
    src("8ff1187af6a920f7d57fd338680db3a5633a3ac5", "src/lib.rs")
    + "\n"
    + decl
    + "\n#[cfg(test)] #[allow(dead_code, unused_imports)] mod memo;\n",
)
put(
    "memo.rs",
    check
    + '\n#[test] fn corpus_memory() {let blocks=load_bz2(&std::env::var("WIKIDATA").unwrap());assert_eq!(blocks.len(),431);'
    + checks
    + "}\n",
)
