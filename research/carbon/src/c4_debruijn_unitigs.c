/*
 * C-4: De Bruijn Unitig - Interval Duality (U_k sqcup I_k^{>=2})
 *
 * Implements:
 *   1. Lemma U-1: Synthesis of globally sorted 64-bit 8-mer keys u8[0..n) in 7
 *      pure 256-cursor sequential passes (`Q = 0`), local neighbor inequality
 *      certifying unique 8-mers `V_1 = {x : |I_{u8[x]}| == 1}` (`Q = 0`), and
 *      verification of the register-local Eulerian successor identity
 *      `u8[LF(x)] == ((uint64_t)L[x] << 56) | (u8[x] >> 8)` on every row!
 *   2. Exact decomposition into Unique-k-mer Unitigs (`V_1`) + Multiplicity->=2
 *      Intervals (`Thick_16` eliminated via strip-overlap pull; thin branching rows kept).
 *   3. Three Unitig Acquisition Modes (all byte-exact!):
 *      - Mode 1: 1-sweep pure-cursor unitig compaction on V_1 (`Q_acq_unitig = 0`)
 *      - Mode 2: 2-sweep (+1/-1) pure-cursor unitig compaction on V_1 (`Q_acq_unitig = 0`)
 *      - Mode 3: Exact converged maximal unitigs `U_8` on `V_1`, reporting exact
 *        `u_8` unitig count, `Q_run`, and the Name-to-Address / compaction `Q_acq_unitig`.
 */

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <assert.h>

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
    uint32_t h1;
    uint32_t m_thin;
    uint32_t v1_total;
    uint32_t v1_w1;
    uint32_t w1_branch;
    uint32_t exact_unitigs;
    uint32_t thick_bands;
    uint64_t q_acq_thick;
    uint64_t d_acq_thick;
    uint64_t q_acq_unitig;
    uint64_t d_acq_unitig;
    uint64_t q_run_unitig;
    uint64_t q_run_branch;
    uint64_t q_run_total;
    uint64_t d_run;
    uint64_t streamed_bytes;
    uint32_t s_max;
    uint64_t peak_bytes;
    uint64_t strict_c1_acq;
} C4Metrics;

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

