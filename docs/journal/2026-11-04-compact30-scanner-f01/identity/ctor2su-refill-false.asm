# <c30walk::ctor2su::W<false>>::refill
# size 0xec

target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

0000000000097670 <<c30walk::ctor2su::W<false>>::refill>:
   97670:	push   %r15
   97672:	push   %r14
   97674:	push   %rbx
   97675:	mov    %rdi,%rbx
   97678:	lea    (%rsi,%rsi,4),%rax
   9767c:	lea    (%rdi,%rax,8),%r15
   97680:	add    $0x40,%r15
   97684:	mov    0x40(%rdi,%rax,8),%r14
   97689:	sub    0x50(%rdi,%rax,8),%r14
   9768e:	je     97698 <<c30walk::ctor2su::W<false>>::refill+0x28>
   97690:	mov    %rbx,%rdi
   97693:	call   97070 <<c30walk::ctor2su::W<false>>::finish>
   97698:	shr    $1,%r14
   9769b:	add    0x18(%r15),%r14d
   9769f:	mov    %r14d,0x18(%r15)
   976a3:	xorps  %xmm0,%xmm0
   976a6:	movups %xmm0,(%r15)
   976aa:	movq   $0x0,0x10(%r15)
   976b2:	mov    0x1c(%r15),%eax
   976b6:	cmp    %eax,%r14d
   976b9:	jae    97744 <<c30walk::ctor2su::W<false>>::refill+0xd4>
   976bf:	mov    %r14d,%ecx
   976c2:	mov    $0x88888889,%edi
   976c7:	imul   %rcx,%rdi
   976cb:	mov    %rdi,%rcx
   976ce:	shr    $0x24,%rcx
   976d2:	mov    %ecx,%r9d
   976d5:	shl    $0x5,%r9d
   976d9:	sub    %ecx,%r9d
   976dc:	sub    %ecx,%r9d
   976df:	lea    0x1e(%r9),%edx
   976e3:	cmp    %eax,%edx
   976e5:	cmovae %eax,%edx
   976e8:	shr    $0x2a,%rdi
   976ec:	mov    0x10(%rbx),%rsi
   976f0:	cmp    %rsi,%rdi
   976f3:	jae    97750 <<c30walk::ctor2su::W<false>>::refill+0xe0>
   976f5:	mov    %r14d,%esi
   976f8:	mov    %rcx,%r8
   976fb:	shl    $0x6,%r8
   976ff:	add    0x28(%rbx),%r8
   97703:	sub    %r9d,%esi
   97706:	mov    0x8(%rbx),%r9
   9770a:	mov    (%r9,%rdi,8),%rdi
   9770e:	bt     %rcx,%rdi
   97712:	jae    9771f <<c30walk::ctor2su::W<false>>::refill+0xaf>
   97714:	mov    $0xfffff,%edi
   97719:	and    0x3c(%r8),%edi
   9771d:	jmp    97724 <<c30walk::ctor2su::W<false>>::refill+0xb4>
   9771f:	mov    $0xffffffff,%edi
   97724:	mov    %esi,%esi
   97726:	lea    (%r8,%rsi,2),%rsi
   9772a:	mov    %rsi,0x10(%r15)
   9772e:	mov    %rsi,(%r15)
   97731:	sub    %r14d,%edx
   97734:	lea    (%rsi,%rdx,2),%rdx
   97738:	mov    %rdx,0x8(%r15)
   9773c:	mov    %ecx,0x20(%r15)
   97740:	mov    %edi,0x24(%r15)
   97744:	cmp    %eax,%r14d
   97747:	setb   %al
   9774a:	pop    %rbx
   9774b:	pop    %r14
   9774d:	pop    %r15
   9774f:	ret
   97750:	lea    0xafdc1(%rip),%rdx        # 147518 <__frame_dummy_init_array_entry+0x288>
   97757:	call   f0873 <core::panicking::panic_bounds_check>
