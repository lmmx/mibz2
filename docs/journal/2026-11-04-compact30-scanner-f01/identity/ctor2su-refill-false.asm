# <c30walk::ctor2su::W<false>>::refill
# size 0xec

target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

0000000000097660 <<c30walk::ctor2su::W<false>>::refill>:
   97660:	push   %r15
   97662:	push   %r14
   97664:	push   %rbx
   97665:	mov    %rdi,%rbx
   97668:	lea    (%rsi,%rsi,4),%rax
   9766c:	lea    (%rdi,%rax,8),%r15
   97670:	add    $0x40,%r15
   97674:	mov    0x40(%rdi,%rax,8),%r14
   97679:	sub    0x50(%rdi,%rax,8),%r14
   9767e:	je     97688 <<c30walk::ctor2su::W<false>>::refill+0x28>
   97680:	mov    %rbx,%rdi
   97683:	call   97060 <<c30walk::ctor2su::W<false>>::finish>
   97688:	shr    $1,%r14
   9768b:	add    0x18(%r15),%r14d
   9768f:	mov    %r14d,0x18(%r15)
   97693:	xorps  %xmm0,%xmm0
   97696:	movups %xmm0,(%r15)
   9769a:	movq   $0x0,0x10(%r15)
   976a2:	mov    0x1c(%r15),%eax
   976a6:	cmp    %eax,%r14d
   976a9:	jae    97734 <<c30walk::ctor2su::W<false>>::refill+0xd4>
   976af:	mov    %r14d,%ecx
   976b2:	mov    $0x88888889,%edi
   976b7:	imul   %rcx,%rdi
   976bb:	mov    %rdi,%rcx
   976be:	shr    $0x24,%rcx
   976c2:	mov    %ecx,%r9d
   976c5:	shl    $0x5,%r9d
   976c9:	sub    %ecx,%r9d
   976cc:	sub    %ecx,%r9d
   976cf:	lea    0x1e(%r9),%edx
   976d3:	cmp    %eax,%edx
   976d5:	cmovae %eax,%edx
   976d8:	shr    $0x2a,%rdi
   976dc:	mov    0x10(%rbx),%rsi
   976e0:	cmp    %rsi,%rdi
   976e3:	jae    97740 <<c30walk::ctor2su::W<false>>::refill+0xe0>
   976e5:	mov    %r14d,%esi
   976e8:	mov    %rcx,%r8
   976eb:	shl    $0x6,%r8
   976ef:	add    0x28(%rbx),%r8
   976f3:	sub    %r9d,%esi
   976f6:	mov    0x8(%rbx),%r9
   976fa:	mov    (%r9,%rdi,8),%rdi
   976fe:	bt     %rcx,%rdi
   97702:	jae    9770f <<c30walk::ctor2su::W<false>>::refill+0xaf>
   97704:	mov    $0xfffff,%edi
   97709:	and    0x3c(%r8),%edi
   9770d:	jmp    97714 <<c30walk::ctor2su::W<false>>::refill+0xb4>
   9770f:	mov    $0xffffffff,%edi
   97714:	mov    %esi,%esi
   97716:	lea    (%r8,%rsi,2),%rsi
   9771a:	mov    %rsi,0x10(%r15)
   9771e:	mov    %rsi,(%r15)
   97721:	sub    %r14d,%edx
   97724:	lea    (%rsi,%rdx,2),%rdx
   97728:	mov    %rdx,0x8(%r15)
   9772c:	mov    %ecx,0x20(%r15)
   97730:	mov    %edi,0x24(%r15)
   97734:	cmp    %eax,%r14d
   97737:	setb   %al
   9773a:	pop    %rbx
   9773b:	pop    %r14
   9773d:	pop    %r15
   9773f:	ret
   97740:	lea    0xafd61(%rip),%rdx        # 1474a8 <__frame_dummy_init_array_entry+0x288>
   97747:	call   f0803 <core::panicking::panic_bounds_check>
