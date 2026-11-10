"""Timestamps for each row `ab run` writes (the target-gated noninferiority workflow; registration
§2.8): `ab` flushes a row as each child ends, and this polls the file every 0.2 s and writes,
for each new complete line, its index and the wall-clock time it was seen. Passive: it reads the
file and nothing else, and runs beside both arms alike.

    python3 bench/tg_ni_stamp.py RUNS.jsonl ROW-TIMES.txt STOP-FILE

It ends once STOP-FILE exists and no new line has arrived since. A truncation (ab opening the
file) restarts the count. Row i's time is when it was seen, at most about 0.2 s after `ab`
wrote it.
"""

import os
import sys
import time


def follow(path, out, stop, interval=0.2, clock=time.time, sleep=time.sleep):
    pos, buf, n = 0, b"", 0
    with open(out, "w") as o:
        while True:
            stopping = os.path.exists(stop)
            size = os.path.getsize(path) if os.path.exists(path) else 0
            if size < pos:
                pos, buf, n = 0, b"", 0
                o.write("# truncated\n")
            got = False
            if size > pos:
                with open(path, "rb") as f:
                    f.seek(pos)
                    data = f.read(size - pos)
                pos += len(data)
                buf += data
                now = clock()
                while b"\n" in buf:
                    _, buf = buf.split(b"\n", 1)
                    o.write(f"{n} {now:.3f}\n")
                    n += 1
                    got = True
                o.flush()
            if stopping and not got:
                return n
            sleep(interval)


if __name__ == "__main__":
    follow(*sys.argv[1:4])
