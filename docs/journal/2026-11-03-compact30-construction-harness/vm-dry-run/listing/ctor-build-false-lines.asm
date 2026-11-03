
./target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

000000000002a370 <<c30walk::ctor::Ctor>::build::<false>>:
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:711
   2a370:	push   %rbp
   2a371:	push   %r15
   2a373:	push   %r14
   2a375:	push   %r13
   2a377:	push   %r12
   2a379:	push   %rbx
   2a37a:	sub    $0x1000,%rsp
   2a381:	movq   $0x0,(%rsp)
   2a389:	sub    $0x118,%rsp
   2a390:	mov    0x1150(%rsp),%rax
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/mod.rs:3035
   2a398:	movq   $0x0,0x10(%r9)
   2a3a0:	movq   $0x0,0x10(%rax)
clear<c30walk::RawG, alloc::alloc::Global>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/mod.rs:3035
   2a3a8:	cmp    $0xeffe2,%rdx
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:722
   2a3af:	ja     2aa4b <<c30walk::ctor::Ctor>::build::<false>+0x6db>
   2a3b5:	mov    %rcx,%rbx
   2a3b8:	pxor   %xmm0,%xmm0
   2a3bc:	xor    %eax,%eax
   2a3be:	pxor   %xmm2,%xmm2
   2a3c2:	pxor   %xmm1,%xmm1
   2a3c6:	cs nopw 0x0(%rax,%rax,1)
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/iter/macros.rs:279
   2a3d0:	movq   (%rbx,%rax,4),%xmm3
