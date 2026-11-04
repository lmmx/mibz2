# <c30walk::ctor2::W<false>>::refill
# size 0xec

target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

000000000008e8f0 <<c30walk::ctor2::W<false>>::refill>:
   8e8f0:	push   %r15
   8e8f2:	push   %r14
   8e8f4:	push   %rbx
   8e8f5:	mov    %rdi,%rbx
   8e8f8:	lea    (%rsi,%rsi,4),%rax
   8e8fc:	lea    (%rdi,%rax,8),%r15
   8e900:	add    $0x40,%r15
   8e904:	mov    0x40(%rdi,%rax,8),%r14
   8e909:	sub    0x50(%rdi,%rax,8),%r14
   8e90e:	je     8e918 <<c30walk::ctor2::W<false>>::refill+0x28>
   8e910:	mov    %rbx,%rdi
   8e913:	call   8e2f0 <<c30walk::ctor2::W<false>>::finish>
   8e918:	shr    $1,%r14
   8e91b:	add    0x18(%r15),%r14d
   8e91f:	mov    %r14d,0x18(%r15)
   8e923:	xorps  %xmm0,%xmm0
   8e926:	movups %xmm0,(%r15)
   8e92a:	movq   $0x0,0x10(%r15)
   8e932:	mov    0x1c(%r15),%eax
   8e936:	cmp    %eax,%r14d
   8e939:	jae    8e9c4 <<c30walk::ctor2::W<false>>::refill+0xd4>
   8e93f:	mov    %r14d,%ecx
   8e942:	mov    $0x88888889,%edi
   8e947:	imul   %rcx,%rdi
   8e94b:	mov    %rdi,%rcx
   8e94e:	shr    $0x24,%rcx
   8e952:	mov    %ecx,%r9d
   8e955:	shl    $0x5,%r9d
   8e959:	sub    %ecx,%r9d
   8e95c:	sub    %ecx,%r9d
   8e95f:	lea    0x1e(%r9),%edx
   8e963:	cmp    %eax,%edx
   8e965:	cmovae %eax,%edx
   8e968:	shr    $0x2a,%rdi
   8e96c:	mov    0x10(%rbx),%rsi
   8e970:	cmp    %rsi,%rdi
   8e973:	jae    8e9d0 <<c30walk::ctor2::W<false>>::refill+0xe0>
   8e975:	mov    %r14d,%esi
   8e978:	mov    %rcx,%r8
   8e97b:	shl    $0x6,%r8
   8e97f:	add    0x28(%rbx),%r8
   8e983:	sub    %r9d,%esi
   8e986:	mov    0x8(%rbx),%r9
   8e98a:	mov    (%r9,%rdi,8),%rdi
   8e98e:	bt     %rcx,%rdi
   8e992:	jae    8e99f <<c30walk::ctor2::W<false>>::refill+0xaf>
   8e994:	mov    $0xfffff,%edi
   8e999:	and    0x3c(%r8),%edi
   8e99d:	jmp    8e9a4 <<c30walk::ctor2::W<false>>::refill+0xb4>
   8e99f:	mov    $0xffffffff,%edi
   8e9a4:	mov    %esi,%esi
   8e9a6:	lea    (%r8,%rsi,2),%rsi
   8e9aa:	mov    %rsi,0x10(%r15)
   8e9ae:	mov    %rsi,(%r15)
   8e9b1:	sub    %r14d,%edx
   8e9b4:	lea    (%rsi,%rdx,2),%rdx
   8e9b8:	mov    %rdx,0x8(%r15)
   8e9bc:	mov    %ecx,0x20(%r15)
   8e9c0:	mov    %edi,0x24(%r15)
   8e9c4:	cmp    %eax,%r14d
   8e9c7:	setb   %al
   8e9ca:	pop    %rbx
   8e9cb:	pop    %r14
   8e9cd:	pop    %r15
   8e9cf:	ret
   8e9d0:	lea    0xaed49(%rip),%rdx        # 13d720 <__frame_dummy_init_array_entry+0x240>
   8e9d7:	call   e6ac3 <core::panicking::panic_bounds_check>
