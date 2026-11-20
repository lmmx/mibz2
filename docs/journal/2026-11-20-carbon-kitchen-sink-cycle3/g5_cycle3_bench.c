/*
 * Carbon Kitchen Sink — Cycle 3 Gate G5 Pre-Registered C Falsifier (g5_cycle3_bench.c)
 *
 * Factorial 2x2 (+ baseline) evaluation on Pre-Column `Symbols -> n-byte text` across all 6
 * authenticated Wikidata BWT blocks (21 paired interleaved rounds after 3 warm-up rounds;
 * verified clean under ASan/UBSan, byte-exact against dense LF, and 64B canary checked):
 *
 *   C0:2Stage-qi16   : Shipped AVX2 `runs::front` (`Symbols -> CAP=127 u16 records`)
 *                      + shipped `Open<16, true, true>` (`qi16`)
 *   C1:2Stage-fast16 : Shipped AVX2 `runs::front` + `Open-fast16` (Cycle 2's B0)
 *   C2:2Stage-Vec16  : Shipped AVX2 `runs::front` + `Open-Vec16` (non-4K-aliased Z^16 branchless
 *                      AVX2 queue iota + 8-wide AVX2 band wire + unified Free-Monoid Slot16 emit)
 *   C3:LRB-qi16      : Direct Left-Regular-Band `Symbols -> Open-qi16` (isolates pure LRB fusion)
 *   C4:LRB-Vec16     : Synthesised Left-Regular-Band `Symbols -> Open-Vec16`
 *
 * Pre-registered Gate G5 pass criteria:
 *   1. Synthesis vs shipped (`C4 / C0`): <= 0.850x on >= 5/6 blocks, all 6/6 <= 0.900x.
 *   2. Unconfounded marginal LRB gain (`C4 / C2`): <= 0.930x on >= 5/6 blocks, all 6/6 < 1.000x.
 */

#define _POSIX_C_SOURCE 200809L
#include <assert.h>
#include <immintrin.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#define MAX_N 900100
#define MAX_SYMS 900100
#define MAX_RECS 900100
#define MAX_RUNS 900100
#define MAX_CODES (256 + 131072)
#define MAX_NODES 100000
#define MAX_RULES 100000
#define OUT_PAD 128
#define CANARY_LEN 64
#define CANARY_BYTE 0xA5
#define LIMIT 16u

static inline uint64_t now_ns(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (uint64_t)ts.tv_sec * 1000000000ULL + (uint64_t)ts.tv_nsec;
}

typedef struct __attribute__((aligned(64))) {
    uint8_t a[16], b[16], c[16], d[16];
} MtfStep;

static MtfStep MTF_STEPS[32];

static void init_mtf_steps(void) {
    const uint8_t Z = 0x80u;
    for (int nn = 0; nn < 32; nn++) {
        for (int i = 0; i < 16; i++) {
            MTF_STEPS[nn].b[i] = Z;
            MTF_STEPS[nn].d[i] = Z;
            if (nn < 16) {
                MTF_STEPS[nn].a[i] = (i == 0) ? (uint8_t)nn : (i <= nn ? (uint8_t)(i - 1) : (uint8_t)i);
                MTF_STEPS[nn].c[i] = (uint8_t)i;
            } else {
                MTF_STEPS[nn].a[i] = (i == 0) ? Z : (uint8_t)(i - 1);
                MTF_STEPS[nn].c[i] = (i == 0) ? Z : (i <= nn - 16 ? (uint8_t)(i - 1) : (uint8_t)i);
                if (i == 0) {
                    MTF_STEPS[nn].b[i] = (uint8_t)(nn - 16);
                    MTF_STEPS[nn].d[i] = 15u;
                }
            }
        }
    }
}

static inline uint8_t mtf_second_avx2(uint8_t *tail, uint32_t t, __m128i f1, __m256i idx) {
    uint8_t v = tail[t];
    __m256i below0 = _mm256_inserti128_si256(_mm256_castsi128_si256(f1), f1, 1);
    if (t < 32u) {
        __m256i r0 = _mm256_load_si256((const __m256i *)tail);
        __m256i on0 = _mm256_alignr_epi8(r0, _mm256_permute2x128_si256(r0, below0, 0x03), 15);
        __m256i m = _mm256_cmpgt_epi8(_mm256_set1_epi8((char)(t + 1)), idx);
        _mm256_store_si256((__m256i *)tail, _mm256_blendv_epi8(r0, on0, m));
    } else {
        __m256i tv = _mm256_set1_epi8((char)(uint8_t)t);
        __m256i r[7];
        for (int j = 0; j < 7; j++) r[j] = _mm256_load_si256((const __m256i *)(tail + 32 * j));
        for (int j = 0; j < 7; j++) {
            __m256i p = _mm256_add_epi8(idx, _mm256_set1_epi8((char)(uint8_t)(32 * j)));
            __m256i m = _mm256_cmpeq_epi8(_mm256_min_epu8(p, tv), p);
            __m256i below = (j == 0) ? below0 : r[j - 1];
            __m256i onj = _mm256_alignr_epi8(r[j], _mm256_permute2x128_si256(r[j], below, 0x03), 15);
            _mm256_store_si256((__m256i *)(tail + 32 * j), _mm256_blendv_epi8(r[j], onj, m));
        }
    }
    return v;
}

static uint32_t encode_bzip2_symbols(
    const uint8_t *L, uint32_t n,
    uint8_t seq_to_unseq[256], uint16_t *syms, uint16_t *out_eob
) {
    uint8_t present[256] = {0};
    for (uint32_t i = 0; i < n; i++) present[L[i]] = 1;
    uint8_t yy[256];
    int alpha = 0;
    memset(seq_to_unseq, 0, 256);
    for (int c = 0; c < 256; c++) {
        if (present[c]) {
            seq_to_unseq[alpha] = (uint8_t)c;
            yy[alpha] = (uint8_t)c;
            alpha++;
        }
    }
    uint16_t eob = (uint16_t)(alpha + 1);
    *out_eob = eob;
    uint32_t ns = 0, i = 0;
    while (i < n) {
        uint8_t c = L[i];
        uint32_t j = i + 1;
        while (j < n && L[j] == c) j++;
        uint32_t w = j - i;
        int idx = 0;
        while (yy[idx] != c) idx++;
        uint32_t rem;
        if (idx == 0) { assert(i == 0); rem = w; }
        else {
            syms[ns++] = (uint16_t)(idx + 1);
            memmove(yy + 1, yy, (size_t)idx);
            yy[0] = c;
            rem = w - 1u;
        }
        while (rem > 0) {
            rem -= 1u;
            syms[ns++] = (uint16_t)(rem & 1u);
            rem >>= 1;
        }
        i = j;
    }
    syms[ns++] = eob;
    return ns;
}

static void dense_lf_invert(const uint8_t *L, uint32_t n, uint32_t p, uint8_t *out) {
    uint32_t counts[256] = {0};
    for (uint32_t i = 0; i < n; i++) counts[L[i]]++;
    uint32_t occ[256], acc = 0;
    for (int c = 0; c < 256; c++) { occ[c] = acc; acc += counts[c]; }
    uint32_t *lf = (uint32_t *)malloc(n * sizeof(uint32_t));
    for (uint32_t i = 0; i < n; i++) lf[i] = occ[L[i]]++;
    uint8_t *rev = (uint8_t *)malloc(n);
    uint32_t cur = p, plen = 0;
    for (uint32_t step = 0; step < n; step++) {
        rev[plen++] = L[cur];
        cur = lf[cur];
        if (cur == p) break;
    }
    for (uint32_t i = 0; i < plen; i++) out[i] = rev[plen - 1 - i];
    uint32_t wr = plen;
    while (wr < n) {
        uint32_t take = (n - wr) < plen ? (n - wr) : plen;
        memcpy(out + wr, out, take);
        wr += take;
    }
    free(rev);
    free(lf);
}

typedef struct { uint32_t prev[2], next[2], width, word; } RvNode;
typedef struct { uint32_t left, right, len, first; } RvRule;
typedef struct { uint32_t from, to, width; uint8_t byte; } Strip;
typedef struct { uint32_t end, delta, code; } Band;
typedef struct __attribute__((aligned(16))) { uint8_t bytes[16]; } Slot16;
typedef struct { uint32_t s0, len; } LongWord;

typedef struct {
    double t_front_us;
    double t_pass1_us;
    double t_rv_us;
    double t_wire_us;
    double t_emit_us;
    double t_total_us;
} BenchStats;

typedef struct {
    /* Standard queues (for qi16 / fast16) */
    uint32_t *q_vecs[256];
    uint32_t q_len[256];
    uint32_t q_cap[256];

    /* Non-4K-aliased queues with +16 vector slack (for Vec16 arms C2 & C4) */
    uint32_t *vq_vecs[256];
    uint32_t vq_cap[256];

    Strip *strips;
    uint32_t *image_order;
    Band *bands;
    uint8_t *spelling;
    uint32_t *offsets;
    uint32_t *edges;
    uint32_t *extra;

    /* Unified Free-Monoid Slot16 table */
    Slot16 *slot16;
    uint32_t *slot_len;
    LongWord *long_words;

    RvNode *nodes;
    RvRule *rules;
    uint32_t *order0, *order1, *term0, *term1, *bottom, *stack;

    uint64_t *lit_bmap;
    uint64_t *one_bmap;
    uint16_t *front_recs;
} Workspace;

