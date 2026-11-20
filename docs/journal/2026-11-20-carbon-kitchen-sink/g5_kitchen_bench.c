/*
 * Carbon Kitchen Sink — Cycle 1 Gate G5 Paid-Cost Falsifier (g5_kitchen_bench.c)
 *
 * Pre-registered head-to-head comparison from identical entry state I0
 * (capped u16 run records `byte << 8 | len`, CAP = 127, primary p, block len n):
 *
 *   1. Open-qi16          : Faithful C port of shipped src/open.rs + src/open/endpoint.rs
 *                           (Open<16, INDUCE=true, QUEUE=true>, reusable workspace).
 *   2. DECO-w2            : Decoupled Exact-Chain Open (min_w = 2), standard append_code.
 *   3. DECO-w1            : Decoupled Exact-Chain Open (min_w = 1, 16-lane lockstep chain trace),
 *                           standard append_code.
 *   4. Open-qi16-fastemit : Open-qi16 with branchless 16B unaligned short-word emit (fair baseline).
 *   5. DECO-w1-fastemit   : DECO-w1 with the exact same branchless 16B short-word emit.
 *   6. TTSR-RV            : Full Train-Track Series-Reduced Accelerated RV (m = 0 row walks).
 *
 * Verification:
 *   - Every arm's output is verified byte-for-byte against dense LF on all 6 blocks.
 *   - 64-byte canary guards after output buffers verified intact.
 *   - 21 paired, interleaved rounds (after 3 warm-up rounds), rotating arm order per round.
 */

#define _POSIX_C_SOURCE 200809L
#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#define MAX_N 900100
#define MAX_RECS 900100
#define MAX_RUNS 900100
#define MAX_NODES 300000
#define MAX_RULES 300000
#define OUT_PAD 128
#define CANARY_LEN 64
#define CANARY_BYTE 0xA5
#define LIMIT 16u

static inline uint64_t now_ns(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (uint64_t)ts.tv_sec * 1000000000ULL + (uint64_t)ts.tv_nsec;
}

/* Build production capped run records (CAP = 127) from raw .L column */
static uint32_t build_capped_records(const uint8_t *L, uint32_t n, uint16_t *recs) {
    uint32_t nr = 0;
    uint32_t i = 0;
    while (i < n) {
        uint8_t c = L[i];
        uint32_t j = i + 1;
        while (j < n && L[j] == c) j++;
        uint32_t rem = j - i;
        while (rem > 0) {
            uint32_t w = rem > 127 ? 127 : rem;
            recs[nr++] = (uint16_t)(((uint16_t)c << 8) | w);
            rem -= w;
        }
        i = j;
    }
    return nr;
}

/* Reference dense LF inversion */
static void dense_lf_invert(const uint8_t *L, uint32_t n, uint32_t p, uint8_t *out) {
    uint32_t counts[256] = {0};
    for (uint32_t i = 0; i < n; i++) counts[L[i]]++;
    uint32_t occ[256];
    uint32_t acc = 0;
    for (int c = 0; c < 256; c++) {
        occ[c] = acc;
        acc += counts[c];
    }
    uint32_t *lf = (uint32_t *)malloc(n * sizeof(uint32_t));
    for (uint32_t i = 0; i < n; i++) {
        uint8_t b = L[i];
        lf[i] = occ[b]++;
    }
    uint8_t *rev = (uint8_t *)malloc(n);
    uint32_t cur = p, plen = 0;
    for (uint32_t step = 0; step < n; step++) {
        rev[plen++] = L[cur];
        cur = lf[cur];
        if (cur == p) break;
    }
    for (uint32_t i = 0; i < plen; i++) {
        out[i] = rev[plen - 1 - i];
    }
    uint32_t written = plen;
    while (written < n) {
        uint32_t take = (n - written) < plen ? (n - written) : plen;
        memcpy(out + written, out, take);
        written += take;
    }
    free(rev);
    free(lf);
}

/* Shared RV Node & Rule structs matching src/open/endpoint.rs */
typedef struct {
    uint32_t prev[2];
    uint32_t next[2];
    uint32_t width;
    uint32_t word;
} RvNode;

typedef struct {
    uint32_t left;
    uint32_t right;
    uint32_t len;
    uint32_t first;
} RvRule;

typedef struct {
    uint32_t from;
    uint32_t to;
    uint32_t width;
    uint8_t byte;
} Strip;

typedef struct {
    uint32_t end;
    uint32_t delta;
    uint32_t code;
} Band;

typedef struct {
    uint32_t head_m;
    uint16_t width;
    int32_t sfx_id;
} DecoQueueItem;

typedef struct {
    double t_pass1_us;
    double t_chains_us;
    double t_rv_us;
    double t_wire_us;
    double t_emit_us;
    double t_total_us;
    uint32_t m_rows;
    uint32_t rv_steps;
} PhaseStats;

