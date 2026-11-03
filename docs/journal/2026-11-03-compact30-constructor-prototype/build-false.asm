
target/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

0000000000023410 <<c30walk::ctor::Ctor>::build::<false>>:
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:711
   23410:	push   %rbp
   23411:	push   %r15
   23413:	push   %r14
   23415:	push   %r13
   23417:	push   %r12
   23419:	push   %rbx
   2341a:	sub    $0x1000,%rsp
   23421:	movq   $0x0,(%rsp)
   23429:	sub    $0x118,%rsp
   23430:	mov    0x1150(%rsp),%rax
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:3035
   23438:	movq   $0x0,0x10(%r9)
   23440:	movq   $0x0,0x10(%rax)
clear<c30walk::RawG, alloc::alloc::Global>():
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:3035
   23448:	cmp    $0xeffe2,%rdx
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:722
   2344f:	ja     23aeb <<c30walk::ctor::Ctor>::build::<false>+0x6db>
   23455:	mov    %rcx,%rbx
   23458:	pxor   %xmm0,%xmm0
   2345c:	xor    %eax,%eax
   2345e:	pxor   %xmm2,%xmm2
   23462:	pxor   %xmm1,%xmm1
   23466:	cs nopw 0x0(%rax,%rax,1)
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:279
   23470:	movq   (%rbx,%rax,4),%xmm3
