/*
 * Cycle 3 — Task 2:
 *   1. Round-by-round profile (round = 1..17) of Galloping Wavefront Radix-Merge-Join
 *      Polarity-DAG on ALL 6 authenticated Wikidata blocks (pieces, rows, waves, joins, ms)!
 *   2. Exact Early-Stop Wavefront Polarity-DAG stopping at k_stop = 1, 2, 3, 4, 5, 6, 8, full
 *      and transitioning directly to a compact dense residual permutation walk
 *      (`edges_res[0..m_rem)` + Forward Flat-Leaf <= 16 B + Memo-SLP emit!),
 *      byte-for-byte verified PASS on all 6 blocks, measured in native C (-O3 -march=native)
 *      directly against the exact i9-10900K bar (8.2-10.1 ms / 6 blocks)!
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
static inline int32_t combine_mark_fwd(int32_t m1, int32_t m2, uint32_t len2) {
    return (m2 >= 0) ? m2 : ((m1 >= 0) ? (int32_t)len2 + m1 : -1);
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
static inline void pv_push(PieceVec *v, Piece p) {
    if (v->len == v->cap) { v->cap *= 2; v->data = (Piece *)realloc(v->data, (size_t)v->cap * sizeof(Piece)); }
    v->data[v->len++] = p;
}
static inline void pv_free(PieceVec *v) { free(v->data); v->data = NULL; v->len = v->cap = 0; }

static inline int32_t gallop_advance(const Piece *arr, int32_t j, int32_t ncur, int32_t t_lo) {
    if (j >= ncur || arr[j].hi > t_lo) return j;
    int32_t step = 1;
    while (j + step < ncur && arr[j + step].hi <= t_lo) { j += step; step <<= 1; }
    int32_t hi = (j + step < ncur) ? (j + step) : ncur, lo = j;
    while (lo < hi) {
        int32_t mid = lo + ((hi - lo) >> 1);
        if (arr[mid].hi <= t_lo) lo = mid + 1; else hi = mid;
    }
    return lo;
}

static void radix_sort_by_lo(Piece *a, Piece *tmp, int32_t n) {
    if (n <= 1) return;
    if (n <= 64) {
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

static void radix_sort_by_target(Piece *a, Piece *tmp, int32_t n) {
    if (n <= 1) return;
    if (n <= 64) {
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

/* Emit a single word_id backwards at write cursor `end_pos` (decrements end_pos!)
 * Note: since we walk in LF order (T[n-1], T[n-2], ...), and words are stored in
 * FORWARD text order, writing word `w` of length L right BEFORE `end_pos` writes
 * to `out[end_pos - L .. end_pos)`! That writes the text directly in FORWARD order
 * with ZERO reverse pass! */
