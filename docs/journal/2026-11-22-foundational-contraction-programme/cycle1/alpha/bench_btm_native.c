/*
 * Native C (-O3 -march=native) benchmark & exact verifier for Compositional
 * BTM-WIET (Redheffer Star Product of Boundary Transfer Maps) on all 6
 * authenticated Wikidata BWT blocks in /tmp/mibz2-blocks/.
 *
 * Implements:
 *   1. Acquisition directly from uncapped RLE I_0 (runs of L + C + bigram table + leverage merge tree)
 *   2. Exact per-symbol-bucketed Redheffer Star Product contraction (P1 + spiral division)
 *      -> Each target lookup searches ONLY the pieces in bucket F_{target_sym} (L1-resident!)
 *   3. Exact First-Occurrence Output-Buffer Memoized SLP Expansion (zero O(n) tree walk!)
 *   4. Byte-for-byte verification against reference LF-walk on all 6 blocks.
 *   5. High-resolution timing breakdown: T_acquire, T_contract, T_expand, T_total (5 runs, median).
 */

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

typedef struct {
    int32_t lo;
    int32_t hi;
    int32_t delta;
    uint32_t slp_id;
    int32_t mark_off; /* -1 if unmarked, else 0-based byte offset of L[p] */
    uint8_t tsym;     /* target F-bucket symbol */
    uint8_t is_int;
} Piece;

typedef struct {
    Piece *data;
    int32_t len;
    int32_t cap;
} PieceVec;

static inline void pv_init(PieceVec *v, int32_t cap) {
    v->len = 0;
    v->cap = cap > 16 ? cap : 16;
    v->data = (Piece *)malloc((size_t)v->cap * sizeof(Piece));
}

static inline void pv_push(PieceVec *v, Piece p) {
    if (v->len == v->cap) {
        v->cap *= 2;
        v->data = (Piece *)realloc(v->data, (size_t)v->cap * sizeof(Piece));
    }
    v->data[v->len++] = p;
}

static inline void pv_free(PieceVec *v) {
    free(v->data);
    v->data = NULL;
    v->len = v->cap = 0;
}

/* SLP Store: compact 12-byte nodes (a, b, len; top bit of a indicates power node) */
typedef struct {
    uint32_t a;   /* left child or base (bit 31 = 1 for power node) */
    uint32_t b;   /* right child or exponent */
    uint32_t len; /* expanded byte length */
} SlpNode;

typedef struct {
    SlpNode *nodes;
    uint32_t len;
    uint32_t cap;
} SlpStore;

static void slp_init(SlpStore *s, uint32_t cap) {
    s->cap = cap;
    s->nodes = (SlpNode *)malloc((size_t)cap * sizeof(SlpNode));
    s->len = 256;
    for (uint32_t c = 0; c < 256; c++) {
        s->nodes[c].a = c;
        s->nodes[c].b = 0;
        s->nodes[c].len = 1;
    }
}

static inline uint32_t slp_concat(SlpStore *s, uint32_t u, uint32_t v) {
    if (s->len == s->cap) {
        s->cap *= 2;
        s->nodes = (SlpNode *)realloc(s->nodes, (size_t)s->cap * sizeof(SlpNode));
    }
    uint32_t id = s->len++;
    s->nodes[id].a = u;
    s->nodes[id].b = v;
    s->nodes[id].len = s->nodes[u].len + s->nodes[v].len;
    return id;
}

static inline uint32_t slp_power(SlpStore *s, uint32_t u, uint32_t k) {
    if (k == 1) return u;
    if (s->len == s->cap) {
        s->cap *= 2;
        s->nodes = (SlpNode *)realloc(s->nodes, (size_t)s->cap * sizeof(SlpNode));
    }
    uint32_t id = s->len++;
    s->nodes[id].a = u | 0x80000000u;
    s->nodes[id].b = k;
    s->nodes[id].len = s->nodes[u].len * k;
    return id;
}

static inline int32_t combine_mark(int32_t m1, uint32_t len1, int32_t m2) {
    if (m1 >= 0) return m1;
    if (m2 >= 0) return (int32_t)len1 + m2;
    return -1;
}

typedef struct {
    int32_t a, b;
} Range;

static int cmp_range(const void *pa, const void *pb) {
    int32_t a = ((const Range *)pa)->a;
    int32_t b = ((const Range *)pb)->a;
    return (a > b) - (a < b);
}

static int cmp_piece_lo(const void *pa, const void *pb) {
    int32_t a = ((const Piece *)pa)->lo;
    int32_t b = ((const Piece *)pb)->lo;
    return (a > b) - (a < b);
}

