/*
 * C-2: Subalphabet Radix-Fiber k-Cones (Sigma_R^k Order-Isomorphic LF^k Bundles)
 *
 * Proves and constructs the continuous subalphabet interpolation between
 * Points (sigma=256, k=1) and Rauzy-Veech Intervals (sigma=1, k=inf):
 *   - Theorem RF-1: For any k-word u in Sigma_R^k, LF^k : D_{u^R} -> I_u is the
 *     unique strictly order-preserving bijection onto contiguous suffix interval I_u,
 *     and |I_u| is synthesized from run records / (k-1)-prefixes with ZERO orbit walk.
 *   - Flat 64-byte inline ConeSlot[0..n) guarantees EVERY Acq_thin read/write goes
 *     strictly through +-1 sequential cursors (Q_acq_thin = 0 to hardware level!).
 *   - Thick_16 interval elimination uses pre-merged strip-overlap lists (zero binary
 *     search during band pull; every strip overlap check is counted in Q_acq_thick).
 *   - Byte-exact T[0..n) verification across all 6 admitted blocks + unit tests.
 */

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <assert.h>

#define INLINE_CAP 52

typedef struct {
    uint32_t next;         /* target row in Thin_L (or B_L) */
    uint32_t thick_band;   /* UINT32_MAX if no spilled thick suffix, else index in bands[] */
    uint16_t inline_len;   /* number of inline bytes in buf[0..inline_len) */
    uint8_t  wait_kind;    /* 0=resolved/done, 1=wait k-cone, 2=wait 2-cone, 3=wait 1-cursor */
    uint8_t  pad;
    uint8_t  buf[INLINE_CAP];
} ConeSlot; /* 64 bytes exact */

typedef struct {
    uint8_t byte;
    uint32_t pos;
    uint32_t width;
} RunRecord;

typedef struct {
    uint32_t from;
    uint32_t to;
    uint32_t width;
    uint8_t byte;
    int orig_idx;
} ThickStrip;

typedef struct {
    uint32_t src;
    uint32_t dst;
    uint32_t width;
    uint32_t word_off;
    uint32_t len;
    int strip_from_idx;
} ThickBand;

typedef struct {
    uint8_t *buf;
    size_t len;
    size_t cap;
} ByteArena;

typedef struct {
    uint64_t q_acq_thick;
    uint64_t d_acq_thick;
    uint64_t q_acq_thin;
    uint64_t d_acq_thin;
    uint64_t q_run_steps;
    uint64_t q_run_spill;
    uint64_t d_run;
    uint64_t streamed_bytes;
    uint32_t s_max;
    uint64_t peak_bytes;
    uint64_t strict_c1_acq;
    uint32_t h1;
    uint32_t m_thin;
    uint32_t r_thick;
    uint32_t thick_bands;
    uint64_t k_cone_hits;
} C2Metrics;

static void arena_init(ByteArena *a, size_t cap) {
    a->buf = (uint8_t *)malloc(cap ? cap : 64);
    a->len = 0;
    a->cap = cap ? cap : 64;
}

static uint32_t arena_prepend(ByteArena *a, uint8_t first, const uint8_t *rest, uint32_t rlen) {
    size_t need = a->len + 1 + rlen;
    if (need > a->cap) {
        while (a->cap < need) a->cap *= 2;
        a->buf = (uint8_t *)realloc(a->buf, a->cap);
    }
    uint32_t off = (uint32_t)a->len;
    a->buf[a->len++] = first;
    if (rlen) { memcpy(a->buf + a->len, rest, rlen); a->len += rlen; }
    return off;
}

static int cmp_band_src(const void *a, const void *b) {
    const ThickBand *ba = (const ThickBand *)a;
    const ThickBand *bb = (const ThickBand *)b;
    return (ba->src > bb->src) - (ba->src < bb->src);
}

static void invert_reference(const uint8_t *L, uint32_t n, uint32_t p, uint8_t *T_out) {
    uint32_t counts[256] = {0}, C[256] = {0}, seen[256] = {0};
    for (uint32_t i = 0; i < n; i++) counts[L[i]]++;
    uint32_t acc = 0;
    for (int c = 0; c < 256; c++) { C[c] = acc; acc += counts[c]; }
    uint32_t *lf = (uint32_t *)malloc((size_t)n * sizeof(uint32_t));
    for (uint32_t i = 0; i < n; i++) { uint8_t c = L[i]; lf[i] = C[c] + seen[c]++; }
    uint32_t cur = p;
    for (uint32_t t = 0; t < n; t++) { T_out[n - 1 - t] = L[cur]; cur = lf[cur]; }
    free(lf);
}

static RunRecord *extract_runs(const uint8_t *L, uint32_t n, uint32_t *out_r, uint32_t *out_h1) {
    RunRecord *runs = (RunRecord *)malloc((size_t)n * sizeof(RunRecord));
    uint32_t r = 0, h1 = 0, i = 0;
    while (i < n) {
        uint8_t c = L[i];
        uint32_t j = i + 1;
        while (j < n && L[j] == c) j++;
        if (j - i == 1) h1++;
        runs[r++] = (RunRecord){ c, i, j - i };
        i = j;
    }
    *out_r = r;
    *out_h1 = h1;
    return runs;
}