static Workspace *ws_create(void) {
    Workspace *w = (Workspace *)calloc(1, sizeof(Workspace));
    for (int a = 0; a < 256; a++) {
        w->q_cap[a] = 4096;
        w->q_vecs[a] = (uint32_t *)malloc(w->q_cap[a] * sizeof(uint32_t));
        /* Coprime stride 4112 u32s = 16,448B = 257 cache lines (avoids 64-set 4K L1d aliasing!) */
        w->vq_cap[a] = 4112;
        w->vq_vecs[a] = (uint32_t *)malloc(w->vq_cap[a] * sizeof(uint32_t));
    }
    w->strips = (Strip *)malloc(MAX_RUNS * sizeof(Strip));
    w->image_order = (uint32_t *)malloc(MAX_RUNS * sizeof(uint32_t));
    w->bands = (Band *)malloc(MAX_RUNS * sizeof(Band));
    w->spelling = (uint8_t *)calloc(MAX_N + 1024, 1);
    w->offsets = (uint32_t *)malloc(MAX_RUNS * sizeof(uint32_t));
    w->edges = (uint32_t *)malloc((MAX_N + 32) * sizeof(uint32_t));
    w->extra = (uint32_t *)malloc((MAX_N + 32) * sizeof(uint32_t));

    if (posix_memalign((void **)&w->slot16, 64, MAX_CODES * sizeof(Slot16)) != 0) abort();
    memset(w->slot16, 0, MAX_CODES * sizeof(Slot16));
    w->slot_len = (uint32_t *)malloc(MAX_CODES * sizeof(uint32_t));
    w->long_words = (LongWord *)malloc(MAX_CODES * sizeof(LongWord));
    for (int a = 0; a < 256; a++) {
        w->slot16[a].bytes[0] = (uint8_t)a;
        w->slot_len[a] = 1u;
    }

    w->nodes = (RvNode *)malloc(MAX_NODES * sizeof(RvNode));
    w->rules = (RvRule *)malloc(MAX_RULES * sizeof(RvRule));
    w->order0 = (uint32_t *)malloc(MAX_NODES * sizeof(uint32_t));
    w->order1 = (uint32_t *)malloc(MAX_NODES * sizeof(uint32_t));
    w->term0 = (uint32_t *)malloc(MAX_NODES * sizeof(uint32_t));
    w->term1 = (uint32_t *)malloc(MAX_NODES * sizeof(uint32_t));
    w->bottom = (uint32_t *)malloc(MAX_NODES * sizeof(uint32_t));
    w->stack = (uint32_t *)malloc(MAX_RULES * 2 * sizeof(uint32_t));

    w->lit_bmap = (uint64_t *)malloc((MAX_SYMS / 64 + 8) * sizeof(uint64_t));
    w->one_bmap = (uint64_t *)malloc((MAX_SYMS / 64 + 8) * sizeof(uint64_t));
    w->front_recs = (uint16_t *)malloc(MAX_RECS * sizeof(uint16_t));
    return w;
}

static void ws_destroy(Workspace *w) {
    for (int a = 0; a < 256; a++) {
        free(w->q_vecs[a]);
        free(w->vq_vecs[a]);
    }
    free(w->strips); free(w->image_order); free(w->bands); free(w->spelling);
    free(w->offsets); free(w->edges); free(w->extra);
    free(w->slot16); free(w->slot_len); free(w->long_words);
    free(w->nodes); free(w->rules); free(w->order0); free(w->order1);
    free(w->term0); free(w->term1); free(w->bottom); free(w->stack);
    free(w->lit_bmap); free(w->one_bmap); free(w->front_recs);
    free(w);
}

static inline void q_push_scalar(Workspace *ws, uint8_t a, uint32_t start, uint32_t w) {
    uint32_t len = ws->q_len[a];
    if (__builtin_expect(len + w > ws->q_cap[a], 0)) {
        uint32_t ncap = ws->q_cap[a] * 2;
        while (ncap < len + w) ncap *= 2;
        ws->q_cap[a] = ncap;
        ws->q_vecs[a] = (uint32_t *)realloc(ws->q_vecs[a], ncap * sizeof(uint32_t));
    }
    uint32_t *dst = ws->q_vecs[a] + len;
    for (uint32_t i = 0; i < w; i++) dst[i] = start + i;
    ws->q_len[a] = len + w;
}

static uint32_t * __attribute__((noinline)) vq_grow(
    Workspace *ws, uint8_t a, uint32_t *q_cur[256], uint32_t *q_lim[256]
) {
    uint32_t used = (uint32_t)(q_cur[a] - ws->vq_vecs[a]);
    uint32_t ncap = (ws->vq_cap[a] * 2u) | 16u; /* preserve odd-16B / coprime L1d set stride */
    while (ncap < used + 32u) ncap = (ncap * 2u) | 16u;
    ws->vq_cap[a] = ncap;
    ws->vq_vecs[a] = (uint32_t *)realloc(ws->vq_vecs[a], ncap * sizeof(uint32_t));
    q_cur[a] = ws->vq_vecs[a] + used;
    q_lim[a] = ws->vq_vecs[a] + ncap - 16u;
    return q_cur[a];
}

static inline void rv_cut(RvNode *nodes, uint32_t id, int side) {
    uint32_t pr = nodes[id].prev[side], nx = nodes[id].next[side];
    nodes[pr].next[side] = nx; nodes[nx].prev[side] = pr;
}
static inline void rv_insert(RvNode *nodes, uint32_t at, uint32_t id, int side) {
    uint32_t nx = nodes[at].next[side];
    nodes[id].prev[side] = at; nodes[id].next[side] = nx;
    nodes[at].next[side] = id; nodes[nx].prev[side] = id;
}
static inline uint32_t rv_tok_len(const RvRule *rules, uint32_t w) {
    return (w <= 256) ? (w != 0 ? 1u : 0u) : rules[w - 257].len;
}
static inline uint32_t rv_join(RvRule *rules, uint32_t *n_rules, uint32_t l, uint32_t r) {
    if (l == 0) return r;
    if (r == 0) return l;
    uint32_t idx = (*n_rules)++;
    rules[idx] = (RvRule){l, r, rv_tok_len(rules, l) + rv_tok_len(rules, r), UINT32_MAX};
    return idx + 257u;
}

static void rv_eliminate(RvNode *nodes, RvRule *rules, uint32_t *n_rules, uint32_t rem) {
    while (rem > 0) {
        uint32_t x = nodes[0].prev[0], y = nodes[0].prev[1];
        uint32_t wx = nodes[x].width, wy = nodes[y].width;
        uint32_t w = wx < wy ? wx : wy;
        rem -= w;
        if (x == y) { rv_cut(nodes, x, 0); rv_cut(nodes, x, 1); continue; }
        uint32_t word = rv_join(rules, n_rules, nodes[x].word, nodes[y].word);
        if (wx <= wy) {
            rv_cut(nodes, x, 0); rv_insert(nodes, y, x, 0); nodes[x].word = word;
            if (wx == wy) { rv_cut(nodes, y, 0); rv_cut(nodes, y, 1); }
            else nodes[y].width = wy - w;
        } else {
            rv_cut(nodes, y, 1); rv_insert(nodes, x, y, 1); nodes[y].word = word;
            nodes[x].width = wx - w;
        }
    }
}

static uint32_t rv_append_word(uint32_t word, RvRule *rules, uint32_t *stack, uint8_t *spelling, uint32_t sp_len) {
    uint32_t sp = 0; stack[sp++] = word;
    while (sp > 0) {
        uint32_t id = stack[--sp];
        if (id <= 256) spelling[sp_len++] = (uint8_t)(id - 1);
        else {
            RvRule *r = &rules[id - 257];
            if (r->first == UINT32_MAX) {
                r->first = sp_len; stack[sp++] = r->right; stack[sp++] = r->left;
            } else {
                memmove(spelling + sp_len, spelling + r->first, r->len);
                sp_len += r->len;
            }
        }
    }
    return sp_len;
}

