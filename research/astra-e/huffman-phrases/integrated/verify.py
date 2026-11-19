"""Check gate/malformed receipts and compare the actual rustc dependency closure."""
import hashlib
import json
from pathlib import Path
import re
import shlex
import sys

out=Path(sys.argv[1]).resolve()
build=Path(sys.argv[2]).resolve()
def sha(b):return hashlib.sha256(b).hexdigest()
arms={}
for arm in ("stock","phrase"):
    tree=build/arm
    summary=json.loads((out/arm/"summary.json").read_text())
    rows=list(map(json.loads,(out/arm/"check.jsonl").read_text().splitlines()))
    assert rows[-1]==dict(arm="candidate",verdict="PASS",failures=0)
    assert all(r["pass"] and r["failure"] is None for r in rows[:-1])
    assert summary["gate"]==dict(verdict="PASS",cases=len(rows)-1,failed=[],exit=0) and len(rows)>=339
    assert any(r["bytes"]==209700196 and "wikidata" in r["case"] for r in rows[:-1])
    log=(out/(arm+"-generated-test.txt")).read_text()
    assert "test block::tests::symbols_read_as_in_memory ... ok" in log and "test result: ok. 1 passed" in log
    expected=out/("hashes-"+arm+".txt")
    binary=build/(arm+"-target/release/moonshot")
    assert expected.read_text().splitlines()[0].split()[0]==sha(binary.read_bytes())
    deps=binary.with_suffix(".d").read_text().splitlines()[0]
    files=[Path(n).resolve()for n in shlex.split(deps.split(": ",1)[1])]
    module={}
    for f in files:
        if not f.is_relative_to(tree) or not f.is_file():continue
        relative=str(f.relative_to(tree));b=f.read_bytes()
        # The scalar generator embeds its checkout in exactly three Rust module paths.
        if relative=="research/astra-e/front-fusion/cap-batch/worker/kernel.rs":
            expr=rb'(#\[path\s*=\s*")'+re.escape(str(tree).encode())+rb'(?=/)'
            b,n=re.subn(expr,lambda m:m[1]+b"$DECODER",b);assert n==3
        module[relative]=sha(b)
    assert {"src/block.rs","src/bits.rs","src/parallel.rs","research/astra-e/front-fusion/cap-batch/worker/kernel.rs"}<=module.keys()
    arms[arm]=dict(binary=sha(binary.read_bytes()),canonical_module_sha256=module,cases=len(rows)-1,
                   version=summary["scorer"],arm=summary["info"]["candidate"],reported_cpus=summary["info"]["cpus"])
assert arms["stock"]["binary"]!=arms["phrase"]["binary"]
assert arms["stock"]["version"]==arms["phrase"]["version"]
assert (out/"refresh-stock.txt").read_bytes()==(out/"refresh-phrase.txt").read_bytes()
x,y=(arms[a]["canonical_module_sha256"]for a in ("stock","phrase"))
assert x.keys()==y.keys() and [k for k in x if x[k]!=y[k]]==["src/block.rs"]
print(json.dumps(dict(arms=arms,only_different_crate_or_research_source="src/block.rs",
                      fetched_and_scorer_projection=(out/"refresh-stock.txt").read_text().splitlines()),indent=2))
