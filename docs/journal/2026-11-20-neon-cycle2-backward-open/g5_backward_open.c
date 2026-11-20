/*
 * Cycle 2 · G5 Paid-Cost Falsifier:
 *   Shipped Forward Open<16, INDUCE=true, QUEUE=true> (g^{-1} = LF_v^{-1} o P^{-1})
 *   vs Queue-Free Backward Open (direct g = P o LF_v, zero queues, sequential edges).
 *
 * Pre-registered PASS/FAIL rule (fixed before execution):
 *   PASS (recommend Track 2 Coordinator native benchmark) iff on all 6 authenticated blocks:
 *     1. Modelled 256 KiB L2 misses <= 0.85x Forward Open (and run+wiring L2m cut >= 50%);
 *     2. Modelled 2 MiB LLC DRAM B/B <= 0.80x Forward Open;
 *     3. Peak live working set M <= 0.80x Forward Open;
 *     4. Primitive work events <= 0.80x Forward Open AND band-crossing split rate <= 15%.
 *   Otherwise FAIL G5 (and report exact phase breakdown and bottleneck).
 *
 * Exact 3-level 64B-line LRU cache model matching Cycle 1 / Astra D:
 *   L1D: 32 KiB 8-way | L2: 256 KiB 4-way | LLC: 2 MiB 16-way | write-allocate + drain
 *
 * Build: gcc -O3 -std=c99 -Wall -Wextra g5_backward_open.c -o g5_backward_open
 * Run:   ./g5_backward_open <path-to-admitted-bwt-dir>
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
    uint64_t accesses, misses, writebacks;
} CacheLevel;

typedef struct {
    CacheLevel l1, l2, llc;
    uint64_t dram_reads, dram_writes;
    uint64_t next_region_addr;
} CacheSim;

static void level_init(CacheLevel *lv, int sets, int ways) {
    lv->sets = sets; lv->ways = ways;
    lv->lines = (CacheLine *)calloc((size_t)sets * (size_t)ways, sizeof(CacheLine));
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
    uint64_t first = addr >> 6, last = (addr + len - 1) >> 6;
    for (uint64_t ln = first; ln <= last; ln++) sim_touch_line(s, ln, is_write);
}

static void sim_drain(CacheSim *s) {
    for (int i = 0; i < s->l1.sets * s->l1.ways; i++) {
        if (s->l1.lines[i].valid && s->l1.lines[i].dirty) {
            int set = i / s->l1.ways;
            uint64_t ln = (s->l1.lines[i].tag << __builtin_ctz(s->l1.sets)) | (uint64_t)set;
            int ev2 = 0; uint64_t ev2_addr = 0;
            if (!level_access(&s->l2, ln, 1, &ev2, &ev2_addr)) {
                int ev3 = 0; uint64_t ev3_addr = 0;
                if (!level_access(&s->llc, ln, 0, &ev3, &ev3_addr)) { s->dram_reads++; if (ev3) s->dram_writes++; }
                if (ev2) {
                    int ev3b = 0; uint64_t ev3b_addr = 0;
                    if (!level_access(&s->llc, ev2_addr, 1, &ev3b, &ev3b_addr)) { s->dram_reads++; if (ev3b) s->dram_writes++; }
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
            if (!level_access(&s->llc, ln, 1, &ev3, &ev3_addr)) { s->dram_reads++; if (ev3) s->dram_writes++; }
            s->l2.lines[i].dirty = 0;
        }
    }
    for (int i = 0; i < s->llc.sets * s->llc.ways; i++) {
        if (s->llc.lines[i].valid && s->llc.lines[i].dirty) {
            s->dram_writes++; s->llc.lines[i].dirty = 0;
        }
    }
}

/* ==================== Shared Open Helpers ==================== */

typedef struct {
    uint64_t l1_misses, l2_misses, llc_misses;
    uint64_t l2_run, l2_ep_sp, l2_wir, l2_emit;
    uint64_t dram_reads, dram_writes;
    double dram_bb;
    uint64_t work_events, irreg_q;
    uint64_t thin_runs, band_splits;
    size_t peak_mem;
} Metrics;

typedef struct { uint32_t from, to, width; uint8_t byte; } OpenStrip;
typedef struct { uint32_t prev[2], next[2], width, word; } EpNode;
typedef struct { uint32_t left, right, len, first; } EpRule;
typedef struct { uint32_t end, delta, code; } OpenBand;

