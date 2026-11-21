// Gate G5 Final Single-Thread & 4-Worker Contended Confrontation (Wormhole Cycle 5):
//   C0: Open058   — Exact mibz2 0.5.8 baseline (256 Vec<u32> queues + 24B Node + 5 ord/term arrays + U16 emit)
//   W1: CPC-Arena — Theorem 5.1 I_0 Pigeonhole 1-Arena Queue + Theorem 5.2 Direct-Linked 16B Node16
//                   + Theorem 5.3 Full Chaitin Endpoint-in-edges Overlay + Fused Short-Inline16
//   W2: CPC-Micro — Kakutani Conjugate u16-Stream (0 B queues!) + Direct-Linked 16B Node16
//                   + Full Chaitin Endpoint-in-edges Overlay (1.15 MiB total scratch = 0.25x Open058!)

#define _POSIX_C_SOURCE 200809L
#include <assert.h>
#include <immintrin.h>
#include <pthread.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

static uint64_t now_ns(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (uint64_t)ts.tv_sec * 1000000000ULL + (uint64_t)ts.tv_nsec;
}

typedef struct {
    uint16_t *recs;  // length nrecs + 1 (sentinel at nrecs!)
    size_t nrecs;
    uint32_t counts[256];
    uint32_t per[256];
} RunsInput;

static void build_i0_runs(const uint8_t *L, size_t n, RunsInput *ri) {
    memset(ri->counts, 0, sizeof(ri->counts));
    memset(ri->per, 0, sizeof(ri->per));
    ri->recs = (uint16_t *)malloc((n + 2) * sizeof(uint16_t));
    size_t nr = 0, i = 0;
    while (i < n) {
        uint8_t c = L[i];
        size_t j = i + 1;
        while (j < n && L[j] == c) j++;
        ri->counts[c] += (uint32_t)(j - i);
        size_t left = j - i;
        while (left > 0) {
            size_t ln = left > 127 ? 127 : left;
            ri->recs[nr++] = (uint16_t)(((uint16_t)c << 8) | (uint16_t)ln);
            ri->per[c]++;
            left -= ln;
        }
        i = j;
    }
    ri->nrecs = nr;
    // Sentinel with length 0 and distinct byte from last record
    uint8_t last_c = nr ? (uint8_t)(ri->recs[nr - 1] >> 8) : 0;
    ri->recs[nr] = (uint16_t)((uint16_t)(uint8_t)(last_c ^ 0xFFu) << 8);
}

static void plain_invert(const uint8_t *L, size_t n, uint32_t p, uint8_t *out) {
    uint32_t counts[256] = {0}, ftab[256];
    for (size_t i = 0; i < n; i++) counts[L[i]]++;
    uint32_t acc = 0;
    for (int a = 0; a < 256; a++) { ftab[a] = acc; acc += counts[a]; }
    uint32_t *psi = (uint32_t *)malloc(n * sizeof(uint32_t));
    for (size_t i = 0; i < n; i++) psi[ftab[L[i]]++] = (uint32_t)i;
    uint32_t cur = psi[p];
    for (size_t t = 0; t < n; t++) { out[t] = L[cur]; cur = psi[cur]; }
    free(psi);
}

typedef struct { uint32_t from, to, width; uint8_t byte, pad[3]; } Strip;
typedef struct { uint32_t prev[2], next[2], width, word; } Node32;
// Theorem 5.2: 16-byte cache-line-aligned Node16 (4 nodes per 64B cache line!)
typedef struct { uint16_t prev[2], next[2]; uint32_t width, word; } Node16;
typedef struct { uint32_t left, right, len, first; } Rule;
typedef struct { uint32_t end, delta, code; } Band;

#define ENSURE_DBL(ptr, cap, need, T) do { \
    if (__builtin_expect((cap) < (need), 0)) { \
        size_t ncap = (cap) ? (cap) * 2 : 1024; \
        if (ncap < (need)) ncap = (need); \
        (ptr) = (T *)realloc((ptr), ncap * sizeof(T)); \
        (cap) = ncap; \
    } \
} while (0)

#define ENSURE_TIGHT(ptr, cap, need, T) do { \
    if (__builtin_expect((cap) < (need), 0)) { \
        size_t ncap = (need) + ((need) >> 3) + 64; \
        (ptr) = (T *)realloc((ptr), ncap * sizeof(T)); \
        (cap) = ncap; \
    } \
} while (0)

// ============================================================================
// C0: Exact mibz2 0.5.8 Baseline (Open<16, true, true>)
// ============================================================================
typedef struct {
    uint32_t *q_buf[256];
    uint32_t q_cap[256];
    Strip *strips;      size_t strips_cap;
    uint32_t *img_ord;  size_t img_cap;
    Node32 *nodes;      size_t nodes_cap;
    Rule *rules;        size_t rules_cap;
    uint32_t *ord0, *ord1, *term0, *term1, *bottom, *stack;
    size_t ord_cap, stk_cap;
    Band *bands;        size_t bands_cap;
    uint8_t *spelling;  size_t spell_cap;
    uint32_t *offsets;  size_t off_cap;
    uint32_t *edges;    size_t edges_cap;
    uint8_t (*inline16)[16]; size_t inl_cap;
    uint8_t *lens;
    int inl_init;
} Open058Scratch;

static void open058_init(Open058Scratch *s) { memset(s, 0, sizeof(*s)); }
static void open058_free(Open058Scratch *s) {
    for (int a = 0; a < 256; a++) free(s->q_buf[a]);
    free(s->strips); free(s->img_ord); free(s->nodes); free(s->rules);
    free(s->ord0); free(s->ord1); free(s->term0); free(s->term1);
    free(s->bottom); free(s->stack); free(s->bands); free(s->spelling);
    free(s->offsets); free(s->edges); free(s->inline16); free(s->lens);
}
static size_t open058_ws(const Open058Scratch *s) {
    size_t b = 0;
    for (int a = 0; a < 256; a++) b += s->q_cap[a] * 4;
    b += s->strips_cap * sizeof(Strip) + s->img_cap * 4;
    b += s->nodes_cap * sizeof(Node32) + s->rules_cap * sizeof(Rule);
    b += s->ord_cap * 20 + s->stk_cap * 4 + s->bands_cap * sizeof(Band);
    b += s->spell_cap + s->off_cap * 4 + s->edges_cap * 4 + s->inl_cap * 17;
    return b;
}

static inline void q058_push16(Open058Scratch *s, uint8_t a, uint32_t at, uint32_t m, uint32_t w) {
    uint32_t need = at + (w > 16 ? w : 16);
    if (__builtin_expect(s->q_cap[a] < need, 0)) {
        uint32_t ncap = s->q_cap[a] ? s->q_cap[a] * 2 : 1024;
        if (ncap < need) ncap = need;
        s->q_buf[a] = (uint32_t *)realloc(s->q_buf[a], ncap * 4);
        s->q_cap[a] = ncap;
    }
    uint32_t *dst = s->q_buf[a] + at;
    __m256i base = _mm256_set1_epi32((int)m);
    _mm256_storeu_si256((__m256i *)(dst + 0), _mm256_add_epi32(base, _mm256_setr_epi32(0,1,2,3,4,5,6,7)));
    _mm256_storeu_si256((__m256i *)(dst + 8), _mm256_add_epi32(base, _mm256_setr_epi32(8,9,10,11,12,13,14,15)));
}

