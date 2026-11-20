/*
 * Gate G5 Paid-Cost Falsifier & Controlled 6-Arm Confrontation Harness for Carbon Cycle 8:
 * Conjugate F-Space Bandless Schur Elimination (CFSE)
 * vs Shipped Open (qi16), Strengthened Open (U16Q), and Cycle 7's GIM-RV+BVSMC-2P.
 *
 * PRE-REGISTERED PASS/FAIL RULE (frozen before compilation and execution):
 * Across all six authenticated Wikidata BWT blocks (block-0, 2, 85, 143, 210, 207),
 * 25 paired interleaved rounds after 3 warmup rounds (rotating arm order (r + a) % 6 each round):
 *
 *   Arm 0: Open-qi16         — exact C mirror of shipped src/open.rs (q_push_scalar + endpoint.rs + 1-lane emit)
 *   Arm 1: CFSE-Scalar-1L    — pure non-SIMD architectural swap (scalar alpha_raw in dead out[] + M_64 + bandless
 *                              F-space wire + scalar memcpy 1-lane right-to-left emit; zero SIMD queue/emit!)
 *   Arm 2: Open+U16Q-1L      — strengthened 1-lane Open (U16Q + endpoint.rs + inline16 + 1-lane SIMD emit)
 *   Arm 3: CFSE-SIMD-1L      — 1-lane SIMD CFSE (store_seq16 alpha_raw in dead out[] + M_64 + bandless F-space
 *                              wire + vpinsrb right-aligned inline16_ra + 1-lane right-to-left SIMD emit;
 *                              0 B queues[256], 0 B bands[], 0 B offsets[])
 *   Arm 4: GIM-RV+BVSMC-2P   — Cycle 7 best zero-pool decoder (U16Q queues[256] + GIM-RV M_64 + bands[] +
 *                              offsets[] + 8-lane BVSMC-2P(8,64))
 *   Arm 5: CFSE+BVSMC-2P     — Cycle 8 zero-queue, bandless, zero-offset, zero-pool decoder (CFSE + 8-lane
 *                              right-to-left F-space BVSMC-2P(8,64))
 *
 * PASS iff ALL FOUR hold:
 *   (1) Exactness & Memory Safety: Arms 1..5 match dense LF byte-for-byte on 6/6 blocks (5,399,692 B),
 *       pass 8/8 adversarial self-tests (n=1, n=5 multi-cycle, 5k single-byte periodic, thick primary
 *       split/head/tail, 32k all-width-16 out-scratch fallback, 50k periodic shear >64B GimRule DAG,
 *       and overrun canary checks), and pass AddressSanitizer + UndefinedBehaviorSanitizer cleanly;
 *   (2) Pure Non-SIMD Architectural Isolation (breaking #659 Backward-Open regression):
 *       - Arm 1 (CFSE-Scalar-1L) construct median <= 0.80x Arm 0 (Open-qi16) construct on >= 5/6 blocks;
 *       - Arm 1 (CFSE-Scalar-1L) total median     <= 0.88x Arm 0 (Open-qi16) total     on >= 5/6 blocks;
 *   (3) Strengthened 1-Lane Gain:
 *       - Arm 3 (CFSE-SIMD-1L)   total median     <= 0.85x Arm 2 (Open+U16Q-1L) total  on >= 5/6 blocks;
 *   (4) Zero-Queue + Zero-Pool Multi-Lane End-to-End Compounding:
 *       - Arm 5 (CFSE+BVSMC-2P)  total median     <= 0.68x Arm 0 (Open-qi16) total     on >= 5/6 blocks;
 *       - Arm 5 (CFSE+BVSMC-2P)  total median     <= 0.97x Arm 4 (GIM-RV+BVSMC-2P)     on >= 5/6 blocks,
 *         while eliminating queues[256], bands[], and offsets[] (> 650 KiB scratch saved per worker).
 *
 * NOTE (METHOD.md §5): Single-thread VM C timings are calibrated kernel diagnostics, NOT proof of
 * 20-thread native end-to-end throughput; native adoption requires Coordinator Rust verification.
 */

#define _POSIX_C_SOURCE 200809L
#include <assert.h>
#include <immintrin.h>
#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#define LIMIT 16u
#define MAX_CUTS 64u
#define GIM_CAP 64u

typedef struct {
    uint32_t from, to, width;
    uint8_t byte;
} Strip;

typedef struct {
    uint32_t prev[2];
    uint32_t next[2];
    uint32_t width;
    uint32_t word;
} EpNode;

typedef struct {
    uint32_t left, right, len, first;
} EpRule;

typedef struct {
    uint32_t end, delta, code;
} Band;

/* =========================================================================
 * Baseline L-Space Kernel (Arms 0, 2, 4: Open-qi16, Open+U16Q, GIM-RV+BVSMC-2P)
 * ========================================================================= */
typedef struct {
    uint32_t *queues[256];
    uint32_t q_len[256];
    uint32_t q_cap[256];

    Strip *strips;
    size_t strips_len, strips_cap;
    uint32_t *image_order;
    size_t img_cap;

    EpNode *nodes;
    size_t nodes_len, nodes_cap;
    EpRule *rules;
    size_t rules_len, rules_cap;

    uint32_t *order[2];
    size_t order_len[2], order_cap[2];
    uint32_t *terminals[2];
    size_t term_len[2], term_cap[2];

    uint32_t *bottom;
    size_t bottom_cap;
    uint32_t *stack;
    size_t stack_len, stack_cap;
    Band *bands;
    size_t bands_len, bands_cap;

    uint8_t *small_arena;
    uint32_t arena_len, arena_cap;

    uint8_t *spelling;
    size_t spell_len, spell_cap;
    uint32_t *offsets;
    size_t off_len, off_cap;

    __m128i *inline16;
    uint16_t *w_len;
    size_t dict_cap;

    uint32_t *edges;
    size_t edges_cap;
    uint32_t *extra;
    size_t extra_cap;
    int wide;

    uint32_t m, mark, bits, words;
    uint8_t *dummy_buf;
} LKernel;

/* =========================================================================
 * Conjugate F-Space Bandless Kernel (Arms 1, 3, 5: CFSE — NO queues[256], NO order[2],
 * NO terminals[2], NO bands[], NO offsets[]!)
 * ========================================================================= */
typedef struct {
    Strip *strips;
    size_t strips_len, strips_cap;
    uint32_t *image_order;
    size_t img_cap;

    EpNode *nodes;
    size_t nodes_len, nodes_cap;
    EpRule *rules;
    size_t rules_len, rules_cap;

    uint32_t *bottom;
    size_t bottom_cap;
    uint32_t *stack;
    size_t stack_len, stack_cap;

    uint8_t *small_arena;
    uint32_t arena_len, arena_cap;

    /* Overflow buffer ONLY for >16B words (~1.8-6.4% of bands, 26-47 KiB vs 55-115 KiB!) */
    uint8_t *long_spell;
    size_t spell_len, spell_cap;

    /* Dual-purpose 16B dictionary:
     *   - if w_len[code] <= 16: right-aligned word bytes at [16 - ln .. 16)
     *   - if w_len[code] >  16: uint64_t[2] = { o0_in_long_spell, true_byte_len } */
    __m128i *inline16_ra;
    uint16_t *w_len;
    size_t dict_cap;

    uint32_t *edges;
    size_t edges_cap;
    uint32_t *extra;
    size_t extra_cap;

    /* Allocated ONLY if 4*(m+16) > n on dense synthetic blocks; 0 B on all 6 Wikidata blocks! */
    uint32_t *alpha_fallback;
    size_t alpha_fb_cap;
    int used_out_scratch;

    int wide;
    uint32_t m, mark_f, bits, words;
    uint8_t *dummy_buf;
} CfseKernel;

static double now_sec(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (double)ts.tv_sec + (double)ts.tv_nsec * 1e-9;
}

static inline void store_seq16(uint32_t *dst, uint32_t base) {
    const __m256i iota0 = _mm256_setr_epi32(0, 1, 2, 3, 4, 5, 6, 7);
    const __m256i iota1 = _mm256_setr_epi32(8, 9, 10, 11, 12, 13, 14, 15);
    __m256i b = _mm256_set1_epi32((int)base);
    _mm256_storeu_si256((__m256i *)dst, _mm256_add_epi32(b, iota0));
    _mm256_storeu_si256((__m256i *)(dst + 8), _mm256_add_epi32(b, iota1));
}

/* -------------------------------------------------------------------------
 * LKernel Init / Free / Construct / Emit (Exact from Cycle 7 g5_gimrv.c)
 * ------------------------------------------------------------------------- */
static void lkernel_init(LKernel *k, size_t max_n) {
    memset(k, 0, sizeof(*k));
    for (int a = 0; a < 256; a++) {
        k->q_cap[a] = 1024;
        k->queues[a] = (uint32_t *)malloc((k->q_cap[a] + 16) * sizeof(uint32_t));
    }
    k->strips_cap = 8192;
    k->strips = (Strip *)malloc(k->strips_cap * sizeof(Strip));
    k->img_cap = 8192;
    k->image_order = (uint32_t *)malloc(k->img_cap * sizeof(uint32_t));

    k->nodes_cap = 32768;
    k->nodes = (EpNode *)malloc(k->nodes_cap * sizeof(EpNode));
    k->rules_cap = 32768;
    k->rules = (EpRule *)malloc(k->rules_cap * sizeof(EpRule));
    for (int s = 0; s < 2; s++) {
        k->order_cap[s] = 32768;
        k->order[s] = (uint32_t *)malloc(k->order_cap[s] * sizeof(uint32_t));
        k->term_cap[s] = 16384;
        k->terminals[s] = (uint32_t *)malloc(k->term_cap[s] * sizeof(uint32_t));
    }
    k->bottom_cap = 32768;
    k->bottom = (uint32_t *)malloc(k->bottom_cap * sizeof(uint32_t));
    k->stack_cap = 4096;
    k->stack = (uint32_t *)malloc(k->stack_cap * sizeof(uint32_t));
    k->bands_cap = 16384;
    k->bands = (Band *)malloc(k->bands_cap * sizeof(Band));

    k->arena_cap = 262144;
    k->small_arena = (uint8_t *)malloc(k->arena_cap + 128);
    memset(k->small_arena, 0, 320);
    for (uint32_t c = 0; c < 256; c++) k->small_arena[c] = (uint8_t)c;
    k->arena_len = 256;

    k->spell_cap = 131072;
    k->spelling = (uint8_t *)malloc(k->spell_cap + 128);
    k->off_cap = 16384;
    k->offsets = (uint32_t *)malloc(k->off_cap * sizeof(uint32_t));

    k->dict_cap = 16384;
    k->inline16 = (__m128i *)aligned_alloc(64, k->dict_cap * sizeof(__m128i));
    k->w_len = (uint16_t *)calloc(k->dict_cap, sizeof(uint16_t));
    memset(k->inline16, 0, 256 * sizeof(__m128i));
    for (uint32_t c = 0; c < 256; c++) {
        ((uint8_t *)&k->inline16[c])[0] = (uint8_t)c;
        k->w_len[c] = 1u;
    }

    k->edges_cap = max_n + 16;
    k->edges = (uint32_t *)malloc(k->edges_cap * sizeof(uint32_t));
    k->extra_cap = max_n + 16;
    k->extra = (uint32_t *)malloc(k->extra_cap * sizeof(uint32_t));
    k->dummy_buf = (uint8_t *)malloc(4096);
}

static void lkernel_free(LKernel *k) {
    for (int a = 0; a < 256; a++) free(k->queues[a]);
    free(k->strips);
    free(k->image_order);
    free(k->nodes);
    free(k->rules);
    for (int s = 0; s < 2; s++) {
        free(k->order[s]);
        free(k->terminals[s]);
    }
    free(k->bottom);
    free(k->stack);
    free(k->bands);
    free(k->small_arena);
    free(k->spelling);
    free(k->offsets);
    free(k->inline16);
    free(k->w_len);
    free(k->edges);
    free(k->extra);
    free(k->dummy_buf);
}

static size_t lkernel_scratch_bytes(const LKernel *k, int include_4vec, size_t m_used) {
    size_t q_bytes = 0;
    for (int a = 0; a < 256; a++) q_bytes += (size_t)k->q_cap[a] * 4u;
    size_t vec4 = 0;
    if (include_4vec) {
        for (int s = 0; s < 2; s++) vec4 += (k->order_cap[s] + k->term_cap[s]) * 4u;
    }
    return q_bytes
         + k->strips_cap * sizeof(Strip)
         + k->img_cap * 4u
         + k->nodes_cap * sizeof(EpNode)
         + k->rules_len * sizeof(EpRule)
         + vec4
         + k->bottom_cap * 4u
         + k->bands_len * sizeof(Band)
         + k->arena_len
         + k->spell_len
         + k->off_len * 4u
         + (size_t)(256u + k->words) * (sizeof(__m128i) + sizeof(uint16_t))
         + m_used * 4u;
}

static inline void l_dict_ensure(LKernel *k, size_t need) {
    if (__builtin_expect(need > k->dict_cap, 0)) {
        size_t old_cap = k->dict_cap;
        while (need > k->dict_cap) k->dict_cap *= 2;
        __m128i *n16 = (__m128i *)aligned_alloc(64, k->dict_cap * sizeof(__m128i));
        memcpy(n16, k->inline16, old_cap * sizeof(__m128i));
        free(k->inline16);
        k->inline16 = n16;
        k->w_len = (uint16_t *)realloc(k->w_len, k->dict_cap * sizeof(uint16_t));
    }
}

static inline void q_push_scalar(LKernel *k, uint8_t a, uint32_t start, uint32_t w) {
    uint32_t len = k->q_len[a];
    if (__builtin_expect(len + w > k->q_cap[a], 0)) {
        while (len + w > k->q_cap[a]) k->q_cap[a] *= 2;
        k->queues[a] = (uint32_t *)realloc(k->queues[a], (k->q_cap[a] + 16) * sizeof(uint32_t));
    }
    uint32_t *dst = k->queues[a] + len;
    for (uint32_t i = 0; i < w; i++) dst[i] = start + i;
    k->q_len[a] = len + w;
}

static inline void q_push_u16(LKernel *k, uint8_t a, uint32_t start, uint32_t w) {
    uint32_t len = k->q_len[a];
    if (__builtin_expect(len + 16 > k->q_cap[a], 0)) {
        while (len + 16 > k->q_cap[a]) k->q_cap[a] *= 2;
        k->queues[a] = (uint32_t *)realloc(k->queues[a], (k->q_cap[a] + 16) * sizeof(uint32_t));
    }
    store_seq16(k->queues[a] + len, start);
    k->q_len[a] = len + w;
}