fold<u32, usize, core::iter::adapters::map::map_fold::{closure_env#0}<&u32, usize, usize, c30walk::ctor::{impl#2}::build::{closure_env#0}<false>, core::iter::traits::accum::{impl#48}::sum::{closure_env#0}<core::iter::adapters::map::Map<core::slice::iter::Iter<u32>, c30walk::ctor::{impl#2}::build::{closure_env#0}<false>>>>>():
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:279
   23475:	movq   0x8(%rbx,%rax,4),%xmm4
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:722
   2347b:	punpckldq %xmm0,%xmm3
/rustc/2d8144b78/library/core/src/iter/traits/accum.rs:55
   2347f:	paddq  %xmm2,%xmm3
{closure#0}<false>():
./examples/c30walk/ctor.rs:722
   23483:	punpckldq %xmm0,%xmm4
{closure#0}<core::iter::adapters::map::Map<core::slice::iter::Iter<u32>, c30walk::ctor::{impl#2}::build::{closure_env#0}<false>>>():
/rustc/2d8144b78/library/core/src/iter/traits/accum.rs:55
   23487:	paddq  %xmm1,%xmm4
fold<u32, usize, core::iter::adapters::map::map_fold::{closure_env#0}<&u32, usize, usize, c30walk::ctor::{impl#2}::build::{closure_env#0}<false>, core::iter::traits::accum::{impl#48}::sum::{closure_env#0}<core::iter::adapters::map::Map<core::slice::iter::Iter<u32>, c30walk::ctor::{impl#2}::build::{closure_env#0}<false>>>>>():
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:279
   2348b:	movq   0x10(%rbx,%rax,4),%xmm2
   23491:	movq   0x18(%rbx,%rax,4),%xmm1
{closure#0}<false>():
./examples/c30walk/ctor.rs:722
   23497:	punpckldq %xmm0,%xmm2
{closure#0}<core::iter::adapters::map::Map<core::slice::iter::Iter<u32>, c30walk::ctor::{impl#2}::build::{closure_env#0}<false>>>():
/rustc/2d8144b78/library/core/src/iter/traits/accum.rs:55
   2349b:	paddq  %xmm3,%xmm2
{closure#0}<false>():
./examples/c30walk/ctor.rs:722
   2349f:	punpckldq %xmm0,%xmm1
{closure#0}<core::iter::adapters::map::Map<core::slice::iter::Iter<u32>, c30walk::ctor::{impl#2}::build::{closure_env#0}<false>>>():
/rustc/2d8144b78/library/core/src/iter/traits/accum.rs:55
   234a3:	paddq  %xmm4,%xmm1
fold<u32, usize, core::iter::adapters::map::map_fold::{closure_env#0}<&u32, usize, usize, c30walk::ctor::{impl#2}::build::{closure_env#0}<false>, core::iter::traits::accum::{impl#48}::sum::{closure_env#0}<core::iter::adapters::map::Map<core::slice::iter::Iter<u32>, c30walk::ctor::{impl#2}::build::{closure_env#0}<false>>>>>():
   234a7:	add    $0x8,%rax
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:284
   234ab:	cmp    $0x100,%rax
   234b1:	jne    23470 <<c30walk::ctor::Ctor>::build::<false>+0x60>
   234b3:	paddq  %xmm2,%xmm1
   234b7:	pshufd $0xee,%xmm1,%xmm0
   234bc:	paddq  %xmm1,%xmm0
   234c0:	movq   %xmm0,%rax
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:722
   234c5:	cmp    %rdx,%rax
   234c8:	jne    23aeb <<c30walk::ctor::Ctor>::build::<false>+0x6db>
   234ce:	mov    %r8d,0x3c(%rsp)
   234d3:	mov    %rsi,0x28(%rsp)
   234d8:	mov    %rdi,%r15
   234db:	mov    %rdx,0x20(%rsp)
   234e0:	mov    %r9,0x50(%rsp)
   234e5:	lea    0xd18(%rsp),%rdi
   234ed:	xor    %r12d,%r12d
   234f0:	mov    0x96461(%rip),%r14        # b9958 <memset@GLIBC_2.2.5>
./examples/c30walk/ctor.rs:726
   234f7:	mov    $0x400,%edx
   234fc:	xor    %esi,%esi
   234fe:	call   *%r14
   23501:	lea    0x918(%rsp),%rdi
   23509:	mov    $0x400,%edx
   2350e:	xor    %esi,%esi
   23510:	call   *%r14
   23513:	movabs $0x8888888888888889,%r9
   2351d:	xor    %esi,%esi
   2351f:	xor    %edi,%edi
   23521:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
./examples/c30walk.rs:95
   23530:	mov    %r12,%rax
phi():
./examples/c30walk.rs:95
   23533:	mul    %r9
   23536:	mov    %rdx,%rcx
   23539:	shr    $0x4,%rcx
   2353d:	mov    %ecx,%eax
   2353f:	shl    $0x5,%eax
   23542:	add    %ecx,%ecx
   23544:	sub    %eax,%ecx
   23546:	add    %r12d,%ecx
   23549:	or     %eax,%ecx
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:730
   2354b:	mov    (%rbx,%rdi,4),%r8d
   2354f:	add    %r8,%r12
./examples/c30walk.rs:95
   23552:	mov    %r12,%rax
phi():
./examples/c30walk.rs:95
   23555:	mul    %r9
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:729
   23558:	mov    %ecx,0x918(%rsp,%rdi,4)
phi():
./examples/c30walk.rs:95
   2355f:	shr    $0x4,%rdx
   23563:	mov    %edx,%eax
   23565:	shl    $0x5,%eax
   23568:	add    %edx,%edx
   2356a:	sub    %eax,%edx
   2356c:	add    %r12d,%edx
   2356f:	or     %eax,%edx
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:731
   23571:	mov    %edx,0xd18(%rsp,%rdi,4)
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:971
   23578:	inc    %rdi
./examples/c30walk/ctor.rs:732
   2357b:	cmp    $0x1,%r8
   2357f:	sbb    $0xffffffffffffffff,%rsi
/rustc/2d8144b78/library/core/src/cmp.rs:1916
   23583:	cmp    $0x100,%rdi
/rustc/2d8144b78/library/core/src/iter/range.rs:900
   2358a:	jne    23530 <<c30walk::ctor::Ctor>::build::<false>+0x120>
   2358c:	mov    0x20(%rsp),%rdx
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:3756
   23591:	mov    %edx,%eax
div_ceil():
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:3756
   23593:	mov    $0x88888889,%ecx
   23598:	imul   %rax,%rcx
   2359c:	shr    $0x24,%rcx
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:3758
   235a0:	imul   $0xeeeeeeef,%edx,%eax
   235a6:	ror    $1,%eax
   235a8:	cmp    $0x8888889,%eax
   235ad:	sbb    $0xffffffffffffffff,%rcx
   235b1:	mov    %rcx,%r12
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:734
   235b4:	cmpl   $0x0,0x3c(%rsp)
   235b9:	mov    0x50(%rsp),%r8
   235be:	mov    0x1150(%rsp),%r9
   235c6:	mov    %r15,%rdi
   235c9:	mov    %rcx,0x48(%rsp)
./examples/c30walk/ctor.rs:736
   235ce:	jne    23618 <<c30walk::ctor::Ctor>::build::<false>+0x208>
./examples/c30walk/ctor.rs:738
   235d0:	test   %rdx,%rdx
   235d3:	je     24ca8 <<c30walk::ctor::Ctor>::build::<false>+0x1898>
./examples/c30walk.rs:95
   235d9:	lea    -0x1(%rdx),%eax
phi():
./examples/c30walk.rs:95
   235dc:	mov    $0x88888889,%ecx
   235e1:	imul   %rax,%rcx
   235e5:	shr    $0x23,%rcx
   235e9:	and    $0xfffffffe,%ecx
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:741
   235ec:	lea    (%rcx,%rdx,1),%eax
   235ef:	dec    %eax
   235f1:	shr    $1,%eax
   235f3:	mov    $0x80010003,%ecx
   235f8:	imul   %rax,%rcx
   235fc:	shr    $0x2e,%rcx
   23600:	inc    %ecx
   23602:	imul   %rcx,%rsi
   23606:	dec    %rsi
   23609:	mov    0x48(%rsp),%rcx
/rustc/2d8144b78/library/core/src/cmp.rs:1077
   2360e:	cmp    %rcx,%rsi
min<usize>():
/rustc/2d8144b78/library/core/src/cmp.rs:1077
   23611:	cmovae %rcx,%rsi
   23615:	mov    %rsi,%r12
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:619
   23618:	mov    (%r8),%rax
./examples/c30walk/ctor.rs:744
   2361b:	lea    0x1(%rcx),%rbp
   2361f:	cmp    %rcx,%rax
   23622:	jbe    24cc0 <<c30walk::ctor::Ctor>::build::<false>+0x18b0>
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:619
   23628:	mov    (%r9),%rax
./examples/c30walk/ctor.rs:750
   2362b:	cmp    %r12,%rax
   2362e:	jb     24d17 <<c30walk::ctor::Ctor>::build::<false>+0x1907>
   23634:	mov    0x48(%rsp),%rax
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:3758
   23639:	lea    0x3f(%rax),%r14
div_ceil():
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:3758
   2363d:	shr    $0x6,%r14
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:3035
   23641:	movq   $0x0,0x10(%rdi)
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:3525
   23649:	je     236a6 <<c30walk::ctor::Ctor>::build::<false>+0x296>
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:767
   2364b:	cmp    (%rdi),%r14
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:673
   2364e:	ja     24e0d <<c30walk::ctor::Ctor>::build::<false>+0x19fd>
   23654:	xor    %r13d,%r13d
extend_with<u64, alloc::alloc::Global>():
   23657:	mov    %rdi,%r12
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:614
   2365a:	mov    0x8(%rdi),%r15
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   2365e:	lea    (%r15,%r13,8),%rdi
/rustc/2d8144b78/library/core/src/cmp.rs:1916
   23662:	cmp    $0x1,%r14
/rustc/2d8144b78/library/core/src/iter/range.rs:900
   23666:	je     23697 <<c30walk::ctor::Ctor>::build::<false>+0x287>
spec_next<usize>():
/rustc/2d8144b78/library/core/src/iter/range.rs:900
   23668:	lea    -0x8(,%r14,8),%rdx
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   23670:	xor    %esi,%esi
write<u64>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   23672:	call   *0x962e0(%rip)        # b9958 <memset@GLIBC_2.2.5>
   23678:	mov    0x1150(%rsp),%r9
   23680:	mov    0x50(%rsp),%r8
spec_next<usize>():
/rustc/2d8144b78/library/core/src/iter/range.rs:900
   23685:	lea    (%r14,%r13,1),%rax
   23689:	lea    (%r15,%rax,8),%rdi
   2368d:	add    $0xfffffffffffffff8,%rdi
   23691:	add    %r14,%r13
   23694:	dec    %r13
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   23697:	movq   $0x0,(%rdi)
/rustc/2d8144b78/library/alloc/src/vec/set_len_on_drop.rs:19
   2369e:	inc    %r13
increment_len():
/rustc/2d8144b78/library/alloc/src/vec/set_len_on_drop.rs:19
   236a1:	mov    %r12,%rdi
   236a4:	jmp    236a9 <<c30walk::ctor::Ctor>::build::<false>+0x299>
resize<u64, alloc::alloc::Global>():
   236a6:	xor    %r13d,%r13d
   236a9:	mov    %r13,0x10(%rdi)
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:614
   236ad:	mov    0x8(%r8),%rcx
./examples/c30walk/ctor.rs:759
   236b1:	mov    %rcx,%rax
   236b4:	and    $0x3f,%rax
   236b8:	mov    %rax,0xd8(%rsp)
   236c0:	jne    24dd1 <<c30walk::ctor::Ctor>::build::<false>+0x19c1>
   236c6:	mov    %rbp,0xc8(%rsp)
./examples/c30walk/ctor.rs:756
   236ce:	lea    0x18(%rdi),%r14
   236d2:	mov    %rdi,0xd0(%rsp)
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:614
   236da:	mov    0x8(%rdi),%rdx
non_null<alloc::alloc::Global, u64>():
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:614
   236de:	mov    %rcx,0x90(%rsp)
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:761
   236e6:	mov    %rcx,0xf8(%rsp)
   236ee:	mov    0x48(%rsp),%rax
   236f3:	mov    %rax,0x100(%rsp)
   236fb:	mov    %r9,0xd8(%rsp)
   23703:	mov    %rdx,0x10(%rsp)
   23708:	mov    %rdx,0xe0(%rsp)
   23710:	mov    %r13,0x18(%rsp)
   23715:	mov    %r13,0xe8(%rsp)
   2371d:	lea    0x108(%rsp),%rdi
   23725:	lea    0x918(%rsp),%rsi
   2372d:	mov    0x96244(%rip),%r15        # b9978 <memcpy@GLIBC_2.14>
   23734:	mov    $0x400,%edx
   23739:	call   *%r15
   2373c:	lea    0x508(%rsp),%rdi
   23744:	lea    0xd18(%rsp),%rsi
   2374c:	mov    $0x400,%edx
   23751:	call   *%r15
   23754:	movabs $0x8888888888888889,%r8
   2375e:	mov    0x20(%rsp),%r9
   23763:	movl   $0x0,0x910(%rsp)
   2376e:	movq   $0x0,0x908(%rsp)
   2377a:	mov    %r14,0xf0(%rsp)
   23782:	mov    $0xffffffffffffffff,%rcx
   23789:	xor    %r13d,%r13d
   2378c:	movabs $0xeeeeeeeeeeeeeeef,%r11
   23796:	movabs $0x888888888888889,%r12
   237a0:	pxor   %xmm0,%xmm0
   237a4:	pcmpeqd %xmm1,%xmm1
   237a8:	xor    %ebp,%ebp
   237aa:	mov    0x28(%rsp),%r10
   237af:	jmp    23870 <<c30walk::ctor::Ctor>::build::<false>+0x460>
   237b4:	data16 data16 cs nopw 0x0(%rax,%rax,1)
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:614
   237c0:	mov    0x8(%rdi),%rax
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   237c4:	mov    %r15,%rcx
add<c30walk::RawG>():
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   237c7:	shl    $0x7,%rcx
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   237cb:	movdqa %xmm0,0x70(%rax,%rcx,1)
write<c30walk::RawG>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   237d1:	movdqa %xmm0,0x60(%rax,%rcx,1)
   237d7:	movdqa %xmm0,0x50(%rax,%rcx,1)
   237dd:	movdqa %xmm0,0x40(%rax,%rcx,1)
   237e3:	movdqa %xmm0,0x30(%rax,%rcx,1)
   237e9:	movdqa %xmm0,0x20(%rax,%rcx,1)
   237ef:	movdqa %xmm0,0x10(%rax,%rcx,1)
   237f5:	movdqa %xmm0,(%rax,%rcx,1)
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:1046
   237fa:	mov    %r15,%rax
push_mut<c30walk::RawG, alloc::alloc::Global>():
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:1046
   237fd:	inc    %rax
   23800:	mov    %rax,0x10(%rdi)
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   23804:	mov    %r14,%rax
add<c30walk::Rec>():
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   23807:	shl    $0x6,%rax
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:371
   2380b:	or     $0x10000000,%r15d
create_raw<false>():
./examples/c30walk/ctor.rs:371
   23812:	mov    0x90(%rsp),%rcx
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   2381a:	movdqu %xmm1,0x2c(%rcx,%rax,1)
write<c30walk::Rec>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   23820:	movdqa %xmm1,0x20(%rcx,%rax,1)
   23826:	movdqa %xmm1,0x10(%rcx,%rax,1)
   2382c:	movdqa %xmm1,(%rcx,%rax,1)
   23831:	mov    %r15d,0x3c(%rcx,%rax,1)
create_raw<false>():
./examples/c30walk/ctor.rs:373
   23836:	mov    %r14,%rdi
   23839:	shr    $0x6,%rdi
   2383d:	cmp    0x18(%rsp),%rdi
   23842:	jae    24ec8 <<c30walk::ctor::Ctor>::build::<false>+0x1ab8>
   23848:	mov    $0x1,%eax
   2384d:	mov    %r14d,%ecx
   23850:	shl    %cl,%rax
   23853:	mov    0x10(%rsp),%rcx
   23858:	or     %rax,(%rcx,%rdi,8)
   2385c:	mov    %r14,%rcx
<c30walk::ctor::Ctor>::build::<false>:
   2385f:	add    $0x4,%rbp
/rustc/2d8144b78/library/core/src/ptr/non_null.rs:1714
   23863:	cmp    $0x400,%rbp
next<u32>():
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   2386a:	je     238f8 <<c30walk::ctor::Ctor>::build::<false>+0x4e8>
<c30walk::ctor::Ctor>::build::<false>:
   23870:	mov    %r13,%rax
scatter<false>():
./examples/c30walk/ctor.rs:841
   23873:	mov    (%rbx,%rbp,1),%r13d
./examples/c30walk/ctor.rs:842
   23877:	add    %rax,%r13
   2387a:	cmp    %r9,%r13
   2387d:	setae  %sil
./examples/c30walk/ctor.rs:844
   23881:	mov    %r13,%rax
   23884:	mul    %r8
   23887:	mov    %rdx,%r14
   2388a:	shr    $0x4,%r14
   2388e:	mov    %r13,%rax
   23891:	imul   %r11,%rax
   23895:	ror    $1,%rax
   23898:	cmp    %r12,%rax
   2389b:	setb   %al
   2389e:	cmp    %rcx,%r14
   238a1:	sete   %dl
./examples/c30walk/ctor.rs:843
   238a4:	or     %sil,%dl
   238a7:	or     %al,%dl
   238a9:	jne    2385f <<c30walk::ctor::Ctor>::build::<false>+0x44f>
   238ab:	mov    0x1150(%rsp),%rdi
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:3054
   238b3:	mov    0x10(%rdi),%r15
push_mut<c30walk::RawG, alloc::alloc::Global>():
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:1040
   238b7:	cmp    (%rdi),%r15
   238ba:	jne    237c0 <<c30walk::ctor::Ctor>::build::<false>+0x3b0>
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:1041
   238c0:	call   4d300 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   238c5:	pcmpeqd %xmm1,%xmm1
   238c9:	pxor   %xmm0,%xmm0
   238cd:	movabs $0xeeeeeeeeeeeeeeef,%r11
   238d7:	movabs $0x8888888888888889,%r8
   238e1:	mov    0x28(%rsp),%r10
   238e6:	mov    0x20(%rsp),%r9
   238eb:	mov    0x1150(%rsp),%rdi
   238f3:	jmp    237c0 <<c30walk::ctor::Ctor>::build::<false>+0x3b0>
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:734
   238f8:	cmpl   $0x0,0x3c(%rsp)
   238fd:	mov    0x1150(%rsp),%r11
   23905:	mov    0x48(%rsp),%rdi
   2390a:	mov    0x18(%rsp),%rsi
   2390f:	mov    0x10(%rsp),%r8
scatter<false>():
./examples/c30walk/ctor.rs:855
   23914:	je     23aa6 <<c30walk::ctor::Ctor>::build::<false>+0x696>
   2391a:	test   %rdi,%rdi
   2391d:	je     23aa6 <<c30walk::ctor::Ctor>::build::<false>+0x696>
   23923:	mov    %r9,%r14
   23926:	shl    $0x20,%r14
   2392a:	mov    0x3c(%rsp),%r15d
   2392f:	xor    %ebx,%ebx
   23931:	pxor   %xmm0,%xmm0
   23935:	pcmpeqd %xmm1,%xmm1
   23939:	mov    0x90(%rsp),%rbp
   23941:	mov    %r14,0x8(%rsp)
   23946:	jmp    239d0 <<c30walk::ctor::Ctor>::build::<false>+0x5c0>
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:614
   2394b:	mov    0x8(%r11),%rax
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   2394f:	mov    %r15,%rcx
add<c30walk::RawG>():
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   23952:	shl    $0x7,%rcx
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   23956:	movdqa %xmm0,0x70(%rax,%rcx,1)
write<c30walk::RawG>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   2395c:	movdqa %xmm0,0x60(%rax,%rcx,1)
   23962:	movdqa %xmm0,0x50(%rax,%rcx,1)
   23968:	movdqa %xmm0,0x40(%rax,%rcx,1)
   2396e:	movdqa %xmm0,0x30(%rax,%rcx,1)
   23974:	movdqa %xmm0,0x20(%rax,%rcx,1)
   2397a:	movdqa %xmm0,0x10(%rax,%rcx,1)
   23980:	movdqa %xmm0,(%rax,%rcx,1)
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:1046
   23985:	mov    %r15,%rax
push_mut<c30walk::RawG, alloc::alloc::Global>():
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:1046
   23988:	inc    %rax
   2398b:	mov    %rax,0x10(%r11)
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:371
   2398f:	or     $0x10000000,%r15d
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   23996:	movdqu %xmm1,0x2c(%rbp)
write<c30walk::Rec>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   2399b:	movdqa %xmm1,0x20(%rbp)
   239a0:	movdqa %xmm1,0x10(%rbp)
   239a5:	movdqa %xmm1,0x0(%rbp)
   239aa:	mov    %r15d,0x3c(%rbp)
create_raw<false>():
./examples/c30walk/ctor.rs:373
   239ae:	or     %r12,(%r8,%r13,8)
   239b2:	mov    %r14,%r15
   239b5:	mov    0x8(%rsp),%r14
scatter<false>():
   239ba:	nopw   0x0(%rax,%rax,1)
   239c0:	inc    %rbx
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/cmp.rs:1916
   239c3:	add    $0x40,%rbp
lt():
/rustc/2d8144b78/library/core/src/cmp.rs:1916
   239c7:	cmp    %rbx,%rdi
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/iter/range.rs:900
   239ca:	je     23aa6 <<c30walk::ctor::Ctor>::build::<false>+0x696>
./examples/c30walk/ctor.rs:323
   239d0:	mov    %rbx,%r13
is_raw<false>():
./examples/c30walk/ctor.rs:323
   239d3:	shr    $0x6,%r13
   239d7:	cmp    %rsi,%r13
   239da:	jae    24ed9 <<c30walk::ctor::Ctor>::build::<false>+0x1ac9>
   239e0:	mov    (%r8,%r13,8),%rax
scatter<false>():
./examples/c30walk/ctor.rs:857
   239e4:	mov    $0x1,%r12d
   239ea:	mov    %ebx,%ecx
   239ec:	shl    %cl,%r12
   239ef:	bt     %rbx,%rax
   239f3:	jb     239c0 <<c30walk::ctor::Ctor>::build::<false>+0x5b0>
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:73
   239f5:	mov    %rbx,%rax
forced():
./examples/c30walk/ctor.rs:73
   239f8:	xor    %r14,%rax
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk.rs:743
   239fb:	mov    %rax,%rcx
mix():
./examples/c30walk.rs:743
   239fe:	shr    $0x21,%rcx
   23a02:	xor    %rax,%rcx
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:2660
   23a05:	movabs $0xff51afd7ed558ccd,%rax
wrapping_mul():
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:2660
   23a0f:	imul   %rax,%rcx
mix():
./examples/c30walk.rs:745
   23a13:	mov    %rcx,%rax
   23a16:	shr    $0x21,%rax
   23a1a:	xor    %rcx,%rax
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:2660
   23a1d:	movabs $0xc4ceb9fe1a85ec53,%rcx
wrapping_mul():
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:2660
   23a27:	imul   %rcx,%rax
mix():
./examples/c30walk.rs:747
   23a2b:	mov    %rax,%rcx
   23a2e:	shr    $0x21,%rcx
   23a32:	xor    %rax,%rcx
forced():
./examples/c30walk/ctor.rs:73
   23a35:	mov    %rcx,%rax
   23a38:	shr    $0x3,%rax
   23a3c:	movabs $0x20c49ba5e353f7cf,%rdx
   23a46:	mul    %rdx
   23a49:	shr    $0x4,%rdx
   23a4d:	imul   $0x3e8,%rdx,%rax
   23a54:	sub    %rax,%rcx
   23a57:	cmp    %r15,%rcx
scatter<false>():
./examples/c30walk/ctor.rs:857
   23a5a:	jae    239c0 <<c30walk::ctor::Ctor>::build::<false>+0x5b0>
   23a60:	mov    %r15,%r14
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:3054
   23a63:	mov    0x10(%r11),%r15
push_mut<c30walk::RawG, alloc::alloc::Global>():
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:1040
   23a67:	cmp    (%r11),%r15
   23a6a:	jne    2394b <<c30walk::ctor::Ctor>::build::<false>+0x53b>
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:1041
   23a70:	mov    %r11,%rdi
   23a73:	call   4d300 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   23a78:	pcmpeqd %xmm1,%xmm1
   23a7c:	pxor   %xmm0,%xmm0
   23a80:	mov    0x10(%rsp),%r8
   23a85:	mov    0x18(%rsp),%rsi
   23a8a:	mov    0x48(%rsp),%rdi
   23a8f:	mov    0x28(%rsp),%r10
   23a94:	mov    0x20(%rsp),%r9
   23a99:	mov    0x1150(%rsp),%r11
   23aa1:	jmp    2394b <<c30walk::ctor::Ctor>::build::<false>+0x53b>
scatter<false>():
./examples/c30walk/ctor.rs:867
   23aa6:	cmp    $0x10,%r9
   23aaa:	jae    23aff <<c30walk::ctor::Ctor>::build::<false>+0x6ef>
   23aac:	xor    %r12d,%r12d
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/non_null.rs:1714
   23aaf:	cmp    %r9,%r12
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   23ab2:	jne    249ff <<c30walk::ctor::Ctor>::build::<false>+0x15ef>
/rustc/2d8144b78/library/core/src/array/equality.rs:157
   23ab8:	mov    $0x400,%edx
spec_eq<u32, u32, 256>():
/rustc/2d8144b78/library/core/src/array/equality.rs:157
   23abd:	lea    0x108(%rsp),%rdi
   23ac5:	lea    0x508(%rsp),%rsi
   23acd:	call   *0x95ead(%rip)        # b9980 <bcmp@GLIBC_2.2.5>
   23ad3:	mov    0x1150(%rsp),%r11
   23adb:	test   %eax,%eax
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:776
   23add:	je     24bbd <<c30walk::ctor::Ctor>::build::<false>+0x17ad>
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:3035
   23ae3:	movq   $0x0,0x10(%r11)
   23aeb:	xor    %eax,%eax
./examples/c30walk/ctor.rs:826
   23aed:	add    $0x1118,%rsp
   23af4:	pop    %rbx
   23af5:	pop    %r12
   23af7:	pop    %r13
   23af9:	pop    %r14
   23afb:	pop    %r15
   23afd:	pop    %rbp
   23afe:	ret
   23aff:	mov    $0x10,%r14d
   23b05:	xor    %r12d,%r12d
   23b08:	pxor   %xmm4,%xmm4
   23b0c:	xor    %ebp,%ebp
   23b0e:	jmp    23b2b <<c30walk::ctor::Ctor>::build::<false>+0x71b>
   23b10:	mov    0xb0(%rsp),%r12
scatter<false>():
./examples/c30walk/ctor.rs:867
   23b18:	lea    0x10(%r12),%rax
   23b1d:	xor    %ebp,%ebp
   23b1f:	mov    %rax,%r14
   23b22:	cmp    %r9,%rax
   23b25:	ja     24d7a <<c30walk::ctor::Ctor>::build::<false>+0x196a>
   23b2b:	mov    %r12,%rdx
   23b2e:	mov    %r13,%rdi
./examples/c30walk/ctor.rs:868
   23b31:	test   $0x1,%bpl
   23b35:	jne    23b63 <<c30walk::ctor::Ctor>::build::<false>+0x753>
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:77
   23b37:	lea    0x8(%rdx),%rsi
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:1050
   23b3b:	cmp    $0xfffffffffffffff7,%rdx
checked_sub():
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:1050
   23b3f:	ja     24daa <<c30walk::ctor::Ctor>::build::<false>+0x199a>
   23b45:	cmp    %r9,%rsi
   23b48:	ja     24daa <<c30walk::ctor::Ctor>::build::<false>+0x199a>
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:77
   23b4e:	lea    0x10(%rdx),%rax
rd():
./examples/c30walk/ctor.rs:77
   23b52:	cmp    %r9,%rax
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:1050
   23b55:	ja     24dbc <<c30walk::ctor::Ctor>::build::<false>+0x19ac>
/rustc/2d8144b78/library/core/src/result.rs:1718
   23b5b:	mov    (%r10,%rdx,1),%rbx
   23b5f:	mov    (%r10,%rsi,1),%rdi
scatter<false>():
./examples/c30walk/ctor.rs:872
   23b63:	mov    %rbx,%rax
   23b66:	xor    %rdi,%rax
   23b69:	mov    %rbx,%rcx
   23b6c:	rol    $0x8,%rcx
   23b70:	xor    %rbx,%rcx
   23b73:	or     %rax,%rcx
   23b76:	mov    %r14,0xb0(%rsp)
   23b7e:	jne    23c00 <<c30walk::ctor::Ctor>::build::<false>+0x7f0>
./examples/c30walk/ctor.rs:875
   23b84:	mov    %r14,%rcx
   23b87:	mov    %rdx,0x88(%rsp)
   23b8f:	sub    %rdx,%rcx
   23b92:	xor    %edx,%edx
   23b94:	mov    %r14,%rax
   23b97:	nopw   0x0(%rax,%rax,1)
   23ba0:	mov    %rax,%r12
   23ba3:	mov    %rdx,%r14
   23ba6:	mov    %rcx,%r15
   23ba9:	add    $0x10,%rax
   23bad:	cmp    %r9,%rax
   23bb0:	setbe  %bpl
   23bb4:	ja     23dd6 <<c30walk::ctor::Ctor>::build::<false>+0x9c6>
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:77
   23bba:	lea    0x8(%r12),%rsi
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:1050
   23bbf:	cmp    $0xfffffffffffffff7,%r12
checked_sub():
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:1050
   23bc3:	ja     24d98 <<c30walk::ctor::Ctor>::build::<false>+0x1988>
   23bc9:	cmp    %r9,%rsi
   23bcc:	ja     24d98 <<c30walk::ctor::Ctor>::build::<false>+0x1988>
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/result.rs:1718
   23bd2:	mov    (%r10,%r12,1),%rbx
   23bd6:	mov    0x8(%r10,%r12,1),%r13
scatter<false>():
./examples/c30walk/ctor.rs:877
   23bdb:	cmp    %rdi,%rbx
   23bde:	jne    23bed <<c30walk::ctor::Ctor>::build::<false>+0x7dd>
   23be0:	lea    0x1(%r14),%rdx
   23be4:	lea    0x10(%r15),%rcx
   23be8:	cmp    %rdi,%r13
   23beb:	je     23ba0 <<c30walk::ctor::Ctor>::build::<false>+0x790>
   23bed:	lea    0x10(%r12),%rax
   23bf2:	mov    %rax,0x78(%rsp)
   23bf7:	jmp    23de0 <<c30walk::ctor::Ctor>::build::<false>+0x9d0>
<c30walk::ctor::Ctor>::build::<false>:
   23bfc:	nopl   0x0(%rax)
checked_sub():
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:1050
   23c00:	cmp    $0xfffffffffffffff0,%rdx
   23c04:	jae    24df8 <<c30walk::ctor::Ctor>::build::<false>+0x19e8>
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/slice/index.rs:89
   23c0a:	add    %r10,%rdx
get_offset_len_noubcheck<u8>():
/rustc/2d8144b78/library/core/src/slice/index.rs:89
   23c0d:	mov    0xf8(%rsp),%rbx
   23c15:	mov    0xe8(%rsp),%r8
   23c1d:	mov    0xd8(%rsp),%rax
   23c25:	mov    %rax,0x18(%rsp)
   23c2a:	mov    0xe0(%rsp),%rax
   23c32:	mov    %rax,0x10(%rsp)
   23c37:	mov    0x910(%rsp),%r12d
   23c3f:	mov    0x908(%rsp),%r15
   23c47:	xor    %ebp,%ebp
   23c49:	mov    %rbx,0x8(%rsp)
   23c4e:	jmp    23c99 <<c30walk::ctor::Ctor>::build::<false>+0x889>
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   23c50:	mov    %r14d,0x3c(%rax)
   23c54:	movw   $0x0,(%rax)
   23c59:	nopl   0x0(%rax)
advance<false>():
./examples/c30walk/ctor.rs:350
   23c60:	lea    0x1(%r15),%rax
./examples/c30walk/ctor.rs:351
   23c64:	add    $0xffffffffffffffe3,%r15
   23c68:	xor    %ecx,%ecx
   23c6a:	cmp    $0x1e,%rax
   23c6e:	setae  %cl
   23c71:	cmovb  %rax,%r15
   23c75:	lea    (%r12,%rcx,2),%r12d
   23c79:	inc    %r12d
   23c7c:	mov    %r15,0x908(%rsp)
   23c84:	mov    %r12d,0x910(%rsp)
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/non_null.rs:656
   23c8c:	inc    %rbp
/rustc/2d8144b78/library/core/src/ptr/non_null.rs:1714
   23c8f:	cmp    $0x10,%rbp
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   23c93:	je     23b10 <<c30walk::ctor::Ctor>::build::<false>+0x700>
scatter<false>():
./examples/c30walk/ctor.rs:889
   23c99:	movzbl (%rdx,%rbp,1),%eax
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:602
   23c9d:	mov    0x108(%rsp,%rax,4),%esi
one<false>():
./examples/c30walk/ctor.rs:603
   23ca4:	cmp    0x508(%rsp,%rax,4),%esi
   23cab:	jae    23ae3 <<c30walk::ctor::Ctor>::build::<false>+0x6d3>
./examples/c30walk/ctor.rs:607
   23cb1:	mov    %esi,%r13d
   23cb4:	and    $0x1f,%r13d
   23cb8:	xor    %ecx,%ecx
   23cba:	cmp    $0x1d,%r13d
   23cbe:	sete   %cl
   23cc1:	lea    (%rsi,%rcx,2),%ecx
   23cc4:	inc    %ecx
   23cc6:	mov    %ecx,0x108(%rsp,%rax,4)
./examples/c30walk/ctor.rs:608
   23ccd:	shr    $0x5,%esi
./examples/c30walk/ctor.rs:610
   23cd0:	mov    %r12d,%r14d
   23cd3:	shl    $0x8,%r14d
   23cd7:	or     %eax,%r14d
./examples/c30walk/ctor.rs:611
   23cda:	test   %r13d,%r13d
   23cdd:	je     23d30 <<c30walk::ctor::Ctor>::build::<false>+0x920>
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   23cdf:	mov    %rsi,%rcx
add<c30walk::Rec>():
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   23ce2:	shl    $0x6,%rcx
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1733
   23ce6:	mov    0x3c(%rbx,%rcx,1),%eax
one<false>():
./examples/c30walk/ctor.rs:621
   23cea:	test   $0x10000000,%eax
   23cef:	jne    23d70 <<c30walk::ctor::Ctor>::build::<false>+0x960>
   23cf1:	mov    %rdx,%rbx
./examples/c30walk/ctor.rs:625
   23cf4:	mov    %eax,%edi
   23cf6:	shr    $0x8,%edi
   23cf9:	mov    %r12d,%edx
   23cfc:	sub    %edi,%edx
./examples/c30walk/ctor.rs:626
   23cfe:	cmp    $0xfffe,%edx
   23d04:	jae    23d9a <<c30walk::ctor::Ctor>::build::<false>+0x98a>
   23d0a:	mov    0x8(%rsp),%rax
   23d0f:	add    %rax,%rcx
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   23d12:	mov    %dx,(%rcx,%r13,2)
write<u16>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   23d17:	mov    %rbx,%rdx
   23d1a:	mov    %rax,%rbx
one<false>():
./examples/c30walk/ctor.rs:611
   23d1d:	jmp    23c60 <<c30walk::ctor::Ctor>::build::<false>+0x850>
   23d22:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:323
   23d30:	mov    %esi,%edi
is_raw<false>():
./examples/c30walk/ctor.rs:323
   23d32:	shr    $0x6,%edi
   23d35:	cmp    %r8,%rdi
   23d38:	jae    24ee8 <<c30walk::ctor::Ctor>::build::<false>+0x1ad8>
   23d3e:	mov    0x10(%rsp),%rax
   23d43:	mov    (%rax,%rdi,8),%rcx
one<false>():
   23d47:	mov    %rsi,%rax
   23d4a:	shl    $0x6,%rax
   23d4e:	add    %rbx,%rax
./examples/c30walk/ctor.rs:612
   23d51:	bt     %rsi,%rcx
   23d55:	jae    23c50 <<c30walk::ctor::Ctor>::build::<false>+0x840>
   23d5b:	mov    0x3c(%rax),%eax
./examples/c30walk/ctor.rs:613
   23d5e:	mov    $0xfffff,%ecx
   23d63:	and    %ecx,%eax
   23d65:	jmp    23d75 <<c30walk::ctor::Ctor>::build::<false>+0x965>
   23d67:	nopw   0x0(%rax,%rax,1)
./examples/c30walk/ctor.rs:622
   23d70:	and    $0xfffff,%eax
./examples/c30walk/ctor.rs:635
   23d75:	mov    %eax,%edi
   23d77:	mov    0x18(%rsp),%rax
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:1903
   23d7c:	mov    0x10(%rax),%rsi
/rustc/2d8144b78/library/core/src/slice/index.rs:278
   23d80:	cmp    %rdi,%rsi
index_mut<c30walk::RawG>():
/rustc/2d8144b78/library/core/src/slice/index.rs:278
   23d83:	jbe    24bb1 <<c30walk::ctor::Ctor>::build::<false>+0x17a1>
   23d89:	shl    $0x7,%rdi
   23d8d:	add    0x8(%rax),%rdi
one<false>():
./examples/c30walk/ctor.rs:635
   23d91:	mov    %r14d,(%rdi,%r13,4)
   23d95:	jmp    23c60 <<c30walk::ctor::Ctor>::build::<false>+0x850>
./examples/c30walk/ctor.rs:627
   23d9a:	lea    0xd8(%rsp),%rdi
   23da2:	mov    %r13,%rdx
   23da5:	mov    %eax,%ecx
   23da7:	mov    %r8,0x60(%rsp)
   23dac:	call   4dcb0 <<c30walk::ctor::W<false>>::promote>
   23db1:	mov    0x60(%rsp),%r8
   23db6:	pxor   %xmm4,%xmm4
   23dba:	mov    0x28(%rsp),%r10
   23dbf:	mov    0x20(%rsp),%r9
   23dc4:	mov    0x1150(%rsp),%r11
   23dcc:	mov    %rbx,%rdx
   23dcf:	mov    0x8(%rsp),%rbx
./examples/c30walk/ctor.rs:621
   23dd4:	jmp    23d75 <<c30walk::ctor::Ctor>::build::<false>+0x965>
scatter<false>():
./examples/c30walk/ctor.rs:884
   23dd6:	lea    0x10(%r12),%rax
   23ddb:	mov    %rax,0x78(%rsp)
   23de0:	mov    0x88(%rsp),%rax
   23de8:	mov    %r12,%rcx
   23deb:	sub    %rax,%rcx
   23dee:	mov    %rdi,0x70(%rsp)
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:536
   23df3:	movzbl %dil,%esi
run<false>():
./examples/c30walk/ctor.rs:537
   23df7:	mov    %esi,%r11d
   23dfa:	mov    0x108(%rsp,%r11,4),%edi
./examples/c30walk/ctor.rs:538
   23e02:	mov    %edi,%r8d
   23e05:	and    $0x1f,%r8d
./examples/c30walk/ctor.rs:539
   23e09:	lea    (%rcx,%r8,1),%rax
   23e0d:	mov    %rax,0x10(%rsp)
./examples/c30walk/ctor.rs:541
   23e12:	movabs $0x8888888888888889,%rdx
   23e1c:	mul    %rdx
   23e1f:	shr    $0x4,%rdx
./examples/c30walk/ctor.rs:542
   23e23:	lea    (%rcx,%rdi,1),%eax
   23e26:	mov    %rdx,0x60(%rsp)
   23e2b:	lea    (%rax,%rdx,2),%eax
./examples/c30walk/ctor.rs:543
   23e2e:	cmp    0x508(%rsp,%r11,4),%eax
   23e36:	mov    0x1150(%rsp),%r11
   23e3e:	ja     23ae3 <<c30walk::ctor::Ctor>::build::<false>+0x6d3>
./examples/c30walk/ctor.rs:546
   23e44:	mov    %eax,0x108(%rsp,%rsi,4)
   23e4b:	mov    %rdi,0xa8(%rsp)
./examples/c30walk/ctor.rs:554
   23e53:	shr    $0x5,%edi
./examples/c30walk/ctor.rs:556
   23e56:	test   %r8d,%r8d
   23e59:	mov    %r8,0x40(%rsp)
   23e5e:	je     23f6a <<c30walk::ctor::Ctor>::build::<false>+0xb5a>
   23e64:	mov    %rsi,0xb8(%rsp)
./examples/c30walk/ctor.rs:557
   23e6c:	mov    $0x1e,%edx
   23e71:	sub    %r8,%rdx
   23e74:	lea    (%rcx,%r8,1),%rax
   23e78:	cmp    $0x1e,%rax
   23e7c:	cmovb  %rcx,%rdx
   23e80:	mov    %rdx,0x30(%rsp)
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:426
   23e85:	mov    0x908(%rsp),%rsi
fragment<false>():
./examples/c30walk/ctor.rs:426
   23e8d:	cmp    $0x1e,%rsi
   23e91:	jae    24ef7 <<c30walk::ctor::Ctor>::build::<false>+0x1ae7>
   23e97:	mov    %r8,%r10
   23e9a:	mov    %rsi,%rcx
   23e9d:	shl    $0x6,%rcx
   23ea1:	lea    -0x147e8(%rip),%rax        # f6c0 <c30walk::ctor::PATTERNS>
   23ea8:	mov    %rcx,0x58(%rsp)
   23ead:	lea    (%rax,%rcx,1),%r9
./examples/c30walk/ctor.rs:427
   23eb1:	mov    0x910(%rsp),%eax
   23eb8:	mov    %eax,0x18(%rsp)
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:330
   23ebc:	mov    0xf8(%rsp),%rcx
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   23ec4:	mov    %rdi,%rax
add<c30walk::Rec>():
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   23ec7:	shl    $0x6,%rax
   23ecb:	mov    %rax,0x80(%rsp)
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1733
   23ed3:	mov    0x3c(%rcx,%rax,1),%eax
fragment<false>():
./examples/c30walk/ctor.rs:442
   23ed7:	test   $0x10000000,%eax
   23edc:	mov    %rdi,0x8(%rsp)
   23ee1:	mov    %rsi,0xc0(%rsp)
   23ee9:	mov    %r9,0x68(%rsp)
   23eee:	jne    23f7d <<c30walk::ctor::Ctor>::build::<false>+0xb6d>
   23ef4:	mov    0x30(%rsp),%r8
./examples/c30walk/ctor.rs:448
   23ef9:	lea    -0x1(%r8),%rdi
   23efd:	cmp    $0x1d,%rdi
   23f01:	ja     24f2e <<c30walk::ctor::Ctor>::build::<false>+0x1b1e>
   23f07:	mov    %eax,%edi
   23f09:	shr    $0x8,%edi
   23f0c:	mov    0x18(%rsp),%edx
   23f10:	sub    %edi,%edx
   23f12:	movzwl -0x2(%r9,%r8,2),%edi
   23f18:	add    %edx,%edi
   23f1a:	cmp    $0xfffd,%edi
   23f20:	ja     249cd <<c30walk::ctor::Ctor>::build::<false>+0x15bd>
   23f26:	mov    0x10(%rsp),%rax
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   23f2b:	cmp    $0x1e,%rax
next<u16>():
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   23f2f:	mov    $0x1e,%edi
   23f34:	cmovb  %rax,%rdi
   23f38:	mov    %r10,%rax
   23f3b:	not    %rax
   23f3e:	add    %rdi,%rax
   23f41:	movabs $0x7fffffffffffffff,%rdi
   23f4b:	and    %rdi,%rax
   23f4e:	cmp    $0x3,%rax
   23f52:	jae    24060 <<c30walk::ctor::Ctor>::build::<false>+0xc50>
   23f58:	xor    %eax,%eax
   23f5a:	mov    %r9,%r10
   23f5d:	mov    0x80(%rsp),%r9
   23f65:	jmp    2431f <<c30walk::ctor::Ctor>::build::<false>+0xf0f>
run<false>():
   23f6a:	mov    0x60(%rsp),%rcx
./examples/c30walk/ctor.rs:572
   23f6f:	test   %rcx,%rcx
   23f72:	jne    243cc <<c30walk::ctor::Ctor>::build::<false>+0xfbc>
   23f78:	jmp    24413 <<c30walk::ctor::Ctor>::build::<false>+0x1003>
fragment<false>():
./examples/c30walk/ctor.rs:443
   23f7d:	and    $0xfffff,%eax
   23f82:	mov    0x30(%rsp),%r8
./examples/c30walk/ctor.rs:464
   23f87:	mov    0xd8(%rsp),%rcx
   23f8f:	mov    %eax,%edi
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:1903
   23f91:	mov    0x10(%rcx),%rsi
/rustc/2d8144b78/library/core/src/slice/index.rs:278
   23f95:	cmp    %rdi,%rsi
index_mut<c30walk::RawG>():
/rustc/2d8144b78/library/core/src/slice/index.rs:278
   23f98:	jbe    24f3f <<c30walk::ctor::Ctor>::build::<false>+0x1b2f>
fragment<false>():
./examples/c30walk/ctor.rs:464
   23f9e:	lea    (%r8,%r10,1),%rsi
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:1050
   23fa2:	cmp    %r10,%rsi
checked_sub():
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:1050
   23fa5:	jb     24e3e <<c30walk::ctor::Ctor>::build::<false>+0x1a2e>
   23fab:	cmp    $0x21,%rsi
   23faf:	jae    24e3e <<c30walk::ctor::Ctor>::build::<false>+0x1a2e>
fragment<false>():
   23fb5:	cmp    $0x1f,%r8
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:1050
   23fb9:	jae    24e52 <<c30walk::ctor::Ctor>::build::<false>+0x1a42>
/rustc/2d8144b78/library/core/src/iter/adapters/zip.rs:304
   23fbf:	test   %r8,%r8
next<core::slice::iter::IterMut<u32>, core::slice::iter::Iter<u16>>():
/rustc/2d8144b78/library/core/src/iter/adapters/zip.rs:304
   23fc2:	je     24357 <<c30walk::ctor::Ctor>::build::<false>+0xf47>
   23fc8:	mov    %r8,%r10
   23fcb:	mov    0x8(%rcx),%r8
   23fcf:	shl    $0x7,%rdi
   23fd3:	lea    (%r8,%rdi,1),%rdx
   23fd7:	mov    0x40(%rsp),%r9
   23fdc:	lea    (%rdx,%r9,4),%rax
   23fe0:	movzbl 0x70(%rsp),%ecx
   23fe5:	cmp    $0x8,%r10
   23fe9:	jb     2401c <<c30walk::ctor::Ctor>::build::<false>+0xc0c>
   23feb:	add    %r9d,%r9d
   23fee:	mov    0x58(%rsp),%r10
   23ff3:	sub    %r9,%r10
   23ff6:	lea    -0x1493d(%rip),%r9        # f6c0 <c30walk::ctor::PATTERNS>
   23ffd:	add    %r9,%r10
   24000:	lea    (%r10,%rsi,2),%r9
   24004:	cmp    %r9,%rax
/rustc/2d8144b78/library/core/src/iter/adapters/zip.rs:308
   24007:	jae    240bf <<c30walk::ctor::Ctor>::build::<false>+0xcaf>
   2400d:	lea    (%rdx,%rsi,4),%rdx
   24011:	cmp    %rdx,0x68(%rsp)
   24016:	jae    240bf <<c30walk::ctor::Ctor>::build::<false>+0xcaf>
   2401c:	xor    %edx,%edx
   2401e:	mov    0x30(%rsp),%r8
   24023:	mov    0x58(%rsp),%rsi
/rustc/2d8144b78/library/core/src/iter/adapters/zip.rs:304
   24028:	lea    (%rsi,%rdx,2),%rsi
   2402c:	lea    -0x14973(%rip),%rdi        # f6c0 <c30walk::ctor::PATTERNS>
   24033:	add    %rdi,%rsi
fragment<false>():
   24036:	cs nopw 0x0(%rax,%rax,1)
./examples/c30walk/ctor.rs:465
   24040:	movzwl (%rsi),%edi
./examples/c30walk/ctor.rs:466
   24043:	add    0x18(%rsp),%edi
   24047:	shl    $0x8,%edi
   2404a:	or     %ecx,%edi
   2404c:	mov    %edi,(%rax,%rdx,4)
next<core::slice::iter::IterMut<u32>, core::slice::iter::Iter<u16>>():
/rustc/2d8144b78/library/core/src/iter/adapters/zip.rs:308
   2404f:	inc    %rdx
/rustc/2d8144b78/library/core/src/iter/adapters/zip.rs:304
   24052:	add    $0x2,%rsi
   24056:	cmp    %r8,%rdx
   24059:	jb     24040 <<c30walk::ctor::Ctor>::build::<false>+0xc30>
   2405b:	jmp    24357 <<c30walk::ctor::Ctor>::build::<false>+0xf47>
next<u16>():
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   24060:	mov    %rcx,%rdi
   24063:	lea    -0x149aa(%rip),%r9        # f6c0 <c30walk::ctor::PATTERNS>
   2406a:	sub    %r9,%rdi
   2406d:	mov    0x80(%rsp),%r9
   24075:	add    %r9,%rdi
   24078:	lea    (%r10,%r10,1),%esi
   2407c:	add    %rsi,%rdi
   2407f:	sub    0x58(%rsp),%rdi
   24084:	cmp    $0x20,%rdi
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/iter/adapters/enumerate.rs:82
   24088:	jae    24096 <<c30walk::ctor::Ctor>::build::<false>+0xc86>
next<core::slice::iter::Iter<u16>>():
   2408a:	xor    %eax,%eax
   2408c:	mov    0x68(%rsp),%r10
/rustc/2d8144b78/library/core/src/iter/adapters/enumerate.rs:82
   24091:	jmp    2431f <<c30walk::ctor::Ctor>::build::<false>+0xf0f>
   24096:	mov    %rsi,0x98(%rsp)
   2409e:	lea    0x1(%rax),%rsi
next<u16>():
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   240a2:	cmp    $0xf,%rax
   240a6:	mov    %ebp,0x38(%rsp)
   240aa:	mov    %rsi,0xa0(%rsp)
   240b2:	jae    24178 <<c30walk::ctor::Ctor>::build::<false>+0xd68>
   240b8:	xor    %eax,%eax
   240ba:	jmp    24256 <<c30walk::ctor::Ctor>::build::<false>+0xe46>
fragment<false>():
   240bf:	mov    0x30(%rsp),%rdx
   240c4:	and    $0x18,%edx
   240c7:	movd   %ecx,%xmm0
   240cb:	pshufd $0x0,%xmm0,%xmm0
   240d0:	movd   0x18(%rsp),%xmm1
   240d6:	pshufd $0x0,%xmm1,%xmm1
   240db:	mov    0x40(%rsp),%r10
next<core::slice::iter::IterMut<u32>, core::slice::iter::Iter<u16>>():
/rustc/2d8144b78/library/core/src/iter/adapters/zip.rs:308
   240e0:	lea    (%r10,%r15,1),%rsi
   240e4:	cmp    $0x1e,%rsi
   240e8:	mov    $0x1e,%r9d
   240ee:	cmovae %r9,%rsi
   240f2:	sub    %r10d,%esi
   240f5:	and    $0x18,%esi
   240f8:	lea    -0x14a3f(%rip),%r9        # f6c0 <c30walk::ctor::PATTERNS>
   240ff:	mov    0x58(%rsp),%r10
   24104:	add    %r10,%r9
   24107:	add    $0x8,%r9
   2410b:	mov    0x40(%rsp),%r10
   24110:	lea    (%rdi,%r10,4),%rdi
   24114:	add    %r8,%rdi
   24117:	add    $0x10,%rdi
   2411b:	xor    %r8d,%r8d
fragment<false>():
   2411e:	xchg   %ax,%ax
./examples/c30walk/ctor.rs:465
   24120:	movq   -0x8(%r9,%r8,2),%xmm2
   24127:	movq   (%r9,%r8,2),%xmm3
./examples/c30walk/ctor.rs:466
   2412d:	punpcklwd %xmm4,%xmm2
   24131:	punpcklwd %xmm4,%xmm3
   24135:	paddd  %xmm1,%xmm2
   24139:	paddd  %xmm1,%xmm3
   2413d:	pslld  $0x8,%xmm2
   24142:	pslld  $0x8,%xmm3
   24147:	por    %xmm0,%xmm2
   2414b:	por    %xmm0,%xmm3
   2414f:	movdqu %xmm2,-0x10(%rdi,%r8,4)
   24156:	movdqu %xmm3,(%rdi,%r8,4)
next<core::slice::iter::IterMut<u32>, core::slice::iter::Iter<u16>>():
/rustc/2d8144b78/library/core/src/iter/adapters/zip.rs:308
   2415c:	add    $0x8,%r8
/rustc/2d8144b78/library/core/src/iter/adapters/zip.rs:304
   24160:	cmp    %r8,%rsi
   24163:	jne    24120 <<c30walk::ctor::Ctor>::build::<false>+0xd10>
   24165:	mov    0x30(%rsp),%r8
   2416a:	cmp    %rdx,%r8
   2416d:	jne    24023 <<c30walk::ctor::Ctor>::build::<false>+0xc13>
   24173:	jmp    24357 <<c30walk::ctor::Ctor>::build::<false>+0xf47>
fragment<false>():
   24178:	mov    %rsi,%rax
   2417b:	and    $0xfffffffffffffff0,%rax
   2417f:	movd   %edx,%xmm0
   24183:	pshuflw $0x0,%xmm0,%xmm0
   24188:	pshufd $0x44,%xmm0,%xmm0
next<u16>():
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   2418d:	lea    -0x14ad4(%rip),%rdi        # f6c0 <c30walk::ctor::PATTERNS>
   24194:	mov    %r10,%rsi
   24197:	mov    0x58(%rsp),%r10
   2419c:	lea    0x10(%r10,%rdi,1),%r10
   241a1:	mov    0xa8(%rsp),%r11
   241a9:	not    %r11d
   241ac:	lea    (%rsi,%r15,1),%rdi
   241b0:	cmp    $0x1e,%rdi
   241b4:	mov    $0x1e,%ebp
   241b9:	cmovae %rbp,%rdi
   241bd:	movabs $0x7fffffffffffffe0,%rbp
   241c7:	or     %rbp,%r11
   241ca:	add    %rdi,%r11
   241cd:	movabs $0x7fffffffffffffff,%rdi
   241d7:	and    %rdi,%r11
   241da:	inc    %r11
   241dd:	and    $0xfffffffffffffff0,%r11
   241e1:	mov    0x98(%rsp),%rsi
   241e9:	lea    (%r9,%rsi,1),%rdi
   241ed:	lea    (%rcx,%rdi,1),%rbp
   241f1:	add    $0x10,%rbp
   241f5:	xor    %edi,%edi
fragment<false>():
   241f7:	nopw   0x0(%rax,%rax,1)
   24200:	movdqa -0x10(%r10,%rdi,2),%xmm1
./examples/c30walk/ctor.rs:457
   24207:	paddw  %xmm0,%xmm1
   2420b:	movdqa (%r10,%rdi,2),%xmm2
   24211:	paddw  %xmm0,%xmm2
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   24215:	movdqu %xmm1,-0x10(%rbp,%rdi,2)
write<u16>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   2421b:	movdqu %xmm2,0x0(%rbp,%rdi,2)
next<core::slice::iter::Iter<u16>>():
/rustc/2d8144b78/library/core/src/iter/adapters/enumerate.rs:82
   24221:	add    $0x10,%rdi
next<u16>():
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   24225:	cmp    %rdi,%r11
   24228:	jne    24200 <<c30walk::ctor::Ctor>::build::<false>+0xdf0>
   2422a:	mov    0xa0(%rsp),%rsi
   24232:	cmp    %rax,%rsi
   24235:	mov    0x1150(%rsp),%r11
   2423d:	mov    0x38(%rsp),%ebp
   24241:	je     24357 <<c30walk::ctor::Ctor>::build::<false>+0xf47>
fragment<false>():
   24247:	test   $0xc,%sil
   2424b:	je     249b6 <<c30walk::ctor::Ctor>::build::<false>+0x15a6>
   24251:	mov    0x40(%rsp),%r10
   24256:	mov    %rax,%rbp
   24259:	mov    %rsi,%rax
   2425c:	and    $0xfffffffffffffffc,%rax
   24260:	mov    0x58(%rsp),%rsi
   24265:	lea    (%rsi,%rbp,2),%r11
   24269:	lea    -0x14bb0(%rip),%rdi        # f6c0 <c30walk::ctor::PATTERNS>
   24270:	add    %rdi,%r11
   24273:	mov    0xa8(%rsp),%r9
   2427b:	not    %r9d
   2427e:	lea    (%r10,%r15,1),%rdi
   24282:	cmp    $0x1e,%rdi
   24286:	mov    $0x1e,%r10d
   2428c:	cmovae %r10,%rdi
   24290:	movabs $0x7fffffffffffffe0,%r10
   2429a:	or     %r10,%r9
   2429d:	add    %rdi,%r9
   242a0:	mov    0x68(%rsp),%rsi
   242a5:	lea    (%rsi,%rax,2),%r10
   242a9:	mov    %r9,%rsi
   242ac:	movd   %edx,%xmm0
   242b0:	pshuflw $0x0,%xmm0,%xmm0
   242b5:	movabs $0x7fffffffffffffff,%rdi
   242bf:	and    %rdi,%rsi
   242c2:	inc    %rsi
   242c5:	and    $0xfffffffffffffffc,%rsi
   242c9:	sub    %rbp,%rsi
   242cc:	mov    0x80(%rsp),%r9
   242d4:	lea    (%r9,%rbp,2),%rbp
   242d8:	add    0x98(%rsp),%rbp
   242e0:	add    %rcx,%rbp
   242e3:	xor    %edi,%edi
   242e5:	data16 cs nopw 0x0(%rax,%rax,1)
./examples/c30walk/ctor.rs:456
   242f0:	movq   (%r11,%rdi,2),%xmm1
./examples/c30walk/ctor.rs:457
   242f6:	paddw  %xmm0,%xmm1
write<u16>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   242fa:	movq   %xmm1,0x0(%rbp,%rdi,2)
next<u16>():
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   24300:	add    $0x4,%rdi
   24304:	cmp    %rdi,%rsi
   24307:	jne    242f0 <<c30walk::ctor::Ctor>::build::<false>+0xee0>
   24309:	cmp    %rax,0xa0(%rsp)
   24311:	mov    0x1150(%rsp),%r11
   24319:	mov    0x38(%rsp),%ebp
   2431d:	je     24357 <<c30walk::ctor::Ctor>::build::<false>+0xf47>
fragment<false>():
   2431f:	mov    0x68(%rsp),%rsi
./examples/c30walk/ctor.rs:456
   24324:	lea    (%rsi,%r8,2),%rsi
next<u16>():
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   24328:	lea    (%r9,%rax,2),%rax
   2432c:	mov    0x40(%rsp),%rdi
   24331:	lea    (%rax,%rdi,2),%rax
   24335:	add    %rax,%rcx
fragment<false>():
   24338:	nopl   0x0(%rax,%rax,1)
   24340:	movzwl (%r10),%eax
./examples/c30walk/ctor.rs:457
   24344:	add    %dx,%ax
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/non_null.rs:656
   24347:	add    $0x2,%r10
write<u16>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   2434b:	mov    %ax,(%rcx)
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/non_null.rs:1714
   2434e:	add    $0x2,%rcx
eq<u16>():
/rustc/2d8144b78/library/core/src/ptr/non_null.rs:1714
   24352:	cmp    %rsi,%r10
next<u16>():
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   24355:	jne    24340 <<c30walk::ctor::Ctor>::build::<false>+0xf30>
fragment<false>():
   24357:	mov    0xc0(%rsp),%rdx
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:350
   2435f:	add    %r8,%rdx
advance<false>():
./examples/c30walk/ctor.rs:351
   24362:	lea    0x2(%r8),%eax
   24366:	mov    %rdx,%rcx
   24369:	sub    $0x1e,%rcx
   2436d:	cmovb  %rdx,%rcx
   24371:	cmovb  %r8d,%eax
   24375:	add    0x18(%rsp),%eax
   24379:	mov    %rcx,0x908(%rsp)
   24381:	mov    %eax,0x910(%rsp)
run<false>():
./examples/c30walk/ctor.rs:557
   24388:	cmpq   $0x1e,0x10(%rsp)
./examples/c30walk/ctor.rs:566
   2438e:	jae    243a4 <<c30walk::ctor::Ctor>::build::<false>+0xf94>
   24390:	mov    0x20(%rsp),%r9
   24395:	mov    0x28(%rsp),%r10
   2439a:	mov    0x78(%rsp),%rax
./examples/c30walk/ctor.rs:581
   2439f:	jmp    23b1f <<c30walk::ctor::Ctor>::build::<false>+0x70f>
   243a4:	mov    0x8(%rsp),%rdi
./examples/c30walk/ctor.rs:569
   243a9:	inc    %rdi
   243ac:	mov    0x60(%rsp),%rax
./examples/c30walk/ctor.rs:570
   243b1:	lea    -0x1(%rax),%rcx
   243b5:	mov    0x20(%rsp),%r9
   243ba:	mov    0x28(%rsp),%r10
   243bf:	mov    0xb8(%rsp),%rsi
./examples/c30walk/ctor.rs:572
   243c7:	test   %rcx,%rcx
   243ca:	je     24413 <<c30walk::ctor::Ctor>::build::<false>+0x1003>
<c30walk::ctor::Ctor>::build::<false>:
   243cc:	mov    %rdi,0x8(%rsp)
./examples/c30walk/ctor.rs:734
   243d1:	cmpl   $0x0,0x3c(%rsp)
   243d6:	mov    %rcx,0x18(%rsp)
run<false>():
./examples/c30walk/ctor.rs:573
   243db:	je     24590 <<c30walk::ctor::Ctor>::build::<false>+0x1180>
./examples/c30walk/ctor.rs:574
   243e1:	lea    0xd8(%rsp),%rdi
   243e9:	mov    0x8(%rsp),%rdx
   243ee:	call   4d8f0 <<c30walk::ctor::W<false>>::complete_checked>
   243f3:	mov    0x8(%rsp),%rdi
   243f8:	pxor   %xmm4,%xmm4
   243fc:	mov    0x28(%rsp),%r10
   24401:	mov    0x20(%rsp),%r9
   24406:	mov    0x1150(%rsp),%r11
./examples/c30walk/ctor.rs:578
   2440e:	add    0x18(%rsp),%rdi
./examples/c30walk/ctor.rs:580
   24413:	imul   $0xffffffffffffffe2,0x60(%rsp),%rdx
./examples/c30walk/ctor.rs:581
   24419:	mov    %rdx,%rsi
   2441c:	add    0x10(%rsp),%rsi
   24421:	je     247d7 <<c30walk::ctor::Ctor>::build::<false>+0x13c7>
   24427:	mov    %rdi,%r9
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:426
   2442a:	mov    0x908(%rsp),%rdi
fragment<false>():
./examples/c30walk/ctor.rs:426
   24432:	cmp    $0x1d,%rdi
   24436:	ja     24f0b <<c30walk::ctor::Ctor>::build::<false>+0x1afb>
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:323
   2443c:	mov    %r9,%rax
is_raw<false>():
./examples/c30walk/ctor.rs:323
   2443f:	shr    $0x6,%rax
   24443:	mov    0xe8(%rsp),%rcx
   2444b:	cmp    %rcx,%rax
   2444e:	mov    0x40(%rsp),%r11
   24453:	jae    24f1c <<c30walk::ctor::Ctor>::build::<false>+0x1b0c>
fragment<false>():
   24459:	mov    %rdi,0x18(%rsp)
   2445e:	mov    %rdi,%r8
   24461:	shl    $0x6,%r8
   24465:	lea    -0x14dac(%rip),%rcx        # f6c0 <c30walk::ctor::PATTERNS>
   2446c:	mov    %r8,%r10
   2446f:	lea    (%rcx,%r8,1),%rdi
   24473:	mov    0x910(%rsp),%r8d
is_raw<false>():
./examples/c30walk/ctor.rs:323
   2447b:	mov    0xe0(%rsp),%rcx
   24483:	mov    (%rcx,%rax,8),%rax
fragment<false>():
./examples/c30walk/ctor.rs:429
   24487:	bt     %r9,%rax
   2448b:	mov    %ebp,0x38(%rsp)
   2448f:	jae    245e8 <<c30walk::ctor::Ctor>::build::<false>+0x11d8>
./examples/c30walk/ctor.rs:464
   24495:	mov    0xd8(%rsp),%r11
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:330
   2449d:	mov    0xf8(%rsp),%rax
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   244a5:	shl    $0x6,%r9
add<c30walk::Rec>():
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   244a9:	mov    0x3c(%rax,%r9,1),%ecx
fragment<false>():
./examples/c30walk/ctor.rs:430
   244ae:	and    $0xfffff,%ecx
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:1903
   244b4:	mov    0x10(%r11),%rax
/rustc/2d8144b78/library/core/src/slice/index.rs:278
   244b8:	cmp    %rcx,%rax
index_mut<c30walk::RawG>():
/rustc/2d8144b78/library/core/src/slice/index.rs:278
   244bb:	jbe    24f4b <<c30walk::ctor::Ctor>::build::<false>+0x1b3b>
fragment<false>():
   244c1:	cmp    $0x21,%rsi
   244c5:	mov    %r10,0x10(%rsp)
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:1050
   244ca:	jae    24e68 <<c30walk::ctor::Ctor>::build::<false>+0x1a58>
fragment<false>():
   244d0:	cmp    $0x1e,%rsi
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:1050
   244d4:	ja     24e7b <<c30walk::ctor::Ctor>::build::<false>+0x1a6b>
fragment<false>():
   244da:	mov    %rdi,%r9
   244dd:	mov    0x8(%r11),%rdi
   244e1:	mov    %ecx,%r11d
   244e4:	shl    $0x7,%r11d
   244e8:	add    %rdi,%r11
   244eb:	movzbl 0x70(%rsp),%ebp
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/iter/adapters/zip.rs:304
   244f0:	cmp    $0xc,%rsi
next<core::slice::iter::IterMut<u32>, core::slice::iter::Iter<u16>>():
/rustc/2d8144b78/library/core/src/iter/adapters/zip.rs:304
   244f4:	jb     24557 <<c30walk::ctor::Ctor>::build::<false>+0x1147>
   244f6:	mov    0x88(%rsp),%r10
   244fe:	not    %r10
   24501:	mov    0xb0(%rsp),%rax
   24509:	add    %r10,%rax
   2450c:	mov    0x40(%rsp),%r10
   24511:	shl    $0x4,%r14
   24515:	add    %rax,%r14
   24518:	add    %r10,%r14
   2451b:	add    %r14,%rdx
   2451e:	lea    -0x14e65(%rip),%rax        # f6c0 <c30walk::ctor::PATTERNS>
   24525:	mov    0x10(%rsp),%r14
   2452a:	add    %r14,%rax
   2452d:	lea    (%rax,%rdx,2),%rax
   24531:	add    $0x2,%rax
   24535:	cmp    %rax,%r11
/rustc/2d8144b78/library/core/src/iter/adapters/zip.rs:308
   24538:	jae    247e1 <<c30walk::ctor::Ctor>::build::<false>+0x13d1>
   2453e:	mov    %ecx,%eax
   24540:	shl    $0x7,%eax
   24543:	add    %rdi,%rax
   24546:	lea    (%rax,%rdx,4),%rax
   2454a:	add    $0x4,%rax
   2454e:	cmp    %rax,%r9
   24551:	jae    247e1 <<c30walk::ctor::Ctor>::build::<false>+0x13d1>
   24557:	xor    %r10d,%r10d
   2455a:	mov    0x10(%rsp),%rax
/rustc/2d8144b78/library/core/src/iter/adapters/zip.rs:304
   2455f:	lea    (%rax,%r10,2),%rax
   24563:	lea    -0x14eaa(%rip),%rcx        # f6c0 <c30walk::ctor::PATTERNS>
   2456a:	add    %rcx,%rax
fragment<false>():
   2456d:	nopl   (%rax)
./examples/c30walk/ctor.rs:465
   24570:	movzwl (%rax),%ecx
./examples/c30walk/ctor.rs:466
   24573:	add    %r8d,%ecx
   24576:	shl    $0x8,%ecx
   24579:	or     %ebp,%ecx
   2457b:	mov    %ecx,(%r11,%r10,4)
next<core::slice::iter::IterMut<u32>, core::slice::iter::Iter<u16>>():
/rustc/2d8144b78/library/core/src/iter/adapters/zip.rs:308
   2457f:	inc    %r10
/rustc/2d8144b78/library/core/src/iter/adapters/zip.rs:304
   24582:	add    $0x2,%rax
   24586:	cmp    %rsi,%r10
   24589:	jb     24570 <<c30walk::ctor::Ctor>::build::<false>+0x1160>
   2458b:	jmp    24685 <<c30walk::ctor::Ctor>::build::<false>+0x1275>
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:485
   24590:	mov    0x908(%rsp),%rdi
complete<false>():
./examples/c30walk/ctor.rs:485
   24598:	cmp    $0x1d,%rdi
   2459c:	ja     24f5d <<c30walk::ctor::Ctor>::build::<false>+0x1b4d>
   245a2:	shl    $0x6,%rdi
   245a6:	lea    -0x14eed(%rip),%rax        # f6c0 <c30walk::ctor::PATTERNS>
   245ad:	add    %rax,%rdi
./examples/c30walk/ctor.rs:486
   245b0:	mov    0x910(%rsp),%edx
   245b7:	mov    %edx,0x30(%rsp)
   245bb:	shl    $0x8,%edx
   245be:	movzbl 0x70(%rsp),%eax
   245c3:	or     %edx,%eax
./examples/c30walk/ctor.rs:487
   245c5:	mov    0xf8(%rsp),%rdx
   245cd:	mov    0x8(%rsp),%rsi
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   245d2:	shl    $0x6,%rsi
/rustc/2d8144b78/library/core/src/iter/range.rs:900
   245d6:	cmp    $0x1,%rcx
spec_next<usize>():
/rustc/2d8144b78/library/core/src/iter/range.rs:900
   245da:	jne    246d0 <<c30walk::ctor::Ctor>::build::<false>+0x12c0>
   245e0:	xor    %r8d,%r8d
   245e3:	jmp    2476c <<c30walk::ctor::Ctor>::build::<false>+0x135c>
fragment<false>():
./examples/c30walk/ctor.rs:433
   245e8:	mov    %r8d,%eax
   245eb:	shl    $0x8,%eax
   245ee:	movzbl 0x70(%rsp),%edx
   245f3:	or     %eax,%edx
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:337
   245f5:	mov    0xf8(%rsp),%rcx
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   245fd:	shl    $0x6,%r9
add<c30walk::Rec>():
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   24601:	mov    %r9,0x8(%rsp)
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   24606:	mov    %edx,0x3c(%rcx,%r9,1)
write<u32>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   2460b:	cmp    $0x1f,%rsi
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:1050
   2460f:	jae    24e8e <<c30walk::ctor::Ctor>::build::<false>+0x1a7e>
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   24615:	imul   $0xffffffffffffffc4,0x60(%rsp),%rax
next<u16>():
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   2461b:	mov    0x10(%rsp),%rdx
   24620:	lea    (%rax,%rdx,2),%rax
   24624:	add    $0xfffffffffffffffe,%rax
   24628:	cmp    $0x6,%rax
   2462c:	jb     24653 <<c30walk::ctor::Ctor>::build::<false>+0x1243>
   2462e:	mov    %r10,%r9
   24631:	mov    %r11,%r10
   24634:	mov    %rcx,%rdx
   24637:	lea    -0x14f7e(%rip),%r11        # f6c0 <c30walk::ctor::PATTERNS>
   2463e:	sub    %r11,%rdx
   24641:	add    0x8(%rsp),%rdx
   24646:	sub    %r9,%rdx
   24649:	cmp    $0x20,%rdx
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/iter/adapters/enumerate.rs:82
   2464d:	jae    24893 <<c30walk::ctor::Ctor>::build::<false>+0x1483>
fragment<false>():
   24653:	mov    %rdi,%rax
   24656:	xor    %r14d,%r14d
./examples/c30walk/ctor.rs:435
   24659:	lea    (%rdi,%rsi,2),%rdx
   2465d:	mov    0x8(%rsp),%r9
next<u16>():
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   24662:	lea    (%r9,%r14,2),%r9
   24666:	add    %r9,%rcx
fragment<false>():
   24669:	nopl   0x0(%rax)
./examples/c30walk/ctor.rs:435
   24670:	movzwl (%rax),%r9d
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/non_null.rs:656
   24674:	add    $0x2,%rax
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   24678:	mov    %r9w,(%rcx)
/rustc/2d8144b78/library/core/src/ptr/non_null.rs:1714
   2467c:	add    $0x2,%rcx
eq<u16>():
/rustc/2d8144b78/library/core/src/ptr/non_null.rs:1714
   24680:	cmp    %rdx,%rax
next<u16>():
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   24683:	jne    24670 <<c30walk::ctor::Ctor>::build::<false>+0x1260>
fragment<false>():
   24685:	mov    0x18(%rsp),%rdx
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:350
   2468a:	add    %rsi,%rdx
advance<false>():
./examples/c30walk/ctor.rs:351
   2468d:	lea    0x2(%rsi),%eax
   24690:	mov    %rdx,%rcx
   24693:	sub    $0x1e,%rcx
   24697:	cmovb  %rdx,%rcx
   2469b:	cmovb  %esi,%eax
   2469e:	add    %r8d,%eax
   246a1:	mov    %rcx,0x908(%rsp)
   246a9:	mov    %eax,0x910(%rsp)
   246b0:	mov    0x1150(%rsp),%r11
   246b8:	mov    0x20(%rsp),%r9
   246bd:	mov    0x28(%rsp),%r10
   246c2:	mov    0x38(%rsp),%ebp
   246c6:	mov    0x78(%rsp),%rax
run<false>():
./examples/c30walk/ctor.rs:581
   246cb:	jmp    23b1f <<c30walk::ctor::Ctor>::build::<false>+0x70f>
spec_next<usize>():
/rustc/2d8144b78/library/core/src/iter/range.rs:900
   246d0:	mov    %rcx,%r9
   246d3:	and    $0xfffffffffffffffe,%r9
   246d7:	lea    (%rdx,%rsi,1),%r10
   246db:	add    $0x7c,%r10
   246df:	xor    %r8d,%r8d
<c30walk::ctor::Ctor>::build::<false>:
   246e2:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
write<c30walk::Rec>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   246f0:	movdqu 0x2c(%rdi),%xmm0
   246f5:	movdqu %xmm0,-0x50(%r10)
   246fb:	movdqa (%rdi),%xmm1
   246ff:	movdqa 0x10(%rdi),%xmm2
   24704:	movdqa 0x20(%rdi),%xmm3
   24709:	movdqa %xmm3,-0x5c(%r10)
   2470f:	movdqa %xmm2,-0x6c(%r10)
   24715:	movdqa %xmm1,-0x7c(%r10)
   2471b:	mov    %eax,-0x40(%r10)
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:971
   2471f:	add    $0x2,%r8
write<c30walk::Rec>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   24723:	movdqa %xmm1,-0x3c(%r10)
   24729:	movdqa %xmm2,-0x2c(%r10)
   2472f:	movdqa %xmm3,-0x1c(%r10)
   24735:	movdqu %xmm0,-0x10(%r10)
   2473b:	lea    0x2000(%rax),%r11d
   24742:	mov    %r11d,(%r10)
complete<false>():
./examples/c30walk/ctor.rs:490
   24745:	add    $0x4000,%eax
spec_next<usize>():
/rustc/2d8144b78/library/core/src/iter/range.rs:900
   2474a:	sub    $0xffffffffffffff80,%r10
   2474e:	cmp    %r8,%r9
   24751:	jne    246f0 <<c30walk::ctor::Ctor>::build::<false>+0x12e0>
   24753:	testb  $0x1,0x18(%rsp)
   24758:	mov    0x1150(%rsp),%r11
   24760:	mov    0x20(%rsp),%r9
   24765:	mov    0x28(%rsp),%r10
   2476a:	je     247a4 <<c30walk::ctor::Ctor>::build::<false>+0x1394>
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   2476c:	add    %rsi,%rdx
   2476f:	shl    $0x6,%r8
write<c30walk::Rec>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   24773:	movups 0x2c(%rdi),%xmm0
   24777:	movups %xmm0,0x2c(%rdx,%r8,1)
   2477d:	movdqa (%rdi),%xmm0
   24781:	movdqa 0x10(%rdi),%xmm1
   24786:	movdqa 0x20(%rdi),%xmm2
   2478b:	movdqa %xmm2,0x20(%rdx,%r8,1)
   24792:	movdqa %xmm1,0x10(%rdx,%r8,1)
   24799:	movdqa %xmm0,(%rdx,%r8,1)
   2479f:	mov    %eax,0x3c(%rdx,%r8,1)
complete<false>():
   247a4:	mov    0x18(%rsp),%rax
./examples/c30walk/ctor.rs:492
   247a9:	mov    %eax,%ecx
   247ab:	shl    $0x5,%ecx
   247ae:	mov    0x30(%rsp),%edx
   247b2:	add    %ecx,%edx
   247b4:	mov    %edx,0x910(%rsp)
   247bb:	mov    0x8(%rsp),%rdi
run<false>():
./examples/c30walk/ctor.rs:578
   247c0:	add    %rax,%rdi
./examples/c30walk/ctor.rs:580
   247c3:	imul   $0xffffffffffffffe2,0x60(%rsp),%rdx
./examples/c30walk/ctor.rs:581
   247c9:	mov    %rdx,%rsi
   247cc:	add    0x10(%rsp),%rsi
   247d1:	jne    24427 <<c30walk::ctor::Ctor>::build::<false>+0x1017>
   247d7:	mov    0x78(%rsp),%rax
   247dc:	jmp    23b1f <<c30walk::ctor::Ctor>::build::<false>+0x70f>
   247e1:	mov    %r10,%r14
   247e4:	mov    %esi,%r10d
   247e7:	and    $0x18,%r10d
   247eb:	movd   %ebp,%xmm0
   247ef:	pshufd $0x0,%xmm0,%xmm0
   247f4:	movd   %r8d,%xmm1
   247f9:	pshufd $0x0,%xmm1,%xmm1
next<core::slice::iter::IterMut<u32>, core::slice::iter::Iter<u16>>():
/rustc/2d8144b78/library/core/src/iter/adapters/zip.rs:308
   247fe:	add    %r15,%r14
   24801:	mov    %r14,%rax
   24804:	movabs $0x8888888888888889,%rdx
   2480e:	mul    %rdx
   24811:	shr    $0x4,%edx
   24814:	lea    (%r14,%rdx,2),%eax
   24818:	and    $0x18,%eax
   2481b:	lea    -0x15162(%rip),%rdx        # f6c0 <c30walk::ctor::PATTERNS>
   24822:	mov    0x10(%rsp),%r9
   24827:	add    %r9,%rdx
   2482a:	add    $0x8,%rdx
   2482e:	shl    $0x7,%ecx
   24831:	add    %rdi,%rcx
   24834:	add    $0x10,%rcx
   24838:	xor    %r14d,%r14d
fragment<false>():
   2483b:	nopl   0x0(%rax,%rax,1)
./examples/c30walk/ctor.rs:465
   24840:	movq   -0x8(%rdx,%r14,2),%xmm2
   24847:	movq   (%rdx,%r14,2),%xmm3
./examples/c30walk/ctor.rs:466
   2484d:	punpcklwd %xmm4,%xmm2
   24851:	punpcklwd %xmm4,%xmm3
   24855:	paddd  %xmm1,%xmm2
   24859:	paddd  %xmm1,%xmm3
   2485d:	pslld  $0x8,%xmm2
   24862:	pslld  $0x8,%xmm3
   24867:	por    %xmm0,%xmm2
   2486b:	por    %xmm0,%xmm3
   2486f:	movdqu %xmm2,-0x10(%rcx,%r14,4)
   24876:	movdqu %xmm3,(%rcx,%r14,4)
next<core::slice::iter::IterMut<u32>, core::slice::iter::Iter<u16>>():
/rustc/2d8144b78/library/core/src/iter/adapters/zip.rs:308
   2487c:	add    $0x8,%r14
/rustc/2d8144b78/library/core/src/iter/adapters/zip.rs:304
   24880:	cmp    %r14,%rax
   24883:	jne    24840 <<c30walk::ctor::Ctor>::build::<false>+0x1430>
   24885:	cmp    %r10,%rsi
   24888:	je     24685 <<c30walk::ctor::Ctor>::build::<false>+0x1275>
   2488e:	jmp    2455a <<c30walk::ctor::Ctor>::build::<false>+0x114a>
fragment<false>():
   24893:	mov    %rax,%r11
   24896:	shr    $1,%r11
   24899:	inc    %r11
next<u16>():
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   2489c:	cmp    $0x1e,%rax
   248a0:	jae    248aa <<c30walk::ctor::Ctor>::build::<false>+0x149a>
   248a2:	xor    %r14d,%r14d
   248a5:	jmp    24939 <<c30walk::ctor::Ctor>::build::<false>+0x1529>
   248aa:	mov    %r11,0x10(%rsp)
   248af:	mov    %r11,%r14
   248b2:	and    $0xfffffffffffffff0,%r14
   248b6:	lea    -0x151fd(%rip),%rax        # f6c0 <c30walk::ctor::PATTERNS>
   248bd:	lea    (%r9,%rax,1),%rbp
   248c1:	add    $0x10,%rbp
   248c5:	lea    (%r10,%r15,1),%rax
   248c9:	lea    (%rax,%rax,1),%r11
   248cd:	movabs $0x8888888888888889,%rdx
   248d7:	mul    %rdx
   248da:	shr    $0x4,%rdx
   248de:	imul   $0x3c,%rdx,%rax
   248e2:	sub    %rax,%r11
   248e5:	add    $0xfffffffffffffffe,%r11
   248e9:	shr    $1,%r11
   248ec:	inc    %r11
   248ef:	and    $0xfffffffffffffff0,%r11
   248f3:	mov    0x8(%rsp),%rax
   248f8:	add    %rcx,%rax
   248fb:	add    $0x10,%rax
   248ff:	xor    %edx,%edx
fragment<false>():
./examples/c30walk/ctor.rs:435
   24901:	movdqa -0x10(%rbp,%rdx,2),%xmm0
   24907:	movdqa 0x0(%rbp,%rdx,2),%xmm1
write<u16>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   2490d:	movdqu %xmm0,-0x10(%rax,%rdx,2)
   24913:	movdqu %xmm1,(%rax,%rdx,2)
next<core::slice::iter::Iter<u16>>():
/rustc/2d8144b78/library/core/src/iter/adapters/enumerate.rs:82
   24918:	add    $0x10,%rdx
next<u16>():
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   2491c:	cmp    %rdx,%r11
   2491f:	jne    24901 <<c30walk::ctor::Ctor>::build::<false>+0x14f1>
   24921:	mov    0x10(%rsp),%r11
   24926:	cmp    %r14,%r11
   24929:	je     24685 <<c30walk::ctor::Ctor>::build::<false>+0x1275>
fragment<false>():
   2492f:	test   $0xc,%r11b
   24933:	je     249c4 <<c30walk::ctor::Ctor>::build::<false>+0x15b4>
   24939:	movabs $0x8888888888888889,%rdx
   24943:	mov    %r10,%rax
   24946:	lea    (%r9,%r14,2),%r9
   2494a:	add    %r15,%rax
   2494d:	lea    (%rax,%rax,1),%r15
   24951:	mul    %rdx
   24954:	shr    $0x4,%rdx
   24958:	imul   $0x3c,%rdx,%rax
   2495c:	sub    %rax,%r15
   2495f:	add    $0xfffffffffffffffe,%r15
   24963:	shr    $1,%r15
   24966:	inc    %r15
   24969:	and    $0xfffffffffffffffc,%r15
   2496d:	sub    %r14,%r15
   24970:	mov    0x8(%rsp),%rax
   24975:	lea    (%rax,%r14,2),%rdx
   24979:	mov    %r11,%r10
   2497c:	mov    %r11,%r14
   2497f:	and    $0xfffffffffffffffc,%r14
   24983:	lea    (%rdi,%r14,2),%rax
   24987:	lea    -0x152ce(%rip),%r11        # f6c0 <c30walk::ctor::PATTERNS>
   2498e:	add    %r11,%r9
   24991:	add    %rcx,%rdx
   24994:	xor    %r11d,%r11d
./examples/c30walk/ctor.rs:435
   24997:	mov    (%r9,%r11,2),%rbp
write<u16>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   2499b:	mov    %rbp,(%rdx,%r11,2)
next<u16>():
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   2499f:	add    $0x4,%r11
   249a3:	cmp    %r11,%r15
   249a6:	jne    24997 <<c30walk::ctor::Ctor>::build::<false>+0x1587>
   249a8:	cmp    %r14,%r10
   249ab:	jne    24659 <<c30walk::ctor::Ctor>::build::<false>+0x1249>
   249b1:	jmp    24685 <<c30walk::ctor::Ctor>::build::<false>+0x1275>
run<false>():
   249b6:	mov    0x68(%rsp),%rsi
   249bb:	lea    (%rsi,%rax,2),%r10
   249bf:	jmp    2431f <<c30walk::ctor::Ctor>::build::<false>+0xf0f>
   249c4:	lea    (%rdi,%r14,2),%rax
   249c8:	jmp    24659 <<c30walk::ctor::Ctor>::build::<false>+0x1249>
fragment<false>():
./examples/c30walk/ctor.rs:452
   249cd:	lea    0xd8(%rsp),%rdi
   249d5:	mov    0x8(%rsp),%rsi
   249da:	mov    %r10,%rdx
   249dd:	mov    %eax,%ecx
   249df:	call   4dcb0 <<c30walk::ctor::W<false>>::promote>
   249e4:	mov    0x30(%rsp),%r8
   249e9:	mov    0x40(%rsp),%r10
   249ee:	pxor   %xmm4,%xmm4
   249f2:	mov    0x1150(%rsp),%r11
./examples/c30walk/ctor.rs:442
   249fa:	jmp    23f87 <<c30walk::ctor::Ctor>::build::<false>+0xb77>
scatter<false>():
   249ff:	mov    0xf8(%rsp),%r14
   24a07:	mov    0xe8(%rsp),%r8
   24a0f:	mov    0xd8(%rsp),%rdx
   24a17:	mov    0xe0(%rsp),%rax
   24a1f:	mov    %rax,0x18(%rsp)
   24a24:	mov    0x910(%rsp),%ebx
   24a2b:	mov    0x908(%rsp),%r15
   24a33:	mov    %rdx,0x10(%rsp)
   24a38:	jmp    24a95 <<c30walk::ctor::Ctor>::build::<false>+0x1685>
   24a3a:	nopw   0x0(%rax,%rax,1)
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:622
   24a40:	and    $0xfffff,%eax
one<false>():
./examples/c30walk/ctor.rs:635
   24a45:	mov    %eax,%edi
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:1903
   24a47:	mov    0x10(%rdx),%rsi
/rustc/2d8144b78/library/core/src/slice/index.rs:278
   24a4b:	cmp    %rdi,%rsi
index_mut<c30walk::RawG>():
/rustc/2d8144b78/library/core/src/slice/index.rs:278
   24a4e:	jbe    24bb1 <<c30walk::ctor::Ctor>::build::<false>+0x17a1>
   24a54:	shl    $0x7,%rdi
   24a58:	add    0x8(%rdx),%rdi
one<false>():
./examples/c30walk/ctor.rs:635
   24a5c:	mov    %ebp,(%rdi,%r13,4)
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:350
   24a60:	lea    0x1(%r15),%rax
advance<false>():
./examples/c30walk/ctor.rs:351
   24a64:	add    $0xffffffffffffffe3,%r15
   24a68:	xor    %ecx,%ecx
   24a6a:	cmp    $0x1e,%rax
   24a6e:	setae  %cl
   24a71:	cmovb  %rax,%r15
   24a75:	lea    (%rbx,%rcx,2),%ebx
   24a78:	inc    %ebx
   24a7a:	mov    %r15,0x908(%rsp)
   24a82:	mov    %ebx,0x910(%rsp)
eq<u8>():
/rustc/2d8144b78/library/core/src/ptr/non_null.rs:1714
   24a89:	inc    %r12
   24a8c:	cmp    %r12,%r9
next<u8>():
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   24a8f:	je     23ab8 <<c30walk::ctor::Ctor>::build::<false>+0x6a8>
scatter<false>():
./examples/c30walk/ctor.rs:897
   24a95:	movzbl (%r10,%r12,1),%eax
one<false>():
./examples/c30walk/ctor.rs:602
   24a9a:	mov    0x108(%rsp,%rax,4),%esi
./examples/c30walk/ctor.rs:603
   24aa1:	cmp    0x508(%rsp,%rax,4),%esi
   24aa8:	jae    23ae3 <<c30walk::ctor::Ctor>::build::<false>+0x6d3>
./examples/c30walk/ctor.rs:607
   24aae:	mov    %esi,%r13d
   24ab1:	and    $0x1f,%r13d
   24ab5:	xor    %ecx,%ecx
   24ab7:	cmp    $0x1d,%r13d
   24abb:	sete   %cl
   24abe:	lea    (%rsi,%rcx,2),%ecx
   24ac1:	inc    %ecx
   24ac3:	mov    %ecx,0x108(%rsp,%rax,4)
./examples/c30walk/ctor.rs:608
   24aca:	shr    $0x5,%esi
./examples/c30walk/ctor.rs:610
   24acd:	mov    %ebx,%ebp
   24acf:	shl    $0x8,%ebp
   24ad2:	or     %eax,%ebp
./examples/c30walk/ctor.rs:611
   24ad4:	test   %r13d,%r13d
   24ad7:	je     24b20 <<c30walk::ctor::Ctor>::build::<false>+0x1710>
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   24ad9:	mov    %rsi,%rcx
add<c30walk::Rec>():
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   24adc:	shl    $0x6,%rcx
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1733
   24ae0:	mov    0x3c(%r14,%rcx,1),%eax
one<false>():
./examples/c30walk/ctor.rs:621
   24ae5:	test   $0x10000000,%eax
   24aea:	jne    24a40 <<c30walk::ctor::Ctor>::build::<false>+0x1630>
./examples/c30walk/ctor.rs:625
   24af0:	mov    %eax,%edi
   24af2:	shr    $0x8,%edi
   24af5:	mov    %ebx,%edx
   24af7:	sub    %edi,%edx
./examples/c30walk/ctor.rs:626
   24af9:	cmp    $0xfffe,%edx
   24aff:	jae    24b6f <<c30walk::ctor::Ctor>::build::<false>+0x175f>
   24b01:	add    %r14,%rcx
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   24b04:	mov    %dx,(%rcx,%r13,2)
write<u16>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   24b09:	mov    0x10(%rsp),%rdx
one<false>():
./examples/c30walk/ctor.rs:611
   24b0e:	jmp    24a60 <<c30walk::ctor::Ctor>::build::<false>+0x1650>
   24b13:	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:323
   24b20:	mov    %esi,%edi
is_raw<false>():
./examples/c30walk/ctor.rs:323
   24b22:	shr    $0x6,%edi
   24b25:	cmp    %r8,%rdi
   24b28:	jae    24ee8 <<c30walk::ctor::Ctor>::build::<false>+0x1ad8>
   24b2e:	mov    0x18(%rsp),%rax
   24b33:	mov    (%rax,%rdi,8),%rcx
one<false>():
   24b37:	mov    %rsi,%rax
   24b3a:	shl    $0x6,%rax
   24b3e:	add    %r14,%rax
./examples/c30walk/ctor.rs:612
   24b41:	bt     %rsi,%rcx
   24b45:	jae    24b62 <<c30walk::ctor::Ctor>::build::<false>+0x1752>
   24b47:	mov    0x3c(%rax),%eax
./examples/c30walk/ctor.rs:613
   24b4a:	mov    $0xfffff,%ecx
   24b4f:	and    %ecx,%eax
./examples/c30walk/ctor.rs:635
   24b51:	mov    %eax,%edi
as_mut_slice<c30walk::RawG, alloc::alloc::Global>():
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:1903
   24b53:	mov    0x10(%rdx),%rsi
index_mut<c30walk::RawG>():
/rustc/2d8144b78/library/core/src/slice/index.rs:278
   24b57:	cmp    %rdi,%rsi
   24b5a:	ja     24a54 <<c30walk::ctor::Ctor>::build::<false>+0x1644>
   24b60:	jmp    24bb1 <<c30walk::ctor::Ctor>::build::<false>+0x17a1>
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   24b62:	mov    %ebp,0x3c(%rax)
   24b65:	movw   $0x0,(%rax)
write<u16>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   24b6a:	jmp    24a60 <<c30walk::ctor::Ctor>::build::<false>+0x1650>
one<false>():
./examples/c30walk/ctor.rs:627
   24b6f:	lea    0xd8(%rsp),%rdi
   24b77:	mov    %r13,%rdx
   24b7a:	mov    %eax,%ecx
   24b7c:	mov    %r8,0x8(%rsp)
   24b81:	call   4dcb0 <<c30walk::ctor::W<false>>::promote>
   24b86:	mov    0x8(%rsp),%r8
   24b8b:	mov    0x28(%rsp),%r10
   24b90:	mov    0x20(%rsp),%r9
   24b95:	mov    0x1150(%rsp),%r11
   24b9d:	mov    0x10(%rsp),%rdx
./examples/c30walk/ctor.rs:635
   24ba2:	mov    %eax,%edi
as_mut_slice<c30walk::RawG, alloc::alloc::Global>():
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:1903
   24ba4:	mov    0x10(%rdx),%rsi
index_mut<c30walk::RawG>():
/rustc/2d8144b78/library/core/src/slice/index.rs:278
   24ba8:	cmp    %rdi,%rsi
   24bab:	ja     24a54 <<c30walk::ctor::Ctor>::build::<false>+0x1644>
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/slice/index.rs:278
   24bb1:	lea    0x92090(%rip),%rdx        # b6c48 <__frame_dummy_init_array_entry+0x12f8>
index_mut<c30walk::RawG>():
/rustc/2d8144b78/library/core/src/slice/index.rs:278
   24bb8:	call   6292b <core::panicking::panic_bounds_check>
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:2472
   24bbd:	xor    %ecx,%ecx
saturating_sub():
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:2472
   24bbf:	mov    0x48(%rsp),%rsi
   24bc4:	test   %rsi,%rsi
   24bc7:	setne  %al
   24bca:	mov    0x20(%rsp),%rdx
   24bcf:	mov    0x90(%rsp),%rbx
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:785
   24bd7:	je     24c28 <<c30walk::ctor::Ctor>::build::<false>+0x1818>
   24bd9:	mov    %al,%cl
   24bdb:	mov    %rsi,%rax
   24bde:	sub    %rcx,%rax
   24be1:	imul   $0xffffffffffffffe2,%rax,%rcx
   24be5:	add    %rdx,%rcx
   24be8:	cmp    $0x1e,%rcx
   24bec:	jae    24c28 <<c30walk::ctor::Ctor>::build::<false>+0x1818>
/rustc/2d8144b78/library/core/src/iter/range.rs:900
   24bee:	lea    (%rbx,%rsi,4),%rcx
spec_next<usize>():
/rustc/2d8144b78/library/core/src/iter/range.rs:900
   24bf2:	lea    (%rcx,%rdx,2),%rdi
   24bf6:	add    $0xfffffffffffffffc,%rdi
   24bfa:	lea    (%rax,%rax,1),%rcx
   24bfe:	shl    $0x5,%rax
   24c02:	sub    %rcx,%rax
   24c05:	sub    %rdx,%rax
   24c08:	lea    0x3c(,%rax,2),%rdx
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   24c10:	mov    $0xff,%esi
write<u16>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   24c15:	call   *0x94d3d(%rip)        # b9958 <memset@GLIBC_2.2.5>
   24c1b:	mov    0x48(%rsp),%rsi
   24c20:	mov    0x1150(%rsp),%r11
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   24c28:	shl    $0x6,%rsi
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   24c2c:	pcmpeqd %xmm0,%xmm0
write<c30walk::Rec>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   24c30:	movdqu %xmm0,0x2c(%rbx,%rsi,1)
   24c36:	movdqa %xmm0,0x20(%rbx,%rsi,1)
   24c3c:	movdqa %xmm0,0x10(%rbx,%rsi,1)
   24c42:	movdqa %xmm0,(%rbx,%rsi,1)
   24c47:	movl   $0x100fffff,0x3c(%rbx,%rsi,1)
   24c4f:	mov    0xd0(%rsp),%rax
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:614
   24c57:	mov    0x8(%rax),%rdx
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:1864
   24c5b:	mov    0x10(%rax),%rcx
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:614
   24c5f:	mov    0x8(%r11),%r8
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:1903
   24c63:	mov    0x10(%r11),%r9
as_mut_slice<c30walk::RawG, alloc::alloc::Global>():
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:1903
   24c67:	lea    0xd8(%rsp),%rdi
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:795
   24c6f:	mov    %rbx,%rsi
   24c72:	mov    %r11,%rbx
   24c75:	call   51560 <c30walk::ctor::canonicalise>
   24c7a:	mov    0x50(%rsp),%rax
   24c7f:	mov    0xc8(%rsp),%rcx
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:2221
   24c87:	mov    %rcx,0x10(%rax)
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:614
   24c8b:	mov    0x8(%rbx),%rax
./examples/c30walk/ctor.rs:804
   24c8f:	and    $0x3f,%rax
   24c93:	mov    %rax,0xd8(%rsp)
   24c9b:	jne    24ea1 <<c30walk::ctor::Ctor>::build::<false>+0x1a91>
   24ca1:	mov    $0x1,%al
   24ca3:	jmp    23aed <<c30walk::ctor::Ctor>::build::<false>+0x6dd>
   24ca8:	xor    %r12d,%r12d
   24cab:	mov    0x48(%rsp),%rcx
capacity<alloc::alloc::Global>():
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:619
   24cb0:	mov    (%r8),%rax
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:744
   24cb3:	lea    0x1(%rcx),%rbp
   24cb7:	cmp    %rcx,%rax
   24cba:	ja     23628 <<c30walk::ctor::Ctor>::build::<false>+0x218>
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:634
   24cc0:	test   %rax,%rax
current_memory<alloc::alloc::Global>():
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:634
   24cc3:	je     24cd4 <<c30walk::ctor::Ctor>::build::<false>+0x18c4>
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:745
   24cc5:	mov    0x8(%r8),%rdi
<std::alloc::System as core::alloc::global::GlobalAlloc>::dealloc:
/rustc/2d8144b78/library/std/src/sys/alloc/unix.rs:48
   24cc9:	call   *0x94c99(%rip)        # b9968 <free@GLIBC_2.2.5>
   24ccf:	mov    0x50(%rsp),%r8
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:745
   24cd4:	movq   $0x0,(%r8)
   24cdb:	movq   $0x40,0x8(%r8)
   24ce3:	movq   $0x0,0x10(%r8)
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:728
   24ceb:	mov    $0x40,%edx
try_reserve_exact<alloc::alloc::Global>():
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:728
   24cf0:	mov    %r8,%rdi
   24cf3:	mov    %rbp,%rsi
   24cf6:	call   4ce80 <<alloc::raw_vec::RawVecInner>::grow_exact>
   24cfb:	mov    0x50(%rsp),%r8
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/result.rs:2173
   24d00:	cmp    $0xffffffffffffffff,%rax
branch<(), alloc::collections::TryReserveError>():
/rustc/2d8144b78/library/core/src/result.rs:2173
   24d04:	mov    0x1150(%rsp),%r9
   24d0c:	mov    %r15,%rdi
   24d0f:	je     23628 <<c30walk::ctor::Ctor>::build::<false>+0x218>
   24d15:	jmp    24d6f <<c30walk::ctor::Ctor>::build::<false>+0x195f>
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:634
   24d17:	test   %rax,%rax
current_memory<alloc::alloc::Global>():
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:634
   24d1a:	je     24d2e <<c30walk::ctor::Ctor>::build::<false>+0x191e>
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:751
   24d1c:	mov    0x8(%r9),%rdi
<std::alloc::System as core::alloc::global::GlobalAlloc>::dealloc:
/rustc/2d8144b78/library/std/src/sys/alloc/unix.rs:48
   24d20:	call   *0x94c42(%rip)        # b9968 <free@GLIBC_2.2.5>
   24d26:	mov    0x1150(%rsp),%r9
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:751
   24d2e:	movq   $0x0,(%r9)
   24d35:	movq   $0x40,0x8(%r9)
   24d3d:	movq   $0x0,0x10(%r9)
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:728
   24d45:	mov    $0x80,%edx
try_reserve_exact<alloc::alloc::Global>():
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:728
   24d4a:	mov    %r9,%rdi
   24d4d:	mov    %r12,%rsi
   24d50:	call   4ce80 <<alloc::raw_vec::RawVecInner>::grow_exact>
   24d55:	mov    0x1150(%rsp),%r9
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/result.rs:2173
   24d5d:	cmp    $0xffffffffffffffff,%rax
branch<(), alloc::collections::TryReserveError>():
/rustc/2d8144b78/library/core/src/result.rs:2173
   24d61:	mov    0x50(%rsp),%r8
   24d66:	mov    %r15,%rdi
   24d69:	je     23634 <<c30walk::ctor::Ctor>::build::<false>+0x224>
<c30walk::ctor::Ctor>::build::<false>:
   24d6f:	mov    %rax,%rdi
   24d72:	mov    %rdx,%rsi
   24d75:	call   6108b <alloc::raw_vec::handle_error>
/rustc/2d8144b78/library/core/src/slice/index.rs:568
   24d7a:	cmp    %r9,%r12
index<u8>():
/rustc/2d8144b78/library/core/src/slice/index.rs:568
   24d7d:	jbe    23aaf <<c30walk::ctor::Ctor>::build::<false>+0x69f>
/rustc/2d8144b78/library/core/src/slice/index.rs:569
   24d83:	lea    0x90d96(%rip),%rcx        # b5b20 <__frame_dummy_init_array_entry+0x1d0>
   24d8a:	mov    %r12,%rdi
   24d8d:	mov    %r9,%rsi
   24d90:	mov    %r9,%rdx
   24d93:	call   63690 <core::slice::index::slice_index_fail>
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/slice/index.rs:443
   24d98:	lea    0x92449(%rip),%rcx        # b71e8 <__frame_dummy_init_array_entry+0x1898>
index<u8>():
/rustc/2d8144b78/library/core/src/slice/index.rs:443
   24d9f:	mov    %r12,%rdi
   24da2:	mov    %r9,%rdx
   24da5:	call   63690 <core::slice::index::slice_index_fail>
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/slice/index.rs:443
   24daa:	lea    0x92437(%rip),%rcx        # b71e8 <__frame_dummy_init_array_entry+0x1898>
index<u8>():
/rustc/2d8144b78/library/core/src/slice/index.rs:443
   24db1:	mov    %rdx,%rdi
   24db4:	mov    %r9,%rdx
   24db7:	call   63690 <core::slice::index::slice_index_fail>
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/slice/index.rs:443
   24dbc:	lea    0x92425(%rip),%rcx        # b71e8 <__frame_dummy_init_array_entry+0x1898>
index<u8>():
/rustc/2d8144b78/library/core/src/slice/index.rs:443
   24dc3:	mov    %rsi,%rdi
   24dc6:	mov    %rax,%rsi
   24dc9:	mov    %r9,%rdx
   24dcc:	call   63690 <core::slice::index::slice_index_fail>
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:759
   24dd1:	lea    -0x1b418(%rip),%rsi        # 99c0 <GCC_except_table733+0x974>
   24dd8:	lea    -0x1611c(%rip),%rdx        # ecc3 <GCC_except_table733+0x5c77>
   24ddf:	lea    0x90d22(%rip),%r8        # b5b08 <__frame_dummy_init_array_entry+0x1b8>
   24de6:	lea    0xd8(%rsp),%rdi
   24dee:	mov    $0x2f,%ecx
   24df3:	call   62cd2 <core::panicking::assert_failed::<usize, usize>>
/rustc/2d8144b78/library/core/src/slice/index.rs:443
   24df8:	lea    0x90d39(%rip),%rcx        # b5b38 <__frame_dummy_init_array_entry+0x1e8>
index<u8>():
/rustc/2d8144b78/library/core/src/slice/index.rs:443
   24dff:	mov    %rdx,%rdi
   24e02:	mov    %r14,%rsi
   24e05:	mov    %r9,%rdx
   24e08:	call   63690 <core::slice::index::slice_index_fail>
reserve<alloc::alloc::Global>():
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:675
   24e0d:	mov    $0x8,%ecx
   24e12:	mov    $0x8,%r8d
   24e18:	mov    %r15,%rdi
   24e1b:	xor    %esi,%esi
   24e1d:	mov    %r14,%rdx
   24e20:	call   2f490 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   24e25:	mov    %r15,%rdi
   24e28:	mov    0x1150(%rsp),%r9
   24e30:	mov    0x50(%rsp),%r8
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:3054
   24e35:	mov    0x10(%r15),%r13
len<u64, alloc::alloc::Global>():
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:3054
   24e39:	jmp    23657 <<c30walk::ctor::Ctor>::build::<false>+0x247>
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/slice/index.rs:456
   24e3e:	lea    0x91edb(%rip),%rcx        # b6d20 <__frame_dummy_init_array_entry+0x13d0>
index_mut<u32>():
/rustc/2d8144b78/library/core/src/slice/index.rs:456
   24e45:	mov    $0x20,%edx
   24e4a:	mov    %r10,%rdi
   24e4d:	call   63690 <core::slice::index::slice_index_fail>
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/slice/index.rs:443
   24e52:	lea    0x91eaf(%rip),%rcx        # b6d08 <__frame_dummy_init_array_entry+0x13b8>
index<u16>():
/rustc/2d8144b78/library/core/src/slice/index.rs:443
   24e59:	mov    $0x1e,%edx
   24e5e:	xor    %edi,%edi
   24e60:	mov    %r8,%rsi
   24e63:	call   63690 <core::slice::index::slice_index_fail>
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/slice/index.rs:456
   24e68:	lea    0x91eb1(%rip),%rcx        # b6d20 <__frame_dummy_init_array_entry+0x13d0>
index_mut<u32>():
/rustc/2d8144b78/library/core/src/slice/index.rs:456
   24e6f:	mov    $0x20,%edx
   24e74:	xor    %edi,%edi
   24e76:	call   63690 <core::slice::index::slice_index_fail>
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/slice/index.rs:443
   24e7b:	lea    0x91e86(%rip),%rcx        # b6d08 <__frame_dummy_init_array_entry+0x13b8>
   24e82:	mov    $0x1e,%edx
index<u16>():
/rustc/2d8144b78/library/core/src/slice/index.rs:443
   24e87:	xor    %edi,%edi
   24e89:	call   63690 <core::slice::index::slice_index_fail>
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/slice/index.rs:443
   24e8e:	lea    0x91e2b(%rip),%rcx        # b6cc0 <__frame_dummy_init_array_entry+0x1370>
index<u16>():
/rustc/2d8144b78/library/core/src/slice/index.rs:443
   24e95:	mov    $0x1e,%edx
   24e9a:	xor    %edi,%edi
   24e9c:	call   63690 <core::slice::index::slice_index_fail>
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:804
   24ea1:	lea    -0x1b4e8(%rip),%rsi        # 99c0 <GCC_except_table733+0x974>
   24ea8:	lea    -0x16206(%rip),%rdx        # eca9 <GCC_except_table733+0x5c5d>
   24eaf:	lea    0x90c3a(%rip),%r8        # b5af0 <__frame_dummy_init_array_entry+0x1a0>
   24eb6:	lea    0xd8(%rsp),%rdi
   24ebe:	mov    $0x35,%ecx
   24ec3:	call   62cd2 <core::panicking::assert_failed::<usize, usize>>
create_raw<false>():
./examples/c30walk/ctor.rs:373
   24ec8:	lea    0x91d49(%rip),%rdx        # b6c18 <__frame_dummy_init_array_entry+0x12c8>
   24ecf:	mov    0x18(%rsp),%rsi
   24ed4:	call   6292b <core::panicking::panic_bounds_check>
is_raw<false>():
./examples/c30walk/ctor.rs:323
   24ed9:	lea    0x90c70(%rip),%rdx        # b5b50 <__frame_dummy_init_array_entry+0x200>
   24ee0:	mov    %r13,%rdi
   24ee3:	call   6292b <core::panicking::panic_bounds_check>
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:323
   24ee8:	lea    0x90c61(%rip),%rdx        # b5b50 <__frame_dummy_init_array_entry+0x200>
is_raw<false>():
./examples/c30walk/ctor.rs:323
   24eef:	mov    %r8,%rsi
   24ef2:	call   6292b <core::panicking::panic_bounds_check>
fragment<false>():
./examples/c30walk/ctor.rs:426
   24ef7:	lea    0x91daa(%rip),%rdx        # b6ca8 <__frame_dummy_init_array_entry+0x1358>
   24efe:	mov    %rsi,%rdi
   24f01:	mov    $0x1e,%esi
   24f06:	call   6292b <core::panicking::panic_bounds_check>
   24f0b:	lea    0x91d96(%rip),%rdx        # b6ca8 <__frame_dummy_init_array_entry+0x1358>
   24f12:	mov    $0x1e,%esi
   24f17:	call   6292b <core::panicking::panic_bounds_check>
is_raw<false>():
./examples/c30walk/ctor.rs:323
   24f1c:	lea    0x90c2d(%rip),%rdx        # b5b50 <__frame_dummy_init_array_entry+0x200>
   24f23:	mov    %rax,%rdi
   24f26:	mov    %rcx,%rsi
   24f29:	call   6292b <core::panicking::panic_bounds_check>
fragment<false>():
./examples/c30walk/ctor.rs:448
   24f2e:	lea    0x91da3(%rip),%rdx        # b6cd8 <__frame_dummy_init_array_entry+0x1388>
   24f35:	mov    $0x1e,%esi
   24f3a:	call   6292b <core::panicking::panic_bounds_check>
index_mut<c30walk::RawG>():
/rustc/2d8144b78/library/core/src/slice/index.rs:278
   24f3f:	lea    0x91daa(%rip),%rdx        # b6cf0 <__frame_dummy_init_array_entry+0x13a0>
   24f46:	call   6292b <core::panicking::panic_bounds_check>
   24f4b:	lea    0x91d9e(%rip),%rdx        # b6cf0 <__frame_dummy_init_array_entry+0x13a0>
   24f52:	mov    %rcx,%rdi
   24f55:	mov    %rax,%rsi
   24f58:	call   6292b <core::panicking::panic_bounds_check>
complete<false>():
./examples/c30walk/ctor.rs:485
   24f5d:	lea    0x91d2c(%rip),%rdx        # b6c90 <__frame_dummy_init_array_entry+0x1340>
   24f64:	mov    $0x1e,%esi
   24f69:	call   6292b <core::panicking::panic_bounds_check>
