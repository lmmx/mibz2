/*
 * Gate G5 Paid-Cost Falsifier & Controlled 7-Arm Confrontation Harness for Carbon Cycle 11:
 * Tight Radix-Slab FlatQ (Runs.{counts, per}) + Primary-Split RunQ + 4KiB Bipolar Dead-Out 1P Splicing (FlatQ-BP4K)
 * vs Shipped Open (qi16), Open+U16Q, Cycle 7 GIM-RV+BV2P, Cycle 9 R4-DGIM+BV2P, and Cycle 10 W24-BP1P(2K).
 *
 * PRE-REGISTERED PASS/FAIL RULE (frozen before compilation and execution):
 * Across all six authenticated Wikidata BWT blocks (block-0, 2, 85, 143, 210, 207),
 * 25 paired interleaved rounds after 3 warmup rounds (rotating arm order (step + r + 3) % 7):
 *
 *   Arm 0: Open-qi16           — exact C mirror of shipped src/open.rs
 *   Arm 1: Open+U16Q-1L        — calibrated Open+U16Q (#664/#666)
 *   Arm 2: GIM-RV(L16)+BV2P    — Cycle 7 promoted baseline
 *   Arm 3: R4-DGIM(L16)+BV2P   — Cycle 9 baseline
 *   Arm 4: W24-R4+BP1P(2K)     — Cycle 10 baseline (U24Q + R4-DGIM + 2KiB Bipolar 1P, 04b68a7)
 *   Arm 5: PaidPre-FlatQ+BP4K  — Self-contained paid records pre-pass (counts+per inside construct) + FlatQ + BP4K(48)
 *   Arm 6: RunsMeta-FlatQ+BP4K — Production Runs.{counts,per} (src/runs.rs:91-92) + FlatQ + p-split + BP4K(48)
 *
 * PASS iff ALL FOUR hold:
 *   (1) Exactness, Overflow Ceiling (<= 256 KiB), & Full Sanitizer Cleanliness:
 *       - Arms 1..6 match dense LF byte-for-byte on 6/6 blocks (5,399,692 B), pass 8/8 adversarial
 *         self-tests (incl. multi-record pos+128 straddle around p and 32B left/right overrun canaries),
 *         and exit with 0 errors and 0 leaks under -fsanitize=address,undefined;
 *       - Arm 6 BP4K peak recycled overflow buffer <= 256 KiB (<= 64 pages of 4 KiB) on 6/6 blocks;
 *   (2) Sub-0.58x End-to-End Speedup over Shipped Open-qi16:
 *       - Arm 6 (RunsMeta-FlatQ+BP4K) total median <= 0.58x Arm 0 (Open-qi16) total on all 6/6 blocks;
 *   (3) End-to-End Speedup over Cycle 7 GIM-RV(L16)+BV2P:
 *       - Arm 6 (RunsMeta-FlatQ+BP4K) total median <= 0.85x Arm 2 (GIM-RV+BV2P) total on >= 5/6 blocks
 *         (and < 0.90x on 6/6 blocks);
 *   (4) Direct Gain over Cycle 10 W24-R4+BP1P(2K):
 *       - (4a) Arm 6 total median     <= 0.92x Arm 4 total     on >= 5/6 blocks (and < 1.00x on 6/6), AND
 *              Arm 6 construct median <= 0.88x Arm 4 construct on >= 5/6 blocks;
 *       - (4b) Arm 5 (PaidPre-FlatQ+BP4K, self-contained paid pre-pass) total median < 1.00x Arm 4 total
 *              on >= 5/6 blocks.
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

#define BP4K_PAGE        4096u
#define BP4K_SAFE        4080u
#define BP4K_MAX_EXTENTS 400u
#define BP4K_MAX_SLOTS   400u
#define BP4K_OVF_CAP     96u /* 96 * 4 KiB = 384 KiB static guard; G5 checks <= 64 pages (256 KiB) */

typedef struct {
    uint32_t *flat_q;
    size_t flat_cap;
    size_t last_slab_entries;
    uint8_t *ovf4k;
    size_t peak_ovf4k_pages;
    size_t evict4k_pages;
} FlatQ4K;

static void fq4k_init(FlatQ4K *fq, size_t max_n) {
    memset(fq, 0, sizeof(*fq));
    fq->flat_cap = max_n + 256u * 64u;
    fq->flat_q   = (uint32_t *)aligned_alloc(64, fq->flat_cap * sizeof(uint32_t));
    fq->ovf4k    = (uint8_t *)aligned_alloc(64, (size_t)BP4K_OVF_CAP * BP4K_PAGE);
}

static void fq4k_free(FlatQ4K *fq) {
    free(fq->flat_q);
    free(fq->ovf4k);
}

static inline void store_u24_raw(uint32_t *dst, uint32_t start) {
    const __m256i i0 = _mm256_setr_epi32(0, 1, 2, 3, 4, 5, 6, 7);
    const __m256i i1 = _mm256_setr_epi32(8, 9, 10, 11, 12, 13, 14, 15);
    const __m256i i2 = _mm256_setr_epi32(16, 17, 18, 19, 20, 21, 22, 23);
    __m256i b = _mm256_set1_epi32((int32_t)start);
    _mm256_storeu_si256((__m256i *)(dst + 0),  _mm256_add_epi32(b, i0));
    _mm256_storeu_si256((__m256i *)(dst + 8),  _mm256_add_epi32(b, i1));
    _mm256_storeu_si256((__m256i *)(dst + 16), _mm256_add_epi32(b, i2));
}

static void flatq_w24_r4_construct(
    R4Kernel *k, FlatQ4K *fq,
    const uint16_t *records, size_t n_recs,
    const uint32_t counts_in[256], const uint32_t per_in[256],
    uint32_t n, uint32_t p, uint32_t reserve_sentinels
) {
    uint32_t counts_local[256], per_local[256];
    const uint32_t * restrict counts = counts_in;
    const uint32_t * restrict per    = per_in;

    if (!counts_in || !per_in) {
        memset(counts_local, 0, sizeof(counts_local));
        memset(per_local,    0, sizeof(per_local));
        for (size_t ri = 0; ri < n_recs; ri++) {
            uint16_t rc = records[ri];
            uint8_t b = (uint8_t)(rc >> 8);
            counts_local[b] += (uint32_t)(rc & 255u);
            per_local[b]++;
        }
        counts = counts_local;
        per    = per_local;
    }

    uint32_t *q_start[256], *q_cur[256];
    uint32_t thick_rows[256] = {0}, thick[256] = {0};

    size_t off_q = 0, tight_sum = 0;
    for (int a = 0; a < 256; a++) {
        uint64_t by_per = (uint64_t)per[a] * 24ULL;
        uint32_t ub = (counts[a] < by_per) ? counts[a] : (uint32_t)by_per;
        q_start[a] = fq->flat_q + off_q;
        q_cur[a]   = q_start[a];
        tight_sum += (size_t)ub + 1u;
        off_q     += (size_t)ub + 36u;
    }
    assert(off_q <= fq->flat_cap);
    fq->last_slab_entries = tight_sum;
    k->strips_len = 0;

    uint32_t m = 0, mark = 0, pos = 0;
    size_t i = 0;

    #define FQ_PUSH24(C, W) do { \
        uint32_t *_d = q_cur[(C)]; \
        store_u24_raw(_d, m); \
        q_cur[(C)] = _d + (W); \
        m += (W); \
    } while (0)

    /* Phase 1A: Fast pre-primary scan while pos + 128 <= p */
    while (i < n_recs && pos + 128u <= p) {
        uint16_t rec = records[i++];
        uint8_t c = (uint8_t)(rec >> 8);
        uint32_t width = (uint32_t)(rec & 255u);
        if (__builtin_expect(i < n_recs && (uint8_t)(records[i] >> 8) == c, 0)) {
            do { width += (uint32_t)(records[i++] & 255u); } while (i < n_recs && (uint8_t)(records[i] >> 8) == c);
            if (pos + width > p) {
                uint32_t end = pos + width;
                if (width <= 24u) {
                    mark = m + (p - pos);
                    FQ_PUSH24(c, width);
                } else {
                    uint32_t cur_all = (uint32_t)(q_cur[c] - q_start[c]) + thick_rows[c];
                    if (pos < p) { uint32_t w1 = p - pos; r4_push_strip(k, pos, cur_all, w1, c); thick[c]++; cur_all += w1; thick_rows[c] += w1; }
                    mark = m; FQ_PUSH24(c, 1u); cur_all += 1u;
                    if (p + 1 < end) { uint32_t w2 = end - p - 1; r4_push_strip(k, p + 1, cur_all, w2, c); thick[c]++; thick_rows[c] += w2; }
                }
                pos = end;
                goto post_primary_scan;
            }
        }
        if (__builtin_expect(width <= 24u, 1)) {
            FQ_PUSH24(c, width);
        } else {
            uint32_t cur_all = (uint32_t)(q_cur[c] - q_start[c]) + thick_rows[c];
            r4_push_strip(k, pos, cur_all, width, c);
            thick[c]++; thick_rows[c] += width;
        }
        pos += width;
    }

    /* Phase 1B: Straddle window around p */
    while (i < n_recs && pos <= p) {
        uint16_t rec = records[i++];
        uint8_t c = (uint8_t)(rec >> 8);
        uint32_t width = (uint32_t)(rec & 255u);
        while (i < n_recs && (uint8_t)(records[i] >> 8) == c) width += (uint32_t)(records[i++] & 255u);
        uint32_t end = pos + width;
        if (width <= 24u) {
            if (p >= pos && p < end) mark = m + (p - pos);
            FQ_PUSH24(c, width);
        } else if (p >= pos && p < end) {
            uint32_t cur_all = (uint32_t)(q_cur[c] - q_start[c]) + thick_rows[c];
            if (pos < p) { uint32_t w1 = p - pos; r4_push_strip(k, pos, cur_all, w1, c); thick[c]++; cur_all += w1; thick_rows[c] += w1; }
            mark = m; FQ_PUSH24(c, 1u); cur_all += 1u;
            if (p + 1 < end) { uint32_t w2 = end - p - 1; r4_push_strip(k, p + 1, cur_all, w2, c); thick[c]++; thick_rows[c] += w2; }
        } else {
            uint32_t cur_all = (uint32_t)(q_cur[c] - q_start[c]) + thick_rows[c];
            r4_push_strip(k, pos, cur_all, width, c);
            thick[c]++; thick_rows[c] += width;
        }
        pos = end;
    }

