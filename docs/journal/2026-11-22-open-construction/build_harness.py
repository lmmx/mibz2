"""Generate the #714 paired harness in a disposable master worktree (BENCH_TREE)."""

from pathlib import Path
import subprocess as sp
import os

root = Path(os.environ["BENCH_TREE"]).resolve()


def source(rev, p):
    return sp.check_output(["git", "show", rev + ":" + p], cwd=root, text=True)


def put(rel, s):
    p = root / rel
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(s)


def before(s, mark, code):
    assert s.count(mark) == 1, mark
    return s.replace(mark, code + "\n" + mark)


def record(i, v):
    return f"crate::timing::add({i}, {v}.elapsed());"


now = "std::time::Instant::now()"
import sys

arms = {
    "m": "8ff1187af6a920f7d57fd338680db3a5633a3ac5",
    "a": "8ff1187af6a920f7d57fd338680db3a5633a3ac5",
    "p": "26a5f6aed23a430c7447616782c0a2b6db946e2f",
}
arms.update(arg.split("=", 1) for arg in sys.argv[1:])
decl = ""
harness = source(
    "8ff1187af6a920f7d57fd338680db3a5633a3ac5",
    "docs/journal/2026-11-21-open-memory-diet/diet_harness.rs",
)
harness = (
    harness[: harness.index('        "scratch" =>')]
    + harness[harness.index('        "t1" =>') :]
)
setup = (
    harness[: harness.index("fn decode(")]
    .replace("use super::*;", "use crate::runs::Runs;\nuse std::time::Instant;")
    .replace(
        "use std::time::Instant;\nuse std::time::Instant;", "use std::time::Instant;"
    )
)
harness = (
    harness[harness.index("fn decode(") :]
    .replace("super::diet_ref", "crate::m")
    .replace("super::", "crate::")
)
setup += """pub(crate) static STAGES: [std::sync::atomic::AtomicU64; 7] = [const {std::sync::atomic::AtomicU64::new(0)}; 7];
pub(crate) fn add(i: usize, d: std::time::Duration) { STAGES[i].fetch_add(d.as_nanos() as u64, std::sync::atomic::Ordering::Relaxed); }
fn read_stages() -> [u64;7] { std::array::from_fn(|i| STAGES[i].swap(0, std::sync::atomic::Ordering::Relaxed)) }
fn corpus<K: crate::bwt::InverseBwt+'static>(data: &'static [u8], workers: usize) -> f64 {
    let t=Instant::now();
    let mut dec=crate::BzReader::parallel_workers_with::<K>(data, workers, 2*workers, crate::Kernels::detect());
    let len=std::io::copy(&mut dec,&mut std::io::sink()).unwrap();
    assert_eq!(len,209700196);
    t.elapsed().as_secs_f64()*1e6
}
"""
plain = source("8ff1187af6a920f7d57fd338680db3a5633a3ac5", "src/open/b5.rs")
mem = plain[
    plain.index("fn scratch(k:") : plain.index("/// Worst-case scratch")
].replace("fn scratch(", "pub(crate) fn scratch(")
for arm, rev in arms.items():
    for profile in [False, True]:
        name = (arm + "s" if arm != "a" else "aas") if profile else arm
        memsrc = source(rev, "src/open/b5.rs")
        mem = memsrc[
            memsrc.index("fn scratch(k:") : memsrc.index("/// Worst-case scratch")
        ].replace("fn scratch(", "pub(crate) fn scratch(")
        op = source(rev, "src/open.rs").split("#[cfg(test)]\nmod checks;")[0] + mem
        ep = source(rev, "src/open/endpoint.rs").split("#[cfg(test)]\nmod tests {")[0]
        if profile:
            op = before(
                op,
                "        out.resize(n as usize + INLINE, 0);",
                f"let first_clock={now};",
            )
            op = before(op, "        if over ||", record(0, "first_clock"))
            op = before(op, "        let First {\n", f"let setup_clock={now};")
            op = before(op, "            self.induced(", record(1, "setup_clock"))
            ep = before(ep, "        let Kernel {", f"let nodes_clock={now};")
            ep = before(
                ep,
                "        while !if spirals",
                record(1, "nodes_clock") + f"let elim_clock={now};",
            )
            ep = before(
                ep,
                "        stats.steps = steps;",
                record(2, "elim_clock") + f"let bands_clock={now};",
            )
            ep = before(ep, "        cursors\n", record(3, "bands_clock"))
            op = before(op, "        stats.bands = ", f"let wire_clock={now};")
            op = before(op, "        (stats, mark, bits)\n", record(4, "wire_clock"))
            if arm in ("m", "a"):
                op = before(op, "        inline_words(\n", f"let inline_clock={now};")
                op = before(op, "        emit(\n", record(5, "inline_clock"))
            op = before(op, "        self.walk(n,", f"let emit_clock={now};")
            op = before(op, "        Some(stats)\n", record(6, "emit_clock"))
        put(f"src/{name}/mod.rs", op)
        put(f"src/{name}/endpoint.rs", ep)
        put(f"src/{name}/outq.rs", source(rev, "src/open/outq.rs"))
        decl += f"#[cfg(test)] #[allow(dead_code)] mod {name};\n"
    if arm == "m":
        continue
    ext = """
        "parity" => {
            let mut kr = crate::m::Kernel::default();
            let mut k = Kernel::default();
            for b in &bs {
                let (mut orig, mut out)=(vec![],vec![]);
                decode_ref(&mut kr,b,&mut orig); decode(&mut k,b,&mut out);
                assert!(orig==out && out==crate::open::b5::plain(&b.1,b.2));
                println!("MEM {} {} {}", b.0,crate::m::scratch(&kr),crate::ARM::scratch(&k));
            }
        }
        "stage" => {
            let (mut kr,mut k)=(crate::ms::Kernel::default(),crate::ARMs::Kernel::default());
            let mut out=Vec::new();
            for b in &bs {
                let mut v=vec![[Vec::new(),Vec::new()];7];
                for i in 0..2*reps+4 {
                    let w=(i+i/2)%2;
                    if w==0 {kr.run::<true,true>(b.3.records(),b.1.len(),b.2,LIMIT,&mut out);}
                    else {k.run::<true,true>(b.3.records(),b.1.len(),b.2,LIMIT,&mut out);}
                    let d=read_stages();
                    if i>=4 {for s in 0..7 {v[s][w].push(d[s] as f64 / 1000.);}}
                }
                for (s,[a,bv]) in v.iter_mut().enumerate() {println!("ST {}.{} {:.3} {:.3}",b.0,s,median(a),median(bv));}
            }
        }
        "corpus" | "cs" => {
            let data: &'static [u8]=Box::leak(std::fs::read(std::env::var("WIKIDATA").unwrap()).unwrap().into_boxed_slice());
            let workers:usize=std::env::var("WORKERS").unwrap().parse().unwrap();
            let reverse:usize=std::env::var("REVERSE").unwrap().parse().unwrap();
            let mut pair=[[0.;8];2];
            for i in 0..2 {let w=(reverse+i)%2;
                let wall=match (mode.as_str(),w) {
                    ("cs",0)=>corpus::<crate::ms::OpenCells>(data,workers),
                    ("cs",_)=>corpus::<crate::ARMs::OpenCells>(data,workers),
                    (_,0)=>corpus::<crate::m::OpenCells>(data,workers),
                    _=>corpus::<crate::ARM::OpenCells>(data,workers),
                };
                pair[w][0]=wall;
                for (s,n) in read_stages().into_iter().enumerate() {pair[w][s+1]=n as f64/1000.;}
            }
            for j in 0..if mode=="cs"{8}else{1} {println!("CORP {}.{} {:.2} {:.2}",workers,j,pair[0][j],pair[1][j]);}
        }
"""
    ext = (
        """
        "stage4" => {
            let group=std::sync::Barrier::new(4);
            // A pass is four simultaneous decodes of this block. Read counters
            // only after the last worker stops, before the next pass starts.
            std::thread::scope(|scope| {
                for worker in 0..4 {
                    let (group,bs)=(&group,&bs);
                    scope.spawn(move || {
                        let (mut kr,mut k)=(crate::ms::Kernel::default(),crate::ARMs::Kernel::default());
                        let mut out=Vec::new();
                        for b in bs {
                            let mut samples=vec![[Vec::new(),Vec::new()];7];
                            for i in 0..2*reps+4 {
                                let w=(i+i/2)%2;
                                group.wait();
                                if w==0 {kr.run::<true,true>(b.3.records(),b.1.len(),b.2,LIMIT,&mut out);}
                                else {k.run::<true,true>(b.3.records(),b.1.len(),b.2,LIMIT,&mut out);}
                                group.wait();
                                if worker==0 {
                                    let d=read_stages();
                                    if i>=4 {for s in 0..7 {samples[s][w].push(d[s] as f64/4000.);}}
                                }
                            }
                            if worker==0 {for (s,[a,bv]) in samples.iter_mut().enumerate() {println!("ST {}.{} {:.3} {:.3}",b.0,s,median(a),median(bv));}}
                        }
                    });
                }
            });
        }

"""
        + ext
    )
    setup += (
        f"mod arm_{arm} {{\nuse super::*;\nuse crate::{arm}::{{Kernel, Counts, LIMIT}};\n"
        + harness.replace(
            '        _ => panic!("DIET_MODE"),',
            ext.replace("ARMs", "aas" if arm == "a" else arm + "s").replace("ARM", arm)
            + '\n        _ => panic!("DIET_MODE"),',
        )
        + "}\n"
    )
