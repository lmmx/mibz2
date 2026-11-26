"""Real periodic cyclic BWTs attaining floor(n/f) expansions for every marker length."""
from itertools import groupby
from search import Column, heaviest


def check(marker_size, lap, copies):
    period = b"a" * marker_size + b"b" * (lap-marker_size)
    assert 0 < marker_size < lap
    rotations = sorted(range(lap), key=lambda i: period[i:]+period[:i])
    last = bytes(period[i-1] for i in rotations for _ in range(copies))
    runs = [(a,len(list(g))) for a,g in groupby(last)]
    col = Column(runs)
    section = col.interval(b"a"*marker_size)
    assert section[1] - section[0] == copies
    word, child, counters = heaviest(col,section)
    assert word == period and child == section
    assert counters["pops"] == lap == len(last)//copies
    assert counters["expanded_mass"] == len(last)
    return len(last)


if __name__ == "__main__":
    cases = rows = 0
    for marker in range(1,13):
        for tail in (1,2,5,17):
            for copies in (1,2,3,8,31):
                rows += check(marker,marker+tail,copies)
                cases += 1
    print(f"{cases} cyclic byte BWTs, {rows} total rows, n/f expansions attained in every case")
