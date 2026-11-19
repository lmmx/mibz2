"""Record one native child process, including construction of its offline index."""
from pathlib import Path
import os
import sys
import time

path, *command = sys.argv[1:]
before = time.monotonic()
pid = os.posix_spawnp(command[0], command, os.environ)
_, status, info = os.wait4(pid, 0)
Path(path).write_text(f"elapsed_s {time.monotonic()-before:.6f} user_cpu_s {info.ru_utime:.6f} "
                      f"system_cpu_s {info.ru_stime:.6f} peak_kib {info.ru_maxrss}\n")
sys.exit(os.waitstatus_to_exitcode(status))
