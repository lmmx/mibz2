/* chaingrind: the moonshot scorer's cache and dependency model (bench/moonshot/README.md).
 *
 * A Valgrind tool. Between the client requests MS_BEGIN and MS_END (bench/moonshot's
 * `proxy` driver: one worker's decode_block over sampled blocks) it replays every data
 * access in program order through a three-level cache model, and carries, for every value
 * (IR temporaries, guest registers by byte, memory by 8-byte granule), the cycle it becomes
 * ready and the dependent chain that made it late, in a simple out-of-order timing model:
 *
 *   - an instruction dispatches no earlier than the one WIDTH before it plus a cycle, nor
 *     before the instruction ROB before it has retired (in-order retirement);
 *   - a value is ready at max(its inputs, its dispatch) plus its operation's latency;
 *   - a load issues when its address is ready, a miss past L1 also no earlier than the
 *     miss MSHR before it completed (the fill buffers), and its value is ready after the
 *     latency of the level that held the line (a store's value forwarded when later);
 *   - a store writes its data's readiness into the memory shadow and holds nothing up.
 *
 * Each value also carries three chain counts: the loads at any level, the loads served past
 * L2 (L3 or DRAM), and the loads served by DRAM, on the path that made it ready last, counted
 * only while that path is binding (an input ready after the instruction dispatched). A load whose address waited on a
 * value with a DRAM count of at least one is a dependent DRAM miss: a link in a serially
 * dependent L3-miss chain (MONODROMY record 71's metric). The same for past-L2 loads, and
 * for loads at any level (a dependent stage: records 72 and 75 count about 0.2-0.25 a byte
 * for lazy Cells, by the algorithm).
 *
 * Model, not measurement: no hardware prefetcher but a simple L2 streamer (--prefetch),
 * true LRU, non-inclusive levels, write-allocate, virtual set indexing, no coherence and no
 * other cores (contention enters only as the LLC's share, --llc-kb, and as the latencies,
 * --l3-lat and --dram-lat, which the scorer sets per worker count). Control dependences are
 * ignored, as a predicted branch would.
 *
 * Output (at exit): one line "MOONSHOT-PROXY {json}" of totals over the measured regions.
 */
#include "pub_tool_basics.h"
#include "pub_tool_tooliface.h"
#include "pub_tool_libcassert.h"
#include "pub_tool_libcprint.h"
#include "pub_tool_libcbase.h"
#include "pub_tool_options.h"
#include "pub_tool_machine.h"
#include "pub_tool_mallocfree.h"
#include "pub_tool_clreq.h"

#define MS_BEGIN (VG_USERREQ_TOOL_BASE('M', 'S'))
#define MS_END (VG_USERREQ_TOOL_BASE('M', 'S') + 1)

/* ---- options ---- */
static Long clo_llc_kb = 2048;
static Long clo_llc_ways = 16;
static Long clo_l1_lat = 5;
static Long clo_l2_lat = 14;
static Long clo_l3_lat = 42;
static Long clo_dram_lat = 350;
static Long clo_rob = 224;
static Long clo_width = 4;
static Long clo_mshr = 12;
static Long clo_sb = 56;
static Bool clo_prefetch = True;

/* ---- the clock ---- */
static Bool active = False;
static ULong n_instr = 0;
static ULong dispatch_t = 0;   /* the current instruction's dispatch cycle */
static ULong complete_t = 0;   /* its results' latest ready cycle */
static ULong last_retire = 0;
static ULong *retire_ring;     /* ROB entries */
static ULong *dispatch_ring;   /* WIDTH entries */
static ULong *mshr_ring;       /* MSHR entries */
static ULong n_mshr = 0;
static ULong *sb_ring;          /* store buffer entries: each store's drain cycle */
static ULong n_sb = 0, sb_last = 0;
static ULong region_start = 0;

/* ---- shadows: ready cycle and chain (past-L2 count << 16 | DRAM count) ---- */
static ULong *t_ready;
static UInt *t_chain;
static Int t_cap = 0;
#define NO_TMP 0xFFFFFFFFu
static ULong r_ready[4096];
static UInt r_chain[4096];

typedef struct {
   ULong ready;
   UInt chain;
   UInt pad;
} Gran;
#define CHUNK_BITS 16
#define CHUNK_GRANS (1 << (CHUNK_BITS - 3))
#define TABLE (1 << 18)
static Addr chunk_key[TABLE];
static Gran *chunk_val[TABLE];

static Gran *gran(Addr a, Bool create)
{
   Addr key = (a >> CHUNK_BITS) + 1;
   UInt h = (UInt)((key * 0x9E3779B97F4A7C15ULL) >> 46) & (TABLE - 1);
   while (chunk_key[h] != 0) {
      if (chunk_key[h] == key)
         return &chunk_val[h][(a >> 3) & (CHUNK_GRANS - 1)];
      h = (h + 1) & (TABLE - 1);
   }
   if (!create)
      return NULL;
   chunk_key[h] = key;
   chunk_val[h] = VG_(calloc)("cg.chunk", CHUNK_GRANS, sizeof(Gran));
   return &chunk_val[h][(a >> 3) & (CHUNK_GRANS - 1)];
}