/* Reusable pre-allocated workspace for all kernels */
typedef struct {
    /* Open queues: 256 dynamic-free preallocated buffers or two-pass?
     * Note: in src/open.rs, self.queues is Vec<Vec<u32>> whose inner Vecs keep capacity across blocks!
     * Giving each of the 256 buckets its exact persisted capacity (up to MAX_N total or 256 Vecs)
     * matches src/open.rs's steady-state zero-realloc behaviour! */
    uint32_t *q_vecs[256];
    uint32_t q_len[256];
    uint32_t q_cap[256];

    Strip *strips;
    uint32_t *image_order;
    Band *bands;
    uint8_t *spelling;
    uint32_t *offsets;
    uint32_t *edges;
    uint32_t *extra;

    RvNode *nodes;
    RvRule *rules;
    uint32_t *order0;
    uint32_t *order1;
    uint32_t *term0;
    uint32_t *term1;
    uint32_t *bottom;
    uint32_t *stack;

    /* DECO & TTSR buffers */
    uint32_t *run_start;
    uint32_t *run_width;
    uint8_t *run_sym;
    uint8_t *run_thin;
    uint32_t *img_start;
    uint32_t *cod_order;
    int32_t *succ_exact;
    uint8_t *has_pred;
    int32_t *tail_head_m;
    int32_t *tail_sfx_id;
    uint8_t *sfx_bytes;
    uint32_t *sfx_offsets;
    uint32_t *chain_heads;
    DecoQueueItem *deco_q;

    /* TTSR extra buffers */
    uint32_t *in_deg;
    uint32_t *out_deg;
    uint32_t *arcs_u;
    uint32_t *arcs_v;
    uint32_t *arcs_w;
    uint32_t *arcs_uo;
    uint32_t *arcs_vo;
    int32_t *unique_succ;
    uint32_t *unique_voff;
    uint32_t *core_runs;
    uint32_t *core_out_head;
    uint32_t *core_out_cnt;
    uint32_t *core_in_head;
    uint32_t *core_in_cnt;
    uint64_t *core_in_pairs; /* (vo << 32) | pid */
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
    w->spelling = (uint8_t *)malloc((MAX_N * 2 + 1024));
    w->offsets = (uint32_t *)malloc(MAX_RUNS * 2 * sizeof(uint32_t));
    w->edges = (uint32_t *)malloc(MAX_N * sizeof(uint32_t));
    w->extra = (uint32_t *)malloc(MAX_N * sizeof(uint32_t));

    w->nodes = (RvNode *)malloc(MAX_NODES * sizeof(RvNode));
    w->rules = (RvRule *)malloc(MAX_RULES * sizeof(RvRule));
    w->order0 = (uint32_t *)malloc(MAX_NODES * sizeof(uint32_t));
    w->order1 = (uint32_t *)malloc(MAX_NODES * sizeof(uint32_t));
    w->term0 = (uint32_t *)malloc(MAX_NODES * sizeof(uint32_t));
    w->term1 = (uint32_t *)malloc(MAX_NODES * sizeof(uint32_t));
    w->bottom = (uint32_t *)malloc(MAX_NODES * sizeof(uint32_t));
    w->stack = (uint32_t *)malloc(MAX_RULES * 2 * sizeof(uint32_t));

    w->run_start = (uint32_t *)malloc(MAX_RUNS * sizeof(uint32_t));
    w->run_width = (uint32_t *)malloc(MAX_RUNS * sizeof(uint32_t));
    w->run_sym = (uint8_t *)malloc(MAX_RUNS);
    w->run_thin = (uint8_t *)malloc(MAX_RUNS);
    w->img_start = (uint32_t *)malloc(MAX_RUNS * sizeof(uint32_t));
    w->cod_order = (uint32_t *)malloc(MAX_RUNS * sizeof(uint32_t));
    w->succ_exact = (int32_t *)malloc(MAX_RUNS * sizeof(int32_t));
    w->has_pred = (uint8_t *)malloc(MAX_RUNS);
    w->tail_head_m = (int32_t *)malloc(MAX_RUNS * sizeof(int32_t));
    w->tail_sfx_id = (int32_t *)malloc(MAX_RUNS * sizeof(int32_t));
    w->sfx_bytes = (uint8_t *)malloc(MAX_N + 1024);
    w->sfx_offsets = (uint32_t *)malloc(MAX_RUNS * sizeof(uint32_t));
    w->chain_heads = (uint32_t *)malloc(MAX_RUNS * sizeof(uint32_t));
    w->deco_q = (DecoQueueItem *)malloc(MAX_RUNS * sizeof(DecoQueueItem));

    w->in_deg = (uint32_t *)malloc(MAX_RUNS * sizeof(uint32_t));
    w->out_deg = (uint32_t *)malloc(MAX_RUNS * sizeof(uint32_t));
    w->arcs_u = (uint32_t *)malloc(MAX_RUNS * 3 * sizeof(uint32_t));
    w->arcs_v = (uint32_t *)malloc(MAX_RUNS * 3 * sizeof(uint32_t));
    w->arcs_w = (uint32_t *)malloc(MAX_RUNS * 3 * sizeof(uint32_t));
    w->arcs_uo = (uint32_t *)malloc(MAX_RUNS * 3 * sizeof(uint32_t));
    w->arcs_vo = (uint32_t *)malloc(MAX_RUNS * 3 * sizeof(uint32_t));
    w->unique_succ = (int32_t *)malloc(MAX_RUNS * sizeof(int32_t));
    w->unique_voff = (uint32_t *)malloc(MAX_RUNS * sizeof(uint32_t));
    w->core_runs = (uint32_t *)malloc(MAX_RUNS * sizeof(uint32_t));
    w->core_out_head = (uint32_t *)malloc((MAX_RUNS + 1) * sizeof(uint32_t));
    w->core_out_cnt = (uint32_t *)malloc(MAX_RUNS * sizeof(uint32_t));
    w->core_in_head = (uint32_t *)malloc((MAX_RUNS + 1) * sizeof(uint32_t));
    w->core_in_cnt = (uint32_t *)malloc(MAX_RUNS * sizeof(uint32_t));
    w->core_in_pairs = (uint64_t *)malloc(MAX_RUNS * 2 * sizeof(uint64_t));
    return w;
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
    uint32_t pr = nodes[id].prev[side];
    uint32_t nx = nodes[id].next[side];
    nodes[pr].next[side] = nx;
    nodes[nx].prev[side] = pr;
}

static inline void rv_insert(RvNode *nodes, uint32_t at, uint32_t id, int side) {
    uint32_t nx = nodes[at].next[side];
    nodes[id].prev[side] = at;
    nodes[id].next[side] = nx;
    nodes[at].next[side] = id;
    nodes[nx].prev[side] = id;
}

static inline uint32_t rv_tok_len(const RvRule *rules, uint32_t w) {
    if (w <= 256) return w != 0 ? 1u : 0u;
    return rules[w - 257].len;
}

static inline uint32_t rv_join(RvRule *rules, uint32_t *n_rules, uint32_t left, uint32_t right) {
    if (left == 0) return right;
    if (right == 0) return left;
    uint32_t idx = (*n_rules)++;
    rules[idx].left = left;
    rules[idx].right = right;
    rules[idx].len = rv_tok_len(rules, left) + rv_tok_len(rules, right);
    rules[idx].first = UINT32_MAX;
    return idx + 257u;
}

static uint32_t rv_eliminate(RvNode *nodes, RvRule *rules, uint32_t *n_rules, uint32_t rem) {
    uint32_t steps = 0;
    while (rem > 0) {
        uint32_t x = nodes[0].prev[0];
        uint32_t y = nodes[0].prev[1];
        uint32_t wx = nodes[x].width;
        uint32_t wy = nodes[y].width;
        uint32_t w = wx < wy ? wx : wy;
        rem -= w;
        steps++;
        if (x == y) {
            rv_cut(nodes, x, 0);
            rv_cut(nodes, x, 1);
            continue;
        }
        uint32_t word = rv_join(rules, n_rules, nodes[x].word, nodes[y].word);
        if (wx <= wy) {
            rv_cut(nodes, x, 0);
            rv_insert(nodes, y, x, 0);
            nodes[x].word = word;
            if (wx == wy) {
                rv_cut(nodes, y, 0);
                rv_cut(nodes, y, 1);
            } else {
                nodes[y].width = wy - w;
            }
        } else {
            rv_cut(nodes, y, 1);
            rv_insert(nodes, x, y, 1);
            nodes[y].word = word;
            nodes[x].width = wx - w;
        }
    }
    return steps;
}

static uint32_t rv_append_word(
    uint32_t word,
    RvRule *rules,
    uint32_t *stack,
    uint8_t *spelling,
    uint32_t sp_len
) {
    uint32_t sp = 0;
    stack[sp++] = word;
    while (sp > 0) {
        uint32_t id = stack[--sp];
        if (id <= 256) {
            spelling[sp_len++] = (uint8_t)(id - 1);
        } else {
            RvRule *r = &rules[id - 257];
            if (r->first == UINT32_MAX) {
                r->first = sp_len;
                stack[sp++] = r->right;
                stack[sp++] = r->left;
            } else {
                memmove(spelling + sp_len, spelling + r->first, r->len);
                sp_len += r->len;
            }
        }
    }
    return sp_len;
}

static void emit_standard(
    const uint32_t *edges,
    const uint32_t *extra,
    int wide,
    const uint8_t *spelling,
    const uint32_t *offsets,
    uint32_t m,
    uint32_t n,
    uint32_t mark,
    uint32_t bits,
    uint8_t *out
) {
    uint32_t mask = (1u << bits) - 1u;
    uint32_t j = mark;
    uint32_t out_len = 0;
    for (uint32_t step = 0; step < m; step++) {
        uint32_t edge = edges[j];
        uint32_t code = wide ? extra[j] : (edge >> bits);
        j = edge & mask;
        if (code < 256u) {
            out[out_len++] = (uint8_t)code;
        } else {
            uint32_t w = code - 256u;
            uint32_t s0 = offsets[w];
            uint32_t len = offsets[w + 1] - s0;
            if (out_len + len <= n) {
                memcpy(out + out_len, spelling + s0, len);
                out_len += len;
            } else {
                uint32_t take = n - out_len;
                memcpy(out + out_len, spelling + s0, take);
                out_len = n;
            }
        }
        if (j == mark || out_len >= n) break;
    }
    uint32_t plen = out_len;
    while (out_len < n) {
        uint32_t take = (n - out_len) < plen ? (n - out_len) : plen;
        memcpy(out + out_len, out, take);
        out_len += take;
    }
}