static void run_open058(Open058Scratch *s, const RunsInput *ri, uint32_t n, uint32_t p, uint8_t *out) {
    const uint16_t *recs = ri->recs;
    const size_t nrecs = ri->nrecs;
    uint32_t all[256] = {0}, thin[256] = {0}, thick[256] = {0};
    uint32_t m = 0, mark = 0;
    size_t nstrips = 0;
    ENSURE_DBL(s->strips, s->strips_cap, 4096, Strip);

    uint32_t pos = 0;
    size_t i = 0;
    while (i < nrecs) {
        uint16_t rec = recs[i++];
        uint8_t c = (uint8_t)(rec >> 8);
        uint32_t w = (uint32_t)(rec & 255);
        while (i < nrecs && (uint8_t)(recs[i] >> 8) == c) { w += (uint32_t)(recs[i] & 255); i++; }
        if (w <= 16) {
            if (p - pos < w) mark = m + (p - pos);
            q058_push16(s, c, thin[c], m, w);
            thin[c] += w; m += w; all[c] += w;
        } else if (p - pos >= w) {
            ENSURE_DBL(s->strips, s->strips_cap, nstrips + 1, Strip);
            s->strips[nstrips++] = (Strip){pos, all[c], w, c, {0,0,0}};
            thick[c]++; all[c] += w;
        } else {
            ENSURE_DBL(s->strips, s->strips_cap, nstrips + 2, Strip);
            if (pos < p) { uint32_t lw = p - pos; s->strips[nstrips++] = (Strip){pos, all[c], lw, c, {0,0,0}}; thick[c]++; all[c] += lw; }
            mark = m; q058_push16(s, c, thin[c], m, 1); thin[c]++; m++; all[c]++;
            if (p + 1 < pos + w) { uint32_t rw = pos + w - p - 1; s->strips[nstrips++] = (Strip){p + 1, all[c], rw, c, {0,0,0}}; thick[c]++; all[c] += rw; }
        }
        pos += w;
    }

    uint32_t first[256], write[256], acc0 = 0, acc1 = 0;
    for (int a = 0; a < 256; a++) { first[a] = acc0; acc0 += all[a]; write[a] = acc1; acc1 += thick[a]; }
    ENSURE_DBL(s->img_ord, s->img_cap, nstrips + 1, uint32_t);
    for (size_t si = 0; si < nstrips; si++) { uint8_t a = s->strips[si].byte; s->strips[si].to += first[a]; s->img_ord[write[a]++] = (uint32_t)si; }

    size_t max_nodes = 3 * nstrips + 8;
    ENSURE_DBL(s->nodes, s->nodes_cap, max_nodes, Node32);
    if (s->ord_cap < max_nodes) {
        size_t nc = max_nodes * 2;
        s->ord0 = (uint32_t *)realloc(s->ord0, nc * 4); s->ord1 = (uint32_t *)realloc(s->ord1, nc * 4);
        s->term0 = (uint32_t *)realloc(s->term0, nc * 4); s->term1 = (uint32_t *)realloc(s->term1, nc * 4);
        s->bottom = (uint32_t *)realloc(s->bottom, nc * 4); s->ord_cap = nc;
    }
    Node32 *nd = s->nodes;
    nd[0] = (Node32){0};
    uint32_t nnodes = 1;
    size_t n_o0 = 0, n_o1 = 0, n_t0 = 0, n_t1 = 0;
    for (size_t si = 0; si < nstrips; si++) nd[nnodes++] = (Node32){{0,0},{0,0}, s->strips[si].width, (uint32_t)s->strips[si].byte + 1};
    uint32_t x = 0;
    for (size_t si = 0; si < nstrips; si++) {
        if (x < s->strips[si].from) { uint32_t id = nnodes++; nd[id] = (Node32){{0,0},{0,0}, s->strips[si].from - x, 0}; s->ord0[n_o0++] = id; s->term1[n_t1++] = id; }
        s->ord0[n_o0++] = (uint32_t)si + 1; x = s->strips[si].from + s->strips[si].width;
    }
    if (x < n) { uint32_t id = nnodes++; nd[id] = (Node32){{0,0},{0,0}, n - x, 0}; s->ord0[n_o0++] = id; s->term1[n_t1++] = id; }
    x = 0;
    for (size_t k = 0; k < nstrips; k++) {
        uint32_t si = s->img_ord[k];
        if (x < s->strips[si].to) { uint32_t id = nnodes++; nd[id] = (Node32){{0,0},{0,0}, s->strips[si].to - x, 0}; s->ord1[n_o1++] = id; s->term0[n_t0++] = id; }
        s->ord1[n_o1++] = si + 1; x = s->strips[si].to + s->strips[si].width;
    }
    if (x < n) { uint32_t id = nnodes++; nd[id] = (Node32){{0,0},{0,0}, n - x, 0}; s->ord1[n_o1++] = id; s->term0[n_t0++] = id; }

    uint32_t prv = 0;
    for (size_t k = 0; k < n_t0; k++) { uint32_t id = s->term0[k]; nd[prv].next[0] = id; nd[id].prev[0] = prv; prv = id; }
    for (size_t k = 0; k < n_o0; k++) { uint32_t id = s->ord0[k];  nd[prv].next[0] = id; nd[id].prev[0] = prv; prv = id; }
    nd[prv].next[0] = 0; nd[0].prev[0] = prv;
    prv = 0;
    for (size_t k = 0; k < n_t1; k++) { uint32_t id = s->term1[k]; nd[prv].next[1] = id; nd[id].prev[1] = prv; prv = id; }
    for (size_t k = 0; k < n_o1; k++) { uint32_t id = s->ord1[k];  nd[prv].next[1] = id; nd[id].prev[1] = prv; prv = id; }
    nd[prv].next[1] = 0; nd[0].prev[1] = prv;

    size_t nrules = 0;
    uint32_t rem = n;
    while (rem > 0) {
        uint32_t nx = nd[0].prev[0], ny = nd[0].prev[1];
        uint32_t aw = nd[nx].width, bw = nd[ny].width, w = aw < bw ? aw : bw;
        rem -= w;
        if (nx == ny) {
            nd[nd[nx].prev[0]].next[0] = nd[nx].next[0]; nd[nd[nx].next[0]].prev[0] = nd[nx].prev[0];
            nd[nd[nx].prev[1]].next[1] = nd[nx].next[1]; nd[nd[nx].next[1]].prev[1] = nd[nx].prev[1];
            continue;
        }
        uint32_t lw = nd[nx].word, rw = nd[ny].word, wj;
        if (lw == 0) wj = rw; else if (rw == 0) wj = lw;
        else {
            ENSURE_DBL(s->rules, s->rules_cap, nrules + 1, Rule);
            uint32_t ll = lw <= 256 ? 1 : s->rules[lw - 257].len;
            uint32_t rl = rw <= 256 ? 1 : s->rules[rw - 257].len;
            wj = (uint32_t)nrules + 257; s->rules[nrules++] = (Rule){lw, rw, ll + rl, UINT32_MAX};
        }
        if (aw <= bw) {
            nd[nd[nx].prev[0]].next[0] = nd[nx].next[0]; nd[nd[nx].next[0]].prev[0] = nd[nx].prev[0];
            uint32_t qn = nd[ny].next[0]; nd[nx].prev[0] = ny; nd[nx].next[0] = qn; nd[ny].next[0] = nx; nd[qn].prev[0] = nx;
            nd[nx].word = wj;
            if (aw == bw) {
                nd[nd[ny].prev[0]].next[0] = nd[ny].next[0]; nd[nd[ny].next[0]].prev[0] = nd[ny].prev[0];
                nd[nd[ny].prev[1]].next[1] = nd[ny].next[1]; nd[nd[ny].next[1]].prev[1] = nd[ny].prev[1];
            } else nd[ny].width -= w;
        } else {
            nd[nd[ny].prev[1]].next[1] = nd[ny].next[1]; nd[nd[ny].next[1]].prev[1] = nd[ny].prev[1];
            uint32_t qn = nd[nx].next[1]; nd[ny].prev[1] = nx; nd[ny].next[1] = qn; nd[nx].next[1] = ny; nd[qn].prev[1] = ny;
            nd[ny].word = wj; nd[nx].width -= w;
        }
    }

    uint32_t id = nd[0].next[1]; x = 0;
    while (id != 0) { s->bottom[id] = x; x += nd[id].width; id = nd[id].next[1]; }

    size_t nbands = 0, nspell = 0, noff = 1, nwords = 0;
    ENSURE_DBL(s->offsets, s->off_cap, 4096, uint32_t); s->offsets[0] = 0;
    ENSURE_DBL(s->bands, s->bands_cap, nnodes + 256, Band);
    ENSURE_DBL(s->spelling, s->spell_cap, 65536, uint8_t);
    ENSURE_DBL(s->stack, s->stk_cap, 1024, uint32_t);
    size_t cursors[257];
    id = nd[0].next[0];
    uint32_t in_node = 0, k = 0;
    for (int a = 0; a < 256; a++) {
        cursors[a] = nbands;
        uint32_t end = k + thin[a];
        while (k < end) {
            uint32_t nw = nd[id].width, width = (end - k) < (nw - in_node) ? (end - k) : (nw - in_node);
            uint32_t dst = s->bottom[id] + in_node, wd = nd[id].word, code;
            if (wd == 0) code = (uint32_t)a;
            else {
                nwords++;
                uint32_t h = wd <= 256 ? 1 : s->rules[wd - 257].len;
                ENSURE_DBL(s->spelling, s->spell_cap, nspell + h + 2, uint8_t);
                size_t sp = 0; s->stack[sp++] = wd;
                while (sp > 0) {
                    uint32_t cur = s->stack[--sp];
                    if (cur <= 256) s->spelling[nspell++] = (uint8_t)(cur - 1);
                    else {
                        Rule *rl = &s->rules[cur - 257];
                        if (rl->first == UINT32_MAX) { rl->first = (uint32_t)nspell; ENSURE_DBL(s->stack, s->stk_cap, sp + 2, uint32_t); s->stack[sp++] = rl->right; s->stack[sp++] = rl->left; }
                        else { memcpy(s->spelling + nspell, s->spelling + rl->first, rl->len); nspell += rl->len; }
                    }
                }
                s->spelling[nspell++] = (uint8_t)a;
                ENSURE_DBL(s->offsets, s->off_cap, noff + 1, uint32_t); s->offsets[noff++] = (uint32_t)nspell;
                code = 255 + (uint32_t)nwords;
            }
            ENSURE_DBL(s->bands, s->bands_cap, nbands + 1, Band);
            s->bands[nbands++] = (Band){k + width, dst - k, code};
            in_node += width; k += width;
            if (in_node == nw) { id = nd[id].next[0]; in_node = 0; }
        }
    }
    cursors[256] = nbands;

    uint32_t bits = m <= 1 ? 1 : (32u - (uint32_t)__builtin_clz(m - 1));
    ENSURE_DBL(s->edges, s->edges_cap, m, uint32_t);
    k = 0;
    for (int a = 0; a < 256; a++) {
        size_t from = 0; const uint32_t *qa = s->q_buf[a];
        for (size_t bi = cursors[a]; bi < cursors[a + 1]; bi++) {
            Band b = s->bands[bi]; uint32_t span = b.end - k, dst = k + b.delta, pack = b.code << bits;
            uint32_t *ed = s->edges + dst; const uint32_t *src = qa + from;
            for (uint32_t d = 0; d < span; d++) ed[d] = pack | src[d];
            from += span; k = b.end;
        }
    }

    size_t ninl = 256 + nwords;
    if (s->inl_cap < ninl) {
        size_t nc = ninl * 2;
        s->inline16 = (uint8_t (*)[16])realloc(s->inline16, nc * 16);
        s->lens = (uint8_t *)realloc(s->lens, nc);
        s->inl_cap = nc; s->inl_init = 0;
    }
    if (!s->inl_init) {
        for (int c = 0; c < 256; c++) { memset(s->inline16[c], 0, 16); s->inline16[c][0] = (uint8_t)c; s->lens[c] = 1; }
        s->inl_init = 1;
    }
    for (size_t w = 0; w < nwords; w++) {
        uint32_t o0 = s->offsets[w], wl = s->offsets[w + 1] - o0;
        memset(s->inline16[256 + w], 0, 16);
        memcpy(s->inline16[256 + w], s->spelling + o0, wl < 16 ? wl : 16);
        s->lens[256 + w] = wl <= 16 ? (uint8_t)wl : 255;
    }

    uint32_t mask = (1u << bits) - 1, j = mark;
    size_t at = 0;
    for (uint32_t step = 0; step < m; step++) {
        uint32_t edge = s->edges[j], code = edge >> bits; j = edge & mask; uint8_t len = s->lens[code];
        if (__builtin_expect(len != 255, 1)) {
            _mm_storeu_si128((__m128i *)(out + at), _mm_loadu_si128((const __m128i *)s->inline16[code]));
            at += len;
        } else {
            uint32_t w = code - 256, o0 = s->offsets[w], wl = s->offsets[w + 1] - o0;
            size_t take = wl < (n - at) ? wl : (n - at);
            memcpy(out + at, s->spelling + o0, take); at += take;
        }
        if (j == mark || at >= n) break;
    }
    if (at > n) at = n;
    while (at < n) { size_t take = at < (n - at) ? at : (n - at); memcpy(out + at, out, take); at += take; }
}

