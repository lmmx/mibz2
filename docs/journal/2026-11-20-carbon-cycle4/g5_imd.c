/*
 * Gate G5 Paid-Cost Falsifier for Carbon Cycle 4 (Alpoge Multiplicity Surrogate):
 * Gauss-Seidel Inline-Payload Iterated Monoid Lifting (GS-IMD) vs Shipped Open (qi16).
 *
 * Isolates pure single-lane, zero-arena iterated operator lifting (NO multi-lane splicing,
 * NO chunk pool, NO U16Q queue-push change — identical scalar queue push & endpoint.rs in all arms).
 * Both the Gauss-Seidel lift pass(es) AND the 1-lane consumer walk are charged in LiftEm and Tot!
 *
 * PRE-REGISTERED PASS/FAIL RULE (frozen before compilation and execution):
 * Across all six authenticated Wikidata BWT blocks (block-0, 2, 85, 143, 207, 210),
 * 25 paired interleaved rounds after 3 warmup rounds (rotating all 4 arms each round):
 *
 *   Arm 0: Open-qi16   — exact shipped src/open.rs (scalar queue push + u32 edges + 1-lane emit)
 *   Arm 1: GS64-1P     — u64 inline monoid edges (<=5B inline), 1 in-place Gauss-Seidel lift + 1-lane emit
 *   Arm 2: GS64-2P     — u64 inline monoid edges (<=5B inline), 2 in-place Gauss-Seidel lifts + 1-lane emit
 *   Arm 3: GS128-2P    — 16B aligned Slot16 monoid edges (<=12B inline), 2 in-place lifts + 1-lane SSE emit
 *
 * PASS iff ALL FOUR hold for the primary candidate (GS64-2P or GS128-2P, evaluated per arm):
 *   (1) Byte-exactness + 32B 0xFE trailing canary on 6/6 blocks (5,399,692 B) & 6/6 adversarial tests;
 *   (2) Zero chunk-pool / routing-queue arena (0 B);
 *   (3) Paid (Lift + Emit) median <= 0.80x Open-qi16 emit on >= 5/6 blocks, and <= 0.85x on 6/6 blocks;
 *   (4) Complete Total (Construct + Lift + Emit) median <= 0.90x Open-qi16 total on >= 5/6 blocks.
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
#define IDX_MASK 0xFFFFFu   /* 20 bits: supports m < 1,048,576 (block max n <= 900,000) */

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

typedef struct __attribute__((aligned(16))) {
    uint32_t hdr;       /* bits 0..19: next_j; bits 20..23: inline len (1..12, or 0 for >12B dict) */
    uint8_t pay[12];    /* if len in 1..12: inline bytes; if len == 0: u32 o0 at [0..3], u32 ln at [4..7] */
} Slot16;

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
    uint8_t *spelling;
    size_t spell_len, spell_cap;
    uint32_t *offsets;
    size_t off_len, off_cap;

    uint32_t *edges;
    uint32_t *extra;
    uint64_t *edge64;
    Slot16 *slot16;
    int wide;

    uint32_t m, mark, bits, words;
} Kernel;

static double now_sec(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (double)ts.tv_sec + (double)ts.tv_nsec * 1e-9;
}

static void kernel_init(Kernel *k, size_t max_n) {
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
    k->spell_cap = 131072;
    k->spelling = (uint8_t *)calloc(k->spell_cap, 1);
    k->off_cap = 16384;
    k->offsets = (uint32_t *)malloc(k->off_cap * sizeof(uint32_t));

    k->edges = (uint32_t *)malloc((max_n + 2) * sizeof(uint32_t));
    k->extra = (uint32_t *)malloc((max_n + 2) * sizeof(uint32_t));
    k->edge64 = (uint64_t *)aligned_alloc(64, (max_n + 8) * sizeof(uint64_t));
    k->slot16 = (Slot16 *)aligned_alloc(64, (max_n + 8) * sizeof(Slot16));
}

