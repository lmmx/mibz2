/*
 * Gate G5 Paid-Cost Falsifier & Diagnostic Harness for Carbon Cycle 1:
 * Sentinel-Spliced Monoid Contraction (SSMC-Open) vs Shipped Open (qi16).
 *
 * PRE-REGISTERED PASS/FAIL RULE (frozen before compilation and execution):
 * Primary candidate: SSMC-Open(K=8, S=64) against exact Shipped Open<16, INDUCE=true, QUEUE=true>
 * across all six authenticated Wikidata BWT blocks (block-0, 2, 85, 143, 207, 210), 25 paired
 * interleaved rounds after 3 warmup rounds (rotating arm order every round).
 *
 * PASS iff ALL FIVE hold:
 *   (1) Byte-exactness: 100% byte-identical output vs dense LF walk on all 6 blocks (5,399,692 B)
 *       and on adversarial multi-cycle / short-period / fixed-point self-tests;
 *   (2) Critical-path reduction: outer parallel loop steps D_{8,64} <= 0.22 * m (>= 4.5x cut) on 6/6 blocks;
 *   (3) Emission speedup: paired median emit time <= 0.65 * Open emit on >= 5/6 blocks, and < 0.75x on 6/6;
 *   (4) End-to-end whole-kernel speedup: paired median (construct + emit) <= 0.88 * Open total on >= 5/6
 *       blocks, and <= 0.90x on 6/6 blocks;
 *   (5) Scratch memory bound: chunked byte arena <= 1.25 MiB (1,310,720 B).
 *
 * NOTE (METHOD.md §5): Single-thread VM C wall times are diagnostic evidence for hidden instruction,
 * register-spill, and memcpy-stitch costs, NOT proof of native i9 multi-threaded competitiveness.
 */

#define _POSIX_C_SOURCE 200809L
#include <assert.h>
#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#define LIMIT 16u
#define CHUNK_SIZE 4096u
#define MAX_CHUNKS 300u   /* 300 * 4096 = 1,228,800 B (1.17 MiB <= 1.25 MiB) */
#define MAX_CUTS 128u

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

typedef struct {
    /* Persistent scratch matching src/open.rs Kernel */
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

    uint32_t *edges;   /* capacity m + 1 (slot m reserved for SSMC dummy self-loop) */
    size_t edges_cap;
    uint32_t *extra;
    size_t extra_cap;
    int wide;

    uint32_t m, mark, bits, words;

    /* SSMC fixed 1.17 MiB chunk pool + 4 KiB dummy sink */
    uint8_t *chunk_pool;
    uint8_t *dummy_buf;
    int32_t chunk_next[MAX_CHUNKS];
    uint32_t chunk_len[MAX_CHUNKS];
} OpenKernel;

static double now_sec(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (double)ts.tv_sec + (double)ts.tv_nsec * 1e-9;
}