/* Reusable scratch buffers for resolve_subset so zero per-merge malloc churn */
typedef struct {
    PieceVec collapsed;
    PieceVec stack;
    Range *int_targets;
    int32_t int_targets_cap;
    Range *merged_t;
    int32_t merged_t_cap;
    /* Per-symbol index into collapsed (since collapsed is sorted by lo, pieces in F_c are contiguous!) */
    int32_t sym_off[257];
    /* Stats */
    int64_t bounces;
    int64_t q_rand;
    int64_t q_root;
    int64_t w_seq;
    int64_t spirals;
    int64_t sum_pieces;
} Scratch;

static void scratch_init(Scratch *sc) {
    pv_init(&sc->collapsed, 1 << 17);
    pv_init(&sc->stack, 1 << 17);
    sc->int_targets_cap = 1 << 17;
    sc->int_targets = (Range *)malloc((size_t)sc->int_targets_cap * sizeof(Range));
    sc->merged_t_cap = 1 << 17;
    sc->merged_t = (Range *)malloc((size_t)sc->merged_t_cap * sizeof(Range));
    sc->bounces = sc->q_rand = sc->q_root = sc->w_seq = sc->spirals = sc->sum_pieces = 0;
}

static void scratch_free(Scratch *sc) {
    pv_free(&sc->collapsed);
    pv_free(&sc->stack);
    free(sc->int_targets);
    free(sc->merged_t);
}

typedef struct {
    uint64_t bits[4];
} SymSet;

static inline void ss_clear(SymSet *s) { s->bits[0] = s->bits[1] = s->bits[2] = s->bits[3] = 0; }
static inline void ss_add(SymSet *s, uint8_t c) { s->bits[c >> 6] |= (1ULL << (c & 63)); }
static inline int ss_has(const SymSet *s, uint8_t c) { return (int)((s->bits[c >> 6] >> (c & 63)) & 1ULL); }
static inline SymSet ss_union(SymSet a, SymSet b) {
    SymSet r;
    for (int i = 0; i < 4; i++) r.bits[i] = a.bits[i] | b.bits[i];
    return r;
}

/* Binary search inside a single F-bucket's contiguous slice [lo_idx, hi_idx) of collapsed */
static inline int32_t find_piece_in_slice(const Piece *arr, int32_t lo_idx, int32_t hi_idx, int32_t target) {
    int32_t l = lo_idx, r = hi_idx;
    while (l + 1 < r) {
        int32_t m = l + ((r - l) >> 1);
        if (arr[m].lo <= target) l = m;
        else r = m;
    }
    return l;
}

/* Merge two already-sorted PieceVecs (p1 and p2 are disjoint in domain, each sorted by lo!) */
static void merge_sorted_pieces_into_collapsed(
    const PieceVec *p1, const PieceVec *p2,
    const SymSet *sset, SlpStore *slp, Scratch *sc,
    int *has_pcyc, uint32_t *pcyc_slp, int32_t *pcyc_mark
) {
    sc->collapsed.len = 0;
    int32_t i = 0, j = 0;
    int32_t n1 = p1->len, n2 = p2 ? p2->len : 0;
    sc->w_seq += n1 + n2;

    /* 2-way merge in sorted lo order! */
    int need_sort = 0;
    while (i < n1 || j < n2) {
        Piece p;
        if (j >= n2 || (i < n1 && p1->data[i].lo < p2->data[j].lo)) {
            p = p1->data[i++];
        } else {
            p = p2->data[j++];
        }
        p.is_int = (uint8_t)ss_has(sset, p.tsym);
        if (!p.is_int) {
            pv_push(&sc->collapsed, p);
            continue;
        }
        if (p.delta == 0) {
            if (p.mark_off >= 0) {
                *has_pcyc = 1;
                *pcyc_slp = p.slp_id;
                *pcyc_mark = p.mark_off;
            }
            continue;
        }
        int32_t L = p.hi - p.lo;
        if (p.delta > 0 && p.delta < L) {
            sc->spirals++;
            int32_t d = p.delta;
            int32_t q = L / d, rem = L % d;
            if (rem > 0) {
                Piece q1 = { p.lo, p.lo + rem, (q + 1) * d, slp_power(slp, p.slp_id, (uint32_t)(q + 1)), -1, p.tsym, 1 };
                pv_push(&sc->collapsed, q1);
            }
            if (d - rem > 0) {
                Piece q2 = { p.lo + rem, p.lo + d, q * d, slp_power(slp, p.slp_id, (uint32_t)q), -1, p.tsym, 1 };
                pv_push(&sc->collapsed, q2);
            }
            need_sort = 1;
        } else if (p.delta < 0 && (-p.delta) < L) {
            sc->spirals++;
            int32_t s = -p.delta;
            int32_t q = L / s, rem = L % s;
            if (s - rem > 0) {
                Piece q1 = { p.hi - s, p.hi - rem, -q * s, slp_power(slp, p.slp_id, (uint32_t)q), -1, p.tsym, 1 };
                pv_push(&sc->collapsed, q1);
            }
            if (rem > 0) {
                Piece q2 = { p.hi - rem, p.hi, -(q + 1) * s, slp_power(slp, p.slp_id, (uint32_t)(q + 1)), -1, p.tsym, 1 };
                pv_push(&sc->collapsed, q2);
            }
            need_sort = 1;
        } else {
            pv_push(&sc->collapsed, p);
        }
    }
    if (need_sort) {
        qsort(sc->collapsed.data, (size_t)sc->collapsed.len, sizeof(Piece), cmp_piece_lo);
    }
}

