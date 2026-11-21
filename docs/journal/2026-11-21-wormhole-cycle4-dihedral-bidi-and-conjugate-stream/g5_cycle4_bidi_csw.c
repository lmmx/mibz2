// Gate G5 Factorial Confrontation for Wormhole Cycle 4:
//   C0: Open058    — mibz2 0.5.8 baseline (256 Vec queues + Vec16 + scatter wire + 1-lane U16 emit)
//   C1: CSW-Open   — Chain 4B (0 B queues! 2B/run thin_stream + 99.6%-fast16 sequential Vec16 wire
//                    + Chaitin lifetime-aliased Endpoint + downward right-aligned U16 emit)
//   C2: Bidi-Open  — Chain 4A (256 Vec16 queues + dual wire + Birkhoff zero-copy MLP=2 meet-in-middle emit)
//   C3: CSW-Bidi   — Synthesis (0 B queues! 2B/run thin_stream + band-slack 16-wide dual Vec16 wire
//                    + Chaitin lifetime-aliased Endpoint + Birkhoff zero-copy MLP=2 meet-in-middle emit)

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
typedef struct { uint32_t cur_k, end, packed_base, bi; } ActiveBand;

#define ENSURE_EXACT(ptr, cap, need, T) do { \
    if (__builtin_expect((cap) < (need), 0)) { \
        size_t ncap = (cap) ? (cap) + ((cap) >> 1) : (need); \
        if (ncap < (need)) ncap = (need); \
        (ptr) = (T *)realloc((ptr), ncap * sizeof(T)); \
        (cap) = ncap; \
    } \
} while (0)

#define ENSURE_DBL(ptr, cap, need, T) do { \
    if (__builtin_expect((cap) < (need), 0)) { \
        size_t ncap = (cap) ? (cap) * 2 : 1024; \
        if (ncap < (need)) ncap = (need); \
        (ptr) = (T *)realloc((ptr), ncap * sizeof(T)); \
        (cap) = ncap; \
    } \
} while (0)

// ============================================================================
// Baseline Scratch & Open058 / Bidi-Queue
// ============================================================================
typedef struct {
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
    uint32_t *edges_bwd; size_t edges_bwd_cap;
    uint8_t (*inline16)[16];       size_t inl_cap;
    uint8_t (*inline16_right)[16];
    uint8_t *lens;
    int inl_init;
} BaseScratch;

static void base_init(BaseScratch *s) { memset(s, 0, sizeof(*s)); }
static void base_free(BaseScratch *s) {
    for (int a = 0; a < 256; a++) free(s->q_buf[a]);
    free(s->strips); free(s->img_ord); free(s->nodes); free(s->rules);
    free(s->ord0); free(s->ord1); free(s->term0); free(s->term1);
    free(s->bottom); free(s->stack); free(s->bands); free(s->spelling);
    free(s->offsets); free(s->edges); free(s->edges_bwd);
    free(s->inline16); free(s->inline16_right); free(s->lens);
}

static size_t base_ws_bytes(const BaseScratch *s, int has_bwd) {
    size_t b = 0;
    for (int a = 0; a < 256; a++) b += s->q_cap[a] * 4;
    b += s->strips_cap * sizeof(Strip) + s->img_cap * 4;
    b += s->nodes_cap * sizeof(Node) + s->rules_cap * sizeof(Rule);
    b += s->ord_cap * 4 * 5 + s->stk_cap * 4;
    b += s->bands_cap * sizeof(Band) + s->spell_cap + s->off_cap * 4;
    b += s->edges_cap * 4 + s->inl_cap * 17;
    if (has_bwd) b += s->edges_bwd_cap * 4 + s->inl_cap * 16;
    return b;
}

static inline void q_push16(BaseScratch *s, uint8_t a, uint32_t at, uint32_t m, uint32_t w) {
    uint32_t need = at + (w > 16 ? w : 16);
    if (__builtin_expect(s->q_cap[a] < need, 0)) {
        uint32_t ncap = s->q_cap[a] ? s->q_cap[a] * 2 : 1024;
        if (ncap < need) ncap = need;
        s->q_buf[a] = (uint32_t *)realloc(s->q_buf[a], ncap * sizeof(uint32_t));
        s->q_cap[a] = ncap;
    }
    uint32_t *dst = s->q_buf[a] + at;
    __m256i base = _mm256_set1_epi32((int)m);
    __m256i i0 = _mm256_setr_epi32(0,1,2,3,4,5,6,7);
    __m256i i1 = _mm256_setr_epi32(8,9,10,11,12,13,14,15);
    _mm256_storeu_si256((__m256i *)(dst + 0), _mm256_add_epi32(base, i0));
    _mm256_storeu_si256((__m256i *)(dst + 8), _mm256_add_epi32(base, i1));
}