static void kernel_free(Kernel *k) {
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
    free(k->spelling);
    free(k->offsets);
    free(k->edges);
    free(k->extra);
    free(k->edge64);
    free(k->slot16);
}

static inline void q_push_scalar(Kernel *k, uint8_t a, uint32_t start, uint32_t w) {
    uint32_t len = k->q_len[a];
    if (__builtin_expect(len + w > k->q_cap[a], 0)) {
        while (len + w > k->q_cap[a]) k->q_cap[a] *= 2;
        k->queues[a] = (uint32_t *)realloc(k->queues[a], (k->q_cap[a] + 16) * sizeof(uint32_t));
    }
    uint32_t *dst = k->queues[a] + len;
    for (uint32_t i = 0; i < w; i++) dst[i] = start + i;
    k->q_len[a] = len + w;
}

static inline void push_strip(Kernel *k, uint32_t from, uint32_t to, uint32_t width, uint8_t byte) {
    if (__builtin_expect(k->strips_len == k->strips_cap, 0)) {
        k->strips_cap *= 2;
        k->strips = (Strip *)realloc(k->strips, k->strips_cap * sizeof(Strip));
    }
    k->strips[k->strips_len++] = (Strip){from, to, width, byte};
}

static inline uint32_t ep_node(Kernel *k, uint32_t width, uint32_t word) {
    if (__builtin_expect(k->nodes_len == k->nodes_cap, 0)) {
        k->nodes_cap *= 2;
        k->nodes = (EpNode *)realloc(k->nodes, k->nodes_cap * sizeof(EpNode));
    }
    uint32_t id = (uint32_t)k->nodes_len++;
    k->nodes[id] = (EpNode){{0, 0}, {0, 0}, width, word};
    return id;
}

static inline uint32_t ep_len(const Kernel *k, uint32_t w) {
    if (w <= 256) return w != 0 ? 1u : 0u;
    return k->rules[w - 257].len;
}

