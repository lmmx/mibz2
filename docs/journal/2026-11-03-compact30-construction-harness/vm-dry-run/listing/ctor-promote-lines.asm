
./target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

00000000000585e0 <<c30walk::ctor::W<false>>::promote>:
<c30walk::ctor::W<false>>::promote:
./examples/c30walk/ctor.rs:383
   585e0:	push   %rbp
   585e1:	push   %r15
   585e3:	push   %r14
   585e5:	push   %r13
   585e7:	push   %r12
   585e9:	push   %rbx
   585ea:	sub    $0x18,%rsp
./examples/c30walk/ctor.rs:389
   585ee:	mov    (%rdi),%rbx
./examples/c30walk/ctor.rs:344
   585f1:	mov    0x20(%rdi),%r14
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mut_ptr.rs:961
   585f5:	mov    %rsi,%r15
add<c30walk::Rec>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mut_ptr.rs:961
   585f8:	shl    $0x6,%r15
<c30walk::ctor::W<false>>::promote:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/mod.rs:3054
   585fc:	mov    0x10(%rbx),%rax
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/mod.rs:1040
   58600:	cmp    (%rbx),%rax
push_mut<c30walk::RawG, alloc::alloc::Global>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/mod.rs:1040
   58603:	je     587bb <<c30walk::ctor::W<false>>::promote+0x1db>
<c30walk::ctor::W<false>>::promote:
   58609:	lea    (%r14,%r15,1),%r8
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/raw_vec/mod.rs:614
   5860d:	mov    0x8(%rbx),%r9
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mut_ptr.rs:961
   58611:	mov    %rax,%r10
add<c30walk::RawG>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mut_ptr.rs:961
   58614:	shl    $0x7,%r10
<c30walk::ctor::W<false>>::promote:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1933
   58618:	pxor   %xmm0,%xmm0
write<c30walk::RawG>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1933
   5861c:	movdqa %xmm0,0x70(%r9,%r10,1)
   58623:	movdqa %xmm0,0x60(%r9,%r10,1)
   5862a:	movdqa %xmm0,0x50(%r9,%r10,1)
   58631:	movdqa %xmm0,0x40(%r9,%r10,1)
   58638:	movdqa %xmm0,0x30(%r9,%r10,1)
   5863f:	movdqa %xmm0,0x20(%r9,%r10,1)
   58646:	movdqa %xmm0,0x10(%r9,%r10,1)
   5864d:	movdqa %xmm0,(%r9,%r10,1)
push_mut<c30walk::RawG, alloc::alloc::Global>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/mod.rs:1046
   58653:	mov    %rax,%r9
   58656:	inc    %r9
   58659:	mov    %r9,0x10(%rbx)
<c30walk::ctor::W<false>>::promote:
./examples/c30walk/ctor.rs:394
   5865d:	mov    %eax,%r12d
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/raw_vec/mod.rs:614
   58660:	mov    0x8(%rbx),%rbx
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/index.rs:278
   58664:	shl    $0x7,%r12
index_mut<c30walk::RawG>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/index.rs:278
   58668:	lea    (%rbx,%r12,1),%r9
<c30walk::ctor::W<false>>::promote:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mut_ptr.rs:961
   5866c:	lea    (%r9,%rdx,4),%r10
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/iter/macros.rs:180
   58670:	lea    -0x4(,%rdx,4),%r11
next<u32>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/iter/macros.rs:180
   58678:	cmp    $0x3c,%r11
   5867c:	jb     586a9 <<c30walk::ctor::W<false>>::promote+0xc9>
<c30walk::ctor::W<false>>::promote:
./examples/c30walk/ctor.rs:395
   5867e:	shl    $0x2,%rdx
next<u32>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/iter/macros.rs:180
   58682:	add    $0xfffffffffffffffc,%rdx
   58686:	shr    $1,%rdx
   58689:	add    %r14,%rdx
   5868c:	add    %r15,%rdx
   5868f:	add    $0x2,%rdx
   58693:	cmp    %rdx,%r9
   58696:	setb   %dl
   58699:	cmp    %r10,%r8
   5869c:	setb   %bpl
<c30walk::ctor::W<false>>::promote:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/adapters/enumerate.rs:82
   586a0:	test   %bpl,%dl
next<core::slice::iter::IterMut<u32>>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/adapters/enumerate.rs:82
   586a3:	je     58735 <<c30walk::ctor::W<false>>::promote+0x155>
   586a9:	xor    %edx,%edx
next<u32>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/iter/macros.rs:180
   586ab:	lea    (%r15,%rdx,2),%rdx
   586af:	add    %rdx,%r14
<c30walk::ctor::W<false>>::promote:
   586b2:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
read<u16>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1733
   586c0:	movzwl (%r14),%edx
<c30walk::ctor::W<false>>::promote:
./examples/c30walk/ctor.rs:396
   586c4:	shl    $0x8,%edx
   586c7:	add    %ecx,%edx
   586c9:	mov    %edx,(%r9)
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/non_null.rs:656
   586cc:	add    $0x4,%r9
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/non_null.rs:1714
   586d0:	add    $0x2,%r14
