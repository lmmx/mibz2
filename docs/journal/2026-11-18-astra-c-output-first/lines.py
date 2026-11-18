"""Independent cold, fully associative, write-allocate LRU models, 64-byte lines."""
from collections import Counter, OrderedDict

# Disjoint aligned regions. Each append-array generation has its own origin.
RAW, HIST, BASE, DIRECTORY, HEAD, OUT, NODE = (i << 30 for i in range(7))


class Cache:
    def __init__(self, kib):
        self.limit = kib * 1024 // 64
        self.resident = OrderedDict()
        self.fills = Counter()
        self.evicted_dirty = 0
        self.read_fills = 0

    def touch(self, line, write):
        table = self.resident
        if line in table:
            if write:
                table[line] = True
            table.move_to_end(line)
        else:
            self.read_fills += not write
            self.fills[min(line * 64 >> 30, NODE >> 30)] += 1
            table[line] = write
            if len(table) > self.limit:
                self.evicted_dirty += table.popitem(last=False)[1]

    def result(self):
        return sum(self.fills.values()), self.evicted_dirty + sum(self.resident.values())


class Traffic:
    def __init__(self):
        self.caches = {k: Cache(k) for k in (32, 256, 2048)}
        self.lines = set()
        self.bytes = Counter()

    def at(self, address, length, write=False):
        self.bytes[write] += length
        for line in range(address // 64, (address + length - 1) // 64 + 1):
            self.lines.add(line)
            for cache in self.caches.values():
                cache.touch(line, write)

    def report(self):
        result = dict(read_bytes=self.bytes[False], write_bytes=self.bytes[True],
                      distinct_lines=len(self.lines))
        for k, cache in self.caches.items():
            result[f"fills{k}"], result[f"dirty{k}"] = cache.result()
            result[f"read_fills{k}"] = cache.read_fills
            for name, region in (("base", BASE), ("dir", DIRECTORY), ("head", HEAD),
                                 ("node", NODE), ("out", OUT)):
                result[f"{name}_fills{k}"] = cache.fills[region >> 30]
        return result
