"""Construct a second phrase BWT over the run-encoded integer tokens from F2b."""
import argparse
import bisect
import csv
import hashlib
import itertools
import struct
import sys
from collections import Counter
from pathlib import Path


def read(path):
    data = path.read_bytes()
    n, p, skip, m, alphabet, runs = struct.unpack_from("<6I", data)
    at = 24
    bases, words = [], []
    for _ in range(alphabet):
        base, width = struct.unpack_from("<II", data, at)
        at += 8
        bases.append(base)
        words.append(data[at:at+width])
        at += width
    column = list(struct.iter_unpack("<II", data[at:]))
    assert len(column) == runs and sum(v for _, v in column) == m
    return n, p, skip, bases, words, column


class Layer:
    def __init__(self, column, bases, words, delimiter):
        self.starts = []
        self.ids = []
        self.lf = []
        counts = Counter()
        size = 0
        for word, width in column:
            self.starts.append(size)
            self.ids.append(word)
            self.lf.append(bases[word] + counts[word])
            counts[word] += width
            size += width
        self.starts.append(size)
        self.delimiter = delimiter
        self.base = bases[delimiter]
        self.lookups = self.steps = self.selectors = self.intersections = self.aliases = 0
        self.copy_spans = self.merge = self.visits = 0
        # Nodes initially represent an interval and its first new symbol, then
        # keep only a compressed edge and terminal/branch selectors.
        self.node = []
        self.spell = []
        self.c2 = []
        self.runs = []
        if not counts[delimiter]:
            return
        self.node = [[self.base,self.base+counts[delimiter],-1,delimiter,None,None,-1]]
        for i, node in enumerate(self.node):
            lo,hi,parent,c = node[:4]
            suffix = [c]
            while True:
                k = self.find(lo)
                if self.starts[k+1] < hi or self.ids[k] == delimiter:
                    break
                self.aliases += 1
                suffix.append(self.ids[k])
                new = self.lf[k] + lo - self.starts[k]
                lo,hi = new,new+hi-lo
            node[4] = suffix
            selectors = node[5] = []
            children = {}
            while self.starts[k] < hi:
                self.intersections += 1
                a = max(lo,self.starts[k])
                width = min(hi,self.starts[k+1]) - a
                mapped = self.lf[k] + a-self.starts[k]
                c = self.ids[k]
                if c == delimiter:
                    child = -1
                    if node[6] == -1:
                        node[6] = len(self.spell)
                        self.c2.append(mapped-self.base)
                        # No dense level-one tokens are generated: expand each
                        # component from its dictionary spelling in the F2b dump.
                        parts = []
                        j = i
                        while j != -1:
                            for name in reversed(self.node[j][4]):
                                parts.append(words[name])
                                self.copy_spans += 1
                            j = self.node[j][2]
                        self.spell.append(b"".join(parts))
                else:
                    if c not in children:
                        children[c] = len(self.node)
                        self.node.append([mapped,mapped,i,c,None,None,-1])
                    child = children[c]
                    assert self.node[child][1] == mapped
                    self.node[child][1] += width
                if selectors and selectors[-1][0] == child:
                    selectors[-1][1] += width
                else:
                    selectors.append([child,width])
                k += 1
            self.selectors += len(selectors)
        # Distinct node children emit disjoint labels; a terminal emits its
        # parent's own label. Thus unit selector runs require no re-sorting.
        outs = [[] for _ in self.node]
        pos = [0] * len(self.node)
        used = [0] * len(self.node)
        for i in reversed(range(len(self.node))):
            dest = outs[i]
            for child,width in self.node[i][5]:
                if child == -1:
                    dest.append((self.node[i][6],width))
                    self.merge += 1
                    continue
                while width:
                    name, length = outs[child][pos[child]]
                    take = min(width,length-used[child])
                    dest.append((name,take))
                    self.merge += 1
                    width -= take
                    used[child] += take
                    if used[child] == length:
                        pos[child] += 1
                        used[child] = 0
            assert all(a[0] != b[0] for a,b in zip(dest,dest[1:]))
        self.runs = outs[0]
        nextslot = self.c2.copy()
        self.t = [None] * counts[delimiter]
        row = 0
        for name,width in self.runs:
            at = nextslot[name]
            nextslot[name] += width
            for j in range(at,at+width):
                self.t[j] = (row,name)
                row += 1
        assert row == len(self.t) and all(x is not None for x in self.t)

    def find(self,x):
        self.lookups += 1
        return bisect.bisect_right(self.starts,x)-1

    def expand(self,p,skip,n,base_words):
        if not self.runs:
            return None
        x = p
        while True:
            k = self.find(x)
            x = self.lf[k] + x-self.starts[k]
            c = self.ids[k]
            if c == self.delimiter:
                break
            self.steps += 1
            skip += len(base_words[c])
            if x == p:
                return None
        row = x-self.base
        out = bytearray()
        while len(out)<n:
            row,word = self.t[row]
            self.visits += 1
            part = self.spell[word]
            assert skip < len(part)
            out.extend(part[skip:skip+n-len(out)])
            skip=0
        return out


