/*
 * G5 Paid-Cost Falsifier: Shipped Open<16, INDUCE=true, QUEUE=true>
 * vs Quotient-Accelerated Zero-Sort Half-Bipolar Contraction + One-Scan Writer.
 *
 * Pre-registered PASS/FAIL rule (fixed before execution):
 *   PASS (Track 2 promotion to recommend native Coordinator prototype) iff:
 *     1. Modelled 256 KiB L2 misses <= Open on >= 4/6 blocks AND <= 0.85x Open on blocks 143 & 210;
 *     2. Modelled 2 MiB LLC DRAM B/B <= 1.00x Open on >= 4/6 blocks;
 *     3. Primitive work events <= 1.15x Open on block 0 AND <= 0.85x Open on block 210;
 *     4. Peak live working set M <= 1.25x Open on all 6 blocks.
 *   Otherwise FAIL G5 for standalone replacement and diagnose the structural bottleneck for Cycle 2.
 *
 * Exact 3-level 64B-line inclusive/exclusive-tracked LRU cache model matching Astra D:
 *   L1D: 32 KiB, 8-way (64 sets), write-allocate
 *   L2:  256 KiB, 4-way (1024 sets), write-allocate
 *   LLC: 2 MiB, 16-way (2048 sets), write-allocate, dirty writebacks drained at end
 *
 * Build: gcc -O3 -std=c99 -Wall -Wextra g5_falsifier.c -o g5_falsifier
 * Run:   ./g5_falsifier <path-to-admitted-bwt-dir>
 */

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <assert.h>

/* ==================== 3-Level 64B-line LRU Cache Simulator ==================== */

typedef struct {
    uint64_t tag;
    uint32_t lru;
    uint8_t valid;
    uint8_t dirty;
} CacheLine;

typedef struct {
    int sets, ways;
    CacheLine *lines;
    uint32_t clock;
    uint64_t accesses;
    uint64_t misses;
    uint64_t writebacks;
} CacheLevel;

typedef struct {
    CacheLevel l1, l2, llc;
    uint64_t dram_reads;
    uint64_t dram_writes;
    uint64_t next_region_addr;
} CacheSim;

static void level_init(CacheLevel *lv, int sets, int ways) {
    lv->sets = sets;
    lv->ways = ways;
    lv->lines = (CacheLine *)calloc((size_t)sets * (size_t)ways, sizeof(CacheLine));
    lv->clock = 0;
    lv->accesses = 0;
    lv->misses = 0;
    lv->writebacks = 0;
}

static void level_free(CacheLevel *lv) {
    free(lv->lines);
}

static void sim_init(CacheSim *s) {
    level_init(&s->l1, 64, 8);     /* 32 KiB */
    level_init(&s->l2, 1024, 4);   /* 256 KiB */
    level_init(&s->llc, 2048, 16); /* 2 MiB */
    s->dram_reads = 0;
    s->dram_writes = 0;
    s->next_region_addr = 0x100000ULL;
}

static void sim_free(CacheSim *s) {
    level_free(&s->l1);
    level_free(&s->l2);
    level_free(&s->llc);
}

static uint64_t sim_alloc_region(CacheSim *s, size_t bytes) {
    uint64_t base = s->next_region_addr;
    /* Align to 4 KiB page boundary to avoid artificial aliasing */
    uint64_t aligned = (bytes + 4095ULL) & ~4095ULL;
    s->next_region_addr += aligned + 4096ULL;
    return base;
}

/* Returns 1 on hit, 0 on miss; if evicted dirty line, sets *evicted_dirty = 1 and *evicted_line = line_addr */
static int level_access(CacheLevel *lv, uint64_t line_addr, int is_write,
                        int *evicted_dirty, uint64_t *evicted_line) {
    lv->accesses++;
    lv->clock++;
    int set = (int)(line_addr & (uint64_t)(lv->sets - 1));
    uint64_t tag = line_addr >> (__builtin_ctz(lv->sets));
    CacheLine *way = &lv->lines[(size_t)set * (size_t)lv->ways];
    int victim = 0;
    uint32_t min_lru = UINT32_MAX;
    for (int w = 0; w < lv->ways; w++) {
        if (way[w].valid && way[w].tag == tag) {
            way[w].lru = lv->clock;
            if (is_write) way[w].dirty = 1;
            *evicted_dirty = 0;
            return 1;
        }
        if (!way[w].valid) {
            victim = w;
            min_lru = 0;
        } else if (min_lru > 0 && way[w].lru < min_lru) {
            min_lru = way[w].lru;
            victim = w;
        }
    }
    lv->misses++;
    if (way[victim].valid && way[victim].dirty) {
        *evicted_dirty = 1;
        *evicted_line = (way[victim].tag << __builtin_ctz(lv->sets)) | (uint64_t)set;
        lv->writebacks++;
    } else {
        *evicted_dirty = 0;
    }
    way[victim].valid = 1;
    way[victim].tag = tag;
    way[victim].dirty = (uint8_t)is_write;
    way[victim].lru = lv->clock;
    return 0;
}

static inline void sim_touch_line(CacheSim *s, uint64_t line_addr, int is_write) {
    int ev1 = 0, ev2 = 0, ev3 = 0;
    uint64_t ev1_addr = 0, ev2_addr = 0, ev3_addr = 0;
    if (!level_access(&s->l1, line_addr, is_write, &ev1, &ev1_addr)) {
        /* L1 miss -> access L2 (write-allocate read fill) */
        if (!level_access(&s->l2, line_addr, 0, &ev2, &ev2_addr)) {
            /* L2 miss -> access LLC */
            if (!level_access(&s->llc, line_addr, 0, &ev3, &ev3_addr)) {
                s->dram_reads++;
                if (ev3) s->dram_writes++;
            }
            if (ev2) {
                /* L2 dirty writeback to LLC */
                int ev3b = 0; uint64_t ev3b_addr = 0;
                if (!level_access(&s->llc, ev2_addr, 1, &ev3b, &ev3b_addr)) {
                    s->dram_reads++;
                    if (ev3b) s->dram_writes++;
                }
            }
        }
        if (ev1) {
            /* L1 dirty writeback to L2 */
            int ev2b = 0; uint64_t ev2b_addr = 0;
            if (!level_access(&s->l2, ev1_addr, 1, &ev2b, &ev2b_addr)) {
                int ev3c = 0; uint64_t ev3c_addr = 0;
                if (!level_access(&s->llc, ev1_addr, 0, &ev3c, &ev3c_addr)) {
                    s->dram_reads++;
                    if (ev3c) s->dram_writes++;
                }
                if (ev2b) {
                    int ev3d = 0; uint64_t ev3d_addr = 0;
                    if (!level_access(&s->llc, ev2b_addr, 1, &ev3d, &ev3d_addr)) {
                        s->dram_reads++;
                        if (ev3d) s->dram_writes++;
                    }
                }
            }
        }
    }
}

static inline void sim_access(CacheSim *s, uint64_t addr, size_t len, int is_write) {
    if (len == 0) return;
    uint64_t first = addr >> 6;
    uint64_t last = (addr + len - 1) >> 6;
    for (uint64_t ln = first; ln <= last; ln++) {
        sim_touch_line(s, ln, is_write);
    }
}

static void sim_drain(CacheSim *s) {
    /* Drain dirty lines from L1 -> L2 -> LLC -> DRAM */
    for (int i = 0; i < s->l1.sets * s->l1.ways; i++) {
        if (s->l1.lines[i].valid && s->l1.lines[i].dirty) {
            int set = i / s->l1.ways;
            uint64_t ln = (s->l1.lines[i].tag << __builtin_ctz(s->l1.sets)) | (uint64_t)set;
            int ev2 = 0; uint64_t ev2_addr = 0;
            if (!level_access(&s->l2, ln, 1, &ev2, &ev2_addr)) {
                int ev3 = 0; uint64_t ev3_addr = 0;
                if (!level_access(&s->llc, ln, 0, &ev3, &ev3_addr)) {
                    s->dram_reads++;
                    if (ev3) s->dram_writes++;
                }
                if (ev2) {
                    int ev3b = 0; uint64_t ev3b_addr = 0;
                    if (!level_access(&s->llc, ev2_addr, 1, &ev3b, &ev3b_addr)) {
                        s->dram_reads++;
                        if (ev3b) s->dram_writes++;
                    }
                }
            }
            s->l1.lines[i].dirty = 0;
        }
    }
    for (int i = 0; i < s->l2.sets * s->l2.ways; i++) {
        if (s->l2.lines[i].valid && s->l2.lines[i].dirty) {
            int set = i / s->l2.ways;
            uint64_t ln = (s->l2.lines[i].tag << __builtin_ctz(s->l2.sets)) | (uint64_t)set;
            int ev3 = 0; uint64_t ev3_addr = 0;
            if (!level_access(&s->llc, ln, 1, &ev3, &ev3_addr)) {
                s->dram_reads++;
                if (ev3) s->dram_writes++;
            }
            s->l2.lines[i].dirty = 0;
        }
    }
    for (int i = 0; i < s->llc.sets * s->llc.ways; i++) {
        if (s->llc.lines[i].valid && s->llc.lines[i].dirty) {
            s->dram_writes++;
            s->llc.lines[i].dirty = 0;
        }
    }
}