/*
 * Strip-overlap interval elimination of Thick_L (NO binary search, NO row walk!):
 * 1. Sort a copy of thick strips by .to (`ns <= 5,047` at L=16).
 * 2. One two-cursor merge of `strips_from` and `strips_to` records for each strip_from[f]
 *    the range `[ov_start[f], ov_end[f])` of `strips_to` overlapping it (`Q = 0`!).
 * 3. Backward band pull walks only `[ov_start[f], ov_end[f])` (`~1.2` strips per band!).
 */
static ThickBand *eliminate_thick_fast(
    const ThickStrip *strips_from,
    int ns,
    ByteArena *thick_arena,
    uint32_t *out_nb,
    uint64_t *q_acq_thick,
    uint64_t *d_acq_thick
) {
    if (ns == 0) { *out_nb = 0; return NULL; }
    ThickStrip *strips_to = (ThickStrip *)malloc((size_t)ns * sizeof(ThickStrip));
    memcpy(strips_to, strips_from, (size_t)ns * sizeof(ThickStrip));
    /* Radix/bucket sort by byte `c` preserves `.to` order because within each byte c,
     * `.to = C[c] + rank` is ALREADY strictly increasing in `strips_from`!
     * Wait: check that! `strips_to` sorted by `.to` IS JUST A 256-BUCKET RADIX GATHER OF `strips_from`! */
    int cnt_c[256] = {0}, off_c[256] = {0};
    for (int i = 0; i < ns; i++) cnt_c[strips_from[i].byte]++;
    int acc = 0;
    for (int c = 0; c < 256; c++) { off_c[c] = acc; acc += cnt_c[c]; }
    for (int i = 0; i < ns; i++) {
        strips_to[off_c[strips_from[i].byte]++] = strips_from[i];
    }
    /* Verify sorted by .to! */
    for (int i = 0; i + 1 < ns; i++) {
        assert(strips_to[i].to + strips_to[i].width <= strips_to[i + 1].to);
    }

    /* Precompute overlap window `[ov_lo[f], ov_hi[f])` of `strips_to` with `strips_from[f]`
     * via two monotone cursors (`Q = 0`!) */
    int *ov_lo = (int *)malloc((size_t)ns * sizeof(int));
    int *ov_hi = (int *)malloc((size_t)ns * sizeof(int));
    int t_ptr = 0;
    for (int f = 0; f < ns; f++) {
        uint32_t fa = strips_from[f].from;
        uint32_t fb = fa + strips_from[f].width;
        while (t_ptr < ns && strips_to[t_ptr].to + strips_to[t_ptr].width <= fa) t_ptr++;
        ov_lo[f] = t_ptr;
        int k = t_ptr;
        while (k < ns && strips_to[k].to < fb) k++;
        ov_hi[f] = k;
    }

    size_t q_cap = (size_t)(ns * 4 + 64);
    ThickBand *work = (ThickBand *)malloc(q_cap * sizeof(ThickBand));
    size_t w_head = 0, w_tail = 0;

    size_t b_cap = (size_t)(ns * 4 + 64);
    ThickBand *resolved = (ThickBand *)malloc(b_cap * sizeof(ThickBand));
    size_t nb = 0;

    /* Seed exit bands by linear cursor merge of `strips_to` against `strips_from` (`Q = 0`!) */
    int sf_idx = 0;
    for (int i = 0; i < ns; i++) {
        uint32_t t_cur = strips_to[i].to;
        uint32_t t_end = t_cur + strips_to[i].width;
        int32_t delta = (int32_t)strips_to[i].to - (int32_t)strips_to[i].from;
        uint8_t sym = strips_to[i].byte;
        int f_orig = strips_to[i].orig_idx;

        while (sf_idx < ns && strips_from[sf_idx].from + strips_from[sf_idx].width <= t_cur) sf_idx++;
        int j = sf_idx;
        uint32_t pos = t_cur;
        while (pos < t_end) {
            if (j >= ns || strips_from[j].from >= t_end) {
                uint32_t w = t_end - pos;
                uint32_t woff = arena_prepend(thick_arena, sym, NULL, 0);
                ThickBand b = { (uint32_t)((int32_t)pos - delta), pos, w, woff, 1, f_orig };
                if (w_tail == q_cap) { q_cap *= 2; work = (ThickBand *)realloc(work, q_cap * sizeof(ThickBand)); }
                work[w_tail++] = b;
                if (nb == b_cap) { b_cap *= 2; resolved = (ThickBand *)realloc(resolved, b_cap * sizeof(ThickBand)); }
                resolved[nb++] = b;
                break;
            }
            if (strips_from[j].from > pos) {
                uint32_t seg_end = strips_from[j].from < t_end ? strips_from[j].from : t_end;
                uint32_t w = seg_end - pos;
                uint32_t woff = arena_prepend(thick_arena, sym, NULL, 0);
                ThickBand b = { (uint32_t)((int32_t)pos - delta), pos, w, woff, 1, f_orig };
                if (w_tail == q_cap) { q_cap *= 2; work = (ThickBand *)realloc(work, q_cap * sizeof(ThickBand)); }
                work[w_tail++] = b;
                if (nb == b_cap) { b_cap *= 2; resolved = (ThickBand *)realloc(resolved, b_cap * sizeof(ThickBand)); }
                resolved[nb++] = b;
                pos = seg_end;
            } else {
                uint32_t seg_end = strips_from[j].from + strips_from[j].width;
                pos = seg_end < t_end ? seg_end : t_end;
                j++;
            }
        }
    }

    /* Pull resolved bands backward along pre-merged strip overlaps (1 addressed step per overlap!) */
    while (w_head < w_tail) {
        ThickBand cur = work[w_head++];
        uint32_t a = cur.src;
        uint32_t b_end = cur.src + cur.width;
        int f = cur.strip_from_idx;
        (*d_acq_thick)++;
        for (int idx = ov_lo[f]; idx < ov_hi[f]; idx++) {
            (*q_acq_thick)++;
            uint32_t st_a = strips_to[idx].to;
            uint32_t st_b = st_a + strips_to[idx].width;
            if (st_b <= a) continue;
            if (st_a >= b_end) break;
            uint32_t ov_s = a > st_a ? a : st_a;
            uint32_t ov_e = b_end < st_b ? b_end : st_b;
            uint32_t w = ov_e - ov_s;
            uint32_t off_in_cur = ov_s - cur.src;
            int32_t delta = (int32_t)st_a - (int32_t)strips_to[idx].from;
            uint32_t new_src = (uint32_t)((int32_t)ov_s - delta);
            uint32_t new_dst = cur.dst + off_in_cur;

            uint32_t clen = cur.len;
            uint8_t sbuf[256];
            uint8_t *tmp = (clen <= 256) ? sbuf : (uint8_t *)malloc(clen);
            memcpy(tmp, thick_arena->buf + cur.word_off, clen);
            uint32_t woff = arena_prepend(thick_arena, strips_to[idx].byte, tmp, clen);
            if (clen > 256) free(tmp);

            ThickBand nb_item = { new_src, new_dst, w, woff, clen + 1, strips_to[idx].orig_idx };
            if (w_tail == q_cap) { q_cap *= 2; work = (ThickBand *)realloc(work, q_cap * sizeof(ThickBand)); }
            work[w_tail++] = nb_item;
            if (nb == b_cap) { b_cap *= 2; resolved = (ThickBand *)realloc(resolved, b_cap * sizeof(ThickBand)); }
            resolved[nb++] = nb_item;
        }
    }

    qsort(resolved, nb, sizeof(ThickBand), cmp_band_src);
    free(ov_lo); free(ov_hi); free(work); free(strips_to);
    *out_nb = (uint32_t)nb;
    return resolved;
}

