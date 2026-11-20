/*
 * Cycle 5 · Branchless BSO-512 (1-lane & 4-lane/8-lane) vs Shipped Forward Open
 *
 * Tests the branch-misprediction hypothesis on BSO-512:
 *   - 100% branchless AVX2 6-plane rank (zero is_hi branch, zero lane branch, 8 KiB LT_MASK table)
 *   - No mega_chk load on the max_tc < 65536 fast path
 *   - Direct band_hint[y >> 4] + branchless 1-step advance (eliminates unpredictable band loop branch)
 *   - Compare 1-lane Branchless BSO-512 vs 4-lane Branchless BSO-512 vs Shipped Open!
 */
#define _POSIX_C_SOURCE 200809L
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <assert.h>
#include <time.h>
#include <immintrin.h>

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
    uint64_t l1_misses, l2_misses, llc_misses, dram_bytes;
    size_t peak_mem;
    uint64_t l2_p1, l2_p2, l2_p3, l2_p4a, l2_p4b;
    double ms_p1, ms_p2, ms_p3, ms_p4a, ms_p4b;
    uint32_t m_ret;
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
typedef struct { uint32_t from, to, width; uint8_t byte; } OpenStrip;
typedef struct { uint32_t end, delta, code; } OpenBand;

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

/* ==================== Shipped Forward Open ==================== */
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
                if (qlen[c] + sw[s] > qcap[c]) { while (qlen[c] + sw[s] > qcap[c]) qcap[c] *= 2; qdata[c] = (uint32_t *)realloc(qdata[c], qcap[c] * 4); }
                for (uint32_t u = 0; u < sw[s]; u++) { qdata[c][qlen[c]] = m + u; sim_access(&sim, qaddr[c] + (uint64_t)qlen[c] * 4, 4, 1); qlen[c]++; }
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
        EpNode a = nodes[ix], b = nodes[iy]; uint32_t w = a.width < b.width ? a.width : b.width; rem -= w;
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
            k += w; qi += w; bi++;
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
    }
    met->l2_p4b = sim.l2.misses - met->l2_p1 - met->l2_p2 - met->l2_p3; double t4 = now_ms(); met->ms_p4b = t4 - t3;
    sim_drain(&sim);
    met->l1_misses = sim.l1.misses; met->l2_misses = sim.l2.misses; met->llc_misses = sim.llc.misses;
    met->dram_bytes = (sim.dram_reads + sim.dram_writes) * 64ULL;
    met->peak_mem = nrecs * 2 + (size_t)m * 8 + ep_mem + n; met->m_ret = m;
    for (int a = 0; a < 256; a++) free(qdata[a]);
    free(strips); free(bands); free(spelling); free(offsets); free(edges); sim_free(&sim);
    return out;
}

/* ==================== Branchless BSO-512 (SB512 = 512 B/512 rows = 1.00 B/row) ==================== */
typedef struct __attribute__((aligned(64))) {
    uint16_t chk[64];       /* 128 B = 2 lines */
    uint64_t pl[2][6][4];   /* 384 B = 6 lines: [is_hi][bitplane][lane] */
} SB512;

typedef struct { uint32_t end_code; int32_t delta; } Band8;

static uint64_t LT_MASK[256][4] __attribute__((aligned(64)));
static uint64_t CODE_XNOR[64][6] __attribute__((aligned(64)));
static void init_bso_tables(void) {
    static int inited = 0;
    if (inited) return;
    for (int pos = 0; pos < 256; pos++) {
        int ln = pos >> 6, bt = pos & 63;
        for (int q = 0; q < 4; q++) {
            if (q < ln) LT_MASK[pos][q] = ~0ULL;
            else if (q == ln) LT_MASK[pos][q] = (1ULL << bt) - 1ULL;
            else LT_MASK[pos][q] = 0ULL;
        }
    }
    for (int c = 0; c < 64; c++) {
        for (int b = 0; b < 6; b++) CODE_XNOR[c][b] = ~(-(uint64_t)((c >> b) & 1));
    }
    inited = 1;
}