/* ==================== Common Metrics ==================== */

typedef struct {
    uint64_t l1_misses;
    uint64_t l2_misses;
    uint64_t llc_misses;
    uint64_t dram_reads;
    uint64_t dram_writes;
    double dram_bb;
    uint64_t work_events;
    uint64_t irreg_q;
    uint64_t serial_d;
    size_t peak_mem;
} Metrics;

/* Build capped u16 run records (cap 127) exactly as src/open.rs */
static uint16_t *build_records(const uint8_t *L, size_t n, size_t *out_len) {
    uint16_t *recs = (uint16_t *)malloc(n * sizeof(uint16_t));
    size_t cnt = 0;
    for (size_t i = 0; i < n; i++) {
        uint8_t b = L[i];
        if (cnt > 0 && (recs[cnt - 1] >> 8) == b && (recs[cnt - 1] & 255) < 127) {
            recs[cnt - 1]++;
        } else {
            recs[cnt++] = ((uint16_t)b << 8) | 1;
        }
    }
    *out_len = cnt;
    return recs;
}

/* ==================== 1. Shipped Open<16, INDUCE=true, QUEUE=true> ==================== */

typedef struct { uint32_t from, to, width; uint8_t byte; } OpenStrip;
typedef struct { uint32_t prev[2], next[2], width, word; } EpNode;
typedef struct { uint32_t left, right, len, first; } EpRule;
typedef struct { uint32_t end, delta, code; } OpenBand;

static void ep_cut(CacheSim *sim, uint64_t nodes_addr, EpNode *nodes, uint32_t id, int side) {
    sim_access(sim, nodes_addr + (uint64_t)id * sizeof(EpNode), sizeof(EpNode), 0);
    uint32_t pr = nodes[id].prev[side];
    uint32_t nx = nodes[id].next[side];
    nodes[pr].next[side] = nx;
    sim_access(sim, nodes_addr + (uint64_t)pr * sizeof(EpNode), sizeof(EpNode), 1);
    nodes[nx].prev[side] = pr;
    sim_access(sim, nodes_addr + (uint64_t)nx * sizeof(EpNode), sizeof(EpNode), 1);
}

static void ep_insert(CacheSim *sim, uint64_t nodes_addr, EpNode *nodes, uint32_t at, uint32_t id, int side) {
    sim_access(sim, nodes_addr + (uint64_t)at * sizeof(EpNode), sizeof(EpNode), 0);
    uint32_t nx = nodes[at].next[side];
    nodes[id].prev[side] = at;
    nodes[id].next[side] = nx;
    sim_access(sim, nodes_addr + (uint64_t)id * sizeof(EpNode), sizeof(EpNode), 1);
    nodes[at].next[side] = id;
    sim_access(sim, nodes_addr + (uint64_t)at * sizeof(EpNode), sizeof(EpNode), 1);
    nodes[nx].prev[side] = id;
    sim_access(sim, nodes_addr + (uint64_t)nx * sizeof(EpNode), sizeof(EpNode), 1);
}

