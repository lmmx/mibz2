# <c30walk::ctor2d::W<false>>::refill
# size 0x11f

target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

0000000000094330 <<c30walk::ctor2d::W<false>>::refill>:
   94330:	push   %r15
   94332:	push   %r14
   94334:	push   %r12
   94336:	push   %rbx
   94337:	push   %rax
   94338:	mov    %rsi,%rbx
   9433b:	mov    %rdi,%r14
   9433e:	mov    0x40(%rdi,%rsi,8),%r15
   94343:	lea    (%rsi,%rsi,2),%rax
   94347:	lea    (%rdi,%rax,8),%r12
   9434b:	add    $0x1040,%r12
   94352:	sub    0x1040(%rdi,%rax,8),%r15
   9435a:	je     94367 <<c30walk::ctor2d::W<false>>::refill+0x37>
   9435c:	mov    %r14,%rdi
   9435f:	mov    %rbx,%rsi
   94362:	call   93d20 <<c30walk::ctor2d::W<false>>::finish>
   94367:	shr    $1,%r15
   9436a:	add    0x8(%r12),%r15d
   9436f:	mov    %r15d,0x8(%r12)
   94374:	movq   $0x0,(%r12)
   9437c:	movq   $0x0,0x40(%r14,%rbx,8)
   94385:	movq   $0x0,0x840(%r14,%rbx,8)
   94391:	mov    0xc(%r12),%eax
   94396:	cmp    %eax,%r15d
   94399:	jae    94431 <<c30walk::ctor2d::W<false>>::refill+0x101>
   9439f:	mov    %r15d,%ecx
   943a2:	mov    $0x88888889,%edi
   943a7:	imul   %rcx,%rdi
   943ab:	mov    %rdi,%rcx
   943ae:	shr    $0x24,%rcx
   943b2:	mov    %ecx,%r9d
   943b5:	shl    $0x5,%r9d
   943b9:	sub    %ecx,%r9d
   943bc:	sub    %ecx,%r9d
   943bf:	lea    0x1e(%r9),%edx
   943c3:	cmp    %eax,%edx
   943c5:	cmovae %eax,%edx
   943c8:	shr    $0x2a,%rdi
   943cc:	mov    0x10(%r14),%rsi
   943d0:	cmp    %rsi,%rdi
   943d3:	jae    94443 <<c30walk::ctor2d::W<false>>::refill+0x113>
   943d5:	mov    %r15d,%esi
   943d8:	mov    %rcx,%r8
   943db:	shl    $0x6,%r8
   943df:	add    0x28(%r14),%r8
   943e3:	sub    %r9d,%esi
   943e6:	mov    0x8(%r14),%r9
   943ea:	mov    (%r9,%rdi,8),%rdi
   943ee:	bt     %rcx,%rdi
   943f2:	jae    943ff <<c30walk::ctor2d::W<false>>::refill+0xcf>
   943f4:	mov    $0xfffff,%edi
   943f9:	and    0x3c(%r8),%edi
   943fd:	jmp    94404 <<c30walk::ctor2d::W<false>>::refill+0xd4>
   943ff:	mov    $0xffffffff,%edi
   94404:	mov    %esi,%esi
   94406:	lea    (%r8,%rsi,2),%rsi
   9440a:	mov    %rsi,0x40(%r14,%rbx,8)
   9440f:	sub    %r15d,%edx
   94412:	lea    (%rsi,%rdx,2),%rdx
   94416:	mov    %rdx,0x840(%r14,%rbx,8)
   9441e:	mov    %rsi,(%r12)
   94422:	mov    %r15d,0x8(%r12)
   94427:	mov    %ecx,0x10(%r12)
   9442c:	mov    %edi,0x14(%r12)
   94431:	cmp    %eax,%r15d
   94434:	setb   %al
   94437:	add    $0x8,%rsp
   9443b:	pop    %rbx
   9443c:	pop    %r12
   9443e:	pop    %r14
   94440:	pop    %r15
   94442:	ret
   94443:	lea    0x9b8ee(%rip),%rdx        # 12fd38 <__frame_dummy_init_array_entry+0x218>
   9444a:	call   d9103 <core::panicking::panic_bounds_check>