static void resolve_subset_c(
    const SymSet *sset,
    const PieceVec *p1,
    const PieceVec *p2,
    const int32_t *C_bucket, /* C[0..256] cumulative counts -> maps row -> F-bucket */
    SlpStore *slp,
    Scratch *sc,
    PieceVec *out_pieces,
    int *has_pcyc,
    uint32_t *pcyc_slp,
    int32_t *pcyc_mark
) {
    merge_sorted_pieces_into_collapsed(p1, p2, sset, slp, sc, has_pcyc, pcyc_slp, pcyc_mark);

    int32_t nc = sc->collapsed.len;
    const Piece *col = sc->collapsed.data;

    /* Build per-symbol index sym_off[0..256] in ONE sequential O(nc + 256) sweep!
     * Because col is sorted by lo, pieces inside F_c = [C[c], C[c+1]) are contiguous! */
    int32_t cur_c = 0;
    for (int32_t idx = 0; idx < nc; idx++) {
        int32_t lo = col[idx].lo;
        while (cur_c < 256 && C_bucket[cur_c + 1] <= lo) {
            sc->sym_off[cur_c + 1] = idx;
            cur_c++;
        }
    }
    while (cur_c < 256) {
        sc->sym_off[cur_c + 1] = nc;
        cur_c++;
    }
    sc->sym_off[0] = 0;

    /* Collect internal target intervals */
    int32_t nt = 0;
    if (nc > sc->int_targets_cap) {
        sc->int_targets_cap = nc * 2;
        sc->int_targets = (Range *)realloc(sc->int_targets, (size_t)sc->int_targets_cap * sizeof(Range));
        sc->merged_t_cap = nc * 2;
        sc->merged_t = (Range *)realloc(sc->merged_t, (size_t)sc->merged_t_cap * sizeof(Range));
    }
    int32_t primary_pos = -1;
    uint8_t primary_fsym = 0;
    for (int32_t i = 0; i < nc; i++) {
        if (col[i].mark_off >= 0) {
            primary_pos = col[i].lo;
        }
        if (col[i].is_int) {
            sc->int_targets[nt].a = col[i].lo + col[i].delta;
            sc->int_targets[nt].b = col[i].hi + col[i].delta;
            nt++;
        }
    }

    if (nt == 0) {
        /* Fast path: ZERO internal transitions! Copy directly to out_pieces! */
        out_pieces->len = 0;
        if (nc > out_pieces->cap) {
            out_pieces->cap = nc > 16 ? nc : 16;
            out_pieces->data = (Piece *)realloc(out_pieces->data, (size_t)out_pieces->cap * sizeof(Piece));
        }
        memcpy(out_pieces->data, col, (size_t)nc * sizeof(Piece));
        out_pieces->len = nc;
        sc->sum_pieces += nc;
        return;
    }

    qsort(sc->int_targets, (size_t)nt, sizeof(Range), cmp_range);
    sc->w_seq += nt;

    int32_t nm = 0;
    for (int32_t i = 0; i < nt; i++) {
        if (nm > 0 && sc->merged_t[nm - 1].b == sc->int_targets[i].a) {
            sc->merged_t[nm - 1].b = sc->int_targets[i].b;
        } else {
            sc->merged_t[nm++] = sc->int_targets[i];
        }
    }

    /* Compute entry_work = domain(col) \ merged_t and push directly onto stack */
    sc->stack.len = 0;
    out_pieces->len = 0;
    int32_t mt_idx = 0;
    int saw_primary_exit = 0;

    for (int32_t i = 0; i < nc; i++) {
        Piece p = col[i];
        int32_t pos = p.lo;
        while (pos < p.hi) {
            while (mt_idx < nm && sc->merged_t[mt_idx].b <= pos) mt_idx++;
            if (mt_idx >= nm || sc->merged_t[mt_idx].a >= p.hi) {
                Piece seg = p;
                seg.lo = pos;
                seg.hi = p.hi;
                if (!seg.is_int) {
                    pv_push(out_pieces, seg);
                    if (seg.mark_off >= 0) saw_primary_exit = 1;
                } else {
                    pv_push(&sc->stack, seg);
                }
                pos = p.hi;
            } else {
                int32_t ta = sc->merged_t[mt_idx].a;
                int32_t tb = sc->merged_t[mt_idx].b;
                if (pos < ta) {
                    Piece seg = p;
                    seg.lo = pos;
                    seg.hi = ta;
                    if (!seg.is_int) {
                        pv_push(out_pieces, seg);
                        if (seg.mark_off >= 0) saw_primary_exit = 1;
                    } else {
                        pv_push(&sc->stack, seg);
                    }
                    pos = ta;
                }
                pos = (p.hi < tb) ? p.hi : tb;
            }
        }
    }

    /* Forward trace bouncing pieces using L1-resident per-symbol slices! */
    while (sc->stack.len > 0) {
        Piece cur = sc->stack.data[--sc->stack.len];
        sc->bounces++;
        int32_t t_lo = cur.lo + cur.delta;
        int32_t t_hi = cur.hi + cur.delta;
        uint8_t ts = cur.tsym;
        int32_t idx = find_piece_in_slice(col, sc->sym_off[ts], sc->sym_off[ts + 1], t_lo);
        int32_t pos = t_lo;
        uint32_t w_len = slp->nodes[cur.slp_id].len;

        while (pos < t_hi) {
            sc->q_rand++;
            const Piece *q = &col[idx];
            int32_t ov_hi = (t_hi < q->hi) ? t_hi : q->hi;
            Piece nxt;
            nxt.lo = pos - cur.delta;
            nxt.hi = ov_hi - cur.delta;
            nxt.delta = cur.delta + q->delta;
            nxt.tsym = q->tsym;
            nxt.slp_id = slp_concat(slp, cur.slp_id, q->slp_id);
            nxt.mark_off = combine_mark(cur.mark_off, w_len, q->mark_off);
            nxt.is_int = q->is_int;
            if (!nxt.is_int) {
                pv_push(out_pieces, nxt);
                if (nxt.mark_off >= 0) saw_primary_exit = 1;
            } else {
                pv_push(&sc->stack, nxt);
            }
            pos = ov_hi;
            idx++;
        }
    }

    /* Closed cycle containing primary mark? */
    if (primary_pos >= 0 && !saw_primary_exit && !(*has_pcyc)) {
        /* Find bucket of primary_pos */
        int32_t idx = find_piece_in_slice(col, 0, nc, primary_pos);
        sc->q_rand++;
        sc->q_root++;
        const Piece *p0 = &col[idx];
        uint32_t w_acc = p0->slp_id;
        int32_t m_acc = p0->mark_off;
        int32_t x = primary_pos + p0->delta;
        uint8_t ts = p0->tsym;
        while (x != primary_pos) {
            sc->q_rand++;
            sc->q_root++;
            idx = find_piece_in_slice(col, sc->sym_off[ts], sc->sym_off[ts + 1], x);
            const Piece *q = &col[idx];
            w_acc = slp_concat(slp, w_acc, q->slp_id);
            x = x + q->delta;
            ts = q->tsym;
        }
        *has_pcyc = 1;
        *pcyc_slp = w_acc;
        *pcyc_mark = m_acc;
    }

    if (out_pieces->len > 1) {
        qsort(out_pieces->data, (size_t)out_pieces->len, sizeof(Piece), cmp_piece_lo);
        sc->w_seq += out_pieces->len;
    }
    sc->sum_pieces += out_pieces->len;
}

