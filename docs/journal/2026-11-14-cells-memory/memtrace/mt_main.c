/* memtrace: a minimal valgrind tool for the Cells memory census.

   Every guest load and store (plain, guarded, CAS, LL/SC and dirty helpers' declared memory
   effects) calls a helper with its address and size. While tracing is on (client request
   0 turns it on, 1 off), the helper appends one 64-bit record to a buffer that is written
   to the file named by --out= (a FIFO read by consumer.rs):

     bits 0..47   address
     bits 48..61  size in bytes
     bits 62..63  0 load, 1 store, 2 marker

   Client request 2 (a marker) appends (2 << 62 | code) and four payload words, in order
   with the accesses around it. A read-modify-write instruction is one load and one store.
   Nothing is aggregated here: the consumer does the counting.

   Build: build.sh. Run: valgrind --tool=memtrace --out=FIFO PROGRAM ... with VALGRIND_LIB
   pointing at a directory holding this tool beside the system's valgrind files. */

#include "pub_tool_basics.h"
#include "pub_tool_tooliface.h"
#include "pub_tool_libcassert.h"
#include "pub_tool_libcprint.h"
#include "pub_tool_libcfile.h"
#include "pub_tool_libcbase.h"
#include "pub_tool_options.h"
#include "pub_tool_machine.h"
#include "pub_tool_vki.h"
#include "valgrind.h"

static const HChar* clo_out = NULL;

#define BUFN (1 << 16)
static ULong obuf[BUFN];
static UInt onb = 0;
static Int ofd = -1;
static Bool on = False;

static void oflush(void)
{
   const HChar* p = (const HChar*)obuf;
   Int want = (Int)(onb * 8);
   while (want > 0) {
      Int w = VG_(write)(ofd, p, want);
      if (w <= 0)
         VG_(tool_panic)("memtrace: write to --out failed");
      p += w;
      want -= w;
   }
   onb = 0;
}

static inline void oput(ULong x)
{
   obuf[onb++] = x;
   if (onb == BUFN)
      oflush();
}

#define ADDR_MASK 0xFFFFFFFFFFFFULL

static VG_REGPARM(2) void mt_load(Addr a, SizeT sz)
{
   if (on)
      oput(((ULong)a & ADDR_MASK) | ((ULong)(sz & 0x3FFF) << 48));
}

static VG_REGPARM(2) void mt_store(Addr a, SizeT sz)
{
   if (on)
      oput(((ULong)a & ADDR_MASK) | ((ULong)(sz & 0x3FFF) << 48) | (1ULL << 62));
}

static void add_event(IRSB* sb, IRExpr* addr, Int size, Bool store, IRExpr* guard)
{
   IRExpr** args = mkIRExprVec_2(addr, mkIRExpr_HWord((HWord)size));
   IRDirty* di = store
      ? unsafeIRDirty_0_N(2, "mt_store", VG_(fnptr_to_fnentry)(mt_store), args)
      : unsafeIRDirty_0_N(2, "mt_load", VG_(fnptr_to_fnentry)(mt_load), args);
   if (guard)
      di->guard = guard;
   addStmtToIRSB(sb, IRStmt_Dirty(di));
}