// ============================================================================
// Shared Direct-Linked 16B Node16 Endpoint inside Chaitin-Aliased edges[]!
//   Hosts strips + img_ord + nodes16 + rules + bottom 100% inside edges_overlay!
// ============================================================================
static void run_direct_node16_endpoint_aliased(
    const Strip *strips_in, size_t nstrips, const uint32_t all[256], const uint32_t thick[256],
    const uint32_t thin[256], uint32_t n, uint8_t *overlay_mem, size_t overlay_bytes,
    Band **p_bands, size_t *p_bands_cap,
    uint8_t **p_spell, size_t *p_spell_cap,
    uint32_t **p_off, size_t *p_off_cap,
    uint8_t (**p_inl)[16], uint8_t **p_lens, size_t *p_inl_cap, int *p_inl_init, int right_align,
    size_t cursors[257], size_t *out_nwords
) {
    // Layout inside overlay_mem (16B aligned):
    //   [0 .. nstrips*sizeof(Strip)): strips (already at start of overlay_mem!)
    //   next: Node16[max_nodes]  (16B * max_nodes)
    //   next: Rule[max_nodes]    (16B * max_nodes)
    //   next: bottom[max_nodes]  (4B * max_nodes)
    //   next: img_ord[nstrips]   (4B * nstrips)
    //   next: stack[512]         (2 KiB)
    size_t max_nodes = 3 * nstrips + 8;
    assert(max_nodes < 65535);
    size_t off_strips = ((nstrips + 2) * sizeof(Strip) + 15u) & ~15u;
    size_t off_nodes  = off_strips + max_nodes * sizeof(Node16);
    size_t off_rules  = off_nodes  + max_nodes * sizeof(Rule);
    size_t off_bot    = off_rules  + ((max_nodes * 4 + 15u) & ~15u);
    size_t off_img    = off_bot    + (((nstrips + 4) * 4 + 15u) & ~15u);
    size_t off_end    = off_img    + 2048;
    assert(off_end <= overlay_bytes);
    (void)strips_in;

    Strip    *strips  = (Strip *)overlay_mem;
    Node16   *nd      = (Node16 *)(overlay_mem + off_strips);
    Rule     *rules   = (Rule *)(overlay_mem + off_nodes);
    uint32_t *bottom  = (uint32_t *)(overlay_mem + off_rules);
    uint32_t *img_ord = (uint32_t *)(overlay_mem + off_bot);
    uint32_t *stk     = (uint32_t *)(overlay_mem + off_img);

    uint32_t first[256], write[256], acc0 = 0, acc1 = 0;
    for (int a = 0; a < 256; a++) { first[a] = acc0; acc0 += all[a]; write[a] = acc1; acc1 += thick[a]; }
    for (size_t si = 0; si < nstrips; si++) {
        uint8_t a = strips[si].byte;
        strips[si].to += first[a];
        img_ord[write[a]++] = (uint32_t)si;
    }

    nd[0] = (Node16){0};
    uint16_t nnodes = 1;
    for (size_t si = 0; si < nstrips; si++) {
        nd[nnodes++] = (Node16){{0,0},{0,0}, strips[si].width, (uint32_t)strips[si].byte + 1};
    }

    // Direct O(1) chain construction (zero ord0/1, zero term0/1 arrays!)
    uint16_t t0_h = 0, t0_t = 0, o0_h = 0, o0_t = 0;
    uint16_t t1_h = 0, t1_t = 0, o1_h = 0, o1_t = 0;
    #define APP(side, h, t, id) do { \
        uint16_t _id = (id); \
        if ((h) == 0) { nd[_id].prev[side] = 0; nd[_id].next[side] = 0; (h) = (t) = _id; } \
        else { nd[t].next[side] = _id; nd[_id].prev[side] = (t); nd[_id].next[side] = 0; (t) = _id; } \
    } while (0)

    uint32_t x = 0;
    for (size_t si = 0; si < nstrips; si++) {
        if (x < strips[si].from) {
            uint16_t id = nnodes++;
            nd[id] = (Node16){{0,0},{0,0}, strips[si].from - x, 0};
            APP(0, o0_h, o0_t, id); APP(1, t1_h, t1_t, id);
        }
        APP(0, o0_h, o0_t, (uint16_t)(si + 1));
        x = strips[si].from + strips[si].width;
    }
    if (x < n) {
        uint16_t id = nnodes++;
        nd[id] = (Node16){{0,0},{0,0}, n - x, 0};
        APP(0, o0_h, o0_t, id); APP(1, t1_h, t1_t, id);
    }

    x = 0;
    for (size_t k = 0; k < nstrips; k++) {
        uint32_t si = img_ord[k];
        if (x < strips[si].to) {
            uint16_t id = nnodes++;
            nd[id] = (Node16){{0,0},{0,0}, strips[si].to - x, 0};
            APP(1, o1_h, o1_t, id); APP(0, t0_h, t0_t, id);
        }
        APP(1, o1_h, o1_t, (uint16_t)(si + 1));
        x = strips[si].to + strips[si].width;
    }
    if (x < n) {
        uint16_t id = nnodes++;
        nd[id] = (Node16){{0,0},{0,0}, n - x, 0};
        APP(1, o1_h, o1_t, id); APP(0, t0_h, t0_t, id);
    }
    #undef APP

    if (t0_h && o0_h) { nd[0].next[0] = t0_h; nd[t0_h].prev[0] = 0; nd[t0_t].next[0] = o0_h; nd[o0_h].prev[0] = t0_t; nd[o0_t].next[0] = 0; nd[0].prev[0] = o0_t; }
    else if (t0_h)    { nd[0].next[0] = t0_h; nd[t0_h].prev[0] = 0; nd[t0_t].next[0] = 0; nd[0].prev[0] = t0_t; }
    else              { nd[0].next[0] = o0_h; nd[o0_h].prev[0] = 0; nd[o0_t].next[0] = 0; nd[0].prev[0] = o0_t; }

    if (t1_h && o1_h) { nd[0].next[1] = t1_h; nd[t1_h].prev[1] = 0; nd[t1_t].next[1] = o1_h; nd[o1_h].prev[1] = t1_t; nd[o1_t].next[1] = 0; nd[0].prev[1] = o1_t; }
    else if (t1_h)    { nd[0].next[1] = t1_h; nd[t1_h].prev[1] = 0; nd[t1_t].next[1] = 0; nd[0].prev[1] = t1_t; }
    else              { nd[0].next[1] = o1_h; nd[o1_h].prev[1] = 0; nd[o1_t].next[1] = 0; nd[0].prev[1] = o1_t; }

    size_t nrules = 0;
    uint32_t rem = n;
    while (rem > 0) {
        uint16_t nx = nd[0].prev[0], ny = nd[0].prev[1];
        uint32_t aw = nd[nx].width, bw = nd[ny].width, w = aw < bw ? aw : bw;
        rem -= w;
        if (__builtin_expect(nx == ny, 0)) {
            nd[nd[nx].prev[0]].next[0] = nd[nx].next[0]; nd[nd[nx].next[0]].prev[0] = nd[nx].prev[0];
            nd[nd[nx].prev[1]].next[1] = nd[nx].next[1]; nd[nd[nx].next[1]].prev[1] = nd[nx].prev[1];
            continue;
        }
        uint32_t lw = nd[nx].word, rw = nd[ny].word, wj;
        if (lw == 0) wj = rw; else if (rw == 0) wj = lw;
        else {
            uint32_t ll = lw <= 256 ? 1 : rules[lw - 257].len;
            uint32_t rl = rw <= 256 ? 1 : rules[rw - 257].len;
            wj = (uint32_t)nrules + 257;
            rules[nrules++] = (Rule){lw, rw, ll + rl, UINT32_MAX};
        }
        if (aw <= bw) {
            nd[nd[nx].prev[0]].next[0] = nd[nx].next[0]; nd[nd[nx].next[0]].prev[0] = nd[nx].prev[0];
            uint16_t qn = nd[ny].next[0]; nd[nx].prev[0] = ny; nd[nx].next[0] = qn; nd[ny].next[0] = nx; nd[qn].prev[0] = nx;
            nd[nx].word = wj;
            if (aw == bw) {
                nd[nd[ny].prev[0]].next[0] = nd[ny].next[0]; nd[nd[ny].next[0]].prev[0] = nd[ny].prev[0];
                nd[nd[ny].prev[1]].next[1] = nd[ny].next[1]; nd[nd[ny].next[1]].prev[1] = nd[ny].prev[1];
            } else nd[ny].width -= w;
        } else {
            nd[nd[ny].prev[1]].next[1] = nd[ny].next[1]; nd[nd[ny].next[1]].prev[1] = nd[ny].prev[1];
            uint16_t qn = nd[nx].next[1]; nd[ny].prev[1] = nx; nd[ny].next[1] = qn; nd[nx].next[1] = ny; nd[qn].prev[1] = ny;
            nd[ny].word = wj; nd[nx].width -= w;
        }
    }

    size_t n_leaves = 0;
    uint16_t id = nd[0].next[1]; x = 0;
    while (id != 0) { bottom[id] = x; x += nd[id].width; id = nd[id].next[1]; n_leaves++; }

    // Theorem: nbands <= n_leaves + 256 (each band either finishes one leaf or ends one of 256 byte buckets).
    size_t max_words_ub = n_leaves + 256;
    if (*p_inl_cap < 256 + max_words_ub) {
        size_t nc = 256 + max_words_ub + 64;
        *p_inl = (uint8_t (*)[16])realloc(*p_inl, nc * 16);
        *p_lens = (uint8_t *)realloc(*p_lens, nc);
        *p_inl_cap = nc; *p_inl_init = 0;
    }
    if (!*p_inl_init) {
        for (int c = 0; c < 256; c++) {
            memset((*p_inl)[c], 0, 16);
            (*p_inl)[c][right_align ? 15 : 0] = (uint8_t)c;
            (*p_lens)[c] = 1;
        }
        *p_inl_init = 1;
    }

    size_t nbands = 0, nspell = 0, noff = 1, nwords = 0;
    ENSURE_TIGHT(*p_off, *p_off_cap, max_words_ub + 2, uint32_t); (*p_off)[0] = 0;
    ENSURE_TIGHT(*p_bands, *p_bands_cap, max_words_ub + 16, Band);
    ENSURE_TIGHT(*p_spell, *p_spell_cap, 65536, uint8_t);

    id = nd[0].next[0];
    uint32_t in_node = 0, k = 0;
    for (int a = 0; a < 256; a++) {
        cursors[a] = nbands;
        uint32_t end = k + thin[a];
        while (k < end) {
            uint32_t nw = nd[id].width, width = (end - k) < (nw - in_node) ? (end - k) : (nw - in_node);
            uint32_t dst = bottom[id] + in_node, wd = nd[id].word, code;
            if (wd == 0) code = (uint32_t)a;
            else {
                uint32_t h = wd <= 256 ? 1 : rules[wd - 257].len;
                uint32_t wl = h + 1;
                ENSURE_TIGHT(*p_spell, *p_spell_cap, nspell + wl + 16, uint8_t);
                uint8_t *sp_buf = *p_spell;
                uint32_t o0 = (uint32_t)nspell;
                size_t sp = 0; stk[sp++] = wd;
                while (sp > 0) {
                    uint32_t cur = stk[--sp];
                    if (cur <= 256) sp_buf[nspell++] = (uint8_t)(cur - 1);
                    else {
                        Rule *rl = &rules[cur - 257];
                        if (rl->first == UINT32_MAX) { rl->first = (uint32_t)nspell; stk[sp++] = rl->right; stk[sp++] = rl->left; }
                        else { memcpy(sp_buf + nspell, sp_buf + rl->first, rl->len); nspell += rl->len; }
                    }
                }
                sp_buf[nspell++] = (uint8_t)a;
                (*p_off)[noff++] = (uint32_t)nspell;
                code = 256 + (uint32_t)nwords++;
                if (wl <= 16) {
                    (*p_lens)[code] = (uint8_t)wl;
                    _mm_storeu_si128((__m128i *)(*p_inl)[code], _mm_setzero_si128());
                    memcpy((*p_inl)[code] + (right_align ? (16 - wl) : 0), sp_buf + o0, wl);
                } else {
                    (*p_lens)[code] = 255;
                }
            }
            (*p_bands)[nbands++] = (Band){k + width, dst - k, code};
            in_node += width; k += width;
            if (in_node == nw) { id = nd[id].next[0]; in_node = 0; }
        }
    }
    cursors[256] = nbands;
    *out_nwords = nwords;
}

