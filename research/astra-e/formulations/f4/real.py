"""Price complete transforms against published cut geometry and verify marked hashes."""
import argparse
import csv
import hashlib
import sys
from pathlib import Path

import numpy as np
from phrases import expanded_marked, spelling, transform

root = Path(__file__).resolve().parents[4]
journal = root / "docs/journal"
sys.path.insert(0, str(journal / "2026-11-18-astra-c-output-first"))
from inputs import paths, read_input
sys.path.insert(0, str(journal / "astra-a-reinvention"))
from pairs import cuts


def main():
    cli = argparse.ArgumentParser()
    cli.add_argument("input")
    args = cli.parse_args()
    archive = {int(x["block"]): x for x in csv.DictReader(
        (journal / "2026-11-18-astra-c-output-first/section.csv").open())}
    prior = {int(x["block"]): x for x in csv.DictReader(
        (journal / "astra-a-reinvention/evidence/pairs.csv").open())}
    writer = csv.writer(sys.stdout, lineterminator="\n")
    counters = ["input_runs","overlays","prefix_searches","prefix_pieces","prefix_intervals",
                "token_searches","token_pieces","alphabet","trie_nodes","token_rows","token_runs"]
    writer.writerow(["block","order","n","p","new_p","skip",*counters,"word_bytes","max_word",
                     "original_cuts","astra_a_pair_cuts","residual_cuts","marked_sha256"])
    for path in paths(args.input):
        index = int(path.name.split(".")[0])
        runs, p = read_input(path)
        n = sum(w for _, w in runs)
        for asc in [True, False]:
            result = transform(runs, p, asc)
            tokens, rank, skip, parents, dictionary, stats = result
            marked = bytes(expanded_marked(result, n))
            sha = hashlib.sha256(marked).hexdigest()
            assert sha == archive[index]["marked_sha256"]
            col = np.repeat(np.array([c for c, _ in tokens], dtype=np.uint32), [w for _, w in tokens])
            lengths = [len(spelling(id, parents)) for id in dictionary]
            writer.writerow([index, "asc" if asc else "desc",n,p,rank,skip,
                             *[stats[k] for k in counters],sum(lengths),max(lengths),
                             prior[index]["original_cuts"],prior[index]["pair_cuts"],cuts(col, rank),sha])
            sys.stdout.flush()


if __name__ == "__main__":
    main()