static inline uint32_t bso512_lf_branchless(
    const SB512 *sbs, uint64_t sbs_addr,
    const uint32_t *C_code, const uint8_t *code2sym,
    const uint32_t *esc_lf, uint64_t esc_lf_addr,
    const uint8_t *esc_byte, uint64_t esc_b_addr,
    uint32_t z, uint8_t *out_lit, CacheSim *sim
) {
    uint32_t s = z >> 9;
    uint32_t is_hi = (z >> 8) & 1u;
    uint32_t in256 = z & 255u;
    uint32_t lane = in256 >> 6, bit = in256 & 63u;
    const uint64_t (*p)[4] = sbs[s].pl[is_hi];
    sim_access(sim, sbs_addr + (uint64_t)s * sizeof(SB512) + 128 + (uint64_t)is_hi * 192, 192, 0);

    uint32_t c = (uint32_t)(((p[0][lane] >> bit) & 1ULL)
               | (((p[1][lane] >> bit) & 1ULL) << 1)
               | (((p[2][lane] >> bit) & 1ULL) << 2)
               | (((p[3][lane] >> bit) & 1ULL) << 3)
               | (((p[4][lane] >> bit) & 1ULL) << 4)
               | (((p[5][lane] >> bit) & 1ULL) << 5));

    uint32_t sb_idx = s + is_hi;
    sim_access(sim, sbs_addr + (uint64_t)sb_idx * sizeof(SB512) + (uint64_t)c * 2, 2, 0);
    int32_t base_cnt = (int32_t)sbs[sb_idx].chk[c];

    const uint64_t *xn = CODE_XNOR[c];
    __m256i e0 = _mm256_xor_si256(_mm256_load_si256((const __m256i *)p[0]), _mm256_set1_epi64x((int64_t)xn[0]));
    __m256i e1 = _mm256_xor_si256(_mm256_load_si256((const __m256i *)p[1]), _mm256_set1_epi64x((int64_t)xn[1]));
    __m256i e2 = _mm256_xor_si256(_mm256_load_si256((const __m256i *)p[2]), _mm256_set1_epi64x((int64_t)xn[2]));
    __m256i e3 = _mm256_xor_si256(_mm256_load_si256((const __m256i *)p[3]), _mm256_set1_epi64x((int64_t)xn[3]));
    __m256i e4 = _mm256_xor_si256(_mm256_load_si256((const __m256i *)p[4]), _mm256_set1_epi64x((int64_t)xn[4]));
    __m256i e5 = _mm256_xor_si256(_mm256_load_si256((const __m256i *)p[5]), _mm256_set1_epi64x((int64_t)xn[5]));

    __m256i flip = _mm256_set1_epi64x(-(int64_t)is_hi);
    __m256i msk  = _mm256_xor_si256(_mm256_load_si256((const __m256i *)LT_MASK[in256]), flip);
    __m256i eq   = _mm256_and_si256(_mm256_and_si256(_mm256_and_si256(e0, e1), _mm256_and_si256(e2, e3)),
                                    _mm256_and_si256(_mm256_and_si256(e4, e5), msk));
    /* Extract 4 uint64_t lanes directly via registers (zero stack round-trip!) */
    __m128i lo128 = _mm256_castsi256_si128(eq);
    __m128i hi128 = _mm256_extracti128_si256(eq, 1);
    int32_t local_cnt = (int32_t)(
        __builtin_popcountll((uint64_t)_mm_cvtsi128_si64(lo128)) +
        __builtin_popcountll((uint64_t)_mm_extract_epi64(lo128, 1)) +
        __builtin_popcountll((uint64_t)_mm_cvtsi128_si64(hi128)) +
        __builtin_popcountll((uint64_t)_mm_extract_epi64(hi128, 1))
    );
    int32_t sgn = 1 - (int32_t)(is_hi << 1);
    uint32_t rnk = (uint32_t)(base_cnt + sgn * local_cnt);

    if (__builtin_expect(c < 63u, 1)) {
        if (out_lit) *out_lit = code2sym[c];
        return C_code[c] + rnk;
    } else {
        sim_access(sim, esc_lf_addr + (uint64_t)rnk * 4, 4, 0);
        if (out_lit) { sim_access(sim, esc_b_addr + rnk, 1, 0); *out_lit = esc_byte[rnk]; }
        return esc_lf[rnk];
    }
}

typedef struct { uint32_t cnt; uint8_t sym; } SymFreq;
static int cmp_symfreq_desc(const void *a, const void *b) {
    uint32_t ca = ((const SymFreq *)a)->cnt, cb = ((const SymFreq *)b)->cnt;
    return (ca < cb) ? 1 : (ca > cb ? -1 : 0);
}