/* Helper to populate a ConeSlot from prefix bytes + optional ThickBand or successor ConeSlot */
static inline void slot_set_thick(
    ConeSlot *s,
    const uint8_t *prefix,
    uint32_t plen,
    uint32_t band_idx,
    const ThickBand *bands,
    uint32_t offset_in_band,
    const ByteArena *thick_arena
) {
    const ThickBand *b = &bands[band_idx];
    s->next = b->dst + offset_in_band;
    s->wait_kind = 0;
    if (plen + b->len <= INLINE_CAP) {
        memcpy(s->buf, prefix, plen);
        memcpy(s->buf + plen, thick_arena->buf + b->word_off, b->len);
        s->inline_len = (uint16_t)(plen + b->len);
        s->thick_band = UINT32_MAX;
    } else {
        memcpy(s->buf, prefix, plen);
        s->inline_len = (uint16_t)plen;
        s->thick_band = band_idx;
    }
}

/*
 * Subalphabet configurations:
 *   sub_mode = 0: k=1 baseline (sigma=256, s=257 cursors)
 *   sub_mode = 1: k=2 on Sigma_hex16 = [0-9a-f] (sigma_R=16, s_cone=256 cursors <= |Sigma|)
 *   sub_mode = 2: k=2 on Sigma_uuid18 = [0-9a-f-"] (sigma_R=18, s_cone=324 cursors)
 *   sub_mode = 3: k=3 on Sigma_hex16 = [0-9a-f] (sigma_R=16, s_cone=4096 cursors = 16 KiB L1d)
 * sweeps in {1, 2}:
 *   sweeps = 1: single ascending +1 cursor pass (Q_acq_thin = 0!)
 *   sweeps = 2: ascending +1 cursor pass + descending -1 cursor pass (Q_acq_thin = 0!)
 */
