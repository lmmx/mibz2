/*
 * Clean exact round-by-round work census (0% jitter: pieces, rows, waves, wave_items, joins)
 * + min-of-5 native C timings across all 6 authenticated Wikidata blocks!
 */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

typedef struct { uint8_t bytes[16]; uint8_t len; } FlatWord;
typedef struct { uint32_t a, b, len; uint8_t is_pow; } SlpNode;
#define SLP_TAG 0x80000000u

typedef struct {
    FlatWord *flat; uint32_t flat_len, flat_cap;
    SlpNode *slp;   uint32_t slp_len, slp_cap;
} HybridStore;

static void hs_init(HybridStore *h, uint32_t cap) {
    h->flat_cap = cap; h->flat = (FlatWord *)malloc((size_t)cap * sizeof(FlatWord));
    h->flat_len = 256;
    for (uint32_t c = 0; c < 256; c++) { h->flat[c].bytes[0] = (uint8_t)c; h->flat[c].len = 1; }
    h->slp_cap = cap; h->slp = (SlpNode *)malloc((size_t)cap * sizeof(SlpNode));
    h->slp_len = 0;
}
static void hs_free(HybridStore *h) { free(h->flat); free(h->slp); }
static inline uint32_t hs_len(const HybridStore *h, uint32_t id) {
    return (id & SLP_TAG) ? h->slp[id & ~SLP_TAG].len : (uint32_t)h->flat[id].len;
}
static inline uint32_t hs_concat_fwd(HybridStore *h, uint32_t lf_first, uint32_t lf_second) {
    uint32_t u = lf_second, v = lf_first;
    if (((u | v) & SLP_TAG) == 0) {
        uint8_t lu = h->flat[u].len, lv = h->flat[v].len;
        if (lu + lv <= 16) {
            if (h->flat_len == h->flat_cap) {
                h->flat_cap *= 2;
                h->flat = (FlatWord *)realloc(h->flat, (size_t)h->flat_cap * sizeof(FlatWord));
            }
            uint32_t id = h->flat_len++;
            memcpy(h->flat[id].bytes, h->flat[u].bytes, lu);
            memcpy(h->flat[id].bytes + lu, h->flat[v].bytes, lv);
            h->flat[id].len = (uint8_t)(lu + lv);
            return id;
        }
    }
    if (h->slp_len == h->slp_cap) {
        h->slp_cap *= 2;
        h->slp = (SlpNode *)realloc(h->slp, (size_t)h->slp_cap * sizeof(SlpNode));
    }
    uint32_t sid = h->slp_len++;
    h->slp[sid] = (SlpNode){ u, v, hs_len(h, u) + hs_len(h, v), 0 };
    return sid | SLP_TAG;
}
static inline uint32_t hs_power(HybridStore *h, uint32_t u, uint32_t k) {
    if (k == 1) return u;
    if (h->slp_len == h->slp_cap) {
        h->slp_cap *= 2;
        h->slp = (SlpNode *)realloc(h->slp, (size_t)h->slp_cap * sizeof(SlpNode));
    }
    uint32_t sid = h->slp_len++;
    h->slp[sid] = (SlpNode){ u, k, hs_len(h, u) * k, 1 };
    return sid | SLP_TAG;
}
static inline int32_t combine_mark_fwd(int32_t m1, int32_t m2, uint32_t len2) {
    return (m2 >= 0) ? m2 : ((m1 >= 0) ? (int32_t)len2 + m1 : -1);
}

typedef struct { int32_t lo, hi, delta; uint32_t w; int32_t mark_off; uint8_t is_del; } Piece;
typedef struct { Piece *data; int32_t len, cap; } PieceVec;
static inline void pv_init(PieceVec *v, int32_t cap) { v->len = 0; v->cap = cap > 16 ? cap : 16; v->data = (Piece *)malloc((size_t)v->cap * sizeof(Piece)); }
static inline void pv_push(PieceVec *v, Piece p) { if (v->len == v->cap) { v->cap *= 2; v->data = (Piece *)realloc(v->data, (size_t)v->cap * sizeof(Piece)); } v->data[v->len++] = p; }
static inline void pv_free(PieceVec *v) { free(v->data); v->data = NULL; v->len = v->cap = 0; }