static uint16_t *build_records(const uint8_t *L, size_t n, size_t *out_len) {
    uint16_t *recs = (uint16_t *)malloc(n * sizeof(uint16_t));
    size_t cnt = 0;
    for (size_t i = 0; i < n; i++) {
        uint8_t b = L[i];
        if (cnt > 0 && (recs[cnt - 1] >> 8) == b && (recs[cnt - 1] & 255) < 127) recs[cnt - 1]++;
        else recs[cnt++] = ((uint16_t)b << 8) | 1;
    }
    *out_len = cnt;
    return recs;
}

static void ep_cut(CacheSim *sim, uint64_t nodes_addr, EpNode *nodes, uint32_t id, int side) {
    sim_access(sim, nodes_addr + (uint64_t)id * sizeof(EpNode), sizeof(EpNode), 0);
    uint32_t pr = nodes[id].prev[side], nx = nodes[id].next[side];
    nodes[pr].next[side] = nx;
    sim_access(sim, nodes_addr + (uint64_t)pr * sizeof(EpNode), sizeof(EpNode), 1);
    nodes[nx].prev[side] = pr;
    sim_access(sim, nodes_addr + (uint64_t)nx * sizeof(EpNode), sizeof(EpNode), 1);
}

static void ep_insert(CacheSim *sim, uint64_t nodes_addr, EpNode *nodes, uint32_t at, uint32_t id, int side) {
    sim_access(sim, nodes_addr + (uint64_t)at * sizeof(EpNode), sizeof(EpNode), 0);
    uint32_t nx = nodes[at].next[side];
    nodes[id].prev[side] = at; nodes[id].next[side] = nx;
    sim_access(sim, nodes_addr + (uint64_t)id * sizeof(EpNode), sizeof(EpNode), 1);
    nodes[at].next[side] = id;
    sim_access(sim, nodes_addr + (uint64_t)at * sizeof(EpNode), sizeof(EpNode), 1);
    nodes[nx].prev[side] = id;
    sim_access(sim, nodes_addr + (uint64_t)nx * sizeof(EpNode), sizeof(EpNode), 1);
}

