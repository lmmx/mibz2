/* Calibration probe: Neon's C cache model, unchanged, plus an i16 arm (Open QUEUE=false:
   direct wiring by a second run pass with scattered edge writes), to compare the model's
   qi16-vs-i16 ranking with the known native ranking (PR #653: i16 total 1.10-1.16x qi16). */
#define main orig_main
#include "g5_backward_open.c"
#undef main

static uint8_t *run_forward_i16(const uint16_t *recs, size_t nrecs, uint32_t n, uint32_t p, Metrics *met) {
    const uint32_t LIMIT = 16;
    CacheSim sim; sim_init(&sim);
    memset(met, 0, sizeof(*met));
    uint64_t rec_addr = sim_alloc_region(&sim, nrecs * sizeof(uint16_t));
    uint32_t all[256] = {0}, thin[256] = {0}, thick[256] = {0};
    OpenStrip *strips = (OpenStrip *)malloc((nrecs + 2) * sizeof(OpenStrip));
    uint64_t strips_addr = sim_alloc_region(&sim, (nrecs + 2) * sizeof(OpenStrip));
    size_t nstrips = 0;
    uint32_t m = 0, mark = 0, pos = 0;
    size_t ri = 0;
    /* pass 1: classify, no queues */
    while (ri < nrecs) {
        sim_access(&sim, rec_addr + ri * 2, 2, 0);
        uint16_t rec = recs[ri++]; uint8_t c = (uint8_t)(rec >> 8); uint32_t w = rec & 255;
        while (ri < nrecs) { sim_access(&sim, rec_addr + ri * 2, 2, 0); if ((recs[ri] >> 8) != c) break; w += recs[ri] & 255; ri++; }
        met->work_events++;
        uint32_t end = pos + w; uint32_t sx[3], sw[3]; int sk[3], ns = 0;
        if (w > LIMIT && pos <= p && p < end) {
            if (pos < p) { sx[ns] = pos; sw[ns] = p - pos; sk[ns++] = 0; }
            sx[ns] = p; sw[ns] = 1; sk[ns++] = 1;
            if (p + 1 < end) { sx[ns] = p + 1; sw[ns] = end - p - 1; sk[ns++] = 0; }
        } else { sx[ns] = pos; sw[ns] = w; sk[ns++] = (w <= LIMIT); }
        for (int s = 0; s < ns; s++) {
            if (sk[s]) { if (sx[s] <= p && p < sx[s] + sw[s]) mark = m + (p - sx[s]); thin[c] += sw[s]; m += sw[s]; }
            else { strips[nstrips] = (OpenStrip){sx[s], all[c], sw[s], c}; sim_access(&sim, strips_addr + nstrips * sizeof(OpenStrip), sizeof(OpenStrip), 1); nstrips++; thick[c]++; }
            all[c] += sw[s];
        }
        pos = end;
    }
    OpenBand *bands; uint64_t bands_addr; size_t nbands;
    uint8_t *spelling; uint64_t spell_addr; size_t nspell;
    uint32_t *offsets; uint64_t off_addr; size_t noff;
    size_t cursors[256], ep_mem = 0;
    run_endpoint_and_spell(&sim, met, strips, strips_addr, nstrips, all, thin, thick, n, m,
                           &bands, &bands_addr, &nbands, &spelling, &spell_addr, &nspell,
                           &offsets, &off_addr, &noff, cursors, &ep_mem);
    uint32_t bits = 32 - (uint32_t)__builtin_clz(m > 1 ? m - 1 : 1);
    uint32_t *edges = (uint32_t *)malloc((size_t)m * 4);
    uint64_t edges_addr = sim_alloc_region(&sim, (size_t)m * 4);
    /* stream[a] = start rank of symbol a's thin rows in the per-symbol order (k coordinate) */
    uint32_t stream[256], acc = 0;
    for (int a = 0; a < 256; a++) { stream[a] = acc; acc += thin[a]; }
    size_t cur[256]; for (int a = 0; a < 256; a++) cur[a] = cursors[a];
    /* pass 2: re-read records, thin segments in domain order, scattered writes edges[dest] = pack|j */
    pos = 0; ri = 0; uint32_t j = 0;
    while (ri < nrecs) {
        sim_access(&sim, rec_addr + ri * 2, 2, 0);
        uint16_t rec = recs[ri++]; uint8_t c = (uint8_t)(rec >> 8); uint32_t w = rec & 255;
        while (ri < nrecs) { sim_access(&sim, rec_addr + ri * 2, 2, 0); if ((recs[ri] >> 8) != c) break; w += recs[ri] & 255; ri++; }
        met->work_events++;
        uint32_t end = pos + w; uint32_t sw[3]; int sk[3], ns = 0;
        if (w > LIMIT && pos <= p && p < end) {
            if (pos < p) { sw[ns] = p - pos; sk[ns++] = 0; }
            sw[ns] = 1; sk[ns++] = 1;
            if (p + 1 < end) { sw[ns] = end - p - 1; sk[ns++] = 0; }
        } else { sw[ns] = w; sk[ns++] = (w <= LIMIT); }
        for (int s = 0; s < ns; s++) {
            if (!sk[s]) continue;
            uint32_t z = stream[c], zend = z + sw[s];
            while (z < zend) {
                sim_access(&sim, bands_addr + cur[c] * sizeof(OpenBand), sizeof(OpenBand), 0);
                OpenBand b = bands[cur[c]];
                uint32_t e2 = zend < b.end ? zend : b.end, span = e2 - z, dest = z + b.delta, pack = b.code << bits;
                for (uint32_t t = 0; t < span; t++) edges[dest + t] = pack | (j + t);
                sim_access(&sim, edges_addr + (uint64_t)dest * 4, (size_t)span * 4, 1);
                met->work_events += span; met->irreg_q++;
                j += span; z = e2; if (z == b.end) cur[c]++;
            }
            stream[c] = zend;
        }
        pos = end;
    }
    met->peak_mem = nrecs * 2 + ep_mem + (size_t)m * 4 + n;
    uint8_t *out = (uint8_t *)malloc(n);
    uint64_t out_addr = sim_alloc_region(&sim, n);
    size_t out_len = 0; uint32_t mask = (1u << bits) - 1; uint32_t q = mark;
    for (uint32_t step = 0; step < m; step++) {
        sim_access(&sim, edges_addr + (uint64_t)q * 4, 4, 0);
        uint32_t edge = edges[q], code = edge >> bits; q = edge & mask;
        met->work_events++; met->irreg_q++;
        if (code < 256) { out[out_len] = (uint8_t)code; sim_access(&sim, out_addr + out_len, 1, 1); out_len++; }
        else { uint32_t wi = code - 256; sim_access(&sim, off_addr + (uint64_t)wi * 4, 8, 0);
            uint32_t s0 = offsets[wi], ln = offsets[wi + 1] - s0; sim_access(&sim, spell_addr + s0, ln, 0);
            uint32_t take = (out_len + ln <= n) ? ln : (n - (uint32_t)out_len);
            memcpy(out + out_len, spelling + s0, take); sim_access(&sim, out_addr + out_len, take, 1); out_len += take; met->irreg_q++; }
        if (q == mark || out_len >= n) break;
    }
    assert(out_len == n);
    sim_drain(&sim);
    met->l1_misses = sim.l1.misses; met->l2_misses = sim.l2.misses; met->llc_misses = sim.llc.misses;
    met->dram_reads = sim.dram_reads; met->dram_writes = sim.dram_writes;
    met->dram_bb = 64.0 * (double)(sim.dram_reads + sim.dram_writes) / (double)n;
    free(strips); free(bands); free(spelling); free(offsets); free(edges); sim_free(&sim);
    return out;
}

