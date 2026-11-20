/*
 * Gate G5 Paid-Cost Falsifier & Controlled 7-Arm Confrontation Harness for Carbon Cycle 10:
 * Co-Equilibrated U24Q + R4-DGIM + Bipolar Dead-Out 1-Pass Splicing (W24-BP1P)
 * vs Shipped Open (qi16), Calibrated Open+U16Q, Cycle 7 GIM-RV+BVSMC-2P, and Cycle 9 R4-DGIM+BVSMC-2P.
 *
 * PRE-REGISTERED PASS/FAIL RULE (frozen before compilation and execution):
 * Across all six authenticated Wikidata BWT blocks (block-0, 2, 85, 143, 210, 207),
 * 25 paired interleaved rounds after 3 warmup rounds (rotating arm order (step + r + 3) % 7):
 *
 *   Arm 0: Open-qi16          — exact C mirror of shipped src/open.rs (q_push_scalar + endpoint.rs + 1L emit)
 *   Arm 1: Open+U16Q-1L       — calibrated Open+U16Q (#664/#666: U16Q + endpoint.rs + inline16 + 1L SIMD emit)
 *   Arm 2: GIM-RV(L16)+BV2P   — Cycle 7 best promoted architecture (U16Q + inline-linked GIM-RV + BVSMC-2P)
 *   Arm 3: R4-DGIM(L16)+BV2P  — Cycle 9 best zero-pool architecture (U16Q + R4-DGIM + BVSMC-2P)
 *   Arm 4: W24-R4+BV2P        — LIMIT=24 (U24Q) + R4-DGIM + 2-pass BVSMC-2P (tests Thm 10.1 2P separability null)
 *   Arm 5: L16-R4+BP1P        — LIMIT=16 (U16Q) + R4-DGIM + 1-pass Bipolar Dead-Out BP-BVSMC-1P(8,48)
 *   Arm 6: W24-R4+BP1P        — Co-Equilibrated LIMIT=24 (U24Q) + R4-DGIM + 1-pass Bipolar Dead-Out BP-BVSMC-1P(8,48)
 *
 * PASS iff ALL FOUR hold:
 *   (1) Exactness, Overflow Memory Bound, & Full Sanitizer Cleanliness:
 *       - Arms 1..6 match dense LF byte-for-byte on 6/6 blocks (5,399,692 B), pass 8/8 adversarial
 *         self-tests (with 32B left/right overrun canaries), and exit with 0 errors and 0 leaks under
 *         -fsanitize=address,undefined;
 *       - Arm 6 BP-BVSMC-1P peak recycled overflow buffer <= 256 KiB (<= 128 pages of 2 KiB, >= 4.8x
 *         smaller than Cycle 2 BVSMC-1P's 1,230 KiB chunk pool) on 6/6 blocks;
 *   (2) End-to-End Speedup over Shipped Open-qi16:
 *       - Arm 6 (W24-R4+BP1P) total median <= 0.60x Arm 0 (Open-qi16) total on >= 5/6 blocks
 *         (and <= 0.65x on 6/6 blocks);
 *   (3) End-to-End Speedup over Cycle 7 Promoted GIM-RV+BVSMC-2P:
 *       - Arm 6 (W24-R4+BP1P) total median <= 0.92x Arm 2 (GIM-RV(L16)+BV2P) total on >= 5/6 blocks
 *         (and < 1.00x on 6/6 blocks);
 *   (4) Co-Design Synergy over Cycle 9 R4-DGIM(L16)+BVSMC-2P:
 *       - Arm 6 (W24-R4+BP1P) total median     <= 0.94x Arm 3 (R4-DGIM(L16)+BV2P) total on >= 5/6 blocks
 *         (and < 1.00x on 6/6 blocks), AND
 *       - Arm 6 (W24-R4+BP1P) construct median <= 0.92x Arm 3 construct on >= 5/6 blocks.
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
#define R4_FAR 8u
#define R4_PAD_SLOTS 16u

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

/* 16-byte aligned Top Slot (4 per 64B cache line, 100% u32-indexed!) */
typedef struct {
    uint32_t prev;
    uint32_t width;
    uint32_t word;
    uint32_t other; /* bot index during RV; reused after remaining==0 to store bottom L-coord x! */
} R4Top;

/* 8-byte aligned Bottom Slot (8 per 64B cache line, 100% u32-indexed!) */
typedef struct {
    uint32_t prev;
    uint32_t top;
} R4Bot;

/* =========================================================================
 * Baseline Doubly-Linked Kernel (Arms 0, 2, 3, 5: Open-qi16, Open+U16Q, GIM-RV)
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
 * Cycle 9 R4-DGIM Kernel (Arms 1, 4, 6: ZERO next[], ZERO order/terminals,
 * ZERO bottom[], ZERO offsets[], long-only spelling!)
 * ========================================================================= */
typedef struct {
    uint32_t *queues[256];
    uint32_t q_len[256];
    uint32_t q_cap[256];

    Strip *strips;
    size_t strips_len, strips_cap;
    uint32_t *image_order;
    size_t img_cap;
    uint32_t *strip_top;
    size_t strip_top_cap;

    R4Top *tops;
    size_t tops_cap;
    R4Bot *bots;
    size_t bots_cap;

    EpRule *rules;
    size_t rules_len, rules_cap;
    uint32_t *stack;
    size_t stack_len, stack_cap;
    Band *bands;
    size_t bands_len, bands_cap;

    uint8_t *small_arena;
    uint32_t arena_len, arena_cap;
    uint8_t *long_spell;
    size_t spell_len, spell_cap;

    __m128i *inline16;
    uint16_t *w_len;
    size_t dict_cap;

    uint32_t *edges;
    size_t edges_cap;
    uint32_t *extra;
    size_t extra_cap;
    int wide;

    uint32_t m, mark, bits, words;
    size_t active_tops, active_bots;
    uint8_t *dummy_buf;
} R4Kernel;

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
 * Baseline LKernel (Open-qi16, Open+U16Q, GIM-RV)
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
    free(k->strips); free(k->image_order); free(k->nodes); free(k->rules);
    for (int s = 0; s < 2; s++) { free(k->order[s]); free(k->terminals[s]); }
    free(k->bottom); free(k->stack); free(k->bands); free(k->small_arena);
    free(k->spelling); free(k->offsets); free(k->inline16); free(k->w_len);
    free(k->edges); free(k->extra); free(k->dummy_buf);
}

static size_t lkernel_active_bytes(const LKernel *k, int include_4vec, int include_arena) {
    size_t q_used = 0;
    for (int a = 0; a < 256; a++) q_used += (size_t)k->q_len[a] * 4u;
    size_t vec4 = 0;
    if (include_4vec) {
        for (int s = 0; s < 2; s++) vec4 += (k->order_len[s] + k->term_len[s]) * 4u;
    }
    return q_used
         + k->strips_len * sizeof(Strip)
         + k->strips_len * 4u
         + k->nodes_len * sizeof(EpNode)
         + k->rules_len * sizeof(EpRule)
         + vec4
         + k->nodes_len * 4u /* bottom */
         + k->bands_len * sizeof(Band)
         + (include_arena ? k->arena_len : 0u)
         + k->spell_len
         + k->off_len * 4u
         + (size_t)(256u + k->words) * (sizeof(__m128i) + sizeof(uint16_t))
         + (size_t)k->m * 4u;
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
    nodes[p].next[side] = nx; nodes[nx].prev[side] = p;
}

static inline void ep_insert(EpNode *nodes, uint32_t at, uint32_t id, int side) {
    uint32_t nx = nodes[at].next[side];
    nodes[id].prev[side] = at; nodes[id].next[side] = nx;
    nodes[at].next[side] = id; nodes[nx].prev[side] = id;
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
        uint32_t idx = k->image_order[ii]; Strip s = k->strips[idx];
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
            if (wide) { for (uint32_t s = 0; s < span; s++) k->extra[dst + s] = b.code; }
            uint32_t * restrict edst = k->edges + dst;
            const uint32_t * restrict qsrc = qa + from;
            for (uint32_t s = 0; s < span; s++) edst[s] = pack | qsrc[s];
            from += span; kc = b.end;
        }
    }
    k->m = m; k->mark = mark; k->bits = bits; k->words = words; k->wide = wide;
}

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
            if (wide) { for (uint32_t s = 0; s < span; s++) k->extra[dst + s] = b.code; }
            uint32_t * restrict edst = k->edges + dst;
            const uint32_t * restrict qsrc = qa + from;
            for (uint32_t s = 0; s < span; s++) edst[s] = pack | qsrc[s];
            from += span; kc = b.end;
        }
    }
    k->m = m; k->mark = mark; k->bits = bits; k->words = words; k->wide = wide;
}

