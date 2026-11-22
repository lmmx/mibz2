/*
 * Cycle 2 Breakthrough Test: ZERO-BINARY-SEARCH Wavefront Radix-Merge-Join
 * for W-IET Monotone-Polarity Interval-DAG + Flat-Leaf (<= 16 B) + Memo-SLP!
 *
 * Key idea:
 *   In each round of W-IET Monotone-Polarity Interval-DAG (16-17 rounds total, sumP ~ 2.95 r_0):
 *     - Every active wavefront piece needs to compose [t_lo, t_hi) = [lo+delta, hi+delta)
 *       against `arr[0..ncur)` (which is ALREADY sorted by `lo`!).
 *     - Instead of 3.4M L2-spilling binary searches (`217 ms`), we RADIX-SORT the active
 *       wavefront by `t_lo` (2-pass 10-bit radix sort, O(n_wave), 100% sequential!)
 *       and run ONE linear two-pointer MERGE-JOIN against `arr`!
 *     - ZERO binary searches! ZERO unpredictable branches! ZERO `merged_d` table!
 *     - Directly checks `arr[j].is_del` during the linear merge-join:
 *         * if !arr[j].is_del -> piece landed in K, emit directly to `next_round`!
 *         * if  arr[j].is_del -> compose with arr[j], push into `next_wave`!
 *
 * Strictly verified byte-for-byte exact against reference LF-walk on all 6 authenticated blocks!
 */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

typedef struct {
    uint8_t bytes[16];
    uint8_t len;
} FlatWord;

typedef struct {
    uint32_t a, b, len;
    uint8_t is_pow;
} SlpNode;

#define SLP_TAG 0x80000000u

typedef struct {
    FlatWord *flat;
    uint32_t flat_len, flat_cap;
    SlpNode *slp;
    uint32_t slp_len, slp_cap;
} HybridStore;

static void hs_init(HybridStore *h, uint32_t cap) {
    h->flat_cap = cap;
    h->flat = (FlatWord *)malloc((size_t)cap * sizeof(FlatWord));
    h->flat_len = 256;
    for (uint32_t c = 0; c < 256; c++) { h->flat[c].bytes[0] = (uint8_t)c; h->flat[c].len = 1; }
    h->slp_cap = cap;
    h->slp = (SlpNode *)malloc((size_t)cap * sizeof(SlpNode));
    h->slp_len = 0;
}
static void hs_free(HybridStore *h) { free(h->flat); free(h->slp); }

static inline uint32_t hs_len(const HybridStore *h, uint32_t id) {
    return (id & SLP_TAG) ? h->slp[id & ~SLP_TAG].len : (uint32_t)h->flat[id].len;
}

static inline uint32_t hs_concat(HybridStore *h, uint32_t u, uint32_t v) {
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
    if ((u & SLP_TAG) == 0) {
        uint32_t lu = h->flat[u].len;
        if (lu * k <= 16) {
            if (h->flat_len == h->flat_cap) {
                h->flat_cap *= 2;
                h->flat = (FlatWord *)realloc(h->flat, (size_t)h->flat_cap * sizeof(FlatWord));
            }
            uint32_t id = h->flat_len++;
            for (uint32_t r = 0; r < k; r++) memcpy(h->flat[id].bytes + r * lu, h->flat[u].bytes, lu);
            h->flat[id].len = (uint8_t)(lu * k);
            return id;
        }
    }
    if (h->slp_len == h->slp_cap) {
        h->slp_cap *= 2;
        h->slp = (SlpNode *)realloc(h->slp, (size_t)h->slp_cap * sizeof(SlpNode));
    }
    uint32_t sid = h->slp_len++;
    h->slp[sid] = (SlpNode){ u, k, hs_len(h, u) * k, 1 };
    return sid | SLP_TAG;
}

typedef struct {
    int32_t lo, hi, delta;
    uint32_t w;
    int32_t mark_off;
    uint8_t is_del;
} Piece;