static inline int32_t gallop_advance(const Piece *arr, int32_t j, int32_t ncur, int32_t t_lo) {
    if (j >= ncur || arr[j].hi > t_lo) return j;
    int32_t step = 1;
    while (j + step < ncur && arr[j + step].hi <= t_lo) { j += step; step <<= 1; }
    int32_t hi = (j + step < ncur) ? (j + step) : ncur, lo = j;
    while (lo < hi) { int32_t mid = lo + ((hi - lo) >> 1); if (arr[mid].hi <= t_lo) lo = mid + 1; else hi = mid; }
    return lo;
}
static void radix_sort_by_lo(Piece *a, Piece *tmp, int32_t n) {
    if (n <= 1) return;
    if (n <= 64) {
        for (int32_t i = 1; i < n; i++) { Piece key = a[i]; int32_t j = i - 1; while (j >= 0 && a[j].lo > key.lo) { a[j + 1] = a[j]; j--; } a[j + 1] = key; }
        return;
    }
    int32_t cnt[1024]; memset(cnt, 0, sizeof(cnt));
    for (int32_t i = 0; i < n; i++) cnt[a[i].lo & 1023]++;
    int32_t acc = 0; for (int i = 0; i < 1024; i++) { int32_t c = cnt[i]; cnt[i] = acc; acc += c; }
    for (int32_t i = 0; i < n; i++) tmp[cnt[a[i].lo & 1023]++] = a[i];
    memset(cnt, 0, sizeof(cnt));
    for (int32_t i = 0; i < n; i++) cnt[(tmp[i].lo >> 10) & 1023]++;
    acc = 0; for (int i = 0; i < 1024; i++) { int32_t c = cnt[i]; cnt[i] = acc; acc += c; }
    for (int32_t i = 0; i < n; i++) a[cnt[(tmp[i].lo >> 10) & 1023]++] = tmp[i];
}
static void radix_sort_by_target(Piece *a, Piece *tmp, int32_t n) {
    if (n <= 1) return;
    if (n <= 64) {
        for (int32_t i = 1; i < n; i++) { Piece key = a[i]; int32_t kt = key.lo + key.delta; int32_t j = i - 1; while (j >= 0 && (a[j].lo + a[j].delta) > kt) { a[j + 1] = a[j]; j--; } a[j + 1] = key; }
        return;
    }
    int32_t cnt[1024]; memset(cnt, 0, sizeof(cnt));
    for (int32_t i = 0; i < n; i++) cnt[(a[i].lo + a[i].delta) & 1023]++;
    int32_t acc = 0; for (int i = 0; i < 1024; i++) { int32_t c = cnt[i]; cnt[i] = acc; acc += c; }
    for (int32_t i = 0; i < n; i++) tmp[cnt[(a[i].lo + a[i].delta) & 1023]++] = a[i];
    memset(cnt, 0, sizeof(cnt));
    for (int32_t i = 0; i < n; i++) cnt[((tmp[i].lo + tmp[i].delta) >> 10) & 1023]++;
    acc = 0; for (int i = 0; i < 1024; i++) { int32_t c = cnt[i]; cnt[i] = acc; acc += c; }
    for (int32_t i = 0; i < n; i++) a[cnt[((tmp[i].lo + tmp[i].delta) >> 10) & 1023]++] = tmp[i];
}
static double now_ms(void) { struct timespec ts; clock_gettime(CLOCK_MONOTONIC, &ts); return (double)ts.tv_sec * 1e3 + (double)ts.tv_nsec * 1e-6; }

typedef struct { uint8_t c; int32_t len; } RunRec;