static inline void repeat_period(size_t n, uint8_t *out, size_t *out_len) {
    size_t len = *out_len; if (len > n) len = n;
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
            const uint8_t *src; uint32_t real_ln;
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
 * Cycle 9 Candidate: R4-DGIM (Rule-R4 16B/8B Overwrite + FAR=8 + Zero-bottom
 * + Zero-offsets Direct-Arena M_64)
 * ========================================================================= */
static void r4_init(R4Kernel *k, size_t max_n) {
    memset(k, 0, sizeof(*k));
    for (int a = 0; a < 256; a++) {
        k->q_cap[a] = 1024;
        k->queues[a] = (uint32_t *)malloc((k->q_cap[a] + 32) * 4u);
    }
    k->strips_cap = 8192;
    k->strips = (Strip *)malloc(k->strips_cap * sizeof(Strip));
    k->img_cap = 8192;
    k->image_order = (uint32_t *)malloc(k->img_cap * 4u);
    k->strip_top_cap = 8192;
    k->strip_top = (uint32_t *)malloc(k->strip_top_cap * 4u);

    k->tops_cap = 32768;
    k->tops = (R4Top *)aligned_alloc(64, k->tops_cap * sizeof(R4Top));
    k->bots_cap = 32768;
    k->bots = (R4Bot *)aligned_alloc(64, k->bots_cap * sizeof(R4Bot));
    memset(k->tops, 0, R4_PAD_SLOTS * sizeof(R4Top));
    memset(k->bots, 0, R4_PAD_SLOTS * sizeof(R4Bot));

    k->rules_cap = 2048;
    k->rules = (EpRule *)malloc(k->rules_cap * sizeof(EpRule));
    k->stack_cap = 2048;
    k->stack = (uint32_t *)malloc(k->stack_cap * 4u);
    k->bands_cap = 16384;
    k->bands = (Band *)malloc(k->bands_cap * sizeof(Band));

    k->arena_cap = 262144;
    k->small_arena = (uint8_t *)malloc(k->arena_cap + 128);
    memset(k->small_arena, 0, 320);
    for (uint32_t c = 0; c < 256; c++) k->small_arena[c] = (uint8_t)c;
    k->arena_len = 256u;

    k->spell_cap = 65536;
    k->long_spell = (uint8_t *)malloc(k->spell_cap + 128);

    k->dict_cap = 16384;
    k->inline16 = (__m128i *)aligned_alloc(64, k->dict_cap * sizeof(__m128i));
    k->w_len = (uint16_t *)calloc(k->dict_cap, sizeof(uint16_t));
    memset(k->inline16, 0, 256 * sizeof(__m128i));
    for (uint32_t c = 0; c < 256; c++) {
        ((uint8_t *)&k->inline16[c])[0] = (uint8_t)c;
        k->w_len[c] = 1u;
    }

    k->edges_cap = max_n + 16;
    k->edges = (uint32_t *)malloc(k->edges_cap * 4u);
    k->extra_cap = max_n + 16;
    k->extra = (uint32_t *)malloc(k->extra_cap * 4u);
    k->dummy_buf = (uint8_t *)malloc(4096);
}

static void r4_free(R4Kernel *k) {
    for (int a = 0; a < 256; a++) free(k->queues[a]);
    free(k->strips); free(k->image_order); free(k->strip_top);
    free(k->tops); free(k->bots); free(k->rules); free(k->stack);
    free(k->bands); free(k->small_arena); free(k->long_spell);
    free(k->inline16); free(k->w_len); free(k->edges); free(k->extra);
    free(k->dummy_buf);
}

static size_t r4_active_bytes(const R4Kernel *k) {
    size_t q_used = 0;
    for (int a = 0; a < 256; a++) q_used += (size_t)k->q_len[a] * 4u;
    return q_used
         + k->strips_len * sizeof(Strip)
         + k->strips_len * 8u /* image_order + strip_top */
         + k->active_tops * sizeof(R4Top)
         + k->active_bots * sizeof(R4Bot)
         + k->rules_len * sizeof(EpRule)
         + k->bands_len * sizeof(Band)
         + k->arena_len
         + k->spell_len
         + (size_t)(256u + k->words) * (sizeof(__m128i) + sizeof(uint16_t))
         + (size_t)k->m * 4u;
}

static inline void r4_q_push_scalar(R4Kernel *k, uint8_t a, uint32_t start, uint32_t w) {
    uint32_t len = k->q_len[a];
    if (__builtin_expect(len + w > k->q_cap[a], 0)) {
        while (len + w > k->q_cap[a]) k->q_cap[a] *= 2;
        k->queues[a] = (uint32_t *)realloc(k->queues[a], (k->q_cap[a] + 32u) * 4u);
    }
    uint32_t *dst = k->queues[a] + len;
    for (uint32_t i = 0; i < w; i++) dst[i] = start + i;
    k->q_len[a] = len + w;
}

static inline void r4_q_push_u16(R4Kernel *k, uint8_t a, uint32_t start, uint32_t w) {
    uint32_t len = k->q_len[a];
    if (__builtin_expect(len + 16u > k->q_cap[a], 0)) {
        while (len + 16u > k->q_cap[a]) k->q_cap[a] *= 2;
        k->queues[a] = (uint32_t *)realloc(k->queues[a], (k->q_cap[a] + 32u) * 4u);
    }
    store_seq16(k->queues[a] + len, start);
    k->q_len[a] = len + w;
}

static inline void r4_q_push_u24(R4Kernel *k, uint8_t a, uint32_t start, uint32_t w) {
    uint32_t len = k->q_len[a];
    if (__builtin_expect(len + 24u > k->q_cap[a], 0)) {
        while (len + 24u > k->q_cap[a]) k->q_cap[a] *= 2;
        k->queues[a] = (uint32_t *)realloc(k->queues[a], (k->q_cap[a] + 32u) * 4u);
    }
    uint32_t *dst = k->queues[a] + len;
    const __m256i i0 = _mm256_setr_epi32(0, 1, 2, 3, 4, 5, 6, 7);
    const __m256i i1 = _mm256_setr_epi32(8, 9, 10, 11, 12, 13, 14, 15);
    const __m256i i2 = _mm256_setr_epi32(16, 17, 18, 19, 20, 21, 22, 23);
    __m256i b = _mm256_set1_epi32((int32_t)start);
    _mm256_storeu_si256((__m256i *)(dst + 0),  _mm256_add_epi32(b, i0));
    _mm256_storeu_si256((__m256i *)(dst + 8),  _mm256_add_epi32(b, i1));
    _mm256_storeu_si256((__m256i *)(dst + 16), _mm256_add_epi32(b, i2));
    k->q_len[a] = len + w;
}

static inline void r4_push_strip(R4Kernel *k, uint32_t from, uint32_t to, uint32_t width, uint8_t byte) {
    if (__builtin_expect(k->strips_len == k->strips_cap, 0)) {
        k->strips_cap *= 2;
        k->strips = (Strip *)realloc(k->strips, k->strips_cap * sizeof(Strip));
    }
    k->strips[k->strips_len++] = (Strip){from, to, width, byte};
}

static inline void r4_dict_ensure(R4Kernel *k, size_t need) {
    if (__builtin_expect(need > k->dict_cap, 0)) {
        size_t old = k->dict_cap;
        while (need > k->dict_cap) k->dict_cap *= 2;
        __m128i *n16 = (__m128i *)aligned_alloc(64, k->dict_cap * sizeof(__m128i));
        memcpy(n16, k->inline16, old * sizeof(__m128i));
        free(k->inline16);
        k->inline16 = n16;
        k->w_len = (uint16_t *)realloc(k->w_len, k->dict_cap * sizeof(uint16_t));
    }
}

static inline void r4_spell_ensure(R4Kernel *k, size_t add) {
    if (__builtin_expect(k->spell_len + add + 80 > k->spell_cap, 0)) {
        while (k->spell_len + add + 80 > k->spell_cap) k->spell_cap *= 2;
        k->long_spell = (uint8_t *)realloc(k->long_spell, k->spell_cap + 128);
    }
}

static inline uint32_t r4_len(const R4Kernel *k, uint32_t w) {
    if (w == 0) return 0u;
    if ((w & 0x80000000u) == 0) return w >> 24;
    return k->rules[w & 0x7FFFFFFFu].len;
}

static inline uint32_t r4_join(R4Kernel *k, uint32_t aw, uint32_t bw) {
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
    uint32_t ln = r4_len(k, aw) + r4_len(k, bw);
    k->rules[rid] = (EpRule){aw, bw, ln, UINT32_MAX};
    return 0x80000000u | rid;
}

__attribute__((noinline))
static void r4_append_rule(R4Kernel *k, uint32_t word) {
    k->stack_len = 0;
    k->stack[k->stack_len++] = word;
    while (k->stack_len > 0) {
        uint32_t w = k->stack[--k->stack_len];
        if ((w & 0x80000000u) == 0) {
            uint32_t wl = w >> 24, woff = w & 0x00FFFFFFu;
            r4_spell_ensure(k, wl);
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
                r4_spell_ensure(k, rl->len);
                memmove(k->long_spell + k->spell_len, k->long_spell + rl->first, rl->len);
                k->spell_len += rl->len;
            }
        }
    }
}