// ============================================================================
// W1: Chaitin-Open (Compact 1.25x Queues + Direct Node16 + Full Chaitin Overlay)
// ============================================================================
typedef struct {
    uint32_t *q_buf[256];
    uint32_t q_cap[256];
    uint32_t *edges;    size_t edges_cap; // ALSO hosts 100% of Endpoint in Phases 1-2!
    Band *bands;        size_t bands_cap;
    uint8_t *spelling;  size_t spell_cap;
    uint32_t *offsets;  size_t off_cap;
    uint8_t (*inline16)[16]; size_t inl_cap;
    uint8_t *lens;
    int inl_init;
} CpcArenaScratch;

static void w1_init(CpcArenaScratch *s) { memset(s, 0, sizeof(*s)); }
static void w1_free(CpcArenaScratch *s) {
    for (int a = 0; a < 256; a++) free(s->q_buf[a]);
    free(s->edges); free(s->bands);
    free(s->spelling); free(s->offsets); free(s->inline16); free(s->lens);
}
static size_t w1_ws(const CpcArenaScratch *s) {
    size_t qb = 0;
    for (int a = 0; a < 256; a++) qb += (size_t)s->q_cap[a] * 4;
    return qb + s->edges_cap * 4 + s->bands_cap * sizeof(Band)
         + s->spell_cap + s->off_cap * 4 + s->inl_cap * 17;
}

