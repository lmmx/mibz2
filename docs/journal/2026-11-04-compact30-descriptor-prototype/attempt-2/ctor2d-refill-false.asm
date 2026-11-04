# <c30walk::ctor2d::W<false>>::refill
# size 0x11f

target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

0000000000092850 <<c30walk::ctor2d::W<false>>::refill>:
   92850:	push   %r15
   92852:	push   %r14
   92854:	push   %r12
   92856:	push   %rbx
   92857:	push   %rax
   92858:	mov    %rsi,%rbx
   9285b:	mov    %rdi,%r14
   9285e:	mov    0x40(%rdi,%rsi,8),%r15
   92863:	lea    (%rsi,%rsi,2),%rax
   92867:	lea    (%rdi,%rax,8),%r12
   9286b:	add    $0x1040,%r12
   92872:	sub    0x1040(%rdi,%rax,8),%r15
   9287a:	je     92887 <<c30walk::ctor2d::W<false>>::refill+0x37>
   9287c:	mov    %r14,%rdi
   9287f:	mov    %rbx,%rsi
   92882:	call   92240 <<c30walk::ctor2d::W<false>>::finish>
   92887:	shr    $1,%r15
   9288a:	add    0x8(%r12),%r15d
   9288f:	mov    %r15d,0x8(%r12)
   92894:	movq   $0x0,(%r12)
   9289c:	movq   $0x0,0x40(%r14,%rbx,8)
   928a5:	movq   $0x0,0x840(%r14,%rbx,8)
   928b1:	mov    0xc(%r12),%eax
   928b6:	cmp    %eax,%r15d
   928b9:	jae    92951 <<c30walk::ctor2d::W<false>>::refill+0x101>
   928bf:	mov    %r15d,%ecx
   928c2:	mov    $0x88888889,%edi
   928c7:	imul   %rcx,%rdi
   928cb:	mov    %rdi,%rcx
   928ce:	shr    $0x24,%rcx
   928d2:	mov    %ecx,%r9d
   928d5:	shl    $0x5,%r9d
   928d9:	sub    %ecx,%r9d
   928dc:	sub    %ecx,%r9d
   928df:	lea    0x1e(%r9),%edx
   928e3:	cmp    %eax,%edx
   928e5:	cmovae %eax,%edx
   928e8:	shr    $0x2a,%rdi
   928ec:	mov    0x10(%r14),%rsi
   928f0:	cmp    %rsi,%rdi
   928f3:	jae    92963 <<c30walk::ctor2d::W<false>>::refill+0x113>
   928f5:	mov    %r15d,%esi
   928f8:	mov    %rcx,%r8
   928fb:	shl    $0x6,%r8
   928ff:	add    0x28(%r14),%r8
   92903:	sub    %r9d,%esi
   92906:	mov    0x8(%r14),%r9
   9290a:	mov    (%r9,%rdi,8),%rdi
   9290e:	bt     %rcx,%rdi
   92912:	jae    9291f <<c30walk::ctor2d::W<false>>::refill+0xcf>
   92914:	mov    $0xfffff,%edi
   92919:	and    0x3c(%r8),%edi
   9291d:	jmp    92924 <<c30walk::ctor2d::W<false>>::refill+0xd4>
   9291f:	mov    $0xffffffff,%edi
   92924:	mov    %esi,%esi
   92926:	lea    (%r8,%rsi,2),%rsi
   9292a:	mov    %rsi,0x40(%r14,%rbx,8)
   9292f:	sub    %r15d,%edx
   92932:	lea    (%rsi,%rdx,2),%rdx
   92936:	mov    %rdx,0x840(%r14,%rbx,8)
   9293e:	mov    %rsi,(%r12)
   92942:	mov    %r15d,0x8(%r12)
   92947:	mov    %ecx,0x10(%r12)
   9294c:	mov    %edi,0x14(%r12)
   92951:	cmp    %eax,%r15d
   92954:	setb   %al
   92957:	add    $0x8,%rsp
   9295b:	pop    %rbx
   9295c:	pop    %r12
   9295e:	pop    %r14
   92960:	pop    %r15
   92962:	ret
   92963:	lea    0x9b7be(%rip),%rdx        # 12e128 <__frame_dummy_init_array_entry+0x218>
   9296a:	call   d7523 <core::panicking::panic_bounds_check>
