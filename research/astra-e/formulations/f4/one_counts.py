"""Build an untimed copy of the one-pass comparator and count actual allocated lengths."""
import os
from pathlib import Path
import subprocess

d = Path(__file__).resolve().parent
root = d.parents[3]
tmp = d / "target"
# one.sh generates both source files. Keep their measured hashes unchanged.
s = (tmp/"instrumented_one.rs").read_text()
begin = '        assert!(!out.is_empty() && out.len() <= n);'
assert s.count(begin) == 1
s = s.replace(begin, '''        eprintln!("counts,{},{},{},{},{},{},{},{},{},{},{},{},{}", n, m, self.a.runs.len(),
            self.build.spans.len(),self.build.prefix.len(),self.build.trie.len()-1,
            self.a.offsets.len()-1,self.a.bytes.len(),self.table.len()*4,self.wide.len()*4,
            self.backward.len(),skip,out.len());
''' + begin)
(tmp/"one_count_kernel.rs").write_text(s)
s = (tmp/"one.rs").read_text().replace("instrumented_one.rs","one_count_kernel.rs")
(tmp/"one_count_main.rs").write_text(s)
deps = Path(os.environ.get("CARGO_TARGET_DIR", str(root/"target")))/"release/deps"
subprocess.run(["rustc","--edition=2021","-O","-L",f"dependency={deps}","--extern",f"mibz2={deps}/libmibz2.rlib",
                str(tmp/"one_count_main.rs"),"-o",str(tmp/"one_count")],check=True)
r = subprocess.run([str(tmp/"one_count"),str(root/"corpora/wikidata.bz2"),"0"],capture_output=True,text=True,check=True)
ordinals = [1,19,61,108,150,194,240,282,329,371,407]
print("ordinal,original_rows,token_rows,token_runs,input_overlays,prefix_intervals,trie_nodes,token_alphabet,"
      "dictionary_bytes,table_bytes,wide_code_bytes,marked_links,carried_skip,output_period")
for idx,line in zip(ordinals,r.stderr.splitlines(),strict=True):
    tag,rest=line.split(",",1)
    assert tag=="counts"
    print(f"{idx},{rest}")
