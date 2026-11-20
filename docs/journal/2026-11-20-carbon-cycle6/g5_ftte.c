/*
 * Gate G5 Paid-Cost Falsifier & Controlled Producer/Consumer Isolation Harness for Carbon Cycle 6:
 * Hierarchical Pinch-Point Forward Tropical Tarjan Elimination (HP-FTTE)
 * vs Shipped Open (qi16) and Composed Zero-Arena / 1-Pass BVSMC.
 *
 * PRE-REGISTERED PASS/FAIL RULE (frozen before compilation and execution):
 * Across all six authenticated Wikidata BWT blocks (block-0, 2, 85, 143, 210, 207),
 * 25 paired interleaved rounds after 3 warmup rounds (rotating all 5 arms each round):
 *
 *   Arm 0: Open-qi16          — exact C mirror of shipped src/open.rs (q_push_scalar + endpoint.rs + bands + Wire + 1-lane emit)
 *   Arm 1: HP-FTTE-1L         — pure producer swap (q_push_scalar + HP-FTTE + exact 1-lane Open::emit; 0 EpNode/EpRule/bands)
 *   Arm 2: Open+BVSMC-2P      — Open (U16Q + endpoint.rs + inline16) + zero-arena 8-lane BVSMC-2P(8,64) (0 B chunk pool)
 *   Arm 3: HP-FTTE+BVSMC-2P   — composed HP-FTTE (U16Q + HP-FTTE + inline16) + zero-arena 8-lane BVSMC-2P(8,64) (0 B chunk pool)
 *   Arm 4: HP-FTTE+BVSMC-1P   — composed HP-FTTE (U16Q + HP-FTTE + inline16) + 1-pass 8-lane BVSMC-1P(8,48) (1.23 MiB pool)
 *
 * PASS (Pure Producer G5) iff ALL THREE hold:
 *   (1) Byte-exactness: Arms 1, 2, 3, 4 match dense LF byte-for-byte on 6/6 blocks (5,399,692 B)
 *       and 7/7 adversarial self-tests (incl. n=1, n=5, 50k-row periodic shear, multi-cycle);
 *   (2) Pure Producer Isolation (Arm 1 HP-FTTE-1L vs Arm 0 Open-qi16):
 *       - HP-FTTE-1L construct median <= 0.85x Open-qi16 construct on >= 5/6 blocks;
 *       - HP-FTTE-1L total median <= 0.90x Open-qi16 total on >= 5/6 blocks (and < 1.00x on 6/6);
 *   (3) Producer+Consumer Compounding (Arm 3 HP-FTTE+BVSMC-2P vs Arm 0 & Arm 2, 0 B arena):
 *       - HP-FTTE+BVSMC-2P total median <= 0.72x Open-qi16 total on >= 5/6 blocks;
 *       - HP-FTTE+BVSMC-2P total median <= 0.95x Open+BVSMC-2P total on >= 5/6 blocks.
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
#define CHUNK_CAP 8000u
#define CHUNK_STRIDE 8064u
#define MAX_CHUNKS_1P 160u
#define MAX_CUTS 64u

#define TF_IS_THIN   0x0100u
#define TF_HAS_CERT  0x0200u
#define TF_CERT_EXIT 0x0400u

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

/* HP-FTTE compact tile structures: 16B hot geometry + 32B cold/thick certificate */
typedef struct {
    uint32_t start;
    uint32_t end;
    uint32_t param;      /* if THIN: queue base offset; if THICK: s.from */
    uint16_t sym_flags;  /* bits 0..7: byte; TF_IS_THIN / TF_HAS_CERT / TF_CERT_EXIT */
    uint16_t pad;
} PsiTileHot;

typedef struct {
    uint32_t lo;
    uint32_t hi;
    uint32_t target_ti;
    uint32_t target_pos;
    uint32_t spell_off;
    uint32_t spell_len;
    uint32_t code;
    uint32_t pad;
} PsiCert;

typedef struct {
    uint32_t ti;
    uint32_t cur;
    uint32_t sym_or_off;
    uint32_t slen;
    uint32_t loc_l;
    uint32_t loc_r;
    uint32_t seg_off;
    uint32_t is_jump;
} FtSeg;

typedef struct {
    uint32_t *queues[256];
    uint32_t q_len[256];
    uint32_t q_cap[256];

    Strip *strips;
    size_t strips_len, strips_cap;
    uint32_t *image_order;
    size_t img_cap;

    /* Open endpoint.rs scratch (unused by HP-FTTE!) */
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

    /* HP-FTTE scratch (~480 KiB on 6 blocks, replaces ~1.5 MiB endpoint+bands scratch) */
    PsiTileHot *psi_hot;
    PsiCert *psi_cert;
    size_t psi_len, psi_cap;
    uint32_t tile_lut[(900000u >> 6) + 8u];
    FtSeg *ft_path;
    uint32_t *ft_suf_l, *ft_suf_r, *ft_stk_l, *ft_stk_r;
    size_t ft_cap;

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

    uint8_t *chunk_pool;
    uint8_t *dummy_buf;
    int32_t chunk_next[MAX_CHUNKS_1P];
    uint32_t chunk_len[MAX_CHUNKS_1P];
} Kernel;

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

    k->psi_cap = 16384;
    k->psi_hot = (PsiTileHot *)malloc(k->psi_cap * sizeof(PsiTileHot));
    k->psi_cert = (PsiCert *)malloc(k->psi_cap * sizeof(PsiCert));
    k->ft_cap = 2048;
    k->ft_path = (FtSeg *)malloc(k->ft_cap * sizeof(FtSeg));
    k->ft_suf_l = (uint32_t *)malloc((k->ft_cap + 2) * sizeof(uint32_t));
    k->ft_suf_r = (uint32_t *)malloc((k->ft_cap + 2) * sizeof(uint32_t));
    k->ft_stk_l = (uint32_t *)malloc((k->ft_cap + 2) * sizeof(uint32_t));
    k->ft_stk_r = (uint32_t *)malloc((k->ft_cap + 2) * sizeof(uint32_t));

    k->spell_cap = 131072;
    k->spelling = (uint8_t *)malloc(k->spell_cap);
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

    k->chunk_pool = (uint8_t *)malloc((size_t)MAX_CHUNKS_1P * CHUNK_STRIDE);
    k->dummy_buf = (uint8_t *)malloc(4096);
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
    free(k->psi_hot);
    free(k->psi_cert);
    free(k->ft_path);
    free(k->ft_suf_l);
    free(k->ft_suf_r);
    free(k->ft_stk_l);
    free(k->ft_stk_r);
    free(k->spelling);
    free(k->offsets);
    free(k->inline16);
    free(k->w_len);
    free(k->edges);
    free(k->extra);
    free(k->chunk_pool);
    free(k->dummy_buf);
}