/* Branchless 16-byte short-word emit (requires out padded with >= 64 spare bytes & spelling padded with >= 16 bytes) */
static void emit_fast16(
    const uint32_t *edges,
    const uint32_t *extra,
    int wide,
    const uint8_t *spelling,
    const uint32_t *offsets,
    uint32_t m,
    uint32_t n,
    uint32_t mark,
    uint32_t bits,
    uint8_t *out
) {
    uint32_t mask = (1u << bits) - 1u;
    uint32_t j = mark;
    uint32_t out_len = 0;
    for (uint32_t step = 0; step < m; step++) {
        uint32_t edge = edges[j];
        uint32_t code = wide ? extra[j] : (edge >> bits);
        j = edge & mask;
        if (code < 256u) {
            out[out_len++] = (uint8_t)code;
        } else {
            uint32_t w = code - 256u;
            uint32_t s0 = offsets[w];
            uint32_t len = offsets[w + 1] - s0;
            if (__builtin_expect(len <= 16u && out_len + 16u <= n + 64u, 1)) {
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
        memcpy(out + out_len, out, take);
        out_len += take;
    }
}

/* ============================================================================
 * 1. BASELINE: Open-qi16 (faithful C port of src/open.rs + endpoint.rs)
 * ============================================================================ */
static PhaseStats run_open_qi16(
    Workspace *ws,
    const uint16_t *records,
    uint32_t n_recs,
    uint32_t n,
    uint32_t p,
    uint8_t *out,
    int fast_emit
) {
    PhaseStats st = {0};
    uint64_t t0 = now_ns();

    uint32_t all[256] = {0};
    uint32_t thin[256] = {0};
    uint32_t thick[256] = {0};
    for (int a = 0; a < 256; a++) ws->q_len[a] = 0;

    uint32_t m = 0, mark = 0, n_strips = 0;
    uint32_t pos = 0, i = 0;
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
        if (width > LIMIT && p >= pos && p < end) {
            if (pos < p) {
                uint32_t w1 = p - pos;
                ws->strips[n_strips++] = (Strip){pos, all[c], w1, c};
                thick[c]++;
                all[c] += w1;
            }
            mark = m;
            q_push_range(ws, c, m, 1);
            thin[c] += 1;
            m += 1;
            all[c] += 1;
            if (p + 1 < end) {
                uint32_t w2 = end - (p + 1);
                ws->strips[n_strips++] = (Strip){p + 1, all[c], w2, c};
                thick[c]++;
                all[c] += w2;
            }
        } else if (width <= LIMIT) {
            if (p >= pos && p < end) mark = m + (p - pos);
            q_push_range(ws, c, m, width);
            thin[c] += width;
            m += width;
            all[c] += width;
        } else {
            ws->strips[n_strips++] = (Strip){pos, all[c], width, c};
            thick[c]++;
            all[c] += width;
        }
        pos = end;
    }

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

    /* Endpoint RV */
    RvNode *nodes = ws->nodes;
    nodes[0] = (RvNode){{0, 0}, {0, 0}, 0, 0};
    uint32_t n_nodes = 1;
    for (uint32_t si = 0; si < n_strips; si++) {
        nodes[n_nodes++] = (RvNode){{0, 0}, {0, 0}, ws->strips[si].width, (uint32_t)ws->strips[si].byte + 1u};
    }
    uint32_t n_o0 = 0, n_o1 = 0, n_t0 = 0, n_t1 = 0;
    uint32_t x = 0;
    for (uint32_t si = 0; si < n_strips; si++) {
        if (x < ws->strips[si].from) {
            uint32_t id = n_nodes++;
            nodes[id] = (RvNode){{0, 0}, {0, 0}, ws->strips[si].from - x, 0};
            ws->order0[n_o0++] = id;
            ws->term1[n_t1++] = id;
        }
        ws->order0[n_o0++] = si + 1u;
        x = ws->strips[si].from + ws->strips[si].width;
    }
    if (x < n) {
        uint32_t id = n_nodes++;
        nodes[id] = (RvNode){{0, 0}, {0, 0}, n - x, 0};
        ws->order0[n_o0++] = id;
        ws->term1[n_t1++] = id;
    }
    x = 0;
    for (uint32_t idx = 0; idx < n_strips; idx++) {
        uint32_t si = ws->image_order[idx];
        if (x < ws->strips[si].to) {
            uint32_t id = n_nodes++;
            nodes[id] = (RvNode){{0, 0}, {0, 0}, ws->strips[si].to - x, 0};
            ws->order1[n_o1++] = id;
            ws->term0[n_t0++] = id;
        }
        ws->order1[n_o1++] = si + 1u;
        x = ws->strips[si].to + ws->strips[si].width;
    }
    if (x < n) {
        uint32_t id = n_nodes++;
        nodes[id] = (RvNode){{0, 0}, {0, 0}, n - x, 0};
        ws->order1[n_o1++] = id;
        ws->term0[n_t0++] = id;
    }

    uint32_t prev = 0;
    for (uint32_t u = 0; u < n_t0; u++) {
        uint32_t id = ws->term0[u];
        nodes[prev].next[0] = id; nodes[id].prev[0] = prev; prev = id;
    }
    for (uint32_t u = 0; u < n_o0; u++) {
        uint32_t id = ws->order0[u];
        nodes[prev].next[0] = id; nodes[id].prev[0] = prev; prev = id;
    }
    nodes[prev].next[0] = 0; nodes[0].prev[0] = prev;

    prev = 0;
    for (uint32_t u = 0; u < n_t1; u++) {
        uint32_t id = ws->term1[u];
        nodes[prev].next[1] = id; nodes[id].prev[1] = prev; prev = id;
    }
    for (uint32_t u = 0; u < n_o1; u++) {
        uint32_t id = ws->order1[u];
        nodes[prev].next[1] = id; nodes[id].prev[1] = prev; prev = id;
    }
    nodes[prev].next[1] = 0; nodes[0].prev[1] = prev;

    uint32_t n_rules = 0;
    uint32_t rv_steps = rv_eliminate(nodes, ws->rules, &n_rules, n);

    uint32_t id = nodes[0].next[1];
    x = 0;
    while (id != 0) {
        ws->bottom[id] = x;
        x += nodes[id].width;
        id = nodes[id].next[1];
    }

    uint32_t cursors[257];
    uint32_t n_bands = 0, sp_len = 0, n_words = 0;
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
            if (nodes[id].word == 0) {
                code = a;
            } else {
                sp_len = rv_append_word(nodes[id].word, ws->rules, ws->stack, ws->spelling, sp_len);
                ws->spelling[sp_len++] = (uint8_t)a;
                n_words++;
                ws->offsets[n_words] = sp_len;
                code = 255u + n_words;
            }
            ws->bands[n_bands++] = (Band){k + w, dst - k, code};
            in_node += w;
            k += w;
            if (in_node == nw) {
                id = nodes[id].next[0];
                in_node = 0;
            }
        }
    }
    cursors[256] = n_bands;
    uint64_t t2 = now_ns();

    /* Wiring */
    uint32_t bits = m <= 1 ? 1u : (32u - (uint32_t)__builtin_clz(m - 1u));
    int wide = (255u + n_words) > (UINT32_MAX >> bits);
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
            uint32_t *edst = ws->edges + dst;
            const uint32_t *src = qsrc + from;
            for (uint32_t u = 0; u < span; u++) {
                edst[u] = pack | src[u];
            }
            from += span;
            k = b.end;
        }
    }
    uint64_t t3 = now_ns();

    /* Emit */
    if (fast_emit) {
        emit_fast16(ws->edges, ws->extra, wide, ws->spelling, ws->offsets, m, n, mark, bits, out);
    } else {
        emit_standard(ws->edges, ws->extra, wide, ws->spelling, ws->offsets, m, n, mark, bits, out);
    }
    uint64_t t4 = now_ns();

    st.t_pass1_us = (double)(t1 - t0) / 1e3;
    st.t_chains_us = 0.0;
    st.t_rv_us = (double)(t2 - t1) / 1e3;
    st.t_wire_us = (double)(t3 - t2) / 1e3;
    st.t_emit_us = (double)(t4 - t3) / 1e3;
    st.t_total_us = (double)(t4 - t0) / 1e3;
    st.m_rows = m;
    st.rv_steps = rv_steps;
    return st;
}

