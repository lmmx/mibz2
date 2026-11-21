#include <stdint.h>
#include <stdlib.h>
#include <string.h>

/*
 * Exact O(n log n) cyclic Burrows-Wheeler Transform via two-pass radix prefix doubling.
 * Breaks ties on identical full-period rotations by original start index `i` (matching
 * Python's `sorted((T[i:] + T[:i], i) for i in range(n))` byte-for-byte!).
 */
int cyclic_bwt_u8(const uint8_t *T, int32_t n, uint8_t *L_out, int32_t *p_out) {
    if (n <= 0) return -1;
    if (n == 1) {
        L_out[0] = T[0];
        *p_out = 0;
        return 0;
    }

    int32_t max_c = (n > 256) ? n : 256;
    int32_t *sa = (int32_t *)malloc((size_t)n * sizeof(int32_t));
    int32_t *sa_tmp = (int32_t *)malloc((size_t)n * sizeof(int32_t));
    int32_t *rk = (int32_t *)malloc((size_t)n * sizeof(int32_t));
    int32_t *rk_tmp = (int32_t *)malloc((size_t)n * sizeof(int32_t));
    int32_t *cnt = (int32_t *)malloc((size_t)(max_c + 1) * sizeof(int32_t));
    if (!sa || !sa_tmp || !rk || !rk_tmp || !cnt) {
        free(sa); free(sa_tmp); free(rk); free(rk_tmp); free(cnt);
        return -2;
    }

    memset(cnt, 0, 256 * sizeof(int32_t));
    for (int32_t i = 0; i < n; ++i) {
        rk[i] = T[i];
        cnt[T[i]]++;
    }
    for (int32_t c = 1; c < 256; ++c) cnt[c] += cnt[c - 1];
    /* Stable sort: iterate backwards so smaller `i` comes first on ties */
    for (int32_t i = n - 1; i >= 0; --i) {
        sa[--cnt[T[i]]] = i;
    }

    int32_t classes = 1;
    rk_tmp[sa[0]] = 0;
    for (int32_t idx = 1; idx < n; ++idx) {
        if (T[sa[idx]] != T[sa[idx - 1]]) classes++;
        rk_tmp[sa[idx]] = classes - 1;
    }
    memcpy(rk, rk_tmp, (size_t)n * sizeof(int32_t));

    for (int32_t step = 1; step < n && classes < n; step <<= 1) {
        /* Pass 1: stable sort by second key rk[(i + step) % n], breaking ties by `i` */
        memset(cnt, 0, (size_t)classes * sizeof(int32_t));
        for (int32_t i = 0; i < n; ++i) {
            int32_t k2 = rk[(i + step) % n];
            cnt[k2]++;
        }
        for (int32_t c = 1; c < classes; ++c) cnt[c] += cnt[c - 1];
        /* Iterate i from n-1 down to 0 so ties on k2 preserve ascending `i`! */
        for (int32_t i = n - 1; i >= 0; --i) {
            int32_t k2 = rk[(i + step) % n];
            sa_tmp[--cnt[k2]] = i;
        }

        /* Pass 2: stable sort `sa_tmp` by first key rk[u] */
        memset(cnt, 0, (size_t)classes * sizeof(int32_t));
        for (int32_t idx = 0; idx < n; ++idx) {
            cnt[rk[sa_tmp[idx]]]++;
        }
        for (int32_t c = 1; c < classes; ++c) cnt[c] += cnt[c - 1];
        for (int32_t idx = n - 1; idx >= 0; --idx) {
            int32_t u = sa_tmp[idx];
            sa[--cnt[rk[u]]] = u;
        }

        classes = 1;
        rk_tmp[sa[0]] = 0;
        for (int32_t idx = 1; idx < n; ++idx) {
            int32_t u = sa[idx], v = sa[idx - 1];
            if (rk[u] != rk[v] || rk[(u + step) % n] != rk[(v + step) % n]) {
                classes++;
            }
            rk_tmp[u] = classes - 1;
        }
        memcpy(rk, rk_tmp, (size_t)n * sizeof(int32_t));
    }

    int32_t p = 0;
    for (int32_t idx = 0; idx < n; ++idx) {
        int32_t start = sa[idx];
        if (start == 0) p = idx;
        L_out[idx] = T[(start == 0) ? (n - 1) : (start - 1)];
    }
    *p_out = p;

    free(sa); free(sa_tmp); free(rk); free(rk_tmp); free(cnt);
    return 0;
}
