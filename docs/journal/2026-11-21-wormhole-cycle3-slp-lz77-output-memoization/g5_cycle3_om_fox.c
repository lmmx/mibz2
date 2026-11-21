// Gate G5 Paid-Cost Falsifier for Wormhole Cycle 3:
// Confronts mibz2 0.5.8 Baseline (Open<16> with Vec16 queue + U16 inline16 emit)
// against OM-Fox<Lambda> (Compact short-only inline16 + Zero-spelling Rytter-Gasieniec
// output-memoized SLP decoding + Zorich/AHT periodic quotienting) at Lambda in {16, 8, 4, 2}.

#define _POSIX_C_SOURCE 200809L
#include <assert.h>
#include <immintrin.h>
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

static size_t build_records(const uint8_t *L, size_t n, uint16_t *recs) {
    size_t nr = 0, i = 0;
    while (i < n) {
        uint8_t c = L[i];
        size_t j = i + 1;
        while (j < n && L[j] == c && (j - i) < 127) j++;
        recs[nr++] = (uint16_t)(((uint16_t)c << 8) | (uint16_t)(j - i));
        i = j;
    }
    return nr;
}

static void plain_invert(const uint8_t *L, size_t n, uint32_t p, uint8_t *out) {
    uint32_t counts[256] = {0}, ftab[256];
    for (size_t i = 0; i < n; i++) counts[L[i]]++;
    uint32_t acc = 0;
    for (int a = 0; a < 256; a++) { ftab[a] = acc; acc += counts[a]; }
    uint32_t *psi = (uint32_t *)malloc(n * sizeof(uint32_t));
    for (size_t i = 0; i < n; i++) psi[ftab[L[i]]++] = (uint32_t)i;
    uint32_t cur = psi[p];
    for (size_t t = 0; t < n; t++) {
        out[t] = L[cur];
        cur = psi[cur];
    }
    free(psi);
}

typedef struct { uint32_t from, to, width; uint8_t byte; } Strip;
typedef struct { uint32_t prev[2], next[2], width, word; } Node;
typedef struct { uint32_t left, right, len, first; } Rule;
typedef struct { uint32_t end, delta, code; } Band;
typedef struct { uint32_t word, len; uint8_t tail; } LongHandle;

typedef struct {
    // Shared Reusable Buffers
    uint32_t *q_buf[256];
    uint32_t q_cap[256];
    Strip *strips;      size_t strips_cap;
    uint32_t *img_ord;  size_t img_cap;
    Node *nodes;        size_t nodes_cap;
    Rule *rules;        size_t rules_cap;
    uint32_t *ord0, *ord1, *term0, *term1, *bottom, *stack;
    size_t ord_cap, stk_cap;
    Band *bands;        size_t bands_cap;
    uint8_t *spelling;  size_t spell_cap;
    uint32_t *offsets;  size_t off_cap;
    uint32_t *edges;    size_t edges_cap;
    uint8_t (*inline16)[16]; size_t inl_cap;
    uint8_t *lens;
    LongHandle *longs;  size_t longs_cap;
    int inl_init;
} OpenScratch;

static void scratch_init(OpenScratch *s) {
    memset(s, 0, sizeof(*s));
}

static void scratch_free(OpenScratch *s) {
    for (int a = 0; a < 256; a++) free(s->q_buf[a]);
    free(s->strips); free(s->img_ord); free(s->nodes); free(s->rules);
    free(s->ord0); free(s->ord1); free(s->term0); free(s->term1);
    free(s->bottom); free(s->stack); free(s->bands); free(s->spelling);
    free(s->offsets); free(s->edges); free(s->inline16); free(s->lens);
    free(s->longs);
}

#define ENSURE(ptr, cap, need, T) do { \
    if ((cap) < (need)) { \
        size_t ncap = (cap) ? (cap) * 2 : 1024; \
        if (ncap < (need)) ncap = (need); \
        (ptr) = (T *)realloc((ptr), ncap * sizeof(T)); \
        (cap) = ncap; \
    } \
} while (0)

static inline void q_push16(OpenScratch *s, uint8_t a, uint32_t at, uint32_t m, uint32_t w) {
    uint32_t need = at + (w > 16 ? w : 16);
    if (__builtin_expect(s->q_cap[a] < need, 0)) {
        uint32_t ncap = s->q_cap[a] ? s->q_cap[a] * 2 : 1024;
        if (ncap < need) ncap = need;
        s->q_buf[a] = (uint32_t *)realloc(s->q_buf[a], ncap * sizeof(uint32_t));
        s->q_cap[a] = ncap;
    }
    uint32_t *dst = s->q_buf[a] + at;
    if (w <= 16) {
        __m256i base = _mm256_set1_epi32((int)m);
        __m256i i0 = _mm256_setr_epi32(0,1,2,3,4,5,6,7);
        __m256i i1 = _mm256_setr_epi32(8,9,10,11,12,13,14,15);
        _mm256_storeu_si256((__m256i *)(dst + 0), _mm256_add_epi32(base, i0));
        _mm256_storeu_si256((__m256i *)(dst + 8), _mm256_add_epi32(base, i1));
    } else {
        for (uint32_t i = 0; i < w; i++) dst[i] = m + i;
    }
}

