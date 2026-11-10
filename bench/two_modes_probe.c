/*
 * The two modes' discriminator: the spin probes and the run's readings around one decode
 * (docs/journal/2026-11-10-two-modes-discriminator.md, built from §3 of
 * docs/journal/2026-11-10-two-modes.md). Standalone, unprivileged, and outside the decoder:
 * it never links mibz2 and never changes what the decode runs.
 *
 *   two_modes_probe child ARM THREADS FILE
 *       What `ab run --arms ARM@two_modes_probe` spawns for one run. It reads the counters
 *       before the run, starts the probes, and forks the real harness's child
 *       (TWO_MODES_AB child ARM THREADS FILE) with the decoder's CPU mask set before exec
 *       and read back after it. The decode's stdout and stderr are ab's own pipe and
 *       terminal, untouched: ab reads the child's JSON line exactly as it always does. While
 *       the decode runs: each of its threads' last CPU (field 39 of
 *       /proc/PID/task/TID/stat) every 2 ms, and every CPU's scaling_cur_freq every 10 ms.
 *       After it exits: its wait4 rusage, the counters again, and one JSON line appended to
 *       TWO_MODES_OUT. It exits as the decode did (its code, or its signal re-raised).
 *       Environment: TWO_MODES_AB (the harness's executable), TWO_MODES_OUT (the JSONL file),
 *       TWO_MODES_PASS (1 or 2), TWO_MODES_IDLE_CPU (P-idle's CPU), TWO_MODES_SIB_CPU (P-sib's
 *       CPU, or empty: no P-sib), TWO_MODES_DECODER_CPUS (the decoder's CPU list),
 *       TWO_MODES_SCHEDULE (each run's slot, P probed or N not, by its index in the pass).
 *
 *   two_modes_probe sustained FILE
 *       The sustained block, before the probe passes: ab's own child decodes FILE (the corpus
 *       concatenated, one process, unpinned, no probe running), its input's progress
 *       (/proc/PID/io rchar) every 10 ms, every CPU's scaling_cur_freq and its AnonHugePages
 *       every 100 ms, its counters and rusage around it; one JSON line appended to TWO_MODES_OUT.
 *       Its stdout is the child's record.
 *
 *   two_modes_probe selftest CPU MS
 *       Untimed plumbing check: one probe pinned to CPU for MS milliseconds, its pinning read
 *       back, its samples counted. One JSON line on stdout; exit 0 only if the probe was
 *       pinned exactly to CPU and sampled. Its rate is printed and never judged.
 *
 * The probe: a thread pinned to one CPU runs a loop of register arithmetic (one dependent
 * add a iteration, no memory access) and every 2^20 iterations stores rdtsc into a
 * preallocated ring. The TSC is invariant (constant_tsc, nonstop_tsc; the preflight checks
 * both), so 2^20 over the ticks between two samples is the core's effective iteration rate:
 * frequency scaling, stop-clock duty and a busy sibling all lower it. No counter, no
 * privilege.
 */
#define _GNU_SOURCE
#include <dirent.h>
#include <errno.h>
#include <fcntl.h>
#include <poll.h>
#include <pthread.h>
#include <sched.h>
#include <signal.h>
#include <stdatomic.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/prctl.h>
#include <sys/resource.h>
#include <sys/syscall.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <time.h>
#include <unistd.h>

#define SCHEMA "two_modes/probe/2"
#define ITERS_LOG2 20
#define RING (1u << 17) /* 2^17 samples: about 26 s at 0.2 ms a sample */
#define PLACE_MS 2
#define FREQ_EVERY 5 /* placement ticks per frequency reading: 10 ms */
#define MAX_TIDS 512
#define MAX_CPUS 1024

static inline uint64_t rdtsc(void) {
    uint32_t lo, hi;
    __asm__ volatile("rdtsc" : "=a"(lo), "=d"(hi));
    return ((uint64_t)hi << 32) | lo;
}

static uint64_t mono_ns(void) {
    struct timespec t;
    clock_gettime(CLOCK_MONOTONIC, &t);
    return (uint64_t)t.tv_sec * 1000000000ull + (uint64_t)t.tv_nsec;
}

static uint64_t unix_ns(void) {
    struct timespec t;
    clock_gettime(CLOCK_REALTIME, &t);
    return (uint64_t)t.tv_sec * 1000000000ull + (uint64_t)t.tv_nsec;
}

/* ---- JSON text built in memory, written in one write() at the end ---- */

typedef struct {
    char *buf;
    size_t len;
    FILE *f;
} Text;

static void text_open(Text *t) {
    t->buf = NULL;
    t->len = 0;
    t->f = open_memstream(&t->buf, &t->len);
    if (!t->f) {
        perror("two_modes_probe: open_memstream");
        exit(70);
    }
}

static void json_str(FILE *f, const char *s) {
    fputc('"', f);
    for (; *s; s++) {
        unsigned char c = (unsigned char)*s;
        if (c == '"' || c == '\\')
            fprintf(f, "\\%c", c);
        else if (c < 0x20)
            fprintf(f, "\\u%04x", c);
        else
            fputc(c, f);
    }
    fputc('"', f);
}

