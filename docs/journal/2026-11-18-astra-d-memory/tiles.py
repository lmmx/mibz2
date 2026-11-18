"""Size a run-translation table and price a first-to-last predecessor scan exactly."""
import csv
import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[3]/"bench"))
import irec

def geometry(rec, direction):
    runs = []
    cuts = [0]
    last = -1
    end = 0
    for c,w in zip(rec.a["l_byte"],rec.a["l_lam"]):
        if c != last:
            if end: cuts.append(end)
            runs.append((c, 0))
        runs[-1] = (c, runs[-1][1]+w)
        end += w
        last = c
    cuts.append(end)
    assert end == rec.n
    if direction == "psi":
        cuts = [0]
        for _, w in sorted(runs, key=lambda run: run[0]):
            cuts.append(cuts[-1]+w)
    return cuts

def price(cuts, tile):
    n = cuts[-1]
    b = sorted(set(cuts) | set(range(0,n,tile)))
    k = probes = 0
    for a,z in zip(b,b[1:]):
        k = 1 if a%tile == 0 else k+1
        probes += k*(z-a)
    return len(b)-1, probes

if __name__=="__main__":
    w = csv.writer(sys.stdout,lineterminator="\n")
    w.writerow(("ordinal","rows","runs","direction","tile","pieces","navigation_bytes","descriptor_probes","header_loads"))
    for p in sorted(Path(sys.argv[1]).glob("*.irec.xz")):
        rec = irec.load(p)
        assert rec.period == rec.n
        # 256 u32 cumulative symbol ends. Four bytes for every packed header and piece.
        for direction in ("lf","psi"):
            starts = geometry(rec, direction)
            for tile in (64,128,256):
                pieces,probes = price(starts,tile)
                nav = 4*(pieces+(rec.n+tile-1)//tile+256)
                w.writerow((rec.head["index"],rec.n,len(starts)-1,direction,tile,pieces,nav,probes,rec.n))
