# <c30walk::ctor3::W<false>>::finish_slow
# size 0x5f8

target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

0000000000080a90 <<c30walk::ctor3::W<false>>::finish_slow>:
   80a90:	push   %rbp
   80a91:	push   %r15
   80a93:	push   %r14
   80a95:	push   %r13
   80a97:	push   %r12
   80a99:	push   %rbx
   80a9a:	sub    $0x38,%rsp
   80a9e:	mov    0x18(%rdi),%rbp
   80aa2:	mov    0x10(%rbp),%r8
   80aa6:	mov    %r8,%r13
   80aa9:	sub    $0x1,%r13
   80aad:	jb     8101a <<c30walk::ctor3::W<false>>::finish_slow+0x58a>
   80ab3:	mov    %rdi,%r11
   80ab6:	lea    (%rsi,%rsi,4),%rax
   80aba:	mov    0x40(%rdi,%rax,8),%r14
   80abf:	mov    0x50(%rdi,%rax,8),%rdi
   80ac4:	mov    0x58(%r11,%rax,8),%r12d
   80ac9:	mov    0x60(%r11,%rax,8),%ecx
   80ace:	mov    0x64(%r11,%rax,8),%eax
   80ad3:	sub    %rdi,%r14
   80ad6:	mov    %r14,0x8(%rsp)
   80adb:	shr    $1,%r14
   80ade:	mov    0x8(%rbp),%r10
   80ae2:	mov    %r13,%rdx
   80ae5:	shl    $0xa,%rdx
   80ae9:	add    %r10,%rdx
   80aec:	mov    %r13,%rbx
   80aef:	cmp    %r12d,(%rdx,%rsi,4)
   80af3:	jbe    80b68 <<c30walk::ctor3::W<false>>::finish_slow+0xd8>
   80af5:	mov    %r11,%r15
   80af8:	lea    (%r12,%r14,1),%edx
   80afc:	dec    %edx
   80afe:	mov    %r8,%r9
   80b01:	shl    $0xa,%r9
   80b05:	lea    (%r9,%rsi,4),%r9
   80b09:	lea    (%r10,%r9,1),%r11
   80b0d:	add    $0xfffffffffffffc00,%r11
   80b14:	mov    %r8,%rbx
   80b17:	nopw   0x0(%rax,%rax,1)
   80b20:	dec    %rbx
   80b23:	cmp    %r8,%rbx
   80b26:	jae    80fce <<c30walk::ctor3::W<false>>::finish_slow+0x53e>
   80b2c:	cmp    %r12d,(%r11)
   80b2f:	lea    -0x400(%r11),%r11
   80b36:	ja     80b20 <<c30walk::ctor3::W<false>>::finish_slow+0x90>
   80b38:	add    %r10,%r9
   80b3b:	add    $0xfffffffffffffc00,%r9
   80b42:	mov    %r8,%r13
   80b45:	mov    %r15,%r11
   80b48:	nopl   0x0(%rax,%rax,1)
   80b50:	dec    %r13
   80b53:	cmp    %r8,%r13
   80b56:	jae    80fe0 <<c30walk::ctor3::W<false>>::finish_slow+0x550>
   80b5c:	cmp    %edx,(%r9)
   80b5f:	lea    -0x400(%r9),%r9
   80b66:	ja     80b50 <<c30walk::ctor3::W<false>>::finish_slow+0xc0>
   80b68:	imul   $0xffffffffffffffe2,%rcx,%r15
   80b6c:	add    %r12,%r15
   80b6f:	test   %eax,%eax
   80b71:	mov    %r14,0x10(%rsp)
   80b76:	js     80c7b <<c30walk::ctor3::W<false>>::finish_slow+0x1eb>
   80b7c:	mov    $0xffffffff,%r9d
   80b82:	cmp    %r13,%rbx
   80b85:	jae    80b9f <<c30walk::ctor3::W<false>>::finish_slow+0x10f>
   80b87:	lea    0x1(%rbx),%rcx
   80b8b:	cmp    %r8,%rcx
   80b8e:	jae    8103d <<c30walk::ctor3::W<false>>::finish_slow+0x5ad>
   80b94:	shl    $0xa,%rcx
   80b98:	add    %rcx,%r10
   80b9b:	mov    (%r10,%rsi,4),%r9d
   80b9f:	mov    (%r11),%rdx
   80ba2:	mov    0x10(%rdx),%rcx
   80ba6:	cmp    %rax,%rcx
   80ba9:	jbe    8102b <<c30walk::ctor3::W<false>>::finish_slow+0x59b>
   80baf:	test   %r14,%r14
   80bb2:	je     81079 <<c30walk::ctor3::W<false>>::finish_slow+0x5e9>
   80bb8:	shl    $0x7,%rax
   80bbc:	add    0x8(%rdx),%rax
   80bc0:	lea    0x800(,%rsi,4),%rdx
   80bc8:	xor    %r11d,%r11d
   80bcb:	nopl   0x0(%rax,%rax,1)
   80bd0:	lea    (%r12,%r11,1),%r10d
   80bd4:	cmp    %r9d,%r10d
   80bd7:	jae    80c10 <<c30walk::ctor3::W<false>>::finish_slow+0x180>
   80bd9:	lea    (%r11,%r15,1),%rcx
   80bdd:	cmp    $0x20,%rcx
   80be1:	jae    80ff2 <<c30walk::ctor3::W<false>>::finish_slow+0x562>
   80be7:	movzwl (%rdi,%r11,2),%r8d
   80bec:	inc    %r11
   80bef:	mov    %ebx,%r10d
   80bf2:	shl    $0x18,%r10d
   80bf6:	shl    $0x8,%r8d
   80bfa:	or     %r10d,%r8d
   80bfd:	or     %esi,%r8d
   80c00:	mov    %r8d,(%rax,%rcx,4)
   80c04:	cmp    0x10(%rsp),%r11
   80c09:	jne    80bd0 <<c30walk::ctor3::W<false>>::finish_slow+0x140>
   80c0b:	jmp    80c59 <<c30walk::ctor3::W<false>>::finish_slow+0x1c9>
   80c0d:	nopl   (%rax)
   80c10:	mov    %rbx,%r14
   80c13:	shl    $0xa,%r14
   80c17:	add    %rdx,%r14
   80c1a:	mov    %rbx,%rcx
   80c1d:	jmp    80c2f <<c30walk::ctor3::W<false>>::finish_slow+0x19f>
   80c1f:	nop
   80c20:	add    $0x400,%r14
   80c27:	mov    %rbx,%rcx
   80c2a:	cmp    %r9d,%r10d
   80c2d:	jb     80bd9 <<c30walk::ctor3::W<false>>::finish_slow+0x149>
   80c2f:	lea    0x1(%rcx),%rbx
   80c33:	mov    $0xffffffff,%r9d
   80c39:	cmp    %r13,%rbx
   80c3c:	jae    80c20 <<c30walk::ctor3::W<false>>::finish_slow+0x190>
   80c3e:	add    $0x2,%rcx
   80c42:	mov    0x10(%rbp),%r8
   80c46:	cmp    %r8,%rcx
   80c49:	jae    80fbc <<c30walk::ctor3::W<false>>::finish_slow+0x52c>
   80c4f:	mov    0x8(%rbp),%rcx
   80c53:	mov    (%rcx,%r14,1),%r9d
   80c57:	jmp    80c20 <<c30walk::ctor3::W<false>>::finish_slow+0x190>
   80c59:	mov    0x8(%rsp),%rdx
   80c5e:	and    $0xfffffffffffffffe,%rdx
   80c62:	mov    $0xff,%esi
   80c67:	add    $0x38,%rsp
   80c6b:	pop    %rbx
   80c6c:	pop    %r12
   80c6e:	pop    %r13
   80c70:	pop    %r14
   80c72:	pop    %r15
   80c74:	pop    %rbp
   80c75:	jmp    *0x9be9d(%rip)        # 11cb18 <memset@GLIBC_2.2.5>
   80c7b:	movzwl (%rdi),%eax
   80c7e:	movzwl -0x2(%rdi,%r14,2),%r8d
   80c84:	mov    %ebx,%edx
   80c86:	shl    $0x10,%edx
   80c89:	or     %eax,%edx
   80c8b:	mov    %r13d,%r9d
   80c8e:	shl    $0x10,%r9d
   80c92:	or     %r8d,%r9d
   80c95:	sub    %edx,%r9d
   80c98:	cmp    $0xfffe,%r9d
   80c9f:	jae    80d1d <<c30walk::ctor3::W<false>>::finish_slow+0x28d>
   80ca1:	mov    0x28(%r11),%r8
   80ca5:	shl    $0x6,%rcx
   80ca9:	shl    $0x8,%edx
   80cac:	or     %edx,%esi
   80cae:	mov    %esi,0x3c(%r8,%rcx,1)
   80cb3:	test   %r14,%r14
   80cb6:	je     81079 <<c30walk::ctor3::W<false>>::finish_slow+0x5e9>
   80cbc:	cmp    $0x1e,%r14
   80cc0:	jne    80eb6 <<c30walk::ctor3::W<false>>::finish_slow+0x426>
   80cc6:	movd   %eax,%xmm0
   80cca:	pshuflw $0x0,%xmm0,%xmm0
   80ccf:	pshufd $0x44,%xmm0,%xmm1
   80cd4:	movdqu (%rdi),%xmm2
   80cd8:	movdqu 0x10(%rdi),%xmm3
   80cdd:	movdqu 0x20(%rdi),%xmm4
   80ce2:	psubw  %xmm1,%xmm2
   80ce6:	movdqu %xmm2,(%rdi)
   80cea:	psubw  %xmm1,%xmm3
   80cee:	movdqu %xmm3,0x10(%rdi)
   80cf3:	psubw  %xmm1,%xmm4
   80cf7:	movdqu %xmm4,0x20(%rdi)
   80cfc:	movq   0x30(%rdi),%xmm1
   80d01:	psubw  %xmm0,%xmm1
   80d05:	movq   %xmm1,0x30(%rdi)
   80d0a:	movd   0x38(%rdi),%xmm1
   80d0f:	psubw  %xmm0,%xmm1
   80d13:	movd   %xmm1,0x38(%rdi)
   80d18:	jmp    81079 <<c30walk::ctor3::W<false>>::finish_slow+0x5e9>
   80d1d:	mov    (%r11),%r10
   80d20:	mov    0x10(%r10),%r9
   80d24:	cmp    (%r10),%r9
   80d27:	mov    %r11,0x30(%rsp)
   80d2c:	je     80f7c <<c30walk::ctor3::W<false>>::finish_slow+0x4ec>
   80d32:	mov    0x8(%r10),%rax
   80d36:	mov    %r9,%rdx
   80d39:	shl    $0x7,%rdx
   80d3d:	pxor   %xmm0,%xmm0
   80d41:	movdqa %xmm0,0x70(%rax,%rdx,1)
   80d47:	movdqa %xmm0,0x60(%rax,%rdx,1)
   80d4d:	movdqa %xmm0,0x50(%rax,%rdx,1)
   80d53:	movdqa %xmm0,0x40(%rax,%rdx,1)
   80d59:	movdqa %xmm0,0x30(%rax,%rdx,1)
   80d5f:	movdqa %xmm0,0x20(%rax,%rdx,1)
   80d65:	movdqa %xmm0,0x10(%rax,%rdx,1)
   80d6b:	movdqa %xmm0,(%rax,%rdx,1)
   80d70:	lea    0x1(%r9),%rax
   80d74:	mov    %rax,0x10(%r10)
   80d78:	mov    $0xffffffff,%r8d
   80d7e:	cmp    %r13,%rbx
   80d81:	jae    80da0 <<c30walk::ctor3::W<false>>::finish_slow+0x310>
   80d83:	lea    0x1(%rbx),%rax
   80d87:	mov    0x10(%rbp),%r8
   80d8b:	cmp    %r8,%rax
   80d8e:	jae    8105b <<c30walk::ctor3::W<false>>::finish_slow+0x5cb>
   80d94:	shl    $0xa,%rax
   80d98:	add    0x8(%rbp),%rax
   80d9c:	mov    (%rax,%rsi,4),%r8d
   80da0:	mov    %r9,0x8(%rsp)
   80da5:	test   %r14,%r14
   80da8:	je     80e55 <<c30walk::ctor3::W<false>>::finish_slow+0x3c5>
   80dae:	add    0x8(%r10),%rdx
   80db2:	lea    0x800(,%rsi,4),%r9
   80dba:	xor    %r10d,%r10d
   80dbd:	nopl   (%rax)
   80dc0:	lea    (%r12,%r10,1),%r11d
   80dc4:	cmp    %r8d,%r11d
   80dc7:	jae    80e00 <<c30walk::ctor3::W<false>>::finish_slow+0x370>
   80dc9:	lea    (%r10,%r15,1),%rax
   80dcd:	cmp    $0x20,%rax
   80dd1:	jae    81006 <<c30walk::ctor3::W<false>>::finish_slow+0x576>
   80dd7:	movzwl (%rdi,%r10,2),%r11d
   80ddc:	inc    %r10
   80ddf:	mov    %ebx,%r14d
   80de2:	shl    $0x18,%r14d
   80de6:	shl    $0x8,%r11d
   80dea:	or     %r14d,%r11d
   80ded:	or     %esi,%r11d
   80df0:	mov    %r11d,(%rdx,%rax,4)
   80df4:	cmp    0x10(%rsp),%r10
   80df9:	jne    80dc0 <<c30walk::ctor3::W<false>>::finish_slow+0x330>
   80dfb:	jmp    80e55 <<c30walk::ctor3::W<false>>::finish_slow+0x3c5>
   80dfd:	nopl   (%rax)
   80e00:	mov    %rbx,%r14
   80e03:	shl    $0xa,%r14
   80e07:	add    %r9,%r14
   80e0a:	mov    %rbx,%rax
   80e0d:	jmp    80e27 <<c30walk::ctor3::W<false>>::finish_slow+0x397>
   80e0f:	nop
   80e10:	mov    0x8(%rbp),%rax
   80e14:	mov    (%rax,%r14,1),%r8d
   80e18:	add    $0x400,%r14
   80e1f:	mov    %rbx,%rax
   80e22:	cmp    %r8d,%r11d
   80e25:	jb     80dc9 <<c30walk::ctor3::W<false>>::finish_slow+0x339>
   80e27:	lea    0x1(%rax),%rbx
   80e2b:	mov    $0xffffffff,%r8d
   80e31:	cmp    %r13,%rbx
   80e34:	jae    80e18 <<c30walk::ctor3::W<false>>::finish_slow+0x388>
   80e36:	add    $0x2,%rax
   80e3a:	mov    0x10(%rbp),%r8
   80e3e:	cmp    %r8,%rax
   80e41:	jb     80e10 <<c30walk::ctor3::W<false>>::finish_slow+0x380>
   80e43:	lea    0x97ede(%rip),%rdx        # 118d28 <__frame_dummy_init_array_entry+0x1cf8>
   80e4a:	mov    %rax,%rdi
   80e4d:	mov    %r8,%rsi
   80e50:	call   c1643 <core::panicking::panic_bounds_check>
   80e55:	mov    0x30(%rsp),%r8
   80e5a:	mov    0x28(%r8),%rax
   80e5e:	mov    %rcx,%rdx
   80e61:	shl    $0x6,%rdx
   80e65:	mov    0x8(%rsp),%rsi
   80e6a:	or     $0x10000000,%esi
   80e70:	pcmpeqd %xmm0,%xmm0
   80e74:	movdqu %xmm0,0x2c(%rax,%rdx,1)
   80e7a:	movdqa %xmm0,0x20(%rax,%rdx,1)
   80e80:	movdqa %xmm0,0x10(%rax,%rdx,1)
   80e86:	movdqa %xmm0,(%rax,%rdx,1)
   80e8b:	mov    %esi,0x3c(%rax,%rdx,1)
   80e8f:	mov    %ecx,%edi
   80e91:	shr    $0x6,%edi
   80e94:	mov    0x10(%r8),%rsi
   80e98:	cmp    %rsi,%rdi
   80e9b:	jae    8104f <<c30walk::ctor3::W<false>>::finish_slow+0x5bf>
   80ea1:	mov    $0x1,%eax
   80ea6:	shl    %cl,%rax
   80ea9:	mov    0x8(%r8),%rcx
   80ead:	or     %rax,(%rcx,%rdi,8)
   80eb1:	jmp    81079 <<c30walk::ctor3::W<false>>::finish_slow+0x5e9>
   80eb6:	mov    0x8(%rsp),%rcx
   80ebb:	cmp    $0x8,%rcx
   80ebf:	jae    80ec8 <<c30walk::ctor3::W<false>>::finish_slow+0x438>
   80ec1:	xor    %ecx,%ecx
   80ec3:	jmp    8106d <<c30walk::ctor3::W<false>>::finish_slow+0x5dd>
   80ec8:	movabs $0x7ffffffffffffff0,%rsi
   80ed2:	cmp    $0x20,%rcx
   80ed6:	jae    80edc <<c30walk::ctor3::W<false>>::finish_slow+0x44c>
   80ed8:	xor    %ecx,%ecx
   80eda:	jmp    80f3b <<c30walk::ctor3::W<false>>::finish_slow+0x4ab>
   80edc:	mov    %r14,%rcx
   80edf:	and    %rsi,%rcx
   80ee2:	movd   %eax,%xmm0
   80ee6:	pshuflw $0x0,%xmm0,%xmm0
   80eeb:	pshufd $0x44,%xmm0,%xmm0
   80ef0:	xor    %edx,%edx
   80ef2:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   80f00:	movdqu (%rdi,%rdx,2),%xmm1
   80f05:	movdqu 0x10(%rdi,%rdx,2),%xmm2
   80f0b:	psubw  %xmm0,%xmm1
   80f0f:	psubw  %xmm0,%xmm2
   80f13:	movdqu %xmm1,(%rdi,%rdx,2)
   80f18:	movdqu %xmm2,0x10(%rdi,%rdx,2)
   80f1e:	add    $0x10,%rdx
   80f22:	cmp    %rdx,%rcx
   80f25:	jne    80f00 <<c30walk::ctor3::W<false>>::finish_slow+0x470>
   80f27:	cmp    %rcx,%r14
   80f2a:	je     81079 <<c30walk::ctor3::W<false>>::finish_slow+0x5e9>
   80f30:	testb  $0x18,0x8(%rsp)
   80f35:	je     8106d <<c30walk::ctor3::W<false>>::finish_slow+0x5dd>
   80f3b:	mov    %rcx,%rdx
   80f3e:	add    $0xc,%rsi
   80f42:	mov    %rsi,%rcx
   80f45:	and    %r14,%rcx
   80f48:	movd   %eax,%xmm0
   80f4c:	pshuflw $0x0,%xmm0,%xmm0
   80f51:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   80f60:	movq   (%rdi,%rdx,2),%xmm1
   80f65:	psubw  %xmm0,%xmm1
   80f69:	movq   %xmm1,(%rdi,%rdx,2)
   80f6e:	add    $0x4,%rdx
   80f72:	cmp    %rdx,%rcx
   80f75:	jne    80f60 <<c30walk::ctor3::W<false>>::finish_slow+0x4d0>
   80f77:	jmp    81074 <<c30walk::ctor3::W<false>>::finish_slow+0x5e4>
   80f7c:	mov    %rdi,0x8(%rsp)
   80f81:	mov    %r10,%rdi
   80f84:	mov    %rsi,0x20(%rsp)
   80f89:	mov    %rcx,0x28(%rsp)
   80f8e:	mov    %r9,%r14
   80f91:	mov    %r10,0x18(%rsp)
   80f96:	call   87070 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   80f9b:	mov    0x18(%rsp),%r10
   80fa0:	mov    %r14,%r9
   80fa3:	mov    0x28(%rsp),%rcx
   80fa8:	mov    0x20(%rsp),%rsi
   80fad:	mov    0x10(%rsp),%r14
   80fb2:	mov    0x8(%rsp),%rdi
   80fb7:	jmp    80d32 <<c30walk::ctor3::W<false>>::finish_slow+0x2a2>
   80fbc:	lea    0x97d65(%rip),%rdx        # 118d28 <__frame_dummy_init_array_entry+0x1cf8>
   80fc3:	mov    %rcx,%rdi
   80fc6:	mov    %r8,%rsi
   80fc9:	call   c1643 <core::panicking::panic_bounds_check>
   80fce:	lea    0x97e2b(%rip),%rdx        # 118e00 <__frame_dummy_init_array_entry+0x1dd0>
   80fd5:	mov    %rbx,%rdi
   80fd8:	mov    %r8,%rsi
   80fdb:	call   c1643 <core::panicking::panic_bounds_check>
   80fe0:	lea    0x97e19(%rip),%rdx        # 118e00 <__frame_dummy_init_array_entry+0x1dd0>
   80fe7:	mov    %r13,%rdi
   80fea:	mov    %r8,%rsi
   80fed:	call   c1643 <core::panicking::panic_bounds_check>
   80ff2:	lea    0x97d17(%rip),%rdx        # 118d10 <__frame_dummy_init_array_entry+0x1ce0>
   80ff9:	mov    $0x20,%esi
   80ffe:	mov    %rcx,%rdi
   81001:	call   c1643 <core::panicking::panic_bounds_check>
   81006:	lea    0x97d03(%rip),%rdx        # 118d10 <__frame_dummy_init_array_entry+0x1ce0>
   8100d:	mov    $0x20,%esi
   81012:	mov    %rax,%rdi
   81015:	call   c1643 <core::panicking::panic_bounds_check>
   8101a:	lea    0x97c8f(%rip),%rdx        # 118cb0 <__frame_dummy_init_array_entry+0x1c80>
   81021:	mov    %r13,%rdi
   81024:	xor    %esi,%esi
   81026:	call   c1643 <core::panicking::panic_bounds_check>
   8102b:	lea    0x97cc6(%rip),%rdx        # 118cf8 <__frame_dummy_init_array_entry+0x1cc8>
   81032:	mov    %rax,%rdi
   81035:	mov    %rcx,%rsi
   81038:	call   c1643 <core::panicking::panic_bounds_check>
   8103d:	lea    0x97c9c(%rip),%rdx        # 118ce0 <__frame_dummy_init_array_entry+0x1cb0>
   81044:	mov    %rcx,%rdi
   81047:	mov    %r8,%rsi
   8104a:	call   c1643 <core::panicking::panic_bounds_check>
   8104f:	lea    0x97c72(%rip),%rdx        # 118cc8 <__frame_dummy_init_array_entry+0x1c98>
   81056:	call   c1643 <core::panicking::panic_bounds_check>
   8105b:	lea    0x97c7e(%rip),%rdx        # 118ce0 <__frame_dummy_init_array_entry+0x1cb0>
   81062:	mov    %rax,%rdi
   81065:	mov    %r8,%rsi
   81068:	call   c1643 <core::panicking::panic_bounds_check>
   8106d:	sub    %ax,(%rdi,%rcx,2)
   81071:	inc    %rcx
   81074:	cmp    %rcx,%r14
   81077:	jne    8106d <<c30walk::ctor3::W<false>>::finish_slow+0x5dd>
   81079:	add    $0x38,%rsp
   8107d:	pop    %rbx
   8107e:	pop    %r12
   81080:	pop    %r13
   81082:	pop    %r14
   81084:	pop    %r15
   81086:	pop    %rbp
   81087:	ret