static inline void l_push_strip(LKernel *k, uint32_t from, uint32_t to, uint32_t width, uint8_t byte) {
    if (__builtin_expect(k->strips_len == k->strips_cap, 0)) {
        k->strips_cap *= 2;
        k->strips = (Strip *)realloc(k->strips, k->strips_cap * sizeof(Strip));
    }
    k->strips[k->strips_len++] = (Strip){from, to, width, byte};
}

static inline void l_spell_ensure(LKernel *k, size_t add) {
    if (__builtin_expect(k->spell_len + add + 80 > k->spell_cap, 0)) {
        while (k->spell_len + add + 80 > k->spell_cap) k->spell_cap *= 2;
        k->spelling = (uint8_t *)realloc(k->spelling, k->spell_cap + 128);
    }
}

static inline void push_u32(uint32_t **arr, size_t *len, size_t *cap, uint32_t v) {
    if (__builtin_expect(*len == *cap, 0)) {
        *cap *= 2;
        *arr = (uint32_t *)realloc(*arr, (*cap) * sizeof(uint32_t));
    }
    (*arr)[(*len)++] = v;
}

static inline uint32_t ep_new_node(LKernel *k, uint32_t width, uint32_t word) {
    if (__builtin_expect(k->nodes_len == k->nodes_cap, 0)) {
        k->nodes_cap *= 2;
        k->nodes = (EpNode *)realloc(k->nodes, k->nodes_cap * sizeof(EpNode));
    }
    uint32_t id = (uint32_t)k->nodes_len++;
    k->nodes[id] = (EpNode){{0, 0}, {0, 0}, width, word};
    return id;
}

static inline void ep_cut(EpNode *nodes, uint32_t id, int side) {
    uint32_t p = nodes[id].prev[side], nx = nodes[id].next[side];
    nodes[p].next[side] = nx;
    nodes[nx].prev[side] = p;
}

static inline void ep_insert(EpNode *nodes, uint32_t at, uint32_t id, int side) {
    uint32_t nx = nodes[at].next[side];
    nodes[id].prev[side] = at;
    nodes[id].next[side] = nx;
    nodes[at].next[side] = id;
    nodes[nx].prev[side] = id;
}

static inline uint32_t ep_len(const LKernel *k, uint32_t w) {
    if (w <= 256u) return w != 0 ? 1u : 0u;
    return k->rules[w - 257u].len;
}

static inline uint32_t ep_join(LKernel *k, uint32_t left, uint32_t right) {
    if (left == 0) return right;
    if (right == 0) return left;
    if (__builtin_expect(k->rules_len == k->rules_cap, 0)) {
        k->rules_cap *= 2;
        k->rules = (EpRule *)realloc(k->rules, k->rules_cap * sizeof(EpRule));
    }
    uint32_t id = (uint32_t)k->rules_len + 257u;
    uint32_t ln = ep_len(k, left) + ep_len(k, right);
    k->rules[k->rules_len++] = (EpRule){left, right, ln, UINT32_MAX};
    return id;
}

static inline void ep_gaps(LKernel *k, int side, uint32_t start, uint32_t end) {
    if (start < end) {
        uint32_t id = ep_new_node(k, end - start, 0);
        push_u32(&k->order[side], &k->order_len[side], &k->order_cap[side], id);
        push_u32(&k->terminals[1 - side], &k->term_len[1 - side], &k->term_cap[1 - side], id);
    }
}

static void ep_append(LKernel *k, uint32_t word) {
    k->stack_len = 0;
    push_u32(&k->stack, &k->stack_len, &k->stack_cap, word);
    while (k->stack_len > 0) {
        uint32_t id = k->stack[--k->stack_len];
        if (id <= 256u) {
            l_spell_ensure(k, 1);
            k->spelling[k->spell_len++] = (uint8_t)(id - 1u);
        } else {
            EpRule *rl = &k->rules[id - 257u];
            if (rl->first == UINT32_MAX) {
                rl->first = (uint32_t)k->spell_len;
                push_u32(&k->stack, &k->stack_len, &k->stack_cap, rl->right);
                push_u32(&k->stack, &k->stack_len, &k->stack_cap, rl->left);
            } else {
                l_spell_ensure(k, rl->len);
                memmove(k->spelling + k->spell_len, k->spelling + rl->first, rl->len);
                k->spell_len += rl->len;
            }
        }
    }
}

static void open_construct(
    LKernel *k, const uint16_t *records, size_t n_recs, uint32_t n, uint32_t p,
    uint32_t limit, int use_u16_push, int build_inline16, uint32_t reserve_sentinels
) {
    uint32_t all_c[256] = {0}, thin[256] = {0}, thick[256] = {0};
    for (int a = 0; a < 256; a++) k->q_len[a] = 0;
    k->strips_len = 0;

    uint32_t m = 0, mark = 0, pos = 0;
    size_t i = 0;
    while (i < n_recs) {
        uint16_t rec = records[i++];
        uint8_t c = (uint8_t)(rec >> 8);
        uint32_t width = (uint32_t)(rec & 255u);
        while (i < n_recs && (uint8_t)(records[i] >> 8) == c) width += (uint32_t)(records[i++] & 255u);
        uint32_t end = pos + width;
        if (width > limit && p >= pos && p < end) {
            if (pos < p) { l_push_strip(k, pos, all_c[c], p - pos, c); thick[c]++; all_c[c] += p - pos; }
            mark = m;
            if (use_u16_push) q_push_u16(k, c, m, 1); else q_push_scalar(k, c, m, 1);
            thin[c] += 1; m += 1; all_c[c] += 1;
            if (p + 1 < end) { uint32_t w2 = end - p - 1; l_push_strip(k, p + 1, all_c[c], w2, c); thick[c]++; all_c[c] += w2; }
        } else if (width <= limit) {
            if (p >= pos && p < end) mark = m + (p - pos);
            if (use_u16_push) q_push_u16(k, c, m, width); else q_push_scalar(k, c, m, width);
            thin[c] += width; m += width; all_c[c] += width;
        } else {
            l_push_strip(k, pos, all_c[c], width, c); thick[c]++; all_c[c] += width;
        }
        pos = end;
    }

    uint32_t first[256], write[256], tot = 0, tw = 0;
    for (int a = 0; a < 256; a++) {
        first[a] = tot; tot += all_c[a];
        write[a] = tw; tw += thick[a];
    }
    if (k->strips_len > k->img_cap) {
        k->img_cap = k->strips_len * 2;
        k->image_order = (uint32_t *)realloc(k->image_order, k->img_cap * sizeof(uint32_t));
    }
    for (size_t si = 0; si < k->strips_len; si++) {
        uint8_t a = k->strips[si].byte;
        k->strips[si].to += first[a];
        k->image_order[write[a]++] = (uint32_t)si;
    }

    k->nodes_len = 0; k->rules_len = 0;
    ep_new_node(k, 0, 0);
    for (int s = 0; s < 2; s++) { k->order_len[s] = 0; k->term_len[s] = 0; }
    for (size_t si = 0; si < k->strips_len; si++) ep_new_node(k, k->strips[si].width, (uint32_t)k->strips[si].byte + 1u);

    uint32_t x = 0;
    for (size_t si = 0; si < k->strips_len; si++) {
        ep_gaps(k, 0, x, k->strips[si].from);
        push_u32(&k->order[0], &k->order_len[0], &k->order_cap[0], (uint32_t)si + 1u);
        x = k->strips[si].from + k->strips[si].width;
    }
    ep_gaps(k, 0, x, n);

    x = 0;
    for (size_t ii = 0; ii < k->strips_len; ii++) {
        uint32_t idx = k->image_order[ii];
        Strip s = k->strips[idx];
        ep_gaps(k, 1, x, s.to);
        push_u32(&k->order[1], &k->order_len[1], &k->order_cap[1], idx + 1u);
        x = s.to + s.width;
    }
    ep_gaps(k, 1, x, n);

    EpNode *nodes = k->nodes;
    for (int side = 0; side < 2; side++) {
        uint32_t prev = 0;
        for (size_t ti = 0; ti < k->term_len[side]; ti++) {
            uint32_t id = k->terminals[side][ti];
            nodes[prev].next[side] = id; nodes[id].prev[side] = prev; prev = id;
        }
        for (size_t oi = 0; oi < k->order_len[side]; oi++) {
            uint32_t id = k->order[side][oi];
            nodes[prev].next[side] = id; nodes[id].prev[side] = prev; prev = id;
        }
        nodes[prev].next[side] = 0; nodes[0].prev[side] = prev;
    }

    uint32_t remaining = n;
    while (remaining > 0) {
        uint32_t xid = nodes[0].prev[0], yid = nodes[0].prev[1];
        EpNode a = nodes[xid], b = nodes[yid];
        uint32_t w = a.width < b.width ? a.width : b.width;
        remaining -= w;
        if (xid == yid) { ep_cut(nodes, xid, 0); ep_cut(nodes, xid, 1); continue; }
        uint32_t wd = ep_join(k, a.word, b.word);
        nodes = k->nodes;
        if (a.width <= b.width) {
            ep_cut(nodes, xid, 0); ep_insert(nodes, yid, xid, 0); nodes[xid].word = wd;
            if (a.width == b.width) { ep_cut(nodes, yid, 0); ep_cut(nodes, yid, 1); }
            else nodes[yid].width -= w;
        } else {
            ep_cut(nodes, yid, 1); ep_insert(nodes, xid, yid, 1); nodes[yid].word = wd;
            nodes[xid].width -= w;
        }
    }

    if (k->nodes_len > k->bottom_cap) {
        k->bottom_cap = k->nodes_len * 2;
        k->bottom = (uint32_t *)realloc(k->bottom, k->bottom_cap * sizeof(uint32_t));
    }
    uint32_t id = nodes[0].next[1]; x = 0;
    while (id != 0) { k->bottom[id] = x; x += nodes[id].width; id = nodes[id].next[1]; }

    k->bands_len = 0; k->spell_len = 0; k->off_len = 0; k->offsets[k->off_len++] = 0;
    uint32_t cursors[256]; id = nodes[0].next[0];
    uint32_t in_node = 0, kc = 0, words = 0;
    for (int a = 0; a < 256; a++) {
        cursors[a] = (uint32_t)k->bands_len;
        uint32_t end = kc + thin[a];
        while (kc < end) {
            EpNode nd = nodes[id];
            uint32_t rem_a = end - kc, rem_n = nd.width - in_node;
            uint32_t width = rem_a < rem_n ? rem_a : rem_n;
            uint32_t dst = k->bottom[id] + in_node;
            uint32_t code;
            if (nd.word == 0) {
                code = (uint32_t)a;
            } else {
                uint32_t w_idx = words++;
                code = 256u + w_idx;
                uint32_t o0 = (uint32_t)k->spell_len;
                ep_append(k, nd.word);
                l_spell_ensure(k, 1);
                k->spelling[k->spell_len++] = (uint8_t)a;
                uint32_t o1 = (uint32_t)k->spell_len;
                push_u32(&k->offsets, &k->off_len, &k->off_cap, o1);
                if (build_inline16) {
                    l_dict_ensure(k, (size_t)code + reserve_sentinels + 16);
                    uint32_t ln = o1 - o0;
                    k->w_len[code] = ln <= 65534u ? (uint16_t)ln : 65534u;
                    k->inline16[code] = _mm_loadu_si128((const __m128i *)(k->spelling + o0));
                }
            }
            if (__builtin_expect(k->bands_len == k->bands_cap, 0)) {
                k->bands_cap *= 2;
                k->bands = (Band *)realloc(k->bands, k->bands_cap * sizeof(Band));
            }
            k->bands[k->bands_len++] = (Band){kc + width, dst - kc, code};
            in_node += width; kc += width;
            if (in_node == nd.width) { id = nd.next[0]; in_node = 0; }
        }
    }
    if (build_inline16) l_dict_ensure(k, 256u + words + reserve_sentinels + 4);

    uint32_t bits = m <= 1 ? 1u : (32u - (uint32_t)__builtin_clz(m - 1));
    uint32_t max_code = 255u + words + reserve_sentinels;
    int wide = max_code > (UINT32_MAX >> bits);

    kc = 0;
    for (int a = 0; a < 256; a++) {
        uint32_t b_end_idx = (a + 1 < 256) ? cursors[a + 1] : (uint32_t)k->bands_len;
        uint32_t from = 0;
        const uint32_t * restrict qa = k->queues[a];
        for (uint32_t bi = cursors[a]; bi < b_end_idx; bi++) {
            Band b = k->bands[bi];
            uint32_t span = b.end - kc, dst = kc + b.delta;
            uint32_t pack = wide ? 0u : (b.code << bits);
            if (wide) {
                for (uint32_t s = 0; s < span; s++) k->extra[dst + s] = b.code;
            }
            uint32_t * restrict edst = k->edges + dst;
            const uint32_t * restrict qsrc = qa + from;
            for (uint32_t s = 0; s < span; s++) edst[s] = pack | qsrc[s];
            from += span; kc = b.end;
        }
    }

    k->m = m; k->mark = mark; k->bits = bits; k->words = words; k->wide = wide;
}

/* Cycle 7 GIM-RV Constructor (Arm 4 baseline) */
static inline uint32_t gim_len(const LKernel *k, uint32_t w) {
    if (w == 0) return 0u;
    if ((w & 0x80000000u) == 0) return w >> 24;
    return k->rules[w & 0x7FFFFFFFu].len;
}