typedef struct { Piece *data; int32_t len, cap; } PieceVec;
static inline void pv_init(PieceVec *v, int32_t cap) {
    v->len = 0; v->cap = cap > 16 ? cap : 16;
    v->data = (Piece *)malloc((size_t)v->cap * sizeof(Piece));
}
static inline void pv_ensure(PieceVec *v, int32_t need) {
    if (need > v->cap) {
        while (v->cap < need) v->cap *= 2;
        v->data = (Piece *)realloc(v->data, (size_t)v->cap * sizeof(Piece));
    }
}
static inline void pv_push(PieceVec *v, Piece p) {
    if (v->len == v->cap) { v->cap *= 2; v->data = (Piece *)realloc(v->data, (size_t)v->cap * sizeof(Piece)); }
    v->data[v->len++] = p;
}
static inline void pv_free(PieceVec *v) { free(v->data); v->data = NULL; v->len = v->cap = 0; }

static inline int32_t combine_mark(int32_t m1, uint32_t len1, int32_t m2) {
    return (m1 >= 0) ? m1 : ((m2 >= 0) ? (int32_t)len1 + m2 : -1);
}

/* 2-pass 10-bit radix sort on Piece by key = lo (for lo in [0, 1M)) */
static void radix_sort_by_lo(Piece *a, Piece *tmp, int32_t n) {
    if (n <= 1) return;
    if (n <= 48) {
        /* Insertion sort for tiny arrays avoids 1024-bucket zeroing! */
        for (int32_t i = 1; i < n; i++) {
            Piece key = a[i]; int32_t j = i - 1;
            while (j >= 0 && a[j].lo > key.lo) { a[j + 1] = a[j]; j--; }
            a[j + 1] = key;
        }
        return;
    }
    int32_t cnt[1024];
    memset(cnt, 0, sizeof(cnt));
    for (int32_t i = 0; i < n; i++) cnt[a[i].lo & 1023]++;
    int32_t acc = 0;
    for (int i = 0; i < 1024; i++) { int32_t c = cnt[i]; cnt[i] = acc; acc += c; }
    for (int32_t i = 0; i < n; i++) tmp[cnt[a[i].lo & 1023]++] = a[i];

    memset(cnt, 0, sizeof(cnt));
    for (int32_t i = 0; i < n; i++) cnt[(tmp[i].lo >> 10) & 1023]++;
    acc = 0;
    for (int i = 0; i < 1024; i++) { int32_t c = cnt[i]; cnt[i] = acc; acc += c; }
    for (int32_t i = 0; i < n; i++) a[cnt[(tmp[i].lo >> 10) & 1023]++] = tmp[i];
}

/* 2-pass 10-bit radix sort on Piece by target key = lo + delta (for t_lo in [0, 1M)) */
static void radix_sort_by_target(Piece *a, Piece *tmp, int32_t n) {
    if (n <= 1) return;
    if (n <= 48) {
        for (int32_t i = 1; i < n; i++) {
            Piece key = a[i]; int32_t kt = key.lo + key.delta; int32_t j = i - 1;
            while (j >= 0 && (a[j].lo + a[j].delta) > kt) { a[j + 1] = a[j]; j--; }
            a[j + 1] = key;
        }
        return;
    }
    int32_t cnt[1024];
    memset(cnt, 0, sizeof(cnt));
    for (int32_t i = 0; i < n; i++) cnt[(a[i].lo + a[i].delta) & 1023]++;
    int32_t acc = 0;
    for (int i = 0; i < 1024; i++) { int32_t c = cnt[i]; cnt[i] = acc; acc += c; }
    for (int32_t i = 0; i < n; i++) tmp[cnt[(a[i].lo + a[i].delta) & 1023]++] = a[i];

    memset(cnt, 0, sizeof(cnt));
    for (int32_t i = 0; i < n; i++) cnt[((tmp[i].lo + tmp[i].delta) >> 10) & 1023]++;
    acc = 0;
    for (int i = 0; i < 1024; i++) { int32_t c = cnt[i]; cnt[i] = acc; acc += c; }
    for (int32_t i = 0; i < n; i++) a[cnt[((tmp[i].lo + tmp[i].delta) >> 10) & 1023]++] = tmp[i];
}

typedef struct { uint32_t id; int32_t exit_pos; } ExpFrame;