static inline void w1_q_push16(CpcArenaScratch *s, uint32_t a, uint32_t at, uint32_t m, __m256i iota0, __m256i iota8) {
    uint32_t need = at + 16u;
    if (__builtin_expect(s->q_cap[a] < need, 0)) {
        uint32_t ncap = need + (need >> 2) + 64u;
        s->q_buf[a] = (uint32_t *)realloc(s->q_buf[a], (size_t)ncap * 4);
        s->q_cap[a] = ncap;
    }
    uint32_t *dst = s->q_buf[a] + at;
    __m256i vb = _mm256_set1_epi32((int)m);
    _mm256_storeu_si256((__m256i *)(dst + 0), _mm256_add_epi32(vb, iota0));
    _mm256_storeu_si256((__m256i *)(dst + 8), _mm256_add_epi32(vb, iota8));
}

static inline size_t overlay_needed_u32(size_t nstrips, uint32_t m) {
    size_t max_nodes  = 3 * nstrips + 8;
    size_t off_strips = ((nstrips + 2) * sizeof(Strip) + 15u) & ~15u;
    size_t off_nodes  = off_strips + max_nodes * sizeof(Node16);
    size_t off_rules  = off_nodes  + max_nodes * sizeof(Rule);
    size_t off_bot    = off_rules  + ((max_nodes * 4 + 15u) & ~15u);
    size_t off_img    = off_bot    + (((nstrips + 4) * 4 + 15u) & ~15u);
    size_t off_end    = off_img    + 2048;
    size_t ov_u32     = (off_end + 3u) / 4u;
    size_t m_u32      = (size_t)m + 16u;
    return ov_u32 > m_u32 ? ov_u32 : m_u32;
}

