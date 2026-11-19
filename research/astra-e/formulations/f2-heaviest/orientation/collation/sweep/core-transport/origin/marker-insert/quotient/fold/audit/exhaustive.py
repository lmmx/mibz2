"""Enumerate ordinary stable token columns and every whole-class contraction.

Each path is a list of original ordinal vertices, so a final comparison detects
changes of phase, physical membership and spelling even when bytes coincide.
Only the final seams of isolated deterministic labels are compared circularly.
"""
import argparse
from collections import Counter, defaultdict
from itertools import product
import json


def canonical_cycle(path):
    i = path.index(min(path))
    return path[i:] + path[:i]


class Column:
    def __init__(self, incoming):
        self.labels = sorted(incoming)
        positions = defaultdict(list)
        for row, symbol in enumerate(incoming):
            positions[symbol].append(row)
        # Stable occurrence number maps source ordinal to destination ordinal.
        self.dest = [row for symbol in sorted(positions) for row in positions[symbol]]
        self.initial = tuple((i,) for i in range(len(incoming)))
        # Positive nonuniform lengths also exercise offsets inside old words.
        self.spell = {k: bytes([k + 65]) * (k + 1) for k in positions}

    def geometry(self, state):
        groups = defaultdict(list)
        opts = defaultdict(set)
        old = self.labels
        slot = {p[0]:i for i, p in enumerate(state)}
        targets = []
        for i, path in enumerate(state):
            k = old[path[0]]
            groups[k].append(i)
            target = slot[self.dest[path[-1]]]
            targets.append(target)
            opts[k].add(old[state[target][0]])
        return groups, targets, opts

    def check_marks(self, state, groups, targets):
        # Equality and cyclic adjacency of atom IDs determine marked output for
        # *every* horizon, including lengths exceeding the elementary period.
        old = self.labels
        atoms = []
        for path in state:
            for x, y in zip(path, path[1:]):
                assert self.dest[x] == y
            atoms.append(tuple((x, b) for x in path for b in range(len(self.spell[old[x]]))))
        # Define the transferred mark (new vertex, offset) at each old byte.
        inverse = {atom:(i, b) for i, word in enumerate(atoms) for b, atom in enumerate(word)}
        expected = sum(len(self.spell[k]) for k in old)
        assert len(inverse) == expected == sum(map(len, atoms))
        assert [p[0] for p in state] == sorted(p[0] for p in state)
        for indexes in groups.values():
            assert all(targets[x] < targets[y] for x,y in zip(indexes,indexes[1:]))
            assert len({tuple(old[x] for x in state[i]) for i in indexes}) == 1
        for (x, b), (i, offset) in inverse.items():
            raw_next = (x, b+1) if b+1 < len(self.spell[old[x]]) else (self.dest[x], 0)
            out_next = (i, offset+1) if offset+1 < len(atoms[i]) else (targets[i], 0)
            assert inverse[raw_next] == out_next
        return expected

    @staticmethod
    def contract(state, indexes, targets):
        chosen = set(indexes)
        killed = {targets[i] for i in chosen}
        assert len(killed) == len(chosen) and not (killed & chosen)
        return tuple(p + state[targets[i]] if i in chosen else p
                     for i,p in enumerate(state) if i not in killed)

    @staticmethod
    def normal_form(state, groups, targets):
        # For a pure self class, stable order forces each surviving row to be a
        # fixed point. Its elementary original orbit then is that row's payload.
        frozen = []
        frozen_rows = set()
        for indexes in groups.values():
            if all(targets[i] == i for i in indexes):
                frozen.append(tuple(canonical_cycle(state[i]) for i in indexes))
                frozen_rows.update(indexes)
        active = tuple(p for i,p in enumerate(state) if i not in frozen_rows)
        return active, tuple(sorted(frozen))

    def run_formula(self, state, groups, targets):
        # A pure self-label occupies its own solid block in incoming order.
        incoming = [None]*len(state)
        for i, j in enumerate(targets):
            incoming[j] = self.labels[state[i][0]]
        pure = {k for k, ids in groups.items() if all(targets[i]==i for i in ids)}
        keep = [i for i,v in enumerate(incoming) if v not in pure]
        active = [incoming[i] for i in keep]
        runs = lambda s: int(bool(s)) + sum(x != y for x,y in zip(s,s[1:]))
        cuts = sum(incoming[x] == incoming[y] and x+1 < y for x,y in zip(keep,keep[1:]))
        assert runs(incoming) == runs(active) + len(pure) + cuts
        return incoming, runs(incoming)

    def verify(self, incoming, totals, witnesses):
        pending = [self.initial]
        seen = set(pending)
        strict = set()
        normalized = set()
        run_counts = {}
        while pending:
            state = pending.pop()
            groups, targets, opts = self.geometry(state)
            totals['states'] += 1
            totals['mark_successors_verified'] += self.check_marks(state,groups,targets)
            succ = {k:next(iter(s)) for k,s in opts.items() if len(s)==1 and k not in s}
            if not succ:
                totals['terminal_states'] += 1
                terminal, run_count = self.run_formula(state,groups,targets)
                run_counts[run_count] = terminal
                strict.add(state)
                normalized.add(self.normal_form(state,groups,targets))
                continue
            if len(succ)>1:
                totals['fork_states'] += 1
            for k, j in succ.items():
                if (j in succ and succ[j] != k and len(groups[j])>len(groups[k]) and
                    len(opts[succ[j]])>1 and 'branching_chain' not in witnesses):
                    witnesses['branching_chain'] = dict(incoming=incoming,state=state,
                        labels=self.labels,successors=succ,source=k,middle=j,end=succ[j],destinations=targets)
                nxt = self.contract(state,groups[k],targets)
                assert len(nxt) == len(state) - len(groups[k])
                totals['legal_singleton_steps'] += 1
                if nxt not in seen:
                    seen.add(nxt)
                    pending.append(nxt)
        totals['initial_columns'] += 1
        if len(strict)>1:
            totals['columns_with_multiple_strict_ends'] += 1
            if 'first_strict_divergence' not in witnesses:
                witnesses['first_strict_divergence'] = dict(incoming=incoming,
                    labels=self.labels,destinations=self.dest,terminal_states=sorted(strict))
        if len(run_counts)>1:
            totals['columns_with_different_terminal_rle'] += 1
            if 'first_rle_divergence' not in witnesses:
                witnesses['first_rle_divergence'] = dict(incoming=incoming,labels=self.labels,
                    destinations=self.dest,terminal_incoming_by_runs=run_counts)
        assert len(normalized)==1, (incoming, normalized)
        assert len({len(s) for s in strict})==1
        n = next(iter(normalized))
        if n[0]:
            totals['columns_retaining_active_rows'] += 1


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--max-rows',type=int,default=7)
    p.add_argument('--max-classes',type=int,default=4)
    args=p.parse_args()
    counts=Counter()
    witnesses={}
    for n in range(1,args.max_rows+1):
        for k in range(1,min(n,args.max_classes)+1):
            for incoming in product(range(k),repeat=n):
                if len(set(incoming))==k:
                    Column(incoming).verify(incoming,counts,witnesses)
    print(json.dumps(dict(config=vars(args),totals=counts,witnesses=witnesses),indent=2))


if __name__=='__main__':
    main()
