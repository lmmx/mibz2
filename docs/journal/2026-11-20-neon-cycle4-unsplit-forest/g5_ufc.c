/*
 * Cycle 4 · G5 Paid-Cost Falsifier & Native VM Timing Diagnostic:
 *   1. Shipped Forward Open<16, INDUCE=true, QUEUE=true> (baseline)
 *   2. UFC-Full: Flat-Round-1 Unsplit-Piece Functional In-Forest Contraction (8-way MLP)
 *                + Iterated L2-Resident UFC (Rounds 2..K) + Direct-Rotated First-Occurrence SLP Emit
 *   3. UFC-1 + Open: Flat-Round-1 UFC (8-way MLP) -> Open Endpoint on reduced thick strips (592-2,046)
 *                    + L2-resident Open edges walk (26k-70k retained rows!)
 *
 * Pre-registered PASS/FAIL rule (fixed before compilation/execution):
 *   For a candidate (UFC-Full OR UFC-1+Open) vs Shipped Forward Open, PASS G5 iff on ALL 6 blocks:
 *     1. Byte-exact output on all 5,399,692 bytes;
 *     2. Total modelled 256 KiB L2 misses <= 0.85x Shipped Open;
 *     3. Total modelled 2 MiB LLC DRAM B/B <= 0.90x Shipped Open;
 *     4. Peak live working set M <= 1.15x Shipped Open;
 *     5. Uninstrumented -O3 C VM diagnostic time <= 0.90x Shipped Open.
 *   Otherwise FAIL G5 (and report full phase breakdown in both cache model and VM timing).
 *
 * Exact 3-level 64B-line LRU cache model matching Cycles 1-3 / Astra D:
 *   L1D: 32 KiB 8-way | L2: 256 KiB 4-way | LLC: 2 MiB 16-way | write-allocate + drain
 *
 * Build (cache sim): gcc -O3 -std=c99 -Wall -Wextra -Werror g5_ufc.c -o g5_ufc
 * Build (VM timing): gcc -O3 -std=c99 -Wall -Wextra -Werror -DNO_CACHE_SIM g5_ufc.c -o g5_ufc_bench
 * Run:               ./g5_ufc <path-to-admitted-bwt-dir>
 */

#define _POSIX_C_SOURCE 200809L
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <assert.h>
#include <time.h>

/* ==================== 3-Level 64B-line LRU Cache Simulator ==================== */

typedef struct { uint64_t tag; uint32_t lru; uint8_t valid; uint8_t dirty; } CacheLine;
typedef struct { int sets, ways; CacheLine *lines; uint32_t clock; uint64_t accesses, misses, writebacks; } CacheLevel;
typedef struct { CacheLevel l1, l2, llc; uint64_t dram_reads, dram_writes, next_region_addr; } CacheSim;

static void level_init(CacheLevel *lv, int sets, int ways) {
    lv->sets = sets; lv->ways = ways;
#ifndef NO_CACHE_SIM
    lv->lines = (CacheLine *)calloc((size_t)sets * (size_t)ways, sizeof(CacheLine));
#else
    lv->lines = NULL;
#endif
    lv->clock = 0; lv->accesses = 0; lv->misses = 0; lv->writebacks = 0;
}
static void level_free(CacheLevel *lv) { free(lv->lines); }
static void sim_init(CacheSim *s) {
    level_init(&s->l1, 64, 8);
    level_init(&s->l2, 1024, 4);
    level_init(&s->llc, 2048, 16);
    s->dram_reads = 0; s->dram_writes = 0; s->next_region_addr = 0x100000ULL;
}
static void sim_free(CacheSim *s) { level_free(&s->l1); level_free(&s->l2); level_free(&s->llc); }
static uint64_t sim_alloc_region(CacheSim *s, size_t bytes) {
    uint64_t base = s->next_region_addr;
    uint64_t aligned = (bytes + 4095ULL) & ~4095ULL;
    s->next_region_addr += aligned + 4096ULL;
    return base;
}

#ifndef NO_CACHE_SIM
static int level_access(CacheLevel *lv, uint64_t line_addr, int is_write,
                        int *evicted_dirty, uint64_t *evicted_line) {
    lv->accesses++; lv->clock++;
    int shift = __builtin_ctz(lv->sets);
    int set = (int)(line_addr & (uint64_t)(lv->sets - 1));
    uint64_t tag = line_addr >> shift;
    CacheLine *way = &lv->lines[(size_t)set * (size_t)lv->ways];
    int victim = 0; uint32_t min_lru = UINT32_MAX;
    for (int w = 0; w < lv->ways; w++) {
        if (way[w].valid && way[w].tag == tag) {
            way[w].lru = lv->clock;
            if (is_write) way[w].dirty = 1;
            *evicted_dirty = 0;
            return 1;
        }
        if (!way[w].valid) { victim = w; min_lru = 0; }
        else if (min_lru > 0 && way[w].lru < min_lru) { min_lru = way[w].lru; victim = w; }
    }
    lv->misses++;
    if (way[victim].valid && way[victim].dirty) {
        *evicted_dirty = 1;
        *evicted_line = (way[victim].tag << shift) | (uint64_t)set;
        lv->writebacks++;
    } else *evicted_dirty = 0;
    way[victim].valid = 1; way[victim].tag = tag;
    way[victim].dirty = (uint8_t)is_write; way[victim].lru = lv->clock;
    return 0;
}
static inline void sim_touch_line(CacheSim *s, uint64_t line_addr, int is_write) {
    int ev1 = 0, ev2 = 0, ev3 = 0;
    uint64_t ev1_a = 0, ev2_a = 0, ev3_a = 0;
    if (!level_access(&s->l1, line_addr, is_write, &ev1, &ev1_a)) {
        if (!level_access(&s->l2, line_addr, 0, &ev2, &ev2_a)) {
            if (!level_access(&s->llc, line_addr, 0, &ev3, &ev3_a)) {
                s->dram_reads++; if (ev3) s->dram_writes++;
            }
            if (ev2) {
                int ev3b = 0; uint64_t ev3b_a = 0;
                if (!level_access(&s->llc, ev2_a, 1, &ev3b, &ev3b_a)) {
                    s->dram_reads++; if (ev3b) s->dram_writes++;
                }
            }
        }
        if (ev1) {
            int ev2b = 0; uint64_t ev2b_a = 0;
            if (!level_access(&s->l2, ev1_a, 1, &ev2b, &ev2b_a)) {
                int ev3c = 0; uint64_t ev3c_a = 0;
                if (!level_access(&s->llc, ev1_a, 0, &ev3c, &ev3c_a)) {
                    s->dram_reads++; if (ev3c) s->dram_writes++;
                }
                if (ev2b) {
                    int ev3d = 0; uint64_t ev3d_a = 0;
                    if (!level_access(&s->llc, ev2b_a, 1, &ev3d, &ev3d_a)) {
                        s->dram_reads++; if (ev3d) s->dram_writes++;
                    }
                }
            }
        }
    }
}
static inline void sim_access(CacheSim *s, uint64_t addr, size_t len, int is_write) {
    if (len == 0) return;
    uint64_t l0 = addr >> 6, l1 = (addr + len - 1) >> 6;
    for (uint64_t l = l0; l <= l1; l++) sim_touch_line(s, l, is_write);
}
static void sim_drain(CacheSim *s) {
    for (int i = 0; i < s->l1.sets * s->l1.ways; i++) {
        if (s->l1.lines[i].valid && s->l1.lines[i].dirty) {
            uint64_t line = (s->l1.lines[i].tag << __builtin_ctz(s->l1.sets)) | (uint64_t)(i / s->l1.ways);
            int ev2 = 0; uint64_t ev2_a = 0;
            if (!level_access(&s->l2, line, 1, &ev2, &ev2_a)) {
                int ev3 = 0; uint64_t ev3_a = 0;
                if (!level_access(&s->llc, line, 0, &ev3, &ev3_a)) { s->dram_reads++; if (ev3) s->dram_writes++; }
                if (ev2) {
                    int ev3b = 0; uint64_t ev3b_a = 0;
                    if (!level_access(&s->llc, ev2_a, 1, &ev3b, &ev3b_a)) { s->dram_reads++; if (ev3b) s->dram_writes++; }
                }
            }
            s->l1.lines[i].dirty = 0;
        }
    }
    for (int i = 0; i < s->l2.sets * s->l2.ways; i++) {
        if (s->l2.lines[i].valid && s->l2.lines[i].dirty) {
            uint64_t line = (s->l2.lines[i].tag << __builtin_ctz(s->l2.sets)) | (uint64_t)(i / s->l2.ways);
            int ev3 = 0; uint64_t ev3_a = 0;
            if (!level_access(&s->llc, line, 1, &ev3, &ev3_a)) { s->dram_reads++; if (ev3) s->dram_writes++; }
            s->l2.lines[i].dirty = 0;
        }
    }
    for (int i = 0; i < s->llc.sets * s->llc.ways; i++) {
        if (s->llc.lines[i].valid && s->llc.lines[i].dirty) {
            s->dram_writes++; s->llc.lines[i].dirty = 0;
        }
    }
}
#else
static inline void sim_access(CacheSim *s, uint64_t addr, size_t len, int is_write) {
    (void)s; (void)addr; (void)len; (void)is_write;
}
static void sim_drain(CacheSim *s) { (void)s; }
#endif

typedef struct {
    uint64_t l1_misses, l2_misses, llc_misses;
    uint64_t dram_bytes, work_events;
    size_t peak_mem;
    uint64_t l2_p1, l2_p2, l2_p3, l2_p4;
    double ms_p1, ms_p2, ms_p3, ms_p4;
    uint32_t extra1, extra2;
} Metrics;

static inline double now_ms(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return ts.tv_sec * 1000.0 + ts.tv_nsec * 1e-6;
}

/* ==================== Capped Run Records & Dense Reference ==================== */

static uint16_t *column_to_records(const uint8_t *L, size_t n, size_t *out_nrecs) {
    size_t cap = 65536, nr = 0;
    uint16_t *recs = (uint16_t *)malloc(cap * sizeof(uint16_t));
    size_t i = 0;
    while (i < n) {
        uint8_t c = L[i];
        size_t j = i + 1;
        while (j < n && L[j] == c) j++;
        size_t left = j - i;
        while (left > 0) {
            size_t take = left > 127 ? 127 : left;
            if (nr == cap) { cap *= 2; recs = (uint16_t *)realloc(recs, cap * sizeof(uint16_t)); }
            recs[nr++] = (uint16_t)(((uint16_t)c << 8) | (uint16_t)take);
            left -= take;
        }
        i = j;
    }
    *out_nrecs = nr;
    return recs;
}

static uint8_t *dense_reference(const uint8_t *L, uint32_t n, uint32_t p) {
    uint32_t cnt[256] = {0}, C[256];
    for (uint32_t i = 0; i < n; i++) cnt[L[i]]++;
    uint32_t t = 0;
    for (int c = 0; c < 256; c++) { C[c] = t; t += cnt[c]; }
    uint32_t *lf = (uint32_t *)malloc(n * sizeof(uint32_t));
    for (uint32_t i = 0; i < n; i++) lf[i] = C[L[i]]++;
    uint8_t *out = (uint8_t *)malloc(n);
    uint32_t x = p;
    for (uint32_t i = 0; i < n; i++) { out[n - 1 - i] = L[x]; x = lf[x]; }
    free(lf);
    return out;
}

/* ==================== 1. Baseline: Shipped Forward Open ==================== */

typedef struct { uint32_t from, to, width; uint8_t byte; } OpenStrip;
typedef struct { uint32_t end, delta, code; } OpenBand;
typedef struct { uint32_t prev[2], next[2], width, word; } EpNode;
typedef struct { uint32_t left, right, len, first; } EpRule;

static inline void ep_cut(CacheSim *sim, uint64_t nodes_addr, EpNode *nodes, uint32_t id, int side) {
    uint32_t pr = nodes[id].prev[side], nx = nodes[id].next[side];
    nodes[pr].next[side] = nx; nodes[nx].prev[side] = pr;
    sim_access(sim, nodes_addr + (uint64_t)pr * sizeof(EpNode), sizeof(EpNode), 1);
    sim_access(sim, nodes_addr + (uint64_t)nx * sizeof(EpNode), sizeof(EpNode), 1);
}
static inline void ep_insert(CacheSim *sim, uint64_t nodes_addr, EpNode *nodes, uint32_t at, uint32_t id, int side) {
    sim_access(sim, nodes_addr + (uint64_t)at * sizeof(EpNode), sizeof(EpNode), 0);
    uint32_t nx = nodes[at].next[side];
    nodes[id].prev[side] = at; nodes[id].next[side] = nx;
    nodes[at].next[side] = id; nodes[nx].prev[side] = id;
    sim_access(sim, nodes_addr + (uint64_t)id * sizeof(EpNode), sizeof(EpNode), 1);
    sim_access(sim, nodes_addr + (uint64_t)at * sizeof(EpNode), sizeof(EpNode), 1);
    sim_access(sim, nodes_addr + (uint64_t)nx * sizeof(EpNode), sizeof(EpNode), 1);
}