static void run_w1_cpc_arena(CpcArenaScratch *s, const RunsInput *ri, uint32_t n, uint32_t p, uint8_t *out) {
    // Before Pass 1, only strips[0..nstrips) (20-68 KiB) is written at start of s->edges!
    ENSURE_TIGHT(s->edges, s->edges_cap, 16384, uint32_t);
    Strip *strips = (Strip *)s->edges;
    const __m256i iota0 = _mm256_setr_epi32(0,1,2,3,4,5,6,7);
    const __m256i iota8 = _mm256_setr_epi32(8,9,10,11,12,13,14,15);

    uint32_t all[256] = {0}, thin[256] = {0}, thick[256] = {0};
    uint32_t m = 0, mark = 0, pos = 0;
    size_t nstrips = 0, i = 0;
    const uint16_t * restrict recs = ri->recs;
    const size_t nrecs = ri->nrecs;

    // Sentinel at recs[nrecs] eliminates i < nrecs inside merge test!
    while (i < nrecs) {
        uint16_t rec = recs[i++];
        uint32_t c = rec >> 8;
        uint32_t w = rec & 255u;
        if (__builtin_expect((recs[i] >> 8) == c, 0)) {
            do { w += recs[i++] & 255u; } while ((recs[i] >> 8) == c);
        }
        if (__builtin_expect(w <= 16, 1)) {
            if (__builtin_expect(p - pos < w, 0)) mark = m + (p - pos);
            w1_q_push16(s, c, thin[c], m, iota0, iota8);
            thin[c] += w; m += w; all[c] += w;
        } else if (__builtin_expect(p - pos >= w, 1)) {
            if (__builtin_expect((nstrips + 2) * (sizeof(Strip) / 4) > s->edges_cap, 0)) {
                ENSURE_DBL(s->edges, s->edges_cap, (nstrips + 2) * (sizeof(Strip) / 4), uint32_t);
                strips = (Strip *)s->edges;
            }
            strips[nstrips++] = (Strip){pos, all[c], w, (uint8_t)c, {0,0,0}};
            thick[c]++; all[c] += w;
        } else {
            if (__builtin_expect((nstrips + 3) * (sizeof(Strip) / 4) > s->edges_cap, 0)) {
                ENSURE_DBL(s->edges, s->edges_cap, (nstrips + 3) * (sizeof(Strip) / 4), uint32_t);
                strips = (Strip *)s->edges;
            }
            if (pos < p) { uint32_t lw = p - pos; strips[nstrips++] = (Strip){pos, all[c], lw, (uint8_t)c, {0,0,0}}; thick[c]++; all[c] += lw; }
            mark = m;
            w1_q_push16(s, c, thin[c], m, iota0, iota8);
            thin[c] += 1; m += 1; all[c] += 1;
            if (p + 1 < pos + w) { uint32_t rw = pos + w - p - 1; strips[nstrips++] = (Strip){p + 1, all[c], rw, (uint8_t)c, {0,0,0}}; thick[c]++; all[c] += rw; }
        }
        pos += w;
    }

    // Exact post-Pass-1 Chaitin overlay capacity (Theorem 5.3: 174.7-546.4 KiB < 351.1-798.0 KiB m*4 on 6/6 blocks!)
    ENSURE_TIGHT(s->edges, s->edges_cap, overlay_needed_u32(nstrips, m), uint32_t);
    strips = (Strip *)s->edges;

    size_t cursors[257], nwords = 0;
    run_direct_node16_endpoint_aliased(
        strips, nstrips, all, thick, thin, n, (uint8_t *)s->edges, s->edges_cap * 4,
        &s->bands, &s->bands_cap, &s->spelling, &s->spell_cap, &s->offsets, &s->off_cap,
        &s->inline16, &s->lens, &s->inl_cap, &s->inl_init, 0, cursors, &nwords
    );
    // Endpoint inside s->edges is now 100% dead; wire s->edges[0..m) via fast band-span SIMD!
    uint32_t bits = m <= 1 ? 1 : (32u - (uint32_t)__builtin_clz(m - 1));
    uint32_t k = 0;
    uint32_t * restrict edges = s->edges;
    for (int a = 0; a < 256; a++) {
        const uint32_t * restrict src = s->q_buf[a];
        for (size_t bi = cursors[a]; bi < cursors[a + 1]; bi++) {
            Band b = s->bands[bi];
            uint32_t span = b.end - k, dst = k + b.delta, pack = b.code << bits;
            uint32_t * restrict ed = edges + dst;
            for (uint32_t d = 0; d < span; d++) ed[d] = pack | src[d];
            src += span; k = b.end;
        }
    }

    uint32_t mask = (1u << bits) - 1, j = mark;
    size_t at = 0;
    const uint8_t * restrict lens = s->lens;
    const uint8_t (* restrict inl)[16] = s->inline16;
    for (uint32_t step = 0; step < m; step++) {
        uint32_t edge = edges[j], code = edge >> bits; j = edge & mask;
        uint8_t len = lens[code];
        if (__builtin_expect(len != 255, 1)) {
            _mm_storeu_si128((__m128i *)(out + at), _mm_loadu_si128((const __m128i *)inl[code]));
            at += len;
        } else {
            uint32_t w = code - 256, o0 = s->offsets[w], wl = s->offsets[w + 1] - o0;
            size_t take = wl < (n - at) ? wl : (n - at);
            memcpy(out + at, s->spelling + o0, take); at += take;
        }
        if (j == mark || at >= n) break;
    }
    if (at > n) at = n;
    while (at < n) { size_t take = at < (n - at) ? at : (n - at); memcpy(out + at, out, take); at += take; }
}

// ============================================================================
// W2: CPC-Micro (0 B Queues! u16 Stream + Direct Node16 + Full Chaitin Overlay)
// ============================================================================
typedef struct {
    uint16_t *thin_stream; size_t ts_cap;
    uint32_t *edges;       size_t edges_cap; // Hosts 100% of Endpoint in Phases 1-2!
    Band *bands;           size_t bands_cap;
    uint8_t *spelling;     size_t spell_cap;
    uint32_t *offsets;     size_t off_cap;
    uint8_t (*inline16_right)[16]; size_t inl_cap;
    uint8_t *lens;
    int inl_init;
} CpcMicroScratch;

static void w2_init(CpcMicroScratch *s) { memset(s, 0, sizeof(*s)); }
static void w2_free(CpcMicroScratch *s) {
    free(s->thin_stream); free(s->edges); free(s->bands);
    free(s->spelling); free(s->offsets); free(s->inline16_right); free(s->lens);
}
static size_t w2_ws(const CpcMicroScratch *s) {
    return s->ts_cap * 2 + s->edges_cap * 4 + s->bands_cap * sizeof(Band)
         + s->spell_cap + s->off_cap * 4 + s->inl_cap * 17;
}