/* Standard RV + scalar wire (used by qi16 / fast16 arms C0, C1, C3) */
static void finish_construct_std(
    Workspace *ws, uint32_t n, uint32_t m, uint32_t n_strips,
    const uint32_t all[256], const uint32_t thin[256], const uint32_t thick[256],
    uint32_t *out_bits, int *out_wide, double *t_rv_us, double *t_wire_us
) {
    uint32_t first[256], write_pos[256], tot_a = 0, tot_t = 0;
    for (int a = 0; a < 256; a++) {
        first[a] = tot_a; tot_a += all[a];
        write_pos[a] = tot_t; tot_t += thick[a];
    }
    for (uint32_t si = 0; si < n_strips; si++) {
        uint8_t a = ws->strips[si].byte;
        ws->strips[si].to += first[a];
        ws->image_order[write_pos[a]++] = si;
    }
    uint64_t t1 = now_ns();

    RvNode *nodes = ws->nodes;
    nodes[0] = (RvNode){{0, 0}, {0, 0}, 0, 0};
    uint32_t n_nodes = 1;
    for (uint32_t si = 0; si < n_strips; si++) {
        nodes[n_nodes++] = (RvNode){{0, 0}, {0, 0}, ws->strips[si].width, (uint32_t)ws->strips[si].byte + 1u};
    }
    uint32_t n_o0 = 0, n_o1 = 0, n_t0 = 0, n_t1 = 0, x = 0;
    for (uint32_t si = 0; si < n_strips; si++) {
        if (x < ws->strips[si].from) {
            uint32_t id = n_nodes++;
            nodes[id] = (RvNode){{0, 0}, {0, 0}, ws->strips[si].from - x, 0};
            ws->order0[n_o0++] = id; ws->term1[n_t1++] = id;
        }
        ws->order0[n_o0++] = si + 1u;
        x = ws->strips[si].from + ws->strips[si].width;
    }
    if (x < n) {
        uint32_t id = n_nodes++;
        nodes[id] = (RvNode){{0, 0}, {0, 0}, n - x, 0};
        ws->order0[n_o0++] = id; ws->term1[n_t1++] = id;
    }
    x = 0;
    for (uint32_t idx = 0; idx < n_strips; idx++) {
        uint32_t si = ws->image_order[idx];
        if (x < ws->strips[si].to) {
            uint32_t id = n_nodes++;
            nodes[id] = (RvNode){{0, 0}, {0, 0}, ws->strips[si].to - x, 0};
            ws->order1[n_o1++] = id; ws->term0[n_t0++] = id;
        }
        ws->order1[n_o1++] = si + 1u;
        x = ws->strips[si].to + ws->strips[si].width;
    }
    if (x < n) {
        uint32_t id = n_nodes++;
        nodes[id] = (RvNode){{0, 0}, {0, 0}, n - x, 0};
        ws->order1[n_o1++] = id; ws->term0[n_t0++] = id;
    }

    uint32_t prev = 0;
    for (uint32_t u = 0; u < n_t0; u++) { uint32_t id = ws->term0[u]; nodes[prev].next[0] = id; nodes[id].prev[0] = prev; prev = id; }
    for (uint32_t u = 0; u < n_o0; u++) { uint32_t id = ws->order0[u]; nodes[prev].next[0] = id; nodes[id].prev[0] = prev; prev = id; }
    nodes[prev].next[0] = 0; nodes[0].prev[0] = prev;

    prev = 0;
    for (uint32_t u = 0; u < n_t1; u++) { uint32_t id = ws->term1[u]; nodes[prev].next[1] = id; nodes[id].prev[1] = prev; prev = id; }
    for (uint32_t u = 0; u < n_o1; u++) { uint32_t id = ws->order1[u]; nodes[prev].next[1] = id; nodes[id].prev[1] = prev; prev = id; }
    nodes[prev].next[1] = 0; nodes[0].prev[1] = prev;

    uint32_t n_rules = 0;
    rv_eliminate(nodes, ws->rules, &n_rules, n);

    uint32_t id = nodes[0].next[1]; x = 0;
    while (id != 0) { ws->bottom[id] = x; x += nodes[id].width; id = nodes[id].next[1]; }

    uint32_t cursors[257], n_bands = 0, sp_len = 0, n_words = 0;
    ws->offsets[0] = 0;
    id = nodes[0].next[0];
    uint32_t in_node = 0, k = 0;
    for (uint32_t a = 0; a < 256; a++) {
        cursors[a] = n_bands;
        uint32_t end_a = k + thin[a];
        while (k < end_a) {
            uint32_t nw = nodes[id].width;
            uint32_t w = (end_a - k) < (nw - in_node) ? (end_a - k) : (nw - in_node);
            uint32_t dst = ws->bottom[id] + in_node;
            uint32_t code;
            if (nodes[id].word == 0) code = a;
            else {
                sp_len = rv_append_word(nodes[id].word, ws->rules, ws->stack, ws->spelling, sp_len);
                ws->spelling[sp_len++] = (uint8_t)a;
                n_words++; ws->offsets[n_words] = sp_len;
                code = 255u + n_words;
            }
            ws->bands[n_bands++] = (Band){k + w, dst - k, code};
            in_node += w; k += w;
            if (in_node == nw) { id = nodes[id].next[0]; in_node = 0; }
        }
    }
    cursors[256] = n_bands;
    uint64_t t2 = now_ns();

    uint32_t bits = m <= 1 ? 1u : (32u - (uint32_t)__builtin_clz(m - 1u));
    int wide = (255u + n_words) > (UINT32_MAX >> bits);
    k = 0;
    for (int a = 0; a < 256; a++) {
        uint32_t b_start = cursors[a], b_end = cursors[a + 1], from = 0;
        const uint32_t *qsrc = ws->q_vecs[a];
        for (uint32_t bi = b_start; bi < b_end; bi++) {
            Band b = ws->bands[bi];
            uint32_t span = b.end - k, dst = k + b.delta;
            uint32_t pack = wide ? 0u : (b.code << bits);
            if (wide) for (uint32_t u = 0; u < span; u++) ws->extra[dst + u] = b.code;
            uint32_t *edst = ws->edges + dst;
            const uint32_t *src = qsrc + from;
            for (uint32_t u = 0; u < span; u++) edst[u] = pack | src[u];
            from += span; k = b.end;
        }
    }
    uint64_t t3 = now_ns();

    *out_bits = bits; *out_wide = wide;
    *t_rv_us = (double)(t2 - t1) / 1e3;
    *t_wire_us = (double)(t3 - t2) / 1e3;
}

/* Vec16 RV (populating Slot16 monoid table) + 8-wide AVX2 band wiring (used identically by C2 & C4) */
static void finish_construct_vec16(
    Workspace *ws, uint32_t n, uint32_t m, uint32_t n_strips,
    const uint32_t all[256], const uint32_t thin[256], const uint32_t thick[256],
    uint32_t *out_bits, int *out_wide, double *t_rv_us, double *t_wire_us
) {
    uint32_t first[256], write_pos[256], tot_a = 0, tot_t = 0;
    for (int a = 0; a < 256; a++) {
        first[a] = tot_a; tot_a += all[a];
        write_pos[a] = tot_t; tot_t += thick[a];
    }
    for (uint32_t si = 0; si < n_strips; si++) {
        uint8_t a = ws->strips[si].byte;
        ws->strips[si].to += first[a];
        ws->image_order[write_pos[a]++] = si;
    }
    uint64_t t1 = now_ns();

    RvNode *nodes = ws->nodes;
    nodes[0] = (RvNode){{0, 0}, {0, 0}, 0, 0};
    uint32_t n_nodes = 1;
    for (uint32_t si = 0; si < n_strips; si++) {
        nodes[n_nodes++] = (RvNode){{0, 0}, {0, 0}, ws->strips[si].width, (uint32_t)ws->strips[si].byte + 1u};
    }
    uint32_t n_o0 = 0, n_o1 = 0, n_t0 = 0, n_t1 = 0, x = 0;
    for (uint32_t si = 0; si < n_strips; si++) {
        if (x < ws->strips[si].from) {
            uint32_t id = n_nodes++;
            nodes[id] = (RvNode){{0, 0}, {0, 0}, ws->strips[si].from - x, 0};
            ws->order0[n_o0++] = id; ws->term1[n_t1++] = id;
        }
        ws->order0[n_o0++] = si + 1u;
        x = ws->strips[si].from + ws->strips[si].width;
    }
    if (x < n) {
        uint32_t id = n_nodes++;
        nodes[id] = (RvNode){{0, 0}, {0, 0}, n - x, 0};
        ws->order0[n_o0++] = id; ws->term1[n_t1++] = id;
    }
    x = 0;
    for (uint32_t idx = 0; idx < n_strips; idx++) {
        uint32_t si = ws->image_order[idx];
        if (x < ws->strips[si].to) {
            uint32_t id = n_nodes++;
            nodes[id] = (RvNode){{0, 0}, {0, 0}, ws->strips[si].to - x, 0};
            ws->order1[n_o1++] = id; ws->term0[n_t0++] = id;
        }
        ws->order1[n_o1++] = si + 1u;
        x = ws->strips[si].to + ws->strips[si].width;
    }
    if (x < n) {
        uint32_t id = n_nodes++;
        nodes[id] = (RvNode){{0, 0}, {0, 0}, n - x, 0};
        ws->order1[n_o1++] = id; ws->term0[n_t0++] = id;
    }

    uint32_t prev = 0;
    for (uint32_t u = 0; u < n_t0; u++) { uint32_t id = ws->term0[u]; nodes[prev].next[0] = id; nodes[id].prev[0] = prev; prev = id; }
    for (uint32_t u = 0; u < n_o0; u++) { uint32_t id = ws->order0[u]; nodes[prev].next[0] = id; nodes[id].prev[0] = prev; prev = id; }
    nodes[prev].next[0] = 0; nodes[0].prev[0] = prev;

    prev = 0;
    for (uint32_t u = 0; u < n_t1; u++) { uint32_t id = ws->term1[u]; nodes[prev].next[1] = id; nodes[id].prev[1] = prev; prev = id; }
    for (uint32_t u = 0; u < n_o1; u++) { uint32_t id = ws->order1[u]; nodes[prev].next[1] = id; nodes[id].prev[1] = prev; prev = id; }
    nodes[prev].next[1] = 0; nodes[0].prev[1] = prev;

    uint32_t n_rules = 0;
    rv_eliminate(nodes, ws->rules, &n_rules, n);

    uint32_t id = nodes[0].next[1]; x = 0;
    while (id != 0) { ws->bottom[id] = x; x += nodes[id].width; id = nodes[id].next[1]; }

    uint32_t cursors[257], n_bands = 0, sp_len = 0, n_words = 0;
    id = nodes[0].next[0];
    uint32_t in_node = 0, k = 0;
    for (uint32_t a = 0; a < 256; a++) {
        cursors[a] = n_bands;
        uint32_t end_a = k + thin[a];
        while (k < end_a) {
            uint32_t nw = nodes[id].width;
            uint32_t w = (end_a - k) < (nw - in_node) ? (end_a - k) : (nw - in_node);
            uint32_t dst = ws->bottom[id] + in_node;
            uint32_t code;
            if (nodes[id].word == 0) code = a;
            else {
                uint32_t s0 = sp_len;
                sp_len = rv_append_word(nodes[id].word, ws->rules, ws->stack, ws->spelling, sp_len);
                ws->spelling[sp_len++] = (uint8_t)a;
                uint32_t ln = sp_len - s0;
                code = 256u + n_words;
                ws->slot_len[code] = ln;
                if (__builtin_expect(ln <= 16u, 1)) {
                    _mm_store_si128((__m128i *)ws->slot16[code].bytes,
                                    _mm_loadu_si128((const __m128i *)(ws->spelling + s0)));
                } else {
                    ws->long_words[n_words] = (LongWord){s0, ln};
                }
                n_words++;
            }
            ws->bands[n_bands++] = (Band){k + w, dst - k, code};
            in_node += w; k += w;
            if (in_node == nw) { id = nodes[id].next[0]; in_node = 0; }
        }
    }
    cursors[256] = n_bands;
    uint64_t t2 = now_ns();

    uint32_t bits = m <= 1 ? 1u : (32u - (uint32_t)__builtin_clz(m - 1u));
    int wide = (255u + n_words) > (UINT32_MAX >> bits);
    k = 0;
    for (int a = 0; a < 256; a++) {
        uint32_t b_start = cursors[a], b_end = cursors[a + 1], from = 0;
        const uint32_t *qsrc = ws->vq_vecs[a];
        for (uint32_t bi = b_start; bi < b_end; bi++) {
            Band b = ws->bands[bi];
            uint32_t span = b.end - k, dst = k + b.delta;
            uint32_t pack = wide ? 0u : (b.code << bits);
            if (__builtin_expect(wide, 0)) {
                for (uint32_t u = 0; u < span; u++) ws->extra[dst + u] = b.code;
            }
            uint32_t *edst = ws->edges + dst;
            const uint32_t *src = qsrc + from;
            uint32_t u = 0;
            __m256i vpack = _mm256_set1_epi32((int)pack);
            while (u + 8u <= span) {
                __m256i vj = _mm256_loadu_si256((const __m256i *)(src + u));
                _mm256_storeu_si256((__m256i *)(edst + u), _mm256_or_si256(vpack, vj));
                u += 8u;
            }
            for (; u < span; u++) edst[u] = pack | src[u];
            from += span; k = b.end;
        }
    }
    uint64_t t3 = now_ns();

    *out_bits = bits; *out_wide = wide;
    *t_rv_us = (double)(t2 - t1) / 1e3;
    *t_wire_us = (double)(t3 - t2) / 1e3;
}

