"""Input-only screen for contractions inside first-column byte buckets."""
from collections import Counter


def buckets(bands):
    sizes = Counter()
    for _, _, w, c in bands:
        sizes[c] += w
    seen = Counter()
    i = begin = 0
    for c, size in sorted(sizes.items()):
        end = begin + size
        row = begin
        left = seen[c]
        pieces = []
        while row < end:
            x,y,w,b = bands[i]
            take = min(x+w, end) - row
            pieces.append((row,take,b,y+row-x))
            seen[b] += take
            row += take
            i += row == x+w
        yield c,begin,end,left,pieces
        begin = end


def sides(c, begin, end, left, pieces):
    """Each result is oriented from the exterior of the bucket toward its fixed rows."""
    exits = sum(w for _,w,b,_ in pieces if b != c)
    wanted = left
    split = begin
    for x,w,b,_ in pieces:
        if not wanted:
            break
        if b == c:
            continue
        take = min(w,wanted)
        split = x+take
        wanted -= take
    assert not wanted
    for low, high, reverse, active in [(begin,split,False,left), (split,end,True,exits-left)]:
        spans = []
        for x,w,b,_ in pieces:
            lo, hi = max(low,x), min(high,x+w)
            if lo < hi:
                inside = b == c
                if spans and spans[-1][2] == inside:
                    spans[-1] = (spans[-1][0],hi,inside)
                else:
                    spans.append((lo,hi,inside))
        if reverse:
            spans.reverse()
        yield low,high,reverse,active,spans


def census(bands):
    count = Counter(chunks=0,inside_rows=0,fixed_rows=0,full_turns=0,
                    full_turn_rows=0,partial_turns=0,exit_groups=0,peak_live=0)
    for c, begin, end, left, pieces in buckets(bands):
        count['chunks'] += len(pieces)
        for _,_,_,active,spans in sides(c,begin,end,left,pieces):
            count['peak_live'] = max(active,count['peak_live'])
            for lo,hi,inside in spans:
                w = hi-lo
                if inside:
                    count['inside_rows'] += w
                    if active:
                        turns, rest = divmod(w, active)
                        count['full_turns'] += turns
                        count['full_turn_rows'] += turns*active
                        count['partial_turns'] += bool(rest)
                    else:
                        count['fixed_rows'] += w
                else:
                    count['exit_groups'] += 1
                    active -= w
                    assert active >= 0
            assert active == 0
    return dict(count)


if __name__ == '__main__':
    import sys
    from pathlib import Path
    from screen import inputs
    for block, n, _, _, bands in inputs(Path(sys.argv[1]), [0]):
        print(block,n,census(bands))