/* ============================================================================
 * 2. DECO: Decoupled Exact-Chain Open (min_w = 1 or 2)
 * ============================================================================ */
static PhaseStats run_deco(
    Workspace *ws,
    const uint16_t *records,
    uint32_t n_recs,
    uint32_t n,
    uint32_t p,
    uint32_t min_w,
    uint8_t *out,
    int fast_emit
) {
    PhaseStats st = {0};
    uint64_t t0 = now_ns();

    /* Pass 1: Merge capped records into maximal runs, splitting at p if needed */
    uint32_t all_counts[256] = {0};
    uint32_t sym_r_cnt[256] = {0};
    uint32_t r = 0, p_run = 0;
    uint32_t pos = 0, i = 0;
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
        uint8_t keep = (width <= LIMIT) ? 1u : 0u;
        if (p >= pos && p < end && (!keep || width >= min_w)) {
            if (pos < p) {
                ws->run_start[r] = pos;
                ws->run_width[r] = p - pos;
                ws->run_sym[r] = c;
                ws->run_thin[r] = keep;
                sym_r_cnt[c]++;
                r++;
            }
            p_run = r;
            ws->run_start[r] = p;
            ws->run_width[r] = 1u;
            ws->run_sym[r] = c;
            ws->run_thin[r] = 1u;
            sym_r_cnt[c]++;
            r++;
            if (p + 1 < end) {
                ws->run_start[r] = p + 1;
                ws->run_width[r] = end - (p + 1);
                ws->run_sym[r] = c;
                ws->run_thin[r] = keep;
                sym_r_cnt[c]++;
                r++;
            }
        } else {
            if (p >= pos && p < end) p_run = r;
            ws->run_start[r] = pos;
            ws->run_width[r] = width;
            ws->run_sym[r] = c;
            ws->run_thin[r] = keep;
            sym_r_cnt[c]++;
            r++;
        }
        all_counts[c] += width;
        pos = end;
    }

    uint32_t occ[256], cod_write[256];
    uint32_t acc_c = 0, acc_r = 0;
    for (int a = 0; a < 256; a++) {
        occ[a] = acc_c; acc_c += all_counts[a];
        cod_write[a] = acc_r; acc_r += sym_r_cnt[a];
    }
    for (uint32_t rid = 0; rid < r; rid++) {
        uint8_t c = ws->run_sym[rid];
        ws->img_start[rid] = occ[c];
        occ[c] += ws->run_width[rid];
        ws->cod_order[cod_write[c]++] = rid;
    }

    /* Two-pointer exact thin-to-thin match scan (<= 2r comparisons, Q = 0) */
    memset(ws->succ_exact, 0xFF, r * sizeof(int32_t));
    memset(ws->has_pred, 0, r);
    uint32_t mi = 0, vi = 0;
    while (mi < r && vi < r) {
        uint32_t u = ws->cod_order[mi];
        uint32_t iu = ws->img_start[u];
        uint32_t sv = ws->run_start[vi];
        if (iu == sv) {
            uint32_t wu = ws->run_width[u];
            if (u != vi && vi != p_run && ws->run_thin[u] && ws->run_thin[vi] &&
                wu == ws->run_width[vi] && wu >= min_w) {
                ws->succ_exact[u] = (int32_t)vi;
                ws->has_pred[vi] = 1u;
            }
            mi++;
            vi++;
        } else if (iu < sv) {
            mi++;
        } else {
            vi++;
        }
    }
    uint64_t t1 = now_ns();

    /* Step 2: Assign compressed ranks [0, m_prime), 16-lane lockstep chain trace, and flat queue fill */
    uint32_t m_prime = 0, mark = 0;
    uint32_t n_chains = 0;
    uint32_t thin_e_counts[256] = {0};
    uint32_t q_cnt[256] = {0};

    for (uint32_t u = 0; u < r; u++) {
        if (ws->run_thin[u] && !ws->has_pred[u]) {
            uint32_t wu = ws->run_width[u];
            if (u == p_run) mark = m_prime + (p - ws->run_start[u]);
            if (ws->succ_exact[u] < 0) {
                ws->tail_head_m[u] = (int32_t)m_prime;
                ws->tail_sfx_id[u] = -1;
                uint8_t a = ws->run_sym[u];
                thin_e_counts[a] += wu;
                q_cnt[a]++;
            } else {
                ws->tail_head_m[u] = (int32_t)m_prime; /* temporarily store at head u */
                ws->chain_heads[n_chains++] = u;
            }
            m_prime += wu;
        }
    }

    /* 16-lane lockstep batched chain trace for L >= 2 chains! */
    uint32_t sfx_len = 0, n_sfx = 0;
    ws->sfx_offsets[0] = 0;
    {
        #define LANES 16
        uint32_t cur_u[LANES];
        uint32_t head_m_lane[LANES];
        uint32_t head_w_lane[LANES];
        uint32_t len_lane[LANES];
        uint8_t buf_lane[LANES][64];
        uint32_t next_ch = 0;
        int active = 0;

        for (int l = 0; l < LANES && next_ch < n_chains; l++) {
            uint32_t u0 = ws->chain_heads[next_ch++];
            cur_u[l] = u0;
            head_m_lane[l] = (uint32_t)ws->tail_head_m[u0];
            head_w_lane[l] = ws->run_width[u0];
            len_lane[l] = 0;
            active++;
        }

        while (active > 0) {
            for (int l = 0; l < LANES; l++) {
                if (head_w_lane[l] == 0) continue;
                uint32_t u = cur_u[l];
                int32_t nx = ws->succ_exact[u];
                if (nx >= 0 && len_lane[l] < 64) {
                    buf_lane[l][len_lane[l]++] = ws->run_sym[u];
                    cur_u[l] = (uint32_t)nx;
                } else {
                    /* Finish lane (handle rare >64 chain via scalar tail if needed) */
                    uint32_t sfx_start = sfx_len;
                    if (__builtin_expect(nx >= 0, 0)) {
                        /* Fallback for chains > 64: push remaining onto stack, then reverse */
                        uint32_t sp_tmp = 0;
                        while (nx >= 0) {
                            ws->stack[sp_tmp++] = ws->run_sym[u];
                            u = (uint32_t)nx;
                            nx = ws->succ_exact[u];
                        }
                        while (sp_tmp > 0) ws->sfx_bytes[sfx_len++] = (uint8_t)ws->stack[--sp_tmp];
                    }
                    uint32_t ll = len_lane[l];
                    for (uint32_t k_rev = 0; k_rev < ll; k_rev++) {
                        ws->sfx_bytes[sfx_len++] = buf_lane[l][ll - 1u - k_rev];
                    }
                    (void)sfx_start;
                    int32_t sid = (int32_t)n_sfx++;
                    ws->sfx_offsets[n_sfx] = sfx_len;

                    ws->tail_head_m[u] = (int32_t)head_m_lane[l];
                    ws->tail_sfx_id[u] = sid;
                    uint8_t a = ws->run_sym[u];
                    thin_e_counts[a] += head_w_lane[l];
                    q_cnt[a]++;

                    if (next_ch < n_chains) {
                        uint32_t u0 = ws->chain_heads[next_ch++];
                        cur_u[l] = u0;
                        head_m_lane[l] = (uint32_t)ws->tail_head_m[u0];
                        head_w_lane[l] = ws->run_width[u0];
                        len_lane[l] = 0;
                    } else {
                        head_w_lane[l] = 0;
                        active--;
                    }
                }
            }
        }
        #undef LANES
    }

    uint32_t q_head[257], q_wr[256];
    uint32_t q_acc = 0;
    for (int a = 0; a < 256; a++) {
        q_head[a] = q_acc;
        q_wr[a] = q_acc;
        q_acc += q_cnt[a];
    }
    q_head[256] = q_acc;

    for (uint32_t z = 0; z < r; z++) {
        if (ws->run_thin[z] && ws->succ_exact[z] < 0) {
            uint8_t a = ws->run_sym[z];
            ws->deco_q[q_wr[a]++] = (DecoQueueItem){
                (uint32_t)ws->tail_head_m[z],
                (uint16_t)ws->run_width[z],
                ws->tail_sfx_id[z]
            };
        }
    }
    uint64_t t2 = now_ns();

    /* Step 3: Void-Deleted Endpoint RV on the H undivided thick strips */
    RvNode *nodes = ws->nodes;
    nodes[0] = (RvNode){{0, 0}, {0, 0}, 0, 0};
    uint32_t n_nodes = 1;
    uint32_t n_o0 = 0, n_o1 = 0, n_t0 = 0, n_t1 = 0;
    uint32_t thick_total = 0;

    /* Assign node IDs to thick runs and build domain order0 + term1 in one scan */
    uint32_t gap_dom = 0;
    for (uint32_t u = 0; u < r; u++) {
        if (ws->run_thin[u]) {
            if (!ws->has_pred[u]) gap_dom += ws->run_width[u];
        } else {
            if (gap_dom > 0) {
                uint32_t gid = n_nodes++;
                nodes[gid] = (RvNode){{0, 0}, {0, 0}, gap_dom, 0};
                ws->order0[n_o0++] = gid;
                ws->term1[n_t1++] = gid;
                gap_dom = 0;
            }
            uint32_t tid = n_nodes++;
            uint32_t wu = ws->run_width[u];
            nodes[tid] = (RvNode){{0, 0}, {0, 0}, wu, (uint32_t)ws->run_sym[u] + 1u};
            ws->tail_head_m[u] = (int32_t)tid; /* reuse array for thick node ID! */
            ws->order0[n_o0++] = tid;
            thick_total += wu;
        }
    }
    if (gap_dom > 0) {
        uint32_t gid = n_nodes++;
        nodes[gid] = (RvNode){{0, 0}, {0, 0}, gap_dom, 0};
        ws->order0[n_o0++] = gid;
        ws->term1[n_t1++] = gid;
    }

    /* Codomain scan over cod_order[0..r] */
    uint32_t gap_cod = 0;
    for (uint32_t idx = 0; idx < r; idx++) {
        uint32_t z = ws->cod_order[idx];
        if (ws->run_thin[z]) {
            if (ws->succ_exact[z] < 0) gap_cod += ws->run_width[z];
        } else {
            if (gap_cod > 0) {
                uint32_t gid = n_nodes++;
                nodes[gid] = (RvNode){{0, 0}, {0, 0}, gap_cod, 0};
                ws->order1[n_o1++] = gid;
                ws->term0[n_t0++] = gid;
                gap_cod = 0;
            }
            ws->order1[n_o1++] = (uint32_t)ws->tail_head_m[z];
        }
    }
    if (gap_cod > 0) {
        uint32_t gid = n_nodes++;
        nodes[gid] = (RvNode){{0, 0}, {0, 0}, gap_cod, 0};
        ws->order1[n_o1++] = gid;
        ws->term0[n_t0++] = gid;
    }

    uint32_t prev = 0;
    for (uint32_t u = 0; u < n_t0; u++) {
        uint32_t id = ws->term0[u];
        nodes[prev].next[0] = id; nodes[id].prev[0] = prev; prev = id;
    }
    for (uint32_t u = 0; u < n_o0; u++) {
        uint32_t id = ws->order0[u];
        nodes[prev].next[0] = id; nodes[id].prev[0] = prev; prev = id;
    }
    nodes[prev].next[0] = 0; nodes[0].prev[0] = prev;

    prev = 0;
    for (uint32_t u = 0; u < n_t1; u++) {
        uint32_t id = ws->term1[u];
        nodes[prev].next[1] = id; nodes[id].prev[1] = prev; prev = id;
    }
    for (uint32_t u = 0; u < n_o1; u++) {
        uint32_t id = ws->order1[u];
        nodes[prev].next[1] = id; nodes[id].prev[1] = prev; prev = id;
    }
    nodes[prev].next[1] = 0; nodes[0].prev[1] = prev;

    uint32_t n_rules = 0;
    uint32_t rv_steps = rv_eliminate(nodes, ws->rules, &n_rules, m_prime + thick_total);

    uint32_t id = nodes[0].next[1];
    uint32_t bx = 0;
    while (id != 0) {
        ws->bottom[id] = bx;
        bx += nodes[id].width;
        id = nodes[id].next[1];
    }
    uint64_t t3 = now_ns();

    /* Step 4: Band spelling + Edge wiring into edges[0..m_prime] */
    /* Upper bound on total words: bands + n_chains <= r */
    uint32_t bits = m_prime <= 1 ? 1u : (32u - (uint32_t)__builtin_clz(m_prime - 1u));
    /* Conservatively check if wide could happen: max words <= n_o0 + n_sfx + 256 */
    int wide = (255u + n_nodes + n_sfx + 256u) > (UINT32_MAX >> bits);

    uint32_t sp_len = 0, n_words = 0;
    ws->offsets[0] = 0;
    id = nodes[0].next[0];
    uint32_t in_node = 0, k = 0;

    for (uint32_t a = 0; a < 256; a++) {
        uint32_t end_a = k + thin_e_counts[a];
        const DecoQueueItem *qa = ws->deco_q + q_head[a];
        uint32_t qi = 0, q_off = 0;
        while (k < end_a) {
            uint32_t nw = nodes[id].width;
            uint32_t b_width = (end_a - k) < (nw - in_node) ? (end_a - k) : (nw - in_node);
            uint32_t dst_base = ws->bottom[id] + in_node;

            uint32_t base_code, base_s0 = 0, base_s1 = 0;
            if (nodes[id].word == 0) {
                base_code = a;
            } else {
                base_s0 = sp_len;
                sp_len = rv_append_word(nodes[id].word, ws->rules, ws->stack, ws->spelling, sp_len);
                ws->spelling[sp_len++] = (uint8_t)a;
                base_s1 = sp_len;
                n_words++;
                ws->offsets[n_words] = sp_len;
                base_code = 255u + n_words;
            }

            uint32_t rem_b = b_width;
            uint32_t cur_dst = dst_base;
            while (rem_b > 0) {
                DecoQueueItem item = qa[qi];
                uint32_t avail = (uint32_t)item.width - q_off;
                uint32_t span = rem_b < avail ? rem_b : avail;
                uint32_t src_j = item.head_m + q_off;
                uint32_t use_code;
                if (item.sfx_id < 0) {
                    use_code = base_code;
                } else {
                    if (base_code < 256u) {
                        ws->spelling[sp_len++] = (uint8_t)a;
                    } else {
                        uint32_t blen = base_s1 - base_s0;
                        memmove(ws->spelling + sp_len, ws->spelling + base_s0, blen);
                        sp_len += blen;
                    }
                    uint32_t sf0 = ws->sfx_offsets[item.sfx_id];
                    uint32_t sflen = ws->sfx_offsets[item.sfx_id + 1] - sf0;
                    memcpy(ws->spelling + sp_len, ws->sfx_bytes + sf0, sflen);
                    sp_len += sflen;
                    n_words++;
                    ws->offsets[n_words] = sp_len;
                    use_code = 255u + n_words;
                }

                uint32_t pack = wide ? 0u : (use_code << bits);
                if (wide) {
                    for (uint32_t u = 0; u < span; u++) ws->extra[cur_dst + u] = use_code;
                }
                uint32_t *edst = ws->edges + cur_dst;
                for (uint32_t u = 0; u < span; u++) {
                    edst[u] = pack | (src_j + u);
                }
                cur_dst += span;
                rem_b -= span;
                q_off += span;
                if (q_off == item.width) {
                    qi++;
                    q_off = 0;
                }
            }

            in_node += b_width;
            k += b_width;
            if (in_node == nw) {
                id = nodes[id].next[0];
                in_node = 0;
            }
        }
    }
    uint64_t t4 = now_ns();

    /* Step 5: Emit */
    if (fast_emit) {
        emit_fast16(ws->edges, ws->extra, wide, ws->spelling, ws->offsets, m_prime, n, mark, bits, out);
    } else {
        emit_standard(ws->edges, ws->extra, wide, ws->spelling, ws->offsets, m_prime, n, mark, bits, out);
    }
    uint64_t t5 = now_ns();

    st.t_pass1_us = (double)(t1 - t0) / 1e3;
    st.t_chains_us = (double)(t2 - t1) / 1e3;
    st.t_rv_us = (double)(t3 - t2) / 1e3;
    st.t_wire_us = (double)(t4 - t3) / 1e3;
    st.t_emit_us = (double)(t5 - t4) / 1e3;
    st.t_total_us = (double)(t5 - t0) / 1e3;
    st.m_rows = m_prime;
    st.rv_steps = rv_steps;
    return st;
}