/* First-Occurrence Output-Buffer Memoized SLP Expansion (non-recursive!) */
typedef struct {
    uint32_t nid;
    int32_t exit_pos; /* >= 0 if exit marker, -1 if visit */
} ExpFrame;

static void expand_slp_memo(const SlpStore *slp, uint32_t root_id, uint8_t *out, int32_t *first_pos, ExpFrame *stack) {
    int32_t sp = 0;
    int32_t cur = 0;
    stack[sp++] = (ExpFrame){ root_id, -1 };

    while (sp > 0) {
        ExpFrame f = stack[--sp];
        uint32_t nid = f.nid;
        if (f.exit_pos >= 0) {
            first_pos[nid] = f.exit_pos;
            continue;
        }
        int32_t fp = first_pos[nid];
        if (fp >= 0) {
            uint32_t ln = slp->nodes[nid].len;
            memcpy(out + cur, out + fp, ln);
            cur += (int32_t)ln;
            continue;
        }
        if (nid < 256) {
            out[cur] = (uint8_t)nid;
            first_pos[nid] = cur;
            cur++;
        } else {
            SlpNode nd = slp->nodes[nid];
            if ((nd.a & 0x80000000u) == 0) {
                /* Concat node */
                stack[sp++] = (ExpFrame){ nid, cur };
                stack[sp++] = (ExpFrame){ nd.b, -1 };
                stack[sp++] = (ExpFrame){ nd.a, -1 };
            } else {
                /* Power node: rare on Wikidata (0 spirals on block-0), handle cleanly */
                uint32_t base = nd.a & 0x7FFFFFFFu;
                uint32_t rep = nd.b;
                uint32_t blen = slp->nodes[base].len;
                int32_t sh = cur;
                /* Expand base once inline using mini-recursion on remaining stack */
                int32_t save_sp = sp;
                stack[sp++] = (ExpFrame){ base, -1 };
                while (sp > save_sp) {
                    ExpFrame g = stack[--sp];
                    uint32_t gid = g.nid;
                    if (g.exit_pos >= 0) { first_pos[gid] = g.exit_pos; continue; }
                    int32_t gfp = first_pos[gid];
                    if (gfp >= 0) {
                        uint32_t gln = slp->nodes[gid].len;
                        memcpy(out + cur, out + gfp, gln);
                        cur += (int32_t)gln;
                    } else if (gid < 256) {
                        out[cur] = (uint8_t)gid;
                        first_pos[gid] = cur++;
                    } else {
                        SlpNode gnd = slp->nodes[gid];
                        stack[sp++] = (ExpFrame){ gid, cur };
                        stack[sp++] = (ExpFrame){ gnd.b, -1 };
                        stack[sp++] = (ExpFrame){ gnd.a, -1 };
                    }
                }
                first_pos[base] = sh;
                uint32_t written = blen;
                uint32_t target = blen * rep;
                while (written < target) {
                    uint32_t chunk = (written <= target - written) ? written : (target - written);
                    memcpy(out + sh + written, out + sh, chunk);
                    written += chunk;
                }
                cur = sh + (int32_t)target;
                first_pos[nid] = sh;
            }
        }
    }
}

