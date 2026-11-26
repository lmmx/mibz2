"""Dense stable-permutation oracle for selected complete returns and nested word cylinders."""
from array import array
from collections import Counter
import csv
from itertools import groupby, product
from pathlib import Path
import sys
from search import Column, heaviest


class Oracle:
    def __init__(self, last):
        self.forward = array("I", sorted(range(len(last)), key=last.__getitem__))
        self.first = bytes(last[i] for i in self.forward)

    def matches(self, row, pattern):
        for byte in pattern:
            if self.first[row] != byte:
                return False
            row = self.forward[row]
        return True

    def next_section(self, section):
        start, end = section
        counts = Counter()
        runs = {}
        for row in range(start, end):
            word = bytearray()
            current = row
            while True:
                word.append(self.first[current])
                current = self.forward[current]
                if start <= current < end:
                    break
            label = bytes(word)
            counts[label] += 1
            if label in runs:
                assert runs[label][1] == row
                runs[label][1] += 1
            else:
                runs[label] = [row, row + 1]
        winner = max(counts, key=lambda w: (counts[w], -runs[w][0]))
        return winner, tuple(runs[winner]), len(counts), sum(map(len, counts))

    def mark(self, primary, section):
        row, wait = primary, 0
        while not section[0] <= row < section[1]:
            row = self.forward[row]
            wait += 1
            if row == primary:
                return -1
        return wait


def audit(runs, primary, first_marker, depth, exhaustive=False):
    last = bytes(b for b, width in runs for _ in range(width))
    column = Column(runs)
    truth = Oracle(last)
    section = column.interval(first_marker)
    marker = first_marker
    n = len(last)
    for level in range(1, depth + 1):
        if section[0] == section[1]:
            break
        word, child, stats = heaviest(column, section)
        expected, interval, types, dictionary = truth.next_section(section)
        assert (word, child) == (expected, interval)
        parent_marker = marker
        marker = word + marker
        assert marker.startswith(parent_marker) and column.interval(marker) == child
        if exhaustive:
            assert [i for i in range(n) if truth.matches(i, marker)] == list(range(*child))
        yield dict(level=level, rows=n, runs=len(runs), marker=parent_marker.hex(), next_marker=marker.hex(),
                   tokens=section[1]-section[0], frequency=child[1]-child[0], winner_bytes=len(word),
                   distinct_returns=types, total_distinct_spelling=dictionary,
                   next_mark_distance=truth.mark(primary, child), **stats)
        section = child


def small():
    total = overlaps = missing = 0
    for n in range(1, 9):
        for last in product(range(2), repeat=n):
            runs = [(k, len(list(rows))) for k, rows in groupby(last)]
            for length in range(1, 4):
                for pattern in product(range(2), repeat=length):
                    for row in audit(runs, n//2, bytes(pattern), 3, True):
                        total += 1
                        overlaps += row["winner_bytes"] < len(bytes.fromhex(row["marker"]))
                        missing += row["next_mark_distance"] == -1
    print(f"{total} dense nested selections, {overlaps} overlapping winner returns, {missing} marks on a cycle missing the child")


def real(tapes):
    project = Path(__file__).resolve().parents[4]
    sys.path.insert(0, str(project / "docs/journal/2026-11-18-astra-c-output-first"))
    from inputs import paths, read_input
    writer = None
    for path in paths(tapes):
        runs, primary = read_input(path)
        for obs in audit(runs, primary, b'"', 4):
            row = {"ordinal": int(path.name.split(".")[0]), **obs}
            if writer is None:
                writer = csv.DictWriter(sys.stdout, fieldnames=row, lineterminator="\n")
                writer.writeheader()
            writer.writerow(row)
        sys.stdout.flush()


if __name__ == "__main__":
    if len(sys.argv) == 1:
        small()
    else:
        real(sys.argv[1])