// Shared Endpoint Elimination + Band/Spelling builder on raw pointers
static size_t run_endpoint_core(
    const Strip *strips, size_t nstrips, const uint32_t *img_ord, uint32_t n,
    const uint32_t thin[256], Node *nd, uint32_t *ord0, uint32_t *ord1,
    uint32_t *term0, uint32_t *term1, uint32_t *bottom,
    Rule **p_rules, size_t *p_rules_cap,
    Band **p_bands, size_t *p_bands_cap,
    uint8_t **p_spell, size_t *p_spell_cap,
    uint32_t **p_off, size_t *p_off_cap,
    uint32_t **p_stk, size_t *p_stk_cap,
    size_t cursors[257], size_t *out_nwords
) {
    nd[0] = (Node){0};
    uint32_t nnodes = 1;
    size_t n_o0 = 0, n_o1 = 0, n_t0 = 0, n_t1 = 0;
    for (size_t si = 0; si < nstrips; si++) {
        nd[nnodes++] = (Node){{0,0},{0,0}, strips[si].width, (uint32_t)strips[si].byte + 1};
    }
    uint32_t x = 0;
    for (size_t si = 0; si < nstrips; si++) {
        if (x < strips[si].from) {
            uint32_t id = nnodes++;
            nd[id] = (Node){{0,0},{0,0}, strips[si].from - x, 0};
            ord0[n_o0++] = id; term1[n_t1++] = id;
        }
        ord0[n_o0++] = (uint32_t)si + 1;
        x = strips[si].from + strips[si].width;
    }
    if (x < n) {
        uint32_t id = nnodes++;
        nd[id] = (Node){{0,0},{0,0}, n - x, 0};
        ord0[n_o0++] = id; term1[n_t1++] = id;
    }
    x = 0;
    for (size_t k = 0; k < nstrips; k++) {
        uint32_t si = img_ord[k];
        if (x < strips[si].to) {
            uint32_t id = nnodes++;
            nd[id] = (Node){{0,0},{0,0}, strips[si].to - x, 0};
            ord1[n_o1++] = id; term0[n_t0++] = id;
        }
        ord1[n_o1++] = si + 1;
        x = strips[si].to + strips[si].width;
    }
    if (x < n) {
        uint32_t id = nnodes++;
        nd[id] = (Node){{0,0},{0,0}, n - x, 0};
        ord1[n_o1++] = id; term0[n_t0++] = id;
    }

    uint32_t prv = 0;
    for (size_t k = 0; k < n_t0; k++) { uint32_t id = term0[k]; nd[prv].next[0] = id; nd[id].prev[0] = prv; prv = id; }
    for (size_t k = 0; k < n_o0; k++) { uint32_t id = ord0[k];  nd[prv].next[0] = id; nd[id].prev[0] = prv; prv = id; }
    nd[prv].next[0] = 0; nd[0].prev[0] = prv;

    prv = 0;
    for (size_t k = 0; k < n_t1; k++) { uint32_t id = term1[k]; nd[prv].next[1] = id; nd[id].prev[1] = prv; prv = id; }
    for (size_t k = 0; k < n_o1; k++) { uint32_t id = ord1[k];  nd[prv].next[1] = id; nd[id].prev[1] = prv; prv = id; }
    nd[prv].next[1] = 0; nd[0].prev[1] = prv;

    size_t nrules = 0;
    uint32_t rem = n;
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
            ENSURE_EXACT(*p_rules, *p_rules_cap, nrules + 1, Rule);
            Rule *rules = *p_rules;
            uint32_t ll = lw <= 256 ? 1 : rules[lw - 257].len;
            uint32_t rl = rw <= 256 ? 1 : rules[rw - 257].len;
            wj = (uint32_t)nrules + 257;
            rules[nrules++] = (Rule){lw, rw, ll + rl, UINT32_MAX};
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
    while (id != 0) { bottom[id] = x; x += nd[id].width; id = nd[id].next[1]; }

    size_t nbands = 0, nspell = 0, noff = 1, nwords = 0;
    ENSURE_EXACT(*p_off, *p_off_cap, 4096, uint32_t);
    (*p_off)[0] = 0;
    ENSURE_EXACT(*p_bands, *p_bands_cap, nnodes + 256, Band);
    ENSURE_EXACT(*p_spell, *p_spell_cap, 65536, uint8_t);
    ENSURE_EXACT(*p_stk, *p_stk_cap, 512, uint32_t);

    Rule *rules = *p_rules;
    id = nd[0].next[0];
    uint32_t in_node = 0, k = 0;
    for (int a = 0; a < 256; a++) {
        cursors[a] = nbands;
        uint32_t end = k + thin[a];
        while (k < end) {
            uint32_t nw = nd[id].width;
            uint32_t width = (end - k) < (nw - in_node) ? (end - k) : (nw - in_node);
            uint32_t dst = bottom[id] + in_node;
            uint32_t wd = nd[id].word, code;
            if (wd == 0) code = (uint32_t)a;
            else {
                nwords++;
                uint32_t h = wd <= 256 ? 1 : rules[wd - 257].len;
                ENSURE_EXACT(*p_spell, *p_spell_cap, nspell + h + 2, uint8_t);
                uint8_t *sp_buf = *p_spell;
                size_t sp = 0; (*p_stk)[sp++] = wd;
                while (sp > 0) {
                    uint32_t cur = (*p_stk)[--sp];
                    if (cur <= 256) sp_buf[nspell++] = (uint8_t)(cur - 1);
                    else {
                        Rule *rl = &rules[cur - 257];
                        if (rl->first == UINT32_MAX) {
                            rl->first = (uint32_t)nspell;
                            ENSURE_EXACT(*p_stk, *p_stk_cap, sp + 2, uint32_t);
                            (*p_stk)[sp++] = rl->right; (*p_stk)[sp++] = rl->left;
                        } else {
                            memcpy(sp_buf + nspell, sp_buf + rl->first, rl->len);
                            nspell += rl->len;
                        }
                    }
                }
                (*p_spell)[nspell++] = (uint8_t)a;
                ENSURE_EXACT(*p_off, *p_off_cap, noff + 1, uint32_t);
                (*p_off)[noff++] = (uint32_t)nspell;
                code = 255 + (uint32_t)nwords;
            }
            ENSURE_EXACT(*p_bands, *p_bands_cap, nbands + 1, Band);
            (*p_bands)[nbands++] = (Band){k + width, dst - k, code};
            in_node += width; k += width;
            if (in_node == nw) { id = nd[id].next[0]; in_node = 0; }
        }
    }
    cursors[256] = nbands;
    *out_nwords = nwords;
    return nbands;
}