static IRSB* mt_instrument(VgCallbackClosure* closure, IRSB* sbIn,
                           const VexGuestLayout* layout, const VexGuestExtents* vge,
                           const VexArchInfo* archinfo_host, IRType gWordTy, IRType hWordTy)
{
   IRSB* sbOut;
   Int i;
   IRTypeEnv* tyenv = sbIn->tyenv;
   if (gWordTy != hWordTy)
      VG_(tool_panic)("memtrace: host/guest word size mismatch");
   sbOut = deepCopyIRSBExceptStmts(sbIn);
   i = 0;
   while (i < sbIn->stmts_used && sbIn->stmts[i]->tag != Ist_IMark) {
      addStmtToIRSB(sbOut, sbIn->stmts[i]);
      i++;
   }
   for (; i < sbIn->stmts_used; i++) {
      IRStmt* st = sbIn->stmts[i];
      if (!st || st->tag == Ist_NoOp)
         continue;
      switch (st->tag) {
      case Ist_WrTmp: {
         IRExpr* data = st->Ist.WrTmp.data;
         if (data->tag == Iex_Load)
            add_event(sbOut, data->Iex.Load.addr, sizeofIRType(data->Iex.Load.ty), False, NULL);
         break;
      }
      case Ist_Store: {
         IRExpr* data = st->Ist.Store.data;
         add_event(sbOut, st->Ist.Store.addr, sizeofIRType(typeOfIRExpr(tyenv, data)), True,
                   NULL);
         break;
      }
      case Ist_StoreG: {
         IRStoreG* sg = st->Ist.StoreG.details;
         add_event(sbOut, sg->addr, sizeofIRType(typeOfIRExpr(tyenv, sg->data)), True,
                   sg->guard);
         break;
      }
      case Ist_LoadG: {
         IRLoadG* lg = st->Ist.LoadG.details;
         IRType type = Ity_INVALID, wide = Ity_INVALID;
         typeOfIRLoadGOp(lg->cvt, &wide, &type);
         add_event(sbOut, lg->addr, sizeofIRType(type), False, lg->guard);
         break;
      }
      case Ist_Dirty: {
         IRDirty* d = st->Ist.Dirty.details;
         if (d->mFx != Ifx_None) {
            if (d->mFx == Ifx_Read || d->mFx == Ifx_Modify)
               add_event(sbOut, d->mAddr, d->mSize, False, d->guard);
            if (d->mFx == Ifx_Write || d->mFx == Ifx_Modify)
               add_event(sbOut, d->mAddr, d->mSize, True, d->guard);
         }
         break;
      }
      case Ist_CAS: {
         IRCAS* cas = st->Ist.CAS.details;
         Int size = sizeofIRType(typeOfIRExpr(tyenv, cas->dataLo));
         if (cas->dataHi != NULL)
            size *= 2;
         add_event(sbOut, cas->addr, size, False, NULL);
         add_event(sbOut, cas->addr, size, True, NULL);
         break;
      }
      case Ist_LLSC: {
         if (st->Ist.LLSC.storedata == NULL)
            add_event(sbOut, st->Ist.LLSC.addr,
                      sizeofIRType(typeOfIRTemp(tyenv, st->Ist.LLSC.result)), False, NULL);
         else
            add_event(sbOut, st->Ist.LLSC.addr,
                      sizeofIRType(typeOfIRExpr(tyenv, st->Ist.LLSC.storedata)), True, NULL);
         break;
      }
      default:
         break;
      }
      addStmtToIRSB(sbOut, st);
   }
   return sbOut;
}

static Bool mt_client_request(ThreadId tid, UWord* arg, UWord* ret)
{
   if (!VG_IS_TOOL_USERREQ('M', 'T', arg[0]))
      return False;
   switch (arg[0] - VG_USERREQ_TOOL_BASE('M', 'T')) {
   case 0:
      on = True;
      break;
   case 1:
      on = False;
      break;
   case 2:
      oput((2ULL << 62) | (ULong)(arg[1] & 0xFFFFFFFFULL));
      oput(arg[2]);
      oput(arg[3]);
      oput(arg[4]);
      oput(arg[5]);
      break;
   default:
      return False;
   }
   *ret = 1;
   return True;
}

static Bool mt_process_cmd_line_option(const HChar* arg)
{
   if VG_STR_CLO(arg, "--out", clo_out) {}
   else
      return False;
   return True;
}

static void mt_print_usage(void)
{
   VG_(printf)("    --out=<file>   where the binary records go (a FIFO)\n");
}

static void mt_print_debug_usage(void)
{
   VG_(printf)("    (none)\n");
}

static void mt_post_clo_init(void)
{
   SysRes sr;
   if (clo_out == NULL)
      VG_(tool_panic)("memtrace: --out=<file> is required");
   sr = VG_(open)(clo_out, VKI_O_WRONLY | VKI_O_CREAT | VKI_O_TRUNC, 0644);
   if (sr_isError(sr))
      VG_(tool_panic)("memtrace: cannot open --out");
   ofd = (Int)sr_Res(sr);
}

static void mt_fini(Int exitcode)
{
   oflush();
   VG_(close)(ofd);
}

static void mt_pre_clo_init(void)
{
   VG_(details_name)("memtrace");
   VG_(details_version)(NULL);
   VG_(details_description)("address trace for the mibz2 Cells memory census");
   VG_(details_copyright_author)("mibz2 research");
   VG_(details_bug_reports_to)("none");
   VG_(details_avg_translation_sizeB)(275);
   VG_(basic_tool_funcs)(mt_post_clo_init, mt_instrument, mt_fini);
   VG_(needs_command_line_options)(mt_process_cmd_line_option, mt_print_usage,
                                   mt_print_debug_usage);
   VG_(needs_client_requests)(mt_client_request);
}

VG_DETERMINE_INTERFACE_VERSION(mt_pre_clo_init)