/* ---- small file readers ---- */

/* the whole of a small file into buf (NUL-terminated, trailing newline cut); -errno on failure */
static int read_small(const char *path, char *buf, size_t n) {
    int fd = open(path, O_RDONLY | O_CLOEXEC);
    if (fd < 0)
        return -errno;
    ssize_t r = read(fd, buf, n - 1);
    int e = errno;
    close(fd);
    if (r < 0)
        return -e;
    buf[r] = 0;
    while (r > 0 && (buf[r - 1] == '\n' || buf[r - 1] == ' '))
        buf[--r] = 0;
    return (int)r;
}

/* a CPU list ("0-8,10-18") into a cpu_set_t; -1 if malformed or out of range */
static int parse_cpus(const char *s, cpu_set_t *set) {
    CPU_ZERO(set);
    int any = 0;
    const char *p = s;
    while (*p) {
        char *end;
        long a = strtol(p, &end, 10);
        if (end == p || a < 0 || a >= CPU_SETSIZE)
            return -1;
        long b = a;
        p = end;
        if (*p == '-') {
            p++;
            b = strtol(p, &end, 10);
            if (end == p || b < a || b >= CPU_SETSIZE)
                return -1;
            p = end;
        }
        for (long c = a; c <= b; c++)
            CPU_SET(c, set);
        any = 1;
        if (*p == ',')
            p++;
        else if (*p)
            return -1;
    }
    return any ? 0 : -1;
}

/* a cpu_set_t as a CPU list, "0-8,10-18" */
static void format_cpus(const cpu_set_t *set, char *out, size_t n) {
    size_t k = 0;
    out[0] = 0;
    for (int c = 0; c < CPU_SETSIZE; c++) {
        if (!CPU_ISSET(c, set))
            continue;
        int d = c;
        while (d + 1 < CPU_SETSIZE && CPU_ISSET(d + 1, set))
            d++;
        int w = d == c ? snprintf(out + k, n - k, "%s%d", k ? "," : "", c)
                       : snprintf(out + k, n - k, "%s%d-%d", k ? "," : "", c, d);
        if (w < 0 || (size_t)w >= n - k)
            break;
        k += (size_t)w;
        c = d;
    }
}

/* ---- the probe ---- */

typedef struct {
    const char *name;
    int cpu;
    pthread_t th;
    uint64_t *ring;
    _Atomic uint64_t seq;
    _Atomic int stop;
    _Atomic int ready; /* 1 pinned and read back, -1 failed */
    char readback[256];
    int err;
    int started; /* the thread was created (and must be joined) */
    long long cpu_ns; /* the thread's own CPU time from its first sample to its last */
} Probe;

static void *probe_main(void *arg) {
    Probe *p = arg;
    char nm[16];
    snprintf(nm, sizeof nm, "tm-%s", p->name);
    prctl(PR_SET_NAME, nm, 0, 0, 0);
    cpu_set_t set, got;
    CPU_ZERO(&set);
    CPU_SET(p->cpu, &set);
    if (sched_setaffinity(0, sizeof set, &set) != 0) {
        p->err = errno;
        atomic_store(&p->ready, -1);
        return NULL;
    }
    /* read back: the thread's own mask as the kernel now holds it */
    if (sched_getaffinity(0, sizeof got, &got) != 0) {
        p->err = errno;
        atomic_store(&p->ready, -1);
        return NULL;
    }
    format_cpus(&got, p->readback, sizeof p->readback);
    if (!CPU_EQUAL(&set, &got) || sched_getcpu() != p->cpu) {
        atomic_store(&p->ready, -1);
        return NULL;
    }
    atomic_store(&p->ready, 1);
    uint64_t seq = 0;
    uint64_t x = 0;
    struct timespec c0, c1;
    int have_c0 = 0;
    while (!atomic_load_explicit(&p->stop, memory_order_relaxed)) {
        uint64_t n = 1ull << ITERS_LOG2;
        /* one dependent add an iteration, a counted loop: registers only */
        __asm__ volatile("1:\n\t"
                         "add $1, %0\n\t"
                         "dec %1\n\t"
                         "jnz 1b\n\t"
                         : "+r"(x), "+r"(n)
                         :
                         : "cc");
        p->ring[seq & (RING - 1)] = rdtsc();
        if (!have_c0)
            have_c0 = clock_gettime(CLOCK_THREAD_CPUTIME_ID, &c0) == 0;
        seq++;
        atomic_store_explicit(&p->seq, seq, memory_order_release);
    }
    /* (its CPU time over its sampled span: whether it was descheduled, apart from how fast it
     * ran while scheduled; the TSC alone cannot say why progress stopped) */
    if (have_c0 && clock_gettime(CLOCK_THREAD_CPUTIME_ID, &c1) == 0)
        p->cpu_ns = (long long)(c1.tv_sec - c0.tv_sec) * 1000000000LL + (c1.tv_nsec - c0.tv_nsec);
    else
        p->cpu_ns = -1;
    return NULL;
}