/* Tree node for agglomerative merge plan */
typedef struct {
    int left, right; /* -1 if leaf */
    int sym;
} TreeNode;

static int build_leverage_tree_c(
    const int *present_syms, int n_syms,
    const int64_t row_w[256][256],
    const int32_t pcs_w[256][256],
    const int32_t leaf_sz[256],
    TreeNode *nodes_out
) {
    /* Active cluster table (max 512 nodes) */
    int active[512];
    int n_act = n_syms;
    static int64_t rw[512][512];
    static int32_t pw[512][512];
    int32_t sz[512];

    for (int i = 0; i < n_syms; i++) {
        nodes_out[i].left = -1;
        nodes_out[i].right = -1;
        nodes_out[i].sym = present_syms[i];
        active[i] = i;
        sz[i] = leaf_sz[present_syms[i]];
    }
    for (int i = 0; i < n_syms; i++) {
        int ci = present_syms[i];
        for (int j = 0; j < n_syms; j++) {
            int cj = present_syms[j];
            rw[i][j] = (i == j) ? 0 : (row_w[ci][cj] + row_w[cj][ci]);
            pw[i][j] = (i == j) ? 0 : (pcs_w[ci][cj] + pcs_w[cj][ci]);
        }
    }

    /* Isolate rarest symbol (minimum total cross-row degree) for root! */
    int rarest_pos = 0;
    int64_t best_deg = (1LL << 60);
    for (int i = 0; i < n_act; i++) {
        int u = active[i];
        int64_t d = 0;
        for (int j = 0; j < n_act; j++) d += rw[u][active[j]];
        if (d < best_deg) { best_deg = d; rarest_pos = i; }
    }
    int rarest_id = active[rarest_pos];
    active[rarest_pos] = active[--n_act];

    int next_id = n_syms;
    while (n_act > 1) {
        int bi = 0, bj = 1;
        double best_sc = -1.0;
        for (int i = 0; i < n_act; i++) {
            int u = active[i];
            for (int j = i + 1; j < n_act; j++) {
                int v = active[j];
                int64_t rwt = rw[u][v];
                if (rwt <= 0) continue;
                int32_t pwt = pw[u][v];
                /* width_times_rows objective: rwt^2 / ((pwt + 1) * (sz[u] + sz[v] + 1)) */
                double sc = ((double)rwt * (double)rwt) / (((double)pwt + 1.0) * ((double)(sz[u] + sz[v]) + 1.0));
                if (sc > best_sc) {
                    best_sc = sc;
                    bi = i; bj = j;
                }
            }
        }
        int u = active[bi], v = active[bj];
        int cid = next_id++;
        nodes_out[cid].left = u;
        nodes_out[cid].right = v;
        nodes_out[cid].sym = -1;
        sz[cid] = sz[u] + sz[v];

        /* Remove bj then bi (bj > bi) */
        active[bj] = active[--n_act];
        active[bi] = active[--n_act];

        for (int i = 0; i < n_act; i++) {
            int nb = active[i];
            int64_t nr = rw[u][nb] + rw[v][nb];
            int32_t np = pw[u][nb] + pw[v][nb];
            rw[cid][nb] = rw[nb][cid] = nr;
            pw[cid][nb] = pw[nb][cid] = np;
        }
        rw[cid][cid] = 0;
        pw[cid][cid] = 0;
        active[n_act++] = cid;
    }

    int root_id = next_id++;
    nodes_out[root_id].left = active[0];
    nodes_out[root_id].right = rarest_id;
    nodes_out[root_id].sym = -1;
    return root_id;
}