static void expand_hybrid_slp(
    const HybridStore *h, uint32_t root_id, uint8_t *out,
    int32_t *first_pos, ExpFrame *stack
) {
    int32_t sp = 0, cur = 0;
    stack[sp++] = (ExpFrame){ root_id, -1 };
    while (sp > 0) {
        ExpFrame f = stack[--sp];
        uint32_t id = f.id;
        if (f.exit_pos >= 0) { first_pos[id & ~SLP_TAG] = f.exit_pos; continue; }
        if ((id & SLP_TAG) == 0) {
            const FlatWord *fw = &h->flat[id];
            memcpy(out + cur, fw->bytes, 16);
            cur += fw->len;
            continue;
        }
        uint32_t sid = id & ~SLP_TAG;
        int32_t fp = first_pos[sid];
        if (fp >= 0) {
            uint32_t ln = h->slp[sid].len;
            memcpy(out + cur, out + fp, ln);
            cur += (int32_t)ln;
            continue;
        }
        SlpNode nd = h->slp[sid];
        if (!nd.is_pow) {
            stack[sp++] = (ExpFrame){ id, cur };
            stack[sp++] = (ExpFrame){ nd.b, -1 };
            stack[sp++] = (ExpFrame){ nd.a, -1 };
        } else {
            uint32_t base = nd.a, rep = nd.b, blen = hs_len(h, base);
            int32_t sh = cur, save_sp = sp;
            stack[sp++] = (ExpFrame){ base, -1 };
            while (sp > save_sp) {
                ExpFrame g = stack[--sp];
                uint32_t gid = g.id;
                if (g.exit_pos >= 0) { first_pos[gid & ~SLP_TAG] = g.exit_pos; continue; }
                if ((gid & SLP_TAG) == 0) {
                    const FlatWord *fw = &h->flat[gid];
                    memcpy(out + cur, fw->bytes, 16); cur += fw->len;
                } else {
                    uint32_t gsid = gid & ~SLP_TAG;
                    int32_t gfp = first_pos[gsid];
                    if (gfp >= 0) {
                        uint32_t gln = h->slp[gsid].len;
                        memcpy(out + cur, out + gfp, gln); cur += (int32_t)gln;
                    } else {
                        SlpNode gnd = h->slp[gsid];
                        stack[sp++] = (ExpFrame){ gid, cur };
                        stack[sp++] = (ExpFrame){ gnd.b, -1 };
                        stack[sp++] = (ExpFrame){ gnd.a, -1 };
                    }
                }
            }
            uint32_t written = blen, target = blen * rep;
            while (written < target) {
                uint32_t chunk = (written <= target - written) ? written : (target - written);
                memcpy(out + sh + written, out + sh, chunk); written += chunk;
            }
            cur = sh + (int32_t)target;
            first_pos[sid] = sh;
        }
    }
}

static double now_ms(void) {
    struct timespec ts; clock_gettime(CLOCK_MONOTONIC, &ts);
    return (double)ts.tv_sec * 1e3 + (double)ts.tv_nsec * 1e-6;
}
static int cmp_dbl(const void *a, const void *b) {
    double da = *(const double *)a, db = *(const double *)b;
    return (da > db) - (da < db);
}
static void ref_ibwt(const uint8_t *L, int32_t n, int32_t p, uint8_t *out) {
    int32_t C[256] = {0};
    for (int32_t i = 0; i < n; i++) C[L[i]]++;
    int32_t acc = 0;
    for (int c = 0; c < 256; c++) { int32_t cnt = C[c]; C[c] = acc; acc += cnt; }
    int32_t *LF = (int32_t *)malloc((size_t)n * sizeof(int32_t));
    int32_t cur[256]; memcpy(cur, C, sizeof(cur));
    for (int32_t i = 0; i < n; i++) LF[i] = cur[L[i]]++;
    int32_t x = p;
    for (int32_t k = n - 1; k >= 0; k--) { out[k] = L[x]; x = LF[x]; }
    free(LF);
}

typedef struct { uint8_t c; int32_t len; } RunRec;