static int probe_start(Probe *p, const char *name, int cpu) {
    memset(p, 0, sizeof *p);
    p->name = name;
    p->cpu = cpu;
    p->ring = malloc(RING * sizeof(uint64_t));
    if (!p->ring)
        return -1;
    memset(p->ring, 0, RING * sizeof(uint64_t)); /* touched now: no fault in the loop */
    if (pthread_create(&p->th, NULL, probe_main, p) != 0)
        return -1;
    p->started = 1;
    /* wait (at most 2 s) for its pinning and its first sample */
    uint64_t t0 = mono_ns();
    while (mono_ns() - t0 < 2000000000ull) {
        int r = atomic_load(&p->ready);
        if (r < 0)
            return -1;
        if (r > 0 && atomic_load(&p->seq) >= 1)
            return 0;
        sched_yield();
    }
    return -1;
}

static void probe_stop(Probe *p) {
    atomic_store(&p->stop, 1);
    if (p->started)
        pthread_join(p->th, NULL);
    p->started = 0;
}

static void probe_json(FILE *f, Probe *p) {
    uint64_t n = atomic_load(&p->seq);
    uint64_t kept = n < RING ? n : RING;
    uint64_t first = n - kept;
    fprintf(f, "{\"name\":");
    json_str(f, p->name);
    fprintf(f, ",\"cpu\":%d,\"readback\":", p->cpu);
    json_str(f, p->readback);
    fprintf(f, ",\"pinned\":%s,\"err\":%d,\"iters_per_sample\":%llu,\"n\":%llu,\"dropped\":%llu",
            atomic_load(&p->ready) > 0 ? "true" : "false", p->err, 1ull << ITERS_LOG2,
            (unsigned long long)n, (unsigned long long)first);
    if (p->cpu_ns >= 0)
        fprintf(f, ",\"cpu_ns\":%lld", p->cpu_ns);
    else
        fprintf(f, ",\"cpu_ns\":null");
    if (kept)
        fprintf(f, ",\"last_tsc\":%llu", (unsigned long long)p->ring[(n - 1) & (RING - 1)]);
    if (kept) {
        uint64_t prev = p->ring[first & (RING - 1)];
        fprintf(f, ",\"first_tsc\":%llu,\"dtsc\":[", (unsigned long long)prev);
        for (uint64_t i = first + 1; i < n; i++) {
            uint64_t v = p->ring[i & (RING - 1)];
            fprintf(f, "%s%llu", i == first + 1 ? "" : ",", (unsigned long long)(v - prev));
            prev = v;
        }
        fprintf(f, "]");
    } else {
        fprintf(f, ",\"first_tsc\":null,\"dtsc\":[]");
    }
    fprintf(f, "}");
}

/* ---- the counters before and after: thermal_throttle and vmstat ---- */

static int ncpus_conf(void) {
    long n = sysconf(_SC_NPROCESSORS_CONF);
    return n < 1 ? 1 : (n > MAX_CPUS ? MAX_CPUS : (int)n);
}

static void counters_json(FILE *f, int ncpu) {
    char path[256], val[256];
    fprintf(f, "{\"t_unix_ns\":%llu,\"thermal_throttle\":{", (unsigned long long)unix_ns());
    for (int c = 0; c < ncpu; c++) {
        snprintf(path, sizeof path, "/sys/devices/system/cpu/cpu%d/thermal_throttle", c);
        fprintf(f, "%s\"%d\":", c ? "," : "", c);
        DIR *d = opendir(path);
        if (!d) {
            fprintf(f, "{\"why\":");
            json_str(f, strerror(errno));
            fprintf(f, "}");
            continue;
        }
        /* every file in the directory, as named: *_throttle_count, *_power_limit_count,
         * *_throttle_max_time_ms, *_throttle_total_time_ms, whichever the kernel has */
        struct dirent *e;
        int first = 1;
        fprintf(f, "{");
        while ((e = readdir(d))) {
            if (e->d_name[0] == '.')
                continue;
            char p2[512];
            snprintf(p2, sizeof p2, "%s/%s", path, e->d_name);
            int r = read_small(p2, val, sizeof val);
            fprintf(f, "%s", first ? "" : ",");
            first = 0;
            json_str(f, e->d_name);
            char *end;
            long long v = r > 0 ? strtoll(val, &end, 10) : 0;
            if (r > 0 && *end == 0)
                fprintf(f, ":%lld", v);
            else {
                fprintf(f, ":");
                json_str(f, r < 0 ? strerror(-r) : val);
            }
        }
        closedir(d);
        fprintf(f, "}");
    }
    fprintf(f, "},\"vmstat\":");
    FILE *v = fopen("/proc/vmstat", "re");
    if (!v) {
        fprintf(f, "{\"why\":");
        json_str(f, strerror(errno));
        fprintf(f, "}}");
        return;
    }
    char line[256];
    int first = 1;
    fprintf(f, "{");
    while (fgets(line, sizeof line, v)) {
        char key[128];
        unsigned long long n;
        if (sscanf(line, "%127s %llu", key, &n) != 2)
            continue;
        if (strncmp(key, "thp_", 4) && strcmp(key, "pgfault") && strcmp(key, "pgmajfault") &&
            strncmp(key, "compact_", 8))
            continue;
        fprintf(f, "%s\"%s\":%llu", first ? "" : ",", key, n);
        first = 0;
    }
    fclose(v);
    fprintf(f, "}}");
}