static double now_ms(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (double)ts.tv_sec * 1e3 + (double)ts.tv_nsec * 1e-6;
}

typedef struct {
    uint8_t c;
    int32_t len;
} RunRec;

/* Reference LF walk for exactness verification */
static void ref_ibwt(const uint8_t *L, int32_t n, int32_t p, uint8_t *out) {
    int32_t C[256] = {0};
    for (int32_t i = 0; i < n; i++) C[L[i]]++;
    int32_t acc = 0;
    for (int c = 0; c < 256; c++) { int32_t cnt = C[c]; C[c] = acc; acc += cnt; }
    int32_t *LF = (int32_t *)malloc((size_t)n * sizeof(int32_t));
    int32_t cur[256];
    memcpy(cur, C, sizeof(cur));
    for (int32_t i = 0; i < n; i++) LF[i] = cur[L[i]]++;
    int32_t x = p;
    for (int32_t k = n - 1; k >= 0; k--) {
        out[k] = L[x];
        x = LF[x];
    }
    free(LF);
}

typedef struct {
    const char *file;
    int32_t n;
    int32_t p;
} BlockSpec;

static const BlockSpec BLOCKS[6] = {
    {"block-0.L",   899931, 264045},
    {"block-2.L",   900000, 320807},
    {"block-85.L",  899935, 325088},
    {"block-143.L", 899869, 802868},
    {"block-210.L", 899959, 791429},
    {"block-207.L", 899998, 205180},
};

static int cmp_dbl(const void *a, const void *b) {
    double da = *(const double *)a, db = *(const double *)b;
    return (da > db) - (da < db);
}

static void eval_tree_rec(
    int node_id,
    const TreeNode *tree,
    PieceVec *leaf_pv,
    const int32_t *C,
    SlpStore *slp,
    Scratch *sc,
    SymSet *out_ss,
    PieceVec *out_pv,
    int *has_pcyc,
    uint32_t *pcyc_slp,
    int32_t *pcyc_mark
) {
    if (tree[node_id].left < 0) {
        int c = tree[node_id].sym;
        ss_clear(out_ss);
        ss_add(out_ss, (uint8_t)c);
        *out_pv = leaf_pv[c];
        leaf_pv[c].data = NULL;
        return;
    }
    SymSet sl, sr;
    PieceVec pl, pr;
    eval_tree_rec(tree[node_id].left, tree, leaf_pv, C, slp, sc, &sl, &pl, has_pcyc, pcyc_slp, pcyc_mark);
    eval_tree_rec(tree[node_id].right, tree, leaf_pv, C, slp, sc, &sr, &pr, has_pcyc, pcyc_slp, pcyc_mark);
    *out_ss = ss_union(sl, sr);
    pv_init(out_pv, pl.len + pr.len + 16);
    resolve_subset_c(out_ss, &pl, &pr, C, slp, sc, out_pv, has_pcyc, pcyc_slp, pcyc_mark);
    pv_free(&pl);
    pv_free(&pr);
}