static void r4_dgim_construct(
    R4Kernel *k,
    const uint16_t *records, size_t n_recs, uint32_t n, uint32_t p,
    uint32_t limit, uint32_t reserve_sentinels
) {
    uint32_t all_c[256] = {0}, thin[256] = {0}, thick[256] = {0};
    for (int a = 0; a < 256; a++) k->q_len[a] = 0;
    k->strips_len = 0;

    uint32_t m = 0, mark = 0, pos = 0;
    size_t i = 0;
    if (limit <= 16u) {
        while (i < n_recs) {
            uint16_t rec = records[i++];
            uint8_t c = (uint8_t)(rec >> 8);
            uint32_t width = (uint32_t)(rec & 255u);
            while (i < n_recs && (uint8_t)(records[i] >> 8) == c) width += (uint32_t)(records[i++] & 255u);
            uint32_t end = pos + width;
            if (__builtin_expect(width <= 16u, 1)) {
                if (__builtin_expect(p >= pos && p < end, 0)) mark = m + (p - pos);
                r4_q_push_u16(k, c, m, width);
                thin[c] += width; m += width; all_c[c] += width;
            } else if (__builtin_expect(p >= pos && p < end, 0)) {
                if (pos < p) { r4_push_strip(k, pos, all_c[c], p - pos, c); thick[c]++; all_c[c] += p - pos; }
                mark = m; r4_q_push_u16(k, c, m, 1u); thin[c] += 1u; m += 1u; all_c[c] += 1u;
                if (p + 1 < end) { uint32_t w2 = end - p - 1; r4_push_strip(k, p + 1, all_c[c], w2, c); thick[c]++; all_c[c] += w2; }
            } else {
                r4_push_strip(k, pos, all_c[c], width, c); thick[c]++; all_c[c] += width;
            }
            pos = end;
        }
    } else if (limit == 24u) {
        while (i < n_recs) {
            uint16_t rec = records[i++];
            uint8_t c = (uint8_t)(rec >> 8);
            uint32_t width = (uint32_t)(rec & 255u);
            while (i < n_recs && (uint8_t)(records[i] >> 8) == c) width += (uint32_t)(records[i++] & 255u);
            uint32_t end = pos + width;
            if (__builtin_expect(width <= 24u, 1)) {
                if (__builtin_expect(p >= pos && p < end, 0)) mark = m + (p - pos);
                r4_q_push_u24(k, c, m, width);
                thin[c] += width; m += width; all_c[c] += width;
            } else if (__builtin_expect(p >= pos && p < end, 0)) {
                if (pos < p) { r4_push_strip(k, pos, all_c[c], p - pos, c); thick[c]++; all_c[c] += p - pos; }
                mark = m; r4_q_push_u16(k, c, m, 1u); thin[c] += 1u; m += 1u; all_c[c] += 1u;
                if (p + 1 < end) { uint32_t w2 = end - p - 1; r4_push_strip(k, p + 1, all_c[c], w2, c); thick[c]++; all_c[c] += w2; }
            } else {
                r4_push_strip(k, pos, all_c[c], width, c); thick[c]++; all_c[c] += width;
            }
            pos = end;
        }
    } else {
        while (i < n_recs) {
            uint16_t rec = records[i++];
            uint8_t c = (uint8_t)(rec >> 8);
            uint32_t width = (uint32_t)(rec & 255u);
            while (i < n_recs && (uint8_t)(records[i] >> 8) == c) width += (uint32_t)(records[i++] & 255u);
            uint32_t end = pos + width;
            if (width <= limit) {
                if (p >= pos && p < end) mark = m + (p - pos);
                r4_q_push_scalar(k, c, m, width);
                thin[c] += width; m += width; all_c[c] += width;
            } else if (p >= pos && p < end) {
                if (pos < p) { r4_push_strip(k, pos, all_c[c], p - pos, c); thick[c]++; all_c[c] += p - pos; }
                mark = m; r4_q_push_scalar(k, c, m, 1); thin[c]++; m++; all_c[c]++;
                if (p + 1 < end) { uint32_t w2 = end - p - 1; r4_push_strip(k, p + 1, all_c[c], w2, c); thick[c]++; all_c[c] += w2; }
            } else {
                r4_push_strip(k, pos, all_c[c], width, c); thick[c]++; all_c[c] += width;
            }
            pos = end;
        }
    }

    size_t n_strips = k->strips_len;
    uint32_t first[256], write[256], tot = 0, tw = 0;
    for (int a = 0; a < 256; a++) {
        first[a] = tot; tot += all_c[a];
        write[a] = tw; tw += thick[a];
    }
    if (n_strips > k->img_cap) {
        k->img_cap = n_strips * 2;
        k->image_order = (uint32_t *)realloc(k->image_order, k->img_cap * 4u);
    }
    if (n_strips > k->strip_top_cap) {
        k->strip_top_cap = n_strips * 2;
        k->strip_top = (uint32_t *)realloc(k->strip_top, k->strip_top_cap * 4u);
    }
    for (size_t si = 0; si < n_strips; si++) {
        uint8_t a = k->strips[si].byte;
        k->strips[si].to += first[a];
        k->image_order[write[a]++] = (uint32_t)si;
    }

    size_t max_slots = R4_PAD_SLOTS + 3 * n_strips + 16;
    if (max_slots > k->tops_cap) {
        while (max_slots > k->tops_cap) k->tops_cap *= 2;
        free(k->tops);
        k->tops = (R4Top *)aligned_alloc(64, k->tops_cap * sizeof(R4Top));
        memset(k->tops, 0, R4_PAD_SLOTS * sizeof(R4Top));
    }
    if (max_slots > k->bots_cap) {
        while (max_slots > k->bots_cap) k->bots_cap *= 2;
        free(k->bots);
        k->bots = (R4Bot *)aligned_alloc(64, k->bots_cap * sizeof(R4Bot));
        memset(k->bots, 0, R4_PAD_SLOTS * sizeof(R4Bot));
    }

    R4Top * restrict tops = k->tops;
    R4Bot * restrict bots = k->bots;
    uint32_t * restrict strip_top = k->strip_top;
    k->rules_len = 0;
    k->arena_len = 256u;

    const uint32_t p0_base = R4_PAD_SLOTS;
    const uint32_t a0_base = p0_base + (uint32_t)n_strips + 2u;
    uint32_t t_cur = a0_base;
    uint32_t b_cur = R4_PAD_SLOTS;

    uint32_t x = 0;
    for (size_t si = 0; si < n_strips; si++) {
        Strip s = k->strips[si];
        if (x < s.from) {
            uint32_t gw = s.from - x;
            tops[t_cur] = (R4Top){t_cur - 1u, gw, 0u, b_cur};
            bots[b_cur] = (R4Bot){b_cur - 1u, t_cur};
            t_cur++; b_cur++;
        }
        strip_top[si] = t_cur;
        tops[t_cur] = (R4Top){t_cur - 1u, s.width, (1u << 24) | (uint32_t)s.byte, 0u};
        t_cur++;
        x = s.from + s.width;
    }
    if (x < n) {
        uint32_t gw = n - x;
        tops[t_cur] = (R4Top){t_cur - 1u, gw, 0u, b_cur};
        bots[b_cur] = (R4Bot){b_cur - 1u, t_cur};
        t_cur++; b_cur++;
    }
    const uint32_t a0_end = t_cur;

    uint32_t p0_cur = p0_base;
    x = 0;
    for (size_t ii = 0; ii < n_strips; ii++) {
        uint32_t idx = k->image_order[ii];
        Strip s = k->strips[idx];
        if (x < s.to) {
            uint32_t gw = s.to - x;
            tops[p0_cur] = (R4Top){p0_cur - 1u, gw, 0u, b_cur};
            bots[b_cur]  = (R4Bot){b_cur - 1u, p0_cur};
            p0_cur++; b_cur++;
        }
        uint32_t st = strip_top[idx];
        tops[st].other = b_cur;
        bots[b_cur]    = (R4Bot){b_cur - 1u, st};
        b_cur++;
        x = s.to + s.width;
    }
    if (x < n) {
        uint32_t gw = n - x;
        tops[p0_cur] = (R4Top){p0_cur - 1u, gw, 0u, b_cur};
        bots[b_cur]  = (R4Bot){b_cur - 1u, p0_cur};
        p0_cur++; b_cur++;
    }
    if (p0_cur > p0_base) tops[p0_base].prev = 0u;
    if (b_cur > R4_PAD_SLOTS) bots[R4_PAD_SLOTS].prev = 0u;
    if (a0_end > a0_base) {
        tops[a0_base].prev = (p0_cur > p0_base) ? (p0_cur - 1u) : 0u;
    }
    k->active_tops = (size_t)(p0_cur - p0_base) + (size_t)(a0_end - a0_base);
    k->active_bots = (size_t)(b_cur - R4_PAD_SLOTS);

    uint32_t sa = a0_end - 1u;
    uint32_t sb = b_cur - 1u;
    uint32_t remaining = n;

    #define R4_AHEAD_TOP(S) do { \
        uint32_t _f = (S) > R4_FAR ? ((S) - R4_FAR) : 0u; \
        _mm_prefetch((const char *)&bots[tops[_f].other], _MM_HINT_T0); \
    } while (0)
    #define R4_AHEAD_BOT(S) do { \
        uint32_t _f = (S) > R4_FAR ? ((S) - R4_FAR) : 0u; \
        _mm_prefetch((const char *)&tops[bots[_f].top], _MM_HINT_T0); \
    } while (0)

    while (remaining > 0) {
        R4Top ta = tops[sa];
        uint32_t na = ta.prev, ra = ta.width, aw = ta.word, bs = ta.other;
        R4Bot bb_slot = bots[sb];
        uint32_t nb = bb_slot.prev, ts = bb_slot.top;
        R4Top tb = tops[ts];
        uint32_t rb = tb.width, bw = tb.word;
        uint32_t fa = sa, fb = sb;

        R4_AHEAD_TOP(sa);
        R4_AHEAD_BOT(sb);

        if (bs == sb) {
            remaining -= ra;
            sa = na; sb = nb;
            continue;
        }
        for (;;) {
            uint32_t wd = r4_join(k, aw, bw);
            if (ra == rb) {
                remaining -= ra;
                tops[ts].width = ra;
                tops[ts].word  = wd;
                tops[ts].other = bs;
                bots[bs].top   = ts;
                sa = na; sb = nb;
                break;
            } else if (ra > rb) {
                remaining -= rb;
                tops[ts].width = rb;
                tops[ts].word  = wd;
                tops[ts].other = bs;
                bots[fb].prev  = bots[bs].prev;
                bots[bs]       = (R4Bot){fb, ts};
                bs = fb;
                ra -= rb;

                uint32_t s = nb;
                fb = s;
                R4_AHEAD_BOT(s);
                R4Bot bnext = bots[s];
                nb = bnext.prev; ts = bnext.top;
                R4Top tnext = tops[ts];
                rb = tnext.width; bw = tnext.word;
            } else {
                remaining -= ra;
                bots[bs].top   = ts;
                tops[fa].prev  = tops[ts].prev;
                tops[ts]       = (R4Top){fa, ra, wd, bs};
                ts = fa;
                rb -= ra;

                uint32_t s = na;
                fa = s;
                R4_AHEAD_TOP(s);
                R4Top tnext = tops[s];
                na = tnext.prev; ra = tnext.width; aw = tnext.word; bs = tnext.other;
            }
        }
    }
    #undef R4_AHEAD_TOP
    #undef R4_AHEAD_BOT

    /* Right-to-left scan of P_1 via sb: store exact L-coord x in tops[ts].other (zero bottom[]!) */
    uint32_t x_bot = m;
    while (sb != 0u) {
        R4Bot bslot = bots[sb];
        uint32_t ts = bslot.top;
        x_bot -= tops[ts].width;
        tops[ts].other = x_bot;
        sb = bslot.prev;
    }

    k->bands_len = 0;
    k->spell_len = 0;
    uint32_t words = 0;
    const uint8_t * restrict arena = k->small_arena;

    uint32_t thin_start[256], ts_acc = 0;
    for (int a = 0; a < 256; a++) { thin_start[a] = ts_acc; ts_acc += thin[a]; }

    uint32_t cursors_rev[256];
    uint32_t rem_node = (sa != 0u) ? tops[sa].width : 0u;
    uint32_t kc = m;

    for (int a = 255; a >= 0; a--) {
        cursors_rev[a] = (uint32_t)k->bands_len;
        uint32_t start_a = thin_start[a];
        while (kc > start_a) {
            R4Top nd = tops[sa];
            uint32_t need_a = kc - start_a;
            uint32_t width = need_a < rem_node ? need_a : rem_node;
            rem_node -= width;
            kc -= width;
            uint32_t dst = nd.other + rem_node;
            uint32_t code;
            if (nd.word == 0u) {
                code = (uint32_t)a;
            } else {
                uint32_t w_idx = words++;
                code = 256u + w_idx;
                r4_dict_ensure(k, (size_t)code + reserve_sentinels + 16u);
                if (__builtin_expect((nd.word & 0x80000000u) == 0 && (nd.word >> 24) <= 15u, 1)) {
                    uint32_t wl = nd.word >> 24;
                    const uint8_t *wsrc = arena + (nd.word & 0x00FFFFFFu);
                    __m128i v = _mm_loadu_si128((const __m128i *)wsrc);
                    k->inline16[code] = v;
                    ((uint8_t *)&k->inline16[code])[wl] = (uint8_t)a;
                    k->w_len[code] = (uint16_t)(wl + 1u);
                } else {
                    uint32_t o0 = (uint32_t)k->spell_len;
                    if ((nd.word & 0x80000000u) == 0) {
                        uint32_t wl = nd.word >> 24;
                        const uint8_t *wsrc = arena + (nd.word & 0x00FFFFFFu);
                        r4_spell_ensure(k, wl + 1u);
                        uint8_t * restrict sp = k->long_spell + o0;
                        _mm256_storeu_si256((__m256i *)sp, _mm256_loadu_si256((const __m256i *)wsrc));
                        if (wl > 32u) _mm256_storeu_si256((__m256i *)(sp + 32), _mm256_loadu_si256((const __m256i *)(wsrc + 32)));
                        sp[wl] = (uint8_t)a;
                        k->spell_len = o0 + wl + 1u;
                    } else {
                        r4_append_rule(k, nd.word);
                        r4_spell_ensure(k, 1u);
                        k->long_spell[k->spell_len++] = (uint8_t)a;
                    }
                    uint32_t ln = (uint32_t)k->spell_len - o0;
                    uint64_t meta[2] = {(uint64_t)o0, (uint64_t)ln};
                    memcpy(&k->inline16[code], meta, 16);
                    k->w_len[code] = ln <= 65533u ? (uint16_t)ln : 65534u;
                }
            }
            if (__builtin_expect(k->bands_len == k->bands_cap, 0)) {
                k->bands_cap *= 2;
                k->bands = (Band *)realloc(k->bands, k->bands_cap * sizeof(Band));
            }
            k->bands[k->bands_len++] = (Band){kc - start_a, dst, code};

            if (rem_node == 0u) {
                sa = nd.prev;
                rem_node = (sa != 0u) ? tops[sa].width : 0u;
            }
        }
    }
    r4_dict_ensure(k, 256u + words + reserve_sentinels + 4u);

    uint32_t bits = m <= 1 ? 1u : (32u - (uint32_t)__builtin_clz(m - 1));
    uint32_t max_code = 255u + words + reserve_sentinels;
    int wide = max_code > (UINT32_MAX >> bits);

    for (int a = 255; a >= 0; a--) {
        uint32_t bi_start = cursors_rev[a];
        uint32_t bi_end   = (a > 0) ? cursors_rev[a - 1] : (uint32_t)k->bands_len;
        uint32_t prev_off = thin[a];
        const uint32_t * restrict qa = k->queues[a];
        for (uint32_t bi = bi_start; bi < bi_end; bi++) {
            Band b = k->bands[bi];
            uint32_t qa_off = b.end;
            uint32_t span = prev_off - qa_off;
            prev_off = qa_off;
            uint32_t dst = b.delta;
            uint32_t pack = wide ? 0u : (b.code << bits);
            if (__builtin_expect(wide, 0)) {
                for (uint32_t s = 0; s < span; s++) k->extra[dst + s] = b.code;
            }
            uint32_t * restrict edst = k->edges + dst;
            const uint32_t * restrict qsrc = qa + qa_off;
            for (uint32_t s = 0; s < span; s++) edst[s] = pack | qsrc[s];
        }
    }

    k->m = m; k->mark = mark; k->bits = bits; k->words = words; k->wide = wide;
}