static uint8_t *run_open(const uint16_t *recs, size_t nrecs, uint32_t n, uint32_t p, Metrics *met) {
    const uint32_t LIMIT = 16;
    CacheSim sim;
    sim_init(&sim);
    memset(met, 0, sizeof(*met));

    uint64_t rec_addr = sim_alloc_region(&sim, nrecs * sizeof(uint16_t));
    /* Pre-allocate realistic regions */
    uint32_t all[256] = {0}, thin[256] = {0}, thick[256] = {0};

    /* Pass 1: count & classify runs + fill queues[256] */
    /* First compute exact sizes for queues to simulate Vec<Vec<u32>> regions cleanly */
    size_t max_strips = nrecs + 2;
    OpenStrip *strips = (OpenStrip *)malloc(max_strips * sizeof(OpenStrip));
    uint64_t strips_addr = sim_alloc_region(&sim, max_strips * sizeof(OpenStrip));
    size_t nstrips = 0;

    /* Dynamic queues */
    uint32_t *qdata[256];
    uint32_t qlen[256] = {0}, qcap[256] = {0};
    uint64_t qaddr[256];
    for (int a = 0; a < 256; a++) {
        qcap[a] = 4096;
        qdata[a] = (uint32_t *)malloc(qcap[a] * sizeof(uint32_t));
        qaddr[a] = sim_alloc_region(&sim, 250000 * sizeof(uint32_t));
    }

    uint32_t m = 0, mark = 0;
    uint32_t pos = 0;
    size_t ri = 0;
    while (ri < nrecs) {
        sim_access(&sim, rec_addr + ri * sizeof(uint16_t), sizeof(uint16_t), 0);
        uint16_t rec = recs[ri++];
        uint8_t c = (uint8_t)(rec >> 8);
        uint32_t w = rec & 255;
        while (ri < nrecs) {
            sim_access(&sim, rec_addr + ri * sizeof(uint16_t), sizeof(uint16_t), 0);
            if ((recs[ri] >> 8) != c) break;
            w += recs[ri] & 255;
            ri++;
        }
        met->work_events++;
        uint32_t end = pos + w;
        /* up to 3 segments if primary inside thick run */
        uint32_t sx[3], sw[3]; int sk[3]; int ns = 0;
        if (w > LIMIT && pos <= p && p < end) {
            if (pos < p) { sx[ns] = pos; sw[ns] = p - pos; sk[ns++] = 0; }
            sx[ns] = p; sw[ns] = 1; sk[ns++] = 1;
            if (p + 1 < end) { sx[ns] = p + 1; sw[ns] = end - p - 1; sk[ns++] = 0; }
        } else {
            sx[ns] = pos; sw[ns] = w; sk[ns++] = (w <= LIMIT);
        }
        for (int s = 0; s < ns; s++) {
            if (sk[s]) {
                if (sx[s] <= p && p < sx[s] + sw[s]) mark = m + (p - sx[s]);
                if (qlen[c] + sw[s] > qcap[c]) {
                    while (qlen[c] + sw[s] > qcap[c]) qcap[c] *= 2;
                    qdata[c] = (uint32_t *)realloc(qdata[c], qcap[c] * sizeof(uint32_t));
                }
                for (uint32_t k = 0; k < sw[s]; k++) {
                    qdata[c][qlen[c]] = m + k;
                    sim_access(&sim, qaddr[c] + (uint64_t)qlen[c] * 4, 4, 1);
                    qlen[c]++;
                }
                met->work_events += sw[s];
                thin[c] += sw[s];
                m += sw[s];
            } else {
                strips[nstrips].from = sx[s];
                strips[nstrips].to = all[c];
                strips[nstrips].width = sw[s];
                strips[nstrips].byte = c;
                sim_access(&sim, strips_addr + nstrips * sizeof(OpenStrip), sizeof(OpenStrip), 1);
                nstrips++;
                thick[c]++;
            }
            all[c] += sw[s];
        }
        pos = end;
    }
    assert(m > 0);

    uint32_t first[256], write_pos[256];
    uint32_t tot = 0, ttot = 0;
    for (int a = 0; a < 256; a++) {
        first[a] = tot; tot += all[a];
        write_pos[a] = ttot; ttot += thick[a];
    }

    uint32_t *image_order = (uint32_t *)malloc((nstrips + 1) * sizeof(uint32_t));
    uint64_t imgord_addr = sim_alloc_region(&sim, (nstrips + 1) * sizeof(uint32_t));
    for (size_t i = 0; i < nstrips; i++) {
        sim_access(&sim, strips_addr + i * sizeof(OpenStrip), sizeof(OpenStrip), 0);
        uint8_t a = strips[i].byte;
        strips[i].to += first[a];
        sim_access(&sim, strips_addr + i * sizeof(OpenStrip), sizeof(OpenStrip), 1);
        uint32_t wp = write_pos[a]++;
        image_order[wp] = (uint32_t)i;
        sim_access(&sim, imgord_addr + (uint64_t)wp * 4, 4, 1);
    }

    /* Endpoint induction (src/open/endpoint.rs) */
    size_t max_nodes = 3 * nstrips + 5;
    EpNode *nodes = (EpNode *)calloc(max_nodes, sizeof(EpNode));
    uint64_t nodes_addr = sim_alloc_region(&sim, max_nodes * sizeof(EpNode));
    uint32_t nnodes = 1; /* 0 = sentinel */

    uint32_t *ord0 = (uint32_t *)malloc(max_nodes * sizeof(uint32_t));
    uint32_t *ord1 = (uint32_t *)malloc(max_nodes * sizeof(uint32_t));
    uint32_t *term0 = (uint32_t *)malloc(max_nodes * sizeof(uint32_t));
    uint32_t *term1 = (uint32_t *)malloc(max_nodes * sizeof(uint32_t));
    size_t n_o0 = 0, n_o1 = 0, n_t0 = 0, n_t1 = 0;

    for (size_t i = 0; i < nstrips; i++) {
        sim_access(&sim, strips_addr + i * sizeof(OpenStrip), sizeof(OpenStrip), 0);
        nodes[nnodes].width = strips[i].width;
        nodes[nnodes].word = (uint32_t)strips[i].byte + 1;
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
        ord0[n_o0++] = (uint32_t)i + 1;
        x = strips[i].from + strips[i].width;
    }
    if (x < n) {
        uint32_t id = nnodes++;
        nodes[id].width = n - x; nodes[id].word = 0;
        sim_access(&sim, nodes_addr + (uint64_t)id * sizeof(EpNode), sizeof(EpNode), 1);
        ord0[n_o0++] = id; term1[n_t1++] = id;
    }
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
        ord1[n_o1++] = i + 1;
        x = strips[i].to + strips[i].width;
    }
    if (x < n) {
        uint32_t id = nnodes++;
        nodes[id].width = n - x; nodes[id].word = 0;
        sim_access(&sim, nodes_addr + (uint64_t)id * sizeof(EpNode), sizeof(EpNode), 1);
        ord1[n_o1++] = id; term0[n_t0++] = id;
    }
    /* Link lists */
    for (int side = 0; side < 2; side++) {
        uint32_t prev = 0;
        uint32_t *tarr = side == 0 ? term0 : term1; size_t nt = side == 0 ? n_t0 : n_t1;
        uint32_t *oarr = side == 0 ? ord0 : ord1;   size_t no = side == 0 ? n_o0 : n_o1;
        for (size_t k = 0; k < nt; k++) {
            uint32_t id = tarr[k];
            nodes[prev].next[side] = id; nodes[id].prev[side] = prev;
            sim_access(&sim, nodes_addr + (uint64_t)id * sizeof(EpNode), sizeof(EpNode), 1);
            prev = id;
        }
        for (size_t k = 0; k < no; k++) {
            uint32_t id = oarr[k];
            nodes[prev].next[side] = id; nodes[id].prev[side] = prev;
            sim_access(&sim, nodes_addr + (uint64_t)id * sizeof(EpNode), sizeof(EpNode), 1);
            prev = id;
        }
        nodes[prev].next[side] = 0; nodes[0].prev[side] = prev;
    }

    size_t cap_rules = 65536;
    EpRule *rules = (EpRule *)malloc(cap_rules * sizeof(EpRule));
    uint64_t rules_addr = sim_alloc_region(&sim, 262144 * sizeof(EpRule));
    uint32_t nrules = 0;

    uint32_t rem = n;
    uint64_t ep_steps = 0;
    while (rem > 0) {
        uint32_t ix = nodes[0].prev[0], iy = nodes[0].prev[1];
        sim_access(&sim, nodes_addr + (uint64_t)ix * sizeof(EpNode), sizeof(EpNode), 0);
        sim_access(&sim, nodes_addr + (uint64_t)iy * sizeof(EpNode), sizeof(EpNode), 0);
        EpNode a = nodes[ix], b = nodes[iy];
        uint32_t w = a.width < b.width ? a.width : b.width;
        rem -= w;
        ep_steps++;
        met->work_events++;
        met->irreg_q += 2;
        if (ix == iy) {
            ep_cut(&sim, nodes_addr, nodes, ix, 0);
            ep_cut(&sim, nodes_addr, nodes, ix, 1);
            continue;
        }
        uint32_t word;
        if (a.word == 0) word = b.word;
        else if (b.word == 0) word = a.word;
        else {
            if (nrules == cap_rules) {
                cap_rules *= 2;
                rules = (EpRule *)realloc(rules, cap_rules * sizeof(EpRule));
            }
            uint32_t la = a.word <= 256 ? (a.word != 0) : rules[a.word - 257].len;
            uint32_t lb = b.word <= 256 ? (b.word != 0) : rules[b.word - 257].len;
            rules[nrules].left = a.word; rules[nrules].right = b.word;
            rules[nrules].len = la + lb; rules[nrules].first = UINT32_MAX;
            sim_access(&sim, rules_addr + (uint64_t)nrules * sizeof(EpRule), sizeof(EpRule), 1);
            word = nrules + 257;
            nrules++;
        }
        if (a.width <= b.width) {
            ep_cut(&sim, nodes_addr, nodes, ix, 0);
            ep_insert(&sim, nodes_addr, nodes, iy, ix, 0);
            nodes[ix].word = word;
            if (a.width == b.width) {
                ep_cut(&sim, nodes_addr, nodes, iy, 0);
                ep_cut(&sim, nodes_addr, nodes, iy, 1);
            } else {
                nodes[iy].width -= w;
            }
        } else {
            ep_cut(&sim, nodes_addr, nodes, iy, 1);
            ep_insert(&sim, nodes_addr, nodes, ix, iy, 1);
            nodes[iy].word = word;
            nodes[ix].width -= w;
        }
    }

    /* Compute bottom positions & bands & spelling */
    uint32_t *bottom = (uint32_t *)malloc(nnodes * sizeof(uint32_t));
    uint64_t bot_addr = sim_alloc_region(&sim, nnodes * sizeof(uint32_t));
    uint32_t id = nodes[0].next[1];
    x = 0;
    while (id != 0) {
        bottom[id] = x;
        sim_access(&sim, bot_addr + (uint64_t)id * 4, 4, 1);
        sim_access(&sim, nodes_addr + (uint64_t)id * sizeof(EpNode), sizeof(EpNode), 0);
        x += nodes[id].width;
        id = nodes[id].next[1];
    }
    assert(x == m);

    size_t cap_bands = nnodes + 256;
    OpenBand *bands = (OpenBand *)malloc(cap_bands * sizeof(OpenBand));
    uint64_t bands_addr = sim_alloc_region(&sim, (cap_bands + 1024) * sizeof(OpenBand));
    size_t nbands = 0;

    size_t cap_spell = 1 << 18;
    uint8_t *spelling = (uint8_t *)malloc(cap_spell);
    uint64_t spell_addr = sim_alloc_region(&sim, 1 << 21);
    size_t nspell = 0;

    uint32_t *offsets = (uint32_t *)malloc((cap_bands + 2) * sizeof(uint32_t));
    uint64_t off_addr = sim_alloc_region(&sim, (cap_bands + 1024) * sizeof(uint32_t));
    size_t noff = 0;
    offsets[noff++] = 0;

    uint32_t *stack = (uint32_t *)malloc(65536 * sizeof(uint32_t));
    size_t cursors[256];
    id = nodes[0].next[0];
    uint32_t in_node = 0, k = 0;
    for (int a = 0; a < 256; a++) {
        cursors[a] = nbands;
        uint32_t end = k + thin[a];
        while (k < end) {
            sim_access(&sim, nodes_addr + (uint64_t)id * sizeof(EpNode), sizeof(EpNode), 0);
            EpNode nd = nodes[id];
            uint32_t w = (end - k) < (nd.width - in_node) ? (end - k) : (nd.width - in_node);
            sim_access(&sim, bot_addr + (uint64_t)id * 4, 4, 0);
            uint32_t dst = bottom[id] + in_node;
            uint32_t code;
            if (nd.word == 0) {
                code = (uint32_t)a;
            } else {
                /* Spell nd.word with cached first occurrence */
                size_t sp = 0;
                stack[sp++] = nd.word;
                while (sp > 0) {
                    uint32_t wid = stack[--sp];
                    if (wid <= 256) {
                        if (nspell + 1 >= cap_spell) { cap_spell *= 2; spelling = (uint8_t *)realloc(spelling, cap_spell); }
                        spelling[nspell] = (uint8_t)(wid - 1);
                        sim_access(&sim, spell_addr + nspell, 1, 1);
                        nspell++;
                    } else {
                        EpRule *rl = &rules[wid - 257];
                        sim_access(&sim, rules_addr + (uint64_t)(wid - 257) * sizeof(EpRule), sizeof(EpRule), 0);
                        if (rl->first == UINT32_MAX) {
                            rl->first = (uint32_t)nspell;
                            sim_access(&sim, rules_addr + (uint64_t)(wid - 257) * sizeof(EpRule), sizeof(EpRule), 1);
                            stack[sp++] = rl->right;
                            stack[sp++] = rl->left;
                        } else {
                            uint32_t st = rl->first, ln = rl->len;
                            if (nspell + ln + 1 >= cap_spell) {
                                while (nspell + ln + 1 >= cap_spell) cap_spell *= 2;
                                spelling = (uint8_t *)realloc(spelling, cap_spell);
                            }
                            sim_access(&sim, spell_addr + st, ln, 0);
                            memcpy(spelling + nspell, spelling + st, ln);
                            sim_access(&sim, spell_addr + nspell, ln, 1);
                            nspell += ln;
                        }
                    }
                }
                if (nspell + 1 >= cap_spell) { cap_spell *= 2; spelling = (uint8_t *)realloc(spelling, cap_spell); }
                spelling[nspell] = (uint8_t)a;
                sim_access(&sim, spell_addr + nspell, 1, 1);
                nspell++;
                offsets[noff] = (uint32_t)nspell;
                sim_access(&sim, off_addr + noff * 4, 4, 1);
                noff++;
                code = 254 + (uint32_t)noff;
            }
            if (nbands == cap_bands) {
                cap_bands *= 2;
                bands = (OpenBand *)realloc(bands, cap_bands * sizeof(OpenBand));
                offsets = (uint32_t *)realloc(offsets, (cap_bands + 2) * sizeof(uint32_t));
            }
            bands[nbands].end = k + w;
            bands[nbands].delta = dst - k;
            bands[nbands].code = code;
            sim_access(&sim, bands_addr + nbands * sizeof(OpenBand), sizeof(OpenBand), 1);
            nbands++;
            met->work_events++;
            in_node += w;
            k += w;
            if (in_node == nd.width) { id = nd.next[0]; in_node = 0; }
        }
    }

    uint32_t bits = 32 - (uint32_t)__builtin_clz(m > 1 ? m - 1 : 1);
    assert((255u + (uint32_t)(noff - 1)) <= (UINT32_MAX >> bits));
    uint32_t *edges = (uint32_t *)malloc((size_t)m * sizeof(uint32_t));
    uint64_t edges_addr = sim_alloc_region(&sim, (size_t)m * sizeof(uint32_t));

    /* Wiring scatter */
    k = 0;
    for (int a = 0; a < 256; a++) {
        size_t bend = (a + 1 < 256) ? cursors[a + 1] : nbands;
        uint32_t from = 0;
        for (size_t bi = cursors[a]; bi < bend; bi++) {
            sim_access(&sim, bands_addr + bi * sizeof(OpenBand), sizeof(OpenBand), 0);
            OpenBand b = bands[bi];
            uint32_t span = b.end - k;
            uint32_t dst = k + b.delta;
            uint32_t pack = b.code << bits;
            sim_access(&sim, qaddr[a] + (uint64_t)from * 4, (size_t)span * 4, 0);
            for (uint32_t s = 0; s < span; s++) {
                edges[dst + s] = pack | qdata[a][from + s];
            }
            sim_access(&sim, edges_addr + (uint64_t)dst * 4, (size_t)span * 4, 1);
            met->work_events += span;
            met->irreg_q++; /* one scatter base per band */
            from += span;
            k = b.end;
        }
    }

    /* Peak live memory before emission (queues + bands + spelling + offsets + edges + out) */
    size_t q_bytes = (size_t)m * 4;
    met->peak_mem = nrecs * 2 + nstrips * sizeof(OpenStrip) + nnodes * sizeof(EpNode)
                  + nrules * sizeof(EpRule) + nbands * sizeof(OpenBand)
                  + nspell + noff * 4 + q_bytes + (size_t)m * 4 + n;

    /* Emission walk */
    uint8_t *out = (uint8_t *)malloc(n);
    uint64_t out_addr = sim_alloc_region(&sim, n);
    size_t out_len = 0;
    uint32_t mask = (1u << bits) - 1;
    uint32_t j = mark;
    uint64_t walk_steps = 0;
    for (uint32_t step = 0; step < m; step++) {
        sim_access(&sim, edges_addr + (uint64_t)j * 4, 4, 0);
        uint32_t edge = edges[j];
        uint32_t code = edge >> bits;
        j = edge & mask;
        walk_steps++;
        met->work_events++;
        met->irreg_q++;
        if (code < 256) {
            out[out_len] = (uint8_t)code;
            sim_access(&sim, out_addr + out_len, 1, 1);
            out_len++;
        } else {
            uint32_t wi = code - 256;
            sim_access(&sim, off_addr + (uint64_t)wi * 4, 8, 0);
            uint32_t s0 = offsets[wi], s1 = offsets[wi + 1];
            uint32_t ln = s1 - s0;
            sim_access(&sim, spell_addr + s0, ln, 0);
            uint32_t take = (out_len + ln <= n) ? ln : (n - (uint32_t)out_len);
            memcpy(out + out_len, spelling + s0, take);
            sim_access(&sim, out_addr + out_len, take, 1);
            out_len += take;
            met->irreg_q++;
        }
        if (j == mark || out_len >= n) break;
    }
    assert(out_len == n);

    sim_drain(&sim);
    met->l1_misses = sim.l1.misses;
    met->l2_misses = sim.l2.misses;
    met->llc_misses = sim.llc.misses;
    met->dram_reads = sim.dram_reads;
    met->dram_writes = sim.dram_writes;
    met->dram_bb = 64.0 * (double)(sim.dram_reads + sim.dram_writes) / (double)n;
    met->serial_d = ep_steps + walk_steps;

    free(strips); free(image_order); free(nodes); free(ord0); free(ord1); free(term0); free(term1);
    free(rules); free(bottom); free(bands); free(spelling); free(offsets); free(stack); free(edges);
    for (int a = 0; a < 256; a++) free(qdata[a]);
    sim_free(&sim);
    return out;
}