put("src/timing.rs", setup)
put(
    "src/lib.rs",
    source("8ff1187af6a920f7d57fd338680db3a5633a3ac5", "src/lib.rs")
    + "\n"
    + decl
    + "#[cfg(test)] mod timing;\n",
)
put(
    "src/open.rs",
    source("8ff1187af6a920f7d57fd338680db3a5633a3ac5", "src/open.rs").replace(
        "mod b5;", "pub(crate) mod b5;"
    ),
)
# b5::plain is already independently used by the parent module's tests
put("src/open/b5.rs", plain.replace("pub(super) fn plain", "pub(crate) fn plain"))


# Head to head comparisons reuse the same compiled kernels and change only the calling harness.
t = root / "src/timing.rs"
s = t.read_text()
for arm, prev in zip(list(arms)[3:], list(arms)[2:]):
    prev = os.environ.get("PARENT_" + arm, prev)
    name = f"mod arm_{arm} {{"
    start = s.index(name)
    sub = s[start:].split("\nmod arm_", 1)[0]
    s += "\n" + sub.replace(name, f"mod arm_{arm}v {{").replace(
        "crate::m::", f"crate::{prev}::"
    ).replace("crate::ms::", f"crate::{prev}s::")
t.write_text(s)
p = root / "src/lib.rs"
p.write_text(p.read_text() + "\n#[cfg(test)] mod corpuscheck;\n")
h = source(
    "8ff1187af6a920f7d57fd338680db3a5633a3ac5",
    "docs/journal/2026-11-21-open-dense-admission/admit_harness.rs",
)
ss = (
    "use crate::{runs::Runs, Kernels};\nuse crate::bwt::{InverseBwt, Plain};\nuse crate::stream::{Event,Scanner,SliceSource};\n"
    + h[h.index("pub(super) enum Blk") : h.index("fn load_six")]
    + h[h.index("fn column(") : h.index("type Cand")]
)
ss += """
#[test]
#[ignore]
fn all_blocks() {
    let blocks=load_bz2(&std::env::var("WIKIDATA").unwrap());
    assert_eq!(blocks.len(),431);
    let mut base=crate::m::OpenCells::default();
    let mut candidates: Vec<Box<dyn Adapter>>=vec![ARMS];
    let (mut expected,mut actual)=(vec![],vec![]);
    for b in &blocks {
        dec(&mut base,b,&mut expected);
        for candidate in &mut candidates { candidate.decode(b,&mut actual);assert!(expected==actual); }
        if let Blk::Run{idx,p,runs,..}=b {
            if [41,56,165,270,278].contains(idx) {
                let dir=std::env::var("DENSE_DIR").unwrap();
                std::fs::write(format!("{dir}/dense-{idx}.L"),column(runs)).unwrap();
                println!("DENSE {idx} {p}");
            }
        }
    }
}
trait Adapter { fn decode(&mut self,b:&Blk,out:&mut Vec<u8>); }
impl<I:InverseBwt> Adapter for I { fn decode(&mut self,b:&Blk,out:&mut Vec<u8>) {dec(self,b,out);} }
""".replace(
    "ARMS",
    ",".join(f"Box::new(crate::{k}::OpenCells::default())" for k in list(arms)[2:]),
)
put("src/corpuscheck.rs", ss)