static void emit_standard(
    const uint32_t *edges, const uint32_t *extra, int wide,
    const uint8_t *spelling, const uint32_t *offsets,
    uint32_t m, uint32_t n, uint32_t mark, uint32_t bits, uint8_t *out
) {
    uint32_t mask = (1u << bits) - 1u, j = mark, out_len = 0;
    for (uint32_t step = 0; step < m; step++) {
        uint32_t edge = edges[j];
        uint32_t code = wide ? extra[j] : (edge >> bits);
        j = edge & mask;
        if (code < 256u) out[out_len++] = (uint8_t)code;
        else {
            uint32_t w = code - 256u;
            uint32_t s0 = offsets[w], len = offsets[w + 1] - s0;
            uint32_t take = (out_len + len <= n) ? len : (n - out_len);
            memcpy(out + out_len, spelling + s0, take);
            out_len += take;
        }
        if (j == mark || out_len >= n) break;
    }
    uint32_t plen = out_len;
    while (out_len < n) {
        uint32_t take = (n - out_len) < plen ? (n - out_len) : plen;
        memcpy(out + out_len, out, take); out_len += take;
    }
}

static void emit_fast16(
    const uint32_t *edges, const uint32_t *extra, int wide,
    const uint8_t *spelling, const uint32_t *offsets,
    uint32_t m, uint32_t n, uint32_t mark, uint32_t bits, uint8_t *out
) {
    uint32_t mask = (1u << bits) - 1u, j = mark, out_len = 0;
    for (uint32_t step = 0; step < m; step++) {
        uint32_t edge = edges[j];
        uint32_t code = wide ? extra[j] : (edge >> bits);
        j = edge & mask;
        if (code < 256u) out[out_len++] = (uint8_t)code;
        else {
            uint32_t w = code - 256u;
            uint32_t s0 = offsets[w], len = offsets[w + 1] - s0;
            if (__builtin_expect(len <= 16u, 1)) {
                memcpy(out + out_len, spelling + s0, 16u);
                out_len += len;
            } else {
                uint32_t take = (out_len + len <= n) ? len : (n - out_len);
                memcpy(out + out_len, spelling + s0, take);
                out_len += take;
            }
        }
        if (j == mark || out_len >= n) break;
    }
    if (out_len > n) out_len = n;
    uint32_t plen = out_len;
    while (out_len < n) {
        uint32_t take = (n - out_len) < plen ? (n - out_len) : plen;
        memcpy(out + out_len, out, take); out_len += take;
    }
}

/* Unified Free-Monoid Slot16 emit (zero `code < 256` branch!) */
static void emit_vec16(
    const uint32_t *edges, const uint32_t *extra, int wide,
    const Slot16 *slot16, const uint32_t *slot_len,
    const LongWord *long_words, const uint8_t *spelling,
    uint32_t m, uint32_t n, uint32_t mark, uint32_t bits, uint8_t *out
) {
    uint32_t mask = (1u << bits) - 1u, j = mark, out_len = 0;
    for (uint32_t step = 0; step < m; step++) {
        uint32_t edge = edges[j];
        uint32_t code = __builtin_expect(wide, 0) ? extra[j] : (edge >> bits);
        j = edge & mask;
        uint32_t ln = slot_len[code];
        if (__builtin_expect(ln <= 16u, 1)) {
            _mm_storeu_si128((__m128i *)(out + out_len), _mm_load_si128((const __m128i *)slot16[code].bytes));
            out_len += ln;
        } else {
            LongWord lw = long_words[code - 256u];
            uint32_t take = (out_len + lw.len <= n) ? lw.len : (n - out_len);
            memcpy(out + out_len, spelling + lw.s0, take);
            out_len += take;
        }
        if (j == mark || out_len >= n) break;
    }
    if (out_len > n) out_len = n;
    uint32_t plen = out_len;
    while (out_len < n) {
        uint32_t take = (n - out_len) < plen ? (n - out_len) : plen;
        memcpy(out + out_len, out, take); out_len += take;
    }
}