static inline uint32_t gim_join(LKernel *k, uint32_t aw, uint32_t bw) {
    if (aw == 0) return bw;
    if (bw == 0) return aw;
    if (__builtin_expect(((aw | bw) & 0x80000000u) == 0, 1)) {
        uint32_t al = aw >> 24, bl = bw >> 24, tl = al + bl;
        if (__builtin_expect(tl <= GIM_CAP && k->arena_len + tl < 0x00FFF000u, 1)) {
            if (__builtin_expect(k->arena_len + tl + 64u > k->arena_cap, 0)) {
                while (k->arena_len + tl + 64u > k->arena_cap) k->arena_cap *= 2;
                k->small_arena = (uint8_t *)realloc(k->small_arena, k->arena_cap + 128);
            }
            uint32_t dst_off = k->arena_len;
            const uint8_t *sa = k->small_arena + (aw & 0x00FFFFFFu);
            const uint8_t *sb = k->small_arena + (bw & 0x00FFFFFFu);
            uint8_t *dst = k->small_arena + dst_off;
            if (__builtin_expect(tl <= 16u, 1)) {
                _mm_storeu_si128((__m128i *)dst, _mm_loadu_si128((const __m128i *)sa));
                _mm_storeu_si128((__m128i *)(dst + al), _mm_loadu_si128((const __m128i *)sb));
            } else {
                __m256i va0 = _mm256_loadu_si256((const __m256i *)sa);
                __m256i va1 = al > 32u ? _mm256_loadu_si256((const __m256i *)(sa + 32)) : _mm256_setzero_si256();
                __m256i vb0 = _mm256_loadu_si256((const __m256i *)sb);
                __m256i vb1 = bl > 32u ? _mm256_loadu_si256((const __m256i *)(sb + 32)) : _mm256_setzero_si256();
                _mm256_storeu_si256((__m256i *)dst, va0);
                if (al > 32u) _mm256_storeu_si256((__m256i *)(dst + 32), va1);
                _mm256_storeu_si256((__m256i *)(dst + al), vb0);
                if (bl > 32u) _mm256_storeu_si256((__m256i *)(dst + al + 32), vb1);
            }
            k->arena_len = dst_off + tl;
            return (tl << 24) | dst_off;
        }
    }
    if (__builtin_expect(k->rules_len == k->rules_cap, 0)) {
        k->rules_cap *= 2;
        k->rules = (EpRule *)realloc(k->rules, k->rules_cap * sizeof(EpRule));
    }
    uint32_t rid = (uint32_t)k->rules_len++;
    uint32_t ln = gim_len(k, aw) + gim_len(k, bw);
    k->rules[rid] = (EpRule){aw, bw, ln, UINT32_MAX};
    return 0x80000000u | rid;
}

__attribute__((noinline))
static void gim_append_rule(LKernel *k, uint32_t word) {
    k->stack_len = 0;
    push_u32(&k->stack, &k->stack_len, &k->stack_cap, word);
    while (k->stack_len > 0) {
        uint32_t w = k->stack[--k->stack_len];
        if ((w & 0x80000000u) == 0) {
            uint32_t wl = w >> 24, woff = w & 0x00FFFFFFu;
            l_spell_ensure(k, wl);
            memcpy(k->spelling + k->spell_len, k->small_arena + woff, wl);
            k->spell_len += wl;
        } else {
            EpRule *rl = &k->rules[w & 0x7FFFFFFFu];
            if (rl->first == UINT32_MAX) {
                rl->first = (uint32_t)k->spell_len;
                push_u32(&k->stack, &k->stack_len, &k->stack_cap, rl->right);
                push_u32(&k->stack, &k->stack_len, &k->stack_cap, rl->left);
            } else {
                l_spell_ensure(k, rl->len);
                memmove(k->spelling + k->spell_len, k->spelling + rl->first, rl->len);
                k->spell_len += rl->len;
            }
        }
    }
}

#define LINK_CHAIN(NODES, SIDE, HEAD, TAIL, ID) do { \
    uint32_t _id = (ID); \
    if ((HEAD) == 0) { (HEAD) = _id; } \
    else { (NODES)[(TAIL)].next[(SIDE)] = _id; (NODES)[_id].prev[(SIDE)] = (TAIL); } \
    (TAIL) = _id; \
} while (0)

static void gim_rv_construct(
    LKernel *k, const uint16_t *records, size_t n_recs, uint32_t n, uint32_t p,
    uint32_t limit, uint32_t reserve_sentinels
) {
    uint32_t all_c[256] = {0}, thin[256] = {0}, thick[256] = {0};
    for (int a = 0; a < 256; a++) k->q_len[a] = 0;
    k->strips_len = 0;

    uint32_t m = 0, mark = 0, pos = 0;
    size_t i = 0;
    while (i < n_recs) {
        uint16_t rec = records[i++];
        uint8_t c = (uint8_t)(rec >> 8);
        uint32_t width = (uint32_t)(rec & 255u);
        while (i < n_recs && (uint8_t)(records[i] >> 8) == c) width += (uint32_t)(records[i++] & 255u);
        uint32_t end = pos + width;
        if (__builtin_expect(width <= limit, 1)) {
            if (__builtin_expect(p >= pos && p < end, 0)) mark = m + (p - pos);
            q_push_u16(k, c, m, width);
            thin[c] += width; m += width; all_c[c] += width;
        } else if (__builtin_expect(p >= pos && p < end, 0)) {
            if (pos < p) { l_push_strip(k, pos, all_c[c], p - pos, c); thick[c]++; all_c[c] += p - pos; }
            mark = m; q_push_u16(k, c, m, 1); thin[c] += 1; m += 1; all_c[c] += 1;
            if (p + 1 < end) { uint32_t w2 = end - p - 1; l_push_strip(k, p + 1, all_c[c], w2, c); thick[c]++; all_c[c] += w2; }
        } else {
            l_push_strip(k, pos, all_c[c], width, c); thick[c]++; all_c[c] += width;
        }
        pos = end;
    }

    uint32_t first[256], write[256], tot = 0, tw = 0;
    for (int a = 0; a < 256; a++) {
        first[a] = tot; tot += all_c[a];
        write[a] = tw; tw += thick[a];
    }
    if (k->strips_len > k->img_cap) {
        k->img_cap = k->strips_len * 2;
        k->image_order = (uint32_t *)realloc(k->image_order, k->img_cap * sizeof(uint32_t));
    }
    for (size_t si = 0; si < k->strips_len; si++) {
        uint8_t a = k->strips[si].byte;
        k->strips[si].to += first[a];
        k->image_order[write[a]++] = (uint32_t)si;
    }

    size_t max_nodes = 3 * k->strips_len + 8;
    if (max_nodes > k->nodes_cap) {
        while (max_nodes > k->nodes_cap) k->nodes_cap *= 2;
        k->nodes = (EpNode *)realloc(k->nodes, k->nodes_cap * sizeof(EpNode));
    }
    EpNode * restrict nodes = k->nodes;
    nodes[0] = (EpNode){{0, 0}, {0, 0}, 0, 0};
    uint32_t n_nodes = 1u;
    k->rules_len = 0; k->arena_len = 256u;

    size_t n_strips = k->strips_len;
    for (size_t si = 0; si < n_strips; si++) {
        nodes[n_nodes++] = (EpNode){{0, 0}, {0, 0}, k->strips[si].width, (1u << 24) | (uint32_t)k->strips[si].byte};
    }

    uint32_t th0 = 0, tt0 = 0, oh0 = 0, ot0 = 0;
    uint32_t th1 = 0, tt1 = 0, oh1 = 0, ot1 = 0;
    uint32_t x = 0;
    for (size_t si = 0; si < n_strips; si++) {
        uint32_t sf = k->strips[si].from;
        if (x < sf) {
            uint32_t gid = n_nodes++;
            nodes[gid] = (EpNode){{0, 0}, {0, 0}, sf - x, 0};
            LINK_CHAIN(nodes, 0, oh0, ot0, gid); LINK_CHAIN(nodes, 1, th1, tt1, gid);
        }
        LINK_CHAIN(nodes, 0, oh0, ot0, (uint32_t)si + 1u);
        x = sf + k->strips[si].width;
    }
    if (x < n) {
        uint32_t gid = n_nodes++;
        nodes[gid] = (EpNode){{0, 0}, {0, 0}, n - x, 0};
        LINK_CHAIN(nodes, 0, oh0, ot0, gid); LINK_CHAIN(nodes, 1, th1, tt1, gid);
    }

    x = 0;
    for (size_t ii = 0; ii < n_strips; ii++) {
        uint32_t idx = k->image_order[ii]; Strip s = k->strips[idx];
        if (x < s.to) {
            uint32_t gid = n_nodes++;
            nodes[gid] = (EpNode){{0, 0}, {0, 0}, s.to - x, 0};
            LINK_CHAIN(nodes, 1, oh1, ot1, gid); LINK_CHAIN(nodes, 0, th0, tt0, gid);
        }
        LINK_CHAIN(nodes, 1, oh1, ot1, idx + 1u);
        x = s.to + s.width;
    }
    if (x < n) {
        uint32_t gid = n_nodes++;
        nodes[gid] = (EpNode){{0, 0}, {0, 0}, n - x, 0};
        LINK_CHAIN(nodes, 1, oh1, ot1, gid); LINK_CHAIN(nodes, 0, th0, tt0, gid);
    }
    k->nodes_len = n_nodes;

    {
        uint32_t f0, l0;
        if (th0 && oh0) { nodes[tt0].next[0] = oh0; nodes[oh0].prev[0] = tt0; f0 = th0; l0 = ot0; }
        else if (th0)   { f0 = th0; l0 = tt0; } else { f0 = oh0; l0 = ot0; }
        nodes[0].next[0] = f0; nodes[f0].prev[0] = 0; nodes[l0].next[0] = 0; nodes[0].prev[0] = l0;

        uint32_t f1, l1;
        if (th1 && oh1) { nodes[tt1].next[1] = oh1; nodes[oh1].prev[1] = tt1; f1 = th1; l1 = ot1; }
        else if (th1)   { f1 = th1; l1 = tt1; } else { f1 = oh1; l1 = ot1; }
        nodes[0].next[1] = f1; nodes[f1].prev[1] = 0; nodes[l1].next[1] = 0; nodes[0].prev[1] = l1;
    }

    uint32_t remaining = n;
    while (remaining > 0) {
        uint32_t xid = nodes[0].prev[0], yid = nodes[0].prev[1];
        EpNode a = nodes[xid], b = nodes[yid];
        uint32_t w = a.width < b.width ? a.width : b.width;
        remaining -= w;
        if (xid == yid) { ep_cut(nodes, xid, 0); ep_cut(nodes, xid, 1); continue; }
        uint32_t wd = gim_join(k, a.word, b.word);
        if (a.width <= b.width) {
            ep_cut(nodes, xid, 0); ep_insert(nodes, yid, xid, 0); nodes[xid].word = wd;
            if (a.width == b.width) { ep_cut(nodes, yid, 0); ep_cut(nodes, yid, 1); }
            else nodes[yid].width -= w;
        } else {
            ep_cut(nodes, yid, 1); ep_insert(nodes, xid, yid, 1); nodes[yid].word = wd;
            nodes[xid].width -= w;
        }
    }

    if (n_nodes > k->bottom_cap) {
        k->bottom_cap = n_nodes * 2;
        k->bottom = (uint32_t *)realloc(k->bottom, k->bottom_cap * sizeof(uint32_t));
    }
    uint32_t * restrict bottom = k->bottom;
    uint32_t id = nodes[0].next[1]; x = 0;
    while (id != 0) { bottom[id] = x; x += nodes[id].width; id = nodes[id].next[1]; }

    k->bands_len = 0; k->spell_len = 0; k->off_len = 0; k->offsets[k->off_len++] = 0;
    uint32_t cursors[256]; id = nodes[0].next[0];
    uint32_t in_node = 0, kc = 0, words = 0;
    const uint8_t * restrict arena = k->small_arena;

    for (int a = 0; a < 256; a++) {
        cursors[a] = (uint32_t)k->bands_len;
        uint32_t end = kc + thin[a];
        while (kc < end) {
            EpNode nd = nodes[id];
            uint32_t rem_a = end - kc, rem_n = nd.width - in_node;
            uint32_t width = rem_a < rem_n ? rem_a : rem_n;
            uint32_t dst = bottom[id] + in_node;
            uint32_t code;
            if (nd.word == 0) {
                code = (uint32_t)a;
            } else {
                uint32_t w_idx = words++;
                code = 256u + w_idx;
                uint32_t o0 = (uint32_t)k->spell_len, o1;
                if (__builtin_expect((nd.word & 0x80000000u) == 0, 1)) {
                    uint32_t wl = nd.word >> 24;
                    const uint8_t *wsrc = arena + (nd.word & 0x00FFFFFFu);
                    l_spell_ensure(k, wl + 1u);
                    uint8_t * restrict sp = k->spelling + o0;
                    if (__builtin_expect(wl <= 15u, 1)) _mm_storeu_si128((__m128i *)sp, _mm_loadu_si128((const __m128i *)wsrc));
                    else {
                        _mm256_storeu_si256((__m256i *)sp, _mm256_loadu_si256((const __m256i *)wsrc));
                        if (wl > 32u) _mm256_storeu_si256((__m256i *)(sp + 32), _mm256_loadu_si256((const __m256i *)(wsrc + 32)));
                    }
                    sp[wl] = (uint8_t)a;
                    o1 = o0 + wl + 1u; k->spell_len = o1;
                } else {
                    gim_append_rule(k, nd.word);
                    l_spell_ensure(k, 1u);
                    k->spelling[k->spell_len++] = (uint8_t)a;
                    o1 = (uint32_t)k->spell_len;
                }
                push_u32(&k->offsets, &k->off_len, &k->off_cap, o1);
                l_dict_ensure(k, (size_t)code + reserve_sentinels + 16);
                uint32_t ln = o1 - o0;
                k->w_len[code] = ln <= 65534u ? (uint16_t)ln : 65534u;
                k->inline16[code] = _mm_loadu_si128((const __m128i *)(k->spelling + o0));
            }
            if (__builtin_expect(k->bands_len == k->bands_cap, 0)) {
                k->bands_cap *= 2;
                k->bands = (Band *)realloc(k->bands, k->bands_cap * sizeof(Band));
            }
            k->bands[k->bands_len++] = (Band){kc + width, dst - kc, code};
            in_node += width; kc += width;
            if (in_node == nd.width) { id = nd.next[0]; in_node = 0; }
        }
    }
    l_dict_ensure(k, 256u + words + reserve_sentinels + 4);

    uint32_t bits = m <= 1 ? 1u : (32u - (uint32_t)__builtin_clz(m - 1));
    uint32_t max_code = 255u + words + reserve_sentinels;
    int wide = max_code > (UINT32_MAX >> bits);

    kc = 0;
    for (int a = 0; a < 256; a++) {
        uint32_t b_end_idx = (a + 1 < 256) ? cursors[a + 1] : (uint32_t)k->bands_len;
        uint32_t from = 0;
        const uint32_t * restrict qa = k->queues[a];
        for (uint32_t bi = cursors[a]; bi < b_end_idx; bi++) {
            Band b = k->bands[bi];
            uint32_t span = b.end - kc, dst = kc + b.delta;
            uint32_t pack = wide ? 0u : (b.code << bits);
            if (wide) {
                for (uint32_t s = 0; s < span; s++) k->extra[dst + s] = b.code;
            }
            uint32_t * restrict edst = k->edges + dst;
            const uint32_t * restrict qsrc = qa + from;
            for (uint32_t s = 0; s < span; s++) edst[s] = pack | qsrc[s];
            from += span; kc = b.end;
        }
    }
    k->m = m; k->mark = mark; k->bits = bits; k->words = words; k->wide = wide;
}