# The pilot binaries used the seven original phase counters.
if os.environ.get("PROFILE_VERSION") == "screen":
    sys.exit(0)

for name in arms:
    prof = "aas" if name == "a" else name + "s"
    file = root / "src" / prof / "endpoint.rs"
    ep = file.read_text()
    a = "        e.assign_bottom(stats.retained as u32);"
    if a in ep:
        ep = ep.replace(
            a,
            "        let bottom_clock=std::time::Instant::now();\n"
            + a
            + "\n        crate::timing::add(7,bottom_clock.elapsed());",
        )
    else:
        first = "        let mut id = L::next(e.a, 0, 1);"
        end = "        assert_eq!(x as usize, stats.retained);"
        assert first in ep and end in ep
        ep = ep.replace(
            first, "        let bottom_clock=std::time::Instant::now();\n" + first
        ).replace(end, end + "\n        crate::timing::add(7,bottom_clock.elapsed());")
    file.write_text(ep)
    if name in ("a", "m"):
        op = root / "src" / prof / "mod.rs"
        body = op.read_text()
        body = body.replace("let emit_clock=std::time::Instant::now();\n", "").replace(
            "crate::timing::add(6, emit_clock.elapsed());\n", ""
        )
        pos = body.index("        emit(\n", body.index("    fn walk("))
        end = body.index("        );", pos) + len("        );")
        op.write_text(
            body[:pos]
            + "let emit_clock=std::time::Instant::now();\n"
            + body[pos:end]
            + "\ncrate::timing::add(6,emit_clock.elapsed());"
            + body[end:]
        )
t = root / "src/timing.rs"
s = t.read_text()
for a, b in [
    ("; 7]", "; 8]"),
    (";7]", ";8]"),
    ("0..7 {", "0..8 {"),
    ("[[0.;8];2]", "[[0.;9];2]"),
    ('mode=="cs"{8}', 'mode=="cs"{9}'),
    (
        "let workers = 4;",
        'let workers: usize = std::env::var("WORKERS").unwrap().parse().unwrap();',
    ),
]:
    s = s.replace(a, b)
t.write_text(s)