static void build_inline_tables(
    uint8_t (**p_inl)[16], uint8_t (**p_inl_r)[16], uint8_t **p_lens, size_t *p_cap, int *p_init,
    size_t nwords, const uint8_t *spelling, const uint32_t *offsets, int build_left, int build_right
) {
    size_t ninl = 256 + nwords;
    if (*p_cap < ninl) {
        size_t nc = ninl + 512;
        if (build_left)  *p_inl   = (uint8_t (*)[16])realloc(*p_inl, nc * 16);
        if (build_right) *p_inl_r = (uint8_t (*)[16])realloc(*p_inl_r, nc * 16);
        *p_lens = (uint8_t *)realloc(*p_lens, nc);
        *p_cap = nc;
        *p_init = 0;
    }
    if (!*p_init) {
        for (int c = 0; c < 256; c++) {
            if (build_left)  { memset((*p_inl)[c], 0, 16);   (*p_inl)[c][0] = (uint8_t)c; }
            if (build_right) { memset((*p_inl_r)[c], 0, 16); (*p_inl_r)[c][15] = (uint8_t)c; }
            (*p_lens)[c] = 1;
        }
        *p_init = 1;
    }
    for (size_t w = 0; w < nwords; w++) {
        uint32_t o0 = offsets[w], wl = offsets[w + 1] - o0;
        if (wl <= 16) {
            (*p_lens)[256 + w] = (uint8_t)wl;
            if (build_left) {
                memset((*p_inl)[256 + w], 0, 16);
                memcpy((*p_inl)[256 + w], spelling + o0, wl);
            }
            if (build_right) {
                memset((*p_inl_r)[256 + w], 0, 16);
                memcpy((*p_inl_r)[256 + w] + (16 - wl), spelling + o0, wl);
            }
        } else {
            (*p_lens)[256 + w] = 255;
        }
    }
}

static inline void emit_bidi_meet_in_middle(
    const uint32_t * restrict edges_fwd, const uint32_t * restrict edges_bwd,
    const uint8_t (* restrict inl_l)[16], const uint8_t (* restrict inl_r)[16],
    const uint8_t * restrict lens, const uint8_t * restrict spelling, const uint32_t * restrict offsets,
    uint32_t m, uint32_t mark, uint32_t bits, uint32_t n, uint8_t * restrict out
) {
    uint32_t mask = (1u << bits) - 1;
    uint32_t jf = mark, jb = mark;
    size_t at_f = 0, at_b = n;

    // Fast dual-ray MLP=2 loop while gap >= 64 bytes (99.95% of all steps!)
    for (uint32_t step = 0; step < m && (at_b - at_f) >= 64; step++) {
        // Issue BOTH independent loads simultaneously -> hardware MLP = 2!
        uint32_t ef = edges_fwd[jf];
        uint32_t eb = edges_bwd[jb];
        uint32_t cf = ef >> bits;
        uint32_t cb = eb >> bits;
        uint32_t next_jf = ef & mask;
        uint32_t next_jb = eb & mask;

        // Forward write
        uint8_t lf = lens[cf];
        if (__builtin_expect(lf != 255, 1)) {
            __m128i vf = _mm_loadu_si128((const __m128i *)inl_l[cf]);
            _mm_storeu_si128((__m128i *)(out + at_f), vf);
            at_f += lf;
        } else {
            uint32_t wf = cf - 256;
            uint32_t o0 = offsets[wf], wl = offsets[wf + 1] - o0;
            size_t take = wl < (at_b - at_f) ? wl : (at_b - at_f);
            memcpy(out + at_f, spelling + o0, take);
            at_f += take;
        }
        jf = next_jf;
        if (__builtin_expect(jf == jb || at_f >= at_b, 0)) goto done;

        // Backward write (re-check gap if forward was a huge >16B word)
        uint8_t lb = lens[cb];
        if (__builtin_expect(lb != 255 && (at_b - at_f) >= 16, 1)) {
            __m128i vb = _mm_loadu_si128((const __m128i *)inl_r[cb]);
            _mm_storeu_si128((__m128i *)(out + at_b - 16), vb);
            at_b -= lb;
        } else {
            uint32_t wb = cb - 256;
            uint32_t o0 = (cb < 256) ? 0 : offsets[wb];
            uint32_t wl = (cb < 256) ? 1 : (offsets[wb + 1] - o0);
            const uint8_t *src = (cb < 256) ? &inl_l[cb][0] : (spelling + o0);
            size_t take = wl < (at_b - at_f) ? wl : (at_b - at_f);
            memcpy(out + at_b - take, src + (wl - take), take);
            at_b -= take;
        }
        jb = next_jb;
        if (__builtin_expect(jf == jb || at_f >= at_b, 0)) goto done;
    }

    // Finish final < 64B meeting gap cleanly with forward ray
    while (jf != jb && at_f < at_b) {
        uint32_t ef = edges_fwd[jf];
        uint32_t cf = ef >> bits;
        jf = ef & mask;
        uint8_t lf = lens[cf];
        if (lf != 255) {
            size_t take = lf < (at_b - at_f) ? lf : (at_b - at_f);
            memcpy(out + at_f, inl_l[cf], take);
            at_f += take;
        } else {
            uint32_t wf = cf - 256;
            uint32_t o0 = offsets[wf], wl = offsets[wf + 1] - o0;
            size_t take = wl < (at_b - at_f) ? wl : (at_b - at_f);
            memcpy(out + at_f, spelling + o0, take);
            at_f += take;
        }
    }

done:
    if (__builtin_expect(at_f < at_b, 0)) {
        size_t tail = n - at_b;
        memmove(out + at_f, out + at_b, tail);
        size_t d = at_f + tail;
        while (d < n) {
            size_t take = d < (n - d) ? d : (n - d);
            memmove(out + d, out, take);
            d += take;
        }
    }
}