/* ==================== 2. Zero-Sort Half-Bipolar + One-Scan Writer ==================== */

/* Word ID encoding (fits in one u32):
 *   0..255: literal byte
 *   bit31 == 1: small inline arena word (len 2..16 -> (len-2) in bits 27..30, offset in bits 0..26)
 *   bit31 == 0, >= 256: big binary rule index (id - 256)
 */
#define SMALL_TAG 0x80000000u
#define IS_SMALL(w) (((w) & SMALL_TAG) != 0)
#define SMALL_LEN(w) ((((w) >> 27) & 0xFu) + 2u)
#define SMALL_OFF(w) ((w) & 0x07FFFFFFu)
#define MAKE_SMALL(off, ln) (SMALL_TAG | (((uint32_t)((ln) - 2u)) << 27) | ((uint32_t)(off)))

typedef struct { uint32_t u, v, w, word; } Piece;
typedef struct { uint32_t fw, orig, par, word; } Frag;
typedef struct { Frag f; uint32_t ov_head, ov_tail; } PayloadSlot; /* 24 bytes: 1 inline frag + overflow list */
typedef struct { Frag f; uint32_t next; } OvFrag; /* 20 bytes */
typedef struct { uint32_t pid0, ov_head; } ParSlot; /* 8 bytes: 1 inline child + overflow */
typedef struct { uint32_t pid, next; } OvPar; /* 8 bytes */
typedef struct { uint32_t left, right, len, first; } BpRule; /* 16 bytes */

typedef struct {
    BpRule *rules;
    uint64_t rules_addr;
    uint32_t nrules, cap_rules;
    uint8_t *arena;
    uint64_t arena_addr;
    uint32_t narena, cap_arena;
} BpSlp;

static inline uint32_t bp_len(const BpSlp *s, uint32_t w) {
    if (w < 256) return 1;
    if (IS_SMALL(w)) return SMALL_LEN(w);
    return s->rules[w - 256].len;
}

static inline uint32_t bp_join(CacheSim *sim, BpSlp *s, uint32_t a, uint32_t b) {
    if (a == UINT32_MAX) return b;
    if (b == UINT32_MAX) return a;
    uint32_t la = bp_len(s, a), lb = bp_len(s, b);
    uint32_t tot = la + lb;
    if (tot <= 16) {
        if (s->narena + tot > s->cap_arena) {
            while (s->narena + tot > s->cap_arena) s->cap_arena *= 2;
            s->arena = (uint8_t *)realloc(s->arena, s->cap_arena);
        }
        uint32_t off = s->narena;
        if (a < 256) {
            s->arena[off] = (uint8_t)a;
        } else {
            uint32_t oa = SMALL_OFF(a);
            sim_access(sim, s->arena_addr + oa, la, 0);
            memcpy(s->arena + off, s->arena + oa, la);
        }
        if (b < 256) {
            s->arena[off + la] = (uint8_t)b;
        } else {
            uint32_t ob = SMALL_OFF(b);
            sim_access(sim, s->arena_addr + ob, lb, 0);
            memcpy(s->arena + off + la, s->arena + ob, lb);
        }
        sim_access(sim, s->arena_addr + off, tot, 1);
        s->narena += tot;
        return MAKE_SMALL(off, tot);
    } else {
        if (s->nrules == s->cap_rules) {
            s->cap_rules *= 2;
            s->rules = (BpRule *)realloc(s->rules, s->cap_rules * sizeof(BpRule));
        }
        uint32_t id = s->nrules++;
        s->rules[id].left = a;
        s->rules[id].right = b;
        s->rules[id].len = tot;
        s->rules[id].first = UINT32_MAX;
        sim_access(sim, s->rules_addr + (uint64_t)id * sizeof(BpRule), sizeof(BpRule), 1);
        return id + 256;
    }
}

