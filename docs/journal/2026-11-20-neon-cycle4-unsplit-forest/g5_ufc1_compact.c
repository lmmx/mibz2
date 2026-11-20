/*
 * Cycle 4 · Compact UFC-1+Open (Compact 1B-width R1 + 8-Way MLP + Inline-5B u64 Open Edges)
 * Standalone diagnostic + G5 comparison against Shipped Forward Open on all 6 Wikidata blocks.
 *
 * Pre-registered PASS/FAIL rule (same as g5_ufc.c, fixed before run):
 *   PASS G5 iff on ALL 6 blocks vs Shipped Open:
 *     (1) Byte-exact  (2) L2m <= 0.85x  (3) DRAM B/B <= 0.90x  (4) PeakMem <= 1.15x  (5) VM time <= 0.90x
 */
#define _POSIX_C_SOURCE 200809L
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <assert.h>
#include <time.h>

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
    level_init(&s->l1, 64, 8); level_init(&s->l2, 1024, 4); level_init(&s->llc, 2048, 16);
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
            way[w].lru = lv->clock; if (is_write) way[w].dirty = 1;
            *evicted_dirty = 0; return 1;
        }
        if (!way[w].valid) { victim = w; min_lru = 0; }
        else if (min_lru > 0 && way[w].lru < min_lru) { min_lru = way[w].lru; victim = w; }
    }
    lv->misses++;
    if (way[victim].valid && way[victim].dirty) {
        *evicted_dirty = 1; *evicted_line = (way[victim].tag << shift) | (uint64_t)set; lv->writebacks++;
    } else *evicted_dirty = 0;
    way[victim].valid = 1; way[victim].tag = tag; way[victim].dirty = (uint8_t)is_write; way[victim].lru = lv->clock;
    return 0;
}
static inline void sim_touch_line(CacheSim *s, uint64_t line_addr, int is_write) {
    int ev1 = 0, ev2 = 0, ev3 = 0; uint64_t ev1_a = 0, ev2_a = 0, ev3_a = 0;
    if (!level_access(&s->l1, line_addr, is_write, &ev1, &ev1_a)) {
        if (!level_access(&s->l2, line_addr, 0, &ev2, &ev2_a)) {
            if (!level_access(&s->llc, line_addr, 0, &ev3, &ev3_a)) { s->dram_reads++; if (ev3) s->dram_writes++; }
            if (ev2) { int ev3b = 0; uint64_t ev3b_a = 0; if (!level_access(&s->llc, ev2_a, 1, &ev3b, &ev3b_a)) { s->dram_reads++; if (ev3b) s->dram_writes++; } }
        }
        if (ev1) {
            int ev2b = 0; uint64_t ev2b_a = 0;
            if (!level_access(&s->l2, ev1_a, 1, &ev2b, &ev2b_a)) {
                int ev3c = 0; uint64_t ev3c_a = 0; if (!level_access(&s->llc, ev1_a, 0, &ev3c, &ev3c_a)) { s->dram_reads++; if (ev3c) s->dram_writes++; }
                if (ev2b) { int ev3d = 0; uint64_t ev3d_a = 0; if (!level_access(&s->llc, ev2b_a, 1, &ev3d, &ev3d_a)) { s->dram_reads++; if (ev3d) s->dram_writes++; } }
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
                int ev3 = 0; uint64_t ev3_a = 0; if (!level_access(&s->llc, line, 0, &ev3, &ev3_a)) { s->dram_reads++; if (ev3) s->dram_writes++; }
                if (ev2) { int ev3b = 0; uint64_t ev3b_a = 0; if (!level_access(&s->llc, ev2_a, 1, &ev3b, &ev3b_a)) { s->dram_reads++; if (ev3b) s->dram_writes++; } }
            }
            s->l1.lines[i].dirty = 0;
        }
    }
    for (int i = 0; i < s->l2.sets * s->l2.ways; i++) {
        if (s->l2.lines[i].valid && s->l2.lines[i].dirty) {
            uint64_t line = (s->l2.lines[i].tag << __builtin_ctz(s->l2.sets)) | (uint64_t)(i / s->l2.ways);
            int ev3 = 0; uint64_t ev3_a = 0; if (!level_access(&s->llc, line, 1, &ev3, &ev3_a)) { s->dram_reads++; if (ev3) s->dram_writes++; }
            s->l2.lines[i].dirty = 0;
        }
    }
    for (int i = 0; i < s->llc.sets * s->llc.ways; i++) {
        if (s->llc.lines[i].valid && s->llc.lines[i].dirty) { s->dram_writes++; s->llc.lines[i].dirty = 0; }
    }
}
#else
static inline void sim_access(CacheSim *s, uint64_t addr, size_t len, int is_write) { (void)s; (void)addr; (void)len; (void)is_write; }
static void sim_drain(CacheSim *s) { (void)s; }
#endif

typedef struct {
    uint64_t l2_misses, llc_misses, dram_bytes, work_events;
    size_t peak_mem;
    uint64_t l2_p1, l2_p2, l2_p3, l2_p4, l2_r1_pass13, l2_r1_pass4;
    double ms_p1, ms_p2, ms_p3, ms_p4, ms_r1_13, ms_r1_4;
    uint32_t m_ret, extra2;
} Metrics;

static inline double now_ms(void) {
    struct timespec ts; clock_gettime(CLOCK_MONOTONIC, &ts);
    return ts.tv_sec * 1000.0 + ts.tv_nsec * 1e-6;
}

static uint16_t *column_to_records(const uint8_t *L, size_t n, size_t *out_nrecs) {
    size_t cap = 65536, nr = 0; uint16_t *recs = (uint16_t *)malloc(cap * 2);
    size_t i = 0;
    while (i < n) {
        uint8_t c = L[i]; size_t j = i + 1; while (j < n && L[j] == c) j++;
        size_t left = j - i;
        while (left > 0) {
            size_t take = left > 127 ? 127 : left;
            if (nr == cap) { cap *= 2; recs = (uint16_t *)realloc(recs, cap * 2); }
            recs[nr++] = (uint16_t)(((uint16_t)c << 8) | (uint16_t)take); left -= take;
        }
        i = j;
    }
    *out_nrecs = nr; return recs;
}
static uint8_t *dense_reference(const uint8_t *L, uint32_t n, uint32_t p) {
    uint32_t cnt[256] = {0}, C[256]; for (uint32_t i = 0; i < n; i++) cnt[L[i]]++;
    uint32_t t = 0; for (int c = 0; c < 256; c++) { C[c] = t; t += cnt[c]; }
    uint32_t *lf = (uint32_t *)malloc(n * 4); for (uint32_t i = 0; i < n; i++) lf[i] = C[L[i]]++;
    uint8_t *out = (uint8_t *)malloc(n); uint32_t x = p;
    for (uint32_t i = 0; i < n; i++) { out[n - 1 - i] = L[x]; x = lf[x]; }
    free(lf); return out;
}

typedef struct { uint32_t prev[2], next[2], width, word; } EpNode;
typedef struct { uint32_t left, right, len, first; } EpRule;
static inline void ep_cut(CacheSim *sim, uint64_t na, EpNode *nodes, uint32_t id, int side) {
    uint32_t pr = nodes[id].prev[side], nx = nodes[id].next[side];
    nodes[pr].next[side] = nx; nodes[nx].prev[side] = pr;
    sim_access(sim, na + (uint64_t)pr * sizeof(EpNode), sizeof(EpNode), 1);
    sim_access(sim, na + (uint64_t)nx * sizeof(EpNode), sizeof(EpNode), 1);
}
static inline void ep_insert(CacheSim *sim, uint64_t na, EpNode *nodes, uint32_t at, uint32_t id, int side) {
    sim_access(sim, na + (uint64_t)at * sizeof(EpNode), sizeof(EpNode), 0);
    uint32_t nx = nodes[at].next[side];
    nodes[id].prev[side] = at; nodes[id].next[side] = nx; nodes[at].next[side] = id; nodes[nx].prev[side] = id;
    sim_access(sim, na + (uint64_t)id * sizeof(EpNode), sizeof(EpNode), 1);
    sim_access(sim, na + (uint64_t)at * sizeof(EpNode), sizeof(EpNode), 1);
    sim_access(sim, na + (uint64_t)nx * sizeof(EpNode), sizeof(EpNode), 1);
}

/* ==================== Shipped Forward Open (exact baseline) ==================== */
typedef struct { uint32_t from, to, width; uint8_t byte; } OpenStrip;
typedef struct { uint32_t end, delta, code; } OpenBand;

