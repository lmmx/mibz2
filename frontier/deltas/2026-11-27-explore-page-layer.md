claim: explore
by: claude-code
outcome: done
evidence: docs/notebook/2026-11-27-explore-page-layer.md; docs/journal/2026-11-27-explore/page-layer/ (scripts, raw station_balance A/B); source 66511da9c3b8; corpora/wikidata.bz2 sha256 82f65ace0731

**What was asked:** Recurring `explore`: a phenomenon no claim covers. Chosen: the page layer (transparent huge pages, page faults, glibc arena growth) under the current worker stack, last looked at on 5 Nov before the Open route, never A/B'd.
**What I found:** Scoped STOP for THP; the page layer is not a cost on the production route.
- Kernel time is 5.4-6.3% of CPU (perf cpu-clock, VM); the fault path is <= 0.8%. The bigger visible item is libc `memmove` (~9.5%), ~85% of it the Cells route's segment stitch under `CellsKernel::invert_runs`: already targeted by vectored delivery / slab fusion, not new.
- On the Cells route (t <= 6 here) glibc thread arenas grow one page at a time: 1,321-3,640 `mprotect`s per x4 decode, ~97% under `room_segments` segment buffers; faults 27-50k. On OpenCells (t >= 8) it is 103-129 `mprotect`s and ~18-20k faults (71 MB first touch for 839 MB out). `trim_threshold=4G` does not remove the growth.
- THP via `GLIBC_TUNABLES=glibc.malloc.hugetlb=1` (verified: 10 MiB AnonHugePages): single-thread pinned OpenCells W paired ratio 1.0044 [0.981, 1.027], S 1.0035 [0.982, 1.029] (10 pairs): null. bzcat 2T CPU -3.1% (sys only, Cells route), 4T +0.3%: noise. RSS rises 1.019x at 8T, 1.05-1.07x at 2-4T, breaching the <= 1.01x 8W RSS invariant for no gain.
**What changed:** notebook entry, journal scripts/raw data, this delta, a dead-ends row (THP / page layer on OpenCells). No new claim: the only measurable page-layer cost (Cells route at 4-6T faulting 2.2-2.8x what 8T does) is more evidence for the existing `plan-schedule-and-lazy-threshold` idea, which moves t = 4..7 to OpenCells.
**What's next:** Whoever builds `plan-schedule-and-lazy-threshold` can cite the fault/mprotect table as a secondary benefit (RSS and sys time), not a primary one. Don't reopen THP unless a PMU run on the i9 shows dTLB walks well above the Nov census's 1-2% of cycles at 8 workers.
**Caveats:** VM, 4 vCPU, no HW counters, so no TLB-miss figures; t = 8/10 were oversubscribed (used only for counts and RSS, not timing); no L3-cold or i9 run; THP `always` hosts get huge pages by default, which this does not test. No decision rule was frozen first: a characterisation, not an adoption test.