/* ---- statistics ---- */
static ULong s_loads, s_stores, s_out_bytes, s_cycles;
static ULong s_l1_miss, s_l2_miss, s_llc_miss;            /* demand line lookups */
static ULong s_l2_xfer, s_l3_xfer, s_dram_read, s_dram_write, s_dram_drain, s_prefetched;
static ULong s_past_l2_loads, s_dram_loads;               /* demand loads by where served */
static ULong s_dep_any, s_dep_past_l2, s_dep_dram;                  /* their dependent links */
static ULong s_max_chain_dram, s_max_chain_past_l2;
static ULong s_sb_stall;                                 /* dispatch cycles lost to a full store buffer */
static ULong s_chain_sum_dram;                           /* the DRAM count at each DRAM load */

typedef struct {
   ULong s, e;
   UInt dep;
   UInt pad;
} Interval;
static Interval *iv;
static ULong iv_n = 0, iv_cap = 0;

/* ---- the caches ---- */
typedef struct {
   UInt sets, ways;
   Addr *tag;   /* line + 1; 0 empty; MRU first within a set */
   UChar *dirty;
   ULong *when; /* the cycle the line's fill completes (a line still in flight is waited for) */
} Cache;
static ULong hit_when; /* `cache_access`'s hit: its line's fill cycle */
static Cache L1, L2, LLC;

static void cache_init(Cache *c, ULong bytes, UInt ways)
{
   c->ways = ways;
   c->sets = (UInt)(bytes / 64 / ways);
   if (c->sets == 0)
      c->sets = 1;
   c->tag = VG_(calloc)("cg.tag", (SizeT)c->sets * ways, sizeof(Addr));
   c->dirty = VG_(calloc)("cg.dirty", (SizeT)c->sets * ways, 1);
   c->when = VG_(calloc)("cg.when", (SizeT)c->sets * ways, sizeof(ULong));
}

/* look `line` up: True on a hit, made MRU (dirty if `write`); on a miss it is filled MRU
 * and the LRU line evicted: *evicted its line (0 none) and *evdirty whether it was dirty */
static Bool cache_access(Cache *c, Addr line, Bool write, Addr *evicted, Bool *evdirty)
{
   UInt set = (UInt)(line % c->sets);
   Addr *t = &c->tag[(SizeT)set * c->ways];
   UChar *d = &c->dirty[(SizeT)set * c->ways];
   ULong *w = &c->when[(SizeT)set * c->ways];
   Addr key = line + 1;
   UInt i;
   for (i = 0; i < c->ways; i++) {
      if (t[i] == key) {
         UChar dd = d[i] | (write ? 1 : 0);
         ULong ww = w[i];
         for (; i > 0; i--) {
            t[i] = t[i - 1];
            d[i] = d[i - 1];
            w[i] = w[i - 1];
         }
         t[0] = key;
         d[0] = dd;
         w[0] = ww;
         hit_when = ww;
         *evicted = 0;
         *evdirty = False;
         return True;
      }
   }
   i = c->ways - 1;
   *evicted = t[i] ? t[i] - 1 : 0;
   *evdirty = t[i] && d[i];
   for (; i > 0; i--) {
      t[i] = t[i - 1];
      d[i] = d[i - 1];
      w[i] = w[i - 1];
   }
   t[0] = key;
   d[0] = write ? 1 : 0;
   w[0] = 0;
   return False;
}

/* the line's fill completes at `when` (it is resident) */
static void set_when(Cache *c, Addr line, ULong when)
{
   UInt set = (UInt)(line % c->sets);
   Addr *t = &c->tag[(SizeT)set * c->ways];
   for (UInt i = 0; i < c->ways; i++)
      if (t[i] == line + 1) {
         c->when[(SizeT)set * c->ways + i] = when;
         return;
      }
}

static Bool cache_has(Cache *c, Addr line)
{
   UInt set = (UInt)(line % c->sets);
   Addr *t = &c->tag[(SizeT)set * c->ways];
   for (UInt i = 0; i < c->ways; i++)
      if (t[i] == line + 1)
         return True;
   return False;
}

static void llc_put(Addr line, Bool write);
static void l2_put(Addr line, Bool write)
{
   Addr ev;
   Bool evd;
   if (!cache_access(&L2, line, write, &ev, &evd)) {
      /* (a writeback from L1 that missed: filled here, no demand transfer below) */
   }
   if (evd) {
      s_l3_xfer++;
      llc_put(ev, True);
   }
}
static void llc_put(Addr line, Bool write)
{
   Addr ev;
   Bool evd;
   cache_access(&LLC, line, write, &ev, &evd);
   if (evd)
      s_dram_write++;
}

/* the L2 streamer: recent L2 demand-miss lines; a miss one line past one of them fetches
 * the next lines ahead into L2 (and the LLC), each arriving after its level's latency from
 * the miss's own issue: a demand access before then waits for it */