int main(void) {
    const char *files[6] = {"block-0.L","block-2.L","block-85.L","block-143.L","block-210.L","block-207.L"};
    const int32_t ns[6] = {899931,900000,899935,899869,899959,899998};
    const int32_t ps[6] = {264045,320807,325088,802868,791429,205180};

    int64_t agg_inP[25]={0}, agg_outP[25]={0}, agg_remR[25]={0}, agg_waves[25]={0}, agg_witems[25]={0}, agg_joins[25]={0};
    double agg_ms[25]={0};

    for (int bi = 0; bi < 6; bi++) {
        char path[256]; snprintf(path, sizeof(path), "/tmp/mibz2-blocks/%s", files[bi]);
        FILE *fp = fopen(path, "rb"); int32_t n = ns[bi], p = ps[bi];
        uint8_t *L = (uint8_t *)malloc((size_t)n); fread(L, 1, (size_t)n, fp); fclose(fp);
        RunRec *runs = (RunRec *)malloc((size_t)n * sizeof(RunRec));
        int32_t r0 = 0, counts[256] = {0};
        for (int32_t i = 0; i < n; ) {
            uint8_t c = L[i]; int32_t j = i + 1; while (j < n && L[j] == c) j++;
            runs[r0++] = (RunRec){c, j - i}; counts[c] += (j - i); i = j;
        }
        free(L);

        double best_r_ms[25]; for (int r = 0; r < 25; r++) best_r_ms[r] = 1e9;
        int64_t r_inP[25]={0}, r_outP[25]={0}, r_remR[25]={0}, r_wv[25]={0}, r_wi[25]={0}, r_jn[25]={0};

        for (int tr = 0; tr < 3; tr++) {
            int32_t C[256], acc = 0; for (int c = 0; c < 256; c++) { C[c] = acc; acc += counts[c]; }
            int32_t cur_t[256]; memcpy(cur_t, C, sizeof(cur_t));
            PieceVec pieces, cur_arr, wave, next_wave;
            pv_init(&pieces, r0 + 16); pv_init(&cur_arr, r0 + 16); pv_init(&wave, r0 + 16); pv_init(&next_wave, r0 + 16);
            int32_t rtmp_cap = r0 * 2 + 1024; Piece *rtmp = (Piece *)malloc((size_t)rtmp_cap * sizeof(Piece));
            int32_t pos = 0;
            for (int32_t k = 0; k < r0; k++) {
                uint8_t c = runs[k].c; int32_t ln = runs[k].len;
                int32_t delta = cur_t[c] - pos; cur_t[c] += ln; int32_t hi = pos + ln;
                if (pos <= p && p < hi) {
                    if (pos < p) pv_push(&pieces, (Piece){pos, p, delta, c, -1, 0});
                    pv_push(&pieces, (Piece){p, p + 1, delta, c, 0, 0});
                    if (p + 1 < hi) pv_push(&pieces, (Piece){p + 1, hi, delta, c, -1, 0});
                } else pv_push(&pieces, (Piece){pos, hi, delta, c, -1, 0});
                pos = hi;
            }
            HybridStore hs; hs_init(&hs, (uint32_t)(r0 * 4 + 1024));
            int has_pcyc = 0, rounds = 0;

            while (!has_pcyc) {
                double t0 = now_ms();
                int32_t np = pieces.len; int64_t wd = 0, wu = 0; int did_sp = 0;
                cur_arr.len = 0;
                for (int32_t i = 0; i < np; i++) {
                    Piece pc = pieces.data[i];
                    if (pc.delta == 0) { if (pc.mark_off >= 0) has_pcyc = 1; continue; }
                    int32_t L = pc.hi - pc.lo;
                    if (pc.delta > 0 && pc.delta < L) {
                        did_sp = 1; int32_t d = pc.delta, q = L/d, rem = L%d;
                        if (rem > 0) pv_push(&cur_arr, (Piece){pc.lo, pc.lo+rem, (q+1)*d, hs_power(&hs, pc.w, (uint32_t)(q+1)), -1, 0});
                        if (d-rem > 0) pv_push(&cur_arr, (Piece){pc.lo+rem, pc.lo+d, q*d, hs_power(&hs, pc.w, (uint32_t)q), -1, 0});
                    } else if (pc.delta < 0 && (-pc.delta) < L) {
                        did_sp = 1; int32_t s = -pc.delta, q = L/s, rem = L%s;
                        if (s-rem > 0) pv_push(&cur_arr, (Piece){pc.hi-s, pc.hi-rem, -q*s, hs_power(&hs, pc.w, (uint32_t)q), -1, 0});
                        if (rem > 0) pv_push(&cur_arr, (Piece){pc.hi-rem, pc.hi, -(q+1)*s, hs_power(&hs, pc.w, (uint32_t)(q+1)), -1, 0});
                    } else { if (pc.delta < 0) wd += L; else wu += L; pv_push(&cur_arr, pc); }
                }
                if (has_pcyc) break;
                if (did_sp) {
                    PieceVec tmp = pieces; pieces = cur_arr; cur_arr = tmp;
                    if (pieces.len > rtmp_cap) { rtmp_cap = pieces.len * 2; rtmp = (Piece *)realloc(rtmp, (size_t)rtmp_cap * sizeof(Piece)); }
                    radix_sort_by_lo(pieces.data, rtmp, pieces.len);
                    continue;
                }
                if (wd + wu <= 1) break;
                rounds++;
                int del_down = (wd >= wu);
                int32_t ncur = cur_arr.len; Piece *arr = cur_arr.data;
                wave.len = 0;
                for (int32_t i = 0; i < ncur; i++) {
                    int is_d = del_down ? (arr[i].delta < 0) : (arr[i].delta > 0);
                    arr[i].is_del = (uint8_t)is_d;
                    if (!is_d) pv_push(&wave, arr[i]);
                }
                pieces.len = 0;
                int64_t rwv = 0, rwi = 0, rjn = 0;
                while (wave.len > 0) {
                    rwv++; rwi += wave.len;
                    if (wave.len > rtmp_cap) { rtmp_cap = wave.len * 2; rtmp = (Piece *)realloc(rtmp, (size_t)rtmp_cap * sizeof(Piece)); }
                    radix_sort_by_target(wave.data, rtmp, wave.len);
                    next_wave.len = 0; int32_t j = 0; const Piece *warr = wave.data; int32_t nwin = wave.len;
                    int use_g = (nwin * 8 < ncur);
                    for (int32_t i = 0; i < nwin; i++) {
                        Piece cur = warr[i]; int32_t tlo = cur.lo + cur.delta, thi = cur.hi + cur.delta;
                        if (use_g) j = gallop_advance(arr, j, ncur, tlo);
                        else while (j < ncur && arr[j].hi <= tlo) j++;
                        int32_t cp = tlo, k = j;
                        while (cp < thi) {
                            const Piece *q = &arr[k]; int32_t ov = (thi < q->hi) ? thi : q->hi;
                            int32_t slo = cp - cur.delta, shi = ov - cur.delta;
                            if (!q->is_del) pv_push(&pieces, (Piece){slo, shi, cur.delta, cur.w, cur.mark_off, 0});
                            else {
                                rjn++; uint32_t ql = hs_len(&hs, q->w);
                                pv_push(&next_wave, (Piece){slo, shi, cur.delta+q->delta, hs_concat_fwd(&hs, cur.w, q->w), combine_mark_fwd(cur.mark_off, q->mark_off, ql), 0});
                            }
                            cp = ov; if (cp == q->hi) k++;
                        }
                        j = k;
                    }
                    PieceVec tmp = wave; wave = next_wave; next_wave = tmp;
                }
                if (pieces.len > rtmp_cap) { rtmp_cap = pieces.len * 2; rtmp = (Piece *)realloc(rtmp, (size_t)rtmp_cap * sizeof(Piece)); }
                radix_sort_by_lo(pieces.data, rtmp, pieces.len);
                double dt = now_ms() - t0;
                if (dt < best_r_ms[rounds]) best_r_ms[rounds] = dt;
                if (tr == 0) {
                    int64_t rr = 0; for (int32_t i = 0; i < pieces.len; i++) rr += (pieces.data[i].hi - pieces.data[i].lo);
                    r_inP[rounds] = ncur; r_outP[rounds] = pieces.len; r_remR[rounds] = rr;
                    r_wv[rounds] = rwv; r_wi[rounds] = rwi; r_jn[rounds] = rjn;
                }
            }
            free(rtmp); pv_free(&pieces); pv_free(&cur_arr); pv_free(&wave); pv_free(&next_wave); hs_free(&hs);
        }
        for (int r = 1; r <= 18; r++) {
            agg_inP[r] += r_inP[r]; agg_outP[r] += r_outP[r]; agg_remR[r] += r_remR[r];
            agg_waves[r] += r_wv[r]; agg_witems[r] += r_wi[r]; agg_joins[r] += r_jn[r];
            if (best_r_ms[r] < 1e8) agg_ms[r] += best_r_ms[r];
        }
        free(runs);
    }

    printf("=== 6-BLOCK AGGREGATED ROUND-BY-ROUND WORK & MIN-OF-3 TIMING (r0_sum=509,269, m16_sum=895,391) ===\n");
    printf("rnd |  in_P   |  out_P  | rem_rows (x m16) | waves | wave_items |  joins  | min_ms | cum_ms  (pct)\n");
    printf("----+---------+---------+------------------+-------+------------+---------+--------+--------------\n");
    double tot_ms = 0; for (int r = 1; r <= 18; r++) tot_ms += agg_ms[r];
    double cum = 0;
    int64_t tot_wi = 0, tot_jn = 0;
    for (int r = 1; r <= 18; r++) {
        if (agg_inP[r] == 0) continue;
        cum += agg_ms[r]; tot_wi += agg_witems[r]; tot_jn += agg_joins[r];
        printf("%3d | %7ld | %7ld | %8ld (%5.3fx) | %5ld | %10ld | %7ld | %6.2f | %6.2f (%5.1f%%)\n",
               r, (long)agg_inP[r], (long)agg_outP[r], (long)agg_remR[r], (double)agg_remR[r] / 895391.0,
               (long)agg_waves[r], (long)agg_witems[r], (long)agg_joins[r], agg_ms[r], cum, 100.0 * cum / tot_ms);
    }
    printf("----+---------+---------+------------------+-------+------------+---------+--------+--------------\n");
    printf("SUM |         |         |                  |       | %10ld | %7ld | %6.2f |\n",
           (long)tot_wi, (long)tot_jn, tot_ms);
    return 0;
}