static inline void repeat_period(size_t n, uint8_t *out, size_t *out_len) {
    size_t len = *out_len;
    if (len > n) len = n;
    while (len < n) {
        size_t take = len < (n - len) ? len : (n - len);
        memcpy(out + len, out, take);
        len += take;
    }
    *out_len = n;
}

static void open_emit_orig(const LKernel *k, size_t n, uint8_t *out) {
    const uint32_t * restrict edges = k->edges;
    const uint32_t * restrict extra = k->extra;
    const uint8_t * restrict spelling = k->spelling;
    const uint32_t * restrict offsets = k->offsets;
    const uint32_t m = k->m, mark = k->mark, bits = k->bits, mask = (1u << bits) - 1u;
    const int wide = k->wide;
    uint8_t * restrict out_p = out;
    uint8_t * const out_end = out + n;
    uint32_t j = mark;

    for (uint32_t step = 0; step < m; step++) {
        uint32_t edge = edges[j];
        uint32_t code = wide ? extra[j] : (edge >> bits);
        j = edge & mask;
        if (code < 256u) {
            *out_p++ = (uint8_t)code;
        } else {
            uint32_t w = code - 256u;
            uint32_t o0 = offsets[w], ln = offsets[w + 1] - o0;
            if (__builtin_expect(out_p + ln > out_end, 0)) ln = (uint32_t)(out_end - out_p);
            memcpy(out_p, spelling + o0, ln);
            out_p += ln;
        }
        if (__builtin_expect(j == mark || out_p >= out_end, 0)) break;
    }
    size_t wr = (size_t)(out_p - out);
    repeat_period(n, out, &wr);
}

static void open_emit_u16_1lane(const LKernel *k, size_t n, uint8_t *out) {
    const uint32_t * restrict edges = k->edges;
    const uint32_t * restrict extra = k->extra;
    const __m128i * restrict inline16 = k->inline16;
    const uint16_t * restrict w_len = k->w_len;
    const uint8_t * restrict spelling = k->spelling;
    const uint32_t * restrict offsets = k->offsets;
    const uint32_t m = k->m, mark = k->mark, bits = k->bits, mask = (1u << bits) - 1u;
    const int wide = k->wide;
    uint8_t * restrict out_p = out;
    uint8_t * const out_end = out + n;
    uint8_t * const out_safe = n >= 16 ? (out + n - 15) : out;
    uint32_t j = mark;

    for (uint32_t step = 0; step < m; step++) {
        uint32_t edge = edges[j];
        uint32_t code = wide ? extra[j] : (edge >> bits);
        j = edge & mask;
        uint32_t ln = w_len[code];
        if (__builtin_expect(ln <= 16u && out_p < out_safe, 1)) {
            _mm_storeu_si128((__m128i *)out_p, inline16[code]);
            out_p += ln;
        } else {
            uint8_t b_lit = (uint8_t)code;
            const uint8_t *src;
            uint32_t real_ln;
            if (code < 256u) { src = &b_lit; real_ln = 1u; }
            else { uint32_t w = code - 256u, o0 = offsets[w]; src = spelling + o0; real_ln = offsets[w + 1] - o0; }
            if (out_p + real_ln > out_end) real_ln = (uint32_t)(out_end - out_p);
            memcpy(out_p, src, real_ln);
            out_p += real_ln;
        }
        if (__builtin_expect(j == mark || out_p >= out_end, 0)) break;
    }
    size_t wr = (size_t)(out_p - out);
    repeat_period(n, out, &wr);
}

typedef struct {
    uint8_t *safe_end[8], *hard_end[8];
    int active_flag[8];
    uint32_t cyc_segs[MAX_CUTS], cyc_off[MAX_CUTS], cyc_end[MAX_CUTS];
    uint32_t saved_code[MAX_CUTS], saved_next[MAX_CUTS];
    uint32_t n_cyc, next_idx, active, sbase;
    uint8_t *out;
} LState2P;

static inline void l_emit_bounded(LKernel *k, LState2P *st, int l, uint8_t **P_io, uint32_t cd) {
    uint8_t *P = *P_io;
    if (P >= st->hard_end[l]) return;
    uint32_t ln = k->w_len[cd];
    if (ln <= 16u && P < st->safe_end[l]) {
        _mm_storeu_si128((__m128i *)P, k->inline16[cd]);
        *P_io = P + ln;
        return;
    }
    uint8_t b_lit = (uint8_t)cd;
    const uint8_t *src; uint32_t rem;
    if (cd < 256u) { src = &b_lit; rem = 1u; }
    else { uint32_t w = cd - 256u, o0 = k->offsets[w]; src = k->spelling + o0; rem = k->offsets[w + 1] - o0; }
    uint32_t av = (uint32_t)(st->hard_end[l] - P);
    if (rem > av) rem = av;
    memcpy(P, src, rem);
    *P_io = P + rem;
}

__attribute__((noinline))
static void l_cold_2p(LKernel *k, LState2P *st, int l, uint32_t *J_io, uint8_t **P_io, uint32_t cd) {
    if (!st->active_flag[l]) { *P_io = k->dummy_buf; return; }
    if (cd < st->sbase) {
        l_emit_bounded(k, st, l, P_io, cd);
    } else if (st->next_idx < st->n_cyc) {
        uint32_t idx = st->next_idx++, sg = st->cyc_segs[idx];
        uint32_t off = st->cyc_off[idx], end = st->cyc_end[idx];
        uint8_t *P = st->out + off;
        st->hard_end[l] = st->out + end;
        st->safe_end[l] = (end - off >= 16u) ? (st->out + end - 15u) : (st->out + off);
        l_emit_bounded(k, st, l, &P, st->saved_code[sg]);
        *P_io = P; *J_io = st->saved_next[sg];
    } else {
        st->active_flag[l] = 0; *P_io = k->dummy_buf;
        st->safe_end[l] = k->dummy_buf + 2048; st->hard_end[l] = k->dummy_buf + 4096;
        *J_io = k->m; st->active--;
    }
}

static void l_bvsmc_emit_2p(LKernel *k, size_t n, uint32_t S_req, uint8_t *out) {
    uint32_t * restrict edges = k->edges;
    uint32_t * restrict extra = k->extra;
    const __m128i * restrict inline16 = k->inline16;
    uint16_t * restrict w_len = k->w_len;
    const uint32_t m = k->m, mark = k->mark, bits = k->bits, mask = (1u << bits) - 1u;
    const int wide = k->wide;
    const uint32_t sbase = 256u + k->words;
    uint32_t S_eff = S_req < m ? S_req : m;
    if (S_eff > MAX_CUTS) S_eff = MAX_CUTS;

    LState2P st; st.sbase = sbase; st.out = out;
    uint32_t cuts[MAX_CUTS], saved_raw[MAX_CUTS], seg_bytes[MAX_CUTS + 1] = {0};
    int32_t succ[MAX_CUTS + 1];
    for (uint32_t q = 0; q < S_eff; q++) {
        uint32_t v = (uint32_t)((mark + ((uint64_t)q * m) / S_eff) % m);
        cuts[q] = v; uint32_t raw = edges[v]; saved_raw[q] = raw;
        st.saved_next[q] = raw & mask;
        st.saved_code[q] = wide ? extra[v] : (raw >> bits);
        w_len[sbase + q] = 0u;
    }
    for (uint32_t q = 0; q < S_eff; q++) {
        uint32_t v = cuts[q], sc = sbase + q;
        if (wide) extra[v] = sc; else edges[v] = (sc << bits) | st.saved_next[q];
    }
    edges[m] = m; if (wide) extra[m] = 0;

    uint32_t p1_next = 0, p1_act = 0, j_init[8], q_init[8], b_init[8];
    for (int l = 0; l < 8; l++) {
        if (p1_next < S_eff) {
            uint32_t q = p1_next++; p1_act++; q_init[l] = q;
            uint32_t c0 = st.saved_code[q];
            b_init[l] = c0 < 256u ? 1u : (k->offsets[c0 - 255u] - k->offsets[c0 - 256u]);
            j_init[l] = st.saved_next[q];
        } else { q_init[l] = MAX_CUTS; b_init[l] = 0; j_init[l] = m; }
    }
    uint32_t j0=j_init[0], j1=j_init[1], j2=j_init[2], j3=j_init[3];
    uint32_t j4=j_init[4], j5=j_init[5], j6=j_init[6], j7=j_init[7];
    uint32_t q0=q_init[0], q1=q_init[1], q2=q_init[2], q3=q_init[3];
    uint32_t q4=q_init[4], q5=q_init[5], q6=q_init[6], q7=q_init[7];
    uint32_t b0=b_init[0], b1=b_init[1], b2=b_init[2], b3=b_init[3];
    uint32_t b4=b_init[4], b5=b_init[5], b6=b_init[6], b7=b_init[7];

    #define L_P1_STEP(J, Q, B, CD_EXPR, E_VAR) do { \
        uint32_t cd = (CD_EXPR); J = (E_VAR) & mask; uint32_t wl = w_len[cd]; B += wl; \
        if (__builtin_expect(wl >= 65534u || cd >= sbase, 0)) { \
            if (cd >= sbase) { \
                succ[Q] = (int32_t)(cd - sbase); seg_bytes[Q] = B; \
                if (p1_next < S_eff) { \
                    uint32_t nq = p1_next++; Q = nq; uint32_t c0 = st.saved_code[nq]; \
                    B = c0 < 256u ? 1u : (k->offsets[c0 - 255u] - k->offsets[c0 - 256u]); \
                    J = st.saved_next[nq]; \
                } else { Q = MAX_CUTS; B = 0; J = m; p1_act--; } \
            } else { uint32_t w = cd - 256u; B = (B - wl) + (k->offsets[w + 1] - k->offsets[w]); } \
        } \
    } while (0)

    if (!wide) {
        while (p1_act > 0) {
            uint32_t e0=edges[j0], e1=edges[j1], e2=edges[j2], e3=edges[j3];
            uint32_t e4=edges[j4], e5=edges[j5], e6=edges[j6], e7=edges[j7];
            L_P1_STEP(j0,q0,b0,e0>>bits,e0); L_P1_STEP(j1,q1,b1,e1>>bits,e1);
            L_P1_STEP(j2,q2,b2,e2>>bits,e2); L_P1_STEP(j3,q3,b3,e3>>bits,e3);
            L_P1_STEP(j4,q4,b4,e4>>bits,e4); L_P1_STEP(j5,q5,b5,e5>>bits,e5);
            L_P1_STEP(j6,q6,b6,e6>>bits,e6); L_P1_STEP(j7,q7,b7,e7>>bits,e7);
        }
    } else {
        while (p1_act > 0) {
            uint32_t e0=edges[j0],c0=extra[j0],e1=edges[j1],c1=extra[j1];
            uint32_t e2=edges[j2],c2=extra[j2],e3=edges[j3],c3=extra[j3];
            uint32_t e4=edges[j4],c4=extra[j4],e5=edges[j5],c5=extra[j5];
            uint32_t e6=edges[j6],c6=extra[j6],e7=edges[j7],c7=extra[j7];
            L_P1_STEP(j0,q0,b0,c0,e0); L_P1_STEP(j1,q1,b1,c1,e1);
            L_P1_STEP(j2,q2,b2,c2,e2); L_P1_STEP(j3,q3,b3,c3,e3);
            L_P1_STEP(j4,q4,b4,c4,e4); L_P1_STEP(j5,q5,b5,c5,e5);
            L_P1_STEP(j6,q6,b6,c6,e6); L_P1_STEP(j7,q7,b7,c7,e7);
        }
    }

    st.n_cyc = 0; size_t acc = 0; int32_t cs = 0;
    while (st.n_cyc < S_eff && acc < n) {
        size_t end = acc + seg_bytes[cs]; if (end > n) end = n;
        st.cyc_segs[st.n_cyc] = (uint32_t)cs;
        st.cyc_off[st.n_cyc] = (uint32_t)acc; st.cyc_end[st.n_cyc] = (uint32_t)end;
        st.n_cyc++; acc = end; cs = succ[cs];
        if (cs == 0) break;
    }
    for (uint32_t q = 0; q < S_eff; q++) w_len[sbase + q] = 65535u;

    st.next_idx = 0; st.active = 0;
    uint8_t *p_init2[8];
    for (int l = 0; l < 8; l++) {
        if (st.next_idx < st.n_cyc) {
            uint32_t idx = st.next_idx++, sg = st.cyc_segs[idx];
            uint32_t off = st.cyc_off[idx], end = st.cyc_end[idx];
            st.active++; st.active_flag[l] = 1;
            p_init2[l] = out + off; st.hard_end[l] = out + end;
            st.safe_end[l] = (end - off >= 16u) ? (out + end - 15u) : (out + off);
            l_emit_bounded(k, &st, l, &p_init2[l], st.saved_code[sg]);
            j_init[l] = st.saved_next[sg];
        } else {
            st.active_flag[l] = 0; p_init2[l] = k->dummy_buf;
            st.safe_end[l] = k->dummy_buf + 2048; st.hard_end[l] = k->dummy_buf + 4096;
            j_init[l] = m;
        }
    }
    j0=j_init[0]; j1=j_init[1]; j2=j_init[2]; j3=j_init[3];
    j4=j_init[4]; j5=j_init[5]; j6=j_init[6]; j7=j_init[7];
    uint8_t * restrict p0=p_init2[0], * restrict p1=p_init2[1], * restrict p2=p_init2[2], * restrict p3=p_init2[3];
    uint8_t * restrict p4=p_init2[4], * restrict p5=p_init2[5], * restrict p6=p_init2[6], * restrict p7=p_init2[7];

    #define L_STEP_2P(L, J, P, CD_EXPR, E_VAR) do { \
        uint32_t cd = (CD_EXPR); J = (E_VAR) & mask; uint32_t ln = w_len[cd]; \
        if (__builtin_expect(ln <= 16u && P < st.safe_end[L], 1)) { \
            _mm_storeu_si128((__m128i *)P, inline16[cd]); P += ln; \
        } else { uint8_t *ptmp = P; l_cold_2p(k, &st, L, &(J), &ptmp, cd); P = ptmp; } \
    } while (0)

    if (!wide) {
        while (st.active > 0) {
            uint32_t e0=edges[j0], e1=edges[j1], e2=edges[j2], e3=edges[j3];
            uint32_t e4=edges[j4], e5=edges[j5], e6=edges[j6], e7=edges[j7];
            L_STEP_2P(0,j0,p0,e0>>bits,e0); L_STEP_2P(1,j1,p1,e1>>bits,e1);
            L_STEP_2P(2,j2,p2,e2>>bits,e2); L_STEP_2P(3,j3,p3,e3>>bits,e3);
            L_STEP_2P(4,j4,p4,e4>>bits,e4); L_STEP_2P(5,j5,p5,e5>>bits,e5);
            L_STEP_2P(6,j6,p6,e6>>bits,e6); L_STEP_2P(7,j7,p7,e7>>bits,e7);
        }
    } else {
        while (st.active > 0) {
            uint32_t e0=edges[j0],c0=extra[j0],e1=edges[j1],c1=extra[j1];
            uint32_t e2=edges[j2],c2=extra[j2],e3=edges[j3],c3=extra[j3];
            uint32_t e4=edges[j4],c4=extra[j4],e5=edges[j5],c5=extra[j5];
            uint32_t e6=edges[j6],c6=extra[j6],e7=edges[j7],c7=extra[j7];
            L_STEP_2P(0,j0,p0,c0,e0); L_STEP_2P(1,j1,p1,c1,e1);
            L_STEP_2P(2,j2,p2,c2,e2); L_STEP_2P(3,j3,p3,c3,e3);
            L_STEP_2P(4,j4,p4,c4,e4); L_STEP_2P(5,j5,p5,c5,e5);
            L_STEP_2P(6,j6,p6,c6,e6); L_STEP_2P(7,j7,p7,c7,e7);
        }
    }
    for (uint32_t q = 0; q < S_eff; q++) {
        uint32_t v = cuts[q]; edges[v] = saved_raw[q];
        if (wide) extra[v] = st.saved_code[q];
    }
    repeat_period(n, out, &acc);
}

