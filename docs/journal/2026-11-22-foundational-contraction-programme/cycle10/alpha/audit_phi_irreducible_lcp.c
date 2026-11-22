/*
 * Cycle 10 — Task 2:
 * Exact Phi-Function & Irreducible-LCP Audit on ALL 6 Authenticated Wikidata Blocks!
 *
 * Computes directly from each block's exact inverse text T (verified against L & p):
 *   1. Exact Suffix Array SA[0..n) and Inverse Suffix Array ISA[0..n) (matching the BWT rows!),
 *   2. Exact Kasai O(n) LCP[0..n) (circular LCP between row i and row i-1),
 *   3. Exact Phi-function Phi(SA[i]) = SA[i-1] and exact # of Phi-runs (PLCP breaks),
 *   4. Exact Irreducible LCPs (rows i where L[i] != L[i-1], i.e., run heads of L!):
 *        - count (= r_0), sum(irr_LCP) / n (compare to KKP 2 n log n bound!),
 *        - mean / median / p95 / max irr_LCP stratified by:
 *            * thick runs (w > 16),
 *            * Open thin runs (w <= 16),
 *            * AP4_max critical strips (out_deg >= 2)!
 */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static int cmp_i32(const void *a, const void *b) {
    int32_t x = *(const int32_t *)a, y = *(const int32_t *)b;
    return (x > y) - (x < y);
}