static uint8_t *run_bso_branchless(const uint16_t *recs, size_t nrecs, uint32_t n, uint32_t p, int nlanes, Metrics *met) {
    init_bso_tables();
    const uint32_t LIMIT = 16;
    CacheSim sim; sim_init(&sim); memset(met, 0, sizeof(*met));
    double t0 = now_ms();
    uint64_t rec_addr = sim_alloc_region(&sim, nrecs * 2);

    uint32_t all[256] = {0}, thin[256] = {0}, thick[256] = {0};
    OpenStrip *strips = (OpenStrip *)malloc((nrecs + 2) * sizeof(OpenStrip));
    uint64_t strips_addr = sim_alloc_region(&sim, (nrecs + 2) * sizeof(OpenStrip));
    size_t nstrips = 0;

    uint32_t v_cap = (uint32_t)(nrecs * 2 + 1024); if (v_cap > n + 64) v_cap = n + 64;
    uint8_t *v_raw = (uint8_t *)malloc(v_cap); uint64_t v_addr = sim_alloc_region(&sim, v_cap);
    uint32_t m = 0, mark = 0, pos = 0; size_t ri = 0;
    while (ri < nrecs) {
        sim_access(&sim, rec_addr + ri * 2, 2, 0);
        uint16_t rec = recs[ri++]; uint8_t c = (uint8_t)(rec >> 8); uint32_t w = rec & 255;
        while (ri < nrecs) { sim_access(&sim, rec_addr + ri * 2, 2, 0); if ((recs[ri] >> 8) != c) break; w += recs[ri] & 255; ri++; }
        uint32_t end = pos + w;
        if (w <= LIMIT) {
            if (pos <= p && p < end) mark = m + (p - pos);
            if (m + w > v_cap) { while (m + w > v_cap) v_cap *= 2; v_raw = (uint8_t *)realloc(v_raw, v_cap); }
            for (uint32_t u = 0; u < w; u++) v_raw[m + u] = c;
            sim_access(&sim, v_addr + m, w, 1); thin[c] += w; m += w;
        } else if (pos <= p && p < end) {
            if (pos < p) { strips[nstrips] = (OpenStrip){pos, all[c], p - pos, c}; sim_access(&sim, strips_addr + nstrips * sizeof(OpenStrip), sizeof(OpenStrip), 1); nstrips++; thick[c]++; all[c] += (p - pos); }
            mark = m;
            if (m + 1 > v_cap) { v_cap *= 2; v_raw = (uint8_t *)realloc(v_raw, v_cap); }
            v_raw[m] = c; sim_access(&sim, v_addr + m, 1, 1); thin[c] += 1; m += 1; all[c] += 1;
            if (p + 1 < end) { strips[nstrips] = (OpenStrip){p + 1, all[c], end - p - 1, c}; sim_access(&sim, strips_addr + nstrips * sizeof(OpenStrip), sizeof(OpenStrip), 1); nstrips++; thick[c]++; all[c] += (end - p - 1); }
            pos = end; continue;
        } else {
            strips[nstrips] = (OpenStrip){pos, all[c], w, c}; sim_access(&sim, strips_addr + nstrips * sizeof(OpenStrip), sizeof(OpenStrip), 1); nstrips++; thick[c]++;
        }
        all[c] += w; pos = end;
    }
    met->l2_p1 = sim.l2.misses; double t1 = now_ms(); met->ms_p1 = t1 - t0;

    SymFreq sf[256]; for (int a = 0; a < 256; a++) sf[a] = (SymFreq){thin[a], (uint8_t)a};
    qsort(sf, 256, sizeof(SymFreq), cmp_symfreq_desc);
    assert(sf[0].cnt < 65000u);
    uint8_t sym2code[256], code2sym[64];
    for (int a = 0; a < 256; a++) sym2code[a] = 63;
    memset(code2sym, 0, sizeof(code2sym));
    for (int k = 0; k < 63; k++) { sym2code[sf[k].sym] = (uint8_t)k; code2sym[k] = sf[k].sym; }
    uint32_t C_thin[256], t_acc = 0; for (int a = 0; a < 256; a++) { C_thin[a] = t_acc; t_acc += thin[a]; }
    uint32_t C_code[64]; for (int k = 0; k < 63; k++) C_code[k] = C_thin[code2sym[k]]; C_code[63] = 0;
    uint32_t m_esc = 0; for (int k = 63; k < 256; k++) m_esc += sf[k].cnt;
    uint32_t esc_cur[256]; for (int a = 0; a < 256; a++) esc_cur[a] = C_thin[a];

    uint32_t n_sb = (m + 511u) >> 9;
    SB512 *sbs = NULL; assert(posix_memalign((void **)&sbs, 64, (size_t)(n_sb + 1) * sizeof(SB512)) == 0);
    uint64_t sbs_addr = sim_alloc_region(&sim, (size_t)(n_sb + 1) * sizeof(SB512));
    uint32_t *esc_lf = (uint32_t *)malloc((size_t)(m_esc + 4) * 4); uint8_t *esc_byte = (uint8_t *)malloc((size_t)(m_esc + 4));
    uint64_t esc_lf_addr = sim_alloc_region(&sim, (size_t)(m_esc + 4) * 4); uint64_t esc_b_addr = sim_alloc_region(&sim, (size_t)(m_esc + 4));

    uint32_t run_tot[64] = {0}, ei = 0;
    uint8_t tmp512[512] __attribute__((aligned(32)));
    for (uint32_t s = 0; s < n_sb; s++) {
        for (int k = 0; k < 64; k++) sbs[s].chk[k] = (uint16_t)run_tot[k];
        uint32_t base = s << 9, take = (base + 512u <= m) ? 512u : (m - base);
        sim_access(&sim, v_addr + base, take, 0);
        for (uint32_t u = 0; u < take; u++) {
            uint8_t b = v_raw[base + u], cd = sym2code[b];
            tmp512[u] = (uint8_t)(cd << 2); run_tot[cd]++;
            if (__builtin_expect(cd == 63, 0)) {
                esc_lf[ei] = esc_cur[b]++; esc_byte[ei] = b;
                sim_access(&sim, esc_lf_addr + (uint64_t)ei * 4, 4, 1); sim_access(&sim, esc_b_addr + ei, 1, 1); ei++;
            }
        }
        for (uint32_t u = take; u < 512u; u++) { tmp512[u] = (uint8_t)(63u << 2); run_tot[63]++; }
        for (int half = 0; half < 2; half++) {
            uint64_t (*dst)[4] = sbs[s].pl[half];
            for (int q = 0; q < 4; q++) {
                const uint8_t *src = tmp512 + half * 256 + q * 64;
                __m256i x0 = _mm256_load_si256((const __m256i *)(src)), x1 = _mm256_load_si256((const __m256i *)(src + 32));
                uint32_t m0, m1;
                m0 = (uint32_t)_mm256_movemask_epi8(x0); m1 = (uint32_t)_mm256_movemask_epi8(x1); dst[5][q] = (uint64_t)m0 | ((uint64_t)m1 << 32);
                x0 = _mm256_add_epi8(x0, x0); x1 = _mm256_add_epi8(x1, x1);
                m0 = (uint32_t)_mm256_movemask_epi8(x0); m1 = (uint32_t)_mm256_movemask_epi8(x1); dst[4][q] = (uint64_t)m0 | ((uint64_t)m1 << 32);
                x0 = _mm256_add_epi8(x0, x0); x1 = _mm256_add_epi8(x1, x1);
                m0 = (uint32_t)_mm256_movemask_epi8(x0); m1 = (uint32_t)_mm256_movemask_epi8(x1); dst[3][q] = (uint64_t)m0 | ((uint64_t)m1 << 32);
                x0 = _mm256_add_epi8(x0, x0); x1 = _mm256_add_epi8(x1, x1);
                m0 = (uint32_t)_mm256_movemask_epi8(x0); m1 = (uint32_t)_mm256_movemask_epi8(x1); dst[2][q] = (uint64_t)m0 | ((uint64_t)m1 << 32);
                x0 = _mm256_add_epi8(x0, x0); x1 = _mm256_add_epi8(x1, x1);
                m0 = (uint32_t)_mm256_movemask_epi8(x0); m1 = (uint32_t)_mm256_movemask_epi8(x1); dst[1][q] = (uint64_t)m0 | ((uint64_t)m1 << 32);
                x0 = _mm256_add_epi8(x0, x0); x1 = _mm256_add_epi8(x1, x1);
                m0 = (uint32_t)_mm256_movemask_epi8(x0); m1 = (uint32_t)_mm256_movemask_epi8(x1); dst[0][q] = (uint64_t)m0 | ((uint64_t)m1 << 32);
            }
        }
        sim_access(&sim, sbs_addr + (uint64_t)s * sizeof(SB512), sizeof(SB512), 1);
    }
    for (int k = 0; k < 64; k++) sbs[n_sb].chk[k] = (uint16_t)run_tot[k];
    sim_access(&sim, sbs_addr + (uint64_t)n_sb * sizeof(SB512), 128, 1);
    free(v_raw);
    met->l2_p3 = sim.l2.misses - met->l2_p1; double t_bs = now_ms(); met->ms_p3 = t_bs - t1;

    /* Pass 3: Open endpoint -> Band8 + band_hint[y >> 4] + spelling */
    uint32_t first[256], write_pos[256], tot = 0, ttot = 0;
    for (int a = 0; a < 256; a++) { first[a] = tot; tot += all[a]; write_pos[a] = ttot; ttot += thick[a]; }
    uint32_t *image_order = (uint32_t *)malloc((nstrips + 1) * 4); uint64_t imgord_addr = sim_alloc_region(&sim, (nstrips + 1) * 4);
    for (size_t i = 0; i < nstrips; i++) {
        sim_access(&sim, strips_addr + i * sizeof(OpenStrip), sizeof(OpenStrip), 0);
        uint8_t a = strips[i].byte; strips[i].to += first[a]; sim_access(&sim, strips_addr + i * sizeof(OpenStrip), sizeof(OpenStrip), 1);
        uint32_t wp = write_pos[a]++; image_order[wp] = (uint32_t)i; sim_access(&sim, imgord_addr + (uint64_t)wp * 4, 4, 1);
    }
    size_t max_nodes = 3 * nstrips + 5; EpNode *nodes = (EpNode *)calloc(max_nodes, sizeof(EpNode));
    uint64_t nodes_addr = sim_alloc_region(&sim, max_nodes * sizeof(EpNode)); uint32_t nnodes = 1;
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
    uint64_t rules_addr = sim_alloc_region(&sim, 262144 * sizeof(EpRule)); uint32_t nrules = 0, rem = n;
    while (rem > 0) {
        uint32_t ix = nodes[0].prev[0], iy = nodes[0].prev[1];
        sim_access(&sim, nodes_addr + (uint64_t)ix * sizeof(EpNode), sizeof(EpNode), 0);
        sim_access(&sim, nodes_addr + (uint64_t)iy * sizeof(EpNode), sizeof(EpNode), 0);
        EpNode a = nodes[ix], b = nodes[iy]; uint32_t w = a.width < b.width ? a.width : b.width; rem -= w;
        if (ix == iy) { ep_cut(&sim, nodes_addr, nodes, ix, 0); ep_cut(&sim, nodes_addr, nodes, ix, 1); continue; }
        uint32_t word;
        if (a.word == 0) word = b.word; else if (b.word == 0) word = a.word;
        else {
            if (nrules == cap_rules) { cap_rules *= 2; rules = (EpRule *)realloc(rules, cap_rules * sizeof(EpRule)); }
            uint32_t la = a.word <= 256 ? 1 : rules[a.word - 257].len, lb = b.word <= 256 ? 1 : rules[b.word - 257].len;
            rules[nrules] = (EpRule){a.word, b.word, la + lb, UINT32_MAX}; sim_access(&sim, rules_addr + (uint64_t)nrules * sizeof(EpRule), sizeof(EpRule), 1); word = nrules + 257; nrules++;
        }
        if (a.width <= b.width) {
            ep_cut(&sim, nodes_addr, nodes, ix, 0); ep_insert(&sim, nodes_addr, nodes, iy, ix, 0); nodes[ix].word = word;
            if (a.width == b.width) { ep_cut(&sim, nodes_addr, nodes, iy, 0); ep_cut(&sim, nodes_addr, nodes, iy, 1); } else nodes[iy].width -= w;
        } else { ep_cut(&sim, nodes_addr, nodes, iy, 1); ep_insert(&sim, nodes_addr, nodes, ix, iy, 1); nodes[iy].word = word; nodes[ix].width -= w; }
    }
    uint32_t *bottom = (uint32_t *)malloc(nnodes * 4); uint32_t id = nodes[0].next[1]; x = 0;
    while (id != 0) { bottom[id] = x; x += nodes[id].width; id = nodes[id].next[1]; }
    Band8 *bands = (Band8 *)malloc((nnodes + 16) * sizeof(Band8)); uint64_t bands_addr = sim_alloc_region(&sim, (nnodes + 16) * sizeof(Band8));
    size_t cap_spell = 1 << 17, nspell = 0; uint8_t *spelling = (uint8_t *)malloc(cap_spell); uint64_t spell_addr = sim_alloc_region(&sim, 1 << 20);
    uint32_t *offsets = (uint32_t *)malloc((nnodes + 16) * 4); uint64_t off_addr = sim_alloc_region(&sim, (nnodes + 16) * 4);
    offsets[0] = 0; uint32_t ntw = 0, nbands = 0; uint32_t *stack = (uint32_t *)malloc(65536 * 4);
    id = nodes[0].next[0]; uint32_t k = 0;
    while (id != 0) {
        EpNode nd = nodes[id]; uint32_t dst = bottom[id], tw_id = 0;
        if (nd.word != 0) {
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
            offsets[++ntw] = (uint32_t)nspell; sim_access(&sim, off_addr + (uint64_t)ntw * 4, 4, 1); tw_id = ntw;
        }
        uint32_t end_k = k + nd.width;
        bands[nbands] = (Band8){ end_k | (tw_id << 18), (int32_t)dst - (int32_t)k };
        sim_access(&sim, bands_addr + (uint64_t)nbands * sizeof(Band8), sizeof(Band8), 1);
        nbands++; k = end_k; id = nd.next[0];
    }
    uint32_t nhint = (m + 15u) >> 4;
    uint16_t *band_hint = (uint16_t *)malloc((size_t)(nhint + 2) * 2); uint64_t hint_addr = sim_alloc_region(&sim, (size_t)(nhint + 2) * 2);
    uint32_t bi = 0;
    for (uint32_t h = 0; h < nhint; h++) {
        uint32_t y0 = h << 4;
        while ((bands[bi].end_code & 0x3FFFFu) <= y0) bi++;
        band_hint[h] = (uint16_t)bi; sim_access(&sim, hint_addr + (uint64_t)h * 2, 2, 1);
    }
    size_t ep_mem = nstrips * sizeof(OpenStrip) + nnodes * sizeof(EpNode) + nrules * sizeof(EpRule);
    free(strips); free(image_order); free(nodes); free(ord0); free(ord1); free(term0); free(term1); free(rules); free(bottom); free(stack);
    met->l2_p2 = sim.l2.misses - met->l2_p1 - met->l2_p3; double t2 = now_ms(); met->ms_p2 = t2 - t_bs;

    uint8_t *out = (uint8_t *)malloc(n); uint64_t out_addr = sim_alloc_region(&sim, n);

    if (nlanes <= 1) {
        uint32_t out_pos = n, z = mark;
        for (uint32_t step = 0; step < m && out_pos > 0; step++) {
            uint8_t lit_b = 0;
            uint32_t y = bso512_lf_branchless(sbs, sbs_addr, C_code, code2sym, esc_lf, esc_lf_addr, esc_byte, esc_b_addr, z, &lit_b, &sim);
            out[--out_pos] = lit_b; sim_access(&sim, out_addr + out_pos, 1, 1);
            sim_access(&sim, hint_addr + (uint64_t)(y >> 4) * 2, 2, 0);
            uint32_t bidx = band_hint[y >> 4];
            sim_access(&sim, bands_addr + (uint64_t)bidx * sizeof(Band8), sizeof(Band8), 0);
            bidx += ((bands[bidx].end_code & 0x3FFFFu) <= y);
            Band8 bd = bands[bidx];
            while (__builtin_expect((bd.end_code & 0x3FFFFu) <= y, 0)) {
                bidx++; sim_access(&sim, bands_addr + (uint64_t)bidx * sizeof(Band8), sizeof(Band8), 0); bd = bands[bidx];
            }
            z = (uint32_t)((int32_t)y + bd.delta);
            uint32_t tw_id = bd.end_code >> 18;
            if (__builtin_expect(tw_id > 0, 0) && out_pos > 0) {
                sim_access(&sim, off_addr + (uint64_t)(tw_id - 1) * 4, 8, 0);
                uint32_t st = offsets[tw_id - 1], ln = offsets[tw_id] - st;
                if (ln > out_pos) { st += (ln - out_pos); ln = out_pos; }
                out_pos -= ln;
                sim_access(&sim, spell_addr + st, ln, 0); memcpy(out + out_pos, spelling + st, ln); sim_access(&sim, out_addr + out_pos, ln, 1);
            }
        }
        assert(out_pos == 0);
        met->l2_p4a = 0; met->ms_p4a = 0.0;
        met->l2_p4b = sim.l2.misses - met->l2_p1 - met->l2_p2 - met->l2_p3; double t4 = now_ms(); met->ms_p4b = t4 - t2;
    } else {
        /* 4-lane lockstep Branchless BSO-512 with z & 4095 == 0 zero-bitmap splitters */
        const int K = 4;
        uint32_t n_grid = (m + 4095u) >> 12;
        int mark_on_grid = ((mark & 4095u) == 0);
        uint32_t n_sp = mark_on_grid ? n_grid : (n_grid + 1);
        uint32_t sp_start[256], sp_next[256], sp_bytes[256], sp_steps[256];
        for (uint32_t i = 0; i < n_grid; i++) sp_start[i] = i << 12;
        uint32_t mark_sp = mark_on_grid ? (mark >> 12) : n_grid;
        if (!mark_on_grid) sp_start[mark_sp] = mark;

        uint32_t lz[4], lbytes[4], lsteps[4], lsp[4]; int lact[4] = {0};
        uint32_t next_sp = 0, act = 0;
        for (int i = 0; i < K && next_sp < n_sp; i++) { lsp[i] = next_sp; lz[i] = sp_start[next_sp]; lbytes[i] = 0; lsteps[i] = 0; lact[i] = 1; next_sp++; act++; }
        while (act > 0) {
            uint32_t ly[4];
            for (int i = 0; i < K; i++) {
                if (lact[i]) ly[i] = bso512_lf_branchless(sbs, sbs_addr, C_code, code2sym, esc_lf, esc_lf_addr, esc_byte, esc_b_addr, lz[i], NULL, &sim);
            }
            for (int i = 0; i < K; i++) {
                if (!lact[i]) continue;
                uint32_t y = ly[i];
                sim_access(&sim, hint_addr + (uint64_t)(y >> 4) * 2, 2, 0);
                uint32_t bidx = band_hint[y >> 4];
                sim_access(&sim, bands_addr + (uint64_t)bidx * sizeof(Band8), sizeof(Band8), 0);
                bidx += ((bands[bidx].end_code & 0x3FFFFu) <= y);
                Band8 bd = bands[bidx];
                while (__builtin_expect((bd.end_code & 0x3FFFFu) <= y, 0)) { bidx++; sim_access(&sim, bands_addr + (uint64_t)bidx * sizeof(Band8), sizeof(Band8), 0); bd = bands[bidx]; }
                uint32_t nz = (uint32_t)((int32_t)y + bd.delta), tw_id = bd.end_code >> 18;
                uint32_t add_b = 1;
                if (__builtin_expect(tw_id > 0, 0)) { sim_access(&sim, off_addr + (uint64_t)(tw_id - 1) * 4, 8, 0); add_b += offsets[tw_id] - offsets[tw_id - 1]; }
                lbytes[i] += add_b; lsteps[i]++; lz[i] = nz;
                if (__builtin_expect(((nz & 4095u) == 0) || (nz == mark), 0)) {
                    uint32_t dst_sp = (nz == mark) ? mark_sp : (nz >> 12), src_sp = lsp[i];
                    sp_next[src_sp] = dst_sp; sp_bytes[src_sp] = lbytes[i]; sp_steps[src_sp] = lsteps[i];
                    if (next_sp < n_sp) { lsp[i] = next_sp; lz[i] = sp_start[next_sp]; lbytes[i] = 0; lsteps[i] = 0; next_sp++; }
                    else { lact[i] = 0; act--; }
                }
            }
        }
        met->l2_p4a = sim.l2.misses - met->l2_p1 - met->l2_p2 - met->l2_p3; double t3 = now_ms(); met->ms_p4a = t3 - t2;

        uint32_t cyc_sp[256], cyc_pos[256], cyc_st[256], n_cyc = 0;
        uint32_t cur_sp = mark_sp, cur_pos = n;
        do { cyc_sp[n_cyc] = cur_sp; cyc_pos[n_cyc] = cur_pos; cyc_st[n_cyc] = sp_steps[cur_sp]; cur_pos -= sp_bytes[cur_sp]; cur_sp = sp_next[cur_sp]; n_cyc++; } while (cur_sp != mark_sp);
        assert(cur_pos == 0);

        uint32_t lpos[4], lrem[4]; uint8_t llit[4]; uint32_t next_c = 0; act = 0;
        for (int i = 0; i < K && next_c < n_cyc; i++) { lz[i] = sp_start[cyc_sp[next_c]]; lpos[i] = cyc_pos[next_c]; lrem[i] = cyc_st[next_c]; lact[i] = 1; next_c++; act++; }
        while (act > 0) {
            uint32_t ly[4];
            for (int i = 0; i < K; i++) {
                if (lact[i]) ly[i] = bso512_lf_branchless(sbs, sbs_addr, C_code, code2sym, esc_lf, esc_lf_addr, esc_byte, esc_b_addr, lz[i], &llit[i], &sim);
            }
            for (int i = 0; i < K; i++) {
                if (!lact[i]) continue;
                uint32_t y = ly[i], opos = lpos[i] - 1;
                out[opos] = llit[i]; sim_access(&sim, out_addr + opos, 1, 1);
                sim_access(&sim, hint_addr + (uint64_t)(y >> 4) * 2, 2, 0);
                uint32_t bidx = band_hint[y >> 4];
                sim_access(&sim, bands_addr + (uint64_t)bidx * sizeof(Band8), sizeof(Band8), 0);
                bidx += ((bands[bidx].end_code & 0x3FFFFu) <= y);
                Band8 bd = bands[bidx];
                while (__builtin_expect((bd.end_code & 0x3FFFFu) <= y, 0)) { bidx++; sim_access(&sim, bands_addr + (uint64_t)bidx * sizeof(Band8), sizeof(Band8), 0); bd = bands[bidx]; }
                lz[i] = (uint32_t)((int32_t)y + bd.delta);
                uint32_t tw_id = bd.end_code >> 18;
                if (__builtin_expect(tw_id > 0, 0)) {
                    sim_access(&sim, off_addr + (uint64_t)(tw_id - 1) * 4, 8, 0);
                    uint32_t st = offsets[tw_id - 1], ln = offsets[tw_id] - st; opos -= ln;
                    sim_access(&sim, spell_addr + st, ln, 0); memcpy(out + opos, spelling + st, ln); sim_access(&sim, out_addr + opos, ln, 1);
                }
                lpos[i] = opos;
                if (__builtin_expect(--lrem[i] == 0, 0)) {
                    if (next_c < n_cyc) { lz[i] = sp_start[cyc_sp[next_c]]; lpos[i] = cyc_pos[next_c]; lrem[i] = cyc_st[next_c]; next_c++; }
                    else { lact[i] = 0; act--; }
                }
            }
        }
        met->l2_p4b = sim.l2.misses - met->l2_p1 - met->l2_p2 - met->l2_p3 - met->l2_p4a; double t4 = now_ms(); met->ms_p4b = t4 - t3;
    }

    size_t emit_mem = (size_t)(n_sb + 1) * sizeof(SB512) + (size_t)(m_esc + 4) * 5
                    + (size_t)nbands * sizeof(Band8) + (size_t)nhint * 2 + nspell + (size_t)(ntw + 1) * 4 + n;
    size_t p2_mem = (size_t)(n_sb + 1) * sizeof(SB512) + ep_mem + nrecs * 2;
    met->peak_mem = emit_mem > p2_mem ? emit_mem : p2_mem;
    sim_drain(&sim);
    met->l1_misses = sim.l1.misses; met->l2_misses = sim.l2.misses; met->llc_misses = sim.llc.misses;
    met->dram_bytes = (sim.dram_reads + sim.dram_writes) * 64ULL; met->m_ret = m;
    free(sbs); free(esc_lf); free(esc_byte); free(bands); free(band_hint); free(spelling); free(offsets); sim_free(&sim);
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
    printf("=== Cycle 5 G5: Shipped Open vs Branchless BSO-512 (1-lane) vs Branchless BSO-512 (4-lane) ===\n\n");
    int p1 = 1, p4 = 1;
    for (int bi = 0; bi < 6; bi++) {
        char path[512]; snprintf(path, sizeof(path), "%s/%s", dir, BLOCKS[bi].file);
        FILE *fp = fopen(path, "rb"); assert(fp != NULL);
        uint32_t n = BLOCKS[bi].n, p = BLOCKS[bi].p;
        uint8_t *L = (uint8_t *)malloc(n); assert(fread(L, 1, n, fp) == n); fclose(fp);
        size_t nrecs = 0; uint16_t *recs = column_to_records(L, n, &nrecs);
        uint8_t *ref = dense_reference(L, n, p);

        Metrics mf, m1, m4;
        uint8_t *of = run_forward_open(recs, nrecs, n, p, &mf);
        uint8_t *o1 = run_bso_branchless(recs, nrecs, n, p, 1, &m1);
        uint8_t *o4 = run_bso_branchless(recs, nrecs, n, p, 4, &m4);
        assert(memcmp(of, ref, n) == 0 && memcmp(o1, ref, n) == 0 && memcmp(o4, ref, n) == 0);

        double df = (double)mf.dram_bytes / n, d1 = (double)m1.dram_bytes / n, d4 = (double)m4.dram_bytes / n;
        if (!((double)m1.l2_misses/mf.l2_misses <= 0.85 && d1/df <= 0.90 && (double)m1.peak_mem/mf.peak_mem <= 1.15)) p1 = 0;
        if (!((double)m4.l2_misses/mf.l2_misses <= 0.85 && d4/df <= 0.90 && (double)m4.peak_mem/mf.peak_mem <= 1.15)) p4 = 0;

        printf("%s (m=%u):\n", BLOCKS[bi].file, mf.m_ret);
        printf("  Shipped Open   : L2m=%7lu [p1=%5lu ep=%5lu wir=%5lu em=%6lu] LLC=%5lu DRAM=%.2f B/B Peak=%.2f MB\n",
               (unsigned long)mf.l2_misses, (unsigned long)mf.l2_p1, (unsigned long)mf.l2_p2, (unsigned long)mf.l2_p3, (unsigned long)mf.l2_p4b,
               (unsigned long)mf.llc_misses, df, mf.peak_mem/1e6);
        printf("  BSO-512-BL(1x) : L2m=%7lu (%.3fx) [p1=%4lu bs=%4lu ep=%5lu em=%5lu] LLC=%5lu DRAM=%.2f (%.3fx) Peak=%.2f (%.3fx)\n",
               (unsigned long)m1.l2_misses, (double)m1.l2_misses/mf.l2_misses,
               (unsigned long)m1.l2_p1, (unsigned long)m1.l2_p3, (unsigned long)m1.l2_p2, (unsigned long)m1.l2_p4b,
               (unsigned long)m1.llc_misses, d1, d1/df, m1.peak_mem/1e6, (double)m1.peak_mem/mf.peak_mem);
        printf("  BSO-512-BL(4x) : L2m=%7lu (%.3fx) [p1=%4lu bs=%4lu ep=%5lu 4a=%5lu 4b=%5lu] LLC=%5lu DRAM=%.2f (%.3fx) Peak=%.2f (%.3fx)\n\n",
               (unsigned long)m4.l2_misses, (double)m4.l2_misses/mf.l2_misses,
               (unsigned long)m4.l2_p1, (unsigned long)m4.l2_p3, (unsigned long)m4.l2_p2, (unsigned long)m4.l2_p4a, (unsigned long)m4.l2_p4b,
               (unsigned long)m4.llc_misses, d4, d4/df, m4.peak_mem/1e6, (double)m4.peak_mem/mf.peak_mem);
        free(L); free(recs); free(ref); free(of); free(o1); free(o4);
    }
    printf("CACHE-MODEL VERDICT (6/6): BSO-512-BL(1x)=%s | BSO-512-BL(4x)=%s\n", p1 ? "PASS" : "FAIL", p4 ? "PASS" : "FAIL");
#else
    printf("=== VM Single-Thread C Timing (Open vs Branchless BSO-512 1x & 4x, -O3 -mavx2 -mpopcnt, 15 rounds) ===\n");
    const int ROUNDS = 15;
    for (int bi = 0; bi < 6; bi++) {
        char path[512]; snprintf(path, sizeof(path), "%s/%s", dir, BLOCKS[bi].file);
        FILE *fp = fopen(path, "rb"); assert(fp != NULL);
        uint32_t n = BLOCKS[bi].n, p = BLOCKS[bi].p;
        uint8_t *L = (uint8_t *)malloc(n); assert(fread(L, 1, n, fp) == n); fclose(fp);
        size_t nrecs = 0; uint16_t *recs = column_to_records(L, n, &nrecs);
        uint8_t *ref = dense_reference(L, n, p);
        double tf[15], t1[15], t4[15]; Metrics mf, m1, m4;
        for (int r = 0; r < ROUNDS; r++) {
            double a = now_ms(); uint8_t *of = run_forward_open(recs, nrecs, n, p, &mf); double b = now_ms(); tf[r] = b - a;
            a = now_ms(); uint8_t *o1 = run_bso_branchless(recs, nrecs, n, p, 1, &m1); b = now_ms(); t1[r] = b - a;
            a = now_ms(); uint8_t *o4 = run_bso_branchless(recs, nrecs, n, p, 4, &m4); b = now_ms(); t4[r] = b - a;
            if (r == 0) assert(memcmp(of, ref, n) == 0 && memcmp(o1, ref, n) == 0 && memcmp(o4, ref, n) == 0);
            free(of); free(o1); free(o4);
        }
        qsort(tf, ROUNDS, sizeof(double), cmp_dbl); qsort(t1, ROUNDS, sizeof(double), cmp_dbl); qsort(t4, ROUNDS, sizeof(double), cmp_dbl);
        printf("  %-11s: Open=%.2f ms [em=%.2f] | BL-1x=%.2f ms (%.2fx) [p1=%.2f bs=%.2f ep=%.2f em=%.2f] | BL-4x=%.2f ms (%.2fx) [4a=%.2f 4b=%.2f]\n",
               BLOCKS[bi].file, tf[7], mf.ms_p4b,
               t1[7], t1[7]/tf[7], m1.ms_p1, m1.ms_p3, m1.ms_p2, m1.ms_p4b,
               t4[7], t4[7]/tf[7], m4.ms_p4a, m4.ms_p4b);
        free(L); free(recs); free(ref);
    }
#endif
    return 0;
}