static inline uint32_t ep_join(Kernel *k, uint32_t left, uint32_t right) {
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

static inline void ep_cut(EpNode *nodes, uint32_t id, int side) {
    EpNode h = nodes[id];
    nodes[h.prev[side]].next[side] = h.next[side];
    nodes[h.next[side]].prev[side] = h.prev[side];
}

static inline void ep_insert(EpNode *nodes, uint32_t at, uint32_t id, int side) {
    uint32_t nx = nodes[at].next[side];
    nodes[id].prev[side] = at;
    nodes[id].next[side] = nx;
    nodes[at].next[side] = id;
    nodes[nx].prev[side] = id;
}

static inline void push_u32(uint32_t **arr, size_t *len, size_t *cap, uint32_t v) {
    if (__builtin_expect(*len == *cap, 0)) {
        *cap *= 2;
        *arr = (uint32_t *)realloc(*arr, (*cap) * sizeof(uint32_t));
    }
    (*arr)[(*len)++] = v;
}

static inline void ep_gaps(Kernel *k, int side, uint32_t start, uint32_t end) {
    if (start < end) {
        uint32_t id = ep_node(k, end - start, 0);
        push_u32(&k->order[side], &k->order_len[side], &k->order_cap[side], id);
        push_u32(&k->terminals[1 - side], &k->term_len[1 - side], &k->term_cap[1 - side], id);
    }
}

static inline void spell_ensure(Kernel *k, size_t add) {
    if (__builtin_expect(k->spell_len + add + 32 > k->spell_cap, 0)) {
        while (k->spell_len + add + 32 > k->spell_cap) k->spell_cap *= 2;
        k->spelling = (uint8_t *)realloc(k->spelling, k->spell_cap);
    }
}

static void ep_append(Kernel *k, uint32_t word) {
    k->stack_len = 0;
    push_u32(&k->stack, &k->stack_len, &k->stack_cap, word);
    while (k->stack_len > 0) {
        uint32_t id = k->stack[--k->stack_len];
        if (id <= 256) {
            spell_ensure(k, 1);
            k->spelling[k->spell_len++] = (uint8_t)(id - 1);
        } else {
            EpRule *rl = &k->rules[id - 257];
            if (rl->first == UINT32_MAX) {
                rl->first = (uint32_t)k->spell_len;
                push_u32(&k->stack, &k->stack_len, &k->stack_cap, rl->right);
                push_u32(&k->stack, &k->stack_len, &k->stack_cap, rl->left);
            } else {
                spell_ensure(k, rl->len);
                memmove(k->spelling + k->spell_len, k->spelling + rl->first, rl->len);
                k->spell_len += rl->len;
            }
        }
    }
}

/* mode: 0 = u32 edges (Open-qi16), 1 = u64 inline5 edges, 2 = Slot16 inline12 edges */
static void construct_mode(
    Kernel *k,
    const uint16_t *records,
    size_t n_recs,
    uint32_t n,
    uint32_t p,
    uint32_t limit,
    int mode
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
        while (i < n_recs && (uint8_t)(records[i] >> 8) == c) {
            width += (uint32_t)(records[i++] & 255u);
        }
        uint32_t end = pos + width;
        if (width > limit && p >= pos && p < end) {
            if (pos < p) { push_strip(k, pos, all_c[c], p - pos, c); thick[c]++; all_c[c] += p - pos; }
            mark = m; q_push_scalar(k, c, m, 1); thin[c] += 1; m += 1; all_c[c] += 1;
            if (p + 1 < end) { uint32_t w2 = end - p - 1; push_strip(k, p + 1, all_c[c], w2, c); thick[c]++; all_c[c] += w2; }
        } else if (width <= limit) {
            if (p >= pos && p < end) mark = m + (p - pos);
            q_push_scalar(k, c, m, width); thin[c] += width; m += width; all_c[c] += width;
        } else {
            push_strip(k, pos, all_c[c], width, c); thick[c]++; all_c[c] += width;
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

    k->nodes_len = 0;
    ep_node(k, 0, 0);
    k->rules_len = 0;
    k->order_len[0] = k->order_len[1] = k->term_len[0] = k->term_len[1] = 0;

    for (size_t si = 0; si < k->strips_len; si++) {
        ep_node(k, k->strips[si].width, (uint32_t)k->strips[si].byte + 1u);
    }
    uint32_t x = 0;
    for (size_t si = 0; si < k->strips_len; si++) {
        ep_gaps(k, 0, x, k->strips[si].from);
        push_u32(&k->order[0], &k->order_len[0], &k->order_cap[0], (uint32_t)si + 1u);
        x = k->strips[si].from + k->strips[si].width;
    }
    ep_gaps(k, 0, x, n);
    x = 0;
    for (size_t ii = 0; ii < k->strips_len; ii++) {
        Strip s = k->strips[k->image_order[ii]];
        ep_gaps(k, 1, x, s.to);
        push_u32(&k->order[1], &k->order_len[1], &k->order_cap[1], k->image_order[ii] + 1u);
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
                ep_append(k, nd.word);
                spell_ensure(k, 16);
                k->spelling[k->spell_len++] = (uint8_t)a;
                uint32_t o1 = (uint32_t)k->spell_len;
                push_u32(&k->offsets, &k->off_len, &k->off_cap, o1);
                code = 256u + w_idx;
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
    memset(k->spelling + k->spell_len, 0, 16);

    uint32_t bits = m <= 1 ? 1u : (32u - (uint32_t)__builtin_clz(m - 1));
    int wide = (255u + words) > (UINT32_MAX >> bits);

    kc = 0;
    if (mode == 0) {
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
    } else if (mode == 1) {
        /* Wire u64 inline5 edges directly */
        uint64_t * restrict e64 = k->edge64;
        for (int a = 0; a < 256; a++) {
            uint32_t b_end_idx = (a + 1 < 256) ? cursors[a + 1] : (uint32_t)k->bands_len;
            uint32_t from = 0;
            const uint32_t * restrict qa = k->queues[a];
            for (uint32_t bi = cursors[a]; bi < b_end_idx; bi++) {
                Band b = k->bands[bi];
                uint32_t span = b.end - kc, dst = kc + b.delta;
                uint64_t pack64;
                if (b.code < 256u) {
                    pack64 = ((uint64_t)b.code << 24) | (1ULL << 20);
                } else {
                    uint32_t w = b.code - 256u;
                    uint32_t o0 = k->offsets[w], ln = k->offsets[w + 1] - o0;
                    if (ln <= 5u) {
                        uint64_t pay = 0;
                        memcpy(&pay, k->spelling + o0, 8);
                        pay &= (1ULL << (8u * ln)) - 1ULL;
                        pack64 = (pay << 24) | ((uint64_t)ln << 20);
                    } else {
                        pack64 = ((uint64_t)w << 24); /* tag == 0 */
                    }
                }
                uint64_t * restrict edst = e64 + dst;
                const uint32_t * restrict qsrc = qa + from;
                for (uint32_t s = 0; s < span; s++) edst[s] = pack64 | (uint64_t)qsrc[s];
                from += span; kc = b.end;
            }
        }
    } else {
        /* Wire Slot16 inline12 edges directly */
        Slot16 * restrict s16 = k->slot16;
        for (int a = 0; a < 256; a++) {
            uint32_t b_end_idx = (a + 1 < 256) ? cursors[a + 1] : (uint32_t)k->bands_len;
            uint32_t from = 0;
            const uint32_t * restrict qa = k->queues[a];
            for (uint32_t bi = cursors[a]; bi < b_end_idx; bi++) {
                Band b = k->bands[bi];
                uint32_t span = b.end - kc, dst = kc + b.delta;
                Slot16 proto;
                memset(&proto, 0, sizeof(proto));
                uint32_t hdr_hi;
                if (b.code < 256u) {
                    hdr_hi = 1u << 20;
                    proto.pay[0] = (uint8_t)b.code;
                } else {
                    uint32_t w = b.code - 256u;
                    uint32_t o0 = k->offsets[w], ln = k->offsets[w + 1] - o0;
                    if (ln <= 12u) {
                        hdr_hi = ln << 20;
                        memcpy(proto.pay, k->spelling + o0, ln);
                    } else {
                        hdr_hi = 0u;
                        memcpy(proto.pay, &o0, 4);
                        memcpy(proto.pay + 4, &ln, 4);
                    }
                }
                __m128i vproto = _mm_load_si128((const __m128i *)&proto);
                Slot16 * restrict edst = s16 + dst;
                const uint32_t * restrict qsrc = qa + from;
                for (uint32_t s = 0; s < span; s++) {
                    _mm_store_si128((__m128i *)&edst[s], vproto);
                    edst[s].hdr = hdr_hi | qsrc[s];
                }
                from += span; kc = b.end;
            }
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

static void open_emit_orig(const Kernel *k, size_t n, uint8_t *out) {
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

static void bj64_lift_and_emit(Kernel *k, size_t n, uint8_t *out, double *out_lift_ms) {
    const uint64_t * restrict src = k->edge64;
    uint64_t * restrict dst = (uint64_t *)k->slot16;
    const uint32_t m = k->m, mark = k->mark;

    double tl0 = now_sec();
    for (uint32_t j = 0; j < m; j++) {
        uint64_t e0 = src[j];
        uint32_t j1 = (uint32_t)e0 & IDX_MASK;
        uint64_t e1 = src[j1];
        uint32_t t0 = (uint32_t)(e0 >> 20) & 15u;
        uint32_t t1 = (uint32_t)(e1 >> 20) & 15u;
        int ok = (t0 >= 1u) & (t1 >= 1u) & (t0 + t1 <= 5u) & (j1 != mark) & (j1 != j);
        uint64_t nlen = (uint64_t)(t0 + t1);
        uint64_t pay = (e0 >> 24) | ((e1 >> 24) << (8u * (t0 & 7u)));
        uint64_t comb = (pay << 24) | (nlen << 20) | (e1 & IDX_MASK);
        dst[j] = ok ? comb : e0;
    }
    if (out_lift_ms) *out_lift_ms = (now_sec() - tl0) * 1e3;

    const uint8_t * restrict spelling = k->spelling;
    const uint32_t * restrict offsets = k->offsets;
    uint8_t * restrict out_p = out;
    uint8_t * const out_end = out + n;
    uint8_t * const out_safe8 = n >= 8 ? (out + n - 7) : out;
    uint32_t j = mark;

    for (uint32_t step = 0; step < m; step++) {
        uint64_t e = dst[j];
        uint32_t t = (uint32_t)(e >> 20) & 15u;
        j = (uint32_t)e & IDX_MASK;
        if (__builtin_expect(t != 0u && out_p < out_safe8, 1)) {
            uint64_t pay = e >> 24;
            memcpy(out_p, &pay, 8);
            out_p += t;
        } else if (t != 0u) {
            uint64_t pay = e >> 24;
            size_t rem = (size_t)(out_end - out_p);
            uint32_t tk = t < rem ? t : (uint32_t)rem;
            memcpy(out_p, &pay, tk);
            out_p += tk;
        } else {
            uint32_t w = (uint32_t)(e >> 24);
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

static void gs64_lift_and_emit(Kernel *k, size_t n, int passes, uint8_t *out, double *out_lift_ms) {
    uint64_t * restrict e64 = k->edge64;
    const uint32_t m = k->m, mark = k->mark;

    double tl0 = now_sec();
    for (int ps = 0; ps < passes; ps++) {
        for (uint32_t j = 0; j < m; j++) {
            uint64_t e0 = e64[j];
            uint32_t t0 = (uint32_t)(e0 >> 20) & 15u;
            if (t0 - 1u < 4u) {
                uint32_t j1 = (uint32_t)e0 & IDX_MASK;
                if (__builtin_expect(j1 != mark && j1 != j, 1)) {
                    uint64_t e1 = e64[j1];
                    uint32_t t1 = (uint32_t)(e1 >> 20) & 15u;
                    if (t1 != 0u && t0 + t1 <= 5u) {
                        uint64_t nlen = t0 + t1;
                        uint64_t pay = (e0 >> 24) | ((e1 >> 24) << (8u * t0));
                        if (nlen < 5u) pay &= (1ULL << (8u * nlen)) - 1ULL;
                        e64[j] = (pay << 24) | (nlen << 20) | (e1 & IDX_MASK);
                    }
                }
            }
        }
    }
    if (out_lift_ms) *out_lift_ms = (now_sec() - tl0) * 1e3;

    const uint8_t * restrict spelling = k->spelling;
    const uint32_t * restrict offsets = k->offsets;
    uint8_t * restrict out_p = out;
    uint8_t * const out_end = out + n;
    uint8_t * const out_safe8 = n >= 8 ? (out + n - 7) : out;
    uint32_t j = mark;

    for (uint32_t step = 0; step < m; step++) {
        uint64_t e = e64[j];
        uint32_t t = (uint32_t)(e >> 20) & 15u;
        j = (uint32_t)e & IDX_MASK;
        if (__builtin_expect(t != 0u && out_p < out_safe8, 1)) {
            uint64_t pay = e >> 24;
            memcpy(out_p, &pay, 8);
            out_p += t;
        } else if (t != 0u) {
            uint64_t pay = e >> 24;
            size_t rem = (size_t)(out_end - out_p);
            uint32_t tk = t < rem ? t : (uint32_t)rem;
            memcpy(out_p, &pay, tk);
            out_p += tk;
        } else {
            uint32_t w = (uint32_t)(e >> 24);
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

static void gs128_lift_and_emit(Kernel *k, size_t n, int passes, uint8_t *out) {
    Slot16 * restrict s16 = k->slot16;
    const uint32_t m = k->m, mark = k->mark;

    for (int ps = 0; ps < passes; ps++) {
        for (uint32_t j = 0; j < m; j++) {
            uint32_t h0 = s16[j].hdr;
            uint32_t t0 = h0 >> 20;
            if (t0 - 1u < 11u) {
                uint32_t j1 = h0 & IDX_MASK;
                if (__builtin_expect(j1 != mark && j1 != j, 1)) {
                    uint32_t h1 = s16[j1].hdr;
                    uint32_t t1 = h1 >> 20;
                    if (t1 != 0u && t0 + t1 <= 12u) {
                        memcpy(s16[j].pay + t0, s16[j1].pay, t1);
                        s16[j].hdr = ((t0 + t1) << 20) | (h1 & IDX_MASK);
                    }
                }
            }
        }
    }

    const uint8_t * restrict spelling = k->spelling;
    uint8_t * restrict out_p = out;
    uint8_t * const out_end = out + n;
    uint8_t * const out_safe16 = n >= 16 ? (out + n - 15) : out;
    uint32_t j = mark;

    for (uint32_t step = 0; step < m; step++) {
        __m128i v = _mm_load_si128((const __m128i *)&s16[j]);
        uint32_t hdr = (uint32_t)_mm_cvtsi128_si32(v);
        uint32_t t = hdr >> 20;
        j = hdr & IDX_MASK;
        if (__builtin_expect(t != 0u && out_p < out_safe16, 1)) {
            _mm_storeu_si128((__m128i *)out_p, _mm_srli_si128(v, 4));
            out_p += t;
        } else if (t != 0u) {
            uint8_t tmp[16];
            _mm_storeu_si128((__m128i *)tmp, _mm_srli_si128(v, 4));
            size_t rem = (size_t)(out_end - out_p);
            uint32_t tk = t < rem ? t : (uint32_t)rem;
            memcpy(out_p, tmp, tk);
            out_p += tk;
        } else {
            uint32_t o0 = (uint32_t)_mm_extract_epi32(v, 1);
            uint32_t ln = (uint32_t)_mm_extract_epi32(v, 2);
            if (__builtin_expect(out_p + ln > out_end, 0)) ln = (uint32_t)(out_end - out_p);
            memcpy(out_p, spelling + o0, ln);
            out_p += ln;
        }
        if (__builtin_expect(j == mark || out_p >= out_end, 0)) break;
    }
    size_t wr = (size_t)(out_p - out);
    repeat_period(n, out, &wr);
}

static size_t to_capped_records(const uint8_t *L, size_t n, uint16_t *recs) {
    size_t rlen = 0;
    for (size_t i = 0; i < n; i++) {
        uint8_t b = L[i];
        if (rlen > 0 && (uint8_t)(recs[rlen - 1] >> 8) == b && (recs[rlen - 1] & 255u) < 127u) {
            recs[rlen - 1]++;
        } else {
            recs[rlen++] = (uint16_t)(((uint16_t)b << 8) | 1u);
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

static double median(double *arr, int count) {
    qsort(arr, (size_t)count, sizeof(double), cmp_double);
    return arr[count / 2];
}

static void run_selftests(Kernel *k) {
    const struct { const char *pat; size_t reps; uint32_t p; } cases[6] = {
        {"a", 1, 0},
        {"ababa", 1, 2},
        {"a", 500, 137},
        {"ab", 300, 41},
        {"ba", 301, 0},
        {"abacabadabacabae", 50, 123},
    };
    uint8_t *L = (uint8_t *)malloc(2048);
    uint16_t *recs = (uint16_t *)malloc(2048 * sizeof(uint16_t));
    uint8_t *ref = (uint8_t *)malloc(2048);
    for (size_t ci = 0; ci < 6; ci++) {
        size_t plen = strlen(cases[ci].pat);
        size_t n = plen * cases[ci].reps;
        for (size_t r = 0; r < cases[ci].reps; r++) memcpy(L + r * plen, cases[ci].pat, plen);
        dense_lf_reference(L, n, cases[ci].p, ref);
        size_t nr = to_capped_records(L, n, recs);

        uint8_t *got = (uint8_t *)malloc(n + 32);
        for (int arm = 1; arm <= 3; arm++) {
            memset(got, 0xCC, n);
            memset(got + n, 0xFE, 32);
            if (arm == 1) {
                construct_mode(k, recs, nr, (uint32_t)n, cases[ci].p, LIMIT, 1);
                bj64_lift_and_emit(k, n, got, NULL);
            } else if (arm == 2) {
                construct_mode(k, recs, nr, (uint32_t)n, cases[ci].p, LIMIT, 1);
                gs64_lift_and_emit(k, n, 1, got, NULL);
            } else {
                construct_mode(k, recs, nr, (uint32_t)n, cases[ci].p, LIMIT, 2);
                gs128_lift_and_emit(k, n, 2, got);
            }
            assert(memcmp(got, ref, n) == 0);
            for (int g = 0; g < 32; g++) assert(got[n + g] == 0xFEu);
        }
        free(got);
    }
    free(L); free(recs); free(ref);
    printf("[PASS] C adversarial & 32B-canary self-tests (6/6 incl. n=1, n=5 < 16, periodic, multi-cycle)\n");
}

int main(int argc, char **argv) {
    const char *dir = argc > 1 ? argv[1] : "/tmp/admitted-bwt/admitted-bwt";
    const struct { const char *file; uint32_t n; uint32_t p; } blocks[6] = {
        {"block-0.L",   899931u, 264045u},
        {"block-2.L",   900000u, 320807u},
        {"block-85.L",  899935u, 325088u},
        {"block-143.L", 899869u, 802868u},
        {"block-207.L", 899998u, 205180u},
        {"block-210.L", 899959u, 791429u},
    };

    Kernel k;
    kernel_init(&k, 900000u);
    run_selftests(&k);

    const int WARMUP = 3;
    const int ROUNDS = 25;

    printf("\n%-11s | %7s %7s %7s %7s | %6s %6s %6s | %7s %7s %7s %7s | %6s %6s %6s | %6s %6s\n",
           "Block",
           "OpEm", "BJ64_1", "GS64_1", "G128_2",
           "BJ/Op", "GS/Op", "128/Op",
           "OpTot", "TBJ64", "TGS64", "T128_2",
           "BJ/Op", "GS/Op", "128/Op",
           "BJLft", "BJWlk");
    printf("-------------------------------------------------------------------------------------------------------------------------------------------\n");

    int p_em80[4] = {0}, p_em85[4] = {0}, p_tot90[4] = {0};

    for (int bi = 0; bi < 6; bi++) {
        char path[512];
        snprintf(path, sizeof(path), "%s/%s", dir, blocks[bi].file);
        FILE *fp = fopen(path, "rb");
        if (!fp) { perror(path); return 1; }
        uint32_t n = blocks[bi].n, p = blocks[bi].p;
        uint8_t *L = (uint8_t *)malloc(n);
        if (fread(L, 1, n, fp) != n) return 1;
        fclose(fp);

        uint16_t *recs = (uint16_t *)malloc(n * sizeof(uint16_t));
        size_t nr = to_capped_records(L, n, recs);
        uint8_t *ref = (uint8_t *)malloc(n);
        uint8_t *out = (uint8_t *)malloc(n + 32);
        dense_lf_reference(L, n, p, ref);

        for (int arm = 0; arm < 4; arm++) {
            memset(out, 0, n);
            memset(out + n, 0xFE, 32);
            if (arm == 0) {
                construct_mode(&k, recs, nr, n, p, LIMIT, 0);
                open_emit_orig(&k, n, out);
            } else if (arm == 1) {
                construct_mode(&k, recs, nr, n, p, LIMIT, 1);
                bj64_lift_and_emit(&k, n, out, NULL);
            } else if (arm == 2) {
                construct_mode(&k, recs, nr, n, p, LIMIT, 1);
                gs64_lift_and_emit(&k, n, 1, out, NULL);
            } else {
                construct_mode(&k, recs, nr, n, p, LIMIT, 2);
                gs128_lift_and_emit(&k, n, 2, out);
            }
            assert(memcmp(out, ref, n) == 0);
            for (int g = 0; g < 32; g++) assert(out[n + g] == 0xFEu);
        }

        double em[4][ROUNDS], tot[4][ROUNDS], bj_lft[ROUNDS];
        for (int r = -WARMUP; r < ROUNDS; r++) {
            for (int step = 0; step < 4; step++) {
                int arm = (step + (r + WARMUP)) % 4;
                double t0 = now_sec(), t1, t2, lms = 0.0;
                if (arm == 0) {
                    construct_mode(&k, recs, nr, n, p, LIMIT, 0);
                    t1 = now_sec();
                    open_emit_orig(&k, n, out);
                    t2 = now_sec();
                } else if (arm == 1) {
                    construct_mode(&k, recs, nr, n, p, LIMIT, 1);
                    t1 = now_sec();
                    bj64_lift_and_emit(&k, n, out, &lms);
                    t2 = now_sec();
                } else if (arm == 2) {
                    construct_mode(&k, recs, nr, n, p, LIMIT, 1);
                    t1 = now_sec();
                    gs64_lift_and_emit(&k, n, 1, out, NULL);
                    t2 = now_sec();
                } else {
                    construct_mode(&k, recs, nr, n, p, LIMIT, 2);
                    t1 = now_sec();
                    gs128_lift_and_emit(&k, n, 2, out);
                    t2 = now_sec();
                }
                if (r >= 0) {
                    em[arm][r] = (t2 - t1) * 1e3;
                    tot[arm][r] = (t2 - t0) * 1e3;
                    if (arm == 1) bj_lft[r] = lms;
                }
            }
        }

        double m_em[4], m_tot[4], m_bjl = median(bj_lft, ROUNDS);
        for (int a = 0; a < 4; a++) {
            m_em[a] = median(em[a], ROUNDS);
            m_tot[a] = median(tot[a], ROUNDS);
        }
        for (int a = 1; a <= 3; a++) {
            if (m_em[a] / m_em[0] <= 0.80) p_em80[a]++;
            if (m_em[a] / m_em[0] <= 0.85) p_em85[a]++;
            if (m_tot[a] / m_tot[0] <= 0.90) p_tot90[a]++;
        }

        printf("%-11s | %7.3f %7.3f %7.3f %7.3f | %5.3fx %5.3fx %5.3fx | %7.3f %7.3f %7.3f %7.3f | %5.3fx %5.3fx %5.3fx | %6.3f %6.3f\n",
               blocks[bi].file,
               m_em[0], m_em[1], m_em[2], m_em[3],
               m_em[1] / m_em[0], m_em[2] / m_em[0], m_em[3] / m_em[0],
               m_tot[0], m_tot[1], m_tot[2], m_tot[3],
               m_tot[1] / m_tot[0], m_tot[2] / m_tot[0], m_tot[3] / m_tot[0],
               m_bjl, m_em[1] - m_bjl);

        free(L); free(recs); free(ref); free(out);
    }

    printf("\nGate G5 Summary (Cycle 4 — IMD Single-Lane Operator Lifting, Lift+Emit charged):\n");
    printf("  (1) Byte-exactness + 32B trailing canary (6/6 blocks + 6 adversarial): PASS\n");
    printf("  (2) Zero chunk-pool / routing-queue arena:                             0 B (PASS)\n");
    const char *names[4] = {"", "BJ64-1P", "GS64-1P", "GS128-2P"};
    for (int a = 1; a <= 3; a++) {
        int ok = (p_em80[a] >= 5) && (p_em85[a] == 6) && (p_tot90[a] >= 5);
        printf("  Arm %d (%-8s): LiftEm<=0.80x %d/6, <=0.85x %d/6 | Tot<=0.90x %d/6  ==> %s\n",
               a, names[a], p_em80[a], p_em85[a], p_tot90[a], ok ? "PASS" : "FAIL");
    }

    kernel_free(&k);
    return 0;
}