static size_t scratch_working_set_bytes(const OpenScratch *s, int include_spell) {
    size_t b = 0;
    for (int a = 0; a < 256; a++) b += s->q_cap[a] * 4;
    b += s->strips_cap * sizeof(Strip) + s->img_cap * 4;
    b += s->nodes_cap * sizeof(Node) + s->rules_cap * sizeof(Rule);
    b += s->ord_cap * 4 * 5 + s->stk_cap * 4;
    b += s->bands_cap * sizeof(Band);
    if (include_spell) b += s->spell_cap + s->off_cap * 4;
    b += s->edges_cap * 4 + s->inl_cap * 17 + s->longs_cap * sizeof(LongHandle);
    return b;
}

// Baseline: Exact C port of mibz2 0.5.8 Open<16, INDUCE=true, QUEUE=true>
static void run_open058_baseline(
    OpenScratch *s, const uint16_t *recs, size_t nrecs, uint32_t n, uint32_t p, uint32_t limit, uint8_t *out
) {
    uint32_t all[256] = {0}, thin[256] = {0}, thick[256] = {0};
    uint32_t m = 0, mark = 0;
    size_t nstrips = 0;
    ENSURE(s->strips, s->strips_cap, 4096, Strip);

    uint32_t pos = 0;
    size_t i = 0;
    while (i < nrecs) {
        uint16_t rec = recs[i++];
        uint8_t c = (uint8_t)(rec >> 8);
        uint32_t w = (uint32_t)(rec & 255);
        while (i < nrecs && (uint8_t)(recs[i] >> 8) == c) {
            w += (uint32_t)(recs[i] & 255);
            i++;
        }
        if (w <= limit) {
            if (p - pos < w) mark = m + (p - pos);
            q_push16(s, c, thin[c], m, w);
            thin[c] += w; m += w; all[c] += w;
        } else if (p - pos >= w) {
            ENSURE(s->strips, s->strips_cap, nstrips + 1, Strip);
            s->strips[nstrips++] = (Strip){pos, all[c], w, c};
            thick[c]++; all[c] += w;
        } else {
            ENSURE(s->strips, s->strips_cap, nstrips + 2, Strip);
            if (pos < p) {
                uint32_t lw = p - pos;
                s->strips[nstrips++] = (Strip){pos, all[c], lw, c};
                thick[c]++; all[c] += lw;
            }
            mark = m;
            q_push16(s, c, thin[c], m, 1);
            thin[c] += 1; m += 1; all[c] += 1;
            if (p + 1 < pos + w) {
                uint32_t rw = pos + w - p - 1;
                s->strips[nstrips++] = (Strip){p + 1, all[c], rw, c};
                thick[c]++; all[c] += rw;
            }
        }
        pos += w;
    }

    uint32_t first[256], write[256], acc0 = 0, acc1 = 0;
    for (int a = 0; a < 256; a++) { first[a] = acc0; acc0 += all[a]; write[a] = acc1; acc1 += thick[a]; }
    ENSURE(s->img_ord, s->img_cap, nstrips + 1, uint32_t);
    for (size_t si = 0; si < nstrips; si++) {
        uint8_t a = s->strips[si].byte;
        s->strips[si].to += first[a];
        s->img_ord[write[a]++] = (uint32_t)si;
    }

    size_t max_nodes = 3 * nstrips + 8;
    ENSURE(s->nodes, s->nodes_cap, max_nodes, Node);
    if (s->ord_cap < max_nodes) {
        size_t nc = max_nodes * 2;
        s->ord0 = (uint32_t *)realloc(s->ord0, nc * 4);
        s->ord1 = (uint32_t *)realloc(s->ord1, nc * 4);
        s->term0 = (uint32_t *)realloc(s->term0, nc * 4);
        s->term1 = (uint32_t *)realloc(s->term1, nc * 4);
        s->bottom = (uint32_t *)realloc(s->bottom, nc * 4);
        s->ord_cap = nc;
    }
    ENSURE(s->rules, s->rules_cap, max_nodes, Rule);

    s->nodes[0] = (Node){0};
    uint32_t nnodes = 1;
    size_t n_o0 = 0, n_o1 = 0, n_t0 = 0, n_t1 = 0;
    for (size_t si = 0; si < nstrips; si++) {
        s->nodes[nnodes++] = (Node){{0,0},{0,0}, s->strips[si].width, (uint32_t)s->strips[si].byte + 1};
    }
    uint32_t x = 0;
    for (size_t si = 0; si < nstrips; si++) {
        if (x < s->strips[si].from) {
            uint32_t id = nnodes++;
            s->nodes[id] = (Node){{0,0},{0,0}, s->strips[si].from - x, 0};
            s->ord0[n_o0++] = id; s->term1[n_t1++] = id;
        }
        s->ord0[n_o0++] = (uint32_t)si + 1;
        x = s->strips[si].from + s->strips[si].width;
    }
    if (x < n) {
        uint32_t id = nnodes++;
        s->nodes[id] = (Node){{0,0},{0,0}, n - x, 0};
        s->ord0[n_o0++] = id; s->term1[n_t1++] = id;
    }
    x = 0;
    for (size_t k = 0; k < nstrips; k++) {
        uint32_t si = s->img_ord[k];
        if (x < s->strips[si].to) {
            uint32_t id = nnodes++;
            s->nodes[id] = (Node){{0,0},{0,0}, s->strips[si].to - x, 0};
            s->ord1[n_o1++] = id; s->term0[n_t0++] = id;
        }
        s->ord1[n_o1++] = si + 1;
        x = s->strips[si].to + s->strips[si].width;
    }
    if (x < n) {
        uint32_t id = nnodes++;
        s->nodes[id] = (Node){{0,0},{0,0}, n - x, 0};
        s->ord1[n_o1++] = id; s->term0[n_t0++] = id;
    }

    uint32_t prv = 0;
    for (size_t k = 0; k < n_t0; k++) { uint32_t id = s->term0[k]; s->nodes[prv].next[0] = id; s->nodes[id].prev[0] = prv; prv = id; }
    for (size_t k = 0; k < n_o0; k++) { uint32_t id = s->ord0[k];  s->nodes[prv].next[0] = id; s->nodes[id].prev[0] = prv; prv = id; }
    s->nodes[prv].next[0] = 0; s->nodes[0].prev[0] = prv;

    prv = 0;
    for (size_t k = 0; k < n_t1; k++) { uint32_t id = s->term1[k]; s->nodes[prv].next[1] = id; s->nodes[id].prev[1] = prv; prv = id; }
    for (size_t k = 0; k < n_o1; k++) { uint32_t id = s->ord1[k];  s->nodes[prv].next[1] = id; s->nodes[id].prev[1] = prv; prv = id; }
    s->nodes[prv].next[1] = 0; s->nodes[0].prev[1] = prv;

    size_t nrules = 0;
    uint32_t rem = n;
    Node *nd = s->nodes;
    while (rem > 0) {
        uint32_t nx = nd[0].prev[0], ny = nd[0].prev[1];
        uint32_t aw = nd[nx].width, bw = nd[ny].width;
        uint32_t w = aw < bw ? aw : bw;
        rem -= w;
        if (nx == ny) {
            nd[nd[nx].prev[0]].next[0] = nd[nx].next[0]; nd[nd[nx].next[0]].prev[0] = nd[nx].prev[0];
            nd[nd[nx].prev[1]].next[1] = nd[nx].next[1]; nd[nd[nx].next[1]].prev[1] = nd[nx].prev[1];
            continue;
        }
        uint32_t lw = nd[nx].word, rw = nd[ny].word, wj;
        if (lw == 0) wj = rw;
        else if (rw == 0) wj = lw;
        else {
            ENSURE(s->rules, s->rules_cap, nrules + 1, Rule);
            uint32_t ll = lw <= 256 ? 1 : s->rules[lw - 257].len;
            uint32_t rl = rw <= 256 ? 1 : s->rules[rw - 257].len;
            wj = (uint32_t)nrules + 257;
            s->rules[nrules++] = (Rule){lw, rw, ll + rl, UINT32_MAX};
        }
        if (aw <= bw) {
            nd[nd[nx].prev[0]].next[0] = nd[nx].next[0]; nd[nd[nx].next[0]].prev[0] = nd[nx].prev[0];
            uint32_t qn = nd[ny].next[0];
            nd[nx].prev[0] = ny; nd[nx].next[0] = qn; nd[ny].next[0] = nx; nd[qn].prev[0] = nx;
            nd[nx].word = wj;
            if (aw == bw) {
                nd[nd[ny].prev[0]].next[0] = nd[ny].next[0]; nd[nd[ny].next[0]].prev[0] = nd[ny].prev[0];
                nd[nd[ny].prev[1]].next[1] = nd[ny].next[1]; nd[nd[ny].next[1]].prev[1] = nd[ny].prev[1];
            } else nd[ny].width -= w;
        } else {
            nd[nd[ny].prev[1]].next[1] = nd[ny].next[1]; nd[nd[ny].next[1]].prev[1] = nd[ny].prev[1];
            uint32_t qn = nd[nx].next[1];
            nd[ny].prev[1] = nx; nd[ny].next[1] = qn; nd[nx].next[1] = ny; nd[qn].prev[1] = ny;
            nd[ny].word = wj;
            nd[nx].width -= w;
        }
    }

    uint32_t id = nd[0].next[1];
    x = 0;
    while (id != 0) { s->bottom[id] = x; x += nd[id].width; id = nd[id].next[1]; }

    size_t nbands = 0, nspell = 0, noff = 1, nwords = 0;
    ENSURE(s->offsets, s->off_cap, 4096, uint32_t);
    s->offsets[0] = 0;
    ENSURE(s->bands, s->bands_cap, nnodes + 256, Band);
    ENSURE(s->spelling, s->spell_cap, 65536, uint8_t);
    ENSURE(s->stack, s->stk_cap, 1024, uint32_t);

    size_t cursors[257];
    id = nd[0].next[0];
    uint32_t in_node = 0, k = 0;
    for (int a = 0; a < 256; a++) {
        cursors[a] = nbands;
        uint32_t end = k + thin[a];
        while (k < end) {
            uint32_t nw = nd[id].width;
            uint32_t width = (end - k) < (nw - in_node) ? (end - k) : (nw - in_node);
            uint32_t dst = s->bottom[id] + in_node;
            uint32_t wd = nd[id].word, code;
            if (wd == 0) code = (uint32_t)a;
            else {
                nwords++;
                uint32_t h = wd <= 256 ? 1 : s->rules[wd - 257].len;
                ENSURE(s->spelling, s->spell_cap, nspell + h + 2, uint8_t);
                size_t sp = 0; s->stack[sp++] = wd;
                while (sp > 0) {
                    uint32_t cur = s->stack[--sp];
                    if (cur <= 256) s->spelling[nspell++] = (uint8_t)(cur - 1);
                    else {
                        Rule *rl = &s->rules[cur - 257];
                        if (rl->first == UINT32_MAX) {
                            rl->first = (uint32_t)nspell;
                            ENSURE(s->stack, s->stk_cap, sp + 2, uint32_t);
                            s->stack[sp++] = rl->right; s->stack[sp++] = rl->left;
                        } else {
                            memcpy(s->spelling + nspell, s->spelling + rl->first, rl->len);
                            nspell += rl->len;
                        }
                    }
                }
                s->spelling[nspell++] = (uint8_t)a;
                ENSURE(s->offsets, s->off_cap, noff + 1, uint32_t);
                s->offsets[noff++] = (uint32_t)nspell;
                code = 255 + (uint32_t)nwords;
            }
            ENSURE(s->bands, s->bands_cap, nbands + 1, Band);
            s->bands[nbands++] = (Band){k + width, dst - k, code};
            in_node += width; k += width;
            if (in_node == nw) { id = nd[id].next[0]; in_node = 0; }
        }
    }
    cursors[256] = nbands;

    uint32_t bits = m <= 1 ? 1 : (32u - (uint32_t)__builtin_clz(m - 1));
    assert((255 + nwords) <= (UINT32_MAX >> bits));
    ENSURE(s->edges, s->edges_cap, m, uint32_t);
    k = 0;
    for (int a = 0; a < 256; a++) {
        size_t from = 0;
        const uint32_t *qa = s->q_buf[a];
        for (size_t bi = cursors[a]; bi < cursors[a + 1]; bi++) {
            Band b = s->bands[bi];
            uint32_t span = b.end - k;
            uint32_t dst = k + b.delta;
            uint32_t pack = b.code << bits;
            uint32_t *ed = s->edges + dst;
            const uint32_t *src = qa + from;
            for (uint32_t d = 0; d < span; d++) ed[d] = pack | src[d];
            from += span; k = b.end;
        }
    }

    // inline_words
    size_t ninl = 256 + nwords;
    if (s->inl_cap < ninl) {
        size_t nc = ninl * 2;
        s->inline16 = (uint8_t (*)[16])realloc(s->inline16, nc * 16);
        s->lens = (uint8_t *)realloc(s->lens, nc);
        s->inl_cap = nc;
        s->inl_init = 0;
    }
    if (!s->inl_init) {
        for (int c = 0; c < 256; c++) {
            memset(s->inline16[c], 0, 16);
            s->inline16[c][0] = (uint8_t)c;
            s->lens[c] = 1;
        }
        s->inl_init = 1;
    }
    for (size_t w = 0; w < nwords; w++) {
        uint32_t o0 = s->offsets[w], o1 = s->offsets[w + 1], wl = o1 - o0;
        memset(s->inline16[256 + w], 0, 16);
        memcpy(s->inline16[256 + w], s->spelling + o0, wl < 16 ? wl : 16);
        s->lens[256 + w] = wl <= 16 ? (uint8_t)wl : 255;
    }

    // emit
    uint32_t mask = (1u << bits) - 1;
    uint32_t j = mark;
    size_t at = 0;
    const uint32_t *edges = s->edges;
    for (uint32_t step = 0; step < m; step++) {
        uint32_t edge = edges[j];
        uint32_t code = edge >> bits;
        j = edge & mask;
        uint8_t len = s->lens[code];
        if (__builtin_expect(len != 255, 1)) {
            __m128i v = _mm_loadu_si128((const __m128i *)s->inline16[code]);
            _mm_storeu_si128((__m128i *)(out + at), v);
            at += len;
        } else {
            uint32_t w = code - 256;
            uint32_t o0 = s->offsets[w], wl = s->offsets[w + 1] - o0;
            size_t take = wl < (n - at) ? wl : (n - at);
            memcpy(out + at, s->spelling + o0, take);
            at += take;
        }
        if (j == mark || at >= n) break;
    }
    if (at > n) at = n;
    while (at < n) {
        size_t take = at < (n - at) ? at : (n - at);
        memcpy(out + at, out, take);
        at += take;
    }
}

