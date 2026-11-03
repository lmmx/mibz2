
target/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

0000000000033100 <<c30walk::ctor2::Ctor2>::build::<false>>:
<c30walk::ctor2::Ctor2>::build::<false>:
./examples/c30walk/ctor2.rs:690
   33100:	push   %rbp
   33101:	push   %r15
   33103:	push   %r14
   33105:	push   %r13
   33107:	push   %r12
   33109:	push   %rbx
   3310a:	sub    $0x1000,%rsp
   33111:	movq   $0x0,(%rsp)
   33119:	sub    $0x1000,%rsp
   33120:	movq   $0x0,(%rsp)
   33128:	sub    $0x1000,%rsp
   3312f:	movq   $0x0,(%rsp)
   33137:	sub    $0x1000,%rsp
   3313e:	movq   $0x0,(%rsp)
   33146:	sub    $0x1000,%rsp
   3314d:	movq   $0x0,(%rsp)
   33155:	sub    $0x4e8,%rsp
   3315c:	mov    %r8d,0x1c(%rsp)
   33161:	mov    0x5520(%rsp),%rax
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:3035
   33169:	movq   $0x0,0x10(%r9)
   33171:	movq   $0x0,0x10(%rax)
clear<c30walk::RawG, alloc::alloc::Global>():
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:3035
   33179:	cmp    $0xeffe2,%rdx
<c30walk::ctor2::Ctor2>::build::<false>:
./examples/c30walk/ctor2.rs:701
   33180:	ja     34677 <<c30walk::ctor2::Ctor2>::build::<false>+0x1577>
   33186:	mov    %rcx,%r15
   33189:	mov    %rdx,%r13
   3318c:	pxor   %xmm0,%xmm0
   33190:	xor    %eax,%eax
   33192:	pxor   %xmm2,%xmm2
   33196:	pxor   %xmm1,%xmm1
   3319a:	nopw   0x0(%rax,%rax,1)
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:279
   331a0:	movq   (%r15,%rax,4),%xmm3