static void run_w2_cpc_micro(CpcMicroScratch *s, const RunsInput *ri, uint32_t n, uint32_t p, uint8_t *out) {
    const size_t nrecs = ri->nrecs;
    ENSURE_TIGHT(s->thin_stream, s->ts_cap, nrecs + 4, uint16_t);
    // Before Pass 1, only strips[0..nstrips) (20-68 KiB) is written at start of s->edges!
    ENSURE_TIGHT(s->edges, s->edges_cap, 16384, uint32_t);

    Strip *strips = (Strip *)s->edges;
    uint16_t * restrict ts = s->thin_stream;
    const uint16_t * restrict recs = ri->recs;

    uint32_t all[256] = {0}, thin[256] = {0}, thick[256] = {0};
    uint32_t m = 0, mark = 0, pos = 0;
    size_t nstrips = 0, nthin_runs = 0, i = 0;

    while (i < nrecs) {
        uint16_t rec = recs[i++];
        uint32_t c = rec >> 8;
        uint32_t w = rec & 255u;
        if (__builtin_expect((recs[i] >> 8) == c, 0)) {
            do { w += recs[i++] & 255u; } while ((recs[i] >> 8) == c);
        }
        if (__builtin_expect(w <= 16, 1)) {
            if (__builtin_expect(p - pos < w, 0)) mark = m + (p - pos);
            ts[nthin_runs++] = (uint16_t)((c << 8) | w);
            thin[c] += w; m += w; all[c] += w;
        } else if (__builtin_expect(p - pos >= w, 1)) {
            if (__builtin_expect((nstrips + 2) * (sizeof(Strip) / 4) > s->edges_cap, 0)) {
                ENSURE_DBL(s->edges, s->edges_cap, (nstrips + 2) * (sizeof(Strip) / 4), uint32_t);
                strips = (Strip *)s->edges;
            }
            strips[nstrips++] = (Strip){pos, all[c], w, (uint8_t)c, {0,0,0}};
            thick[c]++; all[c] += w;
        } else {
            if (__builtin_expect((nstrips + 3) * (sizeof(Strip) / 4) > s->edges_cap, 0)) {
                ENSURE_DBL(s->edges, s->edges_cap, (nstrips + 3) * (sizeof(Strip) / 4), uint32_t);
                strips = (Strip *)s->edges;
            }
            if (pos < p) { uint32_t lw = p - pos; strips[nstrips++] = (Strip){pos, all[c], lw, (uint8_t)c, {0,0,0}}; thick[c]++; all[c] += lw; }
            mark = m; ts[nthin_runs++] = (uint16_t)((c << 8) | 1u); thin[c]++; m++; all[c]++;
            if (p + 1 < pos + w) { uint32_t rw = pos + w - p - 1; strips[nstrips++] = (Strip){p + 1, all[c], rw, (uint8_t)c, {0,0,0}}; thick[c]++; all[c] += rw; }
        }
        pos += w;
    }

    ENSURE_TIGHT(s->edges, s->edges_cap, overlay_needed_u32(nstrips, m), uint32_t);
    strips = (Strip *)s->edges;

    size_t cursors[257], nwords = 0;
    run_direct_node16_endpoint_aliased(
        strips, nstrips, all, thick, thin, n, (uint8_t *)s->edges, s->edges_cap * 4,
        &s->bands, &s->bands_cap, &s->spelling, &s->spell_cap, &s->offsets, &s->off_cap,
        &s->inline16_right, &s->lens, &s->inl_cap, &s->inl_init, 1, cursors, &nwords
    );

    uint32_t bits = m <= 1 ? 1 : (32u - (uint32_t)__builtin_clz(m - 1));
    uint32_t ab_z[256], ab_end[256], ab_delta[256], ab_pack[256], ab_bi[256], k_acc = 0;
    for (int a = 0; a < 256; a++) {
        ab_z[a] = k_acc; k_acc += thin[a];
        if (thin[a] > 0) { uint32_t bi = (uint32_t)cursors[a]; Band b = s->bands[bi]; ab_bi[a] = bi; ab_end[a] = b.end; ab_delta[a] = b.delta; ab_pack[a] = b.code << bits; }
        else { ab_bi[a] = ab_end[a] = ab_delta[a] = ab_pack[a] = 0; }
    }
    const __m256i iota0 = _mm256_setr_epi32(0,1,2,3,4,5,6,7);
    const __m256i iota8 = _mm256_setr_epi32(8,9,10,11,12,13,14,15);
    uint32_t * restrict eb = s->edges;
    uint32_t j = 0;
    for (size_t ri_idx = 0; ri_idx < nthin_runs; ri_idx++) {
        uint16_t tok = ts[ri_idx];
        uint32_t a = tok >> 8, w = tok & 255u, z = ab_z[a], end_b = ab_end[a];
        if (__builtin_expect(z + w <= end_b, 1)) {
            __m256i vb = _mm256_set1_epi32((int)(ab_pack[a] | (z + ab_delta[a])));
            _mm256_storeu_si256((__m256i *)(eb + j + 0), _mm256_add_epi32(vb, iota0));
            _mm256_storeu_si256((__m256i *)(eb + j + 8), _mm256_add_epi32(vb, iota8));
            j += w; z += w; ab_z[a] = z;
            if (__builtin_expect(z == end_b, 0)) {
                uint32_t nbi = ab_bi[a] + 1;
                if (nbi < (uint32_t)cursors[a + 1]) { Band nb = s->bands[nbi]; ab_bi[a] = nbi; ab_end[a] = nb.end; ab_delta[a] = nb.delta; ab_pack[a] = nb.code << bits; }
            }
        } else {
            uint32_t end_z = z + w;
            while (z < end_z) {
                uint32_t rem_b = ab_end[a] - z, span = (end_z - z) < rem_b ? (end_z - z) : rem_b, val = ab_pack[a] | (z + ab_delta[a]);
                for (uint32_t d = 0; d < span; d++) eb[j + d] = val + d;
                j += span; z += span;
                if (z == ab_end[a]) {
                    uint32_t nbi = ab_bi[a] + 1;
                    if (nbi < (uint32_t)cursors[a + 1]) { Band nb = s->bands[nbi]; ab_bi[a] = nbi; ab_end[a] = nb.end; ab_delta[a] = nb.delta; ab_pack[a] = nb.code << bits; }
                }
            }
            ab_z[a] = end_z;
        }
    }

    uint32_t mask = (1u << bits) - 1, cur_j = mark;
    size_t at = n;
    const uint8_t * restrict lens = s->lens;
    const uint8_t (* restrict inl_r)[16] = s->inline16_right;
    for (uint32_t step = 0; step < m; step++) {
        uint32_t edge = eb[cur_j], code = edge >> bits; cur_j = edge & mask;
        uint8_t len = lens[code];
        if (__builtin_expect(len != 255 && at >= 16, 1)) {
            _mm_storeu_si128((__m128i *)(out + at - 16), _mm_loadu_si128((const __m128i *)inl_r[code]));
            at -= len;
        } else {
            uint8_t sb = (uint8_t)code; const uint8_t *src; uint32_t wl;
            if (code < 256) { src = &sb; wl = 1; }
            else if (len != 255) { src = inl_r[code] + (16 - len); wl = len; }
            else { uint32_t w = code - 256, o0 = s->offsets[w]; wl = s->offsets[w + 1] - o0; src = s->spelling + o0; }
            size_t take = wl < at ? wl : at;
            memcpy(out + at - take, src + (wl - take), take); at -= take;
        }
        if (cur_j == mark || at == 0) break;
    }
    if (__builtin_expect(at > 0, 0)) {
        size_t d = n - at; memmove(out, out + at, d);
        while (d < n) { size_t take = d < (n - d) ? d : (n - d); memmove(out + d, out, take); d += take; }
    }
}

// ============================================================================
// 4-Worker Concurrent Shared-Cache / Bandwidth Contention Harness
// ============================================================================
typedef struct {
    int arm;
    int iters;
    const RunsInput *ri;
    uint32_t n, p;
    pthread_barrier_t *bar;
    uint64_t elapsed_ns;
    Open058Scratch *sc0;
    CpcArenaScratch *sc1;
    CpcMicroScratch *sc2;
    uint8_t *out;
} WorkerArg;

static void *worker_fn(void *vp) {
    WorkerArg *wa = (WorkerArg *)vp;
    pthread_barrier_wait(wa->bar);
    uint64_t t0 = now_ns();
    for (int it = 0; it < wa->iters; it++) {
        if (wa->arm == 0) run_open058(wa->sc0, wa->ri, wa->n, wa->p, wa->out);
        else if (wa->arm == 1) run_w1_cpc_arena(wa->sc1, wa->ri, wa->n, wa->p, wa->out);
        else run_w2_cpc_micro(wa->sc2, wa->ri, wa->n, wa->p, wa->out);
    }
    wa->elapsed_ns = now_ns() - t0;
    return NULL;
}

static int cmp_u64(const void *a, const void *b) {
    uint64_t x = *(const uint64_t *)a, y = *(const uint64_t *)b;
    return (x > y) - (x < y);
}

