/*
 * Gate G5 Paid-Cost Falsifier & Oracle-Ceiling Harness for Carbon Cycle 3:
 * In-Place Self-Modifying Convoy-Copy Open (SMCC-Paid & SMCC-Oracle) vs Shipped Open (qi16).
 *
 * Tests OPEN-QUESTIONS.md §4 ("discovering sources for bytes already written using information
 * Open already has") and 2026-11-20-working-backwards.md §2 Q2-Q3 ("the copy class" & oracle
 * headroom before building any recogniser).
 *
 * PRE-REGISTERED PASS/FAIL RULE (frozen before compilation and execution):
 * Across all six authenticated Wikidata BWT blocks (block-0, 2, 85, 143, 207, 210),
 * 25 paired interleaved rounds after 3 warmup rounds (rotating all 3 arms each round):
 *
 *   Arm 0: Open-qi16    — exact C mirror of shipped src/open.rs (scalar queue push + 1-lane emit)
 *   Arm 1: SMCC-Paid    — zero-producer-overhead online same-cache-line lockstep convoy discovery
 *                         (min_k = 6, thick-gated, L1-resident 32 KiB copy_dict) + in-place edge
 *                         mutation edges[sib] = ((COPY_BASE + cid) << bits) | (exit + off)
 *   Arm 2: SMCC-Oracle  — uncharged offline greedy oracle pre-wiring EVERY repeated >=6-step code
 *                         substring in edges[] before emit starts (zero online discovery cost,
 *                         measuring the hardware ceiling of output-sourced trajectory skipping)
 *
 * PASS for SMCC-Paid iff ALL FOUR hold:
 *   (1) Byte-exactness: SMCC-Paid and SMCC-Oracle match dense LF byte-for-byte on 6/6 blocks
 *       (5,399,692 B) and 6/6 adversarial self-tests (incl. n=1, periodic, self-overlapping,
 *       and multi-cycle columns), with 32-byte 0xFE trailing buffer overrun canaries intact;
 *   (2) L1 Dictionary Bound: SMCC-Paid copy_dict <= 32 KiB (MAX_PAID_COPIES = 4096 * 8 B);
 *   (3) Paid Emit Gain: SMCC-Paid emit median <= 0.85x Open-qi16 emit on >= 5/6 blocks, and
 *       <= 0.90x on 6/6 blocks;
 *   (4) End-to-End Gain (G4 ~8-10% floor): SMCC-Paid total (construct + emit) median <= 0.92x
 *       Open-qi16 total on >= 5/6 blocks.
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
#define MIN_K 6u
#define MAX_PAID_COPIES 4096u
#define MAX_ORA_COPIES 8192u

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
    uint32_t src_pos;
    uint32_t byte_len;
} CopyEntry;

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
    size_t edges_cap;
    uint32_t *extra;
    size_t extra_cap;
    int wide;

    uint32_t m, mark, bits, words;

    /* 32 KiB L1-resident copy dictionary for SMCC-Paid */
    CopyEntry copy_dict[MAX_PAID_COPIES];
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
    k->spelling = (uint8_t *)malloc(k->spell_cap);
    k->off_cap = 16384;
    k->offsets = (uint32_t *)malloc(k->off_cap * sizeof(uint32_t));

    k->edges_cap = max_n + 2;
    k->edges = (uint32_t *)malloc(k->edges_cap * sizeof(uint32_t));
    k->extra_cap = max_n + 2;
    k->extra = (uint32_t *)malloc(k->extra_cap * sizeof(uint32_t));
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
    if (__builtin_expect(k->spell_len + add + 16 > k->spell_cap, 0)) {
        while (k->spell_len + add + 16 > k->spell_cap) k->spell_cap *= 2;
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

static void construct_open_lockstep(
    Kernel *k,
    const uint16_t *records,
    size_t n_recs,
    uint32_t n,
    uint32_t p,
    uint32_t limit,
    uint32_t reserve_copies
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
                spell_ensure(k, 1);
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

    /* Theorem 2.1: choose 2^bits > m strictly so +1 never carries from j_next into code! */
    uint32_t bits = m == 0 ? 1u : (32u - (uint32_t)__builtin_clz(m));
    uint32_t max_code = 255u + words + reserve_copies;
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

/* Arm 0: exact shipped Open::emit */
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

static inline void slack_around(
    const uint32_t * restrict edges,
    uint32_t m,
    uint32_t mark,
    uint32_t cur,
    uint32_t e_cur,
    uint32_t max_l,
    uint32_t max_r,
    uint32_t *out_nl,
    uint32_t *out_nr
) {
    uint32_t nl = 0;
    while (nl < max_l && cur > nl) {
        uint32_t cand = cur - nl - 1u;
        if (cand == mark || edges[cand] + (nl + 1u) != e_cur) break;
        nl++;
    }
    uint32_t nr = 0;
    while (nr < max_r && cur + nr + 1u < m) {
        uint32_t cand = cur + nr + 1u;
        if (cand == mark || edges[cand] != e_cur + (nr + 1u)) break;
        nr++;
    }
    *out_nl = nl;
    *out_nr = nr;
}

/* Arm 1: SMCC-Paid — online same-cache-line lockstep discovery + in-place edge rewriting */
static void smcc_emit_paid(Kernel *k, size_t n, uint32_t min_k, int gate_thick, uint8_t *out) {
    if (__builtin_expect(k->wide, 0)) {
        open_emit_orig(k, n, out);
        return;
    }
    uint32_t * restrict edges = k->edges;
    const uint8_t * restrict spelling = k->spelling;
    const uint32_t * restrict offsets = k->offsets;
    CopyEntry * restrict copy_dict = k->copy_dict;
    const uint32_t m = k->m, mark = k->mark, bits = k->bits, mask = (1u << bits) - 1u;
    const uint32_t copy_base = 256u + k->words;

    uint32_t n_copies = 0;
    uint8_t * restrict out_p = out;
    uint8_t * const out_end = out + n;
    uint32_t j = mark;
    int prev_thick = 1;
    uint32_t steps_guard = 0;

    while (out_p < out_end && steps_guard < m) {
        steps_guard++;
        uint32_t e = edges[j];
        uint32_t cd = e >> bits;
        uint32_t jn = e & mask;

        if (cd >= copy_base) {
            CopyEntry ce = copy_dict[cd - copy_base];
            uint32_t tk = ce.byte_len;
            size_t rem = (size_t)(out_end - out_p);
            if (__builtin_expect(tk > rem, 0)) tk = (uint32_t)rem;
            memcpy(out_p, out + ce.src_pos, tk);
            out_p += tk;
            j = jn;
            prev_thick = 1;
            if (__builtin_expect(j == mark || out_p >= out_end, 0)) break;
            continue;
        }

        uint32_t cur_l = 0, cur_r = 0;
        if (!gate_thick || cd >= 256u || prev_thick) {
            slack_around(edges, m, mark, j, e, 15u, 15u, &cur_l, &cur_r);
        }

        if (cur_l + cur_r == 0u) {
            if (cd < 256u) {
                *out_p++ = (uint8_t)cd;
                prev_thick = 0;
            } else {
                uint32_t w = cd - 256u;
                uint32_t o0 = offsets[w], ln = offsets[w + 1] - o0;
                if (__builtin_expect(out_p + ln > out_end, 0)) ln = (uint32_t)(out_end - out_p);
                memcpy(out_p, spelling + o0, ln);
                out_p += ln;
                prev_thick = 1;
            }
            j = jn;
            if (__builtin_expect(j == mark || out_p >= out_end, 0)) break;
            continue;
        }

        /* Trace affine-lockstep bundle from start_j = j */
        uint32_t start_j = j;
        uint32_t src_pos = (uint32_t)(out_p - out);
        if (cd < 256u) {
            *out_p++ = (uint8_t)cd;
        } else {
            uint32_t w = cd - 256u;
            uint32_t o0 = offsets[w], ln = offsets[w + 1] - o0;
            if (__builtin_expect(out_p + ln > out_end, 0)) ln = (uint32_t)(out_end - out_p);
            memcpy(out_p, spelling + o0, ln);
            out_p += ln;
        }
        uint32_t cur = jn;
        uint32_t blen_steps = 1u;

        while (out_p < out_end && cur != mark && steps_guard < m) {
            uint32_t ec = edges[cur];
            uint32_t cdc = ec >> bits;
            if (cdc >= copy_base) break;
            uint32_t nl, nr;
            slack_around(edges, m, mark, cur, ec, cur_l, cur_r, &nl, &nr);
            if (nl + nr == 0u) break;
            cur_l = nl;
            cur_r = nr;
            steps_guard++;
            if (cdc < 256u) {
                *out_p++ = (uint8_t)cdc;
            } else {
                uint32_t w = cdc - 256u;
                uint32_t o0 = offsets[w], ln = offsets[w + 1] - o0;
                if (__builtin_expect(out_p + ln > out_end, 0)) ln = (uint32_t)(out_end - out_p);
                memcpy(out_p, spelling + o0, ln);
                out_p += ln;
            }
            cur = ec & mask;
            blen_steps++;
        }

        uint32_t byte_len = (uint32_t)(out_p - out) - src_pos;
        if (blen_steps >= min_k && out_p < out_end && n_copies < MAX_PAID_COPIES) {
            uint32_t cid = copy_base + n_copies;
            copy_dict[n_copies++] = (CopyEntry){src_pos, byte_len};
            for (int32_t off = -(int32_t)cur_l; off <= (int32_t)cur_r; off++) {
                if (off != 0) {
                    uint32_t sib = (uint32_t)((int32_t)start_j + off);
                    if ((edges[sib] >> bits) < copy_base) {
                        edges[sib] = (cid << bits) | (uint32_t)((int32_t)cur + off);
                    }
                }
            }
        }

        j = cur;
        prev_thick = 0;
        if (__builtin_expect(j == mark || out_p >= out_end, 0)) break;
    }

    size_t wr = (size_t)(out_p - out);
    repeat_period(n, out, &wr);
}

/* Offline pre-wiring for Diagnostic Arm 2 (SMCC-Oracle): uncharged in emit timing */
typedef struct {
    uint32_t *ora_edges;
    CopyEntry ora_dict[MAX_ORA_COPIES];
    uint32_t n_ora;
    uint32_t copy_base;
} OraclePlan;

static inline uint64_t hash6(const uint32_t *seq) {
    uint64_t h = 1469598103934665603ULL;
    for (int i = 0; i < 6; i++) {
        h ^= (uint64_t)seq[i];
        h *= 1099511628211ULL;
    }
    return h;
}

static void build_oracle_plan(const Kernel *k, size_t n, uint32_t min_k, OraclePlan *plan) {
    const uint32_t m = k->m, mark = k->mark, bits = k->bits, mask = (1u << bits) - 1u;
    plan->ora_edges = (uint32_t *)malloc((m + 2) * sizeof(uint32_t));
    memcpy(plan->ora_edges, k->edges, m * sizeof(uint32_t));
    plan->n_ora = 0;
    plan->copy_base = 256u + k->words;
    if (k->wide || m < min_k) return;

    uint32_t *verts = (uint32_t *)malloc(m * sizeof(uint32_t));
    uint32_t *seq_codes = (uint32_t *)malloc(m * sizeof(uint32_t));
    uint32_t *pos_at = (uint32_t *)malloc(m * sizeof(uint32_t));
    uint32_t *end_at = (uint32_t *)malloc(m * sizeof(uint32_t));

    uint32_t T = 0, cur_pos = 0, j = mark;
    while (cur_pos < n && T < m) {
        verts[T] = j;
        uint32_t e = k->edges[j];
        uint32_t cd = e >> bits;
        seq_codes[T] = cd;
        pos_at[T] = cur_pos;
        uint32_t blen = (cd < 256u) ? 1u : (k->offsets[cd - 256u + 1u] - k->offsets[cd - 256u]);
        cur_pos = (cur_pos + blen > n) ? (uint32_t)n : (cur_pos + blen);
        end_at[T] = cur_pos;
        T++;
        j = e & mask;
        if (j == mark || cur_pos >= n) break;
    }

    /* Hash table of 8-cand buckets for min_k-grams */
    const uint32_t HSIZE = 1u << 18;
    const uint32_t HMASK = HSIZE - 1u;
    int32_t (*ht_pos)[8] = malloc(HSIZE * sizeof(*ht_pos));
    uint8_t *ht_cnt = (uint8_t *)calloc(HSIZE, 1);

    uint32_t ready_ptr = 0, t1 = 0;
    while (t1 + min_k <= T) {
        while (ready_ptr + min_k <= t1) {
            uint64_t h = min_k == 6 ? hash6(seq_codes + ready_ptr) : (uint64_t)seq_codes[ready_ptr] * 1315423911ULL + seq_codes[ready_ptr + 1];
            uint32_t b = (uint32_t)h & HMASK;
            if (ht_cnt[b] < 8) ht_pos[b][ht_cnt[b]++] = (int32_t)ready_ptr;
            ready_ptr++;
        }
        uint64_t h1 = min_k == 6 ? hash6(seq_codes + t1) : (uint64_t)seq_codes[t1] * 1315423911ULL + seq_codes[t1 + 1];
        uint32_t b1 = (uint32_t)h1 & HMASK;
        int32_t best_t0 = -1;
        uint32_t best_k = 0;
        for (uint8_t ci = 0; ci < ht_cnt[b1]; ci++) {
            uint32_t t0 = (uint32_t)ht_pos[b1][ci];
            int ok = 1;
            for (uint32_t q = 0; q < min_k; q++) {
                if (seq_codes[t0 + q] != seq_codes[t1 + q]) { ok = 0; break; }
            }
            if (!ok) continue;
            uint32_t max_k = (t1 - t0) < (T - t1) ? (t1 - t0) : (T - t1);
            uint32_t kl = min_k;
            while (kl < max_k && seq_codes[t0 + kl] == seq_codes[t1 + kl]) kl++;
            if (kl > best_k) { best_k = kl; best_t0 = (int32_t)t0; }
        }
        if (best_k >= min_k && plan->n_ora < MAX_ORA_COPIES) {
            uint32_t src_pos = pos_at[best_t0];
            uint32_t byte_len = end_at[(uint32_t)best_t0 + best_k - 1u] - src_pos;
            uint32_t exit_v = (t1 + best_k < T) ? verts[t1 + best_k] : mark;
            uint32_t cid = plan->copy_base + plan->n_ora;
            plan->ora_dict[plan->n_ora++] = (CopyEntry){src_pos, byte_len};
            plan->ora_edges[verts[t1]] = (cid << bits) | exit_v;
            t1 += best_k;
        } else {
            t1++;
        }
    }

    free(ht_pos); free(ht_cnt);
    free(verts); free(seq_codes); free(pos_at); free(end_at);
}

/* Arm 2: SMCC-Oracle zero-discovery 1-lane emit over pre-wired edges */
static void smcc_emit_oracle(const Kernel *k, const OraclePlan *plan, size_t n, uint8_t *out) {
    const uint32_t * restrict edges = plan->ora_edges;
    const uint8_t * restrict spelling = k->spelling;
    const uint32_t * restrict offsets = k->offsets;
    const CopyEntry * restrict copy_dict = plan->ora_dict;
    const uint32_t m = k->m, mark = k->mark, bits = k->bits, mask = (1u << bits) - 1u;
    const uint32_t copy_base = plan->copy_base;

    uint8_t * restrict out_p = out;
    uint8_t * const out_end = out + n;
    uint32_t j = mark;

    for (uint32_t step = 0; step < m; step++) {
        uint32_t edge = edges[j];
        uint32_t code = edge >> bits;
        j = edge & mask;
        if (code < 256u) {
            *out_p++ = (uint8_t)code;
        } else if (code < copy_base) {
            uint32_t w = code - 256u;
            uint32_t o0 = offsets[w], ln = offsets[w + 1] - o0;
            if (__builtin_expect(out_p + ln > out_end, 0)) ln = (uint32_t)(out_end - out_p);
            memcpy(out_p, spelling + o0, ln);
            out_p += ln;
        } else {
            CopyEntry ce = copy_dict[code - copy_base];
            uint32_t ln = ce.byte_len;
            if (__builtin_expect(out_p + ln > out_end, 0)) ln = (uint32_t)(out_end - out_p);
            memcpy(out_p, out + ce.src_pos, ln);
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
        {"a", 500, 137},
        {"ab", 300, 41},
        {"ba", 301, 0},
        {"abcde", 200, 777},
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
        for (int gt = 0; gt <= 1; gt++) {
            memset(got, 0xCC, n);
            memset(got + n, 0xFE, 32);
            construct_open_lockstep(k, recs, nr, (uint32_t)n, cases[ci].p, LIMIT, MAX_PAID_COPIES);
            smcc_emit_paid(k, n, 2u, gt, got);
            assert(memcmp(got, ref, n) == 0);
            for (int g = 0; g < 32; g++) assert(got[n + g] == 0xFEu);
        }
        construct_open_lockstep(k, recs, nr, (uint32_t)n, cases[ci].p, LIMIT, MAX_ORA_COPIES);
        OraclePlan op;
        build_oracle_plan(k, n, 2u, &op);
        memset(got, 0xCC, n);
        memset(got + n, 0xFE, 32);
        smcc_emit_oracle(k, &op, n, got);
        assert(memcmp(got, ref, n) == 0);
        for (int g = 0; g < 32; g++) assert(got[n + g] == 0xFEu);
        free(op.ora_edges);
        free(got);
    }
    free(L); free(recs); free(ref);
    printf("[PASS] C adversarial & 32B-canary self-tests (6/6 incl. n=1, periodic, self-overlapping, multi-cycle)\n");
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

    printf("\n%-11s | %7s %7s %7s | %7s %7s | %7s %7s %7s | %7s %7s\n",
           "Block",
           "OpEm", "PaidEm", "OraEm",
           "Paid/Op", "Ora/Op",
           "OpTot", "PaidTot", "OraTot",
           "Paid/Op", "Ora/Op");
    printf("-----------------------------------------------------------------------------------------------------\n");

    int pass_em85 = 0, pass_em90 = 0, pass_tot92 = 0;

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

        /* Precompute uncharged Oracle plan once per block and verify all 3 arms byte-for-byte */
        construct_open_lockstep(&k, recs, nr, n, p, LIMIT, MAX_ORA_COPIES);
        OraclePlan op;
        build_oracle_plan(&k, n, MIN_K, &op);

        for (int arm = 0; arm < 3; arm++) {
            memset(out, 0, n);
            memset(out + n, 0xFE, 32);
            if (arm == 0) {
                construct_open_lockstep(&k, recs, nr, n, p, LIMIT, 0);
                open_emit_orig(&k, n, out);
            } else if (arm == 1) {
                construct_open_lockstep(&k, recs, nr, n, p, LIMIT, MAX_PAID_COPIES);
                smcc_emit_paid(&k, n, MIN_K, 1, out);
            } else {
                construct_open_lockstep(&k, recs, nr, n, p, LIMIT, MAX_ORA_COPIES);
                smcc_emit_oracle(&k, &op, n, out);
            }
            assert(memcmp(out, ref, n) == 0);
            for (int g = 0; g < 32; g++) assert(out[n + g] == 0xFEu);
        }

        double em[3][ROUNDS], tot[3][ROUNDS];
        for (int r = -WARMUP; r < ROUNDS; r++) {
            for (int step = 0; step < 3; step++) {
                int arm = (step + (r + WARMUP)) % 3;
                double t0 = now_sec(), t1, t2;
                if (arm == 0) {
                    construct_open_lockstep(&k, recs, nr, n, p, LIMIT, 0);
                    t1 = now_sec();
                    open_emit_orig(&k, n, out);
                    t2 = now_sec();
                } else if (arm == 1) {
                    construct_open_lockstep(&k, recs, nr, n, p, LIMIT, MAX_PAID_COPIES);
                    t1 = now_sec();
                    smcc_emit_paid(&k, n, MIN_K, 1, out);
                    t2 = now_sec();
                } else {
                    construct_open_lockstep(&k, recs, nr, n, p, LIMIT, MAX_ORA_COPIES);
                    t1 = now_sec();
                    smcc_emit_oracle(&k, &op, n, out);
                    t2 = now_sec();
                }
                if (r >= 0) {
                    em[arm][r] = (t2 - t1) * 1e3;
                    tot[arm][r] = (t2 - t0) * 1e3;
                }
            }
        }

        double m_em[3], m_tot[3];
        for (int a = 0; a < 3; a++) {
            m_em[a] = median(em[a], ROUNDS);
            m_tot[a] = median(tot[a], ROUNDS);
        }

        double r_paid_em  = m_em[1] / m_em[0];
        double r_ora_em   = m_em[2] / m_em[0];
        double r_paid_tot = m_tot[1] / m_tot[0];
        double r_ora_tot  = m_tot[2] / m_tot[0];

        if (r_paid_em <= 0.85) pass_em85++;
        if (r_paid_em <= 0.90) pass_em90++;
        if (r_paid_tot <= 0.92) pass_tot92++;

        printf("%-11s | %7.3f %7.3f %7.3f | %6.3fx %6.3fx | %7.3f %7.3f %7.3f | %6.3fx %6.3fx\n",
               blocks[bi].file,
               m_em[0], m_em[1], m_em[2],
               r_paid_em, r_ora_em,
               m_tot[0], m_tot[1], m_tot[2],
               r_paid_tot, r_ora_tot);

        free(op.ora_edges);
        free(L); free(recs); free(ref); free(out);
    }

    size_t dict_bytes = sizeof(k.copy_dict);
    int mem_ok = dict_bytes <= 32768u;
    int all_pass = (pass_em85 >= 5) && (pass_em90 == 6) && (pass_tot92 >= 5) && mem_ok;

    printf("\nGate G5 Summary (Cycle 3 — SMCC-Paid & SMCC-Oracle):\n");
    printf("  (1) Byte-exactness + 32B trailing canary (6/6 blocks + 6 adversarial): PASS\n");
    printf("  (2) L1 copy_dict footprint <= 32 KiB:                                  %zu B (%s)\n",
           dict_bytes, mem_ok ? "PASS" : "FAIL");
    printf("  (3) SMCC-Paid emit <= 0.85x (>=5/6) and <= 0.90x (6/6):                %d/6, %d/6\n",
           pass_em85, pass_em90);
    printf("  (4) SMCC-Paid total <= 0.92x Open-qi16 (>=5/6):                        %d/6\n",
           pass_tot92);
    printf("  OVERALL PRE-REGISTERED G5 VERDICT (SMCC-Paid):                         %s\n",
           all_pass ? "PASS" : "FAIL");

    kernel_free(&k);
    return 0;
}