fold<u32, usize, core::iter::adapters::map::map_fold::{closure_env#0}<&u32, usize, usize, c30walk::ctor2::{impl#3}::build::{closure_env#0}<false>, core::iter::traits::accum::{impl#48}::sum::{closure_env#0}<core::iter::adapters::map::Map<core::slice::iter::Iter<u32>, c30walk::ctor2::{impl#3}::build::{closure_env#0}<false>>>>>():
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:279
   331a6:	movq   0x8(%r15,%rax,4),%xmm4
<c30walk::ctor2::Ctor2>::build::<false>:
./examples/c30walk/ctor2.rs:701
   331ad:	punpckldq %xmm0,%xmm3
/rustc/2d8144b78/library/core/src/iter/traits/accum.rs:55
   331b1:	paddq  %xmm2,%xmm3
{closure#0}<false>():
./examples/c30walk/ctor2.rs:701
   331b5:	punpckldq %xmm0,%xmm4
{closure#0}<core::iter::adapters::map::Map<core::slice::iter::Iter<u32>, c30walk::ctor2::{impl#3}::build::{closure_env#0}<false>>>():
/rustc/2d8144b78/library/core/src/iter/traits/accum.rs:55
   331b9:	paddq  %xmm1,%xmm4
fold<u32, usize, core::iter::adapters::map::map_fold::{closure_env#0}<&u32, usize, usize, c30walk::ctor2::{impl#3}::build::{closure_env#0}<false>, core::iter::traits::accum::{impl#48}::sum::{closure_env#0}<core::iter::adapters::map::Map<core::slice::iter::Iter<u32>, c30walk::ctor2::{impl#3}::build::{closure_env#0}<false>>>>>():
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:279
   331bd:	movq   0x10(%r15,%rax,4),%xmm2
   331c4:	movq   0x18(%r15,%rax,4),%xmm1
{closure#0}<false>():
./examples/c30walk/ctor2.rs:701
   331cb:	punpckldq %xmm0,%xmm2
{closure#0}<core::iter::adapters::map::Map<core::slice::iter::Iter<u32>, c30walk::ctor2::{impl#3}::build::{closure_env#0}<false>>>():
/rustc/2d8144b78/library/core/src/iter/traits/accum.rs:55
   331cf:	paddq  %xmm3,%xmm2
{closure#0}<false>():
./examples/c30walk/ctor2.rs:701
   331d3:	punpckldq %xmm0,%xmm1
{closure#0}<core::iter::adapters::map::Map<core::slice::iter::Iter<u32>, c30walk::ctor2::{impl#3}::build::{closure_env#0}<false>>>():
/rustc/2d8144b78/library/core/src/iter/traits/accum.rs:55
   331d7:	paddq  %xmm4,%xmm1
fold<u32, usize, core::iter::adapters::map::map_fold::{closure_env#0}<&u32, usize, usize, c30walk::ctor2::{impl#3}::build::{closure_env#0}<false>, core::iter::traits::accum::{impl#48}::sum::{closure_env#0}<core::iter::adapters::map::Map<core::slice::iter::Iter<u32>, c30walk::ctor2::{impl#3}::build::{closure_env#0}<false>>>>>():
   331db:	add    $0x8,%rax
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:284
   331df:	cmp    $0x100,%rax
   331e5:	jne    331a0 <<c30walk::ctor2::Ctor2>::build::<false>+0xa0>
   331e7:	paddq  %xmm2,%xmm1
   331eb:	pshufd $0xee,%xmm1,%xmm0
   331f0:	paddq  %xmm1,%xmm0
   331f4:	movq   %xmm0,%rax
<c30walk::ctor2::Ctor2>::build::<false>:
./examples/c30walk/ctor2.rs:701
   331f9:	cmp    %r13,%rax
   331fc:	jne    34677 <<c30walk::ctor2::Ctor2>::build::<false>+0x1577>
   33202:	pxor   %xmm2,%xmm2
   33206:	xor    %eax,%eax
   33208:	movdqa -0x26a20(%rip),%xmm3        # c7f0 <GCC_except_table884+0x948>
   33210:	pxor   %xmm1,%xmm1
   33214:	pxor   %xmm0,%xmm0
   33218:	nopl   0x0(%rax,%rax,1)
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:279
   33220:	movq   (%r15,%rax,4),%xmm4
fold<u32, usize, core::iter::adapters::map::map_fold::{closure_env#0}<&u32, usize, usize, core::iter::adapters::filter::{impl#3}::count::to_usize::{closure_env#0}<&u32, c30walk::ctor2::{impl#3}::build::{closure_env#1}<false>>, core::iter::traits::accum::{impl#48}::sum::{closure_env#0}<core::iter::adapters::map::Map<core::slice::iter::Iter<u32>, core::iter::adapters::filter::{impl#3}::count::to_usize::{closure_env#0}<&u32, c30walk::ctor2::{impl#3}::build::{closure_env#1}<false>>>>>>():
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:279
   33226:	movq   0x8(%r15,%rax,4),%xmm5
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/iter/adapters/filter.rs:138
   3322d:	pshufd $0xd4,%xmm4,%xmm4
{closure#0}<&u32, c30walk::ctor2::{impl#3}::build::{closure_env#1}<false>>():
/rustc/2d8144b78/library/core/src/iter/adapters/filter.rs:138
   33232:	pcmpeqd %xmm2,%xmm4
   33236:	pandn  %xmm3,%xmm4
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/iter/traits/accum.rs:55
   3323a:	paddq  %xmm1,%xmm4
{closure#0}<&u32, c30walk::ctor2::{impl#3}::build::{closure_env#1}<false>>():
/rustc/2d8144b78/library/core/src/iter/adapters/filter.rs:138
   3323e:	pshufd $0xd4,%xmm5,%xmm5
   33243:	pcmpeqd %xmm2,%xmm5
   33247:	pandn  %xmm3,%xmm5
{closure#0}<core::iter::adapters::map::Map<core::slice::iter::Iter<u32>, core::iter::adapters::filter::{impl#3}::count::to_usize::{closure_env#0}<&u32, c30walk::ctor2::{impl#3}::build::{closure_env#1}<false>>>>():
/rustc/2d8144b78/library/core/src/iter/traits/accum.rs:55
   3324b:	paddq  %xmm0,%xmm5
fold<u32, usize, core::iter::adapters::map::map_fold::{closure_env#0}<&u32, usize, usize, core::iter::adapters::filter::{impl#3}::count::to_usize::{closure_env#0}<&u32, c30walk::ctor2::{impl#3}::build::{closure_env#1}<false>>, core::iter::traits::accum::{impl#48}::sum::{closure_env#0}<core::iter::adapters::map::Map<core::slice::iter::Iter<u32>, core::iter::adapters::filter::{impl#3}::count::to_usize::{closure_env#0}<&u32, c30walk::ctor2::{impl#3}::build::{closure_env#1}<false>>>>>>():
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:279
   3324f:	movq   0x10(%r15,%rax,4),%xmm0
   33256:	movq   0x18(%r15,%rax,4),%xmm6
{closure#0}<&u32, c30walk::ctor2::{impl#3}::build::{closure_env#1}<false>>():
/rustc/2d8144b78/library/core/src/iter/adapters/filter.rs:138
   3325d:	pshufd $0xd4,%xmm0,%xmm1
   33262:	pcmpeqd %xmm2,%xmm1
   33266:	pandn  %xmm3,%xmm1
{closure#0}<core::iter::adapters::map::Map<core::slice::iter::Iter<u32>, core::iter::adapters::filter::{impl#3}::count::to_usize::{closure_env#0}<&u32, c30walk::ctor2::{impl#3}::build::{closure_env#1}<false>>>>():
/rustc/2d8144b78/library/core/src/iter/traits/accum.rs:55
   3326a:	paddq  %xmm4,%xmm1
{closure#0}<&u32, c30walk::ctor2::{impl#3}::build::{closure_env#1}<false>>():
/rustc/2d8144b78/library/core/src/iter/adapters/filter.rs:138
   3326e:	pshufd $0xd4,%xmm6,%xmm0
   33273:	pcmpeqd %xmm2,%xmm0
   33277:	pandn  %xmm3,%xmm0
{closure#0}<core::iter::adapters::map::Map<core::slice::iter::Iter<u32>, core::iter::adapters::filter::{impl#3}::count::to_usize::{closure_env#0}<&u32, c30walk::ctor2::{impl#3}::build::{closure_env#1}<false>>>>():
/rustc/2d8144b78/library/core/src/iter/traits/accum.rs:55
   3327b:	paddq  %xmm5,%xmm0
fold<u32, usize, core::iter::adapters::map::map_fold::{closure_env#0}<&u32, usize, usize, core::iter::adapters::filter::{impl#3}::count::to_usize::{closure_env#0}<&u32, c30walk::ctor2::{impl#3}::build::{closure_env#1}<false>>, core::iter::traits::accum::{impl#48}::sum::{closure_env#0}<core::iter::adapters::map::Map<core::slice::iter::Iter<u32>, core::iter::adapters::filter::{impl#3}::count::to_usize::{closure_env#0}<&u32, c30walk::ctor2::{impl#3}::build::{closure_env#1}<false>>>>>>():
   3327f:	add    $0x8,%rax
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:284
   33283:	cmp    $0x100,%rax
   33289:	jne    33220 <<c30walk::ctor2::Ctor2>::build::<false>+0x120>
<c30walk::ctor2::Ctor2>::build::<false>:
   3328b:	mov    $0x88888889,%ebp
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:3756
   33290:	imul   %r13,%rbp
div_ceil():
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:3756
   33294:	shr    $0x24,%rbp
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:3758
   33298:	imul   $0xeeeeeeef,%r13d,%eax
   3329f:	ror    $1,%eax
   332a1:	cmp    $0x8888889,%eax
   332a6:	sbb    $0xffffffffffffffff,%rbp
   332aa:	mov    %rbp,%rbx
<c30walk::ctor2::Ctor2>::build::<false>:
./examples/c30walk/ctor2.rs:705
   332ad:	cmpl   $0x0,0x1c(%rsp)
./examples/c30walk/ctor2.rs:707
   332b2:	jne    33308 <<c30walk::ctor2::Ctor2>::build::<false>+0x208>
./examples/c30walk/ctor2.rs:709
   332b4:	test   %r13,%r13
   332b7:	je     33306 <<c30walk::ctor2::Ctor2>::build::<false>+0x206>
   332b9:	paddq  %xmm1,%xmm0
   332bd:	pshufd $0xee,%xmm0,%xmm1
   332c2:	paddq  %xmm0,%xmm1
   332c6:	movq   %xmm1,%rax
./examples/c30walk.rs:105
   332cb:	lea    -0x1(%r13),%ecx
phi():
./examples/c30walk.rs:105
   332cf:	mov    $0x88888889,%edx
   332d4:	imul   %rcx,%rdx
   332d8:	shr    $0x23,%rdx
   332dc:	and    $0xfffffffe,%edx
<c30walk::ctor2::Ctor2>::build::<false>:
./examples/c30walk/ctor2.rs:712
   332df:	lea    (%rdx,%r13,1),%ecx
   332e3:	dec    %ecx
   332e5:	shr    $1,%ecx
   332e7:	mov    $0x80010003,%ebx
   332ec:	imul   %rcx,%rbx
   332f0:	shr    $0x2e,%rbx
   332f4:	inc    %ebx
   332f6:	imul   %rax,%rbx
   332fa:	dec    %rbx
/rustc/2d8144b78/library/core/src/cmp.rs:1077
   332fd:	cmp    %rbp,%rbx
min<usize>():
/rustc/2d8144b78/library/core/src/cmp.rs:1077
   33300:	cmovae %rbp,%rbx
   33304:	jmp    33308 <<c30walk::ctor2::Ctor2>::build::<false>+0x208>
<c30walk::ctor2::Ctor2>::build::<false>:
   33306:	xor    %ebx,%ebx
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:619
   33308:	mov    (%r9),%rax
./examples/c30walk/ctor2.rs:714
   3330b:	lea    0x1(%rbp),%rcx
   3330f:	mov    %rcx,0x80(%rsp)
   33317:	cmp    %rbp,%rax
   3331a:	mov    %rsi,0x28(%rsp)
   3331f:	jbe    347df <<c30walk::ctor2::Ctor2>::build::<false>+0x16df>
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:619
   33325:	mov    0x5520(%rsp),%rax
capacity<alloc::alloc::Global>():
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:619
   3332d:	mov    (%rax),%rax
<c30walk::ctor2::Ctor2>::build::<false>:
./examples/c30walk/ctor2.rs:718
   33330:	cmp    %rbx,%rax
   33333:	jb     34834 <<c30walk::ctor2::Ctor2>::build::<false>+0x1734>
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:3035
   33339:	movq   $0x0,0x10(%rdi)
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:3758
   33341:	lea    0x3f(%rbp),%rbx
div_ceil():
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:3758
   33345:	shr    $0x6,%rbx
   33349:	mov    %r13,0x8(%rsp)
   3334e:	mov    %rdi,0x58(%rsp)
   33353:	mov    %rbp,0x60(%rsp)
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:3525
   33358:	je     333b2 <<c30walk::ctor2::Ctor2>::build::<false>+0x2b2>
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:767
   3335a:	cmp    (%rdi),%rbx
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:673
   3335d:	ja     34963 <<c30walk::ctor2::Ctor2>::build::<false>+0x1863>
   33363:	mov    %r9,%r13
extend_with<u64, alloc::alloc::Global>():
   33366:	xor    %r14d,%r14d
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:614
   33369:	mov    0x8(%rdi),%r12
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   3336d:	lea    (%r12,%r14,8),%rdi
/rustc/2d8144b78/library/core/src/cmp.rs:1916
   33371:	cmp    $0x1,%rbx
/rustc/2d8144b78/library/core/src/iter/range.rs:900
   33375:	je     33399 <<c30walk::ctor2::Ctor2>::build::<false>+0x299>
spec_next<usize>():
/rustc/2d8144b78/library/core/src/iter/range.rs:900
   33377:	lea    -0x8(,%rbx,8),%rdx
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   3337f:	xor    %esi,%esi
write<u64>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   33381:	call   *0xc3909(%rip)        # f6c90 <memset@GLIBC_2.2.5>
spec_next<usize>():
/rustc/2d8144b78/library/core/src/iter/range.rs:900
   33387:	lea    (%r14,%rbx,1),%rax
   3338b:	lea    (%r12,%rax,8),%rdi
   3338f:	add    $0xfffffffffffffff8,%rdi
   33393:	add    %rbx,%r14
   33396:	dec    %r14
extend_with<u64, alloc::alloc::Global>():
   33399:	mov    %r13,%r9
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   3339c:	movq   $0x0,(%rdi)
/rustc/2d8144b78/library/alloc/src/vec/set_len_on_drop.rs:19
   333a3:	inc    %r14
increment_len():
/rustc/2d8144b78/library/alloc/src/vec/set_len_on_drop.rs:19
   333a6:	mov    0x8(%rsp),%r13
   333ab:	mov    0x58(%rsp),%rdi
   333b0:	jmp    333b5 <<c30walk::ctor2::Ctor2>::build::<false>+0x2b5>
<c30walk::ctor2::Ctor2>::build::<false>:
   333b2:	xor    %r14d,%r14d
   333b5:	lea    0x18(%rdi),%r12
resize<u64, alloc::alloc::Global>():
   333b9:	mov    %r14,0x10(%rdi)
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:3035
   333bd:	movq   $0x0,0x28(%rdi)
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:767
   333c5:	cmpq   $0xf,0x18(%rdi)
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:673
   333ca:	jbe    34938 <<c30walk::ctor2::Ctor2>::build::<false>+0x1838>
   333d0:	lea    0x30(%rdi),%rax
   333d4:	mov    %rax,(%rsp)
   333d8:	xor    %eax,%eax
   333da:	pxor   %xmm0,%xmm0
   333de:	movabs $0xffffffff00000000,%rcx
   333e8:	nopl   0x0(%rax,%rax,1)
   333f0:	movdqu %xmm0,0x2cf8(%rsp,%rax,1)
   333f9:	movdqu %xmm0,0x2ce8(%rsp,%rax,1)
   33402:	mov    %rcx,0x2d08(%rsp,%rax,1)
   3340a:	movdqu %xmm0,0x2d10(%rsp,%rax,1)
   33413:	movdqu %xmm0,0x2d20(%rsp,%rax,1)
   3341c:	mov    %rcx,0x2d30(%rsp,%rax,1)
   33424:	movdqu %xmm0,0x2d38(%rsp,%rax,1)
   3342d:	movdqu %xmm0,0x2d48(%rsp,%rax,1)
   33436:	mov    %rcx,0x2d58(%rsp,%rax,1)
   3343e:	movdqu %xmm0,0x2d60(%rsp,%rax,1)
   33447:	movdqu %xmm0,0x2d70(%rsp,%rax,1)
   33450:	mov    %rcx,0x2d80(%rsp,%rax,1)
   33458:	add    $0xa0,%rax
   3345e:	cmp    $0x2800,%rax
   33464:	jne    333f0 <<c30walk::ctor2::Ctor2>::build::<false>+0x2f0>
   33466:	mov    %r12,%r14
/rustc/2d8144b78/library/core/src/iter/adapters/zip.rs:304
   33469:	lea    0xc(%r15),%rax
next<core::slice::iter::IterMut<c30walk::ctor2::Desc>, core::slice::iter::Iter<u32>>():
/rustc/2d8144b78/library/core/src/iter/adapters/zip.rs:304
   3346d:	xor    %edx,%edx
   3346f:	mov    $0x25,%ecx
<c30walk::ctor2::Ctor2>::build::<false>:
   33474:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   33480:	mov    -0xc(%rax),%esi
./examples/c30walk/ctor2.rs:739
   33483:	add    %edx,%esi
./examples/c30walk/ctor2.rs:738
   33485:	mov    %edx,0x2c6c(%rsp,%rcx,4)
./examples/c30walk/ctor2.rs:740
   3348c:	mov    %esi,0x2c70(%rsp,%rcx,4)
   33493:	mov    -0x8(%rax),%edx
./examples/c30walk/ctor2.rs:739
   33496:	add    %esi,%edx
./examples/c30walk/ctor2.rs:738
   33498:	mov    %esi,0x2c94(%rsp,%rcx,4)
./examples/c30walk/ctor2.rs:740
   3349f:	mov    %edx,0x2c98(%rsp,%rcx,4)
   334a6:	mov    -0x4(%rax),%esi
./examples/c30walk/ctor2.rs:739
   334a9:	add    %edx,%esi
./examples/c30walk/ctor2.rs:738
   334ab:	mov    %edx,0x2cbc(%rsp,%rcx,4)
./examples/c30walk/ctor2.rs:740
   334b2:	mov    %esi,0x2cc0(%rsp,%rcx,4)
   334b9:	mov    (%rax),%edx
./examples/c30walk/ctor2.rs:739
   334bb:	add    %esi,%edx
./examples/c30walk/ctor2.rs:738
   334bd:	mov    %esi,0x2ce4(%rsp,%rcx,4)
./examples/c30walk/ctor2.rs:740
   334c4:	mov    %edx,0x2ce8(%rsp,%rcx,4)
next<core::slice::iter::IterMut<c30walk::ctor2::Desc>, core::slice::iter::Iter<u32>>():
/rustc/2d8144b78/library/core/src/iter/adapters/zip.rs:304
   334cb:	add    $0x28,%rcx
   334cf:	add    $0x10,%rax
   334d3:	cmp    $0xa25,%rcx
   334da:	jne    33480 <<c30walk::ctor2::Ctor2>::build::<false>+0x380>
<c30walk::ctor2::Ctor2>::build::<false>:
   334dc:	mov    %r9,0x78(%rsp)
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:614
   334e1:	mov    0x8(%r9),%rbx
   334e5:	mov    0x8(%rdi),%rbp
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:1903
   334e9:	mov    0x10(%rdi),%r12
./examples/c30walk/ctor2.rs:743
   334ed:	lea    0xd8(%rsp),%rdi
   334f5:	lea    0x2ce8(%rsp),%rsi
./examples/c30walk/ctor2.rs:750
   334fd:	mov    $0x2800,%edx
   33502:	call   *0xc37a8(%rip)        # f6cb0 <memcpy@GLIBC_2.14>
   33508:	mov    %rbp,%r8
   3350b:	mov    %r12,%rsi
./examples/c30walk/ctor2.rs:743
   3350e:	mov    %rbx,0xc0(%rsp)
   33516:	mov    0x60(%rsp),%rax
   3351b:	mov    %rax,0xc8(%rsp)
   33523:	mov    %r13,0xd0(%rsp)
   3352b:	mov    0x5520(%rsp),%rax
   33533:	mov    %rax,0x98(%rsp)
   3353b:	mov    %rbp,0xa0(%rsp)
   33543:	mov    %r12,0xa8(%rsp)
   3354b:	mov    %r14,0xb0(%rsp)
   33553:	movl   $0x0,0x28e0(%rsp)
   3355e:	movq   $0x0,0x28d8(%rsp)
   3356a:	mov    (%rsp),%rax
   3356e:	mov    %rax,0xb8(%rsp)
   33576:	mov    $0xffffffffffffffff,%rcx
   3357d:	xor    %ebp,%ebp
   3357f:	movabs $0xeeeeeeeeeeeeeeef,%r9
   33589:	movabs $0x888888888888889,%r10
   33593:	pxor   %xmm0,%xmm0
   33597:	pcmpeqd %xmm1,%xmm1
   3359b:	xor    %r12d,%r12d
   3359e:	jmp    33656 <<c30walk::ctor2::Ctor2>::build::<false>+0x556>
   335a3:	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:614
   335b0:	mov    0x8(%rdi),%rax
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   335b4:	mov    %r13,%rcx
add<c30walk::RawG>():
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   335b7:	shl    $0x7,%rcx
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   335bb:	movdqa %xmm0,0x70(%rax,%rcx,1)
write<c30walk::RawG>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   335c1:	movdqa %xmm0,0x60(%rax,%rcx,1)
   335c7:	movdqa %xmm0,0x50(%rax,%rcx,1)
   335cd:	movdqa %xmm0,0x40(%rax,%rcx,1)
   335d3:	movdqa %xmm0,0x30(%rax,%rcx,1)
   335d9:	movdqa %xmm0,0x20(%rax,%rcx,1)
   335df:	movdqa %xmm0,0x10(%rax,%rcx,1)
   335e5:	movdqa %xmm0,(%rax,%rcx,1)
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:1046
   335ea:	mov    %r13,%rax
push_mut<c30walk::RawG, alloc::alloc::Global>():
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:1046
   335ed:	inc    %rax
   335f0:	mov    %rax,0x10(%rdi)
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   335f4:	mov    %r14,%rax
add<c30walk::Rec>():
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   335f7:	shl    $0x6,%rax
<c30walk::ctor2::Ctor2>::build::<false>:
./examples/c30walk/ctor2.rs:315
   335fb:	or     $0x10000000,%r13d
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   33602:	movdqu %xmm1,0x2c(%rbx,%rax,1)
write<c30walk::Rec>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   33608:	movdqa %xmm1,0x20(%rbx,%rax,1)
   3360e:	movdqa %xmm1,0x10(%rbx,%rax,1)
   33614:	movdqa %xmm1,(%rbx,%rax,1)
   33619:	mov    %r13d,0x3c(%rbx,%rax,1)
create_raw<false>():
./examples/c30walk/ctor2.rs:317
   3361e:	mov    %r14,%rdi
   33621:	shr    $0x6,%rdi
   33625:	cmp    %rsi,%rdi
   33628:	jae    349ca <<c30walk::ctor2::Ctor2>::build::<false>+0x18ca>
   3362e:	mov    $0x1,%eax
   33633:	mov    %r14d,%ecx
   33636:	shl    %cl,%rax
   33639:	or     %rax,(%r8,%rdi,8)
   3363d:	mov    %r14,%rcx
   33640:	mov    0x8(%rsp),%r13
<c30walk::ctor2::Ctor2>::build::<false>:
   33645:	add    $0x4,%r12
/rustc/2d8144b78/library/core/src/ptr/non_null.rs:1714
   33649:	cmp    $0x400,%r12
next<u32>():
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   33650:	je     336f0 <<c30walk::ctor2::Ctor2>::build::<false>+0x5f0>
<c30walk::ctor2::Ctor2>::build::<false>:
   33656:	mov    %rbp,%rax
scatter<false>():
./examples/c30walk/ctor2.rs:804
   33659:	mov    (%r15,%r12,1),%ebp
./examples/c30walk/ctor2.rs:805
   3365d:	add    %rax,%rbp
   33660:	cmp    %r13,%rbp
   33663:	setae  %dil
./examples/c30walk/ctor2.rs:807
   33667:	mov    %rbp,%rax
   3366a:	movabs $0x8888888888888889,%rdx
   33674:	mul    %rdx
   33677:	mov    %rdx,%r14
   3367a:	shr    $0x4,%r14
   3367e:	mov    %rbp,%rax
   33681:	imul   %r9,%rax
   33685:	ror    $1,%rax
   33688:	cmp    %r10,%rax
   3368b:	setb   %al
   3368e:	cmp    %rcx,%r14
   33691:	sete   %dl
./examples/c30walk/ctor2.rs:806
   33694:	or     %dil,%dl
   33697:	or     %al,%dl
   33699:	jne    33645 <<c30walk::ctor2::Ctor2>::build::<false>+0x545>
   3369b:	mov    0x5520(%rsp),%rdi
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:3054
   336a3:	mov    0x10(%rdi),%r13
push_mut<c30walk::RawG, alloc::alloc::Global>():
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:1040
   336a7:	cmp    (%rdi),%r13
   336aa:	jne    335b0 <<c30walk::ctor2::Ctor2>::build::<false>+0x4b0>
   336b0:	mov    %rsi,(%rsp)
   336b4:	mov    %r8,0x10(%rsp)
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:1041
   336b9:	call   65930 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   336be:	pcmpeqd %xmm1,%xmm1
   336c2:	pxor   %xmm0,%xmm0
   336c6:	movabs $0x888888888888889,%r10
   336d0:	movabs $0xeeeeeeeeeeeeeeef,%r9
   336da:	mov    0x10(%rsp),%r8
   336df:	mov    (%rsp),%rsi
   336e3:	mov    0x5520(%rsp),%rdi
   336eb:	jmp    335b0 <<c30walk::ctor2::Ctor2>::build::<false>+0x4b0>
<c30walk::ctor2::Ctor2>::build::<false>:
./examples/c30walk/ctor2.rs:705
   336f0:	cmpl   $0x0,0x1c(%rsp)
   336f5:	lea    0xd8(%rsp),%r8
scatter<false>():
./examples/c30walk/ctor2.rs:815
   336fd:	je     338b4 <<c30walk::ctor2::Ctor2>::build::<false>+0x7b4>
./examples/c30walk/ctor2.rs:816
   33703:	mov    0xc8(%rsp),%r11
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/cmp.rs:1916
   3370b:	test   %r11,%r11
/rustc/2d8144b78/library/core/src/iter/range.rs:900
   3370e:	je     338b4 <<c30walk::ctor2::Ctor2>::build::<false>+0x7b4>
scatter<false>():
   33714:	mov    0x8(%rsp),%r9
   33719:	shl    $0x20,%r9
   3371d:	mov    0x1c(%rsp),%r10d
   33722:	mov    0x98(%rsp),%rdi
   3372a:	mov    0xc0(%rsp),%r13
   33732:	xor    %r12d,%r12d
   33735:	movabs $0xff51afd7ed558ccd,%rbx
   3373f:	pxor   %xmm0,%xmm0
   33743:	pcmpeqd %xmm1,%xmm1
   33747:	jmp    337d0 <<c30walk::ctor2::Ctor2>::build::<false>+0x6d0>
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:614
   3374c:	mov    0x8(%rdi),%rax
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   33750:	mov    %rbx,%rcx
add<c30walk::RawG>():
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   33753:	shl    $0x7,%rcx
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   33757:	movdqa %xmm0,0x70(%rax,%rcx,1)
write<c30walk::RawG>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   3375d:	movdqa %xmm0,0x60(%rax,%rcx,1)
   33763:	movdqa %xmm0,0x50(%rax,%rcx,1)
   33769:	movdqa %xmm0,0x40(%rax,%rcx,1)
   3376f:	movdqa %xmm0,0x30(%rax,%rcx,1)
   33775:	movdqa %xmm0,0x20(%rax,%rcx,1)
   3377b:	movdqa %xmm0,0x10(%rax,%rcx,1)
   33781:	movdqa %xmm0,(%rax,%rcx,1)
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:1046
   33786:	mov    %rbx,%rax
push_mut<c30walk::RawG, alloc::alloc::Global>():
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:1046
   33789:	inc    %rax
   3378c:	mov    %rax,0x10(%rdi)
<c30walk::ctor2::Ctor2>::build::<false>:
./examples/c30walk/ctor2.rs:315
   33790:	or     $0x10000000,%ebx
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   33796:	movdqu %xmm1,0x2c(%r13)
write<c30walk::Rec>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   3379c:	movdqa %xmm1,0x20(%r13)
   337a2:	movdqa %xmm1,0x10(%r13)
   337a8:	movdqa %xmm1,0x0(%r13)
   337ae:	mov    %ebx,0x3c(%r13)
create_raw<false>():
./examples/c30walk/ctor2.rs:317
   337b2:	or     %r15,(%r14,%rbp,8)
   337b6:	movabs $0xff51afd7ed558ccd,%rbx
scatter<false>():
   337c0:	inc    %r12
lt():
/rustc/2d8144b78/library/core/src/cmp.rs:1916
   337c3:	add    $0x40,%r13
   337c7:	cmp    %r12,%r11
spec_next<usize>():
/rustc/2d8144b78/library/core/src/iter/range.rs:900
   337ca:	je     338b4 <<c30walk::ctor2::Ctor2>::build::<false>+0x7b4>
<c30walk::ctor2::Ctor2>::build::<false>:
./examples/c30walk/ctor2.rs:285
   337d0:	mov    %r12,%rbp
is_raw<false>():
./examples/c30walk/ctor2.rs:285
   337d3:	shr    $0x6,%rbp
   337d7:	mov    0xa8(%rsp),%rsi
   337df:	cmp    %rsi,%rbp
   337e2:	jae    349d6 <<c30walk::ctor2::Ctor2>::build::<false>+0x18d6>
   337e8:	mov    0xa0(%rsp),%r14
   337f0:	mov    (%r14,%rbp,8),%rax
scatter<false>():
./examples/c30walk/ctor2.rs:817
   337f4:	mov    $0x1,%r15d
   337fa:	mov    %r12d,%ecx
   337fd:	shl    %cl,%r15
   33800:	bt     %r12,%rax
   33804:	jb     337c0 <<c30walk::ctor2::Ctor2>::build::<false>+0x6c0>
<c30walk::ctor2::Ctor2>::build::<false>:
./examples/c30walk/ctor2.rs:95
   33806:	mov    %r12,%rax
forced():
./examples/c30walk/ctor2.rs:95
   33809:	xor    %r9,%rax
<c30walk::ctor2::Ctor2>::build::<false>:
./examples/c30walk.rs:753
   3380c:	mov    %rax,%rcx
mix():
./examples/c30walk.rs:753
   3380f:	shr    $0x21,%rcx
   33813:	xor    %rax,%rcx
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:2660
   33816:	imul   %rbx,%rcx
mix():
./examples/c30walk.rs:755
   3381a:	mov    %rcx,%rax
   3381d:	shr    $0x21,%rax
   33821:	xor    %rcx,%rax
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:2660
   33824:	movabs $0xc4ceb9fe1a85ec53,%rcx
wrapping_mul():
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:2660
   3382e:	imul   %rcx,%rax
mix():
./examples/c30walk.rs:757
   33832:	mov    %rax,%rcx
   33835:	shr    $0x21,%rcx
   33839:	xor    %rax,%rcx
forced():
./examples/c30walk/ctor2.rs:95
   3383c:	mov    %rcx,%rax
   3383f:	shr    $0x3,%rax
   33843:	movabs $0x20c49ba5e353f7cf,%rdx
   3384d:	mul    %rdx
   33850:	shr    $0x4,%rdx
   33854:	imul   $0x3e8,%rdx,%rax
   3385b:	sub    %rax,%rcx
   3385e:	cmp    %r10,%rcx
scatter<false>():
./examples/c30walk/ctor2.rs:817
   33861:	jae    337c0 <<c30walk::ctor2::Ctor2>::build::<false>+0x6c0>
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:3054
   33867:	mov    0x10(%rdi),%rbx
push_mut<c30walk::RawG, alloc::alloc::Global>():
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:1040
   3386b:	cmp    (%rdi),%rbx
   3386e:	jne    3374c <<c30walk::ctor2::Ctor2>::build::<false>+0x64c>
   33874:	mov    %r11,(%rsp)
   33878:	mov    %r9,0x10(%rsp)
   3387d:	mov    %r10,0x50(%rsp)
   33882:	mov    %rdi,0x48(%rsp)
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:1041
   33887:	call   65930 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   3388c:	pcmpeqd %xmm1,%xmm1
   33890:	pxor   %xmm0,%xmm0
   33894:	mov    0x48(%rsp),%rdi
   33899:	mov    0x50(%rsp),%r10
   3389e:	mov    0x10(%rsp),%r9
   338a3:	mov    (%rsp),%r11
   338a7:	lea    0xd8(%rsp),%r8
   338af:	jmp    3374c <<c30walk::ctor2::Ctor2>::build::<false>+0x64c>
scatter<false>():
   338b4:	mov    0x8(%rsp),%r13
./examples/c30walk/ctor2.rs:823
   338b9:	test   %r13,%r13
   338bc:	mov    0x28(%rsp),%r14
   338c1:	je     3460f <<c30walk::ctor2::Ctor2>::build::<false>+0x150f>
   338c7:	xor    %r11d,%r11d
   338ca:	jmp    338dc <<c30walk::ctor2::Ctor2>::build::<false>+0x7dc>
   338cc:	nopl   0x0(%rax)
   338d0:	mov    %r11,%r10
   338d3:	cmp    %r13,%r10
   338d6:	jae    3460f <<c30walk::ctor2::Ctor2>::build::<false>+0x150f>
   338dc:	mov    %r11,%rbp
   338df:	mov    %r9,%r12
<c30walk::ctor2::Ctor2>::build::<false>:
./examples/c30walk/ctor2.rs:323
   338e2:	mov    $0x400,%edx
snapshot<false>():
./examples/c30walk/ctor2.rs:323
   338e7:	lea    0x28e8(%rsp),%rdi
   338ef:	xor    %esi,%esi
   338f1:	call   *0xc3399(%rip)        # f6c90 <memset@GLIBC_2.2.5>
   338f7:	lea    0x118(%rsp),%rax
   338ff:	xor    %ecx,%ecx
   33901:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
<c30walk::ctor2::Ctor2>::build::<false>:
./examples/c30walk/ctor2.rs:247
   33910:	mov    -0x40(%rax),%rdx
rank():
./examples/c30walk/ctor2.rs:247
   33914:	sub    -0x30(%rax),%rdx
   33918:	shr    $1,%rdx
   3391b:	add    -0x28(%rax),%edx
   3391e:	mov    -0x18(%rax),%rsi
   33922:	sub    -0x8(%rax),%rsi
snapshot<false>():
./examples/c30walk/ctor2.rs:325
   33926:	mov    %edx,0x28e8(%rsp,%rcx,4)
rank():
./examples/c30walk/ctor2.rs:247
   3392d:	shr    $1,%rsi
   33930:	add    (%rax),%esi
snapshot<false>():
./examples/c30walk/ctor2.rs:325
   33932:	mov    %esi,0x28ec(%rsp,%rcx,4)
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/iter/adapters/zip.rs:308
   33939:	add    $0x2,%rcx
next<core::slice::iter::IterMut<u32>, core::slice::iter::Iter<c30walk::ctor2::Desc>>():
/rustc/2d8144b78/library/core/src/iter/adapters/zip.rs:304
   3393d:	add    $0x50,%rax
   33941:	cmp    $0x100,%rcx
   33948:	jne    33910 <<c30walk::ctor2::Ctor2>::build::<false>+0x810>
snapshot<false>():
./examples/c30walk/ctor2.rs:327
   3394a:	mov    0xb0(%rsp),%r15
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:1037
   33952:	mov    0x10(%r15),%rbx
push_mut<[u32; 256], alloc::alloc::Global>():
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:1040
   33956:	cmp    (%r15),%rbx
   33959:	je     34602 <<c30walk::ctor2::Ctor2>::build::<false>+0x1502>
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   3395f:	mov    %rbx,%rdi
add<[u32; 256]>():
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   33962:	shl    $0xa,%rdi
   33966:	add    0x8(%r15),%rdi
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   3396a:	mov    $0x400,%edx
write<[u32; 256]>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   3396f:	lea    0x28e8(%rsp),%rsi
   33977:	call   *0xc3333(%rip)        # f6cb0 <memcpy@GLIBC_2.14>
push_mut<[u32; 256], alloc::alloc::Global>():
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:1046
   3397d:	inc    %rbx
   33980:	mov    %rbx,0x10(%r15)
   33984:	mov    %rbp,%r11
scatter<false>():
./examples/c30walk/ctor2.rs:826
   33987:	lea    0xf000(%rbp),%r10
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/cmp.rs:1077
   3398e:	cmp    %r13,%r10
min<usize>():
/rustc/2d8144b78/library/core/src/cmp.rs:1077
   33991:	cmovae %r13,%r10
scatter<false>():
./examples/c30walk/ctor2.rs:828
   33995:	lea    0x10(%rbp),%rdi
   33999:	cmp    %r10,%rdi
   3399c:	mov    %r10,0x20(%rsp)
   339a1:	jbe    33a70 <<c30walk::ctor2::Ctor2>::build::<false>+0x970>
   339a7:	lea    0xd8(%rsp),%r8
   339af:	mov    %r12,%r9
   339b2:	mov    0x38(%rsp),%rcx
./examples/c30walk/ctor2.rs:858
   339b7:	mov    %r10,%rdx
   339ba:	sub    %r11,%rdx
   339bd:	mov    %rcx,0x38(%rsp)
   339c2:	jbe    338d0 <<c30walk::ctor2::Ctor2>::build::<false>+0x7d0>
   339c8:	mov    %r9,%rbp
   339cb:	mov    %r14,%rax
<c30walk::ctor2::Ctor2>::build::<false>:
./examples/c30walk/ctor2.rs:558
   339ce:	mov    0x28e0(%rsp),%r12d
tile<false>():
./examples/c30walk/ctor2.rs:559
   339d6:	mov    $0x1e,%r14d
   339dc:	sub    0x28d8(%rsp),%r14
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/cmp.rs:1077
   339e4:	cmp    %rdx,%r14
min<usize>():
/rustc/2d8144b78/library/core/src/cmp.rs:1077
   339e7:	cmovae %rdx,%r14
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/slice/index.rs:89
   339eb:	lea    (%rax,%r11,1),%r15
/rustc/2d8144b78/library/core/src/ptr/non_null.rs:1714
   339ef:	test   %r14,%r14
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   339f2:	je     34470 <<c30walk::ctor2::Ctor2>::build::<false>+0x1370>
tile<false>():
   339f8:	xor    %r13d,%r13d
   339fb:	nopl   0x0(%rax,%rax,1)
./examples/c30walk/ctor2.rs:560
   33a00:	movzbl (%r15,%r13,1),%esi
<c30walk::ctor2::Ctor2>::build::<false>:
./examples/c30walk/ctor2.rs:542
   33a05:	lea    (%rsi,%rsi,4),%rcx
one<false>():
./examples/c30walk/ctor2.rs:542
   33a09:	lea    (%r8,%rcx,8),%rbx
   33a0d:	mov    (%r8,%rcx,8),%rax
   33a11:	cmp    0x8(%r8,%rcx,8),%rax
   33a16:	je     33a30 <<c30walk::ctor2::Ctor2>::build::<false>+0x930>
   33a18:	lea    (%r12,%r13,1),%rcx
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   33a1c:	mov    %cx,(%rax)
one<false>():
./examples/c30walk/ctor2.rs:547
   33a1f:	addq   $0x2,(%rbx)
eq<u8>():
/rustc/2d8144b78/library/core/src/ptr/non_null.rs:1714
   33a23:	inc    %r13
   33a26:	cmp    %r13,%r14
next<u8>():
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   33a29:	jne    33a00 <<c30walk::ctor2::Ctor2>::build::<false>+0x900>
   33a2b:	jmp    34470 <<c30walk::ctor2::Ctor2>::build::<false>+0x1370>
tile<false>():
   33a30:	mov    %rdx,0x10(%rsp)
   33a35:	mov    %r11,(%rsp)
one<false>():
./examples/c30walk/ctor2.rs:542
   33a39:	lea    0x98(%rsp),%rdi
   33a41:	call   62330 <<c30walk::ctor2::W<false>>::refill>
   33a46:	lea    0xd8(%rsp),%r8
   33a4e:	test   %al,%al
   33a50:	je     34667 <<c30walk::ctor2::Ctor2>::build::<false>+0x1567>
./examples/c30walk/ctor2.rs:546
   33a56:	mov    (%rbx),%rax
   33a59:	mov    0x20(%rsp),%r10
   33a5e:	mov    (%rsp),%r11
   33a62:	mov    0x10(%rsp),%rdx
   33a67:	jmp    33a18 <<c30walk::ctor2::Ctor2>::build::<false>+0x918>
scatter<false>():
   33a69:	nopl   0x0(%rax)
   33a70:	xor    %edx,%edx
   33a72:	lea    0xd8(%rsp),%r8
   33a7a:	mov    %r12,%r9
   33a7d:	mov    0x38(%rsp),%rcx
   33a82:	jmp    33ae3 <<c30walk::ctor2::Ctor2>::build::<false>+0x9e3>
   33a84:	data16 data16 cs nopw 0x0(%rax,%rax,1)
<c30walk::ctor2::Ctor2>::build::<false>:
./examples/c30walk/ctor2.rs:298
   33a90:	mov    0x28d8(%rsp),%rax
advance<false>():
./examples/c30walk/ctor2.rs:298
   33a98:	lea    0x10(%rax),%rcx
./examples/c30walk/ctor2.rs:299
   33a9c:	add    $0xfffffffffffffff2,%rax
   33aa0:	xor    %edx,%edx
   33aa2:	cmp    $0x1e,%rcx
   33aa6:	setae  %dl
   33aa9:	cmovb  %rcx,%rax
   33aad:	mov    %rax,0x28d8(%rsp)
   33ab5:	mov    0x28e0(%rsp),%eax
   33abc:	lea    0x10(%rax,%rdx,2),%eax
   33ac0:	mov    %eax,0x28e0(%rsp)
   33ac7:	mov    0x30(%rsp),%r11
scatter<false>():
./examples/c30walk/ctor2.rs:828
   33acc:	lea    0x10(%r11),%rsi
   33ad0:	xor    %edx,%edx
   33ad2:	mov    %rsi,%rdi
   33ad5:	cmp    %r10,%rsi
   33ad8:	mov    0x28(%rsp),%r14
   33add:	ja     339b7 <<c30walk::ctor2::Ctor2>::build::<false>+0x8b7>
./examples/c30walk/ctor2.rs:829
   33ae3:	test   $0x1,%dl
   33ae6:	jne    33b14 <<c30walk::ctor2::Ctor2>::build::<false>+0xa14>
<c30walk::ctor2::Ctor2>::build::<false>:
./examples/c30walk/ctor2.rs:99
   33ae8:	lea    0x8(%r11),%rsi
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:1050
   33aec:	cmp    $0xfffffffffffffff7,%r11
checked_sub():
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:1050
   33af0:	ja     34898 <<c30walk::ctor2::Ctor2>::build::<false>+0x1798>
   33af6:	cmp    %r13,%rsi
   33af9:	ja     34898 <<c30walk::ctor2::Ctor2>::build::<false>+0x1798>
<c30walk::ctor2::Ctor2>::build::<false>:
./examples/c30walk/ctor2.rs:99
   33aff:	lea    0x10(%r11),%rax
rd():
./examples/c30walk/ctor2.rs:99
   33b03:	cmp    %r13,%rax
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:1050
   33b06:	ja     348bf <<c30walk::ctor2::Ctor2>::build::<false>+0x17bf>
/rustc/2d8144b78/library/core/src/result.rs:1718
   33b0c:	mov    (%r14,%r11,1),%r9
   33b10:	mov    (%r14,%rsi,1),%rcx
scatter<false>():
./examples/c30walk/ctor2.rs:833
   33b14:	mov    %r9,%rax
   33b17:	xor    %rcx,%rax
   33b1a:	mov    %r9,%rdx
   33b1d:	rol    $0x8,%rdx
   33b21:	xor    %r9,%rdx
   33b24:	or     %rax,%rdx
   33b27:	jne    33b80 <<c30walk::ctor2::Ctor2>::build::<false>+0xa80>
   33b29:	mov    %rdi,%rax
   33b2c:	nopl   0x0(%rax)
   33b30:	mov    %rax,%r12
./examples/c30walk/ctor2.rs:835
   33b33:	add    $0x10,%rax
   33b37:	cmp    %r10,%rax
   33b3a:	setbe  %r9b
   33b3e:	ja     33be0 <<c30walk::ctor2::Ctor2>::build::<false>+0xae0>
<c30walk::ctor2::Ctor2>::build::<false>:
./examples/c30walk/ctor2.rs:99
   33b44:	lea    0x8(%r12),%rsi
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:1050
   33b49:	cmp    $0xfffffffffffffff7,%r12
checked_sub():
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:1050
   33b4d:	ja     347ba <<c30walk::ctor2::Ctor2>::build::<false>+0x16ba>
   33b53:	cmp    %r13,%rsi
   33b56:	ja     347ba <<c30walk::ctor2::Ctor2>::build::<false>+0x16ba>
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/result.rs:1718
   33b5c:	mov    (%r14,%r12,1),%rdx
   33b60:	mov    0x8(%r14,%r12,1),%rsi
scatter<false>():
./examples/c30walk/ctor2.rs:837
   33b65:	cmp    %rcx,%rdx
   33b68:	jne    33b6f <<c30walk::ctor2::Ctor2>::build::<false>+0xa6f>
   33b6a:	cmp    %rcx,%rsi
   33b6d:	je     33b30 <<c30walk::ctor2::Ctor2>::build::<false>+0xa30>
   33b6f:	mov    %rsi,0x38(%rsp)
   33b74:	mov    %rdx,0x68(%rsp)
   33b79:	lea    0x10(%r12),%r10
   33b7e:	jmp    33be5 <<c30walk::ctor2::Ctor2>::build::<false>+0xae5>
   33b80:	mov    %rdi,0x30(%rsp)
<c30walk::ctor2::Ctor2>::build::<false>:
./examples/c30walk/ctor2.rs:559
   33b85:	mov    0x28d8(%rsp),%rax
tile<false>():
./examples/c30walk/ctor2.rs:559
   33b8d:	mov    $0x1e,%r12d
   33b93:	sub    %rax,%r12
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/cmp.rs:1077
   33b96:	cmp    $0x10,%r12
min<usize>():
/rustc/2d8144b78/library/core/src/cmp.rs:1077
   33b9a:	mov    $0x10,%ebx
   33b9f:	cmovb  %r12,%rbx
tile<false>():
./examples/c30walk/ctor2.rs:560
   33ba3:	mov    %rbx,%r15
   33ba6:	add    %r11,%r15
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:1050
   33ba9:	jb     348aa <<c30walk::ctor2::Ctor2>::build::<false>+0x17aa>
checked_sub():
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:1050
   33baf:	cmp    %r13,%r15
   33bb2:	ja     348aa <<c30walk::ctor2::Ctor2>::build::<false>+0x17aa>
tile<false>():
   33bb8:	mov    0x28e0(%rsp),%r14d
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/non_null.rs:1714
   33bc0:	cmp    $0x1e,%rax
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   33bc4:	jne    34340 <<c30walk::ctor2::Ctor2>::build::<false>+0x1240>
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:1050
   33bca:	cmp    %r15,0x30(%rsp)
checked_sub():
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:1050
   33bcf:	jae    343ca <<c30walk::ctor2::Ctor2>::build::<false>+0x12ca>
   33bd5:	jmp    348d4 <<c30walk::ctor2::Ctor2>::build::<false>+0x17d4>
scatter<false>():
   33bda:	nopw   0x0(%rax,%rax,1)
./examples/c30walk/ctor2.rs:847
   33be0:	lea    0x10(%r12),%r10
   33be5:	mov    %r12,0x30(%rsp)
   33bea:	sub    %r11,%r12
<c30walk::ctor2::Ctor2>::build::<false>:
./examples/c30walk/ctor2.rs:647
   33bed:	movzbl %cl,%eax
run<false>():
./examples/c30walk/ctor2.rs:647
   33bf0:	mov    %rax,0x90(%rsp)
./examples/c30walk/ctor2.rs:648
   33bf8:	lea    (%rax,%rax,4),%rdx
   33bfc:	mov    (%r8,%rdx,8),%rax
   33c00:	mov    0x1c(%r8,%rdx,8),%esi
<c30walk::ctor2::Ctor2>::build::<false>:
./examples/c30walk/ctor2.rs:247
   33c05:	mov    %rax,%rdi
rank():
./examples/c30walk/ctor2.rs:247
   33c08:	sub    0x10(%r8,%rdx,8),%rdi
   33c0d:	shr    $1,%rdi
   33c10:	add    0x18(%r8,%rdx,8),%edi
run<false>():
./examples/c30walk/ctor2.rs:649
   33c15:	add    %r12,%rdi
   33c18:	cmp    %rsi,%rdi
   33c1b:	ja     34667 <<c30walk::ctor2::Ctor2>::build::<false>+0x1567>
scatter<false>():
   33c21:	mov    %r10,0x70(%rsp)
   33c26:	mov    %r9d,0x44(%rsp)
./examples/c30walk/ctor2.rs:847
   33c2b:	lea    (%r8,%rdx,8),%rbp
   33c2f:	movzbl %cl,%r13d
run<false>():
./examples/c30walk/ctor2.rs:657
   33c33:	mov    0x8(%rbp),%rsi
   33c37:	mov    $0x88888889,%r14d
   33c3d:	lea    -0x1ee04(%rip),%r15        # 14e40 <c30walk::ctor2::PATTERNS>
   33c44:	mov    %rbp,0x88(%rsp)
   33c4c:	jmp    33c87 <<c30walk::ctor2::Ctor2>::build::<false>+0xb87>
   33c4e:	xchg   %ax,%ax
   33c50:	mov    0x3c(%rsi),%edi
<c30walk::ctor2::Ctor2>::build::<false>:
./examples/c30walk/ctor2.rs:353
   33c53:	mov    $0xfffff,%r9d
open<false>():
./examples/c30walk/ctor2.rs:353
   33c59:	and    %r9d,%edi
./examples/c30walk/ctor2.rs:363
   33c5c:	sub    %r8d,%ecx
   33c5f:	sub    %eax,%r8d
<c30walk::ctor2::Ctor2>::build::<false>:
./examples/c30walk/ctor2.rs:291
   33c62:	mov    %r8d,%eax
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   33c65:	lea    (%rsi,%rax,2),%rax
open<false>():
./examples/c30walk/ctor2.rs:361
   33c69:	mov    %rax,0x10(%rbp)
./examples/c30walk/ctor2.rs:362
   33c6d:	mov    %rax,0x0(%rbp)
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   33c71:	lea    (%rax,%rcx,2),%rsi
open<false>():
./examples/c30walk/ctor2.rs:363
   33c75:	mov    %rsi,0x8(%rbp)
./examples/c30walk/ctor2.rs:365
   33c79:	mov    %edx,0x20(%rbp)
./examples/c30walk/ctor2.rs:366
   33c7c:	mov    %edi,0x24(%rbp)
   33c7f:	lea    0xd8(%rsp),%r8
run<false>():
./examples/c30walk/ctor2.rs:658
   33c87:	sub    %rax,%rsi
   33c8a:	shr    $1,%rsi
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/cmp.rs:1077
   33c8d:	cmp    %rsi,%r12
min<usize>():
/rustc/2d8144b78/library/core/src/cmp.rs:1077
   33c90:	cmovb  %r12,%rsi
run<false>():
./examples/c30walk/ctor2.rs:660
   33c94:	test   %rsi,%rsi
   33c97:	je     33d51 <<c30walk::ctor2::Ctor2>::build::<false>+0xc51>
<c30walk::ctor2::Ctor2>::build::<false>:
./examples/c30walk/ctor2.rs:584
   33c9d:	mov    0x28d8(%rsp),%rdi
fill<false>():
./examples/c30walk/ctor2.rs:584
   33ca5:	cmp    $0x1d,%rdi
   33ca9:	ja     349a8 <<c30walk::ctor2::Ctor2>::build::<false>+0x18a8>
   33caf:	cmp    $0x1f,%rsi
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:1050
   33cb3:	jae    347cc <<c30walk::ctor2::Ctor2>::build::<false>+0x16cc>
fill<false>():
   33cb9:	mov    0x28e0(%rsp),%ecx
./examples/c30walk/ctor2.rs:584
   33cc0:	mov    %rdi,%r11
   33cc3:	shl    $0x6,%r11
   33cc7:	lea    (%r15,%r11,1),%rdx
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   33ccb:	lea    -0x2(,%rsi,2),%r9
next<u16>():
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   33cd3:	cmp    $0xe,%r9
   33cd7:	jb     33cec <<c30walk::ctor2::Ctor2>::build::<false>+0xbec>
   33cd9:	mov    %rax,%r8
   33cdc:	sub    %r15,%r8
   33cdf:	sub    %r11,%r8
   33ce2:	cmp    $0x10,%r8
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/iter/adapters/enumerate.rs:82
   33ce6:	jae    34153 <<c30walk::ctor2::Ctor2>::build::<false>+0x1053>
fill<false>():
   33cec:	xor    %r10d,%r10d
   33cef:	mov    %rdx,%r8
./examples/c30walk/ctor2.rs:588
   33cf2:	lea    (%rdx,%rsi,2),%rdx
next<u16>():
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   33cf6:	lea    (%rax,%r10,2),%r9
fill<false>():
   33cfa:	nopw   0x0(%rax,%rax,1)
   33d00:	movzwl (%r8),%r10d
./examples/c30walk/ctor2.rs:589
   33d04:	add    %cx,%r10w
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/non_null.rs:656
   33d08:	add    $0x2,%r8
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   33d0c:	mov    %r10w,(%r9)
/rustc/2d8144b78/library/core/src/ptr/non_null.rs:1714
   33d10:	add    $0x2,%r9
eq<u16>():
/rustc/2d8144b78/library/core/src/ptr/non_null.rs:1714
   33d14:	cmp    %rdx,%r8
next<u16>():
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   33d17:	jne    33d00 <<c30walk::ctor2::Ctor2>::build::<false>+0xc00>
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   33d19:	lea    (%rax,%rsi,2),%rax
fill<false>():
./examples/c30walk/ctor2.rs:591
   33d1d:	mov    %rax,0x0(%rbp)
<c30walk::ctor2::Ctor2>::build::<false>:
./examples/c30walk/ctor2.rs:298
   33d21:	add    %rsi,%rdi
advance<false>():
./examples/c30walk/ctor2.rs:299
   33d24:	lea    0x2(%rsi),%edx
   33d27:	mov    %rdi,%r8
   33d2a:	sub    $0x1e,%r8
   33d2e:	cmovb  %rdi,%r8
   33d32:	cmovb  %esi,%edx
   33d35:	mov    %r8,0x28d8(%rsp)
   33d3d:	add    %ecx,%edx
   33d3f:	mov    %edx,0x28e0(%rsp)
run<false>():
./examples/c30walk/ctor2.rs:662
   33d46:	sub    %rsi,%r12
   33d49:	lea    0xd8(%rsp),%r8
./examples/c30walk/ctor2.rs:664
   33d51:	test   %r12,%r12
   33d54:	je     34330 <<c30walk::ctor2::Ctor2>::build::<false>+0x1230>
   33d5a:	mov    %rax,%r8
<c30walk::ctor2::Ctor2>::build::<false>:
./examples/c30walk/ctor2.rs:373
   33d5d:	cmp    0x10(%rbp),%rax
close<false>():
./examples/c30walk/ctor2.rs:373
   33d61:	je     33d80 <<c30walk::ctor2::Ctor2>::build::<false>+0xc80>
./examples/c30walk/ctor2.rs:374
   33d63:	lea    0x98(%rsp),%rdi
   33d6b:	mov    0x90(%rsp),%rsi
   33d73:	call   61d30 <<c30walk::ctor2::W<false>>::finish>
<c30walk::ctor2::Ctor2>::build::<false>:
./examples/c30walk/ctor2.rs:247
   33d78:	mov    0x0(%rbp),%r8
rank():
./examples/c30walk/ctor2.rs:247
   33d7c:	mov    0x10(%rbp),%rax
   33d80:	sub    %rax,%r8
   33d83:	shr    $1,%r8
   33d86:	add    0x18(%rbp),%r8d
close<false>():
./examples/c30walk/ctor2.rs:377
   33d8a:	mov    %r8d,0x18(%rbp)
./examples/c30walk/ctor2.rs:379
   33d8e:	pxor   %xmm0,%xmm0
   33d92:	movdqu %xmm0,0x0(%rbp)
   33d97:	movq   $0x0,0x10(%rbp)
   33d9f:	cmp    $0x1e,%r12
run<false>():
./examples/c30walk/ctor2.rs:669
   33da3:	jb     342c4 <<c30walk::ctor2::Ctor2>::build::<false>+0x11c4>
   33da9:	mov    %r8,%rcx
   33dac:	imul   %r14,%rcx
   33db0:	shr    $0x24,%rcx
   33db4:	mov    %ecx,%eax
   33db6:	shl    $0x5,%eax
   33db9:	lea    (%rcx,%rcx,1),%edx
   33dbc:	sub    %eax,%edx
   33dbe:	add    %r8d,%edx
   33dc1:	jne    342c4 <<c30walk::ctor2::Ctor2>::build::<false>+0x11c4>
./examples/c30walk/ctor2.rs:672
   33dc7:	mov    %r12,%rax
   33dca:	movabs $0x8888888888888889,%rdx
   33dd4:	mul    %rdx
   33dd7:	shr    $0x4,%rdx
./examples/c30walk/ctor2.rs:671
   33ddb:	mov    %ecx,%r9d
<c30walk::ctor2::Ctor2>::build::<false>:
./examples/c30walk/ctor2.rs:705
   33dde:	cmpl   $0x0,0x1c(%rsp)
   33de3:	mov    %r12,0x50(%rsp)
   33de8:	mov    %rdx,0x48(%rsp)
./examples/c30walk/ctor2.rs:603
   33ded:	je     34110 <<c30walk::ctor2::Ctor2>::build::<false>+0x1010>
complete<false>():
   33df3:	mov    0x28e0(%rsp),%r10d
   33dfb:	mov    0xa8(%rsp),%rax
   33e03:	mov    %rax,(%rsp)
   33e07:	mov    0x98(%rsp),%r11
   33e0f:	mov    0xa0(%rsp),%rax
   33e17:	mov    %rax,0x10(%rsp)
   33e1c:	mov    0xc0(%rsp),%rax
   33e24:	mov    0x28d8(%rsp),%rbx
   33e2c:	mov    %rbx,%rdi
   33e2f:	shl    $0x6,%rdi
   33e33:	add    %r15,%rdi
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/iter/range.rs:900
   33e36:	mov    %r9,%rcx
spec_next<usize>():
/rustc/2d8144b78/library/core/src/iter/range.rs:900
   33e39:	shl    $0x6,%rcx
   33e3d:	lea    (%rax,%rcx,1),%r12
   33e41:	add    $0x3c,%r12
   33e45:	mov    %r10d,%r15d
   33e48:	shl    $0x8,%r15d
   33e4c:	or     %r13d,%r15d
   33e4f:	xor    %ebp,%ebp
   33e51:	jmp    34081 <<c30walk::ctor2::Ctor2>::build::<false>+0xf81>
<c30walk::ctor2::Ctor2>::build::<false>:
   33e56:	cs nopw 0x0(%rax,%rax,1)
non_null<alloc::alloc::Global, c30walk::RawG>():
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:614
   33e60:	mov    0x8(%r11),%rcx
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/slice/index.rs:278
   33e64:	shl    $0x7,%eax
complete<false>():
./examples/c30walk/ctor2.rs:625
   33e67:	mov    %r15d,(%rcx,%rax,1)
./examples/c30walk/ctor2.rs:624
   33e6b:	movzwl 0x2(%rdi),%esi
./examples/c30walk/ctor2.rs:625
   33e6f:	lea    (%r10,%rbp,1),%r14d
   33e73:	add    %r14d,%esi
   33e76:	shl    $0x8,%esi
   33e79:	or     %r13d,%esi
   33e7c:	mov    %esi,0x4(%rcx,%rax,1)
./examples/c30walk/ctor2.rs:624
   33e80:	movzwl 0x4(%rdi),%esi
./examples/c30walk/ctor2.rs:625
   33e84:	add    %r14d,%esi
   33e87:	shl    $0x8,%esi
   33e8a:	or     %r13d,%esi
   33e8d:	mov    %esi,0x8(%rcx,%rax,1)
./examples/c30walk/ctor2.rs:624
   33e91:	movzwl 0x6(%rdi),%esi
./examples/c30walk/ctor2.rs:625
   33e95:	add    %r14d,%esi
   33e98:	shl    $0x8,%esi
   33e9b:	or     %r13d,%esi
   33e9e:	mov    %esi,0xc(%rcx,%rax,1)
./examples/c30walk/ctor2.rs:624
   33ea2:	movzwl 0x8(%rdi),%esi
./examples/c30walk/ctor2.rs:625
   33ea6:	add    %r14d,%esi
   33ea9:	shl    $0x8,%esi
   33eac:	or     %r13d,%esi
   33eaf:	mov    %esi,0x10(%rcx,%rax,1)
./examples/c30walk/ctor2.rs:624
   33eb3:	movzwl 0xa(%rdi),%esi
./examples/c30walk/ctor2.rs:625
   33eb7:	add    %r14d,%esi
   33eba:	shl    $0x8,%esi
   33ebd:	or     %r13d,%esi
   33ec0:	mov    %esi,0x14(%rcx,%rax,1)
./examples/c30walk/ctor2.rs:624
   33ec4:	movzwl 0xc(%rdi),%esi
./examples/c30walk/ctor2.rs:625
   33ec8:	add    %r14d,%esi
   33ecb:	shl    $0x8,%esi
   33ece:	or     %r13d,%esi
   33ed1:	mov    %esi,0x18(%rcx,%rax,1)
./examples/c30walk/ctor2.rs:624
   33ed5:	movzwl 0xe(%rdi),%esi
./examples/c30walk/ctor2.rs:625
   33ed9:	add    %r14d,%esi
   33edc:	shl    $0x8,%esi
   33edf:	or     %r13d,%esi
   33ee2:	mov    %esi,0x1c(%rcx,%rax,1)
./examples/c30walk/ctor2.rs:624
   33ee6:	movzwl 0x10(%rdi),%esi
./examples/c30walk/ctor2.rs:625
   33eea:	add    %r14d,%esi
   33eed:	shl    $0x8,%esi
   33ef0:	or     %r13d,%esi
   33ef3:	mov    %esi,0x20(%rcx,%rax,1)
./examples/c30walk/ctor2.rs:624
   33ef7:	movzwl 0x12(%rdi),%esi
./examples/c30walk/ctor2.rs:625
   33efb:	add    %r14d,%esi
   33efe:	shl    $0x8,%esi
   33f01:	or     %r13d,%esi
   33f04:	mov    %esi,0x24(%rcx,%rax,1)
./examples/c30walk/ctor2.rs:624
   33f08:	movzwl 0x14(%rdi),%esi
./examples/c30walk/ctor2.rs:625
   33f0c:	add    %r14d,%esi
   33f0f:	shl    $0x8,%esi
   33f12:	or     %r13d,%esi
   33f15:	mov    %esi,0x28(%rcx,%rax,1)
./examples/c30walk/ctor2.rs:624
   33f19:	movzwl 0x16(%rdi),%esi
./examples/c30walk/ctor2.rs:625
   33f1d:	add    %r14d,%esi
   33f20:	shl    $0x8,%esi
   33f23:	or     %r13d,%esi
   33f26:	mov    %esi,0x2c(%rcx,%rax,1)
./examples/c30walk/ctor2.rs:624
   33f2a:	movzwl 0x18(%rdi),%esi
./examples/c30walk/ctor2.rs:625
   33f2e:	add    %r14d,%esi
   33f31:	shl    $0x8,%esi
   33f34:	or     %r13d,%esi
   33f37:	mov    %esi,0x30(%rcx,%rax,1)
./examples/c30walk/ctor2.rs:624
   33f3b:	movzwl 0x1a(%rdi),%esi
./examples/c30walk/ctor2.rs:625
   33f3f:	add    %r14d,%esi
   33f42:	shl    $0x8,%esi
   33f45:	or     %r13d,%esi
   33f48:	mov    %esi,0x34(%rcx,%rax,1)
./examples/c30walk/ctor2.rs:624
   33f4c:	movzwl 0x1c(%rdi),%esi
./examples/c30walk/ctor2.rs:625
   33f50:	add    %r14d,%esi
   33f53:	shl    $0x8,%esi
   33f56:	or     %r13d,%esi
   33f59:	mov    %esi,0x38(%rcx,%rax,1)
./examples/c30walk/ctor2.rs:624
   33f5d:	movzwl 0x1e(%rdi),%esi
./examples/c30walk/ctor2.rs:625
   33f61:	add    %r14d,%esi
   33f64:	shl    $0x8,%esi
   33f67:	or     %r13d,%esi
   33f6a:	mov    %esi,0x3c(%rcx,%rax,1)
./examples/c30walk/ctor2.rs:624
   33f6e:	movzwl 0x20(%rdi),%esi
./examples/c30walk/ctor2.rs:625
   33f72:	add    %r14d,%esi
   33f75:	shl    $0x8,%esi
   33f78:	or     %r13d,%esi
   33f7b:	mov    %esi,0x40(%rcx,%rax,1)
./examples/c30walk/ctor2.rs:624
   33f7f:	movzwl 0x22(%rdi),%esi
./examples/c30walk/ctor2.rs:625
   33f83:	add    %r14d,%esi
   33f86:	shl    $0x8,%esi
   33f89:	or     %r13d,%esi
   33f8c:	mov    %esi,0x44(%rcx,%rax,1)
./examples/c30walk/ctor2.rs:624
   33f90:	movzwl 0x24(%rdi),%esi
./examples/c30walk/ctor2.rs:625
   33f94:	add    %r14d,%esi
   33f97:	shl    $0x8,%esi
   33f9a:	or     %r13d,%esi
   33f9d:	mov    %esi,0x48(%rcx,%rax,1)
./examples/c30walk/ctor2.rs:624
   33fa1:	movzwl 0x26(%rdi),%esi
./examples/c30walk/ctor2.rs:625
   33fa5:	add    %r14d,%esi
   33fa8:	shl    $0x8,%esi
   33fab:	or     %r13d,%esi
   33fae:	mov    %esi,0x4c(%rcx,%rax,1)
./examples/c30walk/ctor2.rs:624
   33fb2:	movzwl 0x28(%rdi),%esi
./examples/c30walk/ctor2.rs:625
   33fb6:	add    %r14d,%esi
   33fb9:	shl    $0x8,%esi
   33fbc:	or     %r13d,%esi
   33fbf:	mov    %esi,0x50(%rcx,%rax,1)
./examples/c30walk/ctor2.rs:624
   33fc3:	movzwl 0x2a(%rdi),%esi
./examples/c30walk/ctor2.rs:625
   33fc7:	add    %r14d,%esi
   33fca:	shl    $0x8,%esi
   33fcd:	or     %r13d,%esi
   33fd0:	mov    %esi,0x54(%rcx,%rax,1)
./examples/c30walk/ctor2.rs:624
   33fd4:	movzwl 0x2c(%rdi),%esi
./examples/c30walk/ctor2.rs:625
   33fd8:	add    %r14d,%esi
   33fdb:	shl    $0x8,%esi
   33fde:	or     %r13d,%esi
   33fe1:	mov    %esi,0x58(%rcx,%rax,1)
./examples/c30walk/ctor2.rs:624
   33fe5:	movzwl 0x2e(%rdi),%esi
./examples/c30walk/ctor2.rs:625
   33fe9:	add    %r14d,%esi
   33fec:	shl    $0x8,%esi
   33fef:	or     %r13d,%esi
   33ff2:	mov    %esi,0x5c(%rcx,%rax,1)
./examples/c30walk/ctor2.rs:624
   33ff6:	movzwl 0x30(%rdi),%esi
./examples/c30walk/ctor2.rs:625
   33ffa:	add    %r14d,%esi
   33ffd:	shl    $0x8,%esi
   34000:	or     %r13d,%esi
   34003:	mov    %esi,0x60(%rcx,%rax,1)
./examples/c30walk/ctor2.rs:624
   34007:	movzwl 0x32(%rdi),%esi
./examples/c30walk/ctor2.rs:625
   3400b:	add    %r14d,%esi
   3400e:	shl    $0x8,%esi
   34011:	or     %r13d,%esi
   34014:	mov    %esi,0x64(%rcx,%rax,1)
./examples/c30walk/ctor2.rs:624
   34018:	movzwl 0x34(%rdi),%esi
./examples/c30walk/ctor2.rs:625
   3401c:	add    %r14d,%esi
   3401f:	shl    $0x8,%esi
   34022:	or     %r13d,%esi
   34025:	mov    %esi,0x68(%rcx,%rax,1)
./examples/c30walk/ctor2.rs:624
   34029:	movzwl 0x36(%rdi),%esi
./examples/c30walk/ctor2.rs:625
   3402d:	add    %r14d,%esi
   34030:	shl    $0x8,%esi
   34033:	or     %r13d,%esi
   34036:	mov    %esi,0x6c(%rcx,%rax,1)
./examples/c30walk/ctor2.rs:624
   3403a:	movzwl 0x38(%rdi),%esi
./examples/c30walk/ctor2.rs:625
   3403e:	add    %r14d,%esi
   34041:	shl    $0x8,%esi
   34044:	or     %r13d,%esi
   34047:	mov    %esi,0x70(%rcx,%rax,1)
./examples/c30walk/ctor2.rs:624
   3404b:	movzwl 0x3a(%rdi),%esi
./examples/c30walk/ctor2.rs:625
   3404f:	add    %r14d,%esi
   34052:	shl    $0x8,%esi
   34055:	or     %r13d,%esi
   34058:	mov    %esi,0x74(%rcx,%rax,1)
./examples/c30walk/ctor2.rs:639
   3405c:	lea    (%r10,%rbp,1),%eax
   34060:	add    $0x20,%eax
   34063:	mov    %eax,0x28e0(%rsp)
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/cmp.rs:1916
   3406a:	add    $0x20,%rbp
lt():
/rustc/2d8144b78/library/core/src/cmp.rs:1916
   3406e:	inc    %r9
   34071:	add    $0x2000,%r15d
   34078:	dec    %rdx
spec_next<usize>():
/rustc/2d8144b78/library/core/src/iter/range.rs:900
   3407b:	je     3427c <<c30walk::ctor2::Ctor2>::build::<false>+0x117c>
<c30walk::ctor2::Ctor2>::build::<false>:
./examples/c30walk/ctor2.rs:285
   34081:	mov    %r9,%rax
is_raw<false>():
./examples/c30walk/ctor2.rs:285
   34084:	shr    $0x6,%rax
   34088:	cmp    (%rsp),%rax
   3408c:	jae    348eb <<c30walk::ctor2::Ctor2>::build::<false>+0x17eb>
   34092:	mov    0x10(%rsp),%rcx
   34097:	mov    (%rcx,%rax,8),%rax
complete<false>():
./examples/c30walk/ctor2.rs:620
   3409b:	bt     %r9,%rax
   3409f:	jae    340d0 <<c30walk::ctor2::Ctor2>::build::<false>+0xfd0>
   340a1:	cmp    $0x1e,%rbx
./examples/c30walk/ctor2.rs:622
   340a5:	jae    34910 <<c30walk::ctor2::Ctor2>::build::<false>+0x1810>
   340ab:	mov    (%r12,%rbp,2),%eax
   340af:	and    $0xfffff,%eax
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:1903
   340b4:	mov    0x10(%r11),%rcx
index_mut<c30walk::RawG>():
/rustc/2d8144b78/library/core/src/slice/index.rs:278
   340b8:	cmp    %rax,%rcx
   340bb:	ja     33e60 <<c30walk::ctor2::Ctor2>::build::<false>+0xd60>
   340c1:	jmp    348fe <<c30walk::ctor2::Ctor2>::build::<false>+0x17fe>
complete<false>():
   340c6:	cs nopw 0x0(%rax,%rax,1)
   340d0:	cmp    $0x1e,%rbx
./examples/c30walk/ctor2.rs:631
   340d4:	jae    34924 <<c30walk::ctor2::Ctor2>::build::<false>+0x1824>
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   340da:	movups 0x2c(%rdi),%xmm0
write<c30walk::Rec>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   340de:	movups %xmm0,-0x10(%r12,%rbp,2)
   340e4:	movdqa (%rdi),%xmm0
   340e8:	movdqa 0x10(%rdi),%xmm1
   340ed:	movdqa 0x20(%rdi),%xmm2
   340f2:	movdqa %xmm2,-0x1c(%r12,%rbp,2)
   340f9:	movdqa %xmm1,-0x2c(%r12,%rbp,2)
   34100:	movdqa %xmm0,-0x3c(%r12,%rbp,2)
   34107:	mov    %r15d,(%r12,%rbp,2)
complete<false>():
./examples/c30walk/ctor2.rs:620
   3410b:	jmp    3405c <<c30walk::ctor2::Ctor2>::build::<false>+0xf5c>
./examples/c30walk/ctor2.rs:604
   34110:	mov    0x28d8(%rsp),%rdi
   34118:	cmp    $0x1d,%rdi
   3411c:	ja     349b9 <<c30walk::ctor2::Ctor2>::build::<false>+0x18b9>
   34122:	shl    $0x6,%rdi
   34126:	add    %r15,%rdi
./examples/c30walk/ctor2.rs:605
   34129:	mov    0x28e0(%rsp),%eax
./examples/c30walk/ctor2.rs:606
   34130:	mov    0xc0(%rsp),%rcx
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   34138:	shl    $0x6,%r9
complete<false>():
./examples/c30walk/ctor2.rs:605
   3413c:	mov    %eax,%r14d
   3413f:	shl    $0x8,%r14d
   34143:	or     %r13d,%r14d
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/iter/range.rs:900
   34146:	cmp    $0x3c,%r12
spec_next<usize>():
/rustc/2d8144b78/library/core/src/iter/range.rs:900
   3414a:	jae    341b5 <<c30walk::ctor2::Ctor2>::build::<false>+0x10b5>
   3414c:	xor    %esi,%esi
   3414e:	jmp    3423a <<c30walk::ctor2::Ctor2>::build::<false>+0x113a>
run<false>():
   34153:	shr    $1,%r9
   34156:	inc    %r9
   34159:	mov    %r9,%r10
   3415c:	and    $0xfffffffffffffff8,%r10
   34160:	lea    (%rdx,%r10,2),%r8
   34164:	movd   %ecx,%xmm0
   34168:	pshuflw $0x0,%xmm0,%xmm0
next<core::slice::iter::Iter<u16>>():
/rustc/2d8144b78/library/core/src/iter/adapters/enumerate.rs:82
   3416d:	add    %r15,%r11
   34170:	add    $0x8,%r11
   34174:	xor    %ebx,%ebx
fill<false>():
   34176:	cs nopw 0x0(%rax,%rax,1)
./examples/c30walk/ctor2.rs:588
   34180:	movq   -0x8(%r11,%rbx,2),%xmm1
   34187:	movq   (%r11,%rbx,2),%xmm2
./examples/c30walk/ctor2.rs:589
   3418d:	paddw  %xmm0,%xmm1
   34191:	paddw  %xmm0,%xmm2
write<u16>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   34195:	punpcklqdq %xmm2,%xmm1
   34199:	movdqu %xmm1,(%rax,%rbx,2)
next<core::slice::iter::Iter<u16>>():
/rustc/2d8144b78/library/core/src/iter/adapters/enumerate.rs:82
   3419e:	add    $0x8,%rbx
next<u16>():
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   341a2:	cmp    %rbx,%r10
   341a5:	jne    34180 <<c30walk::ctor2::Ctor2>::build::<false>+0x1080>
   341a7:	cmp    %r10,%r9
   341aa:	jne    33cf2 <<c30walk::ctor2::Ctor2>::build::<false>+0xbf2>
   341b0:	jmp    33d19 <<c30walk::ctor2::Ctor2>::build::<false>+0xc19>
spec_next<usize>():
/rustc/2d8144b78/library/core/src/iter/range.rs:900
   341b5:	mov    %rdx,%r10
   341b8:	movabs $0xffffffffffffffe,%rsi
   341c2:	and    %rsi,%r10
   341c5:	lea    (%rcx,%r9,1),%r11
   341c9:	add    $0x7c,%r11
   341cd:	xor    %esi,%esi
<c30walk::ctor2::Ctor2>::build::<false>:
   341cf:	nop
write<c30walk::Rec>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   341d0:	movdqu 0x2c(%rdi),%xmm0
   341d5:	movdqu %xmm0,-0x50(%r11)
   341db:	movdqa (%rdi),%xmm1
   341df:	movdqa 0x10(%rdi),%xmm2
   341e4:	movdqa 0x20(%rdi),%xmm3
   341e9:	movdqa %xmm3,-0x5c(%r11)
   341ef:	movdqa %xmm2,-0x6c(%r11)
   341f5:	movdqa %xmm1,-0x7c(%r11)
   341fb:	mov    %r14d,-0x40(%r11)
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:971
   341ff:	add    $0x2,%rsi
write<c30walk::Rec>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   34203:	movdqa %xmm1,-0x3c(%r11)
   34209:	movdqa %xmm2,-0x2c(%r11)
   3420f:	movdqa %xmm3,-0x1c(%r11)
   34215:	movdqu %xmm0,-0x10(%r11)
   3421b:	lea    0x2000(%r14),%ebx
   34222:	mov    %ebx,(%r11)
complete<false>():
./examples/c30walk/ctor2.rs:609
   34225:	add    $0x4000,%r14d
spec_next<usize>():
/rustc/2d8144b78/library/core/src/iter/range.rs:900
   3422c:	sub    $0xffffffffffffff80,%r11
   34230:	cmp    %rsi,%r10
   34233:	jne    341d0 <<c30walk::ctor2::Ctor2>::build::<false>+0x10d0>
   34235:	test   $0x1,%dl
   34238:	je     3426e <<c30walk::ctor2::Ctor2>::build::<false>+0x116e>
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   3423a:	add    %r9,%rcx
   3423d:	shl    $0x6,%rsi
write<c30walk::Rec>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   34241:	movups 0x2c(%rdi),%xmm0
   34245:	movups %xmm0,0x2c(%rcx,%rsi,1)
   3424a:	movdqa (%rdi),%xmm0
   3424e:	movdqa 0x10(%rdi),%xmm1
   34253:	movdqa 0x20(%rdi),%xmm2
   34258:	movdqa %xmm2,0x20(%rcx,%rsi,1)
   3425e:	movdqa %xmm1,0x10(%rcx,%rsi,1)
   34264:	movdqa %xmm0,(%rcx,%rsi,1)
   34269:	mov    %r14d,0x3c(%rcx,%rsi,1)
complete<false>():
./examples/c30walk/ctor2.rs:611
   3426e:	mov    %edx,%ecx
   34270:	shl    $0x5,%ecx
   34273:	add    %ecx,%eax
   34275:	mov    %eax,0x28e0(%rsp)
run<false>():
   3427c:	mov    0x48(%rsp),%rdx
./examples/c30walk/ctor2.rs:672
   34281:	mov    %rdx,%rax
   34284:	shl    $0x5,%rax
   34288:	lea    (%rdx,%rdx,1),%rcx
   3428c:	sub    %rax,%rcx
   3428f:	mov    0x50(%rsp),%r12
   34294:	add    %rcx,%r12
   34297:	lea    (%rdx,%rdx,1),%eax
   3429a:	shl    $0x5,%edx
   3429d:	sub    %eax,%edx
./examples/c30walk/ctor2.rs:673
   3429f:	add    %edx,%r8d
   342a2:	mov    0x88(%rsp),%rbp
   342aa:	mov    %r8d,0x18(%rbp)
./examples/c30walk/ctor2.rs:674
   342ae:	test   %r12,%r12
   342b1:	mov    $0x88888889,%r14d
   342b7:	lea    -0x1f47e(%rip),%r15        # 14e40 <c30walk::ctor2::PATTERNS>
   342be:	je     34439 <<c30walk::ctor2::Ctor2>::build::<false>+0x1339>
open<false>():
./examples/c30walk/ctor2.rs:347
   342c4:	mov    0x1c(%rbp),%ecx
field<false>():
./examples/c30walk/ctor2.rs:291
   342c7:	mov    %r8d,%edi
   342ca:	imul   %r14,%rdi
   342ce:	mov    %rdi,%rdx
   342d1:	shr    $0x24,%rdx
   342d5:	mov    %edx,%eax
   342d7:	shl    $0x5,%eax
   342da:	sub    %edx,%eax
   342dc:	sub    %edx,%eax
open<false>():
./examples/c30walk/ctor2.rs:350
   342de:	lea    0x1e(%rax),%esi
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/cmp.rs:1077
   342e1:	cmp    %ecx,%esi
min<u32>():
/rustc/2d8144b78/library/core/src/cmp.rs:1077
   342e3:	cmovb  %esi,%ecx
<c30walk::ctor2::Ctor2>::build::<false>:
./examples/c30walk/ctor2.rs:285
   342e6:	shr    $0x2a,%rdi
is_raw<false>():
./examples/c30walk/ctor2.rs:285
   342ea:	mov    0xa8(%rsp),%rsi
   342f2:	cmp    %rsi,%rdi
   342f5:	jae    3499c <<c30walk::ctor2::Ctor2>::build::<false>+0x189c>
open<false>():
   342fb:	mov    %rdx,%rsi
   342fe:	shl    $0x6,%rsi
   34302:	add    0xc0(%rsp),%rsi
is_raw<false>():
./examples/c30walk/ctor2.rs:285
   3430a:	mov    0xa0(%rsp),%r9
   34312:	mov    (%r9,%rdi,8),%rdi
open<false>():
./examples/c30walk/ctor2.rs:352
   34316:	bt     %rdx,%rdi
   3431a:	jb     33c50 <<c30walk::ctor2::Ctor2>::build::<false>+0xb50>
   34320:	mov    $0xffffffff,%edi
   34325:	jmp    33c5c <<c30walk::ctor2::Ctor2>::build::<false>+0xb5c>
scatter<false>():
   3432a:	nopw   0x0(%rax,%rax,1)
   34330:	mov    0x8(%rsp),%r13
   34335:	jmp    34446 <<c30walk::ctor2::Ctor2>::build::<false>+0x1346>
   3433a:	nopw   0x0(%rax,%rax,1)
   34340:	add    0x28(%rsp),%r11
   34345:	xor    %r13d,%r13d
   34348:	nopl   0x0(%rax,%rax,1)
tile<false>():
./examples/c30walk/ctor2.rs:560
   34350:	movzbl (%r11,%r13,1),%esi
<c30walk::ctor2::Ctor2>::build::<false>:
./examples/c30walk/ctor2.rs:542
   34355:	lea    (%rsi,%rsi,4),%rcx
one<false>():
./examples/c30walk/ctor2.rs:542
   34359:	lea    (%r8,%rcx,8),%rbp
   3435d:	mov    (%r8,%rcx,8),%rax
   34361:	cmp    0x8(%r8,%rcx,8),%rax
   34366:	je     3437e <<c30walk::ctor2::Ctor2>::build::<false>+0x127e>
   34368:	lea    (%r14,%r13,1),%rcx
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   3436c:	mov    %cx,(%rax)
one<false>():
./examples/c30walk/ctor2.rs:547
   3436f:	addq   $0x2,0x0(%rbp)
eq<u8>():
/rustc/2d8144b78/library/core/src/ptr/non_null.rs:1714
   34374:	inc    %r13
   34377:	cmp    %r13,%rbx
next<u8>():
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   3437a:	jne    34350 <<c30walk::ctor2::Ctor2>::build::<false>+0x1250>
   3437c:	jmp    343b0 <<c30walk::ctor2::Ctor2>::build::<false>+0x12b0>
tile<false>():
   3437e:	mov    %r11,(%rsp)
one<false>():
./examples/c30walk/ctor2.rs:542
   34382:	lea    0x98(%rsp),%rdi
   3438a:	call   62330 <<c30walk::ctor2::W<false>>::refill>
   3438f:	lea    0xd8(%rsp),%r8
   34397:	test   %al,%al
   34399:	je     34667 <<c30walk::ctor2::Ctor2>::build::<false>+0x1567>
./examples/c30walk/ctor2.rs:546
   3439f:	mov    0x0(%rbp),%rax
   343a3:	mov    0x20(%rsp),%r10
   343a8:	mov    (%rsp),%r11
   343ac:	jmp    34368 <<c30walk::ctor2::Ctor2>::build::<false>+0x1268>
checked_sub():
   343ae:	xchg   %ax,%ax
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:1050
   343b0:	cmp    %r15,0x30(%rsp)
   343b5:	mov    0x8(%rsp),%r13
   343ba:	jb     348d4 <<c30walk::ctor2::Ctor2>::build::<false>+0x17d4>
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/non_null.rs:1714
   343c0:	cmp    $0xf,%r12
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   343c4:	ja     33a90 <<c30walk::ctor2::Ctor2>::build::<false>+0x990>
next<u8>():
   343ca:	add    0x28(%rsp),%r15
   343cf:	mov    %r15,%r12
   343d2:	sub    %rbx,%r12
   343d5:	add    $0x10,%r12
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   343d9:	add    %r14d,%ebx
   343dc:	add    $0x2,%ebx
tile<false>():
   343df:	nop
./examples/c30walk/ctor2.rs:565
   343e0:	movzbl (%r15),%esi
<c30walk::ctor2::Ctor2>::build::<false>:
./examples/c30walk/ctor2.rs:542
   343e4:	lea    (%rsi,%rsi,4),%rcx
one<false>():
./examples/c30walk/ctor2.rs:542
   343e8:	lea    (%r8,%rcx,8),%r14
   343ec:	mov    (%r8,%rcx,8),%rax
   343f0:	cmp    0x8(%r8,%rcx,8),%rax
   343f5:	je     3440d <<c30walk::ctor2::Ctor2>::build::<false>+0x130d>
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   343f7:	mov    %bx,(%rax)
one<false>():
./examples/c30walk/ctor2.rs:547
   343fa:	addq   $0x2,(%r14)
tile<false>():
   343fe:	inc    %r15
eq<u8>():
/rustc/2d8144b78/library/core/src/ptr/non_null.rs:1714
   34401:	inc    %ebx
   34403:	cmp    %r12,%r15
next<u8>():
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   34406:	jne    343e0 <<c30walk::ctor2::Ctor2>::build::<false>+0x12e0>
   34408:	jmp    33a90 <<c30walk::ctor2::Ctor2>::build::<false>+0x990>
one<false>():
./examples/c30walk/ctor2.rs:542
   3440d:	lea    0x98(%rsp),%rdi
   34415:	call   62330 <<c30walk::ctor2::W<false>>::refill>
   3441a:	lea    0xd8(%rsp),%r8
   34422:	test   %al,%al
   34424:	je     34667 <<c30walk::ctor2::Ctor2>::build::<false>+0x1567>
./examples/c30walk/ctor2.rs:546
   3442a:	mov    (%r14),%rax
   3442d:	mov    0x8(%rsp),%r13
   34432:	mov    0x20(%rsp),%r10
   34437:	jmp    343f7 <<c30walk::ctor2::Ctor2>::build::<false>+0x12f7>
scatter<false>():
   34439:	mov    0x8(%rsp),%r13
   3443e:	lea    0xd8(%rsp),%r8
   34446:	mov    0x20(%rsp),%r10
   3444b:	mov    0x68(%rsp),%r9
   34450:	mov    0x38(%rsp),%rcx
   34455:	mov    0x30(%rsp),%r11
   3445a:	mov    0x44(%rsp),%edx
   3445e:	mov    0x70(%rsp),%rsi
run<false>():
./examples/c30walk/ctor2.rs:664
   34463:	jmp    33ad2 <<c30walk::ctor2::Ctor2>::build::<false>+0x9d2>
tile<false>():
   34468:	nopl   0x0(%rax,%rax,1)
   34470:	lea    (%r14,%r11,1),%rax
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/non_null.rs:1714
   34474:	cmp    %r10,%rax
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   34477:	jne    344c1 <<c30walk::ctor2::Ctor2>::build::<false>+0x13c1>
tile<false>():
./examples/c30walk/ctor2.rs:570
   34479:	cmp    $0x10,%rdx
   3447d:	jne    34537 <<c30walk::ctor2::Ctor2>::build::<false>+0x1437>
<c30walk::ctor2::Ctor2>::build::<false>:
./examples/c30walk/ctor2.rs:298
   34483:	mov    0x28d8(%rsp),%rax
advance<false>():
./examples/c30walk/ctor2.rs:298
   3448b:	lea    0x10(%rax),%rcx
./examples/c30walk/ctor2.rs:299
   3448f:	cmp    $0x1d,%rcx
   34493:	mov    0x8(%rsp),%r13
   34498:	mov    0x28(%rsp),%r14
   3449d:	mov    %rbp,%r9
   344a0:	jbe    345db <<c30walk::ctor2::Ctor2>::build::<false>+0x14db>
./examples/c30walk/ctor2.rs:300
   344a6:	add    $0xfffffffffffffff2,%rax
   344aa:	mov    %rax,0x28d8(%rsp)
./examples/c30walk/ctor2.rs:301
   344b2:	mov    0x28e0(%rsp),%eax
   344b9:	add    $0x12,%eax
   344bc:	jmp    345ed <<c30walk::ctor2::Ctor2>::build::<false>+0x14ed>
tile<false>():
   344c1:	add    $0x2,%r12d
next<u8>():
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   344c5:	movzwl %r12w,%r12d
tile<false>():
   344c9:	nopl   0x0(%rax)
./examples/c30walk/ctor2.rs:565
   344d0:	movzbl (%r15,%r14,1),%esi
<c30walk::ctor2::Ctor2>::build::<false>:
./examples/c30walk/ctor2.rs:542
   344d5:	lea    (%rsi,%rsi,4),%rcx
one<false>():
./examples/c30walk/ctor2.rs:542
   344d9:	lea    (%r8,%rcx,8),%rbx
   344dd:	mov    (%r8,%rcx,8),%rax
   344e1:	cmp    0x8(%r8,%rcx,8),%rax
   344e6:	je     34500 <<c30walk::ctor2::Ctor2>::build::<false>+0x1400>
   344e8:	lea    (%r12,%r14,1),%rcx
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   344ec:	mov    %cx,(%rax)
one<false>():
./examples/c30walk/ctor2.rs:547
   344ef:	addq   $0x2,(%rbx)
eq<u8>():
/rustc/2d8144b78/library/core/src/ptr/non_null.rs:1714
   344f3:	inc    %r14
   344f6:	cmp    %r14,%rdx
next<u8>():
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   344f9:	jne    344d0 <<c30walk::ctor2::Ctor2>::build::<false>+0x13d0>
   344fb:	jmp    34479 <<c30walk::ctor2::Ctor2>::build::<false>+0x1379>
tile<false>():
   34500:	mov    %rdx,0x10(%rsp)
   34505:	mov    %r11,%r13
one<false>():
./examples/c30walk/ctor2.rs:542
   34508:	lea    0x98(%rsp),%rdi
   34510:	call   62330 <<c30walk::ctor2::W<false>>::refill>
   34515:	lea    0xd8(%rsp),%r8
   3451d:	test   %al,%al
   3451f:	je     34667 <<c30walk::ctor2::Ctor2>::build::<false>+0x1567>
./examples/c30walk/ctor2.rs:546
   34525:	mov    (%rbx),%rax
   34528:	mov    0x20(%rsp),%r10
   3452d:	mov    %r13,%r11
   34530:	mov    0x10(%rsp),%rdx
   34535:	jmp    344e8 <<c30walk::ctor2::Ctor2>::build::<false>+0x13e8>
tile<false>():
   34537:	mov    0x28e0(%rsp),%eax
   3453e:	mov    0x28d8(%rsp),%rcx
<c30walk::ctor2::Ctor2>::build::<false>:
./examples/c30walk/ctor2.rs:299
   34546:	inc    %r11
advance<false>():
./examples/c30walk/ctor2.rs:299
   34549:	cmp    %r11,%r10
   3454c:	mov    0x8(%rsp),%r13
   34551:	mov    0x28(%rsp),%r14
   34556:	mov    %rbp,%r9
   34559:	je     345b6 <<c30walk::ctor2::Ctor2>::build::<false>+0x14b6>
   3455b:	mov    %rdx,%r10
   3455e:	and    $0xffffe,%edx
   34564:	data16 data16 cs nopw 0x0(%rax,%rax,1)
./examples/c30walk/ctor2.rs:298
   34570:	lea    0x1(%rcx),%rsi
./examples/c30walk/ctor2.rs:299
   34574:	add    $0xffffffffffffffe3,%rcx
   34578:	xor    %edi,%edi
   3457a:	cmp    $0x1e,%rsi
   3457e:	setae  %dil
   34582:	cmovb  %rsi,%rcx
   34586:	lea    (%rax,%rdi,2),%eax
./examples/c30walk/ctor2.rs:298
   34589:	lea    0x1(%rcx),%rsi
./examples/c30walk/ctor2.rs:299
   3458d:	add    $0xffffffffffffffe3,%rcx
   34591:	xor    %edi,%edi
   34593:	cmp    $0x1e,%rsi
   34597:	setae  %dil
   3459b:	cmovb  %rsi,%rcx
   3459f:	lea    (%rax,%rdi,2),%eax
   345a2:	add    $0x2,%eax
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/iter/range.rs:900
   345a5:	add    $0xfffffffffffffffe,%rdx
spec_next<usize>():
/rustc/2d8144b78/library/core/src/iter/range.rs:900
   345a9:	jne    34570 <<c30walk::ctor2::Ctor2>::build::<false>+0x1470>
   345ab:	test   $0x1,%r10b
   345af:	mov    0x20(%rsp),%r10
   345b4:	je     345d1 <<c30walk::ctor2::Ctor2>::build::<false>+0x14d1>
advance<false>():
./examples/c30walk/ctor2.rs:298
   345b6:	lea    0x1(%rcx),%rdx
./examples/c30walk/ctor2.rs:299
   345ba:	add    $0xffffffffffffffe3,%rcx
   345be:	xor    %esi,%esi
   345c0:	cmp    $0x1e,%rdx
   345c4:	setae  %sil
   345c8:	cmovb  %rdx,%rcx
   345cc:	lea    (%rax,%rsi,2),%eax
   345cf:	inc    %eax
   345d1:	mov    %rcx,0x28d8(%rsp)
   345d9:	jmp    345ed <<c30walk::ctor2::Ctor2>::build::<false>+0x14ed>
./examples/c30walk/ctor2.rs:303
   345db:	mov    %rcx,0x28d8(%rsp)
./examples/c30walk/ctor2.rs:304
   345e3:	mov    0x28e0(%rsp),%eax
   345ea:	add    $0x10,%eax
<c30walk::ctor2::Ctor2>::build::<false>:
   345ed:	mov    %eax,0x28e0(%rsp)
advance<false>():
   345f4:	mov    %r10,%r11
scatter<false>():
./examples/c30walk/ctor2.rs:823
   345f7:	cmp    %r13,%r10
   345fa:	jb     338dc <<c30walk::ctor2::Ctor2>::build::<false>+0x7dc>
   34600:	jmp    3460f <<c30walk::ctor2::Ctor2>::build::<false>+0x150f>
push_mut<[u32; 256], alloc::alloc::Global>():
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:1041
   34602:	mov    %r15,%rdi
   34605:	call   a9ba0 <<alloc::raw_vec::RawVec<[u32; 256]>>::grow_one>
   3460a:	jmp    3395f <<c30walk::ctor2::Ctor2>::build::<false>+0x85f>
scatter<false>():
   3460f:	mov    $0x84,%eax
   34614:	data16 data16 cs nopw 0x0(%rax,%rax,1)
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/cmp.rs:1916
   34620:	lea    0x50(%rax),%rcx
lt():
/rustc/2d8144b78/library/core/src/cmp.rs:1916
   34624:	cmp    $0x28d4,%rcx
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/iter/range.rs:900
   3462b:	je     3468b <<c30walk::ctor2::Ctor2>::build::<false>+0x158b>
./examples/c30walk/ctor2.rs:247
   3462d:	mov    0x54(%rsp,%rax,1),%rdx
rank():
./examples/c30walk/ctor2.rs:247
   34632:	sub    0x64(%rsp,%rax,1),%rdx
   34637:	shr    $1,%rdx
   3463a:	add    0x6c(%rsp,%rax,1),%edx
scatter<false>():
./examples/c30walk/ctor2.rs:866
   3463e:	cmp    0x70(%rsp,%rax,1),%edx
   34642:	jne    34667 <<c30walk::ctor2::Ctor2>::build::<false>+0x1567>
rank():
./examples/c30walk/ctor2.rs:247
   34644:	mov    0x7c(%rsp,%rax,1),%rdx
   34649:	sub    0x8c(%rsp,%rax,1),%rdx
   34651:	shr    $1,%rdx
   34654:	add    0x94(%rsp,%rax,1),%edx
scatter<false>():
./examples/c30walk/ctor2.rs:866
   3465b:	cmp    0x98(%rsp,%rax,1),%edx
   34662:	mov    %rcx,%rax
   34665:	je     34620 <<c30walk::ctor2::Ctor2>::build::<false>+0x1520>
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:3035
   34667:	mov    0x5520(%rsp),%rax
clear<c30walk::RawG, alloc::alloc::Global>():
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:3035
   3466f:	movq   $0x0,0x10(%rax)
<c30walk::ctor2::Ctor2>::build::<false>:
   34677:	xor    %eax,%eax
./examples/c30walk/ctor2.rs:789
   34679:	add    $0x54e8,%rsp
   34680:	pop    %rbx
   34681:	pop    %r12
   34683:	pop    %r13
   34685:	pop    %r14
   34687:	pop    %r15
   34689:	pop    %rbp
   3468a:	ret
   3468b:	xor    %r14d,%r14d
   3468e:	lea    0x98(%rsp),%r15
   34696:	mov    0x78(%rsp),%r12
   3469b:	mov    0x8(%rsp),%r13
   346a0:	mov    0x60(%rsp),%rbp
   346a5:	jmp    346db <<c30walk::ctor2::Ctor2>::build::<false>+0x15db>
   346a7:	nopw   0x0(%rax,%rax,1)
./examples/c30walk/ctor2.rs:247
   346b0:	sub    %rcx,%rax
scatter<false>():
   346b3:	inc    %r14
rank():
./examples/c30walk/ctor2.rs:247
   346b6:	shr    $1,%rax
<c30walk::ctor2::Ctor2>::build::<false>:
./examples/c30walk/ctor2.rs:377
   346b9:	add    %eax,0x18(%r8)
close<false>():
./examples/c30walk/ctor2.rs:379
   346bd:	pxor   %xmm0,%xmm0
   346c1:	movdqu %xmm0,(%r8)
   346c6:	movq   $0x0,0x10(%r8)
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/cmp.rs:1916
   346ce:	add    $0x28,%r8
lt():
/rustc/2d8144b78/library/core/src/cmp.rs:1916
   346d2:	cmp    $0x100,%r14
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/iter/range.rs:900
   346d9:	je     34701 <<c30walk::ctor2::Ctor2>::build::<false>+0x1601>
close<false>():
./examples/c30walk/ctor2.rs:372
   346db:	mov    (%r8),%rcx
   346de:	mov    %rcx,%rax
./examples/c30walk/ctor2.rs:373
   346e1:	cmp    0x10(%r8),%rcx
   346e5:	je     346b0 <<c30walk::ctor2::Ctor2>::build::<false>+0x15b0>
./examples/c30walk/ctor2.rs:374
   346e7:	mov    %r15,%rdi
   346ea:	mov    %r14,%rsi
   346ed:	mov    %r8,%rbx
   346f0:	call   61d30 <<c30walk::ctor2::W<false>>::finish>
   346f5:	mov    %rbx,%r8
rank():
./examples/c30walk/ctor2.rs:247
   346f8:	mov    (%rbx),%rax
   346fb:	mov    0x10(%rbx),%rcx
   346ff:	jmp    346b0 <<c30walk::ctor2::Ctor2>::build::<false>+0x15b0>
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:2472
   34701:	xor    %ecx,%ecx
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:614
   34703:	mov    0x8(%r12),%rbx
saturating_sub():
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:2472
   34708:	test   %rbp,%rbp
   3470b:	setne  %al
<c30walk::ctor2::Ctor2>::build::<false>:
./examples/c30walk/ctor2.rs:766
   3470e:	je     34752 <<c30walk::ctor2::Ctor2>::build::<false>+0x1652>
   34710:	mov    %al,%cl
   34712:	mov    %rbp,%rax
   34715:	sub    %rcx,%rax
   34718:	imul   $0xffffffffffffffe2,%rax,%rcx
   3471c:	add    %r13,%rcx
   3471f:	cmp    $0x1e,%rcx
   34723:	jae    34752 <<c30walk::ctor2::Ctor2>::build::<false>+0x1652>
/rustc/2d8144b78/library/core/src/iter/range.rs:900
   34725:	lea    (%rbx,%rbp,4),%rcx
spec_next<usize>():
/rustc/2d8144b78/library/core/src/iter/range.rs:900
   34729:	lea    (%rcx,%r13,2),%rdi
   3472d:	add    $0xfffffffffffffffc,%rdi
   34731:	lea    (%rax,%rax,1),%rcx
   34735:	shl    $0x5,%rax
   34739:	sub    %rcx,%rax
   3473c:	sub    %r13,%rax
   3473f:	lea    0x3c(,%rax,2),%rdx
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   34747:	mov    $0xff,%esi
write<u16>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   3474c:	call   *0xc253e(%rip)        # f6c90 <memset@GLIBC_2.2.5>
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   34752:	shl    $0x6,%rbp
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   34756:	pcmpeqd %xmm0,%xmm0
write<c30walk::Rec>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   3475a:	movdqu %xmm0,0x2c(%rbx,%rbp,1)
   34760:	movdqa %xmm0,0x20(%rbx,%rbp,1)
   34766:	movdqa %xmm0,0x10(%rbx,%rbp,1)
   3476c:	movdqa %xmm0,(%rbx,%rbp,1)
   34771:	movl   $0x100fffff,0x3c(%rbx,%rbp,1)
   34779:	mov    0x58(%rsp),%rax
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:614
   3477e:	mov    0x8(%rax),%rdx
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:1864
   34782:	mov    0x10(%rax),%rcx
as_slice<u64, alloc::alloc::Global>():
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:1864
   34786:	mov    0x5520(%rsp),%rax
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:614
   3478e:	mov    0x8(%rax),%r8
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:1903
   34792:	mov    0x10(%rax),%r9
as_mut_slice<c30walk::RawG, alloc::alloc::Global>():
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:1903
   34796:	lea    0x98(%rsp),%rdi
<c30walk::ctor2::Ctor2>::build::<false>:
./examples/c30walk/ctor2.rs:776
   3479e:	mov    %rbx,%rsi
   347a1:	call   6dd80 <c30walk::ctor::canonicalise>
   347a6:	mov    0x80(%rsp),%rax
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:2221
   347ae:	mov    %rax,0x10(%r12)
set_len<c30walk::Rec, alloc::alloc::Global>():
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:2221
   347b3:	mov    $0x1,%al
<c30walk::ctor2::Ctor2>::build::<false>:
./examples/c30walk/ctor2.rs:789
   347b5:	jmp    34679 <<c30walk::ctor2::Ctor2>::build::<false>+0x1579>
/rustc/2d8144b78/library/core/src/slice/index.rs:443
   347ba:	lea    0xbf687(%rip),%rcx        # f3e48 <__frame_dummy_init_array_entry+0x23b8>
index<u8>():
/rustc/2d8144b78/library/core/src/slice/index.rs:443
   347c1:	mov    %r12,%rdi
   347c4:	mov    %r13,%rdx
   347c7:	call   9bd80 <core::slice::index::slice_index_fail>
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/slice/index.rs:443
   347cc:	lea    0xbe81d(%rip),%rcx        # f2ff0 <__frame_dummy_init_array_entry+0x1560>
index<u16>():
/rustc/2d8144b78/library/core/src/slice/index.rs:443
   347d3:	mov    $0x1e,%edx
   347d8:	xor    %edi,%edi
   347da:	call   9bd80 <core::slice::index::slice_index_fail>
<c30walk::ctor2::Ctor2>::build::<false>:
   347df:	mov    %rdi,%r12
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:634
   347e2:	test   %rax,%rax
current_memory<alloc::alloc::Global>():
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:634
   347e5:	je     347f7 <<c30walk::ctor2::Ctor2>::build::<false>+0x16f7>
<c30walk::ctor2::Ctor2>::build::<false>:
./examples/c30walk/ctor2.rs:715
   347e7:	mov    0x8(%r9),%rdi
   347eb:	mov    %r9,%r14
<std::alloc::System as core::alloc::global::GlobalAlloc>::dealloc:
/rustc/2d8144b78/library/std/src/sys/alloc/unix.rs:48
   347ee:	call   *0xc24ac(%rip)        # f6ca0 <free@GLIBC_2.2.5>
   347f4:	mov    %r14,%r9
<c30walk::ctor2::Ctor2>::build::<false>:
./examples/c30walk/ctor2.rs:715
   347f7:	movq   $0x0,(%r9)
   347fe:	movq   $0x40,0x8(%r9)
   34806:	movq   $0x0,0x10(%r9)
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:728
   3480e:	mov    $0x40,%edx
try_reserve_exact<alloc::alloc::Global>():
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:728
   34813:	mov    %r9,%rdi
   34816:	lea    0x1(%rbp),%rsi
   3481a:	mov    %r9,%r14
   3481d:	call   65440 <<alloc::raw_vec::RawVecInner>::grow_exact>
   34822:	mov    %r14,%r9
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/result.rs:2173
   34825:	cmp    $0xffffffffffffffff,%rax
branch<(), alloc::collections::TryReserveError>():
/rustc/2d8144b78/library/core/src/result.rs:2173
   34829:	mov    %r12,%rdi
   3482c:	je     33325 <<c30walk::ctor2::Ctor2>::build::<false>+0x225>
   34832:	jmp    3488d <<c30walk::ctor2::Ctor2>::build::<false>+0x178d>
<c30walk::ctor2::Ctor2>::build::<false>:
   34834:	mov    %rdi,%r12
   34837:	mov    %r9,%r14
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:634
   3483a:	test   %rax,%rax
current_memory<alloc::alloc::Global>():
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:634
   3483d:	je     34851 <<c30walk::ctor2::Ctor2>::build::<false>+0x1751>
<c30walk::ctor2::Ctor2>::build::<false>:
./examples/c30walk/ctor2.rs:719
   3483f:	mov    0x5520(%rsp),%rax
   34847:	mov    0x8(%rax),%rdi
<std::alloc::System as core::alloc::global::GlobalAlloc>::dealloc:
/rustc/2d8144b78/library/std/src/sys/alloc/unix.rs:48
   3484b:	call   *0xc244f(%rip)        # f6ca0 <free@GLIBC_2.2.5>
<c30walk::ctor2::Ctor2>::build::<false>:
   34851:	mov    0x5520(%rsp),%rdi
./examples/c30walk/ctor2.rs:719
   34859:	movq   $0x0,(%rdi)
   34860:	movq   $0x40,0x8(%rdi)
   34868:	movq   $0x0,0x10(%rdi)
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:728
   34870:	mov    $0x80,%edx
try_reserve_exact<alloc::alloc::Global>():
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:728
   34875:	mov    %rbx,%rsi
   34878:	call   65440 <<alloc::raw_vec::RawVecInner>::grow_exact>
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/result.rs:2173
   3487d:	cmp    $0xffffffffffffffff,%rax
branch<(), alloc::collections::TryReserveError>():
/rustc/2d8144b78/library/core/src/result.rs:2173
   34881:	mov    %r14,%r9
   34884:	mov    %r12,%rdi
   34887:	je     33339 <<c30walk::ctor2::Ctor2>::build::<false>+0x239>
<c30walk::ctor2::Ctor2>::build::<false>:
   3488d:	mov    %rax,%rdi
   34890:	mov    %rdx,%rsi
   34893:	call   9950b <alloc::raw_vec::handle_error>
/rustc/2d8144b78/library/core/src/slice/index.rs:443
   34898:	lea    0xbf5a9(%rip),%rcx        # f3e48 <__frame_dummy_init_array_entry+0x23b8>
index<u8>():
/rustc/2d8144b78/library/core/src/slice/index.rs:443
   3489f:	mov    %r11,%rdi
   348a2:	mov    %r13,%rdx
   348a5:	call   9bd80 <core::slice::index::slice_index_fail>
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/slice/index.rs:443
   348aa:	lea    0xbe76f(%rip),%rcx        # f3020 <__frame_dummy_init_array_entry+0x1590>
index<u8>():
/rustc/2d8144b78/library/core/src/slice/index.rs:443
   348b1:	mov    %r11,%rdi
   348b4:	mov    %r15,%rsi
   348b7:	mov    %r13,%rdx
   348ba:	call   9bd80 <core::slice::index::slice_index_fail>
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/slice/index.rs:443
   348bf:	lea    0xbf582(%rip),%rcx        # f3e48 <__frame_dummy_init_array_entry+0x23b8>
index<u8>():
/rustc/2d8144b78/library/core/src/slice/index.rs:443
   348c6:	mov    %rsi,%rdi
   348c9:	mov    %rax,%rsi
   348cc:	mov    %r13,%rdx
   348cf:	call   9bd80 <core::slice::index::slice_index_fail>
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/core/src/slice/index.rs:443
   348d4:	lea    0xbe72d(%rip),%rcx        # f3008 <__frame_dummy_init_array_entry+0x1578>
index<u8>():
/rustc/2d8144b78/library/core/src/slice/index.rs:443
   348db:	mov    %r15,%rdi
   348de:	mov    0x30(%rsp),%rsi
   348e3:	mov    %r13,%rdx
   348e6:	call   9bd80 <core::slice::index::slice_index_fail>
is_raw<false>():
./examples/c30walk/ctor2.rs:285
   348eb:	lea    0xbd3de(%rip),%rdx        # f1cd0 <__frame_dummy_init_array_entry+0x240>
   348f2:	mov    %rax,%rdi
   348f5:	mov    (%rsp),%rsi
   348f9:	call   9afa3 <core::panicking::panic_bounds_check>
index_mut<c30walk::RawG>():
/rustc/2d8144b78/library/core/src/slice/index.rs:278
   348fe:	lea    0xbe7ab(%rip),%rdx        # f30b0 <__frame_dummy_init_array_entry+0x1620>
   34905:	mov    %rax,%rdi
   34908:	mov    %rcx,%rsi
   3490b:	call   9afa3 <core::panicking::panic_bounds_check>
complete<false>():
./examples/c30walk/ctor2.rs:622
   34910:	lea    0xbe781(%rip),%rdx        # f3098 <__frame_dummy_init_array_entry+0x1608>
   34917:	mov    $0x1e,%esi
   3491c:	mov    %rbx,%rdi
   3491f:	call   9afa3 <core::panicking::panic_bounds_check>
./examples/c30walk/ctor2.rs:631
   34924:	lea    0xbe755(%rip),%rdx        # f3080 <__frame_dummy_init_array_entry+0x15f0>
   3492b:	mov    $0x1e,%esi
   34930:	mov    %rbx,%rdi
   34933:	call   9afa3 <core::panicking::panic_bounds_check>
reserve<alloc::alloc::Global>():
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:675
   34938:	mov    $0x10,%edx
   3493d:	mov    $0x4,%ecx
   34942:	mov    $0x400,%r8d
   34948:	mov    %rdi,%r14
   3494b:	mov    %r12,%rdi
   3494e:	xor    %esi,%esi
   34950:	mov    %r9,%rbx
   34953:	call   44510 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   34958:	mov    %r14,%rdi
   3495b:	mov    %rbx,%r9
   3495e:	jmp    333d0 <<c30walk::ctor2::Ctor2>::build::<false>+0x2d0>
<c30walk::ctor2::Ctor2>::build::<false>:
   34963:	mov    %r9,%r13
reserve<alloc::alloc::Global>():
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:675
   34966:	mov    $0x8,%ecx
   3496b:	mov    $0x8,%r8d
   34971:	mov    %rdi,%r14
   34974:	xor    %esi,%esi
   34976:	mov    %rbx,%rdx
   34979:	call   44510 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   3497e:	mov    %r14,%rdi
<c30walk::ctor2::Ctor2>::build::<false>:
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:3054
   34981:	mov    0x10(%r14),%r14
non_null<alloc::alloc::Global, u64>():
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:614
   34985:	mov    0x8(%rdi),%r12
add<u64>():
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   34989:	lea    (%r12,%r14,8),%rdi
lt():
/rustc/2d8144b78/library/core/src/cmp.rs:1916
   3498d:	cmp    $0x1,%rbx
spec_next<usize>():
/rustc/2d8144b78/library/core/src/iter/range.rs:900
   34991:	jne    33377 <<c30walk::ctor2::Ctor2>::build::<false>+0x277>
   34997:	jmp    33399 <<c30walk::ctor2::Ctor2>::build::<false>+0x299>
is_raw<false>():
./examples/c30walk/ctor2.rs:285
   3499c:	lea    0xbd32d(%rip),%rdx        # f1cd0 <__frame_dummy_init_array_entry+0x240>
   349a3:	call   9afa3 <core::panicking::panic_bounds_check>
fill<false>():
./examples/c30walk/ctor2.rs:584
   349a8:	lea    0xbe629(%rip),%rdx        # f2fd8 <__frame_dummy_init_array_entry+0x1548>
   349af:	mov    $0x1e,%esi
   349b4:	call   9afa3 <core::panicking::panic_bounds_check>
complete<false>():
./examples/c30walk/ctor2.rs:604
   349b9:	lea    0xbe6a8(%rip),%rdx        # f3068 <__frame_dummy_init_array_entry+0x15d8>
   349c0:	mov    $0x1e,%esi
   349c5:	call   9afa3 <core::panicking::panic_bounds_check>
create_raw<false>():
./examples/c30walk/ctor2.rs:317
   349ca:	lea    0xbe58f(%rip),%rdx        # f2f60 <__frame_dummy_init_array_entry+0x14d0>
   349d1:	call   9afa3 <core::panicking::panic_bounds_check>
is_raw<false>():
./examples/c30walk/ctor2.rs:285
   349d6:	lea    0xbd2f3(%rip),%rdx        # f1cd0 <__frame_dummy_init_array_entry+0x240>
   349dd:	mov    %rbp,%rdi
   349e0:	call   9afa3 <core::panicking::panic_bounds_check>