eq<u32>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/non_null.rs:1714
   586d4:	cmp    %r10,%r9
next<u32>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/iter/macros.rs:180
   586d7:	jne    586c0 <<c30walk::ctor::W<false>>::promote+0xe0>
<c30walk::ctor::W<false>>::promote:
./examples/c30walk/ctor.rs:401
   586d9:	mov    %eax,%ecx
   586db:	or     $0x10000000,%ecx
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1933
   586e1:	pcmpeqd %xmm0,%xmm0
write<c30walk::Rec>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1933
   586e5:	movdqu %xmm0,0x2c(%r8)
   586eb:	movdqa %xmm0,0x20(%r8)
   586f1:	movdqa %xmm0,0x10(%r8)
   586f7:	movdqa %xmm0,(%r8)
   586fc:	mov    %ecx,0x3c(%r8)
<c30walk::ctor::W<false>>::promote:
./examples/c30walk/ctor.rs:403
   58700:	mov    %rsi,%r8
   58703:	shr    $0x6,%r8
   58707:	mov    0x10(%rdi),%rcx
   5870b:	cmp    %rcx,%r8
   5870e:	jae    587ec <<c30walk::ctor::W<false>>::promote+0x20c>
   58714:	mov    $0x1,%edx
   58719:	mov    %esi,%ecx
   5871b:	shl    %cl,%rdx
   5871e:	mov    0x8(%rdi),%rcx
   58722:	or     %rdx,(%rcx,%r8,8)
./examples/c30walk/ctor.rs:414
   58726:	add    $0x18,%rsp
   5872a:	pop    %rbx
   5872b:	pop    %r12
   5872d:	pop    %r13
   5872f:	pop    %r14
   58731:	pop    %r15
   58733:	pop    %rbp
   58734:	ret
   58735:	shr    $0x2,%r11
   58739:	inc    %r11
   5873c:	mov    %r11,%rdx
   5873f:	and    $0xfffffffffffffff8,%rdx
   58743:	lea    (%r9,%rdx,4),%r9
   58747:	movd   %ecx,%xmm0
   5874b:	pshufd $0x0,%xmm0,%xmm0
next<core::slice::iter::IterMut<u32>>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/adapters/enumerate.rs:82
   58750:	add    %r12,%rbx
   58753:	add    $0x10,%rbx
   58757:	lea    (%r15,%r14,1),%r12
   5875b:	add    $0x8,%r12
   5875f:	xor    %r13d,%r13d
   58762:	pxor   %xmm1,%xmm1
read<u16>():
   58766:	cs nopw 0x0(%rax,%rax,1)
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1733
   58770:	movq   -0x8(%r12,%r13,2),%xmm2
   58777:	movq   (%r12,%r13,2),%xmm3
<c30walk::ctor::W<false>>::promote:
./examples/c30walk/ctor.rs:396
   5877d:	punpcklwd %xmm1,%xmm2
   58781:	punpcklwd %xmm1,%xmm3
   58785:	pslld  $0x8,%xmm2
   5878a:	pslld  $0x8,%xmm3
   5878f:	paddd  %xmm0,%xmm2
   58793:	paddd  %xmm0,%xmm3
   58797:	movdqu %xmm2,-0x10(%rbx,%r13,4)
   5879e:	movdqu %xmm3,(%rbx,%r13,4)
next<core::slice::iter::IterMut<u32>>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/adapters/enumerate.rs:82
   587a4:	add    $0x8,%r13
next<u32>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/iter/macros.rs:180
   587a8:	cmp    %r13,%rdx
   587ab:	jne    58770 <<c30walk::ctor::W<false>>::promote+0x190>
   587ad:	cmp    %rdx,%r11
   587b0:	je     586d9 <<c30walk::ctor::W<false>>::promote+0xf9>
   587b6:	jmp    586ab <<c30walk::ctor::W<false>>::promote+0xcb>
<c30walk::ctor::W<false>>::promote:
   587bb:	mov    %rdi,0x10(%rsp)
push_mut<c30walk::RawG, alloc::alloc::Global>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/mod.rs:1041
   587c0:	mov    %rbx,%rdi
   587c3:	mov    %rsi,%r13
   587c6:	mov    %rax,%r12
   587c9:	mov    %ecx,0xc(%rsp)
   587cd:	mov    %rdx,%rbp
   587d0:	call   57bc0 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   587d5:	mov    %rbp,%rdx
   587d8:	mov    0xc(%rsp),%ecx
   587dc:	mov    %r12,%rax
   587df:	mov    %r13,%rsi
   587e2:	mov    0x10(%rsp),%rdi
   587e7:	jmp    58609 <<c30walk::ctor::W<false>>::promote+0x29>
<c30walk::ctor::W<false>>::promote:
./examples/c30walk/ctor.rs:403
   587ec:	lea    0x7a8d5(%rip),%rdx        # d30c8 <__frame_dummy_init_array_entry+0x1718>
   587f3:	mov    %r8,%rdi
   587f6:	mov    %rcx,%rsi
   587f9:	call   7b81b <core::panicking::panic_bounds_check>