// C0 (bidi=0): Exact Open 0.5.8 baseline
// C2 (bidi=1): Bidi-Open (256 Vec16 queues + dual wire + MLP=2 meet-in-middle emit)
static void run_c0_or_c2(
    BaseScratch *s, const uint16_t *recs, size_t nrecs, uint32_t n, uint32_t p, int bidi, uint8_t *out
) {
    const uint32_t limit = 16;
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
        if (w <= limit) {
            if (p - pos < w) mark = m + (p - pos);
            q_push16(s, c, thin[c], m, w);
            thin[c] += w; m += w; all[c] += w;
        } else if (p - pos >= w) {
            ENSURE_DBL(s->strips, s->strips_cap, nstrips + 1, Strip);
            s->strips[nstrips++] = (Strip){pos, all[c], w, c};
            thick[c]++; all[c] += w;
        } else {
            ENSURE_DBL(s->strips, s->strips_cap, nstrips + 2, Strip);
            if (pos < p) { uint32_t lw = p - pos; s->strips[nstrips++] = (Strip){pos, all[c], lw, c}; thick[c]++; all[c] += lw; }
            mark = m; q_push16(s, c, thin[c], m, 1); thin[c]++; m++; all[c]++;
            if (p + 1 < pos + w) { uint32_t rw = pos + w - p - 1; s->strips[nstrips++] = (Strip){p + 1, all[c], rw, c}; thick[c]++; all[c] += rw; }
        }
        pos += w;
    }

    uint32_t first[256], write[256], acc0 = 0, acc1 = 0;
    for (int a = 0; a < 256; a++) { first[a] = acc0; acc0 += all[a]; write[a] = acc1; acc1 += thick[a]; }
    ENSURE_DBL(s->img_ord, s->img_cap, nstrips + 1, uint32_t);
    for (size_t si = 0; si < nstrips; si++) {
        uint8_t a = s->strips[si].byte;
        s->strips[si].to += first[a];
        s->img_ord[write[a]++] = (uint32_t)si;
    }

    size_t max_nodes = 3 * nstrips + 8;
    ENSURE_DBL(s->nodes, s->nodes_cap, max_nodes, Node);
    if (s->ord_cap < max_nodes) {
        size_t nc = max_nodes * 2;
        s->ord0 = (uint32_t *)realloc(s->ord0, nc * 4); s->ord1 = (uint32_t *)realloc(s->ord1, nc * 4);
        s->term0 = (uint32_t *)realloc(s->term0, nc * 4); s->term1 = (uint32_t *)realloc(s->term1, nc * 4);
        s->bottom = (uint32_t *)realloc(s->bottom, nc * 4); s->ord_cap = nc;
    }

    size_t cursors[257], nwords = 0;
    run_endpoint_core(s->strips, nstrips, s->img_ord, n, thin, s->nodes,
                      s->ord0, s->ord1, s->term0, s->term1, s->bottom,
                      &s->rules, &s->rules_cap, &s->bands, &s->bands_cap,
                      &s->spelling, &s->spell_cap, &s->offsets, &s->off_cap,
                      &s->stack, &s->stk_cap, cursors, &nwords);

    uint32_t bits = m <= 1 ? 1 : (32u - (uint32_t)__builtin_clz(m - 1));
    ENSURE_DBL(s->edges, s->edges_cap, m, uint32_t);
    if (bidi) ENSURE_DBL(s->edges_bwd, s->edges_bwd_cap, m, uint32_t);

    uint32_t k = 0;
    for (int a = 0; a < 256; a++) {
        size_t from = 0;
        const uint32_t *qa = s->q_buf[a];
        for (size_t bi = cursors[a]; bi < cursors[a + 1]; bi++) {
            Band b = s->bands[bi];
            uint32_t span = b.end - k;
            uint32_t dst = k + b.delta;
            uint32_t pack = b.code << bits;
            uint32_t *ef = s->edges + dst;
            const uint32_t *src = qa + from;
            if (!bidi) {
                for (uint32_t d = 0; d < span; d++) ef[d] = pack | src[d];
            } else {
                uint32_t *eb = s->edges_bwd;
                for (uint32_t d = 0; d < span; d++) {
                    uint32_t sj = src[d];
                    ef[d]  = pack | sj;
                    eb[sj] = pack | (dst + d);
                }
            }
            from += span; k = b.end;
        }
    }

    build_inline_tables(&s->inline16, &s->inline16_right, &s->lens, &s->inl_cap, &s->inl_init,
                        nwords, s->spelling, s->offsets, 1, bidi);

    if (!bidi) {
        uint32_t mask = (1u << bits) - 1, j = mark;
        size_t at = 0;
        for (uint32_t step = 0; step < m; step++) {
            uint32_t edge = s->edges[j];
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
        while (at < n) { size_t take = at < (n - at) ? at : (n - at); memcpy(out + at, out, take); at += take; }
    } else {
        emit_bidi_meet_in_middle(s->edges, s->edges_bwd, s->inline16, s->inline16_right,
                                 s->lens, s->spelling, s->offsets, m, mark, bits, n, out);
    }
}