/* Shipped AVX2 `runs::front` (`src/runs.rs:front_avx2`) -> CAP=127 u16 records */
static uint32_t front_avx2_to_records(
    Workspace *ws, const uint8_t seq_to_unseq[256],
    const uint16_t *syms, uint32_t m_syms, uint16_t eob_sym, uint32_t *out_n
) {
    uint32_t words = m_syms / 64u + 3u;
    uint32_t *lit32 = (uint32_t *)ws->lit_bmap;
    uint32_t *one32 = (uint32_t *)ws->one_bmap;
    __m256i v1 = _mm256_set1_epi16(1), v0 = _mm256_setzero_si256(), ve = _mm256_set1_epi16((short)eob_sym);

    #define PACK_BITS32(x, y) \
        ((uint32_t)_mm256_movemask_epi8(_mm256_permute4x64_epi64(_mm256_packs_epi16((x), (y)), 0xd8)))

    uint32_t end = UINT32_MAX, ci = 0;
    while (end == UINT32_MAX && ci * 32u + 32u <= m_syms) {
        __m256i x = _mm256_loadu_si256((const __m256i *)(syms + ci * 32u));
        __m256i y = _mm256_loadu_si256((const __m256i *)(syms + ci * 32u + 16u));
        uint32_t run = PACK_BITS32(_mm256_cmpeq_epi16(_mm256_min_epu16(x, v1), x),
                                   _mm256_cmpeq_epi16(_mm256_min_epu16(y, v1), y));
        uint32_t runa = PACK_BITS32(_mm256_cmpeq_epi16(x, v0), _mm256_cmpeq_epi16(y, v0));
        uint32_t eob = PACK_BITS32(_mm256_cmpeq_epi16(x, ve), _mm256_cmpeq_epi16(y, ve)) & ~run;
        uint32_t l = ~run, o = ~runa;
        if (eob != 0) {
            uint32_t t = (uint32_t)__builtin_ctz(eob);
            end = ci * 32u + t;
            uint32_t keep = (uint32_t)((2ULL << t) - 1ULL);
            l &= keep >> 1; o &= keep;
        }
        lit32[ci] = l; one32[ci] = o; ci++;
    }
    if (end == UINT32_MAX) {
        uint32_t l = 0, o = 0;
        for (uint32_t t = 0; ci * 32u + t < m_syms; t++) {
            uint16_t s = syms[ci * 32u + t];
            o |= (uint32_t)(s != 0) << t;
            if (s > 1u) { if (s == eob_sym) { end = ci * 32u + t; break; } l |= 1u << t; }
        }
        lit32[ci] = l; one32[ci] = o; ci++;
    }
    for (uint32_t i = ci; i < 2u * words; i++) { lit32[i] = 0; one32[i] = 0; }
    #undef PACK_BITS32

    __m128i f0 = _mm_loadu_si128((const __m128i *)seq_to_unseq);
    __m128i f1 = _mm_loadu_si128((const __m128i *)(seq_to_unseq + 16));
    uint8_t __attribute__((aligned(32))) tail[224];
    memcpy(tail, seq_to_unseq + 32, 224);
    __m256i idx = _mm256_setr_epi8(
        0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31);

    uint16_t *out = ws->front_recs;
    uint32_t n_rows = 0;
    const uint64_t *litp = ws->lit_bmap, *onep = ws->one_bmap;
    uint32_t last = end / 64u, wi = 0;
    uint64_t w = litp[0];

    #define EMIT_CAP127(byte_c, total_w) do { \
        uint32_t _rem = (uint32_t)(total_w); \
        n_rows += _rem; \
        uint16_t _hi = (uint16_t)((uint16_t)(byte_c) << 8); \
        if (__builtin_expect(_rem <= 127u, 1)) *out++ = _hi | (uint16_t)_rem; \
        else { while (_rem > 0) { uint32_t _l = _rem > 127u ? 127u : _rem; *out++ = _hi | (uint16_t)_l; _rem -= _l; } } \
    } while (0)

    #define SEEK_NEXT(res_tz, has_res) do { \
        has_res = 0; \
        while (1) { \
            if (w != 0) { res_tz = (uint32_t)__builtin_ctzll(w); w &= w - 1ULL; has_res = 1; break; } \
            if (wi == last) break; \
            wi++; w = litp[wi]; \
        } \
    } while (0)

    #define COUNT_BETWEEN(q1, q2) ({ \
        uint32_t _from = (q1) + 1u; uint64_t _bits; \
        memcpy(&_bits, ((const uint8_t *)onep) + (_from >> 3), 8); \
        _bits >>= (_from & 7u); _bzhi_u64(_bits, (q2) - (q1)); \
    })

    uint32_t tz = 0; int has_first = 0;
    SEEK_NEXT(tz, has_first);
    uint32_t q0 = has_first ? (wi * 64u + tz) : end;
    uint64_t init_w = _bzhi_u64(onep[0], q0 + 1u) - 1ULL;
    if (init_w > 0) EMIT_CAP127(seq_to_unseq[0], init_w);

    if (has_first) {
        uint64_t after = onep[wi] >> 1;
        const uint16_t *spw = syms + wi * 64u;
        while (1) {
            uint32_t s = spw[tz];
            if (__builtin_expect(s < 33u, 1)) {
                const MtfStep *st = &MTF_STEPS[s - 1u];
                __m128i a = _mm_load_si128((const __m128i *)st->a);
                __m128i b = _mm_load_si128((const __m128i *)st->b);
                __m128i c = _mm_load_si128((const __m128i *)st->c);
                __m128i d = _mm_load_si128((const __m128i *)st->d);
                __m128i g0 = _mm_or_si128(_mm_shuffle_epi8(f0, a), _mm_shuffle_epi8(f1, b));
                f1 = _mm_or_si128(_mm_shuffle_epi8(f1, c), _mm_shuffle_epi8(f0, d));
                f0 = g0;
            } else {
                uint32_t t_idx = (s - 1u < 255u ? s - 1u : 255u) - 32u;
                uint8_t v = mtf_second_avx2(tail, t_idx, f1, idx);
                f1 = _mm_alignr_epi8(f1, f0, 15);
                f0 = _mm_or_si128(_mm_slli_si128(f0, 1), _mm_cvtsi32_si128((int)v));
            }
            uint8_t c_byte = (uint8_t)_mm_cvtsi128_si32(f0);
            if (__builtin_expect(w != 0, 1)) {
                uint32_t t2 = (uint32_t)__builtin_ctzll(w);
                w &= w - 1ULL;
                EMIT_CAP127(c_byte, _bzhi_u64(after, t2) >> tz);
                tz = t2;
            } else {
                uint32_t q = wi * 64u + tz, tz2 = 0; int has2 = 0;
                SEEK_NEXT(tz2, has2);
                if (has2) {
                    EMIT_CAP127(c_byte, COUNT_BETWEEN(q, wi * 64u + tz2));
                    after = onep[wi] >> 1; spw = syms + wi * 64u; tz = tz2;
                } else {
                    EMIT_CAP127(c_byte, COUNT_BETWEEN(q, end));
                    break;
                }
            }
        }
    }
    #undef EMIT_CAP127
    #undef SEEK_NEXT
    #undef COUNT_BETWEEN

    *out_n = n_rows;
    return (uint32_t)(out - ws->front_recs);
}

/* Two-stage Open-Vec16 pass1 on records (used by C2:2Stage-Vec16) */
static void pass1_records_vec16(
    Workspace *ws, const uint16_t *records, uint32_t n_recs, uint32_t p,
    uint32_t all[256], uint32_t thin[256], uint32_t thick[256],
    uint32_t *out_m, uint32_t *out_mark, uint32_t *out_n_strips
) {
    uint32_t *q_cur[256], *q_lim[256];
    for (int a = 0; a < 256; a++) {
        all[a] = 0; thin[a] = 0; thick[a] = 0;
        q_cur[a] = ws->vq_vecs[a];
        q_lim[a] = ws->vq_vecs[a] + ws->vq_cap[a] - 16u;
    }
    const __m256i iota0 = _mm256_setr_epi32(0, 1, 2, 3, 4, 5, 6, 7);
    const __m256i iota8 = _mm256_setr_epi32(8, 9, 10, 11, 12, 13, 14, 15);

    uint32_t m = 0, mark = 0, n_strips = 0, pos = 0, i = 0;
    while (i < n_recs) {
        uint16_t rec = records[i];
        uint8_t c = (uint8_t)(rec >> 8);
        uint32_t width = (uint32_t)(rec & 255u);
        i++;
        while (i < n_recs && (uint8_t)(records[i] >> 8) == c) {
            width += (uint32_t)(records[i] & 255u);
            i++;
        }
        uint32_t end = pos + width;
        if (width <= LIMIT) {
            if (__builtin_expect(p >= pos && p < end, 0)) mark = m + (p - pos);
            uint32_t *dst = q_cur[c];
            if (__builtin_expect(dst > q_lim[c], 0)) dst = vq_grow(ws, c, q_cur, q_lim);
            __m256i vb = _mm256_set1_epi32((int)m);
            _mm256_storeu_si256((__m256i *)dst, _mm256_add_epi32(vb, iota0));
            _mm256_storeu_si256((__m256i *)(dst + 8), _mm256_add_epi32(vb, iota8));
            q_cur[c] = dst + width;
            thin[c] += width; m += width; all[c] += width;
        } else if (__builtin_expect(p >= pos && p < end, 0)) {
            if (pos < p) {
                uint32_t w1 = p - pos;
                ws->strips[n_strips++] = (Strip){pos, all[c], w1, c};
                thick[c]++; all[c] += w1;
            }
            mark = m;
            uint32_t *dst = q_cur[c];
            if (__builtin_expect(dst > q_lim[c], 0)) dst = vq_grow(ws, c, q_cur, q_lim);
            *dst = m;
            q_cur[c] = dst + 1u;
            thin[c]++; m++; all[c]++;
            if (p + 1u < end) {
                uint32_t w2 = end - (p + 1u);
                ws->strips[n_strips++] = (Strip){p + 1u, all[c], w2, c};
                thick[c]++; all[c] += w2;
            }
        } else {
            ws->strips[n_strips++] = (Strip){pos, all[c], width, c};
            thick[c]++; all[c] += width;
        }
        pos = end;
    }
    *out_m = m; *out_mark = mark; *out_n_strips = n_strips;
}

