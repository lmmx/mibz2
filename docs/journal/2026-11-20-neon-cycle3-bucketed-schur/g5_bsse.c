/*
 * Cycle 3 · G5 Paid-Cost Falsifier & Native VM Timing Diagnostic:
 *   1. Shipped Forward Open<16, INDUCE=true, QUEUE=true> (baseline)
 *   2. Queue-Free Backward Open (Cycle 2 candidate, for direct comparison)
 *   3. Bucketed Sequential Schur Elimination + L2-Slabbed Emission (BSSE)
 *
 * Pre-registered PASS/FAIL rule (fixed before compilation/execution):
 *   For BSSE (bsize = 16384, n_slabs = 8) vs Shipped Forward Open, PASS G5 iff on ALL 6 blocks:
 *     1. Byte-exact output on all 5,399,692 bytes;
 *     2. Total modelled 256 KiB L2 misses <= 0.85x Shipped Open;
 *     3. Total modelled 2 MiB LLC DRAM B/B <= 1.00x Shipped Open;
 *     4. Total modelled work events <= 1.40x Shipped Open;
 *     5. Peak live working set M <= 1.25x Shipped Open.
 *   Otherwise FAIL G5 (and report full phase breakdown + diagnostic variants).
 *
 * Exact 3-level 64B-line LRU cache model matching Cycles 1-2 / Astra D:
 *   L1D: 32 KiB 8-way | L2: 256 KiB 4-way | LLC: 2 MiB 16-way | write-allocate + drain
 *
 * Build (cache sim): gcc -O3 -std=c99 -Wall -Wextra g5_bsse.c -o g5_bsse
 * Build (VM timing): gcc -O3 -std=c99 -Wall -Wextra -DNO_CACHE_SIM g5_bsse.c -o g5_bsse_bench
 * Run:               ./g5_bsse <path-to-admitted-bwt-dir>
 */

#define _POSIX_C_SOURCE 200809L
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <assert.h>
#include <time.h>

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
    uint64_t accesses, misses, writebacks;
} CacheLevel;

