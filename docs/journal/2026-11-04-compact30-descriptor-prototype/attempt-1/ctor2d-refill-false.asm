# <c30walk::ctor2d::W<false>>::refill
# size 0x11f

target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

0000000000092860 <<c30walk::ctor2d::W<false>>::refill>:
   92860:	push   %r15
   92862:	push   %r14
   92864:	push   %r12
   92866:	push   %rbx
   92867:	push   %rax
   92868:	mov    %rsi,%rbx
   9286b:	mov    %rdi,%r14
   9286e:	mov    0x40(%rdi,%rsi,8),%r15
   92873:	lea    (%rsi,%rsi,2),%rax
   92877:	lea    (%rdi,%rax,8),%r12
   9287b:	add    $0x1040,%r12
   92882:	sub    0x1040(%rdi,%rax,8),%r15
   9288a:	je     92897 <<c30walk::ctor2d::W<false>>::refill+0x37>
   9288c:	mov    %r14,%rdi
   9288f:	mov    %rbx,%rsi
   92892:	call   92250 <<c30walk::ctor2d::W<false>>::finish>
   92897:	shr    $1,%r15
   9289a:	add    0x8(%r12),%r15d
   9289f:	mov    %r15d,0x8(%r12)
   928a4:	movq   $0x0,(%r12)
   928ac:	movq   $0x0,0x40(%r14,%rbx,8)
   928b5:	movq   $0x0,0x840(%r14,%rbx,8)
   928c1:	mov    0xc(%r12),%eax
   928c6:	cmp    %eax,%r15d
   928c9:	jae    92961 <<c30walk::ctor2d::W<false>>::refill+0x101>
   928cf:	mov    %r15d,%ecx
   928d2:	mov    $0x88888889,%edi
   928d7:	imul   %rcx,%rdi
   928db:	mov    %rdi,%rcx
   928de:	shr    $0x24,%rcx
   928e2:	mov    %ecx,%r9d
   928e5:	shl    $0x5,%r9d
   928e9:	sub    %ecx,%r9d
   928ec:	sub    %ecx,%r9d
   928ef:	lea    0x1e(%r9),%edx
   928f3:	cmp    %eax,%edx
   928f5:	cmovae %eax,%edx
   928f8:	shr    $0x2a,%rdi
   928fc:	mov    0x10(%r14),%rsi
   92900:	cmp    %rsi,%rdi
   92903:	jae    92973 <<c30walk::ctor2d::W<false>>::refill+0x113>
   92905:	mov    %r15d,%esi
   92908:	mov    %rcx,%r8
   9290b:	shl    $0x6,%r8
   9290f:	add    0x28(%r14),%r8
   92913:	sub    %r9d,%esi
   92916:	mov    0x8(%r14),%r9
   9291a:	mov    (%r9,%rdi,8),%rdi
   9291e:	bt     %rcx,%rdi
   92922:	jae    9292f <<c30walk::ctor2d::W<false>>::refill+0xcf>
   92924:	mov    $0xfffff,%edi
   92929:	and    0x3c(%r8),%edi
   9292d:	jmp    92934 <<c30walk::ctor2d::W<false>>::refill+0xd4>
   9292f:	mov    $0xffffffff,%edi
   92934:	mov    %esi,%esi
   92936:	lea    (%r8,%rsi,2),%rsi
   9293a:	mov    %rsi,0x40(%r14,%rbx,8)
   9293f:	sub    %r15d,%edx
   92942:	lea    (%rsi,%rdx,2),%rdx
   92946:	mov    %rdx,0x840(%r14,%rbx,8)
   9294e:	mov    %rsi,(%r12)
   92952:	mov    %r15d,0x8(%r12)
   92957:	mov    %ecx,0x10(%r12)
   9295c:	mov    %edi,0x14(%r12)
   92961:	cmp    %eax,%r15d
   92964:	setb   %al
   92967:	add    $0x8,%rsp
   9296b:	pop    %rbx
   9296c:	pop    %r12
   9296e:	pop    %r14
   92970:	pop    %r15
   92972:	ret
   92973:	lea    0x9b7be(%rip),%rdx        # 12e138 <__frame_dummy_init_array_entry+0x218>
   9297a:	call   d7533 <core::panicking::panic_bounds_check>