/* =========================================================================
 * Cycle 8 Candidate: Conjugate F-Space Bandless Schur Elimination (CFSE)
 * ========================================================================= */
static void cfse_init(CfseKernel *k, size_t max_n) {
    memset(k, 0, sizeof(*k));
    k->strips_cap = 8192;
    k->strips = (Strip *)malloc(k->strips_cap * sizeof(Strip));
    k->img_cap = 8192;
    k->image_order = (uint32_t *)malloc(k->img_cap * 4u);
    k->nodes_cap = 32768;
    k->nodes = (EpNode *)malloc(k->nodes_cap * sizeof(EpNode));
    k->rules_cap = 2048;
    k->rules = (EpRule *)malloc(k->rules_cap * sizeof(EpRule));
    k->bottom_cap = 32768;
    k->bottom = (uint32_t *)malloc(k->bottom_cap * 4u);
    k->stack_cap = 2048;
    k->stack = (uint32_t *)malloc(k->stack_cap * 4u);

    k->arena_cap = 262144;
    k->small_arena = (uint8_t *)malloc(k->arena_cap + 128);
    memset(k->small_arena, 0, 320);
    /* Seed literals at offsets 32..288 so every woff >= 32 has >=16 readable bytes before woff + wl - 15! */
    for (uint32_t c = 0; c < 256; c++) k->small_arena[32u + c] = (uint8_t)c;
    k->arena_len = 288u;

    k->spell_cap = 65536;
    k->long_spell = (uint8_t *)malloc(k->spell_cap + 128);

    k->dict_cap = 16384;
    k->inline16_ra = (__m128i *)aligned_alloc(64, k->dict_cap * sizeof(__m128i));
    k->w_len = (uint16_t *)calloc(k->dict_cap, sizeof(uint16_t));
    memset(k->inline16_ra, 0, 256 * sizeof(__m128i));
    for (uint32_t c = 0; c < 256; c++) {
        ((uint8_t *)&k->inline16_ra[c])[15] = (uint8_t)c;
        k->w_len[c] = 1u;
    }

    k->edges_cap = max_n + 16;
    k->edges = (uint32_t *)malloc(k->edges_cap * 4u);
    k->extra_cap = max_n + 16;
    k->extra = (uint32_t *)malloc(k->extra_cap * 4u);
    k->dummy_buf = (uint8_t *)malloc(4096);
}

static void cfse_free(CfseKernel *k) {
    free(k->strips); free(k->image_order); free(k->nodes); free(k->rules);
    free(k->bottom); free(k->stack); free(k->small_arena); free(k->long_spell);
    free(k->inline16_ra); free(k->w_len); free(k->edges); free(k->extra);
    free(k->alpha_fallback); free(k->dummy_buf);
}

static size_t cfse_scratch_bytes(const CfseKernel *k, size_t m_used) {
    size_t fb = k->used_out_scratch ? 0u : (k->alpha_fb_cap * 4u);
    return fb
         + k->strips_cap * sizeof(Strip)
         + k->img_cap * 4u
         + k->nodes_cap * sizeof(EpNode)
         + k->rules_len * sizeof(EpRule)
         + k->bottom_cap * 4u
         + k->arena_len
         + k->spell_len
         + (size_t)(256u + k->words) * (sizeof(__m128i) + sizeof(uint16_t))
         + m_used * 4u;
}

static inline void cfse_dict_ensure(CfseKernel *k, size_t need) {
    if (__builtin_expect(need > k->dict_cap, 0)) {
        size_t old = k->dict_cap;
        while (need > k->dict_cap) k->dict_cap *= 2;
        __m128i *n16 = (__m128i *)aligned_alloc(64, k->dict_cap * sizeof(__m128i));
        memcpy(n16, k->inline16_ra, old * sizeof(__m128i));
        free(k->inline16_ra);
        k->inline16_ra = n16;
        k->w_len = (uint16_t *)realloc(k->w_len, k->dict_cap * sizeof(uint16_t));
    }
}

static inline void cfse_spell_ensure(CfseKernel *k, size_t add) {
    if (__builtin_expect(k->spell_len + add + 80 > k->spell_cap, 0)) {
        while (k->spell_len + add + 80 > k->spell_cap) k->spell_cap *= 2;
        k->long_spell = (uint8_t *)realloc(k->long_spell, k->spell_cap + 128);
    }
}

static inline void cfse_push_strip(CfseKernel *k, uint32_t from, uint32_t to, uint32_t width, uint8_t byte) {
    if (__builtin_expect(k->strips_len == k->strips_cap, 0)) {
        k->strips_cap *= 2;
        k->strips = (Strip *)realloc(k->strips, k->strips_cap * sizeof(Strip));
    }
    k->strips[k->strips_len++] = (Strip){from, to, width, byte};
}

static inline uint32_t cfse_len(const CfseKernel *k, uint32_t w) {
    if (w == 0) return 0u;
    if ((w & 0x80000000u) == 0) return w >> 24;
    return k->rules[w & 0x7FFFFFFFu].len;
}

static inline uint32_t cfse_join(CfseKernel *k, uint32_t aw, uint32_t bw, int use_simd_arena) {
    if (aw == 0) return bw;
    if (bw == 0) return aw;
    if (__builtin_expect(((aw | bw) & 0x80000000u) == 0, 1)) {
        uint32_t al = aw >> 24, bl = bw >> 24, tl = al + bl;
        if (__builtin_expect(tl <= GIM_CAP && k->arena_len + tl < 0x00FFF000u, 1)) {
            if (__builtin_expect(k->arena_len + tl + 64u > k->arena_cap, 0)) {
                while (k->arena_len + tl + 64u > k->arena_cap) k->arena_cap *= 2;
                k->small_arena = (uint8_t *)realloc(k->small_arena, k->arena_cap + 128);
            }
            uint32_t dst_off = k->arena_len;
            const uint8_t *sa = k->small_arena + (aw & 0x00FFFFFFu);
            const uint8_t *sb = k->small_arena + (bw & 0x00FFFFFFu);
            uint8_t *dst = k->small_arena + dst_off;
            if (use_simd_arena) {
                if (__builtin_expect(tl <= 16u, 1)) {
                    _mm_storeu_si128((__m128i *)dst, _mm_loadu_si128((const __m128i *)sa));
                    _mm_storeu_si128((__m128i *)(dst + al), _mm_loadu_si128((const __m128i *)sb));
                } else {
                    __m256i va0 = _mm256_loadu_si256((const __m256i *)sa);
                    __m256i va1 = al > 32u ? _mm256_loadu_si256((const __m256i *)(sa + 32)) : _mm256_setzero_si256();
                    __m256i vb0 = _mm256_loadu_si256((const __m256i *)sb);
                    __m256i vb1 = bl > 32u ? _mm256_loadu_si256((const __m256i *)(sb + 32)) : _mm256_setzero_si256();
                    _mm256_storeu_si256((__m256i *)dst, va0);
                    if (al > 32u) _mm256_storeu_si256((__m256i *)(dst + 32), va1);
                    _mm256_storeu_si256((__m256i *)(dst + al), vb0);
                    if (bl > 32u) _mm256_storeu_si256((__m256i *)(dst + al + 32), vb1);
                }
            } else {
                memcpy(dst, sa, al);
                memcpy(dst + al, sb, bl);
            }
            k->arena_len = dst_off + tl;
            return (tl << 24) | dst_off;
        }
    }
    if (__builtin_expect(k->rules_len == k->rules_cap, 0)) {
        k->rules_cap *= 2;
        k->rules = (EpRule *)realloc(k->rules, k->rules_cap * sizeof(EpRule));
    }
    uint32_t rid = (uint32_t)k->rules_len++;
    uint32_t ln = cfse_len(k, aw) + cfse_len(k, bw);
    k->rules[rid] = (EpRule){aw, bw, ln, UINT32_MAX};
    return 0x80000000u | rid;
}

__attribute__((noinline))
static void cfse_append_rule(CfseKernel *k, uint32_t word) {
    k->stack_len = 0;
    k->stack[k->stack_len++] = word;
    while (k->stack_len > 0) {
        uint32_t w = k->stack[--k->stack_len];
        if ((w & 0x80000000u) == 0) {
            uint32_t wl = w >> 24, woff = w & 0x00FFFFFFu;
            cfse_spell_ensure(k, wl);
            memcpy(k->long_spell + k->spell_len, k->small_arena + woff, wl);
            k->spell_len += wl;
        } else {
            EpRule *rl = &k->rules[w & 0x7FFFFFFFu];
            if (rl->first == UINT32_MAX) {
                rl->first = (uint32_t)k->spell_len;
                if (k->stack_len + 2 > k->stack_cap) {
                    k->stack_cap *= 2;
                    k->stack = (uint32_t *)realloc(k->stack, k->stack_cap * 4u);
                }
                k->stack[k->stack_len++] = rl->right;
                k->stack[k->stack_len++] = rl->left;
            } else {
                cfse_spell_ensure(k, rl->len);
                memmove(k->long_spell + k->spell_len, k->long_spell + rl->first, rl->len);
                k->spell_len += rl->len;
            }
        }
    }
}

__attribute__((noinline))
static uint32_t *cfse_migrate_alpha(CfseKernel *k, const uint32_t *cur_alpha, uint32_t m_cur, uint32_t n) {
    if ((size_t)n + 32u > k->alpha_fb_cap) {
        k->alpha_fb_cap = (size_t)n + 32u;
        k->alpha_fallback = (uint32_t *)realloc(k->alpha_fallback, k->alpha_fb_cap * 4u);
    }
    if (m_cur > 0 && cur_alpha != k->alpha_fallback) {
        memcpy(k->alpha_fallback, cur_alpha, (size_t)m_cur * 4u);
    }
    k->used_out_scratch = 0;
    return k->alpha_fallback;
}