#define STREAMS 16
#define AHEAD 4
static Addr streams[STREAMS];
static UInt stream_next = 0;
static void prefetch_after(Addr line, ULong now)
{
   Bool seq = False;
   for (UInt i = 0; i < STREAMS; i++)
      if (streams[i] && (streams[i] + 1 == line || streams[i] - 1 == line)) {
         seq = True;
         streams[i] = line;
         break;
      }
   if (!seq) {
      streams[stream_next] = line;
      stream_next = (stream_next + 1) % STREAMS;
      return;
   }
   for (UInt k = 1; k <= AHEAD; k++) {
      Addr p = line + k;
      if (cache_has(&L2, p))
         continue;
      s_prefetched++;
      s_l3_xfer++;
      ULong arrive = now + clo_l3_lat;
      if (!cache_has(&LLC, p)) {
         s_dram_read++;
         llc_put(p, False);
         arrive = now + clo_dram_lat;
         set_when(&LLC, p, arrive);
      }
      Addr ev;
      Bool evd;
      cache_access(&L2, p, False, &ev, &evd);
      set_when(&L2, p, arrive);
      if (evd) {
         s_l3_xfer++;
         llc_put(ev, True);
      }
   }
}

static inline ULong later(ULong a, ULong b)
{
   return a > b ? a : b;
}

/* a demand access to one line at cycle `now`: the level that served it, 1-4, and in *ready
 * the cycle its data is there (a line still in flight is waited for) */
static UInt access_line(Addr line, Bool write, ULong now, ULong *ready)
{
   Addr ev;
   Bool evd;
   if (cache_access(&L1, line, write, &ev, &evd)) {
      *ready = later(now + clo_l1_lat, hit_when);
      return 1;
   }
   s_l1_miss++;
   s_l2_xfer++;
   if (evd) {
      s_l2_xfer++;
      l2_put(ev, True);
   }
   UInt level;
   if (cache_access(&L2, line, False, &ev, &evd)) {
      *ready = later(now + clo_l2_lat, hit_when);
      level = 2;
   } else {
      s_l2_miss++;
      s_l3_xfer++;
      if (evd) {
         s_l3_xfer++;
         llc_put(ev, True);
      }
      Bool hit = cache_access(&LLC, line, False, &ev, &evd);
      if (hit) {
         *ready = later(now + clo_l3_lat, hit_when);
         level = 3;
      } else {
         s_llc_miss++;
         s_dram_read++;
         *ready = now + clo_dram_lat;
         set_when(&LLC, line, *ready);
         level = 4;
      }
      if (evd)
         s_dram_write++;
      set_when(&L2, line, *ready);
      if (clo_prefetch)
         prefetch_after(line, now);
   }
   set_when(&L1, line, *ready);
   return level;
}

/* ---- the helpers, called from the instrumented code ---- */

static void h_imark(void)
{
   if (!active)
      return;
   ULong retire = complete_t > last_retire ? complete_t : last_retire;
   retire_ring[n_instr % clo_rob] = retire;
   dispatch_ring[n_instr % clo_width] = dispatch_t;
   last_retire = retire;
   n_instr++;
   ULong d = dispatch_t;
   ULong w = dispatch_ring[n_instr % clo_width] + 1;
   ULong r = retire_ring[n_instr % clo_rob];
   if (w > d)
      d = w;
   if (r > d)
      d = r;
   dispatch_t = d;
   complete_t = d;
}

static inline void merge(ULong *ready, UInt *chain, UInt t)
{
   if (t != NO_TMP && t_ready[t] > *ready) {
      *ready = t_ready[t];
      *chain = t_chain[t];
   }
}

static inline void finish(UInt dst, ULong ready, UInt chain, ULong lat)
{
   if (ready <= dispatch_t) {
      ready = dispatch_t;
      chain = 0; /* (inputs ready before dispatch: off the binding path) */
   }
   ready += lat;
   t_ready[dst] = ready;
   t_chain[dst] = chain;
   if (ready > complete_t)
      complete_t = ready;
}

static VG_REGPARM(0) void h_alu(UWord dst, UWord a, UWord b, UWord c, UWord d, UWord lat)
{
   if (!active)
      return;
   ULong ready = 0;
   UInt chain = 0;
   merge(&ready, &chain, (UInt)a);
   merge(&ready, &chain, (UInt)b);
   merge(&ready, &chain, (UInt)c);
   merge(&ready, &chain, (UInt)d);
   finish((UInt)dst, ready, chain, lat);
}

/* more inputs for `dst` (a call with more than four) */
static VG_REGPARM(0) void h_more(UWord dst, UWord a, UWord b, UWord c, UWord d)
{
   if (!active)
      return;
   ULong ready = t_ready[dst];
   UInt chain = t_chain[dst];
   merge(&ready, &chain, (UInt)a);
   merge(&ready, &chain, (UInt)b);
   merge(&ready, &chain, (UInt)c);
   merge(&ready, &chain, (UInt)d);
   t_ready[dst] = ready;
   t_chain[dst] = chain;
   if (ready > complete_t)
      complete_t = ready;
}

static VG_REGPARM(0) void h_get(UWord dst, UWord off, UWord size)
{
   if (!active)
      return;
   ULong ready = 0;
   UInt chain = 0;
   for (UWord i = off; i < off + size && i < 4096; i++)
      if (r_ready[i] > ready) {
         ready = r_ready[i];
         chain = r_chain[i];
      }
   finish((UInt)dst, ready, chain, 0);
}