int main(void) {
    const char *files[6] = {"block-0.L","block-2.L","block-85.L","block-143.L","block-210.L","block-207.L"};
    const int32_t ns[6] = {899931,900000,899935,899869,899959,899998};
    const int32_t ps[6] = {264045,320807,325088,802868,791429,205180};

    printf("=== EXACT PHI-FUNCTION & IRREDUCIBLE-LCP AUDIT ON ALL 6 AUTHENTICATED WIKIDATA BLOCKS ===\n");
    printf("block       |   r0  | Phi_brk | sum_irrLCP (/n) | mean_irr (thick / thin / AP4_crit) | p95_irr | max_irr | sum_allLCP(/n)\n");
    printf("------------+-------+---------+-----------------+------------------------------------+---------+---------+---------------\n");

    int64_t sum_r0 = 0, sum_pb = 0, sum_ilcp = 0, sum_alcp = 0, sum_n = 0;

    for (int bi = 0; bi < 6; bi++) {
        char path[256]; snprintf(path, sizeof(path), "/tmp/mibz2-blocks/%s", files[bi]);
        FILE *fp = fopen(path, "rb"); int32_t n = ns[bi], p = ps[bi];
        uint8_t *L = (uint8_t *)malloc((size_t)n);
        fread(L, 1, (size_t)n, fp); fclose(fp);

        /* Build exact LF, SA, ISA, and T in O(n)!
         * Note: Walking LF from p visits rows x_k = ISA[(n - 1 - k) mod n] where k = 0..n-1:
         * Specifically: p is row of rotation starting at T[0], so SA[p] = 0!
         * LF(p) is row of rotation starting at T[n-1], so SA[LF(p)] = n - 1, etc.! */
        int32_t C[256] = {0};
        for (int32_t i = 0; i < n; i++) C[L[i]]++;
        int32_t acc = 0;
        for (int c = 0; c < 256; c++) { int32_t cnt = C[c]; C[c] = acc; acc += cnt; }
        int32_t *LF = (int32_t *)malloc((size_t)n * sizeof(int32_t));
        int32_t cur[256]; memcpy(cur, C, sizeof(cur));
        for (int32_t i = 0; i < n; i++) LF[i] = cur[L[i]]++;

        int32_t *SA = (int32_t *)malloc((size_t)n * sizeof(int32_t));
        int32_t *ISA = (int32_t *)malloc((size_t)n * sizeof(int32_t));
        uint8_t *T = (uint8_t *)malloc((size_t)n);
        int32_t x = p;
        for (int32_t step = 0; step < n; step++) {
            int32_t pos = (n - step) % n;
            SA[x] = pos;
            ISA[pos] = x;
            T[(pos - 1 + n) % n] = L[x];
            x = LF[x];
        }

        /* Compute exact circular LCP[i] = lcp(T[SA[i]..], T[SA[i-1]..]) via Kasai O(n)! */
        int32_t *LCP = (int32_t *)malloc((size_t)n * sizeof(int32_t));
        int32_t h = 0;
        for (int32_t j = 0; j < n; j++) {
            int32_t r = ISA[j];
            if (r == 0) { LCP[0] = 0; h = 0; continue; }
            int32_t k = SA[r - 1];
            while (h < n && T[(j + h) % n] == T[(k + h) % n]) h++;
            LCP[r] = h;
            if (h > 0) h--;
        }

        /* Compute Phi(j) = SA[ISA[j] - 1] (cyclic wrap for r=0) and count Phi-breaks:
         * j is a Phi-break iff Phi(j) != (Phi(j-1) + 1) mod n! */
        int32_t *Phi = (int32_t *)malloc((size_t)n * sizeof(int32_t));
        for (int32_t r = 0; r < n; r++) {
            int32_t prev_r = (r == 0) ? (n - 1) : (r - 1);
            Phi[SA[r]] = SA[prev_r];
        }
        int32_t phi_breaks = 0;
        for (int32_t j = 0; j < n; j++) {
            int32_t prev_j = (j == 0) ? (n - 1) : (j - 1);
            if (Phi[j] != (Phi[prev_j] + 1) % n) phi_breaks++;
        }

        /* Identify run heads, run widths, and out_deg >= 2 (AP4 critical strips)! */
        int32_t *run_lo = (int32_t *)malloc((size_t)n * sizeof(int32_t));
        int32_t *run_hi = (int32_t *)malloc((size_t)n * sizeof(int32_t));
        int32_t *run_of = (int32_t *)malloc((size_t)n * sizeof(int32_t));
        int32_t r0 = 0;
        for (int32_t i = 0; i < n; ) {
            uint8_t c = L[i]; int32_t j = i + 1;
            while (j < n && L[j] == c) j++;
            run_lo[r0] = i; run_hi[r0] = j;
            for (int32_t t = i; t < j; t++) run_of[t] = r0;
            r0++; i = j;
        }
        /* For each run u: check if out_deg(u) >= 2 (LF(run_lo[u]) and LF(run_hi[u]-1) lie in different runs!) */
        uint8_t *is_ap4_crit = (uint8_t *)calloc((size_t)r0, 1);
        for (int32_t u = 0; u < r0; u++) {
            int32_t w = run_hi[u] - run_lo[u];
            if (w <= 16) {
                int32_t v0 = run_of[LF[run_lo[u]]];
                int32_t v1 = run_of[LF[run_hi[u] - 1]];
                if (v0 != v1) is_ap4_crit[u] = 1;
            }
        }

        int32_t *irr_vals = (int32_t *)malloc((size_t)r0 * sizeof(int32_t));
        int32_t irr_cnt = 0;
        int64_t s_irr = 0, s_all = 0;
        int64_t s_th = 0, c_th = 0, s_tn = 0, c_tn = 0, s_cr = 0, c_cr = 0;
        for (int32_t i = 0; i < n; i++) s_all += LCP[i];
        for (int32_t u = 0; u < r0; u++) {
            int32_t i = run_lo[u];
            int32_t w = run_hi[u] - run_lo[u];
            int32_t lv = LCP[i];
            irr_vals[irr_cnt++] = lv;
            s_irr += lv;
            if (w > 16) { s_th += lv; c_th++; }
            else {
                s_tn += lv; c_tn++;
                if (is_ap4_crit[u]) { s_cr += lv; c_cr++; }
            }
        }
        qsort(irr_vals, (size_t)irr_cnt, sizeof(int32_t), cmp_i32);
        int32_t p95 = irr_vals[(int32_t)(0.95 * (irr_cnt - 1))];
        int32_t mx = irr_vals[irr_cnt - 1];

        printf("%-11s | %5d | %7d | %8ld (%4.2fn) |  %5.1f  (%5.1f / %5.1f / %5.1f)    | %7d | %7d | %6.1fn\n",
               files[bi], r0, phi_breaks, (long)s_irr, (double)s_irr / n,
               (double)s_irr / irr_cnt,
               (double)s_th / (c_th ? c_th : 1),
               (double)s_tn / (c_tn ? c_tn : 1),
               (double)s_cr / (c_cr ? c_cr : 1),
               p95, mx, (double)s_all / n);

        sum_r0 += r0; sum_pb += phi_breaks; sum_ilcp += s_irr; sum_alcp += s_all; sum_n += n;
        free(L); free(LF); free(SA); free(ISA); free(T); free(LCP); free(Phi);
        free(run_lo); free(run_hi); free(run_of); free(is_ap4_crit); free(irr_vals);
    }
    printf("------------+-------+---------+-----------------+------------------------------------+---------+---------+---------------\n");
    printf("SUM/AVG     | %5ld | %7ld | %8ld (%4.2fn) |  %5.1f                             |         |         | %6.1fn\n",
           (long)sum_r0, (long)sum_pb, (long)sum_ilcp, (double)sum_ilcp / sum_n,
           (double)sum_ilcp / sum_r0, (double)sum_alcp / sum_n);
    return 0;
}