// Candidate: OM-Fox<Lambda> (Zero-spelling Rytter-Gasieniec Output-Memoized SLP + Compact Inline16)
static void run_om_fox(
    OpenScratch *s, const uint16_t *recs, size_t nrecs, uint32_t n, uint32_t p, uint32_t limit, uint8_t *out
) {
    uint32_t all[256] = {0}, thin[256] = {0}, thick[256] = {0};
    uint32_t m = 0, mark = 0;
    size_t nstrips = 0;
    ENSURE(s->strips, s->strips_cap, 4096, Strip);

    uint32_t pos = 0;
    size_t i = 0;
    while (i < nrecs) {
        uint16_t rec = recs[i++];
        uint8_t c = (uint8_t)(rec >> 8);
        uint32_t w = (uint32_t)(rec & 255);
        while (i < nrecs && (uint8_t)(recs[i] >> 8) == c) {
            w += (uint32_t)(recs[i] & 255);
            i++;
        }
        if (w <= limit) {
            if (p - pos < w) mark = m + (p - pos);
            q_push16(s, c, thin[c], m, w);
            thin[c] += w; m += w; all[c] += w;
        } else if (p - pos >= w) {
            ENSURE(s->strips, s->strips_cap, nstrips + 1, Strip);
            s->strips[nstrips++] = (Strip){pos, all[c], w, c};
            thick[c]++; all[c] += w;
        } else {
            ENSURE(s->strips, s->strips_cap, nstrips + 2, Strip);
            if (pos < p) {
                uint32_t lw = p - pos;
                s->strips[nstrips++] = (Strip){pos, all[c], lw, c};
                thick[c]++; all[c] += lw;
            }
            mark = m;
            q_push16(s, c, thin[c], m, 1);
            thin[c] += 1; m += 1; all[c] += 1;
            if (p + 1 < pos + w) {
                uint32_t rw = pos + w - p - 1;
                s->strips[nstrips++] = (Strip){p + 1, all[c], rw, c};
                thick[c]++; all[c] += rw;
            }
        }
        pos += w;
    }

    uint32_t first[256], write[256], acc0 = 0, acc1 = 0;
    for (int a = 0; a < 256; a++) { first[a] = acc0; acc0 += all[a]; write[a] = acc1; acc1 += thick[a]; }
    ENSURE(s->img_ord, s->img_cap, nstrips + 1, uint32_t);
    for (size_t si = 0; si < nstrips; si++) {
        uint8_t a = s->strips[si].byte;
        s->strips[si].to += first[a];
        s->img_ord[write[a]++] = (uint32_t)si;
    }

    size_t max_nodes = 3 * nstrips + 8;
    ENSURE(s->nodes, s->nodes_cap, max_nodes, Node);
    if (s->ord_cap < max_nodes) {
        size_t nc = max_nodes * 2;
        s->ord0 = (uint32_t *)realloc(s->ord0, nc * 4);
        s->ord1 = (uint32_t *)realloc(s->ord1, nc * 4);
        s->term0 = (uint32_t *)realloc(s->term0, nc * 4);
        s->term1 = (uint32_t *)realloc(s->term1, nc * 4);
        s->bottom = (uint32_t *)realloc(s->bottom, nc * 4);
        s->ord_cap = nc;
    }
    ENSURE(s->rules, s->rules_cap, max_nodes, Rule);

    s->nodes[0] = (Node){0};
    uint32_t nnodes = 1;
    size_t n_o0 = 0, n_o1 = 0, n_t0 = 0, n_t1 = 0;
    for (size_t si = 0; si < nstrips; si++) {
        s->nodes[nnodes++] = (Node){{0,0},{0,0}, s->strips[si].width, (uint32_t)s->strips[si].byte + 1};
    }
    uint32_t x = 0;
    for (size_t si = 0; si < nstrips; si++) {
        if (x < s->strips[si].from) {
            uint32_t id = nnodes++;
            s->nodes[id] = (Node){{0,0},{0,0}, s->strips[si].from - x, 0};
            s->ord0[n_o0++] = id; s->term1[n_t1++] = id;
        }
        s->ord0[n_o0++] = (uint32_t)si + 1;
        x = s->strips[si].from + s->strips[si].width;
    }
    if (x < n) {
        uint32_t id = nnodes++;
        s->nodes[id] = (Node){{0,0},{0,0}, n - x, 0};
        s->ord0[n_o0++] = id; s->term1[n_t1++] = id;
    }
    x = 0;
    for (size_t k = 0; k < nstrips; k++) {
        uint32_t si = s->img_ord[k];
        if (x < s->strips[si].to) {
            uint32_t id = nnodes++;
            s->nodes[id] = (Node){{0,0},{0,0}, s->strips[si].to - x, 0};
            s->ord1[n_o1++] = id; s->term0[n_t0++] = id;
        }
        s->ord1[n_o1++] = si + 1;
        x = s->strips[si].to + s->strips[si].width;
    }
    if (x < n) {
        uint32_t id = nnodes++;
        s->nodes[id] = (Node){{0,0},{0,0}, n - x, 0};
        s->ord1[n_o1++] = id; s->term0[n_t0++] = id;
    }

    uint32_t prv = 0;
    for (size_t k = 0; k < n_t0; k++) { uint32_t id = s->term0[k]; s->nodes[prv].next[0] = id; s->nodes[id].prev[0] = prv; prv = id; }
    for (size_t k = 0; k < n_o0; k++) { uint32_t id = s->ord0[k];  s->nodes[prv].next[0] = id; s->nodes[id].prev[0] = prv; prv = id; }
    s->nodes[prv].next[0] = 0; s->nodes[0].prev[0] = prv;

    prv = 0;
    for (size_t k = 0; k < n_t1; k++) { uint32_t id = s->term1[k]; s->nodes[prv].next[1] = id; s->nodes[id].prev[1] = prv; prv = id; }
    for (size_t k = 0; k < n_o1; k++) { uint32_t id = s->ord1[k];  s->nodes[prv].next[1] = id; s->nodes[id].prev[1] = prv; prv = id; }
    s->nodes[prv].next[1] = 0; s->nodes[0].prev[1] = prv;

    size_t nrules = 0;
    uint32_t rem = n;
    Node *nd = s->nodes;
    while (rem > 0) {
        uint32_t nx = nd[0].prev[0], ny = nd[0].prev[1];
        uint32_t aw = nd[nx].width, bw = nd[ny].width;
        uint32_t w = aw < bw ? aw : bw;
        rem -= w;
        if (nx == ny) {
            nd[nd[nx].prev[0]].next[0] = nd[nx].next[0]; nd[nd[nx].next[0]].prev[0] = nd[nx].prev[0];
            nd[nd[nx].prev[1]].next[1] = nd[nx].next[1]; nd[nd[nx].next[1]].prev[1] = nd[nx].prev[1];
            continue;
        }
        uint32_t lw = nd[nx].word, rw = nd[ny].word, wj;
        if (lw == 0) wj = rw;
        else if (rw == 0) wj = lw;
        else {
            ENSURE(s->rules, s->rules_cap, nrules + 1, Rule);
            uint32_t ll = lw <= 256 ? 1 : s->rules[lw - 257].len;
            uint32_t rl = rw <= 256 ? 1 : s->rules[rw - 257].len;
            wj = (uint32_t)nrules + 257;
            s->rules[nrules++] = (Rule){lw, rw, ll + rl, UINT32_MAX};
        }
        if (aw <= bw) {
            nd[nd[nx].prev[0]].next[0] = nd[nx].next[0]; nd[nd[nx].next[0]].prev[0] = nd[nx].prev[0];
            uint32_t qn = nd[ny].next[0];
            nd[nx].prev[0] = ny; nd[nx].next[0] = qn; nd[ny].next[0] = nx; nd[qn].prev[0] = nx;
            nd[nx].word = wj;
            if (aw == bw) {
                nd[nd[ny].prev[0]].next[0] = nd[ny].next[0]; nd[nd[ny].next[0]].prev[0] = nd[ny].prev[0];
                nd[nd[ny].prev[1]].next[1] = nd[ny].next[1]; nd[nd[ny].next[1]].prev[1] = nd[ny].prev[1];
            } else nd[ny].width -= w;
        } else {
            nd[nd[ny].prev[1]].next[1] = nd[ny].next[1]; nd[nd[ny].next[1]].prev[1] = nd[ny].prev[1];
            uint32_t qn = nd[nx].next[1];
            nd[ny].prev[1] = nx; nd[ny].next[1] = qn; nd[nx].next[1] = ny; nd[qn].prev[1] = ny;
            nd[ny].word = wj;
            nd[nx].width -= w;
        }
    }

    uint32_t id = nd[0].next[1];
    x = 0;
    while (id != 0) { s->bottom[id] = x; x += nd[id].width; id = nd[id].next[1]; }

    // Ensure compact inline16 initialized
    if (s->inl_cap < 4096) {
        s->inline16 = (uint8_t (*)[16])realloc(s->inline16, 4096 * 16);
        s->lens = (uint8_t *)realloc(s->lens, 4096);
        s->inl_cap = 4096;
        s->inl_init = 0;
    }
    if (!s->inl_init) {
        for (int c = 0; c < 256; c++) {
            memset(s->inline16[c], 0, 16);
            s->inline16[c][0] = (uint8_t)c;
            s->lens[c] = 1;
        }
        s->inl_init = 1;
    }

    size_t short_end = 256, nlongs = 0, nbands = 0;
    ENSURE(s->bands, s->bands_cap, nnodes + 256, Band);
    ENSURE(s->longs, s->longs_cap, 2048, LongHandle);
    ENSURE(s->stack, s->stk_cap, 1024, uint32_t);

    size_t cursors[257];
    id = nd[0].next[0];
    uint32_t in_node = 0, k = 0;
    for (int a = 0; a < 256; a++) {
        cursors[a] = nbands;
        uint32_t end = k + thin[a];
        while (k < end) {
            uint32_t nw = nd[id].width;
            uint32_t width = (end - k) < (nw - in_node) ? (end - k) : (nw - in_node);
            uint32_t dst = s->bottom[id] + in_node;
            uint32_t wd = nd[id].word, code;
            if (wd == 0) code = (uint32_t)a;
            else {
                uint32_t h = wd <= 256 ? 1 : s->rules[wd - 257].len;
                uint32_t step_len = h + 1;
                if (step_len <= 16) {
                    if (short_end >= s->inl_cap) {
                        size_t nc = s->inl_cap * 2;
                        s->inline16 = (uint8_t (*)[16])realloc(s->inline16, nc * 16);
                        s->lens = (uint8_t *)realloc(s->lens, nc);
                        s->inl_cap = nc;
                    }
                    uint8_t *slot = s->inline16[short_end];
                    memset(slot, 0, 16);
                    size_t pos_b = 0, sp = 0;
                    s->stack[sp++] = wd;
                    while (sp > 0) {
                        uint32_t cur = s->stack[--sp];
                        if (cur <= 256) slot[pos_b++] = (uint8_t)(cur - 1);
                        else {
                            Rule rl = s->rules[cur - 257];
                            s->stack[sp++] = rl.right; s->stack[sp++] = rl.left;
                        }
                    }
                    slot[pos_b++] = (uint8_t)a;
                    s->lens[short_end] = (uint8_t)step_len;
                    code = (uint32_t)short_end++;
                } else {
                    ENSURE(s->longs, s->longs_cap, nlongs + 1, LongHandle);
                    s->longs[nlongs] = (LongHandle){wd, step_len, (uint8_t)a};
                    code = (1u << 31) | (uint32_t)nlongs++;
                }
            }
            ENSURE(s->bands, s->bands_cap, nbands + 1, Band);
            s->bands[nbands++] = (Band){k + width, dst - k, code};
            in_node += width; k += width;
            if (in_node == nw) { id = nd[id].next[0]; in_node = 0; }
        }
    }
    cursors[256] = nbands;

    uint32_t bits = m <= 1 ? 1 : (32u - (uint32_t)__builtin_clz(m - 1));
    assert((short_end + nlongs) <= (UINT32_MAX >> bits));
    ENSURE(s->edges, s->edges_cap, m, uint32_t);
    k = 0;
    for (int a = 0; a < 256; a++) {
        size_t from = 0;
        const uint32_t *qa = s->q_buf[a];
        for (size_t bi = cursors[a]; bi < cursors[a + 1]; bi++) {
            Band b = s->bands[bi];
            uint32_t span = b.end - k;
            uint32_t dst = k + b.delta;
            uint32_t real_code = (b.code & (1u << 31)) ? ((uint32_t)short_end + (b.code & 0x7FFFFFFFu)) : b.code;
            uint32_t pack = real_code << bits;
            uint32_t *ed = s->edges + dst;
            const uint32_t *src = qa + from;
            for (uint32_t d = 0; d < span; d++) ed[d] = pack | src[d];
            from += span; k = b.end;
        }
    }

    // Emit with Rytter-Gasieniec Output-Memoized SLP (0 B spelling!)
    uint32_t mask = (1u << bits) - 1;
    uint32_t j = mark;
    size_t at = 0;
    const uint32_t *edges = s->edges;
    const uint32_t short_lim = (uint32_t)short_end;
    for (uint32_t step = 0; step < m; step++) {
        uint32_t edge = edges[j];
        uint32_t code = edge >> bits;
        j = edge & mask;
        if (__builtin_expect(code < short_lim, 1)) {
            uint8_t len = s->lens[code];
            __m128i v = _mm_loadu_si128((const __m128i *)s->inline16[code]);
            _mm_storeu_si128((__m128i *)(out + at), v);
            at += len;
        } else {
            LongHandle lh = s->longs[code - short_lim];
            if (__builtin_expect(at + lh.len <= n, 1)) {
                size_t sp = 0;
                s->stack[sp++] = lh.word << 1;
                while (sp > 0) {
                    uint32_t tagged = s->stack[--sp];
                    uint32_t cur = tagged >> 1;
                    if (cur <= 256) {
                        out[at++] = (uint8_t)(cur - 1);
                    } else {
                        Rule *rl = &s->rules[cur - 257];
                        if (tagged & 1u) {
                            rl->first = (uint32_t)(at - rl->len);
                        } else if (rl->first != UINT32_MAX) {
                            memmove(out + at, out + rl->first, rl->len);
                            at += rl->len;
                        } else {
                            ENSURE(s->stack, s->stk_cap, sp + 3, uint32_t);
                            s->stack[sp++] = (cur << 1) | 1u;
                            s->stack[sp++] = rl->right << 1;
                            s->stack[sp++] = rl->left << 1;
                        }
                    }
                }
                out[at++] = lh.tail;
            } else {
                size_t sp = 0; s->stack[sp++] = lh.word;
                while (sp > 0 && at < n) {
                    uint32_t cur = s->stack[--sp];
                    if (cur <= 256) out[at++] = (uint8_t)(cur - 1);
                    else {
                        Rule *rl = &s->rules[cur - 257];
                        if (rl->first != UINT32_MAX) {
                            size_t take = rl->len < (n - at) ? rl->len : (n - at);
                            memmove(out + at, out + rl->first, take);
                            at += take;
                        } else {
                            ENSURE(s->stack, s->stk_cap, sp + 2, uint32_t);
                            s->stack[sp++] = rl->right; s->stack[sp++] = rl->left;
                        }
                    }
                }
                if (at < n) out[at++] = lh.tail;
            }
        }
        if (j == mark || at >= n) break;
    }
    if (at > n) at = n;
    while (at < n) {
        size_t take = at < (n - at) ? at : (n - at);
        memmove(out + at, out, take);
        at += take;
    }
}

