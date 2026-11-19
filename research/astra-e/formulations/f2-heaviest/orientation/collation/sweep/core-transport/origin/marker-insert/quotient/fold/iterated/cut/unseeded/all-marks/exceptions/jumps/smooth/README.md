# Skip deterministic context states without bitmap-rank lookups

At k=8 just 23,496 of 815,844 contexts observed on the ten primitive marked laps have multiple predecessor labels. These branching contexts account for 1,108,546 physical visits (12.32% of coded bytes). There are 708,399 intervening maximal smooth stretches. The tested clock-free bitmap writer made 2,593,760 dispatches here. At k=16 there are 575,661 branch visits and 376,879 smooth stretches, versus 1,849,093 bitmap-writer dispatches. This leaves a real opportunity to avoid the bitmap part of the hot loop by compiling smooth paths. Each isolated branch still needs its true ordered transition and every byte must be emitted. The counts give no elapsed-time or memory claim. [All source populations and visits](blocks.csv) · [Optimal path layout and spelling storage](chains/) · [Existing bitmap implementation and paid prices](../native/bench/)

```sh
d=$(dirname "$(rg --files | rg '/jumps/smooth/PLAN.md$')")
g++ -std=c++17 -O3 -Wall -Wextra -pedantic "$d/count.cpp" -o /tmp/astra-smooth-count
python3 -B "$d/panel.py" /tmp/astra-smooth-count /tmp/astra-clock-wires | diff - "$d/blocks.csv"
```
