"""Timestamps for each row `ab run` writes (the target-gated noninferiority workflow; registration
§2.8): `ab` flushes a row as each child ends, and this polls the file every 0.2 s and writes,
for each new complete line, its index and the wall-clock time it was seen. Passive: it reads the
file and nothing else, and runs beside both arms alike.

    python3 bench/tg_ni_stamp.py RUNS.jsonl ROW-TIMES.txt STOP-FILE

It ends once STOP-FILE exists and no new line has arrived since. A truncation (ab opening the
file) restarts the count. The first line is `# start T`, the time it began watching.

**The clock:** the runner's CLOCK_REALTIME (Python's time.time()), written to the millisecond.
Row i's time is when the poller saw it, an observation time. The poll interval is 0.2 s, but
that is not a hard bound on the delay after `ab` wrote the row: the poller is scheduled like
any other process.
"""

import os
import sys
import time


def follow(path, out, stop, interval=0.2, clock=time.time, sleep=time.sleep):
    pos, buf, n = 0, b"", 0
    with open(out, "w") as o:
        o.write(f"# start {clock():.3f}\n")
        o.flush()
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