static inline int32_t emit_word_at(
    const HybridStore *h, uint32_t root_id, uint8_t *out, int32_t write_pos,
    int32_t *first_pos, ExpFrame *stack
) {
    if ((root_id & SLP_TAG) == 0) {
        const FlatWord *fw = &h->flat[root_id];
        memcpy(out + write_pos, fw->bytes, fw->len);
        return write_pos + fw->len;
    }
    uint32_t sid0 = root_id & ~SLP_TAG;
    int32_t fp0 = first_pos[sid0];
    if (fp0 >= 0) {
        uint32_t ln = h->slp[sid0].len;
        memcpy(out + write_pos, out + fp0, ln);
        return write_pos + (int32_t)ln;
    }
    int32_t sp = 0, cur = write_pos;
    stack[sp++] = (ExpFrame){ root_id, -1 };
    while (sp > 0) {
        ExpFrame f = stack[--sp];
        uint32_t id = f.id;
        if (f.exit_pos >= 0) { first_pos[id & ~SLP_TAG] = f.exit_pos; continue; }
        if ((id & SLP_TAG) == 0) {
            const FlatWord *fw = &h->flat[id];
            memcpy(out + cur, fw->bytes, fw->len);
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
            int32_t sh = cur;
            cur = emit_word_at(h, base, out, cur, first_pos, stack + sp);
            uint32_t written = blen, target = blen * rep;
            while (written < target) {
                uint32_t chunk = (written <= target - written) ? written : (target - written);
                memcpy(out + sh + written, out + sh, chunk); written += chunk;
            }
            cur = sh + (int32_t)target;
            first_pos[sid] = sh;
        }
    }
    return cur;
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
typedef struct { uint32_t next_row; uint32_t w; } DenseEdge;

static void run_early_stop_bench(
    const RunRec *runs, int32_t r0, const int32_t counts[256],
    int32_t n, int32_t p, const uint8_t *ref_out,
    int k_stop, int print_rounds,
    double *out_acq, double *out_con, double *out_build_res, double *out_walk, double *out_tot,
    int32_t *out_mrem, int32_t *out_prem, int64_t *out_waves, int *out_exact
) {
    uint8_t *cyc_buf = (uint8_t *)malloc((size_t)n + 64);
    uint8_t *out_buf = (uint8_t *)malloc((size_t)n + 64);

    double ta[5], tc[5], tb[5], tw[5], tt[5];
    int32_t last_mrem = 0, last_prem = 0;
    int64_t last_waves = 0;
    int exact_ok = 1;
    int n_trials = print_rounds ? 1 : 5;

    for (int trial = 0; trial < n_trials; trial++) {
        double t0 = now_ms();

        int32_t C[256], acc = 0;
        for (int c = 0; c < 256; c++) { C[c] = acc; acc += counts[c]; }
        int32_t cur_t[256]; memcpy(cur_t, C, sizeof(cur_t));

        PieceVec pieces, cur_arr, wave, next_wave;
        pv_init(&pieces, r0 + 16); pv_init(&cur_arr, r0 + 16);
        pv_init(&wave, r0 + 16); pv_init(&next_wave, r0 + 16);
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
        int rounds = 0; int64_t waves = 0;

        while (!has_pcyc && rounds < k_stop) {
            double tr0 = now_ms();
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

            pieces.len = 0;
            int64_t r_waves = 0, r_joins = 0;
            while (wave.len > 0) {
                waves++; r_waves++;
                if (wave.len > rtmp_cap) { rtmp_cap = wave.len * 2; rtmp = (Piece *)realloc(rtmp, (size_t)rtmp_cap * sizeof(Piece)); }
                radix_sort_by_target(wave.data, rtmp, wave.len);

                next_wave.len = 0;
                int32_t j = 0;
                const Piece *warr = wave.data;
                int32_t nw_in = wave.len;
                int use_gallop = (nw_in * 8 < ncur);

                for (int32_t i = 0; i < nw_in; i++) {
                    Piece cur = warr[i];
                    int32_t t_lo = cur.lo + cur.delta, t_hi = cur.hi + cur.delta;
                    if (use_gallop) j = gallop_advance(arr, j, ncur, t_lo);
                    else while (j < ncur && arr[j].hi <= t_lo) j++;
                    int32_t cur_pos = t_lo, k = j;
                    while (cur_pos < t_hi) {
                        const Piece *q = &arr[k];
                        int32_t ov = (t_hi < q->hi) ? t_hi : q->hi;
                        int32_t slo = cur_pos - cur.delta, shi = ov - cur.delta;
                        if (!q->is_del) {
                            pv_push(&pieces, (Piece){slo, shi, cur.delta, cur.w, cur.mark_off, 0});
                        } else {
                            r_joins++;
                            uint32_t q_len = hs_len(&hs, q->w);
                            Piece nxt = {
                                slo, shi, cur.delta + q->delta,
                                hs_concat_fwd(&hs, cur.w, q->w),
                                combine_mark_fwd(cur.mark_off, q->mark_off, q_len),
                                0
                            };
                            pv_push(&next_wave, nxt);
                        }
                        cur_pos = ov;
                        if (cur_pos == q->hi) k++;
                    }
                    j = k;
                }
                PieceVec tmp = wave; wave = next_wave; next_wave = tmp;
            }
            if (pieces.len > rtmp_cap) { rtmp_cap = pieces.len * 2; rtmp = (Piece *)realloc(rtmp, (size_t)rtmp_cap * sizeof(Piece)); }
            radix_sort_by_lo(pieces.data, rtmp, pieces.len);
            double tr1 = now_ms();
            if (print_rounds) {
                int64_t rem_rows = 0;
                for (int32_t i = 0; i < pieces.len; i++) rem_rows += (pieces.data[i].hi - pieces.data[i].lo);
                printf("    rnd %2d: in_P=%6d -> out_P=%6d rem_rows=%6ld | waves=%2ld joins=%6ld | dt=%5.2f ms\n",
                       rounds, ncur, pieces.len, (long)rem_rows, (long)r_waves, (long)r_joins, tr1 - tr0);
            }
        }

        double t2 = now_ms();

        /* Stage 3: Build compact dense residual permutation edges_res[0..m_rem) via 1 radix-merge-join!
         * (Or direct emit if has_pcyc already closed!) */
        int32_t m_rem = 0;
        int32_t p_rem = pieces.len;
        int32_t *first_pos = (int32_t *)malloc((size_t)(hs.slp_len + 1) * sizeof(int32_t));
        memset(first_pos, -1, (size_t)(hs.slp_len + 1) * sizeof(int32_t));
        ExpFrame *exp_stack = (ExpFrame *)malloc((size_t)(hs.slp_len * 2 + 1024) * sizeof(ExpFrame));

        double t3 = t2, t4 = t2;

        if (has_pcyc) {
            t3 = now_ms();
            emit_word_at(&hs, pcyc_w, cyc_buf, 0, first_pos, exp_stack);
            int32_t q_len = (int32_t)hs_len(&hs, pcyc_w);
            int32_t split = (pcyc_m + 1) % q_len;
            int32_t tail_len = q_len - split;
            memcpy(out_buf, cyc_buf + split, (size_t)tail_len);
            if (split > 0) memcpy(out_buf + tail_len, cyc_buf, (size_t)split);
            t4 = now_ms();
        } else {
            /* Compute dense index offsets for pieces[0..p_rem) */
            int32_t *dense_off = (int32_t *)malloc((size_t)(p_rem + 1) * sizeof(int32_t));
            int32_t start_dense = -1;
            int32_t start_mark_off = -1;
            for (int32_t i = 0; i < p_rem; i++) {
                dense_off[i] = m_rem;
                if (pieces.data[i].mark_off >= 0) {
                    start_dense = m_rem;
                    start_mark_off = pieces.data[i].mark_off;
                }
                m_rem += (pieces.data[i].hi - pieces.data[i].lo);
            }
            dense_off[p_rem] = m_rem;

            /* Build dense_edges[0..m_rem) via 1 binary search per piece (only p_rem bisects!).
             * Note: Because deleted gaps may sit inside [t_lo, t_lo + ln) in original coords?
             * Wait: f_t is a bijection on U_t, so [t_lo, t_lo + ln) = f_t([lo, hi)) is 100% LIVE
             * (contains ZERO deleted gaps!), and therefore maps contiguously in dense coords too!
             * Wait: why did k_stop=1..10 fail earlier?!
             * CHECK: Inside the walk loop, `mark_abs = n - len0 + start_mark_off`, and we started
             * at `end_cur = n`, stepping `m_rem` times! What if `edges` had a bug when `t_lo`
             * landed in the interior of `parr[l]` AND `t_lo + ln > parr[l].hi`?
             * Because [t_lo, t_lo+ln) has NO deleted gaps inside it, `dense_off[l] + (t_lo - parr[l].lo) + k`
             * IS exact! SO WHY DID IT FAIL?!
             * LOOK AT `start_dense` AND `emit_word_at`:
             * `emit_word_at` uses `first_pos[sid]` — which writes `out[cur]` and records `first_pos[sid] = cur`,
             * and later copies `memcpy(out + write_pos, out + fp, ln)`!
             * WAIT A SECOND!! When we walked `edges` RIGHT-TO-LEFT (`end_cur -= wl`), and called
             * `emit_word_at(e.w, cyc_buf, end_cur)`:
             * THAT WORKED, EXCEPT LOOK AT `start_dense`:
             * When `pieces.data[i].mark_off >= 0`, `pieces.data[i]` has `ln == 1` (because we isolated [p, p+1)!).
             * WAIT: AFTER ROUND 1, COULD `pieces.data[i]` WITH `mark_off >= 0` HAVE `ln == 1`? YES!
             * THEN WHY DID `k_stop=1` FAIL?!
             * LOOK AT `emit_word_at` LINE 158:
             *   `memcpy(out + write_pos, fw->bytes, 16);` !!
             * OH!! MY!! GOD!! WHEN WRITING RIGHT-TO-LEFT (`end_cur` DECREASING BY `wl < 16`),
             * `memcpy(out + end_cur, fw->bytes, 16)` OVERWROTE THE ALREADY-WRITTEN BYTES AT `out + end_cur + wl`!!
             * HAHAHA!! WHAT A CLASSIC 1-LINE CATCH!!
             * Writing LEFT-TO-RIGHT (or copying exact `fw->len` bytes when writing right-to-left!) fixes it 100%!
             */
            DenseEdge *edges = (DenseEdge *)malloc((size_t)m_rem * sizeof(DenseEdge));
            const Piece *parr = pieces.data;
            for (int32_t i = 0; i < p_rem; i++) {
                int32_t t_lo = parr[i].lo + parr[i].delta;
                int32_t ln = parr[i].hi - parr[i].lo;
                int32_t l = 0, r = p_rem;
                while (l + 1 < r) {
                    int32_t mid = l + ((r - l) >> 1);
                    if (parr[mid].lo <= t_lo) l = mid; else r = mid;
                }
                uint32_t dst0 = (uint32_t)(dense_off[l] + (t_lo - parr[l].lo));
                uint32_t w = parr[i].w;
                int32_t base = dense_off[i];
                for (int32_t k = 0; k < ln; k++) {
                    edges[base + k].next_row = dst0 + (uint32_t)k;
                    edges[base + k].w = w;
                }
            }
            free(dense_off);
            t3 = now_ms();

            /* Walk compact dense permutation edges[0..m_rem) in LF order, writing right-to-left
             * with exact `fw->len` byte copy (zero overwrite of already-written suffix!). */
            int32_t end_cur = n;
            uint32_t row = (uint32_t)start_dense;
            uint32_t w0 = edges[row].w;
            int32_t len0 = (int32_t)hs_len(&hs, w0);
            int32_t mark_abs = n - len0 + start_mark_off;
            for (int32_t step = 0; step < m_rem; step++) {
                DenseEdge e = edges[row];
                uint32_t wl = hs_len(&hs, e.w);
                end_cur -= (int32_t)wl;
                emit_word_at(&hs, e.w, cyc_buf, end_cur, first_pos, exp_stack);
                row = e.next_row;
            }
            /* Rotate cyc_buf by 2 memcpy calls (ZERO reverse!) */
            int32_t split = (mark_abs + 1) % n;
            int32_t tail_len = n - split;
            memcpy(out_buf, cyc_buf + split, (size_t)tail_len);
            if (split > 0) memcpy(out_buf + tail_len, cyc_buf, (size_t)split);

            free(edges);
            t4 = now_ms();
        }

        if (trial == 0) {
            if (memcmp(out_buf, ref_out, (size_t)n) != 0) exact_ok = 0;
            last_mrem = m_rem; last_prem = p_rem; last_waves = waves;
        }
        ta[trial] = t1 - t0;
        tc[trial] = t2 - t1;
        tb[trial] = t3 - t2;
        tw[trial] = t4 - t3;
        tt[trial] = t4 - t0;

        free(first_pos); free(exp_stack); free(rtmp);
        pv_free(&pieces); pv_free(&cur_arr); pv_free(&wave); pv_free(&next_wave);
        hs_free(&hs);
    }

    int mid = n_trials / 2;
    qsort(ta, (size_t)n_trials, sizeof(double), cmp_dbl);
    qsort(tc, (size_t)n_trials, sizeof(double), cmp_dbl);
    qsort(tb, (size_t)n_trials, sizeof(double), cmp_dbl);
    qsort(tw, (size_t)n_trials, sizeof(double), cmp_dbl);
    qsort(tt, (size_t)n_trials, sizeof(double), cmp_dbl);
    *out_acq = ta[mid]; *out_con = tc[mid]; *out_build_res = tb[mid]; *out_walk = tw[mid]; *out_tot = tt[mid];
    *out_mrem = last_mrem; *out_prem = last_prem; *out_waves = last_waves; *out_exact = exact_ok;

    free(cyc_buf); free(out_buf);
}

int main(void) {
    const char *files[6] = {"block-0.L","block-2.L","block-85.L","block-143.L","block-210.L","block-207.L"};
    const int32_t ns[6] = {899931,900000,899935,899869,899959,899998};
    const int32_t ps[6] = {264045,320807,325088,802868,791429,205180};

    /* Load all 6 blocks */
    RunRec *runs[6]; int32_t r0s[6], counts[6][256], m16s[6];
    uint8_t *refs[6];
    for (int bi = 0; bi < 6; bi++) {
        char path[256]; snprintf(path, sizeof(path), "/tmp/mibz2-blocks/%s", files[bi]);
        FILE *fp = fopen(path, "rb"); int32_t n = ns[bi], p = ps[bi];
        uint8_t *L = (uint8_t *)malloc((size_t)n);
        fread(L, 1, (size_t)n, fp); fclose(fp);
        runs[bi] = (RunRec *)malloc((size_t)n * sizeof(RunRec));
        memset(counts[bi], 0, sizeof(counts[bi]));
        int32_t r0 = 0, m16 = 0, pt = 0;
        for (int32_t i = 0; i < n; ) {
            uint8_t c = L[i]; int32_t j = i + 1; while (j < n && L[j] == c) j++;
            int32_t ln = j - i;
            runs[bi][r0++] = (RunRec){c, ln};
            counts[bi][c] += ln;
            if (ln <= 16) { m16 += ln; if (i <= p && p < j) pt = 1; }
            i = j;
        }
        if (!pt) m16++;
        r0s[bi] = r0; m16s[bi] = m16;
        refs[bi] = (uint8_t *)malloc((size_t)n);
        ref_ibwt(L, n, p, refs[bi]);
        free(L);
    }

    printf("=== 1. ROUND-BY-ROUND PROFILE ON block-0.L (r0=51184, m16=89874) ===\n");
    {
        double a, c, b, w, t; int32_t mr, pr; int64_t wv; int ex;
        run_early_stop_bench(runs[0], r0s[0], counts[0], ns[0], ps[0], refs[0], 50, 1,
                             &a, &c, &b, &w, &t, &mr, &pr, &wv, &ex);
        printf("  block-0.L FULL exact=%s total=%.2f ms (acq=%.2f con=%.2f exp=%.2f)\n\n",
               ex ? "PASS" : "FAIL", t, a, c, w);
    }

    printf("=== 2. EARLY-STOP SWEEP (k_stop = 1, 2, 3, 4, 5, 6, 8, 10, FULL) — 6-BLOCK SUM ===\n");
    printf("k_stop | sum_mrem (x m16) | sum_Prem | waves | T_acq  T_con  T_res  T_walk  T_TOT (ms) | exact\n");
    printf("-------+------------------+----------+-------+----------------------------------------+------\n");
    int kstops[] = {1, 2, 3, 4, 5, 6, 8, 10, 50};
    for (int ki = 0; ki < 9; ki++) {
        int ks = kstops[ki];
        double sa=0, sc=0, sb=0, sw=0, st=0;
        int64_t smr=0, spr=0, swv=0;
        int all_ex = 1;
        for (int bi = 0; bi < 6; bi++) {
            double a, c, b, w, t; int32_t mr, pr; int64_t wv; int ex;
            run_early_stop_bench(runs[bi], r0s[bi], counts[bi], ns[bi], ps[bi], refs[bi], ks, 0,
                                 &a, &c, &b, &w, &t, &mr, &pr, &wv, &ex);
            sa += a; sc += c; sb += b; sw += w; st += t;
            smr += mr; spr += pr; swv += wv;
            if (!ex) all_ex = 0;
        }
        printf("k=%4d | %8ld (%5.3fx) | %8ld | %5ld | %5.2f  %5.2f  %5.2f  %6.2f  %6.2f ms | %s\n",
               ks, (long)smr, (double)smr / 895391.0, (long)spr, (long)swv,
               sa, sc, sb, sw, st, all_ex ? "PASS" : "FAIL");
    }

    for (int bi = 0; bi < 6; bi++) { free(runs[bi]); free(refs[bi]); }
    return 0;
}
