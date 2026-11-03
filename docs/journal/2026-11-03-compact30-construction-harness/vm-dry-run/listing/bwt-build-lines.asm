
./target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

0000000000088570 <mibz2::bwt::build>:
mibz2::bwt::build:
   88570:	push   %rbp
./src/bwt.rs:241
   88571:	push   %r15
   88573:	push   %r14
   88575:	push   %r13
   88577:	push   %r12
   88579:	push   %rbx
   8857a:	sub    $0x808,%rsp
   88581:	mov    %rdx,%r15
   88584:	mov    %rsi,%rbx
   88587:	mov    %rdi,%r14
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/mod.rs:3035
   8858a:	movq   $0x0,0x10(%rcx)
./src/bwt.rs:256
   88592:	cmp    $0xffffff,%rsi
build_one_pass():
./src/bwt.rs:256
   88599:	ja     88716 <mibz2::bwt::build+0x1a6>
   8859f:	pxor   %xmm0,%xmm0
   885a3:	xor    %eax,%eax
   885a5:	pxor   %xmm2,%xmm2
   885a9:	pxor   %xmm1,%xmm1
   885ad:	nopl   (%rax)
mibz2::bwt::build:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/iter/macros.rs:279
   885b0:	movq   (%r15,%rax,4),%xmm3