static uint8_t *run_forward_open(const uint16_t *recs, size_t nrecs, uint32_t n, uint32_t p, Metrics *met) {
    const uint32_t LIMIT = 16;
    CacheSim sim; sim_init(&sim);
    memset(met, 0, sizeof(*met));
    double t0 = now_ms();

    uint64_t rec_addr = sim_alloc_region(&sim, nrecs * 2);
    uint32_t all[256] = {0}, thin[256] = {0}, thick[256] = {0};
    OpenStrip *strips = (OpenStrip *)calloc(nrecs + 2, sizeof(OpenStrip));
    uint64_t strips_addr = sim_alloc_region(&sim, (nrecs + 2) * sizeof(OpenStrip));
    size_t nstrips = 0;

    uint32_t *qdata[256]; uint32_t qlen[256] = {0}, qcap[256] = {0}; uint64_t qaddr[256];
    for (int a = 0; a < 256; a++) {
        qcap[a] = 4096; qdata[a] = (uint32_t *)malloc(qcap[a] * 4);
        qaddr[a] = sim_alloc_region(&sim, 250000 * 4);
    }

    uint32_t m = 0, mark = 0, pos = 0;
    size_t ri = 0;
    while (ri < nrecs) {
        sim_access(&sim, rec_addr + ri * 2, 2, 0);
        uint16_t rec = recs[ri++];
        uint8_t c = (uint8_t)(rec >> 8);
        uint32_t w = rec & 255;
        while (ri < nrecs) {
            sim_access(&sim, rec_addr + ri * 2, 2, 0);
            if ((recs[ri] >> 8) != c) break;
            w += recs[ri] & 255; ri++;
        }
        met->work_events++;
        uint32_t end = pos + w;
        uint32_t sx[3], sw[3]; int sk[3], ns = 0;
        if (w > LIMIT && pos <= p && p < end) {
            if (pos < p) { sx[ns] = pos; sw[ns] = p - pos; sk[ns++] = 0; }
            sx[ns] = p; sw[ns] = 1; sk[ns++] = 1;
            if (p + 1 < end) { sx[ns] = p + 1; sw[ns] = end - p - 1; sk[ns++] = 0; }
        } else { sx[ns] = pos; sw[ns] = w; sk[ns++] = (w <= LIMIT); }
        for (int s = 0; s < ns; s++) {
            if (sk[s]) {
                if (sx[s] <= p && p < sx[s] + sw[s]) mark = m + (p - sx[s]);
                if (qlen[c] + sw[s] > qcap[c]) {
                    while (qlen[c] + sw[s] > qcap[c]) qcap[c] *= 2;
                    qdata[c] = (uint32_t *)realloc(qdata[c], qcap[c] * 4);
                }
                for (uint32_t u = 0; u < sw[s]; u++) {
                    qdata[c][qlen[c]] = m + u;
                    sim_access(&sim, qaddr[c] + (uint64_t)qlen[c] * 4, 4, 1);
                    qlen[c]++; met->work_events++;
                }
                thin[c] += sw[s]; m += sw[s];
            } else {
                strips[nstrips] = (OpenStrip){sx[s], all[c], sw[s], c};
                sim_access(&sim, strips_addr + nstrips * sizeof(OpenStrip), sizeof(OpenStrip), 1);
                nstrips++; thick[c]++;
            }
            all[c] += sw[s];
        }
        pos = end;
    }
    met->l2_p1 = sim.l2.misses;
    double t1 = now_ms(); met->ms_p1 = t1 - t0;

    /* Endpoint & spelling */
    uint32_t first[256], write_pos[256], tot = 0, ttot = 0;
    for (int a = 0; a < 256; a++) { first[a] = tot; tot += all[a]; write_pos[a] = ttot; ttot += thick[a]; }
    uint32_t *image_order = (uint32_t *)malloc((nstrips + 1) * 4);
    uint64_t imgord_addr = sim_alloc_region(&sim, (nstrips + 1) * 4);
    for (size_t i = 0; i < nstrips; i++) {
        sim_access(&sim, strips_addr + i * sizeof(OpenStrip), sizeof(OpenStrip), 0);
        uint8_t a = strips[i].byte;
        strips[i].to += first[a];
        sim_access(&sim, strips_addr + i * sizeof(OpenStrip), sizeof(OpenStrip), 1);
        uint32_t wp = write_pos[a]++;
        image_order[wp] = (uint32_t)i;
        sim_access(&sim, imgord_addr + (uint64_t)wp * 4, 4, 1);
    }
    size_t max_nodes = 3 * nstrips + 5;
    EpNode *nodes = (EpNode *)calloc(max_nodes, sizeof(EpNode));
    uint64_t nodes_addr = sim_alloc_region(&sim, max_nodes * sizeof(EpNode));
    uint32_t nnodes = 1;
    uint32_t *ord0 = (uint32_t *)malloc(max_nodes * 4), *ord1 = (uint32_t *)malloc(max_nodes * 4);
    uint32_t *term0 = (uint32_t *)malloc(max_nodes * 4), *term1 = (uint32_t *)malloc(max_nodes * 4);
    size_t n_o0 = 0, n_o1 = 0, n_t0 = 0, n_t1 = 0;
    for (size_t i = 0; i < nstrips; i++) {
        sim_access(&sim, strips_addr + i * sizeof(OpenStrip), sizeof(OpenStrip), 0);
        nodes[nnodes].width = strips[i].width; nodes[nnodes].word = (uint32_t)strips[i].byte + 1;
        sim_access(&sim, nodes_addr + (uint64_t)nnodes * sizeof(EpNode), sizeof(EpNode), 1);
        nnodes++;
    }
    uint32_t x = 0;
    for (size_t i = 0; i < nstrips; i++) {
        sim_access(&sim, strips_addr + i * sizeof(OpenStrip), sizeof(OpenStrip), 0);
        if (x < strips[i].from) {
            uint32_t id = nnodes++;
            nodes[id].width = strips[i].from - x; nodes[id].word = 0;
            sim_access(&sim, nodes_addr + (uint64_t)id * sizeof(EpNode), sizeof(EpNode), 1);
            ord0[n_o0++] = id; term1[n_t1++] = id;
        }
        ord0[n_o0++] = (uint32_t)i + 1; x = strips[i].from + strips[i].width;
    }
    if (x < n) { uint32_t id = nnodes++; nodes[id].width = n - x; nodes[id].word = 0; sim_access(&sim, nodes_addr + (uint64_t)id * sizeof(EpNode), sizeof(EpNode), 1); ord0[n_o0++] = id; term1[n_t1++] = id; }
    x = 0;
    for (size_t k = 0; k < nstrips; k++) {
        sim_access(&sim, imgord_addr + k * 4, 4, 0);
        uint32_t i = image_order[k];
        sim_access(&sim, strips_addr + (uint64_t)i * sizeof(OpenStrip), sizeof(OpenStrip), 0);
        if (x < strips[i].to) {
            uint32_t id = nnodes++;
            nodes[id].width = strips[i].to - x; nodes[id].word = 0;
            sim_access(&sim, nodes_addr + (uint64_t)id * sizeof(EpNode), sizeof(EpNode), 1);
            ord1[n_o1++] = id; term0[n_t0++] = id;
        }
        ord1[n_o1++] = i + 1; x = strips[i].to + strips[i].width;
    }
    if (x < n) { uint32_t id = nnodes++; nodes[id].width = n - x; nodes[id].word = 0; sim_access(&sim, nodes_addr + (uint64_t)id * sizeof(EpNode), sizeof(EpNode), 1); ord1[n_o1++] = id; term0[n_t0++] = id; }
    for (int side = 0; side < 2; side++) {
        uint32_t prev = 0;
        uint32_t *tarr = side == 0 ? term0 : term1; size_t nt = side == 0 ? n_t0 : n_t1;
        uint32_t *oarr = side == 0 ? ord0 : ord1;   size_t no = side == 0 ? n_o0 : n_o1;
        for (size_t k = 0; k < nt; k++) { uint32_t id = tarr[k]; nodes[prev].next[side] = id; nodes[id].prev[side] = prev; sim_access(&sim, nodes_addr + (uint64_t)id * sizeof(EpNode), sizeof(EpNode), 1); prev = id; }
        for (size_t k = 0; k < no; k++) { uint32_t id = oarr[k]; nodes[prev].next[side] = id; nodes[id].prev[side] = prev; sim_access(&sim, nodes_addr + (uint64_t)id * sizeof(EpNode), sizeof(EpNode), 1); prev = id; }
        nodes[prev].next[side] = 0; nodes[0].prev[side] = prev;
    }
    size_t cap_rules = 65536;
    EpRule *rules = (EpRule *)malloc(cap_rules * sizeof(EpRule));
    uint64_t rules_addr = sim_alloc_region(&sim, 262144 * sizeof(EpRule));
    uint32_t nrules = 0, rem = n;
    while (rem > 0) {
        uint32_t ix = nodes[0].prev[0], iy = nodes[0].prev[1];
        sim_access(&sim, nodes_addr + (uint64_t)ix * sizeof(EpNode), sizeof(EpNode), 0);
        sim_access(&sim, nodes_addr + (uint64_t)iy * sizeof(EpNode), sizeof(EpNode), 0);
        EpNode a = nodes[ix], b = nodes[iy];
        uint32_t w = a.width < b.width ? a.width : b.width;
        rem -= w; met->work_events++;
        if (ix == iy) { ep_cut(&sim, nodes_addr, nodes, ix, 0); ep_cut(&sim, nodes_addr, nodes, ix, 1); continue; }
        uint32_t word;
        if (a.word == 0) word = b.word;
        else if (b.word == 0) word = a.word;
        else {
            if (nrules == cap_rules) { cap_rules *= 2; rules = (EpRule *)realloc(rules, cap_rules * sizeof(EpRule)); }
            uint32_t la = a.word <= 256 ? 1 : rules[a.word - 257].len;
            uint32_t lb = b.word <= 256 ? 1 : rules[b.word - 257].len;
            rules[nrules] = (EpRule){a.word, b.word, la + lb, UINT32_MAX};
            sim_access(&sim, rules_addr + (uint64_t)nrules * sizeof(EpRule), sizeof(EpRule), 1);
            word = nrules + 257; nrules++;
        }
        if (a.width <= b.width) {
            ep_cut(&sim, nodes_addr, nodes, ix, 0); ep_insert(&sim, nodes_addr, nodes, iy, ix, 0);
            nodes[ix].word = word;
            if (a.width == b.width) { ep_cut(&sim, nodes_addr, nodes, iy, 0); ep_cut(&sim, nodes_addr, nodes, iy, 1); }
            else nodes[iy].width -= w;
        } else {
            ep_cut(&sim, nodes_addr, nodes, iy, 1); ep_insert(&sim, nodes_addr, nodes, ix, iy, 1);
            nodes[iy].word = word; nodes[ix].width -= w;
        }
    }
    uint32_t *bottom = (uint32_t *)malloc(nnodes * 4);
    uint64_t bot_addr = sim_alloc_region(&sim, nnodes * 4);
    uint32_t id = nodes[0].next[1]; x = 0;
    while (id != 0) {
        bottom[id] = x; sim_access(&sim, bot_addr + (uint64_t)id * 4, 4, 1);
        sim_access(&sim, nodes_addr + (uint64_t)id * sizeof(EpNode), sizeof(EpNode), 0);
        x += nodes[id].width; id = nodes[id].next[1];
    }
    size_t cap_bands = nnodes + 256, nbands = 0, cap_spell = 1 << 18, nspell = 0, noff = 0;
    OpenBand *bands = (OpenBand *)malloc(cap_bands * sizeof(OpenBand));
    uint64_t bands_addr = sim_alloc_region(&sim, (cap_bands + 1024) * sizeof(OpenBand));
    uint8_t *spelling = (uint8_t *)malloc(cap_spell);
    uint64_t spell_addr = sim_alloc_region(&sim, 1 << 21);
    uint32_t *offsets = (uint32_t *)malloc((cap_bands + 2) * 4);
    uint64_t off_addr = sim_alloc_region(&sim, (cap_bands + 1024) * 4);
    offsets[noff++] = 0;
    uint32_t *stack = (uint32_t *)malloc(65536 * 4);
    id = nodes[0].next[0]; uint32_t in_node = 0, k = 0;
    for (int a = 0; a < 256; a++) {
        uint32_t end = k + thin[a];
        while (k < end) {
            sim_access(&sim, nodes_addr + (uint64_t)id * sizeof(EpNode), sizeof(EpNode), 0);
            EpNode nd = nodes[id];
            uint32_t w = (end - k) < (nd.width - in_node) ? (end - k) : (nd.width - in_node);
            sim_access(&sim, bot_addr + (uint64_t)id * 4, 4, 0);
            uint32_t dst = bottom[id] + in_node, code;
            if (nd.word == 0) code = (uint32_t)a;
            else {
                size_t sp = 0; stack[sp++] = nd.word;
                while (sp > 0) {
                    uint32_t wid = stack[--sp];
                    if (wid <= 256) {
                        if (nspell + 2 >= cap_spell) { cap_spell *= 2; spelling = (uint8_t *)realloc(spelling, cap_spell); }
                        spelling[nspell] = (uint8_t)(wid - 1); sim_access(&sim, spell_addr + nspell, 1, 1); nspell++;
                    } else {
                        EpRule *rl = &rules[wid - 257];
                        sim_access(&sim, rules_addr + (uint64_t)(wid - 257) * sizeof(EpRule), sizeof(EpRule), 0);
                        if (rl->first == UINT32_MAX) {
                            rl->first = (uint32_t)nspell;
                            sim_access(&sim, rules_addr + (uint64_t)(wid - 257) * sizeof(EpRule), sizeof(EpRule), 1);
                            stack[sp++] = rl->right; stack[sp++] = rl->left;
                        } else {
                            uint32_t st = rl->first, ln = rl->len;
                            if (nspell + ln + 2 >= cap_spell) { while (nspell + ln + 2 >= cap_spell) cap_spell *= 2; spelling = (uint8_t *)realloc(spelling, cap_spell); }
                            sim_access(&sim, spell_addr + st, ln, 0);
                            memcpy(spelling + nspell, spelling + st, ln);
                            sim_access(&sim, spell_addr + nspell, ln, 1);
                            nspell += ln;
                        }
                    }
                }
                if (nspell + 2 >= cap_spell) { cap_spell *= 2; spelling = (uint8_t *)realloc(spelling, cap_spell); }
                spelling[nspell] = (uint8_t)a; sim_access(&sim, spell_addr + nspell, 1, 1); nspell++;
                offsets[noff] = (uint32_t)nspell; sim_access(&sim, off_addr + noff * 4, 4, 1); noff++;
                code = 254 + (uint32_t)noff;
            }
            if (nbands == cap_bands) { cap_bands *= 2; bands = (OpenBand *)realloc(bands, cap_bands * sizeof(OpenBand)); offsets = (uint32_t *)realloc(offsets, (cap_bands + 2) * 4); }
            bands[nbands] = (OpenBand){k + w, dst - k, code};
            sim_access(&sim, bands_addr + nbands * sizeof(OpenBand), sizeof(OpenBand), 1);
            nbands++; met->work_events++; in_node += w; k += w;
            if (in_node == nd.width) { id = nd.next[0]; in_node = 0; }
        }
    }
    size_t ep_mem = nstrips * sizeof(OpenStrip) + nnodes * sizeof(EpNode) + nrules * sizeof(EpRule) + nbands * sizeof(OpenBand) + nspell + noff * 4;
    free(image_order); free(nodes); free(ord0); free(ord1); free(term0); free(term1); free(rules); free(bottom); free(stack);
    met->l2_p2 = sim.l2.misses - met->l2_p1;
    double t2 = now_ms(); met->ms_p2 = t2 - t1;

    uint32_t bits = 0; while ((1u << bits) < m) bits++;
    uint32_t mask = (1u << bits) - 1;
    uint32_t *edges = (uint32_t *)malloc((size_t)m * 4);
    uint64_t edges_addr = sim_alloc_region(&sim, (size_t)m * 4);
    k = 0; size_t bi = 0;
    for (int a = 0; a < 256; a++) {
        uint32_t ql = qlen[a], *qd = qdata[a], qi = 0;
        while (qi < ql) {
            sim_access(&sim, bands_addr + bi * sizeof(OpenBand), sizeof(OpenBand), 0);
            OpenBand b = bands[bi];
            uint32_t w = b.end - k, dst = k + b.delta, pack = b.code << bits;
            sim_access(&sim, qaddr[a] + (uint64_t)qi * 4, (size_t)w * 4, 0);
            for (uint32_t u = 0; u < w; u++) edges[dst + u] = pack | qd[qi + u];
            sim_access(&sim, edges_addr + (uint64_t)dst * 4, (size_t)w * 4, 1);
            met->work_events += w; k += w; qi += w; bi++;
        }
    }
    met->l2_p3 = sim.l2.misses - met->l2_p1 - met->l2_p2;
    double t3 = now_ms(); met->ms_p3 = t3 - t2;

    uint8_t *out = (uint8_t *)malloc(n);
    uint64_t out_addr = sim_alloc_region(&sim, n);
    uint32_t out_pos = 0, j = mark;
    for (uint32_t step = 0; step < m && out_pos < n; step++) {
        sim_access(&sim, edges_addr + (uint64_t)j * 4, 4, 0);
        uint32_t edge = edges[j], code = edge >> bits;
        j = edge & mask;
        if (code < 256) { out[out_pos] = (uint8_t)code; sim_access(&sim, out_addr + out_pos, 1, 1); out_pos++; }
        else {
            uint32_t wi = code - 256;
            sim_access(&sim, off_addr + (uint64_t)wi * 4, 8, 0);
            uint32_t st = offsets[wi], ln = offsets[wi + 1] - st;
            if (out_pos + ln > n) ln = n - out_pos;
            sim_access(&sim, spell_addr + st, ln, 0);
            memcpy(out + out_pos, spelling + st, ln);
            sim_access(&sim, out_addr + out_pos, ln, 1);
            out_pos += ln;
        }
        met->work_events++;
    }
    met->l2_p4 = sim.l2.misses - met->l2_p1 - met->l2_p2 - met->l2_p3;
    double t4 = now_ms(); met->ms_p4 = t4 - t3;

    sim_drain(&sim);
    met->l1_misses = sim.l1.misses; met->l2_misses = sim.l2.misses; met->llc_misses = sim.llc.misses;
    met->dram_bytes = (sim.dram_reads + sim.dram_writes) * 64ULL;
    met->peak_mem = nrecs * 2 + (size_t)m * 4 + (size_t)m * 4 + ep_mem + n;
    met->extra1 = m; met->extra2 = (uint32_t)nbands;

    for (int a = 0; a < 256; a++) free(qdata[a]);
    free(strips); free(bands); free(spelling); free(offsets); free(edges);
    sim_free(&sim);
    return out;
}