static inline void r4_get_long(const R4Kernel *k, uint32_t code, const uint8_t **src_out, uint32_t *len_out) {
    uint64_t meta[2];
    memcpy(meta, &k->inline16[code], 16);
    *src_out = k->long_spell + (uint32_t)meta[0];
    *len_out = (uint32_t)meta[1];
}

static inline uint32_t r4_true_len(const R4Kernel *k, uint32_t cd) {
    uint32_t wl = k->w_len[cd];
    if (__builtin_expect(wl < 65534u, 1)) return wl;
    const uint8_t *s; uint32_t rl;
    r4_get_long(k, cd, &s, &rl);
    return rl;
}

/* Arm 6 Consumer: Zero-Pool 8-Lane BVSMC-2P without offsets[] */
static inline void r4_emit_bounded(R4Kernel *k, LState2P *st, int l, uint8_t **P_io, uint32_t cd) {
    uint8_t *P = *P_io;
    if (P >= st->hard_end[l]) return;
    uint32_t ln = k->w_len[cd];
    if (ln <= 16u && P < st->safe_end[l]) {
        _mm_storeu_si128((__m128i *)P, k->inline16[cd]);
        *P_io = P + ln;
        return;
    }
    uint8_t tmp16[16];
    const uint8_t *src; uint32_t rem;
    if (ln <= 16u) { _mm_storeu_si128((__m128i *)tmp16, k->inline16[cd]); src = tmp16; rem = ln; }
    else r4_get_long(k, cd, &src, &rem);
    uint32_t av = (uint32_t)(st->hard_end[l] - P);
    if (rem > av) rem = av;
    memcpy(P, src, rem);
    *P_io = P + rem;
}

__attribute__((noinline))
static void r4_cold_2p(R4Kernel *k, LState2P *st, int l, uint32_t *J_io, uint8_t **P_io, uint32_t cd) {
    if (!st->active_flag[l]) { *P_io = k->dummy_buf; return; }
    if (cd < st->sbase) {
        r4_emit_bounded(k, st, l, P_io, cd);
    } else if (st->next_idx < st->n_cyc) {
        uint32_t idx = st->next_idx++, sg = st->cyc_segs[idx];
        uint32_t off = st->cyc_off[idx], end = st->cyc_end[idx];
        uint8_t *P = st->out + off;
        st->hard_end[l] = st->out + end;
        st->safe_end[l] = (end - off >= 16u) ? (st->out + end - 15u) : (st->out + off);
        r4_emit_bounded(k, st, l, &P, st->saved_code[sg]);
        *P_io = P; *J_io = st->saved_next[sg];
    } else {
        st->active_flag[l] = 0; *P_io = k->dummy_buf;
        st->safe_end[l] = k->dummy_buf + 2048; st->hard_end[l] = k->dummy_buf + 4096;
        *J_io = k->m; st->active--;
    }
}