static void kernel_init(OpenKernel *k, size_t max_n) {
    memset(k, 0, sizeof(*k));
    for (int a = 0; a < 256; a++) {
        k->q_cap[a] = 1024;
        k->queues[a] = (uint32_t *)malloc(k->q_cap[a] * sizeof(uint32_t));
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
    k->spelling = (uint8_t *)malloc(k->spell_cap);
    k->off_cap = 16384;
    k->offsets = (uint32_t *)malloc(k->off_cap * sizeof(uint32_t));

    k->edges_cap = max_n + 2;
    k->edges = (uint32_t *)malloc(k->edges_cap * sizeof(uint32_t));
    k->extra_cap = max_n + 2;
    k->extra = (uint32_t *)malloc(k->extra_cap * sizeof(uint32_t));

    k->chunk_pool = (uint8_t *)malloc((size_t)MAX_CHUNKS * CHUNK_SIZE);
    k->dummy_buf = (uint8_t *)malloc(CHUNK_SIZE);
}

static void kernel_free(OpenKernel *k) {
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
    free(k->chunk_pool);
    free(k->dummy_buf);
}

static inline void q_push_range(OpenKernel *k, uint8_t a, uint32_t start, uint32_t w) {
    uint32_t len = k->q_len[a];
    if (__builtin_expect(len + w > k->q_cap[a], 0)) {
        while (len + w > k->q_cap[a]) k->q_cap[a] *= 2;
        k->queues[a] = (uint32_t *)realloc(k->queues[a], k->q_cap[a] * sizeof(uint32_t));
    }
    uint32_t *dst = k->queues[a] + len;
    for (uint32_t i = 0; i < w; i++) dst[i] = start + i;
    k->q_len[a] = len + w;
}

static inline void push_strip(OpenKernel *k, uint32_t from, uint32_t to, uint32_t width, uint8_t byte) {
    if (__builtin_expect(k->strips_len == k->strips_cap, 0)) {
        k->strips_cap *= 2;
        k->strips = (Strip *)realloc(k->strips, k->strips_cap * sizeof(Strip));
    }
    k->strips[k->strips_len++] = (Strip){from, to, width, byte};
}

static inline uint32_t ep_node(OpenKernel *k, uint32_t width, uint32_t word) {
    if (__builtin_expect(k->nodes_len == k->nodes_cap, 0)) {
        k->nodes_cap *= 2;
        k->nodes = (EpNode *)realloc(k->nodes, k->nodes_cap * sizeof(EpNode));
    }
    uint32_t id = (uint32_t)k->nodes_len++;
    k->nodes[id] = (EpNode){{0, 0}, {0, 0}, width, word};
    return id;
}

static inline uint32_t ep_len(const OpenKernel *k, uint32_t w) {
    if (w <= 256) return w != 0 ? 1u : 0u;
    return k->rules[w - 257].len;
}

static inline uint32_t ep_join(OpenKernel *k, uint32_t left, uint32_t right) {
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

static inline void ep_gaps(OpenKernel *k, int side, uint32_t start, uint32_t end) {
    if (start < end) {
        uint32_t id = ep_node(k, end - start, 0);
        push_u32(&k->order[side], &k->order_len[side], &k->order_cap[side], id);
        push_u32(&k->terminals[1 - side], &k->term_len[1 - side], &k->term_cap[1 - side], id);
    }
}

static inline void spell_ensure(OpenKernel *k, size_t add) {
    if (__builtin_expect(k->spell_len + add > k->spell_cap, 0)) {
        while (k->spell_len + add > k->spell_cap) k->spell_cap *= 2;
        k->spelling = (uint8_t *)realloc(k->spelling, k->spell_cap);
    }
}

static void ep_append(OpenKernel *k, uint32_t word) {
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

/* Exact C translation of src/open.rs Kernel::construct<true, true> */
static void open_construct(
    OpenKernel *k,
    const uint16_t *records,
    size_t n_recs,
    uint32_t n,
    uint32_t p,
    uint32_t limit,
    uint32_t reserve_sentinels
) {
    uint32_t all_c[256] = {0};
    uint32_t thin[256] = {0};
    uint32_t thick[256] = {0};
    for (int a = 0; a < 256; a++) k->q_len[a] = 0;
    k->strips_len = 0;

    uint32_t m = 0, mark = 0, pos = 0;
    size_t i = 0;
    while (i < n_recs) {
        uint16_t rec = records[i++];
        uint8_t c = (uint8_t)(rec >> 8);
        uint32_t width = (uint32_t)(rec & 255u);
        while (i < n_recs && (uint8_t)(records[i] >> 8) == c) {
            width += (uint32_t)(records[i] & 255u);
            i++;
        }
        uint32_t end = pos + width;
        if (width > limit && p >= pos && p < end) {
            if (pos < p) {
                push_strip(k, pos, all_c[c], p - pos, c);
                thick[c]++;
                all_c[c] += p - pos;
            }
            mark = m;
            q_push_range(k, c, m, 1);
            thin[c] += 1;
            m += 1;
            all_c[c] += 1;
            if (p + 1 < end) {
                uint32_t w2 = end - p - 1;
                push_strip(k, p + 1, all_c[c], w2, c);
                thick[c]++;
                all_c[c] += w2;
            }
        } else if (width <= limit) {
            if (p >= pos && p < end) mark = m + (p - pos);
            q_push_range(k, c, m, width);
            thin[c] += width;
            m += width;
            all_c[c] += width;
        } else {
            push_strip(k, pos, all_c[c], width, c);
            thick[c]++;
            all_c[c] += width;
        }
        pos = end;
    }

    uint32_t first[256], write[256];
    uint32_t tot = 0, tw = 0;
    for (int a = 0; a < 256; a++) {
        first[a] = tot;
        tot += all_c[a];
        write[a] = tw;
        tw += thick[a];
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

    /* Endpoint induction */
    k->nodes_len = 0;
    ep_node(k, 0, 0);
    k->rules_len = 0;
    k->order_len[0] = k->order_len[1] = 0;
    k->term_len[0] = k->term_len[1] = 0;

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
            nodes[prev].next[side] = id;
            nodes[id].prev[side] = prev;
            prev = id;
        }
        for (size_t oi = 0; oi < k->order_len[side]; oi++) {
            uint32_t id = k->order[side][oi];
            nodes[prev].next[side] = id;
            nodes[id].prev[side] = prev;
            prev = id;
        }
        nodes[prev].next[side] = 0;
        nodes[0].prev[side] = prev;
    }

    uint32_t remaining = n;
    while (remaining > 0) {
        uint32_t xid = nodes[0].prev[0];
        uint32_t yid = nodes[0].prev[1];
        EpNode a = nodes[xid];
        EpNode b = nodes[yid];
        uint32_t w = a.width < b.width ? a.width : b.width;
        remaining -= w;
        if (xid == yid) {
            ep_cut(nodes, xid, 0);
            ep_cut(nodes, xid, 1);
            continue;
        }
        uint32_t wd = ep_join(k, a.word, b.word);
        nodes = k->nodes;
        if (a.width <= b.width) {
            ep_cut(nodes, xid, 0);
            ep_insert(nodes, yid, xid, 0);
            nodes[xid].word = wd;
            if (a.width == b.width) {
                ep_cut(nodes, yid, 0);
                ep_cut(nodes, yid, 1);
            } else {
                nodes[yid].width -= w;
            }
        } else {
            ep_cut(nodes, yid, 1);
            ep_insert(nodes, xid, yid, 1);
            nodes[yid].word = wd;
            nodes[xid].width -= w;
        }
    }

    if (k->nodes_len > k->bottom_cap) {
        k->bottom_cap = k->nodes_len * 2;
        k->bottom = (uint32_t *)realloc(k->bottom, k->bottom_cap * sizeof(uint32_t));
    }
    uint32_t id = nodes[0].next[1];
    x = 0;
    while (id != 0) {
        k->bottom[id] = x;
        x += nodes[id].width;
        id = nodes[id].next[1];
    }

    k->bands_len = 0;
    k->spell_len = 0;
    k->off_len = 0;
    k->offsets[k->off_len++] = 0;

    uint32_t cursors[256];
    id = nodes[0].next[0];
    uint32_t in_node = 0, kc = 0, words = 0;
    for (int a = 0; a < 256; a++) {
        cursors[a] = (uint32_t)k->bands_len;
        uint32_t end = kc + thin[a];
        while (kc < end) {
            EpNode nd = nodes[id];
            uint32_t rem_a = end - kc;
            uint32_t rem_n = nd.width - in_node;
            uint32_t width = rem_a < rem_n ? rem_a : rem_n;
            uint32_t dst = k->bottom[id] + in_node;
            uint32_t code;
            if (nd.word == 0) {
                code = (uint32_t)a;
            } else {
                words++;
                ep_append(k, nd.word);
                spell_ensure(k, 1);
                k->spelling[k->spell_len++] = (uint8_t)a;
                push_u32(&k->offsets, &k->off_len, &k->off_cap, (uint32_t)k->spell_len);
                code = 254u + (uint32_t)k->off_len;
            }
            if (__builtin_expect(k->bands_len == k->bands_cap, 0)) {
                k->bands_cap *= 2;
                k->bands = (Band *)realloc(k->bands, k->bands_cap * sizeof(Band));
            }
            k->bands[k->bands_len++] = (Band){kc + width, dst - kc, code};
            in_node += width;
            kc += width;
            if (in_node == nd.width) {
                id = nd.next[0];
                in_node = 0;
            }
        }
    }

    uint32_t bits = m <= 1 ? 1u : (32u - (uint32_t)__builtin_clz(m - 1));
    uint32_t max_code = 255u + words + reserve_sentinels;
    int wide = max_code > (UINT32_MAX >> bits);

    kc = 0;
    for (int a = 0; a < 256; a++) {
        uint32_t b_end_idx = (a + 1 < 256) ? cursors[a + 1] : (uint32_t)k->bands_len;
        uint32_t from = 0;
        const uint32_t *qa = k->queues[a];
        for (uint32_t bi = cursors[a]; bi < b_end_idx; bi++) {
            Band b = k->bands[bi];
            uint32_t span = b.end - kc;
            uint32_t dst = kc + b.delta;
            uint32_t pack = wide ? 0u : (b.code << bits);
            if (wide) {
                for (uint32_t s = 0; s < span; s++) k->extra[dst + s] = b.code;
            }
            uint32_t *edst = k->edges + dst;
            const uint32_t *qsrc = qa + from;
            for (uint32_t s = 0; s < span; s++) {
                edst[s] = pack | qsrc[s];
            }
            from += span;
            kc = b.end;
        }
    }

    k->m = m;
    k->mark = mark;
    k->bits = bits;
    k->words = words;
    k->wide = wide;
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

/* Exact C translation of src/open.rs emit() */
static void open_emit(const OpenKernel *k, size_t n, uint8_t *out) {
    const uint32_t *edges = k->edges;
    const uint32_t *extra = k->extra;
    const uint8_t *spelling = k->spelling;
    const uint32_t *offsets = k->offsets;
    const uint32_t m = k->m;
    const uint32_t mark = k->mark;
    const uint32_t bits = k->bits;
    const uint32_t mask = (1u << bits) - 1u;
    const int wide = k->wide;

    uint8_t *out_p = out;
    uint8_t *const out_end = out + n;
    uint32_t j = mark;

    for (uint32_t step = 0; step < m; step++) {
        uint32_t edge = edges[j];
        uint32_t code = wide ? extra[j] : (edge >> bits);
        j = edge & mask;
        if (code < 256u) {
            *out_p++ = (uint8_t)code;
        } else {
            uint32_t w = code - 256u;
            uint32_t o0 = offsets[w], o1 = offsets[w + 1];
            uint32_t ln = o1 - o0;
            if (__builtin_expect(out_p + ln > out_end, 0)) ln = (uint32_t)(out_end - out_p);
            memcpy(out_p, spelling + o0, ln);
            out_p += ln;
        }
        if (__builtin_expect(j == mark || out_p >= out_end, 0)) break;
    }
    size_t written = (size_t)(out_p - out);
    repeat_period(n, out, &written);
}

/* Cold helper when a lane fills a 4 KiB chunk or spans multiple chunks */
typedef struct {
    uint8_t *ptr;
    uint8_t *end;
    int32_t cur_chunk;
    uint32_t seg_id;
} LaneBuf;

static int32_t alloc_chunk(OpenKernel *k, uint32_t *free_idx) {
    uint32_t cid = (*free_idx)++;
    assert(cid < MAX_CHUNKS);
    k->chunk_next[cid] = -1;
    k->chunk_len[cid] = 0;
    return (int32_t)cid;
}

__attribute__((noinline))
static void lane_roll_byte(OpenKernel *k, LaneBuf *lb, uint32_t *free_idx, uint8_t b) {
    if (lb->cur_chunk == -2) {
        /* Dummy parked lane: wrap back to start of dummy_buf */
        lb->ptr = k->dummy_buf;
        lb->end = k->dummy_buf + CHUNK_SIZE;
        *lb->ptr++ = b;
        return;
    }
    assert(lb->cur_chunk >= 0);
    k->chunk_len[lb->cur_chunk] = CHUNK_SIZE;
    int32_t ncid = alloc_chunk(k, free_idx);
    k->chunk_next[lb->cur_chunk] = ncid;
    lb->cur_chunk = ncid;
    lb->ptr = k->chunk_pool + (size_t)ncid * CHUNK_SIZE;
    lb->end = lb->ptr + CHUNK_SIZE;
    *lb->ptr++ = b;
}

__attribute__((noinline))
static void lane_emit_spanning(
    OpenKernel *k,
    LaneBuf *lb,
    uint32_t *free_idx,
    const uint8_t *src,
    uint32_t len
) {
    while (len > 0) {
        uint32_t avail = (uint32_t)(lb->end - lb->ptr);
        if (avail == 0) {
            assert(lb->cur_chunk >= 0);
            k->chunk_len[lb->cur_chunk] = CHUNK_SIZE;
            int32_t ncid = alloc_chunk(k, free_idx);
            k->chunk_next[lb->cur_chunk] = ncid;
            lb->cur_chunk = ncid;
            lb->ptr = k->chunk_pool + (size_t)ncid * CHUNK_SIZE;
            lb->end = lb->ptr + CHUNK_SIZE;
            avail = CHUNK_SIZE;
        }
        uint32_t take = len < avail ? len : avail;
        memcpy(lb->ptr, src, take);
        lb->ptr += take;
        src += take;
        len -= take;
    }
}

static inline void emit_code_to_lane(
    OpenKernel *k,
    LaneBuf *lb,
    uint32_t *free_idx,
    uint32_t code
) {
    if (code < 256u) {
        if (__builtin_expect(lb->ptr == lb->end, 0)) {
            lane_roll_byte(k, lb, free_idx, (uint8_t)code);
        } else {
            *lb->ptr++ = (uint8_t)code;
        }
    } else {
        uint32_t w = code - 256u;
        uint32_t o0 = k->offsets[w], o1 = k->offsets[w + 1];
        uint32_t ln = o1 - o0;
        const uint8_t *src = k->spelling + o0;
        if (__builtin_expect((size_t)(lb->end - lb->ptr) >= ln, 1)) {
            memcpy(lb->ptr, src, ln);
            lb->ptr += ln;
        } else {
            lane_emit_spanning(k, lb, free_idx, src, ln);
        }
    }
}

/*
 * SSMC-Open emit: K concurrent register lanes over S sentinel-cut segments.
 * Supports K = 4 or K = 8 (templated via macro below for zero loop-bound overhead).
 */
#define DEFINE_SSMC_EMIT(FUNC_NAME, K_LANES) \
static uint32_t FUNC_NAME(OpenKernel *k, size_t n, uint32_t S_req, uint8_t *out) { \
    uint32_t *edges = k->edges; \
    uint32_t *extra = k->extra; \
    const uint32_t m = k->m; \
    const uint32_t mark = k->mark; \
    const uint32_t bits = k->bits; \
    const uint32_t mask = (1u << bits) - 1u; \
    const int wide = k->wide; \
    const uint32_t sentinel_base = 256u + k->words; \
\
    uint32_t S_eff = S_req < m ? S_req : m; \
    if (S_eff > MAX_CUTS) S_eff = MAX_CUTS; \
\
    uint32_t cuts[MAX_CUTS]; \
    uint32_t saved_next[MAX_CUTS]; \
    uint32_t saved_code[MAX_CUTS]; \
    uint32_t saved_raw_edge[MAX_CUTS]; \
    int32_t seg_head[MAX_CUTS]; \
    uint32_t seg0_len = 0; \
    int32_t succ_cut[MAX_CUTS]; \
\
    for (uint32_t q = 0; q < S_eff; q++) { \
        uint32_t v = (uint32_t)((mark + ((uint64_t)q * m) / S_eff) % m); \
        cuts[q] = v; \
        uint32_t raw = edges[v]; \
        saved_raw_edge[q] = raw; \
        saved_next[q] = raw & mask; \
        saved_code[q] = wide ? extra[v] : (raw >> bits); \
        succ_cut[q] = -1; \
        seg_head[q] = -1; \
    } \
    for (uint32_t q = 0; q < S_eff; q++) { \
        uint32_t v = cuts[q]; \
        uint32_t scode = sentinel_base + q; \
        if (wide) { \
            extra[v] = scode; \
        } else { \
            edges[v] = (scode << bits) | saved_next[q]; \
        } \
    } \
    /* Dummy fixed-point at slot m: next=m, code=0 */ \
    edges[m] = m; \
    if (wide) extra[m] = 0; \
\
    uint32_t free_chunk = 0; \
    uint32_t next_q = 0; \
    uint32_t active = 0; \
    uint32_t j[K_LANES]; \
    LaneBuf lb[K_LANES]; \
\
    for (int l = 0; l < K_LANES; l++) { \
        if (next_q < S_eff) { \
            uint32_t q = next_q++; \
            active++; \
            lb[l].seg_id = q; \
            if (q == 0) { \
                /* Segment 0 writes directly into out[0..n) with zero chunk/memcpy cost */ \
                lb[l].cur_chunk = -1; \
                lb[l].ptr = out; \
                lb[l].end = out + n; \
            } else { \
                int32_t cid = alloc_chunk(k, &free_chunk); \
                seg_head[q] = cid; \
                lb[l].cur_chunk = cid; \
                lb[l].ptr = k->chunk_pool + (size_t)cid * CHUNK_SIZE; \
                lb[l].end = lb[l].ptr + CHUNK_SIZE; \
            } \
            emit_code_to_lane(k, &lb[l], &free_chunk, saved_code[q]); \
            j[l] = saved_next[q]; \
        } else { \
            lb[l].seg_id = UINT32_MAX; \
            lb[l].cur_chunk = -2; \
            lb[l].ptr = k->dummy_buf; \
            lb[l].end = k->dummy_buf + CHUNK_SIZE; \
            j[l] = m; \
        } \
    } \
\
    uint32_t outer_iters = 0; \
    if (!wide) { \
        const uint8_t *spelling = k->spelling; \
        const uint32_t *offsets = k->offsets; \
        while (active > 0) { \
            outer_iters++; \
            uint32_t e[K_LANES]; \
            for (int l = 0; l < K_LANES; l++) e[l] = edges[j[l]]; \
            for (int l = 0; l < K_LANES; l++) { \
                uint32_t code = e[l] >> bits; \
                j[l] = e[l] & mask; \
                if (__builtin_expect(code < 256u, 1)) { \
                    if (__builtin_expect(lb[l].ptr == lb[l].end, 0)) { \
                        lane_roll_byte(k, &lb[l], &free_chunk, (uint8_t)code); \
                    } else { \
                        *lb[l].ptr++ = (uint8_t)code; \
                    } \
                } else if (__builtin_expect(code < sentinel_base, 1)) { \
                    uint32_t w = code - 256u; \
                    uint32_t o0 = offsets[w], o1 = offsets[w + 1]; \
                    uint32_t ln = o1 - o0; \
                    const uint8_t *src = spelling + o0; \
                    if (__builtin_expect((size_t)(lb[l].end - lb[l].ptr) >= ln, 1)) { \
                        memcpy(lb[l].ptr, src, ln); \
                        lb[l].ptr += ln; \
                    } else { \
                        lane_emit_spanning(k, &lb[l], &free_chunk, src, ln); \
                    } \
                } else { \
                    /* Sentinel reached: close current segment and start next_q or park */ \
                    uint32_t hit_q = code - sentinel_base; \
                    uint32_t cur_q = lb[l].seg_id; \
                    succ_cut[cur_q] = (int32_t)hit_q; \
                    if (cur_q == 0) { \
                        seg0_len = (uint32_t)(lb[l].ptr - out); \
                    } else { \
                        int32_t cid = lb[l].cur_chunk; \
                        uint8_t *cbase = k->chunk_pool + (size_t)cid * CHUNK_SIZE; \
                        k->chunk_len[cid] = (uint32_t)(lb[l].ptr - cbase); \
                    } \
                    if (next_q < S_eff) { \
                        uint32_t nq = next_q++; \
                        lb[l].seg_id = nq; \
                        int32_t cid = alloc_chunk(k, &free_chunk); \
                        seg_head[nq] = cid; \
                        lb[l].cur_chunk = cid; \
                        lb[l].ptr = k->chunk_pool + (size_t)cid * CHUNK_SIZE; \
                        lb[l].end = lb[l].ptr + CHUNK_SIZE; \
                        emit_code_to_lane(k, &lb[l], &free_chunk, saved_code[nq]); \
                        j[l] = saved_next[nq]; \
                    } else { \
                        lb[l].seg_id = UINT32_MAX; \
                        lb[l].cur_chunk = -2; \
                        lb[l].ptr = k->dummy_buf; \
                        lb[l].end = k->dummy_buf + CHUNK_SIZE; \
                        j[l] = m; \
                        active--; \
                    } \
                } \
            } \
        } \
    } else { \
        while (active > 0) { \
            outer_iters++; \
            uint32_t e[K_LANES], c[K_LANES]; \
            for (int l = 0; l < K_LANES; l++) { \
                uint32_t jl = j[l]; \
                e[l] = edges[jl]; \
                c[l] = extra[jl]; \
            } \
            for (int l = 0; l < K_LANES; l++) { \
                uint32_t code = c[l]; \
                j[l] = e[l] & mask; \
                if (code < sentinel_base) { \
                    emit_code_to_lane(k, &lb[l], &free_chunk, code); \
                } else { \
                    uint32_t hit_q = code - sentinel_base; \
                    uint32_t cur_q = lb[l].seg_id; \
                    succ_cut[cur_q] = (int32_t)hit_q; \
                    if (cur_q == 0) { \
                        seg0_len = (uint32_t)(lb[l].ptr - out); \
                    } else { \
                        int32_t cid = lb[l].cur_chunk; \
                        uint8_t *cbase = k->chunk_pool + (size_t)cid * CHUNK_SIZE; \
                        k->chunk_len[cid] = (uint32_t)(lb[l].ptr - cbase); \
                    } \
                    if (next_q < S_eff) { \
                        uint32_t nq = next_q++; \
                        lb[l].seg_id = nq; \
                        int32_t cid = alloc_chunk(k, &free_chunk); \
                        seg_head[nq] = cid; \
                        lb[l].cur_chunk = cid; \
                        lb[l].ptr = k->chunk_pool + (size_t)cid * CHUNK_SIZE; \
                        lb[l].end = lb[l].ptr + CHUNK_SIZE; \
                        emit_code_to_lane(k, &lb[l], &free_chunk, saved_code[nq]); \
                        j[l] = saved_next[nq]; \
                    } else { \
                        lb[l].seg_id = UINT32_MAX; \
                        lb[l].cur_chunk = -2; \
                        lb[l].ptr = k->dummy_buf; \
                        lb[l].end = k->dummy_buf + CHUNK_SIZE; \
                        j[l] = m; \
                        active--; \
                    } \
                } \
            } \
        } \
    } \
\
    /* Restore edges[v_q] so repeated emit benchmarks on the same built table are clean */ \
    for (uint32_t q = 0; q < S_eff; q++) { \
        uint32_t v = cuts[q]; \
        edges[v] = saved_raw_edge[q]; \
        if (wide) extra[v] = saved_code[q]; \
    } \
\
    /* Stitch segments along the marked cycle starting after segment 0 (already in out[0..seg0_len)) */ \
    size_t written = seg0_len; \
    int32_t cur_seg = succ_cut[0]; \
    uint32_t guard = 0; \
    while (cur_seg > 0 && written < n && guard < S_eff) { \
        int32_t cid = seg_head[cur_seg]; \
        while (cid != -1 && written < n) { \
            uint32_t clen = k->chunk_len[cid]; \
            if (written + clen > n) clen = (uint32_t)(n - written); \
            memcpy(out + written, k->chunk_pool + (size_t)cid * CHUNK_SIZE, clen); \
            written += clen; \
            cid = k->chunk_next[cid]; \
        } \
        cur_seg = succ_cut[cur_seg]; \
        guard++; \
    } \
    repeat_period(n, out, &written); \
    return outer_iters; \
}

DEFINE_SSMC_EMIT(ssmc_emit_k4, 4)
DEFINE_SSMC_EMIT(ssmc_emit_k8, 8)

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
    for (int a = 0; a < 256; a++) {
        C[a] = s;
        s += counts[a];
    }
    uint32_t *lf = (uint32_t *)malloc(n * sizeof(uint32_t));
    for (size_t i = 0; i < n; i++) {
        uint8_t b = L[i];
        lf[i] = C[b] + occ[b]++;
    }
    uint8_t *rev = (uint8_t *)malloc(n);
    size_t clen = 0;
    uint32_t cur = p;
    for (size_t step = 0; step < n; step++) {
        rev[clen++] = L[cur];
        cur = lf[cur];
        if (cur == p) break;
    }
    for (size_t i = 0; i < clen; i++) ref_out[i] = rev[clen - 1 - i];
    repeat_period(n, ref_out, &clen);
    free(lf);
    free(rev);
}

static int cmp_double(const void *a, const void *b) {
    double da = *(const double *)a, db = *(const double *)b;
    return (da > db) - (da < db);
}

static double median(double *arr, int count) {
    qsort(arr, (size_t)count, sizeof(double), cmp_double);
    return arr[count / 2];
}

static void run_adversarial_selftests(OpenKernel *k) {
    const struct {
        const char *pat;
        size_t reps;
        uint32_t p;
    } cases[] = {
        {"a", 500, 137},
        {"ab", 300, 41},
        {"ba", 301, 0},
        {"abcde", 200, 777},
    };
    uint8_t *L = (uint8_t *)malloc(2048);
    uint16_t *recs = (uint16_t *)malloc(2048 * sizeof(uint16_t));
    uint8_t *ref = (uint8_t *)malloc(2048);
    uint8_t *got = (uint8_t *)malloc(2048);

    for (size_t ci = 0; ci < sizeof(cases) / sizeof(cases[0]); ci++) {
        size_t plen = strlen(cases[ci].pat);
        size_t n = plen * cases[ci].reps;
        for (size_t r = 0; r < cases[ci].reps; r++) {
            memcpy(L + r * plen, cases[ci].pat, plen);
        }
        dense_lf_reference(L, n, cases[ci].p, ref);
        size_t n_recs = to_capped_records(L, n, recs);
        open_construct(k, recs, n_recs, (uint32_t)n, cases[ci].p, LIMIT, 128);
        memset(got, 0xAA, n);
        ssmc_emit_k8(k, n, 64, got);
        assert(memcmp(got, ref, n) == 0);
        memset(got, 0x55, n);
        ssmc_emit_k4(k, n, 32, got);
        assert(memcmp(got, ref, n) == 0);
    }
    free(L);
    free(recs);
    free(ref);
    free(got);
    printf("[PASS] C adversarial periodic / multi-cycle / fixed-point self-tests (4/4)\n");
}

int main(int argc, char **argv) {
    const char *dir = argc > 1 ? argv[1] : "/tmp/admitted-bwt/admitted-bwt";
    const struct {
        const char *file;
        uint32_t n;
        uint32_t p;
    } blocks[6] = {
        {"block-0.L",   899931u, 264045u},
        {"block-2.L",   900000u, 320807u},
        {"block-85.L",  899935u, 325088u},
        {"block-143.L", 899869u, 802868u},
        {"block-207.L", 899998u, 205180u},
        {"block-210.L", 899959u, 791429u},
    };

    OpenKernel k;
    kernel_init(&k, 900000u);
    run_adversarial_selftests(&k);

    const int WARMUP = 3;
    const int ROUNDS = 25;

    printf("\n%-12s | %7s %7s %6s | %8s %8s %6s | %8s %8s %6s | %6s %6s\n",
           "Block", "m", "D(8,64)", "D/m",
           "OpEm(ms)", "S8Em(ms)", "EmRat",
           "OpTot", "S8Tot", "TotRat",
           "S4Tot", "S8_128");
    printf("------------------------------------------------------------------------------------------------------------\n");

    int pass_depth = 0, pass_emit_65 = 0, pass_emit_75 = 0, pass_tot_88 = 0, pass_tot_90 = 0;

    for (int bi = 0; bi < 6; bi++) {
        char path[512];
        snprintf(path, sizeof(path), "%s/%s", dir, blocks[bi].file);
        FILE *fp = fopen(path, "rb");
        if (!fp) {
            perror(path);
            return 1;
        }
        uint32_t n = blocks[bi].n;
        uint32_t p = blocks[bi].p;
        uint8_t *L = (uint8_t *)malloc(n);
        if (fread(L, 1, n, fp) != n) {
            fprintf(stderr, "short read on %s\n", path);
            return 1;
        }
        fclose(fp);

        uint16_t *recs = (uint16_t *)malloc(n * sizeof(uint16_t));
        size_t n_recs = to_capped_records(L, n, recs);

        uint8_t *ref_out = (uint8_t *)malloc(n);
        uint8_t *out_open = (uint8_t *)malloc(n);
        uint8_t *out_ssmc = (uint8_t *)malloc(n);
        dense_lf_reference(L, n, p, ref_out);

        /* Verify byte-exactness of all 4 arms before timing */
        open_construct(&k, recs, n_recs, n, p, LIMIT, 0);
        open_emit(&k, n, out_open);
        assert(memcmp(out_open, ref_out, n) == 0);

        open_construct(&k, recs, n_recs, n, p, LIMIT, 128);
        uint32_t d_8_64 = ssmc_emit_k8(&k, n, 64, out_ssmc);
        assert(memcmp(out_ssmc, ref_out, n) == 0);

        memset(out_ssmc, 0, n);
        (void)ssmc_emit_k4(&k, n, 32, out_ssmc);
        assert(memcmp(out_ssmc, ref_out, n) == 0);

        memset(out_ssmc, 0, n);
        (void)ssmc_emit_k8(&k, n, 128, out_ssmc);
        assert(memcmp(out_ssmc, ref_out, n) == 0);

        double t_op_em[ROUNDS], t_op_tot[ROUNDS];
        double t_s8_em[ROUNDS], t_s8_tot[ROUNDS];
        double t_s4_tot[ROUNDS], t_s128_tot[ROUNDS];

        for (int r = -WARMUP; r < ROUNDS; r++) {
            for (int step = 0; step < 4; step++) {
                int arm = (step + (r + WARMUP)) % 4;
                if (arm == 0) {
                    double t0 = now_sec();
                    open_construct(&k, recs, n_recs, n, p, LIMIT, 0);
                    double t1 = now_sec();
                    open_emit(&k, n, out_open);
                    double t2 = now_sec();
                    if (r >= 0) {
                        t_op_em[r] = (t2 - t1) * 1e3;
                        t_op_tot[r] = (t2 - t0) * 1e3;
                    }
                } else if (arm == 1) {
                    double t0 = now_sec();
                    open_construct(&k, recs, n_recs, n, p, LIMIT, 64);
                    double t1 = now_sec();
                    ssmc_emit_k8(&k, n, 64, out_ssmc);
                    double t2 = now_sec();
                    if (r >= 0) {
                        t_s8_em[r] = (t2 - t1) * 1e3;
                        t_s8_tot[r] = (t2 - t0) * 1e3;
                    }
                } else if (arm == 2) {
                    double t0 = now_sec();
                    open_construct(&k, recs, n_recs, n, p, LIMIT, 32);
                    ssmc_emit_k4(&k, n, 32, out_ssmc);
                    double t2 = now_sec();
                    if (r >= 0) t_s4_tot[r] = (t2 - t0) * 1e3;
                } else {
                    double t0 = now_sec();
                    open_construct(&k, recs, n_recs, n, p, LIMIT, 128);
                    ssmc_emit_k8(&k, n, 128, out_ssmc);
                    double t2 = now_sec();
                    if (r >= 0) t_s128_tot[r] = (t2 - t0) * 1e3;
                }
            }
        }

        double m_op_em = median(t_op_em, ROUNDS);
        double m_op_tot = median(t_op_tot, ROUNDS);
        double m_s8_em = median(t_s8_em, ROUNDS);
        double m_s8_tot = median(t_s8_tot, ROUNDS);
        double m_s4_tot = median(t_s4_tot, ROUNDS);
        double m_s128_tot = median(t_s128_tot, ROUNDS);

        double d_rat = (double)d_8_64 / (double)k.m;
        double em_rat = m_s8_em / m_op_em;
        double tot_rat = m_s8_tot / m_op_tot;

        if (d_rat <= 0.22) pass_depth++;
        if (em_rat <= 0.65) pass_emit_65++;
        if (em_rat < 0.75) pass_emit_75++;
        if (tot_rat <= 0.88) pass_tot_88++;
        if (tot_rat <= 0.90) pass_tot_90++;

        printf("%-12s | %7u %7u %5.3fx | %8.3f %8.3f %5.3fx | %8.3f %8.3f %5.3fx | %5.3fx %5.3fx\n",
               blocks[bi].file, k.m, d_8_64, d_rat,
               m_op_em, m_s8_em, em_rat,
               m_op_tot, m_s8_tot, tot_rat,
               m_s4_tot / m_op_tot, m_s128_tot / m_op_tot);

        free(L);
        free(recs);
        free(ref_out);
        free(out_open);
        free(out_ssmc);
    }

    size_t arena_bytes = (size_t)MAX_CHUNKS * CHUNK_SIZE + CHUNK_SIZE;
    int mem_ok = arena_bytes <= 1310720u;
    int all_pass = (pass_depth == 6) && (pass_emit_65 >= 5) && (pass_emit_75 == 6)
                   && (pass_tot_88 >= 5) && (pass_tot_90 == 6) && mem_ok;

    printf("\nGate G5 Summary (SSMC-Open K=8, S=64 vs Open-qi16):\n");
    printf("  (1) Byte-exactness (6/6 blocks + 4 adversarial): PASS\n");
    printf("  (2) Depth D <= 0.22*m (need 6/6):                %d/6\n", pass_depth);
    printf("  (3) Emit <= 0.65x (>=5/6) & <0.75x (6/6):        %d/6 (<=0.65x), %d/6 (<0.75x)\n",
           pass_emit_65, pass_emit_75);
    printf("  (4) Total <= 0.88x (>=5/6) & <=0.90x (6/6):      %d/6 (<=0.88x), %d/6 (<=0.90x)\n",
           pass_tot_88, pass_tot_90);
    printf("  (5) Arena footprint <= 1.25 MiB:                 %zu B (%s)\n",
           arena_bytes, mem_ok ? "PASS" : "FAIL");
    printf("  OVERALL PRE-REGISTERED G5 VERDICT:               %s\n",
           all_pass ? "PASS (diagnostic; Coordinator native i9 check required)" : "FAIL");

    kernel_free(&k);
    return 0;
}