static void cfse_construct(
    CfseKernel *k,
    const uint16_t *records,
    size_t n_recs,
    uint32_t n,
    uint32_t p,
    uint8_t *out_scratch,
    int use_simd,
    uint32_t reserve_sentinels
) {
    uint32_t all_c[256] = {0}, thin[256] = {0}, thick[256] = {0};
    k->strips_len = 0;

    uintptr_t out_addr = (uintptr_t)out_scratch;
    uintptr_t aligned_addr = (out_addr + 3u) & ~(uintptr_t)3u;
    size_t pad_bytes = aligned_addr - out_addr;
    size_t out_u32_cap = (n > pad_bytes + 64u) ? ((n - pad_bytes) / 4u) : 0u;

    uint32_t * restrict alpha_raw;
    uint32_t safe_m_limit;
    if (out_u32_cap >= 32u) {
        alpha_raw = (uint32_t *)aligned_addr;
        safe_m_limit = (uint32_t)(out_u32_cap - 16u);
        k->used_out_scratch = 1;
    } else {
        alpha_raw = cfse_migrate_alpha(k, NULL, 0, n);
        safe_m_limit = UINT32_MAX;
    }

    uint32_t m = 0, mark_l = 0, pos = 0;
    size_t i = 0;
    if (use_simd) {
        while (i < n_recs) {
            uint16_t rec = records[i++];
            uint8_t c = (uint8_t)(rec >> 8);
            uint32_t width = (uint32_t)(rec & 255u);
            while (i < n_recs && (uint8_t)(records[i] >> 8) == c) width += (uint32_t)(records[i++] & 255u);
            uint32_t end = pos + width;
            if (__builtin_expect(width <= LIMIT, 1)) {
                if (__builtin_expect(p >= pos && p < end, 0)) mark_l = m + (p - pos);
                if (__builtin_expect(m > safe_m_limit, 0)) {
                    alpha_raw = cfse_migrate_alpha(k, alpha_raw, m, n);
                    safe_m_limit = UINT32_MAX;
                }
                uint32_t tc = thin[c];
                store_seq16(alpha_raw + m, ((uint32_t)c << 20) | tc);
                thin[c] = tc + width; m += width; all_c[c] += width;
            } else if (__builtin_expect(p >= pos && p < end, 0)) {
                if (pos < p) { cfse_push_strip(k, pos, all_c[c], p - pos, c); thick[c]++; all_c[c] += p - pos; }
                mark_l = m;
                if (__builtin_expect(m > safe_m_limit, 0)) {
                    alpha_raw = cfse_migrate_alpha(k, alpha_raw, m, n);
                    safe_m_limit = UINT32_MAX;
                }
                alpha_raw[m] = ((uint32_t)c << 20) | thin[c];
                thin[c]++; m++; all_c[c]++;
                if (p + 1 < end) { uint32_t w2 = end - p - 1; cfse_push_strip(k, p + 1, all_c[c], w2, c); thick[c]++; all_c[c] += w2; }
            } else {
                cfse_push_strip(k, pos, all_c[c], width, c); thick[c]++; all_c[c] += width;
            }
            pos = end;
        }
    } else {
        /* Pure non-SIMD scalar RunScan for Arm 1 */
        while (i < n_recs) {
            uint16_t rec = records[i++];
            uint8_t c = (uint8_t)(rec >> 8);
            uint32_t width = (uint32_t)(rec & 255u);
            while (i < n_recs && (uint8_t)(records[i] >> 8) == c) width += (uint32_t)(records[i++] & 255u);
            uint32_t end = pos + width;
            if (__builtin_expect(width <= LIMIT, 1)) {
                if (__builtin_expect(p >= pos && p < end, 0)) mark_l = m + (p - pos);
                if (__builtin_expect(m > safe_m_limit, 0)) {
                    alpha_raw = cfse_migrate_alpha(k, alpha_raw, m, n);
                    safe_m_limit = UINT32_MAX;
                }
                uint32_t base_v = ((uint32_t)c << 20) | thin[c];
                for (uint32_t s = 0; s < width; s++) alpha_raw[m + s] = base_v + s;
                thin[c] += width; m += width; all_c[c] += width;
            } else if (__builtin_expect(p >= pos && p < end, 0)) {
                if (pos < p) { cfse_push_strip(k, pos, all_c[c], p - pos, c); thick[c]++; all_c[c] += p - pos; }
                mark_l = m;
                if (__builtin_expect(m > safe_m_limit, 0)) {
                    alpha_raw = cfse_migrate_alpha(k, alpha_raw, m, n);
                    safe_m_limit = UINT32_MAX;
                }
                alpha_raw[m] = ((uint32_t)c << 20) | thin[c];
                thin[c]++; m++; all_c[c]++;
                if (p + 1 < end) { uint32_t w2 = end - p - 1; cfse_push_strip(k, p + 1, all_c[c], w2, c); thick[c]++; all_c[c] += w2; }
            } else {
                cfse_push_strip(k, pos, all_c[c], width, c); thick[c]++; all_c[c] += width;
            }
            pos = end;
        }
    }

    uint32_t first[256], write[256], thin_start[256];
    uint32_t tot = 0, tw = 0, ts_acc = 0;
    for (int a = 0; a < 256; a++) {
        first[a] = tot; tot += all_c[a];
        write[a] = tw; tw += thick[a];
        thin_start[a] = ts_acc; ts_acc += thin[a];
    }

    uint32_t mv = alpha_raw[mark_l];
    uint32_t mark_f = thin_start[mv >> 20] + (mv & 0xFFFFFu);

    if (k->strips_len > k->img_cap) {
        k->img_cap = k->strips_len * 2;
        k->image_order = (uint32_t *)realloc(k->image_order, k->img_cap * 4u);
    }
    for (size_t si = 0; si < k->strips_len; si++) {
        uint8_t a = k->strips[si].byte;
        k->strips[si].to += first[a];
        k->image_order[write[a]++] = (uint32_t)si;
    }

    size_t max_nodes = 3 * k->strips_len + 8;
    if (max_nodes > k->nodes_cap) {
        while (max_nodes > k->nodes_cap) k->nodes_cap *= 2;
        k->nodes = (EpNode *)realloc(k->nodes, k->nodes_cap * sizeof(EpNode));
    }
    EpNode * restrict nodes = k->nodes;
    nodes[0] = (EpNode){{0, 0}, {0, 0}, 0, 0};
    uint32_t n_nodes = 1u;
    k->rules_len = 0;
    k->arena_len = 288u;

    size_t n_strips = k->strips_len;
    for (size_t si = 0; si < n_strips; si++) {
        nodes[n_nodes++] = (EpNode){{0, 0}, {0, 0}, k->strips[si].width, (1u << 24) | (32u + (uint32_t)k->strips[si].byte)};
    }

    uint32_t th0 = 0, tt0 = 0, oh0 = 0, ot0 = 0;
    uint32_t th1 = 0, tt1 = 0, oh1 = 0, ot1 = 0;
    uint32_t x = 0;
    for (size_t si = 0; si < n_strips; si++) {
        uint32_t sf = k->strips[si].from;
        if (x < sf) {
            uint32_t gid = n_nodes++;
            nodes[gid] = (EpNode){{0, 0}, {0, 0}, sf - x, 0};
            LINK_CHAIN(nodes, 0, oh0, ot0, gid); LINK_CHAIN(nodes, 1, th1, tt1, gid);
        }
        LINK_CHAIN(nodes, 0, oh0, ot0, (uint32_t)si + 1u);
        x = sf + k->strips[si].width;
    }
    if (x < n) {
        uint32_t gid = n_nodes++;
        nodes[gid] = (EpNode){{0, 0}, {0, 0}, n - x, 0};
        LINK_CHAIN(nodes, 0, oh0, ot0, gid); LINK_CHAIN(nodes, 1, th1, tt1, gid);
    }

    x = 0;
    for (size_t ii = 0; ii < n_strips; ii++) {
        uint32_t idx = k->image_order[ii]; Strip s = k->strips[idx];
        if (x < s.to) {
            uint32_t gid = n_nodes++;
            nodes[gid] = (EpNode){{0, 0}, {0, 0}, s.to - x, 0};
            LINK_CHAIN(nodes, 1, oh1, ot1, gid); LINK_CHAIN(nodes, 0, th0, tt0, gid);
        }
        LINK_CHAIN(nodes, 1, oh1, ot1, idx + 1u);
        x = s.to + s.width;
    }
    if (x < n) {
        uint32_t gid = n_nodes++;
        nodes[gid] = (EpNode){{0, 0}, {0, 0}, n - x, 0};
        LINK_CHAIN(nodes, 1, oh1, ot1, gid); LINK_CHAIN(nodes, 0, th0, tt0, gid);
    }
    k->nodes_len = n_nodes;

    {
        uint32_t f0, l0;
        if (th0 && oh0) { nodes[tt0].next[0] = oh0; nodes[oh0].prev[0] = tt0; f0 = th0; l0 = ot0; }
        else if (th0)   { f0 = th0; l0 = tt0; } else { f0 = oh0; l0 = ot0; }
        nodes[0].next[0] = f0; nodes[f0].prev[0] = 0; nodes[l0].next[0] = 0; nodes[0].prev[0] = l0;

        uint32_t f1, l1;
        if (th1 && oh1) { nodes[tt1].next[1] = oh1; nodes[oh1].prev[1] = tt1; f1 = th1; l1 = ot1; }
        else if (th1)   { f1 = th1; l1 = tt1; } else { f1 = oh1; l1 = ot1; }
        nodes[0].next[1] = f1; nodes[f1].prev[1] = 0; nodes[l1].next[1] = 0; nodes[0].prev[1] = l1;
    }

    uint32_t remaining = n;
    while (remaining > 0) {
        uint32_t xid = nodes[0].prev[0], yid = nodes[0].prev[1];
        EpNode a = nodes[xid], b = nodes[yid];
        uint32_t w = a.width < b.width ? a.width : b.width;
        remaining -= w;
        if (xid == yid) { ep_cut(nodes, xid, 0); ep_cut(nodes, xid, 1); continue; }
        uint32_t wd = cfse_join(k, a.word, b.word, use_simd);
        if (a.width <= b.width) {
            ep_cut(nodes, xid, 0); ep_insert(nodes, yid, xid, 0); nodes[xid].word = wd;
            if (a.width == b.width) { ep_cut(nodes, yid, 0); ep_cut(nodes, yid, 1); }
            else nodes[yid].width -= w;
        } else {
            ep_cut(nodes, yid, 1); ep_insert(nodes, xid, yid, 1); nodes[yid].word = wd;
            nodes[xid].width -= w;
        }
    }

    if (n_nodes > k->bottom_cap) {
        k->bottom_cap = n_nodes * 2;
        k->bottom = (uint32_t *)realloc(k->bottom, k->bottom_cap * 4u);
    }
    uint32_t * restrict bottom = k->bottom;
    uint32_t id = nodes[0].next[1]; x = 0;
    uint32_t n_nz_surv = 0;
    while (id != 0) {
        bottom[id] = x;
        EpNode nd = nodes[id];
        x += nd.width;
        n_nz_surv += (nd.word != 0);
        id = nd.next[1];
    }

    uint32_t bits = m <= 1 ? 1u : (32u - (uint32_t)__builtin_clz(m - 1));
    uint32_t max_words_ub = n_nz_surv + 256u;
    uint32_t max_code_ub = 255u + max_words_ub + reserve_sentinels;
    int wide = max_code_ub > (UINT32_MAX >> bits);
    cfse_dict_ensure(k, (size_t)max_code_ub + 16u);

    k->spell_len = 0;
    id = nodes[0].next[0];
    uint32_t in_node = 0, kc = 0, words = 0;
    const uint8_t * restrict arena = k->small_arena;
    uint32_t * restrict edges_f = k->edges;
    uint32_t * restrict extra_f = k->extra;

    for (int a = 0; a < 256; a++) {
        uint32_t end = kc + thin[a];
        while (kc < end) {
            EpNode nd = nodes[id];
            uint32_t rem_a = end - kc, rem_n = nd.width - in_node;
            uint32_t width = rem_a < rem_n ? rem_a : rem_n;
            uint32_t dst = bottom[id] + in_node;
            uint32_t code;
            if (nd.word == 0) {
                code = (uint32_t)a;
            } else {
                uint32_t w_idx = words++;
                code = 256u + w_idx;
                if (__builtin_expect((nd.word & 0x80000000u) == 0 && (nd.word >> 24) <= 15u, 1)) {
                    uint32_t wl = nd.word >> 24;
                    uint32_t ln = wl + 1u;
                    const uint8_t *wsrc = arena + (nd.word & 0x00FFFFFFu);
                    if (use_simd) {
                        __m128i vra = _mm_loadu_si128((const __m128i *)(wsrc + wl - 15u));
                        k->inline16_ra[code] = _mm_insert_epi8(vra, (int)a, 15);
                    } else {
                        uint8_t * restrict slot = (uint8_t *)&k->inline16_ra[code];
                        memcpy(slot + (16u - ln), wsrc, wl);
                        slot[15] = (uint8_t)a;
                    }
                    k->w_len[code] = (uint16_t)ln;
                } else {
                    uint32_t o0 = (uint32_t)k->spell_len;
                    if ((nd.word & 0x80000000u) == 0) {
                        uint32_t wl = nd.word >> 24;
                        const uint8_t *wsrc = arena + (nd.word & 0x00FFFFFFu);
                        cfse_spell_ensure(k, wl + 1u);
                        uint8_t * restrict sp = k->long_spell + o0;
                        if (use_simd) {
                            _mm256_storeu_si256((__m256i *)sp, _mm256_loadu_si256((const __m256i *)wsrc));
                            if (wl > 32u) _mm256_storeu_si256((__m256i *)(sp + 32), _mm256_loadu_si256((const __m256i *)(wsrc + 32)));
                        } else {
                            memcpy(sp, wsrc, wl);
                        }
                        sp[wl] = (uint8_t)a;
                        k->spell_len = o0 + wl + 1u;
                    } else {
                        cfse_append_rule(k, nd.word);
                        cfse_spell_ensure(k, 1u);
                        k->long_spell[k->spell_len++] = (uint8_t)a;
                    }
                    uint32_t ln = (uint32_t)k->spell_len - o0;
                    uint64_t meta[2] = {(uint64_t)o0, (uint64_t)ln};
                    memcpy(&k->inline16_ra[code], meta, 16);
                    k->w_len[code] = ln <= 65533u ? (uint16_t)ln : 65534u;
                }
            }

            uint32_t pack = wide ? 0u : (code << bits);
            if (__builtin_expect(wide, 0)) {
                for (uint32_t s = 0; s < width; s++) extra_f[kc + s] = code;
            }
            uint32_t * restrict edst = edges_f + kc;
            const uint32_t * restrict asrc = alpha_raw + dst;
            uint32_t s = 0;
            for (; s + 4u <= width; s += 4u) {
                uint32_t v0 = asrc[s], v1 = asrc[s + 1], v2 = asrc[s + 2], v3 = asrc[s + 3];
                edst[s]     = pack | (thin_start[v0 >> 20] + (v0 & 0xFFFFFu));
                edst[s + 1] = pack | (thin_start[v1 >> 20] + (v1 & 0xFFFFFu));
                edst[s + 2] = pack | (thin_start[v2 >> 20] + (v2 & 0xFFFFFu));
                edst[s + 3] = pack | (thin_start[v3 >> 20] + (v3 & 0xFFFFFu));
            }
            for (; s < width; s++) {
                uint32_t v = asrc[s];
                edst[s] = pack | (thin_start[v >> 20] + (v & 0xFFFFFu));
            }

            in_node += width; kc += width;
            if (in_node == nd.width) { id = nd.next[0]; in_node = 0; }
        }
    }

    k->m = m; k->mark_f = mark_f; k->bits = bits; k->words = words; k->wide = wide;
}

static inline void cfse_get_long(const CfseKernel *k, uint32_t code, const uint8_t **src_out, uint32_t *len_out) {
    uint64_t meta[2];
    memcpy(meta, &k->inline16_ra[code], 16);
    *src_out = k->long_spell + (uint32_t)meta[0];
    *len_out = (uint32_t)meta[1];
}

/* Arm 1 Consumer: Pure Scalar Non-SIMD 1-Lane Right-to-Left Emission */
static void cfse_emit_scalar_1lane(const CfseKernel *k, size_t n, uint8_t *out) {
    const uint32_t * restrict edges = k->edges;
    const uint32_t * restrict extra = k->extra;
    const __m128i * restrict inline16_ra = k->inline16_ra;
    const uint16_t * restrict w_len = k->w_len;
    const uint32_t m = k->m, mark = k->mark_f, bits = k->bits, mask = (1u << bits) - 1u;
    const int wide = k->wide;

    uint8_t * restrict P = out + n;
    uint32_t j = mark;

    for (uint32_t step = 0; step < m; step++) {
        uint32_t edge = edges[j];
        uint32_t code = wide ? extra[j] : (edge >> bits);
        j = edge & mask;
        if (code < 256u) {
            *--P = (uint8_t)code;
        } else {
            uint32_t ln = w_len[code];
            const uint8_t *src;
            uint32_t real_ln;
            if (ln <= 16u) {
                src = ((const uint8_t *)&inline16_ra[code]) + (16u - ln);
                real_ln = ln;
            } else {
                cfse_get_long(k, code, &src, &real_ln);
            }
            size_t rem = (size_t)(P - out);
            if (__builtin_expect(real_ln > rem, 0)) {
                src += (real_ln - (uint32_t)rem);
                real_ln = (uint32_t)rem;
            }
            P -= real_ln;
            memcpy(P, src, real_ln);
        }
        if (__builtin_expect(j == mark || P <= out, 0)) break;
    }
    size_t clen = (size_t)((out + n) - P);
    if (__builtin_expect(P > out, 0)) {
        memmove(out, P, clen);
        repeat_period(n, out, &clen);
    }
}