static void r4_bvsmc_emit_2p(R4Kernel *k, size_t n, uint32_t S_req, uint8_t *out) {
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
            b_init[l] = r4_true_len(k, st.saved_code[q]);
            j_init[l] = st.saved_next[q];
        } else { q_init[l] = MAX_CUTS; b_init[l] = 0; j_init[l] = m; }
    }
    uint32_t j0=j_init[0], j1=j_init[1], j2=j_init[2], j3=j_init[3];
    uint32_t j4=j_init[4], j5=j_init[5], j6=j_init[6], j7=j_init[7];
    uint32_t q0=q_init[0], q1=q_init[1], q2=q_init[2], q3=q_init[3];
    uint32_t q4=q_init[4], q5=q_init[5], q6=q_init[6], q7=q_init[7];
    uint32_t b0=b_init[0], b1=b_init[1], b2=b_init[2], b3=b_init[3];
    uint32_t b4=b_init[4], b5=b_init[5], b6=b_init[6], b7=b_init[7];

    #define R4_P1_STEP(J, Q, B, CD_EXPR, E_VAR) do { \
        uint32_t cd = (CD_EXPR); J = (E_VAR) & mask; uint32_t wl = w_len[cd]; B += wl; \
        if (__builtin_expect(wl >= 65534u || cd >= sbase, 0)) { \
            if (cd >= sbase) { \
                succ[Q] = (int32_t)(cd - sbase); seg_bytes[Q] = B; \
                if (p1_next < S_eff) { \
                    uint32_t nq = p1_next++; Q = nq; \
                    B = r4_true_len(k, st.saved_code[nq]); J = st.saved_next[nq]; \
                } else { Q = MAX_CUTS; B = 0; J = m; p1_act--; } \
            } else { B = (B - wl) + r4_true_len(k, cd); } \
        } \
    } while (0)

    if (!wide) {
        while (p1_act > 0) {
            uint32_t e0=edges[j0], e1=edges[j1], e2=edges[j2], e3=edges[j3];
            uint32_t e4=edges[j4], e5=edges[j5], e6=edges[j6], e7=edges[j7];
            R4_P1_STEP(j0,q0,b0,e0>>bits,e0); R4_P1_STEP(j1,q1,b1,e1>>bits,e1);
            R4_P1_STEP(j2,q2,b2,e2>>bits,e2); R4_P1_STEP(j3,q3,b3,e3>>bits,e3);
            R4_P1_STEP(j4,q4,b4,e4>>bits,e4); R4_P1_STEP(j5,q5,b5,e5>>bits,e5);
            R4_P1_STEP(j6,q6,b6,e6>>bits,e6); R4_P1_STEP(j7,q7,b7,e7>>bits,e7);
        }
    } else {
        while (p1_act > 0) {
            uint32_t e0=edges[j0],c0=extra[j0],e1=edges[j1],c1=extra[j1];
            uint32_t e2=edges[j2],c2=extra[j2],e3=edges[j3],c3=extra[j3];
            uint32_t e4=edges[j4],c4=extra[j4],e5=edges[j5],c5=extra[j5];
            uint32_t e6=edges[j6],c6=extra[j6],e7=edges[j7],c7=extra[j7];
            R4_P1_STEP(j0,q0,b0,c0,e0); R4_P1_STEP(j1,q1,b1,c1,e1);
            R4_P1_STEP(j2,q2,b2,c2,e2); R4_P1_STEP(j3,q3,b3,c3,e3);
            R4_P1_STEP(j4,q4,b4,c4,e4); R4_P1_STEP(j5,q5,b5,c5,e5);
            R4_P1_STEP(j6,q6,b6,c6,e6); R4_P1_STEP(j7,q7,b7,c7,e7);
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
            r4_emit_bounded(k, &st, l, &p_init2[l], st.saved_code[sg]);
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

    #define R4_STEP_2P(L, J, P, CD_EXPR, E_VAR) do { \
        uint32_t cd = (CD_EXPR); J = (E_VAR) & mask; uint32_t ln = w_len[cd]; \
        if (__builtin_expect(ln <= 16u && P < st.safe_end[L], 1)) { \
            _mm_storeu_si128((__m128i *)P, inline16[cd]); P += ln; \
        } else { uint8_t *ptmp = P; r4_cold_2p(k, &st, L, &(J), &ptmp, cd); P = ptmp; } \
    } while (0)

    if (!wide) {
        while (st.active > 0) {
            uint32_t e0=edges[j0], e1=edges[j1], e2=edges[j2], e3=edges[j3];
            uint32_t e4=edges[j4], e5=edges[j5], e6=edges[j6], e7=edges[j7];
            R4_STEP_2P(0,j0,p0,e0>>bits,e0); R4_STEP_2P(1,j1,p1,e1>>bits,e1);
            R4_STEP_2P(2,j2,p2,e2>>bits,e2); R4_STEP_2P(3,j3,p3,e3>>bits,e3);
            R4_STEP_2P(4,j4,p4,e4>>bits,e4); R4_STEP_2P(5,j5,p5,e5>>bits,e5);
            R4_STEP_2P(6,j6,p6,e6>>bits,e6); R4_STEP_2P(7,j7,p7,e7>>bits,e7);
        }
    } else {
        while (st.active > 0) {
            uint32_t e0=edges[j0],c0=extra[j0],e1=edges[j1],c1=extra[j1];
            uint32_t e2=edges[j2],c2=extra[j2],e3=edges[j3],c3=extra[j3];
            uint32_t e4=edges[j4],c4=extra[j4],e5=edges[j5],c5=extra[j5];
            uint32_t e6=edges[j6],c6=extra[j6],e7=edges[j7],c7=extra[j7];
            R4_STEP_2P(0,j0,p0,c0,e0); R4_STEP_2P(1,j1,p1,c1,e1);
            R4_STEP_2P(2,j2,p2,c2,e2); R4_STEP_2P(3,j3,p3,c3,e3);
            R4_STEP_2P(4,j4,p4,c4,e4); R4_STEP_2P(5,j5,p5,c5,e5);
            R4_STEP_2P(6,j6,p6,c6,e6); R4_STEP_2P(7,j7,p7,c7,e7);
        }
    }
    for (uint32_t q = 0; q < S_eff; q++) {
        uint32_t v = cuts[q]; edges[v] = saved_raw[q];
        if (wide) extra[v] = st.saved_code[q];
    }
    repeat_period(n, out, &acc);
}


/* =========================================================================
 * Bipolar Dead-Out Paged 1-Pass Splicing + Zero-Stage Kahn Unshuffling (BP-BVSMC-1P)
 * ========================================================================= */
#define BP_PAGE 2048u
#define BP_SAFE 2032u
#define BP_MAX_EXTENTS 640u
#define BP_MAX_SLOTS   640u
#define BP_OVF_CAP     128u /* 128 * 2 KiB = 256 KiB static ceiling */

typedef struct {
    uint8_t *ovf;
    size_t peak_ovf_pages;
    size_t evict_pages;
} BpPool;

static void bp_init(BpPool *bp) {
    memset(bp, 0, sizeof(*bp));
    bp->ovf = (uint8_t *)aligned_alloc(64, (size_t)BP_OVF_CAP * BP_PAGE);
}

static void bp_free(BpPool *bp) {
    free(bp->ovf);
}

typedef struct {
    uint32_t S_eff, sbase, next_q, active, seg0_len;
    size_t n;
    uint8_t *out;
    uint32_t n_phys;
    int32_t top_phys;
    uint32_t n_ext;
    uint32_t ovf_used;

    uint32_t saved_code[48], saved_next[48], q_l[8];
    int32_t succ[48], head[48], eid_l[8];
    uint8_t *base_l[8], *end_l[8];

    uint8_t *ext_ptr[BP_MAX_EXTENTS];
    uint16_t ext_len[BP_MAX_EXTENTS];
    int16_t  ext_phys[BP_MAX_EXTENTS];
    int16_t  ext_ovf[BP_MAX_EXTENTS];
    int32_t  ext_next[BP_MAX_EXTENTS];
} BpState;

static inline uint32_t bp_alloc_ext(BpPool *bp, BpState *st, uint8_t *p0_cur) {
    uint32_t eid = st->n_ext++;
    assert(eid < BP_MAX_EXTENTS);
    st->ext_next[eid] = -1;
    st->ext_len[eid]  = 0;
    size_t p0_used = (st->eid_l[0] == -1) ? (size_t)(p0_cur - st->out) : (size_t)st->seg0_len;
    int32_t min_phys = (int32_t)((p0_used + 4096u) / BP_PAGE) + 1;
    if (st->top_phys >= min_phys) {
        int32_t pid = st->top_phys--;
        st->ext_phys[eid] = (int16_t)pid;
        st->ext_ovf[eid]  = -1;
        st->ext_ptr[eid]  = st->out + (size_t)pid * BP_PAGE;
    } else {
        uint32_t oid = st->ovf_used++;
        assert(oid < BP_OVF_CAP);
        st->ext_phys[eid] = -1;
        st->ext_ovf[eid]  = (int16_t)oid;
        st->ext_ptr[eid]  = bp->ovf + (size_t)oid * BP_PAGE;
    }
    return eid;
}

static inline void bp_emit_slow(R4Kernel *k, BpPool *bp, BpState *st, int l, uint8_t **P_io, uint32_t cd, uint8_t *p0_cur) {
    uint8_t *P = *P_io;
    uint32_t ln = k->w_len[cd];
    if (__builtin_expect(ln <= 16u && P < st->end_l[l], 1)) {
        _mm_storeu_si128((__m128i *)P, k->inline16[cd]);
        *P_io = P + ln;
        return;
    }
    uint8_t tmp16[16];
    const uint8_t *src; uint32_t rem;
    if (ln <= 16u) { _mm_storeu_si128((__m128i *)tmp16, k->inline16[cd]); src = tmp16; rem = ln; }
    else r4_get_long(k, cd, &src, &rem);

    if (st->eid_l[l] == -1) {
        size_t hard_lim = (st->top_phys >= 0) ? ((size_t)(st->top_phys + 1) * BP_PAGE) : 0u;
        if (hard_lim > st->n) hard_lim = st->n;
        size_t cur_off = (size_t)(P - st->out);
        if (cur_off + rem <= hard_lim) {
            memcpy(P, src, rem);
            P += rem;
            st->end_l[0] = (hard_lim >= 32u) ? (st->out + hard_lim - 32u) : st->out;
            *P_io = P;
            return;
        }
        st->seg0_len = (uint32_t)cur_off;
        /* Switch eid_l[0] before bp_alloc_ext so p0_used reads seg0_len */
        st->eid_l[0] = -3;
        uint32_t ne = bp_alloc_ext(bp, st, P);
        st->head[0]  = (int32_t)ne;
        st->eid_l[0] = (int32_t)ne;
        st->base_l[0] = st->ext_ptr[ne];
        P = st->base_l[0];
        st->end_l[0] = P + BP_SAFE;
    }

    while (rem > 0) {
        int32_t ce = st->eid_l[l];
        uint32_t used = (uint32_t)(P - st->base_l[l]);
        if (used == BP_PAGE) {
            st->ext_len[ce] = BP_PAGE;
            uint32_t ne = bp_alloc_ext(bp, st, p0_cur);
            if (st->eid_l[0] == -1) {
                size_t slim = (st->top_phys >= 0) ? ((size_t)(st->top_phys + 1) * BP_PAGE) : 0u;
                if (slim > st->n) slim = st->n;
                st->end_l[0] = (slim >= 32u) ? (st->out + slim - 32u) : st->out;
            }
            st->ext_next[ce] = (int32_t)ne;
            st->eid_l[l] = (int32_t)ne;
            st->base_l[l] = st->ext_ptr[ne];
            P = st->base_l[l];
            st->end_l[l] = P + BP_SAFE;
            used = 0;
        }
        uint32_t av = BP_PAGE - used;
        uint32_t tk = rem < av ? rem : av;
        memcpy(P, src, tk);
        P += tk; src += tk; rem -= tk;
    }
    *P_io = P;
}

