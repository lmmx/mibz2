"""Construct a stable token column over ordered copies V,U. No follower pairs or old words."""
from bisect import bisect_right
from collections import Counter, defaultdict
from copy import copy


def append_run(runs, token, width):
    if runs and runs[-1][0] == token:
        runs[-1] = token, runs[-1][1]+width
    else:
        runs.append((token, width))


class Lift:
    def __init__(self, u, leaves, incoming, words, minima):
        m = u[1]-u[0]
        counts = Counter()
        for c, width in incoming:
            counts[c] += width
        order = sorted(counts, key=minima.get)
        ids = {c:i for i,c in enumerate(order)}
        self.core_count = len(order)
        self.m = m
        self.weights = [words[c] for c in order]
        self.base = []
        cursor = 0
        for c in order:
            self.base.append(cursor)
            cursor += counts[c]
        assert cursor == m and all(self.weights)
        # The atlas arrives in physical cut order, each piece carrying old origin.
        permuted = [None] * len(leaves)
        cursor = u[0]
        prev = None
        for i, (origin, _, width, prefix) in sorted(enumerate(leaves),key=lambda item:item[1][0]):
            assert origin == cursor
            if prefix != prev:
                token = len(self.weights)
                self.weights.append(prefix)
                self.base.append(m+origin-u[0])
                prev = prefix
            permuted[i] = (token,width)
            cursor += width
        assert cursor == u[1]
        self.column = []
        for token,width in permuted:
            append_run(self.column,token,width)
        self.prefix_runs = len(self.column)
        for c,width in incoming:
            append_run(self.column,ids[c],width)
        assert len(self.column) == self.prefix_runs+len(incoming)
        self._index()

    def _index(self):
        m = self.m
        self.starts = [0]
        self.translations = []
        self.select = defaultdict(list)
        free = self.base.copy()
        for token,width in self.column:
            y = self.starts[-1]
            self.starts.append(y+width)
            self.translations.append(free[token]-y)
            self.select[token].append((free[token],y))
            free[token] += width
        assert self.starts[-1] == 2*m and free == self.base[1:]+[2*m]
        self.sizes = [hi-lo for lo,hi in zip(self.base,free)]
        self.core = max(range(self.core_count),key=lambda k:(self.sizes[k],-self.base[k]))
        self.unforced = max(range(len(free)),key=lambda k:(self.sizes[k],-self.base[k]))

    def cap_prefixes(self):
        """Force the existing largest-label rule to choose the dominant core, with the fewest added IDs."""
        cap = self.sizes[self.core]
        if self.unforced == self.core:
            return self
        model = copy(self)
        model.weights = self.weights[:self.core_count]
        model.base = self.base[:self.core_count]
        first_alias = {}
        for token in range(self.core_count,len(self.weights)):
            first_alias[token] = len(model.weights)
            for offset in range(0,self.sizes[token],cap):
                model.weights.append(self.weights[token])
                model.base.append(self.base[token]+offset)
        consumed = Counter()
        model.column = []
        for token, width in self.column[:self.prefix_runs]:
            while width:
                offset = consumed[token]
                alias, used = divmod(offset,cap)
                take = min(width,cap-used)
                append_run(model.column,first_alias[token]+alias,take)
                consumed[token] += take
                width -= take
        model.prefix_runs = len(model.column)
        model.column.extend(self.column[self.prefix_runs:])
        model._index()
        assert model.unforced == model.core == self.core
        assert len(model.column)-len(self.column) <= len(model.weights)-len(self.weights)
        return model

    def forward(self, x):
        token = bisect_right(self.base,x)-1
        bins = self.select[token]
        lo,y = bins[bisect_right(bins,(x,self.starts[-1]))-1]
        return y+x-lo, token

    def induce(self, token):
        lo = self.base[token]
        hi = lo+self.sizes[token]
        d = lo
        segments = []
        values = {}
        dictionary = []
        probes = 0
        while d < hi:
            x = d
            width = hi-d
            rev = []
            while True:
                run = bisect_right(self.starts,x)-1
                width = min(width,self.starts[run+1]-x)
                rev.append(self.column[run][0])
                x += self.translations[run]
                if lo <= x < hi:
                    width = min(width, hi-x)
                    break
                if x < lo:
                    width = min(width, lo-x)
            assert width > 0 and len(rev)%2 == 0
            probes += len(rev)
            name = tuple(reversed(rev))
            if name not in values:
                values[name] = len(dictionary)
                dictionary.append(name)
            segments.append((x,d,width,values[name]))
            d += width
        assert probes <= 2*self.m
        assert len(segments) <= len(self.column)+2
        assert all(a[3] != b[3] for a,b in zip(segments,segments[1:]))
        segments.sort()
        assert segments[0][0]==lo and segments[-1][0]+segments[-1][2]==hi
        assert all(a[0]+a[2]==b[0] for a,b in zip(segments,segments[1:]))
        return Return(lo,hi,segments,dictionary,probes)


class Return:
    def __init__(self,lo,hi,segments,dictionary,probes):
        self.lo,self.hi = lo,hi
        self.segments = segments
        self.starts = [s[0] for s in segments]
        self.dictionary = dictionary
        self.probes = probes

    def marked(self, model, row, skip, n):
        out = bytearray()
        x = row
        live = True
        while len(out) < n:
            if live and self.lo <= x < self.hi:
                s,d,w,k = self.segments[bisect_right(self.starts,x)-1]
                assert s <= x < s+w
                for token in self.dictionary[k]:
                    weight = model.weights[token]
                    cut = min(skip,len(weight))
                    skip -= cut
                    out.extend(weight[cut:cut+n-len(out)])
                    if len(out)==n:
                        break
                x = d+x-s
            else:
                x,token = model.forward(x)
                weight = model.weights[token]
                cut = min(skip,len(weight))
                skip -= cut
                out.extend(weight[cut:cut+n-len(out)])
                if x == row:
                    live = False
        return bytes(out)
