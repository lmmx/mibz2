# bench/occupancy.sh, exactly as CI runs it, with any arguments passed on unchanged:
#   just bench-occupancy
#   just bench-occupancy --threads 1,2,4,8,12,16,20
# It writes occupancy-<UTC time>.json, the file to share.

# Where the parallel decoder's time goes at each thread count (#39), on the pinned corpus
[positional-arguments]
bench-occupancy *args:
    bash bench/occupancy.sh "$@"

# bench/imc.py: the machine's inventory, 30 s of idle readings, a memcpy calibration, then
# the core and memory-controller counters at 1, 6, 8 and 10 threads, each reading beside an
# idle reading of its own. Needs perf and sudo. It writes imc-<UTC time>.json to share.
#   just bench-imc
#   just bench-imc --note "fresh boot, nothing open" --threads 1,4,6,8,10
#   just bench-imc idle calibrate

# The memory controller's and the cores' counters around the decode (#39)
[positional-arguments]
bench-imc *args:
    bash bench/corpus.sh
    python3 bench/imc.py "$@"

# bench-imc (its arguments passed on), then bench-occupancy at 6, 8 and 10 threads; it
# writes imc-<UTC time>.json and occupancy-<UTC time>.json.
#   just bench-quiet --note "fresh boot, nothing open"

# On a quiet machine: the counters, then the occupancy curve at 6, 8 and 10 threads
[positional-arguments]
bench-quiet *args:
    bash bench/corpus.sh
    python3 bench/imc.py "$@"
    bash bench/occupancy.sh --threads 6,8,10