/* ---- placement: each task's comm, state and last CPU (field 39) ---- */

typedef struct {
    int tid;
    char comm[32];
} Tid;

static Tid tids[MAX_TIDS];
static int ntids;

static void remember(int tid, const char *comm) {
    for (int i = 0; i < ntids; i++)
        if (tids[i].tid == tid) {
            snprintf(tids[i].comm, sizeof tids[i].comm, "%s", comm); /* the latest name */
            return;
        }
    if (ntids < MAX_TIDS) {
        tids[ntids].tid = tid;
        snprintf(tids[ntids].comm, sizeof tids[ntids].comm, "%s", comm);
        ntids++;
    }
}

/* one placement sample: [t_us,[[tid,cpu,"S"],...]] */
static void place_sample(FILE *f, int pid, long t_us, int first) {
    char path[128], buf[1024];
    snprintf(path, sizeof path, "/proc/%d/task", pid);
    DIR *d = opendir(path);
    if (!d)
        return;
    fprintf(f, "%s[%ld,[", first ? "" : ",", t_us);
    struct dirent *e;
    int k = 0;
    while ((e = readdir(d))) {
        if (e->d_name[0] < '0' || e->d_name[0] > '9')
            continue;
        char p2[320];
        snprintf(p2, sizeof p2, "/proc/%d/task/%s/stat", pid, e->d_name);
        if (read_small(p2, buf, sizeof buf) <= 0)
            continue;
        char *l = strchr(buf, '(');
        char *r = strrchr(buf, ')');
        if (!l || !r || r < l)
            continue;
        char comm[32];
        size_t cl = (size_t)(r - l - 1);
        if (cl >= sizeof comm)
            cl = sizeof comm - 1;
        memcpy(comm, l + 1, cl);
        comm[cl] = 0;
        /* fields after ')': 3 (state) is the first; 39 (processor) the 37th */
        char *s = r + 2;
        char state = *s;
        int field = 3;
        char *tok = s;
        while (field < 39 && tok) {
            tok = strchr(tok, ' ');
            if (tok)
                tok++;
            field++;
        }
        if (!tok)
            continue;
        int cpu = atoi(tok);
        int tid = atoi(e->d_name);
        remember(tid, comm);
        fprintf(f, "%s[%d,%d,\"%c\"]", k ? "," : "", tid, cpu, state);
        k++;
    }
    closedir(d);
    fprintf(f, "]]");
}

/* ---- the clock during the run: every CPU's scaling_cur_freq ---- */

static int freq_fd[MAX_CPUS];

static void freq_open(int ncpu) {
    char path[128];
    for (int c = 0; c < ncpu; c++) {
        snprintf(path, sizeof path, "/sys/devices/system/cpu/cpu%d/cpufreq/scaling_cur_freq", c);
        freq_fd[c] = open(path, O_RDONLY | O_CLOEXEC);
    }
}

/* one reading: [t_us,read_us,[khz,...]] (null where unreadable) */
static void freq_sample(FILE *f, int ncpu, uint64_t t0, int first) {
    char buf[64];
    uint64_t a = mono_ns();
    long khz[MAX_CPUS];
    for (int c = 0; c < ncpu; c++) {
        khz[c] = -1;
        if (freq_fd[c] < 0)
            continue;
        ssize_t r = pread(freq_fd[c], buf, sizeof buf - 1, 0);
        if (r > 0) {
            buf[r] = 0;
            khz[c] = atol(buf);
        }
    }
    uint64_t b = mono_ns();
    fprintf(f, "%s[%ld,%ld,[", first ? "" : ",", (long)((a - t0) / 1000), (long)((b - a) / 1000));
    for (int c = 0; c < ncpu; c++) {
        if (khz[c] < 0)
            fprintf(f, "%snull", c ? "," : "");
        else
            fprintf(f, "%s%ld", c ? "," : "", khz[c]);
    }
    fprintf(f, "]]");
}

/* ---- the child's huge pages: /proc/PID/smaps_rollup's AnonHugePages, every 10 ms ---- */

static long smaps_last_kb = -1, smaps_max_kb = -1, smaps_last_us = -1, smaps_anon_kb = -1;
static long smaps_reads, smaps_fails;

static void smaps_sample(int pid, long t_us) {
    char path[64], buf[4096];
    snprintf(path, sizeof path, "/proc/%d/smaps_rollup", pid);
    if (read_small(path, buf, sizeof buf) <= 0) {
        smaps_fails++;
        return;
    }
    char *p = strstr(buf, "AnonHugePages:");
    char *a = strstr(buf, "\nAnonymous:");
    if (!p) {
        smaps_fails++;
        return;
    }
    smaps_reads++;
    smaps_last_kb = atol(p + strlen("AnonHugePages:"));
    if (a)
        smaps_anon_kb = atol(a + strlen("\nAnonymous:"));
    smaps_last_us = t_us;
    if (smaps_last_kb > smaps_max_kb)
        smaps_max_kb = smaps_last_kb;
}

