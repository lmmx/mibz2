/*
 * Carbon Kitchen Sink — Cycle 2 Gate G5 Pre-Registered C Falsifier (g5_cycle2_bench.c)
 *
 * Tests two independent mathematical constructions and their composition across all 6 blocks
 * (21 paired, interleaved rounds after 3 warm-up rounds; byte-exact + 64B canary verified):
 *
 * Group A (Entry I0 = capped u16 run records -> n-byte text):
 *   A0. Open-qi16        : Faithful C port of shipped src/open.rs + endpoint.rs
 *   A1. Open-qi16-fast16 : Open-qi16 with branchless 16B short-word emit
 *   A2. OIBS-H4-fast16   : Hysteresis-4 Read-Only Hot-Loop Étale Sheet Shortcutting on Open
 *   A3. OIBS-H6-fast16   : Hysteresis-6 Read-Only Hot-Loop Étale Sheet Shortcutting on Open
 *
 * Group B (Pre-Column Entry = bzip2 Huffman Symbols -> n-byte text, Obligation 5):
 *   B0. TwoStage-AVX2    : Faithful AVX2 `runs::front` (`src/runs.rs` bitmaps + `_bzhi_u64`
 *                          + 2-reg `STEPS[32]` MTF shuffle -> CAP=127 u16 records) + Open-qi16-fast16
 *   B1. LRB-Open-AVX2    : Direct Left-Regular-Band AVX2 MTF+RLE2 -> Open fusion (Cycle 1 Theorem 3:
 *                          zero `records` buffer, zero CAP=127 split, zero `each_run` re-merge)
 *   B2. LRB-OIBS-AVX2    : Composition of LRB-Open-AVX2 + OIBS-H4-fast16
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
#define MAX_NODES 100000
#define MAX_RULES 100000
#define MAX_SHORTCUTS 32768
#define OUT_PAD 128
#define CANARY_LEN 64
#define CANARY_BYTE 0xA5
#define LIMIT 16u

#define SHORTCUT_BIT 0x80000000u
#define VISITED_LEADER 0x7FFFFFFEu
#define EDGE_SENTINEL 0x7FFFFFFDu

static inline uint64_t now_ns(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (uint64_t)ts.tv_sec * 1000000000ULL + (uint64_t)ts.tv_nsec;
}

/* MTF AVX2 first-level shuffle table matching src/runs.rs lines 277-331 */
typedef struct __attribute__((aligned(64))) {
    uint8_t a[16];
    uint8_t b[16];
    uint8_t c[16];
    uint8_t d[16];
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

/* Second-level AVX2 move matching src/mtf.rs:second */
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

/* Reconstruct exact canonical libbzip2 MTF+RLE2 Symbols from raw .L (outside timed region) */
static uint32_t encode_bzip2_symbols(
    const uint8_t *L,
    uint32_t n,
    uint8_t seq_to_unseq[256],
    uint16_t *syms,
    uint16_t *out_eob
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
        if (idx == 0) {
            assert(i == 0);
            rem = w;
        } else {
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

static uint32_t build_capped_records(const uint8_t *L, uint32_t n, uint16_t *recs) {
    uint32_t nr = 0, i = 0;
    while (i < n) {
        uint8_t c = L[i];
        uint32_t j = i + 1;
        while (j < n && L[j] == c) j++;
        uint32_t rem = j - i;
        while (rem > 0) {
            uint32_t w = rem > 127u ? 127u : rem;
            recs[nr++] = (uint16_t)(((uint16_t)c << 8) | w);
            rem -= w;
        }
        i = j;
    }
    return nr;
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
typedef struct { uint32_t next_j, out_pos, byte_len; } Shortcut;

typedef struct {
    double t_front_us;
    double t_pass1_us;
    double t_rv_us;
    double t_wire_us;
    double t_emit_us;
    double t_total_us;
    uint32_t emit_hops;
    uint32_t shortcuts_taken;
} BenchStats;

typedef struct {
    uint32_t *q_vecs[256];
    uint32_t q_len[256];
    uint32_t q_cap[256];

    Strip *strips;
    uint32_t *image_order;
    Band *bands;
    uint8_t *spelling;
    uint32_t *offsets;
    uint32_t *edges_raw; /* allocated MAX_N + 4; edges = edges_raw + 1 */
    uint32_t *extra;

    RvNode *nodes;
    RvRule *rules;
    uint32_t *order0, *order1, *term0, *term1, *bottom, *stack;

    /* AVX2 front scratch (matching Runs.lit, Runs.one, Runs.runs) */
    uint64_t *lit_bmap;
    uint64_t *one_bmap;
    uint16_t *front_recs;

    /* OIBS shortcut pool */
    Shortcut *shortcuts;
} Workspace;

static Workspace *ws_create(void) {
    Workspace *w = (Workspace *)calloc(1, sizeof(Workspace));
    for (int a = 0; a < 256; a++) {
        w->q_cap[a] = 4096;
        w->q_vecs[a] = (uint32_t *)malloc(w->q_cap[a] * sizeof(uint32_t));
    }
    w->strips = (Strip *)malloc(MAX_RUNS * sizeof(Strip));
    w->image_order = (uint32_t *)malloc(MAX_RUNS * sizeof(uint32_t));
    w->bands = (Band *)malloc(MAX_RUNS * sizeof(Band));
    w->spelling = (uint8_t *)malloc(MAX_N + 1024);
    w->offsets = (uint32_t *)malloc(MAX_RUNS * sizeof(uint32_t));
    w->edges_raw = (uint32_t *)malloc((MAX_N + 64) * sizeof(uint32_t));
    w->extra = (uint32_t *)malloc(MAX_N * sizeof(uint32_t));

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
    w->shortcuts = (Shortcut *)malloc(MAX_SHORTCUTS * sizeof(Shortcut));
    return w;
}

static void ws_destroy(Workspace *w) {
    for (int a = 0; a < 256; a++) free(w->q_vecs[a]);
    free(w->strips); free(w->image_order); free(w->bands); free(w->spelling);
    free(w->offsets); free(w->edges_raw); free(w->extra);
    free(w->nodes); free(w->rules); free(w->order0); free(w->order1);
    free(w->term0); free(w->term1); free(w->bottom); free(w->stack);
    free(w->lit_bmap); free(w->one_bmap); free(w->front_recs); free(w->shortcuts);
    free(w);
}

static inline void q_push_range(Workspace *ws, uint8_t a, uint32_t start, uint32_t w) {
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

static uint32_t rv_eliminate(RvNode *nodes, RvRule *rules, uint32_t *n_rules, uint32_t rem) {
    uint32_t steps = 0;
    while (rem > 0) {
        uint32_t x = nodes[0].prev[0], y = nodes[0].prev[1];
        uint32_t wx = nodes[x].width, wy = nodes[y].width;
        uint32_t w = wx < wy ? wx : wy;
        rem -= w; steps++;
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
    return steps;
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

/* Shared backend of Open after queues & strips are populated */
static void open_finish_construct(
    Workspace *ws,
    uint32_t n,
    uint32_t m,
    uint32_t n_strips,
    const uint32_t all[256],
    const uint32_t thin[256],
    const uint32_t thick[256],
    uint32_t *out_bits,
    int *out_wide,
    double *t_rv_us,
    double *t_wire_us
) {
    uint32_t first[256], write_pos[256];
    uint32_t tot_a = 0, tot_t = 0;
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
    (void)rv_eliminate(nodes, ws->rules, &n_rules, n);

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

    /* Ensure m < (1 << bits) AND bit 31 stays clear for OIBS SHORTCUT_BIT */
    uint32_t bits = m <= 1 ? 1u : (32u - (uint32_t)__builtin_clz(m)); /* m < (1 << bits) guaranteed! */
    int wide = (255u + n_words) > (0x7FFFFFFFu >> bits);

    uint32_t *edges = ws->edges_raw + 32;
    for (int s = -32; s < 0; s++) edges[s] = EDGE_SENTINEL;
    for (uint32_t s = 0; s < 32u; s++) edges[m + s] = EDGE_SENTINEL;

    k = 0;
    for (int a = 0; a < 256; a++) {
        uint32_t b_start = cursors[a], b_end = cursors[a + 1];
        uint32_t from = 0;
        const uint32_t *qsrc = ws->q_vecs[a];
        for (uint32_t bi = b_start; bi < b_end; bi++) {
            Band b = ws->bands[bi];
            uint32_t span = b.end - k;
            uint32_t dst = k + b.delta;
            uint32_t pack = wide ? 0u : (b.code << bits);
            if (wide) {
                for (uint32_t u = 0; u < span; u++) ws->extra[dst + u] = b.code;
            }
            uint32_t *edst = edges + dst;
            const uint32_t *src = qsrc + from;
            for (uint32_t u = 0; u < span; u++) edst[u] = pack | src[u];
            from += span; k = b.end;
        }
    }
    uint64_t t3 = now_ns();

    *out_bits = bits;
    *out_wide = wide;
    *t_rv_us = (double)(t2 - t1) / 1e3;
    *t_wire_us = (double)(t3 - t2) / 1e3;
}

static void emit_standard(
    const uint32_t *edges, const uint32_t *extra, int wide,
    const uint8_t *spelling, const uint32_t *offsets,
    uint32_t m, uint32_t n, uint32_t mark, uint32_t bits, uint8_t *out
) {
    uint32_t mask = (1u << bits) - 1u;
    uint32_t j = mark, out_len = 0;
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
    uint32_t mask = (1u << bits) - 1u;
    uint32_t j = mark, out_len = 0;
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

/* Hysteresis-Gated Read-Only Hot-Loop Étale Sheet Shortcutting (OIBS-H) */
static void emit_oibs_fast16(
    uint32_t *edges, const uint32_t *extra, int wide,
    const uint8_t *spelling, const uint32_t *offsets,
    Shortcut *sc_pool,
    uint32_t m, uint32_t n, uint32_t mark, uint32_t bits,
    uint32_t h_trig, uint32_t k_min,
    uint8_t *out,
    uint32_t *out_hops, uint32_t *out_sc_taken
) {
    if (__builtin_expect(wide, 0)) {
        emit_fast16(edges, extra, wide, spelling, offsets, m, n, mark, bits, out);
        *out_hops = m; *out_sc_taken = 0;
        return;
    }
    uint32_t mask = (1u << bits) - 1u;
    intptr_t j = (intptr_t)mark;
    intptr_t mark_i = (intptr_t)mark;
    uint32_t out_len = 0;
    uint32_t streak = 0, n_sc = 0, hops = 0, sc_taken = 0;

    for (uint32_t step = 0; step < m; step++) {
        uint32_t e = edges[j];
        hops++;
        if (__builtin_expect(!((e & SHORTCUT_BIT) || streak >= h_trig), 1)) {
            /* HOT PATH: zero writes to edges, no bounds checks (sentinel padded!) */
            uint32_t coh = (edges[j + 1] == e + 1u) | (edges[j - 1] == e - 1u);
            streak = coh ? (streak + 1u) : 0u;

            uint32_t code = e >> bits;
            j = (intptr_t)(e & mask);
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
            if (j == mark_i || out_len >= n) break;
            continue;
        }

        /* COLD PATH (< 4.4% of iterations): either a shortcut hit or a sheet leader! */
        if (e & SHORTCUT_BIT) {
            Shortcut sc = sc_pool[e & 0x7FFFFFFFu];
            j = (intptr_t)sc.next_j;
            streak = 0;
            sc_taken++;
            uint32_t take = (out_len + sc.byte_len <= n) ? sc.byte_len : (n - out_len);
            /* Copy from earlier completed output (non-overlapping by No-Wraparound Lemma!) */
            memmove(out + out_len, out + sc.out_pos, take);
            out_len += take;
            if (j == mark_i || out_len >= n) break;
            continue;
        }

        streak = 0;
        intptr_t L_span = 0, R_span = 0;
        while (L_span < 15 && (j - (L_span + 1)) != mark_i &&
               edges[j - (L_span + 1)] == e - (uint32_t)(L_span + 1)) {
            L_span++;
        }
        while (R_span < 15 && (j + (R_span + 1)) != mark_i &&
               edges[j + (R_span + 1)] == e + (uint32_t)(R_span + 1)) {
            R_span++;
        }

        if ((L_span == 0 && R_span == 0) || n_sc + 32 >= MAX_SHORTCUTS) {
            uint32_t code = e >> bits;
            j = (intptr_t)(e & mask);
            if (code < 256u) out[out_len++] = (uint8_t)code;
            else {
                uint32_t w = code - 256u;
                uint32_t s0 = offsets[w], len = offsets[w + 1] - s0;
                uint32_t take = (out_len + len <= n) ? len : (n - out_len);
                memcpy(out + out_len, spelling + s0, take);
                out_len += take;
            }
            if (j == mark_i || out_len >= n) break;
            continue;
        }

        intptr_t start_j = j, cur_j = j;
        uint32_t start_out = out_len, sheet_steps = 0;
        intptr_t cur_L = L_span, cur_R = R_span;

        while (1) {
            uint32_t ce = edges[cur_j];
            if ((ce & SHORTCUT_BIT) || ce == VISITED_LEADER) break;
            if (sheet_steps > 0) {
                hops++;
                intptr_t nl = 0, nr = 0;
                while (nl < cur_L && (cur_j - (nl + 1)) != mark_i &&
                       edges[cur_j - (nl + 1)] == ce - (uint32_t)(nl + 1)) nl++;
                while (nr < cur_R && (cur_j + (nr + 1)) != mark_i &&
                       edges[cur_j + (nr + 1)] == ce + (uint32_t)(nr + 1)) nr++;
                if (nl == 0 && nr == 0) break;
                if (sheet_steps >= k_min && (nl < cur_L || nr < cur_R) && n_sc + 32 < MAX_SHORTCUTS) {
                    uint32_t blen = (out_len <= n ? out_len : n) - start_out;
                    for (intptr_t d = -cur_L; d < -nl; d++) {
                        intptr_t sj = start_j + d;
                        if (sj != mark_i && edges[sj] != VISITED_LEADER && !(edges[sj] & SHORTCUT_BIT)) {
                            uint32_t sid = n_sc++;
                            sc_pool[sid] = (Shortcut){(uint32_t)(cur_j + d), start_out, blen};
                            edges[sj] = SHORTCUT_BIT | sid;
                        }
                    }
                    for (intptr_t d = nr + 1; d <= cur_R; d++) {
                        intptr_t sj = start_j + d;
                        if (sj != mark_i && edges[sj] != VISITED_LEADER && !(edges[sj] & SHORTCUT_BIT)) {
                            uint32_t sid = n_sc++;
                            sc_pool[sid] = (Shortcut){(uint32_t)(cur_j + d), start_out, blen};
                            edges[sj] = SHORTCUT_BIT | sid;
                        }
                    }
                }
                cur_L = nl; cur_R = nr;
            }

            edges[cur_j] = VISITED_LEADER;
            uint32_t ccode = ce >> bits;
            cur_j = (intptr_t)(ce & mask);
            if (ccode < 256u) out[out_len++] = (uint8_t)ccode;
            else {
                uint32_t w = ccode - 256u;
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
            sheet_steps++;
            if (cur_j == mark_i || out_len >= n) break;
        }

        if (sheet_steps >= k_min && (cur_L > 0 || cur_R > 0) && n_sc + 32 < MAX_SHORTCUTS) {
            uint32_t blen = (out_len <= n ? out_len : n) - start_out;
            for (intptr_t d = -cur_L; d <= cur_R; d++) {
                if (d == 0) continue;
                intptr_t sj = start_j + d;
                if (sj != mark_i && edges[sj] != VISITED_LEADER && !(edges[sj] & SHORTCUT_BIT)) {
                    uint32_t sid = n_sc++;
                    sc_pool[sid] = (Shortcut){(uint32_t)(cur_j + d), start_out, blen};
                    edges[sj] = SHORTCUT_BIT | sid;
                }
            }
        }
        j = cur_j;
        if (j == mark_i || out_len >= n) break;
    }

    if (out_len > n) out_len = n;
    uint32_t plen = out_len;
    while (out_len < n) {
        uint32_t take = (n - out_len) < plen ? (n - out_len) : plen;
        memcpy(out + out_len, out, take); out_len += take;
    }
    *out_hops = hops;
    *out_sc_taken = sc_taken;
}

/* Faithful AVX2 `runs::front` (`src/runs.rs:front_avx2`) -> u16 records */
static uint32_t front_avx2_to_records(
    Workspace *ws,
    const uint8_t seq_to_unseq[256],
    const uint16_t *syms,
    uint32_t m_syms,
    uint16_t eob_sym,
    uint32_t *out_n
) {
    uint32_t words = m_syms / 64u + 3u;
    uint32_t *lit32 = (uint32_t *)ws->lit_bmap;
    uint32_t *one32 = (uint32_t *)ws->one_bmap;

    __m256i v1 = _mm256_set1_epi16(1);
    __m256i v0 = _mm256_setzero_si256();
    __m256i ve = _mm256_set1_epi16((short)eob_sym);

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
            l &= keep >> 1;
            o &= keep;
        }
        lit32[ci] = l;
        one32[ci] = o;
        ci++;
    }
    if (end == UINT32_MAX) {
        uint32_t l = 0, o = 0;
        for (uint32_t t = 0; ci * 32u + t < m_syms; t++) {
            uint16_t s = syms[ci * 32u + t];
            o |= (uint32_t)(s != 0) << t;
            if (s > 1u) {
                if (s == eob_sym) { end = ci * 32u + t; break; }
                l |= 1u << t;
            }
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
    const uint64_t *litp = ws->lit_bmap;
    const uint64_t *onep = ws->one_bmap;
    uint32_t last = end / 64u, wi = 0;
    uint64_t w = litp[0];

    #define EMIT_CAP127(byte_c, total_w) do { \
        uint32_t _rem = (uint32_t)(total_w); \
        n_rows += _rem; \
        uint16_t _hi = (uint16_t)((uint16_t)(byte_c) << 8); \
        if (__builtin_expect(_rem <= 127u, 1)) { \
            *out++ = _hi | (uint16_t)_rem; \
        } else { \
            while (_rem > 0) { \
                uint32_t _l = _rem > 127u ? 127u : _rem; \
                *out++ = _hi | (uint16_t)_l; \
                _rem -= _l; \
            } \
        } \
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
        uint32_t _from = (q1) + 1u; \
        uint64_t _bits; \
        memcpy(&_bits, ((const uint8_t *)onep) + (_from >> 3), 8); \
        _bits >>= (_from & 7u); \
        _bzhi_u64(_bits, (q2) - (q1)); \
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
                uint64_t tot = _bzhi_u64(after, t2) >> tz;
                EMIT_CAP127(c_byte, tot);
                tz = t2;
            } else {
                uint32_t q = wi * 64u + tz;
                uint32_t tz2 = 0; int has2 = 0;
                SEEK_NEXT(tz2, has2);
                if (has2) {
                    uint32_t q2 = wi * 64u + tz2;
                    EMIT_CAP127(c_byte, COUNT_BETWEEN(q, q2));
                    after = onep[wi] >> 1;
                    spw = syms + wi * 64u;
                    tz = tz2;
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

/* LRB-Open-AVX2: Direct Pre-Column AVX2 MTF+RLE2 -> Open (zero records buffer, zero CAP=127, zero each_run!) */
static void lrb_open_front_fused_avx2(
    Workspace *ws,
    const uint8_t seq_to_unseq[256],
    const uint16_t *syms,
    uint32_t m_syms,
    uint16_t eob_sym,
    uint32_t p,
    uint32_t all[256],
    uint32_t thin[256],
    uint32_t thick[256],
    uint32_t *out_n,
    uint32_t *out_m,
    uint32_t *out_mark,
    uint32_t *out_n_strips
) {
    uint32_t words = m_syms / 64u + 3u;
    uint32_t *lit32 = (uint32_t *)ws->lit_bmap;
    uint32_t *one32 = (uint32_t *)ws->one_bmap;

    __m256i v1 = _mm256_set1_epi16(1);
    __m256i v0 = _mm256_setzero_si256();
    __m256i ve = _mm256_set1_epi16((short)eob_sym);

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
            l &= keep >> 1;
            o &= keep;
        }
        lit32[ci] = l; one32[ci] = o; ci++;
    }
    if (end == UINT32_MAX) {
        uint32_t l = 0, o = 0;
        for (uint32_t t = 0; ci * 32u + t < m_syms; t++) {
            uint16_t s = syms[ci * 32u + t];
            o |= (uint32_t)(s != 0) << t;
            if (s > 1u) {
                if (s == eob_sym) { end = ci * 32u + t; break; }
                l |= 1u << t;
            }
        }
        lit32[ci] = l; one32[ci] = o; ci++;
    }
    for (uint32_t i = ci; i < 2u * words; i++) { lit32[i] = 0; one32[i] = 0; }
    #undef PACK_BITS32

    for (int a = 0; a < 256; a++) {
        all[a] = 0; thin[a] = 0; thick[a] = 0;
        ws->q_len[a] = 0;
    }

    __m128i f0 = _mm_loadu_si128((const __m128i *)seq_to_unseq);
    __m128i f1 = _mm_loadu_si128((const __m128i *)(seq_to_unseq + 16));
    uint8_t __attribute__((aligned(32))) tail[224];
    memcpy(tail, seq_to_unseq + 32, 224);
    __m256i idx = _mm256_setr_epi8(
        0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31);

    uint32_t pos = 0, m = 0, mark = 0, n_strips = 0;
    const uint64_t *litp = ws->lit_bmap;
    const uint64_t *onep = ws->one_bmap;
    uint32_t last = end / 64u, wi = 0;
    uint64_t w = litp[0];

    /* By Cycle 1 Theorem 3, EVERY call to VISIT_MAXIMAL_RUN receives a COMPLETE maximal run! */
    #define VISIT_MAXIMAL_RUN(byte_c, total_w) do { \
        uint8_t _c = (uint8_t)(byte_c); \
        uint32_t _width = (uint32_t)(total_w); \
        uint32_t _end = pos + _width; \
        if (_width <= LIMIT) { \
            if (__builtin_expect(p >= pos && p < _end, 0)) mark = m + (p - pos); \
            q_push_range(ws, _c, m, _width); \
            thin[_c] += _width; \
            m += _width; \
            all[_c] += _width; \
        } else if (__builtin_expect(p >= pos && p < _end, 0)) { \
            if (pos < p) { \
                uint32_t _w1 = p - pos; \
                ws->strips[n_strips++] = (Strip){pos, all[_c], _w1, _c}; \
                thick[_c]++; all[_c] += _w1; \
            } \
            mark = m; \
            q_push_range(ws, _c, m, 1u); \
            thin[_c] += 1u; m += 1u; all[_c] += 1u; \
            if (p + 1u < _end) { \
                uint32_t _w2 = _end - (p + 1u); \
                ws->strips[n_strips++] = (Strip){p + 1u, all[_c], _w2, _c}; \
                thick[_c]++; all[_c] += _w2; \
            } \
        } else { \
            ws->strips[n_strips++] = (Strip){pos, all[_c], _width, _c}; \
            thick[_c]++; \
            all[_c] += _width; \
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
        uint32_t _from = (q1) + 1u; \
        uint64_t _bits; \
        memcpy(&_bits, ((const uint8_t *)onep) + (_from >> 3), 8); \
        _bits >>= (_from & 7u); \
        _bzhi_u64(_bits, (q2) - (q1)); \
    })

    uint32_t tz = 0; int has_first = 0;
    SEEK_NEXT(tz, has_first);
    uint32_t q0 = has_first ? (wi * 64u + tz) : end;
    uint64_t init_w = _bzhi_u64(onep[0], q0 + 1u) - 1ULL;
    if (init_w > 0) VISIT_MAXIMAL_RUN(seq_to_unseq[0], init_w);

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
                uint64_t tot = _bzhi_u64(after, t2) >> tz;
                VISIT_MAXIMAL_RUN(c_byte, tot);
                tz = t2;
            } else {
                uint32_t q = wi * 64u + tz;
                uint32_t tz2 = 0; int has2 = 0;
                SEEK_NEXT(tz2, has2);
                if (has2) {
                    uint32_t q2 = wi * 64u + tz2;
                    VISIT_MAXIMAL_RUN(c_byte, COUNT_BETWEEN(q, q2));
                    after = onep[wi] >> 1;
                    spw = syms + wi * 64u;
                    tz = tz2;
                } else {
                    VISIT_MAXIMAL_RUN(c_byte, COUNT_BETWEEN(q, end));
                    break;
                }
            }
        }
    }
    #undef VISIT_MAXIMAL_RUN
    #undef SEEK_NEXT
    #undef COUNT_BETWEEN

    *out_n = pos;
    *out_m = m;
    *out_mark = mark;
    *out_n_strips = n_strips;
}

static BenchStats run_records_arm(
    Workspace *ws,
    const uint16_t *records, uint32_t n_recs,
    uint32_t n, uint32_t p,
    int mode, /* 0: std, 1: fast16, 2: OIBS-H4-fast16, 3: OIBS-H6-fast16 */
    uint8_t *out
) {
    BenchStats st = {0};
    uint64_t t0 = now_ns();

    uint32_t all[256] = {0}, thin[256] = {0}, thick[256] = {0};
    for (int a = 0; a < 256; a++) ws->q_len[a] = 0;
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
            q_push_range(ws, c, m, width);
            thin[c] += width; m += width; all[c] += width;
        } else if (__builtin_expect(p >= pos && p < end, 0)) {
            if (pos < p) {
                uint32_t w1 = p - pos;
                ws->strips[n_strips++] = (Strip){pos, all[c], w1, c};
                thick[c]++; all[c] += w1;
            }
            mark = m; q_push_range(ws, c, m, 1); thin[c]++; m++; all[c]++;
            if (p + 1 < end) {
                uint32_t w2 = end - (p + 1);
                ws->strips[n_strips++] = (Strip){p + 1, all[c], w2, c};
                thick[c]++; all[c] += w2;
            }
        } else {
            ws->strips[n_strips++] = (Strip){pos, all[c], width, c};
            thick[c]++; all[c] += width;
        }
        pos = end;
    }
    uint64_t t1 = now_ns();

    uint32_t bits = 0; int wide = 0;
    open_finish_construct(ws, n, m, n_strips, all, thin, thick, &bits, &wide, &st.t_rv_us, &st.t_wire_us);

    uint64_t te0 = now_ns();
    uint32_t *edges = ws->edges_raw + 32;
    if (mode == 0) {
        emit_standard(edges, ws->extra, wide, ws->spelling, ws->offsets, m, n, mark, bits, out);
        st.emit_hops = m;
    } else if (mode == 1) {
        emit_fast16(edges, ws->extra, wide, ws->spelling, ws->offsets, m, n, mark, bits, out);
        st.emit_hops = m;
    } else if (mode == 2) {
        emit_oibs_fast16(edges, ws->extra, wide, ws->spelling, ws->offsets, ws->shortcuts,
                         m, n, mark, bits, 4u, 4u, out, &st.emit_hops, &st.shortcuts_taken);
    } else {
        emit_oibs_fast16(edges, ws->extra, wide, ws->spelling, ws->offsets, ws->shortcuts,
                         m, n, mark, bits, 6u, 6u, out, &st.emit_hops, &st.shortcuts_taken);
    }
    uint64_t te1 = now_ns();

    st.t_front_us = 0.0;
    st.t_pass1_us = (double)(t1 - t0) / 1e3;
    st.t_emit_us = (double)(te1 - te0) / 1e3;
    st.t_total_us = (double)(te1 - t0) / 1e3;
    return st;
}

static BenchStats run_syms_arm(
    Workspace *ws,
    const uint8_t seq_to_unseq[256],
    const uint16_t *syms, uint32_t m_syms, uint16_t eob_sym,
    uint32_t p,
    int mode, /* 0: TwoStage-AVX2 + Open-fast16, 1: LRB-Open-AVX2-fast16, 2: LRB-OIBS-H4-AVX2 */
    uint8_t *out
) {
    BenchStats st = {0};
    uint64_t t0 = now_ns();

    if (mode == 0) {
        uint32_t n = 0;
        uint32_t nr = front_avx2_to_records(ws, seq_to_unseq, syms, m_syms, eob_sym, &n);
        uint64_t tf = now_ns();
        BenchStats sub = run_records_arm(ws, ws->front_recs, nr, n, p, 1, out);
        uint64_t t_end = now_ns();
        sub.t_front_us = (double)(tf - t0) / 1e3;
        sub.t_total_us = (double)(t_end - t0) / 1e3;
        return sub;
    }

    uint32_t all[256], thin[256], thick[256];
    uint32_t n = 0, m = 0, mark = 0, n_strips = 0;
    lrb_open_front_fused_avx2(ws, seq_to_unseq, syms, m_syms, eob_sym, p,
                              all, thin, thick, &n, &m, &mark, &n_strips);
    uint64_t t1 = now_ns();

    uint32_t bits = 0; int wide = 0;
    open_finish_construct(ws, n, m, n_strips, all, thin, thick, &bits, &wide, &st.t_rv_us, &st.t_wire_us);

    uint64_t te0 = now_ns();
    uint32_t *edges = ws->edges_raw + 32;
    if (mode == 1) {
        emit_fast16(edges, ws->extra, wide, ws->spelling, ws->offsets, m, n, mark, bits, out);
        st.emit_hops = m;
    } else {
        emit_oibs_fast16(edges, ws->extra, wide, ws->spelling, ws->offsets, ws->shortcuts,
                         m, n, mark, bits, 4u, 4u, out, &st.emit_hops, &st.shortcuts_taken);
    }
    uint64_t te1 = now_ns();

    st.t_front_us = (double)(t1 - t0) / 1e3;
    st.t_pass1_us = 0.0; /* eliminated! */
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

    const char *names[7] = {
        "A0:Open-qi16",
        "A1:Open-fast16",
        "A2:OIBS-H4-fast16",
        "A3:OIBS-H6-fast16",
        "B0:2Stage-AVX2",
        "B1:LRB-Open-AVX2",
        "B2:LRB-OIBS-AVX2"
    };

    Workspace *ws = ws_create();
    uint8_t *L = (uint8_t *)malloc(MAX_N);
    uint16_t *recs = (uint16_t *)malloc(MAX_RECS * sizeof(uint16_t));
    uint16_t *syms = (uint16_t *)malloc(MAX_SYMS * sizeof(uint16_t));
    uint8_t seq_to_unseq[256];
    uint8_t *ref_out = (uint8_t *)malloc(MAX_N);
    uint8_t *test_out = (uint8_t *)malloc(MAX_N + OUT_PAD + CANARY_LEN);

    #define WARMUP 3
    #define ROUNDS 21
    #define NARMS 7

    printf("=== Carbon Kitchen Sink — Cycle 2 Gate G5 Pre-Registered C Falsifier (%d paired interleaved rounds) ===\n\n", ROUNDS);

    for (int bi = 0; bi < 6; bi++) {
        FILE *fp = fopen(specs[bi].file, "rb");
        if (!fp) { perror(specs[bi].file); return 1; }
        size_t rd = fread(L, 1, specs[bi].n, fp);
        fclose(fp);
        assert(rd == specs[bi].n);

        uint32_t n = specs[bi].n, p = specs[bi].p;
        uint32_t n_recs = build_capped_records(L, n, recs);
        uint16_t eob_sym = 0;
        uint32_t m_syms = encode_bzip2_symbols(L, n, seq_to_unseq, syms, &eob_sym);
        dense_lf_invert(L, n, p, ref_out);

        BenchStats last_st[NARMS];
        for (int arm = 0; arm < NARMS; arm++) {
            memset(test_out, 0, n + OUT_PAD);
            memset(test_out + n + OUT_PAD, CANARY_BYTE, CANARY_LEN);
            memset(ws->spelling, 0, 64);

            BenchStats st;
            if (arm < 4) st = run_records_arm(ws, recs, n_recs, n, p, arm, test_out);
            else st = run_syms_arm(ws, seq_to_unseq, syms, m_syms, eob_sym, p, arm - 4, test_out);

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
            last_st[arm] = st;
        }

        double tot[NARMS][ROUNDS], emit_arr[NARMS][ROUNDS];
        double fr_s[NARMS] = {0}, p1_s[NARMS] = {0}, rv_s[NARMS] = {0}, wr_s[NARMS] = {0}, em_s[NARMS] = {0};

        for (int rnd = -WARMUP; rnd < ROUNDS; rnd++) {
            for (int slot = 0; slot < NARMS; slot++) {
                int arm = (slot + (rnd + WARMUP)) % NARMS;
                BenchStats st;
                if (arm < 4) st = run_records_arm(ws, recs, n_recs, n, p, arm, test_out);
                else st = run_syms_arm(ws, seq_to_unseq, syms, m_syms, eob_sym, p, arm - 4, test_out);

                if (rnd >= 0) {
                    tot[arm][rnd] = st.t_total_us;
                    emit_arr[arm][rnd] = st.t_emit_us;
                    fr_s[arm] += st.t_front_us;
                    p1_s[arm] += st.t_pass1_us;
                    rv_s[arm] += st.t_rv_us;
                    wr_s[arm] += st.t_wire_us;
                    em_s[arm] += st.t_emit_us;
                }
            }
        }

        for (int arm = 0; arm < NARMS; arm++) {
            qsort(tot[arm], ROUNDS, sizeof(double), cmp_double);
            qsort(emit_arr[arm], ROUNDS, sizeof(double), cmp_double);
        }
        double base_a = tot[1][ROUNDS / 2]; /* A1:Open-fast16 */
        double base_b = tot[4][ROUNDS / 2]; /* B0:2Stage-AVX2 */

        printf("--- %s (n=%u, syms=%u, R=%u) [ALL 7 ARMS BYTE-EXACT + CANARY OK] ---\n",
               specs[bi].file + 31, n, m_syms, n_recs);
        printf("  %-18s | %8s (%8s..%8s) | %6s | %7s %7s %7s %7s %7s | %8s %6s\n",
               "Arm", "Med(us)", "p25", "p75", "Ratio",
               "front", "pass1", "rv", "wire", "emitMed", "hops", "sc_hit");
        for (int arm = 0; arm < NARMS; arm++) {
            double med = tot[arm][ROUNDS / 2];
            double p25 = tot[arm][ROUNDS / 4];
            double p75 = tot[arm][(3 * ROUNDS) / 4];
            double em_med = emit_arr[arm][ROUNDS / 2];
            double ratio = (arm < 4) ? (med / base_a) : (med / base_b);
            printf("  %-18s | %8.1f (%8.1f..%8.1f) | %5.3fx | %7.1f %7.1f %7.1f %7.1f %7.1f | %8u %6u\n",
                   names[arm], med, p25, p75, ratio,
                   fr_s[arm] / ROUNDS, p1_s[arm] / ROUNDS, rv_s[arm] / ROUNDS,
                   wr_s[arm] / ROUNDS, em_med,
                   last_st[arm].emit_hops, last_st[arm].shortcuts_taken);
        }
        printf("\n");
    }

    free(test_out); free(ref_out); free(syms); free(recs); free(L);
    ws_destroy(ws);
    return 0;
}