int main(int argc, char **argv) {
    const BlockSpec blocks[6] = {
        {"block-0.L",899931,264045,51184},{"block-2.L",900000,320807,54532},{"block-85.L",899935,325088,79375},
        {"block-143.L",899869,802868,108335},{"block-207.L",899998,205180,87111},{"block-210.L",899959,791429,128732}};
    printf("model ratios i16/qi16 (native: total 1.10-1.16x, i.e. i16 SLOWER) and Bwd/qi16 for reference\n");
    printf("%-11s | i16/qi16: L1m  L2m  DRAM  work  mem | Bwd/qi16: L1m  L2m  DRAM  work\n", "block");
    for (int i = 0; i < 6; i++) {
        char path[512]; snprintf(path, sizeof path, "%s/%s", argv[1], blocks[i].file);
        FILE *f = fopen(path, "rb"); uint8_t *L = malloc(blocks[i].n); assert(fread(L,1,blocks[i].n,f)==blocks[i].n); fclose(f);
        size_t nr = 0; uint16_t *recs = build_records(L, blocks[i].n, &nr);
        Metrics mq, mi, mb;
        uint8_t *oq = run_forward_open(recs, nr, blocks[i].n, blocks[i].p, &mq);
        uint8_t *oi = run_forward_i16(recs, nr, blocks[i].n, blocks[i].p, &mi);
        uint8_t *ob = run_backward_open(recs, nr, blocks[i].n, blocks[i].p, &mb);
        assert(memcmp(oq, oi, blocks[i].n) == 0 && memcmp(oq, ob, blocks[i].n) == 0);
        printf("%-11s |          %4.2f %4.2f %4.2f  %4.2f %4.2f |          %4.2f %4.2f %4.2f  %4.2f\n", blocks[i].file,
            (double)mi.l1_misses/mq.l1_misses, (double)mi.l2_misses/mq.l2_misses, mi.dram_bb/mq.dram_bb,
            (double)mi.work_events/mq.work_events, (double)mi.peak_mem/mq.peak_mem,
            (double)mb.l1_misses/mq.l1_misses, (double)mb.l2_misses/mq.l2_misses, mb.dram_bb/mq.dram_bb,
            (double)mb.work_events/mq.work_events);
        free(L); free(recs); free(oq); free(oi); free(ob);
    }
    return 0;
}