/* ==================== 2. Flat-Round-1 UFC (Shared Front-End, 8-Way MLP) ==================== */

typedef struct { uint32_t from, to, w, wid; } Piece;
typedef struct { uint32_t kid, sub_off, ow, v_fi, in_v; uint8_t sym_c; } RawArc;

/* Compact 4-byte forest entry:
 * bit 31: 1 = SPLITTER (bits 0..22 = k_id, bits 23..30 = sym_c)
 * bit 31: 0 = UNSPLIT  (bits 0..17 = v_fi, bits 18..22 = delta_small (31 => big_delta table), bits 23..30 = sym_c)
 * Note: 2^18 = 262,144 > max r (128,732) on all admitted blocks!
 */
#define F_SPLIT_BIT (1u << 31)
#define F_DSMALL_MAX 31u

typedef struct {
    Piece *pieces;
    uint64_t pieces_addr;
    uint32_t npieces;
    uint32_t new_n, new_p, mark_shift;
    uint8_t *lit_spell;
    uint64_t lit_spell_addr;
    uint32_t lit_spell_len;
    uint32_t *lit_off; /* lit_off[i]..lit_off[i+1] for lit word i */
    uint64_t lit_off_addr;
    uint32_t n_lits;
    size_t r1_peak_mem;
    uint32_t r_orig, nK;
} R1Result;