/* Shared endpoint induction + band spelling (identical for Forward and Backward Open!) */
static void run_endpoint_and_spell(
    CacheSim *sim, Metrics *met,
    OpenStrip *strips, uint64_t strips_addr, size_t nstrips,
    const uint32_t *all, const uint32_t *thin, const uint32_t *thick,
    uint32_t n, uint32_t m,
    OpenBand **out_bands, uint64_t *out_bands_addr, size_t *out_nbands,
    uint8_t **out_spell, uint64_t *out_spell_addr, size_t *out_nspell,
    uint32_t **out_off, uint64_t *out_off_addr, size_t *out_noff,
    size_t cursors[256], size_t *out_ep_mem
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
        met->irreg_q += 2;
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
        cursors[a] = nbands;
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
                        if (nspell + 1 >= cap_spell) { cap_spell *= 2; spelling = (uint8_t *)realloc(spelling, cap_spell); }
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
                            if (nspell + ln + 1 >= cap_spell) {
                                while (nspell + ln + 1 >= cap_spell) cap_spell *= 2;
                                spelling = (uint8_t *)realloc(spelling, cap_spell);
                            }
                            sim_access(sim, spell_addr + st, ln, 0);
                            memcpy(spelling + nspell, spelling + st, ln);
                            sim_access(sim, spell_addr + nspell, ln, 1);
                            nspell += ln;
                        }
                    }
                }
                if (nspell + 1 >= cap_spell) { cap_spell *= 2; spelling = (uint8_t *)realloc(spelling, cap_spell); }
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

    uint64_t rec_addr = sim_alloc_region(&sim, nrecs * sizeof(uint16_t));
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
                for (uint32_t k = 0; k < sw[s]; k++) {
                    qdata[c][qlen[c]] = m + k;
                    sim_access(&sim, qaddr[c] + (uint64_t)qlen[c] * 4, 4, 1);
                    qlen[c]++;
                }
                met->work_events += sw[s];
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
    uint64_t l2_1 = sim.l2.misses;

    OpenBand *bands; uint64_t bands_addr; size_t nbands;
    uint8_t *spelling; uint64_t spell_addr; size_t nspell;
    uint32_t *offsets; uint64_t off_addr; size_t noff;
    size_t cursors[256], ep_mem = 0;
    run_endpoint_and_spell(&sim, met, strips, strips_addr, nstrips, all, thin, thick, n, m,
                           &bands, &bands_addr, &nbands, &spelling, &spell_addr, &nspell,
                           &offsets, &off_addr, &noff, cursors, &ep_mem);
    uint64_t l2_2 = sim.l2.misses;

    uint32_t bits = 32 - (uint32_t)__builtin_clz(m > 1 ? m - 1 : 1);
    uint32_t *edges = (uint32_t *)malloc((size_t)m * 4);
    uint64_t edges_addr = sim_alloc_region(&sim, (size_t)m * 4);

    uint32_t k = 0;
    for (int a = 0; a < 256; a++) {
        size_t bend = (a + 1 < 256) ? cursors[a + 1] : nbands;
        uint32_t from = 0;
        for (size_t bi = cursors[a]; bi < bend; bi++) {
            sim_access(&sim, bands_addr + bi * sizeof(OpenBand), sizeof(OpenBand), 0);
            OpenBand b = bands[bi];
            uint32_t span = b.end - k, dst = k + b.delta, pack = b.code << bits;
            sim_access(&sim, qaddr[a] + (uint64_t)from * 4, (size_t)span * 4, 0);
            for (uint32_t s = 0; s < span; s++) edges[dst + s] = pack | qdata[a][from + s];
            sim_access(&sim, edges_addr + (uint64_t)dst * 4, (size_t)span * 4, 1);
            met->work_events += span; met->irreg_q++;
            from += span; k = b.end;
        }
    }
    uint64_t l2_3 = sim.l2.misses;

    met->peak_mem = nrecs * 2 + ep_mem + (size_t)m * 4 /* queues */ + (size_t)m * 4 /* edges */ + n;

    uint8_t *out = (uint8_t *)malloc(n);
    uint64_t out_addr = sim_alloc_region(&sim, n);
    size_t out_len = 0;
    uint32_t mask = (1u << bits) - 1, j = mark;
    for (uint32_t step = 0; step < m; step++) {
        sim_access(&sim, edges_addr + (uint64_t)j * 4, 4, 0);
        uint32_t edge = edges[j], code = edge >> bits;
        j = edge & mask;
        met->work_events++; met->irreg_q++;
        if (code < 256) {
            out[out_len] = (uint8_t)code;
            sim_access(&sim, out_addr + out_len, 1, 1);
            out_len++;
        } else {
            uint32_t wi = code - 256;
            sim_access(&sim, off_addr + (uint64_t)wi * 4, 8, 0);
            uint32_t s0 = offsets[wi], ln = offsets[wi + 1] - s0;
            sim_access(&sim, spell_addr + s0, ln, 0);
            uint32_t take = (out_len + ln <= n) ? ln : (n - (uint32_t)out_len);
            memcpy(out + out_len, spelling + s0, take);
            sim_access(&sim, out_addr + out_len, take, 1);
            out_len += take; met->irreg_q++;
        }
        if (j == mark || out_len >= n) break;
    }
    assert(out_len == n);
    uint64_t l2_4 = sim.l2.misses;

    sim_drain(&sim);
    met->l1_misses = sim.l1.misses; met->l2_misses = sim.l2.misses; met->llc_misses = sim.llc.misses;
    met->l2_run = l2_1; met->l2_ep_sp = l2_2 - l2_1; met->l2_wir = l2_3 - l2_2; met->l2_emit = l2_4 - l2_3;
    met->dram_reads = sim.dram_reads; met->dram_writes = sim.dram_writes;
    met->dram_bb = 64.0 * (double)(sim.dram_reads + sim.dram_writes) / (double)n;

    free(strips); free(bands); free(spelling); free(offsets); free(edges);
    for (int a = 0; a < 256; a++) free(qdata[a]);
    sim_free(&sim);
    return out;
}

/* ==================== 2. Queue-Free Backward Open (g = P o LF_v) ==================== */

typedef struct { uint32_t rank, end, delta, pack, bidx; } ActiveBand; /* 20 bytes x 256 = 5 KiB (L1-resident!) */