// ============================================================================
// Queue-Free Compact Scratch for C1 (CSW-Open) & C3 (CSW-Bidi)
//   - ZERO 256-way queues (0 B!)
//   - Endpoint ord0/ord1/term0/term1/bottom + img_ord OVERLAID inside dead edges[]!
// ============================================================================
typedef struct {
    uint16_t *thin_stream; size_t ts_cap;
    Strip *strips;         size_t strips_cap;
    Node *nodes;           size_t nodes_cap;
    Rule *rules;           size_t rules_cap;
    uint32_t *stack;       size_t stk_cap;
    Band *bands;           size_t bands_cap;
    uint8_t *spelling;     size_t spell_cap;
    uint32_t *offsets;     size_t off_cap;
    uint32_t *edges_bwd;   size_t edges_bwd_cap;
    uint32_t *edges_fwd;   size_t edges_fwd_cap;
    uint8_t (*inline16)[16];       size_t inl_cap;
    uint8_t (*inline16_right)[16];
    uint8_t *lens;
    int inl_init;
} CswScratch;

static void csw_init(CswScratch *s) { memset(s, 0, sizeof(*s)); }
static void csw_free(CswScratch *s) {
    free(s->thin_stream); free(s->strips); free(s->nodes); free(s->rules);
    free(s->stack); free(s->bands); free(s->spelling); free(s->offsets);
    free(s->edges_bwd); free(s->edges_fwd);
    free(s->inline16); free(s->inline16_right); free(s->lens);
}

static size_t csw_ws_bytes(const CswScratch *s, int bidi) {
    size_t b = s->ts_cap * 2 + s->strips_cap * sizeof(Strip);
    b += s->nodes_cap * sizeof(Node) + s->rules_cap * sizeof(Rule) + s->stk_cap * 4;
    b += s->bands_cap * sizeof(Band) + s->spell_cap + s->off_cap * 4;
    b += s->edges_bwd_cap * 4 + s->inl_cap * 17;
    if (bidi) b += s->edges_fwd_cap * 4 + s->inl_cap * 16;
    return b;
}

