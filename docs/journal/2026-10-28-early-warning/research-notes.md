# Early contention measurement for mibz2: research notes (2026-10-28)

Legend: **[S]** = established in the cited source. **[I]** = my inference or calculation, not stated by the source.

---

## 1. Co-runner interference methodology: concrete stressor designs

### Bubble-Up (Mars, Tang, Hundt, Skadron, Soffa, MICRO 2011)
PDF: https://research.google.com/pubs/archive/37675.pdf
- [S] Two steps. (1) Sensitivity: run the target with a "bubble" on the remaining cores of the socket and dial its pressure up, giving a QoS-vs-pressure **sensitivity curve**. (2) Contentiousness: run the candidate co-runner against a single-threaded **reporter** whose own sensitivity curve was trained against the bubble once per architecture. The reporter's slowdown is read backwards through that curve to give the co-runner a **bubble score** (an equivalent bubble size). Prediction = target curve evaluated at the co-runner's score. Reported accuracy: within 1–2% of measured degradation.
- [S] The pressure dial is the **bubble's working-set size in MB** ("a pressure size of 1 means our bubble will continuously smash a 1MB chunk").
- [S] The bubble is multithreaded and mixes loads and stores, random and streaming:
  - random part: an LFSR PRNG (mask `0xd0000001u`, period 2^32), not `rand()`, to keep compute between accesses tiny;
  - random part: a basic block of 100 independent read-modify-writes `dump[k] += data_chunk[r]++` in manual SSA form, for high MLP;
  - streaming part: the scalar part of STREAM (`a[i] = s*mid[i]`, then back), which also drives the hardware prefetcher.
- [S] The reporter mixes random and streaming access at low ILP, with a working set of about 20 MB, so it feels the LLC, bandwidth and prefetcher together.
- [S] Design rules: the curve must be monotonic, the dial must span "no pressure" to beyond the worst co-runner, and the bubble should load the memory subsystem broadly rather than a single component.

### Bubble-Flux (Yang, Breslow, Mars, Tang, ISCA 2013)
PDF: https://www.jasonmars.org/wp-content/uploads/2020/05/2508148.2485974.pdf
- [S] It is an online "Dynamic Bubble". It first ran 250 ms bursts per pressure level, but that was too noisy, so it settled on **2 s per pressure score**. Bubble sizes were **5–55 MB in 5 MB steps** on a 20 MB-L3 Sandy Bridge.
- [S] PiPo (phase-in/phase-out) pauses part of the bubble's execution. The degradation is estimated as the difference between QoS while the bubble is paused and QoS while it runs. This is a paired on/off measurement within one run, which cancels drift.
- [S] Web-search fell about 20% at bubble score 5 (25 MB). The sharp drop from 20 MB (the LLC size) to 25 MB was attributed to bandwidth rather than capacity.

### iBench (Delimitrou & Kozyrakis, IISWC 2013)
PDF: https://people.csail.mit.edu/delimitrou/papers/2013.iiswc.ibench.pdf · code: https://github.com/stanford-mast/iBench
- [S] There are 15 "sources of interference" (SoIs), each with intensity from 0 to 100% and designed so that impact rises roughly linearly with intensity:
  - LLC capacity: random accesses covering a growing fraction of the LLC. Footprint and pace are tuned to LLC size and associativity read from /proc/cpuinfo. L1/L2 sweeps run alongside so the accesses reach the LLC.
  - LLC bandwidth: streaming over a fraction of the LLC.
  - Memory bandwidth: streaming over a small part of memory (about 10%) to keep it separate from capacity.
  - Memory capacity: random or strided accesses over a growing footprint.
  - There are also L2, L1i/L1d, TLB, prefetcher, CPU, network and storage SoIs. The default kernel duration is 10 ms.
- [S] The open-source code is much cruder than the paper:
  - `l3.c` reads `/sys/devices/system/cpu/cpu0/cache/index3/size` and `memcpy`s half of that buffer onto the other half in a loop;
  - `memBw.c` is a STREAM-scale loop over `N = 2,000,000` doubles.
