# <c30walk::ctor2s::W<false>>::finish
# size 0x5fa

target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

000000000008bb00 <<c30walk::ctor2s::W<false>>::finish>:
   8bb00:	push   %rbp
   8bb01:	push   %r15
   8bb03:	push   %r14
   8bb05:	push   %r13
   8bb07:	push   %r12
   8bb09:	push   %rbx
   8bb0a:	sub    $0x38,%rsp
   8bb0e:	mov    0x18(%rdi),%rbp
   8bb12:	mov    0x10(%rbp),%r8
   8bb16:	mov    %r8,%r13
   8bb19:	sub    $0x1,%r13
   8bb1d:	jb     8c08a <<c30walk::ctor2s::W<false>>::finish+0x58a>
   8bb23:	mov    %rdi,%r11
   8bb26:	lea    (%rsi,%rsi,4),%rax
   8bb2a:	mov    0x40(%rdi,%rax,8),%r14
   8bb2f:	mov    0x50(%rdi,%rax,8),%rdi
   8bb34:	mov    0x58(%r11,%rax,8),%r12d
   8bb39:	mov    0x60(%r11,%rax,8),%ecx
   8bb3e:	mov    0x64(%r11,%rax,8),%eax
   8bb43:	sub    %rdi,%r14
   8bb46:	mov    %r14,0x8(%rsp)
   8bb4b:	shr    $1,%r14
   8bb4e:	mov    0x8(%rbp),%r10
   8bb52:	mov    %r13,%rdx
   8bb55:	shl    $0xa,%rdx
   8bb59:	add    %r10,%rdx
   8bb5c:	mov    %r13,%rbx
   8bb5f:	cmp    %r12d,(%rdx,%rsi,4)
   8bb63:	jbe    8bbd8 <<c30walk::ctor2s::W<false>>::finish+0xd8>
   8bb65:	mov    %r11,%r15
   8bb68:	lea    (%r12,%r14,1),%edx
   8bb6c:	dec    %edx
   8bb6e:	mov    %r8,%r9
   8bb71:	shl    $0xa,%r9
   8bb75:	lea    (%r9,%rsi,4),%r9
   8bb79:	lea    (%r10,%r9,1),%r11
   8bb7d:	add    $0xfffffffffffffc00,%r11
   8bb84:	mov    %r8,%rbx
   8bb87:	nopw   0x0(%rax,%rax,1)
   8bb90:	dec    %rbx
   8bb93:	cmp    %r8,%rbx
   8bb96:	jae    8c03e <<c30walk::ctor2s::W<false>>::finish+0x53e>
   8bb9c:	cmp    %r12d,(%r11)
   8bb9f:	lea    -0x400(%r11),%r11
   8bba6:	ja     8bb90 <<c30walk::ctor2s::W<false>>::finish+0x90>
   8bba8:	add    %r10,%r9
   8bbab:	add    $0xfffffffffffffc00,%r9
   8bbb2:	mov    %r8,%r13
   8bbb5:	mov    %r15,%r11
   8bbb8:	nopl   0x0(%rax,%rax,1)
   8bbc0:	dec    %r13
   8bbc3:	cmp    %r8,%r13
   8bbc6:	jae    8c050 <<c30walk::ctor2s::W<false>>::finish+0x550>
   8bbcc:	cmp    %edx,(%r9)
   8bbcf:	lea    -0x400(%r9),%r9
   8bbd6:	ja     8bbc0 <<c30walk::ctor2s::W<false>>::finish+0xc0>
   8bbd8:	imul   $0xffffffffffffffe2,%rcx,%r15
   8bbdc:	add    %r12,%r15
   8bbdf:	cmp    $0xffffffff,%eax
   8bbe2:	mov    %r14,0x10(%rsp)
   8bbe7:	je     8bceb <<c30walk::ctor2s::W<false>>::finish+0x1eb>
   8bbed:	mov    $0xffffffff,%r9d
   8bbf3:	cmp    %r13,%rbx
   8bbf6:	jae    8bc10 <<c30walk::ctor2s::W<false>>::finish+0x110>
   8bbf8:	lea    0x1(%rbx),%rcx
   8bbfc:	cmp    %r8,%rcx
   8bbff:	jae    8c0ad <<c30walk::ctor2s::W<false>>::finish+0x5ad>
   8bc05:	shl    $0xa,%rcx
   8bc09:	add    %rcx,%r10
   8bc0c:	mov    (%r10,%rsi,4),%r9d
   8bc10:	mov    (%r11),%rdx
   8bc13:	mov    0x10(%rdx),%rcx
   8bc17:	cmp    %rax,%rcx
   8bc1a:	jbe    8c09b <<c30walk::ctor2s::W<false>>::finish+0x59b>
   8bc20:	test   %r14,%r14
   8bc23:	je     8c0eb <<c30walk::ctor2s::W<false>>::finish+0x5eb>
   8bc29:	shl    $0x7,%rax
   8bc2d:	add    0x8(%rdx),%rax
   8bc31:	lea    0x800(,%rsi,4),%rdx
   8bc39:	xor    %r11d,%r11d
   8bc3c:	nopl   0x0(%rax)
   8bc40:	lea    (%r12,%r11,1),%r10d
   8bc44:	cmp    %r9d,%r10d
   8bc47:	jae    8bc80 <<c30walk::ctor2s::W<false>>::finish+0x180>
   8bc49:	lea    (%r11,%r15,1),%rcx
   8bc4d:	cmp    $0x20,%rcx
   8bc51:	jae    8c062 <<c30walk::ctor2s::W<false>>::finish+0x562>
   8bc57:	movzwl (%rdi,%r11,2),%r8d
   8bc5c:	inc    %r11
   8bc5f:	mov    %ebx,%r10d
   8bc62:	shl    $0x18,%r10d
   8bc66:	shl    $0x8,%r8d
   8bc6a:	or     %r10d,%r8d
   8bc6d:	or     %esi,%r8d
   8bc70:	mov    %r8d,(%rax,%rcx,4)
   8bc74:	cmp    0x10(%rsp),%r11
   8bc79:	jne    8bc40 <<c30walk::ctor2s::W<false>>::finish+0x140>
   8bc7b:	jmp    8bcc9 <<c30walk::ctor2s::W<false>>::finish+0x1c9>
   8bc7d:	nopl   (%rax)
   8bc80:	mov    %rbx,%r14
   8bc83:	shl    $0xa,%r14
   8bc87:	add    %rdx,%r14
   8bc8a:	mov    %rbx,%rcx
   8bc8d:	jmp    8bc9f <<c30walk::ctor2s::W<false>>::finish+0x19f>
   8bc8f:	nop
   8bc90:	add    $0x400,%r14
   8bc97:	mov    %rbx,%rcx
   8bc9a:	cmp    %r9d,%r10d
   8bc9d:	jb     8bc49 <<c30walk::ctor2s::W<false>>::finish+0x149>
   8bc9f:	lea    0x1(%rcx),%rbx
   8bca3:	mov    $0xffffffff,%r9d
   8bca9:	cmp    %r13,%rbx
   8bcac:	jae    8bc90 <<c30walk::ctor2s::W<false>>::finish+0x190>
   8bcae:	add    $0x2,%rcx
   8bcb2:	mov    0x10(%rbp),%r8
   8bcb6:	cmp    %r8,%rcx
   8bcb9:	jae    8c02c <<c30walk::ctor2s::W<false>>::finish+0x52c>
   8bcbf:	mov    0x8(%rbp),%rcx
   8bcc3:	mov    (%rcx,%r14,1),%r9d
   8bcc7:	jmp    8bc90 <<c30walk::ctor2s::W<false>>::finish+0x190>
   8bcc9:	mov    0x8(%rsp),%rdx
   8bcce:	and    $0xfffffffffffffffe,%rdx
   8bcd2:	mov    $0xff,%esi
   8bcd7:	add    $0x38,%rsp
   8bcdb:	pop    %rbx
   8bcdc:	pop    %r12
   8bcde:	pop    %r13
   8bce0:	pop    %r14
   8bce2:	pop    %r15
   8bce4:	pop    %rbp
   8bce5:	jmp    *0xa91a5(%rip)        # 134e90 <memset@GLIBC_2.2.5>
   8bceb:	movzwl (%rdi),%eax
   8bcee:	movzwl -0x2(%rdi,%r14,2),%r8d
   8bcf4:	mov    %ebx,%edx
   8bcf6:	shl    $0x10,%edx
   8bcf9:	or     %eax,%edx
   8bcfb:	mov    %r13d,%r9d
   8bcfe:	shl    $0x10,%r9d
   8bd02:	or     %r8d,%r9d
   8bd05:	sub    %edx,%r9d
   8bd08:	cmp    $0xfffe,%r9d
   8bd0f:	jae    8bd8d <<c30walk::ctor2s::W<false>>::finish+0x28d>
   8bd11:	mov    0x28(%r11),%r8
   8bd15:	shl    $0x6,%rcx
   8bd19:	shl    $0x8,%edx
   8bd1c:	or     %edx,%esi
   8bd1e:	mov    %esi,0x3c(%r8,%rcx,1)
   8bd23:	test   %r14,%r14
   8bd26:	je     8c0eb <<c30walk::ctor2s::W<false>>::finish+0x5eb>
   8bd2c:	cmp    $0x1e,%r14
   8bd30:	jne    8bf26 <<c30walk::ctor2s::W<false>>::finish+0x426>
   8bd36:	movd   %eax,%xmm0
   8bd3a:	pshuflw $0x0,%xmm0,%xmm0
   8bd3f:	pshufd $0x44,%xmm0,%xmm1
   8bd44:	movdqu (%rdi),%xmm2
   8bd48:	movdqu 0x10(%rdi),%xmm3
   8bd4d:	movdqu 0x20(%rdi),%xmm4
   8bd52:	psubw  %xmm1,%xmm2
   8bd56:	movdqu %xmm2,(%rdi)
   8bd5a:	psubw  %xmm1,%xmm3
   8bd5e:	movdqu %xmm3,0x10(%rdi)
   8bd63:	psubw  %xmm1,%xmm4
   8bd67:	movdqu %xmm4,0x20(%rdi)
   8bd6c:	movq   0x30(%rdi),%xmm1
   8bd71:	psubw  %xmm0,%xmm1
   8bd75:	movq   %xmm1,0x30(%rdi)
   8bd7a:	movd   0x38(%rdi),%xmm1
   8bd7f:	psubw  %xmm0,%xmm1
   8bd83:	movd   %xmm1,0x38(%rdi)
   8bd88:	jmp    8c0eb <<c30walk::ctor2s::W<false>>::finish+0x5eb>
   8bd8d:	mov    (%r11),%r10
   8bd90:	mov    0x10(%r10),%r9
   8bd94:	cmp    (%r10),%r9
   8bd97:	mov    %r11,0x30(%rsp)
   8bd9c:	je     8bfec <<c30walk::ctor2s::W<false>>::finish+0x4ec>
   8bda2:	mov    0x8(%r10),%rax
   8bda6:	mov    %r9,%rdx
   8bda9:	shl    $0x7,%rdx
   8bdad:	pxor   %xmm0,%xmm0
   8bdb1:	movdqa %xmm0,0x70(%rax,%rdx,1)
   8bdb7:	movdqa %xmm0,0x60(%rax,%rdx,1)
   8bdbd:	movdqa %xmm0,0x50(%rax,%rdx,1)
   8bdc3:	movdqa %xmm0,0x40(%rax,%rdx,1)
   8bdc9:	movdqa %xmm0,0x30(%rax,%rdx,1)
   8bdcf:	movdqa %xmm0,0x20(%rax,%rdx,1)
   8bdd5:	movdqa %xmm0,0x10(%rax,%rdx,1)
   8bddb:	movdqa %xmm0,(%rax,%rdx,1)
   8bde0:	lea    0x1(%r9),%rax
   8bde4:	mov    %rax,0x10(%r10)
   8bde8:	mov    $0xffffffff,%r8d
   8bdee:	cmp    %r13,%rbx
   8bdf1:	jae    8be10 <<c30walk::ctor2s::W<false>>::finish+0x310>
   8bdf3:	lea    0x1(%rbx),%rax
   8bdf7:	mov    0x10(%rbp),%r8
   8bdfb:	cmp    %r8,%rax
   8bdfe:	jae    8c0cb <<c30walk::ctor2s::W<false>>::finish+0x5cb>
   8be04:	shl    $0xa,%rax
   8be08:	add    0x8(%rbp),%rax
   8be0c:	mov    (%rax,%rsi,4),%r8d
   8be10:	mov    %r9,0x8(%rsp)
   8be15:	test   %r14,%r14
   8be18:	je     8bec5 <<c30walk::ctor2s::W<false>>::finish+0x3c5>
   8be1e:	add    0x8(%r10),%rdx
   8be22:	lea    0x800(,%rsi,4),%r9
   8be2a:	xor    %r10d,%r10d
   8be2d:	nopl   (%rax)
   8be30:	lea    (%r12,%r10,1),%r11d
   8be34:	cmp    %r8d,%r11d
   8be37:	jae    8be70 <<c30walk::ctor2s::W<false>>::finish+0x370>
   8be39:	lea    (%r10,%r15,1),%rax
   8be3d:	cmp    $0x20,%rax
   8be41:	jae    8c076 <<c30walk::ctor2s::W<false>>::finish+0x576>
   8be47:	movzwl (%rdi,%r10,2),%r11d
   8be4c:	inc    %r10
   8be4f:	mov    %ebx,%r14d
   8be52:	shl    $0x18,%r14d
   8be56:	shl    $0x8,%r11d
   8be5a:	or     %r14d,%r11d
   8be5d:	or     %esi,%r11d
   8be60:	mov    %r11d,(%rdx,%rax,4)
   8be64:	cmp    0x10(%rsp),%r10
   8be69:	jne    8be30 <<c30walk::ctor2s::W<false>>::finish+0x330>
   8be6b:	jmp    8bec5 <<c30walk::ctor2s::W<false>>::finish+0x3c5>
   8be6d:	nopl   (%rax)
   8be70:	mov    %rbx,%r14
   8be73:	shl    $0xa,%r14
   8be77:	add    %r9,%r14
   8be7a:	mov    %rbx,%rax
   8be7d:	jmp    8be97 <<c30walk::ctor2s::W<false>>::finish+0x397>
   8be7f:	nop
   8be80:	mov    0x8(%rbp),%rax
   8be84:	mov    (%rax,%r14,1),%r8d
   8be88:	add    $0x400,%r14
   8be8f:	mov    %rbx,%rax
   8be92:	cmp    %r8d,%r11d
   8be95:	jb     8be39 <<c30walk::ctor2s::W<false>>::finish+0x339>
   8be97:	lea    0x1(%rax),%rbx
   8be9b:	mov    $0xffffffff,%r8d
   8bea1:	cmp    %r13,%rbx
   8bea4:	jae    8be88 <<c30walk::ctor2s::W<false>>::finish+0x388>
   8bea6:	add    $0x2,%rax
   8beaa:	mov    0x10(%rbp),%r8
   8beae:	cmp    %r8,%rax
   8beb1:	jb     8be80 <<c30walk::ctor2s::W<false>>::finish+0x380>
   8beb3:	lea    0xa4c86(%rip),%rdx        # 130b40 <__frame_dummy_init_array_entry+0x2030>
   8beba:	mov    %rax,%rdi
   8bebd:	mov    %r8,%rsi
   8bec0:	call   d8123 <core::panicking::panic_bounds_check>
   8bec5:	mov    0x30(%rsp),%r8
   8beca:	mov    0x28(%r8),%rax
   8bece:	mov    %rcx,%rdx
   8bed1:	shl    $0x6,%rdx
   8bed5:	mov    0x8(%rsp),%rsi
   8beda:	or     $0x10000000,%esi
   8bee0:	pcmpeqd %xmm0,%xmm0
   8bee4:	movdqu %xmm0,0x2c(%rax,%rdx,1)
   8beea:	movdqa %xmm0,0x20(%rax,%rdx,1)
   8bef0:	movdqa %xmm0,0x10(%rax,%rdx,1)
   8bef6:	movdqa %xmm0,(%rax,%rdx,1)
   8befb:	mov    %esi,0x3c(%rax,%rdx,1)
   8beff:	mov    %ecx,%edi
   8bf01:	shr    $0x6,%edi
   8bf04:	mov    0x10(%r8),%rsi
   8bf08:	cmp    %rsi,%rdi
   8bf0b:	jae    8c0bf <<c30walk::ctor2s::W<false>>::finish+0x5bf>
   8bf11:	mov    $0x1,%eax
   8bf16:	shl    %cl,%rax
   8bf19:	mov    0x8(%r8),%rcx
   8bf1d:	or     %rax,(%rcx,%rdi,8)
   8bf21:	jmp    8c0eb <<c30walk::ctor2s::W<false>>::finish+0x5eb>
   8bf26:	mov    0x8(%rsp),%rcx
   8bf2b:	cmp    $0x8,%rcx
   8bf2f:	jae    8bf38 <<c30walk::ctor2s::W<false>>::finish+0x438>
   8bf31:	xor    %ecx,%ecx
   8bf33:	jmp    8c0e2 <<c30walk::ctor2s::W<false>>::finish+0x5e2>
   8bf38:	movabs $0x7ffffffffffffff0,%rsi
   8bf42:	cmp    $0x20,%rcx
   8bf46:	jae    8bf4c <<c30walk::ctor2s::W<false>>::finish+0x44c>
   8bf48:	xor    %ecx,%ecx
   8bf4a:	jmp    8bfab <<c30walk::ctor2s::W<false>>::finish+0x4ab>
   8bf4c:	mov    %r14,%rcx
   8bf4f:	and    %rsi,%rcx
   8bf52:	movd   %eax,%xmm0
   8bf56:	pshuflw $0x0,%xmm0,%xmm0
   8bf5b:	pshufd $0x44,%xmm0,%xmm0
   8bf60:	xor    %edx,%edx
   8bf62:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   8bf70:	movdqu (%rdi,%rdx,2),%xmm1
   8bf75:	movdqu 0x10(%rdi,%rdx,2),%xmm2
   8bf7b:	psubw  %xmm0,%xmm1
   8bf7f:	psubw  %xmm0,%xmm2
   8bf83:	movdqu %xmm1,(%rdi,%rdx,2)
   8bf88:	movdqu %xmm2,0x10(%rdi,%rdx,2)
   8bf8e:	add    $0x10,%rdx
   8bf92:	cmp    %rdx,%rcx
   8bf95:	jne    8bf70 <<c30walk::ctor2s::W<false>>::finish+0x470>
   8bf97:	cmp    %rcx,%r14
   8bf9a:	je     8c0eb <<c30walk::ctor2s::W<false>>::finish+0x5eb>
   8bfa0:	testb  $0x18,0x8(%rsp)
   8bfa5:	je     8c0e2 <<c30walk::ctor2s::W<false>>::finish+0x5e2>
   8bfab:	mov    %rcx,%rdx
   8bfae:	add    $0xc,%rsi
   8bfb2:	mov    %rsi,%rcx
   8bfb5:	and    %r14,%rcx
   8bfb8:	movd   %eax,%xmm0
   8bfbc:	pshuflw $0x0,%xmm0,%xmm0
   8bfc1:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   8bfd0:	movq   (%rdi,%rdx,2),%xmm1
   8bfd5:	psubw  %xmm0,%xmm1
   8bfd9:	movq   %xmm1,(%rdi,%rdx,2)
   8bfde:	add    $0x4,%rdx
   8bfe2:	cmp    %rdx,%rcx
   8bfe5:	jne    8bfd0 <<c30walk::ctor2s::W<false>>::finish+0x4d0>
   8bfe7:	jmp    8c0dd <<c30walk::ctor2s::W<false>>::finish+0x5dd>
   8bfec:	mov    %rdi,0x8(%rsp)
   8bff1:	mov    %r10,%rdi
   8bff4:	mov    %rsi,0x20(%rsp)
   8bff9:	mov    %rcx,0x28(%rsp)
   8bffe:	mov    %r9,%r14
   8c001:	mov    %r10,0x18(%rsp)
   8c006:	call   91340 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   8c00b:	mov    0x18(%rsp),%r10
   8c010:	mov    %r14,%r9
   8c013:	mov    0x28(%rsp),%rcx
   8c018:	mov    0x20(%rsp),%rsi
   8c01d:	mov    0x10(%rsp),%r14
   8c022:	mov    0x8(%rsp),%rdi
   8c027:	jmp    8bda2 <<c30walk::ctor2s::W<false>>::finish+0x2a2>
   8c02c:	lea    0xa4b0d(%rip),%rdx        # 130b40 <__frame_dummy_init_array_entry+0x2030>
   8c033:	mov    %rcx,%rdi
   8c036:	mov    %r8,%rsi
   8c039:	call   d8123 <core::panicking::panic_bounds_check>
   8c03e:	lea    0xa4c03(%rip),%rdx        # 130c48 <__frame_dummy_init_array_entry+0x2138>
   8c045:	mov    %rbx,%rdi
   8c048:	mov    %r8,%rsi
   8c04b:	call   d8123 <core::panicking::panic_bounds_check>
   8c050:	lea    0xa4bf1(%rip),%rdx        # 130c48 <__frame_dummy_init_array_entry+0x2138>
   8c057:	mov    %r13,%rdi
   8c05a:	mov    %r8,%rsi
   8c05d:	call   d8123 <core::panicking::panic_bounds_check>
   8c062:	lea    0xa4abf(%rip),%rdx        # 130b28 <__frame_dummy_init_array_entry+0x2018>
   8c069:	mov    $0x20,%esi
   8c06e:	mov    %rcx,%rdi
   8c071:	call   d8123 <core::panicking::panic_bounds_check>
   8c076:	lea    0xa4aab(%rip),%rdx        # 130b28 <__frame_dummy_init_array_entry+0x2018>
   8c07d:	mov    $0x20,%esi
   8c082:	mov    %rax,%rdi
   8c085:	call   d8123 <core::panicking::panic_bounds_check>
   8c08a:	lea    0xa4b27(%rip),%rdx        # 130bb8 <__frame_dummy_init_array_entry+0x20a8>
   8c091:	mov    %r13,%rdi
   8c094:	xor    %esi,%esi
   8c096:	call   d8123 <core::panicking::panic_bounds_check>
   8c09b:	lea    0xa4a6e(%rip),%rdx        # 130b10 <__frame_dummy_init_array_entry+0x2000>
   8c0a2:	mov    %rax,%rdi
   8c0a5:	mov    %rcx,%rsi
   8c0a8:	call   d8123 <core::panicking::panic_bounds_check>
   8c0ad:	lea    0xa4a44(%rip),%rdx        # 130af8 <__frame_dummy_init_array_entry+0x1fe8>
   8c0b4:	mov    %rcx,%rdi
   8c0b7:	mov    %r8,%rsi
   8c0ba:	call   d8123 <core::panicking::panic_bounds_check>
   8c0bf:	lea    0xa4b0a(%rip),%rdx        # 130bd0 <__frame_dummy_init_array_entry+0x20c0>
   8c0c6:	call   d8123 <core::panicking::panic_bounds_check>
   8c0cb:	lea    0xa4a26(%rip),%rdx        # 130af8 <__frame_dummy_init_array_entry+0x1fe8>
   8c0d2:	mov    %rax,%rdi
   8c0d5:	mov    %r8,%rsi
   8c0d8:	call   d8123 <core::panicking::panic_bounds_check>
   8c0dd:	cmp    %rcx,%r14
   8c0e0:	je     8c0eb <<c30walk::ctor2s::W<false>>::finish+0x5eb>
   8c0e2:	sub    %ax,(%rdi,%rcx,2)
   8c0e6:	inc    %rcx
   8c0e9:	jmp    8c0dd <<c30walk::ctor2s::W<false>>::finish+0x5dd>
   8c0eb:	add    $0x38,%rsp
   8c0ef:	pop    %rbx
   8c0f0:	pop    %r12
   8c0f2:	pop    %r13
   8c0f4:	pop    %r14
   8c0f6:	pop    %r15
   8c0f8:	pop    %rbp
   8c0f9:	ret
