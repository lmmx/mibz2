"""Run the actual gap constructor on capped-archive inputs and verify independent hashes."""
import csv
import hashlib
import sys
from pathlib import Path
from gaps import construct,expand

root=Path(__file__).resolve().parents[4]
study=root/"docs/journal/2026-11-18-astra-c-output-first"
sys.path.insert(0,str(study))
from inputs import read_input,paths
prior={int(r["block"]):r for r in csv.DictReader((study/"section.csv").open())}
geometry={int(r["stream"]):r for r in csv.DictReader((root/"research/astra-e/formulations/f2/geometry.csv").open())
          if r["delimiter_byte_values"]=="34."}
# Match the independent systematic tapes to the actual stream geometry using the
# existing verified F2b primary and size. Frozen has systematic index 0, stream index 1.
native={(int(r["rows"]),int(r["primary"])):int(r["stream"]) for r in csv.DictReader((root/"research/astra-e/formulations/f2b/geometry.csv").open())}
fields="n runs tokens mark_lookups reverse_probes success_steps discard_steps label_probes no_reuse_label_probes shared_depth token_runs word_count trie_nodes dict_bytes".split()
writer=csv.writer(sys.stdout,lineterminator="\n")
writer.writerow(["block","stream",*fields,"old_run_searches","old_merge_pieces","marked_sha256"])
for path in paths(sys.argv[1]):
    block=int(path.name.split(".")[0])
    runs,p=read_input(path)
    f,c=construct(runs,p,34)
    assert f is not None
    sha=hashlib.sha256(expand(f,c["n"],c["tokens"])).hexdigest()
    assert sha==prior[block]["marked_sha256"]
    stream=native[c["n"],p]
    old=geometry.get(stream)
    if old is not None:
        assert all(c[k]==int(old[v]) for k,v in [("tokens","token_rows"),("token_runs","token_runs"),("word_count","words"),("dict_bytes","dictionary_bytes")])
    writer.writerow([block,stream,*[c[k]for k in fields],old["run_searches"] if old else "",old["merge_pieces"] if old else "",sha])
    sys.stdout.flush()