static R1Result run_ufc_round1(CacheSim *sim, Metrics *met, const uint16_t *recs, size_t nrecs, uint32_t n, uint32_t p) {
    R1Result res;
    memset(&res, 0, sizeof(res));
    uint64_t rec_addr = sim_alloc_region(sim, nrecs * 2);

    /* Pass 1: Coalesce capped records into r maximal runs (sequential) */
    uint32_t *l_from = (uint32_t *)malloc((nrecs + 2) * 4);
    uint8_t  *l_sym  = (uint8_t  *)malloc(nrecs + 2);
    uint64_t l_from_addr = sim_alloc_region(sim, (nrecs + 2) * 4);
    uint64_t l_sym_addr  = sim_alloc_region(sim, nrecs + 2);
    uint32_t sym_rcnt[256] = {0};
    uint32_t r = 0, pos = 0, p_lrun = 0;
    size_t ri = 0;
    while (ri < nrecs) {
        sim_access(sim, rec_addr + ri * 2, 2, 0);
        uint16_t rec = recs[ri++];
        uint8_t c = (uint8_t)(rec >> 8);
        uint32_t w = rec & 255;
        while (ri < nrecs) {
            sim_access(sim, rec_addr + ri * 2, 2, 0);
            if ((recs[ri] >> 8) != c) break;
            w += recs[ri] & 255; ri++;
        }
        if (pos <= p && p < pos + w) p_lrun = r;
        l_from[r] = pos; l_sym[r] = c;
        sim_access(sim, l_from_addr + (uint64_t)r * 4, 4, 1);
        sim_access(sim, l_sym_addr + r, 1, 1);
        sym_rcnt[c]++;
        r++; pos += w; met->work_events++;
    }
    l_from[r] = n;
    sim_access(sim, l_from_addr + (uint64_t)r * 4, 4, 1);
    assert(pos == n && r < (1u << 18));
    res.r_orig = r;

    /* Pass 2: F-order run index f_of_l[k] (sequential) and f_w[fi] (256 buckets, u16 w/ escape!) */
    uint32_t sym_rend[256], cur_off[256], t = 0;
    for (int c = 0; c < 256; c++) { cur_off[c] = t; t += sym_rcnt[c]; sym_rend[c] = t; }

    uint32_t *f_of_l = (uint32_t *)malloc((size_t)r * 4);
    uint32_t *f_w    = (uint32_t *)malloc((size_t)r * 4);
    uint64_t f_of_l_addr = sim_alloc_region(sim, (size_t)r * 4);
    uint64_t f_w_addr    = sim_alloc_region(sim, (size_t)r * 4);
    for (uint32_t k = 0; k < r; k++) {
        sim_access(sim, l_sym_addr + k, 1, 0);
        sim_access(sim, l_from_addr + (uint64_t)k * 4, 8, 0);
        uint8_t c = l_sym[k];
        uint32_t w = l_from[k + 1] - l_from[k];
        uint32_t fi = cur_off[c]++;
        f_of_l[k] = fi;
        f_w[fi] = w;
        sim_access(sim, f_of_l_addr + (uint64_t)k * 4, 4, 1);
        sim_access(sim, f_w_addr + (uint64_t)fi * 4, 4, 1);
        met->work_events++;
    }
    free(l_sym);
    uint32_t p_fi = f_of_l[p_lrun];
    uint32_t p_in_run = p - l_from[p_lrun];

    /* Pass 3: Linear 2-pointer merge (fi vs v) -> compact 4B/run forest[0..r] + raw_arcs (ZERO random accesses!) */
    uint32_t *forest = (uint32_t *)malloc((size_t)r * 4);
    uint64_t forest_addr = sim_alloc_region(sim, (size_t)r * 4);
    /* Big-delta escape table for d >= 31: sparse array or indexed by fi (only touched on <5% of hops!) */
    uint32_t *big_delta = (uint32_t *)malloc((size_t)r * 4);
    uint64_t big_d_addr = sim_alloc_region(sim, (size_t)r * 4);

    uint32_t cap_K = (r / 4) + 256, nK = 0;
    uint32_t *k_new_from = (uint32_t *)malloc((size_t)cap_K * 4);
    uint64_t knf_addr = sim_alloc_region(sim, (size_t)cap_K * 4);

    uint32_t cap_arcs = (r / 2) + 512, narcs = 0;
    RawArc *raw_arcs = (RawArc *)malloc((size_t)cap_arcs * sizeof(RawArc));
    uint64_t arcs_addr = sim_alloc_region(sim, (size_t)cap_arcs * sizeof(RawArc));

    uint32_t v = 0, t0 = 0, new_n = 0;
    int sym_c = 0;
    for (uint32_t fi = 0; fi < r; fi++) {
        while (fi >= sym_rend[sym_c]) sym_c++;
        sim_access(sim, f_w_addr + (uint64_t)fi * 4, 4, 0);
        uint32_t w = f_w[fi], t1 = t0 + w;
        while (l_from[v + 1] <= t0) { v++; sim_access(sim, l_from_addr + (uint64_t)(v + 1) * 4, 4, 0); }
        sim_access(sim, l_from_addr + (uint64_t)v * 4, 8, 0);
        if (l_from[v + 1] >= t1) {
            sim_access(sim, f_of_l_addr + (uint64_t)v * 4, 4, 0);
            uint32_t v_fi = f_of_l[v];
            uint32_t d = t0 - l_from[v];
            uint32_t ds = (d < F_DSMALL_MAX) ? d : F_DSMALL_MAX;
            if (ds == F_DSMALL_MAX) {
                big_delta[fi] = d;
                sim_access(sim, big_d_addr + (uint64_t)fi * 4, 4, 1);
            }
            forest[fi] = v_fi | (ds << 18) | ((uint32_t)sym_c << 23);
            sim_access(sim, forest_addr + (uint64_t)fi * 4, 4, 1);
        } else {
            if (nK == cap_K) { cap_K *= 2; k_new_from = (uint32_t *)realloc(k_new_from, (size_t)cap_K * 4); }
            uint32_t kid = nK++;
            k_new_from[kid] = new_n;
            sim_access(sim, knf_addr + (uint64_t)kid * 4, 4, 1);
            new_n += w;
            forest[fi] = F_SPLIT_BIT | kid | ((uint32_t)sym_c << 23);
            sim_access(sim, forest_addr + (uint64_t)fi * 4, 4, 1);
            uint32_t vv = v;
            while (vv < r) {
                sim_access(sim, l_from_addr + (uint64_t)vv * 4, 8, 0);
                if (l_from[vv] >= t1) break;
                uint32_t ov0 = t0 > l_from[vv] ? t0 : l_from[vv];
                uint32_t ov1 = t1 < l_from[vv + 1] ? t1 : l_from[vv + 1];
                sim_access(sim, f_of_l_addr + (uint64_t)vv * 4, 4, 0);
                if (narcs == cap_arcs) { cap_arcs *= 2; raw_arcs = (RawArc *)realloc(raw_arcs, (size_t)cap_arcs * sizeof(RawArc)); }
                raw_arcs[narcs] = (RawArc){kid, ov0 - t0, ov1 - ov0, f_of_l[vv], ov0 - l_from[vv], (uint8_t)sym_c};
                sim_access(sim, arcs_addr + (uint64_t)narcs * sizeof(RawArc), sizeof(RawArc), 1);
                narcs++; vv++;
            }
            v = vv - 1;
        }
        t0 = t1; met->work_events++;
    }
    free(l_from); free(f_of_l); free(f_w);
    res.nK = nK;

    /* Advance p_fi through forest to its root in K */
    uint32_t mark_shift = 0, new_p = 0;
    {
        uint32_t cur = p_fi, acc_d = 0;
        for (;;) {
            sim_access(sim, forest_addr + (uint64_t)cur * 4, 4, 0);
            uint32_t fe = forest[cur];
            if (fe & F_SPLIT_BIT) {
                uint32_t kid = fe & 0x7FFFFFu;
                new_p = k_new_from[kid] + acc_d + p_in_run;
                break;
            }
            uint32_t ds = (fe >> 18) & 31u;
            uint32_t d = (ds < F_DSMALL_MAX) ? ds : big_delta[cur];
            acc_d += d; mark_shift++;
            cur = fe & 0x3FFFFu;
        }
    }

    /* Pass 4: 8-Way Parallel Lockstep Forest Trace -> flat literal spelling + sorted pieces! */
    Piece *pieces = (Piece *)malloc((size_t)narcs * sizeof(Piece));
    uint64_t pieces_addr = sim_alloc_region(sim, (size_t)narcs * sizeof(Piece));
    size_t cap_lit = 262144, lit_len = 0;
    uint8_t *lit_spell = (uint8_t *)malloc(cap_lit);
    uint64_t lit_spell_addr = sim_alloc_region(sim, 524288);
    uint32_t *lit_off = (uint32_t *)malloc((size_t)(narcs + 2) * 4);
    uint64_t lit_off_addr = sim_alloc_region(sim, (size_t)(narcs + 2) * 4);
    lit_off[0] = 0;

    /* 8-lane lockstep state (in L1/registers!) */
    uint32_t l_arc[8], l_cur[8], l_accd[8], l_blen[8];
    uint8_t  l_buf[8][128];
    int      l_active[8] = {0};
    uint32_t next_arc = 0, active_cnt = 0;

    /* Prime 8 lanes */
    for (int lane = 0; lane < 8 && next_arc < narcs; lane++) {
        uint32_t ai = next_arc++;
        sim_access(sim, arcs_addr + (uint64_t)ai * sizeof(RawArc), sizeof(RawArc), 0);
        l_arc[lane] = ai; l_cur[lane] = raw_arcs[ai].v_fi; l_accd[lane] = 0;
        l_buf[lane][0] = raw_arcs[ai].sym_c; l_blen[lane] = 1;
        l_active[lane] = 1; active_cnt++;
    }

    while (active_cnt > 0) {
        /* Lockstep 8-way independent loads from forest[0..r]! */
        uint32_t fe[8];
        for (int lane = 0; lane < 8; lane++) {
            if (l_active[lane]) {
                sim_access(sim, forest_addr + (uint64_t)l_cur[lane] * 4, 4, 0);
                fe[lane] = forest[l_cur[lane]];
            } else fe[lane] = 0;
        }
        for (int lane = 0; lane < 8; lane++) {
            if (!l_active[lane]) continue;
            uint32_t e = fe[lane];
            if (e & F_SPLIT_BIT) {
                uint32_t ai = l_arc[lane];
                RawArc ra = raw_arcs[ai];
                uint32_t rt_kid = e & 0x7FFFFFu;
                sim_access(sim, knf_addr + (uint64_t)ra.kid * 4, 4, 0);
                sim_access(sim, knf_addr + (uint64_t)rt_kid * 4, 4, 0);
                uint32_t s_from = k_new_from[ra.kid] + ra.sub_off;
                uint32_t s_to   = k_new_from[rt_kid] + l_accd[lane] + ra.in_v;
                uint32_t bl = l_blen[lane];
                if (lit_len + bl + 4 >= cap_lit) { cap_lit *= 2; lit_spell = (uint8_t *)realloc(lit_spell, cap_lit); }
                uint32_t st = (uint32_t)lit_len;
                for (uint32_t q = 0; q < bl; q++) lit_spell[lit_len++] = l_buf[lane][bl - 1 - q];
                sim_access(sim, lit_spell_addr + st, bl, 1);
                /* Store piece with (st, bl) encoded via piece index? Keep separate lit_id = ai! Wait: out-of-order completion!
                 * Instead pack lit_off[lid] sequentially as lid = n_lits++! */
                uint32_t lid = res.n_lits++;
                lit_off[lid + 1] = (uint32_t)lit_len;
                sim_access(sim, lit_off_addr + (uint64_t)(lid + 1) * 4, 4, 1);
                pieces[ai] = (Piece){s_from, s_to, ra.ow, lid};
                sim_access(sim, pieces_addr + (uint64_t)ai * sizeof(Piece), sizeof(Piece), 1);

                /* Refill lane */
                if (next_arc < narcs) {
                    uint32_t nai = next_arc++;
                    sim_access(sim, arcs_addr + (uint64_t)nai * sizeof(RawArc), sizeof(RawArc), 0);
                    l_arc[lane] = nai; l_cur[lane] = raw_arcs[nai].v_fi; l_accd[lane] = 0;
                    l_buf[lane][0] = raw_arcs[nai].sym_c; l_blen[lane] = 1;
                } else {
                    l_active[lane] = 0; active_cnt--;
                }
            } else {
                uint32_t ds = (e >> 18) & 31u;
                uint32_t d;
                if (ds < F_DSMALL_MAX) d = ds;
                else { sim_access(sim, big_d_addr + (uint64_t)l_cur[lane] * 4, 4, 0); d = big_delta[l_cur[lane]]; }
                l_accd[lane] += d;
                assert(l_blen[lane] < 128);
                l_buf[lane][l_blen[lane]++] = (uint8_t)((e >> 23) & 255u);
                l_cur[lane] = e & 0x3FFFFu;
                met->work_events++;
            }
        }
    }

    /* Peak live memory during Round 1 */
    size_t m_p2 = nrecs * 2 + (size_t)r * 4 * 3 + (size_t)r; /* l_from, l_sym, f_of_l, f_w */
    size_t m_p4 = nrecs * 2 + (size_t)r * 4 * 2 + (size_t)narcs * (sizeof(RawArc) + sizeof(Piece) + 4) + lit_len + (size_t)nK * 4;
    res.r1_peak_mem = m_p2 > m_p4 ? m_p2 : m_p4;

    free(forest); free(big_delta); free(k_new_from); free(raw_arcs);
    res.pieces = pieces; res.pieces_addr = pieces_addr; res.npieces = narcs;
    res.new_n = new_n; res.new_p = new_p; res.mark_shift = mark_shift;
    res.lit_spell = lit_spell; res.lit_spell_addr = lit_spell_addr; res.lit_spell_len = (uint32_t)lit_len;
    res.lit_off = lit_off; res.lit_off_addr = lit_off_addr;
    return res;
}

/* ==================== Radix Sort Helper (2-Pass 10-bit on to <= 900,000) ==================== */

static void radix_sort_by_to(CacheSim *sim, const Piece *pcs, uint64_t pcs_addr, uint32_t np,
                             uint32_t *ord, uint64_t ord_addr, uint32_t *tmp, uint64_t tmp_addr) {
    uint32_t c0[1024] = {0}, c1[1024] = {0};
    for (uint32_t i = 0; i < np; i++) {
        sim_access(sim, pcs_addr + (uint64_t)i * sizeof(Piece), sizeof(Piece), 0);
        uint32_t t = pcs[i].to;
        c0[t & 1023u]++;
        c1[(t >> 10) & 1023u]++;
    }
    uint32_t s0 = 0, s1 = 0;
    for (int b = 0; b < 1024; b++) { uint32_t v0 = c0[b]; c0[b] = s0; s0 += v0; uint32_t v1 = c1[b]; c1[b] = s1; s1 += v1; }
    for (uint32_t i = 0; i < np; i++) {
        sim_access(sim, pcs_addr + (uint64_t)i * sizeof(Piece), sizeof(Piece), 0);
        uint32_t d = c0[pcs[i].to & 1023u]++;
        tmp[d] = i;
        sim_access(sim, tmp_addr + (uint64_t)d * 4, 4, 1);
    }
    for (uint32_t i = 0; i < np; i++) {
        sim_access(sim, tmp_addr + (uint64_t)i * 4, 4, 0);
        uint32_t id = tmp[i];
        sim_access(sim, pcs_addr + (uint64_t)id * sizeof(Piece), sizeof(Piece), 0);
        uint32_t d = c1[(pcs[id].to >> 10) & 1023u]++;
        ord[d] = id;
        sim_access(sim, ord_addr + (uint64_t)d * 4, 4, 1);
    }
}

/* ==================== 3. UFC-Full (Flat-R1 + Iterated L2 UFC + Direct-Rotated Emit) ==================== */

typedef struct { uint32_t left, right, len, first; } SlpRule;

static inline uint32_t ufc_wlen(uint32_t wid, uint32_t n_lits, const uint32_t *lit_off, const SlpRule *rules) {
    return (wid < n_lits) ? (lit_off[wid + 1] - lit_off[wid]) : rules[wid - n_lits].len;
}