fold<u32, usize, core::iter::adapters::map::map_fold::{closure_env#0}<&u32, usize, usize, mibz2::bwt::build_one_pass::{closure_env#0}, core::iter::traits::accum::{impl#48}::sum::{closure_env#0}<core::iter::adapters::map::Map<core::slice::iter::Iter<u32>, mibz2::bwt::build_one_pass::{closure_env#0}>>>>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/iter/macros.rs:279
   885b6:	movq   0x8(%r15,%rax,4),%xmm4
mibz2::bwt::build:
./src/bwt.rs:256
   885bd:	punpckldq %xmm0,%xmm3
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/traits/accum.rs:55
   885c1:	paddq  %xmm2,%xmm3
{closure#0}():
./src/bwt.rs:256
   885c5:	punpckldq %xmm0,%xmm4
{closure#0}<core::iter::adapters::map::Map<core::slice::iter::Iter<u32>, mibz2::bwt::build_one_pass::{closure_env#0}>>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/traits/accum.rs:55
   885c9:	paddq  %xmm1,%xmm4
fold<u32, usize, core::iter::adapters::map::map_fold::{closure_env#0}<&u32, usize, usize, mibz2::bwt::build_one_pass::{closure_env#0}, core::iter::traits::accum::{impl#48}::sum::{closure_env#0}<core::iter::adapters::map::Map<core::slice::iter::Iter<u32>, mibz2::bwt::build_one_pass::{closure_env#0}>>>>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/iter/macros.rs:279
   885cd:	movq   0x10(%r15,%rax,4),%xmm2
   885d4:	movq   0x18(%r15,%rax,4),%xmm1
{closure#0}():
./src/bwt.rs:256
   885db:	punpckldq %xmm0,%xmm2
{closure#0}<core::iter::adapters::map::Map<core::slice::iter::Iter<u32>, mibz2::bwt::build_one_pass::{closure_env#0}>>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/traits/accum.rs:55
   885df:	paddq  %xmm3,%xmm2
{closure#0}():
./src/bwt.rs:256
   885e3:	punpckldq %xmm0,%xmm1
{closure#0}<core::iter::adapters::map::Map<core::slice::iter::Iter<u32>, mibz2::bwt::build_one_pass::{closure_env#0}>>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/traits/accum.rs:55
   885e7:	paddq  %xmm4,%xmm1
fold<u32, usize, core::iter::adapters::map::map_fold::{closure_env#0}<&u32, usize, usize, mibz2::bwt::build_one_pass::{closure_env#0}, core::iter::traits::accum::{impl#48}::sum::{closure_env#0}<core::iter::adapters::map::Map<core::slice::iter::Iter<u32>, mibz2::bwt::build_one_pass::{closure_env#0}>>>>():
   885eb:	add    $0x8,%rax
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/iter/macros.rs:284
   885ef:	cmp    $0x100,%rax
   885f5:	jne    885b0 <mibz2::bwt::build+0x40>
   885f7:	paddq  %xmm2,%xmm1
   885fb:	pshufd $0xee,%xmm1,%xmm0
   88600:	paddq  %xmm1,%xmm0
   88604:	movq   %xmm0,%rax
build_one_pass():
./src/bwt.rs:256
   88609:	cmp    %rbx,%rax
   8860c:	jne    88716 <mibz2::bwt::build+0x1a6>
   88612:	mov    %rcx,%r12
   88615:	lea    0x8(%rsp),%rdi
   8861a:	xor    %ebp,%ebp
   8861c:	mov    0x4de2d(%rip),%r13        # d6450 <memset@GLIBC_2.2.5>
./src/bwt.rs:259
   88623:	mov    $0x400,%edx
   88628:	xor    %esi,%esi
   8862a:	call   *%r13
   8862d:	lea    0x408(%rsp),%rdi
   88635:	mov    $0x400,%edx
   8863a:	xor    %esi,%esi
   8863c:	call   *%r13
   8863f:	mov    $0x1,%eax
   88644:	data16 data16 cs nopw 0x0(%rax,%rax,1)
./src/bwt.rs:262
   88650:	mov    %ebp,0x404(%rsp,%rax,4)
./src/bwt.rs:263
   88657:	add    -0x4(%r15,%rax,4),%ebp
./src/bwt.rs:264
   8865c:	mov    %ebp,0x4(%rsp,%rax,4)
./src/bwt.rs:262
   88660:	mov    %ebp,0x408(%rsp,%rax,4)
./src/bwt.rs:263
   88667:	add    (%r15,%rax,4),%ebp
./src/bwt.rs:264
   8866b:	mov    %ebp,0x8(%rsp,%rax,4)
mibz2::bwt::build:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/cmp.rs:1916
   8866f:	add    $0x2,%rax
lt():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/cmp.rs:1916
   88673:	cmp    $0x101,%rax
mibz2::bwt::build:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/range.rs:900
   88679:	jne    88650 <mibz2::bwt::build+0xe0>
build_one_pass():
   8867b:	mov    %r12,%rcx
mibz2::bwt::build:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/raw_vec/mod.rs:767
   8867e:	cmp    (%r12),%rbx
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/raw_vec/mod.rs:673
   88682:	ja     887d6 <mibz2::bwt::build+0x266>
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/raw_vec/mod.rs:614
   88688:	mov    0x8(%rcx),%rdx
non_null<alloc::alloc::Global, u32>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/raw_vec/mod.rs:614
   8868c:	xor    %r8d,%r8d
build_one_pass():
./src/bwt.rs:282
   8868f:	cmp    $0x10,%rbx
   88693:	jae    88802 <mibz2::bwt::build+0x292>
mibz2::bwt::build:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/cmp.rs:1916
   88699:	cmp    %rbx,%r8
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/range.rs:900
   8869c:	jae    886f7 <mibz2::bwt::build+0x187>
./src/bwt.rs:272
   8869e:	mov    %r8d,%eax
{closure#1}():
./src/bwt.rs:272
   886a1:	shl    $0x8,%eax
   886a4:	data16 data16 cs nopw 0x0(%rax,%rax,1)
./src/bwt.rs:270
   886b0:	movzbl (%r14,%r8,1),%esi
./src/bwt.rs:271
   886b5:	lea    0x0(,%rsi,4),%r9d
   886bd:	mov    0x408(%rsp,%r9,1),%edi
./src/bwt.rs:272
   886c5:	cmp    0x8(%rsp,%r9,1),%edi
   886ca:	jae    88716 <mibz2::bwt::build+0x1a6>
./src/bwt.rs:276
   886cc:	cmp    %rdi,%rbx
   886cf:	jbe    88e38 <mibz2::bwt::build+0x8c8>
   886d5:	movzbl (%r14,%rdi,1),%r9d
   886da:	add    %eax,%r9d
   886dd:	mov    %r9d,(%rdx,%rdi,4)
./src/bwt.rs:277
   886e1:	inc    %edi
   886e3:	mov    %edi,0x408(%rsp,%rsi,4)
mibz2::bwt::build:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/num/uint_macros.rs:971
   886ea:	inc    %r8
lt():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/cmp.rs:1916
   886ed:	add    $0x100,%eax
   886f2:	cmp    %r8,%rbx
spec_next<usize>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/range.rs:900
   886f5:	jne    886b0 <mibz2::bwt::build+0x140>
build_one_pass():
   886f7:	lea    0x408(%rsp),%rdi
   886ff:	lea    0x8(%rsp),%rsi
mibz2::bwt::build:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/array/equality.rs:157
   88704:	mov    $0x400,%edx
spec_eq<u32, u32, 256>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/array/equality.rs:157
   88709:	call   *0x4dd69(%rip)        # d6478 <bcmp@GLIBC_2.2.5>
   8870f:	mov    %r12,%rcx
   88712:	test   %eax,%eax
build_one_pass():
./src/bwt.rs:310
   88714:	je     88740 <mibz2::bwt::build+0x1d0>
mibz2::bwt::build:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/mod.rs:3035
   88716:	movq   $0x0,0x10(%rcx)
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/raw_vec/mod.rs:767
   8871e:	cmp    (%rcx),%rbx
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/raw_vec/mod.rs:673
   88721:	ja     88749 <mibz2::bwt::build+0x1d9>
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/raw_vec/mod.rs:614
   88723:	mov    0x8(%rcx),%r13
non_null<alloc::alloc::Global, u32>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/raw_vec/mod.rs:614
   88727:	xor    %r12d,%r12d
mibz2::bwt::build:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/non_null.rs:1714
   8872a:	test   %rbx,%rbx
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/iter/macros.rs:25
   8872d:	je     88e9a <mibz2::bwt::build+0x92a>
fold<u8, (), core::iter::adapters::map::map_fold::{closure_env#0}<&u8, u32, (), mibz2::bwt::build_two_pass::{closure_env#0}, core::iter::traits::iterator::Iterator::for_each::call::{closure_env#0}<u32, alloc::vec::{impl#21}::extend_trusted::{closure_env#0}<u32, alloc::alloc::Global, core::iter::adapters::map::Map<core::slice::iter::Iter<u8>, mibz2::bwt::build_two_pass::{closure_env#0}>>>>>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/iter/macros.rs:284
   88733:	cmp    $0x8,%rbx
   88737:	jae    88777 <mibz2::bwt::build+0x207>
mibz2::bwt::build:
   88739:	xor    %eax,%eax
   8873b:	jmp    88e85 <mibz2::bwt::build+0x915>
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/mod.rs:2221
   88740:	mov    %rbx,0x10(%rcx)
set_len<u32, alloc::alloc::Global>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/mod.rs:2221
   88744:	jmp    88e73 <mibz2::bwt::build+0x903>
mibz2::bwt::build:
   88749:	mov    %rcx,%r12
reserve<alloc::alloc::Global>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/raw_vec/mod.rs:675
   8874c:	mov    $0x4,%ecx
   88751:	mov    $0x4,%r8d
   88757:	mov    %r12,%rdi
   8875a:	xor    %esi,%esi
   8875c:	mov    %rbx,%rdx
   8875f:	call   876b0 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   88764:	mov    %r12,%rcx
non_null<alloc::alloc::Global, u32>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/raw_vec/mod.rs:614
   88767:	mov    0x8(%r12),%r13
mibz2::bwt::build:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/set_len_on_drop.rs:14
   8876c:	mov    0x10(%r12),%r12
fold<u8, (), core::iter::adapters::map::map_fold::{closure_env#0}<&u8, u32, (), mibz2::bwt::build_two_pass::{closure_env#0}, core::iter::traits::iterator::Iterator::for_each::call::{closure_env#0}<u32, alloc::vec::{impl#21}::extend_trusted::{closure_env#0}<u32, alloc::alloc::Global, core::iter::adapters::map::Map<core::slice::iter::Iter<u8>, mibz2::bwt::build_two_pass::{closure_env#0}>>>>>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/iter/macros.rs:284
   88771:	cmp    $0x8,%rbx
   88775:	jb     88739 <mibz2::bwt::build+0x1c9>
   88777:	movabs $0x7ffffffffffffff0,%rax
   88781:	or     $0x8,%rax
   88785:	and    %rbx,%rax
   88788:	lea    0x10(,%r12,4),%rdx
   88790:	add    %r13,%rdx
   88793:	add    %rax,%r12
   88796:	xor    %esi,%esi
   88798:	pxor   %xmm0,%xmm0
   8879c:	nopl   0x0(%rax)
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/iter/macros.rs:279
   887a0:	movd   (%r14,%rsi,1),%xmm1
   887a6:	movd   0x4(%r14,%rsi,1),%xmm2
mibz2::bwt::build:
./src/bwt.rs:325
   887ad:	punpcklbw %xmm0,%xmm1
{closure#0}():
./src/bwt.rs:325
   887b1:	punpcklwd %xmm0,%xmm1
   887b5:	punpcklbw %xmm0,%xmm2
   887b9:	punpcklwd %xmm0,%xmm2
mibz2::bwt::build:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1933
   887bd:	movdqu %xmm1,-0x10(%rdx,%rsi,4)
write<u32>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1933
   887c3:	movdqu %xmm2,(%rdx,%rsi,4)
fold<u8, (), core::iter::adapters::map::map_fold::{closure_env#0}<&u8, u32, (), mibz2::bwt::build_two_pass::{closure_env#0}, core::iter::traits::iterator::Iterator::for_each::call::{closure_env#0}<u32, alloc::vec::{impl#21}::extend_trusted::{closure_env#0}<u32, alloc::alloc::Global, core::iter::adapters::map::Map<core::slice::iter::Iter<u8>, mibz2::bwt::build_two_pass::{closure_env#0}>>>>>():
   887c8:	add    $0x8,%rsi
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/iter/macros.rs:284
   887cc:	cmp    %rsi,%rax
   887cf:	jne    887a0 <mibz2::bwt::build+0x230>
   887d1:	jmp    88e95 <mibz2::bwt::build+0x925>
reserve<alloc::alloc::Global>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/raw_vec/mod.rs:675
   887d6:	mov    $0x4,%ecx
   887db:	mov    $0x4,%r8d
   887e1:	mov    %r12,%rdi
   887e4:	xor    %esi,%esi
   887e6:	mov    %rbx,%rdx
   887e9:	call   876b0 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   887ee:	mov    %r12,%rcx
non_null<alloc::alloc::Global, u32>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/raw_vec/mod.rs:614
   887f1:	mov    0x8(%rcx),%rdx
   887f5:	xor    %r8d,%r8d
build_one_pass():
./src/bwt.rs:282
   887f8:	cmp    $0x10,%rbx
   887fc:	jb     88699 <mibz2::bwt::build+0x129>
   88802:	mov    $0x8,%esi
   88807:	pxor   %xmm0,%xmm0
   8880b:	movdqa -0x7dfd3(%rip),%xmm1        # a840 <GCC_except_table819+0x74>
   88813:	movdqa -0x7db1b(%rip),%xmm2        # ad00 <GCC_except_table819+0x534>
   8881b:	movdqa -0x7d713(%rip),%xmm3        # b110 <GCC_except_table819+0x944>
   88823:	jmp    8884b <mibz2::bwt::build+0x2db>
   88825:	data16 cs nopw 0x0(%rax,%rax,1)
   88830:	lea    0x10(%rsi),%rax
   88834:	add    $0x18,%rsi
   88838:	add    $0x1000,%r8d
   8883f:	cmp    %rbx,%rsi
   88842:	mov    %rax,%rsi
   88845:	ja     88e07 <mibz2::bwt::build+0x897>
mibz2::bwt::build:
./src/bwt.rs:370
   8884b:	lea    -0x8(%rsi),%rax
read_u64():
./src/bwt.rs:370
   8884f:	cmp    %rbx,%rsi
mibz2::bwt::build:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/num/uint_macros.rs:1050
   88852:	ja     88e47 <mibz2::bwt::build+0x8d7>
   88858:	lea    0x7(%rsi),%rdi
checked_sub():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/num/uint_macros.rs:1050
   8885c:	cmp    %rbx,%rdi
   8885f:	jae    88e1c <mibz2::bwt::build+0x8ac>
mibz2::bwt::build:
./src/bwt.rs:370
   88865:	mov    -0x8(%r14,%rsi,1),%r9
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/result.rs:1718
   8886a:	mov    (%r14,%rsi,1),%rax
build_one_pass():
./src/bwt.rs:284
   8886e:	mov    %r9,%rdi
   88871:	xor    %rax,%rdi
   88874:	mov    %r9,%r10
   88877:	rol    $0x8,%r10
   8887b:	xor    %r9,%r10
   8887e:	or     %rdi,%r10
   88881:	jne    88970 <mibz2::bwt::build+0x400>
./src/bwt.rs:285
   88887:	movzbl %r9b,%r9d
./src/bwt.rs:286
   8888b:	lea    0x0(,%r9,4),%r10d
   88893:	mov    0x408(%rsp,%r10,1),%edi
./src/bwt.rs:287
   8889b:	lea    0x10(%rdi),%rax
   8889f:	mov    0x8(%rsp,%r10,1),%r10d
   888a4:	cmp    %r10,%rax
   888a7:	ja     88716 <mibz2::bwt::build+0x1a6>
mibz2::bwt::build:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/index.rs:438
   888ad:	cmp    %rbx,%rax
index<u8>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/index.rs:438
   888b0:	ja     88e26 <mibz2::bwt::build+0x8b6>
build_one_pass():
./src/bwt.rs:291
   888b6:	movzbl (%r14,%rdi,1),%eax
./src/bwt.rs:293
   888bb:	add    %r8d,%eax
   888be:	mov    %eax,(%rdx,%rdi,4)
./src/bwt.rs:291
   888c1:	movd   0x1(%r14,%rdi,1),%xmm4
./src/bwt.rs:293
   888c8:	punpcklbw %xmm0,%xmm4
   888cc:	punpcklwd %xmm0,%xmm4
   888d0:	movd   %r8d,%xmm5
   888d5:	pshufd $0x0,%xmm5,%xmm5
   888da:	por    %xmm5,%xmm4
   888de:	por    %xmm1,%xmm4
   888e2:	movdqu %xmm4,0x4(%rdx,%rdi,4)
./src/bwt.rs:291
   888e8:	movd   0x5(%r14,%rdi,1),%xmm4
./src/bwt.rs:293
   888ef:	punpcklbw %xmm0,%xmm4
   888f3:	punpcklwd %xmm0,%xmm4
   888f7:	por    %xmm5,%xmm4
   888fb:	por    %xmm2,%xmm4
   888ff:	movdqu %xmm4,0x14(%rdx,%rdi,4)
./src/bwt.rs:291
   88905:	movd   0x9(%r14,%rdi,1),%xmm4
./src/bwt.rs:293
   8890c:	punpcklbw %xmm0,%xmm4
   88910:	punpcklwd %xmm0,%xmm4
   88914:	por    %xmm5,%xmm4
   88918:	por    %xmm3,%xmm4
   8891c:	movdqu %xmm4,0x24(%rdx,%rdi,4)
./src/bwt.rs:291
   88922:	movzbl 0xd(%r14,%rdi,1),%eax
./src/bwt.rs:293
   88928:	lea    0xd00(%r8,%rax,1),%eax
   88930:	mov    %eax,0x34(%rdx,%rdi,4)
./src/bwt.rs:291
   88934:	movzbl 0xe(%r14,%rdi,1),%eax
./src/bwt.rs:293
   8893a:	lea    0xe00(%r8,%rax,1),%eax
   88942:	mov    %eax,0x38(%rdx,%rdi,4)
./src/bwt.rs:291
   88946:	movzbl 0xf(%r14,%rdi,1),%eax
./src/bwt.rs:293
   8894c:	lea    0xf00(%r8,%rax,1),%eax
   88954:	mov    %eax,0x3c(%rdx,%rdi,4)
./src/bwt.rs:295
   88958:	lea    0x10(%rdi),%eax
   8895b:	mov    %eax,0x408(%rsp,%r9,4)
./src/bwt.rs:284
   88963:	jmp    88830 <mibz2::bwt::build+0x2c0>
   88968:	nopl   0x0(%rax,%rax,1)
mibz2::bwt::build:
./src/bwt.rs:270
   88970:	movzbl %r9b,%r10d
{closure#1}():
./src/bwt.rs:271
   88974:	lea    0x0(,%r10,4),%r11d
   8897c:	mov    0x408(%rsp,%r11,1),%edi
./src/bwt.rs:272
   88984:	cmp    0x8(%rsp,%r11,1),%edi
   88989:	jae    88716 <mibz2::bwt::build+0x1a6>
./src/bwt.rs:276
   8898f:	cmp    %rdi,%rbx
   88992:	jbe    88e38 <mibz2::bwt::build+0x8c8>
   88998:	mov    %r9,%r11
   8899b:	shr    $0x8,%r11
   8899f:	movzbl (%r14,%rdi,1),%ebp
   889a4:	add    %r8d,%ebp
   889a7:	mov    %ebp,(%rdx,%rdi,4)
./src/bwt.rs:277
   889aa:	inc    %edi
   889ac:	mov    %edi,0x408(%rsp,%r10,4)
./src/bwt.rs:270
   889b4:	movzbl %r11b,%r10d
./src/bwt.rs:271
   889b8:	lea    0x0(,%r10,4),%r11d
   889c0:	mov    0x408(%rsp,%r11,1),%edi
./src/bwt.rs:272
   889c8:	cmp    0x8(%rsp,%r11,1),%edi
   889cd:	jae    88716 <mibz2::bwt::build+0x1a6>
./src/bwt.rs:276
   889d3:	cmp    %rdi,%rbx
   889d6:	jbe    88e38 <mibz2::bwt::build+0x8c8>
   889dc:	mov    %r9,%r11
   889df:	shr    $0x10,%r11
   889e3:	movzbl (%r14,%rdi,1),%r13d
   889e8:	lea    (%r8,%r13,1),%ebp
   889ec:	add    $0x100,%ebp
   889f2:	mov    %ebp,(%rdx,%rdi,4)
./src/bwt.rs:277
   889f5:	inc    %edi
   889f7:	mov    %edi,0x408(%rsp,%r10,4)
./src/bwt.rs:270
   889ff:	movzbl %r11b,%r10d
./src/bwt.rs:271
   88a03:	lea    0x0(,%r10,4),%r11d
   88a0b:	mov    0x408(%rsp,%r11,1),%edi
./src/bwt.rs:272
   88a13:	cmp    0x8(%rsp,%r11,1),%edi
   88a18:	jae    88716 <mibz2::bwt::build+0x1a6>
./src/bwt.rs:276
   88a1e:	cmp    %rdi,%rbx
   88a21:	jbe    88e38 <mibz2::bwt::build+0x8c8>
   88a27:	mov    %r9,%r11
   88a2a:	shr    $0x18,%r11
   88a2e:	movzbl (%r14,%rdi,1),%r13d
   88a33:	lea    (%r8,%r13,1),%ebp
   88a37:	add    $0x200,%ebp
   88a3d:	mov    %ebp,(%rdx,%rdi,4)
./src/bwt.rs:277
   88a40:	inc    %edi
   88a42:	mov    %edi,0x408(%rsp,%r10,4)
./src/bwt.rs:270
   88a4a:	movzbl %r11b,%r10d
./src/bwt.rs:271
   88a4e:	lea    0x0(,%r10,4),%r11d
   88a56:	mov    0x408(%rsp,%r11,1),%edi
./src/bwt.rs:272
   88a5e:	cmp    0x8(%rsp,%r11,1),%edi
   88a63:	jae    88716 <mibz2::bwt::build+0x1a6>
./src/bwt.rs:276
   88a69:	cmp    %rdi,%rbx
   88a6c:	jbe    88e38 <mibz2::bwt::build+0x8c8>
   88a72:	mov    %r9,%r11
   88a75:	shr    $0x20,%r11
   88a79:	movzbl (%r14,%rdi,1),%r13d
   88a7e:	lea    (%r8,%r13,1),%ebp
   88a82:	add    $0x300,%ebp
   88a88:	mov    %ebp,(%rdx,%rdi,4)
./src/bwt.rs:277
   88a8b:	inc    %edi
   88a8d:	mov    %edi,0x408(%rsp,%r10,4)
./src/bwt.rs:270
   88a95:	movzbl %r11b,%r10d
./src/bwt.rs:271
   88a99:	lea    0x0(,%r10,4),%r11d
   88aa1:	mov    0x408(%rsp,%r11,1),%edi
./src/bwt.rs:272
   88aa9:	cmp    0x8(%rsp,%r11,1),%edi
   88aae:	jae    88716 <mibz2::bwt::build+0x1a6>
./src/bwt.rs:276
   88ab4:	cmp    %rdi,%rbx
   88ab7:	jbe    88e38 <mibz2::bwt::build+0x8c8>
   88abd:	mov    %r9,%r11
   88ac0:	shr    $0x28,%r11
   88ac4:	movzbl (%r14,%rdi,1),%r13d
   88ac9:	lea    (%r8,%r13,1),%ebp
   88acd:	add    $0x400,%ebp
   88ad3:	mov    %ebp,(%rdx,%rdi,4)
./src/bwt.rs:277
   88ad6:	inc    %edi
   88ad8:	mov    %edi,0x408(%rsp,%r10,4)
./src/bwt.rs:270
   88ae0:	movzbl %r11b,%r10d
./src/bwt.rs:271
   88ae4:	lea    0x0(,%r10,4),%r11d
   88aec:	mov    0x408(%rsp,%r11,1),%edi
./src/bwt.rs:272
   88af4:	cmp    0x8(%rsp,%r11,1),%edi
   88af9:	jae    88716 <mibz2::bwt::build+0x1a6>
./src/bwt.rs:276
   88aff:	cmp    %rdi,%rbx
   88b02:	jbe    88e38 <mibz2::bwt::build+0x8c8>
   88b08:	mov    %r9,%r11
   88b0b:	shr    $0x30,%r11
   88b0f:	movzbl (%r14,%rdi,1),%r13d
   88b14:	lea    (%r8,%r13,1),%ebp
   88b18:	add    $0x500,%ebp
   88b1e:	mov    %ebp,(%rdx,%rdi,4)
./src/bwt.rs:277
   88b21:	inc    %edi
   88b23:	mov    %edi,0x408(%rsp,%r10,4)
./src/bwt.rs:270
   88b2b:	movzbl %r11b,%r10d
./src/bwt.rs:271
   88b2f:	lea    0x0(,%r10,4),%r11d
   88b37:	mov    0x408(%rsp,%r11,1),%edi
./src/bwt.rs:272
   88b3f:	cmp    0x8(%rsp,%r11,1),%edi
   88b44:	jae    88716 <mibz2::bwt::build+0x1a6>
./src/bwt.rs:276
   88b4a:	cmp    %rdi,%rbx
   88b4d:	jbe    88e38 <mibz2::bwt::build+0x8c8>
   88b53:	shr    $0x38,%r9
   88b57:	movzbl (%r14,%rdi,1),%r11d
   88b5c:	add    %r8d,%r11d
   88b5f:	add    $0x600,%r11d
   88b66:	mov    %r11d,(%rdx,%rdi,4)
./src/bwt.rs:277
   88b6a:	inc    %edi
   88b6c:	mov    %edi,0x408(%rsp,%r10,4)
./src/bwt.rs:271
   88b74:	lea    0x0(,%r9,4),%r10d
   88b7c:	mov    0x408(%rsp,%r10,1),%edi
./src/bwt.rs:272
   88b84:	cmp    0x8(%rsp,%r10,1),%edi
   88b89:	jae    88716 <mibz2::bwt::build+0x1a6>
./src/bwt.rs:276
   88b8f:	cmp    %rdi,%rbx
   88b92:	jbe    88e38 <mibz2::bwt::build+0x8c8>
   88b98:	movzbl (%r14,%rdi,1),%r10d
   88b9d:	add    %r8d,%r10d
   88ba0:	add    $0x700,%r10d
   88ba7:	mov    %r10d,(%rdx,%rdi,4)
./src/bwt.rs:277
   88bab:	inc    %edi
   88bad:	mov    %edi,0x408(%rsp,%r9,4)
./src/bwt.rs:270
   88bb5:	movzbl %al,%r9d
./src/bwt.rs:271
   88bb9:	lea    0x0(,%r9,4),%r10d
   88bc1:	mov    0x408(%rsp,%r10,1),%edi
./src/bwt.rs:272
   88bc9:	cmp    0x8(%rsp,%r10,1),%edi
   88bce:	jae    88716 <mibz2::bwt::build+0x1a6>
./src/bwt.rs:276
   88bd4:	cmp    %rdi,%rbx
   88bd7:	jbe    88e38 <mibz2::bwt::build+0x8c8>
   88bdd:	mov    %rax,%r10
   88be0:	shr    $0x8,%r10
   88be4:	movzbl (%r14,%rdi,1),%r11d
   88be9:	add    %r8d,%r11d
   88bec:	add    $0x800,%r11d
   88bf3:	mov    %r11d,(%rdx,%rdi,4)
./src/bwt.rs:277
   88bf7:	inc    %edi
   88bf9:	mov    %edi,0x408(%rsp,%r9,4)
./src/bwt.rs:270
   88c01:	movzbl %r10b,%r9d
./src/bwt.rs:271
   88c05:	lea    0x0(,%r9,4),%r10d
   88c0d:	mov    0x408(%rsp,%r10,1),%edi
./src/bwt.rs:272
   88c15:	cmp    0x8(%rsp,%r10,1),%edi
   88c1a:	jae    88716 <mibz2::bwt::build+0x1a6>
./src/bwt.rs:276
   88c20:	cmp    %rdi,%rbx
   88c23:	jbe    88e38 <mibz2::bwt::build+0x8c8>
   88c29:	mov    %rax,%r10
   88c2c:	shr    $0x10,%r10
   88c30:	movzbl (%r14,%rdi,1),%r11d
   88c35:	add    %r8d,%r11d
   88c38:	add    $0x900,%r11d
   88c3f:	mov    %r11d,(%rdx,%rdi,4)
./src/bwt.rs:277
   88c43:	inc    %edi
   88c45:	mov    %edi,0x408(%rsp,%r9,4)
./src/bwt.rs:270
   88c4d:	movzbl %r10b,%r9d
./src/bwt.rs:271
   88c51:	lea    0x0(,%r9,4),%r10d
   88c59:	mov    0x408(%rsp,%r10,1),%edi
./src/bwt.rs:272
   88c61:	cmp    0x8(%rsp,%r10,1),%edi
   88c66:	jae    88716 <mibz2::bwt::build+0x1a6>
./src/bwt.rs:276
   88c6c:	cmp    %rdi,%rbx
   88c6f:	jbe    88e38 <mibz2::bwt::build+0x8c8>
   88c75:	mov    %rax,%r10
   88c78:	shr    $0x18,%r10
   88c7c:	movzbl (%r14,%rdi,1),%r11d
   88c81:	add    %r8d,%r11d
   88c84:	add    $0xa00,%r11d
   88c8b:	mov    %r11d,(%rdx,%rdi,4)
./src/bwt.rs:277
   88c8f:	inc    %edi
   88c91:	mov    %edi,0x408(%rsp,%r9,4)
./src/bwt.rs:270
   88c99:	movzbl %r10b,%r9d
./src/bwt.rs:271
   88c9d:	lea    0x0(,%r9,4),%r10d
   88ca5:	mov    0x408(%rsp,%r10,1),%edi
./src/bwt.rs:272
   88cad:	cmp    0x8(%rsp,%r10,1),%edi
   88cb2:	jae    88716 <mibz2::bwt::build+0x1a6>
./src/bwt.rs:276
   88cb8:	cmp    %rdi,%rbx
   88cbb:	jbe    88e38 <mibz2::bwt::build+0x8c8>
   88cc1:	mov    %rax,%r10
   88cc4:	shr    $0x20,%r10
   88cc8:	movzbl (%r14,%rdi,1),%r11d
   88ccd:	add    %r8d,%r11d
   88cd0:	add    $0xb00,%r11d
   88cd7:	mov    %r11d,(%rdx,%rdi,4)
./src/bwt.rs:277
   88cdb:	inc    %edi
   88cdd:	mov    %edi,0x408(%rsp,%r9,4)
./src/bwt.rs:270
   88ce5:	movzbl %r10b,%r9d
./src/bwt.rs:271
   88ce9:	lea    0x0(,%r9,4),%r10d
   88cf1:	mov    0x408(%rsp,%r10,1),%edi
./src/bwt.rs:272
   88cf9:	cmp    0x8(%rsp,%r10,1),%edi
   88cfe:	jae    88716 <mibz2::bwt::build+0x1a6>
./src/bwt.rs:276
   88d04:	cmp    %rdi,%rbx
   88d07:	jbe    88e38 <mibz2::bwt::build+0x8c8>
   88d0d:	mov    %rax,%r10
   88d10:	shr    $0x28,%r10
   88d14:	movzbl (%r14,%rdi,1),%r11d
   88d19:	add    %r8d,%r11d
   88d1c:	add    $0xc00,%r11d
   88d23:	mov    %r11d,(%rdx,%rdi,4)
./src/bwt.rs:277
   88d27:	inc    %edi
   88d29:	mov    %edi,0x408(%rsp,%r9,4)
./src/bwt.rs:270
   88d31:	movzbl %r10b,%r9d
./src/bwt.rs:271
   88d35:	lea    0x0(,%r9,4),%r10d
   88d3d:	mov    0x408(%rsp,%r10,1),%edi
./src/bwt.rs:272
   88d45:	cmp    0x8(%rsp,%r10,1),%edi
   88d4a:	jae    88716 <mibz2::bwt::build+0x1a6>
./src/bwt.rs:276
   88d50:	cmp    %rdi,%rbx
   88d53:	jbe    88e38 <mibz2::bwt::build+0x8c8>
   88d59:	mov    %rax,%r10
   88d5c:	shr    $0x30,%r10
   88d60:	movzbl (%r14,%rdi,1),%r11d
   88d65:	add    %r8d,%r11d
   88d68:	add    $0xd00,%r11d
   88d6f:	mov    %r11d,(%rdx,%rdi,4)
./src/bwt.rs:277
   88d73:	inc    %edi
   88d75:	mov    %edi,0x408(%rsp,%r9,4)
./src/bwt.rs:270
   88d7d:	movzbl %r10b,%r9d
./src/bwt.rs:271
   88d81:	lea    0x0(,%r9,4),%r10d
   88d89:	mov    0x408(%rsp,%r10,1),%edi
./src/bwt.rs:272
   88d91:	cmp    0x8(%rsp,%r10,1),%edi
   88d96:	jae    88716 <mibz2::bwt::build+0x1a6>
./src/bwt.rs:276
   88d9c:	cmp    %rdi,%rbx
   88d9f:	jbe    88e38 <mibz2::bwt::build+0x8c8>
   88da5:	shr    $0x38,%rax
   88da9:	movzbl (%r14,%rdi,1),%r10d
   88dae:	add    %r8d,%r10d
   88db1:	add    $0xe00,%r10d
   88db8:	mov    %r10d,(%rdx,%rdi,4)
./src/bwt.rs:277
   88dbc:	inc    %edi
   88dbe:	mov    %edi,0x408(%rsp,%r9,4)
./src/bwt.rs:271
   88dc6:	lea    0x0(,%rax,4),%r9d
   88dce:	mov    0x408(%rsp,%r9,1),%edi
./src/bwt.rs:272
   88dd6:	cmp    0x8(%rsp,%r9,1),%edi
   88ddb:	jae    88716 <mibz2::bwt::build+0x1a6>
./src/bwt.rs:276
   88de1:	cmp    %rdi,%rbx
   88de4:	jbe    88e38 <mibz2::bwt::build+0x8c8>
   88de6:	movzbl (%r14,%rdi,1),%r9d
   88deb:	add    %r8d,%r9d
   88dee:	add    $0xf00,%r9d
   88df5:	mov    %r9d,(%rdx,%rdi,4)
./src/bwt.rs:277
   88df9:	inc    %edi
   88dfb:	mov    %edi,0x408(%rsp,%rax,4)
   88e02:	jmp    88830 <mibz2::bwt::build+0x2c0>
lt():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/cmp.rs:1916
   88e07:	add    $0xfffffffffffffff8,%rax
   88e0b:	mov    %rax,%r8
   88e0e:	cmp    %rbx,%r8
spec_next<usize>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/range.rs:900
   88e11:	jb     8869e <mibz2::bwt::build+0x12e>
   88e17:	jmp    886f7 <mibz2::bwt::build+0x187>
checked_sub():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/num/uint_macros.rs:1050
   88e1c:	mov    %ebx,%edi
   88e1e:	and    $0xfffff0,%edi
   88e24:	jmp    88e59 <mibz2::bwt::build+0x8e9>
index<u8>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/index.rs:443
   88e26:	lea    0x4ba7b(%rip),%rcx        # d48a8 <__frame_dummy_init_array_entry+0x2ef8>
mibz2::bwt::build:
   88e2d:	mov    %rax,%rsi
   88e30:	mov    %rbx,%rdx
   88e33:	call   7c580 <core::slice::index::slice_index_fail>
./src/bwt.rs:276
   88e38:	lea    0x4b949(%rip),%rdx        # d4788 <__frame_dummy_init_array_entry+0x2dd8>
{closure#1}():
./src/bwt.rs:276
   88e3f:	mov    %rbx,%rsi
   88e42:	call   7b81b <core::panicking::panic_bounds_check>
mibz2::bwt::build:
   88e47:	lea    0x4baba(%rip),%rcx        # d4908 <__frame_dummy_init_array_entry+0x2f58>
   88e4e:	mov    %rax,%rdi
   88e51:	mov    %rbx,%rdx
   88e54:	call   7c580 <core::slice::index::slice_index_fail>
   88e59:	or     $0x8,%rdi
   88e5d:	add    $0x10,%rax
   88e61:	lea    0x4baa0(%rip),%rcx        # d4908 <__frame_dummy_init_array_entry+0x2f58>
   88e68:	mov    %rax,%rsi
   88e6b:	mov    %rbx,%rdx
   88e6e:	call   7c580 <core::slice::index::slice_index_fail>
./src/bwt.rs:245
   88e73:	add    $0x808,%rsp
   88e7a:	pop    %rbx
   88e7b:	pop    %r12
   88e7d:	pop    %r13
   88e7f:	pop    %r14
   88e81:	pop    %r15
   88e83:	pop    %rbp
   88e84:	ret
fold<u8, (), core::iter::adapters::map::map_fold::{closure_env#0}<&u8, u32, (), mibz2::bwt::build_two_pass::{closure_env#0}, core::iter::traits::iterator::Iterator::for_each::call::{closure_env#0}<u32, alloc::vec::{impl#21}::extend_trusted::{closure_env#0}<u32, alloc::alloc::Global, core::iter::adapters::map::Map<core::slice::iter::Iter<u8>, mibz2::bwt::build_two_pass::{closure_env#0}>>>>>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/iter/macros.rs:279
   88e85:	movzbl (%r14,%rax,1),%edx
write<u32>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1933
   88e8a:	mov    %edx,0x0(%r13,%r12,4)
mibz2::bwt::build:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/set_len_on_drop.rs:19
   88e8f:	inc    %r12
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/num/uint_macros.rs:971
   88e92:	inc    %rax
fold<u8, (), core::iter::adapters::map::map_fold::{closure_env#0}<&u8, u32, (), mibz2::bwt::build_two_pass::{closure_env#0}, core::iter::traits::iterator::Iterator::for_each::call::{closure_env#0}<u32, alloc::vec::{impl#21}::extend_trusted::{closure_env#0}<u32, alloc::alloc::Global, core::iter::adapters::map::Map<core::slice::iter::Iter<u8>, mibz2::bwt::build_two_pass::{closure_env#0}>>>>>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/iter/macros.rs:284
   88e95:	cmp    %rax,%rbx
   88e98:	jne    88e85 <mibz2::bwt::build+0x915>
mibz2::bwt::build:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/set_len_on_drop.rs:31
   88e9a:	mov    %r12,0x10(%rcx)
drop():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/set_len_on_drop.rs:31
   88e9e:	lea    0x8(%rsp),%rdi
   88ea3:	xor    %ebp,%ebp
mibz2::bwt::build:
./src/bwt.rs:326
   88ea5:	mov    $0x400,%edx
build_two_pass():
./src/bwt.rs:326
   88eaa:	xor    %esi,%esi
   88eac:	call   *0x4d59e(%rip)        # d6450 <memset@GLIBC_2.2.5>
   88eb2:	xor    %eax,%eax
   88eb4:	data16 data16 cs nopw 0x0(%rax,%rax,1)
./src/bwt.rs:329
   88ec0:	mov    %eax,0x8(%rsp,%rbp,4)
./src/bwt.rs:330
   88ec4:	add    (%r15,%rbp,4),%eax
./src/bwt.rs:329
   88ec8:	mov    %eax,0xc(%rsp,%rbp,4)
./src/bwt.rs:330
   88ecc:	add    0x4(%r15,%rbp,4),%eax
./src/bwt.rs:329
   88ed1:	mov    %eax,0x10(%rsp,%rbp,4)
./src/bwt.rs:330
   88ed5:	add    0x8(%r15,%rbp,4),%eax
./src/bwt.rs:329
   88eda:	mov    %eax,0x14(%rsp,%rbp,4)
./src/bwt.rs:330
   88ede:	add    0xc(%r15,%rbp,4),%eax
mibz2::bwt::build:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/num/uint_macros.rs:971
   88ee3:	add    $0x4,%rbp
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/cmp.rs:1916
   88ee7:	cmp    $0x100,%rbp
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/range.rs:900
   88eee:	jne    88ec0 <mibz2::bwt::build+0x950>
build_two_pass():
   88ef0:	xor    %ecx,%ecx
./src/bwt.rs:337
   88ef2:	cmp    $0x10,%rbx
   88ef6:	jb     8938a <mibz2::bwt::build+0xe1a>
   88efc:	mov    $0x8,%esi
   88f01:	movdqa -0x7df29(%rip),%xmm0        # afe0 <GCC_except_table819+0x814>
   88f09:	movdqa -0x7e6d1(%rip),%xmm1        # a840 <GCC_except_table819+0x74>
   88f11:	movdqa -0x7e219(%rip),%xmm2        # ad00 <GCC_except_table819+0x534>
   88f19:	movdqa -0x7de11(%rip),%xmm3        # b110 <GCC_except_table819+0x944>
   88f21:	movq   -0x7e2a9(%rip),%xmm4        # ac80 <GCC_except_table819+0x4b4>
   88f29:	movq   -0x7e281(%rip),%xmm5        # acb0 <GCC_except_table819+0x4e4>
   88f31:	jmp    8900d <mibz2::bwt::build+0xa9d>
   88f36:	cs nopw 0x0(%rax,%rax,1)
./src/bwt.rs:340
   88f40:	movzbl %dl,%edx
./src/bwt.rs:341
   88f43:	mov    %edx,%eax
   88f45:	mov    0x8(%rsp,%rax,4),%edi
./src/bwt.rs:343
   88f49:	lea    0x10(%rdi),%rax
mibz2::bwt::build:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/index.rs:451
   88f4d:	cmp    %r12,%rax
index_mut<u32>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/index.rs:451
   88f50:	ja     893e6 <mibz2::bwt::build+0xe76>
build_two_pass():
./src/bwt.rs:344
   88f56:	movzbl 0x0(%r13,%rdi,4),%eax
   88f5c:	add    %ecx,%eax
   88f5e:	mov    %eax,0x0(%r13,%rdi,4)
   88f63:	movdqu 0x4(%r13,%rdi,4),%xmm6
   88f6a:	movdqu 0x14(%r13,%rdi,4),%xmm7
   88f71:	movdqu 0x24(%r13,%rdi,4),%xmm8
   88f78:	pand   %xmm0,%xmm6
   88f7c:	movd   %ecx,%xmm9
   88f81:	pshufd $0x0,%xmm9,%xmm9
   88f87:	por    %xmm9,%xmm6
   88f8c:	por    %xmm1,%xmm6
   88f90:	movdqu %xmm6,0x4(%r13,%rdi,4)
   88f97:	pand   %xmm0,%xmm7
   88f9b:	por    %xmm9,%xmm7
   88fa0:	por    %xmm2,%xmm7
   88fa4:	movdqu %xmm7,0x14(%r13,%rdi,4)
   88fab:	pand   %xmm0,%xmm8
   88fb0:	por    %xmm9,%xmm8
   88fb5:	por    %xmm3,%xmm8
   88fba:	movdqu %xmm8,0x24(%r13,%rdi,4)
   88fc1:	movq   0x34(%r13,%rdi,4),%xmm6
   88fc8:	pand   %xmm4,%xmm6
   88fcc:	por    %xmm9,%xmm6
   88fd1:	por    %xmm5,%xmm6
   88fd5:	movq   %xmm6,0x34(%r13,%rdi,4)
   88fdc:	movzbl 0x3c(%r13,%rdi,4),%eax
   88fe2:	add    %ecx,%eax
   88fe4:	add    $0xf00,%eax
   88fe9:	mov    %eax,0x3c(%r13,%rdi,4)
./src/bwt.rs:346
   88fee:	addl   $0x10,0x8(%rsp,%rdx,4)
./src/bwt.rs:337
   88ff3:	lea    0x10(%rsi),%rax
   88ff7:	add    $0x18,%rsi
   88ffb:	add    $0x1000,%ecx
   89001:	cmp    %rbx,%rsi
   89004:	mov    %rax,%rsi
   89007:	ja     89383 <mibz2::bwt::build+0xe13>
mibz2::bwt::build:
./src/bwt.rs:370
   8900d:	lea    -0x8(%rsi),%rax
read_u64():
./src/bwt.rs:370
   89011:	cmp    %rbx,%rsi
   89014:	ja     88e47 <mibz2::bwt::build+0x8d7>
mibz2::bwt::build:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/num/uint_macros.rs:1050
   8901a:	lea    0x7(%rsi),%rdx
checked_sub():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/num/uint_macros.rs:1050
   8901e:	cmp    %rbx,%rdx
   89021:	jae    893d4 <mibz2::bwt::build+0xe64>
mibz2::bwt::build:
./src/bwt.rs:370
   89027:	mov    -0x8(%r14,%rsi,1),%rdx
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/result.rs:1718
   8902c:	mov    (%r14,%rsi,1),%rax
build_two_pass():
./src/bwt.rs:339
   89030:	mov    %rdx,%rdi
   89033:	xor    %rax,%rdi
   89036:	mov    %rdx,%r8
   89039:	rol    $0x8,%r8
   8903d:	xor    %rdx,%r8
   89040:	or     %rdi,%r8
   89043:	je     88f40 <mibz2::bwt::build+0x9d0>
mibz2::bwt::build:
./src/bwt.rs:360
   89049:	movzbl %dl,%r8d
link():
./src/bwt.rs:360
   8904d:	mov    %r8d,%edi
   89050:	mov    0x8(%rsp,%rdi,4),%edi
./src/bwt.rs:363
   89054:	cmp    %rdi,%r12
   89057:	jbe    893f8 <mibz2::bwt::build+0xe88>
   8905d:	mov    %rdx,%r9
   89060:	shr    $0x8,%r9
   89064:	movzbl 0x0(%r13,%rdi,4),%r10d
   8906a:	add    %ecx,%r10d
   8906d:	mov    %r10d,0x0(%r13,%rdi,4)
./src/bwt.rs:364
   89072:	inc    %edi
   89074:	mov    %edi,0x8(%rsp,%r8,4)
./src/bwt.rs:360
   89079:	movzbl %r9b,%r8d
   8907d:	mov    %r8d,%edi
   89080:	mov    0x8(%rsp,%rdi,4),%edi
./src/bwt.rs:363
   89084:	cmp    %rdi,%r12
   89087:	jbe    893f8 <mibz2::bwt::build+0xe88>
   8908d:	mov    %rdx,%r9
   89090:	shr    $0x10,%r9
   89094:	movzbl 0x0(%r13,%rdi,4),%r10d
   8909a:	add    %ecx,%r10d
   8909d:	add    $0x100,%r10d
   890a4:	mov    %r10d,0x0(%r13,%rdi,4)
./src/bwt.rs:364
   890a9:	inc    %edi
   890ab:	mov    %edi,0x8(%rsp,%r8,4)
./src/bwt.rs:360
   890b0:	movzbl %r9b,%r8d
   890b4:	mov    %r8d,%edi
   890b7:	mov    0x8(%rsp,%rdi,4),%edi
./src/bwt.rs:363
   890bb:	cmp    %rdi,%r12
   890be:	jbe    893f8 <mibz2::bwt::build+0xe88>
   890c4:	mov    %rdx,%r9
   890c7:	shr    $0x18,%r9
   890cb:	movzbl 0x0(%r13,%rdi,4),%r10d
   890d1:	add    %ecx,%r10d
   890d4:	add    $0x200,%r10d
   890db:	mov    %r10d,0x0(%r13,%rdi,4)
./src/bwt.rs:364
   890e0:	inc    %edi
   890e2:	mov    %edi,0x8(%rsp,%r8,4)
./src/bwt.rs:360
   890e7:	movzbl %r9b,%r8d
   890eb:	mov    %r8d,%edi
   890ee:	mov    0x8(%rsp,%rdi,4),%edi
./src/bwt.rs:363
   890f2:	cmp    %rdi,%r12
   890f5:	jbe    893f8 <mibz2::bwt::build+0xe88>
   890fb:	mov    %rdx,%r9
   890fe:	shr    $0x20,%r9
   89102:	movzbl 0x0(%r13,%rdi,4),%r10d
   89108:	add    %ecx,%r10d
   8910b:	add    $0x300,%r10d
   89112:	mov    %r10d,0x0(%r13,%rdi,4)
./src/bwt.rs:364
   89117:	inc    %edi
   89119:	mov    %edi,0x8(%rsp,%r8,4)
./src/bwt.rs:360
   8911e:	movzbl %r9b,%r8d
   89122:	mov    %r8d,%edi
   89125:	mov    0x8(%rsp,%rdi,4),%edi
./src/bwt.rs:363
   89129:	cmp    %rdi,%r12
   8912c:	jbe    893f8 <mibz2::bwt::build+0xe88>
   89132:	mov    %rdx,%r9
   89135:	shr    $0x28,%r9
   89139:	movzbl 0x0(%r13,%rdi,4),%r10d
   8913f:	add    %ecx,%r10d
   89142:	add    $0x400,%r10d
   89149:	mov    %r10d,0x0(%r13,%rdi,4)
./src/bwt.rs:364
   8914e:	inc    %edi
   89150:	mov    %edi,0x8(%rsp,%r8,4)
./src/bwt.rs:360
   89155:	movzbl %r9b,%r8d
   89159:	mov    %r8d,%edi
   8915c:	mov    0x8(%rsp,%rdi,4),%edi
./src/bwt.rs:363
   89160:	cmp    %rdi,%r12
   89163:	jbe    893f8 <mibz2::bwt::build+0xe88>
   89169:	mov    %rdx,%r9
   8916c:	shr    $0x30,%r9
   89170:	movzbl 0x0(%r13,%rdi,4),%r10d
   89176:	add    %ecx,%r10d
   89179:	add    $0x500,%r10d
   89180:	mov    %r10d,0x0(%r13,%rdi,4)
./src/bwt.rs:364
   89185:	inc    %edi
   89187:	mov    %edi,0x8(%rsp,%r8,4)
./src/bwt.rs:360
   8918c:	movzbl %r9b,%r8d
   89190:	mov    %r8d,%edi
   89193:	mov    0x8(%rsp,%rdi,4),%edi
./src/bwt.rs:363
   89197:	cmp    %rdi,%r12
   8919a:	jbe    893f8 <mibz2::bwt::build+0xe88>
   891a0:	shr    $0x38,%rdx
   891a4:	movzbl 0x0(%r13,%rdi,4),%r9d
   891aa:	add    %ecx,%r9d
   891ad:	add    $0x600,%r9d
   891b4:	mov    %r9d,0x0(%r13,%rdi,4)
./src/bwt.rs:364
   891b9:	inc    %edi
   891bb:	mov    %edi,0x8(%rsp,%r8,4)
./src/bwt.rs:360
   891c0:	mov    0x8(%rsp,%rdx,4),%edi
./src/bwt.rs:363
   891c4:	cmp    %rdi,%r12
   891c7:	jbe    893f8 <mibz2::bwt::build+0xe88>
   891cd:	movzbl 0x0(%r13,%rdi,4),%r8d
   891d3:	add    %ecx,%r8d
   891d6:	add    $0x700,%r8d
   891dd:	mov    %r8d,0x0(%r13,%rdi,4)
./src/bwt.rs:364
   891e2:	inc    %edi
   891e4:	mov    %edi,0x8(%rsp,%rdx,4)
./src/bwt.rs:360
   891e8:	movzbl %al,%edx
   891eb:	mov    %edx,%edi
   891ed:	mov    0x8(%rsp,%rdi,4),%edi
./src/bwt.rs:363
   891f1:	cmp    %rdi,%r12
   891f4:	jbe    893f8 <mibz2::bwt::build+0xe88>
   891fa:	mov    %rax,%r8
   891fd:	shr    $0x8,%r8
   89201:	movzbl 0x0(%r13,%rdi,4),%r9d
   89207:	add    %ecx,%r9d
   8920a:	add    $0x800,%r9d
   89211:	mov    %r9d,0x0(%r13,%rdi,4)
./src/bwt.rs:364
   89216:	inc    %edi
   89218:	mov    %edi,0x8(%rsp,%rdx,4)
./src/bwt.rs:360
   8921c:	movzbl %r8b,%edx
   89220:	mov    %edx,%edi
   89222:	mov    0x8(%rsp,%rdi,4),%edi
./src/bwt.rs:363
   89226:	cmp    %rdi,%r12
   89229:	jbe    893f8 <mibz2::bwt::build+0xe88>
   8922f:	mov    %rax,%r8
   89232:	shr    $0x10,%r8
   89236:	movzbl 0x0(%r13,%rdi,4),%r9d
   8923c:	add    %ecx,%r9d
   8923f:	add    $0x900,%r9d
   89246:	mov    %r9d,0x0(%r13,%rdi,4)
./src/bwt.rs:364
   8924b:	inc    %edi
   8924d:	mov    %edi,0x8(%rsp,%rdx,4)
./src/bwt.rs:360
   89251:	movzbl %r8b,%edx
   89255:	mov    %edx,%edi
   89257:	mov    0x8(%rsp,%rdi,4),%edi
./src/bwt.rs:363
   8925b:	cmp    %rdi,%r12
   8925e:	jbe    893f8 <mibz2::bwt::build+0xe88>
   89264:	mov    %rax,%r8
   89267:	shr    $0x18,%r8
   8926b:	movzbl 0x0(%r13,%rdi,4),%r9d
   89271:	add    %ecx,%r9d
   89274:	add    $0xa00,%r9d
   8927b:	mov    %r9d,0x0(%r13,%rdi,4)
./src/bwt.rs:364
   89280:	inc    %edi
   89282:	mov    %edi,0x8(%rsp,%rdx,4)
./src/bwt.rs:360
   89286:	movzbl %r8b,%edx
   8928a:	mov    %edx,%edi
   8928c:	mov    0x8(%rsp,%rdi,4),%edi
./src/bwt.rs:363
   89290:	cmp    %rdi,%r12
   89293:	jbe    893f8 <mibz2::bwt::build+0xe88>
   89299:	mov    %rax,%r8
   8929c:	shr    $0x20,%r8
   892a0:	movzbl 0x0(%r13,%rdi,4),%r9d
   892a6:	add    %ecx,%r9d
   892a9:	add    $0xb00,%r9d
   892b0:	mov    %r9d,0x0(%r13,%rdi,4)
./src/bwt.rs:364
   892b5:	inc    %edi
   892b7:	mov    %edi,0x8(%rsp,%rdx,4)
./src/bwt.rs:360
   892bb:	movzbl %r8b,%edx
   892bf:	mov    %edx,%edi
   892c1:	mov    0x8(%rsp,%rdi,4),%edi
./src/bwt.rs:363
   892c5:	cmp    %rdi,%r12
   892c8:	jbe    893f8 <mibz2::bwt::build+0xe88>
   892ce:	mov    %rax,%r8
   892d1:	shr    $0x28,%r8
   892d5:	movzbl 0x0(%r13,%rdi,4),%r9d
   892db:	add    %ecx,%r9d
   892de:	add    $0xc00,%r9d
   892e5:	mov    %r9d,0x0(%r13,%rdi,4)
./src/bwt.rs:364
   892ea:	inc    %edi
   892ec:	mov    %edi,0x8(%rsp,%rdx,4)
./src/bwt.rs:360
   892f0:	movzbl %r8b,%edx
   892f4:	mov    %edx,%edi
   892f6:	mov    0x8(%rsp,%rdi,4),%edi
./src/bwt.rs:363
   892fa:	cmp    %rdi,%r12
   892fd:	jbe    893f8 <mibz2::bwt::build+0xe88>
   89303:	mov    %rax,%r8
   89306:	shr    $0x30,%r8
   8930a:	movzbl 0x0(%r13,%rdi,4),%r9d
   89310:	add    %ecx,%r9d
   89313:	add    $0xd00,%r9d
   8931a:	mov    %r9d,0x0(%r13,%rdi,4)
./src/bwt.rs:364
   8931f:	inc    %edi
   89321:	mov    %edi,0x8(%rsp,%rdx,4)
./src/bwt.rs:360
   89325:	movzbl %r8b,%edx
   89329:	mov    %edx,%edi
   8932b:	mov    0x8(%rsp,%rdi,4),%edi
./src/bwt.rs:363
   8932f:	cmp    %rdi,%r12
   89332:	jbe    893f8 <mibz2::bwt::build+0xe88>
   89338:	shr    $0x38,%rax
   8933c:	movzbl 0x0(%r13,%rdi,4),%r8d
   89342:	add    %ecx,%r8d
   89345:	add    $0xe00,%r8d
   8934c:	mov    %r8d,0x0(%r13,%rdi,4)
./src/bwt.rs:364
   89351:	inc    %edi
   89353:	mov    %edi,0x8(%rsp,%rdx,4)
./src/bwt.rs:360
   89357:	mov    0x8(%rsp,%rax,4),%edi
./src/bwt.rs:363
   8935b:	cmp    %rdi,%r12
   8935e:	jbe    893f8 <mibz2::bwt::build+0xe88>
   89364:	movzbl 0x0(%r13,%rdi,4),%edx
   8936a:	add    %ecx,%edx
   8936c:	add    $0xf00,%edx
   89372:	mov    %edx,0x0(%r13,%rdi,4)
./src/bwt.rs:364
   89377:	lea    0x1(%rdi),%edx
   8937a:	mov    %edx,0x8(%rsp,%rax,4)
   8937e:	jmp    88ff3 <mibz2::bwt::build+0xa83>
build_two_pass():
./src/bwt.rs:352
   89383:	add    $0xfffffffffffffff8,%rax
   89387:	mov    %rax,%rcx
mibz2::bwt::build:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/cmp.rs:1916
   8938a:	cmp    %rbx,%rcx
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/range.rs:900
   8938d:	jae    88e73 <mibz2::bwt::build+0x903>
./src/bwt.rs:363
   89393:	mov    %ecx,%eax
link():
./src/bwt.rs:363
   89395:	shl    $0x8,%eax
   89398:	nopl   0x0(%rax,%rax,1)
./src/bwt.rs:359
   893a0:	movzbl (%r14,%rcx,1),%edx
./src/bwt.rs:360
   893a5:	mov    0x8(%rsp,%rdx,4),%edi
./src/bwt.rs:363
   893a9:	cmp    %rdi,%r12
   893ac:	jbe    893f8 <mibz2::bwt::build+0xe88>
mibz2::bwt::build:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/num/uint_macros.rs:971
   893ae:	inc    %rcx
link():
./src/bwt.rs:363
   893b1:	movzbl 0x0(%r13,%rdi,4),%esi
   893b7:	add    %eax,%esi
   893b9:	mov    %esi,0x0(%r13,%rdi,4)
./src/bwt.rs:364
   893be:	lea    0x1(%rdi),%esi
   893c1:	mov    %esi,0x8(%rsp,%rdx,4)
lt():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/cmp.rs:1916
   893c5:	add    $0x100,%eax
   893ca:	cmp    %rcx,%rbx
spec_next<usize>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/range.rs:900
   893cd:	jne    893a0 <mibz2::bwt::build+0xe30>
   893cf:	jmp    88e73 <mibz2::bwt::build+0x903>
build_two_pass():
   893d4:	movabs $0x7ffffffffffffff0,%rdi
./src/bwt.rs:337
   893de:	and    %rbx,%rdi
   893e1:	jmp    88e59 <mibz2::bwt::build+0x8e9>
index_mut<u32>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/index.rs:456
   893e6:	lea    0x4b4d3(%rip),%rcx        # d48c0 <__frame_dummy_init_array_entry+0x2f10>
   893ed:	mov    %rax,%rsi
   893f0:	mov    %r12,%rdx
   893f3:	call   7c580 <core::slice::index::slice_index_fail>
mibz2::bwt::build:
./src/bwt.rs:363
   893f8:	lea    0x4b4d9(%rip),%rdx        # d48d8 <__frame_dummy_init_array_entry+0x2f28>
link():
./src/bwt.rs:363
   893ff:	mov    %r12,%rsi
   89402:	call   7b81b <core::panicking::panic_bounds_check>