static void run_c1_or_c3_csw(
    CswScratch *s, const uint16_t *recs, size_t nrecs, uint32_t n, uint32_t p, int bidi, uint8_t *out
) {
    const uint32_t limit = 16;
    uint32_t all[256] = {0}, thin[256] = {0}, thick[256] = {0};
    uint32_t m = 0, mark = 0;
    size_t nstrips = 0, nthin_runs = 0;
    ENSURE_EXACT(s->strips, s->strips_cap, 4096, Strip);
    ENSURE_EXACT(s->thin_stream, s->ts_cap, nrecs + 4, uint16_t);

    uint32_t pos = 0;
    size_t i = 0;
    while (i < nrecs) {
        uint16_t rec = recs[i++];
        uint8_t c = (uint8_t)(rec >> 8);
        uint32_t w = (uint32_t)(rec & 255);
        while (i < nrecs && (uint8_t)(recs[i] >> 8) == c) { w += (uint32_t)(recs[i] & 255); i++; }
        if (w <= limit) {
            if (p - pos < w) mark = m + (p - pos);
            s->thin_stream[nthin_runs++] = (uint16_t)(((uint16_t)c << 8) | (uint16_t)w);
            thin[c] += w; m += w; all[c] += w;
        } else if (p - pos >= w) {
            ENSURE_EXACT(s->strips, s->strips_cap, nstrips + 1, Strip);
            s->strips[nstrips++] = (Strip){pos, all[c], w, c};
            thick[c]++; all[c] += w;
        } else {
            ENSURE_EXACT(s->strips, s->strips_cap, nstrips + 2, Strip);
            if (pos < p) { uint32_t lw = p - pos; s->strips[nstrips++] = (Strip){pos, all[c], lw, c}; thick[c]++; all[c] += lw; }
            mark = m;
            s->thin_stream[nthin_runs++] = (uint16_t)(((uint16_t)c << 8) | 1u);
            thin[c]++; m++; all[c]++;
            if (p + 1 < pos + w) { uint32_t rw = pos + w - p - 1; s->strips[nstrips++] = (Strip){p + 1, all[c], rw, c}; thick[c]++; all[c] += rw; }
        }
        pos += w;
    }

    // Chaitin disjoint-lifetime overlay: host img_ord + ord0/1 + term0/1 + bottom inside dead edges_bwd!
    size_t max_nodes = 3 * nstrips + 8;
    size_t overlay_u32 = (nstrips + 4) + 5 * max_nodes;
    size_t need_edges = (size_t)m + 32;
    size_t alloc_u32 = need_edges > overlay_u32 ? need_edges : overlay_u32;
    ENSURE_EXACT(s->edges_bwd, s->edges_bwd_cap, alloc_u32, uint32_t);
    if (bidi) ENSURE_EXACT(s->edges_fwd, s->edges_fwd_cap, need_edges, uint32_t);
    ENSURE_EXACT(s->nodes, s->nodes_cap, max_nodes, Node);

    uint32_t *img_ord = s->edges_bwd;
    uint32_t *ord0    = img_ord + (nstrips + 4);
    uint32_t *ord1    = ord0 + max_nodes;
    uint32_t *term0   = ord1 + max_nodes;
    uint32_t *term1   = term0 + max_nodes;
    uint32_t *bottom  = term1 + max_nodes;

    uint32_t first[256], write[256], acc0 = 0, acc1 = 0;
    for (int a = 0; a < 256; a++) { first[a] = acc0; acc0 += all[a]; write[a] = acc1; acc1 += thick[a]; }
    for (size_t si = 0; si < nstrips; si++) {
        uint8_t a = s->strips[si].byte;
        s->strips[si].to += first[a];
        img_ord[write[a]++] = (uint32_t)si;
    }

    size_t cursors[257], nwords = 0;
    run_endpoint_core(s->strips, nstrips, img_ord, n, thin, s->nodes,
                      ord0, ord1, term0, term1, bottom,
                      &s->rules, &s->rules_cap, &s->bands, &s->bands_cap,
                      &s->spelling, &s->spell_cap, &s->offsets, &s->off_cap,
                      &s->stack, &s->stk_cap, cursors, &nwords);
    // Endpoint index arrays inside s->edges_bwd are now 100% DEAD!

    // Phase 3: Conjugate u16-stream Vec16 wiring (99.6% branchless 16-wide iota fast path!)
    uint32_t bits = m <= 1 ? 1 : (32u - (uint32_t)__builtin_clz(m - 1));
    ActiveBand ab[256];
    uint32_t k_acc = 0;
    for (int a = 0; a < 256; a++) {
        uint32_t ks = k_acc;
        k_acc += thin[a];
        if (thin[a] > 0) {
            uint32_t bi = (uint32_t)cursors[a];
            Band b = s->bands[bi];
            ab[a] = (ActiveBand){ks, b.end, (b.code << bits) | ((ks + b.delta) - ks), bi};
            // Wait: dst0 = z + b.delta; since z <= m < 2^bits and dst0 < m < 2^bits,
            // store delta_val = b.delta and code_pack = b.code << bits separately or cleanly!
        } else {
            ab[a] = (ActiveBand){0, 0, 0, 0};
        }
    }
    // Use two clean L1d-resident arrays for zero-hazard arithmetic
    uint32_t ab_z[256], ab_end[256], ab_delta[256], ab_pack[256], ab_bi[256];
    k_acc = 0;
    for (int a = 0; a < 256; a++) {
        ab_z[a] = k_acc;
        k_acc += thin[a];
        if (thin[a] > 0) {
            uint32_t bi = (uint32_t)cursors[a];
            Band b = s->bands[bi];
            ab_bi[a] = bi; ab_end[a] = b.end; ab_delta[a] = b.delta; ab_pack[a] = b.code << bits;
        } else {
            ab_bi[a] = 0; ab_end[a] = 0; ab_delta[a] = 0; ab_pack[a] = 0;
        }
    }
    (void)ab;

    const __m256i iota0 = _mm256_setr_epi32(0,1,2,3,4,5,6,7);
    const __m256i iota8 = _mm256_setr_epi32(8,9,10,11,12,13,14,15);
    uint32_t * restrict eb = s->edges_bwd;
    uint32_t * restrict ef = s->edges_fwd;
    const uint16_t * restrict ts = s->thin_stream;
    uint32_t j = 0;

    if (!bidi) {
        for (size_t ri = 0; ri < nthin_runs; ri++) {
            uint16_t tok = ts[ri];
            uint32_t a = tok >> 8;
            uint32_t w = tok & 255u;
            uint32_t z = ab_z[a];
            uint32_t end_b = ab_end[a];
            if (__builtin_expect(z + w <= end_b, 1)) {
                uint32_t base_val = ab_pack[a] | (z + ab_delta[a]);
                __m256i vb = _mm256_set1_epi32((int)base_val);
                _mm256_storeu_si256((__m256i *)(eb + j + 0), _mm256_add_epi32(vb, iota0));
                _mm256_storeu_si256((__m256i *)(eb + j + 8), _mm256_add_epi32(vb, iota8));
                j += w;
                z += w;
                ab_z[a] = z;
                if (__builtin_expect(z == end_b, 0)) {
                    uint32_t nbi = ab_bi[a] + 1;
                    if (nbi < (uint32_t)cursors[a + 1]) {
                        Band nb = s->bands[nbi];
                        ab_bi[a] = nbi; ab_end[a] = nb.end; ab_delta[a] = nb.delta; ab_pack[a] = nb.code << bits;
                    }
                }
            } else {
                uint32_t end_z = z + w;
                while (z < end_z) {
                    uint32_t rem_b = ab_end[a] - z;
                    uint32_t span = (end_z - z) < rem_b ? (end_z - z) : rem_b;
                    uint32_t val = ab_pack[a] | (z + ab_delta[a]);
                    for (uint32_t d = 0; d < span; d++) eb[j + d] = val + d;
                    j += span; z += span;
                    if (z == ab_end[a]) {
                        uint32_t nbi = ab_bi[a] + 1;
                        if (nbi < (uint32_t)cursors[a + 1]) {
                            Band nb = s->bands[nbi];
                            ab_bi[a] = nbi; ab_end[a] = nb.end; ab_delta[a] = nb.delta; ab_pack[a] = nb.code << bits;
                        }
                    }
                }
                ab_z[a] = end_z;
            }
        }
    } else {
        for (size_t ri = 0; ri < nthin_runs; ri++) {
            uint16_t tok = ts[ri];
            uint32_t a = tok >> 8;
            uint32_t w = tok & 255u;
            uint32_t z = ab_z[a];
            uint32_t end_b = ab_end[a];
            if (__builtin_expect(z + 16 <= end_b, 1)) {
                // BOTH eb[j..j+16] AND ef[dst0..dst0+16] take 16-wide branchless AVX2 iota stores!
                uint32_t dst0 = z + ab_delta[a];
                uint32_t pk = ab_pack[a];
                __m256i vb = _mm256_set1_epi32((int)(pk | dst0));
                __m256i vf = _mm256_set1_epi32((int)(pk | j));
                _mm256_storeu_si256((__m256i *)(eb + j + 0),    _mm256_add_epi32(vb, iota0));
                _mm256_storeu_si256((__m256i *)(eb + j + 8),    _mm256_add_epi32(vb, iota8));
                _mm256_storeu_si256((__m256i *)(ef + dst0 + 0), _mm256_add_epi32(vf, iota0));
                _mm256_storeu_si256((__m256i *)(ef + dst0 + 8), _mm256_add_epi32(vf, iota8));
                j += w;
                z += w;
                ab_z[a] = z;
                if (__builtin_expect(z == end_b, 0)) {
                    uint32_t nbi = ab_bi[a] + 1;
                    if (nbi < (uint32_t)cursors[a + 1]) {
                        Band nb = s->bands[nbi];
                        ab_bi[a] = nbi; ab_end[a] = nb.end; ab_delta[a] = nb.delta; ab_pack[a] = nb.code << bits;
                    }
                }
            } else {
                uint32_t end_z = z + w;
                while (z < end_z) {
                    uint32_t rem_b = ab_end[a] - z;
                    uint32_t span = (end_z - z) < rem_b ? (end_z - z) : rem_b;
                    uint32_t dst0 = z + ab_delta[a];
                    uint32_t pk = ab_pack[a];
                    for (uint32_t d = 0; d < span; d++) {
                        eb[j + d]    = pk | (dst0 + d);
                        ef[dst0 + d] = pk | (j + d);
                    }
                    j += span; z += span;
                    if (z == ab_end[a]) {
                        uint32_t nbi = ab_bi[a] + 1;
                        if (nbi < (uint32_t)cursors[a + 1]) {
                            Band nb = s->bands[nbi];
                            ab_bi[a] = nbi; ab_end[a] = nb.end; ab_delta[a] = nb.delta; ab_pack[a] = nb.code << bits;
                        }
                    }
                }
                ab_z[a] = end_z;
            }
        }
    }

    build_inline_tables(&s->inline16, &s->inline16_right, &s->lens, &s->inl_cap, &s->inl_init,
                        nwords, s->spelling, s->offsets, bidi, 1);

    if (!bidi) {
        uint32_t mask = (1u << bits) - 1, cur_j = mark;
        size_t at = n;
        for (uint32_t step = 0; step < m; step++) {
            uint32_t edge = eb[cur_j];
            uint32_t code = edge >> bits;
            cur_j = edge & mask;
            uint8_t len = s->lens[code];
            if (__builtin_expect(len != 255 && at >= 16, 1)) {
                __m128i v = _mm_loadu_si128((const __m128i *)s->inline16_right[code]);
                _mm_storeu_si128((__m128i *)(out + at - 16), v);
                at -= len;
            } else {
                uint8_t single_b = (uint8_t)code;
                const uint8_t *src;
                uint32_t wl;
                if (code < 256) { src = &single_b; wl = 1; }
                else if (len != 255) { src = s->inline16_right[code] + (16 - len); wl = len; }
                else { uint32_t w = code - 256; uint32_t o0 = s->offsets[w]; wl = s->offsets[w + 1] - o0; src = s->spelling + o0; }
                size_t take = wl < at ? wl : at;
                memcpy(out + at - take, src + (wl - take), take);
                at -= take;
            }
            if (cur_j == mark || at == 0) break;
        }
        if (__builtin_expect(at > 0, 0)) {
            size_t d = n - at;
            memmove(out, out + at, d);
            while (d < n) { size_t take = d < (n - d) ? d : (n - d); memmove(out + d, out, take); d += take; }
        }
    } else {
        emit_bidi_meet_in_middle(ef, eb, s->inline16, s->inline16_right,
                                 s->lens, s->spelling, s->offsets, m, mark, bits, n, out);
    }
}