post_primary_scan:
    /* Phase 1C: Fast post-primary scan (pos > p guaranteed: zero p checks) */
    while (i < n_recs) {
        uint16_t rec = records[i++];
        uint8_t c = (uint8_t)(rec >> 8);
        uint32_t width = (uint32_t)(rec & 255u);
        if (__builtin_expect(i < n_recs && (uint8_t)(records[i] >> 8) == c, 0)) {
            do { width += (uint32_t)(records[i++] & 255u); } while (i < n_recs && (uint8_t)(records[i] >> 8) == c);
        }
        if (__builtin_expect(width <= 24u, 1)) {
            FQ_PUSH24(c, width);
        } else {
            uint32_t cur_all = (uint32_t)(q_cur[c] - q_start[c]) + thick_rows[c];
            r4_push_strip(k, pos, cur_all, width, c);
            thick[c]++; thick_rows[c] += width;
        }
        pos += width;
    }
    #undef FQ_PUSH24

    uint32_t thin[256], first[256], write[256], tot = 0, tw = 0;
    for (int a = 0; a < 256; a++) {
        thin[a]      = (uint32_t)(q_cur[a] - q_start[a]);
        k->q_len[a]  = thin[a];
        first[a]     = tot; tot += counts[a];
        write[a]     = tw;  tw  += thick[a];
    }

    size_t n_strips = k->strips_len;
    if (n_strips > k->img_cap) {
        k->img_cap = n_strips * 2 + 16;
        k->image_order = (uint32_t *)realloc(k->image_order, k->img_cap * 4u);
    }
    if (n_strips > k->strip_top_cap) {
        k->strip_top_cap = n_strips * 2 + 16;
        k->strip_top = (uint32_t *)realloc(k->strip_top, k->strip_top_cap * 4u);
    }
    for (size_t si = 0; si < n_strips; si++) {
        uint8_t a = k->strips[si].byte;
        k->strips[si].to += first[a];
        k->image_order[write[a]++] = (uint32_t)si;
    }

    size_t need_slots = R4_PAD_SLOTS + 3 * n_strips + 16;
    if (need_slots > k->tops_cap) {
        while (need_slots > k->tops_cap) k->tops_cap *= 2;
        free(k->tops); k->tops = (R4Top *)aligned_alloc(64, k->tops_cap * sizeof(R4Top));
    }
    if (need_slots > k->bots_cap) {
        while (need_slots > k->bots_cap) k->bots_cap *= 2;
        free(k->bots); k->bots = (R4Bot *)aligned_alloc(64, k->bots_cap * sizeof(R4Bot));
    }
    R4Top * restrict tops = k->tops;
    R4Bot * restrict bots = k->bots;
    uint32_t * restrict strip_top = k->strip_top;
    for (uint32_t pad = 0; pad < R4_PAD_SLOTS; pad++) { tops[pad] = (R4Top){0,0,0,0}; bots[pad] = (R4Bot){0,0}; }
    k->rules_len = 0; k->arena_len = 256u;

    const uint32_t p0_base = R4_PAD_SLOTS;
    const uint32_t a0_base = p0_base + (uint32_t)n_strips + 2u;
    uint32_t t_cur = a0_base, b_cur = R4_PAD_SLOTS, xc = 0;
    for (size_t si = 0; si < n_strips; si++) {
        Strip s = k->strips[si];
        if (xc < s.from) {
            tops[t_cur] = (R4Top){t_cur - 1u, s.from - xc, 0u, b_cur};
            bots[b_cur] = (R4Bot){b_cur - 1u, t_cur};
            t_cur++; b_cur++;
        }
        strip_top[si] = t_cur;
        tops[t_cur] = (R4Top){t_cur - 1u, s.width, (1u << 24) | (uint32_t)s.byte, 0u};
        t_cur++;
        xc = s.from + s.width;
    }
    if (xc < n) {
        tops[t_cur] = (R4Top){t_cur - 1u, n - xc, 0u, b_cur};
        bots[b_cur] = (R4Bot){b_cur - 1u, t_cur};
        t_cur++; b_cur++;
    }
    const uint32_t a0_end = t_cur;
    uint32_t p0_cur = p0_base; xc = 0;
    for (size_t ii = 0; ii < n_strips; ii++) {
        uint32_t idx = k->image_order[ii]; Strip s = k->strips[idx];
        if (xc < s.to) {
            tops[p0_cur] = (R4Top){p0_cur - 1u, s.to - xc, 0u, b_cur};
            bots[b_cur]  = (R4Bot){b_cur - 1u, p0_cur};
            p0_cur++; b_cur++;
        }
        uint32_t st_i = strip_top[idx];
        tops[st_i].other = b_cur;
        bots[b_cur] = (R4Bot){b_cur - 1u, st_i};
        b_cur++;
        xc = s.to + s.width;
    }
    if (xc < n) {
        tops[p0_cur] = (R4Top){p0_cur - 1u, n - xc, 0u, b_cur};
        bots[b_cur]  = (R4Bot){b_cur - 1u, p0_cur};
        p0_cur++; b_cur++;
    }
    if (p0_cur > p0_base) tops[p0_base].prev = 0u;
    if (b_cur > R4_PAD_SLOTS) bots[R4_PAD_SLOTS].prev = 0u;
    if (a0_end > a0_base) tops[a0_base].prev = (p0_cur > p0_base) ? (p0_cur - 1u) : 0u;
    k->active_tops = (size_t)(p0_cur - p0_base) + (size_t)(a0_end - a0_base);
    k->active_bots = (size_t)(b_cur - R4_PAD_SLOTS);

    uint32_t sa = a0_end - 1u, sb = b_cur - 1u, remaining = n;
    while (remaining > 0) {
        R4Top ta = tops[sa];
        uint32_t na = ta.prev, ra = ta.width, aw = ta.word, bs = ta.other;
        R4Bot bb_slot = bots[sb];
        uint32_t nb = bb_slot.prev, ts = bb_slot.top;
        R4Top tb = tops[ts];
        uint32_t rb = tb.width, bw = tb.word;
        uint32_t fa = sa, fb = sb;
        _mm_prefetch((const char *)&bots[tops[sa > R4_FAR ? sa - R4_FAR : 0u].other], _MM_HINT_T0);
        _mm_prefetch((const char *)&tops[bots[sb > R4_FAR ? sb - R4_FAR : 0u].top],   _MM_HINT_T0);
        if (bs == sb) { remaining -= ra; sa = na; sb = nb; continue; }
        for (;;) {
            uint32_t wd = r4_join(k, aw, bw);
            if (ra == rb) {
                remaining -= ra;
                tops[ts].width = ra; tops[ts].word = wd; tops[ts].other = bs; bots[bs].top = ts;
                sa = na; sb = nb; break;
            } else if (ra > rb) {
                remaining -= rb;
                tops[ts].width = rb; tops[ts].word = wd; tops[ts].other = bs;
                bots[fb].prev = bots[bs].prev; bots[bs] = (R4Bot){fb, ts}; bs = fb; ra -= rb;
                uint32_t s = nb; fb = s;
                _mm_prefetch((const char *)&tops[bots[s > R4_FAR ? s - R4_FAR : 0u].top], _MM_HINT_T0);
                R4Bot bnext = bots[s]; nb = bnext.prev; ts = bnext.top;
                R4Top tnext = tops[ts]; rb = tnext.width; bw = tnext.word;
            } else {
                remaining -= ra;
                bots[bs].top = ts; tops[fa].prev = tops[ts].prev;
                tops[ts] = (R4Top){fa, ra, wd, bs}; ts = fa; rb -= ra;
                uint32_t s = na; fa = s;
                _mm_prefetch((const char *)&bots[tops[s > R4_FAR ? s - R4_FAR : 0u].other], _MM_HINT_T0);
                R4Top tnext = tops[s]; na = tnext.prev; ra = tnext.width; aw = tnext.word; bs = tnext.other;
            }
        }
    }

    uint32_t x_bot = m;
    while (sb != 0u) {
        R4Bot bslot = bots[sb]; uint32_t ts = bslot.top;
        x_bot -= tops[ts].width; tops[ts].other = x_bot; sb = bslot.prev;
    }

    k->bands_len = 0; k->spell_len = 0;
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
            rem_node -= width; kc -= width;
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
                    k->inline16[code] = _mm_loadu_si128((const __m128i *)wsrc);
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
            if (rem_node == 0u) { sa = nd.prev; rem_node = (sa != 0u) ? tops[sa].width : 0u; }
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
        const uint32_t * restrict qa = q_start[a];
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

/* 4 KiB Page-Granular Bipolar Dead-Out 1P Splicing (BP4K-BVSMC-1P) */
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

    uint8_t *ext_ptr[BP4K_MAX_EXTENTS];
    uint16_t ext_len[BP4K_MAX_EXTENTS];
    int16_t  ext_phys[BP4K_MAX_EXTENTS];
    int16_t  ext_ovf[BP4K_MAX_EXTENTS];
    int32_t  ext_next[BP4K_MAX_EXTENTS];
} Bp4KState;

static inline uint32_t bp4k_alloc_ext(FlatQ4K *fq, Bp4KState *st, uint8_t *p0_cur) {
    uint32_t eid = st->n_ext++;
    assert(eid < BP4K_MAX_EXTENTS);
    st->ext_next[eid] = -1;
    st->ext_len[eid]  = 0;
    size_t p0_used = (st->eid_l[0] == -1) ? (size_t)(p0_cur - st->out) : (size_t)st->seg0_len;
    int32_t min_phys = (int32_t)((p0_used + 2u * BP4K_PAGE) / BP4K_PAGE) + 1;
    if (st->top_phys >= min_phys) {
        int32_t pid = st->top_phys--;
        st->ext_phys[eid] = (int16_t)pid;
        st->ext_ovf[eid]  = -1;
        st->ext_ptr[eid]  = st->out + (size_t)pid * BP4K_PAGE;
    } else {
        uint32_t oid = st->ovf_used++;
        assert(oid < BP4K_OVF_CAP);
        st->ext_phys[eid] = -1;
        st->ext_ovf[eid]  = (int16_t)oid;
        st->ext_ptr[eid]  = fq->ovf4k + (size_t)oid * BP4K_PAGE;
    }
    return eid;
}

static inline void bp4k_emit_slow(R4Kernel *k, FlatQ4K *fq, Bp4KState *st, int l, uint8_t **P_io, uint32_t cd, uint8_t *p0_cur) {
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
        size_t hard_lim = (st->top_phys >= 0) ? ((size_t)(st->top_phys + 1) * BP4K_PAGE) : 0u;
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
        st->eid_l[0] = -3;
        uint32_t ne = bp4k_alloc_ext(fq, st, P);
        st->head[0]   = (int32_t)ne;
        st->eid_l[0]  = (int32_t)ne;
        st->base_l[0] = st->ext_ptr[ne];
        P = st->base_l[0];
        st->end_l[0]  = P + BP4K_SAFE;
    }

    while (rem > 0) {
        int32_t ce = st->eid_l[l];
        uint32_t used = (uint32_t)(P - st->base_l[l]);
        if (used == BP4K_PAGE) {
            st->ext_len[ce] = BP4K_PAGE;
            uint32_t ne = bp4k_alloc_ext(fq, st, p0_cur);
            if (st->eid_l[0] == -1) {
                size_t slim = (st->top_phys >= 0) ? ((size_t)(st->top_phys + 1) * BP4K_PAGE) : 0u;
                if (slim > st->n) slim = st->n;
                st->end_l[0] = (slim >= 32u) ? (st->out + slim - 32u) : st->out;
            }
            st->ext_next[ce] = (int32_t)ne;
            st->eid_l[l]     = (int32_t)ne;
            st->base_l[l]    = st->ext_ptr[ne];
            P = st->base_l[l];
            st->end_l[l]     = P + BP4K_SAFE;
            used = 0;
        }
        uint32_t av = BP4K_PAGE - used;
        uint32_t tk = rem < av ? rem : av;
        memcpy(P, src, tk);
        P += tk; src += tk; rem -= tk;
    }
    *P_io = P;
}