/* Fused LRB + Vec16 front (used by C4:LRB-Vec16) */
static void lrb_vec16_fused_front(
    Workspace *ws, const uint8_t seq_to_unseq[256],
    const uint16_t *syms, uint32_t m_syms, uint16_t eob_sym, uint32_t p,
    uint32_t all[256], uint32_t thin[256], uint32_t thick[256],
    uint32_t *out_n, uint32_t *out_m, uint32_t *out_mark, uint32_t *out_n_strips
) {
    uint32_t words = m_syms / 64u + 3u;
    uint32_t *lit32 = (uint32_t *)ws->lit_bmap;
    uint32_t *one32 = (uint32_t *)ws->one_bmap;
    __m256i v1 = _mm256_set1_epi16(1), v0 = _mm256_setzero_si256(), ve = _mm256_set1_epi16((short)eob_sym);

    #define PACK_BITS32(x, y) \
        ((uint32_t)_mm256_movemask_epi8(_mm256_permute4x64_epi64(_mm256_packs_epi16((x), (y)), 0xd8)))

    uint32_t end = UINT32_MAX, ci = 0;
    while (end == UINT32_MAX && ci * 32u + 32u <= m_syms) {
        __m256i x = _mm256_loadu_si256((const __m256i *)(syms + ci * 32u));
        __m256i y = _mm256_loadu_si256((const __m256i *)(syms + ci * 32u + 16u));
        uint32_t run = PACK_BITS32(_mm256_cmpeq_epi16(_mm256_min_epu16(x, v1), x),
                                   _mm256_cmpeq_epi16(_mm256_min_epu16(y, v1), y));
        uint32_t runa = PACK_BITS32(_mm256_cmpeq_epi16(x, v0), _mm256_cmpeq_epi16(y, v0));
        uint32_t eob = PACK_BITS32(_mm256_cmpeq_epi16(x, ve), _mm256_cmpeq_epi16(y, ve)) & ~run;
        uint32_t l = ~run, o = ~runa;
        if (eob != 0) {
            uint32_t t = (uint32_t)__builtin_ctz(eob);
            end = ci * 32u + t;
            uint32_t keep = (uint32_t)((2ULL << t) - 1ULL);
            l &= keep >> 1; o &= keep;
        }
        lit32[ci] = l; one32[ci] = o; ci++;
    }
    if (end == UINT32_MAX) {
        uint32_t l = 0, o = 0;
        for (uint32_t t = 0; ci * 32u + t < m_syms; t++) {
            uint16_t s = syms[ci * 32u + t];
            o |= (uint32_t)(s != 0) << t;
            if (s > 1u) { if (s == eob_sym) { end = ci * 32u + t; break; } l |= 1u << t; }
        }
        lit32[ci] = l; one32[ci] = o; ci++;
    }
    for (uint32_t i = ci; i < 2u * words; i++) { lit32[i] = 0; one32[i] = 0; }
    #undef PACK_BITS32

    uint32_t *q_cur[256], *q_lim[256];
    for (int a = 0; a < 256; a++) {
        all[a] = 0; thin[a] = 0; thick[a] = 0;
        q_cur[a] = ws->vq_vecs[a];
        q_lim[a] = ws->vq_vecs[a] + ws->vq_cap[a] - 16u;
    }
    const __m256i iota0 = _mm256_setr_epi32(0, 1, 2, 3, 4, 5, 6, 7);
    const __m256i iota8 = _mm256_setr_epi32(8, 9, 10, 11, 12, 13, 14, 15);

    __m128i f0 = _mm_loadu_si128((const __m128i *)seq_to_unseq);
    __m128i f1 = _mm_loadu_si128((const __m128i *)(seq_to_unseq + 16));
    uint8_t __attribute__((aligned(32))) tail[224];
    memcpy(tail, seq_to_unseq + 32, 224);
    __m256i idx = _mm256_setr_epi8(
        0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31);

    uint32_t pos = 0, m = 0, mark = 0, n_strips = 0;
    const uint64_t *litp = ws->lit_bmap, *onep = ws->one_bmap;
    uint32_t last = end / 64u, wi = 0;
    uint64_t w = litp[0];

    #define VISIT_VEC16(byte_c, total_w) do { \
        uint8_t _c = (uint8_t)(byte_c); \
        uint32_t _width = (uint32_t)(total_w); \
        uint32_t _end = pos + _width; \
        if (_width <= LIMIT) { \
            if (__builtin_expect(p >= pos && p < _end, 0)) mark = m + (p - pos); \
            uint32_t *_dst = q_cur[_c]; \
            if (__builtin_expect(_dst > q_lim[_c], 0)) _dst = vq_grow(ws, _c, q_cur, q_lim); \
            __m256i _vb = _mm256_set1_epi32((int)m); \
            _mm256_storeu_si256((__m256i *)_dst, _mm256_add_epi32(_vb, iota0)); \
            _mm256_storeu_si256((__m256i *)(_dst + 8), _mm256_add_epi32(_vb, iota8)); \
            q_cur[_c] = _dst + _width; \
            thin[_c] += _width; m += _width; all[_c] += _width; \
        } else if (__builtin_expect(p >= pos && p < _end, 0)) { \
            if (pos < p) { \
                uint32_t _w1 = p - pos; \
                ws->strips[n_strips++] = (Strip){pos, all[_c], _w1, _c}; \
                thick[_c]++; all[_c] += _w1; \
            } \
            mark = m; \
            uint32_t *_dst = q_cur[_c]; \
            if (__builtin_expect(_dst > q_lim[_c], 0)) _dst = vq_grow(ws, _c, q_cur, q_lim); \
            *_dst = m; q_cur[_c] = _dst + 1u; \
            thin[_c]++; m++; all[_c]++; \
            if (p + 1u < _end) { \
                uint32_t _w2 = _end - (p + 1u); \
                ws->strips[n_strips++] = (Strip){p + 1u, all[_c], _w2, _c}; \
                thick[_c]++; all[_c] += _w2; \
            } \
        } else { \
            ws->strips[n_strips++] = (Strip){pos, all[_c], _width, _c}; \
            thick[_c]++; all[_c] += _width; \
        } \
        pos = _end; \
    } while (0)

    #define SEEK_NEXT(res_tz, has_res) do { \
        has_res = 0; \
        while (1) { \
            if (w != 0) { res_tz = (uint32_t)__builtin_ctzll(w); w &= w - 1ULL; has_res = 1; break; } \
            if (wi == last) break; \
            wi++; w = litp[wi]; \
        } \
    } while (0)

    #define COUNT_BETWEEN(q1, q2) ({ \
        uint32_t _from = (q1) + 1u; uint64_t _bits; \
        memcpy(&_bits, ((const uint8_t *)onep) + (_from >> 3), 8); \
        _bits >>= (_from & 7u); _bzhi_u64(_bits, (q2) - (q1)); \
    })

    uint32_t tz = 0; int has_first = 0;
    SEEK_NEXT(tz, has_first);
    uint32_t q0 = has_first ? (wi * 64u + tz) : end;
    uint64_t init_w = _bzhi_u64(onep[0], q0 + 1u) - 1ULL;
    if (init_w > 0) VISIT_VEC16(seq_to_unseq[0], init_w);

    if (has_first) {
        uint64_t after = onep[wi] >> 1;
        const uint16_t *spw = syms + wi * 64u;
        while (1) {
            uint32_t s = spw[tz];
            if (__builtin_expect(s < 33u, 1)) {
                const MtfStep *st = &MTF_STEPS[s - 1u];
                __m128i a = _mm_load_si128((const __m128i *)st->a);
                __m128i b = _mm_load_si128((const __m128i *)st->b);
                __m128i c = _mm_load_si128((const __m128i *)st->c);
                __m128i d = _mm_load_si128((const __m128i *)st->d);
                __m128i g0 = _mm_or_si128(_mm_shuffle_epi8(f0, a), _mm_shuffle_epi8(f1, b));
                f1 = _mm_or_si128(_mm_shuffle_epi8(f1, c), _mm_shuffle_epi8(f0, d));
                f0 = g0;
            } else {
                uint32_t t_idx = (s - 1u < 255u ? s - 1u : 255u) - 32u;
                uint8_t v = mtf_second_avx2(tail, t_idx, f1, idx);
                f1 = _mm_alignr_epi8(f1, f0, 15);
                f0 = _mm_or_si128(_mm_slli_si128(f0, 1), _mm_cvtsi32_si128((int)v));
            }
            uint8_t c_byte = (uint8_t)_mm_cvtsi128_si32(f0);
            if (__builtin_expect(w != 0, 1)) {
                uint32_t t2 = (uint32_t)__builtin_ctzll(w);
                w &= w - 1ULL;
                VISIT_VEC16(c_byte, _bzhi_u64(after, t2) >> tz);
                tz = t2;
            } else {
                uint32_t q = wi * 64u + tz, tz2 = 0; int has2 = 0;
                SEEK_NEXT(tz2, has2);
                if (has2) {
                    VISIT_VEC16(c_byte, COUNT_BETWEEN(q, wi * 64u + tz2));
                    after = onep[wi] >> 1; spw = syms + wi * 64u; tz = tz2;
                } else {
                    VISIT_VEC16(c_byte, COUNT_BETWEEN(q, end));
                    break;
                }
            }
        }
    }
    #undef VISIT_VEC16
    #undef SEEK_NEXT
    #undef COUNT_BETWEEN

    *out_n = pos; *out_m = m; *out_mark = mark; *out_n_strips = n_strips;
}