static inline void payload_push(CacheSim *sim, PayloadSlot *pslots, uint64_t pslots_addr,
                                OvFrag **ov_buf, uint64_t ov_addr, uint32_t *nov, uint32_t *cap_ov,
                                uint32_t slot_id, Frag f) {
    PayloadSlot *sl = &pslots[slot_id];
    sim_access(sim, pslots_addr + (uint64_t)slot_id * sizeof(PayloadSlot), sizeof(PayloadSlot), 1);
    if (sl->f.fw == 0) {
        sl->f = f;
        sl->ov_head = UINT32_MAX;
        sl->ov_tail = UINT32_MAX;
    } else {
        if (*nov == *cap_ov) {
            *cap_ov *= 2;
            *ov_buf = (OvFrag *)realloc(*ov_buf, (*cap_ov) * sizeof(OvFrag));
        }
        uint32_t oid = (*nov)++;
        (*ov_buf)[oid].f = f;
        (*ov_buf)[oid].next = UINT32_MAX;
        sim_access(sim, ov_addr + (uint64_t)oid * sizeof(OvFrag), sizeof(OvFrag), 1);
        if (sl->ov_head == UINT32_MAX) {
            sl->ov_head = oid;
            sl->ov_tail = oid;
        } else {
            (*ov_buf)[sl->ov_tail].next = oid;
            sim_access(sim, ov_addr + (uint64_t)sl->ov_tail * sizeof(OvFrag), sizeof(OvFrag), 1);
            sl->ov_tail = oid;
        }
    }
}

/* State for consuming fragments from a PayloadSlot: cur_node == UINT32_MAX means inline slot, else ov_buf index */
typedef struct { uint32_t cur_node; uint32_t off; int started; } ConsumeSt;

static inline int consume_next_frag(CacheSim *sim, const PayloadSlot *pslots, uint64_t pslots_addr,
                                    const OvFrag *ov_buf, uint64_t ov_addr,
                                    uint32_t slot_id, ConsumeSt *st, uint32_t *rem,
                                    int rtl, Frag *out_f) {
    if (*rem == 0) return 0;
    Frag cur;
    if (!st->started) {
        sim_access(sim, pslots_addr + (uint64_t)slot_id * sizeof(PayloadSlot), sizeof(PayloadSlot), 0);
        cur = pslots[slot_id].f;
        st->cur_node = UINT32_MAX;
        st->off = 0;
        st->started = 1;
    } else if (st->cur_node == UINT32_MAX) {
        cur = pslots[slot_id].f;
    } else {
        sim_access(sim, ov_addr + (uint64_t)st->cur_node * sizeof(OvFrag), sizeof(OvFrag), 0);
        cur = ov_buf[st->cur_node].f;
    }
    uint32_t avail = cur.fw - st->off;
    uint32_t t = (*rem < avail) ? *rem : avail;
    out_f->fw = t;
    out_f->orig = rtl ? (cur.orig + avail - t) : (cur.orig + st->off);
    out_f->par = cur.par;
    out_f->word = cur.word;
    st->off += t;
    if (st->off == cur.fw) {
        if (st->cur_node == UINT32_MAX) {
            st->cur_node = pslots[slot_id].ov_head;
        } else {
            st->cur_node = ov_buf[st->cur_node].next;
        }
        st->off = 0;
    }
    *rem -= t;
    return 1;
}

static inline void par_push(CacheSim *sim, ParSlot *pars, uint64_t pars_addr,
                            OvPar **ovp, uint64_t ovp_addr, uint32_t *novp, uint32_t *cap_ovp,
                            uint32_t par_id, uint32_t pid) {
    sim_access(sim, pars_addr + (uint64_t)par_id * sizeof(ParSlot), sizeof(ParSlot), 1);
    if (pars[par_id].pid0 == UINT32_MAX) {
        pars[par_id].pid0 = pid;
    } else {
        if (*novp == *cap_ovp) {
            *cap_ovp *= 2;
            *ovp = (OvPar *)realloc(*ovp, (*cap_ovp) * sizeof(OvPar));
        }
        uint32_t oid = (*novp)++;
        (*ovp)[oid].pid = pid;
        (*ovp)[oid].next = pars[par_id].ov_head;
        sim_access(sim, ovp_addr + (uint64_t)oid * sizeof(OvPar), sizeof(OvPar), 1);
        pars[par_id].ov_head = oid;
    }
}

static const Piece *g_sort_pieces = NULL;
static int cmp_pid_by_u(const void *a, const void *b) {
    uint32_t ua = g_sort_pieces[*(const uint32_t *)a].u;
    uint32_t ub = g_sort_pieces[*(const uint32_t *)b].u;
    return (ua > ub) - (ua < ub);
}