static VG_REGPARM(0) void h_put(UWord off, UWord size, UWord src)
{
   if (!active)
      return;
   ULong ready = src == NO_TMP ? 0 : t_ready[src];
   UInt chain = src == NO_TMP ? 0 : t_chain[src];
   for (UWord i = off; i < off + size && i < 4096; i++) {
      r_ready[i] = ready;
      r_chain[i] = chain;
   }
}

/* a chain's three counts, packed: loads at any level (bits 24-31), loads past L2 (12-23),
 * loads from DRAM (0-11), each saturating */
#define ANY(c) ((c) >> 24)
#define PAST(c) (((c) >> 12) & 0xFFF)
#define DRAM(c) ((c) & 0xFFF)
static inline UInt bump(UInt chain, UInt level)
{
   UInt any = ANY(chain), past = PAST(chain), dram = DRAM(chain);
   if (any < 0xFF)
      any++;
   if (level >= 3 && past < 0xFFF)
      past++;
   if (level == 4 && dram < 0xFFF)
      dram++;
   return any << 24 | past << 12 | dram;
}

static void record_interval(ULong s, ULong e, Bool dep)
{
   if (iv_n == iv_cap) {
      iv_cap = iv_cap ? iv_cap * 2 : 1 << 16;
      iv = VG_(realloc)("cg.iv", iv, iv_cap * sizeof(Interval));
   }
   iv[iv_n].s = s;
   iv[iv_n].e = e;
   iv[iv_n].dep = dep;
   iv_n++;
}

static VG_REGPARM(0) void h_load(UWord dst, UWord addr, UWord atmp, UWord size)
{
   if (!active)
      return;
   s_loads++;
   ULong a_ready = 0;
   UInt a_chain = 0;
   merge(&a_ready, &a_chain, (UInt)atmp);
   Bool waited = a_ready > dispatch_t;
   if (!waited) {
      a_ready = dispatch_t;
      a_chain = 0;
   }
   UInt level = 1;
   ULong ready = a_ready + clo_l1_lat;
   Addr first = addr >> 6, last = (addr + (size ? size : 1) - 1) >> 6;
   for (Addr l = first; l <= last; l++) {
      ULong r;
      UInt lv = access_line(l, False, a_ready, &r);
      if (lv > level)
         level = lv;
      if (r > ready)
         ready = r;
   }
   ULong issue = a_ready;
   if (level > 1 && clo_mshr > 0) {
      /* the fill buffers: the miss MSHR before this one must have completed */
      ULong free_at = mshr_ring[n_mshr % clo_mshr];
      if (free_at > issue) {
         ready += free_at - issue;
         issue = free_at;
      }
      mshr_ring[n_mshr % clo_mshr] = ready;
      n_mshr++;
   }
   UInt chain = bump(a_chain, level);
   /* a value stored here later than the cache would give it */
   for (Addr g = addr >> 3; g <= (addr + (size ? size : 1) - 1) >> 3; g++) {
      Gran *m = gran(g << 3, False);
      if (m && m->ready > ready) {
         ready = m->ready;
         chain = m->chain;
      }
   }
   if (waited && ANY(a_chain) >= 1)
      s_dep_any++;
   if (level >= 3) {
      s_past_l2_loads++;
      if (waited && PAST(a_chain) >= 1)
         s_dep_past_l2++;
      if (PAST(chain) > s_max_chain_past_l2)
         s_max_chain_past_l2 = PAST(chain);
   }
   if (level == 4) {
      Bool dep = waited && DRAM(a_chain) >= 1;
      s_dram_loads++;
      s_dep_dram += dep;
      s_chain_sum_dram += DRAM(chain);
      if (DRAM(chain) > s_max_chain_dram)
         s_max_chain_dram = DRAM(chain);
      record_interval(issue, ready, dep);
   }
   if (dst != NO_TMP) {
      t_ready[dst] = ready;
      t_chain[dst] = chain;
   }
   if (ready > complete_t)
      complete_t = ready;
}

static VG_REGPARM(0) void h_store(UWord addr, UWord atmp, UWord size, UWord dtmp)
{
   if (!active)
      return;
   s_stores++;
   ULong ready = 0;
   UInt chain = 0;
   merge(&ready, &chain, (UInt)dtmp);
   merge(&ready, &chain, (UInt)atmp);
   if (ready < dispatch_t) {
      ready = dispatch_t;
      chain = 0;
   }
   /* the store buffer: it drains in order, a store once its data is ready and its line is
    * owned (its RFO issued at dispatch, overlapping the others'); a full buffer stalls
    * dispatch until the store SB before this one has drained */
   ULong owned = dispatch_t;
   Addr first = addr >> 6, last = (addr + (size ? size : 1) - 1) >> 6;
   for (Addr l = first; l <= last; l++) {
      ULong r;
      access_line(l, True, dispatch_t, &r);
      if (r > owned)
         owned = r;
   }
   if (clo_sb > 0) {
      ULong *slot = &sb_ring[n_sb % clo_sb];
      if (*slot > dispatch_t) {
         s_sb_stall += *slot - dispatch_t;
         dispatch_t = *slot;
      }
      ULong drain = later(later(sb_last + 1, owned), ready);
      sb_last = drain;
      *slot = drain;
      n_sb++;
   }
   for (Addr g = addr >> 3; g <= (addr + (size ? size : 1) - 1) >> 3; g++) {
      Gran *m = gran(g << 3, True);
      m->ready = ready;
      m->chain = chain;
   }
}