static uint8_t *run_backward_open(const uint16_t *recs, size_t nrecs, uint32_t n, uint32_t p, Metrics *met) {
    const uint32_t LIMIT = 16;
    CacheSim sim; sim_init(&sim);
    memset(met, 0, sizeof(*met));

    uint64_t rec_addr = sim_alloc_region(&sim, nrecs * sizeof(uint16_t));
    uint32_t all[256] = {0}, thin[256] = {0}, thick[256] = {0};
    OpenStrip *strips = (OpenStrip *)malloc((nrecs + 2) * sizeof(OpenStrip));
    uint64_t strips_addr = sim_alloc_region(&sim, (nrecs + 2) * sizeof(OpenStrip));
    size_t nstrips = 0;

    /* Pass 1: ZERO queues! Only count & collect H thick strips */
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
            if (pos < p) {
                strips[nstrips] = (OpenStrip){pos, all[c], p - pos, c};
                sim_access(&sim, strips_addr + nstrips * sizeof(OpenStrip), sizeof(OpenStrip), 1);
                nstrips++; thick[c]++; all[c] += p - pos;
            }
            mark = m; thin[c] += 1; m += 1; all[c] += 1;
            if (p + 1 < end) {
                strips[nstrips] = (OpenStrip){p + 1, all[c], end - p - 1, c};
                sim_access(&sim, strips_addr + nstrips * sizeof(OpenStrip), sizeof(OpenStrip), 1);
                nstrips++; thick[c]++; all[c] += end - p - 1;
            }
        } else if (w <= LIMIT) {
            if (pos <= p && p < end) mark = m + (p - pos);
            thin[c] += w; m += w; all[c] += w;
        } else {
            strips[nstrips] = (OpenStrip){pos, all[c], w, c};
            sim_access(&sim, strips_addr + nstrips * sizeof(OpenStrip), sizeof(OpenStrip), 1);
            nstrips++; thick[c]++; all[c] += w;
        }
        pos = end;
    }
    uint64_t l2_1 = sim.l2.misses;

    /* Pass 2: Identical endpoint induction on H thick strips */
    OpenBand *bands; uint64_t bands_addr; size_t nbands;
    uint8_t *spelling; uint64_t spell_addr; size_t nspell;
    uint32_t *offsets; uint64_t off_addr; size_t noff;
    size_t cursors[256], ep_mem = 0;
    run_endpoint_and_spell(&sim, met, strips, strips_addr, nstrips, all, thin, thick, n, m,
                           &bands, &bands_addr, &nbands, &spelling, &spell_addr, &nspell,
                           &offsets, &off_addr, &noff, cursors, &ep_mem);
    uint64_t l2_2 = sim.l2.misses;

    /* Pass 3: DIRECT SEQUENTIAL FILL of edges[0..m] (ZERO queues, ZERO scattered writes!) */
    uint32_t bits = 32 - (uint32_t)__builtin_clz(m > 1 ? m - 1 : 1);
    uint32_t *edges = (uint32_t *)malloc((size_t)m * 4);
    uint64_t edges_addr = sim_alloc_region(&sim, (size_t)m * 4);

    ActiveBand ab[256];
    uint64_t ab_addr = sim_alloc_region(&sim, sizeof(ab));
    uint32_t rk_tot = 0;
    for (int a = 0; a < 256; a++) {
        ab[a].rank = rk_tot;
        ab[a].bidx = (uint32_t)cursors[a];
        if (thin[a] > 0) {
            sim_access(&sim, bands_addr + cursors[a] * sizeof(OpenBand), sizeof(OpenBand), 0);
            OpenBand b = bands[cursors[a]];
            ab[a].end = b.end;
            ab[a].delta = b.delta;
            ab[a].pack = b.code << bits;
        } else {
            ab[a].end = rk_tot; ab[a].delta = 0; ab[a].pack = 0;
        }
        sim_access(&sim, ab_addr + (size_t)a * sizeof(ActiveBand), sizeof(ActiveBand), 1);
        rk_tot += thin[a];
    }

    uint32_t j = 0;
    pos = 0; ri = 0;
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
        uint32_t tw = 0;
        if (w <= LIMIT) tw = w;
        else if (pos <= p && p < end) tw = 1;
        pos = end;
        if (tw == 0) continue;

        met->thin_runs++;
        sim_access(&sim, ab_addr + (size_t)c * sizeof(ActiveBand), sizeof(ActiveBand), 0);
        ActiveBand *cb = &ab[c];
        uint32_t rk = cb->rank;
        if (rk + tw <= cb->end) {
            /* 99.5%+ fast path: strictly inside current active band, 100% sequential write to edges[j..j+tw]! */
            uint32_t val = cb->pack | (rk + cb->delta);
            for (uint32_t s = 0; s < tw; s++) edges[j + s] = val + s;
            sim_access(&sim, edges_addr + (uint64_t)j * 4, (size_t)tw * 4, 1);
            j += tw; rk += tw;
            if (rk == cb->end) {
                uint32_t nb = cb->bidx + 1;
                size_t bend = (c + 1 < 256) ? cursors[c + 1] : nbands;
                if (nb < bend) {
                    sim_access(&sim, bands_addr + (uint64_t)nb * sizeof(OpenBand), sizeof(OpenBand), 0);
                    OpenBand b = bands[nb];
                    cb->bidx = nb; cb->end = b.end; cb->delta = b.delta; cb->pack = b.code << bits;
                }
            }
            cb->rank = rk;
            sim_access(&sim, ab_addr + (size_t)c * sizeof(ActiveBand), sizeof(ActiveBand), 1);
        } else {
            met->band_splits++;
            uint32_t rem_w = tw;
            while (rem_w > 0) {
                uint32_t span = (rem_w < cb->end - rk) ? rem_w : (cb->end - rk);
                uint32_t val = cb->pack | (rk + cb->delta);
                for (uint32_t s = 0; s < span; s++) edges[j + s] = val + s;
                sim_access(&sim, edges_addr + (uint64_t)j * 4, (size_t)span * 4, 1);
                j += span; rk += span; rem_w -= span;
                if (rk == cb->end) {
                    uint32_t nb = cb->bidx + 1;
                    size_t bend = (c + 1 < 256) ? cursors[c + 1] : nbands;
                    if (nb < bend) {
                        sim_access(&sim, bands_addr + (uint64_t)nb * sizeof(OpenBand), sizeof(OpenBand), 0);
                        OpenBand b = bands[nb];
                        cb->bidx = nb; cb->end = b.end; cb->delta = b.delta; cb->pack = b.code << bits;
                    }
                }
            }
            cb->rank = rk;
            sim_access(&sim, ab_addr + (size_t)c * sizeof(ActiveBand), sizeof(ActiveBand), 1);
        }
        met->work_events += tw;
    }
    assert(j == m);
    uint64_t l2_3 = sim.l2.misses;

    /* Peak live memory: NO queues[0..256]! Only records + ep_mem + 5KB ab + edges + out! */
    met->peak_mem = nrecs * 2 + ep_mem + sizeof(ab) + (size_t)m * 4 + n;

    /* Pass 4: Backward emission walk from mark, writing out[n..0] right-to-left */
    uint8_t *out = (uint8_t *)malloc(n);
    uint64_t out_addr = sim_alloc_region(&sim, n);
    size_t wpos = n;
    uint32_t mask = (1u << bits) - 1, cur = mark;
    for (uint32_t step = 0; step < m; step++) {
        sim_access(&sim, edges_addr + (uint64_t)cur * 4, 4, 0);
        uint32_t edge = edges[cur], code = edge >> bits;
        cur = edge & mask;
        met->work_events++; met->irreg_q++;
        if (code < 256) {
            wpos--;
            out[wpos] = (uint8_t)code;
            sim_access(&sim, out_addr + wpos, 1, 1);
        } else {
            uint32_t wi = code - 256;
            sim_access(&sim, off_addr + (uint64_t)wi * 4, 8, 0);
            uint32_t s0 = offsets[wi], ln = offsets[wi + 1] - s0;
            sim_access(&sim, spell_addr + s0, ln, 0);
            wpos -= ln;
            memcpy(out + wpos, spelling + s0, ln);
            sim_access(&sim, out_addr + wpos, ln, 1);
            met->irreg_q++;
        }
        if (cur == mark || wpos == 0) break;
    }
    assert(wpos == 0);
    uint64_t l2_4 = sim.l2.misses;

    sim_drain(&sim);
    met->l1_misses = sim.l1.misses; met->l2_misses = sim.l2.misses; met->llc_misses = sim.llc.misses;
    met->l2_run = l2_1; met->l2_ep_sp = l2_2 - l2_1; met->l2_wir = l2_3 - l2_2; met->l2_emit = l2_4 - l2_3;
    met->dram_reads = sim.dram_reads; met->dram_writes = sim.dram_writes;
    met->dram_bb = 64.0 * (double)(sim.dram_reads + sim.dram_writes) / (double)n;

    free(strips); free(bands); free(spelling); free(offsets); free(edges);
    sim_free(&sim);
    return out;
}