def runs(ids):
    return [(name,sum(1 for _ in group)) for name,group in itertools.groupby(ids)]


def small():
    words = [b"abc",b"d",b"efgh"]
    cases = took = fallback = 0
    for n in range(1,6):
        for ll in itertools.product(range(3),repeat=n):
            psi = sorted(range(n),key=ll.__getitem__)
            total = sum(len(words[x]) for x in ll)
            count = Counter(ll)
            bases = [sum(count[i] for i in range(c)) for c in range(3)]
            for d in range(3):
                layer = Layer(runs(ll),bases,words,d)
                for p in range(n):
                    first = ll[psi[p]]
                    for offset in range(len(words[first])):
                        cases+=1
                        row = p
                        expected = bytearray()
                        covered = False
                        while len(expected)<offset+total:
                            row = psi[row]
                            covered |= ll[row] == d
                            expected.extend(words[ll[row]])
                        got = layer.expand(p,offset,total,words)
                        assert (got is not None)==covered
                        if covered:
                            assert got==expected[offset:offset+total]
                            took+=1
                        else:
                            fallback+=1
    print(f"weighted marked cases={cases}, own={took}, exact fallback={fallback}")


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("directory",nargs="?",type=Path)
    p.add_argument("--small",action="store_true")
    a=p.parse_args()
    if a.small:
        small()
        return
    out = csv.writer(sys.stdout)
    out.writerow(["stream","target","rows","first_rows","first_runs","first_words","first_dictionary_bytes",
        "second_delimiter_hex","second_rows","second_runs","second_words","second_lookup","second_nodes",
        "second_unary","second_selectors","second_merge_pieces","second_dict_copy_spans","second_dict_bytes",
        "second_token_visits","rewind_token_steps","output_sha256"])
    for path in sorted(a.directory.glob("*.token"),key=lambda x:int(x.stem)):
        n,p,skip,bases,words,column=read(path)
        m=sum(w for _,w in column)
        freq=Counter()
        for name,width in column:
            freq[name]+=width
        expected=path.with_suffix(".expected").read_bytes()
        for target in (1,8,32):
            # Closest to the target count; break ties in token collation order.
            d=min(freq,key=lambda c:(abs(target*freq[c]-m),bases[c]))
            layer=Layer(column,bases,words,d)
            inverse=layer.expand(p,skip,n,words)
            assert inverse==expected
            out.writerow([path.stem,target,n,m,len(column),len(words),sum(map(len,words)),
                words[d].hex(),len(layer.t),len(layer.runs),len(layer.spell),layer.lookups,len(layer.node),
                layer.aliases,layer.selectors,layer.merge,layer.copy_spans,sum(map(len,layer.spell)),
                layer.visits,layer.steps,hashlib.sha256(inverse).hexdigest()])

if __name__=="__main__":
    main()