static uint32_t ufc_join(uint32_t l, uint32_t r, uint32_t n_lits, const uint32_t *lit_off,
                         SlpRule **rules, uint32_t *nrules, uint32_t *cap_rules,
                         CacheSim *sim, uint64_t rules_addr) {
    if (l == UINT32_MAX) return r;
    if (r == UINT32_MAX) return l;
    if (*nrules == *cap_rules) { *cap_rules *= 2; *rules = (SlpRule *)realloc(*rules, (size_t)(*cap_rules) * sizeof(SlpRule)); }
    uint32_t ll = ufc_wlen(l, n_lits, lit_off, *rules);
    uint32_t rl = ufc_wlen(r, n_lits, lit_off, *rules);
    uint32_t idx = (*nrules)++;
    (*rules)[idx] = (SlpRule){l, r, ll + rl, UINT32_MAX};
    sim_access(sim, rules_addr + (uint64_t)idx * sizeof(SlpRule), sizeof(SlpRule), 1);
    return n_lits + idx;
}

static uint8_t *run_ufc_full(const uint16_t *recs, size_t nrecs, uint32_t n, uint32_t p, Metrics *met) {
    CacheSim sim; sim_init(&sim);
    memset(met, 0, sizeof(*met));
    double t0 = now_ms();

    R1Result r1 = run_ufc_round1(&sim, met, recs, nrecs, n, p);
    met->l2_p1 = sim.l2.misses;
    double t1 = now_ms(); met->ms_p1 = t1 - t0;

    Piece *pcs = r1.pieces;
    uint64_t pcs_addr = r1.pieces_addr;
    uint32_t r = r1.npieces;
    uint32_t cur_p = r1.new_p, mark_shift = r1.mark_shift;
    uint32_t n_lits = r1.n_lits;

    uint32_t cap_rules = 131072, nrules = 0;
    SlpRule *rules = (SlpRule *)malloc((size_t)cap_rules * sizeof(SlpRule));
    uint64_t rules_addr = sim_alloc_region(&sim, 262144 * sizeof(SlpRule));

    uint32_t max_r = r + 64;
    uint32_t *to_ord = (uint32_t *)malloc((size_t)max_r * 4);
    uint32_t *tmp_ord = (uint32_t *)malloc((size_t)max_r * 4);
    uint32_t *succ = (uint32_t *)malloc((size_t)max_r * 4);
    uint32_t *delta = (uint32_t *)malloc((size_t)max_r * 4);
    uint8_t  *state = (uint8_t  *)malloc((size_t)max_r);
    uint32_t *root = (uint32_t *)malloc((size_t)max_r * 4);
    uint32_t *acc_d = (uint32_t *)malloc((size_t)max_r * 4);
    uint32_t *acc_w = (uint32_t *)malloc((size_t)max_r * 4);
    uint32_t *k_nf = (uint32_t *)malloc((size_t)max_r * 4);
    uint32_t *path = (uint32_t *)malloc((size_t)max_r * 4);
    Piece *next_pcs = (Piece *)malloc((size_t)max_r * sizeof(Piece));

    uint64_t to_ord_addr = sim_alloc_region(&sim, (size_t)max_r * 4);
    uint64_t tmp_ord_addr = sim_alloc_region(&sim, (size_t)max_r * 4);
    uint64_t succ_addr = sim_alloc_region(&sim, (size_t)max_r * 4);
    uint64_t delta_addr = sim_alloc_region(&sim, (size_t)max_r * 4);
    uint64_t state_addr = sim_alloc_region(&sim, (size_t)max_r);
    uint64_t root_addr = sim_alloc_region(&sim, (size_t)max_r * 4);
    uint64_t acc_d_addr = sim_alloc_region(&sim, (size_t)max_r * 4);
    uint64_t acc_w_addr = sim_alloc_region(&sim, (size_t)max_r * 4);
    uint64_t knf_addr = sim_alloc_region(&sim, (size_t)max_r * 4);
    uint64_t next_pcs_addr = sim_alloc_region(&sim, (size_t)max_r * sizeof(Piece));

    uint32_t closed_word = UINT32_MAX;
    uint32_t rounds = 1;

    while (1) {
        rounds++;
        if (r == 1) { closed_word = pcs[0].wid; break; }

        /* Binary search p_idx in sorted pcs[0..r].from */
        uint32_t lo = 0, hi = r;
        while (lo + 1 < hi) {
            uint32_t mid = (lo + hi) >> 1;
            if (pcs[mid].from <= cur_p) lo = mid; else hi = mid;
        }
        uint32_t p_idx = lo;

        /* Sort pieces by `to` via 2-pass 10-bit radix sort */
        radix_sort_by_to(&sim, pcs, pcs_addr, r, to_ord, to_ord_addr, tmp_ord, tmp_ord_addr);

        /* Linear 2-pointer merge (to_ord vs pcs.from) -> succ & delta */
        uint32_t v = 0;
        for (uint32_t oi = 0; oi < r; oi++) {
            sim_access(&sim, to_ord_addr + (uint64_t)oi * 4, 4, 0);
            uint32_t k = to_ord[oi];
            sim_access(&sim, pcs_addr + (uint64_t)k * sizeof(Piece), sizeof(Piece), 0);
            uint32_t t0 = pcs[k].to, t1 = t0 + pcs[k].w;
            while (v + 1 < r && pcs[v].from + pcs[v].w <= t0) { v++; sim_access(&sim, pcs_addr + (uint64_t)v * sizeof(Piece), sizeof(Piece), 0); }
            sim_access(&sim, pcs_addr + (uint64_t)v * sizeof(Piece), sizeof(Piece), 0);
            if (pcs[v].from + pcs[v].w >= t1) {
                succ[k] = v;
                delta[k] = t0 - pcs[v].from;
                state[k] = 0;
            } else {
                succ[k] = UINT32_MAX;
                state[k] = 2; root[k] = k; acc_d[k] = 0; acc_w[k] = UINT32_MAX;
            }
            sim_access(&sim, succ_addr + (uint64_t)k * 4, 4, 1);
            sim_access(&sim, delta_addr + (uint64_t)k * 4, 4, 1);
            sim_access(&sim, state_addr + k, 1, 1);
            met->work_events++;
        }

        /* Memoized forest resolution */
        for (uint32_t u = 0; u < r; u++) {
            sim_access(&sim, state_addr + u, 1, 0);
            if (state[u] != 0) continue;
            uint32_t plen = 0, c = u;
            while (state[c] == 0) {
                state[c] = 1; sim_access(&sim, state_addr + c, 1, 1);
                path[plen++] = c;
                sim_access(&sim, succ_addr + (uint64_t)c * 4, 4, 0);
                c = succ[c];
                sim_access(&sim, state_addr + c, 1, 0);
            }
            if (state[c] == 1) {
                /* Cycle inside U! */
                uint32_t ci = 0; while (path[ci] != c) ci++;
                for (uint32_t q = 0; q < plen; q++) { state[path[q]] = 3; sim_access(&sim, state_addr + path[q], 1, 1); }
                uint32_t pi = UINT32_MAX;
                for (uint32_t q = ci; q < plen; q++) if (path[q] == p_idx) { pi = q - ci; break; }
                if (pi != UINT32_MAX) {
                    uint32_t clen = plen - ci, wtot = UINT32_MAX;
                    for (uint32_t stp = 0; stp < clen; stp++) {
                        uint32_t nd = path[ci + ((pi + stp) % clen)];
                        wtot = ufc_join(pcs[nd].wid, wtot, n_lits, r1.lit_off, &rules, &nrules, &cap_rules, &sim, rules_addr);
                    }
                    closed_word = wtot;
                    break;
                }
                continue;
            } else if (state[c] == 3) {
                for (uint32_t q = 0; q < plen; q++) { state[path[q]] = 3; sim_access(&sim, state_addr + path[q], 1, 1); }
                continue;
            } else {
                sim_access(&sim, root_addr + (uint64_t)c * 4, 4, 0);
                sim_access(&sim, acc_d_addr + (uint64_t)c * 4, 4, 0);
                sim_access(&sim, acc_w_addr + (uint64_t)c * 4, 4, 0);
                uint32_t rt = root[c], ad = acc_d[c], aw = acc_w[c];
                for (int q = (int)plen - 1; q >= 0; q--) {
                    uint32_t nd = path[q];
                    sim_access(&sim, delta_addr + (uint64_t)nd * 4, 4, 0);
                    sim_access(&sim, pcs_addr + (uint64_t)nd * sizeof(Piece), sizeof(Piece), 0);
                    ad += delta[nd];
                    aw = ufc_join(aw, pcs[nd].wid, n_lits, r1.lit_off, &rules, &nrules, &cap_rules, &sim, rules_addr);
                    root[nd] = rt; acc_d[nd] = ad; acc_w[nd] = aw; state[nd] = 2;
                    sim_access(&sim, root_addr + (uint64_t)nd * 4, 4, 1);
                    sim_access(&sim, acc_d_addr + (uint64_t)nd * 4, 4, 1);
                    sim_access(&sim, acc_w_addr + (uint64_t)nd * 4, 4, 1);
                    sim_access(&sim, state_addr + nd, 1, 1);
                    met->work_events++;
                }
            }
        }
        if (closed_word != UINT32_MAX) break;

        if (succ[p_idx] != UINT32_MAX) {
            mark_shift += ufc_wlen(acc_w[p_idx], n_lits, r1.lit_off, rules);
            uint32_t in_p = cur_p - pcs[p_idx].from;
            uint32_t rt_p = root[p_idx];
            cur_p = pcs[rt_p].from + acc_d[p_idx] + in_p;
            p_idx = rt_p;
        }

        uint32_t new_n = 0;
        for (uint32_t k = 0; k < r; k++) {
            if (succ[k] == UINT32_MAX) {
                k_nf[k] = new_n;
                sim_access(&sim, knf_addr + (uint64_t)k * 4, 4, 1);
                new_n += pcs[k].w;
            }
        }
        uint32_t new_p = k_nf[p_idx] + (cur_p - pcs[p_idx].from);

        /* Emit next_pcs via 2-pointer merge over to_ord for splitters (wait: to keep next_pcs sorted by from,
         * we can iterate k in 0..r-1 using binary search OR iterate to_ord & scatter! Wait: we want next_pcs
         * sorted by `from`, and there are only <= 2,000 splitters in Round 2+! Binary search on `from` for
         * ONLY the ~1,000-2,000 splitters takes 12 steps in L1!) */
        uint32_t nr2 = 0;
        for (uint32_t k = 0; k < r; k++) {
            sim_access(&sim, succ_addr + (uint64_t)k * 4, 4, 0);
            if (succ[k] != UINT32_MAX) continue;
            sim_access(&sim, pcs_addr + (uint64_t)k * sizeof(Piece), sizeof(Piece), 0);
            uint32_t t0 = pcs[k].to, t1 = t0 + pcs[k].w, wid_k = pcs[k].wid;
            uint32_t base_f = k_nf[k];
            uint32_t lo2 = 0, hi2 = r;
            while (lo2 + 1 < hi2) {
                uint32_t mid = (lo2 + hi2) >> 1;
                if (pcs[mid].from <= t0) lo2 = mid; else hi2 = mid;
            }
            uint32_t vv = lo2;
            while (vv < r) {
                sim_access(&sim, pcs_addr + (uint64_t)vv * sizeof(Piece), sizeof(Piece), 0);
                if (pcs[vv].from >= t1) break;
                uint32_t ov0 = t0 > pcs[vv].from ? t0 : pcs[vv].from;
                uint32_t ov1 = t1 < (pcs[vv].from + pcs[vv].w) ? t1 : (pcs[vv].from + pcs[vv].w);
                uint32_t ow = ov1 - ov0, in_v = ov0 - pcs[vv].from;
                uint32_t sf = base_f + (ov0 - t0), st, sw;
                sim_access(&sim, succ_addr + (uint64_t)vv * 4, 4, 0);
                if (succ[vv] == UINT32_MAX) {
                    st = k_nf[vv] + in_v; sw = wid_k;
                } else {
                    sim_access(&sim, root_addr + (uint64_t)vv * 4, 4, 0);
                    sim_access(&sim, acc_d_addr + (uint64_t)vv * 4, 4, 0);
                    sim_access(&sim, acc_w_addr + (uint64_t)vv * 4, 4, 0);
                    st = k_nf[root[vv]] + acc_d[vv] + in_v;
                    sw = ufc_join(acc_w[vv], wid_k, n_lits, r1.lit_off, &rules, &nrules, &cap_rules, &sim, rules_addr);
                }
                next_pcs[nr2] = (Piece){sf, st, ow, sw};
                sim_access(&sim, next_pcs_addr + (uint64_t)nr2 * sizeof(Piece), sizeof(Piece), 1);
                nr2++; vv++; met->work_events++;
            }
        }
        Piece *tp = pcs; pcs = next_pcs; next_pcs = tp;
        uint64_t ta = pcs_addr; pcs_addr = next_pcs_addr; next_pcs_addr = ta;
        r = nr2; cur_p = new_p;
    }
    met->l2_p2 = sim.l2.misses - met->l2_p1;
    double t2 = now_ms(); met->ms_p2 = t2 - t1;

    size_t r2_mem = nrecs * 2 + (size_t)nrules * sizeof(SlpRule) + r1.lit_spell_len + (size_t)n_lits * 4 + (size_t)max_r * 64;
    free(pcs); free(next_pcs); free(to_ord); free(tmp_ord); free(succ); free(delta);
    free(state); free(root); free(acc_d); free(acc_w); free(k_nf); free(path);

    /* Direct-Rotated First-Occurrence SLP Expansion into out[0..n] (ZERO extra rotation buffer!) */
    uint32_t L_cyc = ufc_wlen(closed_word, n_lits, r1.lit_off, rules);
    assert(L_cyc == n); /* all 6 admitted Wikidata blocks are primitive! */
    uint32_t sh = mark_shift % L_cyc;
    uint8_t *out = (uint8_t *)malloc(n);
    uint64_t out_addr = sim_alloc_region(&sim, n);

    uint32_t *stack = (uint32_t *)malloc(65536 * 4);
    uint32_t sp = 0; stack[sp++] = closed_word;
    uint32_t vpos = 0; /* virtual position in unrotated out_cyc [0..L_cyc) */
    uint32_t tail_base = L_cyc - sh; /* virtual [0..sh) -> physical [tail_base..L_cyc); virtual [sh..L_cyc) -> physical [0..tail_base) */

    while (sp > 0) {
        uint32_t wid = stack[--sp];
        if (wid < n_lits) {
            sim_access(&sim, r1.lit_off_addr + (uint64_t)wid * 4, 8, 0);
            uint32_t st = r1.lit_off[wid], ln = r1.lit_off[wid + 1] - st;
            sim_access(&sim, r1.lit_spell_addr + st, ln, 0);
            if (vpos >= sh) {
                uint32_t dst = vpos - sh;
                memcpy(out + dst, r1.lit_spell + st, ln);
                sim_access(&sim, out_addr + dst, ln, 1);
            } else if (vpos + ln <= sh) {
                uint32_t dst = tail_base + vpos;
                memcpy(out + dst, r1.lit_spell + st, ln);
                sim_access(&sim, out_addr + dst, ln, 1);
            } else {
                uint32_t p1 = sh - vpos, p2 = ln - p1;
                memcpy(out + tail_base + vpos, r1.lit_spell + st, p1);
                memcpy(out, r1.lit_spell + st + p1, p2);
                sim_access(&sim, out_addr + tail_base + vpos, p1, 1);
                sim_access(&sim, out_addr, p2, 1);
            }
            vpos += ln; met->work_events++;
        } else {
            uint32_t ri = wid - n_lits;
            sim_access(&sim, rules_addr + (uint64_t)ri * sizeof(SlpRule), sizeof(SlpRule), 0);
            SlpRule *rl = &rules[ri];
            if (rl->first == UINT32_MAX) {
                rl->first = vpos;
                sim_access(&sim, rules_addr + (uint64_t)ri * sizeof(SlpRule), sizeof(SlpRule), 1);
                stack[sp++] = rl->right; stack[sp++] = rl->left;
            } else {
                uint32_t sv = rl->first, ln = rl->len;
                /* Copy virtual [sv..sv+ln) -> virtual [vpos..vpos+ln), splitting only if either crosses `sh`! */
                uint32_t rem = ln, cs = sv, cd = vpos;
                while (rem > 0) {
                    uint32_t psrc = (cs < sh) ? (tail_base + cs) : (cs - sh);
                    uint32_t pdst = (cd < sh) ? (tail_base + cd) : (cd - sh);
                    uint32_t s_avail = (cs < sh) ? (sh - cs) : (L_cyc - cs);
                    uint32_t d_avail = (cd < sh) ? (sh - cd) : (L_cyc - cd);
                    uint32_t chunk = rem;
                    if (s_avail < chunk) chunk = s_avail;
                    if (d_avail < chunk) chunk = d_avail;
                    sim_access(&sim, out_addr + psrc, chunk, 0);
                    memcpy(out + pdst, out + psrc, chunk);
                    sim_access(&sim, out_addr + pdst, chunk, 1);
                    cs += chunk; cd += chunk; rem -= chunk;
                }
                vpos += ln; met->work_events++;
            }
        }
    }
    assert(vpos == L_cyc);
    met->l2_p3 = sim.l2.misses - met->l2_p1 - met->l2_p2;
    double t3 = now_ms(); met->ms_p3 = t3 - t2;

    size_t emit_mem = (size_t)nrules * sizeof(SlpRule) + r1.lit_spell_len + (size_t)n_lits * 4 + n;
    size_t pk = r1.r1_peak_mem;
    if (r2_mem > pk) pk = r2_mem;
    if (emit_mem > pk) pk = emit_mem;

    sim_drain(&sim);
    met->l1_misses = sim.l1.misses; met->l2_misses = sim.l2.misses; met->llc_misses = sim.llc.misses;
    met->dram_bytes = (sim.dram_reads + sim.dram_writes) * 64ULL;
    met->peak_mem = pk;
    met->extra1 = nrules; met->extra2 = rounds;

    free(rules); free(stack); free(r1.lit_spell); free(r1.lit_off);
    sim_free(&sim);
    return out;
}