/* ---- the run ---- */

static const char *need(const char *k) {
    const char *v = getenv(k);
    if (!v) {
        fprintf(stderr, "two_modes_probe: %s is not set (run through the recipe)\n", k);
        exit(70);
    }
    return v;
}

static int pidfd_open_(pid_t pid) {
#ifdef SYS_pidfd_open
    return (int)syscall(SYS_pidfd_open, pid, 0);
#else
    (void)pid;
    errno = ENOSYS;
    return -1;
#endif
}

static int run_child(int argc, char **argv) {
    if (argc != 5) {
        fprintf(stderr, "usage: two_modes_probe child ARM THREADS FILE\n");
        return 64;
    }
    const char *ab = need("TWO_MODES_AB");
    const char *out = need("TWO_MODES_OUT");
    int pass = atoi(need("TWO_MODES_PASS"));
    int idle_cpu = atoi(need("TWO_MODES_IDLE_CPU"));
    const char *sib = need("TWO_MODES_SIB_CPU");
    const char *dec = need("TWO_MODES_DECODER_CPUS");
    const char *sched = need("TWO_MODES_SCHEDULE");
    int sib_cpu = *sib ? atoi(sib) : -1;
    /* this run's index in the pass (the records already in TWO_MODES_OUT), and whether its
     * slot in the fixed schedule ("P" probed, "N" no probe, repeating) runs the probes */
    long run_index = 0;
    {
        FILE *o = fopen(out, "re");
        if (o) {
            int ch;
            while ((ch = fgetc(o)) != EOF)
                run_index += ch == '\n';
            fclose(o);
        }
    }
    size_t sl = strlen(sched);
    if (sl == 0 || strspn(sched, "PN") != sl) {
        fprintf(stderr, "two_modes_probe: TWO_MODES_SCHEDULE %s is not a P/N schedule\n", sched);
        return 70;
    }
    int probed = sched[run_index % (long)sl] == 'P';
    cpu_set_t dec_set;
    if (parse_cpus(dec, &dec_set) != 0) {
        fprintf(stderr, "two_modes_probe: TWO_MODES_DECODER_CPUS %s is not a CPU list\n", dec);
        return 70;
    }
    prctl(PR_SET_NAME, "tm-wrap", 0, 0, 0);
    int ncpu = ncpus_conf();
    Text t;
    text_open(&t);
    FILE *f = t.f;
    uint64_t t_unix0 = unix_ns();
    uint64_t mono0 = mono_ns(), tsc0 = rdtsc();
    fprintf(f, "{\"schema\":\"%s\",\"pass\":%d,\"run_index\":%ld,\"probed\":%s,\"schedule\":", SCHEMA,
            pass, run_index, probed ? "true" : "false");
    json_str(f, sched);
    fprintf(f, ",\"arm\":");
    json_str(f, argv[2]);
    fprintf(f, ",\"threads\":%d,\"file\":", atoi(argv[3]));
    json_str(f, argv[4]);
    fprintf(f, ",\"t_unix_ns_start\":%llu,\"before\":", (unsigned long long)t_unix0);
    counters_json(f, ncpu);
    /* the probes, pinned and sampling before the decode starts (none in an "N" slot: the
     * decoder's mask is the same, the probe core left idle) */
    Probe probes[2];
    int np = 0, perr = 0;
    if (probed && probe_start(&probes[np++], "P-idle", idle_cpu) != 0)
        perr = 1;
    if (probed && !perr && sib_cpu >= 0 && probe_start(&probes[np++], "P-sib", sib_cpu) != 0)
        perr = 1;
    if (perr) {
        fprintf(stderr, "two_modes_probe: a probe could not be pinned to its CPU\n");
        for (int i = 0; i < np; i++)
            probe_stop(&probes[i]);
        return 71;
    }
    freq_open(ncpu);
    /* the decode: the decoder's mask set in the child before exec, the exec confirmed by
     * the close-on-exec pipe's EOF */
    int pp[2];
    if (pipe2(pp, O_CLOEXEC) != 0) {
        perror("two_modes_probe: pipe2");
        return 70;
    }
    uint64_t tsc_fork = rdtsc();
    pid_t pid = fork();
    if (pid < 0) {
        perror("two_modes_probe: fork");
        return 70;
    }
    if (pid == 0) {
        close(pp[0]);
        int e = 0;
        if (sched_setaffinity(0, sizeof dec_set, &dec_set) != 0) {
            e = errno;
            if (write(pp[1], &e, sizeof e) < 0) {
            }
            _exit(71);
        }
        char *args[] = {(char *)ab, "child", argv[2], argv[3], argv[4], NULL};
        execv(ab, args);
        e = errno;
        if (write(pp[1], &e, sizeof e) < 0) {
        }
        _exit(127);
    }
    close(pp[1]);
    int child_err = 0;
    ssize_t got = read(pp[0], &child_err, sizeof child_err);
    close(pp[0]);
    uint64_t tsc_exec = rdtsc();
    uint64_t mono_exec = mono_ns();
    char readback[512] = "", status_list[512] = "";
    cpu_set_t rb;
    if (got == 0 && sched_getaffinity(pid, sizeof rb, &rb) == 0)
        format_cpus(&rb, readback, sizeof readback);
    if (got == 0) {
        char path[64], buf[4096];
        snprintf(path, sizeof path, "/proc/%d/status", pid);
        if (read_small(path, buf, sizeof buf) > 0) {
            char *p = strstr(buf, "Cpus_allowed_list:");
            if (p) {
                p += strlen("Cpus_allowed_list:");
                while (*p == ' ' || *p == '\t')
                    p++;
                size_t n = strcspn(p, "\n");
                if (n >= sizeof status_list)
                    n = sizeof status_list - 1;
                memcpy(status_list, p, n);
                status_list[n] = 0;
            }
        }
    }
    /* the samplers, until the decode exits: placement every 2 ms, the clock every 10 ms */
    Text pl, fq;
    text_open(&pl);
    text_open(&fq);
    int pfd = pidfd_open_(pid);
    int status = 0;
    struct rusage ru;
    memset(&ru, 0, sizeof ru);
    long ticks = 0, nfreq = 0;
    uint64_t next = mono_exec;
    for (;;) {
        uint64_t now = mono_ns();
        if (now >= next) {
            place_sample(pl.f, pid, (long)((now - mono_exec) / 1000), ticks == 0);
            if (ticks % FREQ_EVERY == 0)
            {
                freq_sample(fq.f, ncpu, mono_exec, nfreq++ == 0);
                smaps_sample(pid, (long)((mono_ns() - mono_exec) / 1000));
            }
            ticks++;
            next += PLACE_MS * 1000000ull;
            if (next < now)
                next = now + PLACE_MS * 1000000ull;
        }
        now = mono_ns();
        int wait_ms = next > now ? (int)((next - now + 999999) / 1000000) : 0;
        if (pfd >= 0) {
            struct pollfd q = {.fd = pfd, .events = POLLIN};
            poll(&q, 1, wait_ms);
        } else {
            struct timespec ts = {0, (long)(next > now ? next - now : 0)};
            nanosleep(&ts, NULL);
        }
        pid_t w = wait4(pid, &status, WNOHANG, &ru);
        if (w == pid)
            break;
        if (w < 0 && errno != EINTR) {
            perror("two_modes_probe: wait4");
            break;
        }
    }
    uint64_t tsc_exit = rdtsc();
    uint64_t mono_exit = mono_ns();
    if (pfd >= 0)
        close(pfd);
    for (int i = 0; i < np; i++)
        probe_stop(&probes[i]);
    fclose(pl.f);
    fclose(fq.f);
    uint64_t tsc1 = rdtsc(), mono1 = mono_ns();
    double tsc_hz = (double)(tsc1 - tsc0) * 1e9 / (double)(mono1 - mono0);
    fprintf(f, ",\"after\":");
    counters_json(f, ncpu);
    fprintf(f, ",\"pid\":%d,\"exec_errno\":%d", pid, got > 0 ? child_err : 0);
    fprintf(f,
            ",\"smaps\":{\"every_ms\":%d,\"reads\":%ld,\"fails\":%ld,\"anon_huge_kb_last\":%ld,"
            "\"anon_huge_kb_max\":%ld,\"anonymous_kb_last\":%ld,\"last_t_us\":%ld}",
            PLACE_MS * FREQ_EVERY, smaps_reads, smaps_fails, smaps_last_kb, smaps_max_kb,
            smaps_anon_kb, smaps_last_us);
    fprintf(f, ",\"decoder_cpus\":");
    json_str(f, dec);
    fprintf(f, ",\"decoder_cpus_readback\":");
    json_str(f, readback);
    fprintf(f, ",\"decoder_cpus_status\":");
    json_str(f, status_list);
    fprintf(f,
            ",\"tsc\":{\"start\":%llu,\"fork\":%llu,\"exec\":%llu,\"exit\":%llu,\"end\":%llu},"
            "\"mono_ns\":{\"start\":%llu,\"exec\":%llu,\"exit\":%llu,\"end\":%llu},"
            "\"tsc_hz\":%.1f",
            (unsigned long long)tsc0, (unsigned long long)tsc_fork, (unsigned long long)tsc_exec,
            (unsigned long long)tsc_exit, (unsigned long long)tsc1, (unsigned long long)mono0,
            (unsigned long long)mono_exec, (unsigned long long)mono_exit,
            (unsigned long long)mono1, tsc_hz);
    fprintf(f, ",\"probes\":[");
    for (int i = 0; i < np; i++) {
        if (i)
            fprintf(f, ",");
        probe_json(f, &probes[i]);
    }
    fprintf(f, "],\"placement\":{\"every_ms\":%d,\"comm\":{", PLACE_MS);
    for (int i = 0; i < ntids; i++) {
        fprintf(f, "%s\"%d\":", i ? "," : "", tids[i].tid);
        json_str(f, tids[i].comm);
    }
    fprintf(f, "},\"samples\":[%s]}", pl.buf ? pl.buf : "");
    fprintf(f, ",\"freq\":{\"every_ms\":%d,\"cpus\":%d,\"samples\":[%s]}", PLACE_MS * FREQ_EVERY,
            ncpu, fq.buf ? fq.buf : "");
    fprintf(f,
            ",\"rusage\":{\"minflt\":%ld,\"majflt\":%ld,\"utime_s\":%.6f,\"stime_s\":%.6f,"
            "\"maxrss_kb\":%ld,\"nvcsw\":%ld,\"nivcsw\":%ld}",
            ru.ru_minflt, ru.ru_majflt, ru.ru_utime.tv_sec + ru.ru_utime.tv_usec / 1e6,
            ru.ru_stime.tv_sec + ru.ru_stime.tv_usec / 1e6, ru.ru_maxrss, ru.ru_nvcsw,
            ru.ru_nivcsw);
    if (WIFEXITED(status))
        fprintf(f, ",\"status\":{\"exit\":%d}", WEXITSTATUS(status));
    else if (WIFSIGNALED(status))
        fprintf(f, ",\"status\":{\"signal\":%d}", WTERMSIG(status));
    else
        fprintf(f, ",\"status\":{\"unknown\":%d}", status);
    fprintf(f, "}\n");
    fclose(f);
    int fd = open(out, O_WRONLY | O_CREAT | O_APPEND | O_CLOEXEC, 0644);
    if (fd < 0 || write(fd, t.buf, t.len) != (ssize_t)t.len) {
        perror("two_modes_probe: writing TWO_MODES_OUT");
        return 70;
    }
    close(fd);
    if (WIFSIGNALED(status)) {
        signal(WTERMSIG(status), SIG_DFL);
        raise(WTERMSIG(status));
    }
    return WIFEXITED(status) ? WEXITSTATUS(status) : 70;
}