- [S] Paragon (ASPLOS 2013, https://people.csail.mit.edu/delimitrou/papers/2013.asplos.paragon.pdf) turns this into a score. Intensity is raised until the application drops below **95% of its isolated performance**, and that intensity is the **tolerated-interference score**. **Caused** interference is the intensity at which the microbenchmark itself slows by 5%. Online, a new app is profiled against only **two randomly chosen SoIs for one minute each**. The missing scores are filled in by SVD/PQ collaborative filtering, with an average error of 5.3%.

### CPI² (Zhang, Tune, Hagmann, Jnagal, Gokhale, Wilkes, EuroSys 2013)
PDF: https://static.googleusercontent.com/media/research.google.com/en//pubs/archive/40737.pdf
- [S] It samples CPI for **10 s once a minute** and builds a per-job, per-platform CPI distribution.
- [S] Outlier rule: above the predicted **mean + 2σ** (about 5% of samples). Samples from tasks using **< 0.25 CPU-sec/sec** are ignored because CPI inflates at low utilization.
- [S] Anomaly rule: **3 outliers in a 5-minute window**.
- [S] Antagonist search correlates the suspect's CPU usage with the victim's above-threshold CPI over a 10-minute window, with a 0.35 correlation cutoff. The remedy is hard-capping, e.g. 25 ms per 250 ms window.
- [I] CPU-seconds/byte = CPI × (instructions/byte) / frequency. At constant clock and constant instructions/byte (both true in your i9 data), "CPU amplification" is the CPI² signal. CPI² gives a tested alarm rule (2σ, 3-of-5) and a known pitfall (low utilization inflates CPI).

### Heracles (Lo et al., ISCA 2015)
PDF: https://static.googleusercontent.com/media/research.google.com/en//pubs/archive/43792.pdf
- [S] Antagonists:
  - cache: a streaming array of **¼, ½ and almost all of the LLC** ("LLC small/med/big");
  - DRAM: a much larger streaming array;
  - HyperThread: a register-only spinloop;
  - power: a power virus.
- [S] Key observation: interference is harmless until a shared resource **saturates**, and past that inflection point latency degrades "extremely rapidly". The controller avoids saturation:
  - it shrinks BE cores when DRAM bandwidth reaches **90% of peak streaming bandwidth**;
  - it uses gradient descent on cores and LLC ways, converging in about 30 s;
  - the top-level loop runs every 15 s.

### Dirigent (Zhu & Erez, ASPLOS 2016)
PDF: https://lph.ece.utexas.edu/merez/uploads/MattanErez/asplos2016_dirigent_paper.pdf
- [S] It profiles a solo **(time, progress)** trace offline and samples progress online every **ΔT = 5 ms**. Per segment, a time penalty is the ratio of measured to profiled progress rate, which predicts completion time *while the task is still running*. It notes "cache inertia": partition changes act slowly.
- [I] This is the closest prior art to "early, not late" measurement. Record per-block progress (bytes out per ms) against a solo profile and flag a slowing rate within the first blocks.

### PARTIES (Chen, Delimitrou, Martínez, ASPLOS 2019)
PDF: https://people.csail.mit.edu/delimitrou/papers/2019.asplos.parties.pdf
- [S] Every 500 ms it computes per-app tail-latency slack. It upsizes an app if slack < 0.05 and downsizes the one with the most slack if slack > 0.2. Sensitivity was characterised with microbenchmarks that thrash memory bandwidth on idle cores.

### Cache Pirating and Bandwidth Bandit (Eklov, Nikoleris, Black-Schaffer, Hagersten; ICPP 2011 / CGO 2013)
Pages: https://www.it.uu.se/research/group/uart/measurement/shared_resource_sensitivity.html · tool README: https://github.com/fraghag/pirate/blob/master/README.md
- [S] The Pirate holds a controlled amount of shared cache. It checks its own miss ratio: while it does not miss, it holds its whole working set and uses no off-chip bandwidth. The result is the target's performance as a function of *available cache*.
- [S] The Bandit steals bandwidth with sequential and random patterns confined to a few cache sets, so it does not pollute the cache. It separates *latency-sensitive* from *bandwidth-sensitive* applications. There is little correlation between how much bandwidth an application consumes and how sensitive it is.
- [S] Pirate caveats: DVFS can lower the Pirate's CPI, the OS takes some cache, and exclusive caches add L1+L2+L3.
- [I] Pirating and Bandit split exactly the two effects your block-size sweep separated (capacity vs bandwidth). They do it without changing the decoder.

### What transfers to a tiny CI smoke test on a 4-vCPU VM [I]
- **The hardware is only 2 physical cores.** GitHub's standard Linux runner has 4 vCPU and 16 GB (https://docs.github.com/en/actions/reference/runners/github-hosted-runners). Public Geekbench uploads labelled as such runners report an "AMD EPYC 9V74, 2 Cores, 4 Threads". Another reports "EPYC 7763, 1 Core, 2 Threads, L3 256 MB" (https://browser.geekbench.com/v6/cpu/17796608), where 256 MB is the whole socket, not the 32 MB CCD the vCPUs actually share. So:
  - (a) your CI "4 workers" are 2 SMT pairs. Siblings share L1D, the fill buffers and L2;
  - (b) the sysfs or CPUID L3 size is wrong for sizing a stressor. iBench's `l3.c` approach is unusable there.
- **Calibrate instead of trusting sysfs.** Before each bench, run a sub-second single-thread dependent pointer-chase over working sets of 256 KB→64 MB (×√2 steps). The latency knees give an *effective* L2 and L3 size as seen at that moment, neighbours included. This is the Pirate's self-verification done with timing instead of counters (perf events are unavailable on GitHub-hosted runners: https://pythonspeed.com/articles/consistent-benchmarking-in-ci/).
- **Stressor set with equal CPU and pinning.** Run the decoder on 1 vCPU and the stressor on another, with 3 working-set points:
  - (i) a Bubble-style random+stream kernel at ½×C_eff and 2×C_eff;
  - (ii) a Bandit-style streaming kernel touching few sets;
  - (iii) Lemire's `testingmlp`-style random lanes (https://github.com/lemire/testingmlp) for random-DRAM pressure.
  At 2 s per point (Bubble-Flux's figure) with PiPo-style on/off pairing inside each point, this is about 15–20 s. Report slowdown per stressor, not one number.
- **Pin each pairing both ways.** Run it on SMT siblings *and* on different cores (read `thread_siblings_list`). On 2 cores these give different answers, and mixing them silently is a noise source.
- **Expect cloud noise.** The coefficient of variation is 2.66% on GitHub-hosted runners vs 0.56% on bare metal (https://codspeed.io/blog/benchmarks-in-ci-without-noise), and cloud CVs range from 0.03% to over 100% across benchmarks and instances (Laaber et al., EMSE 2019, https://research.chalmers.se/publication/511491/file/511491_Fulltext.pdf). The defence is same-instance, randomized, interleaved A/B. RMIT (Abedi & Brecht, ICPE 2017, https://research.spec.org/icpe_proceedings/2017/proceedings/p287.pdf) showed that naive methods "falsely report that two identical systems differ by 38%" at 95% confidence. Your paired A/B already is RMIT-like. Apply the same pairing to stressor-on and stressor-off.

---

## 2. Cloud-VM caveats; instruction-count and simulated-cache CI

- **Cachegrind** (https://valgrind.org/docs/manual/cg-manual.html):
  - [S] Cache simulation is off by default (`--cache-sim=yes`).
  - [S] The model is I1, D1 and one LL, "approximates ... an AMD Athlon circa 2002", with no prefetcher modelled, virtual addresses, no kernel activity, and no TLB-miss or speculative misses.
  - [S] `--LL=size,assoc,line`; the number of sets must be a power of 2.
  - [S] Valgrind **serialises threads** ("only one thread is running at a time") and switches every `--scheduling-quantum` (default **100000 basic blocks**). All threads therefore share one simulated D1 and one LL (https://valgrind.org/docs/manual/manual-core.html).
- [I] **Does an LL model capture working-set growth?** Yes, for capacity. A single-thread run with `--LL` set to C/N (power-of-2 sets, e.g. 16-way 2 MiB, 4 MiB or 8 MiB) gives a deterministic **miss-ratio curve**. Footprint theory (§3) says N fairly interleaved copies each see about C/N. A run of N worker threads under one simulated LL also roughly models sharing, because the 100k-BB quantum interleaves the threads' T-vectors.
- [I] What it misses:
  - the bandwidth/queueing collapse, which needs a timing model;
  - inclusive-L3 effects, non-LRU replacement and the L2 streamer;
  - software prefetch, which it ignores. Check that this is true for your build.
- [I] Proxy: "LL misses per output MiB at LL = 4 MiB and 8 MiB" is a deterministic CI number. It moves when T-vector bytes per symbol, lane count or buffer sizes change, and it is the right early warning for "working set outgrew the cache".
- **DynamoRIO drcachesim** (https://dynamorio.org/sec_drcachesim_ops.html) is a better simulator for this:
  - [S] it simulates **`-cores` (default 4) private L1s with a shared `-LL_size` (default 8M, 16-way)**;
  - [S] it has `-replace_policy` LRU/LFU/FIFO and a `-data_prefetcher nextline` model;
  - [S] it has a TLB simulator and a `-tool reuse_distance` mode.
  - [I] It models "N workers on N cores sharing one LL" directly, which cachegrind cannot.
- **iai-callgrind / Gungraun** (https://gungraun.github.io/gungraun/latest/html/):
  - [S] it runs Callgrind, Cachegrind or DHAT and reports "Estimated Cycles", which Valgrind calls a relative metric;
  - [S] per-thread metrics need `--separate-threads=yes`, and spawned threads report zero unless toggles or client requests target them (https://gungraun.github.io/gungraun/latest/html/benchmarks/library_benchmarks/threads_and_subprocesses.html).
- **CodSpeed** (https://codspeed.io/docs/instruments/cpu.md):
  - [S] the CPU-simulation instrument estimates cycles ≈ instruction cycles + L1-miss × L2/L3 cost (10–40 cycles) + LL-miss × RAM cost (100+). Each benchmark runs once; syscall time is excluded; cache sizes are not published;
  - [S] the walltime instrument on standard hosted runners "will lead to inconsistent data", and they sell bare-metal "Macro runners" instead;
  - [S] the memory instrument (Jan 2026, https://codspeed.io/docs/instruments/memory) tracks peak heap, allocation counts and a memory timeline via eBPF on malloc/free;
  - [I] none of its instruments models shared-cache contention.
- **Bencher** (https://bencher.dev/docs/explanation/thresholds/): [S] thresholds can be percentage, z-score, t-test, log-normal, IQR, delta-IQR or static, over a sample/time window. Unknown measure names are created on the fly, so CPU-s/GiB or simulated LL-misses/MiB can be tracked as first-class series.
- **Practitioner precedent:**
  - [S] SQLite tracks performance by cachegrind cycle counts ("repeatable to 7 or more significant digits" vs wall time "scarcely ... beyond one"), https://sqlite.org/cpu.html;
  - [S] rustc-perf defaults to `instructions:u` because cycles and wall time are too noisy, https://internals.rust-lang.org/t/what-is-perf-rust-lang-org-measuring-and-why-is-instructions-u-the-default/9815;
  - [S] Sightglass supports cycles, instructions retired, perf cache counters and Callgrind, and applies Mann-Whitney U tests with effect sizes only when significant, https://github.com/bytecodealliance/sightglass.

---

## 3. Analytical early-warning models

### USL (Gunther)
- [S] X(N) = γN / (1 + α(N−1) + βN(N−1)); N* = √((1−α)/β). Gunther asks for "at least half a dozen data points" and warns the model "is not a crystal ball" (http://www.perfdynamics.com/Manifesto/USLscalability.html).
- [S] A USL fit to a subset can miss the peak badly: one subset fit predicted a peak near N = 30 where the full data peaked at about 20 (https://www.r-bloggers.com/2012/04/postgresql-scalability-analysis-deconstructed/).
- [S] Negative α (σ) captures superlinearity (https://arxiv.org/pdf/2006.04969).
- [I] Do not use USL to extrapolate a knee from 1/2/4 on CI:
  1. Three points exactly determine three parameters, so the fit has no residual and nothing to check it against.
  2. The mibz2 knee is a **capacity threshold** (aggregate T-vectors vs L3) followed by a **bandwidth/activation ceiling**. USL's β term is pairwise crosstalk (∝ N²), not a threshold, so β fit below the threshold is ≈0 and predicts no peak.
  3. On the CI box, 4 = 2 SMT pairs, so the curve mixes SMT and contention.
- [I] USL is still useful as a *descriptor*: report α from the 1/2/4 sweep as a contention coefficient and alarm when it rises.

### Roofline / cache-aware roofline
- [S] The cache-aware roofline (Ilic, Pratas, Sousa, IEEE CAL 2014) uses core-observed bandwidth per cache level, measured by microbenchmarks (https://hpcas.inesc-id.pt/~ilic/roofline.php).
- [I] Plain roofline fits mibz2 poorly: the inverse BWT is latency- and MLP-bound random access, not bandwidth-bound streaming. The useful "roof" is **random cache lines per second**, both per core (MLP ÷ latency, Little's law) and per socket (DRAM activation rate; see §6).

### Miss-ratio curves and footprint theory
- [S] HOTL (Xiang, Ding, Luo, Bao, ASPLOS 2013, https://www.cs.rochester.edu/u/cding/dpal/files/Xiang+_ASPLOS13.pdf):
  - **footprint is composable**: the combined footprint of co-runners is the sum, while miss counts are not;
  - converting footprint to MRC predicts shared-cache miss ratio "without parallel testing";
  - sampling overhead is < 0.5% on average;
  - the sampling implementation does not handle multithreaded code.
- [S] Chandra, Guo, Kim, Solihin (HPCA 2005) predict co-run L2 contention from each thread's solo stack-distance or circular-sequence profile (FOA, SDC and Prob models).
- [S] SHARDS (Waldspurger et al., FAST 2015, https://waldspurger.org/carl/papers/shards-fast15.pdf) builds MRCs from hash-sampled references, in constant space with an adaptive rate.
- [I] **For mibz2 an analytic MRC is enough.** The inverse BWT is a permutation walk: each T entry is read once per block, and each 64 B line, holding 16 entries, is hit 16 times at essentially random times. For random line access over W bytes, hit ratio h ≈ min(1, C_eff/W). With N workers and composable footprints, h(N) ≈ min(1, C_eff/(N·W)).
  - The MRC is **linear, not a cliff**, so misses begin before N·W reaches C. That matches your 1.48× CPU amplification at 2→4 workers at -9 (aggregate 7–14 MB < 20 MB): inclusive L3, other buffers and imperfect LRU already push C_eff below C.
  - Inputs are W per worker (the ledger) and C_eff (the pointer-chase calibration).

---

## 4. Prior art in parallel decompressors and inverse BWT

- [S] lbzip2's man page: one worker per block; memory grows linearly with threads; smaller blocks can speed up (de)compression as core counts rise. Nothing about a cache knee (https://www.mankier.com/1/lbzip2).
- [S] indexed_bzip2 reports 1 thread 4.2 MB/s, 2 threads 7.6 MB/s, 12 threads 31.4 MB/s and **24 threads 31.8 MB/s** on a 12-core CPU. It flattens at physical cores, with no cache discussion (https://github.com/mxmlnkn/indexed_bzip2).
- [S] rapidgzip (HPDC 2023, https://arxiv.org/pdf/2308.08955): the chunk size defaults to 4 MiB; memory is ∝ parallelism × chunk × ratio; it scales to 128 cores. No cache-size-based defaults.
- [S] zstd `-T0` defaults to **physical** cores (`--auto-threads={physical,logical}`, default physical) (https://man.archlinux.org/man/zstd.1.en.txt).
- [I] I found no decompressor that documents an L3-driven thread knee or sizes thread defaults by cache. mibz2's finding appears to be new to that literature.
- [S] Kärkkäinen & Puglisi, "Cache friendly Burrows-Wheeler inversion" (2011, https://www.cs.helsinki.fi/u/tpkarkka/publications/ccp2011.pdf):
  - Seward: "the maximum speed of BWT decompression is unavoidably constrained by the rate at which cache misses are serviced";
  - their `copy` algorithm exploits chains (runs in L) to copy repeats cache-friendly, using 6n bytes;
  - they name parallel inversion as future work, "reconstruct the text S at several places simultaneously".
- [S] Kärkkäinen, Kempa, Puglisi, "Slashing the time for BWT inversion" (DCC 2012): 2.3–4× faster, roughly halving both the number of misses and the cost per miss, using a super-alphabet (2 symbols per step).
- [S] **libsais 2.12.0 (released 18 Oct 2026)** (https://github.com/IlyaGrebnov/libsais, CHANGES):
  - it adds **cyclic BWT and inverse BWT "as used by bzip2"** and "significantly improved performance of inverse BWT, both single- and multi-threaded";
  - in source, `libsais_unbwt_omp(T,U,A,n,freq,i,threads)` inverts **one BWT from only the primary index** in parallel. It uses a bi-gram LF/PSI table (2 symbols per access). It marks about one hashed row per 2^k rows in P (`UNBWT_MARK`, `libsais_unbwt_mark_row`). Threads decode the chains between marks into 16,320-byte chunks plus a slack buffer, then link and stitch them;
  - constants: `UNBWT_CHAINS` = 24 slots per thread (64 for n ≥ 16 Mi with ≤ 4 threads); chain threads = max(8, n/262144) capped at threads and 64; enabled for n ≥ 262,144, so a 900k block qualifies;
  - footprint: about 5n bytes in place (README).
- [I] libsais is a direct, current, implemented answer to "cooperative multi-thread inversion of one block". k threads on one shared T-vector cut live T-vectors from N to N/k without idling cores, at the cost of a stitch pass and chunk slack. mibz2's 32 lanes are already single-thread multi-chain, so the step to multi-thread is short. libsais also uses a bi-gram table (more bytes per entry, half the steps), which trades footprint against misses; measure both. Licence (Apache-2.0 with LLVM exceptions) allows reading or porting.

---

## 5. Throughput-driven concurrency controllers

- **.NET ThreadPool:**
  - [S] Hellerstein, Morrison, Eilebrecht, SIGMETRICS PER 2009 (https://static.googleusercontent.com/media/research.google.com/en//pubs/archive/35085.pdf) model throughput vs concurrency as unimodal. They use **stochastic gradient approximation** (u_{k+1} = u_k + g·d_k), **change-point detection** to drop stale samples and detect curve shifts, and **dead-time** states while thread counts settle. Their workload model includes a memory "expansion factor" e = max{1, Σ q_i u_i}, a thrashing term analogous to your cache cliff.
  - [S] The shipped knobs include WavePeriod 4, TargetSignalToNoiseRatio 300, MaxChangePerSecond 4, SampleIntervalLow/High 10/200 ms and Bias 15 (a thread cost). It adjusts at least every 500 ms and grows only while CPU < 95% (https://mattwarren.org/2017/04/13/The-CLR-Thread-Pool-Thread-Injection-Algorithm).
  - [S] Failure in practice: "does not work very well in low-thread-count situations, and can be more intrusive than helpful, particularly in CPU-limited environments" (https://github.com/dotnet/runtime/issues/46077, 2-vCPU c5.large). A separate issue reports thread counts flipping in a periodic pattern that changed every 3–4 weeks.
- **Go:** [S] Go 1.25 sets GOMAXPROCS from the cgroup CPU limit, rounds up and re-checks periodically. It deliberately ignores CPU requests (shares) and explains throttling (100 ms periods) vs a parallelism cap (https://go.dev/blog/container-aware-gomaxprocs).
- **JVM:** [S] JDK-8281181 stopped using cpu.shares for `availableProcessors` (https://mail.openjdk.org/pipermail/hotspot-dev/2022-March/058191.html).
- **Rust / rayon:** [S] `available_parallelism` reads the affinity mask and cgroup quota. It may overcount in VMs with CPU limits or on overcommitted hosts, says nothing about SMT, and ignores system load (https://doc.rust-lang.org/std/thread/fn.available_parallelism.html).
- [I] Nothing in the mainstream runtimes chooses threads by cache size. For mibz2 a static cap from the footprint model (N_max ≈ C_eff/W) is more robust than hill-climbing, given the low-thread-count failures above and 3-round CI noise. A controller could only refine it. Use CPU-s/GiB rising as the brake signal; it is CPI²'s signal, and a slope is easier to estimate than a throughput peak.

---

## 6. Other effects

- **SMT:**
  - [S] Heracles found even a register-only spinloop sibling hurt websearch at high load, and advised against HyperThread sharing;
  - [S] indexed_bzip2 gained nothing from 12→24 threads;
  - [I] siblings split the L1D, the fill buffers (Lemire measures MLP ≈ 9 on Skylake; "10 outstanding requests per core", https://lemire.me/blog/2018/11/05/measuring-the-memory-level-parallelism-of-a-system-using-a-small-c-program/) and the L2 superqueue. 32 lanes per worker already exceed one core's MLP, so 2 workers per core gain little for the walk phase and double the live T-vectors. On the i9, workers 11–20 are siblings.
- **Allocator (measure only):**
  - [S] glibc's mmap threshold is dynamic. Freeing a block larger than the current threshold, up to 32 MiB on 64-bit, raises the threshold to that size, and the trim threshold becomes twice it (https://man7.org/linux/man-pages/man3/mallopt.3.html).
  - [I] After the first 3.6 MB T-vector is freed, later ones come from per-thread arenas instead of mmap. RSS then depends on arena count and freeing order. Log `malloc_info(3)` or mallinfo2 per run in the footprint ledger. Measurement only; no change.
- **THP:** [I] the 1–2% page-walk share says THP is a small lever. Ubuntu defaults to `madvise`, so 4 KiB pages are likely now; record `/sys/kernel/mm/transparent_hugepage/enabled` in the environment capture.
- **Prefetchers:**
  - [S] Intel documents MSR 0x1A4 bits 0–3 for the L2 streamer, L2 adjacent-line, DCU and DCU-IP prefetchers (https://github.com/LLNL/msr-safe/issues/20);
  - [S] Bubble-Up deliberately drives the prefetcher as a shared resource;
  - [I] the adjacent-line prefetcher can turn each random 64 B miss into 128 B of DRAM traffic. That costs bus bandwidth rather than extra row activations, since both lines are usually in the same DRAM row. A one-off local A/B with bit 1 set is a cheap hypothesis test (root and bare metal only; not CI).
- **DRAM activation ceiling (the likely second knee):**
  - [S] McCalpin, Intel forum (https://community.intel.com/t5/Software-Tuning-Performance/Too-low-memory-bandwidth-utilization/td-p/1134260): DDR4-2666 x8 parts have tFAW = max(20 clk, 21 ns), which allows 4 activates per 21 ns per rank. "It was common that the four-activate-window limited throughput for random accesses to 50% of peak."
  - [S] Moscibroda & Mutlu (USENIX Security 2007, https://db.usenix.org/events/sec07/tech/full_papers/moscibroda/moscibroda.pdf): FR-FCFS row-hit-first scheduling lets a streaming "memory hog" slow a co-runner 2.9× on dual core while barely slowing itself.
  - [I] Back-of-envelope for the i9 (2 channels; rank count unknown):
    - single-rank ceiling ≈ 2 × 4/21 ns ≈ **381 M random lines/s** (24 GB/s, 57% of 42.7 GB/s);
    - at 20 workers, h ≈ 20/(20×3.6) ≈ 0.28. Each output byte is one T-step, so the output ceiling ≈ 381 M/(1−0.28) ≈ **530 MB/s**, compared with the **490 MB/s** you measured;
    - per-miss occupancy 81 → 175 ns is the textbook loaded-latency rise near saturation;
    - this is a consistency check, not proof. Confirm with a random-lanes stressor's aggregate lines/s (testingmlp-style, or Intel MLC `--loaded_latency`, https://www.intel.com/content/www/us/en/download/736633/intel-memory-latency-checker-intel-mlc.html).
  - [I] Streaming writes (output, T-vector build) mixed with random reads add read/write turnaround. Refresh (a few % per rank) does not scale with threads.
- **Uncore/ring clock:** [I] on client parts the ring clock follows the core clock under load. If core clocks are constant, as you observed, the uncore is probably constant too. Record it if accessible (MSR 0x621 on Intel); it is not a lead.

---

## Synthesis: a cheap early-warning bundle [I]

1. **Calibrate** (< 1 s): a pointer-chase sweep gives C_eff (L2 and L3) and idle latency. A 1-thread and all-thread random-lanes test gives per-core MLP and the socket random-lines/s ceiling R.
2. **Ledger:** W = live bytes per worker (T-vector plus buffers) and the in-flight high-water mark.
3. **Predict:** h(N) = min(1, C_eff/(N·W)). Throughput ≈ min(N·X₁, R/(1−h(N))) bytes/s. CPU amplification rises as (1−h(N)) × miss cost. Flag when N·W > ~0.5·C_eff.
4. **Measure** 1/2/4 with CPU-s/GiB. Pin explicitly (separate cores vs siblings) and alarm on CPI²-style rules.
5. **Deterministic CI proxy:** cachegrind `--LL=4MiB,16,64` and `8MiB` LL-misses per output MiB, or drcachesim `-cores 4 -LL_size 8M`. This catches W growth without hardware.
6. **Fingerprint** (≈ 20 s, full tier): decoder vs {compute spinner, Bubble-style kernel at ½ and 2 × C_eff, Bandit-style stream, random lanes}, each run as a paired on/off measurement.