/* ==================== 4. UFC-1 + Open (Flat-R1 UFC -> L2-Resident Open) ==================== */

static uint8_t *run_ufc1_open(const uint16_t *recs, size_t nrecs, uint32_t n, uint32_t p, Metrics *met) {
    const uint32_t LIMIT = 16;
    CacheSim sim; sim_init(&sim);
    memset(met, 0, sizeof(*met));
    double t0 = now_ms();

    R1Result r1 = run_ufc_round1(&sim, met, recs, nrecs, n, p);
    met->l2_p1 = sim.l2.misses;
    double t1 = now_ms(); met->ms_p1 = t1 - t0;

    Piece *pcs = r1.pieces;
    uint64_t pcs_addr = r1.pieces_addr;
    uint32_t np = r1.npieces;
    uint32_t n1 = r1.new_n, p1 = r1.new_p, mark_shift = r1.mark_shift;
    uint32_t n_lits = r1.n_lits;

    /* Classify reduced pieces into thin (w <= LIMIT, retained) and thick (w > LIMIT, eliminated).
     * Split thick piece containing p1 so p1 is a retained singleton! */
    typedef struct { uint32_t from, to, w, wid; } PStrip;
    PStrip *strips = (PStrip *)malloc((size_t)(np + 4) * sizeof(PStrip));
    uint64_t strips_addr = sim_alloc_region(&sim, (size_t)(np + 4) * sizeof(PStrip));
    PStrip *thins = (PStrip *)malloc((size_t)(np + 4) * sizeof(PStrip));
    uint64_t thins_addr = sim_alloc_region(&sim, (size_t)(np + 4) * sizeof(PStrip));
    uint32_t nstrips = 0, nthins = 0, m1 = 0, mark = 0;

    for (uint32_t i = 0; i < np; i++) {
        sim_access(&sim, pcs_addr + (uint64_t)i * sizeof(Piece), sizeof(Piece), 0);
        Piece pc = pcs[i];
        uint32_t end = pc.from + pc.w;
        if (pc.w > LIMIT && pc.from <= p1 && p1 < end) {
            if (pc.from < p1) {
                uint32_t lw = p1 - pc.from;
                strips[nstrips] = (PStrip){pc.from, pc.to, lw, pc.wid};
                sim_access(&sim, strips_addr + (uint64_t)nstrips * sizeof(PStrip), sizeof(PStrip), 1);
                nstrips++;
            }
            mark = m1;
            thins[nthins] = (PStrip){m1, pc.to + (p1 - pc.from), 1, pc.wid};
            sim_access(&sim, thins_addr + (uint64_t)nthins * sizeof(PStrip), sizeof(PStrip), 1);
            nthins++; m1 += 1;
            if (p1 + 1 < end) {
                uint32_t rw = end - p1 - 1;
                strips[nstrips] = (PStrip){p1 + 1, pc.to + (p1 + 1 - pc.from), rw, pc.wid};
                sim_access(&sim, strips_addr + (uint64_t)nstrips * sizeof(PStrip), sizeof(PStrip), 1);
                nstrips++;
            }
        } else if (pc.w <= LIMIT) {
            if (pc.from <= p1 && p1 < end) mark = m1 + (p1 - pc.from);
            thins[nthins] = (PStrip){m1, pc.to, pc.w, pc.wid};
            sim_access(&sim, thins_addr + (uint64_t)nthins * sizeof(PStrip), sizeof(PStrip), 1);
            nthins++; m1 += pc.w;
        } else {
            strips[nstrips] = (PStrip){pc.from, pc.to, pc.w, pc.wid};
            sim_access(&sim, strips_addr + (uint64_t)nstrips * sizeof(PStrip), sizeof(PStrip), 1);
            nstrips++;
        }
        met->work_events++;
    }
    free(pcs);

    /* Radix-sort the ONLY nstrips (592-2,046!) thick strips by `to` */
    uint32_t *s_ord = (uint32_t *)malloc((size_t)(nstrips + 1) * 4);
    uint32_t *s_tmp = (uint32_t *)malloc((size_t)(nstrips + 1) * 4);
    uint64_t sord_addr = sim_alloc_region(&sim, (size_t)(nstrips + 1) * 4);
    {
        uint32_t c0[1024] = {0}, c1[1024] = {0};
        for (uint32_t i = 0; i < nstrips; i++) {
            sim_access(&sim, strips_addr + (uint64_t)i * sizeof(PStrip), sizeof(PStrip), 0);
            c0[strips[i].to & 1023u]++; c1[(strips[i].to >> 10) & 1023u]++;
        }
        uint32_t a0 = 0, a1 = 0;
        for (int b = 0; b < 1024; b++) { uint32_t v0 = c0[b]; c0[b] = a0; a0 += v0; uint32_t v1 = c1[b]; c1[b] = a1; a1 += v1; }
        for (uint32_t i = 0; i < nstrips; i++) s_tmp[c0[strips[i].to & 1023u]++] = i;
        for (uint32_t i = 0; i < nstrips; i++) {
            uint32_t id = s_tmp[i]; uint32_t d = c1[(strips[id].to >> 10) & 1023u]++;
            s_ord[d] = id; sim_access(&sim, sord_addr + (uint64_t)d * 4, 4, 1);
        }
    }
    free(s_tmp);

    /* Open Endpoint Elimination on ONLY nstrips (592-2,046!) strips!
     * Node word encoding: 0 = gap/empty; 1..n_lits = lit_word (wid = word - 1); > n_lits = EpRule (ri = word - n_lits - 1) */
    size_t max_nodes = 3 * (size_t)nstrips + 5;
    EpNode *nodes = (EpNode *)calloc(max_nodes, sizeof(EpNode));
    uint64_t nodes_addr = sim_alloc_region(&sim, max_nodes * sizeof(EpNode));
    uint32_t nnodes = 1;
    uint32_t *ord0 = (uint32_t *)malloc(max_nodes * 4), *ord1 = (uint32_t *)malloc(max_nodes * 4);
    uint32_t *term0 = (uint32_t *)malloc(max_nodes * 4), *term1 = (uint32_t *)malloc(max_nodes * 4);
    size_t n_o0 = 0, n_o1 = 0, n_t0 = 0, n_t1 = 0;

    for (uint32_t i = 0; i < nstrips; i++) {
        nodes[nnodes].width = strips[i].w; nodes[nnodes].word = strips[i].wid + 1;
        sim_access(&sim, nodes_addr + (uint64_t)nnodes * sizeof(EpNode), sizeof(EpNode), 1);
        nnodes++;
    }
    uint32_t x = 0;
    for (uint32_t i = 0; i < nstrips; i++) {
        if (x < strips[i].from) { uint32_t id = nnodes++; nodes[id].width = strips[i].from - x; ord0[n_o0++] = id; term1[n_t1++] = id; }
        ord0[n_o0++] = i + 1; x = strips[i].from + strips[i].w;
    }
    if (x < n1) { uint32_t id = nnodes++; nodes[id].width = n1 - x; ord0[n_o0++] = id; term1[n_t1++] = id; }
    x = 0;
    for (uint32_t k = 0; k < nstrips; k++) {
        uint32_t i = s_ord[k];
        if (x < strips[i].to) { uint32_t id = nnodes++; nodes[id].width = strips[i].to - x; ord1[n_o1++] = id; term0[n_t0++] = id; }
        ord1[n_o1++] = i + 1; x = strips[i].to + strips[i].w;
    }
    if (x < n1) { uint32_t id = nnodes++; nodes[id].width = n1 - x; ord1[n_o1++] = id; term0[n_t0++] = id; }
    for (int side = 0; side < 2; side++) {
        uint32_t prev = 0;
        uint32_t *tarr = side == 0 ? term0 : term1; size_t nt = side == 0 ? n_t0 : n_t1;
        uint32_t *oarr = side == 0 ? ord0 : ord1;   size_t no = side == 0 ? n_o0 : n_o1;
        for (size_t k = 0; k < nt; k++) { uint32_t id = tarr[k]; nodes[prev].next[side] = id; nodes[id].prev[side] = prev; prev = id; }
        for (size_t k = 0; k < no; k++) { uint32_t id = oarr[k]; nodes[prev].next[side] = id; nodes[id].prev[side] = prev; prev = id; }
        nodes[prev].next[side] = 0; nodes[0].prev[side] = prev;
    }

    size_t cap_rules = 16384;
    EpRule *rules = (EpRule *)malloc(cap_rules * sizeof(EpRule));
    uint64_t rules_addr = sim_alloc_region(&sim, 65536 * sizeof(EpRule));
    uint32_t nrules = 0, rem = n1;
    while (rem > 0) {
        uint32_t ix = nodes[0].prev[0], iy = nodes[0].prev[1];
        sim_access(&sim, nodes_addr + (uint64_t)ix * sizeof(EpNode), sizeof(EpNode), 0);
        sim_access(&sim, nodes_addr + (uint64_t)iy * sizeof(EpNode), sizeof(EpNode), 0);
        EpNode a = nodes[ix], b = nodes[iy];
        uint32_t w = a.width < b.width ? a.width : b.width;
        rem -= w; met->work_events++;
        if (ix == iy) { ep_cut(&sim, nodes_addr, nodes, ix, 0); ep_cut(&sim, nodes_addr, nodes, ix, 1); continue; }
        uint32_t word;
        if (a.word == 0) word = b.word;
        else if (b.word == 0) word = a.word;
        else {
            if (nrules == cap_rules) { cap_rules *= 2; rules = (EpRule *)realloc(rules, cap_rules * sizeof(EpRule)); }
            uint32_t la = (a.word <= n_lits) ? (r1.lit_off[a.word] - r1.lit_off[a.word - 1]) : rules[a.word - n_lits - 1].len;
            uint32_t lb = (b.word <= n_lits) ? (r1.lit_off[b.word] - r1.lit_off[b.word - 1]) : rules[b.word - n_lits - 1].len;
            rules[nrules] = (EpRule){a.word, b.word, la + lb, UINT32_MAX};
            sim_access(&sim, rules_addr + (uint64_t)nrules * sizeof(EpRule), sizeof(EpRule), 1);
            word = nrules + n_lits + 1; nrules++;
        }
        if (a.width <= b.width) {
            ep_cut(&sim, nodes_addr, nodes, ix, 0); ep_insert(&sim, nodes_addr, nodes, iy, ix, 0);
            nodes[ix].word = word;
            if (a.width == b.width) { ep_cut(&sim, nodes_addr, nodes, iy, 0); ep_cut(&sim, nodes_addr, nodes, iy, 1); }
            else nodes[iy].width -= w;
        } else {
            ep_cut(&sim, nodes_addr, nodes, iy, 1); ep_insert(&sim, nodes_addr, nodes, ix, iy, 1);
            nodes[iy].word = word; nodes[ix].width -= w;
        }
    }

    /* Compute bottom (L-rank in [0..m1)) and top bands (in F-gap rank [0..m1))! */
    uint32_t *bottom = (uint32_t *)malloc(nnodes * 4);
    uint32_t id = nodes[0].next[1]; x = 0;
    while (id != 0) { bottom[id] = x; x += nodes[id].width; id = nodes[id].next[1]; }
    assert(x == m1);

    /* Spell thick band words into thick_spell (only for bands with node.word != 0!) */
    typedef struct { uint32_t end, delta, tw_id; } TBand; /* tw_id = 0 => no thick word; >= 1 => thick_off[tw_id-1..tw_id] */
    TBand *bands = (TBand *)malloc((nnodes + 16) * sizeof(TBand));
    uint64_t bands_addr = sim_alloc_region(&sim, (nnodes + 16) * sizeof(TBand));
    size_t cap_ts = 131072, ts_len = 0;
    uint8_t *tspell = (uint8_t *)malloc(cap_ts);
    uint64_t tspell_addr = sim_alloc_region(&sim, 262144);
    uint32_t *toff = (uint32_t *)malloc((nnodes + 16) * 4);
    uint64_t toff_addr = sim_alloc_region(&sim, (nnodes + 16) * 4);
    toff[0] = 0; uint32_t ntw = 0, nbands = 0;
    uint32_t *stack = (uint32_t *)malloc(65536 * 4);

    id = nodes[0].next[0]; uint32_t k = 0;
    while (id != 0) {
        EpNode nd = nodes[id];
        uint32_t dst = bottom[id], tw_id = 0;
        if (nd.word != 0) {
            size_t sp = 0; stack[sp++] = nd.word;
            while (sp > 0) {
                uint32_t wid = stack[--sp];
                if (wid <= n_lits) {
                    uint32_t st = r1.lit_off[wid - 1], ln = r1.lit_off[wid] - st;
                    if (ts_len + ln + 2 >= cap_ts) { while (ts_len + ln + 2 >= cap_ts) cap_ts *= 2; tspell = (uint8_t *)realloc(tspell, cap_ts); }
                    sim_access(&sim, r1.lit_spell_addr + st, ln, 0);
                    memcpy(tspell + ts_len, r1.lit_spell + st, ln);
                    sim_access(&sim, tspell_addr + ts_len, ln, 1);
                    ts_len += ln;
                } else {
                    EpRule *rl = &rules[wid - n_lits - 1];
                    if (rl->first == UINT32_MAX) {
                        rl->first = (uint32_t)ts_len;
                        stack[sp++] = rl->right; stack[sp++] = rl->left;
                    } else {
                        uint32_t st = rl->first, ln = rl->len;
                        if (ts_len + ln + 2 >= cap_ts) { while (ts_len + ln + 2 >= cap_ts) cap_ts *= 2; tspell = (uint8_t *)realloc(tspell, cap_ts); }
                        sim_access(&sim, tspell_addr + st, ln, 0);
                        memcpy(tspell + ts_len, tspell + st, ln);
                        sim_access(&sim, tspell_addr + ts_len, ln, 1);
                        ts_len += ln;
                    }
                }
            }
            toff[++ntw] = (uint32_t)ts_len;
            sim_access(&sim, toff_addr + (uint64_t)ntw * 4, 4, 1);
            tw_id = ntw;
        }
        bands[nbands] = (TBand){k + nd.width, dst - k, tw_id};
        sim_access(&sim, bands_addr + (uint64_t)nbands * sizeof(TBand), sizeof(TBand), 1);
        nbands++; k += nd.width; id = nd.next[0];
    }
    assert(k == m1);
    free(nodes); free(ord0); free(ord1); free(term0); free(term1); free(rules); free(bottom); free(stack);
    met->l2_p2 = sim.l2.misses - met->l2_p1;
    double t2 = now_ms(); met->ms_p2 = t2 - t1;

    /* Wire L2-resident edges[0..m1] in BACKWARD direction!
     * Radix-sort the thin pieces (nthins = 13k-33k) by `to`, merge against s_ord (592-2,046 thick strips)
     * to get each thin piece's exact F-gap rank y in [0..m1), then step through bands!
     * Edge layout: since m1 <= 70,083 < 2^17, pack 1 edge per retained row as:
     *   edges_next[z] (u32): next_z (17b) | tw_id (15b, 0 = none!)
     *   edges_lit[z]  (u16): lit_id (16b, since n_lits <= 35,790 < 65,536!) -> 6 B/row = 156-420 KiB!
     * Wait: even cleaner & 100% cache-friendly: ONE 8-byte struct { uint32_t next_and_tw; uint32_t lit_id; } OR
     * 6B split arrays: edges_lit[z] is ONLY read in L2 (52-140 KiB), edges_next[z] is 104-280 KiB! */
    uint32_t *t_ord = (uint32_t *)malloc((size_t)(nthins + 1) * 4);
    uint32_t *t_tmp = (uint32_t *)malloc((size_t)(nthins + 1) * 4);
    uint64_t tord_addr = sim_alloc_region(&sim, (size_t)(nthins + 1) * 4);
    {
        uint32_t c0[1024] = {0}, c1[1024] = {0};
        for (uint32_t i = 0; i < nthins; i++) {
            sim_access(&sim, thins_addr + (uint64_t)i * sizeof(PStrip), sizeof(PStrip), 0);
            c0[thins[i].to & 1023u]++; c1[(thins[i].to >> 10) & 1023u]++;
        }
        uint32_t a0 = 0, a1 = 0;
        for (int b = 0; b < 1024; b++) { uint32_t v0 = c0[b]; c0[b] = a0; a0 += v0; uint32_t v1 = c1[b]; c1[b] = a1; a1 += v1; }
        for (uint32_t i = 0; i < nthins; i++) t_tmp[c0[thins[i].to & 1023u]++] = i;
        for (uint32_t i = 0; i < nthins; i++) { uint32_t tid = t_tmp[i]; t_ord[c1[(thins[tid].to >> 10) & 1023u]++] = tid; }
    }
    free(t_tmp);

    assert(m1 < (1u << 17) && ntw < (1u << 15) && n_lits < 65536u);
    /* Store FORWARD edges: edges_fwd[dst] = (tw_id << 17) | src, edges_lit[dst] = lit_id!
     * Wait: in forward direction (Psi), walking forward from mark writes out[0..n] ASCENDING!
     * Let's verify: backward step takes src -> (thin word lit_id) -> (thick word tw_id) -> dst.
     * In FORWARD text order, backward step's word is (thick word tw_id) + (thin word lit_id)!
     * So in FORWARD walk: at `dst`, edge goes to `src`, emitting `thick word tw_id` THEN `thin word lit_id`!
     * That matches Shipped Open's exact forward emit direction! */
    uint32_t *edges_nxt = (uint32_t *)malloc((size_t)m1 * 4);
    uint16_t *edges_lit = (uint16_t *)malloc((size_t)m1 * 2);
    uint64_t enxt_addr = sim_alloc_region(&sim, (size_t)m1 * 4);
    uint64_t elit_addr = sim_alloc_region(&sim, (size_t)m1 * 2);

    uint32_t si = 0, thick_before = 0, bi = 0;
    for (uint32_t ti = 0; ti < nthins; ti++) {
        sim_access(&sim, tord_addr + (uint64_t)ti * 4, 4, 0);
        uint32_t tid = t_ord[ti];
        sim_access(&sim, thins_addr + (uint64_t)tid * sizeof(PStrip), sizeof(PStrip), 0);
        PStrip th = thins[tid];
        while (si < nstrips && strips[s_ord[si]].to < th.to) {
            thick_before += strips[s_ord[si]].w;
            si++;
        }
        uint32_t y = th.to - thick_before;
        uint32_t rem_w = th.w, src = th.from;
        while (rem_w > 0) {
            while (bands[bi].end <= y) bi++;
            sim_access(&sim, bands_addr + (uint64_t)bi * sizeof(TBand), sizeof(TBand), 0);
            TBand bd = bands[bi];
            uint32_t avail = bd.end - y;
            uint32_t take = rem_w < avail ? rem_w : avail;
            uint32_t dst0 = y + bd.delta;
            uint32_t pack = bd.tw_id << 17;
            for (uint32_t u = 0; u < take; u++) {
                edges_nxt[dst0 + u] = pack | (src + u);
                edges_lit[dst0 + u] = (uint16_t)th.wid;
            }
            sim_access(&sim, enxt_addr + (uint64_t)dst0 * 4, (size_t)take * 4, 1);
            sim_access(&sim, elit_addr + (uint64_t)dst0 * 2, (size_t)take * 2, 1);
            y += take; src += take; rem_w -= take; met->work_events += take;
        }
    }
    free(strips); free(s_ord); free(thins); free(t_ord); free(bands);
    met->l2_p3 = sim.l2.misses - met->l2_p1 - met->l2_p2;
    double t3 = now_ms(); met->ms_p3 = t3 - t2;

    /* Forward emit over ONLY m1 (26k-70k) L2-resident edges with direct-rotated writes into out[0..n]! */
    uint8_t *out = (uint8_t *)malloc(n);
    uint64_t out_addr = sim_alloc_region(&sim, n);
    uint32_t sh = mark_shift % n;
    uint32_t tail_base = n - sh;
    uint32_t vpos = 0, j = mark;

    for (uint32_t step = 0; step < m1 && vpos < n; step++) {
        sim_access(&sim, enxt_addr + (uint64_t)j * 4, 4, 0);
        sim_access(&sim, elit_addr + (uint64_t)j * 2, 2, 0);
        uint32_t e = edges_nxt[j];
        uint32_t lid = edges_lit[j];
        uint32_t tw_id = e >> 17;
        j = e & 0x1FFFFu;
        if (tw_id > 0) {
            sim_access(&sim, toff_addr + (uint64_t)(tw_id - 1) * 4, 8, 0);
            uint32_t st = toff[tw_id - 1], ln = toff[tw_id] - st;
            sim_access(&sim, tspell_addr + st, ln, 0);
            if (vpos >= sh) { memcpy(out + (vpos - sh), tspell + st, ln); sim_access(&sim, out_addr + (vpos - sh), ln, 1); }
            else if (vpos + ln <= sh) { memcpy(out + tail_base + vpos, tspell + st, ln); sim_access(&sim, out_addr + tail_base + vpos, ln, 1); }
            else { uint32_t a = sh - vpos, b = ln - a; memcpy(out + tail_base + vpos, tspell + st, a); memcpy(out, tspell + st + a, b); sim_access(&sim, out_addr + tail_base + vpos, a, 1); sim_access(&sim, out_addr, b, 1); }
            vpos += ln;
        }
        {
            sim_access(&sim, r1.lit_off_addr + (uint64_t)lid * 4, 8, 0);
            uint32_t st = r1.lit_off[lid], ln = r1.lit_off[lid + 1] - st;
            sim_access(&sim, r1.lit_spell_addr + st, ln, 0);
            if (vpos >= sh) { memcpy(out + (vpos - sh), r1.lit_spell + st, ln); sim_access(&sim, out_addr + (vpos - sh), ln, 1); }
            else if (vpos + ln <= sh) { memcpy(out + tail_base + vpos, r1.lit_spell + st, ln); sim_access(&sim, out_addr + tail_base + vpos, ln, 1); }
            else { uint32_t a = sh - vpos, b = ln - a; memcpy(out + tail_base + vpos, r1.lit_spell + st, a); memcpy(out, r1.lit_spell + st + a, b); sim_access(&sim, out_addr + tail_base + vpos, a, 1); sim_access(&sim, out_addr, b, 1); }
            vpos += ln;
        }
        met->work_events++;
    }
    assert(vpos == n);
    met->l2_p4 = sim.l2.misses - met->l2_p1 - met->l2_p2 - met->l2_p3;
    double t4 = now_ms(); met->ms_p4 = t4 - t3;

    size_t emit_mem = (size_t)m1 * 6 + r1.lit_spell_len + (size_t)n_lits * 4 + ts_len + (size_t)ntw * 4 + n;
    size_t pk = r1.r1_peak_mem > emit_mem ? r1.r1_peak_mem : emit_mem;

    sim_drain(&sim);
    met->l1_misses = sim.l1.misses; met->l2_misses = sim.l2.misses; met->llc_misses = sim.llc.misses;
    met->dram_bytes = (sim.dram_reads + sim.dram_writes) * 64ULL;
    met->peak_mem = pk;
    met->extra1 = m1; met->extra2 = nstrips;

    free(edges_nxt); free(edges_lit); free(tspell); free(toff); free(r1.lit_spell); free(r1.lit_off);
    sim_free(&sim);
    return out;
}