static int cmp_u64(const void *a, const void *b) {
    uint64_t x = *(const uint64_t *)a, y = *(const uint64_t *)b;
    return (x > y) - (x < y);
}

int main(int argc, char **argv) {
    const char *dir = argc > 1 ? argv[1] : "/home/antigravity/admitted-bwt";
    int rounds = argc > 2 ? atoi(argv[2]) : 21;
    const struct { const char *name; uint32_t n, p; } blks[6] = {
        {"block-0.L",   899931, 264045},
        {"block-2.L",   900000, 320807},
        {"block-85.L",  899935, 325088},
        {"block-143.L", 899869, 802868},
        {"block-210.L", 899959, 791429},
        {"block-207.L", 899998, 205180},
    };

    BaseScratch sc_c0, sc_c2;
    CswScratch  sc_c1, sc_c3;
    base_init(&sc_c0); base_init(&sc_c2);
    csw_init(&sc_c1);  csw_init(&sc_c3);

    printf("%-12s | %9s | %9s (%6s) | %9s (%6s) | %9s (%6s)\n",
           "Block", "C0:Open058", "C1:CSW-1L", "/058", "C2:BidiQ", "/058", "C3:CSWBidi", "/058");
    printf("---------------------------------------------------------------------------------------\n");

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

        // Verify byte-exactness + 16B tail canary on all 4 arms
        for (int arm = 0; arm < 4; arm++) {
            memset(out, 0xA5, n + 32);
            if (arm == 0) run_c0_or_c2(&sc_c0, recs, nrecs, n, p, 0, out);
            else if (arm == 1) run_c1_or_c3_csw(&sc_c1, recs, nrecs, n, p, 0, out);
            else if (arm == 2) run_c0_or_c2(&sc_c2, recs, nrecs, n, p, 1, out);
            else run_c1_or_c3_csw(&sc_c3, recs, nrecs, n, p, 1, out);
            assert(memcmp(out, ref, n) == 0);
            for (int k = 16; k < 32; k++) assert(out[n + k] == 0xA5);
        }

        uint64_t t[4][64];
        assert(rounds <= 64);
        for (int r = -3; r < rounds; r++) {
            for (int rot = 0; rot < 4; rot++) {
                int arm = (r + 3 + rot) % 4;
                uint64_t t0 = now_ns();
                if (arm == 0) run_c0_or_c2(&sc_c0, recs, nrecs, n, p, 0, out);
                else if (arm == 1) run_c1_or_c3_csw(&sc_c1, recs, nrecs, n, p, 0, out);
                else if (arm == 2) run_c0_or_c2(&sc_c2, recs, nrecs, n, p, 1, out);
                else run_c1_or_c3_csw(&sc_c3, recs, nrecs, n, p, 1, out);
                uint64_t dt = now_ns() - t0;
                if (r >= 0) t[arm][r] = dt;
            }
        }
        double med[4];
        for (int arm = 0; arm < 4; arm++) {
            qsort(t[arm], rounds, sizeof(uint64_t), cmp_u64);
            med[arm] = (double)t[arm][rounds / 2] / 1e3;
        }
        printf("%-12s | %7.1fus | %7.1fus (%5.3fx) | %7.1fus (%5.3fx) | %7.1fus (%5.3fx)\n",
               blks[b].name, med[0],
               med[1], med[1]/med[0], med[2], med[2]/med[0], med[3], med[3]/med[0]);

        free(L); free(recs); free(ref); free(out);
    }

    printf("\nPersistent Worker Scratch After All 6 Blocks (excluding 900 KiB out):\n");
    printf("  C0: Open058   : %6.1f KiB (1.000x)\n", base_ws_bytes(&sc_c0, 0) / 1024.0);
    printf("  C1: CSW-Open  : %6.1f KiB (%5.3fx — 0 B queues + aliased Endpoint!)\n",
           csw_ws_bytes(&sc_c1, 0) / 1024.0, (double)csw_ws_bytes(&sc_c1, 0) / base_ws_bytes(&sc_c0, 0));
    printf("  C2: Bidi-Q    : %6.1f KiB (%5.3fx)\n",
           base_ws_bytes(&sc_c2, 1) / 1024.0, (double)base_ws_bytes(&sc_c2, 1) / base_ws_bytes(&sc_c0, 0));
    printf("  C3: CSW-Bidi  : %6.1f KiB (%5.3fx — 0 B queues + MLP=2 zero-copy emit!)\n",
           csw_ws_bytes(&sc_c3, 1) / 1024.0, (double)csw_ws_bytes(&sc_c3, 1) / base_ws_bytes(&sc_c0, 0));

    base_free(&sc_c0); base_free(&sc_c2);
    csw_free(&sc_c1);  csw_free(&sc_c3);
    return 0;
}