__attribute__((noinline))
static void bp_cold(R4Kernel *k, BpPool *bp, BpState *st, int l, uint32_t *J_io, uint8_t **P_io, uint32_t cd, uint8_t *p0_cur) {
    uint8_t *P = *P_io;
    if (cd < st->sbase) {
        if (st->eid_l[l] == -2) { *P_io = k->dummy_buf; return; }
        bp_emit_slow(k, bp, st, l, P_io, cd, p0_cur);
    } else {
        uint32_t hq = cd - st->sbase, cq = st->q_l[l];
        st->succ[cq] = (int32_t)hq;
        if (st->eid_l[l] == -1) {
            st->seg0_len = (uint32_t)(P - st->out);
            st->eid_l[l] = -3;
        } else if (st->eid_l[l] >= 0) {
            st->ext_len[st->eid_l[l]] = (uint16_t)(P - st->base_l[l]);
        }
        if (st->next_q < st->S_eff) {
            uint32_t nq = st->next_q++;
            st->q_l[l] = nq;
            uint32_t ne = bp_alloc_ext(bp, st, p0_cur);
            if (st->eid_l[0] == -1) {
                size_t slim = (st->top_phys >= 0) ? ((size_t)(st->top_phys + 1) * BP_PAGE) : 0u;
                if (slim > st->n) slim = st->n;
                st->end_l[0] = (slim >= 32u) ? (st->out + slim - 32u) : st->out;
            }
            st->head[nq]  = (int32_t)ne;
            st->eid_l[l]  = (int32_t)ne;
            st->base_l[l] = st->ext_ptr[ne];
            P = st->base_l[l];
            st->end_l[l]  = P + BP_SAFE;
            bp_emit_slow(k, bp, st, l, &P, st->saved_code[nq], p0_cur);
            *P_io = P;
            *J_io = st->saved_next[nq];
        } else {
            st->eid_l[l] = -2;
            *P_io = k->dummy_buf;
            st->end_l[l] = k->dummy_buf + 2048;
            *J_io = k->m;
            st->active--;
        }
    }
}