static int cmp_u64(const void *a, const void *b) {
    uint64_t x = *(const uint64_t *)a, y = *(const uint64_t *)b;
    return (x > y) - (x < y);
}

int main(int argc, char **argv) {
    const char *dir = argc > 1 ? argv[1] : "/home/antigravity/admitted-bwt";
    int rounds = argc > 2 ? atoi(argv[2]) : 11;
    const struct { const char *name; uint32_t n, p; } blks[6] = {
        {"block-0.L",   899931, 264045},
        {"block-2.L",   900000, 320807},
        {"block-85.L",  899935, 325088},
        {"block-143.L", 899869, 802868},
        {"block-210.L", 899959, 791429},
        {"block-207.L", 899998, 205180},
    };

    OpenScratch sc_base, sc_om16, sc_om8, sc_om4, sc_om2;
    scratch_init(&sc_base); scratch_init(&sc_om16); scratch_init(&sc_om8);
    scratch_init(&sc_om4);  scratch_init(&sc_om2);

    printf("%-12s | %9s | %9s (%6s) | %9s (%6s) | %9s (%6s) | %9s (%6s)\n",
           "Block", "Open058", "OMFox-16", "/058", "OMFox-8", "/058", "OMFox-4", "/058", "OMFox-2", "/058");
    printf("--------------------------------------------------------------------------------------------------------\n");

    for (int b = 0; b < 6; b++) {
        char path[512];
        snprintf(path, sizeof(path), "%s/%s", dir, blks[b].name);
        FILE *f = fopen(path, "rb");
        assert(f != NULL);
        uint32_t n = blks[b].n, p = blks[b].p;
        uint8_t *L = (uint8_t *)malloc(n);
        assert(fread(L, 1, n, f) == n);
        fclose(f);

        uint16_t *recs = (uint16_t *)malloc(n * sizeof(uint16_t));
        size_t nrecs = build_records(L, n, recs);
        uint8_t *ref = (uint8_t *)malloc(n);
        plain_invert(L, n, p, ref);
        uint8_t *out = (uint8_t *)malloc(n + 32);

        // Byte-exactness verification for all 5 arms
        memset(out, 0xA5, n + 32);
        run_open058_baseline(&sc_base, recs, nrecs, n, p, 16, out);
        assert(memcmp(out, ref, n) == 0);

        const uint32_t lims[4] = {16, 8, 4, 2};
        OpenScratch *scs[4] = {&sc_om16, &sc_om8, &sc_om4, &sc_om2};
        for (int a = 0; a < 4; a++) {
            memset(out, 0xA5, n + 32);
            run_om_fox(scs[a], recs, nrecs, n, p, lims[a], out);
            assert(memcmp(out, ref, n) == 0);
        }

        uint64_t t_base[64], t_om[4][64];
        assert(rounds <= 64);
        for (int r = -2; r < rounds; r++) {
            for (int rot = 0; rot < 5; rot++) {
                int arm = (r + 2 + rot) % 5;
                uint64_t t0 = now_ns();
                if (arm == 0) run_open058_baseline(&sc_base, recs, nrecs, n, p, 16, out);
                else run_om_fox(scs[arm - 1], recs, nrecs, n, p, lims[arm - 1], out);
                uint64_t dt = now_ns() - t0;
                if (r >= 0) {
                    if (arm == 0) t_base[r] = dt;
                    else t_om[arm - 1][r] = dt;
                }
            }
        }
        qsort(t_base, rounds, sizeof(uint64_t), cmp_u64);
        double mb = (double)t_base[rounds / 2] / 1e3;
        double m_om[4];
        for (int a = 0; a < 4; a++) {
            qsort(t_om[a], rounds, sizeof(uint64_t), cmp_u64);
            m_om[a] = (double)t_om[a][rounds / 2] / 1e3;
        }
        printf("%-12s | %7.1fus | %7.1fus (%5.3fx) | %7.1fus (%5.3fx) | %7.1fus (%5.3fx) | %7.1fus (%5.3fx)\n",
               blks[b].name, mb,
               m_om[0], m_om[0]/mb, m_om[1], m_om[1]/mb, m_om[2], m_om[2]/mb, m_om[3], m_om[3]/mb);

        free(L); free(recs); free(ref); free(out);
    }

    printf("\nScratch Footprint After All 6 Blocks (excluding 900 KiB out):\n");
    printf("  Open058<16> : %6.1f KiB\n", scratch_working_set_bytes(&sc_base, 1) / 1024.0);
    printf("  OM-Fox<16>  : %6.1f KiB (0 B spelling!)\n", scratch_working_set_bytes(&sc_om16, 0) / 1024.0);
    printf("  OM-Fox<8>   : %6.1f KiB (0 B spelling!)\n", scratch_working_set_bytes(&sc_om8,  0) / 1024.0);
    printf("  OM-Fox<4>   : %6.1f KiB (0 B spelling!)\n", scratch_working_set_bytes(&sc_om4,  0) / 1024.0);
    printf("  OM-Fox<2>   : %6.1f KiB (0 B spelling!)\n", scratch_working_set_bytes(&sc_om2,  0) / 1024.0);

    scratch_free(&sc_base); scratch_free(&sc_om16); scratch_free(&sc_om8);
    scratch_free(&sc_om4);  scratch_free(&sc_om2);
    return 0;
}