int main(void) {
    printf("block        |   r0  |   m16  |  Q_rand (x m16) | |G_slp| | T_acq  T_con  T_exp  T_tot (ms) | exact\n");
    printf("-------------+-------+--------+-----------------+---------+-----------------------------------+------\n");

    double sum_acq = 0, sum_con = 0, sum_exp = 0, sum_tot = 0;

    for (int bi = 0; bi < 6; bi++) {
        char path[256];
        snprintf(path, sizeof(path), "/tmp/mibz2-blocks/%s", BLOCKS[bi].file);
        FILE *fp = fopen(path, "rb");
        if (!fp) { perror(path); return 1; }
        int32_t n = BLOCKS[bi].n;
        int32_t p = BLOCKS[bi].p;
        uint8_t *L = (uint8_t *)malloc((size_t)n);
        if ((int32_t)fread(L, 1, (size_t)n, fp) != n) { fprintf(stderr, "short read\n"); return 1; }
        fclose(fp);

        /* Pre-encode uncapped maximal runs I_0 (so I_0 is the exact entry state!) */
        RunRec *runs = (RunRec *)malloc((size_t)n * sizeof(RunRec));
        int32_t r0 = 0;
        int32_t counts[256] = {0};
        int32_t m16 = 0;
        int p_thin = 0;
        for (int32_t i = 0; i < n; ) {
            uint8_t c = L[i];
            int32_t j = i + 1;
            while (j < n && L[j] == c) j++;
            int32_t ln = j - i;
            runs[r0].c = c;
            runs[r0].len = ln;
            r0++;
            counts[c] += ln;
            if (ln <= 16) {
                m16 += ln;
                if (i <= p && p < j) p_thin = 1;
            }
            i = j;
        }
        if (!p_thin) m16++;

        uint8_t *ref_out = (uint8_t *)malloc((size_t)n);
        ref_ibwt(L, n, p, ref_out);
        free(L);

        uint8_t *cyc_buf = (uint8_t *)malloc((size_t)n);
        uint8_t *out_buf = (uint8_t *)malloc((size_t)n);

        double t_acq[5], t_con[5], t_exp[5], t_tot[5];
        int64_t last_q = 0;
        uint32_t last_slp = 0;
        int exact_ok = 1;

        for (int trial = 0; trial < 5; trial++) {
            double t0 = now_ms();

            /* === STAGE 1: Acquire pieces + 256x256 bigram table + leverage merge tree from I_0 === */
            int32_t C[257];
            int present_syms[256], n_syms = 0;
            int32_t acc = 0;
            for (int c = 0; c < 256; c++) {
                C[c] = acc;
                if (counts[c] > 0) present_syms[n_syms++] = c;
                acc += counts[c];
            }
            C[256] = acc;

            int32_t cur_t[256];
            memcpy(cur_t, C, 256 * sizeof(int32_t));

            PieceVec raw_bkt[256];
            for (int i = 0; i < 256; i++) raw_bkt[i].data = NULL, raw_bkt[i].len = raw_bkt[i].cap = 0;
            for (int i = 0; i < n_syms; i++) pv_init(&raw_bkt[present_syms[i]], 64);

            static int64_t row_w[256][256];
            static int32_t pcs_w[256][256];
            memset(row_w, 0, sizeof(row_w));
            memset(pcs_w, 0, sizeof(pcs_w));

            int b_idx = 0;
            int32_t r_lo = 0;
            for (int32_t k = 0; k < r0; k++) {
                uint8_t c = runs[k].c;
                int32_t ln = runs[k].len;
                int32_t r_hi = r_lo + ln;
                int32_t delta = cur_t[c] - r_lo;
                cur_t[c] += ln;

                int32_t pos = r_lo;
                while (pos < r_hi) {
                    while (b_idx < n_syms && C[present_syms[b_idx] + 1] <= pos) b_idx++;
                    uint8_t bc = (uint8_t)present_syms[b_idx];
                    int32_t b_hi = C[bc + 1];
                    int32_t seg_hi = (r_hi < b_hi) ? r_hi : b_hi;
                    row_w[bc][c] += (seg_hi - pos);
                    pcs_w[bc][c] += 1;
                    if (pos <= p && p < seg_hi) {
                        if (pos < p) pv_push(&raw_bkt[bc], (Piece){pos, p, delta, c, -1, c, 0});
                        pv_push(&raw_bkt[bc], (Piece){p, p + 1, delta, c, 0, c, 0});
                        if (p + 1 < seg_hi) pv_push(&raw_bkt[bc], (Piece){p + 1, seg_hi, delta, c, -1, c, 0});
                    } else {
                        pv_push(&raw_bkt[bc], (Piece){pos, seg_hi, delta, c, -1, c, 0});
                    }
                    pos = seg_hi;
                }
                r_lo = r_hi;
            }

            int32_t leaf_sz[256] = {0};
            for (int i = 0; i < n_syms; i++) leaf_sz[present_syms[i]] = raw_bkt[present_syms[i]].len;
            TreeNode tree[512];
            int root_node = build_leverage_tree_c(present_syms, n_syms, row_w, pcs_w, leaf_sz, tree);

            double t1 = now_ms();

            /* === STAGE 2: Compositional Redheffer Star Product Contraction === */
            SlpStore slp;
            slp_init(&slp, (uint32_t)(r0 * 4 + 1024));
            Scratch sc;
            scratch_init(&sc);

            int has_pcyc = 0;
            uint32_t pcyc_slp = 0;
            int32_t pcyc_mark = -1;

            PieceVec leaf_out[256];
            for (int i = 0; i < 256; i++) leaf_out[i].data = NULL, leaf_out[i].len = leaf_out[i].cap = 0;
            for (int i = 0; i < n_syms; i++) {
                uint8_t c = (uint8_t)present_syms[i];
                SymSet ss; ss_clear(&ss); ss_add(&ss, c);
                pv_init(&leaf_out[c], raw_bkt[c].len + 8);
                resolve_subset_c(&ss, &raw_bkt[c], NULL, C, &slp, &sc, &leaf_out[c], &has_pcyc, &pcyc_slp, &pcyc_mark);
                pv_free(&raw_bkt[c]);
            }

            SymSet root_ss;
            PieceVec root_pv;
            eval_tree_rec(root_node, tree, leaf_out, C, &slp, &sc, &root_ss, &root_pv, &has_pcyc, &pcyc_slp, &pcyc_mark);
            pv_free(&root_pv);

            double t2 = now_ms();

            /* === STAGE 3: First-Occurrence Memoized SLP Expansion + Rotate/Reverse === */
            int32_t *first_pos = (int32_t *)malloc((size_t)slp.len * sizeof(int32_t));
            memset(first_pos, -1, (size_t)slp.len * sizeof(int32_t));
            ExpFrame *exp_stack = (ExpFrame *)malloc((size_t)(slp.len * 2 + 1024) * sizeof(ExpFrame));
            expand_slp_memo(&slp, pcyc_slp, cyc_buf, first_pos, exp_stack);

            /* Rotate by pcyc_mark and reverse into out_buf */
            int32_t q = (int32_t)slp.nodes[pcyc_slp].len;
            int32_t dst = n - 1;
            for (int32_t k = pcyc_mark; k < q && dst >= 0; k++) out_buf[dst--] = cyc_buf[k];
            for (int32_t k = 0; k < pcyc_mark && dst >= 0; k++) out_buf[dst--] = cyc_buf[k];

            double t3 = now_ms();

            if (trial == 0) {
                if (memcmp(out_buf, ref_out, (size_t)n) != 0) exact_ok = 0;
                last_q = sc.q_rand;
                last_slp = slp.len - 256;
            }

            t_acq[trial] = t1 - t0;
            t_con[trial] = t2 - t1;
            t_exp[trial] = t3 - t2;
            t_tot[trial] = t3 - t0;

            free(first_pos);
            free(exp_stack);
            scratch_free(&sc);
            free(slp.nodes);
        }

        qsort(t_acq, 5, sizeof(double), cmp_dbl);
        qsort(t_con, 5, sizeof(double), cmp_dbl);
        qsort(t_exp, 5, sizeof(double), cmp_dbl);
        qsort(t_tot, 5, sizeof(double), cmp_dbl);

        sum_acq += t_acq[2];
        sum_con += t_con[2];
        sum_exp += t_exp[2];
        sum_tot += t_tot[2];

        printf("%-12s | %5d | %6d | %7ld (%5.3fx) | %7u | %6.2f %6.2f %6.2f %6.2f ms | %s\n",
               BLOCKS[bi].file, r0, m16, (long)last_q, (double)last_q / (double)m16,
               last_slp, t_acq[2], t_con[2], t_exp[2], t_tot[2],
               exact_ok ? "PASS" : "FAIL");

        free(runs);
        free(ref_out);
        free(cyc_buf);
        free(out_buf);
    }

    printf("-------------+-------+--------+-----------------+---------+-----------------------------------+------\n");
    printf("SUM 6 BLOCKS |       |        |                 |         | %6.2f %6.2f %6.2f %6.2f ms |\n",
           sum_acq, sum_con, sum_exp, sum_tot);
    return 0;
}
