# <c30walk::ctor2d::W<false>>::refill
# size 0x11f

target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

0000000000095290 <<c30walk::ctor2d::W<false>>::refill>:
   95290:	push   %r15
   95292:	push   %r14
   95294:	push   %r12
   95296:	push   %rbx
   95297:	push   %rax
   95298:	mov    %rsi,%rbx
   9529b:	mov    %rdi,%r14
   9529e:	mov    0x40(%rdi,%rsi,8),%r15
   952a3:	lea    (%rsi,%rsi,2),%rax
   952a7:	lea    (%rdi,%rax,8),%r12
   952ab:	add    $0x1040,%r12
   952b2:	sub    0x1040(%rdi,%rax,8),%r15
   952ba:	je     952c7 <<c30walk::ctor2d::W<false>>::refill+0x37>
   952bc:	mov    %r14,%rdi
   952bf:	mov    %rbx,%rsi
   952c2:	call   94c80 <<c30walk::ctor2d::W<false>>::finish>
   952c7:	shr    $1,%r15
   952ca:	add    0x8(%r12),%r15d
   952cf:	mov    %r15d,0x8(%r12)
   952d4:	movq   $0x0,(%r12)
   952dc:	movq   $0x0,0x40(%r14,%rbx,8)
   952e5:	movq   $0x0,0x840(%r14,%rbx,8)
   952f1:	mov    0xc(%r12),%eax
   952f6:	cmp    %eax,%r15d
   952f9:	jae    95391 <<c30walk::ctor2d::W<false>>::refill+0x101>
   952ff:	mov    %r15d,%ecx
   95302:	mov    $0x88888889,%edi
   95307:	imul   %rcx,%rdi
   9530b:	mov    %rdi,%rcx
   9530e:	shr    $0x24,%rcx
   95312:	mov    %ecx,%r9d
   95315:	shl    $0x5,%r9d
   95319:	sub    %ecx,%r9d
   9531c:	sub    %ecx,%r9d
   9531f:	lea    0x1e(%r9),%edx
   95323:	cmp    %eax,%edx
   95325:	cmovae %eax,%edx
   95328:	shr    $0x2a,%rdi
   9532c:	mov    0x10(%r14),%rsi
   95330:	cmp    %rsi,%rdi
   95333:	jae    953a3 <<c30walk::ctor2d::W<false>>::refill+0x113>
   95335:	mov    %r15d,%esi
   95338:	mov    %rcx,%r8
   9533b:	shl    $0x6,%r8
   9533f:	add    0x28(%r14),%r8
   95343:	sub    %r9d,%esi
   95346:	mov    0x8(%r14),%r9
   9534a:	mov    (%r9,%rdi,8),%rdi
   9534e:	bt     %rcx,%rdi
   95352:	jae    9535f <<c30walk::ctor2d::W<false>>::refill+0xcf>
   95354:	mov    $0xfffff,%edi
   95359:	and    0x3c(%r8),%edi
   9535d:	jmp    95364 <<c30walk::ctor2d::W<false>>::refill+0xd4>
   9535f:	mov    $0xffffffff,%edi
   95364:	mov    %esi,%esi
   95366:	lea    (%r8,%rsi,2),%rsi
   9536a:	mov    %rsi,0x40(%r14,%rbx,8)
   9536f:	sub    %r15d,%edx
   95372:	lea    (%rsi,%rdx,2),%rdx
   95376:	mov    %rdx,0x840(%r14,%rbx,8)
   9537e:	mov    %rsi,(%r12)
   95382:	mov    %r15d,0x8(%r12)
   95387:	mov    %ecx,0x10(%r12)
   9538c:	mov    %edi,0x14(%r12)
   95391:	cmp    %eax,%r15d
   95394:	setb   %al
   95397:	add    $0x8,%rsp
   9539b:	pop    %rbx
   9539c:	pop    %r12
   9539e:	pop    %r14
   953a0:	pop    %r15
   953a2:	ret
   953a3:	lea    0x9b92e(%rip),%rdx        # 130cd8 <__frame_dummy_init_array_entry+0x218>
   953aa:	call   da0a3 <core::panicking::panic_bounds_check>