int main(int argc, char **argv) {
    const char *dir = argc > 1 ? argv[1] : "/home/antigravity/admitted-bwt";
    int rounds = argc > 2 ? atoi(argv[2]) : 25;
    const struct { const char *name; uint32_t n, p; } blks[6] = {
        {"block-0.L",   899931, 264045},
        {"block-2.L",   900000, 320807},
        {"block-85.L",  899935, 325088},
        {"block-143.L", 899869, 802868},
        {"block-210.L", 899959, 791429},
        {"block-207.L", 899998, 205180},
    };

    Open058Scratch sc0; CpcArenaScratch sc1; CpcMicroScratch sc2;
    open058_init(&sc0); w1_init(&sc1); w2_init(&sc2);

    printf("=== 1-THREAD PAIRED ROTATED CONFRONTATION (%d rounds) ===\n", rounds);
    printf("%-12s | %9s | %10s (%6s) | %10s (%6s)\n",
           "Block", "Open058", "W1:CPCArena", "/058", "W2:CPCMicro", "/058");
    printf("------------------------------------------------------------------------\n");

    #define CHECK(cond) do { if (!(cond)) { fprintf(stderr, "CHECK FAILED line %d: %s\n", __LINE__, #cond); abort(); } } while (0)
    RunsInput all_ri[6];
    for (int b = 0; b < 6; b++) {
        char path[512]; snprintf(path, sizeof(path), "%s/%s", dir, blks[b].name);
        FILE *f = fopen(path, "rb"); CHECK(f != NULL);
        uint32_t n = blks[b].n, p = blks[b].p;
        uint8_t *L = (uint8_t *)malloc(n); CHECK(fread(L, 1, n, f) == n); fclose(f);
        build_i0_runs(L, n, &all_ri[b]);

        uint8_t *ref = (uint8_t *)malloc(n); plain_invert(L, n, p, ref);
        uint8_t *out = (uint8_t *)malloc(n + 32);

        // Byte-exact + canary verification
        for (int arm = 0; arm < 3; arm++) {
            memset(out, 0xA5, n + 32);
            if (arm == 0) run_open058(&sc0, &all_ri[b], n, p, out);
            else if (arm == 1) run_w1_cpc_arena(&sc1, &all_ri[b], n, p, out);
            else run_w2_cpc_micro(&sc2, &all_ri[b], n, p, out);
            CHECK(memcmp(out, ref, n) == 0);
            for (int k = 16; k < 32; k++) CHECK(out[n + k] == 0xA5);
        }

        uint64_t t[3][64]; CHECK(rounds <= 64);
        for (int r = -3; r < rounds; r++) {
            for (int rot = 0; rot < 3; rot++) {
                int arm = (r + 3 + rot) % 3;
                uint64_t t0 = now_ns();
                if (arm == 0) run_open058(&sc0, &all_ri[b], n, p, out);
                else if (arm == 1) run_w1_cpc_arena(&sc1, &all_ri[b], n, p, out);
                else run_w2_cpc_micro(&sc2, &all_ri[b], n, p, out);
                uint64_t dt = now_ns() - t0;
                if (r >= 0) t[arm][r] = dt;
            }
        }
        double med[3];
        for (int arm = 0; arm < 3; arm++) {
            qsort(t[arm], rounds, sizeof(uint64_t), cmp_u64);
            med[arm] = (double)t[arm][rounds / 2] / 1e3;
        }
        printf("%-12s | %7.1fus | %8.1fus (%5.3fx) | %8.1fus (%5.3fx)\n",
               blks[b].name, med[0], med[1], med[1]/med[0], med[2], med[2]/med[0]);

        free(L); free(ref); free(out);
    }

    printf("\nPersistent Per-Worker Scratch After All 6 Blocks (excluding 900 KiB out):\n");
    {
        double m7_0 = 7.0 * (open058_ws(&sc0) + 900000.0) / (1024.0 * 1024.0);
        double m7_1 = 7.0 * (w1_ws(&sc1) + 900000.0) / (1024.0 * 1024.0);
        double m7_2 = 7.0 * (w2_ws(&sc2) + 900000.0) / (1024.0 * 1024.0);
        double m10_0 = 10.0 * (open058_ws(&sc0) + 900000.0) / (1024.0 * 1024.0);
        double m10_1 = 10.0 * (w1_ws(&sc1) + 900000.0) / (1024.0 * 1024.0);
        double m10_2 = 10.0 * (w2_ws(&sc2) + 900000.0) / (1024.0 * 1024.0);
        printf("  C0: Open058   : %6.1f KiB (1.000x) -> 7w=%5.1f MiB (%s), 10w=%5.1f MiB\n",
               open058_ws(&sc0)/1024.0, m7_0, m7_0 <= 20.0 ? "<=20 MiB L3" : ">20 MiB L3!", m10_0);
        printf("  W1: CPC-Arena : %6.1f KiB (%5.3fx) -> 7w=%5.1f MiB (%s), 10w=%5.1f MiB\n",
               w1_ws(&sc1)/1024.0, (double)w1_ws(&sc1)/open058_ws(&sc0), m7_1, m7_1 <= 20.0 ? "<=20 MiB L3" : ">20 MiB L3!", m10_1);
        printf("  W2: CPC-Micro : %6.1f KiB (%5.3fx) -> 7w=%5.1f MiB (%s), 10w=%5.1f MiB\n",
               w2_ws(&sc2)/1024.0, (double)w2_ws(&sc2)/open058_ws(&sc0), m7_2, m7_2 <= 20.0 ? "<=20 MiB L3" : ">20 MiB L3!", m10_2);
    }

    // 4-worker barrier-synchronized multi-thread run (each worker with independent scratch & block)
    if (rounds >= 5) {
        const int NT = 4, BATCH = 8;
        Open058Scratch mt0[NT]; CpcArenaScratch mt1[NT]; CpcMicroScratch mt2[NT];
        uint8_t *mt_out[NT];
        for (int w = 0; w < NT; w++) {
            open058_init(&mt0[w]); w1_init(&mt1[w]); w2_init(&mt2[w]);
            mt_out[w] = (uint8_t *)malloc(900032);
        }
        printf("\n=== 4-WORKER CONCURRENT BARRIER THROUGHPUT (%d rounds x %d decodes/worker) ===\n", rounds, BATCH);
        printf("%-12s | %10s | %10s (%6s) | %10s (%6s)\n",
               "Block", "Open058/4t", "W1:Arena/4t", "/058", "W2:Micro/4t", "/058");
        printf("--------------------------------------------------------------------------\n");
        pthread_barrier_t bar;
        pthread_barrier_init(&bar, NULL, NT);

        for (int b = 0; b < 6; b++) {
            uint64_t mt_t[3][64];
            for (int r = -2; r < rounds; r++) {
                for (int rot = 0; rot < 3; rot++) {
                    int arm = (r + 2 + rot) % 3;
                    pthread_t th[NT]; WorkerArg wa[NT];
                    for (int w = 0; w < NT; w++) {
                        wa[w] = (WorkerArg){arm, BATCH, &all_ri[b], blks[b].n, blks[b].p, &bar, 0, &mt0[w], &mt1[w], &mt2[w], mt_out[w]};
                        pthread_create(&th[w], NULL, worker_fn, &wa[w]);
                    }
                    uint64_t slowest = 0;
                    for (int w = 0; w < NT; w++) {
                        pthread_join(th[w], NULL);
                        if (wa[w].elapsed_ns > slowest) slowest = wa[w].elapsed_ns;
                    }
                    if (r >= 0) mt_t[arm][r] = slowest / BATCH;
                }
            }
            double med[3];
            for (int arm = 0; arm < 3; arm++) {
                qsort(mt_t[arm], rounds, sizeof(uint64_t), cmp_u64);
                med[arm] = (double)mt_t[arm][rounds / 2] / 1e3;
            }
            printf("%-12s | %8.1fus | %8.1fus (%5.3fx) | %8.1fus (%5.3fx)\n",
                   blks[b].name, med[0], med[1], med[1]/med[0], med[2], med[2]/med[0]);
        }
        pthread_barrier_destroy(&bar);
        for (int w = 0; w < NT; w++) { open058_free(&mt0[w]); w1_free(&mt1[w]); w2_free(&mt2[w]); free(mt_out[w]); }
    }

    for (int b = 0; b < 6; b++) free(all_ri[b].recs);
    open058_free(&sc0); w1_free(&sc1); w2_free(&sc2);
    return 0;
}