/* Fused LRB + scalar qi16 front (used by C3:LRB-qi16) */
static void lrb_qi16_fused_front(
    Workspace *ws, const uint8_t seq_to_unseq[256],
    const uint16_t *syms, uint32_t m_syms, uint16_t eob_sym, uint32_t p,
    uint32_t all[256], uint32_t thin[256], uint32_t thick[256],
    uint32_t *out_n, uint32_t *out_m, uint32_t *out_mark, uint32_t *out_n_strips
) {
    uint32_t words = m_syms / 64u + 3u;
    uint32_t *lit32 = (uint32_t *)ws->lit_bmap;
    uint32_t *one32 = (uint32_t *)ws->one_bmap;
    __m256i v1 = _mm256_set1_epi16(1), v0 = _mm256_setzero_si256(), ve = _mm256_set1_epi16((short)eob_sym);

    #define PACK_BITS32(x, y) \
        ((uint32_t)_mm256_movemask_epi8(_mm256_permute4x64_epi64(_mm256_packs_epi16((x), (y)), 0xd8)))

    uint32_t end = UINT32_MAX, ci = 0;
    while (end == UINT32_MAX && ci * 32u + 32u <= m_syms) {
        __m256i x = _mm256_loadu_si256((const __m256i *)(syms + ci * 32u));
        __m256i y = _mm256_loadu_si256((const __m256i *)(syms + ci * 32u + 16u));
        uint32_t run = PACK_BITS32(_mm256_cmpeq_epi16(_mm256_min_epu16(x, v1), x),
                                   _mm256_cmpeq_epi16(_mm256_min_epu16(y, v1), y));
        uint32_t runa = PACK_BITS32(_mm256_cmpeq_epi16(x, v0), _mm256_cmpeq_epi16(y, v0));
        uint32_t eob = PACK_BITS32(_mm256_cmpeq_epi16(x, ve), _mm256_cmpeq_epi16(y, ve)) & ~run;
        uint32_t l = ~run, o = ~runa;
        if (eob != 0) {
            uint32_t t = (uint32_t)__builtin_ctz(eob);
            end = ci * 32u + t;
            uint32_t keep = (uint32_t)((2ULL << t) - 1ULL);
            l &= keep >> 1; o &= keep;
        }
        lit32[ci] = l; one32[ci] = o; ci++;
    }
    if (end == UINT32_MAX) {
        uint32_t l = 0, o = 0;
        for (uint32_t t = 0; ci * 32u + t < m_syms; t++) {
            uint16_t s = syms[ci * 32u + t];
            o |= (uint32_t)(s != 0) << t;
            if (s > 1u) { if (s == eob_sym) { end = ci * 32u + t; break; } l |= 1u << t; }
        }
        lit32[ci] = l; one32[ci] = o; ci++;
    }
    for (uint32_t i = ci; i < 2u * words; i++) { lit32[i] = 0; one32[i] = 0; }
    #undef PACK_BITS32

    for (int a = 0; a < 256; a++) { all[a] = 0; thin[a] = 0; thick[a] = 0; ws->q_len[a] = 0; }

    __m128i f0 = _mm_loadu_si128((const __m128i *)seq_to_unseq);
    __m128i f1 = _mm_loadu_si128((const __m128i *)(seq_to_unseq + 16));
    uint8_t __attribute__((aligned(32))) tail[224];
    memcpy(tail, seq_to_unseq + 32, 224);
    __m256i idx = _mm256_setr_epi8(
        0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31);

    uint32_t pos = 0, m = 0, mark = 0, n_strips = 0;
    const uint64_t *litp = ws->lit_bmap, *onep = ws->one_bmap;
    uint32_t last = end / 64u, wi = 0;
    uint64_t w = litp[0];

    #define VISIT_QI16(byte_c, total_w) do { \
        uint8_t _c = (uint8_t)(byte_c); \
        uint32_t _width = (uint32_t)(total_w); \
        uint32_t _end = pos + _width; \
        if (_width <= LIMIT) { \
            if (__builtin_expect(p >= pos && p < _end, 0)) mark = m + (p - pos); \
            q_push_scalar(ws, _c, m, _width); \
            thin[_c] += _width; m += _width; all[_c] += _width; \
        } else if (__builtin_expect(p >= pos && p < _end, 0)) { \
            if (pos < p) { \
                uint32_t _w1 = p - pos; \
                ws->strips[n_strips++] = (Strip){pos, all[_c], _w1, _c}; \
                thick[_c]++; all[_c] += _w1; \
            } \
            mark = m; q_push_scalar(ws, _c, m, 1u); \
            thin[_c]++; m++; all[_c]++; \
            if (p + 1u < _end) { \
                uint32_t _w2 = _end - (p + 1u); \
                ws->strips[n_strips++] = (Strip){p + 1u, all[_c], _w2, _c}; \
                thick[_c]++; all[_c] += _w2; \
            } \
        } else { \
            ws->strips[n_strips++] = (Strip){pos, all[_c], _width, _c}; \
            thick[_c]++; all[_c] += _width; \
        } \
        pos = _end; \
    } while (0)

    #define SEEK_NEXT(res_tz, has_res) do { \
        has_res = 0; \
        while (1) { \
            if (w != 0) { res_tz = (uint32_t)__builtin_ctzll(w); w &= w - 1ULL; has_res = 1; break; } \
            if (wi == last) break; \
            wi++; w = litp[wi]; \
        } \
    } while (0)

    #define COUNT_BETWEEN(q1, q2) ({ \
        uint32_t _from = (q1) + 1u; uint64_t _bits; \
        memcpy(&_bits, ((const uint8_t *)onep) + (_from >> 3), 8); \
        _bits >>= (_from & 7u); _bzhi_u64(_bits, (q2) - (q1)); \
    })

    uint32_t tz = 0; int has_first = 0;
    SEEK_NEXT(tz, has_first);
    uint32_t q0 = has_first ? (wi * 64u + tz) : end;
    uint64_t init_w = _bzhi_u64(onep[0], q0 + 1u) - 1ULL;
    if (init_w > 0) VISIT_QI16(seq_to_unseq[0], init_w);

    if (has_first) {
        uint64_t after = onep[wi] >> 1;
        const uint16_t *spw = syms + wi * 64u;
        while (1) {
            uint32_t s = spw[tz];
            if (__builtin_expect(s < 33u, 1)) {
                const MtfStep *st = &MTF_STEPS[s - 1u];
                __m128i a = _mm_load_si128((const __m128i *)st->a);
                __m128i b = _mm_load_si128((const __m128i *)st->b);
                __m128i c = _mm_load_si128((const __m128i *)st->c);
                __m128i d = _mm_load_si128((const __m128i *)st->d);
                __m128i g0 = _mm_or_si128(_mm_shuffle_epi8(f0, a), _mm_shuffle_epi8(f1, b));
                f1 = _mm_or_si128(_mm_shuffle_epi8(f1, c), _mm_shuffle_epi8(f0, d));
                f0 = g0;
            } else {
                uint32_t t_idx = (s - 1u < 255u ? s - 1u : 255u) - 32u;
                uint8_t v = mtf_second_avx2(tail, t_idx, f1, idx);
                f1 = _mm_alignr_epi8(f1, f0, 15);
                f0 = _mm_or_si128(_mm_slli_si128(f0, 1), _mm_cvtsi32_si128((int)v));
            }
            uint8_t c_byte = (uint8_t)_mm_cvtsi128_si32(f0);
            if (__builtin_expect(w != 0, 1)) {
                uint32_t t2 = (uint32_t)__builtin_ctzll(w);
                w &= w - 1ULL;
                VISIT_QI16(c_byte, _bzhi_u64(after, t2) >> tz);
                tz = t2;
            } else {
                uint32_t q = wi * 64u + tz, tz2 = 0; int has2 = 0;
                SEEK_NEXT(tz2, has2);
                if (has2) {
                    VISIT_QI16(c_byte, COUNT_BETWEEN(q, wi * 64u + tz2));
                    after = onep[wi] >> 1; spw = syms + wi * 64u; tz = tz2;
                } else {
                    VISIT_QI16(c_byte, COUNT_BETWEEN(q, end));
                    break;
                }
            }
        }
    }
    #undef VISIT_QI16
    #undef SEEK_NEXT
    #undef COUNT_BETWEEN

    *out_n = pos; *out_m = m; *out_mark = mark; *out_n_strips = n_strips;
}