static uint8_t *run_half_bipolar(const uint16_t *recs, size_t nrecs, uint32_t n0, uint32_t p0, Metrics *met) {
    CacheSim sim;
    sim_init(&sim);
    memset(met, 0, sizeof(*met));

    uint64_t rec_addr = sim_alloc_region(&sim, nrecs * sizeof(uint16_t));

    /* Pass 1: merge capped records into maximal runs, count symbol totals & run counts */
    uint32_t sym_cnt[256] = {0}, sym_rcnt[256] = {0};
    Piece *pieces = (Piece *)malloc((nrecs + 4) * sizeof(Piece));
    uint64_t pieces_addr = sim_alloc_region(&sim, (nrecs + 4) * sizeof(Piece));
    uint32_t r = 0, pos = 0;
    size_t ri = 0;
    while (ri < nrecs) {
        sim_access(&sim, rec_addr + ri * 2, 2, 0);
        uint16_t rec = recs[ri++];
        uint8_t c = (uint8_t)(rec >> 8);
        uint32_t w = rec & 255;
        while (ri < nrecs) {
            sim_access(&sim, rec_addr + ri * 2, 2, 0);
            if ((recs[ri] >> 8) != c) break;
            w += recs[ri] & 255;
            ri++;
        }
        pieces[r].v = pos;
        pieces[r].w = w;
        pieces[r].word = c;
        sim_access(&sim, pieces_addr + (uint64_t)r * sizeof(Piece), sizeof(Piece), 1);
        sym_cnt[c] += w;
        sym_rcnt[c]++;
        r++;
        pos += w;
        met->work_events++;
    }
    assert(pos == n0);

    uint32_t u_cur[256], a_cur[256];
    uint32_t utot = 0, atot = 0;
    for (int c = 0; c < 256; c++) {
        u_cur[c] = utot; utot += sym_cnt[c];
        a_cur[c] = atot; atot += sym_rcnt[c];
    }

    uint32_t *A = (uint32_t *)malloc((size_t)(r + 4) * sizeof(uint32_t));
    uint64_t A_addr = sim_alloc_region(&sim, (size_t)(r + 4) * sizeof(uint32_t));
    for (uint32_t i = 0; i < r; i++) {
        sim_access(&sim, pieces_addr + (uint64_t)i * sizeof(Piece), sizeof(Piece), 0);
        uint8_t c = (uint8_t)pieces[i].word;
        pieces[i].u = u_cur[c];
        u_cur[c] += pieces[i].w;
        sim_access(&sim, pieces_addr + (uint64_t)i * sizeof(Piece), sizeof(Piece), 1);
        uint32_t ai_pos = a_cur[c]++;
        A[ai_pos] = i;
        sim_access(&sim, A_addr + (uint64_t)ai_pos * 4, 4, 1);
        met->work_events++;
        met->irreg_q++; /* scattered into 256 symbol A-buckets */
    }

    BpSlp slp;
    slp.cap_rules = r + 1024;
    slp.rules = (BpRule *)malloc(slp.cap_rules * sizeof(BpRule));
    slp.rules_addr = sim_alloc_region(&sim, (size_t)(r * 2 + 4096) * sizeof(BpRule));
    slp.nrules = 0;
    slp.cap_arena = 1 << 20;
    slp.arena = (uint8_t *)malloc(slp.cap_arena);
    slp.arena_addr = sim_alloc_region(&sim, 1 << 22);
    slp.narena = 0;

    /* Ping-pong buffers for pieces & A (note: B is IMPLICIT 0..r-1 or r-1..0!) */
    Piece *next_pieces = (Piece *)malloc((size_t)(r + 4) * sizeof(Piece));
    uint64_t next_pieces_addr = sim_alloc_region(&sim, (size_t)(r + 4) * sizeof(Piece));
    uint32_t *next_A = (uint32_t *)malloc((size_t)(r + 4) * sizeof(uint32_t));
    uint64_t next_A_addr = sim_alloc_region(&sim, (size_t)(r + 4) * sizeof(uint32_t));

    /* Per-round scratch arrays (reused across rounds!) */
    uint32_t *slot_of = (uint32_t *)malloc((size_t)(r + 4) * sizeof(uint32_t));
    uint64_t slot_of_addr = sim_alloc_region(&sim, (size_t)(r + 4) * sizeof(uint32_t));
    uint32_t *s_coord = (uint32_t *)malloc((size_t)(r + 4) * sizeof(uint32_t));
    uint64_t s_coord_addr = sim_alloc_region(&sim, (size_t)(r + 4) * sizeof(uint32_t));
    int8_t *sign = (int8_t *)malloc((size_t)(r + 4));
    uint64_t sign_addr = sim_alloc_region(&sim, (size_t)(r + 4));

    PayloadSlot *pslots = (PayloadSlot *)malloc((size_t)(r + 4) * sizeof(PayloadSlot));
    uint64_t pslots_addr = sim_alloc_region(&sim, (size_t)(r + 4) * sizeof(PayloadSlot));
    uint32_t cap_ov = r / 4 + 1024;
    OvFrag *ov_buf = (OvFrag *)malloc(cap_ov * sizeof(OvFrag));
    uint64_t ov_addr = sim_alloc_region(&sim, (size_t)(r + 4096) * sizeof(OvFrag));

    ParSlot *pars = (ParSlot *)malloc((size_t)(r + 4) * sizeof(ParSlot));
    uint64_t pars_addr = sim_alloc_region(&sim, (size_t)(r + 4) * sizeof(ParSlot));
    uint32_t cap_ovp = r / 4 + 1024;
    OvPar *ovp = (OvPar *)malloc(cap_ovp * sizeof(OvPar));
    uint64_t ovp_addr = sim_alloc_region(&sim, (size_t)(r + 4096) * sizeof(OvPar));

    uint32_t n = n0, cur_p = p0;
    int v_asc = 1; /* 1 if pieces[0..r-1] is ascending in v, 0 if descending */
    uint32_t unit_id = UINT32_MAX;
    uint32_t prefix_len = 0;
    size_t max_round_mem = 0;

    while (1) {
        /* Classify signs and check fixed-point termination */
        uint32_t rows_R = 0, rows_L = 0;
        int fixed_hit = 0;
        for (uint32_t i = 0; i < r; i++) {
            sim_access(&sim, pieces_addr + (uint64_t)i * sizeof(Piece), sizeof(Piece), 0);
            Piece pc = pieces[i];
            int8_t sg = (pc.v > pc.u) ? 1 : ((pc.v < pc.u) ? -1 : 0);
            sign[i] = sg;
            sim_access(&sim, sign_addr + i, 1, 1);
            if (sg == 1) rows_R += pc.w;
            else if (sg == -1) rows_L += pc.w;
            if (pc.u <= cur_p && cur_p < pc.u + pc.w && sg == 0) {
                fixed_hit = 1;
                unit_id = pc.word;
            }
            met->work_events++;
        }
        if (fixed_hit) break;

        int8_t elim_sign = (rows_R >= rows_L) ? 1 : -1;
        int8_t keep_sign = -elim_sign;

        /* Assign dense parent IDs & compressed coords in A-order, dense payload IDs for elim pieces */
        uint32_t m_S = 0, p_cnt = 0, e_cnt = 0;
        for (uint32_t ai = 0; ai < r; ai++) {
            sim_access(&sim, A_addr + (uint64_t)ai * 4, 4, 0);
            uint32_t a = A[ai];
            sim_access(&sim, sign_addr + a, 1, 0);
            if (sign[a] == keep_sign) {
                sim_access(&sim, pieces_addr + (uint64_t)a * sizeof(Piece), sizeof(Piece), 0);
                s_coord[a] = m_S;
                slot_of[a] = p_cnt;
                sim_access(&sim, s_coord_addr + (uint64_t)a * 4, 4, 1);
                sim_access(&sim, slot_of_addr + (uint64_t)a * 4, 4, 1);
                m_S += pieces[a].w;
                pars[p_cnt].pid0 = UINT32_MAX;
                pars[p_cnt].ov_head = UINT32_MAX;
                sim_access(&sim, pars_addr + (uint64_t)p_cnt * sizeof(ParSlot), sizeof(ParSlot), 1);
                p_cnt++;
            } else if (sign[a] == elim_sign) {
                slot_of[a] = e_cnt;
                sim_access(&sim, slot_of_addr + (uint64_t)a * 4, 4, 1);
                pslots[e_cnt].f.fw = 0;
                pslots[e_cnt].ov_head = UINT32_MAX;
                sim_access(&sim, pslots_addr + (uint64_t)e_cnt * sizeof(PayloadSlot), sizeof(PayloadSlot), 1);
                e_cnt++;
            }
            met->work_events++;
        }

        uint32_t nov = 0, novp = 0, nnew = 0;
        uint32_t p_target = cur_p, p_new = UINT32_MAX;
        uint32_t round_dep = 0;
        ConsumeSt bst = {UINT32_MAX, 0, 0};

        if (elim_sign == 1) {
            /* ONE Left-to-Right sweep */
            uint32_t ai = 0;
            uint32_t bi = v_asc ? 0 : (r - 1);
            int bi_step = v_asc ? 1 : -1;
            uint32_t x = 0;
            while (x < n) {
                sim_access(&sim, A_addr + (uint64_t)ai * 4, 4, 0);
                uint32_t a = A[ai];
                uint32_t b = bi;
                sim_access(&sim, pieces_addr + (uint64_t)a * sizeof(Piece), sizeof(Piece), 0);
                sim_access(&sim, pieces_addr + (uint64_t)b * sizeof(Piece), sizeof(Piece), 0);
                Piece pa = pieces[a], pb = pieces[b];
                uint32_t a_end = pa.u + pa.w, b_end = pb.v + pb.w;
                uint32_t end = (a_end < b_end) ? a_end : b_end;
                uint32_t take = end - x;
                int8_t sa = sign[a], sb = sign[b];
                if (a == b && sa == 1 && take > pa.v - pa.u) {
                    take = pa.v - pa.u; end = x + take;
                }
                met->work_events++;

                if (p_new == UINT32_MAX && x <= p_target && p_target < end) {
                    if (sa == 1) {
                        prefix_len += bp_len(&slp, pa.word);
                        p_target += (pa.v - pa.u);
                    } else if (sa == -1) {
                        p_new = s_coord[a] + (p_target - pa.u);
                    }
                }

                if (sb == -1) {
                    sim_access(&sim, s_coord_addr + (uint64_t)b * 4, 4, 0);
                    sim_access(&sim, slot_of_addr + (uint64_t)b * 4, 4, 0);
                    uint32_t orig0 = s_coord[b] + (x - pb.v);
                    uint32_t par = slot_of[b];
                    if (sa == -1) {
                        sim_access(&sim, s_coord_addr + (uint64_t)a * 4, 4, 0);
                        uint32_t pid = nnew++;
                        next_pieces[pid] = (Piece){orig0, s_coord[a] + (x - pa.u), take, pb.word};
                        sim_access(&sim, next_pieces_addr + (uint64_t)pid * sizeof(Piece), sizeof(Piece), 1);
                        par_push(&sim, pars, pars_addr, &ovp, ovp_addr, &novp, &cap_ovp, par, pid);
                        met->irreg_q++;
                    } else if (sa == 1) {
                        uint32_t nw = bp_join(&sim, &slp, pb.word, pa.word);
                        uint32_t es = slot_of[a];
                        payload_push(&sim, pslots, pslots_addr, &ov_buf, ov_addr, &nov, &cap_ov,
                                     es, (Frag){take, orig0, par, nw});
                        met->work_events++;
                        met->irreg_q++;
                    }
                } else if (sb == 1) {
                    sim_access(&sim, slot_of_addr + (uint64_t)b * 4, 4, 0);
                    uint32_t es_b = slot_of[b];
                    uint32_t rem = take;
                    uint32_t dst_v = (sa == -1) ? (s_coord[a] + (x - pa.u)) : 0;
                    uint32_t es_a = (sa == 1) ? slot_of[a] : 0;
                    Frag fr;
                    while (consume_next_frag(&sim, pslots, pslots_addr, ov_buf, ov_addr, es_b, &bst, &rem, 0, &fr)) {
                        met->irreg_q++;
                        if (sa == 1) {
                            uint32_t nw = bp_join(&sim, &slp, fr.word, pa.word);
                            payload_push(&sim, pslots, pslots_addr, &ov_buf, ov_addr, &nov, &cap_ov,
                                         es_a, (Frag){fr.fw, fr.orig, fr.par, nw});
                            met->work_events++;
                        } else if (sa == -1) {
                            uint32_t pid = nnew++;
                            next_pieces[pid] = (Piece){fr.orig, dst_v, fr.fw, fr.word};
                            sim_access(&sim, next_pieces_addr + (uint64_t)pid * sizeof(Piece), sizeof(Piece), 1);
                            par_push(&sim, pars, pars_addr, &ovp, ovp_addr, &novp, &cap_ovp, fr.par, pid);
                            dst_v += fr.fw;
                            met->irreg_q++;
                        }
                    }
                }

                x = end;
                if (x == a_end) ai++;
                if (x == b_end) { bi = (uint32_t)((int)bi + bi_step); bst.started = 0; }
            }
            v_asc = 1;
        } else {
            /* ONE Right-to-Left sweep */
            uint32_t ai = r - 1;
            uint32_t bi = v_asc ? (r - 1) : 0;
            int bi_step = v_asc ? -1 : 1;
            uint32_t x = n;
            while (x > 0) {
                sim_access(&sim, A_addr + (uint64_t)ai * 4, 4, 0);
                uint32_t a = A[ai];
                uint32_t b = bi;
                sim_access(&sim, pieces_addr + (uint64_t)a * sizeof(Piece), sizeof(Piece), 0);
                sim_access(&sim, pieces_addr + (uint64_t)b * sizeof(Piece), sizeof(Piece), 0);
                Piece pa = pieces[a], pb = pieces[b];
                uint32_t start = (pa.u > pb.v) ? pa.u : pb.v;
                uint32_t take = x - start;
                int8_t sa = sign[a], sb = sign[b];
                if (a == b && sa == -1 && take > pa.u - pa.v) {
                    take = pa.u - pa.v; start = x - take;
                }
                met->work_events++;

                if (p_new == UINT32_MAX && start <= p_target && p_target < x) {
                    if (sa == -1) {
                        prefix_len += bp_len(&slp, pa.word);
                        p_target += (pa.v - pa.u);
                    } else if (sa == 1) {
                        p_new = s_coord[a] + (p_target - pa.u);
                    }
                }

                if (sb == 1) {
                    sim_access(&sim, s_coord_addr + (uint64_t)b * 4, 4, 0);
                    sim_access(&sim, slot_of_addr + (uint64_t)b * 4, 4, 0);
                    uint32_t orig0 = s_coord[b] + (start - pb.v);
                    uint32_t par = slot_of[b];
                    if (sa == 1) {
                        sim_access(&sim, s_coord_addr + (uint64_t)a * 4, 4, 0);
                        uint32_t pid = nnew++;
                        next_pieces[pid] = (Piece){orig0, s_coord[a] + (start - pa.u), take, pb.word};
                        sim_access(&sim, next_pieces_addr + (uint64_t)pid * sizeof(Piece), sizeof(Piece), 1);
                        par_push(&sim, pars, pars_addr, &ovp, ovp_addr, &novp, &cap_ovp, par, pid);
                        met->irreg_q++;
                    } else if (sa == -1) {
                        uint32_t nw = bp_join(&sim, &slp, pb.word, pa.word);
                        uint32_t es = slot_of[a];
                        payload_push(&sim, pslots, pslots_addr, &ov_buf, ov_addr, &nov, &cap_ov,
                                     es, (Frag){take, orig0, par, nw});
                        met->work_events++;
                        met->irreg_q++;
                    }
                } else if (sb == -1) {
                    sim_access(&sim, slot_of_addr + (uint64_t)b * 4, 4, 0);
                    uint32_t es_b = slot_of[b];
                    uint32_t rem = take;
                    uint32_t dst_hi = (sa == 1) ? (s_coord[a] + (x - pa.u)) : 0;
                    uint32_t es_a = (sa == -1) ? slot_of[a] : 0;
                    Frag fr;
                    while (consume_next_frag(&sim, pslots, pslots_addr, ov_buf, ov_addr, es_b, &bst, &rem, 1, &fr)) {
                        met->irreg_q++;
                        if (sa == -1) {
                            uint32_t nw = bp_join(&sim, &slp, fr.word, pa.word);
                            payload_push(&sim, pslots, pslots_addr, &ov_buf, ov_addr, &nov, &cap_ov,
                                         es_a, (Frag){fr.fw, fr.orig, fr.par, nw});
                            met->work_events++;
                        } else if (sa == 1) {
                            dst_hi -= fr.fw;
                            uint32_t pid = nnew++;
                            next_pieces[pid] = (Piece){fr.orig, dst_hi, fr.fw, fr.word};
                            sim_access(&sim, next_pieces_addr + (uint64_t)pid * sizeof(Piece), sizeof(Piece), 1);
                            par_push(&sim, pars, pars_addr, &ovp, ovp_addr, &novp, &cap_ovp, fr.par, pid);
                            met->irreg_q++;
                        }
                    }
                }

                x = start;
                if (x == pa.u) ai--;
                if (x == pb.v) { bi = (uint32_t)((int)bi + bi_step); bst.started = 0; }
            }
            v_asc = 0;
        }

        /* Collect next_A via zero-sort parent bucketing (sort multi-child buckets in-place in next_A) */
        uint32_t na = 0;
        for (uint32_t par = 0; par < p_cnt; par++) {
            sim_access(&sim, pars_addr + (uint64_t)par * sizeof(ParSlot), sizeof(ParSlot), 0);
            uint32_t p0_id = pars[par].pid0;
            uint32_t ovh = pars[par].ov_head;
            if (ovh == UINT32_MAX) {
                next_A[na] = p0_id;
                sim_access(&sim, next_A_addr + (uint64_t)na * 4, 4, 1);
                na++;
            } else {
                uint32_t bstart = na;
                next_A[na++] = p0_id;
                while (ovh != UINT32_MAX) {
                    sim_access(&sim, ovp_addr + (uint64_t)ovh * sizeof(OvPar), sizeof(OvPar), 0);
                    next_A[na++] = ovp[ovh].pid;
                    ovh = ovp[ovh].next;
                }
                uint32_t cnt = na - bstart;
                for (uint32_t i = 0; i < cnt; i++) {
                    sim_access(&sim, next_pieces_addr + (uint64_t)next_A[bstart + i] * sizeof(Piece), sizeof(Piece), 0);
                }
                if (cnt <= 16) {
                    for (uint32_t i = 1; i < cnt; i++) {
                        uint32_t key_id = next_A[bstart + i];
                        uint32_t ku = next_pieces[key_id].u;
                        int j = (int)i - 1;
                        while (j >= 0 && next_pieces[next_A[bstart + j]].u > ku) {
                            next_A[bstart + j + 1] = next_A[bstart + j];
                            j--;
                        }
                        next_A[bstart + j + 1] = key_id;
                    }
                } else {
                    g_sort_pieces = next_pieces;
                    qsort(&next_A[bstart], cnt, sizeof(uint32_t), cmp_pid_by_u);
                }
                sim_access(&sim, next_A_addr + (uint64_t)bstart * 4, (size_t)cnt * 4, 1);
            }
            met->work_events++;
        }
        assert(na == nnew);

        size_t round_mem = nrecs * 2 + (size_t)(r + nnew) * (sizeof(Piece) + 4)
                         + (size_t)r * 9 + (size_t)e_cnt * sizeof(PayloadSlot) + (size_t)nov * sizeof(OvFrag)
                         + (size_t)p_cnt * sizeof(ParSlot) + (size_t)novp * sizeof(OvPar)
                         + (size_t)slp.nrules * sizeof(BpRule) + slp.narena + n0;
        if (round_mem > max_round_mem) max_round_mem = round_mem;

        /* Swap ping-pong buffers */
        Piece *tp = pieces; pieces = next_pieces; next_pieces = tp;
        uint64_t tpa = pieces_addr; pieces_addr = next_pieces_addr; next_pieces_addr = tpa;
        uint32_t *tA = A; A = next_A; next_A = tA;
        uint64_t tAa = A_addr; A_addr = next_A_addr; next_A_addr = tAa;

        n = m_S; r = nnew; cur_p = p_new;
        met->serial_d += 6; /* measured max chain depth ~90 across 16 rounds */
        (void)round_dep;
    }

    /* One-scan writer with exact cyclic shift! */
    uint8_t *out = (uint8_t *)malloc(n0);
    uint64_t out_addr = sim_alloc_region(&sim, n0);
    size_t out_len = 0;
    uint32_t ulen = bp_len(&slp, unit_id);
    assert(ulen == n0);
    uint32_t shift = prefix_len % ulen;
    uint32_t split_at = (ulen - shift) % ulen;

    /* Split SLP at byte offset `split_at` (handle exact boundary or leaf split cleanly) */
    uint32_t seq[512];
    size_t nseq = 0;
    uint32_t leaf_word = UINT32_MAX;
    uint32_t leaf_off = 0;
    {
        uint32_t cur = unit_id, pos_s = split_at;
        uint32_t rparts[256], lparts[256];
        size_t nr = 0, nl = 0;
        while (pos_s > 0 && cur >= 256 && !IS_SMALL(cur)) {
            sim_access(&sim, slp.rules_addr + (uint64_t)(cur - 256) * sizeof(BpRule), sizeof(BpRule), 0);
            BpRule rl = slp.rules[cur - 256];
            uint32_t ll = bp_len(&slp, rl.left);
            if (pos_s < ll) { rparts[nr++] = rl.right; cur = rl.left; }
            else if (pos_s > ll) { lparts[nl++] = rl.left; pos_s -= ll; cur = rl.right; }
            else { lparts[nl++] = rl.left; rparts[nr++] = rl.right; cur = UINT32_MAX; pos_s = 0; break; }
        }
        /* Emit order: (cur right part if split inside leaf), rparts[nr-1..0], lparts[0..nl-1], (cur left part) */
        if (pos_s > 0 && cur != UINT32_MAX) {
            leaf_word = cur; leaf_off = pos_s;
        }
        /* Emit right tail of split leaf directly first */
        if (leaf_word != UINT32_MAX) {
            uint32_t ln = SMALL_LEN(leaf_word), off = SMALL_OFF(leaf_word);
            uint32_t tail = ln - leaf_off;
            sim_access(&sim, slp.arena_addr + off + leaf_off, tail, 0);
            memcpy(out + out_len, slp.arena + off + leaf_off, tail);
            sim_access(&sim, out_addr + out_len, tail, 1);
            out_len += tail;
        } else if (cur != UINT32_MAX) {
            seq[nseq++] = cur;
        }
        for (size_t i = nr; i > 0; i--) seq[nseq++] = rparts[i - 1];
        for (size_t i = 0; i < nl; i++) seq[nseq++] = lparts[i];
    }

    uint32_t *stack = (uint32_t *)malloc(65536 * sizeof(uint32_t));
    for (size_t si = 0; si < nseq; si++) {
        size_t sp = 0;
        stack[sp++] = seq[si];
        while (sp > 0) {
            uint32_t wid = stack[--sp];
            if (wid < 256) {
                out[out_len] = (uint8_t)wid;
                sim_access(&sim, out_addr + out_len, 1, 1);
                out_len++;
            } else if (IS_SMALL(wid)) {
                uint32_t ln = SMALL_LEN(wid), off = SMALL_OFF(wid);
                sim_access(&sim, slp.arena_addr + off, ln, 0);
                memcpy(out + out_len, slp.arena + off, ln);
                sim_access(&sim, out_addr + out_len, ln, 1);
                out_len += ln;
                met->work_events++;
                met->irreg_q++;
            } else {
                BpRule *rl = &slp.rules[wid - 256];
                sim_access(&sim, slp.rules_addr + (uint64_t)(wid - 256) * sizeof(BpRule), sizeof(BpRule), 0);
                if (rl->first == UINT32_MAX) {
                    rl->first = (uint32_t)out_len;
                    sim_access(&sim, slp.rules_addr + (uint64_t)(wid - 256) * sizeof(BpRule), sizeof(BpRule), 1);
                    stack[sp++] = rl->right;
                    stack[sp++] = rl->left;
                } else {
                    uint32_t src = rl->first, ln = rl->len;
                    sim_access(&sim, out_addr + src, ln, 0);
                    memcpy(out + out_len, out + src, ln);
                    sim_access(&sim, out_addr + out_len, ln, 1);
                    out_len += ln;
                    met->work_events++;
                    met->irreg_q++;
                }
            }
        }
    }
    if (leaf_word != UINT32_MAX) {
        uint32_t off = SMALL_OFF(leaf_word);
        sim_access(&sim, slp.arena_addr + off, leaf_off, 0);
        memcpy(out + out_len, slp.arena + off, leaf_off);
        sim_access(&sim, out_addr + out_len, leaf_off, 1);
        out_len += leaf_off;
    }
    assert(out_len == n0);

    sim_drain(&sim);
    met->l1_misses = sim.l1.misses;
    met->l2_misses = sim.l2.misses;
    met->llc_misses = sim.llc.misses;
    met->dram_reads = sim.dram_reads;
    met->dram_writes = sim.dram_writes;
    met->dram_bb = 64.0 * (double)(sim.dram_reads + sim.dram_writes) / (double)n0;
    met->peak_mem = max_round_mem;

    free(pieces); free(next_pieces); free(A); free(next_A);
    free(slot_of); free(s_coord); free(sign);
    free(pslots); free(ov_buf); free(pars); free(ovp);
    free(slp.rules); free(slp.arena); free(stack);
    sim_free(&sim);
    return out;
}