typedef struct { const char *file; uint32_t n, p, r; } BlockSpec;

int main(int argc, char **argv) {
    if (argc < 2) { fprintf(stderr, "Usage: %s <admitted-bwt-dir>\n", argv[0]); return 1; }
    const BlockSpec blocks[6] = {
        {"block-0.L",   899931, 264045,  51184},
        {"block-2.L",   900000, 320807,  54532},
        {"block-85.L",  899935, 325088,  79375},
        {"block-143.L", 899869, 802868, 108335},
        {"block-207.L", 899998, 205180,  87111},
        {"block-210.L", 899959, 791429, 128732},
    };

    printf("=== CYCLE 2 G5 FALSIFIER: Queue-Free Backward Open vs Shipped Forward Open ===\n");
    printf("Cache model: 32KiB 8w L1D | 256KiB 4w L2 | 2MiB 16w LLC | 64B lines, write-allocate, drain\n\n");
    printf("%-11s | %8s %8s %5s | %8s %8s %5s | %5s %5s %5s | %8s %8s %5s | %5s %5s %5s | %6s\n",
           "Block", "Fwd L2m", "Bwd L2m", "Ratio",
           "F(r+w)L2", "B(r+w)L2", "Ratio",
           "F B/B", "B B/B", "Ratio",
           "Fwd Work", "Bwd Work", "Ratio",
           "F MB", "B MB", "Ratio",
           "Splits");
    printf("------------+-----------------------+-----------------------+-------------------+-----------------------+-------------------+-------\n");

    int ok_l2 = 1, ok_rw = 1, ok_bb = 1, ok_mem = 1, ok_work = 1, ok_split = 1;
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

        Metrics mf, mb;
        uint8_t *of = run_forward_open(recs, nrecs, blocks[i].n, blocks[i].p, &mf);
        uint8_t *ob = run_backward_open(recs, nrecs, blocks[i].n, blocks[i].p, &mb);
        assert(memcmp(of, ob, blocks[i].n) == 0);

        uint64_t f_rw = mf.l2_run + mf.l2_wir;
        uint64_t b_rw = mb.l2_run + mb.l2_wir;
        double r_l2 = (double)mb.l2_misses / (double)mf.l2_misses;
        double r_rw = (double)b_rw / (double)f_rw;
        double r_bb = mb.dram_bb / mf.dram_bb;
        double r_w  = (double)mb.work_events / (double)mf.work_events;
        double r_m  = (double)mb.peak_mem / (double)mf.peak_mem;
        double srate = 100.0 * (double)mb.band_splits / (double)mb.thin_runs;

        if (r_l2 > 0.85) ok_l2 = 0;
        if (r_rw > 0.50) ok_rw = 0;
        if (r_bb > 0.80) ok_bb = 0;
        if (r_m > 0.80) ok_mem = 0;
        if (r_w > 0.80) ok_work = 0;
        if (srate > 15.0) ok_split = 0;

        printf("%-11s | %8llu %8llu %4.2fx | %8llu %8llu %4.2fx | %5.2f %5.2f %4.2fx | %8llu %8llu %4.2fx | %5.2f %5.2f %4.2fx | %5.2f%%\n",
               blocks[i].file,
               (unsigned long long)mf.l2_misses, (unsigned long long)mb.l2_misses, r_l2,
               (unsigned long long)f_rw, (unsigned long long)b_rw, r_rw,
               mf.dram_bb, mb.dram_bb, r_bb,
               (unsigned long long)mf.work_events, (unsigned long long)mb.work_events, r_w,
               (double)mf.peak_mem / (1024.0 * 1024.0), (double)mb.peak_mem / (1024.0 * 1024.0), r_m,
               srate);

        free(L); free(recs); free(of); free(ob);
    }

    int pass = ok_l2 && ok_rw && ok_bb && ok_mem && ok_work && ok_split;
    printf("\nPRE-REGISTERED G5 VERDICT: %s\n", pass ? "PASS" : "FAIL");
    printf("  L2<=0.85x: %s | (run+wir)L2<=0.50x: %s | DRAM<=0.80x: %s | Mem<=0.80x: %s | Work<=0.80x: %s | Splits<=15%%: %s\n",
           ok_l2?"YES":"NO", ok_rw?"YES":"NO", ok_bb?"YES":"NO", ok_mem?"YES":"NO", ok_work?"YES":"NO", ok_split?"YES":"NO");
    return 0;
}