static void r4_bp_bvsmc_emit_1p(R4Kernel *k, BpPool *bp, size_t n, uint32_t S_req, uint8_t *out) {
    uint32_t * restrict edges = k->edges;
    uint32_t * restrict extra = k->extra;
    const __m128i * restrict inline16 = k->inline16;
    uint16_t * restrict w_len = k->w_len;
    const uint32_t m = k->m, mark = k->mark, bits = k->bits, mask = (1u << bits) - 1u;
    const int wide = k->wide;
    const uint32_t sbase = 256u + k->words;

    BpState st;
    uint32_t S_try = S_req < m ? S_req : m;
    if (S_try > 48u) S_try = 48u;
    st.sbase = sbase; st.next_q = 0; st.active = 0; st.seg0_len = 0;
    st.n = n; st.out = out;
    st.n_phys = (uint32_t)(n / BP_PAGE);
    st.top_phys = (int32_t)st.n_phys - 1;
    st.n_ext = 0; st.ovf_used = 0;

    uint32_t cuts[48], saved_raw[48];
    uint32_t n_unique = 0;
    for (uint32_t q = 0; q < S_try; q++) {
        uint32_t v = (uint32_t)((mark + ((uint64_t)q * m) / S_try) % m);
        int dup = 0;
        for (uint32_t prev = 0; prev < n_unique; prev++) if (cuts[prev] == v) { dup = 1; break; }
        if (!dup) cuts[n_unique++] = v;
    }
    st.S_eff = n_unique;

    for (uint32_t q = 0; q < st.S_eff; q++) {
        uint32_t v = cuts[q];
        uint32_t raw = edges[v];
        saved_raw[q] = raw;
        st.saved_next[q] = raw & mask;
        st.saved_code[q] = wide ? extra[v] : (raw >> bits);
        st.succ[q] = -1;
        st.head[q] = -1;
        w_len[sbase + q] = 65535u;
    }
    for (uint32_t q = 0; q < st.S_eff; q++) {
        uint32_t v = cuts[q], sc = sbase + q;
        if (wide) extra[v] = sc;
        else edges[v] = (sc << bits) | st.saved_next[q];
    }
    edges[m] = m;
    if (wide) extra[m] = 0;

    uint32_t j_init[8]; uint8_t *p_init[8];
    p_init[0] = out;
    for (int l = 0; l < 8; l++) {
        if (st.next_q < st.S_eff) {
            uint32_t q = st.next_q++;
            st.active++;
            st.q_l[l] = q;
            if (q == 0) {
                st.eid_l[0]  = -1;
                st.base_l[0] = out;
                p_init[0]    = out;
                size_t slim  = (st.top_phys >= 0) ? ((size_t)(st.top_phys + 1) * BP_PAGE) : 0u;
                if (slim > n) slim = n;
                st.end_l[0]  = (slim >= 32u) ? (out + slim - 32u) : out;
            } else {
                uint32_t ne = bp_alloc_ext(bp, &st, p_init[0]);
                st.head[q]   = (int32_t)ne;
                st.eid_l[l]  = (int32_t)ne;
                st.base_l[l] = st.ext_ptr[ne];
                p_init[l]    = st.base_l[l];
                st.end_l[l]  = p_init[l] + BP_SAFE;
                if (st.eid_l[0] == -1) {
                    size_t slim = (st.top_phys >= 0) ? ((size_t)(st.top_phys + 1) * BP_PAGE) : 0u;
                    if (slim > n) slim = n;
                    st.end_l[0] = (slim >= 32u) ? (out + slim - 32u) : out;
                }
            }
            bp_emit_slow(k, bp, &st, l, &p_init[l], st.saved_code[q], p_init[0]);
            j_init[l] = st.saved_next[q];
        } else {
            st.eid_l[l] = -2;
            p_init[l]   = k->dummy_buf;
            st.end_l[l] = k->dummy_buf + 2048;
            j_init[l]   = m;
        }
    }

    uint32_t j0=j_init[0], j1=j_init[1], j2=j_init[2], j3=j_init[3];
    uint32_t j4=j_init[4], j5=j_init[5], j6=j_init[6], j7=j_init[7];
    uint8_t * restrict p0=p_init[0], * restrict p1=p_init[1], * restrict p2=p_init[2], * restrict p3=p_init[3];
    uint8_t * restrict p4=p_init[4], * restrict p5=p_init[5], * restrict p6=p_init[6], * restrict p7=p_init[7];

    #define BP_STEP(L, J, P, CD_EXPR, E_VAR) do { \
        uint32_t cd = (CD_EXPR); J = (E_VAR) & mask; uint32_t ln = w_len[cd]; \
        if (__builtin_expect(ln <= 16u && P < st.end_l[L], 1)) { \
            _mm_storeu_si128((__m128i *)P, inline16[cd]); P += ln; \
        } else { uint8_t *pt = P; bp_cold(k, bp, &st, L, &(J), &pt, cd, p0); P = pt; } \
    } while (0)

    if (!wide) {
        while (st.active > 0) {
            uint32_t e0=edges[j0], e1=edges[j1], e2=edges[j2], e3=edges[j3];
            uint32_t e4=edges[j4], e5=edges[j5], e6=edges[j6], e7=edges[j7];
            BP_STEP(0,j0,p0,e0>>bits,e0); BP_STEP(1,j1,p1,e1>>bits,e1);
            BP_STEP(2,j2,p2,e2>>bits,e2); BP_STEP(3,j3,p3,e3>>bits,e3);
            BP_STEP(4,j4,p4,e4>>bits,e4); BP_STEP(5,j5,p5,e5>>bits,e5);
            BP_STEP(6,j6,p6,e6>>bits,e6); BP_STEP(7,j7,p7,e7>>bits,e7);
        }
    } else {
        while (st.active > 0) {
            uint32_t e0=edges[j0],c0=extra[j0],e1=edges[j1],c1=extra[j1];
            uint32_t e2=edges[j2],c2=extra[j2],e3=edges[j3],c3=extra[j3];
            uint32_t e4=edges[j4],c4=extra[j4],e5=edges[j5],c5=extra[j5];
            uint32_t e6=edges[j6],c6=extra[j6],e7=edges[j7],c7=extra[j7];
            BP_STEP(0,j0,p0,c0,e0); BP_STEP(1,j1,p1,c1,e1);
            BP_STEP(2,j2,p2,c2,e2); BP_STEP(3,j3,p3,c3,e3);
            BP_STEP(4,j4,p4,c4,e4); BP_STEP(5,j5,p5,c5,e5);
            BP_STEP(6,j6,p6,c6,e6); BP_STEP(7,j7,p7,c7,e7);
        }
    }
    #undef BP_STEP

    for (uint32_t q = 0; q < st.S_eff; q++) {
        uint32_t v = cuts[q];
        edges[v] = saved_raw[q];
        if (wide) extra[v] = st.saved_code[q];
    }

    uint32_t ord_e[BP_MAX_EXTENTS], ord_t0[BP_MAX_EXTENTS], ord_len[BP_MAX_EXTENTS];
    uint32_t n_ord = 0;
    size_t wr = st.seg0_len;
    for (int32_t e = st.head[0]; e >= 0 && wr < n; e = st.ext_next[e]) {
        uint32_t cl = st.ext_len[e];
        if (wr + cl > n) cl = (uint32_t)(n - wr);
        if (cl > 0) { ord_e[n_ord] = (uint32_t)e; ord_t0[n_ord] = (uint32_t)wr; ord_len[n_ord] = cl; n_ord++; wr += cl; }
    }
    int32_t cs = st.succ[0];
    uint32_t steps_s = 1;
    while (cs > 0 && steps_s < st.S_eff && wr < n) {
        for (int32_t e = st.head[cs]; e >= 0 && wr < n; e = st.ext_next[e]) {
            uint32_t cl = st.ext_len[e];
            if (wr + cl > n) cl = (uint32_t)(n - wr);
            if (cl > 0) { ord_e[n_ord] = (uint32_t)e; ord_t0[n_ord] = (uint32_t)wr; ord_len[n_ord] = cl; n_ord++; wr += cl; }
        }
        cs = st.succ[cs];
        steps_s++;
    }

    if (wr > st.seg0_len) {
        uint32_t total_tp = (uint32_t)((wr + BP_PAGE - 1u) / BP_PAGE);
        uint32_t first_tp = (uint32_t)(st.seg0_len / BP_PAGE);
        uint32_t n_slots  = total_tp > st.n_phys ? total_tp : st.n_phys;
        assert(n_slots <= BP_MAX_SLOTS);

        int16_t  phys_to_ord[BP_MAX_SLOTS];
        uint8_t  ref_cnt[BP_MAX_SLOTS];
        uint8_t  tp_done[BP_MAX_SLOTS];
        uint16_t tp_first_ord[BP_MAX_SLOTS], tp_last_ord[BP_MAX_SLOTS];

        memset(ref_cnt, 0, n_slots);
        memset(tp_done, 0, total_tp);
        for (uint32_t s = 0; s < n_slots; s++) phys_to_ord[s] = -1;
        for (uint32_t p = 0; p < total_tp; p++) { tp_first_ord[p] = 65535u; tp_last_ord[p] = 0; }

        uint8_t  ovf_ref[BP_OVF_CAP] = {0};
        uint16_t ovf_free[BP_OVF_CAP];
        uint32_t ovf_free_len = 0;
        uint8_t  ovf_active[BP_OVF_CAP] = {0};

        for (uint32_t oi = 0; oi < n_ord; oi++) {
            uint32_t e = ord_e[oi];
            uint32_t t0 = ord_t0[oi], t1 = t0 + ord_len[oi] - 1u;
            uint32_t p_lo = t0 / BP_PAGE, p_hi = t1 / BP_PAGE;
            uint8_t rc = (uint8_t)(p_hi - p_lo + 1u);
            for (uint32_t p = p_lo; p <= p_hi; p++) {
                if (oi < tp_first_ord[p]) tp_first_ord[p] = (uint16_t)oi;
                tp_last_ord[p] = (uint16_t)oi;
            }
            int16_t sp = st.ext_phys[e];
            if (sp >= 0) {
                phys_to_ord[sp] = (int16_t)oi;
                ref_cnt[sp]     = rc;
            } else {
                int16_t oid = st.ext_ovf[e];
                assert(oid >= 0 && oid < (int16_t)BP_OVF_CAP);
                ovf_ref[oid]    = rc;
                ovf_active[oid] = 1u;
            }
        }
        /* Any initial ovf slot from truncated periodic tail is immediately free */
        for (uint32_t oid = 0; oid < st.ovf_used; oid++) {
            if (!ovf_active[oid]) ovf_free[ovf_free_len++] = (uint16_t)oid;
        }

        uint16_t ready[BP_MAX_SLOTS];
        uint32_t r_len = 0, done_cnt = 0, need_cnt = total_tp - first_tp;
        for (uint32_t p = first_tp; p < total_tp; p++) {
            if (ref_cnt[p] == 0) ready[r_len++] = (uint16_t)p;
        }
        uint32_t scan_evict = first_tp, evicts = 0, peak_ovf = st.ovf_used;

        while (done_cnt < need_cnt) {
            if (r_len == 0) {
                int32_t chosen = -1;
                while (scan_evict < total_tp) {
                    if (ref_cnt[scan_evict] > 0 && !tp_done[scan_evict]) { chosen = (int32_t)scan_evict; break; }
                    scan_evict++;
                }
                if (chosen < 0) {
                    for (uint32_t s = first_tp; s < n_slots; s++) {
                        if (ref_cnt[s] > 0) { chosen = (int32_t)s; break; }
                    }
                }
                assert(chosen >= 0);
                int16_t oi = phys_to_ord[chosen];
                uint32_t e = ord_e[oi];
                uint32_t oid = (ovf_free_len > 0) ? ovf_free[--ovf_free_len] : st.ovf_used++;
                if (st.ovf_used > peak_ovf) peak_ovf = st.ovf_used;
                assert(oid < BP_OVF_CAP);

                uint8_t *dst_ovf = bp->ovf + (size_t)oid * BP_PAGE;
                memcpy(dst_ovf, st.ext_ptr[e], ord_len[oi]);
                st.ext_ptr[e]  = dst_ovf;
                st.ext_phys[e] = -1;
                st.ext_ovf[e]  = (int16_t)oid;
                ovf_ref[oid]   = ref_cnt[chosen];
                ref_cnt[chosen] = 0;
                evicts++;
                if ((uint32_t)chosen < total_tp && !tp_done[chosen]) {
                    ready[r_len++] = (uint16_t)chosen;
                }
                continue;
            }
            uint32_t p = ready[--r_len];
            if (tp_done[p]) continue;
            tp_done[p] = 1u;
            done_cnt++;

            uint32_t p_start = p * BP_PAGE;
            uint32_t p_end   = p_start + BP_PAGE;
            if (p_end > wr) p_end = (uint32_t)wr;
            uint32_t copy_from = p_start < st.seg0_len ? st.seg0_len : p_start;

            if (tp_first_ord[p] != 65535u) {
                for (uint32_t oi = tp_first_ord[p]; oi <= tp_last_ord[p]; oi++) {
                    uint32_t e = ord_e[oi];
                    uint32_t t0 = ord_t0[oi], t1 = t0 + ord_len[oi];
                    uint32_t lo = t0 > copy_from ? t0 : copy_from;
                    uint32_t hi = t1 < p_end ? t1 : p_end;
                    if (lo < hi) {
                        memcpy(out + lo, st.ext_ptr[e] + (lo - t0), hi - lo);
                    }
                    int16_t sp = st.ext_phys[e];
                    if (sp >= 0) {
                        if (--ref_cnt[sp] == 0 && sp >= (int16_t)first_tp && (uint32_t)sp < total_tp && !tp_done[sp]) {
                            ready[r_len++] = (uint16_t)sp;
                        }
                    } else {
                        int16_t oid = st.ext_ovf[e];
                        if (--ovf_ref[oid] == 0) ovf_free[ovf_free_len++] = (uint16_t)oid;
                    }
                }
            }
        }
        bp->peak_ovf_pages = peak_ovf;
        bp->evict_pages    = evicts;
    } else {
        bp->peak_ovf_pages = st.ovf_used;
        bp->evict_pages    = 0;
    }

    repeat_period(n, out, &wr);
}