static VG_REGPARM(0) void h_now(UWord dst)
{
   if (!active)
      return;
   finish((UInt)dst, 0, 0, 1);
}

/* ---- instrumentation ---- */

static UWord tmp_of(IRExpr *e)
{
   return (e && e->tag == Iex_RdTmp) ? (UWord)e->Iex.RdTmp.tmp : NO_TMP;
}

static IRExpr *hw(UWord v)
{
   return mkIRExpr_HWord(v);
}

static void call(IRSB *sb, const HChar *name, void *fn, IRExpr **args, IRExpr *guard)
{
   IRDirty *di = unsafeIRDirty_0_N(0, name, VG_(fnptr_to_fnentry)(fn), args);
   if (guard)
      di->guard = guard;
   addStmtToIRSB(sb, IRStmt_Dirty(di));
}

/* an operation's latency: address-style arithmetic, widening and narrowing fold into the
 * instruction (0); multiplies 3, divides 20, floating point 4, the rest 1 */
static UWord op_lat(IROp op, IRExpr **args, Int n)
{
   switch (op) {
   case Iop_Add64: case Iop_Add32: case Iop_Sub64: case Iop_Sub32:
   case Iop_Shl64: case Iop_Shl32:
      for (Int i = 0; i < n; i++)
         if (args[i]->tag == Iex_Const)
            return 0;
      return 1;
   case Iop_8Uto64: case Iop_8Uto32: case Iop_16Uto64: case Iop_16Uto32: case Iop_32Uto64:
   case Iop_8Sto64: case Iop_8Sto32: case Iop_16Sto64: case Iop_16Sto32: case Iop_32Sto64:
   case Iop_64to32: case Iop_64to16: case Iop_64to8: case Iop_32to16: case Iop_32to8:
   case Iop_64to1: case Iop_32to1: case Iop_1Uto64: case Iop_1Uto32: case Iop_1Uto8:
   case Iop_64HIto32: case Iop_32HLto64:
      return 0;
   case Iop_Mul64: case Iop_Mul32: case Iop_MullU64: case Iop_MullS64: case Iop_MullU32:
   case Iop_MullS32:
      return 3;
   case Iop_DivModU128to64: case Iop_DivModS128to64: case Iop_DivModU64to32:
   case Iop_DivModS64to32: case Iop_DivU64: case Iop_DivS64: case Iop_DivU32: case Iop_DivS32:
      return 20;
   default:
      if (op >= Iop_AddF64 && op <= Iop_SqrtF32)
         return 4;
      return 1;
   }
}

static void alu(IRSB *sb, UWord dst, UWord *in, Int n, UWord lat)
{
   UWord a[4] = {NO_TMP, NO_TMP, NO_TMP, NO_TMP};
   Int i;
   for (i = 0; i < n && i < 4; i++)
      a[i] = in[i];
   call(sb, "h_alu", h_alu, mkIRExprVec_6(hw(dst), hw(a[0]), hw(a[1]), hw(a[2]), hw(a[3]), hw(lat)), NULL);
   while (i < n) {
      UWord b[4] = {NO_TMP, NO_TMP, NO_TMP, NO_TMP};
      for (Int j = 0; j < 4 && i < n; j++, i++)
         b[j] = in[i];
      call(sb, "h_more", h_more, mkIRExprVec_5(hw(dst), hw(b[0]), hw(b[1]), hw(b[2]), hw(b[3])), NULL);
   }
}

static void load(IRSB *sb, UWord dst, IRExpr *addr, IRType ty, IRExpr *guard)
{
   call(sb, "h_load", h_load, mkIRExprVec_4(hw(dst), addr, hw(tmp_of(addr)), hw(sizeofIRType(ty))), guard);
}

static void store(IRSB *sb, IRExpr *addr, IRExpr *data, IRType ty, IRExpr *guard)
{
   call(sb, "h_store", h_store, mkIRExprVec_4(addr, hw(tmp_of(addr)), hw(sizeofIRType(ty)), hw(tmp_of(data))), guard);
}