static BenchStats run_arm(
    Workspace *ws,
    const uint8_t seq_to_unseq[256], const uint16_t *syms, uint32_t m_syms, uint16_t eob_sym,
    uint32_t p, int arm, uint8_t *out
) {
    BenchStats st = {0};
    uint32_t all[256], thin[256], thick[256];
    uint32_t n = 0, m = 0, mark = 0, n_strips = 0, bits = 0;
    int wide = 0;

    uint64_t t0 = now_ns();
    if (arm == 0 || arm == 1) {
        uint32_t nr = front_avx2_to_records(ws, seq_to_unseq, syms, m_syms, eob_sym, &n);
        uint64_t tf = now_ns();
        for (int a = 0; a < 256; a++) { all[a] = 0; thin[a] = 0; thick[a] = 0; ws->q_len[a] = 0; }
        uint32_t pos = 0, i = 0;
        const uint16_t *records = ws->front_recs;
        while (i < nr) {
            uint16_t rec = records[i];
            uint8_t c = (uint8_t)(rec >> 8);
            uint32_t width = (uint32_t)(rec & 255u);
            i++;
            while (i < nr && (uint8_t)(records[i] >> 8) == c) { width += (uint32_t)(records[i] & 255u); i++; }
            uint32_t end = pos + width;
            if (width <= LIMIT) {
                if (__builtin_expect(p >= pos && p < end, 0)) mark = m + (p - pos);
                q_push_scalar(ws, c, m, width);
                thin[c] += width; m += width; all[c] += width;
            } else if (__builtin_expect(p >= pos && p < end, 0)) {
                if (pos < p) { ws->strips[n_strips++] = (Strip){pos, all[c], p - pos, c}; thick[c]++; all[c] += p - pos; }
                mark = m; q_push_scalar(ws, c, m, 1u); thin[c]++; m++; all[c]++;
                if (p + 1u < end) { uint32_t w2 = end - (p + 1u); ws->strips[n_strips++] = (Strip){p + 1u, all[c], w2, c}; thick[c]++; all[c] += w2; }
            } else {
                ws->strips[n_strips++] = (Strip){pos, all[c], width, c}; thick[c]++; all[c] += width;
            }
            pos = end;
        }
        uint64_t tp1 = now_ns();
        finish_construct_std(ws, n, m, n_strips, all, thin, thick, &bits, &wide, &st.t_rv_us, &st.t_wire_us);
        uint64_t te0 = now_ns();
        if (arm == 0) emit_standard(ws->edges, ws->extra, wide, ws->spelling, ws->offsets, m, n, mark, bits, out);
        else emit_fast16(ws->edges, ws->extra, wide, ws->spelling, ws->offsets, m, n, mark, bits, out);
        uint64_t te1 = now_ns();
        st.t_front_us = (double)(tf - t0) / 1e3;
        st.t_pass1_us = (double)(tp1 - tf) / 1e3;
        st.t_emit_us = (double)(te1 - te0) / 1e3;
        st.t_total_us = (double)(te1 - t0) / 1e3;
        return st;
    }

    if (arm == 2) {
        uint32_t nr = front_avx2_to_records(ws, seq_to_unseq, syms, m_syms, eob_sym, &n);
        uint64_t tf = now_ns();
        pass1_records_vec16(ws, ws->front_recs, nr, p, all, thin, thick, &m, &mark, &n_strips);
        uint64_t tp1 = now_ns();
        finish_construct_vec16(ws, n, m, n_strips, all, thin, thick, &bits, &wide, &st.t_rv_us, &st.t_wire_us);
        uint64_t te0 = now_ns();
        emit_vec16(ws->edges, ws->extra, wide, ws->slot16, ws->slot_len, ws->long_words, ws->spelling, m, n, mark, bits, out);
        uint64_t te1 = now_ns();
        st.t_front_us = (double)(tf - t0) / 1e3;
        st.t_pass1_us = (double)(tp1 - tf) / 1e3;
        st.t_emit_us = (double)(te1 - te0) / 1e3;
        st.t_total_us = (double)(te1 - t0) / 1e3;
        return st;
    }

    if (arm == 3) {
        lrb_qi16_fused_front(ws, seq_to_unseq, syms, m_syms, eob_sym, p, all, thin, thick, &n, &m, &mark, &n_strips);
        uint64_t tf = now_ns();
        finish_construct_std(ws, n, m, n_strips, all, thin, thick, &bits, &wide, &st.t_rv_us, &st.t_wire_us);
        uint64_t te0 = now_ns();
        emit_standard(ws->edges, ws->extra, wide, ws->spelling, ws->offsets, m, n, mark, bits, out);
        uint64_t te1 = now_ns();
        st.t_front_us = (double)(tf - t0) / 1e3;
        st.t_pass1_us = 0.0;
        st.t_emit_us = (double)(te1 - te0) / 1e3;
        st.t_total_us = (double)(te1 - t0) / 1e3;
        return st;
    }

    /* arm == 4: C4:LRB-Vec16 */
    lrb_vec16_fused_front(ws, seq_to_unseq, syms, m_syms, eob_sym, p, all, thin, thick, &n, &m, &mark, &n_strips);
    uint64_t tf = now_ns();
    finish_construct_vec16(ws, n, m, n_strips, all, thin, thick, &bits, &wide, &st.t_rv_us, &st.t_wire_us);
    uint64_t te0 = now_ns();
    emit_vec16(ws->edges, ws->extra, wide, ws->slot16, ws->slot_len, ws->long_words, ws->spelling, m, n, mark, bits, out);
    uint64_t te1 = now_ns();
    st.t_front_us = (double)(tf - t0) / 1e3;
    st.t_pass1_us = 0.0;
    st.t_emit_us = (double)(te1 - te0) / 1e3;
    st.t_total_us = (double)(te1 - t0) / 1e3;
    return st;
}

static int cmp_double(const void *a, const void *b) {
    double da = *(const double *)a, db = *(const double *)b;
    return (da > db) - (da < db);
}

typedef struct { const char *file; uint32_t n, p; } BlockSpec;

int main(void) {
    init_mtf_steps();

    static const BlockSpec specs[6] = {
        {"/home/antigravity/admitted-bwt/block-0.L",   899931, 264045},
        {"/home/antigravity/admitted-bwt/block-2.L",   900000, 320807},
        {"/home/antigravity/admitted-bwt/block-85.L",  899935, 325088},
        {"/home/antigravity/admitted-bwt/block-143.L", 899869, 802868},
        {"/home/antigravity/admitted-bwt/block-207.L", 899998, 205180},
        {"/home/antigravity/admitted-bwt/block-210.L", 899959, 791429},
    };

    const char *names[5] = {
        "C0:2Stage-qi16",
        "C1:2Stage-fast16",
        "C2:2Stage-Vec16",
        "C3:LRB-qi16",
        "C4:LRB-Vec16"
    };

    Workspace *ws = ws_create();
    uint8_t *L = (uint8_t *)malloc(MAX_N);
    uint16_t *syms = (uint16_t *)malloc(MAX_SYMS * sizeof(uint16_t));
    uint8_t seq_to_unseq[256];
    uint8_t *ref_out = (uint8_t *)malloc(MAX_N);
    uint8_t *test_out = (uint8_t *)malloc(MAX_N + OUT_PAD + CANARY_LEN);

    #define WARMUP 3
    #define ROUNDS 21
    #define NARMS 5

    printf("=== Carbon Kitchen Sink — Cycle 3 Gate G5 Pre-Registered C Falsifier (%d paired interleaved rounds) ===\n\n", ROUNDS);

    for (int bi = 0; bi < 6; bi++) {
        FILE *fp = fopen(specs[bi].file, "rb");
        if (!fp) { perror(specs[bi].file); return 1; }
        size_t rd = fread(L, 1, specs[bi].n, fp);
        fclose(fp);
        assert(rd == specs[bi].n);

        uint32_t n = specs[bi].n, p = specs[bi].p;
        uint16_t eob_sym = 0;
        uint32_t m_syms = encode_bzip2_symbols(L, n, seq_to_unseq, syms, &eob_sym);
        dense_lf_invert(L, n, p, ref_out);

        for (int arm = 0; arm < NARMS; arm++) {
            memset(test_out, 0, n + OUT_PAD);
            memset(test_out + n + OUT_PAD, CANARY_BYTE, CANARY_LEN);
            (void)run_arm(ws, seq_to_unseq, syms, m_syms, eob_sym, p, arm, test_out);
            if (memcmp(test_out, ref_out, n) != 0) {
                fprintf(stderr, "FAIL byte-exact on %s arm %s!\n", specs[bi].file, names[arm]);
                return 2;
            }
            for (int c = 0; c < CANARY_LEN; c++) {
                if (test_out[n + OUT_PAD + c] != CANARY_BYTE) {
                    fprintf(stderr, "FAIL canary on %s arm %s!\n", specs[bi].file, names[arm]);
                    return 3;
                }
            }
        }

        double tot[NARMS][ROUNDS], emit_arr[NARMS][ROUNDS];
        double fr_s[NARMS] = {0}, p1_s[NARMS] = {0}, rv_s[NARMS] = {0}, wr_s[NARMS] = {0};

        for (int rnd = -WARMUP; rnd < ROUNDS; rnd++) {
            for (int slot = 0; slot < NARMS; slot++) {
                int arm = (slot + (rnd + WARMUP)) % NARMS;
                BenchStats st = run_arm(ws, seq_to_unseq, syms, m_syms, eob_sym, p, arm, test_out);
                if (rnd >= 0) {
                    tot[arm][rnd] = st.t_total_us;
                    emit_arr[arm][rnd] = st.t_emit_us;
                    fr_s[arm] += st.t_front_us;
                    p1_s[arm] += st.t_pass1_us;
                    rv_s[arm] += st.t_rv_us;
                    wr_s[arm] += st.t_wire_us;
                }
            }
        }

        for (int arm = 0; arm < NARMS; arm++) {
            qsort(tot[arm], ROUNDS, sizeof(double), cmp_double);
            qsort(emit_arr[arm], ROUNDS, sizeof(double), cmp_double);
        }
        double base_c0 = tot[0][ROUNDS / 2];
        double base_c2 = tot[2][ROUNDS / 2];

        printf("--- %s (n=%u, syms=%u) [ALL 5 ARMS BYTE-EXACT + CANARY OK] ---\n",
               specs[bi].file + 31, n, m_syms);
        printf("  %-18s | %8s (%8s..%8s) | %7s %7s | %7s %7s %7s %7s %7s\n",
               "Arm", "Med(us)", "p25", "p75", "/C0", "/C2",
               "front", "pass1", "rv", "wire", "emitMed");
        for (int arm = 0; arm < NARMS; arm++) {
            double med = tot[arm][ROUNDS / 2];
            double p25 = tot[arm][ROUNDS / 4];
            double p75 = tot[arm][(3 * ROUNDS) / 4];
            double em_med = emit_arr[arm][ROUNDS / 2];
            printf("  %-18s | %8.1f (%8.1f..%8.1f) | %6.3fx %6.3fx | %7.1f %7.1f %7.1f %7.1f %7.1f\n",
                   names[arm], med, p25, p75, med / base_c0, med / base_c2,
                   fr_s[arm] / ROUNDS, p1_s[arm] / ROUNDS, rv_s[arm] / ROUNDS,
                   wr_s[arm] / ROUNDS, em_med);
        }
        printf("\n");
    }

    free(test_out); free(ref_out); free(syms); free(L);
    ws_destroy(ws);
    return 0;
}