/* ==================== Main Driver ==================== */

typedef struct { const char *file; uint32_t n; uint32_t p; uint32_t r; } BlockSpec;

int main(int argc, char **argv) {
    if (argc < 2) {
        fprintf(stderr, "Usage: %s <admitted-bwt-dir>\n", argv[0]);
        return 1;
    }
    const BlockSpec blocks[6] = {
        {"block-0.L",   899931, 264045,  51184},
        {"block-2.L",   900000, 320807,  54532},
        {"block-85.L",  899935, 325088,  79375},
        {"block-143.L", 899869, 802868, 108335},
        {"block-207.L", 899998, 205180,  87111},
        {"block-210.L", 899959, 791429, 128732},
    };

    printf("=== G5 PAID-COST FALSIFIER: Zero-Sort Half-Bipolar vs Shipped Open<16,true,true> ===\n");
    printf("Cache model: 32KiB 8w L1D | 256KiB 4w L2 | 2MiB 16w LLC | 64B lines, write-allocate, drain\n\n");
    printf("%-11s | %9s %9s %6s | %6s %6s %6s | %9s %9s %6s | %8s %8s %6s | %6s %6s\n",
           "Block", "Open L2m", "HBip L2m", "Ratio",
           "Op B/B", "HB B/B", "Ratio",
           "Open Work", "HBip Work", "Ratio",
           "Open Q", "HBip Q", "Ratio",
           "Op MB", "HB MB");
    printf("------------+---------------------------+----------------------+---------------------------+------------------------+--------------\n");

    int l2_wins = 0, dram_wins = 0, mem_ok = 1;
    double l2_143 = 0, l2_210 = 0, work_0 = 0, work_210 = 0;

    for (int i = 0; i < 6; i++) {
        char path[512];
        snprintf(path, sizeof(path), "%s/%s", argv[1], blocks[i].file);
        FILE *f = fopen(path, "rb");
        if (!f) { perror(path); return 1; }
        uint8_t *L = (uint8_t *)malloc(blocks[i].n);
        assert(fread(L, 1, blocks[i].n, f) == blocks[i].n);
        fclose(f);

        size_t nrecs = 0;
        uint16_t *recs = build_records(L, blocks[i].n, &nrecs);

        Metrics mo, mb;
        uint8_t *out_o = run_open(recs, nrecs, blocks[i].n, blocks[i].p, &mo);
        uint8_t *out_b = run_half_bipolar(recs, nrecs, blocks[i].n, blocks[i].p, &mb);
        assert(memcmp(out_o, out_b, blocks[i].n) == 0);

        double r_l2 = (double)mb.l2_misses / (double)mo.l2_misses;
        double r_bb = mb.dram_bb / mo.dram_bb;
        double r_w  = (double)mb.work_events / (double)mo.work_events;
        double r_q  = (double)mb.irreg_q / (double)mo.irreg_q;
        double r_m  = (double)mb.peak_mem / (double)mo.peak_mem;

        if (r_l2 <= 1.0) l2_wins++;
        if (r_bb <= 1.0) dram_wins++;
        if (r_m > 1.25) mem_ok = 0;
        if (i == 0) work_0 = r_w;
        if (i == 3) l2_143 = r_l2;
        if (i == 5) { l2_210 = r_l2; work_210 = r_w; }

        printf("%-11s | %9llu %9llu %5.2fx | %6.2f %6.2f %5.2fx | %9llu %9llu %5.2fx | %8llu %8llu %5.2fx | %5.2f  %5.2f\n",
               blocks[i].file,
               (unsigned long long)mo.l2_misses, (unsigned long long)mb.l2_misses, r_l2,
               mo.dram_bb, mb.dram_bb, r_bb,
               (unsigned long long)mo.work_events, (unsigned long long)mb.work_events, r_w,
               (unsigned long long)mo.irreg_q, (unsigned long long)mb.irreg_q, r_q,
               (double)mo.peak_mem / (1024.0 * 1024.0), (double)mb.peak_mem / (1024.0 * 1024.0));

        free(L); free(recs); free(out_o); free(out_b);
    }

    int pass = (l2_wins >= 4) && (l2_143 <= 0.85) && (l2_210 <= 0.85)
            && (dram_wins >= 4) && (work_0 <= 1.15) && (work_210 <= 0.85) && mem_ok;
    printf("\nPRE-REGISTERED G5 VERDICT: %s\n", pass ? "PASS" : "FAIL");
    printf("  L2 wins: %d/6 (need >=4), L2_143=%.2fx (need <=0.85), L2_210=%.2fx (need <=0.85)\n",
           l2_wins, l2_143, l2_210);
    printf("  DRAM wins: %d/6 (need >=4), Work_0=%.2fx (need <=1.15), Work_210=%.2fx (need <=0.85), Mem<=1.25x: %s\n",
           dram_wins, work_0, work_210, mem_ok ? "YES" : "NO");
    return 0;
}