typedef struct {
    CacheLevel l1, l2, llc;
    uint64_t dram_reads, dram_writes;
    uint64_t next_region_addr;
} CacheSim;

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
static void sim_free(CacheSim *s) {
    level_free(&s->l1); level_free(&s->l2); level_free(&s->llc);
}
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
    uint64_t ev1_addr = 0, ev2_addr = 0, ev3_addr = 0;
    if (!level_access(&s->l1, line_addr, is_write, &ev1, &ev1_addr)) {
        if (!level_access(&s->l2, line_addr, 0, &ev2, &ev2_addr)) {
            if (!level_access(&s->llc, line_addr, 0, &ev3, &ev3_addr)) {
                s->dram_reads++; if (ev3) s->dram_writes++;
            }
            if (ev2) {
                int ev3b = 0; uint64_t ev3b_addr = 0;
                if (!level_access(&s->llc, ev2_addr, 1, &ev3b, &ev3b_addr)) {
                    s->dram_reads++; if (ev3b) s->dram_writes++;
                }
            }
        }
        if (ev1) {
            int ev2b = 0; uint64_t ev2b_addr = 0;
            if (!level_access(&s->l2, ev1_addr, 1, &ev2b, &ev2b_addr)) {
                int ev3c = 0; uint64_t ev3c_addr = 0;
                if (!level_access(&s->llc, ev1_addr, 0, &ev3c, &ev3c_addr)) {
                    s->dram_reads++; if (ev3c) s->dram_writes++;
                }
                if (ev2b) {
                    int ev3d = 0; uint64_t ev3d_addr = 0;
                    if (!level_access(&s->llc, ev2b_addr, 1, &ev3d, &ev3d_addr)) {
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
    uint64_t dram_bytes;
    uint64_t work_events;
    size_t peak_mem;
    uint64_t l2_p1, l2_ep, l2_p2, l2_p3, l2_p4, l2_p5;
    uint32_t m_retained, nbands;
    uint64_t routed_edges, self_hops;
} Metrics;

/* ==================== Capped Run Records ==================== */

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

/* ==================== Shared Open Endpoint Elimination & Bands ==================== */

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

static void open_endpoint_and_bands(
    CacheSim *sim, Metrics *met,
    OpenStrip *strips, uint64_t strips_addr, size_t nstrips,
    const uint32_t all[256], const uint32_t thin[256], const uint32_t thick[256],
    uint32_t n, uint32_t m,
    OpenBand **out_bands, uint64_t *out_bands_addr, size_t *out_nbands,
    uint8_t **out_spell, uint64_t *out_spell_addr, size_t *out_nspell,
    uint32_t **out_off, uint64_t *out_off_addr, size_t *out_noff,
    size_t *out_ep_mem
) {
    uint32_t first[256], write_pos[256];
    uint32_t tot = 0, ttot = 0;
    for (int a = 0; a < 256; a++) {
        first[a] = tot; tot += all[a];
        write_pos[a] = ttot; ttot += thick[a];
    }
    uint32_t *image_order = (uint32_t *)malloc((nstrips + 1) * sizeof(uint32_t));
    uint64_t imgord_addr = sim_alloc_region(sim, (nstrips + 1) * sizeof(uint32_t));
    for (size_t i = 0; i < nstrips; i++) {
        sim_access(sim, strips_addr + i * sizeof(OpenStrip), sizeof(OpenStrip), 0);
        uint8_t a = strips[i].byte;
        strips[i].to += first[a];
        sim_access(sim, strips_addr + i * sizeof(OpenStrip), sizeof(OpenStrip), 1);
        uint32_t wp = write_pos[a]++;
        image_order[wp] = (uint32_t)i;
        sim_access(sim, imgord_addr + (uint64_t)wp * 4, 4, 1);
    }

    size_t max_nodes = 3 * nstrips + 5;
    EpNode *nodes = (EpNode *)calloc(max_nodes, sizeof(EpNode));
    uint64_t nodes_addr = sim_alloc_region(sim, max_nodes * sizeof(EpNode));
    uint32_t nnodes = 1;

    uint32_t *ord0 = (uint32_t *)malloc(max_nodes * sizeof(uint32_t));
    uint32_t *ord1 = (uint32_t *)malloc(max_nodes * sizeof(uint32_t));
    uint32_t *term0 = (uint32_t *)malloc(max_nodes * sizeof(uint32_t));
    uint32_t *term1 = (uint32_t *)malloc(max_nodes * sizeof(uint32_t));
    size_t n_o0 = 0, n_o1 = 0, n_t0 = 0, n_t1 = 0;

    for (size_t i = 0; i < nstrips; i++) {
        sim_access(sim, strips_addr + i * sizeof(OpenStrip), sizeof(OpenStrip), 0);
        nodes[nnodes].width = strips[i].width;
        nodes[nnodes].word = (uint32_t)strips[i].byte + 1;
        sim_access(sim, nodes_addr + (uint64_t)nnodes * sizeof(EpNode), sizeof(EpNode), 1);
        nnodes++;
    }
    uint32_t x = 0;
    for (size_t i = 0; i < nstrips; i++) {
        sim_access(sim, strips_addr + i * sizeof(OpenStrip), sizeof(OpenStrip), 0);
        if (x < strips[i].from) {
            uint32_t id = nnodes++;
            nodes[id].width = strips[i].from - x; nodes[id].word = 0;
            sim_access(sim, nodes_addr + (uint64_t)id * sizeof(EpNode), sizeof(EpNode), 1);
            ord0[n_o0++] = id; term1[n_t1++] = id;
        }
        ord0[n_o0++] = (uint32_t)i + 1;
        x = strips[i].from + strips[i].width;
    }
    if (x < n) {
        uint32_t id = nnodes++;
        nodes[id].width = n - x; nodes[id].word = 0;
        sim_access(sim, nodes_addr + (uint64_t)id * sizeof(EpNode), sizeof(EpNode), 1);
        ord0[n_o0++] = id; term1[n_t1++] = id;
    }
    x = 0;
    for (size_t k = 0; k < nstrips; k++) {
        sim_access(sim, imgord_addr + k * 4, 4, 0);
        uint32_t i = image_order[k];
        sim_access(sim, strips_addr + (uint64_t)i * sizeof(OpenStrip), sizeof(OpenStrip), 0);
        if (x < strips[i].to) {
            uint32_t id = nnodes++;
            nodes[id].width = strips[i].to - x; nodes[id].word = 0;
            sim_access(sim, nodes_addr + (uint64_t)id * sizeof(EpNode), sizeof(EpNode), 1);
            ord1[n_o1++] = id; term0[n_t0++] = id;
        }
        ord1[n_o1++] = i + 1;
        x = strips[i].to + strips[i].width;
    }
    if (x < n) {
        uint32_t id = nnodes++;
        nodes[id].width = n - x; nodes[id].word = 0;
        sim_access(sim, nodes_addr + (uint64_t)id * sizeof(EpNode), sizeof(EpNode), 1);
        ord1[n_o1++] = id; term0[n_t0++] = id;
    }
    for (int side = 0; side < 2; side++) {
        uint32_t prev = 0;
        uint32_t *tarr = side == 0 ? term0 : term1; size_t nt = side == 0 ? n_t0 : n_t1;
        uint32_t *oarr = side == 0 ? ord0 : ord1;   size_t no = side == 0 ? n_o0 : n_o1;
        for (size_t k = 0; k < nt; k++) {
            uint32_t id = tarr[k];
            nodes[prev].next[side] = id; nodes[id].prev[side] = prev;
            sim_access(sim, nodes_addr + (uint64_t)id * sizeof(EpNode), sizeof(EpNode), 1);
            prev = id;
        }
        for (size_t k = 0; k < no; k++) {
            uint32_t id = oarr[k];
            nodes[prev].next[side] = id; nodes[id].prev[side] = prev;
            sim_access(sim, nodes_addr + (uint64_t)id * sizeof(EpNode), sizeof(EpNode), 1);
            prev = id;
        }
        nodes[prev].next[side] = 0; nodes[0].prev[side] = prev;
    }

    size_t cap_rules = 65536;
    EpRule *rules = (EpRule *)malloc(cap_rules * sizeof(EpRule));
    uint64_t rules_addr = sim_alloc_region(sim, 262144 * sizeof(EpRule));
    uint32_t nrules = 0;

    uint32_t rem = n;
    while (rem > 0) {
        uint32_t ix = nodes[0].prev[0], iy = nodes[0].prev[1];
        sim_access(sim, nodes_addr + (uint64_t)ix * sizeof(EpNode), sizeof(EpNode), 0);
        sim_access(sim, nodes_addr + (uint64_t)iy * sizeof(EpNode), sizeof(EpNode), 0);
        EpNode a = nodes[ix], b = nodes[iy];
        uint32_t w = a.width < b.width ? a.width : b.width;
        rem -= w;
        met->work_events++;
        if (ix == iy) {
            ep_cut(sim, nodes_addr, nodes, ix, 0);
            ep_cut(sim, nodes_addr, nodes, ix, 1);
            continue;
        }
        uint32_t word;
        if (a.word == 0) word = b.word;
        else if (b.word == 0) word = a.word;
        else {
            if (nrules == cap_rules) { cap_rules *= 2; rules = (EpRule *)realloc(rules, cap_rules * sizeof(EpRule)); }
            uint32_t la = a.word <= 256 ? (a.word != 0) : rules[a.word - 257].len;
            uint32_t lb = b.word <= 256 ? (b.word != 0) : rules[b.word - 257].len;
            rules[nrules] = (EpRule){a.word, b.word, la + lb, UINT32_MAX};
            sim_access(sim, rules_addr + (uint64_t)nrules * sizeof(EpRule), sizeof(EpRule), 1);
            word = nrules + 257;
            nrules++;
        }
        if (a.width <= b.width) {
            ep_cut(sim, nodes_addr, nodes, ix, 0);
            ep_insert(sim, nodes_addr, nodes, iy, ix, 0);
            nodes[ix].word = word;
            if (a.width == b.width) { ep_cut(sim, nodes_addr, nodes, iy, 0); ep_cut(sim, nodes_addr, nodes, iy, 1); }
            else nodes[iy].width -= w;
        } else {
            ep_cut(sim, nodes_addr, nodes, iy, 1);
            ep_insert(sim, nodes_addr, nodes, ix, iy, 1);
            nodes[iy].word = word;
            nodes[ix].width -= w;
        }
    }

    uint32_t *bottom = (uint32_t *)malloc(nnodes * sizeof(uint32_t));
    uint64_t bot_addr = sim_alloc_region(sim, nnodes * sizeof(uint32_t));
    uint32_t id = nodes[0].next[1];
    x = 0;
    while (id != 0) {
        bottom[id] = x;
        sim_access(sim, bot_addr + (uint64_t)id * 4, 4, 1);
        sim_access(sim, nodes_addr + (uint64_t)id * sizeof(EpNode), sizeof(EpNode), 0);
        x += nodes[id].width;
        id = nodes[id].next[1];
    }
    assert(x == m);

    size_t cap_bands = nnodes + 256;
    OpenBand *bands = (OpenBand *)malloc(cap_bands * sizeof(OpenBand));
    uint64_t bands_addr = sim_alloc_region(sim, (cap_bands + 1024) * sizeof(OpenBand));
    size_t nbands = 0;

    size_t cap_spell = 1 << 18;
    uint8_t *spelling = (uint8_t *)malloc(cap_spell);
    uint64_t spell_addr = sim_alloc_region(sim, 1 << 21);
    size_t nspell = 0;

    uint32_t *offsets = (uint32_t *)malloc((cap_bands + 2) * sizeof(uint32_t));
    uint64_t off_addr = sim_alloc_region(sim, (cap_bands + 1024) * sizeof(uint32_t));
    size_t noff = 0;
    offsets[noff++] = 0;

    uint32_t *stack = (uint32_t *)malloc(65536 * sizeof(uint32_t));
    id = nodes[0].next[0];
    uint32_t in_node = 0, k = 0;
    for (int a = 0; a < 256; a++) {
        uint32_t end = k + thin[a];
        while (k < end) {
            sim_access(sim, nodes_addr + (uint64_t)id * sizeof(EpNode), sizeof(EpNode), 0);
            EpNode nd = nodes[id];
            uint32_t w = (end - k) < (nd.width - in_node) ? (end - k) : (nd.width - in_node);
            sim_access(sim, bot_addr + (uint64_t)id * 4, 4, 0);
            uint32_t dst = bottom[id] + in_node;
            uint32_t code;
            if (nd.word == 0) {
                code = (uint32_t)a;
            } else {
                size_t sp = 0; stack[sp++] = nd.word;
                while (sp > 0) {
                    uint32_t wid = stack[--sp];
                    if (wid <= 256) {
                        if (nspell + 2 >= cap_spell) { cap_spell *= 2; spelling = (uint8_t *)realloc(spelling, cap_spell); }
                        spelling[nspell] = (uint8_t)(wid - 1);
                        sim_access(sim, spell_addr + nspell, 1, 1);
                        nspell++;
                    } else {
                        EpRule *rl = &rules[wid - 257];
                        sim_access(sim, rules_addr + (uint64_t)(wid - 257) * sizeof(EpRule), sizeof(EpRule), 0);
                        if (rl->first == UINT32_MAX) {
                            rl->first = (uint32_t)nspell;
                            sim_access(sim, rules_addr + (uint64_t)(wid - 257) * sizeof(EpRule), sizeof(EpRule), 1);
                            stack[sp++] = rl->right; stack[sp++] = rl->left;
                        } else {
                            uint32_t st = rl->first, ln = rl->len;
                            if (nspell + ln + 2 >= cap_spell) {
                                while (nspell + ln + 2 >= cap_spell) cap_spell *= 2;
                                spelling = (uint8_t *)realloc(spelling, cap_spell);
                            }
                            sim_access(sim, spell_addr + st, ln, 0);
                            memcpy(spelling + nspell, spelling + st, ln);
                            sim_access(sim, spell_addr + nspell, ln, 1);
                            nspell += ln;
                        }
                    }
                }
                if (nspell + 2 >= cap_spell) { cap_spell *= 2; spelling = (uint8_t *)realloc(spelling, cap_spell); }
                spelling[nspell] = (uint8_t)a;
                sim_access(sim, spell_addr + nspell, 1, 1);
                nspell++;
                offsets[noff] = (uint32_t)nspell;
                sim_access(sim, off_addr + noff * 4, 4, 1);
                noff++;
                code = 254 + (uint32_t)noff;
            }
            if (nbands == cap_bands) {
                cap_bands *= 2;
                bands = (OpenBand *)realloc(bands, cap_bands * sizeof(OpenBand));
                offsets = (uint32_t *)realloc(offsets, (cap_bands + 2) * sizeof(uint32_t));
            }
            bands[nbands] = (OpenBand){k + w, dst - k, code};
            sim_access(sim, bands_addr + nbands * sizeof(OpenBand), sizeof(OpenBand), 1);
            nbands++;
            met->work_events++;
            in_node += w; k += w;
            if (in_node == nd.width) { id = nd.next[0]; in_node = 0; }
        }
    }

    *out_bands = bands; *out_bands_addr = bands_addr; *out_nbands = nbands;
    *out_spell = spelling; *out_spell_addr = spell_addr; *out_nspell = nspell;
    *out_off = offsets; *out_off_addr = off_addr; *out_noff = noff;
    *out_ep_mem = nstrips * sizeof(OpenStrip) + nnodes * sizeof(EpNode) + nrules * sizeof(EpRule)
                + nbands * sizeof(OpenBand) + nspell + noff * 4;

    free(image_order); free(nodes); free(ord0); free(ord1); free(term0); free(term1);
    free(rules); free(bottom); free(stack);
}

/* ==================== 1. Shipped Forward Open (QUEUE=true) ==================== */

static uint8_t *run_forward_open(const uint16_t *recs, size_t nrecs, uint32_t n, uint32_t p, Metrics *met) {
    const uint32_t LIMIT = 16;
    CacheSim sim; sim_init(&sim);
    memset(met, 0, sizeof(*met));

    uint64_t rec_addr = sim_alloc_region(&sim, nrecs * 2);
    uint32_t all[256] = {0}, thin[256] = {0}, thick[256] = {0};
    OpenStrip *strips = (OpenStrip *)malloc((nrecs + 2) * sizeof(OpenStrip));
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

    OpenBand *bands; uint64_t bands_addr; size_t nbands;
    uint8_t *spelling; uint64_t spell_addr; size_t nspell;
    uint32_t *offsets; uint64_t off_addr; size_t noff;
    size_t ep_mem = 0;
    open_endpoint_and_bands(&sim, met, strips, strips_addr, nstrips, all, thin, thick, n, m,
                            &bands, &bands_addr, &nbands, &spelling, &spell_addr, &nspell,
                            &offsets, &off_addr, &noff, &ep_mem);
    met->l2_ep = sim.l2.misses - met->l2_p1;

    uint32_t bits = 0; while ((1u << bits) < m) bits++;
    uint32_t mask = (1u << bits) - 1;
    uint32_t cap_inline = (1u << (32 - bits)) - 1;

    uint32_t *edges = (uint32_t *)malloc(m * 4);
    uint64_t edges_addr = sim_alloc_region(&sim, m * 4);
    uint32_t *extra = (uint32_t *)malloc((m + 1) * 4);
    uint64_t extra_addr = sim_alloc_region(&sim, (m + 1) * 4);
    uint32_t nextra = 0;

    uint32_t k = 0; size_t bi = 0;
    for (int a = 0; a < 256; a++) {
        uint32_t ql = qlen[a]; uint32_t *qd = qdata[a]; uint32_t qi = 0;
        while (qi < ql) {
            sim_access(&sim, bands_addr + bi * sizeof(OpenBand), sizeof(OpenBand), 0);
            OpenBand b = bands[bi];
            uint32_t w = b.end - k;
            uint32_t tag = (b.code < cap_inline) ? (b.code << bits) : (cap_inline << bits);
            for (uint32_t u = 0; u < w; u++) {
                sim_access(&sim, qaddr[a] + (uint64_t)(qi + u) * 4, 4, 0);
                uint32_t target = qd[qi + u];
                uint32_t idx = (k + u) + b.delta;
                if (b.code < cap_inline) {
                    edges[idx] = tag | target;
                    sim_access(&sim, edges_addr + (uint64_t)idx * 4, 4, 1);
                } else {
                    edges[idx] = tag | nextra;
                    sim_access(&sim, edges_addr + (uint64_t)idx * 4, 4, 1);
                    extra[nextra] = target;
                    extra[nextra + 1] = b.code;
                    sim_access(&sim, extra_addr + (uint64_t)nextra * 4, 8, 1);
                    nextra += 2;
                }
                met->work_events++;
            }
            k += w; qi += w; bi++;
        }
    }
    met->l2_p2 = sim.l2.misses - met->l2_p1 - met->l2_ep;

    uint8_t *out = (uint8_t *)malloc(n);
    uint64_t out_addr = sim_alloc_region(&sim, n);
    uint32_t out_pos = 0, j = mark;
    for (uint32_t step = 0; step < m && out_pos < n; step++) {
        sim_access(&sim, edges_addr + (uint64_t)j * 4, 4, 0);
        uint32_t edge = edges[j];
        uint32_t code = edge >> bits;
        uint32_t nxt = edge & mask;
        if (code == cap_inline) {
            sim_access(&sim, extra_addr + (uint64_t)nxt * 4, 8, 0);
            uint32_t ex = nxt;
            nxt = extra[ex]; code = extra[ex + 1];
        }
        if (code < 256) {
            out[out_pos] = (uint8_t)code;
            sim_access(&sim, out_addr + out_pos, 1, 1);
            out_pos++;
        } else {
            uint32_t wi = code - 256;
            sim_access(&sim, off_addr + (uint64_t)wi * 4, 8, 0);
            uint32_t st = offsets[wi], en = offsets[wi + 1], ln = en - st;
            if (out_pos + ln > n) ln = n - out_pos;
            sim_access(&sim, spell_addr + st, ln, 0);
            memcpy(out + out_pos, spelling + st, ln);
            sim_access(&sim, out_addr + out_pos, ln, 1);
            out_pos += ln;
        }
        j = nxt; met->work_events++;
    }
    met->l2_p3 = sim.l2.misses - met->l2_p1 - met->l2_ep - met->l2_p2;

    sim_drain(&sim);
    met->l1_misses = sim.l1.misses; met->l2_misses = sim.l2.misses; met->llc_misses = sim.llc.misses;
    met->dram_bytes = (sim.dram_reads + sim.dram_writes) * 64ULL;
    met->m_retained = m; met->nbands = (uint32_t)nbands;
    met->peak_mem = nrecs * 2 + (size_t)m * 4 + (size_t)m * 4 + nextra * 4 + ep_mem + n;

    for (int a = 0; a < 256; a++) free(qdata[a]);
    free(strips); free(bands); free(spelling); free(offsets); free(edges); free(extra);
    sim_free(&sim);
    return out;
}

/* ==================== 2. Queue-Free Backward Open (Cycle 2) ==================== */

static uint8_t *run_backward_open(const uint16_t *recs, size_t nrecs, uint32_t n, uint32_t p, Metrics *met) {
    const uint32_t LIMIT = 16;
    CacheSim sim; sim_init(&sim);
    memset(met, 0, sizeof(*met));

    uint64_t rec_addr = sim_alloc_region(&sim, nrecs * 2);
    uint32_t all[256] = {0}, thin[256] = {0}, thick[256] = {0};
    OpenStrip *strips = (OpenStrip *)calloc(nrecs + 2, sizeof(OpenStrip));
    uint64_t strips_addr = sim_alloc_region(&sim, (nrecs + 2) * sizeof(OpenStrip));
    size_t nstrips = 0;

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
        if (w > LIMIT && pos <= p && p < end) {
            if (pos < p) { strips[nstrips] = (OpenStrip){pos, all[c], p - pos, c}; sim_access(&sim, strips_addr + nstrips * sizeof(OpenStrip), sizeof(OpenStrip), 1); nstrips++; thick[c]++; }
            mark = m; thin[c] += 1; m += 1;
            if (p + 1 < end) { strips[nstrips] = (OpenStrip){p + 1, all[c] + (p + 1 - pos), end - p - 1, c}; sim_access(&sim, strips_addr + nstrips * sizeof(OpenStrip), sizeof(OpenStrip), 1); nstrips++; thick[c]++; }
        } else if (w <= LIMIT) {
            if (pos <= p && p < end) mark = m + (p - pos);
            thin[c] += w; m += w;
        } else {
            strips[nstrips] = (OpenStrip){pos, all[c], w, c};
            sim_access(&sim, strips_addr + nstrips * sizeof(OpenStrip), sizeof(OpenStrip), 1);
            nstrips++; thick[c]++;
        }
        all[c] += w; pos = end;
    }
    met->l2_p1 = sim.l2.misses;

    OpenBand *bands; uint64_t bands_addr; size_t nbands;
    uint8_t *spelling; uint64_t spell_addr; size_t nspell;
    uint32_t *offsets; uint64_t off_addr; size_t noff;
    size_t ep_mem = 0;
    open_endpoint_and_bands(&sim, met, strips, strips_addr, nstrips, all, thin, thick, n, m,
                            &bands, &bands_addr, &nbands, &spelling, &spell_addr, &nspell,
                            &offsets, &off_addr, &noff, &ep_mem);
    met->l2_ep = sim.l2.misses - met->l2_p1;

    uint32_t bits = 0; while ((1u << bits) < m) bits++;
    uint32_t mask = (1u << bits) - 1;
    uint32_t cap_inline = (1u << (32 - bits)) - 1;

    uint32_t sym_cur[256], band_idx[256];
    OpenBand active[256];
    uint32_t tot = 0; size_t bi = 0;
    for (int a = 0; a < 256; a++) {
        sym_cur[a] = tot;
        if (thin[a] > 0) {
            while (bi < nbands && bands[bi].end <= tot) bi++;
            band_idx[a] = (uint32_t)bi;
            sim_access(&sim, bands_addr + bi * sizeof(OpenBand), sizeof(OpenBand), 0);
            active[a] = bands[bi];
        } else { band_idx[a] = 0; active[a] = (OpenBand){0, 0, 0}; }
        tot += thin[a];
    }

    uint32_t *edges = (uint32_t *)malloc(m * 4);
    uint64_t edges_addr = sim_alloc_region(&sim, m * 4);
    uint32_t *extra = (uint32_t *)malloc((m + 1) * 4);
    uint64_t extra_addr = sim_alloc_region(&sim, (m + 1) * 4);
    uint32_t nextra = 0;

    uint32_t cur_m = 0; pos = 0; ri = 0;
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
        uint32_t end = pos + w;
        uint32_t rw = 0;
        if (w <= LIMIT) rw = w;
        else if (pos <= p && p < end) rw = 1;
        if (rw > 0) {
            uint32_t r = sym_cur[c], rem = rw;
            OpenBand ab = active[c];
            while (rem > 0) {
                uint32_t avail = ab.end - r;
                uint32_t take = rem < avail ? rem : avail;
                uint32_t tag = (ab.code < cap_inline) ? (ab.code << bits) : (cap_inline << bits);
                for (uint32_t u = 0; u < take; u++) {
                    uint32_t dst = (r + u) + ab.delta;
                    if (ab.code < cap_inline) {
                        edges[cur_m] = tag | dst;
                        sim_access(&sim, edges_addr + (uint64_t)cur_m * 4, 4, 1);
                    } else {
                        edges[cur_m] = tag | nextra;
                        sim_access(&sim, edges_addr + (uint64_t)cur_m * 4, 4, 1);
                        extra[nextra] = dst; extra[nextra + 1] = ab.code;
                        sim_access(&sim, extra_addr + (uint64_t)nextra * 4, 8, 1);
                        nextra += 2;
                    }
                    cur_m++; met->work_events++;
                }
                r += take; rem -= take;
                if (r == ab.end) {
                    band_idx[c]++;
                    if (band_idx[c] < nbands) {
                        sim_access(&sim, bands_addr + (uint64_t)band_idx[c] * sizeof(OpenBand), sizeof(OpenBand), 0);
                        ab = bands[band_idx[c]];
                    }
                }
            }
            sym_cur[c] = r; active[c] = ab;
        }
        pos = end;
    }
    met->l2_p2 = sim.l2.misses - met->l2_p1 - met->l2_ep;

    uint8_t *out = (uint8_t *)malloc(n);
    uint64_t out_addr = sim_alloc_region(&sim, n);
    uint32_t write_pos = n, j = mark;
    for (uint32_t step = 0; step < m && write_pos > 0; step++) {
        sim_access(&sim, edges_addr + (uint64_t)j * 4, 4, 0);
        uint32_t edge = edges[j];
        uint32_t code = edge >> bits;
        uint32_t nxt = edge & mask;
        if (code == cap_inline) {
            sim_access(&sim, extra_addr + (uint64_t)nxt * 4, 8, 0);
            uint32_t ex = nxt; nxt = extra[ex]; code = extra[ex + 1];
        }
        if (code < 256) {
            write_pos--;
            out[write_pos] = (uint8_t)code;
            sim_access(&sim, out_addr + write_pos, 1, 1);
        } else {
            uint32_t wi = code - 256;
            sim_access(&sim, off_addr + (uint64_t)wi * 4, 8, 0);
            uint32_t st = offsets[wi], en = offsets[wi + 1], ln = en - st;
            if (ln > write_pos) ln = write_pos;
            write_pos -= ln;
            sim_access(&sim, spell_addr + st, ln, 0);
            memcpy(out + write_pos, spelling + st, ln);
            sim_access(&sim, out_addr + write_pos, ln, 1);
        }
        j = nxt; met->work_events++;
    }
    met->l2_p3 = sim.l2.misses - met->l2_p1 - met->l2_ep - met->l2_p2;

    sim_drain(&sim);
    met->l1_misses = sim.l1.misses; met->l2_misses = sim.l2.misses; met->llc_misses = sim.llc.misses;
    met->dram_bytes = (sim.dram_reads + sim.dram_writes) * 64ULL;
    met->m_retained = m; met->nbands = (uint32_t)nbands;
    met->peak_mem = nrecs * 2 + (size_t)m * 4 + nextra * 4 + ep_mem + n;

    free(strips); free(bands); free(spelling); free(offsets); free(edges); free(extra);
    sim_free(&sim);
    return out;
}

/* ==================== 3. BSSE: Bucketed Sequential Schur Elimination ==================== */

typedef struct {
    uint32_t a; /* packed endpoints or global u */
    uint32_t b; /* length or (v_loc, ln) */
} Edge8;

typedef struct {
    Edge8 *data;
    uint32_t len, cap;
    uint64_t addr;
} VecE8;

typedef struct {
    uint32_t *data;
    uint32_t len, cap;
    uint64_t addr;
} VecU32;

typedef struct {
    uint16_t *data;
    uint32_t len, cap;
    uint64_t addr;
} VecU16;

static inline void ve8_init(VecE8 *v, uint32_t cap, CacheSim *sim, size_t max_bytes) {
    v->cap = cap; v->len = 0;
    v->data = (Edge8 *)malloc((size_t)cap * sizeof(Edge8));
    v->addr = sim_alloc_region(sim, max_bytes);
}
static inline void ve8_push(VecE8 *v, Edge8 x, CacheSim *sim) {
    if (v->len == v->cap) { v->cap *= 2; v->data = (Edge8 *)realloc(v->data, (size_t)v->cap * sizeof(Edge8)); }
    v->data[v->len] = x;
    sim_access(sim, v->addr + (uint64_t)v->len * sizeof(Edge8), sizeof(Edge8), 1);
    v->len++;
}
static inline void vu32_init(VecU32 *v, uint32_t cap, CacheSim *sim, size_t max_bytes) {
    v->cap = cap; v->len = 0;
    v->data = (uint32_t *)malloc((size_t)cap * 4);
    v->addr = sim_alloc_region(sim, max_bytes);
}
static inline void vu32_push(VecU32 *v, uint32_t x, CacheSim *sim) {
    if (v->len == v->cap) { v->cap *= 2; v->data = (uint32_t *)realloc(v->data, (size_t)v->cap * 4); }
    v->data[v->len] = x;
    sim_access(sim, v->addr + (uint64_t)v->len * 4, 4, 1);
    v->len++;
}
static inline void vu16_init(VecU16 *v, uint32_t cap, CacheSim *sim, size_t max_bytes) {
    v->cap = cap; v->len = 0;
    v->data = (uint16_t *)malloc((size_t)cap * 2);
    v->addr = sim_alloc_region(sim, max_bytes);
}
static inline void vu16_push(VecU16 *v, uint16_t x, CacheSim *sim) {
    if (v->len == v->cap) { v->cap *= 2; v->data = (uint16_t *)realloc(v->data, (size_t)v->cap * 2); }
    v->data[v->len] = x;
    sim_access(sim, v->addr + (uint64_t)v->len * 2, 2, 1);
    v->len++;
}

static uint8_t *run_bsse(
    const uint16_t *recs, size_t nrecs, uint32_t n, uint32_t p,
    uint32_t bshift, uint32_t n_slabs, int use_slabbed_out, Metrics *met
) {
    const uint32_t LIMIT = 16;
    const uint32_t bsize = 1u << bshift;
    const uint32_t bmask = bsize - 1u;

    CacheSim sim; sim_init(&sim);
    memset(met, 0, sizeof(*met));

    uint64_t rec_addr = sim_alloc_region(&sim, nrecs * 2);
    uint32_t all[256] = {0}, thin[256] = {0}, thick[256] = {0};
    OpenStrip *strips = (OpenStrip *)calloc(nrecs + 2, sizeof(OpenStrip));
    uint64_t strips_addr = sim_alloc_region(&sim, (nrecs + 2) * sizeof(OpenStrip));
    size_t nstrips = 0;

    /* Pass 1: Thick strip classification (zero queues) */
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
        if (w > LIMIT && pos <= p && p < end) {
            if (pos < p) { strips[nstrips] = (OpenStrip){pos, all[c], p - pos, c}; sim_access(&sim, strips_addr + nstrips * sizeof(OpenStrip), sizeof(OpenStrip), 1); nstrips++; thick[c]++; }
            mark = m; thin[c] += 1; m += 1;
            if (p + 1 < end) { strips[nstrips] = (OpenStrip){p + 1, all[c] + (p + 1 - pos), end - p - 1, c}; sim_access(&sim, strips_addr + nstrips * sizeof(OpenStrip), sizeof(OpenStrip), 1); nstrips++; thick[c]++; }
        } else if (w <= LIMIT) {
            if (pos <= p && p < end) mark = m + (p - pos);
            thin[c] += w; m += w;
        } else {
            strips[nstrips] = (OpenStrip){pos, all[c], w, c};
            sim_access(&sim, strips_addr + nstrips * sizeof(OpenStrip), sizeof(OpenStrip), 1);
            nstrips++; thick[c]++;
        }
        all[c] += w; pos = end;
    }
    met->l2_p1 = sim.l2.misses;

    /* Endpoint elimination & bands (identical to Open) */
    OpenBand *bands; uint64_t bands_addr; size_t nbands;
    uint8_t *spelling; uint64_t spell_addr; size_t nspell;
    uint32_t *offsets; uint64_t off_addr; size_t noff;
    size_t ep_mem = 0;
    open_endpoint_and_bands(&sim, met, strips, strips_addr, nstrips, all, thin, thick, n, m,
                            &bands, &bands_addr, &nbands, &spelling, &spell_addr, &nspell,
                            &offsets, &off_addr, &noff, &ep_mem);
    met->l2_ep = sim.l2.misses - met->l2_p1;
    free(strips);

    uint32_t s = (m + bsize - 1) / bsize;
    assert(s <= 64);
    assert(m <= (1u << (32 - bshift))); /* ensure (u_glob << bshift) | v_loc fits in u32! */

    /* Active band cache (5 KiB L1-resident) */
    uint32_t sym_cur[256], band_idx[256];
    OpenBand active[256];
    uint32_t tot = 0; size_t bi = 0;
    for (int a = 0; a < 256; a++) {
        sym_cur[a] = tot;
        if (thin[a] > 0) {
            while (bi < nbands && bands[bi].end <= tot) bi++;
            band_idx[a] = (uint32_t)bi;
            sim_access(&sim, bands_addr + bi * sizeof(OpenBand), sizeof(OpenBand), 0);
            active[a] = bands[bi];
        } else { band_idx[a] = 0; active[a] = (OpenBand){0, 0, 0}; }
        tot += thin[a];
    }

    /* Phase 2: Queue-free backward scan directly into initial bucket queues */
    uint16_t *orig_code = (uint16_t *)malloc((size_t)m * 2);
    uint64_t orig_code_addr = sim_alloc_region(&sim, (size_t)m * 2);

    VecU32 fwd_init[64];
    VecE8 fwd_rt[64], in_q[64], self_chain[64];
    for (uint32_t k = 0; k < s; k++) {
        vu32_init(&fwd_init[k], bsize, &sim, (size_t)bsize * 4);
        ve8_init(&fwd_rt[k], bsize / 2 + 64, &sim, (size_t)bsize * sizeof(Edge8));
        ve8_init(&in_q[k], bsize, &sim, (size_t)bsize * sizeof(Edge8));
        ve8_init(&self_chain[k], bsize / 2 + 64, &sim, (size_t)bsize * sizeof(Edge8));
    }

    uint32_t cur_m = 0; pos = 0; ri = 0;
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
        uint32_t end = pos + w;
        uint32_t rw = 0;
        if (w <= LIMIT) rw = w;
        else if (pos <= p && p < end) rw = 1;
        if (rw > 0) {
            uint32_t r = sym_cur[c], rem = rw;
            OpenBand ab = active[c];
            while (rem > 0) {
                uint32_t avail = ab.end - r;
                uint32_t take = rem < avail ? rem : avail;
                uint32_t code = ab.code;
                assert(code < 32768);
                uint32_t blen = 1;
                if (code >= 256) {
                    uint32_t wi = code - 256;
                    sim_access(&sim, off_addr + (uint64_t)wi * 4, 8, 0);
                    blen = offsets[wi + 1] - offsets[wi];
                }
                for (uint32_t u = 0; u < take; u++) {
                    uint32_t dst = (r + u) + ab.delta;
                    orig_code[cur_m] = (uint16_t)code;
                    sim_access(&sim, orig_code_addr + (uint64_t)cur_m * 2, 2, 1);
                    uint32_t ju = cur_m >> bshift;
                    uint32_t jd = dst >> bshift;
                    if (ju <= jd) {
                        uint32_t packed = (dst << bshift) | (cur_m & bmask);
                        vu32_push(&fwd_init[ju], packed, &sim);
                    } else {
                        Edge8 e = { (cur_m << bshift) | (dst & bmask), blen };
                        ve8_push(&in_q[jd], e, &sim);
                    }
                    cur_m++; met->work_events++;
                }
                r += take; rem -= take;
                if (r == ab.end) {
                    band_idx[c]++;
                    if (band_idx[c] < nbands) {
                        sim_access(&sim, bands_addr + (uint64_t)band_idx[c] * sizeof(OpenBand), sizeof(OpenBand), 0);
                        ab = bands[band_idx[c]];
                    }
                }
            }
            sym_cur[c] = r; active[c] = ab;
        }
        pos = end;
    }
    free(bands);
    met->l2_p2 = sim.l2.misses - met->l2_p1 - met->l2_ep;

    /* Phase 3: Forward Bucketed Schur Elimination (L2-resident working set per bucket!) */
    uint32_t *nxt_dst = (uint32_t *)malloc((size_t)bsize * 4);
    uint32_t *nxt_ln  = (uint32_t *)malloc((size_t)bsize * 4);
    uint8_t  *visited = (uint8_t  *)malloc((size_t)bsize);
    uint64_t nxt_dst_addr = sim_alloc_region(&sim, (size_t)bsize * 4);
    uint64_t nxt_ln_addr  = sim_alloc_region(&sim, (size_t)bsize * 4);
    uint64_t vis_addr     = sim_alloc_region(&sim, (size_t)bsize);

    VecU16 *req_u = (VecU16 *)calloc((size_t)s * (size_t)s, sizeof(VecU16));
    for (uint32_t k = 0; k < s; k++) {
        for (uint32_t ju = k + 1; ju < s; ju++) {
            vu16_init(&req_u[k * s + ju], 256, &sim, (size_t)bsize * 2);
        }
    }

    uint32_t mark_anchor = mark, mark_shift = 0;
    int close_k = -1;
    uint32_t L_cyc = 0;
    uint32_t *close_pos = NULL;
    uint64_t pos_addr = sim_alloc_region(&sim, (size_t)bsize * 4);

    size_t cur_live_q = (size_t)m * 2 + (size_t)m * 6; /* approximate initial queues */
    size_t peak_q_bytes = cur_live_q;

    for (uint32_t k = 0; k < s; k++) {
        uint32_t base = k << bshift;
        uint32_t sz = (m - base < bsize) ? (m - base) : bsize;

        /* Load fwd_init[k] (4B/edge, reading L1/L2-local orig_code[base..base+sz]) */
        for (uint32_t i = 0; i < fwd_init[k].len; i++) {
            sim_access(&sim, fwd_init[k].addr + (uint64_t)i * 4, 4, 0);
            uint32_t packed = fwd_init[k].data[i];
            uint32_t ul = packed & bmask;
            uint32_t wg = packed >> bshift;
            sim_access(&sim, orig_code_addr + (uint64_t)(base + ul) * 2, 2, 0);
            uint32_t code = orig_code[base + ul];
            uint32_t ln = 1;
            if (code >= 256) {
                uint32_t wi = code - 256;
                sim_access(&sim, off_addr + (uint64_t)wi * 4, 8, 0);
                ln = offsets[wi + 1] - offsets[wi];
            }
            nxt_dst[ul] = wg;
            nxt_ln[ul]  = ln;
            sim_access(&sim, nxt_dst_addr + (uint64_t)ul * 4, 4, 1);
            sim_access(&sim, nxt_ln_addr  + (uint64_t)ul * 4, 4, 1);
            met->work_events++;
        }
        /* Load fwd_rt[k] (8B/edge) */
        for (uint32_t i = 0; i < fwd_rt[k].len; i++) {
            sim_access(&sim, fwd_rt[k].addr + (uint64_t)i * sizeof(Edge8), sizeof(Edge8), 0);
            Edge8 e = fwd_rt[k].data[i];
            uint32_t ul = e.a & bmask;
            uint32_t wg = e.a >> bshift;
            nxt_dst[ul] = wg;
            nxt_ln[ul]  = e.b;
            sim_access(&sim, nxt_dst_addr + (uint64_t)ul * 4, 4, 1);
            sim_access(&sim, nxt_ln_addr  + (uint64_t)ul * 4, 4, 1);
            met->work_events++;
        }
        /* Free fwd_init[k] and fwd_rt[k] immediately */
        free(fwd_init[k].data); fwd_init[k].data = NULL;
        free(fwd_rt[k].data); fwd_rt[k].data = NULL;

        memset(visited, 0, sz);
        sim_access(&sim, vis_addr, sz, 1);

        /* Exact Chain-Head Decomposition over in_q[k] */
        for (uint32_t i = 0; i < in_q[k].len; i++) {
            sim_access(&sim, in_q[k].addr + (uint64_t)i * sizeof(Edge8), sizeof(Edge8), 0);
            Edge8 e = in_q[k].data[i];
            uint32_t u_glob = e.a >> bshift;
            uint32_t v0 = e.a & bmask;
            uint32_t ln1 = e.b;
            uint32_t ju = u_glob >> bshift;
            vu16_push(&req_u[k * s + ju], (uint16_t)(u_glob & bmask), &sim);

            uint32_t cur = v0, acc_ln = 0;
            int hit_mark = 0; uint32_t hit_mark_at = 0;
            for (;;) {
                visited[cur] = 1;
                sim_access(&sim, vis_addr + cur, 1, 1);
                if (base + cur == mark_anchor) { hit_mark = 1; hit_mark_at = acc_ln; }
                sim_access(&sim, nxt_dst_addr + (uint64_t)cur * 4, 4, 0);
                sim_access(&sim, nxt_ln_addr  + (uint64_t)cur * 4, 4, 0);
                uint32_t wg = nxt_dst[cur];
                uint32_t sln = nxt_ln[cur];
                acc_ln += sln;
                uint32_t jw = wg >> bshift;
                met->work_events++;
                if (jw != k) {
                    if (hit_mark) {
                        mark_anchor = wg;
                        mark_shift += (acc_ln - hit_mark_at);
                    }
                    met->routed_edges++;
                    uint32_t tot_ln = ln1 + acc_ln;
                    if (ju <= jw) {
                        Edge8 ne = { (wg << bshift) | (u_glob & bmask), tot_ln };
                        ve8_push(&fwd_rt[ju], ne, &sim);
                    } else {
                        Edge8 ne = { (u_glob << bshift) | (wg & bmask), tot_ln };
                        ve8_push(&in_q[jw], ne, &sim);
                    }
                    break;
                } else {
                    uint32_t nxt_loc = wg & bmask;
                    Edge8 sc = { (cur << bshift) | nxt_loc, sln };
                    ve8_push(&self_chain[k], sc, &sim);
                    met->self_hops++;
                    cur = nxt_loc;
                }
            }
        }

        /* Measure current live queue footprint */
        size_t qnow = (size_t)m * 2 + (size_t)bsize * 9;
        for (uint32_t q = 0; q < s; q++) {
            if (fwd_init[q].data) qnow += (size_t)fwd_init[q].len * 4;
            if (fwd_rt[q].data)   qnow += (size_t)fwd_rt[q].len * sizeof(Edge8);
            if (in_q[q].data)     qnow += (size_t)in_q[q].len * sizeof(Edge8);
            if (self_chain[q].data) qnow += (size_t)self_chain[q].len * sizeof(Edge8);
        }
        for (uint32_t a = 0; a < s * s; a++) qnow += (size_t)req_u[a].len * 2;
        if (qnow > peak_q_bytes) peak_q_bytes = qnow;

        if ((mark_anchor >> bshift) == k) {
            uint32_t mloc = mark_anchor & bmask;
            sim_access(&sim, vis_addr + mloc, 1, 0);
            if (!visited[mloc]) {
                close_k = (int)k;
                close_pos = (uint32_t *)malloc((size_t)bsize * 4);
                for (uint32_t z = 0; z < sz; z++) close_pos[z] = UINT32_MAX;
                sim_access(&sim, pos_addr, (size_t)sz * 4, 1);

                uint32_t cur = mloc, rel = 0;
                for (;;) {
                    close_pos[cur] = rel;
                    sim_access(&sim, pos_addr + (uint64_t)cur * 4, 4, 1);
                    sim_access(&sim, nxt_dst_addr + (uint64_t)cur * 4, 4, 0);
                    sim_access(&sim, nxt_ln_addr  + (uint64_t)cur * 4, 4, 0);
                    uint32_t wg = nxt_dst[cur];
                    rel += nxt_ln[cur];
                    cur = wg & bmask;
                    met->work_events++;
                    if (cur == mloc) break;
                }
                L_cyc = rel;
                for (uint32_t z = 0; z < sz; z++) {
                    sim_access(&sim, pos_addr + (uint64_t)z * 4, 4, 0);
                    if (close_pos[z] != UINT32_MAX) {
                        close_pos[z] = (close_pos[z] + mark_shift) % L_cyc;
                        sim_access(&sim, pos_addr + (uint64_t)z * 4, 4, 1);
                    }
                }
                break;
            }
        }
    }
    assert(close_k >= 0 && L_cyc > 0);
    free(nxt_dst); free(nxt_ln); free(visited);
    /* Free any untouched buckets above close_k */
    for (uint32_t k = (uint32_t)close_k + 1; k < s; k++) {
        free(fwd_init[k].data); free(fwd_rt[k].data);
        free(in_q[k].data); free(self_chain[k].data);
    }
    met->l2_p3 = sim.l2.misses - met->l2_p1 - met->l2_ep - met->l2_p2;

    /* Phase 4: Reverse Position Assignment (j = close_k down to 0) */
    VecU32 *resp_p = (VecU32 *)calloc((size_t)s * (size_t)s, sizeof(VecU32));
    uint32_t *pos_buf = close_pos; /* reuse L2-resident pos buffer across all j! */

    uint32_t slab_sz = (L_cyc + n_slabs - 1) / n_slabs;
    VecU32 out_q[32];
    if (use_slabbed_out) {
        assert(n_slabs <= 32 && slab_sz <= (1u << 17));
        for (uint32_t sl = 0; sl < n_slabs; sl++) {
            vu32_init(&out_q[sl], (m / n_slabs) + 256, &sim, ((size_t)m / n_slabs + 16384) * 4);
        }
    }
    uint8_t *out = (uint8_t *)malloc(n);
    uint64_t out_addr = sim_alloc_region(&sim, n);

    for (int j = close_k; j >= 0; j--) {
        uint32_t base = (uint32_t)j << bshift;
        uint32_t sz = (m - base < bsize) ? (m - base) : bsize;
        if (j < close_k) {
            for (uint32_t z = 0; z < sz; z++) pos_buf[z] = UINT32_MAX;
            sim_access(&sim, pos_addr, (size_t)sz * 4, 1);

            uint32_t rptr[64] = {0};
            for (uint32_t i = 0; i < in_q[j].len; i++) {
                sim_access(&sim, in_q[j].addr + (uint64_t)i * sizeof(Edge8), sizeof(Edge8), 0);
                Edge8 e = in_q[j].data[i];
                uint32_t u_glob = e.a >> bshift;
                uint32_t v0 = e.a & bmask;
                uint32_t ln1 = e.b;
                uint32_t ju = u_glob >> bshift;
                if ((int)ju > close_k) continue;
                VecU32 *rp = &resp_p[(uint32_t)j * s + ju];
                uint32_t ri2 = rptr[ju]++;
                sim_access(&sim, rp->addr + (uint64_t)ri2 * 4, 4, 0);
                uint32_t p_u = rp->data[ri2];
                if (p_u != UINT32_MAX) {
                    uint32_t pv = p_u + ln1;
                    if (pv >= L_cyc) pv %= L_cyc;
                    pos_buf[v0] = pv;
                    sim_access(&sim, pos_addr + (uint64_t)v0 * 4, 4, 1);
                }
                met->work_events++;
            }
            /* Linear topological scan of self_chain[j] */
            for (uint32_t i = 0; i < self_chain[j].len; i++) {
                sim_access(&sim, self_chain[j].addr + (uint64_t)i * sizeof(Edge8), sizeof(Edge8), 0);
                Edge8 sc = self_chain[j].data[i];
                uint32_t u_loc = sc.a >> bshift;
                uint32_t w_loc = sc.a & bmask;
                sim_access(&sim, pos_addr + (uint64_t)u_loc * 4, 4, 0);
                uint32_t p_u = pos_buf[u_loc];
                if (p_u != UINT32_MAX) {
                    uint32_t pw = p_u + sc.b;
                    if (pw >= L_cyc) pw %= L_cyc;
                    pos_buf[w_loc] = pw;
                    sim_access(&sim, pos_addr + (uint64_t)w_loc * 4, 4, 1);
                }
                met->work_events++;
            }
            for (int ju = j + 1; ju <= close_k; ju++) {
                free(resp_p[(uint32_t)j * s + (uint32_t)ju].data);
                resp_p[(uint32_t)j * s + (uint32_t)ju].data = NULL;
            }
        }
        free(in_q[j].data); in_q[j].data = NULL;
        free(self_chain[j].data); self_chain[j].data = NULL;

        /* Answer requests from earlier buckets k < j */
        for (int k = 0; k < j; k++) {
            VecU16 *rq = &req_u[(uint32_t)k * s + (uint32_t)j];
            if (rq->len > 0) {
                VecU32 *rp = &resp_p[(uint32_t)k * s + (uint32_t)j];
                vu32_init(rp, rq->len, &sim, (size_t)rq->len * 4);
                for (uint32_t i = 0; i < rq->len; i++) {
                    sim_access(&sim, rq->addr + (uint64_t)i * 2, 2, 0);
                    uint32_t ul = rq->data[i];
                    sim_access(&sim, pos_addr + (uint64_t)ul * 4, 4, 0);
                    rp->data[i] = pos_buf[ul];
                    sim_access(&sim, rp->addr + (uint64_t)i * 4, 4, 1);
                    met->work_events++;
                }
                rp->len = rq->len;
            }
            free(rq->data); rq->data = NULL;
        }

        /* Emit bucket j's words to output slabs (or directly to out[]) */
        for (uint32_t u_loc = 0; u_loc < sz; u_loc++) {
            sim_access(&sim, pos_addr + (uint64_t)u_loc * 4, 4, 0);
            uint32_t bp = pos_buf[u_loc];
            if (bp == UINT32_MAX) continue;
            sim_access(&sim, orig_code_addr + (uint64_t)(base + u_loc) * 2, 2, 0);
            uint32_t code = orig_code[base + u_loc];
            uint32_t wlen = 1;
            if (code >= 256) {
                uint32_t wi = code - 256;
                sim_access(&sim, off_addr + (uint64_t)wi * 4, 8, 0);
                wlen = offsets[wi + 1] - offsets[wi];
            }
            uint32_t fwd_start = L_cyc - bp - wlen;
            if (use_slabbed_out) {
                uint32_t sl = fwd_start / slab_sz;
                uint32_t off = fwd_start - sl * slab_sz;
                vu32_push(&out_q[sl], (off << 15) | code, &sim);
            } else {
                if (code < 256) {
                    out[fwd_start] = (uint8_t)code;
                    sim_access(&sim, out_addr + fwd_start, 1, 1);
                } else {
                    uint32_t wi = code - 256;
                    uint32_t st = offsets[wi];
                    sim_access(&sim, spell_addr + st, wlen, 0);
                    memcpy(out + fwd_start, spelling + st, wlen);
                    sim_access(&sim, out_addr + fwd_start, wlen, 1);
                }
            }
            met->work_events++;
        }
    }
    free(pos_buf); free(orig_code); free(req_u); free(resp_p);
    met->l2_p4 = sim.l2.misses - met->l2_p1 - met->l2_ep - met->l2_p2 - met->l2_p3;

    /* Phase 5: L2-Slabbed Output Emission */
    if (use_slabbed_out) {
        size_t oq_bytes = 0;
        for (uint32_t sl = 0; sl < n_slabs; sl++) oq_bytes += (size_t)out_q[sl].len * 4;
        if (oq_bytes + n > peak_q_bytes) peak_q_bytes = oq_bytes + n;
        for (uint32_t sl = 0; sl < n_slabs; sl++) {
            uint32_t sl_base = sl * slab_sz;
            for (uint32_t i = 0; i < out_q[sl].len; i++) {
                sim_access(&sim, out_q[sl].addr + (uint64_t)i * 4, 4, 0);
                uint32_t item = out_q[sl].data[i];
                uint32_t off = item >> 15;
                uint32_t code = item & 0x7FFFu;
                uint32_t idx = sl_base + off;
                if (code < 256) {
                    out[idx] = (uint8_t)code;
                    sim_access(&sim, out_addr + idx, 1, 1);
                } else {
                    uint32_t wi = code - 256;
                    sim_access(&sim, off_addr + (uint64_t)wi * 4, 8, 0);
                    uint32_t st = offsets[wi], ln = offsets[wi + 1] - st;
                    sim_access(&sim, spell_addr + st, ln, 0);
                    memcpy(out + idx, spelling + st, ln);
                    sim_access(&sim, out_addr + idx, ln, 1);
                }
                met->work_events++;
            }
            free(out_q[sl].data);
        }
    }
    if (L_cyc < n) {
        for (uint32_t off = L_cyc; off < n; off += L_cyc) {
            uint32_t cp = (n - off < L_cyc) ? (n - off) : L_cyc;
            memcpy(out + off, out, cp);
        }
    }
    met->l2_p5 = sim.l2.misses - met->l2_p1 - met->l2_ep - met->l2_p2 - met->l2_p3 - met->l2_p4;

    sim_drain(&sim);
    met->l1_misses = sim.l1.misses; met->l2_misses = sim.l2.misses; met->llc_misses = sim.llc.misses;
    met->dram_bytes = (sim.dram_reads + sim.dram_writes) * 64ULL;
    met->m_retained = m; met->nbands = (uint32_t)nbands;
    met->peak_mem = nrecs * 2 + ep_mem + peak_q_bytes + n;

    free(spelling); free(offsets);
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
static double now_ms(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return ts.tv_sec * 1000.0 + ts.tv_nsec * 1e-6;
}
static int cmp_dbl(const void *a, const void *b) {
    double da = *(const double *)a, db = *(const double *)b;
    return (da < db) ? -1 : (da > db);
}
#endif

int main(int argc, char **argv) {
    if (argc < 2) { fprintf(stderr, "Usage: %s <admitted-bwt-dir>\n", argv[0]); return 1; }
    const char *dir = argv[1];

#ifndef NO_CACHE_SIM
    printf("=== Cycle 3 G5 Falsifier: Shipped Open vs Backward Open vs BSSE (bsize=16384, slabs=8) ===\n");
    printf("Pre-registered PASS rule (all 6 blocks vs Shipped Open):\n");
    printf("  (1) Byte-exact  (2) L2m <= 0.85x  (3) DRAM B/B <= 1.00x  (4) Work <= 1.40x  (5) PeakMem <= 1.25x\n\n");

    int all_pass = 1;
    for (int bi = 0; bi < 6; bi++) {
        char path[512];
        snprintf(path, sizeof(path), "%s/%s", dir, BLOCKS[bi].file);
        FILE *fp = fopen(path, "rb");
        assert(fp != NULL);
        uint32_t n = BLOCKS[bi].n, p = BLOCKS[bi].p;
        uint8_t *L = (uint8_t *)malloc(n);
        assert(fread(L, 1, n, fp) == n);
        fclose(fp);

        size_t nrecs = 0;
        uint16_t *recs = column_to_records(L, n, &nrecs);
        uint8_t *ref = dense_reference(L, n, p);

        Metrics mf, mb, ms14, ms13, ms14_dir;
        uint8_t *of = run_forward_open(recs, nrecs, n, p, &mf);
        uint8_t *ob = run_backward_open(recs, nrecs, n, p, &mb);
        uint8_t *os14 = run_bsse(recs, nrecs, n, p, 14, 8, 1, &ms14);
        uint8_t *os13 = run_bsse(recs, nrecs, n, p, 13, 8, 1, &ms13);
        uint8_t *os14d = run_bsse(recs, nrecs, n, p, 14, 8, 0, &ms14_dir);

        assert(memcmp(of, ref, n) == 0);
        assert(memcmp(ob, ref, n) == 0);
        assert(memcmp(os14, ref, n) == 0);
        assert(memcmp(os13, ref, n) == 0);
        assert(memcmp(os14d, ref, n) == 0);

        double l2_rat = (double)ms14.l2_misses / (double)mf.l2_misses;
        double dram_f = (double)mf.dram_bytes / n;
        double dram_b = (double)mb.dram_bytes / n;
        double dram_s = (double)ms14.dram_bytes / n;
        double dram_rat = dram_s / dram_f;
        double work_rat = (double)ms14.work_events / (double)mf.work_events;
        double mem_rat  = (double)ms14.peak_mem / (double)mf.peak_mem;

        int bpass = (l2_rat <= 0.85 && dram_rat <= 1.00 && work_rat <= 1.40 && mem_rat <= 1.25);
        if (!bpass) all_pass = 0;

        printf("%s (m=%u, B=%u, routed=%.3fm):\n", BLOCKS[bi].file, mf.m_retained, mf.nbands, (double)ms14.routed_edges / mf.m_retained);
        printf("  Shipped Open : L2m=%7lu [p1=%5lu ep=%5lu wir=%5lu emit=%6lu]  LLC=%6lu  DRAM=%.2f B/B  Work=%7lu  Peak=%.2f MB\n",
               (unsigned long)mf.l2_misses, (unsigned long)mf.l2_p1, (unsigned long)mf.l2_ep,
               (unsigned long)mf.l2_p2, (unsigned long)mf.l2_p3, (unsigned long)mf.llc_misses,
               dram_f, (unsigned long)mf.work_events, mf.peak_mem / 1e6);
        printf("  Backward Open: L2m=%7lu [p1=%5lu ep=%5lu wir=%5lu emit=%6lu]  LLC=%6lu  DRAM=%.2f B/B  Work=%7lu  Peak=%.2f MB\n",
               (unsigned long)mb.l2_misses, (unsigned long)mb.l2_p1, (unsigned long)mb.l2_ep,
               (unsigned long)mb.l2_p2, (unsigned long)mb.l2_p3, (unsigned long)mb.llc_misses,
               dram_b, (unsigned long)mb.work_events, mb.peak_mem / 1e6);
        printf("  BSSE-16K-slab: L2m=%7lu [p1=%5lu ep=%5lu qsc=%5lu fwd=%6lu rev=%6lu slb=%5lu]  LLC=%6lu  DRAM=%.2f B/B  Work=%7lu  Peak=%.2f MB\n",
               (unsigned long)ms14.l2_misses, (unsigned long)ms14.l2_p1, (unsigned long)ms14.l2_ep,
               (unsigned long)ms14.l2_p2, (unsigned long)ms14.l2_p3, (unsigned long)ms14.l2_p4, (unsigned long)ms14.l2_p5,
               (unsigned long)ms14.llc_misses, dram_s, (unsigned long)ms14.work_events, ms14.peak_mem / 1e6);
        printf("  BSSE-16K-rat : L2m=%.3fx  DRAM=%.3fx  Work=%.3fx  Peak=%.3fx  -> %s\n",
               l2_rat, dram_rat, work_rat, mem_rat, bpass ? "PASS" : "FAIL");
        printf("  [Diag 8K-slb]: L2m=%7lu (%.3fx)  DRAM=%.2f B/B (%.3fx)  Work=%.3fx\n",
               (unsigned long)ms13.l2_misses, (double)ms13.l2_misses / mf.l2_misses,
               (double)ms13.dram_bytes / n, (double)ms13.dram_bytes / mf.dram_bytes,
               (double)ms13.work_events / mf.work_events);
        printf("  [Diag 16Kdir]: L2m=%7lu (%.3fx)  DRAM=%.2f B/B (%.3fx)  Work=%.3fx\n\n",
               (unsigned long)ms14_dir.l2_misses, (double)ms14_dir.l2_misses / mf.l2_misses,
               (double)ms14_dir.dram_bytes / n, (double)ms14_dir.dram_bytes / mf.dram_bytes,
               (double)ms14_dir.work_events / mf.work_events);

        free(L); free(recs); free(ref); free(of); free(ob); free(os14); free(os13); free(os14d);
    }
    printf("PRE-REGISTERED G5 VERDICT (BSSE-16K-slab vs Shipped Open, 6/6 blocks): %s\n",
           all_pass ? "PASS" : "FAIL");
#else
    printf("=== VM Single-Thread C Diagnostic Timing (uninstrumented -O3, median of 15 interleaved rounds; NOT i9 benchmark) ===\n");
    const int ROUNDS = 15;
    for (int bi = 0; bi < 6; bi++) {
        char path[512];
        snprintf(path, sizeof(path), "%s/%s", dir, BLOCKS[bi].file);
        FILE *fp = fopen(path, "rb"); assert(fp != NULL);
        uint32_t n = BLOCKS[bi].n, p = BLOCKS[bi].p;
        uint8_t *L = (uint8_t *)malloc(n); assert(fread(L, 1, n, fp) == n); fclose(fp);
        size_t nrecs = 0; uint16_t *recs = column_to_records(L, n, &nrecs);
        uint8_t *ref = dense_reference(L, n, p);
        double tf[15], tb[15], ts14[15], ts13[15], ts14d[15];
        Metrics met;
        for (int r = 0; r < ROUNDS; r++) {
            double t0 = now_ms(); uint8_t *o1 = run_forward_open(recs, nrecs, n, p, &met); double t1 = now_ms(); tf[r] = t1 - t0;
            t0 = now_ms(); uint8_t *o2 = run_backward_open(recs, nrecs, n, p, &met); t1 = now_ms(); tb[r] = t1 - t0;
            t0 = now_ms(); uint8_t *o3 = run_bsse(recs, nrecs, n, p, 14, 8, 1, &met); t1 = now_ms(); ts14[r] = t1 - t0;
            t0 = now_ms(); uint8_t *o4 = run_bsse(recs, nrecs, n, p, 13, 8, 1, &met); t1 = now_ms(); ts13[r] = t1 - t0;
            t0 = now_ms(); uint8_t *o5 = run_bsse(recs, nrecs, n, p, 14, 8, 0, &met); t1 = now_ms(); ts14d[r] = t1 - t0;
            if (r == 0) {
                assert(memcmp(o1, ref, n) == 0 && memcmp(o2, ref, n) == 0 && memcmp(o3, ref, n) == 0 && memcmp(o4, ref, n) == 0 && memcmp(o5, ref, n) == 0);
            }
            free(o1); free(o2); free(o3); free(o4); free(o5);
        }
        qsort(tf, ROUNDS, sizeof(double), cmp_dbl);
        qsort(tb, ROUNDS, sizeof(double), cmp_dbl);
        qsort(ts14, ROUNDS, sizeof(double), cmp_dbl);
        qsort(ts13, ROUNDS, sizeof(double), cmp_dbl);
        qsort(ts14d, ROUNDS, sizeof(double), cmp_dbl);
        printf("  %-11s: Open=%.2f ms | BwdOpen=%.2f ms (%.2fx) | BSSE-16K-slb=%.2f ms (%.2fx) | BSSE-8K-slb=%.2f ms (%.2fx) | BSSE-16K-dir=%.2f ms (%.2fx)\n",
               BLOCKS[bi].file, tf[7], tb[7], tb[7]/tf[7], ts14[7], ts14[7]/tf[7], ts13[7], ts13[7]/tf[7], ts14d[7], ts14d[7]/tf[7]);
        free(L); free(recs); free(ref);
    }
#endif
    return 0;
}
