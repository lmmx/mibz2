# <c30walk::ctor2::W<false>>::refill
# size 0xec

target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

0000000000076af0 <<c30walk::ctor2::W<false>>::refill>:
   76af0:	push   %r15
   76af2:	push   %r14
   76af4:	push   %rbx
   76af5:	mov    %rdi,%rbx
   76af8:	lea    (%rsi,%rsi,4),%rax
   76afc:	lea    (%rdi,%rax,8),%r15
   76b00:	add    $0x40,%r15
   76b04:	mov    0x40(%rdi,%rax,8),%r14
   76b09:	sub    0x50(%rdi,%rax,8),%r14
   76b0e:	je     76b18 <<c30walk::ctor2::W<false>>::refill+0x28>
   76b10:	mov    %rbx,%rdi
   76b13:	call   764f0 <<c30walk::ctor2::W<false>>::finish>
   76b18:	shr    $1,%r14
   76b1b:	add    0x18(%r15),%r14d
   76b1f:	mov    %r14d,0x18(%r15)
   76b23:	xorps  %xmm0,%xmm0
   76b26:	movups %xmm0,(%r15)
   76b2a:	movq   $0x0,0x10(%r15)
   76b32:	mov    0x1c(%r15),%eax
   76b36:	cmp    %eax,%r14d
   76b39:	jae    76bc4 <<c30walk::ctor2::W<false>>::refill+0xd4>
   76b3f:	mov    %r14d,%ecx
   76b42:	mov    $0x88888889,%edi
   76b47:	imul   %rcx,%rdi
   76b4b:	mov    %rdi,%rcx
   76b4e:	shr    $0x24,%rcx
   76b52:	mov    %ecx,%r9d
   76b55:	shl    $0x5,%r9d
   76b59:	sub    %ecx,%r9d
   76b5c:	sub    %ecx,%r9d
   76b5f:	lea    0x1e(%r9),%edx
   76b63:	cmp    %eax,%edx
   76b65:	cmovae %eax,%edx
   76b68:	shr    $0x2a,%rdi
   76b6c:	mov    0x10(%rbx),%rsi
   76b70:	cmp    %rsi,%rdi
   76b73:	jae    76bd0 <<c30walk::ctor2::W<false>>::refill+0xe0>
   76b75:	mov    %r14d,%esi
   76b78:	mov    %rcx,%r8
   76b7b:	shl    $0x6,%r8
   76b7f:	add    0x28(%rbx),%r8
   76b83:	sub    %r9d,%esi
   76b86:	mov    0x8(%rbx),%r9
   76b8a:	mov    (%r9,%rdi,8),%rdi
   76b8e:	bt     %rcx,%rdi
   76b92:	jae    76b9f <<c30walk::ctor2::W<false>>::refill+0xaf>
   76b94:	mov    $0xfffff,%edi
   76b99:	and    0x3c(%r8),%edi
   76b9d:	jmp    76ba4 <<c30walk::ctor2::W<false>>::refill+0xb4>
   76b9f:	mov    $0xffffffff,%edi
   76ba4:	mov    %esi,%esi
   76ba6:	lea    (%r8,%rsi,2),%rsi
   76baa:	mov    %rsi,0x10(%r15)
   76bae:	mov    %rsi,(%r15)
   76bb1:	sub    %r14d,%edx
   76bb4:	lea    (%rsi,%rdx,2),%rdx
   76bb8:	mov    %rdx,0x8(%r15)
   76bbc:	mov    %ecx,0x20(%r15)
   76bc0:	mov    %edi,0x24(%r15)
   76bc4:	cmp    %eax,%r14d
   76bc7:	setb   %al
   76bca:	pop    %rbx
   76bcb:	pop    %r14
   76bcd:	pop    %r15
   76bcf:	ret
   76bd0:	lea    0x8a009(%rip),%rdx        # 100be0 <__frame_dummy_init_array_entry+0x240>
   76bd7:	call   a9f23 <core::panicking::panic_bounds_check>