fold<u32, usize, core::iter::adapters::map::map_fold::{closure_env#0}<&u32, usize, usize, c30walk::ctor::{impl#2}::build::{closure_env#0}<false>, core::iter::traits::accum::{impl#48}::sum::{closure_env#0}<core::iter::adapters::map::Map<core::slice::iter::Iter<u32>, c30walk::ctor::{impl#2}::build::{closure_env#0}<false>>>>>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/iter/macros.rs:279
   2a3d5:	movq   0x8(%rbx,%rax,4),%xmm4
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:722
   2a3db:	punpckldq %xmm0,%xmm3
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/traits/accum.rs:55
   2a3df:	paddq  %xmm2,%xmm3
{closure#0}<false>():
./examples/c30walk/ctor.rs:722
   2a3e3:	punpckldq %xmm0,%xmm4
{closure#0}<core::iter::adapters::map::Map<core::slice::iter::Iter<u32>, c30walk::ctor::{impl#2}::build::{closure_env#0}<false>>>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/traits/accum.rs:55
   2a3e7:	paddq  %xmm1,%xmm4
fold<u32, usize, core::iter::adapters::map::map_fold::{closure_env#0}<&u32, usize, usize, c30walk::ctor::{impl#2}::build::{closure_env#0}<false>, core::iter::traits::accum::{impl#48}::sum::{closure_env#0}<core::iter::adapters::map::Map<core::slice::iter::Iter<u32>, c30walk::ctor::{impl#2}::build::{closure_env#0}<false>>>>>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/iter/macros.rs:279
   2a3eb:	movq   0x10(%rbx,%rax,4),%xmm2
   2a3f1:	movq   0x18(%rbx,%rax,4),%xmm1
{closure#0}<false>():
./examples/c30walk/ctor.rs:722
   2a3f7:	punpckldq %xmm0,%xmm2
{closure#0}<core::iter::adapters::map::Map<core::slice::iter::Iter<u32>, c30walk::ctor::{impl#2}::build::{closure_env#0}<false>>>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/traits/accum.rs:55
   2a3fb:	paddq  %xmm3,%xmm2
{closure#0}<false>():
./examples/c30walk/ctor.rs:722
   2a3ff:	punpckldq %xmm0,%xmm1
{closure#0}<core::iter::adapters::map::Map<core::slice::iter::Iter<u32>, c30walk::ctor::{impl#2}::build::{closure_env#0}<false>>>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/traits/accum.rs:55
   2a403:	paddq  %xmm4,%xmm1
fold<u32, usize, core::iter::adapters::map::map_fold::{closure_env#0}<&u32, usize, usize, c30walk::ctor::{impl#2}::build::{closure_env#0}<false>, core::iter::traits::accum::{impl#48}::sum::{closure_env#0}<core::iter::adapters::map::Map<core::slice::iter::Iter<u32>, c30walk::ctor::{impl#2}::build::{closure_env#0}<false>>>>>():
   2a407:	add    $0x8,%rax
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/iter/macros.rs:284
   2a40b:	cmp    $0x100,%rax
   2a411:	jne    2a3d0 <<c30walk::ctor::Ctor>::build::<false>+0x60>
   2a413:	paddq  %xmm2,%xmm1
   2a417:	pshufd $0xee,%xmm1,%xmm0
   2a41c:	paddq  %xmm1,%xmm0
   2a420:	movq   %xmm0,%rax
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:722
   2a425:	cmp    %rdx,%rax
   2a428:	jne    2aa4b <<c30walk::ctor::Ctor>::build::<false>+0x6db>
   2a42e:	mov    %r8d,0x3c(%rsp)
   2a433:	mov    %rsi,0x28(%rsp)
   2a438:	mov    %rdi,%r15
   2a43b:	mov    %rdx,0x20(%rsp)
   2a440:	mov    %r9,0x50(%rsp)
   2a445:	lea    0xd18(%rsp),%rdi
   2a44d:	xor    %r12d,%r12d
   2a450:	mov    0xabff9(%rip),%r14        # d6450 <memset@GLIBC_2.2.5>
./examples/c30walk/ctor.rs:726
   2a457:	mov    $0x400,%edx
   2a45c:	xor    %esi,%esi
   2a45e:	call   *%r14
   2a461:	lea    0x918(%rsp),%rdi
   2a469:	mov    $0x400,%edx
   2a46e:	xor    %esi,%esi
   2a470:	call   *%r14
   2a473:	movabs $0x8888888888888889,%r9
   2a47d:	xor    %esi,%esi
   2a47f:	xor    %edi,%edi
   2a481:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
./examples/c30walk.rs:101
   2a490:	mov    %r12,%rax
phi():
./examples/c30walk.rs:101
   2a493:	mul    %r9
   2a496:	mov    %rdx,%rcx
   2a499:	shr    $0x4,%rcx
   2a49d:	mov    %ecx,%eax
   2a49f:	shl    $0x5,%eax
   2a4a2:	add    %ecx,%ecx
   2a4a4:	sub    %eax,%ecx
   2a4a6:	add    %r12d,%ecx
   2a4a9:	or     %eax,%ecx
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:730
   2a4ab:	mov    (%rbx,%rdi,4),%r8d
   2a4af:	add    %r8,%r12
./examples/c30walk.rs:101
   2a4b2:	mov    %r12,%rax
phi():
./examples/c30walk.rs:101
   2a4b5:	mul    %r9
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:729
   2a4b8:	mov    %ecx,0x918(%rsp,%rdi,4)
phi():
./examples/c30walk.rs:101
   2a4bf:	shr    $0x4,%rdx
   2a4c3:	mov    %edx,%eax
   2a4c5:	shl    $0x5,%eax
   2a4c8:	add    %edx,%edx
   2a4ca:	sub    %eax,%edx
   2a4cc:	add    %r12d,%edx
   2a4cf:	or     %eax,%edx
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:731
   2a4d1:	mov    %edx,0xd18(%rsp,%rdi,4)
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/num/uint_macros.rs:971
   2a4d8:	inc    %rdi
./examples/c30walk/ctor.rs:732
   2a4db:	cmp    $0x1,%r8
   2a4df:	sbb    $0xffffffffffffffff,%rsi
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/cmp.rs:1916
   2a4e3:	cmp    $0x100,%rdi
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/range.rs:900
   2a4ea:	jne    2a490 <<c30walk::ctor::Ctor>::build::<false>+0x120>
   2a4ec:	mov    0x20(%rsp),%rdx
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/num/uint_macros.rs:3756
   2a4f1:	mov    %edx,%eax
div_ceil():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/num/uint_macros.rs:3756
   2a4f3:	mov    $0x88888889,%ecx
   2a4f8:	imul   %rax,%rcx
   2a4fc:	shr    $0x24,%rcx
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/num/uint_macros.rs:3758
   2a500:	imul   $0xeeeeeeef,%edx,%eax
   2a506:	ror    $1,%eax
   2a508:	cmp    $0x8888889,%eax
   2a50d:	sbb    $0xffffffffffffffff,%rcx
   2a511:	mov    %rcx,%r12
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:734
   2a514:	cmpl   $0x0,0x3c(%rsp)
   2a519:	mov    0x50(%rsp),%r8
   2a51e:	mov    0x1150(%rsp),%r9
   2a526:	mov    %r15,%rdi
   2a529:	mov    %rcx,0x48(%rsp)
./examples/c30walk/ctor.rs:736
   2a52e:	jne    2a578 <<c30walk::ctor::Ctor>::build::<false>+0x208>
./examples/c30walk/ctor.rs:738
   2a530:	test   %rdx,%rdx
   2a533:	je     2bc08 <<c30walk::ctor::Ctor>::build::<false>+0x1898>
./examples/c30walk.rs:101
   2a539:	lea    -0x1(%rdx),%eax
phi():
./examples/c30walk.rs:101
   2a53c:	mov    $0x88888889,%ecx
   2a541:	imul   %rax,%rcx
   2a545:	shr    $0x23,%rcx
   2a549:	and    $0xfffffffe,%ecx
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:741
   2a54c:	lea    (%rcx,%rdx,1),%eax
   2a54f:	dec    %eax
   2a551:	shr    $1,%eax
   2a553:	mov    $0x80010003,%ecx
   2a558:	imul   %rax,%rcx
   2a55c:	shr    $0x2e,%rcx
   2a560:	inc    %ecx
   2a562:	imul   %rcx,%rsi
   2a566:	dec    %rsi
   2a569:	mov    0x48(%rsp),%rcx
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/cmp.rs:1077
   2a56e:	cmp    %rcx,%rsi
min<usize>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/cmp.rs:1077
   2a571:	cmovae %rcx,%rsi
   2a575:	mov    %rsi,%r12
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/raw_vec/mod.rs:619
   2a578:	mov    (%r8),%rax
./examples/c30walk/ctor.rs:744
   2a57b:	lea    0x1(%rcx),%rbp
   2a57f:	cmp    %rcx,%rax
   2a582:	jbe    2bc20 <<c30walk::ctor::Ctor>::build::<false>+0x18b0>
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/raw_vec/mod.rs:619
   2a588:	mov    (%r9),%rax
./examples/c30walk/ctor.rs:750
   2a58b:	cmp    %r12,%rax
   2a58e:	jb     2bc77 <<c30walk::ctor::Ctor>::build::<false>+0x1907>
   2a594:	mov    0x48(%rsp),%rax
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/num/uint_macros.rs:3758
   2a599:	lea    0x3f(%rax),%r14
div_ceil():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/num/uint_macros.rs:3758
   2a59d:	shr    $0x6,%r14
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/mod.rs:3035
   2a5a1:	movq   $0x0,0x10(%rdi)
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/mod.rs:3525
   2a5a9:	je     2a606 <<c30walk::ctor::Ctor>::build::<false>+0x296>
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/raw_vec/mod.rs:767
   2a5ab:	cmp    (%rdi),%r14
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/raw_vec/mod.rs:673
   2a5ae:	ja     2bd6d <<c30walk::ctor::Ctor>::build::<false>+0x19fd>
   2a5b4:	xor    %r13d,%r13d
extend_with<u64, alloc::alloc::Global>():
   2a5b7:	mov    %rdi,%r12
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/raw_vec/mod.rs:614
   2a5ba:	mov    0x8(%rdi),%r15
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mut_ptr.rs:961
   2a5be:	lea    (%r15,%r13,8),%rdi
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/cmp.rs:1916
   2a5c2:	cmp    $0x1,%r14
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/range.rs:900
   2a5c6:	je     2a5f7 <<c30walk::ctor::Ctor>::build::<false>+0x287>
spec_next<usize>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/range.rs:900
   2a5c8:	lea    -0x8(,%r14,8),%rdx
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1933
   2a5d0:	xor    %esi,%esi
write<u64>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1933
   2a5d2:	call   *0xabe78(%rip)        # d6450 <memset@GLIBC_2.2.5>
   2a5d8:	mov    0x1150(%rsp),%r9
   2a5e0:	mov    0x50(%rsp),%r8
spec_next<usize>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/range.rs:900
   2a5e5:	lea    (%r14,%r13,1),%rax
   2a5e9:	lea    (%r15,%rax,8),%rdi
   2a5ed:	add    $0xfffffffffffffff8,%rdi
   2a5f1:	add    %r14,%r13
   2a5f4:	dec    %r13
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1933
   2a5f7:	movq   $0x0,(%rdi)
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/set_len_on_drop.rs:19
   2a5fe:	inc    %r13
increment_len():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/set_len_on_drop.rs:19
   2a601:	mov    %r12,%rdi
   2a604:	jmp    2a609 <<c30walk::ctor::Ctor>::build::<false>+0x299>
resize<u64, alloc::alloc::Global>():
   2a606:	xor    %r13d,%r13d
   2a609:	mov    %r13,0x10(%rdi)
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/raw_vec/mod.rs:614
   2a60d:	mov    0x8(%r8),%rcx
./examples/c30walk/ctor.rs:759
   2a611:	mov    %rcx,%rax
   2a614:	and    $0x3f,%rax
   2a618:	mov    %rax,0xd8(%rsp)
   2a620:	jne    2bd31 <<c30walk::ctor::Ctor>::build::<false>+0x19c1>
   2a626:	mov    %rbp,0xc8(%rsp)
./examples/c30walk/ctor.rs:756
   2a62e:	lea    0x18(%rdi),%r14
   2a632:	mov    %rdi,0xd0(%rsp)
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/raw_vec/mod.rs:614
   2a63a:	mov    0x8(%rdi),%rdx
non_null<alloc::alloc::Global, u64>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/raw_vec/mod.rs:614
   2a63e:	mov    %rcx,0x90(%rsp)
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:761
   2a646:	mov    %rcx,0xf8(%rsp)
   2a64e:	mov    0x48(%rsp),%rax
   2a653:	mov    %rax,0x100(%rsp)
   2a65b:	mov    %r9,0xd8(%rsp)
   2a663:	mov    %rdx,0x10(%rsp)
   2a668:	mov    %rdx,0xe0(%rsp)
   2a670:	mov    %r13,0x18(%rsp)
   2a675:	mov    %r13,0xe8(%rsp)
   2a67d:	lea    0x108(%rsp),%rdi
   2a685:	lea    0x918(%rsp),%rsi
   2a68d:	mov    0xabddc(%rip),%r15        # d6470 <memcpy@GLIBC_2.14>
   2a694:	mov    $0x400,%edx
   2a699:	call   *%r15
   2a69c:	lea    0x508(%rsp),%rdi
   2a6a4:	lea    0xd18(%rsp),%rsi
   2a6ac:	mov    $0x400,%edx
   2a6b1:	call   *%r15
   2a6b4:	movabs $0x8888888888888889,%r8
   2a6be:	mov    0x20(%rsp),%r9
   2a6c3:	movl   $0x0,0x910(%rsp)
   2a6ce:	movq   $0x0,0x908(%rsp)
   2a6da:	mov    %r14,0xf0(%rsp)
   2a6e2:	mov    $0xffffffffffffffff,%rcx
   2a6e9:	xor    %r13d,%r13d
   2a6ec:	movabs $0xeeeeeeeeeeeeeeef,%r11
   2a6f6:	movabs $0x888888888888889,%r12
   2a700:	pxor   %xmm0,%xmm0
   2a704:	pcmpeqd %xmm1,%xmm1
   2a708:	xor    %ebp,%ebp
   2a70a:	mov    0x28(%rsp),%r10
   2a70f:	jmp    2a7d0 <<c30walk::ctor::Ctor>::build::<false>+0x460>
   2a714:	data16 data16 cs nopw 0x0(%rax,%rax,1)
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/raw_vec/mod.rs:614
   2a720:	mov    0x8(%rdi),%rax
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mut_ptr.rs:961
   2a724:	mov    %r15,%rcx
add<c30walk::RawG>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mut_ptr.rs:961
   2a727:	shl    $0x7,%rcx
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1933
   2a72b:	movdqa %xmm0,0x70(%rax,%rcx,1)
write<c30walk::RawG>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1933
   2a731:	movdqa %xmm0,0x60(%rax,%rcx,1)
   2a737:	movdqa %xmm0,0x50(%rax,%rcx,1)
   2a73d:	movdqa %xmm0,0x40(%rax,%rcx,1)
   2a743:	movdqa %xmm0,0x30(%rax,%rcx,1)
   2a749:	movdqa %xmm0,0x20(%rax,%rcx,1)
   2a74f:	movdqa %xmm0,0x10(%rax,%rcx,1)
   2a755:	movdqa %xmm0,(%rax,%rcx,1)
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/mod.rs:1046
   2a75a:	mov    %r15,%rax
push_mut<c30walk::RawG, alloc::alloc::Global>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/mod.rs:1046
   2a75d:	inc    %rax
   2a760:	mov    %rax,0x10(%rdi)
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mut_ptr.rs:961
   2a764:	mov    %r14,%rax
add<c30walk::Rec>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mut_ptr.rs:961
   2a767:	shl    $0x6,%rax
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:371
   2a76b:	or     $0x10000000,%r15d
create_raw<false>():
./examples/c30walk/ctor.rs:371
   2a772:	mov    0x90(%rsp),%rcx
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1933
   2a77a:	movdqu %xmm1,0x2c(%rcx,%rax,1)
write<c30walk::Rec>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1933
   2a780:	movdqa %xmm1,0x20(%rcx,%rax,1)
   2a786:	movdqa %xmm1,0x10(%rcx,%rax,1)
   2a78c:	movdqa %xmm1,(%rcx,%rax,1)
   2a791:	mov    %r15d,0x3c(%rcx,%rax,1)
create_raw<false>():
./examples/c30walk/ctor.rs:373
   2a796:	mov    %r14,%rdi
   2a799:	shr    $0x6,%rdi
   2a79d:	cmp    0x18(%rsp),%rdi
   2a7a2:	jae    2be28 <<c30walk::ctor::Ctor>::build::<false>+0x1ab8>
   2a7a8:	mov    $0x1,%eax
   2a7ad:	mov    %r14d,%ecx
   2a7b0:	shl    %cl,%rax
   2a7b3:	mov    0x10(%rsp),%rcx
   2a7b8:	or     %rax,(%rcx,%rdi,8)
   2a7bc:	mov    %r14,%rcx
<c30walk::ctor::Ctor>::build::<false>:
   2a7bf:	add    $0x4,%rbp
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/non_null.rs:1714
   2a7c3:	cmp    $0x400,%rbp
next<u32>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/iter/macros.rs:180
   2a7ca:	je     2a858 <<c30walk::ctor::Ctor>::build::<false>+0x4e8>
<c30walk::ctor::Ctor>::build::<false>:
   2a7d0:	mov    %r13,%rax
scatter<false>():
./examples/c30walk/ctor.rs:841
   2a7d3:	mov    (%rbx,%rbp,1),%r13d
./examples/c30walk/ctor.rs:842
   2a7d7:	add    %rax,%r13
   2a7da:	cmp    %r9,%r13
   2a7dd:	setae  %sil
./examples/c30walk/ctor.rs:844
   2a7e1:	mov    %r13,%rax
   2a7e4:	mul    %r8
   2a7e7:	mov    %rdx,%r14
   2a7ea:	shr    $0x4,%r14
   2a7ee:	mov    %r13,%rax
   2a7f1:	imul   %r11,%rax
   2a7f5:	ror    $1,%rax
   2a7f8:	cmp    %r12,%rax
   2a7fb:	setb   %al
   2a7fe:	cmp    %rcx,%r14
   2a801:	sete   %dl
./examples/c30walk/ctor.rs:843
   2a804:	or     %sil,%dl
   2a807:	or     %al,%dl
   2a809:	jne    2a7bf <<c30walk::ctor::Ctor>::build::<false>+0x44f>
   2a80b:	mov    0x1150(%rsp),%rdi
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/mod.rs:3054
   2a813:	mov    0x10(%rdi),%r15
push_mut<c30walk::RawG, alloc::alloc::Global>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/mod.rs:1040
   2a817:	cmp    (%rdi),%r15
   2a81a:	jne    2a720 <<c30walk::ctor::Ctor>::build::<false>+0x3b0>
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/mod.rs:1041
   2a820:	call   57bc0 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   2a825:	pcmpeqd %xmm1,%xmm1
   2a829:	pxor   %xmm0,%xmm0
   2a82d:	movabs $0xeeeeeeeeeeeeeeef,%r11
   2a837:	movabs $0x8888888888888889,%r8
   2a841:	mov    0x28(%rsp),%r10
   2a846:	mov    0x20(%rsp),%r9
   2a84b:	mov    0x1150(%rsp),%rdi
   2a853:	jmp    2a720 <<c30walk::ctor::Ctor>::build::<false>+0x3b0>
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:734
   2a858:	cmpl   $0x0,0x3c(%rsp)
   2a85d:	mov    0x1150(%rsp),%r11
   2a865:	mov    0x48(%rsp),%rdi
   2a86a:	mov    0x18(%rsp),%rsi
   2a86f:	mov    0x10(%rsp),%r8
scatter<false>():
./examples/c30walk/ctor.rs:855
   2a874:	je     2aa06 <<c30walk::ctor::Ctor>::build::<false>+0x696>
   2a87a:	test   %rdi,%rdi
   2a87d:	je     2aa06 <<c30walk::ctor::Ctor>::build::<false>+0x696>
   2a883:	mov    %r9,%r14
   2a886:	shl    $0x20,%r14
   2a88a:	mov    0x3c(%rsp),%r15d
   2a88f:	xor    %ebx,%ebx
   2a891:	pxor   %xmm0,%xmm0
   2a895:	pcmpeqd %xmm1,%xmm1
   2a899:	mov    0x90(%rsp),%rbp
   2a8a1:	mov    %r14,0x8(%rsp)
   2a8a6:	jmp    2a930 <<c30walk::ctor::Ctor>::build::<false>+0x5c0>
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/raw_vec/mod.rs:614
   2a8ab:	mov    0x8(%r11),%rax
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mut_ptr.rs:961
   2a8af:	mov    %r15,%rcx
add<c30walk::RawG>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mut_ptr.rs:961
   2a8b2:	shl    $0x7,%rcx
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1933
   2a8b6:	movdqa %xmm0,0x70(%rax,%rcx,1)
write<c30walk::RawG>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1933
   2a8bc:	movdqa %xmm0,0x60(%rax,%rcx,1)
   2a8c2:	movdqa %xmm0,0x50(%rax,%rcx,1)
   2a8c8:	movdqa %xmm0,0x40(%rax,%rcx,1)
   2a8ce:	movdqa %xmm0,0x30(%rax,%rcx,1)
   2a8d4:	movdqa %xmm0,0x20(%rax,%rcx,1)
   2a8da:	movdqa %xmm0,0x10(%rax,%rcx,1)
   2a8e0:	movdqa %xmm0,(%rax,%rcx,1)
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/mod.rs:1046
   2a8e5:	mov    %r15,%rax
push_mut<c30walk::RawG, alloc::alloc::Global>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/mod.rs:1046
   2a8e8:	inc    %rax
   2a8eb:	mov    %rax,0x10(%r11)
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:371
   2a8ef:	or     $0x10000000,%r15d
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1933
   2a8f6:	movdqu %xmm1,0x2c(%rbp)
write<c30walk::Rec>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1933
   2a8fb:	movdqa %xmm1,0x20(%rbp)
   2a900:	movdqa %xmm1,0x10(%rbp)
   2a905:	movdqa %xmm1,0x0(%rbp)
   2a90a:	mov    %r15d,0x3c(%rbp)
create_raw<false>():
./examples/c30walk/ctor.rs:373
   2a90e:	or     %r12,(%r8,%r13,8)
   2a912:	mov    %r14,%r15
   2a915:	mov    0x8(%rsp),%r14
scatter<false>():
   2a91a:	nopw   0x0(%rax,%rax,1)
   2a920:	inc    %rbx
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/cmp.rs:1916
   2a923:	add    $0x40,%rbp
lt():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/cmp.rs:1916
   2a927:	cmp    %rbx,%rdi
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/range.rs:900
   2a92a:	je     2aa06 <<c30walk::ctor::Ctor>::build::<false>+0x696>
./examples/c30walk/ctor.rs:323
   2a930:	mov    %rbx,%r13
is_raw<false>():
./examples/c30walk/ctor.rs:323
   2a933:	shr    $0x6,%r13
   2a937:	cmp    %rsi,%r13
   2a93a:	jae    2be39 <<c30walk::ctor::Ctor>::build::<false>+0x1ac9>
   2a940:	mov    (%r8,%r13,8),%rax
scatter<false>():
./examples/c30walk/ctor.rs:857
   2a944:	mov    $0x1,%r12d
   2a94a:	mov    %ebx,%ecx
   2a94c:	shl    %cl,%r12
   2a94f:	bt     %rbx,%rax
   2a953:	jb     2a920 <<c30walk::ctor::Ctor>::build::<false>+0x5b0>
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:73
   2a955:	mov    %rbx,%rax
forced():
./examples/c30walk/ctor.rs:73
   2a958:	xor    %r14,%rax
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk.rs:749
   2a95b:	mov    %rax,%rcx
mix():
./examples/c30walk.rs:749
   2a95e:	shr    $0x21,%rcx
   2a962:	xor    %rax,%rcx
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/num/uint_macros.rs:2660
   2a965:	movabs $0xff51afd7ed558ccd,%rax
wrapping_mul():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/num/uint_macros.rs:2660
   2a96f:	imul   %rax,%rcx
mix():
./examples/c30walk.rs:751
   2a973:	mov    %rcx,%rax
   2a976:	shr    $0x21,%rax
   2a97a:	xor    %rcx,%rax
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/num/uint_macros.rs:2660
   2a97d:	movabs $0xc4ceb9fe1a85ec53,%rcx
wrapping_mul():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/num/uint_macros.rs:2660
   2a987:	imul   %rcx,%rax
mix():
./examples/c30walk.rs:753
   2a98b:	mov    %rax,%rcx
   2a98e:	shr    $0x21,%rcx
   2a992:	xor    %rax,%rcx
forced():
./examples/c30walk/ctor.rs:73
   2a995:	mov    %rcx,%rax
   2a998:	shr    $0x3,%rax
   2a99c:	movabs $0x20c49ba5e353f7cf,%rdx
   2a9a6:	mul    %rdx
   2a9a9:	shr    $0x4,%rdx
   2a9ad:	imul   $0x3e8,%rdx,%rax
   2a9b4:	sub    %rax,%rcx
   2a9b7:	cmp    %r15,%rcx
scatter<false>():
./examples/c30walk/ctor.rs:857
   2a9ba:	jae    2a920 <<c30walk::ctor::Ctor>::build::<false>+0x5b0>
   2a9c0:	mov    %r15,%r14
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/mod.rs:3054
   2a9c3:	mov    0x10(%r11),%r15
push_mut<c30walk::RawG, alloc::alloc::Global>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/mod.rs:1040
   2a9c7:	cmp    (%r11),%r15
   2a9ca:	jne    2a8ab <<c30walk::ctor::Ctor>::build::<false>+0x53b>
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/mod.rs:1041
   2a9d0:	mov    %r11,%rdi
   2a9d3:	call   57bc0 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   2a9d8:	pcmpeqd %xmm1,%xmm1
   2a9dc:	pxor   %xmm0,%xmm0
   2a9e0:	mov    0x10(%rsp),%r8
   2a9e5:	mov    0x18(%rsp),%rsi
   2a9ea:	mov    0x48(%rsp),%rdi
   2a9ef:	mov    0x28(%rsp),%r10
   2a9f4:	mov    0x20(%rsp),%r9
   2a9f9:	mov    0x1150(%rsp),%r11
   2aa01:	jmp    2a8ab <<c30walk::ctor::Ctor>::build::<false>+0x53b>
scatter<false>():
./examples/c30walk/ctor.rs:867
   2aa06:	cmp    $0x10,%r9
   2aa0a:	jae    2aa5f <<c30walk::ctor::Ctor>::build::<false>+0x6ef>
   2aa0c:	xor    %r12d,%r12d
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/non_null.rs:1714
   2aa0f:	cmp    %r9,%r12
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/iter/macros.rs:180
   2aa12:	jne    2b95f <<c30walk::ctor::Ctor>::build::<false>+0x15ef>
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/array/equality.rs:157
   2aa18:	mov    $0x400,%edx
spec_eq<u32, u32, 256>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/array/equality.rs:157
   2aa1d:	lea    0x108(%rsp),%rdi
   2aa25:	lea    0x508(%rsp),%rsi
   2aa2d:	call   *0xaba45(%rip)        # d6478 <bcmp@GLIBC_2.2.5>
   2aa33:	mov    0x1150(%rsp),%r11
   2aa3b:	test   %eax,%eax
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:776
   2aa3d:	je     2bb1d <<c30walk::ctor::Ctor>::build::<false>+0x17ad>
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/mod.rs:3035
   2aa43:	movq   $0x0,0x10(%r11)
   2aa4b:	xor    %eax,%eax
./examples/c30walk/ctor.rs:826
   2aa4d:	add    $0x1118,%rsp
   2aa54:	pop    %rbx
   2aa55:	pop    %r12
   2aa57:	pop    %r13
   2aa59:	pop    %r14
   2aa5b:	pop    %r15
   2aa5d:	pop    %rbp
   2aa5e:	ret
   2aa5f:	mov    $0x10,%r14d
   2aa65:	xor    %r12d,%r12d
   2aa68:	pxor   %xmm4,%xmm4
   2aa6c:	xor    %ebp,%ebp
   2aa6e:	jmp    2aa8b <<c30walk::ctor::Ctor>::build::<false>+0x71b>
   2aa70:	mov    0xb0(%rsp),%r12
scatter<false>():
./examples/c30walk/ctor.rs:867
   2aa78:	lea    0x10(%r12),%rax
   2aa7d:	xor    %ebp,%ebp
   2aa7f:	mov    %rax,%r14
   2aa82:	cmp    %r9,%rax
   2aa85:	ja     2bcda <<c30walk::ctor::Ctor>::build::<false>+0x196a>
   2aa8b:	mov    %r12,%rdx
   2aa8e:	mov    %r13,%rdi
./examples/c30walk/ctor.rs:868
   2aa91:	test   $0x1,%bpl
   2aa95:	jne    2aac3 <<c30walk::ctor::Ctor>::build::<false>+0x753>
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:77
   2aa97:	lea    0x8(%rdx),%rsi
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/num/uint_macros.rs:1050
   2aa9b:	cmp    $0xfffffffffffffff7,%rdx
checked_sub():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/num/uint_macros.rs:1050
   2aa9f:	ja     2bd0a <<c30walk::ctor::Ctor>::build::<false>+0x199a>
   2aaa5:	cmp    %r9,%rsi
   2aaa8:	ja     2bd0a <<c30walk::ctor::Ctor>::build::<false>+0x199a>
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:77
   2aaae:	lea    0x10(%rdx),%rax
rd():
./examples/c30walk/ctor.rs:77
   2aab2:	cmp    %r9,%rax
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/num/uint_macros.rs:1050
   2aab5:	ja     2bd1c <<c30walk::ctor::Ctor>::build::<false>+0x19ac>
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/result.rs:1718
   2aabb:	mov    (%r10,%rdx,1),%rbx
   2aabf:	mov    (%r10,%rsi,1),%rdi
scatter<false>():
./examples/c30walk/ctor.rs:872
   2aac3:	mov    %rbx,%rax
   2aac6:	xor    %rdi,%rax
   2aac9:	mov    %rbx,%rcx
   2aacc:	rol    $0x8,%rcx
   2aad0:	xor    %rbx,%rcx
   2aad3:	or     %rax,%rcx
   2aad6:	mov    %r14,0xb0(%rsp)
   2aade:	jne    2ab60 <<c30walk::ctor::Ctor>::build::<false>+0x7f0>
./examples/c30walk/ctor.rs:875
   2aae4:	mov    %r14,%rcx
   2aae7:	mov    %rdx,0x88(%rsp)
   2aaef:	sub    %rdx,%rcx
   2aaf2:	xor    %edx,%edx
   2aaf4:	mov    %r14,%rax
   2aaf7:	nopw   0x0(%rax,%rax,1)
   2ab00:	mov    %rax,%r12
   2ab03:	mov    %rdx,%r14
   2ab06:	mov    %rcx,%r15
   2ab09:	add    $0x10,%rax
   2ab0d:	cmp    %r9,%rax
   2ab10:	setbe  %bpl
   2ab14:	ja     2ad36 <<c30walk::ctor::Ctor>::build::<false>+0x9c6>
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:77
   2ab1a:	lea    0x8(%r12),%rsi
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/num/uint_macros.rs:1050
   2ab1f:	cmp    $0xfffffffffffffff7,%r12
checked_sub():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/num/uint_macros.rs:1050
   2ab23:	ja     2bcf8 <<c30walk::ctor::Ctor>::build::<false>+0x1988>
   2ab29:	cmp    %r9,%rsi
   2ab2c:	ja     2bcf8 <<c30walk::ctor::Ctor>::build::<false>+0x1988>
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/result.rs:1718
   2ab32:	mov    (%r10,%r12,1),%rbx
   2ab36:	mov    0x8(%r10,%r12,1),%r13
scatter<false>():
./examples/c30walk/ctor.rs:877
   2ab3b:	cmp    %rdi,%rbx
   2ab3e:	jne    2ab4d <<c30walk::ctor::Ctor>::build::<false>+0x7dd>
   2ab40:	lea    0x1(%r14),%rdx
   2ab44:	lea    0x10(%r15),%rcx
   2ab48:	cmp    %rdi,%r13
   2ab4b:	je     2ab00 <<c30walk::ctor::Ctor>::build::<false>+0x790>
   2ab4d:	lea    0x10(%r12),%rax
   2ab52:	mov    %rax,0x78(%rsp)
   2ab57:	jmp    2ad40 <<c30walk::ctor::Ctor>::build::<false>+0x9d0>
<c30walk::ctor::Ctor>::build::<false>:
   2ab5c:	nopl   0x0(%rax)
checked_sub():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/num/uint_macros.rs:1050
   2ab60:	cmp    $0xfffffffffffffff0,%rdx
   2ab64:	jae    2bd58 <<c30walk::ctor::Ctor>::build::<false>+0x19e8>
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/index.rs:89
   2ab6a:	add    %r10,%rdx
get_offset_len_noubcheck<u8>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/index.rs:89
   2ab6d:	mov    0xf8(%rsp),%rbx
   2ab75:	mov    0xe8(%rsp),%r8
   2ab7d:	mov    0xd8(%rsp),%rax
   2ab85:	mov    %rax,0x18(%rsp)
   2ab8a:	mov    0xe0(%rsp),%rax
   2ab92:	mov    %rax,0x10(%rsp)
   2ab97:	mov    0x910(%rsp),%r12d
   2ab9f:	mov    0x908(%rsp),%r15
   2aba7:	xor    %ebp,%ebp
   2aba9:	mov    %rbx,0x8(%rsp)
   2abae:	jmp    2abf9 <<c30walk::ctor::Ctor>::build::<false>+0x889>
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1933
   2abb0:	mov    %r14d,0x3c(%rax)
   2abb4:	movw   $0x0,(%rax)
   2abb9:	nopl   0x0(%rax)
advance<false>():
./examples/c30walk/ctor.rs:350
   2abc0:	lea    0x1(%r15),%rax
./examples/c30walk/ctor.rs:351
   2abc4:	add    $0xffffffffffffffe3,%r15
   2abc8:	xor    %ecx,%ecx
   2abca:	cmp    $0x1e,%rax
   2abce:	setae  %cl
   2abd1:	cmovb  %rax,%r15
   2abd5:	lea    (%r12,%rcx,2),%r12d
   2abd9:	inc    %r12d
   2abdc:	mov    %r15,0x908(%rsp)
   2abe4:	mov    %r12d,0x910(%rsp)
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/non_null.rs:656
   2abec:	inc    %rbp
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/non_null.rs:1714
   2abef:	cmp    $0x10,%rbp
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/iter/macros.rs:180
   2abf3:	je     2aa70 <<c30walk::ctor::Ctor>::build::<false>+0x700>
scatter<false>():
./examples/c30walk/ctor.rs:889
   2abf9:	movzbl (%rdx,%rbp,1),%eax
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:602
   2abfd:	mov    0x108(%rsp,%rax,4),%esi
one<false>():
./examples/c30walk/ctor.rs:603
   2ac04:	cmp    0x508(%rsp,%rax,4),%esi
   2ac0b:	jae    2aa43 <<c30walk::ctor::Ctor>::build::<false>+0x6d3>
./examples/c30walk/ctor.rs:607
   2ac11:	mov    %esi,%r13d
   2ac14:	and    $0x1f,%r13d
   2ac18:	xor    %ecx,%ecx
   2ac1a:	cmp    $0x1d,%r13d
   2ac1e:	sete   %cl
   2ac21:	lea    (%rsi,%rcx,2),%ecx
   2ac24:	inc    %ecx
   2ac26:	mov    %ecx,0x108(%rsp,%rax,4)
./examples/c30walk/ctor.rs:608
   2ac2d:	shr    $0x5,%esi
./examples/c30walk/ctor.rs:610
   2ac30:	mov    %r12d,%r14d
   2ac33:	shl    $0x8,%r14d
   2ac37:	or     %eax,%r14d
./examples/c30walk/ctor.rs:611
   2ac3a:	test   %r13d,%r13d
   2ac3d:	je     2ac90 <<c30walk::ctor::Ctor>::build::<false>+0x920>
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mut_ptr.rs:961
   2ac3f:	mov    %rsi,%rcx
add<c30walk::Rec>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mut_ptr.rs:961
   2ac42:	shl    $0x6,%rcx
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1733
   2ac46:	mov    0x3c(%rbx,%rcx,1),%eax
one<false>():
./examples/c30walk/ctor.rs:621
   2ac4a:	test   $0x10000000,%eax
   2ac4f:	jne    2acd0 <<c30walk::ctor::Ctor>::build::<false>+0x960>
   2ac51:	mov    %rdx,%rbx
./examples/c30walk/ctor.rs:625
   2ac54:	mov    %eax,%edi
   2ac56:	shr    $0x8,%edi
   2ac59:	mov    %r12d,%edx
   2ac5c:	sub    %edi,%edx
./examples/c30walk/ctor.rs:626
   2ac5e:	cmp    $0xfffe,%edx
   2ac64:	jae    2acfa <<c30walk::ctor::Ctor>::build::<false>+0x98a>
   2ac6a:	mov    0x8(%rsp),%rax
   2ac6f:	add    %rax,%rcx
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1933
   2ac72:	mov    %dx,(%rcx,%r13,2)
write<u16>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1933
   2ac77:	mov    %rbx,%rdx
   2ac7a:	mov    %rax,%rbx
one<false>():
./examples/c30walk/ctor.rs:611
   2ac7d:	jmp    2abc0 <<c30walk::ctor::Ctor>::build::<false>+0x850>
   2ac82:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:323
   2ac90:	mov    %esi,%edi
is_raw<false>():
./examples/c30walk/ctor.rs:323
   2ac92:	shr    $0x6,%edi
   2ac95:	cmp    %r8,%rdi
   2ac98:	jae    2be48 <<c30walk::ctor::Ctor>::build::<false>+0x1ad8>
   2ac9e:	mov    0x10(%rsp),%rax
   2aca3:	mov    (%rax,%rdi,8),%rcx
one<false>():
   2aca7:	mov    %rsi,%rax
   2acaa:	shl    $0x6,%rax
   2acae:	add    %rbx,%rax
./examples/c30walk/ctor.rs:612
   2acb1:	bt     %rsi,%rcx
   2acb5:	jae    2abb0 <<c30walk::ctor::Ctor>::build::<false>+0x840>
   2acbb:	mov    0x3c(%rax),%eax
./examples/c30walk/ctor.rs:613
   2acbe:	mov    $0xfffff,%ecx
   2acc3:	and    %ecx,%eax
   2acc5:	jmp    2acd5 <<c30walk::ctor::Ctor>::build::<false>+0x965>
   2acc7:	nopw   0x0(%rax,%rax,1)
./examples/c30walk/ctor.rs:622
   2acd0:	and    $0xfffff,%eax
./examples/c30walk/ctor.rs:635
   2acd5:	mov    %eax,%edi
   2acd7:	mov    0x18(%rsp),%rax
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/mod.rs:1903
   2acdc:	mov    0x10(%rax),%rsi
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/index.rs:278
   2ace0:	cmp    %rdi,%rsi
index_mut<c30walk::RawG>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/index.rs:278
   2ace3:	jbe    2bb11 <<c30walk::ctor::Ctor>::build::<false>+0x17a1>
   2ace9:	shl    $0x7,%rdi
   2aced:	add    0x8(%rax),%rdi
one<false>():
./examples/c30walk/ctor.rs:635
   2acf1:	mov    %r14d,(%rdi,%r13,4)
   2acf5:	jmp    2abc0 <<c30walk::ctor::Ctor>::build::<false>+0x850>
./examples/c30walk/ctor.rs:627
   2acfa:	lea    0xd8(%rsp),%rdi
   2ad02:	mov    %r13,%rdx
   2ad05:	mov    %eax,%ecx
   2ad07:	mov    %r8,0x60(%rsp)
   2ad0c:	call   585e0 <<c30walk::ctor::W<false>>::promote>
   2ad11:	mov    0x60(%rsp),%r8
   2ad16:	pxor   %xmm4,%xmm4
   2ad1a:	mov    0x28(%rsp),%r10
   2ad1f:	mov    0x20(%rsp),%r9
   2ad24:	mov    0x1150(%rsp),%r11
   2ad2c:	mov    %rbx,%rdx
   2ad2f:	mov    0x8(%rsp),%rbx
./examples/c30walk/ctor.rs:621
   2ad34:	jmp    2acd5 <<c30walk::ctor::Ctor>::build::<false>+0x965>
scatter<false>():
./examples/c30walk/ctor.rs:884
   2ad36:	lea    0x10(%r12),%rax
   2ad3b:	mov    %rax,0x78(%rsp)
   2ad40:	mov    0x88(%rsp),%rax
   2ad48:	mov    %r12,%rcx
   2ad4b:	sub    %rax,%rcx
   2ad4e:	mov    %rdi,0x70(%rsp)
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:536
   2ad53:	movzbl %dil,%esi
run<false>():
./examples/c30walk/ctor.rs:537
   2ad57:	mov    %esi,%r11d
   2ad5a:	mov    0x108(%rsp,%r11,4),%edi
./examples/c30walk/ctor.rs:538
   2ad62:	mov    %edi,%r8d
   2ad65:	and    $0x1f,%r8d
./examples/c30walk/ctor.rs:539
   2ad69:	lea    (%rcx,%r8,1),%rax
   2ad6d:	mov    %rax,0x10(%rsp)
./examples/c30walk/ctor.rs:541
   2ad72:	movabs $0x8888888888888889,%rdx
   2ad7c:	mul    %rdx
   2ad7f:	shr    $0x4,%rdx
./examples/c30walk/ctor.rs:542
   2ad83:	lea    (%rcx,%rdi,1),%eax
   2ad86:	mov    %rdx,0x60(%rsp)
   2ad8b:	lea    (%rax,%rdx,2),%eax
./examples/c30walk/ctor.rs:543
   2ad8e:	cmp    0x508(%rsp,%r11,4),%eax
   2ad96:	mov    0x1150(%rsp),%r11
   2ad9e:	ja     2aa43 <<c30walk::ctor::Ctor>::build::<false>+0x6d3>
./examples/c30walk/ctor.rs:546
   2ada4:	mov    %eax,0x108(%rsp,%rsi,4)
   2adab:	mov    %rdi,0xa8(%rsp)
./examples/c30walk/ctor.rs:554
   2adb3:	shr    $0x5,%edi
./examples/c30walk/ctor.rs:556
   2adb6:	test   %r8d,%r8d
   2adb9:	mov    %r8,0x40(%rsp)
   2adbe:	je     2aeca <<c30walk::ctor::Ctor>::build::<false>+0xb5a>
   2adc4:	mov    %rsi,0xb8(%rsp)
./examples/c30walk/ctor.rs:557
   2adcc:	mov    $0x1e,%edx
   2add1:	sub    %r8,%rdx
   2add4:	lea    (%rcx,%r8,1),%rax
   2add8:	cmp    $0x1e,%rax
   2addc:	cmovb  %rcx,%rdx
   2ade0:	mov    %rdx,0x30(%rsp)
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:426
   2ade5:	mov    0x908(%rsp),%rsi
fragment<false>():
./examples/c30walk/ctor.rs:426
   2aded:	cmp    $0x1e,%rsi
   2adf1:	jae    2be57 <<c30walk::ctor::Ctor>::build::<false>+0x1ae7>
   2adf7:	mov    %r8,%r10
   2adfa:	mov    %rsi,%rcx
   2adfd:	shl    $0x6,%rcx
   2ae01:	lea    -0x19448(%rip),%rax        # 119c0 <c30walk::ctor::PATTERNS>
   2ae08:	mov    %rcx,0x58(%rsp)
   2ae0d:	lea    (%rax,%rcx,1),%r9
./examples/c30walk/ctor.rs:427
   2ae11:	mov    0x910(%rsp),%eax
   2ae18:	mov    %eax,0x18(%rsp)
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:330
   2ae1c:	mov    0xf8(%rsp),%rcx
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mut_ptr.rs:961
   2ae24:	mov    %rdi,%rax
add<c30walk::Rec>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mut_ptr.rs:961
   2ae27:	shl    $0x6,%rax
   2ae2b:	mov    %rax,0x80(%rsp)
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1733
   2ae33:	mov    0x3c(%rcx,%rax,1),%eax
fragment<false>():
./examples/c30walk/ctor.rs:442
   2ae37:	test   $0x10000000,%eax
   2ae3c:	mov    %rdi,0x8(%rsp)
   2ae41:	mov    %rsi,0xc0(%rsp)
   2ae49:	mov    %r9,0x68(%rsp)
   2ae4e:	jne    2aedd <<c30walk::ctor::Ctor>::build::<false>+0xb6d>
   2ae54:	mov    0x30(%rsp),%r8
./examples/c30walk/ctor.rs:448
   2ae59:	lea    -0x1(%r8),%rdi
   2ae5d:	cmp    $0x1d,%rdi
   2ae61:	ja     2be8e <<c30walk::ctor::Ctor>::build::<false>+0x1b1e>
   2ae67:	mov    %eax,%edi
   2ae69:	shr    $0x8,%edi
   2ae6c:	mov    0x18(%rsp),%edx
   2ae70:	sub    %edi,%edx
   2ae72:	movzwl -0x2(%r9,%r8,2),%edi
   2ae78:	add    %edx,%edi
   2ae7a:	cmp    $0xfffd,%edi
   2ae80:	ja     2b92d <<c30walk::ctor::Ctor>::build::<false>+0x15bd>
   2ae86:	mov    0x10(%rsp),%rax
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/iter/macros.rs:180
   2ae8b:	cmp    $0x1e,%rax
next<u16>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/iter/macros.rs:180
   2ae8f:	mov    $0x1e,%edi
   2ae94:	cmovb  %rax,%rdi
   2ae98:	mov    %r10,%rax
   2ae9b:	not    %rax
   2ae9e:	add    %rdi,%rax
   2aea1:	movabs $0x7fffffffffffffff,%rdi
   2aeab:	and    %rdi,%rax
   2aeae:	cmp    $0x3,%rax
   2aeb2:	jae    2afc0 <<c30walk::ctor::Ctor>::build::<false>+0xc50>
   2aeb8:	xor    %eax,%eax
   2aeba:	mov    %r9,%r10
   2aebd:	mov    0x80(%rsp),%r9
   2aec5:	jmp    2b27f <<c30walk::ctor::Ctor>::build::<false>+0xf0f>
run<false>():
   2aeca:	mov    0x60(%rsp),%rcx
./examples/c30walk/ctor.rs:572
   2aecf:	test   %rcx,%rcx
   2aed2:	jne    2b32c <<c30walk::ctor::Ctor>::build::<false>+0xfbc>
   2aed8:	jmp    2b373 <<c30walk::ctor::Ctor>::build::<false>+0x1003>
fragment<false>():
./examples/c30walk/ctor.rs:443
   2aedd:	and    $0xfffff,%eax
   2aee2:	mov    0x30(%rsp),%r8
./examples/c30walk/ctor.rs:464
   2aee7:	mov    0xd8(%rsp),%rcx
   2aeef:	mov    %eax,%edi
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/mod.rs:1903
   2aef1:	mov    0x10(%rcx),%rsi
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/index.rs:278
   2aef5:	cmp    %rdi,%rsi
index_mut<c30walk::RawG>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/index.rs:278
   2aef8:	jbe    2be9f <<c30walk::ctor::Ctor>::build::<false>+0x1b2f>
fragment<false>():
./examples/c30walk/ctor.rs:464
   2aefe:	lea    (%r8,%r10,1),%rsi
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/num/uint_macros.rs:1050
   2af02:	cmp    %r10,%rsi
checked_sub():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/num/uint_macros.rs:1050
   2af05:	jb     2bd9e <<c30walk::ctor::Ctor>::build::<false>+0x1a2e>
   2af0b:	cmp    $0x21,%rsi
   2af0f:	jae    2bd9e <<c30walk::ctor::Ctor>::build::<false>+0x1a2e>
fragment<false>():
   2af15:	cmp    $0x1f,%r8
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/num/uint_macros.rs:1050
   2af19:	jae    2bdb2 <<c30walk::ctor::Ctor>::build::<false>+0x1a42>
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/adapters/zip.rs:304
   2af1f:	test   %r8,%r8
next<core::slice::iter::IterMut<u32>, core::slice::iter::Iter<u16>>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/adapters/zip.rs:304
   2af22:	je     2b2b7 <<c30walk::ctor::Ctor>::build::<false>+0xf47>
   2af28:	mov    %r8,%r10
   2af2b:	mov    0x8(%rcx),%r8
   2af2f:	shl    $0x7,%rdi
   2af33:	lea    (%r8,%rdi,1),%rdx
   2af37:	mov    0x40(%rsp),%r9
   2af3c:	lea    (%rdx,%r9,4),%rax
   2af40:	movzbl 0x70(%rsp),%ecx
   2af45:	cmp    $0x8,%r10
   2af49:	jb     2af7c <<c30walk::ctor::Ctor>::build::<false>+0xc0c>
   2af4b:	add    %r9d,%r9d
   2af4e:	mov    0x58(%rsp),%r10
   2af53:	sub    %r9,%r10
   2af56:	lea    -0x1959d(%rip),%r9        # 119c0 <c30walk::ctor::PATTERNS>
   2af5d:	add    %r9,%r10
   2af60:	lea    (%r10,%rsi,2),%r9
   2af64:	cmp    %r9,%rax
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/adapters/zip.rs:308
   2af67:	jae    2b01f <<c30walk::ctor::Ctor>::build::<false>+0xcaf>
   2af6d:	lea    (%rdx,%rsi,4),%rdx
   2af71:	cmp    %rdx,0x68(%rsp)
   2af76:	jae    2b01f <<c30walk::ctor::Ctor>::build::<false>+0xcaf>
   2af7c:	xor    %edx,%edx
   2af7e:	mov    0x30(%rsp),%r8
   2af83:	mov    0x58(%rsp),%rsi
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/adapters/zip.rs:304
   2af88:	lea    (%rsi,%rdx,2),%rsi
   2af8c:	lea    -0x195d3(%rip),%rdi        # 119c0 <c30walk::ctor::PATTERNS>
   2af93:	add    %rdi,%rsi
fragment<false>():
   2af96:	cs nopw 0x0(%rax,%rax,1)
./examples/c30walk/ctor.rs:465
   2afa0:	movzwl (%rsi),%edi
./examples/c30walk/ctor.rs:466
   2afa3:	add    0x18(%rsp),%edi
   2afa7:	shl    $0x8,%edi
   2afaa:	or     %ecx,%edi
   2afac:	mov    %edi,(%rax,%rdx,4)
next<core::slice::iter::IterMut<u32>, core::slice::iter::Iter<u16>>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/adapters/zip.rs:308
   2afaf:	inc    %rdx
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/adapters/zip.rs:304
   2afb2:	add    $0x2,%rsi
   2afb6:	cmp    %r8,%rdx
   2afb9:	jb     2afa0 <<c30walk::ctor::Ctor>::build::<false>+0xc30>
   2afbb:	jmp    2b2b7 <<c30walk::ctor::Ctor>::build::<false>+0xf47>
next<u16>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/iter/macros.rs:180
   2afc0:	mov    %rcx,%rdi
   2afc3:	lea    -0x1960a(%rip),%r9        # 119c0 <c30walk::ctor::PATTERNS>
   2afca:	sub    %r9,%rdi
   2afcd:	mov    0x80(%rsp),%r9
   2afd5:	add    %r9,%rdi
   2afd8:	lea    (%r10,%r10,1),%esi
   2afdc:	add    %rsi,%rdi
   2afdf:	sub    0x58(%rsp),%rdi
   2afe4:	cmp    $0x20,%rdi
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/adapters/enumerate.rs:82
   2afe8:	jae    2aff6 <<c30walk::ctor::Ctor>::build::<false>+0xc86>
next<core::slice::iter::Iter<u16>>():
   2afea:	xor    %eax,%eax
   2afec:	mov    0x68(%rsp),%r10
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/adapters/enumerate.rs:82
   2aff1:	jmp    2b27f <<c30walk::ctor::Ctor>::build::<false>+0xf0f>
   2aff6:	mov    %rsi,0x98(%rsp)
   2affe:	lea    0x1(%rax),%rsi
next<u16>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/iter/macros.rs:180
   2b002:	cmp    $0xf,%rax
   2b006:	mov    %ebp,0x38(%rsp)
   2b00a:	mov    %rsi,0xa0(%rsp)
   2b012:	jae    2b0d8 <<c30walk::ctor::Ctor>::build::<false>+0xd68>
   2b018:	xor    %eax,%eax
   2b01a:	jmp    2b1b6 <<c30walk::ctor::Ctor>::build::<false>+0xe46>
fragment<false>():
   2b01f:	mov    0x30(%rsp),%rdx
   2b024:	and    $0x18,%edx
   2b027:	movd   %ecx,%xmm0
   2b02b:	pshufd $0x0,%xmm0,%xmm0
   2b030:	movd   0x18(%rsp),%xmm1
   2b036:	pshufd $0x0,%xmm1,%xmm1
   2b03b:	mov    0x40(%rsp),%r10
next<core::slice::iter::IterMut<u32>, core::slice::iter::Iter<u16>>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/adapters/zip.rs:308
   2b040:	lea    (%r10,%r15,1),%rsi
   2b044:	cmp    $0x1e,%rsi
   2b048:	mov    $0x1e,%r9d
   2b04e:	cmovae %r9,%rsi
   2b052:	sub    %r10d,%esi
   2b055:	and    $0x18,%esi
   2b058:	lea    -0x1969f(%rip),%r9        # 119c0 <c30walk::ctor::PATTERNS>
   2b05f:	mov    0x58(%rsp),%r10
   2b064:	add    %r10,%r9
   2b067:	add    $0x8,%r9
   2b06b:	mov    0x40(%rsp),%r10
   2b070:	lea    (%rdi,%r10,4),%rdi
   2b074:	add    %r8,%rdi
   2b077:	add    $0x10,%rdi
   2b07b:	xor    %r8d,%r8d
fragment<false>():
   2b07e:	xchg   %ax,%ax
./examples/c30walk/ctor.rs:465
   2b080:	movq   -0x8(%r9,%r8,2),%xmm2
   2b087:	movq   (%r9,%r8,2),%xmm3
./examples/c30walk/ctor.rs:466
   2b08d:	punpcklwd %xmm4,%xmm2
   2b091:	punpcklwd %xmm4,%xmm3
   2b095:	paddd  %xmm1,%xmm2
   2b099:	paddd  %xmm1,%xmm3
   2b09d:	pslld  $0x8,%xmm2
   2b0a2:	pslld  $0x8,%xmm3
   2b0a7:	por    %xmm0,%xmm2
   2b0ab:	por    %xmm0,%xmm3
   2b0af:	movdqu %xmm2,-0x10(%rdi,%r8,4)
   2b0b6:	movdqu %xmm3,(%rdi,%r8,4)
next<core::slice::iter::IterMut<u32>, core::slice::iter::Iter<u16>>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/adapters/zip.rs:308
   2b0bc:	add    $0x8,%r8
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/adapters/zip.rs:304
   2b0c0:	cmp    %r8,%rsi
   2b0c3:	jne    2b080 <<c30walk::ctor::Ctor>::build::<false>+0xd10>
   2b0c5:	mov    0x30(%rsp),%r8
   2b0ca:	cmp    %rdx,%r8
   2b0cd:	jne    2af83 <<c30walk::ctor::Ctor>::build::<false>+0xc13>
   2b0d3:	jmp    2b2b7 <<c30walk::ctor::Ctor>::build::<false>+0xf47>
fragment<false>():
   2b0d8:	mov    %rsi,%rax
   2b0db:	and    $0xfffffffffffffff0,%rax
   2b0df:	movd   %edx,%xmm0
   2b0e3:	pshuflw $0x0,%xmm0,%xmm0
   2b0e8:	pshufd $0x44,%xmm0,%xmm0
next<u16>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/iter/macros.rs:180
   2b0ed:	lea    -0x19734(%rip),%rdi        # 119c0 <c30walk::ctor::PATTERNS>
   2b0f4:	mov    %r10,%rsi
   2b0f7:	mov    0x58(%rsp),%r10
   2b0fc:	lea    0x10(%r10,%rdi,1),%r10
   2b101:	mov    0xa8(%rsp),%r11
   2b109:	not    %r11d
   2b10c:	lea    (%rsi,%r15,1),%rdi
   2b110:	cmp    $0x1e,%rdi
   2b114:	mov    $0x1e,%ebp
   2b119:	cmovae %rbp,%rdi
   2b11d:	movabs $0x7fffffffffffffe0,%rbp
   2b127:	or     %rbp,%r11
   2b12a:	add    %rdi,%r11
   2b12d:	movabs $0x7fffffffffffffff,%rdi
   2b137:	and    %rdi,%r11
   2b13a:	inc    %r11
   2b13d:	and    $0xfffffffffffffff0,%r11
   2b141:	mov    0x98(%rsp),%rsi
   2b149:	lea    (%r9,%rsi,1),%rdi
   2b14d:	lea    (%rcx,%rdi,1),%rbp
   2b151:	add    $0x10,%rbp
   2b155:	xor    %edi,%edi
fragment<false>():
   2b157:	nopw   0x0(%rax,%rax,1)
   2b160:	movdqa -0x10(%r10,%rdi,2),%xmm1
./examples/c30walk/ctor.rs:457
   2b167:	paddw  %xmm0,%xmm1
   2b16b:	movdqa (%r10,%rdi,2),%xmm2
   2b171:	paddw  %xmm0,%xmm2
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1933
   2b175:	movdqu %xmm1,-0x10(%rbp,%rdi,2)
write<u16>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1933
   2b17b:	movdqu %xmm2,0x0(%rbp,%rdi,2)
next<core::slice::iter::Iter<u16>>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/adapters/enumerate.rs:82
   2b181:	add    $0x10,%rdi
next<u16>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/iter/macros.rs:180
   2b185:	cmp    %rdi,%r11
   2b188:	jne    2b160 <<c30walk::ctor::Ctor>::build::<false>+0xdf0>
   2b18a:	mov    0xa0(%rsp),%rsi
   2b192:	cmp    %rax,%rsi
   2b195:	mov    0x1150(%rsp),%r11
   2b19d:	mov    0x38(%rsp),%ebp
   2b1a1:	je     2b2b7 <<c30walk::ctor::Ctor>::build::<false>+0xf47>
fragment<false>():
   2b1a7:	test   $0xc,%sil
   2b1ab:	je     2b916 <<c30walk::ctor::Ctor>::build::<false>+0x15a6>
   2b1b1:	mov    0x40(%rsp),%r10
   2b1b6:	mov    %rax,%rbp
   2b1b9:	mov    %rsi,%rax
   2b1bc:	and    $0xfffffffffffffffc,%rax
   2b1c0:	mov    0x58(%rsp),%rsi
   2b1c5:	lea    (%rsi,%rbp,2),%r11
   2b1c9:	lea    -0x19810(%rip),%rdi        # 119c0 <c30walk::ctor::PATTERNS>
   2b1d0:	add    %rdi,%r11
   2b1d3:	mov    0xa8(%rsp),%r9
   2b1db:	not    %r9d
   2b1de:	lea    (%r10,%r15,1),%rdi
   2b1e2:	cmp    $0x1e,%rdi
   2b1e6:	mov    $0x1e,%r10d
   2b1ec:	cmovae %r10,%rdi
   2b1f0:	movabs $0x7fffffffffffffe0,%r10
   2b1fa:	or     %r10,%r9
   2b1fd:	add    %rdi,%r9
   2b200:	mov    0x68(%rsp),%rsi
   2b205:	lea    (%rsi,%rax,2),%r10
   2b209:	mov    %r9,%rsi
   2b20c:	movd   %edx,%xmm0
   2b210:	pshuflw $0x0,%xmm0,%xmm0
   2b215:	movabs $0x7fffffffffffffff,%rdi
   2b21f:	and    %rdi,%rsi
   2b222:	inc    %rsi
   2b225:	and    $0xfffffffffffffffc,%rsi
   2b229:	sub    %rbp,%rsi
   2b22c:	mov    0x80(%rsp),%r9
   2b234:	lea    (%r9,%rbp,2),%rbp
   2b238:	add    0x98(%rsp),%rbp
   2b240:	add    %rcx,%rbp
   2b243:	xor    %edi,%edi
   2b245:	data16 cs nopw 0x0(%rax,%rax,1)
./examples/c30walk/ctor.rs:456
   2b250:	movq   (%r11,%rdi,2),%xmm1
./examples/c30walk/ctor.rs:457
   2b256:	paddw  %xmm0,%xmm1
write<u16>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1933
   2b25a:	movq   %xmm1,0x0(%rbp,%rdi,2)
next<u16>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/iter/macros.rs:180
   2b260:	add    $0x4,%rdi
   2b264:	cmp    %rdi,%rsi
   2b267:	jne    2b250 <<c30walk::ctor::Ctor>::build::<false>+0xee0>
   2b269:	cmp    %rax,0xa0(%rsp)
   2b271:	mov    0x1150(%rsp),%r11
   2b279:	mov    0x38(%rsp),%ebp
   2b27d:	je     2b2b7 <<c30walk::ctor::Ctor>::build::<false>+0xf47>
fragment<false>():
   2b27f:	mov    0x68(%rsp),%rsi
./examples/c30walk/ctor.rs:456
   2b284:	lea    (%rsi,%r8,2),%rsi
next<u16>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/iter/macros.rs:180
   2b288:	lea    (%r9,%rax,2),%rax
   2b28c:	mov    0x40(%rsp),%rdi
   2b291:	lea    (%rax,%rdi,2),%rax
   2b295:	add    %rax,%rcx
fragment<false>():
   2b298:	nopl   0x0(%rax,%rax,1)
   2b2a0:	movzwl (%r10),%eax
./examples/c30walk/ctor.rs:457
   2b2a4:	add    %dx,%ax
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/non_null.rs:656
   2b2a7:	add    $0x2,%r10
write<u16>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1933
   2b2ab:	mov    %ax,(%rcx)
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/non_null.rs:1714
   2b2ae:	add    $0x2,%rcx
eq<u16>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/non_null.rs:1714
   2b2b2:	cmp    %rsi,%r10
next<u16>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/iter/macros.rs:180
   2b2b5:	jne    2b2a0 <<c30walk::ctor::Ctor>::build::<false>+0xf30>
fragment<false>():
   2b2b7:	mov    0xc0(%rsp),%rdx
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:350
   2b2bf:	add    %r8,%rdx
advance<false>():
./examples/c30walk/ctor.rs:351
   2b2c2:	lea    0x2(%r8),%eax
   2b2c6:	mov    %rdx,%rcx
   2b2c9:	sub    $0x1e,%rcx
   2b2cd:	cmovb  %rdx,%rcx
   2b2d1:	cmovb  %r8d,%eax
   2b2d5:	add    0x18(%rsp),%eax
   2b2d9:	mov    %rcx,0x908(%rsp)
   2b2e1:	mov    %eax,0x910(%rsp)
run<false>():
./examples/c30walk/ctor.rs:557
   2b2e8:	cmpq   $0x1e,0x10(%rsp)
./examples/c30walk/ctor.rs:566
   2b2ee:	jae    2b304 <<c30walk::ctor::Ctor>::build::<false>+0xf94>
   2b2f0:	mov    0x20(%rsp),%r9
   2b2f5:	mov    0x28(%rsp),%r10
   2b2fa:	mov    0x78(%rsp),%rax
./examples/c30walk/ctor.rs:581
   2b2ff:	jmp    2aa7f <<c30walk::ctor::Ctor>::build::<false>+0x70f>
   2b304:	mov    0x8(%rsp),%rdi
./examples/c30walk/ctor.rs:569
   2b309:	inc    %rdi
   2b30c:	mov    0x60(%rsp),%rax
./examples/c30walk/ctor.rs:570
   2b311:	lea    -0x1(%rax),%rcx
   2b315:	mov    0x20(%rsp),%r9
   2b31a:	mov    0x28(%rsp),%r10
   2b31f:	mov    0xb8(%rsp),%rsi
./examples/c30walk/ctor.rs:572
   2b327:	test   %rcx,%rcx
   2b32a:	je     2b373 <<c30walk::ctor::Ctor>::build::<false>+0x1003>
<c30walk::ctor::Ctor>::build::<false>:
   2b32c:	mov    %rdi,0x8(%rsp)
./examples/c30walk/ctor.rs:734
   2b331:	cmpl   $0x0,0x3c(%rsp)
   2b336:	mov    %rcx,0x18(%rsp)
run<false>():
./examples/c30walk/ctor.rs:573
   2b33b:	je     2b4f0 <<c30walk::ctor::Ctor>::build::<false>+0x1180>
./examples/c30walk/ctor.rs:574
   2b341:	lea    0xd8(%rsp),%rdi
   2b349:	mov    0x8(%rsp),%rdx
   2b34e:	call   58220 <<c30walk::ctor::W<false>>::complete_checked>
   2b353:	mov    0x8(%rsp),%rdi
   2b358:	pxor   %xmm4,%xmm4
   2b35c:	mov    0x28(%rsp),%r10
   2b361:	mov    0x20(%rsp),%r9
   2b366:	mov    0x1150(%rsp),%r11
./examples/c30walk/ctor.rs:578
   2b36e:	add    0x18(%rsp),%rdi
./examples/c30walk/ctor.rs:580
   2b373:	imul   $0xffffffffffffffe2,0x60(%rsp),%rdx
./examples/c30walk/ctor.rs:581
   2b379:	mov    %rdx,%rsi
   2b37c:	add    0x10(%rsp),%rsi
   2b381:	je     2b737 <<c30walk::ctor::Ctor>::build::<false>+0x13c7>
   2b387:	mov    %rdi,%r9
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:426
   2b38a:	mov    0x908(%rsp),%rdi
fragment<false>():
./examples/c30walk/ctor.rs:426
   2b392:	cmp    $0x1d,%rdi
   2b396:	ja     2be6b <<c30walk::ctor::Ctor>::build::<false>+0x1afb>
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:323
   2b39c:	mov    %r9,%rax
is_raw<false>():
./examples/c30walk/ctor.rs:323
   2b39f:	shr    $0x6,%rax
   2b3a3:	mov    0xe8(%rsp),%rcx
   2b3ab:	cmp    %rcx,%rax
   2b3ae:	mov    0x40(%rsp),%r11
   2b3b3:	jae    2be7c <<c30walk::ctor::Ctor>::build::<false>+0x1b0c>
fragment<false>():
   2b3b9:	mov    %rdi,0x18(%rsp)
   2b3be:	mov    %rdi,%r8
   2b3c1:	shl    $0x6,%r8
   2b3c5:	lea    -0x19a0c(%rip),%rcx        # 119c0 <c30walk::ctor::PATTERNS>
   2b3cc:	mov    %r8,%r10
   2b3cf:	lea    (%rcx,%r8,1),%rdi
   2b3d3:	mov    0x910(%rsp),%r8d
is_raw<false>():
./examples/c30walk/ctor.rs:323
   2b3db:	mov    0xe0(%rsp),%rcx
   2b3e3:	mov    (%rcx,%rax,8),%rax
fragment<false>():
./examples/c30walk/ctor.rs:429
   2b3e7:	bt     %r9,%rax
   2b3eb:	mov    %ebp,0x38(%rsp)
   2b3ef:	jae    2b548 <<c30walk::ctor::Ctor>::build::<false>+0x11d8>
./examples/c30walk/ctor.rs:464
   2b3f5:	mov    0xd8(%rsp),%r11
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:330
   2b3fd:	mov    0xf8(%rsp),%rax
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mut_ptr.rs:961
   2b405:	shl    $0x6,%r9
add<c30walk::Rec>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mut_ptr.rs:961
   2b409:	mov    0x3c(%rax,%r9,1),%ecx
fragment<false>():
./examples/c30walk/ctor.rs:430
   2b40e:	and    $0xfffff,%ecx
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/mod.rs:1903
   2b414:	mov    0x10(%r11),%rax
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/index.rs:278
   2b418:	cmp    %rcx,%rax
index_mut<c30walk::RawG>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/index.rs:278
   2b41b:	jbe    2beab <<c30walk::ctor::Ctor>::build::<false>+0x1b3b>
fragment<false>():
   2b421:	cmp    $0x21,%rsi
   2b425:	mov    %r10,0x10(%rsp)
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/num/uint_macros.rs:1050
   2b42a:	jae    2bdc8 <<c30walk::ctor::Ctor>::build::<false>+0x1a58>
fragment<false>():
   2b430:	cmp    $0x1e,%rsi
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/num/uint_macros.rs:1050
   2b434:	ja     2bddb <<c30walk::ctor::Ctor>::build::<false>+0x1a6b>
fragment<false>():
   2b43a:	mov    %rdi,%r9
   2b43d:	mov    0x8(%r11),%rdi
   2b441:	mov    %ecx,%r11d
   2b444:	shl    $0x7,%r11d
   2b448:	add    %rdi,%r11
   2b44b:	movzbl 0x70(%rsp),%ebp
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/adapters/zip.rs:304
   2b450:	cmp    $0xc,%rsi
next<core::slice::iter::IterMut<u32>, core::slice::iter::Iter<u16>>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/adapters/zip.rs:304
   2b454:	jb     2b4b7 <<c30walk::ctor::Ctor>::build::<false>+0x1147>
   2b456:	mov    0x88(%rsp),%r10
   2b45e:	not    %r10
   2b461:	mov    0xb0(%rsp),%rax
   2b469:	add    %r10,%rax
   2b46c:	mov    0x40(%rsp),%r10
   2b471:	shl    $0x4,%r14
   2b475:	add    %rax,%r14
   2b478:	add    %r10,%r14
   2b47b:	add    %r14,%rdx
   2b47e:	lea    -0x19ac5(%rip),%rax        # 119c0 <c30walk::ctor::PATTERNS>
   2b485:	mov    0x10(%rsp),%r14
   2b48a:	add    %r14,%rax
   2b48d:	lea    (%rax,%rdx,2),%rax
   2b491:	add    $0x2,%rax
   2b495:	cmp    %rax,%r11
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/adapters/zip.rs:308
   2b498:	jae    2b741 <<c30walk::ctor::Ctor>::build::<false>+0x13d1>
   2b49e:	mov    %ecx,%eax
   2b4a0:	shl    $0x7,%eax
   2b4a3:	add    %rdi,%rax
   2b4a6:	lea    (%rax,%rdx,4),%rax
   2b4aa:	add    $0x4,%rax
   2b4ae:	cmp    %rax,%r9
   2b4b1:	jae    2b741 <<c30walk::ctor::Ctor>::build::<false>+0x13d1>
   2b4b7:	xor    %r10d,%r10d
   2b4ba:	mov    0x10(%rsp),%rax
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/adapters/zip.rs:304
   2b4bf:	lea    (%rax,%r10,2),%rax
   2b4c3:	lea    -0x19b0a(%rip),%rcx        # 119c0 <c30walk::ctor::PATTERNS>
   2b4ca:	add    %rcx,%rax
fragment<false>():
   2b4cd:	nopl   (%rax)
./examples/c30walk/ctor.rs:465
   2b4d0:	movzwl (%rax),%ecx
./examples/c30walk/ctor.rs:466
   2b4d3:	add    %r8d,%ecx
   2b4d6:	shl    $0x8,%ecx
   2b4d9:	or     %ebp,%ecx
   2b4db:	mov    %ecx,(%r11,%r10,4)
next<core::slice::iter::IterMut<u32>, core::slice::iter::Iter<u16>>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/adapters/zip.rs:308
   2b4df:	inc    %r10
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/adapters/zip.rs:304
   2b4e2:	add    $0x2,%rax
   2b4e6:	cmp    %rsi,%r10
   2b4e9:	jb     2b4d0 <<c30walk::ctor::Ctor>::build::<false>+0x1160>
   2b4eb:	jmp    2b5e5 <<c30walk::ctor::Ctor>::build::<false>+0x1275>
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:485
   2b4f0:	mov    0x908(%rsp),%rdi
complete<false>():
./examples/c30walk/ctor.rs:485
   2b4f8:	cmp    $0x1d,%rdi
   2b4fc:	ja     2bebd <<c30walk::ctor::Ctor>::build::<false>+0x1b4d>
   2b502:	shl    $0x6,%rdi
   2b506:	lea    -0x19b4d(%rip),%rax        # 119c0 <c30walk::ctor::PATTERNS>
   2b50d:	add    %rax,%rdi
./examples/c30walk/ctor.rs:486
   2b510:	mov    0x910(%rsp),%edx
   2b517:	mov    %edx,0x30(%rsp)
   2b51b:	shl    $0x8,%edx
   2b51e:	movzbl 0x70(%rsp),%eax
   2b523:	or     %edx,%eax
./examples/c30walk/ctor.rs:487
   2b525:	mov    0xf8(%rsp),%rdx
   2b52d:	mov    0x8(%rsp),%rsi
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mut_ptr.rs:961
   2b532:	shl    $0x6,%rsi
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/range.rs:900
   2b536:	cmp    $0x1,%rcx
spec_next<usize>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/range.rs:900
   2b53a:	jne    2b630 <<c30walk::ctor::Ctor>::build::<false>+0x12c0>
   2b540:	xor    %r8d,%r8d
   2b543:	jmp    2b6cc <<c30walk::ctor::Ctor>::build::<false>+0x135c>
fragment<false>():
./examples/c30walk/ctor.rs:433
   2b548:	mov    %r8d,%eax
   2b54b:	shl    $0x8,%eax
   2b54e:	movzbl 0x70(%rsp),%edx
   2b553:	or     %eax,%edx
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:337
   2b555:	mov    0xf8(%rsp),%rcx
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mut_ptr.rs:961
   2b55d:	shl    $0x6,%r9
add<c30walk::Rec>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mut_ptr.rs:961
   2b561:	mov    %r9,0x8(%rsp)
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1933
   2b566:	mov    %edx,0x3c(%rcx,%r9,1)
write<u32>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1933
   2b56b:	cmp    $0x1f,%rsi
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/num/uint_macros.rs:1050
   2b56f:	jae    2bdee <<c30walk::ctor::Ctor>::build::<false>+0x1a7e>
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/iter/macros.rs:180
   2b575:	imul   $0xffffffffffffffc4,0x60(%rsp),%rax
next<u16>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/iter/macros.rs:180
   2b57b:	mov    0x10(%rsp),%rdx
   2b580:	lea    (%rax,%rdx,2),%rax
   2b584:	add    $0xfffffffffffffffe,%rax
   2b588:	cmp    $0x6,%rax
   2b58c:	jb     2b5b3 <<c30walk::ctor::Ctor>::build::<false>+0x1243>
   2b58e:	mov    %r10,%r9
   2b591:	mov    %r11,%r10
   2b594:	mov    %rcx,%rdx
   2b597:	lea    -0x19bde(%rip),%r11        # 119c0 <c30walk::ctor::PATTERNS>
   2b59e:	sub    %r11,%rdx
   2b5a1:	add    0x8(%rsp),%rdx
   2b5a6:	sub    %r9,%rdx
   2b5a9:	cmp    $0x20,%rdx
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/adapters/enumerate.rs:82
   2b5ad:	jae    2b7f3 <<c30walk::ctor::Ctor>::build::<false>+0x1483>
fragment<false>():
   2b5b3:	mov    %rdi,%rax
   2b5b6:	xor    %r14d,%r14d
./examples/c30walk/ctor.rs:435
   2b5b9:	lea    (%rdi,%rsi,2),%rdx
   2b5bd:	mov    0x8(%rsp),%r9
next<u16>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/iter/macros.rs:180
   2b5c2:	lea    (%r9,%r14,2),%r9
   2b5c6:	add    %r9,%rcx
fragment<false>():
   2b5c9:	nopl   0x0(%rax)
./examples/c30walk/ctor.rs:435
   2b5d0:	movzwl (%rax),%r9d
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/non_null.rs:656
   2b5d4:	add    $0x2,%rax
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1933
   2b5d8:	mov    %r9w,(%rcx)
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/non_null.rs:1714
   2b5dc:	add    $0x2,%rcx
eq<u16>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/non_null.rs:1714
   2b5e0:	cmp    %rdx,%rax
next<u16>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/iter/macros.rs:180
   2b5e3:	jne    2b5d0 <<c30walk::ctor::Ctor>::build::<false>+0x1260>
fragment<false>():
   2b5e5:	mov    0x18(%rsp),%rdx
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:350
   2b5ea:	add    %rsi,%rdx
advance<false>():
./examples/c30walk/ctor.rs:351
   2b5ed:	lea    0x2(%rsi),%eax
   2b5f0:	mov    %rdx,%rcx
   2b5f3:	sub    $0x1e,%rcx
   2b5f7:	cmovb  %rdx,%rcx
   2b5fb:	cmovb  %esi,%eax
   2b5fe:	add    %r8d,%eax
   2b601:	mov    %rcx,0x908(%rsp)
   2b609:	mov    %eax,0x910(%rsp)
   2b610:	mov    0x1150(%rsp),%r11
   2b618:	mov    0x20(%rsp),%r9
   2b61d:	mov    0x28(%rsp),%r10
   2b622:	mov    0x38(%rsp),%ebp
   2b626:	mov    0x78(%rsp),%rax
run<false>():
./examples/c30walk/ctor.rs:581
   2b62b:	jmp    2aa7f <<c30walk::ctor::Ctor>::build::<false>+0x70f>
spec_next<usize>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/range.rs:900
   2b630:	mov    %rcx,%r9
   2b633:	and    $0xfffffffffffffffe,%r9
   2b637:	lea    (%rdx,%rsi,1),%r10
   2b63b:	add    $0x7c,%r10
   2b63f:	xor    %r8d,%r8d
<c30walk::ctor::Ctor>::build::<false>:
   2b642:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
write<c30walk::Rec>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1933
   2b650:	movdqu 0x2c(%rdi),%xmm0
   2b655:	movdqu %xmm0,-0x50(%r10)
   2b65b:	movdqa (%rdi),%xmm1
   2b65f:	movdqa 0x10(%rdi),%xmm2
   2b664:	movdqa 0x20(%rdi),%xmm3
   2b669:	movdqa %xmm3,-0x5c(%r10)
   2b66f:	movdqa %xmm2,-0x6c(%r10)
   2b675:	movdqa %xmm1,-0x7c(%r10)
   2b67b:	mov    %eax,-0x40(%r10)
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/num/uint_macros.rs:971
   2b67f:	add    $0x2,%r8
write<c30walk::Rec>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1933
   2b683:	movdqa %xmm1,-0x3c(%r10)
   2b689:	movdqa %xmm2,-0x2c(%r10)
   2b68f:	movdqa %xmm3,-0x1c(%r10)
   2b695:	movdqu %xmm0,-0x10(%r10)
   2b69b:	lea    0x2000(%rax),%r11d
   2b6a2:	mov    %r11d,(%r10)
complete<false>():
./examples/c30walk/ctor.rs:490
   2b6a5:	add    $0x4000,%eax
spec_next<usize>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/range.rs:900
   2b6aa:	sub    $0xffffffffffffff80,%r10
   2b6ae:	cmp    %r8,%r9
   2b6b1:	jne    2b650 <<c30walk::ctor::Ctor>::build::<false>+0x12e0>
   2b6b3:	testb  $0x1,0x18(%rsp)
   2b6b8:	mov    0x1150(%rsp),%r11
   2b6c0:	mov    0x20(%rsp),%r9
   2b6c5:	mov    0x28(%rsp),%r10
   2b6ca:	je     2b704 <<c30walk::ctor::Ctor>::build::<false>+0x1394>
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mut_ptr.rs:961
   2b6cc:	add    %rsi,%rdx
   2b6cf:	shl    $0x6,%r8
write<c30walk::Rec>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1933
   2b6d3:	movups 0x2c(%rdi),%xmm0
   2b6d7:	movups %xmm0,0x2c(%rdx,%r8,1)
   2b6dd:	movdqa (%rdi),%xmm0
   2b6e1:	movdqa 0x10(%rdi),%xmm1
   2b6e6:	movdqa 0x20(%rdi),%xmm2
   2b6eb:	movdqa %xmm2,0x20(%rdx,%r8,1)
   2b6f2:	movdqa %xmm1,0x10(%rdx,%r8,1)
   2b6f9:	movdqa %xmm0,(%rdx,%r8,1)
   2b6ff:	mov    %eax,0x3c(%rdx,%r8,1)
complete<false>():
   2b704:	mov    0x18(%rsp),%rax
./examples/c30walk/ctor.rs:492
   2b709:	mov    %eax,%ecx
   2b70b:	shl    $0x5,%ecx
   2b70e:	mov    0x30(%rsp),%edx
   2b712:	add    %ecx,%edx
   2b714:	mov    %edx,0x910(%rsp)
   2b71b:	mov    0x8(%rsp),%rdi
run<false>():
./examples/c30walk/ctor.rs:578
   2b720:	add    %rax,%rdi
./examples/c30walk/ctor.rs:580
   2b723:	imul   $0xffffffffffffffe2,0x60(%rsp),%rdx
./examples/c30walk/ctor.rs:581
   2b729:	mov    %rdx,%rsi
   2b72c:	add    0x10(%rsp),%rsi
   2b731:	jne    2b387 <<c30walk::ctor::Ctor>::build::<false>+0x1017>
   2b737:	mov    0x78(%rsp),%rax
   2b73c:	jmp    2aa7f <<c30walk::ctor::Ctor>::build::<false>+0x70f>
   2b741:	mov    %r10,%r14
   2b744:	mov    %esi,%r10d
   2b747:	and    $0x18,%r10d
   2b74b:	movd   %ebp,%xmm0
   2b74f:	pshufd $0x0,%xmm0,%xmm0
   2b754:	movd   %r8d,%xmm1
   2b759:	pshufd $0x0,%xmm1,%xmm1
next<core::slice::iter::IterMut<u32>, core::slice::iter::Iter<u16>>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/adapters/zip.rs:308
   2b75e:	add    %r15,%r14
   2b761:	mov    %r14,%rax
   2b764:	movabs $0x8888888888888889,%rdx
   2b76e:	mul    %rdx
   2b771:	shr    $0x4,%edx
   2b774:	lea    (%r14,%rdx,2),%eax
   2b778:	and    $0x18,%eax
   2b77b:	lea    -0x19dc2(%rip),%rdx        # 119c0 <c30walk::ctor::PATTERNS>
   2b782:	mov    0x10(%rsp),%r9
   2b787:	add    %r9,%rdx
   2b78a:	add    $0x8,%rdx
   2b78e:	shl    $0x7,%ecx
   2b791:	add    %rdi,%rcx
   2b794:	add    $0x10,%rcx
   2b798:	xor    %r14d,%r14d
fragment<false>():
   2b79b:	nopl   0x0(%rax,%rax,1)
./examples/c30walk/ctor.rs:465
   2b7a0:	movq   -0x8(%rdx,%r14,2),%xmm2
   2b7a7:	movq   (%rdx,%r14,2),%xmm3
./examples/c30walk/ctor.rs:466
   2b7ad:	punpcklwd %xmm4,%xmm2
   2b7b1:	punpcklwd %xmm4,%xmm3
   2b7b5:	paddd  %xmm1,%xmm2
   2b7b9:	paddd  %xmm1,%xmm3
   2b7bd:	pslld  $0x8,%xmm2
   2b7c2:	pslld  $0x8,%xmm3
   2b7c7:	por    %xmm0,%xmm2
   2b7cb:	por    %xmm0,%xmm3
   2b7cf:	movdqu %xmm2,-0x10(%rcx,%r14,4)
   2b7d6:	movdqu %xmm3,(%rcx,%r14,4)
next<core::slice::iter::IterMut<u32>, core::slice::iter::Iter<u16>>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/adapters/zip.rs:308
   2b7dc:	add    $0x8,%r14
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/adapters/zip.rs:304
   2b7e0:	cmp    %r14,%rax
   2b7e3:	jne    2b7a0 <<c30walk::ctor::Ctor>::build::<false>+0x1430>
   2b7e5:	cmp    %r10,%rsi
   2b7e8:	je     2b5e5 <<c30walk::ctor::Ctor>::build::<false>+0x1275>
   2b7ee:	jmp    2b4ba <<c30walk::ctor::Ctor>::build::<false>+0x114a>
fragment<false>():
   2b7f3:	mov    %rax,%r11
   2b7f6:	shr    $1,%r11
   2b7f9:	inc    %r11
next<u16>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/iter/macros.rs:180
   2b7fc:	cmp    $0x1e,%rax
   2b800:	jae    2b80a <<c30walk::ctor::Ctor>::build::<false>+0x149a>
   2b802:	xor    %r14d,%r14d
   2b805:	jmp    2b899 <<c30walk::ctor::Ctor>::build::<false>+0x1529>
   2b80a:	mov    %r11,0x10(%rsp)
   2b80f:	mov    %r11,%r14
   2b812:	and    $0xfffffffffffffff0,%r14
   2b816:	lea    -0x19e5d(%rip),%rax        # 119c0 <c30walk::ctor::PATTERNS>
   2b81d:	lea    (%r9,%rax,1),%rbp
   2b821:	add    $0x10,%rbp
   2b825:	lea    (%r10,%r15,1),%rax
   2b829:	lea    (%rax,%rax,1),%r11
   2b82d:	movabs $0x8888888888888889,%rdx
   2b837:	mul    %rdx
   2b83a:	shr    $0x4,%rdx
   2b83e:	imul   $0x3c,%rdx,%rax
   2b842:	sub    %rax,%r11
   2b845:	add    $0xfffffffffffffffe,%r11
   2b849:	shr    $1,%r11
   2b84c:	inc    %r11
   2b84f:	and    $0xfffffffffffffff0,%r11
   2b853:	mov    0x8(%rsp),%rax
   2b858:	add    %rcx,%rax
   2b85b:	add    $0x10,%rax
   2b85f:	xor    %edx,%edx
fragment<false>():
./examples/c30walk/ctor.rs:435
   2b861:	movdqa -0x10(%rbp,%rdx,2),%xmm0
   2b867:	movdqa 0x0(%rbp,%rdx,2),%xmm1
write<u16>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1933
   2b86d:	movdqu %xmm0,-0x10(%rax,%rdx,2)
   2b873:	movdqu %xmm1,(%rax,%rdx,2)
next<core::slice::iter::Iter<u16>>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/adapters/enumerate.rs:82
   2b878:	add    $0x10,%rdx
next<u16>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/iter/macros.rs:180
   2b87c:	cmp    %rdx,%r11
   2b87f:	jne    2b861 <<c30walk::ctor::Ctor>::build::<false>+0x14f1>
   2b881:	mov    0x10(%rsp),%r11
   2b886:	cmp    %r14,%r11
   2b889:	je     2b5e5 <<c30walk::ctor::Ctor>::build::<false>+0x1275>
fragment<false>():
   2b88f:	test   $0xc,%r11b
   2b893:	je     2b924 <<c30walk::ctor::Ctor>::build::<false>+0x15b4>
   2b899:	movabs $0x8888888888888889,%rdx
   2b8a3:	mov    %r10,%rax
   2b8a6:	lea    (%r9,%r14,2),%r9
   2b8aa:	add    %r15,%rax
   2b8ad:	lea    (%rax,%rax,1),%r15
   2b8b1:	mul    %rdx
   2b8b4:	shr    $0x4,%rdx
   2b8b8:	imul   $0x3c,%rdx,%rax
   2b8bc:	sub    %rax,%r15
   2b8bf:	add    $0xfffffffffffffffe,%r15
   2b8c3:	shr    $1,%r15
   2b8c6:	inc    %r15
   2b8c9:	and    $0xfffffffffffffffc,%r15
   2b8cd:	sub    %r14,%r15
   2b8d0:	mov    0x8(%rsp),%rax
   2b8d5:	lea    (%rax,%r14,2),%rdx
   2b8d9:	mov    %r11,%r10
   2b8dc:	mov    %r11,%r14
   2b8df:	and    $0xfffffffffffffffc,%r14
   2b8e3:	lea    (%rdi,%r14,2),%rax
   2b8e7:	lea    -0x19f2e(%rip),%r11        # 119c0 <c30walk::ctor::PATTERNS>
   2b8ee:	add    %r11,%r9
   2b8f1:	add    %rcx,%rdx
   2b8f4:	xor    %r11d,%r11d
./examples/c30walk/ctor.rs:435
   2b8f7:	mov    (%r9,%r11,2),%rbp
write<u16>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1933
   2b8fb:	mov    %rbp,(%rdx,%r11,2)
next<u16>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/iter/macros.rs:180
   2b8ff:	add    $0x4,%r11
   2b903:	cmp    %r11,%r15
   2b906:	jne    2b8f7 <<c30walk::ctor::Ctor>::build::<false>+0x1587>
   2b908:	cmp    %r14,%r10
   2b90b:	jne    2b5b9 <<c30walk::ctor::Ctor>::build::<false>+0x1249>
   2b911:	jmp    2b5e5 <<c30walk::ctor::Ctor>::build::<false>+0x1275>
run<false>():
   2b916:	mov    0x68(%rsp),%rsi
   2b91b:	lea    (%rsi,%rax,2),%r10
   2b91f:	jmp    2b27f <<c30walk::ctor::Ctor>::build::<false>+0xf0f>
   2b924:	lea    (%rdi,%r14,2),%rax
   2b928:	jmp    2b5b9 <<c30walk::ctor::Ctor>::build::<false>+0x1249>
fragment<false>():
./examples/c30walk/ctor.rs:452
   2b92d:	lea    0xd8(%rsp),%rdi
   2b935:	mov    0x8(%rsp),%rsi
   2b93a:	mov    %r10,%rdx
   2b93d:	mov    %eax,%ecx
   2b93f:	call   585e0 <<c30walk::ctor::W<false>>::promote>
   2b944:	mov    0x30(%rsp),%r8
   2b949:	mov    0x40(%rsp),%r10
   2b94e:	pxor   %xmm4,%xmm4
   2b952:	mov    0x1150(%rsp),%r11
./examples/c30walk/ctor.rs:442
   2b95a:	jmp    2aee7 <<c30walk::ctor::Ctor>::build::<false>+0xb77>
scatter<false>():
   2b95f:	mov    0xf8(%rsp),%r14
   2b967:	mov    0xe8(%rsp),%r8
   2b96f:	mov    0xd8(%rsp),%rdx
   2b977:	mov    0xe0(%rsp),%rax
   2b97f:	mov    %rax,0x18(%rsp)
   2b984:	mov    0x910(%rsp),%ebx
   2b98b:	mov    0x908(%rsp),%r15
   2b993:	mov    %rdx,0x10(%rsp)
   2b998:	jmp    2b9f5 <<c30walk::ctor::Ctor>::build::<false>+0x1685>
   2b99a:	nopw   0x0(%rax,%rax,1)
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:622
   2b9a0:	and    $0xfffff,%eax
one<false>():
./examples/c30walk/ctor.rs:635
   2b9a5:	mov    %eax,%edi
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/mod.rs:1903
   2b9a7:	mov    0x10(%rdx),%rsi
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/index.rs:278
   2b9ab:	cmp    %rdi,%rsi
index_mut<c30walk::RawG>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/index.rs:278
   2b9ae:	jbe    2bb11 <<c30walk::ctor::Ctor>::build::<false>+0x17a1>
   2b9b4:	shl    $0x7,%rdi
   2b9b8:	add    0x8(%rdx),%rdi
one<false>():
./examples/c30walk/ctor.rs:635
   2b9bc:	mov    %ebp,(%rdi,%r13,4)
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:350
   2b9c0:	lea    0x1(%r15),%rax
advance<false>():
./examples/c30walk/ctor.rs:351
   2b9c4:	add    $0xffffffffffffffe3,%r15
   2b9c8:	xor    %ecx,%ecx
   2b9ca:	cmp    $0x1e,%rax
   2b9ce:	setae  %cl
   2b9d1:	cmovb  %rax,%r15
   2b9d5:	lea    (%rbx,%rcx,2),%ebx
   2b9d8:	inc    %ebx
   2b9da:	mov    %r15,0x908(%rsp)
   2b9e2:	mov    %ebx,0x910(%rsp)
eq<u8>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/non_null.rs:1714
   2b9e9:	inc    %r12
   2b9ec:	cmp    %r12,%r9
next<u8>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/iter/macros.rs:180
   2b9ef:	je     2aa18 <<c30walk::ctor::Ctor>::build::<false>+0x6a8>
scatter<false>():
./examples/c30walk/ctor.rs:897
   2b9f5:	movzbl (%r10,%r12,1),%eax
one<false>():
./examples/c30walk/ctor.rs:602
   2b9fa:	mov    0x108(%rsp,%rax,4),%esi
./examples/c30walk/ctor.rs:603
   2ba01:	cmp    0x508(%rsp,%rax,4),%esi
   2ba08:	jae    2aa43 <<c30walk::ctor::Ctor>::build::<false>+0x6d3>
./examples/c30walk/ctor.rs:607
   2ba0e:	mov    %esi,%r13d
   2ba11:	and    $0x1f,%r13d
   2ba15:	xor    %ecx,%ecx
   2ba17:	cmp    $0x1d,%r13d
   2ba1b:	sete   %cl
   2ba1e:	lea    (%rsi,%rcx,2),%ecx
   2ba21:	inc    %ecx
   2ba23:	mov    %ecx,0x108(%rsp,%rax,4)
./examples/c30walk/ctor.rs:608
   2ba2a:	shr    $0x5,%esi
./examples/c30walk/ctor.rs:610
   2ba2d:	mov    %ebx,%ebp
   2ba2f:	shl    $0x8,%ebp
   2ba32:	or     %eax,%ebp
./examples/c30walk/ctor.rs:611
   2ba34:	test   %r13d,%r13d
   2ba37:	je     2ba80 <<c30walk::ctor::Ctor>::build::<false>+0x1710>
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mut_ptr.rs:961
   2ba39:	mov    %rsi,%rcx
add<c30walk::Rec>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mut_ptr.rs:961
   2ba3c:	shl    $0x6,%rcx
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1733
   2ba40:	mov    0x3c(%r14,%rcx,1),%eax
one<false>():
./examples/c30walk/ctor.rs:621
   2ba45:	test   $0x10000000,%eax
   2ba4a:	jne    2b9a0 <<c30walk::ctor::Ctor>::build::<false>+0x1630>
./examples/c30walk/ctor.rs:625
   2ba50:	mov    %eax,%edi
   2ba52:	shr    $0x8,%edi
   2ba55:	mov    %ebx,%edx
   2ba57:	sub    %edi,%edx
./examples/c30walk/ctor.rs:626
   2ba59:	cmp    $0xfffe,%edx
   2ba5f:	jae    2bacf <<c30walk::ctor::Ctor>::build::<false>+0x175f>
   2ba61:	add    %r14,%rcx
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1933
   2ba64:	mov    %dx,(%rcx,%r13,2)
write<u16>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1933
   2ba69:	mov    0x10(%rsp),%rdx
one<false>():
./examples/c30walk/ctor.rs:611
   2ba6e:	jmp    2b9c0 <<c30walk::ctor::Ctor>::build::<false>+0x1650>
   2ba73:	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:323
   2ba80:	mov    %esi,%edi
is_raw<false>():
./examples/c30walk/ctor.rs:323
   2ba82:	shr    $0x6,%edi
   2ba85:	cmp    %r8,%rdi
   2ba88:	jae    2be48 <<c30walk::ctor::Ctor>::build::<false>+0x1ad8>
   2ba8e:	mov    0x18(%rsp),%rax
   2ba93:	mov    (%rax,%rdi,8),%rcx
one<false>():
   2ba97:	mov    %rsi,%rax
   2ba9a:	shl    $0x6,%rax
   2ba9e:	add    %r14,%rax
./examples/c30walk/ctor.rs:612
   2baa1:	bt     %rsi,%rcx
   2baa5:	jae    2bac2 <<c30walk::ctor::Ctor>::build::<false>+0x1752>
   2baa7:	mov    0x3c(%rax),%eax
./examples/c30walk/ctor.rs:613
   2baaa:	mov    $0xfffff,%ecx
   2baaf:	and    %ecx,%eax
./examples/c30walk/ctor.rs:635
   2bab1:	mov    %eax,%edi
as_mut_slice<c30walk::RawG, alloc::alloc::Global>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/mod.rs:1903
   2bab3:	mov    0x10(%rdx),%rsi
index_mut<c30walk::RawG>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/index.rs:278
   2bab7:	cmp    %rdi,%rsi
   2baba:	ja     2b9b4 <<c30walk::ctor::Ctor>::build::<false>+0x1644>
   2bac0:	jmp    2bb11 <<c30walk::ctor::Ctor>::build::<false>+0x17a1>
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1933
   2bac2:	mov    %ebp,0x3c(%rax)
   2bac5:	movw   $0x0,(%rax)
write<u16>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1933
   2baca:	jmp    2b9c0 <<c30walk::ctor::Ctor>::build::<false>+0x1650>
one<false>():
./examples/c30walk/ctor.rs:627
   2bacf:	lea    0xd8(%rsp),%rdi
   2bad7:	mov    %r13,%rdx
   2bada:	mov    %eax,%ecx
   2badc:	mov    %r8,0x8(%rsp)
   2bae1:	call   585e0 <<c30walk::ctor::W<false>>::promote>
   2bae6:	mov    0x8(%rsp),%r8
   2baeb:	mov    0x28(%rsp),%r10
   2baf0:	mov    0x20(%rsp),%r9
   2baf5:	mov    0x1150(%rsp),%r11
   2bafd:	mov    0x10(%rsp),%rdx
./examples/c30walk/ctor.rs:635
   2bb02:	mov    %eax,%edi
as_mut_slice<c30walk::RawG, alloc::alloc::Global>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/mod.rs:1903
   2bb04:	mov    0x10(%rdx),%rsi
index_mut<c30walk::RawG>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/index.rs:278
   2bb08:	cmp    %rdi,%rsi
   2bb0b:	ja     2b9b4 <<c30walk::ctor::Ctor>::build::<false>+0x1644>
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/index.rs:278
   2bb11:	lea    0xa7580(%rip),%rdx        # d3098 <__frame_dummy_init_array_entry+0x16e8>
index_mut<c30walk::RawG>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/index.rs:278
   2bb18:	call   7b81b <core::panicking::panic_bounds_check>
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/num/uint_macros.rs:2472
   2bb1d:	xor    %ecx,%ecx
saturating_sub():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/num/uint_macros.rs:2472
   2bb1f:	mov    0x48(%rsp),%rsi
   2bb24:	test   %rsi,%rsi
   2bb27:	setne  %al
   2bb2a:	mov    0x20(%rsp),%rdx
   2bb2f:	mov    0x90(%rsp),%rbx
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:785
   2bb37:	je     2bb88 <<c30walk::ctor::Ctor>::build::<false>+0x1818>
   2bb39:	mov    %al,%cl
   2bb3b:	mov    %rsi,%rax
   2bb3e:	sub    %rcx,%rax
   2bb41:	imul   $0xffffffffffffffe2,%rax,%rcx
   2bb45:	add    %rdx,%rcx
   2bb48:	cmp    $0x1e,%rcx
   2bb4c:	jae    2bb88 <<c30walk::ctor::Ctor>::build::<false>+0x1818>
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/range.rs:900
   2bb4e:	lea    (%rbx,%rsi,4),%rcx
spec_next<usize>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/range.rs:900
   2bb52:	lea    (%rcx,%rdx,2),%rdi
   2bb56:	add    $0xfffffffffffffffc,%rdi
   2bb5a:	lea    (%rax,%rax,1),%rcx
   2bb5e:	shl    $0x5,%rax
   2bb62:	sub    %rcx,%rax
   2bb65:	sub    %rdx,%rax
   2bb68:	lea    0x3c(,%rax,2),%rdx
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1933
   2bb70:	mov    $0xff,%esi
write<u16>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1933
   2bb75:	call   *0xaa8d5(%rip)        # d6450 <memset@GLIBC_2.2.5>
   2bb7b:	mov    0x48(%rsp),%rsi
   2bb80:	mov    0x1150(%rsp),%r11
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mut_ptr.rs:961
   2bb88:	shl    $0x6,%rsi
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1933
   2bb8c:	pcmpeqd %xmm0,%xmm0
write<c30walk::Rec>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1933
   2bb90:	movdqu %xmm0,0x2c(%rbx,%rsi,1)
   2bb96:	movdqa %xmm0,0x20(%rbx,%rsi,1)
   2bb9c:	movdqa %xmm0,0x10(%rbx,%rsi,1)
   2bba2:	movdqa %xmm0,(%rbx,%rsi,1)
   2bba7:	movl   $0x100fffff,0x3c(%rbx,%rsi,1)
   2bbaf:	mov    0xd0(%rsp),%rax
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/raw_vec/mod.rs:614
   2bbb7:	mov    0x8(%rax),%rdx
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/mod.rs:1864
   2bbbb:	mov    0x10(%rax),%rcx
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/raw_vec/mod.rs:614
   2bbbf:	mov    0x8(%r11),%r8
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/mod.rs:1903
   2bbc3:	mov    0x10(%r11),%r9
as_mut_slice<c30walk::RawG, alloc::alloc::Global>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/mod.rs:1903
   2bbc7:	lea    0xd8(%rsp),%rdi
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:795
   2bbcf:	mov    %rbx,%rsi
   2bbd2:	mov    %r11,%rbx
   2bbd5:	call   5fab0 <c30walk::ctor::canonicalise>
   2bbda:	mov    0x50(%rsp),%rax
   2bbdf:	mov    0xc8(%rsp),%rcx
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/mod.rs:2221
   2bbe7:	mov    %rcx,0x10(%rax)
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/raw_vec/mod.rs:614
   2bbeb:	mov    0x8(%rbx),%rax
./examples/c30walk/ctor.rs:804
   2bbef:	and    $0x3f,%rax
   2bbf3:	mov    %rax,0xd8(%rsp)
   2bbfb:	jne    2be01 <<c30walk::ctor::Ctor>::build::<false>+0x1a91>
   2bc01:	mov    $0x1,%al
   2bc03:	jmp    2aa4d <<c30walk::ctor::Ctor>::build::<false>+0x6dd>
   2bc08:	xor    %r12d,%r12d
   2bc0b:	mov    0x48(%rsp),%rcx
capacity<alloc::alloc::Global>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/raw_vec/mod.rs:619
   2bc10:	mov    (%r8),%rax
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:744
   2bc13:	lea    0x1(%rcx),%rbp
   2bc17:	cmp    %rcx,%rax
   2bc1a:	ja     2a588 <<c30walk::ctor::Ctor>::build::<false>+0x218>
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/raw_vec/mod.rs:634
   2bc20:	test   %rax,%rax
current_memory<alloc::alloc::Global>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/raw_vec/mod.rs:634
   2bc23:	je     2bc34 <<c30walk::ctor::Ctor>::build::<false>+0x18c4>
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:745
   2bc25:	mov    0x8(%r8),%rdi
<std::alloc::System as core::alloc::global::GlobalAlloc>::dealloc:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/std/src/sys/alloc/unix.rs:48
   2bc29:	call   *0xaa831(%rip)        # d6460 <free@GLIBC_2.2.5>
   2bc2f:	mov    0x50(%rsp),%r8
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:745
   2bc34:	movq   $0x0,(%r8)
   2bc3b:	movq   $0x40,0x8(%r8)
   2bc43:	movq   $0x0,0x10(%r8)
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/raw_vec/mod.rs:728
   2bc4b:	mov    $0x40,%edx
try_reserve_exact<alloc::alloc::Global>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/raw_vec/mod.rs:728
   2bc50:	mov    %r8,%rdi
   2bc53:	mov    %rbp,%rsi
   2bc56:	call   57740 <<alloc::raw_vec::RawVecInner>::grow_exact>
   2bc5b:	mov    0x50(%rsp),%r8
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/result.rs:2173
   2bc60:	cmp    $0xffffffffffffffff,%rax
branch<(), alloc::collections::TryReserveError>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/result.rs:2173
   2bc64:	mov    0x1150(%rsp),%r9
   2bc6c:	mov    %r15,%rdi
   2bc6f:	je     2a588 <<c30walk::ctor::Ctor>::build::<false>+0x218>
   2bc75:	jmp    2bccf <<c30walk::ctor::Ctor>::build::<false>+0x195f>
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/raw_vec/mod.rs:634
   2bc77:	test   %rax,%rax
current_memory<alloc::alloc::Global>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/raw_vec/mod.rs:634
   2bc7a:	je     2bc8e <<c30walk::ctor::Ctor>::build::<false>+0x191e>
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:751
   2bc7c:	mov    0x8(%r9),%rdi
<std::alloc::System as core::alloc::global::GlobalAlloc>::dealloc:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/std/src/sys/alloc/unix.rs:48
   2bc80:	call   *0xaa7da(%rip)        # d6460 <free@GLIBC_2.2.5>
   2bc86:	mov    0x1150(%rsp),%r9
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:751
   2bc8e:	movq   $0x0,(%r9)
   2bc95:	movq   $0x40,0x8(%r9)
   2bc9d:	movq   $0x0,0x10(%r9)
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/raw_vec/mod.rs:728
   2bca5:	mov    $0x80,%edx
try_reserve_exact<alloc::alloc::Global>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/raw_vec/mod.rs:728
   2bcaa:	mov    %r9,%rdi
   2bcad:	mov    %r12,%rsi
   2bcb0:	call   57740 <<alloc::raw_vec::RawVecInner>::grow_exact>
   2bcb5:	mov    0x1150(%rsp),%r9
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/result.rs:2173
   2bcbd:	cmp    $0xffffffffffffffff,%rax
branch<(), alloc::collections::TryReserveError>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/result.rs:2173
   2bcc1:	mov    0x50(%rsp),%r8
   2bcc6:	mov    %r15,%rdi
   2bcc9:	je     2a594 <<c30walk::ctor::Ctor>::build::<false>+0x224>
<c30walk::ctor::Ctor>::build::<false>:
   2bccf:	mov    %rax,%rdi
   2bcd2:	mov    %rdx,%rsi
   2bcd5:	call   79fdb <alloc::raw_vec::handle_error>
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/index.rs:568
   2bcda:	cmp    %r9,%r12
index<u8>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/index.rs:568
   2bcdd:	jbe    2aa0f <<c30walk::ctor::Ctor>::build::<false>+0x69f>
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/index.rs:569
   2bce3:	lea    0xa5e96(%rip),%rcx        # d1b80 <__frame_dummy_init_array_entry+0x1d0>
   2bcea:	mov    %r12,%rdi
   2bced:	mov    %r9,%rsi
   2bcf0:	mov    %r9,%rdx
   2bcf3:	call   7c580 <core::slice::index::slice_index_fail>
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/index.rs:443
   2bcf8:	lea    0xa7939(%rip),%rcx        # d3638 <__frame_dummy_init_array_entry+0x1c88>
index<u8>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/index.rs:443
   2bcff:	mov    %r12,%rdi
   2bd02:	mov    %r9,%rdx
   2bd05:	call   7c580 <core::slice::index::slice_index_fail>
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/index.rs:443
   2bd0a:	lea    0xa7927(%rip),%rcx        # d3638 <__frame_dummy_init_array_entry+0x1c88>
index<u8>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/index.rs:443
   2bd11:	mov    %rdx,%rdi
   2bd14:	mov    %r9,%rdx
   2bd17:	call   7c580 <core::slice::index::slice_index_fail>
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/index.rs:443
   2bd1c:	lea    0xa7915(%rip),%rcx        # d3638 <__frame_dummy_init_array_entry+0x1c88>
index<u8>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/index.rs:443
   2bd23:	mov    %rsi,%rdi
   2bd26:	mov    %rax,%rsi
   2bd29:	mov    %r9,%rdx
   2bd2c:	call   7c580 <core::slice::index::slice_index_fail>
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:759
   2bd31:	lea    -0x20bb0(%rip),%rsi        # b188 <GCC_except_table819+0x9bc>
   2bd38:	lea    -0x1ae0b(%rip),%rdx        # 10f34 <GCC_except_table819+0x6768>
   2bd3f:	lea    0xa5e22(%rip),%r8        # d1b68 <__frame_dummy_init_array_entry+0x1b8>
   2bd46:	lea    0xd8(%rsp),%rdi
   2bd4e:	mov    $0x2f,%ecx
   2bd53:	call   7bbc2 <core::panicking::assert_failed::<usize, usize>>
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/index.rs:443
   2bd58:	lea    0xa5e39(%rip),%rcx        # d1b98 <__frame_dummy_init_array_entry+0x1e8>
index<u8>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/index.rs:443
   2bd5f:	mov    %rdx,%rdi
   2bd62:	mov    %r14,%rsi
   2bd65:	mov    %r9,%rdx
   2bd68:	call   7c580 <core::slice::index::slice_index_fail>
reserve<alloc::alloc::Global>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/raw_vec/mod.rs:675
   2bd6d:	mov    $0x8,%ecx
   2bd72:	mov    $0x8,%r8d
   2bd78:	mov    %r15,%rdi
   2bd7b:	xor    %esi,%esi
   2bd7d:	mov    %r14,%rdx
   2bd80:	call   3ad70 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   2bd85:	mov    %r15,%rdi
   2bd88:	mov    0x1150(%rsp),%r9
   2bd90:	mov    0x50(%rsp),%r8
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/mod.rs:3054
   2bd95:	mov    0x10(%r15),%r13
len<u64, alloc::alloc::Global>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/mod.rs:3054
   2bd99:	jmp    2a5b7 <<c30walk::ctor::Ctor>::build::<false>+0x247>
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/index.rs:456
   2bd9e:	lea    0xa73cb(%rip),%rcx        # d3170 <__frame_dummy_init_array_entry+0x17c0>
index_mut<u32>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/index.rs:456
   2bda5:	mov    $0x20,%edx
   2bdaa:	mov    %r10,%rdi
   2bdad:	call   7c580 <core::slice::index::slice_index_fail>
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/index.rs:443
   2bdb2:	lea    0xa739f(%rip),%rcx        # d3158 <__frame_dummy_init_array_entry+0x17a8>
index<u16>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/index.rs:443
   2bdb9:	mov    $0x1e,%edx
   2bdbe:	xor    %edi,%edi
   2bdc0:	mov    %r8,%rsi
   2bdc3:	call   7c580 <core::slice::index::slice_index_fail>
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/index.rs:456
   2bdc8:	lea    0xa73a1(%rip),%rcx        # d3170 <__frame_dummy_init_array_entry+0x17c0>
index_mut<u32>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/index.rs:456
   2bdcf:	mov    $0x20,%edx
   2bdd4:	xor    %edi,%edi
   2bdd6:	call   7c580 <core::slice::index::slice_index_fail>
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/index.rs:443
   2bddb:	lea    0xa7376(%rip),%rcx        # d3158 <__frame_dummy_init_array_entry+0x17a8>
   2bde2:	mov    $0x1e,%edx
index<u16>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/index.rs:443
   2bde7:	xor    %edi,%edi
   2bde9:	call   7c580 <core::slice::index::slice_index_fail>
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/index.rs:443
   2bdee:	lea    0xa731b(%rip),%rcx        # d3110 <__frame_dummy_init_array_entry+0x1760>
index<u16>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/index.rs:443
   2bdf5:	mov    $0x1e,%edx
   2bdfa:	xor    %edi,%edi
   2bdfc:	call   7c580 <core::slice::index::slice_index_fail>
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:804
   2be01:	lea    -0x20c80(%rip),%rsi        # b188 <GCC_except_table819+0x9bc>
   2be08:	lea    -0x1aef5(%rip),%rdx        # 10f1a <GCC_except_table819+0x674e>
   2be0f:	lea    0xa5d3a(%rip),%r8        # d1b50 <__frame_dummy_init_array_entry+0x1a0>
   2be16:	lea    0xd8(%rsp),%rdi
   2be1e:	mov    $0x35,%ecx
   2be23:	call   7bbc2 <core::panicking::assert_failed::<usize, usize>>
create_raw<false>():
./examples/c30walk/ctor.rs:373
   2be28:	lea    0xa7239(%rip),%rdx        # d3068 <__frame_dummy_init_array_entry+0x16b8>
   2be2f:	mov    0x18(%rsp),%rsi
   2be34:	call   7b81b <core::panicking::panic_bounds_check>
is_raw<false>():
./examples/c30walk/ctor.rs:323
   2be39:	lea    0xa5d70(%rip),%rdx        # d1bb0 <__frame_dummy_init_array_entry+0x200>
   2be40:	mov    %r13,%rdi
   2be43:	call   7b81b <core::panicking::panic_bounds_check>
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:323
   2be48:	lea    0xa5d61(%rip),%rdx        # d1bb0 <__frame_dummy_init_array_entry+0x200>
is_raw<false>():
./examples/c30walk/ctor.rs:323
   2be4f:	mov    %r8,%rsi
   2be52:	call   7b81b <core::panicking::panic_bounds_check>
fragment<false>():
./examples/c30walk/ctor.rs:426
   2be57:	lea    0xa729a(%rip),%rdx        # d30f8 <__frame_dummy_init_array_entry+0x1748>
   2be5e:	mov    %rsi,%rdi
   2be61:	mov    $0x1e,%esi
   2be66:	call   7b81b <core::panicking::panic_bounds_check>
   2be6b:	lea    0xa7286(%rip),%rdx        # d30f8 <__frame_dummy_init_array_entry+0x1748>
   2be72:	mov    $0x1e,%esi
   2be77:	call   7b81b <core::panicking::panic_bounds_check>
is_raw<false>():
./examples/c30walk/ctor.rs:323
   2be7c:	lea    0xa5d2d(%rip),%rdx        # d1bb0 <__frame_dummy_init_array_entry+0x200>
   2be83:	mov    %rax,%rdi
   2be86:	mov    %rcx,%rsi
   2be89:	call   7b81b <core::panicking::panic_bounds_check>
fragment<false>():
./examples/c30walk/ctor.rs:448
   2be8e:	lea    0xa7293(%rip),%rdx        # d3128 <__frame_dummy_init_array_entry+0x1778>
   2be95:	mov    $0x1e,%esi
   2be9a:	call   7b81b <core::panicking::panic_bounds_check>
index_mut<c30walk::RawG>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/index.rs:278
   2be9f:	lea    0xa729a(%rip),%rdx        # d3140 <__frame_dummy_init_array_entry+0x1790>
   2bea6:	call   7b81b <core::panicking::panic_bounds_check>
   2beab:	lea    0xa728e(%rip),%rdx        # d3140 <__frame_dummy_init_array_entry+0x1790>
   2beb2:	mov    %rcx,%rdi
   2beb5:	mov    %rax,%rsi
   2beb8:	call   7b81b <core::panicking::panic_bounds_check>
complete<false>():
./examples/c30walk/ctor.rs:485
   2bebd:	lea    0xa721c(%rip),%rdx        # d30e0 <__frame_dummy_init_array_entry+0x1730>
   2bec4:	mov    $0x1e,%esi
   2bec9:	call   7b81b <core::panicking::panic_bounds_check>
