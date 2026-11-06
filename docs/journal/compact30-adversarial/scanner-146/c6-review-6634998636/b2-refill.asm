# <c30walk::ctor2::W<false>>::refill
# size 0xec

target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

000000000008a7e0 <<c30walk::ctor2::W<false>>::refill>:
   8a7e0:	push   %r15
   8a7e2:	push   %r14
   8a7e4:	push   %rbx
   8a7e5:	mov    %rdi,%rbx
   8a7e8:	lea    (%rsi,%rsi,4),%rax
   8a7ec:	lea    (%rdi,%rax,8),%r15
   8a7f0:	add    $0x40,%r15
   8a7f4:	mov    0x40(%rdi,%rax,8),%r14
   8a7f9:	sub    0x50(%rdi,%rax,8),%r14
   8a7fe:	je     8a808 <<c30walk::ctor2::W<false>>::refill+0x28>
   8a800:	mov    %rbx,%rdi
   8a803:	call   8a1e0 <<c30walk::ctor2::W<false>>::finish>
   8a808:	shr    $1,%r14
   8a80b:	add    0x18(%r15),%r14d
   8a80f:	mov    %r14d,0x18(%r15)
   8a813:	xorps  %xmm0,%xmm0
   8a816:	movups %xmm0,(%r15)
   8a81a:	movq   $0x0,0x10(%r15)
   8a822:	mov    0x1c(%r15),%eax
   8a826:	cmp    %eax,%r14d
   8a829:	jae    8a8b4 <<c30walk::ctor2::W<false>>::refill+0xd4>
   8a82f:	mov    %r14d,%ecx
   8a832:	mov    $0x88888889,%edi
   8a837:	imul   %rcx,%rdi
   8a83b:	mov    %rdi,%rcx
   8a83e:	shr    $0x24,%rcx
   8a842:	mov    %ecx,%r9d
   8a845:	shl    $0x5,%r9d
   8a849:	sub    %ecx,%r9d
   8a84c:	sub    %ecx,%r9d
   8a84f:	lea    0x1e(%r9),%edx
   8a853:	cmp    %eax,%edx
   8a855:	cmovae %eax,%edx
   8a858:	shr    $0x2a,%rdi
   8a85c:	mov    0x10(%rbx),%rsi
   8a860:	cmp    %rsi,%rdi
   8a863:	jae    8a8c0 <<c30walk::ctor2::W<false>>::refill+0xe0>
   8a865:	mov    %r14d,%esi
   8a868:	mov    %rcx,%r8
   8a86b:	shl    $0x6,%r8
   8a86f:	add    0x28(%rbx),%r8
   8a873:	sub    %r9d,%esi
   8a876:	mov    0x8(%rbx),%r9
   8a87a:	mov    (%r9,%rdi,8),%rdi
   8a87e:	bt     %rcx,%rdi
   8a882:	jae    8a88f <<c30walk::ctor2::W<false>>::refill+0xaf>
   8a884:	mov    $0xfffff,%edi
   8a889:	and    0x3c(%r8),%edi
   8a88d:	jmp    8a894 <<c30walk::ctor2::W<false>>::refill+0xb4>
   8a88f:	mov    $0xffffffff,%edi
   8a894:	mov    %esi,%esi
   8a896:	lea    (%r8,%rsi,2),%rsi
   8a89a:	mov    %rsi,0x10(%r15)
   8a89e:	mov    %rsi,(%r15)
   8a8a1:	sub    %r14d,%edx
   8a8a4:	lea    (%rsi,%rdx,2),%rdx
   8a8a8:	mov    %rdx,0x8(%r15)
   8a8ac:	mov    %ecx,0x20(%r15)
   8a8b0:	mov    %edi,0x24(%r15)
   8a8b4:	cmp    %eax,%r14d
   8a8b7:	setb   %al
   8a8ba:	pop    %rbx
   8a8bb:	pop    %r14
   8a8bd:	pop    %r15
   8a8bf:	ret
   8a8c0:	lea    0xa54b1(%rip),%rdx        # 12fd78 <__frame_dummy_init_array_entry+0x258>
   8a8c7:	call   d9103 <core::panicking::panic_bounds_check>
