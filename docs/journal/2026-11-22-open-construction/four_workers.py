"""Measure Open scratch in the actual four decoder workers when the reader drops them."""

from pathlib import Path
import sys

w = Path(__file__).parent
exec(compile((w / "memory_harness.py").read_text(), "memory_harness.py", "exec"))
mem = (root / "src/memo.rs").read_text()
for arg in sys.argv[1:]:
    a, rev = arg.split("=")
    file = root / "src" / a / "mod.rs"
    with file.open("a") as f:
        f.write("""
impl<const FORCED: bool> Drop for OpenCellsOf<FORCED> {
    fn drop(&mut self) { crate::memo::record(scratch(&self.open)); }
}
""")
mem += (
    """
static WORKERS: std::sync::Mutex<Vec<usize>> = std::sync::Mutex::new(Vec::new());
pub(crate) fn record(bytes: usize){WORKERS.lock().unwrap().push(bytes);}
fn worker_pass<K: InverseBwt + 'static>(name: &str, data: &'static [u8]) {
 WORKERS.lock().unwrap().clear();
 let mut reader=crate::BzReader::parallel_workers_with::<K>(data,4,8,Kernels::detect());
 let n=std::io::copy(&mut reader,&mut std::io::sink()).unwrap();
 assert_eq!(n,209700196);drop(reader);
 let mut scratch=WORKERS.lock().unwrap().clone();
 scratch.sort();
 println!("FOUR {name} {scratch:?}");
}
#[test] fn four_workers() {
 let data=Box::leak(std::fs::read(std::env::var("WIKIDATA").unwrap()).unwrap().into_boxed_slice());
"""
    + "".join(
        f'worker_pass::<crate::{arg.split("=")[0]}::OpenCells>("{arg.split("=")[0]}",data);\n'
        for arg in sys.argv[1:]
    )
    + "}\n"
)
(root / "src/memo.rs").write_text(mem)