static void wrtmp(IRSB *sb, IRSB *in, IRTemp dst, IRExpr *e)
{
   UWord ins[16];
   Int n = 0;
   switch (e->tag) {
   case Iex_Get:
      call(sb, "h_get", h_get, mkIRExprVec_3(hw(dst), hw(e->Iex.Get.offset), hw(sizeofIRType(e->Iex.Get.ty))), NULL);
      return;
   case Iex_Load:
      load(sb, dst, e->Iex.Load.addr, e->Iex.Load.ty, NULL);
      return;
   case Iex_RdTmp:
   case Iex_Const:
      ins[n++] = tmp_of(e);
      alu(sb, dst, ins, n, 0);
      return;
   case Iex_Unop: {
      IRExpr *a[1] = {e->Iex.Unop.arg};
      ins[n++] = tmp_of(a[0]);
      alu(sb, dst, ins, n, op_lat(e->Iex.Unop.op, a, 1));
      return;
   }
   case Iex_Binop: {
      IRExpr *a[2] = {e->Iex.Binop.arg1, e->Iex.Binop.arg2};
      ins[n++] = tmp_of(a[0]);
      ins[n++] = tmp_of(a[1]);
      alu(sb, dst, ins, n, op_lat(e->Iex.Binop.op, a, 2));
      return;
   }
   case Iex_Triop: {
      IRTriop *t = e->Iex.Triop.details;
      IRExpr *a[3] = {t->arg1, t->arg2, t->arg3};
      for (Int i = 0; i < 3; i++)
         ins[n++] = tmp_of(a[i]);
      alu(sb, dst, ins, n, op_lat(t->op, a, 3));
      return;
   }
   case Iex_Qop: {
      IRQop *q = e->Iex.Qop.details;
      IRExpr *a[4] = {q->arg1, q->arg2, q->arg3, q->arg4};
      for (Int i = 0; i < 4; i++)
         ins[n++] = tmp_of(a[i]);
      alu(sb, dst, ins, n, op_lat(q->op, a, 4));
      return;
   }
   case Iex_ITE:
      ins[n++] = tmp_of(e->Iex.ITE.cond);
      ins[n++] = tmp_of(e->Iex.ITE.iftrue);
      ins[n++] = tmp_of(e->Iex.ITE.iffalse);
      alu(sb, dst, ins, n, 1);
      return;
   case Iex_CCall:
      for (Int i = 0; e->Iex.CCall.args[i] && n < 16; i++)
         ins[n++] = tmp_of(e->Iex.CCall.args[i]);
      alu(sb, dst, ins, n, 1);
      return;
   default: /* GetI and the rest */
      call(sb, "h_now", h_now, mkIRExprVec_1(hw(dst)), NULL);
      return;
   }
}

static IRSB *cg_instrument(VgCallbackClosure *closure, IRSB *sbIn, const VexGuestLayout *layout,
                           const VexGuestExtents *vge, const VexArchInfo *archinfo_host,
                           IRType gWordTy, IRType hWordTy)
{
   if (sbIn->tyenv->types_used > t_cap) {
      Int cap = sbIn->tyenv->types_used * 2;
      t_ready = VG_(realloc)("cg.tr", t_ready, cap * sizeof(ULong));
      t_chain = VG_(realloc)("cg.tc", t_chain, cap * sizeof(UInt));
      for (Int i = t_cap; i < cap; i++) {
         t_ready[i] = 0;
         t_chain[i] = 0;
      }
      t_cap = cap;
   }
   IRSB *sb = deepCopyIRSBExceptStmts(sbIn);
   Int i = 0;
   /* (the preamble before the first IMark is copied as it is) */
   while (i < sbIn->stmts_used && sbIn->stmts[i]->tag != Ist_IMark) {
      addStmtToIRSB(sb, sbIn->stmts[i]);
      i++;
   }
   for (; i < sbIn->stmts_used; i++) {
      IRStmt *st = sbIn->stmts[i];
      if (!st)
         continue;
      switch (st->tag) {
      case Ist_IMark:
         call(sb, "h_imark", h_imark, mkIRExprVec_0(), NULL);
         break;
      case Ist_WrTmp:
         wrtmp(sb, sbIn, st->Ist.WrTmp.tmp, st->Ist.WrTmp.data);
         break;
      case Ist_Put: {
         IRExpr *d = st->Ist.Put.data;
         call(sb, "h_put", h_put,
              mkIRExprVec_3(hw(st->Ist.Put.offset), hw(sizeofIRType(typeOfIRExpr(sbIn->tyenv, d))), hw(tmp_of(d))), NULL);
         break;
      }
      case Ist_Store:
         store(sb, st->Ist.Store.addr, st->Ist.Store.data, typeOfIRExpr(sbIn->tyenv, st->Ist.Store.data), NULL);
         break;
      case Ist_LoadG: {
         IRLoadG *g = st->Ist.LoadG.details;
         UWord alt = tmp_of(g->alt);
         alu(sb, g->dst, &alt, 1, 0);
         IRType ty = Ity_I64;
         switch (g->cvt) {
         case ILGop_Ident64: ty = Ity_I64; break;
         case ILGop_Ident32: ty = Ity_I32; break;
         case ILGop_16Uto32: case ILGop_16Sto32: ty = Ity_I16; break;
         case ILGop_8Uto32: case ILGop_8Sto32: ty = Ity_I8; break;
         default: break;
         }
         load(sb, g->dst, g->addr, ty, g->guard);
         break;
      }
      case Ist_StoreG: {
         IRStoreG *g = st->Ist.StoreG.details;
         store(sb, g->addr, g->data, typeOfIRExpr(sbIn->tyenv, g->data), g->guard);
         break;
      }
      case Ist_CAS: {
         IRCAS *c = st->Ist.CAS.details;
         IRType ty = typeOfIRExpr(sbIn->tyenv, c->dataLo);
         load(sb, c->oldLo, c->addr, ty, NULL);
         if (c->oldHi != IRTemp_INVALID)
            call(sb, "h_now", h_now, mkIRExprVec_1(hw(c->oldHi)), NULL);
         store(sb, c->addr, c->dataLo, ty, NULL);
         break;
      }
      case Ist_LLSC:
         if (st->Ist.LLSC.storedata == NULL) {
            load(sb, st->Ist.LLSC.result, st->Ist.LLSC.addr, typeOfIRTemp(sbIn->tyenv, st->Ist.LLSC.result), NULL);
         } else {
            store(sb, st->Ist.LLSC.addr, st->Ist.LLSC.storedata,
                  typeOfIRExpr(sbIn->tyenv, st->Ist.LLSC.storedata), NULL);
            call(sb, "h_now", h_now, mkIRExprVec_1(hw(st->Ist.LLSC.result)), NULL);
         }
         break;
      case Ist_Dirty:
         if (st->Ist.Dirty.details->tmp != IRTemp_INVALID)
            call(sb, "h_now", h_now, mkIRExprVec_1(hw(st->Ist.Dirty.details->tmp)), NULL);
         break;
      default: /* NoOp, AbiHint, PutI, MBE, Exit */
         break;
      }
      addStmtToIRSB(sb, st);
   }
   return sb;
}