/* =========================================================================
 * Self-Tests & Paired 25-Round 7-Arm Confrontation
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
    LKernel *lk, R4Kernel *rk, BpPool *bp, int arm,
    const uint16_t *recs, size_t n_recs, uint32_t n, uint32_t p,
    uint8_t *out, double *t_con, double *t_em
) {
    double t0 = now_sec(), t1, t2;
    switch (arm) {
        case 0:
            open_construct(lk, recs, n_recs, n, p, 16u, 0, 0, 0);
            t1 = now_sec();
            open_emit_orig(lk, n, out);
            t2 = now_sec();
            break;
        case 1:
            open_construct(lk, recs, n_recs, n, p, 16u, 1, 1, 0);
            t1 = now_sec();
            open_emit_u16_1lane(lk, n, out);
            t2 = now_sec();
            break;
        case 2:
            gim_rv_construct(lk, recs, n_recs, n, p, 16u, 64u);
            t1 = now_sec();
            l_bvsmc_emit_2p(lk, n, 64u, out);
            t2 = now_sec();
            break;
        case 3:
            r4_dgim_construct(rk, recs, n_recs, n, p, 16u, 64u);
            t1 = now_sec();
            r4_bvsmc_emit_2p(rk, n, 64u, out);
            t2 = now_sec();
            break;
        case 4:
            r4_dgim_construct(rk, recs, n_recs, n, p, 24u, 64u);
            t1 = now_sec();
            r4_bvsmc_emit_2p(rk, n, 64u, out);
            t2 = now_sec();
            break;
        case 5:
            r4_dgim_construct(rk, recs, n_recs, n, p, 16u, 48u);
            t1 = now_sec();
            r4_bp_bvsmc_emit_1p(rk, bp, n, 48u, out);
            t2 = now_sec();
            break;
        case 6:
            r4_dgim_construct(rk, recs, n_recs, n, p, 24u, 48u);
            t1 = now_sec();
            r4_bp_bvsmc_emit_1p(rk, bp, n, 48u, out);
            t2 = now_sec();
            break;
        default:
            abort();
    }
    *t_con = t1 - t0;
    *t_em  = t2 - t1;
}

static void run_adversarial_selftests(LKernel *lk, R4Kernel *rk, BpPool *bp) {
    uint8_t *L = (uint8_t *)malloc(60000);
    uint16_t *recs = (uint16_t *)malloc(60000 * 2u);
    uint8_t *ref = (uint8_t *)malloc(60000);
    uint8_t *guard = (uint8_t *)malloc(60064);
    uint8_t *out = guard + 32;

    struct { const char *name; size_t n; uint32_t p; } cases[8] = {
        {"n1_singleton", 1, 0},
        {"n5_multicycle", 5, 3},
        {"all_thin_H0_4k", 4096, 777},
        {"periodic_5k_single_byte", 5000, 1337},
        {"thick_primary_split", 81, 40},
        {"thick_primary_head", 65, 0},
        {"thick_primary_tail", 65, 59},
        {"periodic_shear_50k_gimrule", 49725, 25000},
    };

    for (int ci = 0; ci < 8; ci++) {
        size_t n = cases[ci].n;
        uint32_t p = cases[ci].p;
        if (ci == 0) { L[0] = 'z'; }
        else if (ci == 1) { memcpy(L, "babaa", 5); }
        else if (ci == 2) { for (size_t i = 0; i < 4096; i++) L[i] = (uint8_t)((i / 8) % 19 + 'a'); }
        else if (ci == 3) { memset(L, 'q', 5000); }
        else if (ci == 4) { memset(L, 'a', 40); L[40] = 'b'; memset(L + 41, 'a', 40); }
        else if (ci == 5 || ci == 6) { memset(L, 'm', 60); memset(L + 60, 'n', 5); }
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

        for (int arm = 0; arm < 7; arm++) {
            memset(guard, 0xA5, 60064);
            double tc = 0, te = 0;
            run_arm(lk, rk, bp, arm, recs, nr, (uint32_t)n, p, out, &tc, &te);
            assert(memcmp(out, ref, n) == 0);
            for (int g = 0; g < 32; g++) {
                assert(guard[g] == 0xA5 && out[n + g] == 0xA5);
            }
            if (ci == 7 && arm >= 3) assert(rk->rules_len > 0);
        }
    }
    free(L); free(recs); free(ref); free(guard);
    printf("[PASS] Adversarial self-tests (8/8 incl. n=1, n=5 multi-cycle, 4k H=0 all-thin, 50k >64B EpRule, left/right overrun canaries) across all 7 arms.\n\n");
}

int main(int argc, char **argv) {
    const char *dir = (argc > 1) ? argv[1] : "/tmp/admitted-bwt/admitted-bwt";
    const int rounds = (argc > 2) ? atoi(argv[2]) : 25;
    const char *names[6] = {"block-0.L", "block-2.L", "block-85.L", "block-143.L", "block-210.L", "block-207.L"};
    const uint32_t ps[6]  = {264045,      320807,      325088,       802868,        791429,        205180};
    const char *arm_names[7] = {
        "Open-qi16",
        "Open+U16Q-1L",
        "GIM-RV(L16)+BV2P",
        "R4-DGIM(L16)+BV2P",
        "W24-R4+BV2P",
        "L16-R4+BP1P",
        "W24-R4+BP1P"
    };

    LKernel lk;  lkernel_init(&lk, 1000000);
    R4Kernel rk; r4_init(&rk, 1000000);
    BpPool bp;   bp_init(&bp);
    run_adversarial_selftests(&lk, &rk, &bp);

    uint8_t *L = (uint8_t *)malloc(1000000);
    uint16_t *recs = (uint16_t *)malloc(1000000 * 2u);
    uint8_t *ref = (uint8_t *)malloc(1000000);
    uint8_t *guard = (uint8_t *)malloc(1000128);
    uint8_t *out = guard + 32;

    int pass_ovf = 0, pass_c2_60 = 0, pass_c2_65 = 0;
    int pass_c3_92 = 0, pass_c3_lt1 = 0;
    int pass_c4_94 = 0, pass_c4_lt1 = 0, pass_c4_con92 = 0;

    printf("Block        | Arm                |  Con(ms) Emit(ms)  Tot(ms) |  Tot/Op  Tot/R4 | Active(KiB) Ovf(KiB)\n");
    printf("---------------------------------------------------------------------------------------------------------\n");

    for (int bi = 0; bi < 6; bi++) {
        char path[512];
        snprintf(path, sizeof(path), "%s/%s", dir, names[bi]);
        FILE *fp = fopen(path, "rb");
        if (!fp) { fprintf(stderr, "Cannot open %s\n", path); return 1; }
        size_t n = fread(L, 1, 1000000, fp);
        fclose(fp);

        size_t nr = to_capped_records(L, n, recs);
        dense_lf_reference(L, n, ps[bi], ref);

        double t_con[7][64], t_em[7][64], t_tot[7][64];
        size_t act_bytes[7] = {0}, ovf_bytes[7] = {0};
        assert(rounds <= 64);

        for (int r = -3; r < rounds; r++) {
            for (int step = 0; step < 7; step++) {
                int arm = (step + (r + 3)) % 7;
                if (r == -3) memset(guard, 0xA5, n + 64);
                double tc = 0, te = 0;
                run_arm(&lk, &rk, &bp, arm, recs, nr, (uint32_t)n, ps[bi], out, &tc, &te);
                if (r == -3) {
                    if (memcmp(out, ref, n) != 0) {
                        fprintf(stderr, "BYTE MISMATCH on %s arm %d!\n", names[bi], arm);
                        return 1;
                    }
                    for (int g = 0; g < 32; g++) {
                        assert(guard[g] == 0xA5 && out[n + g] == 0xA5);
                    }
                    if (arm == 0 || arm == 1) act_bytes[arm] = lkernel_active_bytes(&lk, 1, 0);
                    else if (arm == 2) act_bytes[arm] = lkernel_active_bytes(&lk, 0, 1);
                    else if (arm == 3 || arm == 4) act_bytes[arm] = r4_active_bytes(&rk);
                    else {
                        ovf_bytes[arm] = bp.peak_ovf_pages * BP_PAGE;
                        act_bytes[arm] = r4_active_bytes(&rk) + ovf_bytes[arm];
                    }
                }
                if (r >= 0) {
                    t_con[arm][r] = tc * 1e3;
                    t_em[arm][r]  = te * 1e3;
                    t_tot[arm][r] = (tc + te) * 1e3;
                }
            }
        }

        double mc[7], me[7], mt[7];
        for (int a = 0; a < 7; a++) {
            mc[a] = median(t_con[a], rounds);
            me[a] = median(t_em[a], rounds);
            mt[a] = median(t_tot[a], rounds);
        }

        for (int a = 0; a < 7; a++) {
            printf("%-12s | %-18s | %8.3f %8.3f %8.3f | %6.3fx %6.3fx | %10.1f %8.1f\n",
                   a == 0 ? names[bi] : "", arm_names[a],
                   mc[a], me[a], mt[a], mt[a] / mt[0], mt[a] / mt[3],
                   act_bytes[a] / 1024.0, ovf_bytes[a] / 1024.0);
        }
        printf("---------------------------------------------------------------------------------------------------------\n");

        if (ovf_bytes[6] <= 256u * 1024u) pass_ovf++;
        if (mt[6] <= 0.60 * mt[0]) pass_c2_60++;
        if (mt[6] <= 0.65 * mt[0]) pass_c2_65++;
        if (mt[6] <= 0.92 * mt[2]) pass_c3_92++;
        if (mt[6] <  1.00 * mt[2]) pass_c3_lt1++;
        if (mt[6] <= 0.94 * mt[3]) pass_c4_94++;
        if (mt[6] <  1.00 * mt[3]) pass_c4_lt1++;
        if (mc[6] <= 0.92 * mc[3]) pass_c4_con92++;
    }

    int all_pass = (pass_ovf == 6)
                && (pass_c2_60 >= 5) && (pass_c2_65 == 6)
                && (pass_c3_92 >= 5) && (pass_c3_lt1 == 6)
                && (pass_c4_94 >= 5) && (pass_c4_lt1 == 6) && (pass_c4_con92 >= 5);

    printf("\n=== PRE-REGISTERED GATE G5 SUMMARY ===\n");
    printf("  (1)  W24-R4+BP1P Peak Overflow <= 256 KiB              : %d/6 blocks (need  6/6)\n", pass_ovf);
    printf("  (2a) W24-R4+BP1P Tot <= 0.60x Open-qi16 Tot            : %d/6 blocks (need >= 5/6)\n", pass_c2_60);
    printf("  (2b) W24-R4+BP1P Tot <= 0.65x Open-qi16 Tot            : %d/6 blocks (need  6/6)\n", pass_c2_65);
    printf("  (3a) W24-R4+BP1P Tot <= 0.92x GIM-RV(L16)+BV2P Tot     : %d/6 blocks (need >= 5/6)\n", pass_c3_92);
    printf("  (3b) W24-R4+BP1P Tot <  1.00x GIM-RV(L16)+BV2P Tot     : %d/6 blocks (need  6/6)\n", pass_c3_lt1);
    printf("  (4a) W24-R4+BP1P Tot <= 0.94x R4-DGIM(L16)+BV2P Tot    : %d/6 blocks (need >= 5/6)\n", pass_c4_94);
    printf("  (4b) W24-R4+BP1P Tot <  1.00x R4-DGIM(L16)+BV2P Tot    : %d/6 blocks (need  6/6)\n", pass_c4_lt1);
    printf("  (4c) W24-R4+BP1P Con <= 0.92x R4-DGIM(L16)+BV2P Con    : %d/6 blocks (need >= 5/6)\n", pass_c4_con92);
    printf("  STRICT GATE G5 VERDICT: %s\n", all_pass ? "PASS" : "FAIL");

    free(L); free(recs); free(ref); free(guard);
    bp_free(&bp); r4_free(&rk); lkernel_free(&lk);
    return 0;
}