/* ==================== Main Driver ==================== */

typedef struct { const char *file; uint32_t n; uint32_t p; } BlockSpec;
static const BlockSpec BLOCKS[6] = {
    {"block-0.L",   899931, 264045},
    {"block-2.L",   900000, 320807},
    {"block-85.L",  899935, 325088},
    {"block-143.L", 899869, 802868},
    {"block-207.L", 899998, 205180},
    {"block-210.L", 899959, 791429},
};

#ifdef NO_CACHE_SIM
static int cmp_dbl(const void *a, const void *b) {
    double da = *(const double *)a, db = *(const double *)b;
    return (da < db) ? -1 : (da > db);
}
#endif

int main(int argc, char **argv) {
    if (argc < 2) { fprintf(stderr, "Usage: %s <admitted-bwt-dir>\n", argv[0]); return 1; }
    const char *dir = argv[1];

#ifndef NO_CACHE_SIM
    printf("=== Cycle 4 G5 Falsifier: Shipped Open vs UFC-Full vs UFC-1+Open ===\n");
    printf("Pre-registered PASS rule (all 6 blocks vs Shipped Open):\n");
    printf("  (1) Byte-exact  (2) L2m <= 0.85x  (3) DRAM B/B <= 0.90x  (4) PeakMem <= 1.15x  (5) VM time <= 0.90x\n\n");

    int pass_full = 1, pass_u1 = 1;
    for (int bi = 0; bi < 6; bi++) {
        char path[512]; snprintf(path, sizeof(path), "%s/%s", dir, BLOCKS[bi].file);
        FILE *fp = fopen(path, "rb"); assert(fp != NULL);
        uint32_t n = BLOCKS[bi].n, p = BLOCKS[bi].p;
        uint8_t *L = (uint8_t *)malloc(n); assert(fread(L, 1, n, fp) == n); fclose(fp);
        size_t nrecs = 0; uint16_t *recs = column_to_records(L, n, &nrecs);
        uint8_t *ref = dense_reference(L, n, p);

        Metrics mf, mfull, mu1;
        uint8_t *of = run_forward_open(recs, nrecs, n, p, &mf);
        uint8_t *ou = run_ufc_full(recs, nrecs, n, p, &mfull);
        uint8_t *o1 = run_ufc1_open(recs, nrecs, n, p, &mu1);

        assert(memcmp(of, ref, n) == 0);
        assert(memcmp(ou, ref, n) == 0);
        assert(memcmp(o1, ref, n) == 0);

        double df = (double)mf.dram_bytes / n, du = (double)mfull.dram_bytes / n, d1 = (double)mu1.dram_bytes / n;
        double l2u = (double)mfull.l2_misses / mf.l2_misses, l21 = (double)mu1.l2_misses / mf.l2_misses;
        double dru = du / df, dr1 = d1 / df;
        double pku = (double)mfull.peak_mem / mf.peak_mem, pk1 = (double)mu1.peak_mem / mf.peak_mem;

        if (!(l2u <= 0.85 && dru <= 0.90 && pku <= 1.15)) pass_full = 0;
        if (!(l21 <= 0.85 && dr1 <= 0.90 && pk1 <= 1.15)) pass_u1 = 0;

        printf("%s (Open m=%u | UFC-1 m1=%u, strips=%u | UFC-Full rules=%u, rds=%u):\n",
               BLOCKS[bi].file, mf.extra1, mu1.extra1, mu1.extra2, mfull.extra1, mfull.extra2);
        printf("  Shipped Open: L2m=%7lu [p1=%5lu ep=%5lu wir=%5lu emit=%6lu]  LLC=%6lu  DRAM=%.2f B/B  Work=%7lu  Peak=%.2f MB\n",
               (unsigned long)mf.l2_misses, (unsigned long)mf.l2_p1, (unsigned long)mf.l2_p2, (unsigned long)mf.l2_p3, (unsigned long)mf.l2_p4,
               (unsigned long)mf.llc_misses, df, (unsigned long)mf.work_events, mf.peak_mem / 1e6);
        printf("  UFC-Full    : L2m=%7lu [R1=%5lu R2+=%5lu emit=%6lu]         LLC=%6lu  DRAM=%.2f B/B  Work=%7lu  Peak=%.2f MB | L2=%.3fx DRAM=%.3fx Peak=%.3fx\n",
               (unsigned long)mfull.l2_misses, (unsigned long)mfull.l2_p1, (unsigned long)mfull.l2_p2, (unsigned long)mfull.l2_p3,
               (unsigned long)mfull.llc_misses, du, (unsigned long)mfull.work_events, mfull.peak_mem / 1e6, l2u, dru, pku);
        printf("  UFC-1+Open  : L2m=%7lu [R1=%5lu ep=%5lu wir=%5lu emit=%6lu]  LLC=%6lu  DRAM=%.2f B/B  Work=%7lu  Peak=%.2f MB | L2=%.3fx DRAM=%.3fx Peak=%.3fx\n\n",
               (unsigned long)mu1.l2_misses, (unsigned long)mu1.l2_p1, (unsigned long)mu1.l2_p2, (unsigned long)mu1.l2_p3, (unsigned long)mu1.l2_p4,
               (unsigned long)mu1.llc_misses, d1, (unsigned long)mu1.work_events, mu1.peak_mem / 1e6, l21, dr1, pk1);

        free(L); free(recs); free(ref); free(of); free(ou); free(o1);
    }
    printf("CACHE-MODEL VERDICT (6/6 blocks): UFC-Full=%s | UFC-1+Open=%s\n",
           pass_full ? "PASS" : "FAIL", pass_u1 ? "PASS" : "FAIL");
#else
    printf("=== VM Single-Thread C Diagnostic Timing (uninstrumented -O3, median of 15 interleaved rounds; NOT i9 benchmark) ===\n");
    const int ROUNDS = 15;
    for (int bi = 0; bi < 6; bi++) {
        char path[512]; snprintf(path, sizeof(path), "%s/%s", dir, BLOCKS[bi].file);
        FILE *fp = fopen(path, "rb"); assert(fp != NULL);
        uint32_t n = BLOCKS[bi].n, p = BLOCKS[bi].p;
        uint8_t *L = (uint8_t *)malloc(n); assert(fread(L, 1, n, fp) == n); fclose(fp);
        size_t nrecs = 0; uint16_t *recs = column_to_records(L, n, &nrecs);
        uint8_t *ref = dense_reference(L, n, p);
        double tf[15], tu[15], t1[15];
        Metrics mf, mu, m1;
        for (int r = 0; r < ROUNDS; r++) {
            double a = now_ms(); uint8_t *o1 = run_forward_open(recs, nrecs, n, p, &mf); double b = now_ms(); tf[r] = b - a;
            a = now_ms(); uint8_t *o2 = run_ufc_full(recs, nrecs, n, p, &mu); b = now_ms(); tu[r] = b - a;
            a = now_ms(); uint8_t *o3 = run_ufc1_open(recs, nrecs, n, p, &m1); b = now_ms(); t1[r] = b - a;
            if (r == 0) assert(memcmp(o1, ref, n) == 0 && memcmp(o2, ref, n) == 0 && memcmp(o3, ref, n) == 0);
            free(o1); free(o2); free(o3);
        }
        qsort(tf, ROUNDS, sizeof(double), cmp_dbl);
        qsort(tu, ROUNDS, sizeof(double), cmp_dbl);
        qsort(t1, ROUNDS, sizeof(double), cmp_dbl);
        printf("  %-11s: Open=%.2f ms [%.2f/%.2f/%.2f/%.2f] | UFC-Full=%.2f ms (%.2fx) [R1=%.2f R2+=%.2f em=%.2f] | UFC-1+Open=%.2f ms (%.2fx) [R1=%.2f ep=%.2f wir=%.2f em=%.2f]\n",
               BLOCKS[bi].file,
               tf[7], mf.ms_p1, mf.ms_p2, mf.ms_p3, mf.ms_p4,
               tu[7], tu[7]/tf[7], mu.ms_p1, mu.ms_p2, mu.ms_p3,
               t1[7], t1[7]/tf[7], m1.ms_p1, m1.ms_p2, m1.ms_p3, m1.ms_p4);
        free(L); free(recs); free(ref);
    }
#endif
    return 0;
}