/* ---- client requests ---- */

static void drain_cache(Cache *c, ULong *count)
{
   for (SizeT i = 0; i < (SizeT)c->sets * c->ways; i++)
      if (c->tag[i] && c->dirty[i]) {
         (*count)++;
         c->dirty[i] = 0;
      }
}

static Bool cg_client_request(ThreadId tid, UWord *args, UWord *ret)
{
   if (args[0] == MS_BEGIN) {
      active = True;
      region_start = last_retire > dispatch_t ? last_retire : dispatch_t;
      dispatch_t = region_start;
      complete_t = region_start;
      *ret = 1;
      return True;
   }
   if (args[0] == MS_END) {
      ULong end = last_retire > complete_t ? last_retire : complete_t;
      s_cycles += end - region_start;
      s_out_bytes += args[1];
      active = False;
      *ret = 1;
      return True;
   }
   return False;
}

/* ---- options and output ---- */

static Bool cg_option(const HChar *arg)
{
   if VG_INT_CLO(arg, "--llc-kb", clo_llc_kb) {}
   else if VG_INT_CLO(arg, "--llc-ways", clo_llc_ways) {}
   else if VG_INT_CLO(arg, "--l1-lat", clo_l1_lat) {}
   else if VG_INT_CLO(arg, "--l2-lat", clo_l2_lat) {}
   else if VG_INT_CLO(arg, "--l3-lat", clo_l3_lat) {}
   else if VG_INT_CLO(arg, "--dram-lat", clo_dram_lat) {}
   else if VG_INT_CLO(arg, "--rob", clo_rob) {}
   else if VG_INT_CLO(arg, "--width", clo_width) {}
   else if VG_INT_CLO(arg, "--mshr", clo_mshr) {}
   else if VG_INT_CLO(arg, "--sb", clo_sb) {}
   else if VG_BOOL_CLO(arg, "--prefetch", clo_prefetch) {}
   else return False;
   return True;
}

static void cg_usage(void)
{
   VG_(printf)("    --llc-kb=N --llc-ways=N --l1-lat=N --l2-lat=N --l3-lat=N --dram-lat=N\n"
               "    --rob=N --width=N --mshr=N --sb=N --prefetch=yes|no\n");
}

static void cg_debug_usage(void) {}

static void cg_post_clo_init(void)
{
   cache_init(&L1, 32 * 1024, 8);
   cache_init(&L2, 256 * 1024, 4);
   cache_init(&LLC, (ULong)clo_llc_kb * 1024, (UInt)clo_llc_ways);
   if (clo_rob < 1) clo_rob = 1;
   if (clo_width < 1) clo_width = 1;
   retire_ring = VG_(calloc)("cg.rob", clo_rob, sizeof(ULong));
   dispatch_ring = VG_(calloc)("cg.width", clo_width, sizeof(ULong));
   sb_ring = VG_(calloc)("cg.sb", clo_sb > 0 ? clo_sb : 1, sizeof(ULong));
   mshr_ring = VG_(calloc)("cg.mshr", clo_mshr > 0 ? clo_mshr : 1, sizeof(ULong));
}

static Int by_start(const void *a, const void *b)
{
   ULong x = ((const Interval *)a)->s, y = ((const Interval *)b)->s;
   return x < y ? -1 : x > y;
}

static Int by_value(const void *a, const void *b)
{
   ULong x = *(const ULong *)a, y = *(const ULong *)b;
   return x < y ? -1 : x > y;
}