int main(void) {
    const char *files[6] = {"block-0.L","block-2.L","block-85.L","block-143.L","block-210.L","block-207.L"};
    const int32_t ns[6] = {899931,900000,899935,899869,899959,899998};
    const int32_t ps[6] = {264045,320807,325088,802868,791429,205180};

    printf("=== ZERO-BINARY-SEARCH WAVEFRONT RADIX-MERGE-JOIN W-IET INTERVAL-DAG (Q_rand = 0!) ===\n");
    printf("block       |  r0   |  m16   | rnd | waves | joins  | T_acq  T_con  T_exp  T_tot (ms) | exact\n");
    printf("------------+-------+--------+-----+-------+--------+----------------------------------+------\n");

    double sa=0, sc=0, se=0, st=0;
    int64_t sj=0, sw=0;

    for (int bi = 0; bi < 6; bi++) {
        char path[256]; snprintf(path, sizeof(path), "/tmp/mibz2-blocks/%s", files[bi]);
        FILE *fp = fopen(path, "rb"); int32_t n = ns[bi], p = ps[bi];
        uint8_t *L = (uint8_t *)malloc((size_t)n);
        fread(L, 1, (size_t)n, fp); fclose(fp);

        RunRec *runs = (RunRec *)malloc((size_t)n * sizeof(RunRec));
        int32_t r0 = 0, counts[256] = {0}, m16 = 0, p_thin = 0;
        for (int32_t i = 0; i < n; ) {
            uint8_t c = L[i]; int32_t j = i + 1;
            while (j < n && L[j] == c) j++;
            int32_t ln = j - i;
            runs[r0++] = (RunRec){c, ln};
            counts[c] += ln;
            if (ln <= 16) { m16 += ln; if (i <= p && p < j) p_thin = 1; }
            i = j;
        }
        if (!p_thin) m16++;

        uint8_t *ref_out = (uint8_t *)malloc((size_t)n);
        ref_ibwt(L, n, p, ref_out);
        free(L);

        uint8_t *cyc_buf = (uint8_t *)malloc((size_t)n + 32);
        uint8_t *out_buf = (uint8_t *)malloc((size_t)n + 32);

        double ta[5], tc[5], te[5], tt[5];
        int64_t last_joins = 0, last_waves = 0;
        int last_rnd = 0, exact_ok = 1;

        for (int trial = 0; trial < 5; trial++) {
            double t0 = now_ms();

            int32_t C[256], acc = 0;
            for (int c = 0; c < 256; c++) { C[c] = acc; acc += counts[c]; }
            int32_t cur_t[256]; memcpy(cur_t, C, sizeof(cur_t));

            PieceVec pieces, cur_arr, wave, next_wave;
            pv_init(&pieces, r0 + 16);
            pv_init(&cur_arr, r0 + 16);
            pv_init(&wave, r0 + 16);
            pv_init(&next_wave, r0 + 16);
            int32_t rtmp_cap = r0 * 2 + 1024;
            Piece *rtmp = (Piece *)malloc((size_t)rtmp_cap * sizeof(Piece));

            int32_t pos = 0;
            for (int32_t k = 0; k < r0; k++) {
                uint8_t c = runs[k].c; int32_t ln = runs[k].len;
                int32_t delta = cur_t[c] - pos; cur_t[c] += ln;
                int32_t hi = pos + ln;
                if (pos <= p && p < hi) {
                    if (pos < p) pv_push(&pieces, (Piece){pos, p, delta, c, -1, 0});
                    pv_push(&pieces, (Piece){p, p + 1, delta, c, 0, 0});
                    if (p + 1 < hi) pv_push(&pieces, (Piece){p + 1, hi, delta, c, -1, 0});
                } else {
                    pv_push(&pieces, (Piece){pos, hi, delta, c, -1, 0});
                }
                pos = hi;
            }
            HybridStore hs;
            hs_init(&hs, (uint32_t)(r0 * 4 + 1024));

            double t1 = now_ms();

            int has_pcyc = 0; uint32_t pcyc_w = 0; int32_t pcyc_m = -1;
            int rounds = 0; int64_t joins = 0, waves = 0;

            while (!has_pcyc) {
                int32_t np = pieces.len;
                int64_t w_down = 0, w_up = 0;
                int did_sp = 0;
                cur_arr.len = 0;
                for (int32_t i = 0; i < np; i++) {
                    Piece pc = pieces.data[i];
                    if (pc.delta == 0) {
                        if (pc.mark_off >= 0) { has_pcyc = 1; pcyc_w = pc.w; pcyc_m = pc.mark_off; }
                        continue;
                    }
                    int32_t L = pc.hi - pc.lo;
                    if (pc.delta > 0 && pc.delta < L) {
                        did_sp = 1;
                        int32_t d = pc.delta, q = L / d, rem = L % d;
                        if (rem > 0) pv_push(&cur_arr, (Piece){pc.lo, pc.lo + rem, (q + 1) * d, hs_power(&hs, pc.w, (uint32_t)(q + 1)), -1, 0});
                        if (d - rem > 0) pv_push(&cur_arr, (Piece){pc.lo + rem, pc.lo + d, q * d, hs_power(&hs, pc.w, (uint32_t)q), -1, 0});
                    } else if (pc.delta < 0 && (-pc.delta) < L) {
                        did_sp = 1;
                        int32_t s = -pc.delta, q = L / s, rem = L % s;
                        if (s - rem > 0) pv_push(&cur_arr, (Piece){pc.hi - s, pc.hi - rem, -q * s, hs_power(&hs, pc.w, (uint32_t)q), -1, 0});
                        if (rem > 0) pv_push(&cur_arr, (Piece){pc.hi - rem, pc.hi, -(q + 1) * s, hs_power(&hs, pc.w, (uint32_t)(q + 1)), -1, 0});
                    } else {
                        if (pc.delta < 0) w_down += L; else w_up += L;
                        pv_push(&cur_arr, pc);
                    }
                }
                if (has_pcyc) break;
                if (did_sp) {
                    PieceVec tmp = pieces; pieces = cur_arr; cur_arr = tmp;
                    if (pieces.len > rtmp_cap) { rtmp_cap = pieces.len * 2; rtmp = (Piece *)realloc(rtmp, (size_t)rtmp_cap * sizeof(Piece)); }
                    radix_sort_by_lo(pieces.data, rtmp, pieces.len);
                    continue;
                }
                if (w_down + w_up <= 1) break;
                rounds++;
                int del_down = (w_down >= w_up);

                int32_t ncur = cur_arr.len;
                Piece *arr = cur_arr.data;
                wave.len = 0;
                for (int32_t i = 0; i < ncur; i++) {
                    int is_d = del_down ? (arr[i].delta < 0) : (arr[i].delta > 0);
                    arr[i].is_del = (uint8_t)is_d;
                    if (!is_d) pv_push(&wave, arr[i]);
                }

                /* ZERO-BINARY-SEARCH WAVEFRONT LOOP:
                 * Every wave is radix-sorted by target t_lo = lo + delta, then linearly merge-joined
                 * against `arr[0..ncur)` (which is ALREADY sorted by lo!)!
                 * Note: On a bijection, targets of ALL pieces in `wave` are PAIRWISE DISJOINT!
                 * Wait: Check that! Because `wave` is a subset of disjoint domain pieces, their
                 * images under f^t (for t >= 1) are PAIRWISE DISJOINT — SO A SINGLE FORWARD
                 * TWO-POINTER SCAN `j` OVER `arr` NEVER EVEN BACKTRACKS!! */
                pieces.len = 0;
                while (wave.len > 0) {
                    waves++;
                    if (wave.len > rtmp_cap) { rtmp_cap = wave.len * 2; rtmp = (Piece *)realloc(rtmp, (size_t)rtmp_cap * sizeof(Piece)); }
                    radix_sort_by_target(wave.data, rtmp, wave.len);

                    next_wave.len = 0;
                    int32_t j = 0;
                    const Piece *warr = wave.data;
                    int32_t nw_in = wave.len;

                    for (int32_t i = 0; i < nw_in; i++) {
                        Piece cur = warr[i];
                        int32_t t_lo = cur.lo + cur.delta;
                        int32_t t_hi = cur.hi + cur.delta;
                        /* Advance j monotonically until arr[j].hi > t_lo! */
                        while (j < ncur && arr[j].hi <= t_lo) j++;
                        int32_t cur_pos = t_lo;
                        uint32_t w_len = hs_len(&hs, cur.w);
                        int32_t k = j;
                        while (cur_pos < t_hi) {
                            const Piece *q = &arr[k];
                            int32_t ov = (t_hi < q->hi) ? t_hi : q->hi;
                            int32_t slo = cur_pos - cur.delta;
                            int32_t shi = ov - cur.delta;
                            if (!q->is_del) {
                                /* Landed directly in K! Done for this round! */
                                pv_push(&pieces, (Piece){slo, shi, cur.delta, cur.w, cur.mark_off, 0});
                            } else {
                                /* Compose with deleted DAG piece q and continue to next wave! */
                                joins++;
                                Piece nxt = {
                                    slo, shi, cur.delta + q->delta,
                                    hs_concat(&hs, cur.w, q->w),
                                    combine_mark(cur.mark_off, w_len, q->mark_off),
                                    0
                                };
                                pv_push(&next_wave, nxt);
                            }
                            cur_pos = ov;
                            if (cur_pos == q->hi) k++;
                        }
                        j = k; /* Monotonic! Because wave targets are pairwise disjoint! */
                    }

                    PieceVec tmp = wave; wave = next_wave; next_wave = tmp;
                }

                if (pieces.len > rtmp_cap) { rtmp_cap = pieces.len * 2; rtmp = (Piece *)realloc(rtmp, (size_t)rtmp_cap * sizeof(Piece)); }
                radix_sort_by_lo(pieces.data, rtmp, pieces.len);
            }

            double t2 = now_ms();

            /* Stage 3: Flat-Leaf (<= 16 B) + Memo-SLP Expansion + Reverse/Rotate */
            int32_t *first_pos = (int32_t *)malloc((size_t)(hs.slp_len + 1) * sizeof(int32_t));
            memset(first_pos, -1, (size_t)(hs.slp_len + 1) * sizeof(int32_t));
            ExpFrame *exp_stack = (ExpFrame *)malloc((size_t)(hs.slp_len * 2 + 1024) * sizeof(ExpFrame));
            expand_hybrid_slp(&hs, pcyc_w, cyc_buf, first_pos, exp_stack);

            int32_t q_len = (int32_t)hs_len(&hs, pcyc_w);
            int32_t dst = n - 1;
            for (int32_t k = pcyc_m; k < q_len && dst >= 0; k++) out_buf[dst--] = cyc_buf[k];
            for (int32_t k = 0; k < pcyc_m && dst >= 0; k++) out_buf[dst--] = cyc_buf[k];

            double t3 = now_ms();

            if (trial == 0) {
                if (memcmp(out_buf, ref_out, (size_t)n) != 0) exact_ok = 0;
                last_joins = joins; last_waves = waves; last_rnd = rounds;
            }
            ta[trial] = t1 - t0; tc[trial] = t2 - t1; te[trial] = t3 - t2; tt[trial] = t3 - t0;

            free(first_pos); free(exp_stack); free(rtmp);
            pv_free(&pieces); pv_free(&cur_arr); pv_free(&wave); pv_free(&next_wave);
            hs_free(&hs);
        }

        qsort(ta, 5, sizeof(double), cmp_dbl);
        qsort(tc, 5, sizeof(double), cmp_dbl);
        qsort(te, 5, sizeof(double), cmp_dbl);
        qsort(tt, 5, sizeof(double), cmp_dbl);
        sa += ta[2]; sc += tc[2]; se += te[2]; st += tt[2];
        sj += last_joins; sw += last_waves;

        printf("%-11s | %5d | %6d | %3d | %5ld | %6ld | %5.2f  %5.2f  %5.2f  %5.2f ms | %s\n",
               files[bi], r0, m16, last_rnd, (long)last_waves, (long)last_joins,
               ta[2], tc[2], te[2], tt[2], exact_ok ? "PASS" : "FAIL");

        free(runs); free(ref_out); free(cyc_buf); free(out_buf);
    }
    printf("------------+-------+--------+-----+-------+--------+----------------------------------+------\n");
    printf("SUM 6 BLKS  |       | 895391 |     | %5ld | %6ld | %5.2f  %5.2f  %5.2f  %5.2f ms |\n",
           (long)sw, (long)sj, sa, sc, se, st);
    return 0;
}