/* ============================================================================
 * 3. TTSR-RV: Train-Track Series-Reduced Accelerated RV (m = 0 row walks)
 * ============================================================================ */
static PhaseStats run_ttsr_rv(
    Workspace *ws,
    const uint16_t *records,
    uint32_t n_recs,
    uint32_t n,
    uint32_t p,
    uint8_t *out
) {
    PhaseStats st = {0};
    uint64_t t0 = now_ns();

    uint32_t all_counts[256] = {0};
    uint32_t sym_r_cnt[256] = {0};
    uint32_t r = 0, p_run = 0;
    uint32_t pos = 0, i = 0;
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
        if (p >= pos && p < end) {
            if (pos < p) {
                ws->run_start[r] = pos;
                ws->run_width[r] = p - pos;
                ws->run_sym[r] = c;
                sym_r_cnt[c]++;
                r++;
            }
            p_run = r;
            ws->run_start[r] = p;
            ws->run_width[r] = 1u;
            ws->run_sym[r] = c;
            sym_r_cnt[c]++;
            r++;
            if (p + 1 < end) {
                ws->run_start[r] = p + 1;
                ws->run_width[r] = end - (p + 1);
                ws->run_sym[r] = c;
                sym_r_cnt[c]++;
                r++;
            }
        } else {
            ws->run_start[r] = pos;
            ws->run_width[r] = width;
            ws->run_sym[r] = c;
            sym_r_cnt[c]++;
            r++;
        }
        all_counts[c] += width;
        pos = end;
    }

    uint32_t occ[256], cod_write[256];
    uint32_t acc_c = 0, acc_r = 0;
    for (int a = 0; a < 256; a++) {
        occ[a] = acc_c; acc_c += all_counts[a];
        cod_write[a] = acc_r; acc_r += sym_r_cnt[a];
    }
    for (uint32_t rid = 0; rid < r; rid++) {
        uint8_t c = ws->run_sym[rid];
        ws->img_start[rid] = occ[c];
        occ[c] += ws->run_width[rid];
        ws->cod_order[cod_write[c]++] = rid;
    }

    /* Two-pointer merge of codomain and domain partitions */
    memset(ws->in_deg, 0, r * sizeof(uint32_t));
    memset(ws->out_deg, 0, r * sizeof(uint32_t));
    uint32_t n_arcs = 0, mi = 0, vi = 0, cur_pos = 0;
    while (mi < r && vi < r) {
        uint32_t u = ws->cod_order[mi];
        uint32_t u_end = ws->img_start[u] + ws->run_width[u];
        uint32_t v_end = ws->run_start[vi] + ws->run_width[vi];
        uint32_t nxt = u_end < v_end ? u_end : v_end;
        uint32_t w_arc = nxt - cur_pos;
        if (w_arc > 0) {
            ws->arcs_u[n_arcs] = u;
            ws->arcs_v[n_arcs] = vi;
            ws->arcs_w[n_arcs] = w_arc;
            ws->arcs_uo[n_arcs] = cur_pos - ws->img_start[u];
            ws->arcs_vo[n_arcs] = cur_pos - ws->run_start[vi];
            ws->out_deg[u]++;
            ws->in_deg[vi]++;
            n_arcs++;
        }
        cur_pos = nxt;
        if (u_end == nxt) mi++;
        if (v_end == nxt) vi++;
    }
    uint64_t t1 = now_ns();

    /* Identify passthrough runs and organize core outgoing arcs */
    uint32_t n_core = 0;
    ws->core_runs[n_core++] = p_run;
    for (uint32_t u = 0; u < r; u++) {
        uint8_t pt = (ws->in_deg[u] == 1u && ws->out_deg[u] == 1u && u != p_run) ? 1u : 0u;
        ws->has_pred[u] = pt; /* 1 = passthrough */
        if (!pt && u != p_run) ws->core_runs[n_core++] = u;
        ws->core_out_cnt[u] = 0;
        ws->core_in_cnt[u] = 0;
    }

    for (uint32_t ai = 0; ai < n_arcs; ai++) {
        uint32_t u = ws->arcs_u[ai];
        if (ws->out_deg[u] == 1u) {
            ws->unique_succ[u] = (int32_t)ws->arcs_v[ai];
            ws->unique_voff[u] = ws->arcs_vo[ai];
        }
        if (!ws->has_pred[u]) ws->core_out_cnt[u]++;
    }

    uint32_t acc_co = 0;
    for (uint32_t ci = 0; ci < n_core; ci++) {
        uint32_t u = ws->core_runs[ci];
        ws->core_out_head[u] = acc_co;
        ws->tail_head_m[u] = (int32_t)acc_co; /* write cursor */
        acc_co += ws->core_out_cnt[u];
    }
    uint32_t k_pieces = acc_co;

    /* Populate piece widths, initial arc v/vo, in domain uo order (arcs_u in two-pointer merge already has uo increasing!) */
    for (uint32_t ai = 0; ai < n_arcs; ai++) {
        uint32_t u = ws->arcs_u[ai];
        if (!ws->has_pred[u]) {
            uint32_t pid = (uint32_t)(ws->tail_head_m[u]++);
            ws->run_start[pid] = ws->arcs_w[ai]; /* reuse as piece_width[pid] */
            ws->img_start[pid] = ws->arcs_v[ai]; /* initial v */
            ws->cod_order[pid] = ws->arcs_vo[ai]; /* initial vo */
            ws->run_thin[pid] = ws->run_sym[u];  /* source core byte */
        }
    }

    /* Trace passthrough chains in 16-lane lockstep across all k_pieces! */
    uint32_t leaf_len = 0;
    ws->offsets[0] = 0;
    for (uint32_t pid = 0; pid < k_pieces; pid++) {
        uint32_t cur_v = ws->img_start[pid];
        uint32_t cur_vo = ws->cod_order[pid];
        uint32_t sp_tmp = 0;
        ws->stack[sp_tmp++] = ws->run_thin[pid];
        while (ws->has_pred[cur_v]) {
            ws->stack[sp_tmp++] = ws->run_sym[cur_v];
            cur_vo = ws->unique_voff[cur_v];
            cur_v = (uint32_t)ws->unique_succ[cur_v];
        }
        while (sp_tmp > 0) ws->sfx_bytes[leaf_len++] = (uint8_t)ws->stack[--sp_tmp];
        ws->offsets[pid + 1] = leaf_len;
        ws->img_start[pid] = cur_v;
        ws->cod_order[pid] = cur_vo;
        ws->core_in_cnt[cur_v]++;
    }

    uint32_t acc_ci = 0;
    for (uint32_t ci = 0; ci < n_core; ci++) {
        uint32_t u = ws->core_runs[ci];
        ws->core_in_head[u] = acc_ci;
        ws->tail_sfx_id[u] = (int32_t)acc_ci;
        acc_ci += ws->core_in_cnt[u];
    }
    for (uint32_t pid = 0; pid < k_pieces; pid++) {
        uint32_t cv = ws->img_start[pid];
        uint32_t cvo = ws->cod_order[pid];
        uint32_t pos_in = (uint32_t)(ws->tail_sfx_id[cv]++);
        ws->core_in_pairs[pos_in] = ((uint64_t)cvo << 32) | (uint64_t)pid;
    }

    /* Sort incoming pairs within each core run (small degree: insertion sort is blazingly fast!) */
    RvNode *nodes = ws->nodes;
    nodes[0] = (RvNode){{0, 0}, {0, 0}, 0, 0};
    for (uint32_t pid = 0; pid < k_pieces; pid++) {
        uint32_t uid = pid + 1u;
        nodes[uid].width = ws->run_start[pid];
        nodes[uid].word = pid; /* 0..k_pieces-1 are leaves */
        nodes[uid].prev[0] = pid;
        nodes[uid].next[0] = (uid == k_pieces) ? 0u : (uid + 1u);
    }
    nodes[0].next[0] = 1u;
    nodes[0].prev[0] = k_pieces;

    uint32_t prev1 = 0;
    for (uint32_t ci = 0; ci < n_core; ci++) {
        uint32_t u = ws->core_runs[ci];
        uint32_t h0 = ws->core_in_head[u];
        uint32_t cnt = ws->core_in_cnt[u];
        uint64_t *arr = ws->core_in_pairs + h0;
        for (uint32_t a = 1; a < cnt; a++) {
            uint64_t key = arr[a];
            int32_t b = (int32_t)a - 1;
            while (b >= 0 && arr[b] > key) {
                arr[b + 1] = arr[b];
                b--;
            }
            arr[b + 1] = key;
        }
        for (uint32_t a = 0; a < cnt; a++) {
            uint32_t uid = (uint32_t)(arr[a] & 0xFFFFFFFFu) + 1u;
            nodes[prev1].next[1] = uid;
            nodes[uid].prev[1] = prev1;
            prev1 = uid;
        }
    }
    nodes[prev1].next[1] = 0u;
    nodes[0].prev[1] = prev1;
    uint64_t t2 = now_ns();

    /* Core RV Induction keeping y on equal-width merges so node 1 stays at left end */
    uint32_t n_rules = 0, rv_steps = 0, root_tok = 0;
    RvRule *rules = ws->rules;

    #define JOIN_CORE(lt, rt) ({ \
        uint32_t _idx = n_rules++; \
        rules[_idx].left = (lt); \
        rules[_idx].right = (rt); \
        rules[_idx].len = 0; \
        rules[_idx].first = UINT32_MAX; \
        k_pieces + _idx; \
    })

    while (1) {
        uint32_t x = nodes[0].prev[0];
        uint32_t y = nodes[0].prev[1];
        uint32_t wx = nodes[x].width;
        uint32_t wy = nodes[y].width;
        rv_steps++;
        if (x == y) {
            if (x == 1u) {
                root_tok = nodes[1].word;
                break;
            }
            rv_cut(nodes, x, 0);
            rv_cut(nodes, x, 1);
            continue;
        }
        if (wx < wy) {
            rv_cut(nodes, x, 0);
            rv_insert(nodes, y, x, 0);
            nodes[x].word = JOIN_CORE(nodes[x].word, nodes[y].word);
            nodes[y].width = wy - wx;
        } else if (wy < wx) {
            rv_cut(nodes, y, 1);
            rv_insert(nodes, x, y, 1);
            nodes[y].word = JOIN_CORE(nodes[x].word, nodes[y].word);
            nodes[x].width = wx - wy;
        } else {
            nodes[y].word = JOIN_CORE(nodes[x].word, nodes[y].word);
            rv_cut(nodes, y, 1);
            rv_insert(nodes, x, y, 1);
            rv_cut(nodes, x, 0);
            rv_cut(nodes, x, 1);
        }
    }
    #undef JOIN_CORE
    uint64_t t3 = now_ns();

    /* Expand SLP directly into out[0..n) with first-occurrence memoized copies */
    uint32_t out_len = 0;
    int32_t *stk = (int32_t *)ws->stack;
    uint32_t sp = 0;
    stk[sp++] = (int32_t)root_tok;
    while (sp > 0) {
        int32_t tok = stk[--sp];
        if ((uint32_t)tok < k_pieces) {
            uint32_t s0 = ws->offsets[tok];
            uint32_t len = ws->offsets[tok + 1] - s0;
            uint32_t take = (out_len + len <= n) ? len : (n - out_len);
            memcpy(out + out_len, ws->sfx_bytes + s0, take);
            out_len += take;
        } else {
            uint32_t ridx = (uint32_t)tok - k_pieces;
            RvRule *rl = &rules[ridx];
            if (rl->first == UINT32_MAX) {
                rl->first = out_len;
                stk[sp++] = ~(int32_t)ridx;
                stk[sp++] = (int32_t)rl->right;
                stk[sp++] = (int32_t)rl->left;
            } else {
                uint32_t len = rl->len;
                uint32_t take = (out_len + len <= n) ? len : (n - out_len);
                memmove(out + out_len, out + rl->first, take);
                out_len += take;
            }
        }
        while (sp > 0 && stk[sp - 1] < 0) {
            uint32_t fin = (uint32_t)(~stk[--sp]);
            rules[fin].len = out_len - rules[fin].first;
        }
        if (out_len >= n) break;
    }
    uint32_t plen = out_len;
    while (out_len < n) {
        uint32_t take = (n - out_len) < plen ? (n - out_len) : plen;
        memcpy(out + out_len, out, take);
        out_len += take;
    }
    uint64_t t4 = now_ns();

    st.t_pass1_us = (double)(t1 - t0) / 1e3;
    st.t_chains_us = (double)(t2 - t1) / 1e3;
    st.t_rv_us = (double)(t3 - t2) / 1e3;
    st.t_wire_us = 0.0;
    st.t_emit_us = (double)(t4 - t3) / 1e3;
    st.t_total_us = (double)(t4 - t0) / 1e3;
    st.m_rows = 0;
    st.rv_steps = rv_steps;
    return st;
}