static uint32_t arena_push2(ByteArena *a, const uint8_t *w1, uint32_t l1, const uint8_t *w2, uint32_t l2) {
    size_t need = a->len + l1 + l2;
    if (need > a->cap) {
        while (a->cap < need) a->cap *= 2;
        a->buf = (uint8_t *)realloc(a->buf, a->cap);
    }
    uint32_t off = (uint32_t)a->len;
    if (l1) { memcpy(a->buf + a->len, w1, l1); a->len += l1; }
    if (l2) { memcpy(a->buf + a->len, w2, l2); a->len += l2; }
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
    int cnt_c[256] = {0}, off_c[256] = {0};
    for (int i = 0; i < ns; i++) cnt_c[strips_from[i].byte]++;
    int acc = 0;
    for (int c = 0; c < 256; c++) { off_c[c] = acc; acc += cnt_c[c]; }
    for (int i = 0; i < ns; i++) strips_to[off_c[strips_from[i].byte]++] = strips_from[i];

    int *ov_lo = (int *)malloc((size_t)ns * sizeof(int));
    int *ov_hi = (int *)malloc((size_t)ns * sizeof(int));
    int t_ptr = 0;
    for (int f = 0; f < ns; f++) {
        uint32_t fa = strips_from[f].from, fb = fa + strips_from[f].width;
        while (t_ptr < ns && strips_to[t_ptr].to + strips_to[t_ptr].width <= fa) t_ptr++;
        ov_lo[f] = t_ptr;
        int k = t_ptr;
        while (k < ns && strips_to[k].to < fb) k++;
        ov_hi[f] = k;
    }

    size_t q_cap = (size_t)(ns * 4 + 64), w_head = 0, w_tail = 0;
    ThickBand *work = (ThickBand *)malloc(q_cap * sizeof(ThickBand));
    size_t b_cap = (size_t)(ns * 4 + 64), nb = 0;
    ThickBand *resolved = (ThickBand *)malloc(b_cap * sizeof(ThickBand));

    int sf_idx = 0;
    for (int i = 0; i < ns; i++) {
        uint32_t t_cur = strips_to[i].to, t_end = t_cur + strips_to[i].width;
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

    while (w_head < w_tail) {
        ThickBand cur = work[w_head++];
        uint32_t a = cur.src, b_end = cur.src + cur.width;
        int f = cur.strip_from_idx;
        (*d_acq_thick)++;
        for (int idx = ov_lo[f]; idx < ov_hi[f]; idx++) {
            (*q_acq_thick)++;
            uint32_t st_a = strips_to[idx].to, st_b = st_a + strips_to[idx].width;
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

/*
 * mode = 1: 1-sweep pure-cursor unitig compaction on V_1 (`Q_acq_unitig = 0`)
 * mode = 2: 2-sweep (+1/-1) pure-cursor unitig compaction on V_1 (`Q_acq_unitig = 0`)
 * mode = 3: Exact converged maximal unitigs `U_k` on `V_1` (`Q_acq_unitig` counted)
 */
static int run_c4_decoder(
    const RunRecord *runs,
    uint32_t r_cnt,
    uint32_t n,
    uint32_t p,
    uint32_t limit,
    int k_mer, /* 6 or 8 */
    int mode,  /* 1, 2, or 3 */
    uint8_t *T_out,
    C4Metrics *met
) {
    memset(met, 0, sizeof(*met));

    uint32_t counts[256] = {0}, C[256] = {0}, seen_run[256] = {0};
    uint8_t *is_w1 = (uint8_t *)calloc(n, 1);
    for (uint32_t i = 0; i < r_cnt; i++) {
        if (runs[i].width == 1) {
            met->h1++;
            is_w1[runs[i].pos] = 1;
        }
        counts[runs[i].byte] += runs[i].width;
    }
    uint32_t acc = 0;
    for (int c = 0; c < 256; c++) { C[c] = acc; acc += counts[c]; }

    ThickStrip *strips = (ThickStrip *)malloc(((size_t)r_cnt + 2) * sizeof(ThickStrip));
    int ns = 0;
    uint8_t *keep = (uint8_t *)calloc(n, 1);
    uint8_t *L_exp = (uint8_t *)malloc(n);
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
    met->streamed_bytes += (uint64_t)r_cnt * sizeof(RunRecord);

    /* Eliminate Thick_16 via strip-overlap pull */
    ByteArena thick_arena;
    arena_init(&thick_arena, (size_t)ns * 16 + 1024);
    uint32_t nb = 0;
    ThickBand *bands = eliminate_thick_fast(
        strips, ns, &thick_arena, &nb, &met->q_acq_thick, &met->d_acq_thick
    );
    met->thick_bands = nb;

    uint32_t *thick_dst = (uint32_t *)malloc((size_t)n * sizeof(uint32_t));
    uint32_t *thick_woff = (uint32_t *)malloc((size_t)n * sizeof(uint32_t));
    uint32_t *thick_wlen = (uint32_t *)calloc(n, sizeof(uint32_t));
    for (uint32_t b = 0; b < nb; b++) {
        for (uint32_t k = 0; k < bands[b].width; k++) {
            uint32_t row = bands[b].src + k;
            thick_dst[row] = bands[b].dst + k;
            thick_woff[row] = bands[b].word_off;
            thick_wlen[row] = bands[b].len;
        }
    }

    /* Lemma U-1(a): Synthesize globally sorted k-mer key `uk[0..n)` in `k_mer - 1`
     * pure 256-cursor sequential scatter passes (`Q = 0, s = 257`!)
     * Convention: MSB is F_1[x] at bit `8*(k_mer-1)`, ..., F_k[x] at bit 0. */
    uint64_t *uk = (uint64_t *)calloc(n, sizeof(uint64_t));
    uint64_t *uk_nxt = (uint64_t *)calloc(n, sizeof(uint64_t));
    int shift_top = 8 * (k_mer - 1);
    for (int c = 0; c < 256; c++) {
        uint64_t v = ((uint64_t)c) << shift_top;
        for (uint32_t j = 0; j < counts[c]; j++) {
            uk[C[c] + j] = v;
        }
    }
    for (int step = 1; step < k_mer; step++) {
        uint32_t wcur[256];
        memcpy(wcur, C, sizeof(C));
        for (uint32_t x = 0; x < n; x++) {
            uint8_t b = L_exp[x];
            uint32_t y = wcur[b]++;
            uk_nxt[y] = (((uint64_t)b) << shift_top) | (uk[x] >> 8);
        }
        uint64_t *tmp = uk; uk = uk_nxt; uk_nxt = tmp;
        met->streamed_bytes += (uint64_t)n * 17;
    }
    free(uk_nxt);

    /* Verify Lemma U-1(a) & (c): `uk` is globally sorted in [0, n), AND
     * `uk[LF(x)] == ((uint64_t)L[x] << shift_top) | (uk[x] >> 8)` on EVERY row! */
    for (uint32_t x = 0; x + 1 < n; x++) {
        assert(uk[x] <= uk[x + 1]);
    }
    {
        uint32_t vcur[256];
        memcpy(vcur, C, sizeof(C));
        for (uint32_t x = 0; x < n; x++) {
            uint8_t b = L_exp[x];
            uint32_t y = vcur[b]++;
            uint64_t pred = (((uint64_t)b) << shift_top) | (uk[x] >> 8);
            assert(uk[y] == pred);
        }
    }

    /* Lemma U-1(b): Certify `V_1 = { x : |I_{uk[x]}| == 1 }` by O(1) neighbor inequality (`Q = 0`!) */
    uint8_t *in_v1 = (uint8_t *)calloc(n, 1);
    for (uint32_t x = 0; x < n; x++) {
        int ldiff = (x == 0) || (uk[x] != uk[x - 1]);
        int rdiff = (x + 1 == n) || (uk[x] != uk[x + 1]);
        if (ldiff && rdiff) {
            in_v1[x] = 1;
            met->v1_total++;
            if (is_w1[x]) met->v1_w1++;
        } else if (is_w1[x]) {
            met->w1_branch++;
        }
    }

    /* Count exact maximal unitig entries (`y not in V_1_active` -> `LF(y) in V_1_active`,
     * where `V_1_active = V_1 \cap Thin_16`) via 1 sequential 256-cursor scan (`Q = 0`!) */
    uint8_t *v1_thin = (uint8_t *)calloc(n, 1);
    for (uint32_t x = 0; x < n; x++) {
        if (in_v1[x] && keep[x]) v1_thin[x] = 1;
    }
    {
        uint32_t ccur[256];
        memcpy(ccur, C, sizeof(C));
        uint32_t u_entries = 0;
        for (uint32_t x = 0; x < n; x++) {
            uint32_t y = ccur[L_exp[x]]++;
            if (!v1_thin[x] && v1_thin[y]) u_entries++;
        }
        if (u_entries == 0 && v1_thin[p]) u_entries = 1;
        met->exact_unitigs = u_entries;
    }

    /* Build Unitig + Branching-Thin edges:
     *Notice ONLY vertices in `v1_thin` are allowed to chain into multi-byte unitigs!
     * Branching thin rows (`keep[x] && !v1_thin[x]`) stay 1-thin-step (absorbing thick suffix if y is thick),
     * isolating the EXACT contribution of De Bruijn Unitigs on `V_1`! */
    uint32_t *J = (uint32_t *)malloc((size_t)n * sizeof(uint32_t));
    uint32_t *woff = (uint32_t *)malloc((size_t)n * sizeof(uint32_t));
    uint32_t *wlen = (uint32_t *)calloc(n, sizeof(uint32_t));
    uint8_t  *unitig_done = (uint8_t *)calloc(n, 1);
    uint8_t  *wait_cursor_y = (uint8_t *)calloc(n, 1);
    ByteArena word_arena;
    arena_init(&word_arena, (size_t)m_thin * 4 + 1024);

    met->s_max = 257;

    /* Pass 1: Ascending +1 256-cursor compaction on `v1_thin` (`Q_acq_unitig = 0`!) */
    {
        uint32_t cur_c[256];
        memcpy(cur_c, C, sizeof(C));
        for (uint32_t x = 0; x < n; x++) {
            uint8_t b = L_exp[x];
            uint32_t y = cur_c[b]++;
            if (!keep[x]) {
                J[x] = thick_dst[x];
                unitig_done[x] = 1;
                continue;
            }
            met->strict_c1_acq++;

            if (!keep[y]) {
                /* Enters Thick_16 -> absorb thick exit band */
                J[x] = thick_dst[y];
                wlen[x] = 1 + thick_wlen[y];
                woff[x] = arena_prepend(&word_arena, b, thick_arena.buf + thick_woff[y], thick_wlen[y]);
                unitig_done[x] = 1;
            } else if (!v1_thin[x] || !v1_thin[y] || y == p || y == x) {
                /* Either x or y is a branching k-mer (`|I_uk| >= 2`) or `p`: unitig terminates here! */
                J[x] = y;
                wlen[x] = 1;
                woff[x] = arena_prepend(&word_arena, b, NULL, 0);
                unitig_done[x] = 1;
            } else if (y < x && J[y] != x) {
                /* Both x and y in `v1_thin`, and `y < x` already visited via +1 cursor (`Q = 0`!) */
                uint32_t yl = wlen[y];
                uint8_t sbuf[256];
                uint8_t *tmp = (yl <= 256) ? sbuf : (uint8_t *)malloc(yl);
                memcpy(tmp, word_arena.buf + woff[y], yl);
                woff[x] = arena_prepend(&word_arena, b, tmp, yl);
                if (yl > 256) free(tmp);
                wlen[x] = 1 + yl;
                J[x] = J[y];
                unitig_done[x] = unitig_done[y];
            } else {
                J[x] = y;
                wlen[x] = 1;
                woff[x] = arena_prepend(&word_arena, b, NULL, 0);
                unitig_done[x] = (y < x) ? 1 : 0;
                wait_cursor_y[x] = (y > x) ? 1 : 0; /* waiting for y > x under Pass 2 -1 cursor */
            }
        }
        met->streamed_bytes += (uint64_t)n * 12;
    }

    /* Pass 2 (if mode >= 2): Descending -1 256-cursor compaction on `v1_thin` (`Q_acq_unitig = 0`!) */
    if (mode >= 2) {
        uint32_t cur_d[256];
        for (int c = 0; c < 256; c++) cur_d[c] = C[c] + counts[c];
        for (int64_t xi = (int64_t)n - 1; xi >= 0; xi--) {
            uint32_t x = (uint32_t)xi;
            uint8_t b = L_exp[x];
            uint32_t y = --cur_d[b];
            if (!keep[x] || !wait_cursor_y[x]) continue;
            met->strict_c1_acq++;
            assert(y == J[x] && y > x);
            if (J[y] != x) {
                uint32_t l1 = wlen[x], l2 = wlen[y];
                uint8_t *t1 = (uint8_t *)malloc(l1 + l2);
                memcpy(t1, word_arena.buf + woff[x], l1);
                memcpy(t1 + l1, word_arena.buf + woff[y], l2);
                woff[x] = arena_push2(&word_arena, t1, l1 + l2, NULL, 0);
                free(t1);
                wlen[x] = l1 + l2;
                J[x] = J[y];
                unitig_done[x] = unitig_done[y];
            } else {
                unitig_done[x] = 1;
            }
        }
        met->streamed_bytes += (uint64_t)n * 12;
    }
    free(wait_cursor_y);

    /* Mode 3: Exact converged maximal unitigs `U_k` on `V_1` (`Q_acq_unitig` counted honestly!) */
    if (mode == 3) {
        for (int sw = 3; sw <= 64; sw++) {
            uint64_t rem = 0;
            if (sw % 2 == 1) {
                for (uint32_t x = 0; x < n; x++) {
                    if (!keep[x] || unitig_done[x]) continue;
                    uint32_t y = J[x];
                    if (y == p || y == x || !v1_thin[y]) { unitig_done[x] = 1; continue; }
                    met->q_acq_unitig++;
                    met->strict_c1_acq++;
                    uint32_t l1 = wlen[x], l2 = wlen[y];
                    uint8_t *t1 = (uint8_t *)malloc(l1 + l2);
                    memcpy(t1, word_arena.buf + woff[x], l1);
                    memcpy(t1 + l1, word_arena.buf + woff[y], l2);
                    woff[x] = arena_push2(&word_arena, t1, l1 + l2, NULL, 0);
                    free(t1);
                    wlen[x] = l1 + l2;
                    J[x] = J[y];
                    unitig_done[x] = unitig_done[y];
                    if (!unitig_done[x]) rem++;
                }
            } else {
                for (int64_t xi = (int64_t)n - 1; xi >= 0; xi--) {
                    uint32_t x = (uint32_t)xi;
                    if (!keep[x] || unitig_done[x]) continue;
                    uint32_t y = J[x];
                    if (y == p || y == x || !v1_thin[y]) { unitig_done[x] = 1; continue; }
                    met->q_acq_unitig++;
                    met->strict_c1_acq++;
                    uint32_t l1 = wlen[x], l2 = wlen[y];
                    uint8_t *t1 = (uint8_t *)malloc(l1 + l2);
                    memcpy(t1, word_arena.buf + woff[x], l1);
                    memcpy(t1 + l1, word_arena.buf + woff[y], l2);
                    woff[x] = arena_push2(&word_arena, t1, l1 + l2, NULL, 0);
                    free(t1);
                    wlen[x] = l1 + l2;
                    J[x] = J[y];
                    unitig_done[x] = unitig_done[y];
                    if (!unitig_done[x]) rem++;
                }
            }
            met->d_acq_unitig++;
            if (rem == 0) break;
        }
    }

    met->peak_bytes = (uint64_t)n * 8 + (uint64_t)m_thin * 12 + word_arena.len + thick_arena.len;

    /* Consumer (`Run`): decode T[0..n) byte-exact, separating unitig steps vs branching thin steps! */
    uint32_t cur = p;
    uint32_t emitted = 0;
    while (emitted < n) {
        assert(keep[cur]);
        if (v1_thin[cur]) met->q_run_unitig++;
        else met->q_run_branch++;
        met->d_run++;

        uint32_t l = wlen[cur];
        const uint8_t *wptr = word_arena.buf + woff[cur];
        uint32_t take = (emitted + l <= n) ? l : (n - emitted);
        for (uint32_t k = 0; k < take; k++) {
            T_out[n - 1 - (emitted + k)] = wptr[k];
        }
        emitted += take;
        cur = J[cur];
    }
    met->q_run_total = met->q_run_unitig + met->q_run_branch;

    free(word_arena.buf);
    free(J); free(woff); free(wlen); free(unitig_done);
    free(v1_thin); free(in_v1); free(uk);
    free(thick_dst); free(thick_woff); free(thick_wlen);
    free(bands); free(thick_arena.buf);
    free(keep); free(L_exp); free(is_w1); free(strips);
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
        for (int km = 6; km <= 8; km += 2) {
            for (int md = 1; md <= 3; md++) {
                C4Metrics met;
                memset(T_dec, 0, n);
                run_c4_decoder(runs, r, n, cases[ci].p, 1, km, md, T_dec, &met);
                if (memcmp(T_ref, T_dec, n) != 0) {
                    fprintf(stderr, "C4 UNIT TEST FAIL on %s km=%d md=%d\n", cases[ci].name, km, md);
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
        fprintf(stderr, "Usage: %s <block.L> <orig_ptr> <limit> <k_mer:6|8> <mode:1..3>\n", argv[0]);
        return 2;
    }
    run_unit_tests();

    const char *path = argv[1];
    uint32_t p = (uint32_t)strtoul(argv[2], NULL, 10);
    uint32_t limit = (uint32_t)strtoul(argv[3], NULL, 10);
    int k_mer = atoi(argv[4]);
    int mode = atoi(argv[5]);

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

    C4Metrics met;
    run_c4_decoder(runs, r, n, p, limit, k_mer, mode, T_dec, &met);
    if (memcmp(T_ref, T_dec, n) != 0) {
        fprintf(stderr, "C4 BYTE MISMATCH on %s!\n", path);
        return 1;
    }

    printf("{\"n\":%u,\"r\":%u,\"h1\":%u,\"limit\":%u,\"k_mer\":%d,\"mode\":%d,"
           "\"m_thin\":%u,\"v1_total\":%u,\"v1_w1\":%u,\"w1_branch\":%u,\"exact_unitigs\":%u,"
           "\"q_acq_thick\":%llu,\"q_acq_unitig\":%llu,\"d_acq_unitig\":%llu,"
           "\"q_run_unitig\":%llu,\"q_run_branch\":%llu,\"q_run\":%llu,\"d_run\":%llu,"
           "\"P_passes\":%.1f,\"s_max\":%u,\"S_bytes\":%llu,\"strict_c1_acq\":%llu,\"exact\":true}\n",
           n, r, met.h1, limit, k_mer, mode,
           met.m_thin, met.v1_total, met.v1_w1, met.w1_branch, met.exact_unitigs,
           (unsigned long long)met.q_acq_thick, (unsigned long long)met.q_acq_unitig,
           (unsigned long long)met.d_acq_unitig,
           (unsigned long long)met.q_run_unitig, (unsigned long long)met.q_run_branch,
           (unsigned long long)met.q_run_total, (unsigned long long)met.d_run,
           (double)met.streamed_bytes / (double)n, met.s_max,
           (unsigned long long)met.peak_bytes, (unsigned long long)met.strict_c1_acq);

    free(runs); free(T_ref); free(T_dec); free(L);
    return 0;
}