/* Arm 3 Consumer: 1-Lane Right-to-Left SIMD Emission */
static void cfse_emit_simd_1lane(const CfseKernel *k, size_t n, uint8_t *out) {
    const uint32_t * restrict edges = k->edges;
    const uint32_t * restrict extra = k->extra;
    const __m128i * restrict inline16_ra = k->inline16_ra;
    const uint16_t * restrict w_len = k->w_len;
    const uint32_t m = k->m, mark = k->mark_f, bits = k->bits, mask = (1u << bits) - 1u;
    const int wide = k->wide;

    uint8_t * restrict P = out + n;
    uint8_t * const safe_start = n >= 16u ? (out + 16u) : (out + n + 1u);
    uint32_t j = mark;

    for (uint32_t step = 0; step < m; step++) {
        uint32_t edge = edges[j];
        uint32_t code = wide ? extra[j] : (edge >> bits);
        j = edge & mask;
        uint32_t ln = w_len[code];
        if (__builtin_expect(ln <= 16u && P >= safe_start, 1)) {
            _mm_storeu_si128((__m128i *)(P - 16), inline16_ra[code]);
            P -= ln;
        } else {
            const uint8_t *src;
            uint32_t real_ln;
            uint8_t tmp16[16];
            if (ln <= 16u) {
                _mm_storeu_si128((__m128i *)tmp16, inline16_ra[code]);
                src = tmp16 + (16u - ln);
                real_ln = ln;
            } else {
                cfse_get_long(k, code, &src, &real_ln);
            }
            size_t rem = (size_t)(P - out);
            if (real_ln > rem) {
                src += (real_ln - (uint32_t)rem);
                real_ln = (uint32_t)rem;
            }
            P -= real_ln;
            memcpy(P, src, real_ln);
        }
        if (__builtin_expect(j == mark || P <= out, 0)) break;
    }
    size_t clen = (size_t)((out + n) - P);
    if (__builtin_expect(P > out, 0)) {
        memmove(out, P, clen);
        repeat_period(n, out, &clen);
    }
}

/* Arm 5 Consumer: 8-Lane Zero-Arena Right-to-Left F-Space BVSMC-2P */
typedef struct {
    uint8_t *safe_start[8], *hard_start[8];
    int active_flag[8];
    uint32_t cyc_segs[MAX_CUTS], cyc_off[MAX_CUTS], cyc_end[MAX_CUTS];
    uint32_t saved_code[MAX_CUTS], saved_next[MAX_CUTS];
    uint32_t n_cyc, next_idx, active, sbase;
    uint8_t *out;
} CfseState2P;

static inline uint32_t cfse_true_len(const CfseKernel *k, uint32_t cd) {
    uint32_t wl = k->w_len[cd];
    if (__builtin_expect(wl < 65534u, 1)) return wl;
    const uint8_t *s; uint32_t rl;
    cfse_get_long(k, cd, &s, &rl);
    return rl;
}

static inline void cfse_emit_bounded(CfseKernel *k, CfseState2P *st, int l, uint8_t **P_io, uint32_t cd) {
    uint8_t *P = *P_io, *hs = st->hard_start[l];
    if (P <= hs) return;
    uint32_t ln = k->w_len[cd];
    if (ln <= 16u && P >= st->safe_start[l]) {
        _mm_storeu_si128((__m128i *)(P - 16), k->inline16_ra[cd]);
        *P_io = P - ln;
        return;
    }
    const uint8_t *src; uint32_t real_ln; uint8_t tmp16[16];
    if (ln <= 16u) {
        _mm_storeu_si128((__m128i *)tmp16, k->inline16_ra[cd]);
        src = tmp16 + (16u - ln); real_ln = ln;
    } else {
        cfse_get_long(k, cd, &src, &real_ln);
    }
    uint32_t av = (uint32_t)(P - hs);
    if (real_ln > av) { src += (real_ln - av); real_ln = av; }
    P -= real_ln;
    memcpy(P, src, real_ln);
    *P_io = P;
}

__attribute__((noinline))
static void cfse_cold_2p(CfseKernel *k, CfseState2P *st, int l, uint32_t *J_io, uint8_t **P_io, uint32_t cd) {
    if (!st->active_flag[l]) { *P_io = k->dummy_buf + 2048; return; }
    if (cd < st->sbase) {
        cfse_emit_bounded(k, st, l, P_io, cd);
    } else if (st->next_idx < st->n_cyc) {
        uint32_t idx = st->next_idx++, sg = st->cyc_segs[idx];
        uint32_t off = st->cyc_off[idx], end = st->cyc_end[idx];
        uint8_t *P = st->out + end;
        st->hard_start[l] = st->out + off;
        st->safe_start[l] = (end - off >= 16u) ? (st->out + off + 16u) : (st->out + end + 1u);
        cfse_emit_bounded(k, st, l, &P, st->saved_code[sg]);
        *P_io = P; *J_io = st->saved_next[sg];
    } else {
        st->active_flag[l] = 0; *P_io = k->dummy_buf + 2048;
        st->hard_start[l] = k->dummy_buf; st->safe_start[l] = k->dummy_buf + 4096;
        *J_io = k->m; st->active--;
    }
}

static void cfse_bvsmc_emit_2p(CfseKernel *k, size_t n, uint32_t S_req, uint8_t *out) {
    uint32_t * restrict edges = k->edges;
    uint32_t * restrict extra = k->extra;
    const __m128i * restrict inline16_ra = k->inline16_ra;
    uint16_t * restrict w_len = k->w_len;
    const uint32_t m = k->m, mark = k->mark_f, bits = k->bits, mask = (1u << bits) - 1u;
    const int wide = k->wide;
    const uint32_t sbase = 256u + k->words;
    uint32_t S_eff = S_req < m ? S_req : m;
    if (S_eff > MAX_CUTS) S_eff = MAX_CUTS;

    CfseState2P st; st.sbase = sbase; st.out = out;
    uint32_t cuts[MAX_CUTS], saved_raw[MAX_CUTS], seg_bytes[MAX_CUTS + 1] = {0};
    int32_t succ[MAX_CUTS + 1];
    for (uint32_t q = 0; q < S_eff; q++) {
        uint32_t v = (uint32_t)((mark + ((uint64_t)q * m) / S_eff) % m);
        cuts[q] = v; uint32_t raw = edges[v]; saved_raw[q] = raw;
        st.saved_next[q] = raw & mask;
        st.saved_code[q] = wide ? extra[v] : (raw >> bits);
        w_len[sbase + q] = 0u;
    }
    for (uint32_t q = 0; q < S_eff; q++) {
        uint32_t v = cuts[q], sc = sbase + q;
        if (wide) extra[v] = sc; else edges[v] = (sc << bits) | st.saved_next[q];
    }
    edges[m] = m; if (wide) extra[m] = 0;

    uint32_t p1_next = 0, p1_act = 0, j_init[8], q_init[8], b_init[8];
    for (int l = 0; l < 8; l++) {
        if (p1_next < S_eff) {
            uint32_t q = p1_next++; p1_act++; q_init[l] = q;
            b_init[l] = cfse_true_len(k, st.saved_code[q]);
            j_init[l] = st.saved_next[q];
        } else { q_init[l] = MAX_CUTS; b_init[l] = 0; j_init[l] = m; }
    }
    uint32_t j0=j_init[0], j1=j_init[1], j2=j_init[2], j3=j_init[3];
    uint32_t j4=j_init[4], j5=j_init[5], j6=j_init[6], j7=j_init[7];
    uint32_t q0=q_init[0], q1=q_init[1], q2=q_init[2], q3=q_init[3];
    uint32_t q4=q_init[4], q5=q_init[5], q6=q_init[6], q7=q_init[7];
    uint32_t b0=b_init[0], b1=b_init[1], b2=b_init[2], b3=b_init[3];
    uint32_t b4=b_init[4], b5=b_init[5], b6=b_init[6], b7=b_init[7];

    #define CFSE_P1(J, Q, B, CD_EXPR, E_VAR) do { \
        uint32_t cd = (CD_EXPR); J = (E_VAR) & mask; uint32_t wl = w_len[cd]; B += wl; \
        if (__builtin_expect(wl >= 65534u || cd >= sbase, 0)) { \
            if (cd >= sbase) { \
                succ[Q] = (int32_t)(cd - sbase); seg_bytes[Q] = B; \
                if (p1_next < S_eff) { \
                    uint32_t nq = p1_next++; Q = nq; \
                    B = cfse_true_len(k, st.saved_code[nq]); J = st.saved_next[nq]; \
                } else { Q = MAX_CUTS; B = 0; J = m; p1_act--; } \
            } else { B = (B - wl) + cfse_true_len(k, cd); } \
        } \
    } while (0)

    if (!wide) {
        while (p1_act > 0) {
            uint32_t e0=edges[j0], e1=edges[j1], e2=edges[j2], e3=edges[j3];
            uint32_t e4=edges[j4], e5=edges[j5], e6=edges[j6], e7=edges[j7];
            CFSE_P1(j0,q0,b0,e0>>bits,e0); CFSE_P1(j1,q1,b1,e1>>bits,e1);
            CFSE_P1(j2,q2,b2,e2>>bits,e2); CFSE_P1(j3,q3,b3,e3>>bits,e3);
            CFSE_P1(j4,q4,b4,e4>>bits,e4); CFSE_P1(j5,q5,b5,e5>>bits,e5);
            CFSE_P1(j6,q6,b6,e6>>bits,e6); CFSE_P1(j7,q7,b7,e7>>bits,e7);
        }
    } else {
        while (p1_act > 0) {
            uint32_t e0=edges[j0],c0=extra[j0],e1=edges[j1],c1=extra[j1];
            uint32_t e2=edges[j2],c2=extra[j2],e3=edges[j3],c3=extra[j3];
            uint32_t e4=edges[j4],c4=extra[j4],e5=edges[j5],c5=extra[j5];
            uint32_t e6=edges[j6],c6=extra[j6],e7=edges[j7],c7=extra[j7];
            CFSE_P1(j0,q0,b0,c0,e0); CFSE_P1(j1,q1,b1,c1,e1);
            CFSE_P1(j2,q2,b2,c2,e2); CFSE_P1(j3,q3,b3,c3,e3);
            CFSE_P1(j4,q4,b4,c4,e4); CFSE_P1(j5,q5,b5,c5,e5);
            CFSE_P1(j6,q6,b6,c6,e6); CFSE_P1(j7,q7,b7,c7,e7);
        }
    }

    st.n_cyc = 0; size_t clen = 0; int32_t cs = 0; uint32_t raw_seg[MAX_CUTS];
    while (st.n_cyc < S_eff && clen < n) {
        size_t take = seg_bytes[cs];
        if (clen + take > n) take = n - clen;
        st.cyc_segs[st.n_cyc] = (uint32_t)cs;
        raw_seg[st.n_cyc] = (uint32_t)take;
        st.n_cyc++; clen += take; cs = succ[cs];
        if (cs == 0) break;
    }
    size_t rem_pos = clen;
    for (uint32_t idx = 0; idx < st.n_cyc; idx++) {
        uint32_t ls = raw_seg[idx];
        st.cyc_end[idx] = (uint32_t)rem_pos;
        rem_pos -= ls;
        st.cyc_off[idx] = (uint32_t)rem_pos;
    }

    for (uint32_t q = 0; q < S_eff; q++) w_len[sbase + q] = 65535u;

    st.next_idx = 0; st.active = 0;
    uint8_t *p_init2[8];
    for (int l = 0; l < 8; l++) {
        if (st.next_idx < st.n_cyc) {
            uint32_t idx = st.next_idx++, sg = st.cyc_segs[idx];
            uint32_t off = st.cyc_off[idx], end = st.cyc_end[idx];
            st.active++; st.active_flag[l] = 1;
            p_init2[l] = out + end; st.hard_start[l] = out + off;
            st.safe_start[l] = (end - off >= 16u) ? (out + off + 16u) : (out + end + 1u);
            cfse_emit_bounded(k, &st, l, &p_init2[l], st.saved_code[sg]);
            j_init[l] = st.saved_next[sg];
        } else {
            st.active_flag[l] = 0; p_init2[l] = k->dummy_buf + 2048;
            st.hard_start[l] = k->dummy_buf; st.safe_start[l] = k->dummy_buf + 4096;
            j_init[l] = m;
        }
    }
    j0=j_init[0]; j1=j_init[1]; j2=j_init[2]; j3=j_init[3];
    j4=j_init[4]; j5=j_init[5]; j6=j_init[6]; j7=j_init[7];
    uint8_t * restrict p0=p_init2[0], * restrict p1=p_init2[1], * restrict p2=p_init2[2], * restrict p3=p_init2[3];
    uint8_t * restrict p4=p_init2[4], * restrict p5=p_init2[5], * restrict p6=p_init2[6], * restrict p7=p_init2[7];

    #define CFSE_STEP2(L, J, P, CD_EXPR, E_VAR) do { \
        uint32_t cd = (CD_EXPR); J = (E_VAR) & mask; uint32_t ln = w_len[cd]; \
        if (__builtin_expect(ln <= 16u && P >= st.safe_start[L], 1)) { \
            _mm_storeu_si128((__m128i *)(P - 16), inline16_ra[cd]); P -= ln; \
        } else { uint8_t *ptmp = P; cfse_cold_2p(k, &st, L, &(J), &ptmp, cd); P = ptmp; } \
    } while (0)

    if (!wide) {
        while (st.active > 0) {
            uint32_t e0=edges[j0], e1=edges[j1], e2=edges[j2], e3=edges[j3];
            uint32_t e4=edges[j4], e5=edges[j5], e6=edges[j6], e7=edges[j7];
            CFSE_STEP2(0,j0,p0,e0>>bits,e0); CFSE_STEP2(1,j1,p1,e1>>bits,e1);
            CFSE_STEP2(2,j2,p2,e2>>bits,e2); CFSE_STEP2(3,j3,p3,e3>>bits,e3);
            CFSE_STEP2(4,j4,p4,e4>>bits,e4); CFSE_STEP2(5,j5,p5,e5>>bits,e5);
            CFSE_STEP2(6,j6,p6,e6>>bits,e6); CFSE_STEP2(7,j7,p7,e7>>bits,e7);
        }
    } else {
        while (st.active > 0) {
            uint32_t e0=edges[j0],c0=extra[j0],e1=edges[j1],c1=extra[j1];
            uint32_t e2=edges[j2],c2=extra[j2],e3=edges[j3],c3=extra[j3];
            uint32_t e4=edges[j4],c4=extra[j4],e5=edges[j5],c5=extra[j5];
            uint32_t e6=edges[j6],c6=extra[j6],e7=edges[j7],c7=extra[j7];
            CFSE_STEP2(0,j0,p0,c0,e0); CFSE_STEP2(1,j1,p1,c1,e1);
            CFSE_STEP2(2,j2,p2,c2,e2); CFSE_STEP2(3,j3,p3,c3,e3);
            CFSE_STEP2(4,j4,p4,c4,e4); CFSE_STEP2(5,j5,p5,c5,e5);
            CFSE_STEP2(6,j6,p6,c6,e6); CFSE_STEP2(7,j7,p7,c7,e7);
        }
    }

    for (uint32_t q = 0; q < S_eff; q++) {
        uint32_t v = cuts[q]; edges[v] = saved_raw[q];
        if (wide) extra[v] = st.saved_code[q];
    }
    repeat_period(n, out, &clen);
}