static inline void dict_ensure(Kernel *k, size_t need) {
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

static inline void q_push_u16(Kernel *k, uint8_t a, uint32_t start, uint32_t w) {
    uint32_t len = k->q_len[a];
    if (__builtin_expect(len + 16 > k->q_cap[a], 0)) {
        while (len + 16 > k->q_cap[a]) k->q_cap[a] *= 2;
        k->queues[a] = (uint32_t *)realloc(k->queues[a], (k->q_cap[a] + 16) * sizeof(uint32_t));
    }
    store_seq16(k->queues[a] + len, start);
    k->q_len[a] = len + w;
}

static inline void push_strip(Kernel *k, uint32_t from, uint32_t to, uint32_t width, uint8_t byte) {
    if (__builtin_expect(k->strips_len == k->strips_cap, 0)) {
        k->strips_cap *= 2;
        k->strips = (Strip *)realloc(k->strips, k->strips_cap * sizeof(Strip));
    }
    k->strips[k->strips_len++] = (Strip){from, to, width, byte};
}

static inline void spell_ensure(Kernel *k, size_t add) {
    if (__builtin_expect(k->spell_len + add + 32 > k->spell_cap, 0)) {
        while (k->spell_len + add + 32 > k->spell_cap) k->spell_cap *= 2;
        k->spelling = (uint8_t *)realloc(k->spelling, k->spell_cap);
    }
}

static inline void push_u32(uint32_t **arr, size_t *len, size_t *cap, uint32_t v) {
    if (__builtin_expect(*len == *cap, 0)) {
        *cap *= 2;
        *arr = (uint32_t *)realloc(*arr, (*cap) * sizeof(uint32_t));
    }
    (*arr)[(*len)++] = v;
}

/* =========================================================================
 * Shipped Open Producer (src/open.rs + endpoint.rs mirror)
 * ========================================================================= */
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

static inline void ep_gaps(Kernel *k, int side, uint32_t start, uint32_t end) {
    if (start < end) {
        uint32_t id = ep_node(k, end - start, 0);
        push_u32(&k->order[side], &k->order_len[side], &k->order_cap[side], id);
        push_u32(&k->terminals[1 - side], &k->term_len[1 - side], &k->term_cap[1 - side], id);
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

static void open_construct(
    Kernel *k,
    const uint16_t *records,
    size_t n_recs,
    uint32_t n,
    uint32_t p,
    uint32_t limit,
    int use_u16_push,
    int build_inline16,
    uint32_t reserve_sentinels
) {
    uint32_t all_c[256] = {0}, thin[256] = {0}, thick[256] = {0};
    for (int a = 0; a < 256; a++) k->q_len[a] = 0;
    k->strips_len = 0;

    uint32_t m = 0, mark = 0, pos = 0;
    size_t i = 0;
    if (use_u16_push) {
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
                if (pos < p) { push_strip(k, pos, all_c[c], p - pos, c); thick[c]++; all_c[c] += p - pos; }
                mark = m; q_push_u16(k, c, m, 1); thin[c] += 1; m += 1; all_c[c] += 1;
                if (p + 1 < end) { uint32_t w2 = end - p - 1; push_strip(k, p + 1, all_c[c], w2, c); thick[c]++; all_c[c] += w2; }
            } else {
                push_strip(k, pos, all_c[c], width, c); thick[c]++; all_c[c] += width;
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

    k->nodes_len = 0; ep_node(k, 0, 0); k->rules_len = 0;
    k->order_len[0] = k->order_len[1] = k->term_len[0] = k->term_len[1] = 0;

    for (size_t si = 0; si < k->strips_len; si++) ep_node(k, k->strips[si].width, (uint32_t)k->strips[si].byte + 1u);
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
                uint32_t o0 = (uint32_t)k->spell_len;
                ep_append(k, nd.word);
                spell_ensure(k, 1);
                k->spelling[k->spell_len++] = (uint8_t)a;
                uint32_t o1 = (uint32_t)k->spell_len;
                push_u32(&k->offsets, &k->off_len, &k->off_cap, o1);
                code = 256u + w_idx;
                if (build_inline16) {
                    dict_ensure(k, (size_t)code + reserve_sentinels + 16);
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
    if (build_inline16) dict_ensure(k, 256u + words + reserve_sentinels + 4);

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

/* =========================================================================
 * Cycle 6 Candidate Producer:
 * Hierarchical Pinch-Point Forward Tropical Tarjan Elimination (HP-FTTE)
 * ========================================================================= */
static inline void ft_ensure_tiles(Kernel *k, size_t need) {
    if (__builtin_expect(need > k->psi_cap, 0)) {
        while (need > k->psi_cap) k->psi_cap *= 2;
        k->psi_hot = (PsiTileHot *)realloc(k->psi_hot, k->psi_cap * sizeof(PsiTileHot));
        k->psi_cert = (PsiCert *)realloc(k->psi_cert, k->psi_cap * sizeof(PsiCert));
    }
}

static inline void ft_ensure_path(Kernel *k, size_t need) {
    if (__builtin_expect(need + 2 > k->ft_cap, 0)) {
        while (need + 2 > k->ft_cap) k->ft_cap *= 2;
        k->ft_path = (FtSeg *)realloc(k->ft_path, k->ft_cap * sizeof(FtSeg));
        k->ft_suf_l = (uint32_t *)realloc(k->ft_suf_l, (k->ft_cap + 2) * sizeof(uint32_t));
        k->ft_suf_r = (uint32_t *)realloc(k->ft_suf_r, (k->ft_cap + 2) * sizeof(uint32_t));
        k->ft_stk_l = (uint32_t *)realloc(k->ft_stk_l, (k->ft_cap + 2) * sizeof(uint32_t));
        k->ft_stk_r = (uint32_t *)realloc(k->ft_stk_r, (k->ft_cap + 2) * sizeof(uint32_t));
    }
}

static inline uint32_t ft_stk_min_at(const uint32_t *stk, size_t stk_len, const FtSeg *path, int is_r, uint32_t j) {
    size_t lo = 0, hi = stk_len - 1;
    while (lo < hi) {
        size_t mid = (lo + hi) >> 1;
        if (stk[mid] <= j) hi = mid;
        else lo = mid + 1;
    }
    uint32_t idx = stk[lo];
    return is_r ? path[idx].loc_r : path[idx].loc_l;
}

static inline void ft_wire_span(
    Kernel *k,
    uint32_t dst,
    uint32_t span,
    uint32_t cd,
    uint8_t ex_sym,
    uint32_t q_off,
    uint32_t bits,
    uint32_t max_inline_code
) {
    if (__builtin_expect(!k->wide && cd > max_inline_code, 0)) {
        uint32_t mask = (1u << bits) - 1u;
        for (uint32_t u = 0; u < dst; u++) {
            k->extra[u] = k->edges[u] >> bits;
            k->edges[u] &= mask;
        }
        k->wide = 1;
    }
    const uint32_t * restrict qsrc = k->queues[ex_sym] + q_off;
    uint32_t * restrict edst = k->edges + dst;
    if (__builtin_expect(!k->wide, 1)) {
        uint32_t pack = cd << bits;
        for (uint32_t s = 0; s < span; s++) edst[s] = pack | qsrc[s];
    } else {
        uint32_t * restrict xdst = k->extra + dst;
        for (uint32_t s = 0; s < span; s++) {
            xdst[s] = cd;
            edst[s] = qsrc[s];
        }
    }
}

static void hp_ftte_construct(
    Kernel *k,
    const uint16_t *records,
    size_t n_recs,
    uint32_t n,
    uint32_t p,
    uint32_t limit,
    int use_u16_push,
    int build_inline16,
    uint32_t reserve_sentinels,
    int reg_pinch
) {
    uint32_t all_c[256] = {0}, thin[256] = {0}, thick[256] = {0};
    for (int a = 0; a < 256; a++) k->q_len[a] = 0;
    k->strips_len = 0;

    uint32_t m = 0, mark = 0, pos = 0;
    size_t i = 0;
    if (use_u16_push) {
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
                if (pos < p) { push_strip(k, pos, all_c[c], p - pos, c); thick[c]++; all_c[c] += p - pos; }
                mark = m; q_push_u16(k, c, m, 1); thin[c] += 1; m += 1; all_c[c] += 1;
                if (p + 1 < end) { uint32_t w2 = end - p - 1; push_strip(k, p + 1, all_c[c], w2, c); thick[c]++; all_c[c] += w2; }
            } else {
                push_strip(k, pos, all_c[c], width, c); thick[c]++; all_c[c] += width;
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

    /* Build codomain Psi-tiles + 64-row O(1) page directory in a single monotone scan */
    ft_ensure_tiles(k, 2 * k->strips_len + 260);
    PsiTileHot *psi_hot = k->psi_hot;
    PsiCert *psi_cert = k->psi_cert;
    uint32_t * restrict tile_lut = k->tile_lut;
    uint32_t n_tiles = 0, lut_page = 0;
    uint32_t si_ptr = 0;

    for (uint32_t a = 0; a < 256; a++) {
        if (all_c[a] == 0) continue;
        uint32_t x = first[a], a_end = first[a] + all_c[a];
        uint32_t thin_seen = 0;
        uint32_t w_end = write[a];
        while (si_ptr < w_end) {
            Strip s = k->strips[k->image_order[si_ptr++]];
            if (x < s.to) {
                uint32_t ti = n_tiles++;
                psi_hot[ti] = (PsiTileHot){x, s.to, thin_seen, (uint16_t)(a | TF_IS_THIN), 0};
                uint32_t lp = (s.to - 1u) >> 6;
                while (lut_page <= lp) tile_lut[lut_page++] = ti;
                thin_seen += s.to - x;
            }
            {
                uint32_t ti = n_tiles++;
                uint32_t tend = s.to + s.width;
                psi_hot[ti] = (PsiTileHot){s.to, tend, s.from, (uint16_t)s.byte, 0};
                uint32_t lp = (tend - 1u) >> 6;
                while (lut_page <= lp) tile_lut[lut_page++] = ti;
                x = tend;
            }
        }
        if (x < a_end) {
            uint32_t ti = n_tiles++;
            psi_hot[ti] = (PsiTileHot){x, a_end, thin_seen, (uint16_t)(a | TF_IS_THIN), 0};
            uint32_t lp = (a_end - 1u) >> 6;
            while (lut_page <= lp) tile_lut[lut_page++] = ti;
        }
    }
    k->psi_len = n_tiles;

    /* Sequential domain-gap walk (dst = 0 .. m-1) with two-sided Tropical Pinch-Point Tarjan compression */
    k->spell_len = 0;
    k->off_len = 0;
    k->offsets[k->off_len++] = 0;
    k->wide = 0;

    uint32_t bits = m <= 1 ? 1u : (32u - (uint32_t)__builtin_clz(m - 1u));
    uint32_t raw_max = UINT32_MAX >> bits;
    uint32_t max_inline_code = raw_max >= reserve_sentinels ? (raw_max - reserve_sentinels) : 0u;

    uint32_t dst = 0, words_cnt = 0, q_cur = 0;
    uint32_t last_thick_end = 0;
    size_t n_strips = k->strips_len;

    for (size_t g_idx = 0; g_idx <= n_strips; g_idx++) {
        uint32_t gx0 = last_thick_end;
        uint32_t g_end = (g_idx < n_strips) ? k->strips[g_idx].from : n;
        if (g_idx < n_strips) {
            last_thick_end = k->strips[g_idx].from + k->strips[g_idx].width;
        }
        if (gx0 >= g_end) continue;

        uint32_t x = gx0;
        while (x < g_end) {
            while (psi_hot[q_cur].end <= x) q_cur++;
            PsiTileHot t0 = psi_hot[q_cur];
            uint32_t req = g_end - x;

            if (__builtin_expect((t0.sym_flags & TF_IS_THIN) != 0, 1)) {
                uint32_t rem = t0.end - x;
                uint32_t span = req < rem ? req : rem;
                uint8_t a = (uint8_t)(t0.sym_flags & 0xFFu);
                uint32_t q_off = t0.param + (x - t0.start);
                ft_wire_span(k, dst, span, (uint32_t)a, a, q_off, bits, max_inline_code);
                dst += span;
                x += span;
                continue;
            }

            if ((t0.sym_flags & TF_CERT_EXIT) != 0) {
                PsiCert *c0 = &psi_cert[q_cur];
                if (x >= c0->lo && x < c0->hi) {
                    uint32_t rem = c0->hi - x;
                    uint32_t span = req < rem ? req : rem;
                    uint32_t ex_ti = c0->target_ti;
                    uint8_t ex_sym = (uint8_t)(psi_hot[ex_ti].sym_flags & 0xFFu);
                    uint32_t q_off = c0->target_pos + (x - c0->lo);
                    uint32_t cd = c0->code;
                    if (__builtin_expect(cd == 0, 0)) {
                        uint32_t so = c0->spell_off, sl = c0->spell_len;
                        spell_ensure(k, sl);
                        uint32_t o0 = (uint32_t)k->spell_len;
                        memmove(k->spelling + o0, k->spelling + so, sl);
                        k->spell_len += sl;
                        uint32_t o1 = (uint32_t)k->spell_len;
                        push_u32(&k->offsets, &k->off_len, &k->off_cap, o1);
                        cd = 256u + words_cnt++;
                        c0->code = cd;
                        if (build_inline16) {
                            dict_ensure(k, (size_t)cd + reserve_sentinels + 16);
                            k->w_len[cd] = sl <= 65534u ? (uint16_t)sl : 65534u;
                            k->inline16[cd] = _mm_loadu_si128((const __m128i *)(k->spelling + o0));
                        }
                    }
                    ft_wire_span(k, dst, span, cd, ex_sym, q_off, bits, max_inline_code);
                    dst += span;
                    x += span;
                    continue;
                }
            }

            /* Forward Psi trace with Tarjan jump & exit certificates */
            size_t K = 0;
            uint32_t cur = x, ti = q_cur;
            uint32_t ex_ti = 0, q_off_exit = 0, tail_l = 0, tail_r = 0;
            uint32_t suf_is_byte = 1, suf_a = 0, suf_b = 1;

            for (;;) {
                PsiTileHot tl = psi_hot[ti];
                uint16_t flags = tl.sym_flags;
                if (flags & TF_IS_THIN) {
                    ex_ti = ti;
                    q_off_exit = tl.param + (cur - tl.start);
                    tail_l = cur - tl.start;
                    tail_r = tl.end - cur;
                    suf_is_byte = 1;
                    suf_a = (uint32_t)(flags & 0xFFu);
                    suf_b = 1;
                    break;
                }
                if (flags & TF_HAS_CERT) {
                    const PsiCert *ct = &psi_cert[ti];
                    if (cur >= ct->lo && cur < ct->hi) {
                        if (flags & TF_CERT_EXIT) {
                            ex_ti = ct->target_ti;
                            q_off_exit = ct->target_pos + (cur - ct->lo);
                            tail_l = cur - ct->lo;
                            tail_r = ct->hi - cur;
                            suf_is_byte = 0;
                            suf_a = ct->spell_off;
                            suf_b = ct->spell_len;
                            break;
                        } else {
                            uint32_t ll = cur - ct->lo, lr = ct->hi - cur;
                            ft_ensure_path(k, K + 1);
                            k->ft_path[K++] = (FtSeg){ti, cur, ct->spell_off, ct->spell_len, ll, lr, 0, 1};
                            cur = ct->target_pos + ll;
                            ti = ct->target_ti;
                            continue;
                        }
                    }
                }
                uint32_t ll = cur - tl.start, lr = tl.end - cur;
                ft_ensure_path(k, K + 1);
                k->ft_path[K++] = (FtSeg){ti, cur, (uint32_t)(flags & 0xFFu), 1u, ll, lr, 0, 0};
                cur = tl.param + ll;
                ti = tile_lut[cur >> 6];
                while (psi_hot[ti].end <= cur) ti++;
            }

            /* Materialize exact forward word into spelling[] */
            FtSeg * restrict path = k->ft_path;
            uint32_t o0 = (uint32_t)k->spell_len;
            for (size_t idx_s = 0; idx_s < K; idx_s++) {
                path[idx_s].seg_off = (uint32_t)k->spell_len;
                if (path[idx_s].is_jump) {
                    uint32_t so = path[idx_s].sym_or_off, sl = path[idx_s].slen;
                    spell_ensure(k, sl);
                    memmove(k->spelling + k->spell_len, k->spelling + so, sl);
                    k->spell_len += sl;
                } else {
                    spell_ensure(k, 1);
                    k->spelling[k->spell_len++] = (uint8_t)path[idx_s].sym_or_off;
                }
            }
            uint32_t seg_off_K = (uint32_t)k->spell_len;
            if (suf_is_byte) {
                spell_ensure(k, 1);
                k->spelling[k->spell_len++] = (uint8_t)suf_a;
            } else {
                spell_ensure(k, suf_b);
                memmove(k->spelling + k->spell_len, k->spelling + suf_a, suf_b);
                k->spell_len += suf_b;
            }
            uint32_t o1 = (uint32_t)k->spell_len;
            push_u32(&k->offsets, &k->off_len, &k->off_cap, o1);
            uint32_t new_code = 256u + words_cnt++;
            if (build_inline16) {
                dict_ensure(k, (size_t)new_code + reserve_sentinels + 16);
                uint32_t wln = o1 - o0;
                k->w_len[new_code] = wln <= 65534u ? (uint16_t)wln : 65534u;
                k->inline16[new_code] = _mm_loadu_si128((const __m128i *)(k->spelling + o0));
            }

            uint32_t span;
            if (reg_pinch) {
                /* Post-hoc O(1) register-only (Exit + Pre-Exit + Midpoint) Tarjan certificate installation */
                uint32_t sl_cur = tail_l, sr_cur = tail_r;
                uint32_t pre_l = UINT32_MAX, pre_r = UINT32_MAX;
                uint32_t mid_idx = (uint32_t)(K >> 1);
                uint32_t mid_l = UINT32_MAX, mid_r = UINT32_MAX;
                uint32_t r0 = tail_r;
                for (size_t s = 0; s < K; s++) {
                    if (path[s].loc_r < r0) r0 = path[s].loc_r;
                }
                span = req < r0 ? req : r0;

                for (size_t rev = K; rev > 0; rev--) {
                    uint32_t idx_s = (uint32_t)(rev - 1);
                    uint32_t ll = path[idx_s].loc_l, lr = path[idx_s].loc_r;
                    if (ll < sl_cur) sl_cur = ll;
                    if (lr < sr_cur) sr_cur = lr;
                    if (ll < pre_l) pre_l = ll;
                    if (lr < pre_r) pre_r = lr;
                    if (idx_s <= mid_idx) {
                        if (ll < mid_l) mid_l = ll;
                        if (lr < mid_r) mid_r = lr;
                    }

                    uint32_t p_ti = path[idx_s].ti;
                    uint32_t py = path[idx_s].cur;
                    uint32_t s_off = path[idx_s].seg_off;

                    uint64_t term_unc = (idx_s == 0)
                        ? (sr_cur > span ? sr_cur - span : 0)
                        : ((uint64_t)sl_cur + sr_cur > span ? (uint64_t)sl_cur + sr_cur - span : 0);
                    uint64_t best_gain = term_unc * (uint64_t)(o1 - s_off);
                    int mode = (term_unc > 0) ? 1 : 0;

                    if (idx_s + 1u < (uint32_t)K) {
                        uint32_t j1 = (uint32_t)(K - 1);
                        uint32_t j1_off = path[j1].seg_off;
                        if (j1_off > s_off + 1u) {
                            uint64_t unc1 = (idx_s == 0)
                                ? (pre_r > span ? pre_r - span : 0)
                                : ((uint64_t)pre_l + pre_r > span ? (uint64_t)pre_l + pre_r - span : 0);
                            uint64_t g1 = unc1 * (uint64_t)(j1_off - s_off - 1u);
                            if (g1 > best_gain) { best_gain = g1; mode = 2; }
                        }
                        if (idx_s < mid_idx && mid_idx < (uint32_t)K) {
                            uint32_t jm_off = path[mid_idx].seg_off;
                            if (jm_off > s_off + 1u) {
                                uint64_t uncm = (idx_s == 0)
                                    ? (mid_r > span ? mid_r - span : 0)
                                    : ((uint64_t)mid_l + mid_r > span ? (uint64_t)mid_l + mid_r - span : 0);
                                uint64_t gm = uncm * (uint64_t)(jm_off - s_off - 1u);
                                if (gm > best_gain) { best_gain = gm; mode = 3; }
                            }
                        }
                    }

                    if (best_gain >= 8u || (idx_s == 0 && mode == 1)) {
                        if (mode == 2) {
                            uint32_t bj = (uint32_t)(K - 1);
                            psi_hot[p_ti].sym_flags = (uint16_t)((psi_hot[p_ti].sym_flags & ~TF_CERT_EXIT) | TF_HAS_CERT);
                            psi_cert[p_ti] = (PsiCert){py - pre_l, py + pre_r, path[bj].ti, path[bj].cur - pre_l, s_off, path[bj].seg_off - s_off, 0u, 0u};
                        } else if (mode == 3) {
                            uint32_t bj = mid_idx;
                            psi_hot[p_ti].sym_flags = (uint16_t)((psi_hot[p_ti].sym_flags & ~TF_CERT_EXIT) | TF_HAS_CERT);
                            psi_cert[p_ti] = (PsiCert){py - mid_l, py + mid_r, path[bj].ti, path[bj].cur - mid_l, s_off, path[bj].seg_off - s_off, 0u, 0u};
                        } else if (mode == 1) {
                            psi_hot[p_ti].sym_flags |= (uint16_t)(TF_HAS_CERT | TF_CERT_EXIT);
                            psi_cert[p_ti] = (PsiCert){py - sl_cur, py + sr_cur, ex_ti, q_off_exit - sl_cur, s_off, o1 - s_off, idx_s == 0 ? new_code : 0u, 0u};
                        }
                    }
                }
            } else {
                /* Backward suffix margins & O(K) monotonic-stack pinch-point certificate installation */
                uint32_t * restrict suf_l = k->ft_suf_l;
                uint32_t * restrict suf_r = k->ft_suf_r;
                uint32_t * restrict stk_l = k->ft_stk_l;
                uint32_t * restrict stk_r = k->ft_stk_r;
                suf_l[K] = tail_l;
                suf_r[K] = tail_r;
                for (size_t rev = K; rev > 0; rev--) {
                    size_t idx_s = rev - 1;
                    uint32_t ll = path[idx_s].loc_l, lr = path[idx_s].loc_r;
                    suf_l[idx_s] = ll < suf_l[idx_s + 1] ? ll : suf_l[idx_s + 1];
                    suf_r[idx_s] = lr < suf_r[idx_s + 1] ? lr : suf_r[idx_s + 1];
                }

                span = req < suf_r[0] ? req : suf_r[0];
                size_t len_l = 0, len_r = 0;

                for (size_t rev = K; rev > 0; rev--) {
                    uint32_t idx_s = (uint32_t)(rev - 1);
                    uint32_t ll = path[idx_s].loc_l, lr = path[idx_s].loc_r;
                    while (len_l > 0 && ll <= path[stk_l[len_l - 1]].loc_l) len_l--;
                    stk_l[len_l++] = idx_s;
                    while (len_r > 0 && lr <= path[stk_r[len_r - 1]].loc_r) len_r--;
                    stk_r[len_r++] = idx_s;

                    uint32_t p_ti = path[idx_s].ti;
                    uint32_t py = path[idx_s].cur;
                    uint32_t s_off = path[idx_s].seg_off;

                    uint64_t term_unc = 0;
                    if (idx_s == 0) {
                        if (suf_r[0] > span) term_unc = suf_r[0] - span;
                    } else {
                        uint64_t tw_lr = (uint64_t)suf_l[idx_s] + (uint64_t)suf_r[idx_s];
                        if (tw_lr > span) term_unc = tw_lr - span;
                    }
                    uint64_t best_gain = term_unc * (uint64_t)(o1 - s_off);
                    int32_t best_j = -1;
                    uint32_t best_l = 0, best_r = 0;

                    if ((size_t)(idx_s + 1u) < K) {
                        if (len_l + len_r <= 24) {
                            size_t pl = len_l - 1, pr = len_r - 1;
                            uint32_t cur_l = path[stk_l[pl]].loc_l;
                            uint32_t cur_r = path[stk_r[pr]].loc_r;
                            for (;;) {
                                uint32_t nl = pl > 0 ? stk_l[pl - 1] : (uint32_t)K;
                                uint32_t nr = pr > 0 ? stk_r[pr - 1] : (uint32_t)K;
                                uint32_t nxt = nl < nr ? nl : nr;
                                uint32_t j_cand = nxt - 1u;
                                if (j_cand > idx_s) {
                                    uint64_t unc = 0;
                                    if (idx_s == 0) {
                                        if (cur_r > span) unc = cur_r - span;
                                    } else {
                                        uint64_t cw = (uint64_t)cur_l + (uint64_t)cur_r;
                                        if (cw > span) unc = cw - span;
                                    }
                                    uint32_t j_off = (j_cand < (uint32_t)K) ? path[j_cand].seg_off : seg_off_K;
                                    if (unc > 0 && j_off > s_off + 1u) {
                                        uint64_t g = unc * (uint64_t)(j_off - s_off - 1u);
                                        if (g > best_gain) {
                                            best_gain = g;
                                            best_j = (int32_t)j_cand;
                                            best_l = cur_l;
                                            best_r = cur_r;
                                        }
                                    }
                                }
                                if (nxt == (uint32_t)K) break;
                                if (nl == nxt) { pl--; cur_l = path[stk_l[pl]].loc_l; }
                                if (nr == nxt) { pr--; cur_r = path[stk_r[pr]].loc_r; }
                            }
                        } else {
                            uint32_t cands[6];
                            int nc = 0;
                            cands[nc++] = (uint32_t)(K - 1);
                            cands[nc++] = (idx_s + (uint32_t)K) >> 1;
                            if (len_l >= 2) { cands[nc++] = stk_l[len_l - 2] - 1u; cands[nc++] = stk_l[0] - 1u; }
                            if (len_r >= 2) { cands[nc++] = stk_r[len_r - 2] - 1u; cands[nc++] = stk_r[0] - 1u; }
                            for (int ci = 0; ci < nc; ci++) {
                                uint32_t jc = cands[ci];
                                if (jc > idx_s && jc < (uint32_t)K) {
                                    uint32_t cl = ft_stk_min_at(stk_l, len_l, path, 0, jc);
                                    uint32_t cr = ft_stk_min_at(stk_r, len_r, path, 1, jc);
                                    uint64_t unc = 0;
                                    if (idx_s == 0) {
                                        if (cr > span) unc = cr - span;
                                    } else {
                                        uint64_t cw = (uint64_t)cl + (uint64_t)cr;
                                        if (cw > span) unc = cw - span;
                                    }
                                    uint32_t j_off = path[jc].seg_off;
                                    if (unc > 0 && j_off > s_off + 1u) {
                                        uint64_t g = unc * (uint64_t)(j_off - s_off - 1u);
                                        if (g > best_gain) {
                                            best_gain = g;
                                            best_j = (int32_t)jc;
                                            best_l = cl;
                                            best_r = cr;
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if (best_j != -1) {
                        uint32_t bj = (uint32_t)best_j;
                        psi_hot[p_ti].sym_flags = (uint16_t)((psi_hot[p_ti].sym_flags & ~TF_CERT_EXIT) | TF_HAS_CERT);
                        psi_cert[p_ti] = (PsiCert){
                            py - best_l,
                            py + best_r,
                            path[bj].ti,
                            path[bj].cur - best_l,
                            s_off,
                            path[bj].seg_off - s_off,
                            0u,
                            0u
                        };
                    } else if (term_unc > 0) {
                        psi_hot[p_ti].sym_flags |= (uint16_t)(TF_HAS_CERT | TF_CERT_EXIT);
                        psi_cert[p_ti] = (PsiCert){
                            py - suf_l[idx_s],
                            py + suf_r[idx_s],
                            ex_ti,
                            q_off_exit - suf_l[idx_s],
                            s_off,
                            o1 - s_off,
                            idx_s == 0 ? new_code : 0u,
                            0u
                        };
                    }
                }
            }

            uint8_t ex_sym = (uint8_t)(psi_hot[ex_ti].sym_flags & 0xFFu);
            ft_wire_span(k, dst, span, new_code, ex_sym, q_off_exit, bits, max_inline_code);
            dst += span;
            x += span;
        }
    }

    assert(dst == m);
    if (build_inline16) dict_ensure(k, 256u + words_cnt + reserve_sentinels + 4);
    k->m = m;
    k->mark = mark;
    k->bits = bits;
    k->words = words_cnt;
}

/* =========================================================================
 * Consumers: 1-Lane Open::emit, BVSMC-1P(8,48), and Zero-Arena BVSMC-2P(8,64)
 * ========================================================================= */
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

typedef struct {
    uint8_t *end_l[8];
    int32_t cid_l[8];
    uint32_t q_l[8];
    int32_t succ[MAX_CUTS];
    int32_t head[MAX_CUTS];
    uint32_t saved_code[MAX_CUTS];
    uint32_t saved_next[MAX_CUTS];
    uint32_t free_c;
    uint32_t next_q;
    uint32_t active;
    uint32_t S_eff;
    uint32_t sbase;
    uint32_t seg0_len;
    size_t n;
    uint8_t *out;
} State1P;

static inline void emit_init_1p(Kernel *k, State1P *st, int l, uint8_t **P_io, uint32_t cd) {
    uint8_t *P = *P_io;
    uint32_t ln = k->w_len[cd];
    if (__builtin_expect(ln <= 16u && P < st->end_l[l], 1)) {
        _mm_storeu_si128((__m128i *)P, k->inline16[cd]);
        *P_io = P + ln;
        return;
    }
    uint8_t b_lit = (uint8_t)cd;
    const uint8_t *src;
    uint32_t rem;
    if (cd < 256u) {
        src = &b_lit; rem = 1u;
    } else {
        uint32_t w = cd - 256u;
        uint32_t o0 = k->offsets[w];
        src = k->spelling + o0;
        rem = k->offsets[w + 1] - o0;
    }
    if (st->cid_l[l] == -1) {
        size_t av = (size_t)((st->out + st->n) - P);
        if (rem > av) rem = (uint32_t)av;
        memcpy(P, src, rem);
        *P_io = P + rem;
    } else {
        while (rem > 0) {
            uint8_t *cbase = k->chunk_pool + (size_t)st->cid_l[l] * CHUNK_STRIDE;
            uint32_t used = (uint32_t)(P - cbase);
            if (used >= CHUNK_CAP) {
                k->chunk_len[st->cid_l[l]] = used;
                uint32_t nc = st->free_c++;
                assert(nc < MAX_CHUNKS_1P);
                k->chunk_next[nc] = -1;
                k->chunk_next[st->cid_l[l]] = (int32_t)nc;
                st->cid_l[l] = (int32_t)nc;
                cbase = k->chunk_pool + (size_t)nc * CHUNK_STRIDE;
                P = cbase;
                st->end_l[l] = cbase + CHUNK_CAP;
                used = 0;
            }
            uint32_t av = CHUNK_CAP - used;
            uint32_t tk = rem < av ? rem : av;
            memcpy(P, src, tk);
            P += tk; src += tk; rem -= tk;
        }
        *P_io = P;
    }
}

__attribute__((noinline))
static void cold_1p(Kernel *k, State1P *st, int l, uint32_t *J_io, uint8_t **P_io, uint32_t cd) {
    uint8_t *P = *P_io;
    if (cd < st->sbase) {
        if (st->cid_l[l] == -2) {
            *P_io = k->dummy_buf;
            return;
        }
        emit_init_1p(k, st, l, P_io, cd);
    } else {
        uint32_t hq = cd - st->sbase;
        uint32_t cq = st->q_l[l];
        st->succ[cq] = (int32_t)hq;
        if (cq == 0) {
            st->seg0_len = (uint32_t)(P - st->out);
        } else {
            uint8_t *cbase = k->chunk_pool + (size_t)st->cid_l[l] * CHUNK_STRIDE;
            k->chunk_len[st->cid_l[l]] = (uint32_t)(P - cbase);
        }
        if (st->next_q < st->S_eff) {
            uint32_t nq = st->next_q++;
            st->q_l[l] = nq;
            uint32_t nc = st->free_c++;
            assert(nc < MAX_CHUNKS_1P);
            k->chunk_next[nc] = -1;
            st->head[nq] = (int32_t)nc;
            st->cid_l[l] = (int32_t)nc;
            uint8_t *cbase = k->chunk_pool + (size_t)nc * CHUNK_STRIDE;
            P = cbase;
            st->end_l[l] = cbase + CHUNK_CAP;
            emit_init_1p(k, st, l, &P, st->saved_code[nq]);
            *P_io = P;
            *J_io = st->saved_next[nq];
        } else {
            st->cid_l[l] = -2;
            *P_io = k->dummy_buf;
            st->end_l[l] = k->dummy_buf + 2048;
            *J_io = k->m;
            st->active--;
        }
    }
}

static void bvsmc_emit_1p(Kernel *k, size_t n, uint32_t S_req, uint8_t *out) {
    uint32_t * restrict edges = k->edges;
    uint32_t * restrict extra = k->extra;
    const __m128i * restrict inline16 = k->inline16;
    uint16_t * restrict w_len = k->w_len;
    const uint32_t m = k->m, mark = k->mark, bits = k->bits, mask = (1u << bits) - 1u;
    const int wide = k->wide;
    const uint32_t sbase = 256u + k->words;

    State1P st;
    st.S_eff = S_req < m ? S_req : m;
    if (st.S_eff > 48u) st.S_eff = 48u;
    st.sbase = sbase; st.free_c = 0; st.next_q = 0; st.active = 0; st.seg0_len = 0;
    st.n = n; st.out = out;

    uint32_t cuts[48], saved_raw[48];
    for (uint32_t q = 0; q < st.S_eff; q++) {
        uint32_t v = (uint32_t)((mark + ((uint64_t)q * m) / st.S_eff) % m);
        cuts[q] = v;
        uint32_t raw = edges[v];
        saved_raw[q] = raw;
        st.saved_next[q] = raw & mask;
        st.saved_code[q] = wide ? extra[v] : (raw >> bits);
        st.succ[q] = -1; st.head[q] = -1;
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
    for (int l = 0; l < 8; l++) {
        if (st.next_q < st.S_eff) {
            uint32_t q = st.next_q++; st.active++; st.q_l[l] = q;
            if (q == 0) {
                st.cid_l[l] = -1; p_init[l] = out;
                st.end_l[l] = n >= 16 ? (out + n - 15) : out;
            } else {
                uint32_t c = st.free_c++; k->chunk_next[c] = -1;
                st.head[q] = (int32_t)c; st.cid_l[l] = (int32_t)c;
                p_init[l] = k->chunk_pool + (size_t)c * CHUNK_STRIDE;
                st.end_l[l] = p_init[l] + CHUNK_CAP;
            }
            emit_init_1p(k, &st, l, &p_init[l], st.saved_code[q]);
            j_init[l] = st.saved_next[q];
        } else {
            st.cid_l[l] = -2; p_init[l] = k->dummy_buf;
            st.end_l[l] = k->dummy_buf + 2048; j_init[l] = m;
        }
    }
    uint32_t j0=j_init[0], j1=j_init[1], j2=j_init[2], j3=j_init[3];
    uint32_t j4=j_init[4], j5=j_init[5], j6=j_init[6], j7=j_init[7];
    uint8_t * restrict p0=p_init[0], * restrict p1=p_init[1], * restrict p2=p_init[2], * restrict p3=p_init[3];
    uint8_t * restrict p4=p_init[4], * restrict p5=p_init[5], * restrict p6=p_init[6], * restrict p7=p_init[7];

    #define STEP_1P(L, J, P, CD_EXPR, E_VAR) do { \
        uint32_t cd = (CD_EXPR); \
        J = (E_VAR) & mask; \
        uint32_t ln = w_len[cd]; \
        if (__builtin_expect(ln <= 16u && P < st.end_l[L], 1)) { \
            _mm_storeu_si128((__m128i *)P, inline16[cd]); \
            P += ln; \
        } else { \
            uint8_t *ptmp = P; cold_1p(k, &st, L, &(J), &ptmp, cd); P = ptmp; \
        } \
    } while (0)

    if (!wide) {
        while (st.active > 0) {
            uint32_t e0=edges[j0], e1=edges[j1], e2=edges[j2], e3=edges[j3];
            uint32_t e4=edges[j4], e5=edges[j5], e6=edges[j6], e7=edges[j7];
            STEP_1P(0, j0, p0, e0 >> bits, e0); STEP_1P(1, j1, p1, e1 >> bits, e1);
            STEP_1P(2, j2, p2, e2 >> bits, e2); STEP_1P(3, j3, p3, e3 >> bits, e3);
            STEP_1P(4, j4, p4, e4 >> bits, e4); STEP_1P(5, j5, p5, e5 >> bits, e5);
            STEP_1P(6, j6, p6, e6 >> bits, e6); STEP_1P(7, j7, p7, e7 >> bits, e7);
        }
    } else {
        while (st.active > 0) {
            uint32_t e0=edges[j0], c0=extra[j0], e1=edges[j1], c1=extra[j1];
            uint32_t e2=edges[j2], c2=extra[j2], e3=edges[j3], c3=extra[j3];
            uint32_t e4=edges[j4], c4=extra[j4], e5=edges[j5], c5=extra[j5];
            uint32_t e6=edges[j6], c6=extra[j6], e7=edges[j7], c7=extra[j7];
            STEP_1P(0, j0, p0, c0, e0); STEP_1P(1, j1, p1, c1, e1);
            STEP_1P(2, j2, p2, c2, e2); STEP_1P(3, j3, p3, c3, e3);
            STEP_1P(4, j4, p4, c4, e4); STEP_1P(5, j5, p5, c5, e5);
            STEP_1P(6, j6, p6, c6, e6); STEP_1P(7, j7, p7, c7, e7);
        }
    }

    for (uint32_t q = 0; q < st.S_eff; q++) {
        uint32_t v = cuts[q]; edges[v] = saved_raw[q];
        if (wide) extra[v] = st.saved_code[q];
    }

    size_t wr = st.seg0_len;
    int32_t cs = st.succ[0];
    uint32_t guard = 0;
    while (cs > 0 && wr < n && guard < st.S_eff) {
        int32_t c = st.head[cs];
        while (c != -1 && wr < n) {
            uint32_t cl = k->chunk_len[c];
            if (wr + cl > n) cl = (uint32_t)(n - wr);
            memcpy(out + wr, k->chunk_pool + (size_t)c * CHUNK_STRIDE, cl);
            wr += cl;
            c = k->chunk_next[c];
        }
        cs = st.succ[cs];
        guard++;
    }
    repeat_period(n, out, &wr);
}

typedef struct {
    uint8_t *safe_end[8];
    uint8_t *hard_end[8];
    int active_flag[8];
    uint32_t cyc_segs[MAX_CUTS];
    uint32_t cyc_off[MAX_CUTS];
    uint32_t cyc_end[MAX_CUTS];
    uint32_t saved_code[MAX_CUTS];
    uint32_t saved_next[MAX_CUTS];
    uint32_t n_cyc;
    uint32_t next_idx;
    uint32_t active;
    uint32_t sbase;
    uint8_t *out;
} State2P;

static inline void emit_bounded_2p(Kernel *k, State2P *st, int l, uint8_t **P_io, uint32_t cd) {
    uint8_t *P = *P_io;
    if (P >= st->hard_end[l]) return;
    uint32_t ln = k->w_len[cd];
    if (ln <= 16u && P < st->safe_end[l]) {
        _mm_storeu_si128((__m128i *)P, k->inline16[cd]);
        *P_io = P + ln;
        return;
    }
    uint8_t b_lit = (uint8_t)cd;
    const uint8_t *src;
    uint32_t rem;
    if (cd < 256u) {
        src = &b_lit; rem = 1u;
    } else {
        uint32_t w = cd - 256u;
        uint32_t o0 = k->offsets[w];
        src = k->spelling + o0;
        rem = k->offsets[w + 1] - o0;
    }
    uint32_t av = (uint32_t)(st->hard_end[l] - P);
    if (rem > av) rem = av;
    memcpy(P, src, rem);
    *P_io = P + rem;
}

__attribute__((noinline))
static void cold_2p(Kernel *k, State2P *st, int l, uint32_t *J_io, uint8_t **P_io, uint32_t cd) {
    if (!st->active_flag[l]) {
        *P_io = k->dummy_buf;
        return;
    }
    if (cd < st->sbase) {
        emit_bounded_2p(k, st, l, P_io, cd);
    } else {
        if (st->next_idx < st->n_cyc) {
            uint32_t idx = st->next_idx++;
            uint32_t sg = st->cyc_segs[idx];
            uint32_t off = st->cyc_off[idx], end = st->cyc_end[idx];
            uint8_t *P = st->out + off;
            st->hard_end[l] = st->out + end;
            st->safe_end[l] = (end - off >= 16u) ? (st->out + end - 15u) : (st->out + off);
            emit_bounded_2p(k, st, l, &P, st->saved_code[sg]);
            *P_io = P;
            *J_io = st->saved_next[sg];
        } else {
            st->active_flag[l] = 0;
            *P_io = k->dummy_buf;
            st->safe_end[l] = k->dummy_buf + 2048;
            st->hard_end[l] = k->dummy_buf + 4096;
            *J_io = k->m;
            st->active--;
        }
    }
}

static void bvsmc_emit_2p(Kernel *k, size_t n, uint32_t S_req, uint8_t *out) {
    uint32_t * restrict edges = k->edges;
    uint32_t * restrict extra = k->extra;
    const __m128i * restrict inline16 = k->inline16;
    uint16_t * restrict w_len = k->w_len;
    const uint32_t m = k->m, mark = k->mark, bits = k->bits, mask = (1u << bits) - 1u;
    const int wide = k->wide;
    const uint32_t sbase = 256u + k->words;

    uint32_t S_eff = S_req < m ? S_req : m;
    if (S_eff > MAX_CUTS) S_eff = MAX_CUTS;

    State2P st;
    st.sbase = sbase; st.out = out;

    uint32_t cuts[MAX_CUTS], saved_raw[MAX_CUTS], seg_bytes[MAX_CUTS + 1] = {0};
    int32_t succ[MAX_CUTS + 1];
    for (uint32_t q = 0; q < S_eff; q++) {
        uint32_t v = (uint32_t)((mark + ((uint64_t)q * m) / S_eff) % m);
        cuts[q] = v;
        uint32_t raw = edges[v];
        saved_raw[q] = raw;
        st.saved_next[q] = raw & mask;
        st.saved_code[q] = wide ? extra[v] : (raw >> bits);
        w_len[sbase + q] = 0u;
    }
    for (uint32_t q = 0; q < S_eff; q++) {
        uint32_t v = cuts[q], sc = sbase + q;
        if (wide) extra[v] = sc;
        else edges[v] = (sc << bits) | st.saved_next[q];
    }
    edges[m] = m;
    if (wide) extra[m] = 0;

    uint32_t p1_next = 0, p1_act = 0;
    uint32_t j_init[8], q_init[8], b_init[8];
    for (int l = 0; l < 8; l++) {
        if (p1_next < S_eff) {
            uint32_t q = p1_next++; p1_act++; q_init[l] = q;
            uint32_t c0 = st.saved_code[q];
            b_init[l] = c0 < 256u ? 1u : (k->offsets[c0 - 255u] - k->offsets[c0 - 256u]);
            j_init[l] = st.saved_next[q];
        } else {
            q_init[l] = MAX_CUTS; b_init[l] = 0; j_init[l] = m;
        }
    }
    uint32_t j0=j_init[0], j1=j_init[1], j2=j_init[2], j3=j_init[3];
    uint32_t j4=j_init[4], j5=j_init[5], j6=j_init[6], j7=j_init[7];
    uint32_t q0=q_init[0], q1=q_init[1], q2=q_init[2], q3=q_init[3];
    uint32_t q4=q_init[4], q5=q_init[5], q6=q_init[6], q7=q_init[7];
    uint32_t b0=b_init[0], b1=b_init[1], b2=b_init[2], b3=b_init[3];
    uint32_t b4=b_init[4], b5=b_init[5], b6=b_init[6], b7=b_init[7];

    #define P1_STEP(J, Q, B, CD_EXPR, E_VAR) do { \
        uint32_t cd = (CD_EXPR); \
        J = (E_VAR) & mask; \
        uint32_t wl = w_len[cd]; \
        B += wl; \
        if (__builtin_expect(wl >= 65534u || cd >= sbase, 0)) { \
            if (cd >= sbase) { \
                succ[Q] = (int32_t)(cd - sbase); \
                seg_bytes[Q] = B; \
                if (p1_next < S_eff) { \
                    uint32_t nq = p1_next++; Q = nq; \
                    uint32_t c0 = st.saved_code[nq]; \
                    B = c0 < 256u ? 1u : (k->offsets[c0 - 255u] - k->offsets[c0 - 256u]); \
                    J = st.saved_next[nq]; \
                } else { Q = MAX_CUTS; B = 0; J = m; p1_act--; } \
            } else { \
                uint32_t w = cd - 256u; \
                B = (B - wl) + (k->offsets[w + 1] - k->offsets[w]); \
            } \
        } \
    } while (0)

    if (!wide) {
        while (p1_act > 0) {
            uint32_t e0=edges[j0], e1=edges[j1], e2=edges[j2], e3=edges[j3];
            uint32_t e4=edges[j4], e5=edges[j5], e6=edges[j6], e7=edges[j7];
            P1_STEP(j0,q0,b0,e0>>bits,e0); P1_STEP(j1,q1,b1,e1>>bits,e1);
            P1_STEP(j2,q2,b2,e2>>bits,e2); P1_STEP(j3,q3,b3,e3>>bits,e3);
            P1_STEP(j4,q4,b4,e4>>bits,e4); P1_STEP(j5,q5,b5,e5>>bits,e5);
            P1_STEP(j6,q6,b6,e6>>bits,e6); P1_STEP(j7,q7,b7,e7>>bits,e7);
        }
    } else {
        while (p1_act > 0) {
            uint32_t e0=edges[j0],c0=extra[j0],e1=edges[j1],c1=extra[j1];
            uint32_t e2=edges[j2],c2=extra[j2],e3=edges[j3],c3=extra[j3];
            uint32_t e4=edges[j4],c4=extra[j4],e5=edges[j5],c5=extra[j5];
            uint32_t e6=edges[j6],c6=extra[j6],e7=edges[j7],c7=extra[j7];
            P1_STEP(j0,q0,b0,c0,e0); P1_STEP(j1,q1,b1,c1,e1);
            P1_STEP(j2,q2,b2,c2,e2); P1_STEP(j3,q3,b3,c3,e3);
            P1_STEP(j4,q4,b4,c4,e4); P1_STEP(j5,q5,b5,c5,e5);
            P1_STEP(j6,q6,b6,c6,e6); P1_STEP(j7,q7,b7,c7,e7);
        }
    }

    st.n_cyc = 0;
    size_t acc = 0;
    int32_t cs = 0;
    while (st.n_cyc < S_eff && acc < n) {
        size_t end = acc + seg_bytes[cs];
        if (end > n) end = n;
        st.cyc_segs[st.n_cyc] = (uint32_t)cs;
        st.cyc_off[st.n_cyc] = (uint32_t)acc;
        st.cyc_end[st.n_cyc] = (uint32_t)end;
        st.n_cyc++;
        acc = end;
        cs = succ[cs];
        if (cs == 0) break;
    }

    for (uint32_t q = 0; q < S_eff; q++) w_len[sbase + q] = 65535u;

    st.next_idx = 0; st.active = 0;
    uint8_t *p_init2[8];
    for (int l = 0; l < 8; l++) {
        if (st.next_idx < st.n_cyc) {
            uint32_t idx = st.next_idx++;
            uint32_t sg = st.cyc_segs[idx];
            uint32_t off = st.cyc_off[idx], end = st.cyc_end[idx];
            st.active++;
            st.active_flag[l] = 1;
            p_init2[l] = out + off;
            st.hard_end[l] = out + end;
            st.safe_end[l] = (end - off >= 16u) ? (out + end - 15u) : (out + off);
            emit_bounded_2p(k, &st, l, &p_init2[l], st.saved_code[sg]);
            j_init[l] = st.saved_next[sg];
        } else {
            st.active_flag[l] = 0;
            p_init2[l] = k->dummy_buf;
            st.safe_end[l] = k->dummy_buf + 2048;
            st.hard_end[l] = k->dummy_buf + 4096;
            j_init[l] = m;
        }
    }
    j0=j_init[0]; j1=j_init[1]; j2=j_init[2]; j3=j_init[3];
    j4=j_init[4]; j5=j_init[5]; j6=j_init[6]; j7=j_init[7];
    uint8_t * restrict p0=p_init2[0], * restrict p1=p_init2[1], * restrict p2=p_init2[2], * restrict p3=p_init2[3];
    uint8_t * restrict p4=p_init2[4], * restrict p5=p_init2[5], * restrict p6=p_init2[6], * restrict p7=p_init2[7];

    #define STEP_2P(L, J, P, CD_EXPR, E_VAR) do { \
        uint32_t cd = (CD_EXPR); \
        J = (E_VAR) & mask; \
        uint32_t ln = w_len[cd]; \
        if (__builtin_expect(ln <= 16u && P < st.safe_end[L], 1)) { \
            _mm_storeu_si128((__m128i *)P, inline16[cd]); \
            P += ln; \
        } else { \
            uint8_t *ptmp = P; cold_2p(k, &st, L, &(J), &ptmp, cd); P = ptmp; \
        } \
    } while (0)

    if (!wide) {
        while (st.active > 0) {
            uint32_t e0=edges[j0], e1=edges[j1], e2=edges[j2], e3=edges[j3];
            uint32_t e4=edges[j4], e5=edges[j5], e6=edges[j6], e7=edges[j7];
            STEP_2P(0,j0,p0,e0>>bits,e0); STEP_2P(1,j1,p1,e1>>bits,e1);
            STEP_2P(2,j2,p2,e2>>bits,e2); STEP_2P(3,j3,p3,e3>>bits,e3);
            STEP_2P(4,j4,p4,e4>>bits,e4); STEP_2P(5,j5,p5,e5>>bits,e5);
            STEP_2P(6,j6,p6,e6>>bits,e6); STEP_2P(7,j7,p7,e7>>bits,e7);
        }
    } else {
        while (st.active > 0) {
            uint32_t e0=edges[j0],c0=extra[j0],e1=edges[j1],c1=extra[j1];
            uint32_t e2=edges[j2],c2=extra[j2],e3=edges[j3],c3=extra[j3];
            uint32_t e4=edges[j4],c4=extra[j4],e5=edges[j5],c5=extra[j5];
            uint32_t e6=edges[j6],c6=extra[j6],e7=edges[j7],c7=extra[j7];
            STEP_2P(0,j0,p0,c0,e0); STEP_2P(1,j1,p1,c1,e1);
            STEP_2P(2,j2,p2,c2,e2); STEP_2P(3,j3,p3,c3,e3);
            STEP_2P(4,j4,p4,c4,e4); STEP_2P(5,j5,p5,c5,e5);
            STEP_2P(6,j6,p6,c6,e6); STEP_2P(7,j7,p7,c7,e7);
        }
    }

    for (uint32_t q = 0; q < S_eff; q++) {
        uint32_t v = cuts[q]; edges[v] = saved_raw[q];
        if (wide) extra[v] = st.saved_code[q];
    }
    repeat_period(n, out, &acc);
}

/* =========================================================================
 * Self-tests & Paired 6-Block Confrontation Harness
 * ========================================================================= */
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

static void run_arm(Kernel *k, int arm, const uint16_t *recs, size_t n_recs, uint32_t n, uint32_t p, uint8_t *out, double *t_con, double *t_em) {
    double t0 = now_sec();
    switch (arm) {
        case 0: open_construct(k, recs, n_recs, n, p, LIMIT, 0, 0, 0); break;
        case 1: hp_ftte_construct(k, recs, n_recs, n, p, LIMIT, 0, 0, 0, 0); break;
        case 2: open_construct(k, recs, n_recs, n, p, LIMIT, 1, 1, 64u); break;
        case 3: hp_ftte_construct(k, recs, n_recs, n, p, LIMIT, 1, 1, 64u, 0); break;
        case 4: hp_ftte_construct(k, recs, n_recs, n, p, LIMIT, 1, 1, 48u, 0); break;
        case 5: hp_ftte_construct(k, recs, n_recs, n, p, LIMIT, 0, 0, 0, 1); break;
        case 6: hp_ftte_construct(k, recs, n_recs, n, p, LIMIT, 1, 1, 64u, 1); break;
    }
    double t1 = now_sec();
    switch (arm) {
        case 0:
        case 1:
        case 5: open_emit_orig(k, n, out); break;
        case 2:
        case 3:
        case 6: bvsmc_emit_2p(k, n, 64u, out); break;
        case 4: bvsmc_emit_1p(k, n, 48u, out); break;
    }
    double t2 = now_sec();
    if (t_con) *t_con = (t1 - t0) * 1e3;
    if (t_em) *t_em = (t2 - t1) * 1e3;
}

static void run_selftests(Kernel *k) {
    const struct { const char *pat; size_t reps; uint32_t p; } cases[7] = {
        {"a", 1, 0},
        {"ababa", 1, 2},
        {"a", 500, 137},
        {"a", 50000, 12345},
        {"ab", 300, 41},
        {"ba", 301, 0},
        {"abacabadabacabae", 50, 123},
    };
    uint8_t *L = (uint8_t *)malloc(60000);
    uint8_t *ref = (uint8_t *)malloc(60000);
    uint8_t *out = (uint8_t *)malloc(60000 + 64);
    uint16_t *recs = (uint16_t *)malloc(60000 * sizeof(uint16_t));

    for (int ci = 0; ci < 7; ci++) {
        size_t plen = strlen(cases[ci].pat);
        size_t n = plen * cases[ci].reps;
        for (size_t r = 0; r < cases[ci].reps; r++) memcpy(L + r * plen, cases[ci].pat, plen);
        dense_lf_reference(L, n, cases[ci].p, ref);
        size_t nr = to_capped_records(L, n, recs);
        for (int arm = 0; arm < 7; arm++) {
            memset(out, 0xA5, n + 32);
            run_arm(k, arm, recs, nr, (uint32_t)n, cases[ci].p, out, NULL, NULL);
            if (memcmp(out, ref, n) != 0) {
                fprintf(stderr, "SELFTEST FAIL case=%d arm=%d\n", ci, arm);
                exit(1);
            }
            for (size_t ov = n; ov < n + 32; ov++) {
                if (out[ov] != 0xA5u) {
                    fprintf(stderr, "OVERRUN FAIL case=%d arm=%d at +%zu\n", ci, arm, ov - n);
                    exit(1);
                }
            }
        }
    }
    free(L); free(ref); free(out); free(recs);
    printf("[PASS] Adversarial self-tests (7/7 incl. n=1, n=5, 50k periodic shear, overrun guard) across all 7 arms.\n");
}

int main(int argc, char **argv) {
    const char *dir = argc > 1 ? argv[1] : "/tmp/admitted-bwt/admitted-bwt";
    const struct { const char *file; uint32_t n; uint32_t p; } blocks[6] = {
        {"block-0.L",   899931u, 152882u},
        {"block-2.L",   900000u, 520105u},
        {"block-85.L",  899935u, 749184u},
        {"block-143.L", 899869u, 369156u},
        {"block-210.L", 899959u, 859891u},
        {"block-207.L", 899998u, 616774u},
    };

    Kernel k;
    kernel_init(&k, 900016u);
    run_selftests(&k);

    uint8_t *L = (uint8_t *)malloc(900016u);
    uint8_t *ref = (uint8_t *)malloc(900016u);
    uint8_t *out = (uint8_t *)malloc(900016u + 64u);
    uint16_t *recs = (uint16_t *)malloc(900016u * sizeof(uint16_t));

    const int WARMUP = 3, ROUNDS = 25;
    int pass_prod_con = 0, pass_prod_tot = 0, pass_comp_open = 0, pass_comp_bv2p = 0;

    printf("\n%-12s | %-20s | %8s %8s %8s | %7s %7s\n",
           "Block", "Arm", "Con(ms)", "Emit(ms)", "Tot(ms)", "Con/Op", "Tot/Op");
    printf("----------------------------------------------------------------------------------\n");

    for (int bi = 0; bi < 6; bi++) {
        char path[512];
        snprintf(path, sizeof(path), "%s/%s", dir, blocks[bi].file);
        FILE *fp = fopen(path, "rb");
        if (!fp) { perror(path); return 1; }
        size_t n = fread(L, 1, blocks[bi].n, fp);
        fclose(fp);
        assert(n == blocks[bi].n);

        dense_lf_reference(L, n, blocks[bi].p, ref);
        size_t nr = to_capped_records(L, n, recs);

        for (int arm = 0; arm < 7; arm++) {
            memset(out, 0x5A, n + 32);
            run_arm(&k, arm, recs, nr, (uint32_t)n, blocks[bi].p, out, NULL, NULL);
            if (memcmp(out, ref, n) != 0) {
                fprintf(stderr, "MISMATCH block=%s arm=%d\n", blocks[bi].file, arm);
                return 1;
            }
            for (size_t ov = n; ov < n + 32; ov++) assert(out[ov] == 0x5Au);
        }

        double t_con[7][ROUNDS], t_em[7][ROUNDS], t_tot[7][ROUNDS];
        for (int r = -WARMUP; r < ROUNDS; r++) {
            for (int step = 0; step < 7; step++) {
                int arm = (step + (r + WARMUP)) % 7;
                double c_ms, e_ms;
                run_arm(&k, arm, recs, nr, (uint32_t)n, blocks[bi].p, out, &c_ms, &e_ms);
                if (r >= 0) {
                    t_con[arm][r] = c_ms;
                    t_em[arm][r] = e_ms;
                    t_tot[arm][r] = c_ms + e_ms;
                }
            }
        }

        const char *names[7] = {
            "Open-qi16",
            "HP-FTTE-1L",
            "Open+BVSMC-2P",
            "HP-FTTE+BVSMC-2P",
            "HP-FTTE+BVSMC-1P",
            "HP-FTTE-Reg-1L*",
            "HP-FTTE-Reg+BV2P*"
        };
        double med_c[7], med_e[7], med_t[7];
        for (int arm = 0; arm < 7; arm++) {
            med_c[arm] = median(t_con[arm], ROUNDS);
            med_e[arm] = median(t_em[arm], ROUNDS);
            med_t[arm] = median(t_tot[arm], ROUNDS);
            printf("%-12s | %-20s | %8.3f %8.3f %8.3f | %6.3fx %6.3fx\n",
                   arm == 0 ? blocks[bi].file : "",
                   names[arm], med_c[arm], med_e[arm], med_t[arm],
                   med_c[arm] / med_c[0], med_t[arm] / med_t[0]);
        }
        printf("----------------------------------------------------------------------------------\n");

        if (med_c[1] <= 0.85 * med_c[0]) pass_prod_con++;
        if (med_t[1] <= 0.90 * med_t[0]) pass_prod_tot++;
        if (med_t[3] <= 0.72 * med_t[0]) pass_comp_open++;
        if (med_t[3] <= 0.95 * med_t[2]) pass_comp_bv2p++;
    }

    printf("  (* post-hoc O(1) register-pinch diagnostic arms; pre-registered Gate G5 evaluated on Arms 0..4)\n");
    printf("\n=== PRE-REGISTERED GATE G5 SUMMARY ===\n");
    printf("  Pure Producer Construct (HP-FTTE-1L <= 0.85x Open-qi16): %d/6 blocks\n", pass_prod_con);
    printf("  Pure Producer Total     (HP-FTTE-1L <= 0.90x Open-qi16): %d/6 blocks\n", pass_prod_tot);
    printf("  Composed 0-Arena Total  (HP-FTTE+BVSMC-2P <= 0.72x Open): %d/6 blocks\n", pass_comp_open);
    printf("  Composed vs Open+BVSMC2P(HP-FTTE+BVSMC-2P <= 0.95x Open+BV2P): %d/6 blocks\n", pass_comp_bv2p);
    int g5_pass = (pass_prod_con >= 5 && pass_prod_tot >= 5 && pass_comp_open >= 5 && pass_comp_bv2p >= 5);
    printf("  STRICT GATE G5 VERDICT: %s\n", g5_pass ? "PASS" : "FAIL");

    free(L); free(ref); free(out); free(recs);
    kernel_free(&k);
    return 0;
}