__attribute__((noinline))
static void bp4k_cold(R4Kernel *k, FlatQ4K *fq, Bp4KState *st, int l, uint32_t *J_io, uint8_t **P_io, uint32_t cd, uint8_t *p0_cur) {
    uint8_t *P = *P_io;
    if (cd < st->sbase) {
        if (st->eid_l[l] == -2) { *P_io = k->dummy_buf; return; }
        bp4k_emit_slow(k, fq, st, l, P_io, cd, p0_cur);
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
            uint32_t ne = bp4k_alloc_ext(fq, st, p0_cur);
            if (st->eid_l[0] == -1) {
                size_t slim = (st->top_phys >= 0) ? ((size_t)(st->top_phys + 1) * BP4K_PAGE) : 0u;
                if (slim > st->n) slim = st->n;
                st->end_l[0] = (slim >= 32u) ? (st->out + slim - 32u) : st->out;
            }
            st->head[nq]  = (int32_t)ne;
            st->eid_l[l]  = (int32_t)ne;
            st->base_l[l] = st->ext_ptr[ne];
            P = st->base_l[l];
            st->end_l[l]  = P + BP4K_SAFE;
            bp4k_emit_slow(k, fq, st, l, &P, st->saved_code[nq], p0_cur);
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

static void r4_bp4k_bvsmc_emit_1p(R4Kernel *k, FlatQ4K *fq, size_t n, uint32_t S_req, uint8_t *out) {
    uint32_t * restrict edges = k->edges;
    uint32_t * restrict extra = k->extra;
    const __m128i * restrict inline16 = k->inline16;
    uint16_t * restrict w_len = k->w_len;
    const uint32_t m = k->m, mark = k->mark, bits = k->bits, mask = (1u << bits) - 1u;
    const int wide = k->wide;
    const uint32_t sbase = 256u + k->words;

    Bp4KState st;
    uint32_t S_try = S_req < m ? S_req : m;
    if (S_try > 48u) S_try = 48u;
    st.sbase = sbase; st.next_q = 0; st.active = 0; st.seg0_len = 0;
    st.n = n; st.out = out;
    st.n_phys = (uint32_t)(n / BP4K_PAGE);
    st.top_phys = (int32_t)st.n_phys - 1;
    st.n_ext = 0; st.ovf_used = 0;

    uint32_t cuts[48], saved_raw[48], n_unique = 0;
    for (uint32_t q = 0; q < S_try; q++) {
        uint32_t v = (uint32_t)((mark + ((uint64_t)q * m) / S_try) % m);
        int dup = 0;
        for (uint32_t prev = 0; prev < n_unique; prev++) if (cuts[prev] == v) { dup = 1; break; }
        if (!dup) cuts[n_unique++] = v;
    }
    st.S_eff = n_unique;

    for (uint32_t q = 0; q < st.S_eff; q++) {
        uint32_t v = cuts[q], raw = edges[v]; saved_raw[q] = raw;
        st.saved_next[q] = raw & mask;
        st.saved_code[q] = wide ? extra[v] : (raw >> bits);
        st.succ[q] = -1; st.head[q] = -1;
        w_len[sbase + q] = 65535u;
    }
    for (uint32_t q = 0; q < st.S_eff; q++) {
        uint32_t v = cuts[q], sc = sbase + q;
        if (wide) extra[v] = sc; else edges[v] = (sc << bits) | st.saved_next[q];
    }
    edges[m] = m; if (wide) extra[m] = 0;

    uint32_t j_init[8]; uint8_t *p_init[8];
    p_init[0] = out;
    for (int l = 0; l < 8; l++) {
        if (st.next_q < st.S_eff) {
            uint32_t q = st.next_q++; st.active++; st.q_l[l] = q;
            if (q == 0) {
                st.eid_l[0] = -1; st.base_l[0] = out; p_init[0] = out;
                size_t slim = (st.top_phys >= 0) ? ((size_t)(st.top_phys + 1) * BP4K_PAGE) : 0u;
                if (slim > n) slim = n;
                st.end_l[0] = (slim >= 32u) ? (out + slim - 32u) : out;
            } else {
                uint32_t ne = bp4k_alloc_ext(fq, &st, p_init[0]);
                st.head[q] = (int32_t)ne; st.eid_l[l] = (int32_t)ne;
                st.base_l[l] = st.ext_ptr[ne]; p_init[l] = st.base_l[l]; st.end_l[l] = p_init[l] + BP4K_SAFE;
                if (st.eid_l[0] == -1) {
                    size_t slim = (st.top_phys >= 0) ? ((size_t)(st.top_phys + 1) * BP4K_PAGE) : 0u;
                    if (slim > n) slim = n;
                    st.end_l[0] = (slim >= 32u) ? (out + slim - 32u) : out;
                }
            }
            bp4k_emit_slow(k, fq, &st, l, &p_init[l], st.saved_code[q], p_init[0]);
            j_init[l] = st.saved_next[q];
        } else {
            st.eid_l[l] = -2; p_init[l] = k->dummy_buf; st.end_l[l] = k->dummy_buf + 2048; j_init[l] = m;
        }
    }

    uint32_t j0=j_init[0], j1=j_init[1], j2=j_init[2], j3=j_init[3];
    uint32_t j4=j_init[4], j5=j_init[5], j6=j_init[6], j7=j_init[7];
    uint8_t * restrict p0=p_init[0], * restrict p1=p_init[1], * restrict p2=p_init[2], * restrict p3=p_init[3];
    uint8_t * restrict p4=p_init[4], * restrict p5=p_init[5], * restrict p6=p_init[6], * restrict p7=p_init[7];

    #define BP4K_STEP(L, J, P, CD_EXPR, E_VAR) do { \
        uint32_t cd = (CD_EXPR); J = (E_VAR) & mask; uint32_t ln = w_len[cd]; \
        if (__builtin_expect(ln <= 16u && P < st.end_l[L], 1)) { \
            _mm_storeu_si128((__m128i *)P, inline16[cd]); P += ln; \
        } else { uint8_t *pt = P; bp4k_cold(k, fq, &st, L, &(J), &pt, cd, p0); P = pt; } \
    } while (0)

    if (!wide) {
        while (st.active > 0) {
            uint32_t e0=edges[j0], e1=edges[j1], e2=edges[j2], e3=edges[j3];
            uint32_t e4=edges[j4], e5=edges[j5], e6=edges[j6], e7=edges[j7];
            BP4K_STEP(0,j0,p0,e0>>bits,e0); BP4K_STEP(1,j1,p1,e1>>bits,e1);
            BP4K_STEP(2,j2,p2,e2>>bits,e2); BP4K_STEP(3,j3,p3,e3>>bits,e3);
            BP4K_STEP(4,j4,p4,e4>>bits,e4); BP4K_STEP(5,j5,p5,e5>>bits,e5);
            BP4K_STEP(6,j6,p6,e6>>bits,e6); BP4K_STEP(7,j7,p7,e7>>bits,e7);
        }
    } else {
        while (st.active > 0) {
            uint32_t e0=edges[j0],c0=extra[j0],e1=edges[j1],c1=extra[j1];
            uint32_t e2=edges[j2],c2=extra[j2],e3=edges[j3],c3=extra[j3];
            uint32_t e4=edges[j4],c4=extra[j4],e5=edges[j5],c5=extra[j5];
            uint32_t e6=edges[j6],c6=extra[j6],e7=edges[j7],c7=extra[j7];
            BP4K_STEP(0,j0,p0,c0,e0); BP4K_STEP(1,j1,p1,c1,e1);
            BP4K_STEP(2,j2,p2,c2,e2); BP4K_STEP(3,j3,p3,c3,e3);
            BP4K_STEP(4,j4,p4,c4,e4); BP4K_STEP(5,j5,p5,c5,e5);
            BP4K_STEP(6,j6,p6,c6,e6); BP4K_STEP(7,j7,p7,c7,e7);
        }
    }
    #undef BP4K_STEP

    for (uint32_t q = 0; q < st.S_eff; q++) {
        uint32_t v = cuts[q]; edges[v] = saved_raw[q];
        if (wide) extra[v] = st.saved_code[q];
    }

    uint32_t ord_e[BP4K_MAX_EXTENTS], ord_t0[BP4K_MAX_EXTENTS], ord_len[BP4K_MAX_EXTENTS], n_ord = 0;
    size_t wr = st.seg0_len;
    for (int32_t e = st.head[0]; e >= 0 && wr < n; e = st.ext_next[e]) {
        uint32_t cl = st.ext_len[e]; if (wr + cl > n) cl = (uint32_t)(n - wr);
        if (cl > 0) { ord_e[n_ord] = (uint32_t)e; ord_t0[n_ord] = (uint32_t)wr; ord_len[n_ord] = cl; n_ord++; wr += cl; }
    }
    int32_t cs = st.succ[0]; uint32_t steps_s = 1;
    while (cs > 0 && steps_s < st.S_eff && wr < n) {
        for (int32_t e = st.head[cs]; e >= 0 && wr < n; e = st.ext_next[e]) {
            uint32_t cl = st.ext_len[e]; if (wr + cl > n) cl = (uint32_t)(n - wr);
            if (cl > 0) { ord_e[n_ord] = (uint32_t)e; ord_t0[n_ord] = (uint32_t)wr; ord_len[n_ord] = cl; n_ord++; wr += cl; }
        }
        cs = st.succ[cs]; steps_s++;
    }

    if (wr > st.seg0_len) {
        uint32_t total_tp = (uint32_t)((wr + BP4K_PAGE - 1u) / BP4K_PAGE);
        uint32_t first_tp = (uint32_t)(st.seg0_len / BP4K_PAGE);
        uint32_t n_slots  = total_tp > st.n_phys ? total_tp : st.n_phys;
        assert(n_slots <= BP4K_MAX_SLOTS);

        int16_t  phys_to_ord[BP4K_MAX_SLOTS];
        uint8_t  ref_cnt[BP4K_MAX_SLOTS], tp_done[BP4K_MAX_SLOTS];
        uint16_t tp_first_ord[BP4K_MAX_SLOTS], tp_last_ord[BP4K_MAX_SLOTS];
        memset(ref_cnt, 0, n_slots);
        memset(tp_done, 0, total_tp);
        for (uint32_t s = 0; s < n_slots; s++) phys_to_ord[s] = -1;
        for (uint32_t p = 0; p < total_tp; p++) { tp_first_ord[p] = 65535u; tp_last_ord[p] = 0; }

        uint8_t  ovf_ref[BP4K_OVF_CAP] = {0}, ovf_active[BP4K_OVF_CAP] = {0};
        uint16_t ovf_free[BP4K_OVF_CAP];
        uint32_t ovf_free_len = 0;

        for (uint32_t oi = 0; oi < n_ord; oi++) {
            uint32_t e = ord_e[oi], t0 = ord_t0[oi], t1 = t0 + ord_len[oi] - 1u;
            uint32_t p_lo = t0 / BP4K_PAGE, p_hi = t1 / BP4K_PAGE;
            uint8_t rc = (uint8_t)(p_hi - p_lo + 1u);
            for (uint32_t p = p_lo; p <= p_hi; p++) {
                if (oi < tp_first_ord[p]) tp_first_ord[p] = (uint16_t)oi;
                tp_last_ord[p] = (uint16_t)oi;
            }
            int16_t sp = st.ext_phys[e];
            if (sp >= 0) { phys_to_ord[sp] = (int16_t)oi; ref_cnt[sp] = rc; }
            else { int16_t oid = st.ext_ovf[e]; ovf_ref[oid] = rc; ovf_active[oid] = 1u; }
        }
        for (uint32_t oid = 0; oid < st.ovf_used; oid++) {
            if (!ovf_active[oid]) ovf_free[ovf_free_len++] = (uint16_t)oid;
        }

        uint16_t ready[BP4K_MAX_SLOTS];
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
                    for (uint32_t s = first_tp; s < n_slots; s++) if (ref_cnt[s] > 0) { chosen = (int32_t)s; break; }
                }
                assert(chosen >= 0);
                int16_t oi = phys_to_ord[chosen];
                uint32_t e = ord_e[oi];
                uint32_t oid = (ovf_free_len > 0) ? ovf_free[--ovf_free_len] : st.ovf_used++;
                if (st.ovf_used > peak_ovf) peak_ovf = st.ovf_used;
                assert(oid < BP4K_OVF_CAP);

                uint8_t *dst_ovf = fq->ovf4k + (size_t)oid * BP4K_PAGE;
                memcpy(dst_ovf, st.ext_ptr[e], ord_len[oi]);
                st.ext_ptr[e]  = dst_ovf;
                st.ext_phys[e] = -1;
                st.ext_ovf[e]  = (int16_t)oid;
                ovf_ref[oid]   = ref_cnt[chosen];
                ref_cnt[chosen] = 0;
                evicts++;
                if ((uint32_t)chosen < total_tp && !tp_done[chosen]) ready[r_len++] = (uint16_t)chosen;
                continue;
            }
            uint32_t p = ready[--r_len];
            if (tp_done[p]) continue;
            tp_done[p] = 1u; done_cnt++;

            uint32_t p_start = p * BP4K_PAGE, p_end = p_start + BP4K_PAGE;
            if (p_end > wr) p_end = (uint32_t)wr;
            uint32_t copy_from = p_start < st.seg0_len ? st.seg0_len : p_start;
            if (tp_first_ord[p] != 65535u) {
                for (uint32_t oi = tp_first_ord[p]; oi <= tp_last_ord[p]; oi++) {
                    uint32_t e = ord_e[oi], t0 = ord_t0[oi], t1 = t0 + ord_len[oi];
                    uint32_t lo = t0 > copy_from ? t0 : copy_from, hi = t1 < p_end ? t1 : p_end;
                    if (lo < hi) memcpy(out + lo, st.ext_ptr[e] + (lo - t0), hi - lo);
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
        fq->peak_ovf4k_pages = peak_ovf;
        fq->evict4k_pages    = evicts;
    } else {
        fq->peak_ovf4k_pages = st.ovf_used;
        fq->evict4k_pages    = 0;
    }

    repeat_period(n, out, &wr);
}

/* =========================================================================
 * Self-Tests & Paired 25-Round 7-Arm Confrontation
 * ========================================================================= */
static size_t to_capped_records_with_meta(const uint8_t *L, size_t n, uint16_t *recs, uint32_t counts[256], uint32_t per[256]) {
    memset(counts, 0, 256 * 4u);
    memset(per,    0, 256 * 4u);
    size_t rlen = 0;
    for (size_t i = 0; i < n; i++) {
        uint8_t b = L[i];
        counts[b]++;
        if (rlen > 0 && (uint8_t)(recs[rlen - 1] >> 8) == b && (recs[rlen - 1] & 255u) < 127u) {
            recs[rlen - 1]++;
        } else {
            recs[rlen++] = (uint16_t)(((uint16_t)b << 8) | 1u);
            per[b]++;
        }
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


static inline uint32_t r4_pow_word(R4Kernel *k, uint32_t w, uint32_t q) {
    if (w == 0u || q == 0u) return 0u;
    uint32_t acc = 0u, cur = w;
    while (q > 0u) {
        if (q & 1u) acc = (acc == 0u) ? cur : r4_join(k, acc, cur);
        q >>= 1u;
        if (q > 0u) cur = r4_join(k, cur, cur);
    }
    return acc;
}

static void r4_append_rule_fast(R4Kernel *k, uint32_t word) {
    if ((word & 0x80000000u) == 0u) {
        uint32_t wl = word >> 24;
        const uint8_t *wsrc = k->small_arena + (word & 0x00FFFFFFu);
        r4_spell_ensure(k, wl);
        memcpy(k->long_spell + k->spell_len, wsrc, wl);
        k->spell_len += wl;
        return;
    }
    uint32_t ridx = word & 0x7FFFFFFFu;
    EpRule r = k->rules[ridx];
    if (r.left == r.right) {
        size_t o0 = k->spell_len;
        r4_append_rule_fast(k, r.left);
        size_t len = k->spell_len - o0;
        r4_spell_ensure(k, len);
        memcpy(k->long_spell + k->spell_len, k->long_spell + o0, len);
        k->spell_len += len;
    } else {
        r4_append_rule_fast(k, r.left);
        r4_append_rule_fast(k, r.right);
    }
}
static __attribute__((noinline)) void aht_flatq_w24_r4_construct(
    R4Kernel *k, FlatQ4K *fq,
    const uint16_t *records, size_t n_recs,
    const uint32_t counts_in[256], const uint32_t per_in[256],
    uint32_t n, uint32_t p, uint32_t reserve_sentinels
) {
    uint32_t counts_local[256], per_local[256];
    const uint32_t * restrict counts = counts_in;
    const uint32_t * restrict per    = per_in;

    if (!counts_in || !per_in) {
        memset(counts_local, 0, sizeof(counts_local));
        memset(per_local,    0, sizeof(per_local));
        for (size_t ri = 0; ri < n_recs; ri++) {
            uint16_t rc = records[ri];
            uint8_t b = (uint8_t)(rc >> 8);
            counts_local[b] += (uint32_t)(rc & 255u);
            per_local[b]++;
        }
        counts = counts_local;
        per    = per_local;
    }

    uint32_t *q_start[256], *q_cur[256];
    uint32_t thick_rows[256] = {0}, thick[256] = {0};

    size_t off_q = 0, tight_sum = 0;
    for (int a = 0; a < 256; a++) {
        uint64_t by_per = (uint64_t)per[a] * 24ULL;
        uint32_t ub = (counts[a] < by_per) ? counts[a] : (uint32_t)by_per;
        q_start[a] = fq->flat_q + off_q;
        q_cur[a]   = q_start[a];
        tight_sum += (size_t)ub + 1u;
        off_q     += (size_t)ub + 36u;
    }
    assert(off_q <= fq->flat_cap);
    fq->last_slab_entries = tight_sum;
    k->strips_len = 0;

    uint32_t m = 0, mark = 0, pos = 0;
    size_t i = 0;

    #define FQ_PUSH24(C, W) do { \
        uint32_t *_d = q_cur[(C)]; \
        store_u24_raw(_d, m); \
        q_cur[(C)] = _d + (W); \
        m += (W); \
    } while (0)

    /* Phase 1A: Fast pre-primary scan while pos + 128 <= p */
    while (i < n_recs && pos + 128u <= p) {
        uint16_t rec = records[i++];
        uint8_t c = (uint8_t)(rec >> 8);
        uint32_t width = (uint32_t)(rec & 255u);
        if (__builtin_expect(i < n_recs && (uint8_t)(records[i] >> 8) == c, 0)) {
            do { width += (uint32_t)(records[i++] & 255u); } while (i < n_recs && (uint8_t)(records[i] >> 8) == c);
            if (pos + width > p) {
                uint32_t end = pos + width;
                if (width <= 24u) {
                    mark = m + (p - pos);
                    FQ_PUSH24(c, width);
                } else {
                    uint32_t cur_all = (uint32_t)(q_cur[c] - q_start[c]) + thick_rows[c];
                    if (pos < p) { uint32_t w1 = p - pos; r4_push_strip(k, pos, cur_all, w1, c); thick[c]++; cur_all += w1; thick_rows[c] += w1; }
                    mark = m; FQ_PUSH24(c, 1u); cur_all += 1u;
                    if (p + 1 < end) { uint32_t w2 = end - p - 1; r4_push_strip(k, p + 1, cur_all, w2, c); thick[c]++; thick_rows[c] += w2; }
                }
                pos = end;
                goto post_primary_scan;
            }
        }
        if (__builtin_expect(width <= 24u, 1)) {
            FQ_PUSH24(c, width);
        } else {
            uint32_t cur_all = (uint32_t)(q_cur[c] - q_start[c]) + thick_rows[c];
            r4_push_strip(k, pos, cur_all, width, c);
            thick[c]++; thick_rows[c] += width;
        }
        pos += width;
    }

    /* Phase 1B: Straddle window around p */
    while (i < n_recs && pos <= p) {
        uint16_t rec = records[i++];
        uint8_t c = (uint8_t)(rec >> 8);
        uint32_t width = (uint32_t)(rec & 255u);
        while (i < n_recs && (uint8_t)(records[i] >> 8) == c) width += (uint32_t)(records[i++] & 255u);
        uint32_t end = pos + width;
        if (width <= 24u) {
            if (p >= pos && p < end) mark = m + (p - pos);
            FQ_PUSH24(c, width);
        } else if (p >= pos && p < end) {
            uint32_t cur_all = (uint32_t)(q_cur[c] - q_start[c]) + thick_rows[c];
            if (pos < p) { uint32_t w1 = p - pos; r4_push_strip(k, pos, cur_all, w1, c); thick[c]++; cur_all += w1; thick_rows[c] += w1; }
            mark = m; FQ_PUSH24(c, 1u); cur_all += 1u;
            if (p + 1 < end) { uint32_t w2 = end - p - 1; r4_push_strip(k, p + 1, cur_all, w2, c); thick[c]++; thick_rows[c] += w2; }
        } else {
            uint32_t cur_all = (uint32_t)(q_cur[c] - q_start[c]) + thick_rows[c];
            r4_push_strip(k, pos, cur_all, width, c);
            thick[c]++; thick_rows[c] += width;
        }
        pos = end;
    }

post_primary_scan:
    /* Phase 1C: Fast post-primary scan (pos > p guaranteed: zero p checks) */
    while (i < n_recs) {
        uint16_t rec = records[i++];
        uint8_t c = (uint8_t)(rec >> 8);
        uint32_t width = (uint32_t)(rec & 255u);
        if (__builtin_expect(i < n_recs && (uint8_t)(records[i] >> 8) == c, 0)) {
            do { width += (uint32_t)(records[i++] & 255u); } while (i < n_recs && (uint8_t)(records[i] >> 8) == c);
        }
        if (__builtin_expect(width <= 24u, 1)) {
            FQ_PUSH24(c, width);
        } else {
            uint32_t cur_all = (uint32_t)(q_cur[c] - q_start[c]) + thick_rows[c];
            r4_push_strip(k, pos, cur_all, width, c);
            thick[c]++; thick_rows[c] += width;
        }
        pos += width;
    }
    #undef FQ_PUSH24

    uint32_t thin[256], first[256], write[256], tot = 0, tw = 0;
    for (int a = 0; a < 256; a++) {
        thin[a]      = (uint32_t)(q_cur[a] - q_start[a]);
        k->q_len[a]  = thin[a];
        first[a]     = tot; tot += counts[a];
        write[a]     = tw;  tw  += thick[a];
    }

    size_t n_strips = k->strips_len;
    if (n_strips > k->img_cap) {
        k->img_cap = n_strips * 2 + 16;
        k->image_order = (uint32_t *)realloc(k->image_order, k->img_cap * 4u);
    }
    if (n_strips > k->strip_top_cap) {
        k->strip_top_cap = n_strips * 2 + 16;
        k->strip_top = (uint32_t *)realloc(k->strip_top, k->strip_top_cap * 4u);
    }
    for (size_t si = 0; si < n_strips; si++) {
        uint8_t a = k->strips[si].byte;
        k->strips[si].to += first[a];
        k->image_order[write[a]++] = (uint32_t)si;
    }

    size_t need_slots = R4_PAD_SLOTS + 3 * n_strips + 16;
    if (need_slots > k->tops_cap) {
        while (need_slots > k->tops_cap) k->tops_cap *= 2;
        free(k->tops); k->tops = (R4Top *)aligned_alloc(64, k->tops_cap * sizeof(R4Top));
    }
    if (need_slots > k->bots_cap) {
        while (need_slots > k->bots_cap) k->bots_cap *= 2;
        free(k->bots); k->bots = (R4Bot *)aligned_alloc(64, k->bots_cap * sizeof(R4Bot));
    }
    R4Top * restrict tops = k->tops;
    R4Bot * restrict bots = k->bots;
    uint32_t * restrict strip_top = k->strip_top;
    for (uint32_t pad = 0; pad < R4_PAD_SLOTS; pad++) { tops[pad] = (R4Top){0,0,0,0}; bots[pad] = (R4Bot){0,0}; }
    k->rules_len = 0; k->arena_len = 256u;

    const uint32_t p0_base = R4_PAD_SLOTS;
    const uint32_t a0_base = p0_base + (uint32_t)n_strips + 2u;
    uint32_t t_cur = a0_base, b_cur = R4_PAD_SLOTS, xc = 0;
    for (size_t si = 0; si < n_strips; si++) {
        Strip s = k->strips[si];
        if (xc < s.from) {
            tops[t_cur] = (R4Top){t_cur - 1u, s.from - xc, 0u, b_cur};
            bots[b_cur] = (R4Bot){b_cur - 1u, t_cur};
            t_cur++; b_cur++;
        }
        strip_top[si] = t_cur;
        tops[t_cur] = (R4Top){t_cur - 1u, s.width, (1u << 24) | (uint32_t)s.byte, 0u};
        t_cur++;
        xc = s.from + s.width;
    }
    if (xc < n) {
        tops[t_cur] = (R4Top){t_cur - 1u, n - xc, 0u, b_cur};
        bots[b_cur] = (R4Bot){b_cur - 1u, t_cur};
        t_cur++; b_cur++;
    }
    const uint32_t a0_end = t_cur;
    uint32_t p0_cur = p0_base; xc = 0;
    for (size_t ii = 0; ii < n_strips; ii++) {
        uint32_t idx = k->image_order[ii]; Strip s = k->strips[idx];
        if (xc < s.to) {
            tops[p0_cur] = (R4Top){p0_cur - 1u, s.to - xc, 0u, b_cur};
            bots[b_cur]  = (R4Bot){b_cur - 1u, p0_cur};
            p0_cur++; b_cur++;
        }
        uint32_t st_i = strip_top[idx];
        tops[st_i].other = b_cur;
        bots[b_cur] = (R4Bot){b_cur - 1u, st_i};
        b_cur++;
        xc = s.to + s.width;
    }
    if (xc < n) {
        tops[p0_cur] = (R4Top){p0_cur - 1u, n - xc, 0u, b_cur};
        bots[b_cur]  = (R4Bot){b_cur - 1u, p0_cur};
        p0_cur++; b_cur++;
    }
    if (p0_cur > p0_base) tops[p0_base].prev = 0u;
    if (b_cur > R4_PAD_SLOTS) bots[R4_PAD_SLOTS].prev = 0u;
    if (a0_end > a0_base) tops[a0_base].prev = (p0_cur > p0_base) ? (p0_cur - 1u) : 0u;
    k->active_tops = (size_t)(p0_cur - p0_base) + (size_t)(a0_end - a0_base);
    k->active_bots = (size_t)(b_cur - R4_PAD_SLOTS);

    uint32_t sa = a0_end - 1u, sb = b_cur - 1u, remaining = n;
    while (remaining > 0) {
        R4Top ta = tops[sa];
        uint32_t na = ta.prev, ra = ta.width, aw = ta.word, bs = ta.other;
        R4Bot bb_slot = bots[sb];
        uint32_t nb = bb_slot.prev, ts = bb_slot.top;
        R4Top tb = tops[ts];
        uint32_t rb = tb.width, bw = tb.word;
        uint32_t fa = sa, fb = sb;
        _mm_prefetch((const char *)&bots[tops[sa > R4_FAR ? sa - R4_FAR : 0u].other], _MM_HINT_T0);
        _mm_prefetch((const char *)&tops[bots[sb > R4_FAR ? sb - R4_FAR : 0u].top],   _MM_HINT_T0);
        if (bs == sb) { remaining -= ra; sa = na; sb = nb; continue; }
        for (;;) {
            if (ra == rb) {
                uint32_t wd = r4_join(k, aw, bw);
                remaining -= ra;
                tops[ts].width = ra; tops[ts].word = wd; tops[ts].other = bs; bots[bs].top = ts;
                sa = na; sb = nb; break;
            } else if (ra > rb) {
                if (__builtin_expect(nb == bs && ra >= 2u * rb, 0)) {
                    uint32_t q = ra / rb, r_rem = ra % rb, p_b = bots[bs].prev;
                    uint32_t w_pow = r4_pow_word(k, aw, q);
                    uint32_t w_q   = r4_join(k, w_pow, bw);
                    remaining -= q * rb;
                    if (r_rem == 0u) {
                        tops[ts].width = rb; tops[ts].word = w_q; tops[ts].other = bs;
                        bots[bs] = (R4Bot){p_b, ts};
                        sa = na; sb = bs; break;
                    } else {
                        tops[ts].width = rb; tops[ts].word = w_q; tops[ts].other = bs;
                        bots[fb].prev = p_b; bots[bs] = (R4Bot){fb, ts};
                        uint32_t bs_old = bs; bs = fb; fb = bs_old; nb = bs;
                        ra = r_rem; bw = w_q; continue;
                    }
                }
                uint32_t wd = r4_join(k, aw, bw);
                remaining -= rb;
                tops[ts].width = rb; tops[ts].word = wd; tops[ts].other = bs;
                bots[fb].prev = bots[bs].prev; bots[bs] = (R4Bot){fb, ts}; bs = fb; ra -= rb;
                uint32_t s = nb; fb = s;
                _mm_prefetch((const char *)&tops[bots[s > R4_FAR ? s - R4_FAR : 0u].top], _MM_HINT_T0);
                R4Bot bnext = bots[s]; nb = bnext.prev; ts = bnext.top;
                R4Top tnext = tops[ts]; rb = tnext.width; bw = tnext.word;
            } else {
                if (__builtin_expect(na == ts && rb >= 2u * ra, 0)) {
                    uint32_t q = rb / ra, r_rem = rb % ra, p_t = tops[ts].prev;
                    uint32_t w_pow = r4_pow_word(k, bw, q);
                    uint32_t w_q   = r4_join(k, aw, w_pow);
                    remaining -= q * ra;
                    if (r_rem == 0u) {
                        tops[ts] = (R4Top){p_t, ra, w_q, bs}; bots[bs].top = ts;
                        sa = ts; sb = nb; break;
                    } else {
                        bots[bs].top = ts; tops[fa].prev = p_t;
                        tops[ts] = (R4Top){fa, ra, w_q, bs};
                        uint32_t ts_old = ts; ts = fa; fa = ts_old; na = ts;
                        aw = w_q; rb = r_rem; continue;
                    }
                }
                uint32_t wd = r4_join(k, aw, bw);
                remaining -= ra;
                bots[bs].top = ts; tops[fa].prev = tops[ts].prev;
                tops[ts] = (R4Top){fa, ra, wd, bs}; ts = fa; rb -= ra;
                uint32_t s = na; fa = s;
                _mm_prefetch((const char *)&bots[tops[s > R4_FAR ? s - R4_FAR : 0u].other], _MM_HINT_T0);
                R4Top tnext = tops[s]; na = tnext.prev; ra = tnext.width; aw = tnext.word; bs = tnext.other;
            }
        }
    }

    uint32_t x_bot = m;
    while (sb != 0u) {
        R4Bot bslot = bots[sb]; uint32_t ts = bslot.top;
        x_bot -= tops[ts].width; tops[ts].other = x_bot; sb = bslot.prev;
    }

    k->bands_len = 0; k->spell_len = 0;
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
            rem_node -= width; kc -= width;
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
                    k->inline16[code] = _mm_loadu_si128((const __m128i *)wsrc);
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
                        r4_append_rule_fast(k, nd.word);
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
            if (rem_node == 0u) { sa = nd.prev; rem_node = (sa != 0u) ? tops[sa].width : 0u; }
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
        const uint32_t * restrict qa = q_start[a];
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


#define SEG_LANES 32
#define SEG_COUNT 512
#define SEG_TILE  256
#define SEG_START 0x80000000u
#define SEG_LINK  0x7FFFFFFFu

typedef struct {
    uint32_t *tt;
    size_t    tt_cap;
    uint32_t  starts[SEG_COUNT + 4];
    size_t    starts_len;
    uint8_t  *bufs[SEG_COUNT + 4];
    size_t    buf_len[SEG_COUNT + 4];
    size_t    buf_cap[SEG_COUNT + 4];
    size_t    next[SEG_COUNT + 4];
    size_t    first;
    size_t    n;
} SegKernel;

static void seg_init(SegKernel *s) {
    memset(s, 0, sizeof(*s));
    s->tt_cap = 900128;
    s->tt = (uint32_t *)aligned_alloc(64, s->tt_cap * sizeof(uint32_t));
    for (int i = 0; i < SEG_COUNT + 4; i++) {
        s->buf_cap[i] = 4096;
        s->bufs[i] = (uint8_t *)malloc(s->buf_cap[i]);
    }
}

static void seg_free(SegKernel *s) {
    free(s->tt);
    for (int i = 0; i < SEG_COUNT + 4; i++) free(s->bufs[i]);
}

static int cmp_u32(const void *a, const void *b) {
    uint32_t x = *(const uint32_t *)a, y = *(const uint32_t *)b;
    return (x > y) - (x < y);
}

static size_t seg_of_start(const uint32_t *starts, size_t ns, uint32_t row) {
    size_t lo = 0, hi = ns;
    while (lo < hi) {
        size_t mid = (lo + hi) >> 1;
        if (starts[mid] < row) lo = mid + 1;
        else hi = mid;
    }
    return (lo < ns && starts[lo] == row) ? lo : (size_t)-1;
}

typedef struct {
    uint32_t row;
    size_t   seg;
    uint8_t *p;
    uint8_t *end;
} SegLane;

static inline SegLane seg_take(SegKernel *s, size_t seg) {
    uint8_t *base = s->bufs[seg];
    uint32_t q = s->tt[s->starts[seg]] & SEG_LINK;
    base[0] = (uint8_t)q;
    SegLane l = { q >> 8, seg, base + 1, base + s->buf_cap[seg] };
    return l;
}

static inline void seg_finish(SegKernel *s, const SegLane *l) {
    s->buf_len[l->seg] = (size_t)(l->p - s->bufs[l->seg]);
    s->next[l->seg] = seg_of_start(s->starts, s->starts_len, l->row);
}

static void seg_grow(SegKernel *s, SegLane *l) {
    size_t len = (size_t)(l->p - s->bufs[l->seg]);
    size_t ncap = s->buf_cap[l->seg] * 2;
    s->bufs[l->seg] = (uint8_t *)realloc(s->bufs[l->seg], ncap);
    s->buf_cap[l->seg] = ncap;
    l->p = s->bufs[l->seg] + len;
    l->end = s->bufs[l->seg] + ncap;
}

static __attribute__((noinline)) void seg_room_segments(SegKernel *s, size_t n, uint32_t orig_ptr, int back) {
    uint32_t r0 = back ? orig_ptr : (s->tt[orig_ptr] >> 8);
    uint64_t x = 0x2545f4914f6cdd1dULL ^ (uint64_t)n;
    for (int i = 0; i < SEG_COUNT - 1; i++) {
        x ^= x << 13; x ^= x >> 7; x ^= x << 17;
        s->starts[i] = (uint32_t)(x % (uint64_t)n);
    }
    s->starts[SEG_COUNT - 1] = r0;
    qsort(s->starts, SEG_COUNT, sizeof(uint32_t), cmp_u32);
    size_t ns = 0;
    for (int i = 0; i < SEG_COUNT; i++) {
        if (ns == 0 || s->starts[i] != s->starts[ns - 1]) s->starts[ns++] = s->starts[i];
    }
    s->starts_len = ns;
    uint32_t * restrict tt = s->tt;
    for (size_t i = 0; i < ns; i++) tt[s->starts[i]] |= SEG_START;

    size_t cap = 2 * n / ns + 64;
    for (size_t i = 0; i < ns; i++) {
        if (s->buf_cap[i] < cap) {
            s->buf_cap[i] = cap;
            s->bufs[i] = (uint8_t *)realloc(s->bufs[i], cap);
        }
        s->buf_len[i] = 0;
        s->next[i] = (size_t)-1;
    }

    SegLane lanes[SEG_LANES];
    size_t queued = 0, active = 0;
    while (active < SEG_LANES && queued < ns) {
        lanes[active] = seg_take(s, queued);
        _mm_prefetch((const char *)(tt + lanes[active].row), _MM_HINT_T0);
        queued++; active++;
    }

    if (active == SEG_LANES) {
        uint32_t rows[SEG_LANES];
        for (int k = 0; k < SEG_LANES; k++) rows[k] = lanes[k].row;
        uint8_t tile[SEG_LANES][SEG_TILE];
        for (;;) {
            size_t room = cap - 1;
            for (int k = 0; k < SEG_LANES; k++) {
                if (lanes[k].p == lanes[k].end) seg_grow(s, &lanes[k]);
                size_t sp = (size_t)(lanes[k].end - lanes[k].p);
                if (sp < room) room = sp;
            }
            size_t done = 0;
            while (done < room) {
                size_t w = (room - done) < SEG_TILE ? (room - done) : SEG_TILE;
                size_t st[SEG_LANES] = {0};
                for (size_t j = 0; j < w; j++) {
                    uint64_t marked = 0;
                    #define SSTEP(K) do { \
                        uint32_t q = tt[rows[K]]; \
                        if (__builtin_expect((q & SEG_START) != 0, 0)) { marked |= (1ULL << (K)); } \
                        else { tile[K][j] = (uint8_t)q; rows[K] = q >> 8; _mm_prefetch((const char *)(tt + (q >> 8)), _MM_HINT_T0); } \
                    } while (0)
                    SSTEP(0);  SSTEP(1);  SSTEP(2);  SSTEP(3);  SSTEP(4);  SSTEP(5);  SSTEP(6);  SSTEP(7);
                    SSTEP(8);  SSTEP(9);  SSTEP(10); SSTEP(11); SSTEP(12); SSTEP(13); SSTEP(14); SSTEP(15);
                    SSTEP(16); SSTEP(17); SSTEP(18); SSTEP(19); SSTEP(20); SSTEP(21); SSTEP(22); SSTEP(23);
                    SSTEP(24); SSTEP(25); SSTEP(26); SSTEP(27); SSTEP(28); SSTEP(29); SSTEP(30); SSTEP(31);
                    #undef SSTEP
                    while (marked != 0) {
                        int k = __builtin_ctzll(marked);
                        marked &= marked - 1ULL;
                        SegLane *l = &lanes[k];
                        l->row = rows[k];
                        size_t fl = j - st[k];
                        if (fl > 0) { memcpy(l->p, &tile[k][st[k]], fl); l->p += fl; }
                        seg_finish(s, l);
                        if (queued == ns) {
                            for (int i = 0; i < SEG_LANES; i++) {
                                lanes[i].row = rows[i];
                                if (i != k) {
                                    int waiting = (int)((marked >> i) & 1ULL);
                                    size_t to = j + (size_t)(!waiting);
                                    size_t fli = to - st[i];
                                    if (fli > 0) { memcpy(lanes[i].p, &tile[i][st[i]], fli); lanes[i].p += fli; }
                                }
                            }
                            goto full_done;
                        }
                        *l = seg_take(s, queued);
                        rows[k] = l->row;
                        _mm_prefetch((const char *)(tt + l->row), _MM_HINT_T0);
                        queued++;
                        st[k] = j + 1;
                    }
                }
                for (int k = 0; k < SEG_LANES; k++) {
                    size_t fl = w - st[k];
                    if (fl > 0) { memcpy(lanes[k].p, &tile[k][st[k]], fl); lanes[k].p += fl; }
                }
                done += w;
            }
        }
    }
full_done:;
    SegLane live[SEG_LANES];
    size_t nlive = 0;
    for (size_t i = 0; i < active; i++) {
        if (lanes[i].seg != (size_t)-1 && s->next[lanes[i].seg] == (size_t)-1) {
            if (lanes[i].p == lanes[i].end) seg_grow(s, &lanes[i]);
            live[nlive++] = lanes[i];
        }
    }
    while (nlive > 0) {
        size_t i = 0;
        while (i < nlive) {
            SegLane *l = &live[i];
            uint32_t q = tt[l->row];
            if ((q & SEG_START) != 0) {
                seg_finish(s, l);
                live[i] = live[--nlive];
                continue;
            }
            l->row = q >> 8;
            _mm_prefetch((const char *)(tt + l->row), _MM_HINT_T0);
            *(l->p++) = (uint8_t)q;
            if (l->p == l->end) seg_grow(s, l);
            i++;
        }
    }
    for (size_t i = 0; i < ns; i++) tt[s->starts[i]] &= SEG_LINK;
    s->first = seg_of_start(s->starts, ns, r0);
    s->n = n;
}

static __attribute__((noinline, target("avx2"))) void seg_fill_avx2(
    const uint16_t *runs, size_t n_recs, size_t n, const uint32_t counts[256], uint32_t *tt
) {
    (void)n;
    uint32_t next[256];
    uint32_t cum = 0;
    for (int c = 0; c < 256; c++) { next[c] = cum; cum += counts[c]; }
    __m256i lanes = _mm256_setr_epi32(0, 256, 512, 768, 1024, 1280, 1536, 1792);
    __m256i step  = _mm256_set1_epi32(2048);
    uint32_t q = 0;
    for (size_t i = 0; i < n_recs; i++) {
        uint16_t x = runs[i];
        uint32_t c = x >> 8, len = x & 0xffu;
        uint32_t e = c | (next[c] << 8);
        __m256i v = _mm256_add_epi32(_mm256_set1_epi32((int32_t)e), lanes);
        uint32_t w = q, end = q + len;
        for (;;) {
            _mm256_storeu_si256((__m256i *)(tt + w), v);
            w += 8;
            if (w >= end) break;
            v = _mm256_add_epi32(v, step);
        }
        next[c] += len;
        q = end;
    }
}

static __attribute__((noinline)) void seg_assemble_reversed(const SegKernel *s, uint8_t *out) {
    size_t n = s->n, at = n, seg = s->first;
    while (at > 0) {
        const uint8_t *b = s->bufs[seg];
        size_t len = s->buf_len[seg];
        if (len > at) len = at;
        uint8_t *dst = out + at - len;
        for (size_t k = 0; k < len; k++) dst[len - 1 - k] = b[k];
        at -= len;
        seg = s->next[seg];
        if (seg == s->first) break;
    }
    size_t m = n - at;
    if (m > 0 && m < n) {
        memmove(out, out + n - m, m);
        size_t have = m;
        while (have < n) {
            size_t take = have < (n - have) ? have : (n - have);
            memcpy(out + have, out, take);
            have += take;
        }
    }
}

static __attribute__((noinline)) void seg_assemble(const SegKernel *s, uint8_t *out) {
    size_t n = s->n, wr = 0, seg = s->first;
    while (wr < n) {
        const uint8_t *b = s->bufs[seg];
        size_t len = s->buf_len[seg];
        if (len > n - wr) len = n - wr;
        memcpy(out + wr, b, len);
        wr += len;
        seg = s->next[seg];
    }
}

/* Faithful C port of src/bwt.rs::build_one_pass */
static __attribute__((noinline)) void seg_build_one_pass(
    const uint8_t *ll, size_t n, const uint32_t counts[256], uint32_t *tt
) {
    uint32_t ftab[256], lim[256];
    uint32_t cum = 0;
    for (int b = 0; b < 256; b++) { ftab[b] = cum; cum += counts[b]; lim[b] = cum; }
    __m128i z = _mm_setzero_si128();
    size_t i = 0;
    while (i + 16 <= n) {
        __m128i v = _mm_loadu_si128((const __m128i *)(ll + i));
        __m128i prev = _mm_slli_si128(v, 1);
        uint32_t joins = (uint32_t)_mm_movemask_epi8(_mm_cmpeq_epi8(v, prev)) & 0xfffeu;
        if (joins == 0xfffeu) {
            uint8_t c = *(volatile const uint8_t *)(ll + i);
            uint32_t s = ftab[c];
            __m128i sv = _mm_loadu_si128((const __m128i *)(ll + s));
            __m128i lo = _mm_unpacklo_epi8(sv, z), hi = _mm_unpackhi_epi8(sv, z);
            __m128i b = _mm_set1_epi32((int32_t)((uint32_t)i << 8));
            _mm_storeu_si128((__m128i *)(tt + s),      _mm_add_epi32(_mm_unpacklo_epi16(lo, z), _mm_add_epi32(b, _mm_setr_epi32(0, 1<<8, 2<<8, 3<<8))));
            _mm_storeu_si128((__m128i *)(tt + s + 4),  _mm_add_epi32(_mm_unpackhi_epi16(lo, z), _mm_add_epi32(b, _mm_setr_epi32(4<<8, 5<<8, 6<<8, 7<<8))));
            _mm_storeu_si128((__m128i *)(tt + s + 8),  _mm_add_epi32(_mm_unpacklo_epi16(hi, z), _mm_add_epi32(b, _mm_setr_epi32(8<<8, 9<<8, 10<<8, 11<<8))));
            _mm_storeu_si128((__m128i *)(tt + s + 12), _mm_add_epi32(_mm_unpackhi_epi16(hi, z), _mm_add_epi32(b, _mm_setr_epi32(12<<8, 13<<8, 14<<8, 15<<8))));
            ftab[c] = s + 16;
        } else if (joins != 0 && __builtin_popcount(joins) >= 10) {
            uint32_t heads = joins ^ 0xffffu;
            while (heads != 0) {
                uint32_t j = (uint32_t)__builtin_ctz(heads);
                heads &= heads - 1u;
                uint32_t len = (uint32_t)__builtin_ctz(heads | (1u << 16)) - j;
                uint8_t c = *(volatile const uint8_t *)(ll + i + j);
                uint32_t s = ftab[c];
                if (s + 16 <= lim[c]) {
                    __m128i sv = _mm_loadu_si128((const __m128i *)(ll + s));
                    __m128i lo = _mm_unpacklo_epi8(sv, z), hi = _mm_unpackhi_epi8(sv, z);
                    __m128i b = _mm_set1_epi32((int32_t)(((uint32_t)(i + j)) << 8));
                    _mm_storeu_si128((__m128i *)(tt + s),      _mm_add_epi32(_mm_unpacklo_epi16(lo, z), _mm_add_epi32(b, _mm_setr_epi32(0, 1<<8, 2<<8, 3<<8))));
                    _mm_storeu_si128((__m128i *)(tt + s + 4),  _mm_add_epi32(_mm_unpackhi_epi16(lo, z), _mm_add_epi32(b, _mm_setr_epi32(4<<8, 5<<8, 6<<8, 7<<8))));
                    _mm_storeu_si128((__m128i *)(tt + s + 8),  _mm_add_epi32(_mm_unpacklo_epi16(hi, z), _mm_add_epi32(b, _mm_setr_epi32(8<<8, 9<<8, 10<<8, 11<<8))));
                    _mm_storeu_si128((__m128i *)(tt + s + 12), _mm_add_epi32(_mm_unpackhi_epi16(hi, z), _mm_add_epi32(b, _mm_setr_epi32(12<<8, 13<<8, 14<<8, 15<<8))));
                    ftab[c] = s + len;
                } else {
                    for (uint32_t u = i + j; u < i + j + len; u++) {
                        uint8_t cu = *(volatile const uint8_t *)(ll + u);
                        uint32_t su = ftab[cu]++;
                        tt[su] = (uint32_t)ll[su] | (u << 8);
                    }
                }
            }
        } else {
            for (size_t u = i; u < i + 16; u++) {
                uint8_t cu = *(volatile const uint8_t *)(ll + u);
                uint32_t su = ftab[cu]++;
                tt[su] = (uint32_t)ll[su] | ((uint32_t)u << 8);
            }
        }
        i += 16;
    }
    for (; i < n; i++) {
        uint8_t cu = *(volatile const uint8_t *)(ll + i);
        uint32_t su = ftab[cu]++;
        tt[su] = (uint32_t)ll[su] | ((uint32_t)i << 8);
    }
}



static __attribute__((noinline, target("avx2"))) void seqlf_col_build(
    const uint8_t * restrict ll, size_t n, const uint32_t counts[256], uint32_t * restrict tt
) {
    uint32_t ftab[256];
    uint32_t cum = 0, max_cnt = 0;
    for (int b = 0; b < 256; b++) {
        ftab[b] = cum;
        uint32_t cb = counts[b];
        cum += cb;
        if (cb > max_cnt) max_cnt = cb;
    }
    size_t i = 0;
    if (max_cnt < (uint32_t)(n >> 3)) {
        for (; i + 8 <= n; i += 8) {
            uint8_t c0 = ll[i+0]; tt[i+0] = (uint32_t)c0 | (ftab[c0]++ << 8);
            uint8_t c1 = ll[i+1]; tt[i+1] = (uint32_t)c1 | (ftab[c1]++ << 8);
            uint8_t c2 = ll[i+2]; tt[i+2] = (uint32_t)c2 | (ftab[c2]++ << 8);
            uint8_t c3 = ll[i+3]; tt[i+3] = (uint32_t)c3 | (ftab[c3]++ << 8);
            uint8_t c4 = ll[i+4]; tt[i+4] = (uint32_t)c4 | (ftab[c4]++ << 8);
            uint8_t c5 = ll[i+5]; tt[i+5] = (uint32_t)c5 | (ftab[c5]++ << 8);
            uint8_t c6 = ll[i+6]; tt[i+6] = (uint32_t)c6 | (ftab[c6]++ << 8);
            uint8_t c7 = ll[i+7]; tt[i+7] = (uint32_t)c7 | (ftab[c7]++ << 8);
        }
    } else {
        const __m256i lanes0 = _mm256_setr_epi32(0, 256, 512, 768, 1024, 1280, 1536, 1792);
        const __m256i lanes1 = _mm256_setr_epi32(2048, 2304, 2560, 2816, 3072, 3328, 3584, 3840);
        for (; i + 16 <= n; i += 16) {
            uint64_t w0, w1;
            memcpy(&w0, ll + i, 8);
            memcpy(&w1, ll + i + 8, 8);
            uint64_t rot = (w0 << 8) | (w0 >> 56);
            if (__builtin_expect(w0 == w1 && w0 == rot, 0)) {
                uint8_t c = (uint8_t)w0;
                uint32_t e = (uint32_t)c | (ftab[c] << 8);
                __m256i ve = _mm256_set1_epi32((int32_t)e);
                _mm256_storeu_si256((__m256i *)(tt + i),     _mm256_add_epi32(ve, lanes0));
                _mm256_storeu_si256((__m256i *)(tt + i + 8), _mm256_add_epi32(ve, lanes1));
                ftab[c] += 16u;
            } else {
                #define SW_STEP(W, SH, OFF) do { \
                    uint8_t c_ = (uint8_t)((W) >> (SH)); \
                    tt[i + (OFF)] = (uint32_t)c_ | (ftab[c_]++ << 8); \
                } while (0)
                SW_STEP(w0,  0, 0); SW_STEP(w0,  8, 1); SW_STEP(w0, 16, 2); SW_STEP(w0, 24, 3);
                SW_STEP(w0, 32, 4); SW_STEP(w0, 40, 5); SW_STEP(w0, 48, 6); SW_STEP(w0, 56, 7);
                SW_STEP(w1,  0, 8); SW_STEP(w1,  8, 9); SW_STEP(w1, 16,10); SW_STEP(w1, 24,11);
                SW_STEP(w1, 32,12); SW_STEP(w1, 40,13); SW_STEP(w1, 48,14); SW_STEP(w1, 56,15);
                #undef SW_STEP
            }
        }
    }
    for (; i < n; i++) {
        uint8_t c = ll[i];
        tt[i] = (uint32_t)c | (ftab[c]++ << 8);
    }
}

static __attribute__((noinline, target("avx2"))) void seg_assemble_reversed_avx2(const SegKernel *s, uint8_t *out) {
    const __m256i rev16 = _mm256_setr_epi8(
        15,14,13,12,11,10,9,8,7,6,5,4,3,2,1,0,
        15,14,13,12,11,10,9,8,7,6,5,4,3,2,1,0
    );
    size_t n = s->n, at = n, seg = s->first;
    while (at > 0) {
        const uint8_t *b = s->bufs[seg];
        size_t len = s->buf_len[seg];
        if (len > at) len = at;
        uint8_t *dst = out + at - len;
        size_t k = 0;
        while (k + 32 <= len) {
            __m256i v = _mm256_loadu_si256((const __m256i *)(b + k));
            v = _mm256_shuffle_epi8(v, rev16);
            v = _mm256_permute2x128_si256(v, v, 0x01);
            _mm256_storeu_si256((__m256i *)(dst + len - 32 - k), v);
            k += 32;
        }
        for (; k < len; k++) dst[len - 1 - k] = b[k];
        at -= len;
        seg = s->next[seg];
        if (seg == s->first) break;
    }
    size_t m = n - at;
    if (m > 0 && m < n) {
        memmove(out, out + n - m, m);
        for (size_t have = m; have < n; ) {
            size_t take = have < (n - have) ? have : (n - have);
            memcpy(out + have, out, take);
            have += take;
        }
    }
}

static void seg_walk_back_plain(const uint32_t *tt, size_t n, uint32_t orig_ptr, uint8_t *out) {
    size_t i = orig_ptr, m = n;
    for (size_t k = 0; k < n; k++) {
        uint32_t e = tt[i];
        out[n - 1 - k] = (uint8_t)e;
        i = e >> 8;
        if (i == orig_ptr) { m = k + 1; break; }
    }
    if (m > 0 && m < n) {
        memmove(out, out + n - m, m);
        for (size_t have = m; have < n; ) {
            size_t take = have < (n - have) ? have : (n - have);
            memcpy(out + have, out, take);
            have += take;
        }
    }
}

static void seg_walk_fwd_plain(const uint32_t *tt, size_t n, uint32_t orig_ptr, uint8_t *out) {
    uint32_t p = tt[orig_ptr];
    for (size_t k = 0; k < n; k++) {
        p = tt[p >> 8];
        out[k] = (uint8_t)p;
    }
}

static size_t rle1_encode_buf(const uint8_t *in, size_t in_len, uint8_t *out, size_t limit) {
    size_t p = 0, w = 0;
    while (p < in_len && w < limit) {
        uint8_t c = in[p];
        size_t run = 1;
        while (p + run < in_len && in[p + run] == c && run < 255u) run++;
        if (run >= 4u) {
            if (w + 5u > limit + 19u) break;
            out[w++] = c; out[w++] = c; out[w++] = c; out[w++] = c;
            out[w++] = (uint8_t)(run - 4u);
            p += run;
        } else {
            if (w + run > limit) run = limit - w;
            for (size_t k = 0; k < run; k++) out[w++] = c;
            p += run;
        }
    }
    return w;
}

static void cyclic_bwt_encode(const uint8_t *t, uint32_t n, uint8_t *L_out, uint32_t *orig_ptr) {
    assert(n > 0u);
    uint32_t *sa  = (uint32_t *)malloc(n * sizeof(uint32_t));
    uint32_t *rk  = (uint32_t *)malloc(n * sizeof(uint32_t));
    uint32_t *nrk = (uint32_t *)malloc(n * sizeof(uint32_t));
    uint32_t *sa2 = (uint32_t *)malloc(n * sizeof(uint32_t));
    uint32_t *cnt = (uint32_t *)calloc((n > 256u ? n : 256u) + 2u, sizeof(uint32_t));
    for (uint32_t i = 0; i < n; i++) cnt[t[i]]++;
    for (uint32_t c = 1; c < 256u; c++) cnt[c] += cnt[c - 1u];
    for (uint32_t i = n; i-- > 0; ) sa[--cnt[t[i]]] = i;
    rk[sa[0]] = 0;
    uint32_t classes = 1;
    for (uint32_t i = 1; i < n; i++) {
        if (t[sa[i]] != t[sa[i - 1u]]) classes++;
        rk[sa[i]] = classes - 1u;
    }
    for (uint32_t h = 1; h < n && classes < n; h <<= 1) {
        for (uint32_t i = 0; i < n; i++) {
            uint32_t v = sa[i];
            sa2[i] = (v >= h) ? (v - h) : (v + n - h);
        }
        memset(cnt, 0, classes * sizeof(uint32_t));
        for (uint32_t i = 0; i < n; i++) cnt[rk[sa2[i]]]++;
        for (uint32_t c = 1; c < classes; c++) cnt[c] += cnt[c - 1u];
        for (uint32_t i = n; i-- > 0; ) sa[--cnt[rk[sa2[i]]]] = sa2[i];
        nrk[sa[0]] = 0;
        uint32_t ncl = 1;
        for (uint32_t i = 1; i < n; i++) {
            uint32_t a = sa[i], b = sa[i - 1u];
            uint32_t ah = (a + h < n) ? (a + h) : (a + h - n);
            uint32_t bh = (b + h < n) ? (b + h) : (b + h - n);
            if (rk[a] != rk[b] || rk[ah] != rk[bh]) ncl++;
            nrk[a] = ncl - 1u;
        }
        uint32_t *tmp = rk; rk = nrk; nrk = tmp;
        classes = ncl;
    }
    *orig_ptr = 0;
    for (uint32_t i = 0; i < n; i++) {
        uint32_t s = sa[i];
        if (s == 0u) *orig_ptr = i;
        L_out[i] = t[(s == 0u) ? (n - 1u) : (s - 1u)];
    }
    free(sa); free(rk); free(nrk); free(sa2); free(cnt);
}

static uint64_t xs64(uint64_t *s) {
    uint64_t x = *s;
    x ^= x << 13; x ^= x >> 7; x ^= x << 17;
    return (*s = x);
}

static void build_non_wiki_block(int kind, uint8_t *L_out, size_t *n_out, uint32_t *p_out) {
    uint8_t *blk = (uint8_t *)malloc(900100);
    size_t n = 0;
    if (kind == 0) {
        /* onebyte (r/n = 0.0079, sparse): 'a' repeated -> RLE1 "aaaa\xfb" repeated */
        for (size_t i = 0; i + 5 <= 899985; i += 5) {
            blk[i]=blk[i+1]=blk[i+2]=blk[i+3]='a'; blk[i+4]=251u;
        }
        n = 899985;
    } else if (kind == 1) {
        /* pattern (r/n = 0.0079, sparse): "0123456" repeated to n = 899,981 (non-periodic spiral!) */
        for (size_t i = 0; i < 899981; i++) blk[i] = (uint8_t)('0' + (i % 7u));
        n = 899981;
    } else if (kind == 2) {
        /* rle_spiral (r/n = 0.0229, ADMITTED by 1/64 <= r/n < 0.15!): 2.5 MB random xyz runs + 42 MB long run */
        uint64_t rng = 0xfeedbeefdeadcafeULL;
        size_t raw_cap = 45000000, rlen = 0;
        uint8_t *rbuf = (uint8_t *)malloc(raw_cap);
        while (rlen < 2500000) {
            size_t rn = 1u + (size_t)(xs64(&rng) % 600ULL);
            uint8_t c = (uint8_t)"xyz"[xs64(&rng) % 3ULL];
            if (rlen + rn > 2500000) rn = 2500000 - rlen;
            memset(rbuf + rlen, c, rn);
            rlen += rn;
        }
        memset(rbuf + rlen, 'x', raw_cap - rlen);
        n = rle1_encode_buf(rbuf, raw_cap, blk, 899981);
        free(rbuf);
    } else if (kind == 3) {
        /* runs (r/n = 0.1825, 2026-11-19-open-admission.md `runs` density row): tests/common/mod.rs:437-441 */
        uint64_t rng = 0xfeedbeefdeadcafeULL;
        size_t raw_cap = 45000000, rlen = 0;
        uint8_t *rbuf = (uint8_t *)malloc(raw_cap);
        while (rlen < 44000000) {
            size_t rn = 1u + (size_t)(xs64(&rng) % 600ULL);
            uint8_t c = (uint8_t)"xyz"[xs64(&rng) % 3ULL];
            if (rlen + rn > raw_cap) rn = raw_cap - rlen;
            memset(rbuf + rlen, c, rn);
            rlen += rn;
        }
        n = rle1_encode_buf(rbuf, rlen, blk, 899981);
        free(rbuf);
    } else if (kind == 4) {
        /* text0 (r/n = 0.2982, 2026-11-19-open-admission.md `text` density row) */
        const char *srcs[] = {
            "src/bwt.rs", "src/cells.rs", "src/cells_lazy.rs", "src/coop.rs",
            "src/lib.rs", "src/mtf.rs", "src/open.rs", "src/parallel.rs",
            "src/runs.rs", "src/stream.rs", "examples/ab.rs", "examples/par_scan.rs",
            "examples/occupancy.rs", "examples/rauzy.rs", "README.md"
        };
        uint8_t *tbuf = (uint8_t *)malloc(1200000);
        size_t tlen = 0;
        for (size_t si = 0; si < sizeof(srcs)/sizeof(srcs[0]) && tlen < 1050000; si++) {
            FILE *fp = fopen(srcs[si], "rb");
            if (fp) {
                tlen += fread(tbuf + tlen, 1, 1100000 - tlen, fp);
                fclose(fp);
            }
        }
        if (tlen < 920000) {
            const char *toks[] = {
                "pub fn invert(&mut self, ll: &[u8], counts: &[u32; 256], orig_ptr: u32, out: &mut Vec<u8>) {\n",
                "    // The walk is a chain of dependent loads through the block table.\n",
                "    let mut row = orig_ptr as usize;\n",
                "    while remaining > 0 { remaining -= width; }\n",
                "    debug_assert!(pos <= len && idx < capacity);\n"
            };
            uint64_t trng = 0x123456789abcdef0ULL;
            while (tlen < 1000000) {
                const char *tk = toks[xs64(&trng) % 5ULL];
                size_t kl = strlen(tk);
                memcpy(tbuf + tlen, tk, kl); tlen += kl;
                char num[32]; int nl = snprintf(num, sizeof(num), " // %llu\n", (unsigned long long)(xs64(&trng) % 999999ULL));
                memcpy(tbuf + tlen, num, (size_t)nl); tlen += (size_t)nl;
            }
        }
        n = rle1_encode_buf(tbuf, tlen, blk, 899981);
        free(tbuf);
    } else if (kind == 5) {
        /* random (r/n = 0.9922, column route) */
        uint64_t rng = 0x9e3779b97f4a7c15ULL;
        uint8_t *rbuf = (uint8_t *)malloc(950000);
        for (size_t i = 0; i < 950000; i++) rbuf[i] = (uint8_t)xs64(&rng);
        n = rle1_encode_buf(rbuf, 950000, blk, 899981);
        free(rbuf);
    } else {
        /* records (r/n = 0.9969, column route) */
        uint8_t *rbuf = (uint8_t *)malloc(1200000);
        for (uint32_t i = 0; i < 300000u; i++) {
            uint32_t v = ((i * 2654435761u) >> 12) | (i << 20);
            memcpy(rbuf + 4u * i, &v, 4u);
        }
        n = rle1_encode_buf(rbuf, 1200000, blk, 899981);
        free(rbuf);
    }
    cyclic_bwt_encode(blk, (uint32_t)n, L_out, p_out);
    *n_out = n;
    free(blk);
}

static void run_arm12(
    LKernel *lk, R4Kernel *rk, FlatQ4K *fq, SegKernel *sk,
    int arm, const uint8_t *L, const uint16_t *recs, size_t n_recs,
    const uint32_t counts[256], const uint32_t per[256],
    uint32_t n, uint32_t p, uint8_t *out,
    double *t_con, double *t_em
) {
    double t0 = now_sec(), t1, t2;
    int eff_arm = arm;
    int unified_med = 0;
    if (arm == 6) {
        if ((uint64_t)n_recs * 2ULL >= (uint64_t)n) eff_arm = 5;
        else if ((uint64_t)n_recs * 5ULL < (uint64_t)n) eff_arm = 4;
        else unified_med = 1;
    }

    if (unified_med) {
        seg_fill_avx2(recs, n_recs, n, counts, sk->tt);
        t1 = now_sec();
        if (n < 65536u) seg_walk_back_plain(sk->tt, n, p, out);
        else { seg_room_segments(sk, n, p, 1); seg_assemble_reversed_avx2(sk, out); }
        t2 = now_sec();
    } else {
        switch (eff_arm) {
            case 0:
                open_construct(lk, recs, n_recs, n, p, 16u, 0, 0, 0);
                t1 = now_sec();
                open_emit_orig(lk, n, out);
                t2 = now_sec();
                break;
            case 1:
                seg_fill_avx2(recs, n_recs, n, counts, sk->tt);
                t1 = now_sec();
                if (n < 65536u) seg_walk_back_plain(sk->tt, n, p, out);
                else { seg_room_segments(sk, n, p, 1); seg_assemble_reversed(sk, out); }
                t2 = now_sec();
                break;
            case 2:
                seg_build_one_pass(L, n, counts, sk->tt);
                t1 = now_sec();
                if (n < 65536u) seg_walk_fwd_plain(sk->tt, n, p, out);
                else { seg_room_segments(sk, n, p, 0); seg_assemble(sk, out); }
                t2 = now_sec();
                break;
            case 3:
                flatq_w24_r4_construct(rk, fq, recs, n_recs, counts, per, n, p, 48u);
                t1 = now_sec();
                r4_bp4k_bvsmc_emit_1p(rk, fq, n, 48u, out);
                t2 = now_sec();
                break;
            case 4:
                aht_flatq_w24_r4_construct(rk, fq, recs, n_recs, counts, per, n, p, 48u);
                t1 = now_sec();
                r4_bp4k_bvsmc_emit_1p(rk, fq, n, 48u, out);
                t2 = now_sec();
                break;
            case 5:
                seqlf_col_build(L, n, counts, sk->tt);
                t1 = now_sec();
                if (n < 65536u) seg_walk_back_plain(sk->tt, n, p, out);
                else { seg_room_segments(sk, n, p, 1); seg_assemble_reversed_avx2(sk, out); }
                t2 = now_sec();
                break;
            default:
                abort();
        }
    }
    *t_con = (t1 - t0) * 1e3;
    *t_em  = (t2 - t1) * 1e3;
}

static void run_cycle12_selftests(LKernel *lk, R4Kernel *rk, FlatQ4K *fq, SegKernel *sk) {
    uint8_t *L = (uint8_t *)malloc(80000);
    uint8_t *T = (uint8_t *)malloc(80000);
    uint16_t *recs = (uint16_t *)malloc(80000 * 2u);
    uint8_t *ref = (uint8_t *)malloc(80000);
    uint8_t *guard = (uint8_t *)malloc(80128);
    uint8_t *out = guard + 32;
    uint32_t counts[256], per[256];

    for (int ci = 0; ci < 8; ci++) {
        size_t n = 0;
        uint32_t p = 0;
        if (ci == 0) { n = 1; p = 0; L[0] = 'z'; }
        else if (ci == 1) { n = 5; p = 3; memcpy(L, "babaa", 5); }
        else if (ci == 2) { n = 4096; p = 777; for (size_t i = 0; i < n; i++) L[i] = (uint8_t)((i / 8u) % 19u + 'a'); }
        else if (ci == 3) { n = 5000; p = 1337; memset(L, 'q', n); }
        else if (ci == 4) { n = 300; p = 190; memset(L, 'x', 60); memset(L + 60, 'a', 200); memset(L + 260, 'y', 40); }
        else if (ci == 5) {
            n = 70005;
            for (size_t i = 0; i < n; i++) T[i] = (uint8_t)('0' + (i % 7u));
            cyclic_bwt_encode(T, (uint32_t)n, L, &p);
        } else if (ci == 6) {
            size_t pos = 0;
            while (pos + 5 <= 68000) { T[pos]=T[pos+1]=T[pos+2]=T[pos+3]='x'; T[pos+4]=251u; pos += 5; }
            T[pos++] = 'y'; T[pos++] = 'z';
            n = pos;
            cyclic_bwt_encode(T, (uint32_t)n, L, &p);
        } else {
            uint64_t rng = 0xdeadbeefcafebabeULL;
            n = 70000; p = 12345;
            for (size_t i = 0; i < n; i++) L[i] = (uint8_t)xs64(&rng);
        }

        size_t nr = to_capped_records_with_meta(L, n, recs, counts, per);
        dense_lf_reference(L, n, p, ref);

        for (int arm = 0; arm < 7; arm++) {
            memset(guard, 0xA5, n + 64u);
            double tc = 0, te = 0;
            run_arm12(lk, rk, fq, sk, arm, L, recs, nr, counts, per, (uint32_t)n, p, out, &tc, &te);
            assert(memcmp(out, ref, n) == 0);
            for (int g = 0; g < 32; g++) {
                assert(guard[g] == 0xA5 && out[n + (size_t)g] == 0xA5);
            }
        }

        memset(guard, 0xA5, n + 64u);
        memcpy(out, L, n);
        seqlf_col_build(out, n, counts, sk->tt);
        if (n < 65536u) seg_walk_back_plain(sk->tt, n, p, out);
        else { seg_room_segments(sk, n, p, 1); seg_assemble_reversed_avx2(sk, out); }
        assert(memcmp(out, ref, n) == 0);
        for (int g = 0; g < 32; g++) {
            assert(guard[g] == 0xA5 && out[n + (size_t)g] == 0xA5);
        }
    }

    free(L); free(T); free(recs); free(ref); free(guard);
    printf("[PASS] Adversarial self-tests (8/8 incl. >65k top/bottom AHT spirals, multi-cycle, true in-place out==ll, 0xA5 canaries) across all 7 arms.\n\n");
}

int main(int argc, char **argv) {
    setvbuf(stdout, NULL, _IONBF, 0);
    const char *dir = (argc > 1) ? argv[1] : "/tmp/admitted-bwt/admitted-bwt";
    const int rounds = (argc > 2) ? atoi(argv[2]) : 25;
    assert(rounds >= 1 && rounds <= 64);

    const char *wiki_files[6] = {"block-0.L", "block-2.L", "block-85.L", "block-143.L", "block-210.L", "block-207.L"};
    const uint32_t wiki_ps[6] = {264045,      320807,      325088,       802868,        791429,        205180};
    const char *block_names[13] = {
        "block-0.L", "block-2.L", "block-85.L", "block-143.L", "block-210.L", "block-207.L",
        "onebyte",   "pattern",   "rle_spiral", "runs",        "text0",       "random",      "records"
    };

    LKernel   lk; lkernel_init(&lk, 1000000);
    R4Kernel  rk; r4_init(&rk, 1000000);
    FlatQ4K   fq; fq4k_init(&fq, 1000000);
    SegKernel sk; seg_init(&sk);

    run_cycle12_selftests(&lk, &rk, &fq, &sk);

    uint8_t  *L     = (uint8_t *)malloc(1000000);
    uint16_t *recs  = (uint16_t *)malloc(1000000 * 2u);
    uint8_t  *ref   = (uint8_t *)malloc(1000000);
    uint8_t  *guard = (uint8_t *)malloc(1000128);
    uint8_t  *out   = guard + 32;
    uint32_t counts[256], per[256];

    int pass_sparse_ship20 = 0;
    int pass_spiral_c11_15 = 0;
    int pass_wiki_open60   = 0;
    int pass_wiki_c11_103  = 0;
    int pass_ovf_256k      = 0;
    int pass_col_tot80     = 0;
    int pass_col_bld45     = 0;
    int pass_uni_noharm102 = 0;
    int pass_uni_win78     = 0;

    printf("%-11s %6s | %7s %7s %7s | %7s %7s %7s | %7s | %7s %7s %6s\n",
           "Block", "r/n", "Open16", "SegRuns", "SegCol", "C11-FQ", "AHT-FQ", "SeqLF", "Unified", "Uni/Shp", "AHT/C11", "OvfKB");
    printf("-----------------------------------------------------------------------------------------------------------------------\n");

    for (int bi = 0; bi < 13; bi++) {
        size_t n = 0;
        uint32_t p = 0;
        if (bi < 6) {
            char path[512];
            snprintf(path, sizeof(path), "%s/%s", dir, wiki_files[bi]);
            FILE *fp = fopen(path, "rb");
            if (!fp) { fprintf(stderr, "Cannot open %s\n", path); return 1; }
            n = fread(L, 1, 1000000, fp);
            fclose(fp);
            p = wiki_ps[bi];
        } else {
            build_non_wiki_block(bi - 6, L, &n, &p);
        }

        size_t nr = to_capped_records_with_meta(L, n, recs, counts, per);
        dense_lf_reference(L, n, p, ref);

        double t_con[7][64], t_tot[7][64];
        uint32_t aht_ovf_kb = 0;

        for (int r = -3; r < rounds; r++) {
            for (int step = 0; step < 7; step++) {
                int arm = (step + (r + 3)) % 7;
                if (r == -3) memset(guard, 0xA5, n + 64u);
                double tc = 0, te = 0;
                run_arm12(&lk, &rk, &fq, &sk, arm, L, recs, nr, counts, per, (uint32_t)n, p, out, &tc, &te);
                if (r == -3) {
                    assert(memcmp(out, ref, n) == 0);
                    for (int g = 0; g < 32; g++) {
                        assert(guard[g] == 0xA5 && out[n + (size_t)g] == 0xA5);
                    }
                    if (arm == 4) aht_ovf_kb = fq.peak_ovf4k_pages * 4u;
                    if (arm == 5) {
                        memcpy(out, L, n);
                        seqlf_col_build(out, n, counts, sk.tt);
                        seg_room_segments(&sk, n, p, 1);
                        seg_assemble_reversed_avx2(&sk, out);
                        assert(memcmp(out, ref, n) == 0);
                    }
                } else if (r >= 0) {
                    t_con[arm][r] = tc;
                    t_tot[arm][r] = tc + te;
                }
            }
        }

        double med_tot[7], med_con[7];
        for (int a = 0; a < 7; a++) {
            qsort(t_tot[a], (size_t)rounds, sizeof(double), cmp_double);
            qsort(t_con[a], (size_t)rounds, sizeof(double), cmp_double);
            med_tot[a] = t_tot[a][rounds / 2];
            med_con[a] = t_con[a][rounds / 2];
        }

        int is_col = ((uint64_t)nr * 2ULL >= (uint64_t)n);
        int open_admit = (!is_col && (uint64_t)nr * 64ULL >= (uint64_t)n && (uint64_t)nr * 20ULL < (uint64_t)n * 3ULL);
        double shipped_tot = is_col ? med_tot[2] : (open_admit ? med_tot[0] : med_tot[1]);

        double uni_ratio = med_tot[6] / shipped_tot;
        double aht_c11   = med_tot[4] / med_tot[3];

        printf("%-11s %6.4f | %7.3f %7.3f %7.3f | %7.3f %7.3f %7.3f | %7.3f |  %5.3fx  %5.3fx %5u\n",
               block_names[bi], (double)nr / (double)n,
               med_tot[0], med_tot[1], med_tot[2],
               med_tot[3], med_tot[4], med_tot[5],
               med_tot[6], uni_ratio, aht_c11, aht_ovf_kb);

        if (bi < 6) {
            if (med_tot[4] <= 0.60 * med_tot[0]) pass_wiki_open60++;
            if (med_tot[4] <= 1.03 * med_tot[3]) pass_wiki_c11_103++;
            if (aht_ovf_kb <= 256u) pass_ovf_256k++;
        } else if (bi >= 6 && bi <= 8) {
            if (med_tot[4] <= 0.20 * shipped_tot) pass_sparse_ship20++;
            if (bi >= 7 && med_tot[4] <= 0.15 * med_tot[3]) pass_spiral_c11_15++;
            if (aht_ovf_kb <= 256u) pass_ovf_256k++;
        } else if (bi >= 11) {
            if (med_tot[5] <= 0.80 * med_tot[2]) pass_col_tot80++;
            if (med_con[5] <= 0.45 * med_con[2]) pass_col_bld45++;
            printf("  [ColSplit %-7s] SegCol Build=%.3f ms Tot=%.3f ms -> SeqLF Build=%.3f ms (%.3fx) Tot=%.3f ms (%.3fx)\n",
                   block_names[bi], med_con[2], med_tot[2], med_con[5], med_con[5]/med_con[2], med_tot[5], med_tot[5]/med_tot[2]);
        }

        if (uni_ratio <= 1.02) pass_uni_noharm102++;
        if (uni_ratio <= 0.80) pass_uni_win78++;
    }

    printf("\n=== Pre-Registered Gate G5 Summary (Cycle 12: AHT-FlatQ-BP4K + SeqLF-Col) ===\n");
    printf("  G5.2a Sparse/Spiral AHT <= 0.20x Shipped:      %d/3  (req 3/3)\n", pass_sparse_ship20);
    printf("  G5.2b Spiral AHT <= 0.15x Cycle 11 FlatQ:      %d/2  (req 2/2)\n", pass_spiral_c11_15);
    printf("  G5.3a Wikidata AHT <= 0.60x Open-qi16:         %d/6  (req 6/6)\n", pass_wiki_open60);
    printf("  G5.3b Wikidata AHT <= 1.03x Cycle 11 FlatQ:    %d/6  (req 6/6)\n", pass_wiki_c11_103);
    printf("  G5.3c Admitted+Sparse BP4K Ovf <= 256 KiB:     %d/9  (req 9/9)\n", pass_ovf_256k);
    printf("  G5.4a Column SeqLF Tot <= 0.80x Seg-Col Tot:   %d/2  (req 2/2)\n", pass_col_tot80);
    printf("  G5.4b Column SeqLF Build <= 0.45x Seg-Col Bld: %d/2  (req 2/2)\n", pass_col_bld45);
    printf("  G5.5a Unified-C12 <= 1.02x Shipped (no cliff): %d/13 (req 13/13)\n", pass_uni_noharm102);
    printf("  G5.5b Unified-C12 <= 0.80x Shipped:            %d/13 (req >=11/13)\n", pass_uni_win78);

    int overall_pass = (
        pass_sparse_ship20 == 3 &&
        pass_spiral_c11_15 == 2 &&
        pass_wiki_open60   == 6 &&
        pass_wiki_c11_103  == 6 &&
        pass_ovf_256k      == 9 &&
        pass_col_tot80     == 2 &&
        pass_col_bld45     == 2 &&
        pass_uni_noharm102 == 13 &&
        pass_uni_win78     >= 11
    );
    printf("  OVERALL PRE-REGISTERED GATE G5 VERDICT: %s\n", overall_pass ? "PASS" : "FAIL");

    free(L); free(recs); free(ref); free(guard);
    seg_free(&sk);
    lkernel_free(&lk);
    r4_free(&rk);
    fq4k_free(&fq);
    return overall_pass ? 0 : 2;
}