/* =========================================================================
 * Self-Tests & Paired 25-Round 6-Arm Confrontation
 * ========================================================================= */
static size_t to_capped_records(const uint8_t *L, size_t n, uint16_t *recs) {
    size_t rlen = 0;
    for (size_t i = 0; i < n; i++) {
        uint8_t b = L[i];
        if (rlen > 0 && (uint8_t)(recs[rlen - 1] >> 8) == b && (recs[rlen - 1] & 255u) < 127u) recs[rlen - 1]++;
        else recs[rlen++] = (uint16_t)(((uint16_t)b << 8) | 1u);
    }
    return rlen;
}

static void dense_lf_reference(const uint8_t *L, size_t n, uint32_t p, uint8_t *ref_out) {
    uint32_t counts[256] = {0}, C[256], occ[256] = {0};
    for (size_t i = 0; i < n; i++) counts[L[i]]++;
    uint32_t s = 0;
    for (int a = 0; a < 256; a++) { C[a] = s; s += counts[a]; }
    uint32_t *lf = (uint32_t *)malloc(n * sizeof(uint32_t));
    for (size_t i = 0; i < n; i++) { uint8_t b = L[i]; lf[i] = C[b] + occ[b]++; }
    uint8_t *rev = (uint8_t *)malloc(n);
    size_t clen = 0; uint32_t cur = p;
    for (size_t step = 0; step < n; step++) {
        rev[clen++] = L[cur]; cur = lf[cur];
        if (cur == p) break;
    }
    for (size_t i = 0; i < clen; i++) ref_out[i] = rev[clen - 1 - i];
    repeat_period(n, ref_out, &clen);
    free(lf); free(rev);
}

static int cmp_double(const void *a, const void *b) {
    double da = *(const double *)a, db = *(const double *)b;
    return (da > db) - (da < db);
}

static double median(double *arr, int count) {
    qsort(arr, (size_t)count, sizeof(double), cmp_double);
    return arr[count / 2];
}

static void run_arm(
    LKernel *lk, CfseKernel *ck, int arm,
    const uint16_t *recs, size_t n_recs, uint32_t n, uint32_t p,
    uint8_t *out, double *t_con, double *t_em
) {
    double t0 = now_sec(), t1, t2;
    switch (arm) {
        case 0:
            open_construct(lk, recs, n_recs, n, p, LIMIT, 0, 0, 0);
            t1 = now_sec();
            open_emit_orig(lk, n, out);
            t2 = now_sec();
            break;
        case 1:
            cfse_construct(ck, recs, n_recs, n, p, out, 0, 0);
            t1 = now_sec();
            cfse_emit_scalar_1lane(ck, n, out);
            t2 = now_sec();
            break;
        case 2:
            open_construct(lk, recs, n_recs, n, p, LIMIT, 1, 1, 0);
            t1 = now_sec();
            open_emit_u16_1lane(lk, n, out);
            t2 = now_sec();
            break;
        case 3:
            cfse_construct(ck, recs, n_recs, n, p, out, 1, 0);
            t1 = now_sec();
            cfse_emit_simd_1lane(ck, n, out);
            t2 = now_sec();
            break;
        case 4:
            gim_rv_construct(lk, recs, n_recs, n, p, LIMIT, 64u);
            t1 = now_sec();
            l_bvsmc_emit_2p(lk, n, 64u, out);
            t2 = now_sec();
            break;
        case 5:
            cfse_construct(ck, recs, n_recs, n, p, out, 1, 64u);
            t1 = now_sec();
            cfse_bvsmc_emit_2p(ck, n, 64u, out);
            t2 = now_sec();
            break;
        default:
            abort();
    }
    *t_con = t1 - t0;
    *t_em = t2 - t1;
}

static void run_adversarial_selftests(LKernel *lk, CfseKernel *ck) {
    uint8_t *L = (uint8_t *)malloc(60000);
    uint16_t *recs = (uint16_t *)malloc(60000 * 2u);
    uint8_t *ref = (uint8_t *)malloc(60000);
    /* Guarded buffer with 32-byte left & right overrun canaries! */
    uint8_t *guard = (uint8_t *)malloc(60064);
    uint8_t *out = guard + 32;

    struct { const char *name; size_t n; uint32_t p; } cases[8] = {
        {"n1_singleton", 1, 0},
        {"n5_multicycle", 5, 3},
        {"periodic_5k_single_byte", 5000, 1337},
        {"thick_primary_split", 81, 40},
        {"thick_primary_head", 65, 0},
        {"thick_primary_tail", 65, 59},
        {"all_thin_w16_fallback_32k", 32000, 12345},
        {"periodic_shear_50k_gimrule", 49725, 25000},
    };

    for (int ci = 0; ci < 8; ci++) {
        size_t n = cases[ci].n;
        uint32_t p = cases[ci].p;
        if (ci == 0) { L[0] = 'z'; }
        else if (ci == 1) { memcpy(L, "babaa", 5); }
        else if (ci == 2) { memset(L, 'q', 5000); }
        else if (ci == 3) { memset(L, 'a', 40); L[40] = 'b'; memset(L + 41, 'a', 40); }
        else if (ci == 4 || ci == 5) { memset(L, 'm', 60); memset(L + 60, 'n', 5); }
        else if (ci == 6) { for (size_t i = 0; i < 32000; i++) L[i] = (uint8_t)((i / 16) % 17 + 'a'); }
        else {
            size_t pos = 0;
            for (int rep = 0; rep < 124; rep++) {
                memset(L + pos, 'a', 200); pos += 200;
                memset(L + pos, 'b', 200); pos += 200;
                L[pos++] = 'c';
            }
            L[pos++] = 'd';
            assert(pos == n);
        }

        size_t nr = to_capped_records(L, n, recs);
        dense_lf_reference(L, n, p, ref);

        for (int arm = 0; arm < 6; arm++) {
            memset(guard, 0xA5, 60064);
            double tc = 0, te = 0;
            run_arm(lk, ck, arm, recs, nr, (uint32_t)n, p, out, &tc, &te);
            assert(memcmp(out, ref, n) == 0);
            for (int g = 0; g < 32; g++) {
                assert(guard[g] == 0xA5 && out[n + g] == 0xA5);
            }
            if (ci == 6 && (arm == 1 || arm == 3 || arm == 5)) assert(ck->used_out_scratch == 0);
            if (ci == 7 && (arm == 1 || arm == 3 || arm == 5)) assert(ck->rules_len > 0);
        }
    }
    free(L); free(recs); free(ref); free(guard);
    printf("[PASS] Adversarial self-tests (8/8 incl. n=1, n=5 multi-cycle, 32k fallback, 50k >64B GimRule, left/right overrun canaries) across all 6 arms.\n\n");
}

int main(int argc, char **argv) {
    const char *dir = (argc > 1) ? argv[1] : "/tmp/admitted-bwt/admitted-bwt";
    const int rounds = (argc > 2) ? atoi(argv[2]) : 25;
    const char *names[6] = {"block-0.L", "block-2.L", "block-85.L", "block-143.L", "block-210.L", "block-207.L"};
    const uint32_t ps[6]  = {264045,      320807,      325088,       802868,        791429,        205180};
    const char *arm_names[6] = {
        "Open-qi16",
        "CFSE-Scalar-1L",
        "Open+U16Q-1L",
        "CFSE-SIMD-1L",
        "GIM-RV+BVSMC-2P",
        "CFSE+BVSMC-2P"
    };

    LKernel lk; lkernel_init(&lk, 1000000);
    CfseKernel ck; cfse_init(&ck, 1000000);
    run_adversarial_selftests(&lk, &ck);

    uint8_t *L = (uint8_t *)malloc(1000000);
    uint16_t *recs = (uint16_t *)malloc(1000000 * 2u);
    uint8_t *ref = (uint8_t *)malloc(1000000);
    uint8_t *guard = (uint8_t *)malloc(1000128);
    uint8_t *out = guard + 32;

    int pass_c2_con = 0, pass_c2_tot = 0, pass_c3_tot = 0, pass_c4_open = 0, pass_c4_gim = 0;

    printf("Block        | Arm                |  Con(ms) Emit(ms)  Tot(ms) |  Con/Op  Tot/Op | Scratch(KiB)\n");
    printf("-------------------------------------------------------------------------------------------------\n");

    for (int bi = 0; bi < 6; bi++) {
        char path[512];
        snprintf(path, sizeof(path), "%s/%s", dir, names[bi]);
        FILE *fp = fopen(path, "rb");
        if (!fp) { fprintf(stderr, "Cannot open %s\n", path); return 1; }
        size_t n = fread(L, 1, 1000000, fp);
        fclose(fp);

        size_t nr = to_capped_records(L, n, recs);
        dense_lf_reference(L, n, ps[bi], ref);

        double t_con[6][64], t_em[6][64], t_tot[6][64];
        size_t scr_bytes[6] = {0};
        assert(rounds <= 64);

        /* 3 warmup rounds + `rounds` paired interleaved rounds */
        for (int r = -3; r < rounds; r++) {
            for (int step = 0; step < 6; step++) {
                int arm = (step + (r + 3)) % 6;
                if (r == -3) memset(guard, 0xA5, n + 64);
                double tc = 0, te = 0;
                run_arm(&lk, &ck, arm, recs, nr, (uint32_t)n, ps[bi], out, &tc, &te);
                if (r == -3) {
                    if (memcmp(out, ref, n) != 0) {
                        fprintf(stderr, "BYTE MISMATCH on %s arm %d!\n", names[bi], arm);
                        return 1;
                    }
                    for (int g = 0; g < 32; g++) {
                        assert(guard[g] == 0xA5 && out[n + g] == 0xA5);
                    }
                    if (arm == 5) assert(ck.used_out_scratch == 1);
                    if (arm == 0) scr_bytes[arm] = lkernel_scratch_bytes(&lk, 1, lk.m);
                    else if (arm == 2) scr_bytes[arm] = lkernel_scratch_bytes(&lk, 1, lk.m);
                    else if (arm == 4) scr_bytes[arm] = lkernel_scratch_bytes(&lk, 0, lk.m);
                    else scr_bytes[arm] = cfse_scratch_bytes(&ck, ck.m);
                }
                if (r >= 0) {
                    t_con[arm][r] = tc * 1e3;
                    t_em[arm][r]  = te * 1e3;
                    t_tot[arm][r] = (tc + te) * 1e3;
                }
            }
        }

        double mc[6], me[6], mt[6];
        for (int a = 0; a < 6; a++) {
            mc[a] = median(t_con[a], rounds);
            me[a] = median(t_em[a], rounds);
            mt[a] = median(t_tot[a], rounds);
        }

        for (int a = 0; a < 6; a++) {
            printf("%-12s | %-18s | %8.3f %8.3f %8.3f | %7.3fx %7.3fx | %9.1f\n",
                   a == 0 ? names[bi] : "", arm_names[a],
                   mc[a], me[a], mt[a], mc[a] / mc[0], mt[a] / mt[0],
                   scr_bytes[a] / 1024.0);
        }
        printf("-------------------------------------------------------------------------------------------------\n");

        if (mc[1] <= 0.80 * mc[0]) pass_c2_con++;
        if (mt[1] <= 0.88 * mt[0]) pass_c2_tot++;
        if (mt[3] <= 0.85 * mt[2]) pass_c3_tot++;
        if (mt[5] <= 0.68 * mt[0]) pass_c4_open++;
        if (mt[5] <= 0.97 * mt[4]) pass_c4_gim++;
    }

    int all_pass = (pass_c2_con >= 5) && (pass_c2_tot >= 5) && (pass_c3_tot >= 5) && (pass_c4_open >= 5) && (pass_c4_gim >= 5);
    printf("\n=== PRE-REGISTERED GATE G5 SUMMARY ===\n");
    printf("  (2a) Pure Scalar Producer  (CFSE-Scalar Con <= 0.80x Open-qi16 Con): %d/6 blocks\n", pass_c2_con);
    printf("  (2b) Pure Scalar Total     (CFSE-Scalar Tot <= 0.88x Open-qi16 Tot): %d/6 blocks\n", pass_c2_tot);
    printf("  (3)  1-Lane SIMD Total     (CFSE-SIMD-1L    <= 0.85x Open+U16Q-1L ): %d/6 blocks\n", pass_c3_tot);
    printf("  (4a) Composed vs Open-qi16 (CFSE+BVSMC-2P   <= 0.68x Open-qi16 Tot): %d/6 blocks\n", pass_c4_open);
    printf("  (4b) Composed vs GIM-RV    (CFSE+BVSMC-2P   <= 0.97x GIM-RV+BV2P  ): %d/6 blocks\n", pass_c4_gim);
    printf("  STRICT GATE G5 VERDICT: %s\n", all_pass ? "PASS" : "FAIL");

    free(L); free(recs); free(ref); free(guard);
    lkernel_free(&lk); cfse_free(&ck);
    return 0;
}