static int cmp_double(const void *a, const void *b) {
    double da = *(const double *)a, db = *(const double *)b;
    return (da > db) - (da < db);
}

typedef struct {
    const char *file;
    uint32_t n;
    uint32_t p;
} BlockSpec;

int main(void) {
    static const BlockSpec specs[6] = {
        {"/home/antigravity/admitted-bwt/block-0.L",   899931, 264045},
        {"/home/antigravity/admitted-bwt/block-2.L",   900000, 320807},
        {"/home/antigravity/admitted-bwt/block-85.L",  899935, 325088},
        {"/home/antigravity/admitted-bwt/block-143.L", 899869, 802868},
        {"/home/antigravity/admitted-bwt/block-207.L", 899998, 205180},
        {"/home/antigravity/admitted-bwt/block-210.L", 899959, 791429},
    };

    const char *arm_names[6] = {
        "Open-qi16",
        "DECO-w2",
        "DECO-w1",
        "Open-qi16-fast16",
        "DECO-w1-fast16",
        "TTSR-RV"
    };

    Workspace *ws = ws_create();
    uint8_t *L = (uint8_t *)malloc(MAX_N);
    uint16_t *recs = (uint16_t *)malloc(MAX_RECS * sizeof(uint16_t));
    uint8_t *ref_out = (uint8_t *)malloc(MAX_N);
    uint8_t *test_out = (uint8_t *)malloc(MAX_N + OUT_PAD + CANARY_LEN);

    #define WARMUP 3
    #define ROUNDS 21
    #define N_ARMS 6

    printf("=== Carbon Kitchen Sink — Gate G5 Pre-Registered C Falsifier (%d paired interleaved rounds) ===\n\n", ROUNDS);

    for (int bi = 0; bi < 6; bi++) {
        FILE *fp = fopen(specs[bi].file, "rb");
        if (!fp) { perror(specs[bi].file); return 1; }
        size_t rd = fread(L, 1, specs[bi].n, fp);
        fclose(fp);
        assert(rd == specs[bi].n);

        uint32_t n = specs[bi].n;
        uint32_t p = specs[bi].p;
        uint32_t n_recs = build_capped_records(L, n, recs);
        dense_lf_invert(L, n, p, ref_out);

        /* Correctness & canary check on every arm before timing */
        PhaseStats last_st[N_ARMS];
        for (int arm = 0; arm < N_ARMS; arm++) {
            memset(test_out, 0, n + OUT_PAD);
            memset(test_out + n + OUT_PAD, CANARY_BYTE, CANARY_LEN);
            memset(ws->spelling, 0, 64);

            PhaseStats st;
            if (arm == 0) st = run_open_qi16(ws, recs, n_recs, n, p, test_out, 0);
            else if (arm == 1) st = run_deco(ws, recs, n_recs, n, p, 2u, test_out, 0);
            else if (arm == 2) st = run_deco(ws, recs, n_recs, n, p, 1u, test_out, 0);
            else if (arm == 3) st = run_open_qi16(ws, recs, n_recs, n, p, test_out, 1);
            else if (arm == 4) st = run_deco(ws, recs, n_recs, n, p, 1u, test_out, 1);
            else st = run_ttsr_rv(ws, recs, n_recs, n, p, test_out);

            if (memcmp(test_out, ref_out, n) != 0) {
                fprintf(stderr, "FAIL byte-exact check on %s arm %s!\n", specs[bi].file, arm_names[arm]);
                return 2;
            }
            for (int c = 0; c < CANARY_LEN; c++) {
                if (test_out[n + OUT_PAD + c] != CANARY_BYTE) {
                    fprintf(stderr, "FAIL canary guard on %s arm %s!\n", specs[bi].file, arm_names[arm]);
                    return 3;
                }
            }
            last_st[arm] = st;
        }

        double tot_times[N_ARMS][ROUNDS];
        double p1_sum[N_ARMS] = {0}, ch_sum[N_ARMS] = {0}, rv_sum[N_ARMS] = {0}, wr_sum[N_ARMS] = {0}, em_sum[N_ARMS] = {0};

        for (int rnd = -WARMUP; rnd < ROUNDS; rnd++) {
            for (int slot = 0; slot < N_ARMS; slot++) {
                int arm = (slot + (rnd + WARMUP)) % N_ARMS;
                PhaseStats st;
                if (arm == 0) st = run_open_qi16(ws, recs, n_recs, n, p, test_out, 0);
                else if (arm == 1) st = run_deco(ws, recs, n_recs, n, p, 2u, test_out, 0);
                else if (arm == 2) st = run_deco(ws, recs, n_recs, n, p, 1u, test_out, 0);
                else if (arm == 3) st = run_open_qi16(ws, recs, n_recs, n, p, test_out, 1);
                else if (arm == 4) st = run_deco(ws, recs, n_recs, n, p, 1u, test_out, 1);
                else st = run_ttsr_rv(ws, recs, n_recs, n, p, test_out);

                if (rnd >= 0) {
                    tot_times[arm][rnd] = st.t_total_us;
                    p1_sum[arm] += st.t_pass1_us;
                    ch_sum[arm] += st.t_chains_us;
                    rv_sum[arm] += st.t_rv_us;
                    wr_sum[arm] += st.t_wire_us;
                    em_sum[arm] += st.t_emit_us;
                }
            }
        }

        for (int arm = 0; arm < N_ARMS; arm++) {
            qsort(tot_times[arm], ROUNDS, sizeof(double), cmp_double);
        }
        double base_med = tot_times[0][ROUNDS / 2];
        double base_fast_med = tot_times[3][ROUNDS / 2];

        printf("--- %s (n=%u, R_capped=%u, p=%u) [ALL 6 ARMS BYTE-EXACT + CANARY OK] ---\n",
               specs[bi].file + 31, n, n_recs, p);
        printf("  %-18s | %8s (%8s..%8s) | %6s | %6s | %7s %7s %7s %7s %7s | %7s %7s\n",
               "Arm", "Med(us)", "p25", "p75", "vsOpen", "vsFast",
               "pass1", "chains", "rv", "wire", "emit", "m_rows", "rv_stp");
        for (int arm = 0; arm < N_ARMS; arm++) {
            double med = tot_times[arm][ROUNDS / 2];
            double p25 = tot_times[arm][ROUNDS / 4];
            double p75 = tot_times[arm][(3 * ROUNDS) / 4];
            printf("  %-18s | %8.1f (%8.1f..%8.1f) | %5.3fx | %5.3fx | %7.1f %7.1f %7.1f %7.1f %7.1f | %7u %7u\n",
                   arm_names[arm], med, p25, p75,
                   med / base_med, med / base_fast_med,
                   p1_sum[arm] / ROUNDS, ch_sum[arm] / ROUNDS, rv_sum[arm] / ROUNDS,
                   wr_sum[arm] / ROUNDS, em_sum[arm] / ROUNDS,
                   last_st[arm].m_rows, last_st[arm].rv_steps);
        }
        printf("\n");
    }

    free(test_out);
    free(ref_out);
    free(recs);
    free(L);
    return 0;
}