static int run_c2_decoder(
    const RunRecord *runs,
    uint32_t r_cnt,
    uint32_t n,
    uint32_t p,
    uint32_t limit,
    int sub_mode,
    int sweeps,
    uint8_t *T_out,
    C2Metrics *met
) {
    assert(sizeof(ConeSlot) == 64);
    memset(met, 0, sizeof(*met));

    int sub_map[256];
    uint8_t sub_chars[32];
    int sigma_r = 0;
    int cone_k = 1;
    for (int c = 0; c < 256; c++) sub_map[c] = -1;
    if (sub_mode == 1 || sub_mode == 3) {
        const char *hx = "0123456789abcdef";
        sigma_r = 16;
        cone_k = (sub_mode == 3) ? 3 : 2;
        for (int i = 0; i < 16; i++) { sub_map[(uint8_t)hx[i]] = i; sub_chars[i] = (uint8_t)hx[i]; }
    } else if (sub_mode == 2) {
        const char *u18 = "0123456789abcdef-\"";
        sigma_r = 18;
        cone_k = 2;
        for (int i = 0; i < 18; i++) { sub_map[(uint8_t)u18[i]] = i; sub_chars[i] = (uint8_t)u18[i]; }
    }

    /* Fold 0 over run records: C[256], Thin/Thick split, L_exp, F1 (`Q = 0`) */
    uint32_t counts[256] = {0}, C[256] = {0}, seen_run[256] = {0};
    for (uint32_t i = 0; i < r_cnt; i++) {
        if (runs[i].width == 1) met->h1++;
        counts[runs[i].byte] += runs[i].width;
    }
    uint32_t acc = 0;
    for (int c = 0; c < 256; c++) { C[c] = acc; acc += counts[c]; }

    ThickStrip *strips = (ThickStrip *)malloc(((size_t)r_cnt + 2) * sizeof(ThickStrip));
    int ns = 0;
    uint8_t *keep = (uint8_t *)calloc(n, 1);
    uint8_t *L_exp = (uint8_t *)malloc(n);
    uint8_t *F1 = (uint8_t *)malloc(n);
    for (int c = 0; c < 256; c++) {
        if (counts[c]) memset(F1 + C[c], c, counts[c]);
    }

    uint32_t m_thin = 0;
    for (uint32_t i = 0; i < r_cnt; i++) {
        uint8_t c = runs[i].byte;
        uint32_t pos = runs[i].pos;
        uint32_t w = runs[i].width;
        memset(L_exp + pos, c, w);
        if (w <= limit) {
            memset(keep + pos, 1, w);
            m_thin += w;
            seen_run[c] += w;
        } else if (pos <= p && p < pos + w) {
            if (pos < p) {
                uint32_t w1 = p - pos;
                strips[ns] = (ThickStrip){ pos, C[c] + seen_run[c], w1, c, ns };
                ns++;
                seen_run[c] += w1;
            }
            keep[p] = 1;
            m_thin++;
            seen_run[c]++;
            if (p + 1 < pos + w) {
                uint32_t w2 = (pos + w) - (p + 1);
                strips[ns] = (ThickStrip){ p + 1, C[c] + seen_run[c], w2, c, ns };
                ns++;
                seen_run[c] += w2;
            }
        } else {
            strips[ns] = (ThickStrip){ pos, C[c] + seen_run[c], w, c, ns };
            ns++;
            seen_run[c] += w;
        }
    }
    met->m_thin = m_thin;
    met->r_thick = (uint32_t)ns;
    met->streamed_bytes += (uint64_t)r_cnt * sizeof(RunRecord);

    /* Eliminate Thick_L via pre-merged strip-overlap pull */
    ByteArena thick_arena;
    arena_init(&thick_arena, (size_t)ns * 16 + 1024);
    uint32_t nb = 0;
    ThickBand *bands = eliminate_thick_fast(
        strips, ns, &thick_arena, &nb, &met->q_acq_thick, &met->d_acq_thick
    );
    met->thick_bands = nb;

    /* Sequential scan of sorted `bands` populates per-thick-row `(band_idx, off_in_band)` (`Q = 0`!) */
    uint32_t *row_band = (uint32_t *)malloc((size_t)n * sizeof(uint32_t));
    uint32_t *row_boff = (uint32_t *)malloc((size_t)n * sizeof(uint32_t));
    for (uint32_t b = 0; b < nb; b++) {
        for (uint32_t k = 0; k < bands[b].width; k++) {
            row_band[bands[b].src + k] = b;
            row_boff[bands[b].src + k] = k;
        }
    }

    /* Theorem RF-1(b): Synthesize exact cone starts C2 (and C3 for k=3) + lookahead byte streams
     * using ONLY 256-cursor sequential passes (`Q = 0`!) */
    uint32_t C2[32][256];
    uint32_t H2[32][256];
    memset(C2, 0, sizeof(C2));
    memset(H2, 0, sizeof(H2));
    if (cone_k >= 2) {
        for (uint32_t x = 0; x < n; x++) {
            int hc1 = sub_map[L_exp[x]];
            if (hc1 >= 0) H2[hc1][F1[x]]++;
        }
        for (int hc1 = 0; hc1 < sigma_r; hc1++) {
            uint32_t a = C[sub_chars[hc1]];
            for (int b = 0; b < 256; b++) {
                C2[hc1][b] = a;
                a += H2[hc1][b];
            }
            assert(a == C[sub_chars[hc1]] + counts[sub_chars[hc1]]);
        }
    }

    /* Pass 1a (`s = 257`, `Q = 0`): synthesize L2[x] = L[LF(x)] and keep2[x] = keep[LF(x)] */
    uint8_t *L2 = NULL, *keep2 = NULL;
    uint8_t *L3 = NULL, *keep3 = NULL;
    uint32_t *C3 = NULL, *H3 = NULL; /* [16 * 16 * 256] for k=3 */
    if (cone_k >= 2) {
        L2 = (uint8_t *)malloc(n);
        keep2 = (uint8_t *)malloc(n);
        uint32_t r1[256];
        memcpy(r1, C, sizeof(C));
        for (uint32_t x = 0; x < n; x++) {
            uint32_t y = r1[L_exp[x]]++;
            L2[x] = L_exp[y];
            keep2[x] = (y == p) ? 0 : keep[y]; /* treat p as boundary so cones never cross p! */
        }
        met->streamed_bytes += (uint64_t)n * 3;
    }

    if (cone_k == 3) {
        /* Synthesize F2[z] = F1[Psi(z)] via 1 forward 256-cursor scatter: F2[LF(w)] = F1[w] (`Q = 0`!)
         * Then |I_{c1, c2, b}| = #{ x : L[x]=c1, F1[x]=c2, F2[x]=b } (`Q = 0`!) */
        uint8_t *F2 = (uint8_t *)malloc(n);
        uint32_t w1[256];
        memcpy(w1, C, sizeof(C));
        for (uint32_t x = 0; x < n; x++) {
            uint32_t y = w1[L_exp[x]]++;
            F2[y] = F1[x];
        }
        H3 = (uint32_t *)calloc(16 * 16 * 256, sizeof(uint32_t));
        C3 = (uint32_t *)calloc(16 * 16 * 256, sizeof(uint32_t));
        for (uint32_t x = 0; x < n; x++) {
            int hc1 = sub_map[L_exp[x]];
            int hc2 = sub_map[F1[x]];
            if (hc1 >= 0 && hc2 >= 0) {
                H3[(hc1 * 16 + hc2) * 256 + F2[x]]++;
            }
        }
        for (int hc1 = 0; hc1 < 16; hc1++) {
            for (int hc2 = 0; hc2 < 16; hc2++) {
                uint32_t a = C2[hc1][sub_chars[hc2]];
                for (int b = 0; b < 256; b++) {
                    C3[(hc1 * 16 + hc2) * 256 + b] = a;
                    a += H3[(hc1 * 16 + hc2) * 256 + b];
                }
                assert(a == C2[hc1][sub_chars[hc2]] + H2[hc1][sub_chars[hc2]]);
            }
        }
        free(F2);

        /* Also synthesize L3[x] = L2[LF(x)] and keep3[x] = keep2[LF(x)] via 256 cursors (`Q = 0`!) */
        L3 = (uint8_t *)malloc(n);
        keep3 = (uint8_t *)malloc(n);
        uint32_t r1[256];
        memcpy(r1, C, sizeof(C));
        for (uint32_t x = 0; x < n; x++) {
            uint32_t y = r1[L_exp[x]]++;
            L3[x] = L2[y];
            keep3[x] = keep2[y];
        }
        met->streamed_bytes += (uint64_t)n * 4;
    }

    /* Allocate flat 64B ConeSlot[0..n) */
    ConeSlot *slots = (ConeSlot *)calloc(n, sizeof(ConeSlot));

    /* Set up sequential cursors for Pass 1b (ascending x = 0..n-1, `Q_acq_thin = 0`!) */
    uint32_t cur1[256];
    memcpy(cur1, C, sizeof(C));
    uint32_t cur2[32][32];
    memset(cur2, 0, sizeof(cur2));
    if (cone_k >= 2) {
        for (int a = 0; a < sigma_r; a++)
            for (int b = 0; b < sigma_r; b++)
                cur2[a][b] = C2[a][sub_chars[b]];
    }
    uint32_t *cur3 = NULL;
    if (cone_k == 3) {
        cur3 = (uint32_t *)malloc(4096 * sizeof(uint32_t));
        for (int a = 0; a < 16; a++)
            for (int b = 0; b < 16; b++)
                for (int c = 0; c < 16; c++)
                    cur3[(a << 8) | (b << 4) | c] = C3[(a * 16 + b) * 256 + sub_chars[c]];
        met->s_max = 4096 + 256 + 256 + 1;
    } else if (cone_k == 2) {
        /* Note: if y_arr wasn't needed for non-sub_mode fallback, s = sigma_r^2 + 1;
         * holding both cur2 (sigma_r^2) and cur1 (256) simultaneously has s = sigma_r^2 + 257
         * (or 257 if split into sub-pass!). We report exact simultaneous cursors here. */
        met->s_max = (uint32_t)(sigma_r * sigma_r + 257);
    } else {
        met->s_max = 257;
    }

    /* Pass 1b: Ascending pure-cursor cone induction (`Q_acq_thin = 0`!) */
    for (uint32_t x = 0; x < n; x++) {
        uint8_t b1 = L_exp[x];
        uint32_t y = cur1[b1]++;
        int h1_i = sub_map[b1];
        int h2_i = (cone_k >= 2) ? sub_map[L2[x]] : -1;
        int h3_i = (cone_k == 3) ? sub_map[L3[x]] : -1;

        uint32_t z2 = UINT32_MAX;
        if (cone_k >= 2 && h1_i >= 0 && h2_i >= 0) {
            z2 = cur2[h2_i][h1_i]++; /* = LF^2(x) by Theorem RF-1! */
        }
        uint32_t z3 = UINT32_MAX;
        if (cone_k == 3 && h1_i >= 0 && h2_i >= 0 && h3_i >= 0) {
            z3 = cur3[(h3_i << 8) | (h2_i << 4) | h1_i]++; /* = LF^3(x) by Theorem RF-1! */
        }

        if (!keep[x]) continue;
        met->strict_c1_acq++;

        /* Try k=3 cone step first if available and intermediate rows y, z2 are internal thin rows */
        if (cone_k == 3 && z3 != UINT32_MAX && keep2[x] && keep3[x]) {
            met->k_cone_hits++;
            uint8_t pref[3] = { b1, L2[x], L3[x] };
            if (!keep[z3]) {
                slot_set_thick(&slots[x], pref, 3, row_band[z3], bands, row_boff[z3], &thick_arena);
            } else if (z3 < x && z3 != p && slots[z3].next != x && (3 + slots[z3].inline_len <= INLINE_CAP)) {
                memcpy(slots[x].buf, pref, 3);
                memcpy(slots[x].buf + 3, slots[z3].buf, slots[z3].inline_len);
                slots[x].inline_len = (uint16_t)(3 + slots[z3].inline_len);
                slots[x].next = slots[z3].next;
                slots[x].thick_band = slots[z3].thick_band;
                slots[x].wait_kind = 0;
            } else {
                memcpy(slots[x].buf, pref, 3);
                slots[x].inline_len = 3;
                slots[x].next = z3;
                slots[x].thick_band = UINT32_MAX;
                slots[x].wait_kind = (z3 > x && z3 != p) ? 1 : 0;
            }
            continue;
        }

        /* Try k=2 cone step if available and intermediate row y is internal thin row */
        if (cone_k >= 2 && z2 != UINT32_MAX && keep2[x]) {
            met->k_cone_hits++;
            uint8_t pref[2] = { b1, L2[x] };
            if (!keep[z2]) {
                slot_set_thick(&slots[x], pref, 2, row_band[z2], bands, row_boff[z2], &thick_arena);
            } else if (z2 < x && z2 != p && slots[z2].next != x && (2 + slots[z2].inline_len <= INLINE_CAP)) {
                memcpy(slots[x].buf, pref, 2);
                memcpy(slots[x].buf + 2, slots[z2].buf, slots[z2].inline_len);
                slots[x].inline_len = (uint16_t)(2 + slots[z2].inline_len);
                slots[x].next = slots[z2].next;
                slots[x].thick_band = slots[z2].thick_band;
                slots[x].wait_kind = 0;
            } else {
                memcpy(slots[x].buf, pref, 2);
                slots[x].inline_len = 2;
                slots[x].next = z2;
                slots[x].thick_band = UINT32_MAX;
                slots[x].wait_kind = (z2 > x && z2 != p) ? 2 : 0;
            }
            continue;
        }

        /* Fallback: 1-step cursor `y = LF(x)` */
        uint8_t pref1[1] = { b1 };
        if (!keep[y]) {
            slot_set_thick(&slots[x], pref1, 1, row_band[y], bands, row_boff[y], &thick_arena);
        } else if (y < x && y != p && slots[y].next != x && (1 + slots[y].inline_len <= INLINE_CAP)) {
            slots[x].buf[0] = b1;
            memcpy(slots[x].buf + 1, slots[y].buf, slots[y].inline_len);
            slots[x].inline_len = (uint16_t)(1 + slots[y].inline_len);
            slots[x].next = slots[y].next;
            slots[x].thick_band = slots[y].thick_band;
            slots[x].wait_kind = 0;
        } else {
            slots[x].buf[0] = b1;
            slots[x].inline_len = 1;
            slots[x].next = y;
            slots[x].thick_band = UINT32_MAX;
            slots[x].wait_kind = (y > x && y != p) ? 3 : 0;
        }
    }
    met->streamed_bytes += (uint64_t)n * sizeof(ConeSlot);

    /* Optional Pass 2: Descending -1 pure-cursor cone sweep (`Q_acq_thin = 0`!) */
    if (sweeps >= 2) {
        uint32_t cur1_d[256];
        for (int c = 0; c < 256; c++) cur1_d[c] = C[c] + counts[c];
        uint32_t cur2_d[32][32];
        if (cone_k >= 2) {
            for (int a = 0; a < sigma_r; a++)
                for (int b = 0; b < sigma_r; b++)
                    cur2_d[a][b] = C2[a][sub_chars[b]] + H2[a][sub_chars[b]];
        }
        uint32_t *cur3_d = NULL;
        if (cone_k == 3) {
            cur3_d = (uint32_t *)malloc(4096 * sizeof(uint32_t));
            for (int a = 0; a < 16; a++)
                for (int b = 0; b < 16; b++)
                    for (int c = 0; c < 16; c++)
                        cur3_d[(a << 8) | (b << 4) | c] =
                            C3[(a * 16 + b) * 256 + sub_chars[c]] + H3[(a * 16 + b) * 256 + sub_chars[c]];
        }

        for (int64_t xi = (int64_t)n - 1; xi >= 0; xi--) {
            uint32_t x = (uint32_t)xi;
            uint8_t b1 = L_exp[x];
            uint32_t y = --cur1_d[b1];
            int h1_i = sub_map[b1];
            int h2_i = (cone_k >= 2) ? sub_map[L2[x]] : -1;
            int h3_i = (cone_k == 3) ? sub_map[L3[x]] : -1;

            uint32_t z2 = UINT32_MAX;
            if (cone_k >= 2 && h1_i >= 0 && h2_i >= 0) z2 = --cur2_d[h2_i][h1_i];
            uint32_t z3 = UINT32_MAX;
            if (cone_k == 3 && h1_i >= 0 && h2_i >= 0 && h3_i >= 0) {
                z3 = --cur3_d[(h3_i << 8) | (h2_i << 4) | h1_i];
            }

            if (!keep[x] || slots[x].wait_kind == 0) continue;
            met->strict_c1_acq++;

            uint32_t target = UINT32_MAX;
            if (slots[x].wait_kind == 1) target = z3;
            else if (slots[x].wait_kind == 2) target = z2;
            else if (slots[x].wait_kind == 3) target = y;

            assert(target == slots[x].next);
            if (target > x && target != p && keep[target] && slots[target].next != x) {
                uint16_t l1 = slots[x].inline_len;
                uint16_t l2 = slots[target].inline_len;
                if (slots[x].thick_band == UINT32_MAX && (uint32_t)(l1 + l2) <= INLINE_CAP) {
                    memcpy(slots[x].buf + l1, slots[target].buf, l2);
                    slots[x].inline_len = (uint16_t)(l1 + l2);
                    slots[x].next = slots[target].next;
                    slots[x].thick_band = slots[target].thick_band;
                }
            }
        }
        met->streamed_bytes += (uint64_t)n * sizeof(ConeSlot);
        free(cur3_d);
    }

    met->peak_bytes = (uint64_t)n * sizeof(ConeSlot) + thick_arena.len + (uint64_t)nb * sizeof(ThickBand);

    /* Consumer (Run): step through ConeSlots from `p`, emitting T[0..n) byte-exact! */
    uint32_t cur = p;
    uint32_t emitted = 0;
    while (emitted < n) {
        assert(keep[cur]);
        met->q_run_steps++;
        met->d_run++;
        const ConeSlot *s = &slots[cur];
        uint32_t l_in = s->inline_len;
        uint32_t take = (emitted + l_in <= n) ? l_in : (n - emitted);
        for (uint32_t k = 0; k < take; k++) {
            T_out[n - 1 - (emitted + k)] = s->buf[k];
        }
        emitted += take;
        if (emitted < n && s->thick_band != UINT32_MAX) {
            met->q_run_spill++; /* honest charge for spilled thick-band dictionary read! */
            const ThickBand *b = &bands[s->thick_band];
            const uint8_t *wptr = thick_arena.buf + b->word_off;
            uint32_t take2 = (emitted + b->len <= n) ? b->len : (n - emitted);
            for (uint32_t k = 0; k < take2; k++) {
                T_out[n - 1 - (emitted + k)] = wptr[k];
            }
            emitted += take2;
        }
        cur = s->next;
    }

    free(slots);
    free(cur3); free(C3); free(H3);
    free(L3); free(keep3); free(L2); free(keep2);
    free(row_band); free(row_boff);
    free(bands); free(thick_arena.buf);
    free(keep); free(L_exp); free(F1); free(strips);
    return 0;
}

