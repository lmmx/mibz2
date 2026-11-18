"""One uint32 per touched 64-byte line; bit 31 denotes a store."""
import array
import sys


class Trace:
    def __init__(self, path):
        self.file = path.open("wb")
        self.buffer = array.array("I")
        assert self.buffer.itemsize == 4 and sys.byteorder == "little"

    def at(self, addr, size, write=False):
        if not size:
            return
        a = addr // 64
        b = (addr + size - 1) // 64
        assert 0 <= a <= b < 1 << 31
        base = int(write) << 31
        if a == b:
            self.buffer.append(base+a)
        else:
            self.buffer.extend(range(base+a, base+b+1))
        if len(self.buffer) >= 65536:
            self.flush()

    def next_phase(self):
        self.buffer.append(0xffffffff)

    def flush(self):
        self.buffer.tofile(self.file)
        del self.buffer[:]

    def __enter__(self):
        return self

    def __exit__(self, *args):
        self.flush()
        self.file.close()