/* The sustained block: one process (ab's own child, unpinned, no probe) decoding FILE, a
 * concatenation of the corpus, end to end. Every 10 ms the bytes it has read of its input
 * (/proc/PID/io rchar: the input's progress; the decoder reads it in chunks as it goes, and
 * the analysis sums it into 100 ms windows); every 100 ms every CPU's scaling_cur_freq and
 * its AnonHugePages. Around it: the counters and its rusage. */
static long io_rchar(int pid) {
    char path[64], buf[1024];
    snprintf(path, sizeof path, "/proc/%d/io", pid);
    if (read_small(path, buf, sizeof buf) <= 0)
        return -1;
    char *p = strstr(buf, "rchar:");
    return p ? atol(p + strlen("rchar:")) : -1;
}

static int run_sustained(int argc, char **argv) {
    if (argc != 3) {
        fprintf(stderr, "usage: two_modes_probe sustained FILE\n");
        return 64;
    }
    const char *ab = need("TWO_MODES_AB");
    const char *out = need("TWO_MODES_OUT");
    prctl(PR_SET_NAME, "tm-sustain", 0, 0, 0);
    int ncpu = ncpus_conf();
    Text t, io, fq;
    text_open(&t);
    text_open(&io);
    text_open(&fq);
    FILE *f = t.f;
    fprintf(f, "{\"schema\":\"two_modes/sustained/1\",\"file\":");
    json_str(f, argv[2]);
    fprintf(f, ",\"t_unix_ns_start\":%llu,\"before\":", (unsigned long long)unix_ns());
    counters_json(f, ncpu);
    freq_open(ncpu);
    uint64_t m0 = mono_ns();
    pid_t pid = fork();
    if (pid < 0) {
        perror("two_modes_probe: fork");
        return 70;
    }
    if (pid == 0) {
        char *args[] = {(char *)ab, "child", "mibz2-default33", "10", argv[2], NULL};
        execv(ab, args);
        _exit(127);
    }
    int pfd = pidfd_open_(pid);
    int status = 0;
    struct rusage ru;
    memset(&ru, 0, sizeof ru);
    long n = 0;
    uint64_t next = m0;
    for (;;) {
        uint64_t now = mono_ns();
        if (now >= next) {
            long r = io_rchar(pid);
            fprintf(io.f, "%s[%ld,%ld]", n ? "," : "", (long)((now - m0) / 1000), r);
            if (n % 10 == 0) {
                freq_sample(fq.f, ncpu, m0, n == 0);
                smaps_sample(pid, (long)((now - m0) / 1000));
            }
            n++;
            next += 10000000ull;
            if (next < now)
                next = now + 10000000ull;
        }
        now = mono_ns();
        int wait_ms = next > now ? (int)((next - now + 999999) / 1000000) : 0;
        if (pfd >= 0) {
            struct pollfd q = {.fd = pfd, .events = POLLIN};
            poll(&q, 1, wait_ms);
        } else {
            struct timespec ts = {0, (long)(next > now ? next - now : 0)};
            nanosleep(&ts, NULL);
        }
        pid_t w = wait4(pid, &status, WNOHANG, &ru);
        if (w == pid || (w < 0 && errno != EINTR))
            break;
    }
    uint64_t m1 = mono_ns();
    if (pfd >= 0)
        close(pfd);
    fclose(io.f);
    fclose(fq.f);
    fprintf(f, ",\"after\":");
    counters_json(f, ncpu);
    fprintf(f, ",\"pid\":%d,\"span_s\":%.6f,\"io\":{\"every_ms\":10,\"samples\":[%s]}", pid,
            (m1 - m0) / 1e9, io.buf ? io.buf : "");
    fprintf(f, ",\"freq\":{\"every_ms\":100,\"cpus\":%d,\"samples\":[%s]}", ncpu,
            fq.buf ? fq.buf : "");
    fprintf(f,
            ",\"smaps\":{\"every_ms\":100,\"reads\":%ld,\"fails\":%ld,\"anon_huge_kb_last\":%ld,"
            "\"anon_huge_kb_max\":%ld,\"anonymous_kb_last\":%ld}",
            smaps_reads, smaps_fails, smaps_last_kb, smaps_max_kb, smaps_anon_kb);
    fprintf(f,
            ",\"rusage\":{\"minflt\":%ld,\"majflt\":%ld,\"utime_s\":%.6f,\"stime_s\":%.6f,"
            "\"maxrss_kb\":%ld}",
            ru.ru_minflt, ru.ru_majflt, ru.ru_utime.tv_sec + ru.ru_utime.tv_usec / 1e6,
            ru.ru_stime.tv_sec + ru.ru_stime.tv_usec / 1e6, ru.ru_maxrss);
    if (WIFEXITED(status))
        fprintf(f, ",\"status\":{\"exit\":%d}}\n", WEXITSTATUS(status));
    else if (WIFSIGNALED(status))
        fprintf(f, ",\"status\":{\"signal\":%d}}\n", WTERMSIG(status));
    else
        fprintf(f, ",\"status\":{\"unknown\":%d}}\n", status);
    fclose(f);
    int fd = open(out, O_WRONLY | O_CREAT | O_APPEND | O_CLOEXEC, 0644);
    if (fd < 0 || write(fd, t.buf, t.len) != (ssize_t)t.len) {
        perror("two_modes_probe: writing TWO_MODES_OUT");
        return 70;
    }
    close(fd);
    return WIFEXITED(status) ? WEXITSTATUS(status) : 70;
}