static uint8_t *run_forward_open(const uint16_t *recs, size_t nrecs, uint32_t n, uint32_t p, Metrics *met) {
    const uint32_t LIMIT = 16;
    CacheSim sim; sim_init(&sim); memset(met, 0, sizeof(*met));
    double t0 = now_ms();
    uint64_t rec_addr = sim_alloc_region(&sim, nrecs * 2);
    uint32_t all[256] = {0}, thin[256] = {0}, thick[256] = {0};
    OpenStrip *strips = (OpenStrip *)calloc(nrecs + 2, sizeof(OpenStrip));
    uint64_t strips_addr = sim_alloc_region(&sim, (nrecs + 2) * sizeof(OpenStrip));
    size_t nstrips = 0;
    uint32_t *qdata[256]; uint32_t qlen[256] = {0}, qcap[256] = {0}; uint64_t qaddr[256];
    for (int a = 0; a < 256; a++) { qcap[a] = 4096; qdata[a] = (uint32_t *)malloc(qcap[a] * 4); qaddr[a] = sim_alloc_region(&sim, 250000 * 4); }
    uint32_t m = 0, mark = 0, pos = 0; size_t ri = 0;
    while (ri < nrecs) {
        sim_access(&sim, rec_addr + ri * 2, 2, 0);
        uint16_t rec = recs[ri++]; uint8_t c = (uint8_t)(rec >> 8); uint32_t w = rec & 255;
        while (ri < nrecs) { sim_access(&sim, rec_addr + ri * 2, 2, 0); if ((recs[ri] >> 8) != c) break; w += recs[ri] & 255; ri++; }
        met->work_events++; uint32_t end = pos + w;
        uint32_t sx[3], sw[3]; int sk[3], ns = 0;
        if (w > LIMIT && pos <= p && p < end) {
            if (pos < p) { sx[ns] = pos; sw[ns] = p - pos; sk[ns++] = 0; }
            sx[ns] = p; sw[ns] = 1; sk[ns++] = 1;
            if (p + 1 < end) { sx[ns] = p + 1; sw[ns] = end - p - 1; sk[ns++] = 0; }
        } else { sx[ns] = pos; sw[ns] = w; sk[ns++] = (w <= LIMIT); }
        for (int s = 0; s < ns; s++) {
            if (sk[s]) {
                if (sx[s] <= p && p < sx[s] + sw[s]) mark = m + (p - sx[s]);
                if (qlen[c] + sw[s] > qcap[c]) { while (qlen[c] + sw[s] > qcap[c]) qcap[c] *= 2; qdata[c] = (uint32_t *)realloc(qdata[c], qcap[c] * 4); }
                for (uint32_t u = 0; u < sw[s]; u++) { qdata[c][qlen[c]] = m + u; sim_access(&sim, qaddr[c] + (uint64_t)qlen[c] * 4, 4, 1); qlen[c]++; met->work_events++; }
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
    met->l2_p1 = sim.l2.misses; double t1 = now_ms(); met->ms_p1 = t1 - t0;

    uint32_t first[256], write_pos[256], tot = 0, ttot = 0;
    for (int a = 0; a < 256; a++) { first[a] = tot; tot += all[a]; write_pos[a] = ttot; ttot += thick[a]; }
    uint32_t *image_order = (uint32_t *)malloc((nstrips + 1) * 4);
    uint64_t imgord_addr = sim_alloc_region(&sim, (nstrips + 1) * 4);
    for (size_t i = 0; i < nstrips; i++) {
        sim_access(&sim, strips_addr + i * sizeof(OpenStrip), sizeof(OpenStrip), 0);
        uint8_t a = strips[i].byte; strips[i].to += first[a];
        sim_access(&sim, strips_addr + i * sizeof(OpenStrip), sizeof(OpenStrip), 1);
        uint32_t wp = write_pos[a]++; image_order[wp] = (uint32_t)i;
        sim_access(&sim, imgord_addr + (uint64_t)wp * 4, 4, 1);
    }
    size_t max_nodes = 3 * nstrips + 5;
    EpNode *nodes = (EpNode *)calloc(max_nodes, sizeof(EpNode));
    uint64_t nodes_addr = sim_alloc_region(&sim, max_nodes * sizeof(EpNode));
    uint32_t nnodes = 1;
    uint32_t *ord0 = (uint32_t *)malloc(max_nodes * 4), *ord1 = (uint32_t *)malloc(max_nodes * 4);
    uint32_t *term0 = (uint32_t *)malloc(max_nodes * 4), *term1 = (uint32_t *)malloc(max_nodes * 4);
    size_t n_o0 = 0, n_o1 = 0, n_t0 = 0, n_t1 = 0;
    for (size_t i = 0; i < nstrips; i++) { nodes[nnodes].width = strips[i].width; nodes[nnodes].word = (uint32_t)strips[i].byte + 1; nnodes++; }
    uint32_t x = 0;
    for (size_t i = 0; i < nstrips; i++) { if (x < strips[i].from) { uint32_t id = nnodes++; nodes[id].width = strips[i].from - x; ord0[n_o0++] = id; term1[n_t1++] = id; } ord0[n_o0++] = (uint32_t)i + 1; x = strips[i].from + strips[i].width; }
    if (x < n) { uint32_t id = nnodes++; nodes[id].width = n - x; ord0[n_o0++] = id; term1[n_t1++] = id; }
    x = 0;
    for (size_t k = 0; k < nstrips; k++) { uint32_t i = image_order[k]; if (x < strips[i].to) { uint32_t id = nnodes++; nodes[id].width = strips[i].to - x; ord1[n_o1++] = id; term0[n_t0++] = id; } ord1[n_o1++] = i + 1; x = strips[i].to + strips[i].width; }
    if (x < n) { uint32_t id = nnodes++; nodes[id].width = n - x; ord1[n_o1++] = id; term0[n_t0++] = id; }
    for (int side = 0; side < 2; side++) {
        uint32_t prev = 0; uint32_t *tarr = side == 0 ? term0 : term1; size_t nt = side == 0 ? n_t0 : n_t1; uint32_t *oarr = side == 0 ? ord0 : ord1; size_t no = side == 0 ? n_o0 : n_o1;
        for (size_t k = 0; k < nt; k++) { uint32_t id = tarr[k]; nodes[prev].next[side] = id; nodes[id].prev[side] = prev; prev = id; }
        for (size_t k = 0; k < no; k++) { uint32_t id = oarr[k]; nodes[prev].next[side] = id; nodes[id].prev[side] = prev; prev = id; }
        nodes[prev].next[side] = 0; nodes[0].prev[side] = prev;
    }
    size_t cap_rules = 65536; EpRule *rules = (EpRule *)malloc(cap_rules * sizeof(EpRule));
    uint64_t rules_addr = sim_alloc_region(&sim, 262144 * sizeof(EpRule));
    uint32_t nrules = 0, rem = n;
    while (rem > 0) {
        uint32_t ix = nodes[0].prev[0], iy = nodes[0].prev[1];
        sim_access(&sim, nodes_addr + (uint64_t)ix * sizeof(EpNode), sizeof(EpNode), 0);
        sim_access(&sim, nodes_addr + (uint64_t)iy * sizeof(EpNode), sizeof(EpNode), 0);
        EpNode a = nodes[ix], b = nodes[iy]; uint32_t w = a.width < b.width ? a.width : b.width;
        rem -= w; met->work_events++;
        if (ix == iy) { ep_cut(&sim, nodes_addr, nodes, ix, 0); ep_cut(&sim, nodes_addr, nodes, ix, 1); continue; }
        uint32_t word;
        if (a.word == 0) word = b.word; else if (b.word == 0) word = a.word;
        else {
            if (nrules == cap_rules) { cap_rules *= 2; rules = (EpRule *)realloc(rules, cap_rules * sizeof(EpRule)); }
            uint32_t la = a.word <= 256 ? 1 : rules[a.word - 257].len, lb = b.word <= 256 ? 1 : rules[b.word - 257].len;
            rules[nrules] = (EpRule){a.word, b.word, la + lb, UINT32_MAX};
            sim_access(&sim, rules_addr + (uint64_t)nrules * sizeof(EpRule), sizeof(EpRule), 1);
            word = nrules + 257; nrules++;
        }
        if (a.width <= b.width) {
            ep_cut(&sim, nodes_addr, nodes, ix, 0); ep_insert(&sim, nodes_addr, nodes, iy, ix, 0); nodes[ix].word = word;
            if (a.width == b.width) { ep_cut(&sim, nodes_addr, nodes, iy, 0); ep_cut(&sim, nodes_addr, nodes, iy, 1); } else nodes[iy].width -= w;
        } else { ep_cut(&sim, nodes_addr, nodes, iy, 1); ep_insert(&sim, nodes_addr, nodes, ix, iy, 1); nodes[iy].word = word; nodes[ix].width -= w; }
    }
    uint32_t *bottom = (uint32_t *)malloc(nnodes * 4); uint32_t id = nodes[0].next[1]; x = 0;
    while (id != 0) { bottom[id] = x; x += nodes[id].width; id = nodes[id].next[1]; }
    size_t cap_bands = nnodes + 256, nbands = 0, cap_spell = 1 << 18, nspell = 0, noff = 0;
    OpenBand *bands = (OpenBand *)malloc(cap_bands * sizeof(OpenBand));
    uint64_t bands_addr = sim_alloc_region(&sim, (cap_bands + 1024) * sizeof(OpenBand));
    uint8_t *spelling = (uint8_t *)malloc(cap_spell); uint64_t spell_addr = sim_alloc_region(&sim, 1 << 21);
    uint32_t *offsets = (uint32_t *)malloc((cap_bands + 2) * 4); uint64_t off_addr = sim_alloc_region(&sim, (cap_bands + 1024) * 4);
    offsets[noff++] = 0; uint32_t *stack = (uint32_t *)malloc(65536 * 4);
    id = nodes[0].next[0]; uint32_t in_node = 0, k = 0;
    for (int a = 0; a < 256; a++) {
        uint32_t end = k + thin[a];
        while (k < end) {
            EpNode nd = nodes[id]; uint32_t w = (end - k) < (nd.width - in_node) ? (end - k) : (nd.width - in_node);
            uint32_t dst = bottom[id] + in_node, code;
            if (nd.word == 0) code = (uint32_t)a;
            else {
                size_t sp = 0; stack[sp++] = nd.word;
                while (sp > 0) {
                    uint32_t wid = stack[--sp];
                    if (wid <= 256) { if (nspell + 2 >= cap_spell) { cap_spell *= 2; spelling = (uint8_t *)realloc(spelling, cap_spell); } spelling[nspell] = (uint8_t)(wid - 1); sim_access(&sim, spell_addr + nspell, 1, 1); nspell++; }
                    else {
                        EpRule *rl = &rules[wid - 257];
                        if (rl->first == UINT32_MAX) { rl->first = (uint32_t)nspell; stack[sp++] = rl->right; stack[sp++] = rl->left; }
                        else { uint32_t st = rl->first, ln = rl->len; if (nspell + ln + 2 >= cap_spell) { while (nspell + ln + 2 >= cap_spell) cap_spell *= 2; spelling = (uint8_t *)realloc(spelling, cap_spell); } sim_access(&sim, spell_addr + st, ln, 0); memcpy(spelling + nspell, spelling + st, ln); sim_access(&sim, spell_addr + nspell, ln, 1); nspell += ln; }
                    }
                }
                if (nspell + 2 >= cap_spell) { cap_spell *= 2; spelling = (uint8_t *)realloc(spelling, cap_spell); }
                spelling[nspell] = (uint8_t)a; sim_access(&sim, spell_addr + nspell, 1, 1); nspell++;
                offsets[noff] = (uint32_t)nspell; sim_access(&sim, off_addr + noff * 4, 4, 1); noff++; code = 254 + (uint32_t)noff;
            }
            if (nbands == cap_bands) { cap_bands *= 2; bands = (OpenBand *)realloc(bands, cap_bands * sizeof(OpenBand)); offsets = (uint32_t *)realloc(offsets, (cap_bands + 2) * 4); }
            bands[nbands] = (OpenBand){k + w, dst - k, code}; sim_access(&sim, bands_addr + nbands * sizeof(OpenBand), sizeof(OpenBand), 1);
            nbands++; in_node += w; k += w; if (in_node == nd.width) { id = nd.next[0]; in_node = 0; }
        }
    }
    size_t ep_mem = nstrips * sizeof(OpenStrip) + nnodes * sizeof(EpNode) + nrules * sizeof(EpRule) + nbands * sizeof(OpenBand) + nspell + noff * 4;
    free(image_order); free(nodes); free(ord0); free(ord1); free(term0); free(term1); free(rules); free(bottom); free(stack);
    met->l2_p2 = sim.l2.misses - met->l2_p1; double t2 = now_ms(); met->ms_p2 = t2 - t1;

    uint32_t bits = 0; while ((1u << bits) < m) bits++; uint32_t mask = (1u << bits) - 1;
    uint32_t *edges = (uint32_t *)malloc((size_t)m * 4); uint64_t edges_addr = sim_alloc_region(&sim, (size_t)m * 4);
    k = 0; size_t bi = 0;
    for (int a = 0; a < 256; a++) {
        uint32_t ql = qlen[a], *qd = qdata[a], qi = 0;
        while (qi < ql) {
            sim_access(&sim, bands_addr + bi * sizeof(OpenBand), sizeof(OpenBand), 0);
            OpenBand b = bands[bi]; uint32_t w = b.end - k, dst = k + b.delta, pack = b.code << bits;
            sim_access(&sim, qaddr[a] + (uint64_t)qi * 4, (size_t)w * 4, 0);
            for (uint32_t u = 0; u < w; u++) edges[dst + u] = pack | qd[qi + u];
            sim_access(&sim, edges_addr + (uint64_t)dst * 4, (size_t)w * 4, 1);
            met->work_events += w; k += w; qi += w; bi++;
        }
    }
    met->l2_p3 = sim.l2.misses - met->l2_p1 - met->l2_p2; double t3 = now_ms(); met->ms_p3 = t3 - t2;

    uint8_t *out = (uint8_t *)malloc(n); uint64_t out_addr = sim_alloc_region(&sim, n);
    uint32_t out_pos = 0, j = mark;
    for (uint32_t step = 0; step < m && out_pos < n; step++) {
        sim_access(&sim, edges_addr + (uint64_t)j * 4, 4, 0);
        uint32_t edge = edges[j], code = edge >> bits; j = edge & mask;
        if (code < 256) { out[out_pos] = (uint8_t)code; sim_access(&sim, out_addr + out_pos, 1, 1); out_pos++; }
        else {
            uint32_t wi = code - 256; sim_access(&sim, off_addr + (uint64_t)wi * 4, 8, 0);
            uint32_t st = offsets[wi], ln = offsets[wi + 1] - st; if (out_pos + ln > n) ln = n - out_pos;
            sim_access(&sim, spell_addr + st, ln, 0); memcpy(out + out_pos, spelling + st, ln); sim_access(&sim, out_addr + out_pos, ln, 1); out_pos += ln;
        }
        met->work_events++;
    }
    met->l2_p4 = sim.l2.misses - met->l2_p1 - met->l2_p2 - met->l2_p3; double t4 = now_ms(); met->ms_p4 = t4 - t3;
    sim_drain(&sim);
    met->l2_misses = sim.l2.misses; met->llc_misses = sim.llc.misses;
    met->dram_bytes = (sim.dram_reads + sim.dram_writes) * 64ULL;
    met->peak_mem = nrecs * 2 + (size_t)m * 8 + ep_mem + n; met->m_ret = m;
    for (int a = 0; a < 256; a++) free(qdata[a]);
    free(strips); free(bands); free(spelling); free(offsets); free(edges); sim_free(&sim);
    return out;
}

/* ==================== Compact UFC-1 + Open (1B widths + Inline-5B u64 edges + 8-Way MLP) ==================== */

#define F_SPLIT_BIT (1u << 31)
#define F_DSMALL_MAX 31u

typedef struct { uint32_t key, val; } BigDEntry;
static inline void bigd_insert(BigDEntry *tab, uint32_t mask, uint32_t key, uint32_t val, CacheSim *sim, uint64_t addr) {
    uint32_t h = (key * 2654435761u) & mask;
    while (tab[h].key != UINT32_MAX) h = (h + 1) & mask;
    tab[h].key = key; tab[h].val = val;
    sim_access(sim, addr + (uint64_t)h * sizeof(BigDEntry), sizeof(BigDEntry), 1);
}
static inline uint32_t bigd_get(const BigDEntry *tab, uint32_t mask, uint32_t key, CacheSim *sim, uint64_t addr) {
    uint32_t h = (key * 2654435761u) & mask;
    for (;;) {
        sim_access(sim, addr + (uint64_t)h * sizeof(BigDEntry), sizeof(BigDEntry), 0);
        if (tab[h].key == key) return tab[h].val;
        h = (h + 1) & mask;
    }
}

static uint8_t *run_ufc1_compact(const uint16_t *recs, size_t nrecs, uint32_t n, uint32_t p, Metrics *met) {
    const uint32_t LIMIT = 16;
    CacheSim sim; sim_init(&sim); memset(met, 0, sizeof(*met));
    double t0 = now_ms();

    uint64_t rec_addr = sim_alloc_region(&sim, nrecs * 2);

    /* Pass 1: Compact 2 B/run: l_cw[k] = (c << 8) | min(w, 255), with tiny big_lw[] for w >= 255! */
    uint16_t *l_cw = (uint16_t *)malloc((nrecs + 2) * 2);
    uint64_t l_cw_addr = sim_alloc_region(&sim, (nrecs + 2) * 2);
    uint32_t *big_lw = (uint32_t *)malloc(4096 * 4); uint32_t nbig_lw = 0, cap_blw = 4096;
    uint32_t sym_rcnt[256] = {0};
    uint32_t r = 0, pos = 0, p_lrun = 0, p_in_run = 0;
    size_t ri = 0;
    while (ri < nrecs) {
        sim_access(&sim, rec_addr + ri * 2, 2, 0);
        uint16_t rec = recs[ri++]; uint8_t c = (uint8_t)(rec >> 8); uint32_t w = rec & 255;
        while (ri < nrecs) { sim_access(&sim, rec_addr + ri * 2, 2, 0); if ((recs[ri] >> 8) != c) break; w += recs[ri] & 255; ri++; }
        if (pos <= p && p < pos + w) { p_lrun = r; p_in_run = p - pos; }
        if (w < 255) { l_cw[r] = (uint16_t)(((uint16_t)c << 8) | w); }
        else {
            l_cw[r] = (uint16_t)(((uint16_t)c << 8) | 255u);
            if (nbig_lw == cap_blw) { cap_blw *= 2; big_lw = (uint32_t *)realloc(big_lw, cap_blw * 4); }
            big_lw[nbig_lw++] = w;
        }
        sim_access(&sim, l_cw_addr + (uint64_t)r * 2, 2, 1);
        sym_rcnt[c]++; r++; pos += w; met->work_events++;
    }
    assert(pos == n && r < (1u << 18));

    /* Pass 2: f_of_l[k] (4r sequential) + f_w8[fi] (1r across 256 streams — 4x smaller!) */
    uint32_t sym_rend[256], cur_off[256], tt = 0;
    for (int c = 0; c < 256; c++) { cur_off[c] = tt; tt += sym_rcnt[c]; sym_rend[c] = tt; }
    uint32_t *f_of_l = (uint32_t *)malloc((size_t)r * 4);
    uint8_t  *f_w8   = (uint8_t  *)malloc((size_t)r);
    uint64_t f_of_l_addr = sim_alloc_region(&sim, (size_t)r * 4);
    uint64_t f_w8_addr   = sim_alloc_region(&sim, (size_t)r);
    uint32_t bfw_mask = 2047;
    while (bfw_mask < nbig_lw * 4) bfw_mask = bfw_mask * 2 + 1;
    BigDEntry *big_fw = (BigDEntry *)malloc((bfw_mask + 1) * sizeof(BigDEntry));
    for (uint32_t i = 0; i <= bfw_mask; i++) big_fw[i].key = UINT32_MAX;
    uint64_t big_fw_addr = sim_alloc_region(&sim, (bfw_mask + 1) * sizeof(BigDEntry));

    uint32_t bi_lw = 0;
    for (uint32_t k = 0; k < r; k++) {
        sim_access(&sim, l_cw_addr + (uint64_t)k * 2, 2, 0);
        uint16_t cw = l_cw[k]; uint8_t c = (uint8_t)(cw >> 8); uint32_t w = cw & 255;
        if (w == 255) w = big_lw[bi_lw++];
        uint32_t fi = cur_off[c]++;
        f_of_l[k] = fi;
        sim_access(&sim, f_of_l_addr + (uint64_t)k * 4, 4, 1);
        if (w < 255) { f_w8[fi] = (uint8_t)w; }
        else { f_w8[fi] = 255; bigd_insert(big_fw, bfw_mask, fi, w, &sim, big_fw_addr); }
        sim_access(&sim, f_w8_addr + fi, 1, 1);
        met->work_events++;
    }
    uint32_t p_fi = f_of_l[p_lrun];

    /* Pass 3: Linear 2-pointer merge -> 4B/run forest[0..r] + tiny big_d hash + compact RawArc20 */
    uint32_t *forest = (uint32_t *)malloc((size_t)r * 4);
    uint64_t forest_addr = sim_alloc_region(&sim, (size_t)r * 4);
    uint32_t bd_mask = 65535;
    BigDEntry *big_d = (BigDEntry *)malloc((bd_mask + 1) * sizeof(BigDEntry));
    for (uint32_t i = 0; i <= bd_mask; i++) big_d[i].key = UINT32_MAX;
    uint64_t big_d_addr = sim_alloc_region(&sim, 262144 * sizeof(BigDEntry));
    uint32_t nbig_d = 0;

    uint32_t cap_K = (r / 4) + 256, nK = 0;
    uint32_t *k_new_from = (uint32_t *)malloc((size_t)cap_K * 4);
    uint64_t knf_addr = sim_alloc_region(&sim, (size_t)cap_K * 4);
    uint32_t cap_arcs = (r / 2) + 512, narcs = 0;
    typedef struct { uint32_t kid_sym, sub_off, ow, v_fi, in_v; } RawArc20;
    RawArc20 *raw_arcs = (RawArc20 *)malloc((size_t)cap_arcs * sizeof(RawArc20));
    uint64_t arcs_addr = sim_alloc_region(&sim, (size_t)cap_arcs * sizeof(RawArc20));

    uint32_t v = 0, v_blw = 0, v_start = 0, v_w = l_cw[0] & 255;
    if (v_w == 255) v_w = big_lw[v_blw++];
    uint32_t v_end = v_w, t0_pos = 0, new_n = 0;
    int sym_c = 0;

    for (uint32_t fi = 0; fi < r; fi++) {
        while (fi >= sym_rend[sym_c]) sym_c++;
        sim_access(&sim, f_w8_addr + fi, 1, 0);
        uint32_t w = f_w8[fi];
        if (w == 255) w = bigd_get(big_fw, bfw_mask, fi, &sim, big_fw_addr);
        uint32_t t1_pos = t0_pos + w;
        while (v_end <= t0_pos) {
            v++; v_start = v_end;
            sim_access(&sim, l_cw_addr + (uint64_t)v * 2, 2, 0);
            uint32_t nw = l_cw[v] & 255; if (nw == 255) nw = big_lw[v_blw++];
            v_end = v_start + nw;
        }
        if (v_end >= t1_pos) {
            sim_access(&sim, f_of_l_addr + (uint64_t)v * 4, 4, 0);
            uint32_t v_fi = f_of_l[v];
            uint32_t d = t0_pos - v_start;
            uint32_t ds = (d < F_DSMALL_MAX) ? d : F_DSMALL_MAX;
            if (ds == F_DSMALL_MAX) {
                nbig_d++;
                if (nbig_d * 2 >= bd_mask) {
                    uint32_t nmask = bd_mask * 2 + 1;
                    BigDEntry *ntab = (BigDEntry *)malloc((nmask + 1) * sizeof(BigDEntry));
                    for (uint32_t z = 0; z <= nmask; z++) ntab[z].key = UINT32_MAX;
                    for (uint32_t z = 0; z <= bd_mask; z++) {
                        if (big_d[z].key != UINT32_MAX) bigd_insert(ntab, nmask, big_d[z].key, big_d[z].val, &sim, big_d_addr);
                    }
                    free(big_d); big_d = ntab; bd_mask = nmask;
                }
                bigd_insert(big_d, bd_mask, fi, d, &sim, big_d_addr);
            }
            forest[fi] = v_fi | (ds << 18) | ((uint32_t)sym_c << 23);
            sim_access(&sim, forest_addr + (uint64_t)fi * 4, 4, 1);
        } else {
            if (nK == cap_K) { cap_K *= 2; k_new_from = (uint32_t *)realloc(k_new_from, (size_t)cap_K * 4); }
            uint32_t kid = nK++;
            k_new_from[kid] = new_n; sim_access(&sim, knf_addr + (uint64_t)kid * 4, 4, 1);
            new_n += w;
            forest[fi] = F_SPLIT_BIT | kid | ((uint32_t)sym_c << 23);
            sim_access(&sim, forest_addr + (uint64_t)fi * 4, 4, 1);
            for (;;) {
                uint32_t ov0 = t0_pos > v_start ? t0_pos : v_start;
                uint32_t ov1 = t1_pos < v_end ? t1_pos : v_end;
                sim_access(&sim, f_of_l_addr + (uint64_t)v * 4, 4, 0);
                if (narcs == cap_arcs) { cap_arcs *= 2; raw_arcs = (RawArc20 *)realloc(raw_arcs, (size_t)cap_arcs * sizeof(RawArc20)); }
                raw_arcs[narcs] = (RawArc20){kid | ((uint32_t)sym_c << 24), ov0 - t0_pos, ov1 - ov0, f_of_l[v], ov0 - v_start};
                sim_access(&sim, arcs_addr + (uint64_t)narcs * sizeof(RawArc20), sizeof(RawArc20), 1);
                narcs++;
                if (v_end >= t1_pos) break;
                v++; v_start = v_end;
                sim_access(&sim, l_cw_addr + (uint64_t)v * 2, 2, 0);
                uint32_t nw = l_cw[v] & 255; if (nw == 255) nw = big_lw[v_blw++];
                v_end = v_start + nw;
            }
        }
        t0_pos = t1_pos; met->work_events++;
    }
    free(l_cw); free(big_lw); free(f_of_l); free(f_w8); free(big_fw);

    met->l2_r1_pass13 = sim.l2.misses;
    double t_r13 = now_ms(); met->ms_r1_13 = t_r13 - t0;

    uint32_t mark_shift = 0, new_p = 0;
    {
        uint32_t cur = p_fi, acc_d = 0;
        for (;;) {
            sim_access(&sim, forest_addr + (uint64_t)cur * 4, 4, 0);
            uint32_t fe = forest[cur];
            if (fe & F_SPLIT_BIT) { new_p = k_new_from[fe & 0x7FFFFFu] + acc_d + p_in_run; break; }
            uint32_t ds = (fe >> 18) & 31u;
            uint32_t d = (ds < F_DSMALL_MAX) ? ds : bigd_get(big_d, bd_mask, cur, &sim, big_d_addr);
            acc_d += d; mark_shift++; cur = fe & 0x3FFFFu;
        }
    }

    /* Pass 4: 8-Way Parallel Lockstep Forest Trace -> Piece16 with inline <= 5B words! */
    typedef struct { uint32_t from, to, w, wid_or_inline_len; uint64_t inline_bytes; } PThin;
    typedef struct { uint32_t from, to, w, wid; } PThick;
    typedef struct { uint32_t to, w; uint64_t wp; } Piece16;
    Piece16 *pcs = (Piece16 *)malloc((size_t)narcs * sizeof(Piece16));
    uint64_t pcs_addr = sim_alloc_region(&sim, (size_t)narcs * sizeof(Piece16));

    size_t cap_lit = 131072, lit_len = 0;
    uint8_t *lit_spell = (uint8_t *)malloc(cap_lit);
    uint64_t lit_spell_addr = sim_alloc_region(&sim, 262144);
    uint32_t *lit_off = (uint32_t *)malloc((size_t)(narcs + 2) * 4);
    uint64_t lit_off_addr = sim_alloc_region(&sim, (size_t)(narcs + 2) * 4);
    lit_off[0] = 0; uint32_t n_lits = 0;

    uint32_t l_arc[8], l_cur[8], l_accd[8], l_blen[8];
    uint8_t  l_buf[8][128];
    int      l_active[8] = {0};
    uint32_t next_arc = 0, active_cnt = 0;

    for (int lane = 0; lane < 8 && next_arc < narcs; lane++) {
        uint32_t ai = next_arc++;
        sim_access(&sim, arcs_addr + (uint64_t)ai * sizeof(RawArc20), sizeof(RawArc20), 0);
        l_arc[lane] = ai; l_cur[lane] = raw_arcs[ai].v_fi; l_accd[lane] = 0;
        l_buf[lane][0] = (uint8_t)(raw_arcs[ai].kid_sym >> 24); l_blen[lane] = 1;
        l_active[lane] = 1; active_cnt++;
    }
    while (active_cnt > 0) {
        uint32_t fe[8];
        for (int lane = 0; lane < 8; lane++) {
            if (l_active[lane]) { sim_access(&sim, forest_addr + (uint64_t)l_cur[lane] * 4, 4, 0); fe[lane] = forest[l_cur[lane]]; }
            else fe[lane] = 0;
        }
        for (int lane = 0; lane < 8; lane++) {
            if (!l_active[lane]) continue;
            uint32_t e = fe[lane];
            if (e & F_SPLIT_BIT) {
                uint32_t ai = l_arc[lane]; RawArc20 ra = raw_arcs[ai];
                uint32_t rt_kid = e & 0x7FFFFFu;
                sim_access(&sim, knf_addr + (uint64_t)rt_kid * 4, 4, 0);
                uint32_t s_to = k_new_from[rt_kid] + l_accd[lane] + ra.in_v;
                uint32_t bl = l_blen[lane];
                uint64_t wp;
                if (ra.ow <= LIMIT && bl <= 5) {
                    uint64_t bpack = 0;
                    for (uint32_t q = 0; q < bl; q++) bpack |= ((uint64_t)l_buf[lane][bl - 1 - q]) << (q * 8);
                    wp = (1ULL << 63) | ((uint64_t)bl) | (bpack << 3);
                } else {
                    if (lit_len + bl + 4 >= cap_lit) { cap_lit *= 2; lit_spell = (uint8_t *)realloc(lit_spell, cap_lit); }
                    uint32_t st = (uint32_t)lit_len;
                    for (uint32_t q = 0; q < bl; q++) lit_spell[lit_len++] = l_buf[lane][bl - 1 - q];
                    sim_access(&sim, lit_spell_addr + st, bl, 1);
                    uint32_t lid = n_lits++;
                    lit_off[lid + 1] = (uint32_t)lit_len;
                    sim_access(&sim, lit_off_addr + (uint64_t)(lid + 1) * 4, 4, 1);
                    wp = (uint64_t)lid;
                }
                pcs[ai] = (Piece16){s_to, ra.ow, wp};
                sim_access(&sim, pcs_addr + (uint64_t)ai * sizeof(Piece16), sizeof(Piece16), 1);
                if (next_arc < narcs) {
                    uint32_t nai = next_arc++;
                    sim_access(&sim, arcs_addr + (uint64_t)nai * sizeof(RawArc20), sizeof(RawArc20), 0);
                    l_arc[lane] = nai; l_cur[lane] = raw_arcs[nai].v_fi; l_accd[lane] = 0;
                    l_buf[lane][0] = (uint8_t)(raw_arcs[nai].kid_sym >> 24); l_blen[lane] = 1;
                } else { l_active[lane] = 0; active_cnt--; }
            } else {
                uint32_t ds = (e >> 18) & 31u;
                uint32_t d = (ds < F_DSMALL_MAX) ? ds : bigd_get(big_d, bd_mask, l_cur[lane], &sim, big_d_addr);
                l_accd[lane] += d;
                l_buf[lane][l_blen[lane]++] = (uint8_t)((e >> 23) & 255u);
                l_cur[lane] = e & 0x3FFFFu;
                met->work_events++;
            }
        }
    }
    size_t r1_peak = nrecs * 2 + (size_t)r * 4 + (size_t)(bd_mask + 1) * 8 + (size_t)narcs * (sizeof(RawArc20) + sizeof(Piece16)) + lit_len + (size_t)n_lits * 4;
    free(forest); free(big_d); free(k_new_from); free(raw_arcs);

    met->l2_r1_pass4 = sim.l2.misses - met->l2_r1_pass13;
    met->l2_p1 = sim.l2.misses;
    double t1 = now_ms(); met->ms_r1_4 = t1 - t_r13; met->ms_p1 = t1 - t0;

    PThin  *thins  = (PThin  *)malloc((size_t)(narcs + 4) * sizeof(PThin));
    PThick *strips = (PThick *)malloc((size_t)(narcs + 4) * sizeof(PThick));
    uint64_t thins_addr  = sim_alloc_region(&sim, (size_t)(narcs + 4) * sizeof(PThin));
    uint64_t strips_addr = sim_alloc_region(&sim, (size_t)(narcs + 4) * sizeof(PThick));

    uint32_t nstrips = 0, nthins = 0, m1 = 0, mark = 0, cur_from = 0;
    for (uint32_t i = 0; i < narcs; i++) {
        sim_access(&sim, pcs_addr + (uint64_t)i * sizeof(Piece16), sizeof(Piece16), 0);
        Piece16 pc = pcs[i];
        uint32_t end = cur_from + pc.w;
        if (pc.w > LIMIT && cur_from <= new_p && new_p < end) {
            uint32_t wid = (uint32_t)pc.wp;
            if (cur_from < new_p) { strips[nstrips++] = (PThick){cur_from, pc.to, new_p - cur_from, wid}; }
            mark = m1;
            thins[nthins++] = (PThin){m1, pc.to + (new_p - cur_from), 1, wid, 0}; m1 += 1;
            if (new_p + 1 < end) { strips[nstrips++] = (PThick){new_p + 1, pc.to + (new_p + 1 - cur_from), end - new_p - 1, wid}; }
        } else if (pc.w <= LIMIT) {
            if (cur_from <= new_p && new_p < end) mark = m1 + (new_p - cur_from);
            if (pc.wp & (1ULL << 63)) {
                thins[nthins++] = (PThin){m1, pc.to, pc.w, (uint32_t)(pc.wp & 7u) | (1u << 31), (pc.wp >> 3) & 0xFFFFFFFFFFULL};
            } else {
                thins[nthins++] = (PThin){m1, pc.to, pc.w, (uint32_t)pc.wp, 0};
            }
            sim_access(&sim, thins_addr + (uint64_t)(nthins - 1) * sizeof(PThin), sizeof(PThin), 1);
            m1 += pc.w;
        } else {
            strips[nstrips++] = (PThick){cur_from, pc.to, pc.w, (uint32_t)pc.wp};
            sim_access(&sim, strips_addr + (uint64_t)(nstrips - 1) * sizeof(PThick), sizeof(PThick), 1);
        }
        cur_from = end;
    }
    free(pcs);

    uint32_t *s_ord = (uint32_t *)malloc((size_t)(nstrips + 1) * 4);
    uint32_t *s_tmp = (uint32_t *)malloc((size_t)(nstrips + 1) * 4);
    {
        uint32_t c0[1024] = {0}, c1[1024] = {0};
        for (uint32_t i = 0; i < nstrips; i++) { c0[strips[i].to & 1023u]++; c1[(strips[i].to >> 10) & 1023u]++; }
        uint32_t a0 = 0, a1 = 0;
        for (int b = 0; b < 1024; b++) { uint32_t v0 = c0[b]; c0[b] = a0; a0 += v0; uint32_t v1 = c1[b]; c1[b] = a1; a1 += v1; }
        for (uint32_t i = 0; i < nstrips; i++) s_tmp[c0[strips[i].to & 1023u]++] = i;
        for (uint32_t i = 0; i < nstrips; i++) { uint32_t id = s_tmp[i]; s_ord[c1[(strips[id].to >> 10) & 1023u]++] = id; }
    }
    free(s_tmp);

    size_t max_nodes = 3 * (size_t)nstrips + 5;
    EpNode *nodes = (EpNode *)calloc(max_nodes, sizeof(EpNode));
    uint64_t nodes_addr = sim_alloc_region(&sim, max_nodes * sizeof(EpNode));
    uint32_t nnodes = 1;
    uint32_t *ord0 = (uint32_t *)malloc(max_nodes * 4), *ord1 = (uint32_t *)malloc(max_nodes * 4);
    uint32_t *term0 = (uint32_t *)malloc(max_nodes * 4), *term1 = (uint32_t *)malloc(max_nodes * 4);
    size_t n_o0 = 0, n_o1 = 0, n_t0 = 0, n_t1 = 0;
    for (uint32_t i = 0; i < nstrips; i++) { nodes[nnodes].width = strips[i].w; nodes[nnodes].word = strips[i].wid + 1; nnodes++; }
    uint32_t x = 0;
    for (uint32_t i = 0; i < nstrips; i++) { if (x < strips[i].from) { uint32_t id = nnodes++; nodes[id].width = strips[i].from - x; ord0[n_o0++] = id; term1[n_t1++] = id; } ord0[n_o0++] = i + 1; x = strips[i].from + strips[i].w; }
    if (x < new_n) { uint32_t id = nnodes++; nodes[id].width = new_n - x; ord0[n_o0++] = id; term1[n_t1++] = id; }
    x = 0;
    for (uint32_t k = 0; k < nstrips; k++) { uint32_t i = s_ord[k]; if (x < strips[i].to) { uint32_t id = nnodes++; nodes[id].width = strips[i].to - x; ord1[n_o1++] = id; term0[n_t0++] = id; } ord1[n_o1++] = i + 1; x = strips[i].to + strips[i].w; }
    if (x < new_n) { uint32_t id = nnodes++; nodes[id].width = new_n - x; ord1[n_o1++] = id; term0[n_t0++] = id; }
    for (int side = 0; side < 2; side++) {
        uint32_t prev = 0; uint32_t *tarr = side == 0 ? term0 : term1; size_t nt = side == 0 ? n_t0 : n_t1; uint32_t *oarr = side == 0 ? ord0 : ord1; size_t no = side == 0 ? n_o0 : n_o1;
        for (size_t k = 0; k < nt; k++) { uint32_t id = tarr[k]; nodes[prev].next[side] = id; nodes[id].prev[side] = prev; prev = id; }
        for (size_t k = 0; k < no; k++) { uint32_t id = oarr[k]; nodes[prev].next[side] = id; nodes[id].prev[side] = prev; prev = id; }
        nodes[prev].next[side] = 0; nodes[0].prev[side] = prev;
    }
    size_t cap_rules = 16384; EpRule *rules = (EpRule *)malloc(cap_rules * sizeof(EpRule));
    uint64_t rules_addr = sim_alloc_region(&sim, 65536 * sizeof(EpRule));
    uint32_t nrules = 0, rem = new_n;
    while (rem > 0) {
        uint32_t ix = nodes[0].prev[0], iy = nodes[0].prev[1];
        sim_access(&sim, nodes_addr + (uint64_t)ix * sizeof(EpNode), sizeof(EpNode), 0);
        sim_access(&sim, nodes_addr + (uint64_t)iy * sizeof(EpNode), sizeof(EpNode), 0);
        EpNode a = nodes[ix], b = nodes[iy]; uint32_t w = a.width < b.width ? a.width : b.width;
        rem -= w; met->work_events++;
        if (ix == iy) { ep_cut(&sim, nodes_addr, nodes, ix, 0); ep_cut(&sim, nodes_addr, nodes, ix, 1); continue; }
        uint32_t word;
        if (a.word == 0) word = b.word; else if (b.word == 0) word = a.word;
        else {
            if (nrules == cap_rules) { cap_rules *= 2; rules = (EpRule *)realloc(rules, cap_rules * sizeof(EpRule)); }
            uint32_t la = (a.word <= n_lits) ? (lit_off[a.word] - lit_off[a.word - 1]) : rules[a.word - n_lits - 1].len;
            uint32_t lb = (b.word <= n_lits) ? (lit_off[b.word] - lit_off[b.word - 1]) : rules[b.word - n_lits - 1].len;
            rules[nrules] = (EpRule){a.word, b.word, la + lb, UINT32_MAX};
            sim_access(&sim, rules_addr + (uint64_t)nrules * sizeof(EpRule), sizeof(EpRule), 1);
            word = nrules + n_lits + 1; nrules++;
        }
        if (a.width <= b.width) {
            ep_cut(&sim, nodes_addr, nodes, ix, 0); ep_insert(&sim, nodes_addr, nodes, iy, ix, 0); nodes[ix].word = word;
            if (a.width == b.width) { ep_cut(&sim, nodes_addr, nodes, iy, 0); ep_cut(&sim, nodes_addr, nodes, iy, 1); } else nodes[iy].width -= w;
        } else { ep_cut(&sim, nodes_addr, nodes, iy, 1); ep_insert(&sim, nodes_addr, nodes, ix, iy, 1); nodes[iy].word = word; nodes[ix].width -= w; }
    }
    uint32_t *bottom = (uint32_t *)malloc(nnodes * 4); uint32_t id = nodes[0].next[1]; x = 0;
    while (id != 0) { bottom[id] = x; x += nodes[id].width; id = nodes[id].next[1]; }
    assert(x == m1);

    typedef struct { uint32_t end, delta, tw_id; } TBand;
    TBand *bands = (TBand *)malloc((nnodes + 16) * sizeof(TBand));
    uint64_t bands_addr = sim_alloc_region(&sim, (nnodes + 16) * sizeof(TBand));
    size_t cap_ts = 131072, ts_len = 0;
    uint8_t *tspell = (uint8_t *)malloc(cap_ts); uint64_t tspell_addr = sim_alloc_region(&sim, 262144);
    uint32_t *toff = (uint32_t *)malloc((nnodes + 16) * 4); uint64_t toff_addr = sim_alloc_region(&sim, (nnodes + 16) * 4);
    toff[0] = 0; uint32_t ntw = 0, nbands = 0; uint32_t *stack = (uint32_t *)malloc(65536 * 4);
    id = nodes[0].next[0]; uint32_t k = 0;
    while (id != 0) {
        EpNode nd = nodes[id]; uint32_t dst = bottom[id], tw_id = 0;
        if (nd.word != 0) {
            size_t sp = 0; stack[sp++] = nd.word;
            while (sp > 0) {
                uint32_t wid = stack[--sp];
                if (wid <= n_lits) {
                    uint32_t st = lit_off[wid - 1], ln = lit_off[wid] - st;
                    if (ts_len + ln + 2 >= cap_ts) { while (ts_len + ln + 2 >= cap_ts) cap_ts *= 2; tspell = (uint8_t *)realloc(tspell, cap_ts); }
                    sim_access(&sim, lit_spell_addr + st, ln, 0); memcpy(tspell + ts_len, lit_spell + st, ln); sim_access(&sim, tspell_addr + ts_len, ln, 1); ts_len += ln;
                } else {
                    EpRule *rl = &rules[wid - n_lits - 1];
                    if (rl->first == UINT32_MAX) { rl->first = (uint32_t)ts_len; stack[sp++] = rl->right; stack[sp++] = rl->left; }
                    else { uint32_t st = rl->first, ln = rl->len; if (ts_len + ln + 2 >= cap_ts) { while (ts_len + ln + 2 >= cap_ts) cap_ts *= 2; tspell = (uint8_t *)realloc(tspell, cap_ts); } sim_access(&sim, tspell_addr + st, ln, 0); memcpy(tspell + ts_len, tspell + st, ln); sim_access(&sim, tspell_addr + ts_len, ln, 1); ts_len += ln; }
                }
            }
            toff[++ntw] = (uint32_t)ts_len; sim_access(&sim, toff_addr + (uint64_t)ntw * 4, 4, 1); tw_id = ntw;
        }
        bands[nbands] = (TBand){k + nd.width, dst - k, tw_id};
        sim_access(&sim, bands_addr + (uint64_t)nbands * sizeof(TBand), sizeof(TBand), 1);
        nbands++; k += nd.width; id = nd.next[0];
    }
    free(nodes); free(ord0); free(ord1); free(term0); free(term1); free(rules); free(bottom); free(stack);
    met->l2_p2 = sim.l2.misses - met->l2_p1; double t2 = now_ms(); met->ms_p2 = t2 - t1;

    uint32_t *t_ord = (uint32_t *)malloc((size_t)(nthins + 1) * 4);
    uint32_t *t_tmp = (uint32_t *)malloc((size_t)(nthins + 1) * 4);
    uint64_t tord_addr = sim_alloc_region(&sim, (size_t)(nthins + 1) * 4);
    {
        uint32_t c0[1024] = {0}, c1[1024] = {0};
        for (uint32_t i = 0; i < nthins; i++) { sim_access(&sim, thins_addr + (uint64_t)i * sizeof(PThin), sizeof(PThin), 0); c0[thins[i].to & 1023u]++; c1[(thins[i].to >> 10) & 1023u]++; }
        uint32_t a0 = 0, a1 = 0;
        for (int b = 0; b < 1024; b++) { uint32_t v0 = c0[b]; c0[b] = a0; a0 += v0; uint32_t v1 = c1[b]; c1[b] = a1; a1 += v1; }
        for (uint32_t i = 0; i < nthins; i++) t_tmp[c0[thins[i].to & 1023u]++] = i;
        for (uint32_t i = 0; i < nthins; i++) { uint32_t tid = t_tmp[i]; uint32_t d = c1[(thins[tid].to >> 10) & 1023u]++; t_ord[d] = tid; sim_access(&sim, tord_addr + (uint64_t)d * 4, 4, 1); }
    }
    free(t_tmp);

    uint64_t *edges = (uint64_t *)malloc((size_t)m1 * 8);
    uint64_t edges_addr = sim_alloc_region(&sim, (size_t)m1 * 8);
    uint32_t cap_loff = narcs + 2;

    uint32_t si = 0, thick_before = 0, bi = 0;
    for (uint32_t ti = 0; ti < nthins; ti++) {
        sim_access(&sim, tord_addr + (uint64_t)ti * 4, 4, 0);
        uint32_t tid = t_ord[ti];
        sim_access(&sim, thins_addr + (uint64_t)tid * sizeof(PThin), sizeof(PThin), 0);
        PThin th = thins[tid];
        while (si < nstrips && strips[s_ord[si]].to < th.to) { thick_before += strips[s_ord[si]].w; si++; }
        uint32_t y = th.to - thick_before, rem_w = th.w, src = th.from;
        int is_inl = (th.wid_or_inline_len & (1u << 31)) != 0;
        uint32_t inl_len = th.wid_or_inline_len & 7u;
        uint32_t flushed_lid = UINT32_MAX;
        while (rem_w > 0) {
            while (bands[bi].end <= y) bi++;
            TBand bd = bands[bi];
            uint32_t avail = bd.end - y, take = rem_w < avail ? rem_w : avail;
            uint32_t dst0 = y + bd.delta;
            uint64_t pack;
            if (bd.tw_id == 0 && is_inl) {
                pack = ((uint64_t)inl_len << 17) | (th.inline_bytes << 20);
            } else {
                uint32_t lid;
                if (!is_inl) lid = th.wid_or_inline_len;
                else {
                    if (flushed_lid == UINT32_MAX) {
                        if (lit_len + inl_len + 4 >= cap_lit) { cap_lit *= 2; lit_spell = (uint8_t *)realloc(lit_spell, cap_lit); }
                        for (uint32_t q = 0; q < inl_len; q++) lit_spell[lit_len++] = (uint8_t)((th.inline_bytes >> (q * 8)) & 255u);
                        if (n_lits + 2 >= cap_loff) { cap_loff *= 2; lit_off = (uint32_t *)realloc(lit_off, (size_t)cap_loff * 4); }
                        flushed_lid = n_lits++;
                        lit_off[flushed_lid + 1] = (uint32_t)lit_len;
                    }
                    lid = flushed_lid;
                }
                assert(lid < 65536u && bd.tw_id < 32768u);
                pack = ((uint64_t)lid << 20) | ((uint64_t)bd.tw_id << 36);
            }
            for (uint32_t u = 0; u < take; u++) edges[dst0 + u] = pack | (uint64_t)(src + u);
            sim_access(&sim, edges_addr + (uint64_t)dst0 * 8, (size_t)take * 8, 1);
            y += take; src += take; rem_w -= take; met->work_events += take;
        }
    }
    free(strips); free(s_ord); free(thins); free(t_ord); free(bands);
    met->l2_p3 = sim.l2.misses - met->l2_p1 - met->l2_p2; double t3 = now_ms(); met->ms_p3 = t3 - t2;

    uint8_t *out = (uint8_t *)malloc(n + 8);
    uint64_t out_addr = sim_alloc_region(&sim, n + 8);
    uint32_t sh = mark_shift % n, tail_base = n - sh;
    uint32_t vpos = 0, j = mark, fast_hits = 0;

    for (uint32_t step = 0; step < m1 && vpos < n; step++) {
        sim_access(&sim, edges_addr + (uint64_t)j * 8, 8, 0);
        uint64_t e = edges[j];
        j = (uint32_t)(e & 0x1FFFFu);
        uint32_t ilen = (uint32_t)((e >> 17) & 7u);
        if (ilen > 0) {
            fast_hits++;
            uint64_t bval = e >> 20;
            if (vpos >= sh) {
                uint32_t dst = vpos - sh;
                if (dst + 8 <= tail_base) memcpy(out + dst, &bval, 8);
                else { uint8_t tmp8[8]; memcpy(tmp8, &bval, 8); memcpy(out + dst, tmp8, ilen); }
                sim_access(&sim, out_addr + dst, ilen, 1);
            } else if (vpos + ilen <= sh) {
                uint32_t dst = tail_base + vpos;
                memcpy(out + dst, &bval, 8);
                sim_access(&sim, out_addr + dst, ilen, 1);
            } else {
                uint32_t a = sh - vpos, b = ilen - a;
                uint8_t tmp8[8]; memcpy(tmp8, &bval, 8);
                memcpy(out + tail_base + vpos, tmp8, a); memcpy(out, tmp8 + a, b);
                sim_access(&sim, out_addr + tail_base + vpos, a, 1); sim_access(&sim, out_addr, b, 1);
            }
            vpos += ilen;
        } else {
            uint32_t lid = (uint32_t)((e >> 20) & 0xFFFFu);
            uint32_t tw_id = (uint32_t)((e >> 36) & 0x7FFFu);
            if (tw_id > 0) {
                sim_access(&sim, toff_addr + (uint64_t)(tw_id - 1) * 4, 8, 0);
                uint32_t st = toff[tw_id - 1], ln = toff[tw_id] - st;
                sim_access(&sim, tspell_addr + st, ln, 0);
                if (vpos >= sh) { memcpy(out + (vpos - sh), tspell + st, ln); sim_access(&sim, out_addr + (vpos - sh), ln, 1); }
                else if (vpos + ln <= sh) { memcpy(out + tail_base + vpos, tspell + st, ln); sim_access(&sim, out_addr + tail_base + vpos, ln, 1); }
                else { uint32_t a = sh - vpos, b = ln - a; memcpy(out + tail_base + vpos, tspell + st, a); memcpy(out, tspell + st + a, b); sim_access(&sim, out_addr + tail_base + vpos, a, 1); sim_access(&sim, out_addr, b, 1); }
                vpos += ln;
            }
            sim_access(&sim, lit_off_addr + (uint64_t)lid * 4, 8, 0);
            uint32_t st = lit_off[lid], ln = lit_off[lid + 1] - st;
            sim_access(&sim, lit_spell_addr + st, ln, 0);
            if (vpos >= sh) { memcpy(out + (vpos - sh), lit_spell + st, ln); sim_access(&sim, out_addr + (vpos - sh), ln, 1); }
            else if (vpos + ln <= sh) { memcpy(out + tail_base + vpos, lit_spell + st, ln); sim_access(&sim, out_addr + tail_base + vpos, ln, 1); }
            else { uint32_t a = sh - vpos, b = ln - a; memcpy(out + tail_base + vpos, lit_spell + st, a); memcpy(out, lit_spell + st + a, b); sim_access(&sim, out_addr + tail_base + vpos, a, 1); sim_access(&sim, out_addr, b, 1); }
            vpos += ln;
        }
        met->work_events++;
    }
    assert(vpos == n);
    met->l2_p4 = sim.l2.misses - met->l2_p1 - met->l2_p2 - met->l2_p3; double t4 = now_ms(); met->ms_p4 = t4 - t3;

    size_t emit_mem = (size_t)m1 * 8 + lit_len + (size_t)n_lits * 4 + ts_len + (size_t)ntw * 4 + n;
    size_t pk = r1_peak > emit_mem ? r1_peak : emit_mem;
    sim_drain(&sim);
    met->l2_misses = sim.l2.misses; met->llc_misses = sim.llc.misses;
    met->dram_bytes = (sim.dram_reads + sim.dram_writes) * 64ULL;
    met->peak_mem = pk; met->m_ret = m1; met->extra2 = fast_hits;
    free(edges); free(tspell); free(toff); free(lit_spell); free(lit_off); sim_free(&sim);
    return out;
}

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
static int cmp_dbl(const void *a, const void *b) { double da = *(const double *)a, db = *(const double *)b; return (da < db) ? -1 : (da > db); }
#endif

int main(int argc, char **argv) {
    if (argc < 2) { fprintf(stderr, "Usage: %s <admitted-bwt-dir>\n", argv[0]); return 1; }
    const char *dir = argv[1];
#ifndef NO_CACHE_SIM
    printf("=== Cycle 4 G5 (Compact UFC-1+Open w/ Inline-5B u64 edges) vs Shipped Open ===\n\n");
    int all_pass = 1;
    for (int bi = 0; bi < 6; bi++) {
        char path[512]; snprintf(path, sizeof(path), "%s/%s", dir, BLOCKS[bi].file);
        FILE *fp = fopen(path, "rb"); assert(fp != NULL);
        uint32_t n = BLOCKS[bi].n, p = BLOCKS[bi].p;
        uint8_t *L = (uint8_t *)malloc(n); assert(fread(L, 1, n, fp) == n); fclose(fp);
        size_t nrecs = 0; uint16_t *recs = column_to_records(L, n, &nrecs);
        uint8_t *ref = dense_reference(L, n, p);

        Metrics mf, mc;
        uint8_t *of = run_forward_open(recs, nrecs, n, p, &mf);
        uint8_t *oc = run_ufc1_compact(recs, nrecs, n, p, &mc);
        assert(memcmp(of, ref, n) == 0 && memcmp(oc, ref, n) == 0);

        double df = (double)mf.dram_bytes / n, dc = (double)mc.dram_bytes / n;
        double l2r = (double)mc.l2_misses / mf.l2_misses, dr = dc / df, pkr = (double)mc.peak_mem / mf.peak_mem;
        if (!(l2r <= 0.85 && dr <= 0.90 && pkr <= 1.15)) all_pass = 0;
        printf("%s (m=%u -> m1=%u, inline_fast=%.1f%%):\n", BLOCKS[bi].file, mf.m_ret, mc.m_ret, 100.0 * mc.extra2 / mc.m_ret);
        printf("  Shipped Open   : L2m=%7lu [p1=%5lu ep=%5lu wir=%5lu emit=%6lu] LLC=%6lu DRAM=%.2f B/B Work=%7lu Peak=%.2f MB\n",
               (unsigned long)mf.l2_misses, (unsigned long)mf.l2_p1, (unsigned long)mf.l2_p2, (unsigned long)mf.l2_p3, (unsigned long)mf.l2_p4,
               (unsigned long)mf.llc_misses, df, (unsigned long)mf.work_events, mf.peak_mem / 1e6);
        printf("  UFC-1+Open(cmp): L2m=%7lu [R1=%5lu(13:%lu+4:%lu) ep=%5lu wir=%5lu emit=%6lu] LLC=%6lu DRAM=%.2f B/B Work=%7lu Peak=%.2f MB\n",
               (unsigned long)mc.l2_misses, (unsigned long)mc.l2_p1, (unsigned long)mc.l2_r1_pass13, (unsigned long)mc.l2_r1_pass4,
               (unsigned long)mc.l2_p2, (unsigned long)mc.l2_p3, (unsigned long)mc.l2_p4,
               (unsigned long)mc.llc_misses, dc, (unsigned long)mc.work_events, mc.peak_mem / 1e6);
        printf("  Ratios         : L2m=%.3fx  DRAM=%.3fx  Work=%.3fx  Peak=%.3fx\n\n",
               l2r, dr, (double)mc.work_events / mf.work_events, pkr);
        free(L); free(recs); free(ref); free(of); free(oc);
    }
    printf("CACHE-MODEL VERDICT (6/6 blocks): %s\n", all_pass ? "PASS" : "FAIL");
#else
    printf("=== VM Single-Thread C Diagnostic Timing (Compact UFC-1+Open vs Shipped Open, 15 interleaved rounds) ===\n");
    const int ROUNDS = 15;
    for (int bi = 0; bi < 6; bi++) {
        char path[512]; snprintf(path, sizeof(path), "%s/%s", dir, BLOCKS[bi].file);
        FILE *fp = fopen(path, "rb"); assert(fp != NULL);
        uint32_t n = BLOCKS[bi].n, p = BLOCKS[bi].p;
        uint8_t *L = (uint8_t *)malloc(n); assert(fread(L, 1, n, fp) == n); fclose(fp);
        size_t nrecs = 0; uint16_t *recs = column_to_records(L, n, &nrecs);
        uint8_t *ref = dense_reference(L, n, p);
        double tf[15], tc[15]; Metrics mf, mc;
        for (int r = 0; r < ROUNDS; r++) {
            double a = now_ms(); uint8_t *o1 = run_forward_open(recs, nrecs, n, p, &mf); double b = now_ms(); tf[r] = b - a;
            a = now_ms(); uint8_t *o2 = run_ufc1_compact(recs, nrecs, n, p, &mc); b = now_ms(); tc[r] = b - a;
            if (r == 0) assert(memcmp(o1, ref, n) == 0 && memcmp(o2, ref, n) == 0);
            free(o1); free(o2);
        }
        qsort(tf, ROUNDS, sizeof(double), cmp_dbl); qsort(tc, ROUNDS, sizeof(double), cmp_dbl);
        printf("  %-11s: Open=%.2f ms [p1=%.2f ep=%.2f wir=%.2f em=%.2f] | UFC1-Cmp=%.2f ms (%.2fx) [R1=%.2f(13:%.2f+4:%.2f) ep=%.2f wir=%.2f em=%.2f]\n",
               BLOCKS[bi].file, tf[7], mf.ms_p1, mf.ms_p2, mf.ms_p3, mf.ms_p4,
               tc[7], tc[7]/tf[7], mc.ms_p1, mc.ms_r1_13, mc.ms_r1_4, mc.ms_p2, mc.ms_p3, mc.ms_p4);
        free(L); free(recs); free(ref);
    }
#endif
    return 0;
}