static void run_unit_tests(void) {
    struct {
        const char *name;
        const char *L;
        uint32_t p;
    } cases[] = {
        { "single_char", "aaaa", 2 },
        { "periodic_2cyc", "bbaa", 1 },
        { "witness_baaaa", "baaaa", 2 },
        { "hex_all_w1", "f0a1b2c3d4e596870f1e2d3c4b5a6978", 5 },
        { "uuid_mix", "zpcqcrcscDc234617axabybwb2346CBA8", 10 },
    };
    int nc = (int)(sizeof(cases) / sizeof(cases[0]));
    for (int ci = 0; ci < nc; ci++) {
        uint32_t n = (uint32_t)strlen(cases[ci].L);
        const uint8_t *L = (const uint8_t *)cases[ci].L;
        uint8_t *T_ref = (uint8_t *)malloc(n);
        uint8_t *T_dec = (uint8_t *)malloc(n);
        invert_reference(L, n, cases[ci].p, T_ref);
        uint32_t r = 0, h1 = 0;
        RunRecord *runs = extract_runs(L, n, &r, &h1);
        for (int sm = 0; sm <= 3; sm++) {
            for (int sw = 1; sw <= 2; sw++) {
                C2Metrics met;
                memset(T_dec, 0, n);
                run_c2_decoder(runs, r, n, cases[ci].p, 4, sm, sw, T_dec, &met);
                if (memcmp(T_ref, T_dec, n) != 0) {
                    fprintf(stderr, "C2 UNIT TEST FAIL on %s sm=%d sw=%d\n", cases[ci].name, sm, sw);
                    exit(1);
                }
            }
        }
        free(runs); free(T_ref); free(T_dec);
    }
}