static int run_selftest(int argc, char **argv) {
    if (argc != 4) {
        fprintf(stderr, "usage: two_modes_probe selftest CPU MS\n");
        return 64;
    }
    int cpu = atoi(argv[2]);
    int ms = atoi(argv[3]);
    uint64_t m0 = mono_ns(), t0 = rdtsc();
    Probe p;
    int ok = probe_start(&p, "P-idle", cpu) == 0;
    if (ok) {
        struct timespec ts = {ms / 1000, (long)(ms % 1000) * 1000000L};
        nanosleep(&ts, NULL);
    }
    probe_stop(&p);
    uint64_t m1 = mono_ns(), t1 = rdtsc();
    double tsc_hz = (double)(t1 - t0) * 1e9 / (double)(m1 - m0);
    uint64_t n = atomic_load(&p.seq);
    /* the rate over the whole span (never judged: plumbing only) */
    double rate = 0;
    if (n >= 2) {
        uint64_t a = p.ring[0], b = p.ring[(n - 1) & (RING - 1)];
        if (n <= RING && b > a)
            rate = (double)((n - 1) << ITERS_LOG2) * tsc_hz / (double)(b - a) / 1e9;
    }
    printf("{\"schema\":\"two_modes/selftest/1\",\"cpu\":%d,\"readback\":", cpu);
    json_str(stdout, p.readback);
    printf(",\"pinned\":%s,\"err\":%d,\"ms\":%d,\"samples\":%llu,\"tsc_hz\":%.1f,"
           "\"giter_per_s\":%.4f}\n",
           atomic_load(&p.ready) > 0 ? "true" : "false", p.err, ms, (unsigned long long)n,
           tsc_hz, rate);
    char want[16];
    snprintf(want, sizeof want, "%d", cpu);
    return ok && strcmp(p.readback, want) == 0 && n >= 2 ? 0 : 1;
}

int main(int argc, char **argv) {
    if (argc >= 2 && strcmp(argv[1], "child") == 0)
        return run_child(argc, argv);
    if (argc >= 2 && strcmp(argv[1], "selftest") == 0)
        return run_selftest(argc, argv);
    if (argc >= 2 && strcmp(argv[1], "sustained") == 0)
        return run_sustained(argc, argv);
    fprintf(stderr,
            "usage: two_modes_probe child ARM THREADS FILE | sustained FILE | selftest CPU MS\n");
    return 64;
}
