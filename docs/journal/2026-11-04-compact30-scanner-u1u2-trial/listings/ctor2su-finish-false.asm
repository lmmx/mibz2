# <c30walk::ctor2su::W<false>>::finish
# size 0x5fa

target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

00000000000968c0 <<c30walk::ctor2su::W<false>>::finish>:
   968c0:	push   %rbp
   968c1:	push   %r15
   968c3:	push   %r14
   968c5:	push   %r13
   968c7:	push   %r12
   968c9:	push   %rbx
   968ca:	sub    $0x38,%rsp
   968ce:	mov    0x18(%rdi),%rbp
   968d2:	mov    0x10(%rbp),%r8
   968d6:	mov    %r8,%r13
   968d9:	sub    $0x1,%r13
   968dd:	jb     96e4a <<c30walk::ctor2su::W<false>>::finish+0x58a>
   968e3:	mov    %rdi,%r11
   968e6:	lea    (%rsi,%rsi,4),%rax
   968ea:	mov    0x40(%rdi,%rax,8),%r14
   968ef:	mov    0x50(%rdi,%rax,8),%rdi
   968f4:	mov    0x58(%r11,%rax,8),%r12d
   968f9:	mov    0x60(%r11,%rax,8),%ecx
   968fe:	mov    0x64(%r11,%rax,8),%eax
   96903:	sub    %rdi,%r14
   96906:	mov    %r14,0x8(%rsp)
   9690b:	shr    $1,%r14
   9690e:	mov    0x8(%rbp),%r10
   96912:	mov    %r13,%rdx
   96915:	shl    $0xa,%rdx
   96919:	add    %r10,%rdx
   9691c:	mov    %r13,%rbx
   9691f:	cmp    %r12d,(%rdx,%rsi,4)
   96923:	jbe    96998 <<c30walk::ctor2su::W<false>>::finish+0xd8>
   96925:	mov    %r11,%r15
   96928:	lea    (%r12,%r14,1),%edx
   9692c:	dec    %edx
   9692e:	mov    %r8,%r9
   96931:	shl    $0xa,%r9
   96935:	lea    (%r9,%rsi,4),%r9
   96939:	lea    (%r10,%r9,1),%r11
   9693d:	add    $0xfffffffffffffc00,%r11
   96944:	mov    %r8,%rbx
   96947:	nopw   0x0(%rax,%rax,1)
   96950:	dec    %rbx
   96953:	cmp    %r8,%rbx
   96956:	jae    96dfe <<c30walk::ctor2su::W<false>>::finish+0x53e>
   9695c:	cmp    %r12d,(%r11)
   9695f:	lea    -0x400(%r11),%r11
   96966:	ja     96950 <<c30walk::ctor2su::W<false>>::finish+0x90>
   96968:	add    %r10,%r9
   9696b:	add    $0xfffffffffffffc00,%r9
   96972:	mov    %r8,%r13
   96975:	mov    %r15,%r11
   96978:	nopl   0x0(%rax,%rax,1)
   96980:	dec    %r13
   96983:	cmp    %r8,%r13
   96986:	jae    96e10 <<c30walk::ctor2su::W<false>>::finish+0x550>
   9698c:	cmp    %edx,(%r9)
   9698f:	lea    -0x400(%r9),%r9
   96996:	ja     96980 <<c30walk::ctor2su::W<false>>::finish+0xc0>
   96998:	imul   $0xffffffffffffffe2,%rcx,%r15
   9699c:	add    %r12,%r15
   9699f:	cmp    $0xffffffff,%eax
   969a2:	mov    %r14,0x10(%rsp)
   969a7:	je     96aab <<c30walk::ctor2su::W<false>>::finish+0x1eb>
   969ad:	mov    $0xffffffff,%r9d
   969b3:	cmp    %r13,%rbx
   969b6:	jae    969d0 <<c30walk::ctor2su::W<false>>::finish+0x110>
   969b8:	lea    0x1(%rbx),%rcx
   969bc:	cmp    %r8,%rcx
   969bf:	jae    96e6d <<c30walk::ctor2su::W<false>>::finish+0x5ad>
   969c5:	shl    $0xa,%rcx
   969c9:	add    %rcx,%r10
   969cc:	mov    (%r10,%rsi,4),%r9d
   969d0:	mov    (%r11),%rdx
   969d3:	mov    0x10(%rdx),%rcx
   969d7:	cmp    %rax,%rcx
   969da:	jbe    96e5b <<c30walk::ctor2su::W<false>>::finish+0x59b>
   969e0:	test   %r14,%r14
   969e3:	je     96eab <<c30walk::ctor2su::W<false>>::finish+0x5eb>
   969e9:	shl    $0x7,%rax
   969ed:	add    0x8(%rdx),%rax
   969f1:	lea    0x800(,%rsi,4),%rdx
   969f9:	xor    %r11d,%r11d
   969fc:	nopl   0x0(%rax)
   96a00:	lea    (%r12,%r11,1),%r10d
   96a04:	cmp    %r9d,%r10d
   96a07:	jae    96a40 <<c30walk::ctor2su::W<false>>::finish+0x180>
   96a09:	lea    (%r11,%r15,1),%rcx
   96a0d:	cmp    $0x20,%rcx
   96a11:	jae    96e22 <<c30walk::ctor2su::W<false>>::finish+0x562>
   96a17:	movzwl (%rdi,%r11,2),%r8d
   96a1c:	inc    %r11
   96a1f:	mov    %ebx,%r10d
   96a22:	shl    $0x18,%r10d
   96a26:	shl    $0x8,%r8d
   96a2a:	or     %r10d,%r8d
   96a2d:	or     %esi,%r8d
   96a30:	mov    %r8d,(%rax,%rcx,4)
   96a34:	cmp    0x10(%rsp),%r11
   96a39:	jne    96a00 <<c30walk::ctor2su::W<false>>::finish+0x140>
   96a3b:	jmp    96a89 <<c30walk::ctor2su::W<false>>::finish+0x1c9>
   96a3d:	nopl   (%rax)
   96a40:	mov    %rbx,%r14
   96a43:	shl    $0xa,%r14
   96a47:	add    %rdx,%r14
   96a4a:	mov    %rbx,%rcx
   96a4d:	jmp    96a5f <<c30walk::ctor2su::W<false>>::finish+0x19f>
   96a4f:	nop
   96a50:	add    $0x400,%r14
   96a57:	mov    %rbx,%rcx
   96a5a:	cmp    %r9d,%r10d
   96a5d:	jb     96a09 <<c30walk::ctor2su::W<false>>::finish+0x149>
   96a5f:	lea    0x1(%rcx),%rbx
   96a63:	mov    $0xffffffff,%r9d
   96a69:	cmp    %r13,%rbx
   96a6c:	jae    96a50 <<c30walk::ctor2su::W<false>>::finish+0x190>
   96a6e:	add    $0x2,%rcx
   96a72:	mov    0x10(%rbp),%r8
   96a76:	cmp    %r8,%rcx
   96a79:	jae    96dec <<c30walk::ctor2su::W<false>>::finish+0x52c>
   96a7f:	mov    0x8(%rbp),%rcx
   96a83:	mov    (%rcx,%r14,1),%r9d
   96a87:	jmp    96a50 <<c30walk::ctor2su::W<false>>::finish+0x190>
   96a89:	mov    0x8(%rsp),%rdx
   96a8e:	and    $0xfffffffffffffffe,%rdx
   96a92:	mov    $0xff,%esi
   96a97:	add    $0x38,%rsp
   96a9b:	pop    %rbx
   96a9c:	pop    %r12
   96a9e:	pop    %r13
   96aa0:	pop    %r14
   96aa2:	pop    %r15
   96aa4:	pop    %rbp
   96aa5:	jmp    *0xb6b45(%rip)        # 14d5f0 <memset@GLIBC_2.2.5>
   96aab:	movzwl (%rdi),%eax
   96aae:	movzwl -0x2(%rdi,%r14,2),%r8d
   96ab4:	mov    %ebx,%edx
   96ab6:	shl    $0x10,%edx
   96ab9:	or     %eax,%edx
   96abb:	mov    %r13d,%r9d
   96abe:	shl    $0x10,%r9d
   96ac2:	or     %r8d,%r9d
   96ac5:	sub    %edx,%r9d
   96ac8:	cmp    $0xfffe,%r9d
   96acf:	jae    96b4d <<c30walk::ctor2su::W<false>>::finish+0x28d>
   96ad1:	mov    0x28(%r11),%r8
   96ad5:	shl    $0x6,%rcx
   96ad9:	shl    $0x8,%edx
   96adc:	or     %edx,%esi
   96ade:	mov    %esi,0x3c(%r8,%rcx,1)
   96ae3:	test   %r14,%r14
   96ae6:	je     96eab <<c30walk::ctor2su::W<false>>::finish+0x5eb>
   96aec:	cmp    $0x1e,%r14
   96af0:	jne    96ce6 <<c30walk::ctor2su::W<false>>::finish+0x426>
   96af6:	movd   %eax,%xmm0
   96afa:	pshuflw $0x0,%xmm0,%xmm0
   96aff:	pshufd $0x44,%xmm0,%xmm1
   96b04:	movdqu (%rdi),%xmm2
   96b08:	movdqu 0x10(%rdi),%xmm3
   96b0d:	movdqu 0x20(%rdi),%xmm4
   96b12:	psubw  %xmm1,%xmm2
   96b16:	movdqu %xmm2,(%rdi)
   96b1a:	psubw  %xmm1,%xmm3
   96b1e:	movdqu %xmm3,0x10(%rdi)
   96b23:	psubw  %xmm1,%xmm4
   96b27:	movdqu %xmm4,0x20(%rdi)
   96b2c:	movq   0x30(%rdi),%xmm1
   96b31:	psubw  %xmm0,%xmm1
   96b35:	movq   %xmm1,0x30(%rdi)
   96b3a:	movd   0x38(%rdi),%xmm1
   96b3f:	psubw  %xmm0,%xmm1
   96b43:	movd   %xmm1,0x38(%rdi)
   96b48:	jmp    96eab <<c30walk::ctor2su::W<false>>::finish+0x5eb>
   96b4d:	mov    (%r11),%r10
   96b50:	mov    0x10(%r10),%r9
   96b54:	cmp    (%r10),%r9
   96b57:	mov    %r11,0x30(%rsp)
   96b5c:	je     96dac <<c30walk::ctor2su::W<false>>::finish+0x4ec>
   96b62:	mov    0x8(%r10),%rax
   96b66:	mov    %r9,%rdx
   96b69:	shl    $0x7,%rdx
   96b6d:	pxor   %xmm0,%xmm0
   96b71:	movdqa %xmm0,0x70(%rax,%rdx,1)
   96b77:	movdqa %xmm0,0x60(%rax,%rdx,1)
   96b7d:	movdqa %xmm0,0x50(%rax,%rdx,1)
   96b83:	movdqa %xmm0,0x40(%rax,%rdx,1)
   96b89:	movdqa %xmm0,0x30(%rax,%rdx,1)
   96b8f:	movdqa %xmm0,0x20(%rax,%rdx,1)
   96b95:	movdqa %xmm0,0x10(%rax,%rdx,1)
   96b9b:	movdqa %xmm0,(%rax,%rdx,1)
   96ba0:	lea    0x1(%r9),%rax
   96ba4:	mov    %rax,0x10(%r10)
   96ba8:	mov    $0xffffffff,%r8d
   96bae:	cmp    %r13,%rbx
   96bb1:	jae    96bd0 <<c30walk::ctor2su::W<false>>::finish+0x310>
   96bb3:	lea    0x1(%rbx),%rax
   96bb7:	mov    0x10(%rbp),%r8
   96bbb:	cmp    %r8,%rax
   96bbe:	jae    96e8b <<c30walk::ctor2su::W<false>>::finish+0x5cb>
   96bc4:	shl    $0xa,%rax
   96bc8:	add    0x8(%rbp),%rax
   96bcc:	mov    (%rax,%rsi,4),%r8d
   96bd0:	mov    %r9,0x8(%rsp)
   96bd5:	test   %r14,%r14
   96bd8:	je     96c85 <<c30walk::ctor2su::W<false>>::finish+0x3c5>
   96bde:	add    0x8(%r10),%rdx
   96be2:	lea    0x800(,%rsi,4),%r9
   96bea:	xor    %r10d,%r10d
   96bed:	nopl   (%rax)
   96bf0:	lea    (%r12,%r10,1),%r11d
   96bf4:	cmp    %r8d,%r11d
   96bf7:	jae    96c30 <<c30walk::ctor2su::W<false>>::finish+0x370>
   96bf9:	lea    (%r10,%r15,1),%rax
   96bfd:	cmp    $0x20,%rax
   96c01:	jae    96e36 <<c30walk::ctor2su::W<false>>::finish+0x576>
   96c07:	movzwl (%rdi,%r10,2),%r11d
   96c0c:	inc    %r10
   96c0f:	mov    %ebx,%r14d
   96c12:	shl    $0x18,%r14d
   96c16:	shl    $0x8,%r11d
   96c1a:	or     %r14d,%r11d
   96c1d:	or     %esi,%r11d
   96c20:	mov    %r11d,(%rdx,%rax,4)
   96c24:	cmp    0x10(%rsp),%r10
   96c29:	jne    96bf0 <<c30walk::ctor2su::W<false>>::finish+0x330>
   96c2b:	jmp    96c85 <<c30walk::ctor2su::W<false>>::finish+0x3c5>
   96c2d:	nopl   (%rax)
   96c30:	mov    %rbx,%r14
   96c33:	shl    $0xa,%r14
   96c37:	add    %r9,%r14
   96c3a:	mov    %rbx,%rax
   96c3d:	jmp    96c57 <<c30walk::ctor2su::W<false>>::finish+0x397>
   96c3f:	nop
   96c40:	mov    0x8(%rbp),%rax
   96c44:	mov    (%rax,%r14,1),%r8d
   96c48:	add    $0x400,%r14
   96c4f:	mov    %rbx,%rax
   96c52:	cmp    %r8d,%r11d
   96c55:	jb     96bf9 <<c30walk::ctor2su::W<false>>::finish+0x339>
   96c57:	lea    0x1(%rax),%rbx
   96c5b:	mov    $0xffffffff,%r8d
   96c61:	cmp    %r13,%rbx
   96c64:	jae    96c48 <<c30walk::ctor2su::W<false>>::finish+0x388>
   96c66:	add    $0x2,%rax
   96c6a:	mov    0x10(%rbp),%r8
   96c6e:	cmp    %r8,%rax
   96c71:	jb     96c40 <<c30walk::ctor2su::W<false>>::finish+0x380>
   96c73:	lea    0xb1f1e(%rip),%rdx        # 148b98 <__frame_dummy_init_array_entry+0x2308>
   96c7a:	mov    %rax,%rdi
   96c7d:	mov    %r8,%rsi
   96c80:	call   efe73 <core::panicking::panic_bounds_check>
   96c85:	mov    0x30(%rsp),%r8
   96c8a:	mov    0x28(%r8),%rax
   96c8e:	mov    %rcx,%rdx
   96c91:	shl    $0x6,%rdx
   96c95:	mov    0x8(%rsp),%rsi
   96c9a:	or     $0x10000000,%esi
   96ca0:	pcmpeqd %xmm0,%xmm0
   96ca4:	movdqu %xmm0,0x2c(%rax,%rdx,1)
   96caa:	movdqa %xmm0,0x20(%rax,%rdx,1)
   96cb0:	movdqa %xmm0,0x10(%rax,%rdx,1)
   96cb6:	movdqa %xmm0,(%rax,%rdx,1)
   96cbb:	mov    %esi,0x3c(%rax,%rdx,1)
   96cbf:	mov    %ecx,%edi
   96cc1:	shr    $0x6,%edi
   96cc4:	mov    0x10(%r8),%rsi
   96cc8:	cmp    %rsi,%rdi
   96ccb:	jae    96e7f <<c30walk::ctor2su::W<false>>::finish+0x5bf>
   96cd1:	mov    $0x1,%eax
   96cd6:	shl    %cl,%rax
   96cd9:	mov    0x8(%r8),%rcx
   96cdd:	or     %rax,(%rcx,%rdi,8)
   96ce1:	jmp    96eab <<c30walk::ctor2su::W<false>>::finish+0x5eb>
   96ce6:	mov    0x8(%rsp),%rcx
   96ceb:	cmp    $0x8,%rcx
   96cef:	jae    96cf8 <<c30walk::ctor2su::W<false>>::finish+0x438>
   96cf1:	xor    %ecx,%ecx
   96cf3:	jmp    96ea2 <<c30walk::ctor2su::W<false>>::finish+0x5e2>
   96cf8:	movabs $0x7ffffffffffffff0,%rsi
   96d02:	cmp    $0x20,%rcx
   96d06:	jae    96d0c <<c30walk::ctor2su::W<false>>::finish+0x44c>
   96d08:	xor    %ecx,%ecx
   96d0a:	jmp    96d6b <<c30walk::ctor2su::W<false>>::finish+0x4ab>
   96d0c:	mov    %r14,%rcx
   96d0f:	and    %rsi,%rcx
   96d12:	movd   %eax,%xmm0
   96d16:	pshuflw $0x0,%xmm0,%xmm0
   96d1b:	pshufd $0x44,%xmm0,%xmm0
   96d20:	xor    %edx,%edx
   96d22:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   96d30:	movdqu (%rdi,%rdx,2),%xmm1
   96d35:	movdqu 0x10(%rdi,%rdx,2),%xmm2
   96d3b:	psubw  %xmm0,%xmm1
   96d3f:	psubw  %xmm0,%xmm2
   96d43:	movdqu %xmm1,(%rdi,%rdx,2)
   96d48:	movdqu %xmm2,0x10(%rdi,%rdx,2)
   96d4e:	add    $0x10,%rdx
   96d52:	cmp    %rdx,%rcx
   96d55:	jne    96d30 <<c30walk::ctor2su::W<false>>::finish+0x470>
   96d57:	cmp    %rcx,%r14
   96d5a:	je     96eab <<c30walk::ctor2su::W<false>>::finish+0x5eb>
   96d60:	testb  $0x18,0x8(%rsp)
   96d65:	je     96ea2 <<c30walk::ctor2su::W<false>>::finish+0x5e2>
   96d6b:	mov    %rcx,%rdx
   96d6e:	add    $0xc,%rsi
   96d72:	mov    %rsi,%rcx
   96d75:	and    %r14,%rcx
   96d78:	movd   %eax,%xmm0
   96d7c:	pshuflw $0x0,%xmm0,%xmm0
   96d81:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   96d90:	movq   (%rdi,%rdx,2),%xmm1
   96d95:	psubw  %xmm0,%xmm1
   96d99:	movq   %xmm1,(%rdi,%rdx,2)
   96d9e:	add    $0x4,%rdx
   96da2:	cmp    %rdx,%rcx
   96da5:	jne    96d90 <<c30walk::ctor2su::W<false>>::finish+0x4d0>
   96da7:	jmp    96e9d <<c30walk::ctor2su::W<false>>::finish+0x5dd>
   96dac:	mov    %rdi,0x8(%rsp)
   96db1:	mov    %r10,%rdi
   96db4:	mov    %rsi,0x20(%rsp)
   96db9:	mov    %rcx,0x28(%rsp)
   96dbe:	mov    %r9,%r14
   96dc1:	mov    %r10,0x18(%rsp)
   96dc6:	call   9c170 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   96dcb:	mov    0x18(%rsp),%r10
   96dd0:	mov    %r14,%r9
   96dd3:	mov    0x28(%rsp),%rcx
   96dd8:	mov    0x20(%rsp),%rsi
   96ddd:	mov    0x10(%rsp),%r14
   96de2:	mov    0x8(%rsp),%rdi
   96de7:	jmp    96b62 <<c30walk::ctor2su::W<false>>::finish+0x2a2>
   96dec:	lea    0xb1da5(%rip),%rdx        # 148b98 <__frame_dummy_init_array_entry+0x2308>
   96df3:	mov    %rcx,%rdi
   96df6:	mov    %r8,%rsi
   96df9:	call   efe73 <core::panicking::panic_bounds_check>
   96dfe:	lea    0xb1e9b(%rip),%rdx        # 148ca0 <__frame_dummy_init_array_entry+0x2410>
   96e05:	mov    %rbx,%rdi
   96e08:	mov    %r8,%rsi
   96e0b:	call   efe73 <core::panicking::panic_bounds_check>
   96e10:	lea    0xb1e89(%rip),%rdx        # 148ca0 <__frame_dummy_init_array_entry+0x2410>
   96e17:	mov    %r13,%rdi
   96e1a:	mov    %r8,%rsi
   96e1d:	call   efe73 <core::panicking::panic_bounds_check>
   96e22:	lea    0xb1d57(%rip),%rdx        # 148b80 <__frame_dummy_init_array_entry+0x22f0>
   96e29:	mov    $0x20,%esi
   96e2e:	mov    %rcx,%rdi
   96e31:	call   efe73 <core::panicking::panic_bounds_check>
   96e36:	lea    0xb1d43(%rip),%rdx        # 148b80 <__frame_dummy_init_array_entry+0x22f0>
   96e3d:	mov    $0x20,%esi
   96e42:	mov    %rax,%rdi
   96e45:	call   efe73 <core::panicking::panic_bounds_check>
   96e4a:	lea    0xb1dbf(%rip),%rdx        # 148c10 <__frame_dummy_init_array_entry+0x2380>
   96e51:	mov    %r13,%rdi
   96e54:	xor    %esi,%esi
   96e56:	call   efe73 <core::panicking::panic_bounds_check>
   96e5b:	lea    0xb1d06(%rip),%rdx        # 148b68 <__frame_dummy_init_array_entry+0x22d8>
   96e62:	mov    %rax,%rdi
   96e65:	mov    %rcx,%rsi
   96e68:	call   efe73 <core::panicking::panic_bounds_check>
   96e6d:	lea    0xb1cdc(%rip),%rdx        # 148b50 <__frame_dummy_init_array_entry+0x22c0>
   96e74:	mov    %rcx,%rdi
   96e77:	mov    %r8,%rsi
   96e7a:	call   efe73 <core::panicking::panic_bounds_check>
   96e7f:	lea    0xb1da2(%rip),%rdx        # 148c28 <__frame_dummy_init_array_entry+0x2398>
   96e86:	call   efe73 <core::panicking::panic_bounds_check>
   96e8b:	lea    0xb1cbe(%rip),%rdx        # 148b50 <__frame_dummy_init_array_entry+0x22c0>
   96e92:	mov    %rax,%rdi
   96e95:	mov    %r8,%rsi
   96e98:	call   efe73 <core::panicking::panic_bounds_check>
   96e9d:	cmp    %rcx,%r14
   96ea0:	je     96eab <<c30walk::ctor2su::W<false>>::finish+0x5eb>
   96ea2:	sub    %ax,(%rdi,%rcx,2)
   96ea6:	inc    %rcx
   96ea9:	jmp    96e9d <<c30walk::ctor2su::W<false>>::finish+0x5dd>
   96eab:	add    $0x38,%rsp
   96eaf:	pop    %rbx
   96eb0:	pop    %r12
   96eb2:	pop    %r13
   96eb4:	pop    %r14
   96eb6:	pop    %r15
   96eb8:	pop    %rbp
   96eb9:	ret