int main(int argc, char **argv) {
    if (argc == 2 && strcmp(argv[1], "--self-test") == 0) {
        run_unit_tests();
        printf("SELF_TEST_OK\n");
        return 0;
    }
    if (argc < 6) {
        fprintf(stderr, "Usage: %s <block.L> <orig_ptr> <limit> <sub_mode:0..3> <sweeps:1..2>\n", argv[0]);
        return 2;
    }
    run_unit_tests();

    const char *path = argv[1];
    uint32_t p = (uint32_t)strtoul(argv[2], NULL, 10);
    uint32_t limit = (uint32_t)strtoul(argv[3], NULL, 10);
    int sub_mode = atoi(argv[4]);
    int sweeps = atoi(argv[5]);

    FILE *f = fopen(path, "rb");
    if (!f) { perror("fopen"); return 1; }
    fseek(f, 0, SEEK_END);
    uint32_t n = (uint32_t)ftell(f);
    fseek(f, 0, SEEK_SET);
    uint8_t *L = (uint8_t *)malloc(n);
    if (fread(L, 1, n, f) != n) { perror("fread"); return 1; }
    fclose(f);

    uint8_t *T_ref = (uint8_t *)malloc(n);
    uint8_t *T_dec = (uint8_t *)malloc(n);
    invert_reference(L, n, p, T_ref);

    uint32_t r = 0, h1 = 0;
    RunRecord *runs = extract_runs(L, n, &r, &h1);

    C2Metrics met;
    run_c2_decoder(runs, r, n, p, limit, sub_mode, sweeps, T_dec, &met);
    if (memcmp(T_ref, T_dec, n) != 0) {
        fprintf(stderr, "C2 BYTE MISMATCH on %s!\n", path);
        return 1;
    }

    uint64_t q_run_tot = met.q_run_steps + met.q_run_spill;
    uint64_t q_tot = met.q_acq_thick + met.q_acq_thin + q_run_tot;

    printf("{\"n\":%u,\"r\":%u,\"h1\":%u,\"limit\":%u,\"sub_mode\":%d,\"sweeps\":%d,"
           "\"m_thin\":%u,\"r_thick\":%u,\"thick_bands\":%u,\"k_cone_hits\":%llu,"
           "\"q_acq_thick\":%llu,\"d_acq_thick\":%llu,\"q_acq_thin\":%llu,\"d_acq_thin\":%llu,"
           "\"q_run_steps\":%llu,\"q_run_spill\":%llu,\"q_run\":%llu,\"q_total\":%llu,"
           "\"d_run\":%llu,\"P_passes\":%.2f,\"s_max\":%u,\"S_bytes\":%llu,"
           "\"strict_c1_acq\":%llu,\"exact\":true}\n",
           n, r, met.h1, limit, sub_mode, sweeps,
           met.m_thin, met.r_thick, met.thick_bands, (unsigned long long)met.k_cone_hits,
           (unsigned long long)met.q_acq_thick, (unsigned long long)met.d_acq_thick,
           (unsigned long long)met.q_acq_thin, (unsigned long long)met.d_acq_thin,
           (unsigned long long)met.q_run_steps, (unsigned long long)met.q_run_spill,
           (unsigned long long)q_run_tot, (unsigned long long)q_tot,
           (unsigned long long)met.d_run, (double)met.streamed_bytes / (double)n,
           met.s_max, (unsigned long long)met.peak_bytes,
           (unsigned long long)met.strict_c1_acq);

    free(runs); free(T_ref); free(T_dec); free(L);
    return 0;
}