/* how many of the sorted `v[0..n)` are below `x` */
static ULong below(const ULong *v, ULong n, ULong x)
{
   ULong lo = 0, hi = n;
   while (lo < hi) {
      ULong mid = (lo + hi) / 2;
      if (v[mid] < x)
         lo = mid + 1;
      else
         hi = mid;
   }
   return lo;
}

static void cg_fini(Int exitcode)
{
   ULong l1d = 0, l2d = 0, llcd = 0;
   drain_cache(&L1, &l1d);
   drain_cache(&L2, &l2d);
   drain_cache(&LLC, &llcd);
   s_dram_drain = llcd;
   /* overlap: the DRAM loads' latency over the time at least one is outstanding (memory-level
    * parallelism), and for each dependent one, the others outstanding beside it */
   ULong busy = 0, lat_sum = 0, dep_n = 0, dep_others = 0;
   if (iv_n > 0) {
      VG_(ssort)(iv, iv_n, sizeof(Interval), by_start);
      ULong cs = iv[0].s, ce = iv[0].e;
      for (ULong i = 0; i < iv_n; i++) {
         lat_sum += iv[i].e - iv[i].s;
         if (iv[i].s > ce) {
            busy += ce - cs;
            cs = iv[i].s;
            ce = iv[i].e;
         } else if (iv[i].e > ce) {
            ce = iv[i].e;
         }
      }
      busy += ce - cs;
      ULong *starts = VG_(malloc)("cg.s", iv_n * sizeof(ULong));
      ULong *ends = VG_(malloc)("cg.e", iv_n * sizeof(ULong));
      for (ULong i = 0; i < iv_n; i++) {
         starts[i] = iv[i].s;
         ends[i] = iv[i].e;
      }
      VG_(ssort)(ends, iv_n, sizeof(ULong), by_value);
      for (ULong i = 0; i < iv_n; i++) {
         if (!iv[i].dep)
            continue;
         /* intervals with start < e and end > s, itself excluded */
         ULong started = below(starts, iv_n, iv[i].e);
         ULong ended = below(ends, iv_n, iv[i].s + 1);
         dep_n++;
         dep_others += started - ended - 1;
      }
      VG_(free)(starts);
      VG_(free)(ends);
   }
   VG_(umsg)("MOONSHOT-PROXY {\"out_bytes\":%llu,\"instructions\":%llu,\"cycles\":%llu,"
             "\"loads\":%llu,\"stores\":%llu,\"l1_miss\":%llu,\"l2_miss\":%llu,\"llc_miss\":%llu,"
             "\"l2_transfers\":%llu,\"l3_transfers\":%llu,\"dram_read_lines\":%llu,"
             "\"dram_write_lines\":%llu,\"dram_drain_lines\":%llu,\"prefetched_lines\":%llu,"
             "\"past_l2_loads\":%llu,\"dram_loads\":%llu,\"dep_past_l2_loads\":%llu,"
             "\"dep_dram_loads\":%llu,\"dep_any_loads\":%llu,\"max_chain_past_l2\":%llu,\"max_chain_dram\":%llu,"
             "\"chain_sum_dram\":%llu,\"dram_latency_sum\":%llu,\"dram_busy_cycles\":%llu,"
             "\"dep_overlap_sum\":%llu,\"dep_overlap_n\":%llu,\"store_buffer_stall_cycles\":%llu,"
             "\"config\":{\"llc_kb\":%lld,\"llc_ways\":%lld,\"l1_lat\":%lld,\"l2_lat\":%lld,"
             "\"l3_lat\":%lld,\"dram_lat\":%lld,\"rob\":%lld,\"width\":%lld,\"mshr\":%lld,\"sb\":%lld,"
             "\"prefetch\":%s}}\n",
             s_out_bytes, n_instr, s_cycles, s_loads, s_stores, s_l1_miss, s_l2_miss, s_llc_miss,
             s_l2_xfer, s_l3_xfer, s_dram_read, s_dram_write, s_dram_drain, s_prefetched,
             s_past_l2_loads, s_dram_loads, s_dep_past_l2, s_dep_dram, s_dep_any, s_max_chain_past_l2,
             s_max_chain_dram, s_chain_sum_dram, lat_sum, busy, dep_others, dep_n, s_sb_stall,
             clo_llc_kb, clo_llc_ways, clo_l1_lat, clo_l2_lat, clo_l3_lat, clo_dram_lat, clo_rob,
             clo_width, clo_mshr, clo_sb, clo_prefetch ? "true" : "false");
}

static void cg_pre_clo_init(void)
{
   VG_(details_name)("chaingrind");
   VG_(details_version)("0.1");
   VG_(details_description)("the moonshot scorer's cache and dependent-chain model");
   VG_(details_copyright_author)("mibz2's authors (MIT OR Apache-2.0)");
   VG_(details_bug_reports_to)("https://github.com/lmmx/mibz2");
   VG_(details_avg_translation_sizeB)(640);
   VG_(basic_tool_funcs)(cg_post_clo_init, cg_instrument, cg_fini);
   VG_(needs_command_line_options)(cg_option, cg_usage, cg_debug_usage);
   VG_(needs_client_requests)(cg_client_request);
}

VG_DETERMINE_INTERFACE_VERSION(cg_pre_clo_init)
