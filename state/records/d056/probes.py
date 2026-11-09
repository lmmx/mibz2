import base64,hashlib,json,re,importlib.util
from pathlib import Path
inp=json.loads(Path("d056/input.json").read_text())
fixture=base64.b64decode(inp["fixture"])
def totals(text):
    rows=re.findall(r"team records (\d+) column \d+, small records (\d+) column \d+, cases with team records \d+, rebuilt (\d+)",text)
    return {"team":sum(int(x[0]) for x in rows),"small":sum(int(x[1]) for x in rows),"rebuilt":sum(int(x[2]) for x in rows),"candidates":len(rows)}
# Literal prefix model for one-word records after count_share/count_rows:
# every share is empty; each freshly cleared histogram is all-zero.
def prefix(word,n,count,shares,old=False):
    if old or n>=1<<24: return False
    if word>>8!=0 or word>>8!=n: return False
    slot=start=0
    for c in range(256):
        a=start
        for _ in range(shares):
            runs=rows=0
            slot+=runs
            a+=rows
        if a-start!=(count if c==7 else 0): return False
        start=a
    return slot==0 and start==n
accepted=[]
for w in range(512):
 for n in [0,1,2]:
  for count in [0,1]:
   for shares in range(1,6):
    actual=prefix(w,n,count,shares)
    expected=w<256 and n==0 and count==0
    assert actual==expected
    if actual: accepted.append([w,n,count,shares])
out={"fixture_bytes":len(fixture),"fixture_sha256":hashlib.sha256(fixture).hexdigest(),"fuzz_totals":{k:totals(v) for k,v in inp["logs"].items()},"singleton_cases":512*3*2*5,"accepted":len(accepted),"newly_accepted_words_at_n0_zero_counts":"0..255; one per supplied share count","canonical_oracle":"literal [0] only; 255 other low-byte sentinels differ","scope":"Python prefix arithmetic and log reconciliation; no Rust execution","ci_fuzz_totals":totals(inp["ci_fuzz"])}
Path("d056/results.json").write_text(json.dumps(out,indent=2)+"\n")
print(json.dumps(out))
