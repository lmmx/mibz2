# <c30walk::ctor2d::W<false>>::finish
# size 0x60a

target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

0000000000094c80 <<c30walk::ctor2d::W<false>>::finish>:
   94c80:	push   %rbp
   94c81:	push   %r15
   94c83:	push   %r14
   94c85:	push   %r13
   94c87:	push   %r12
   94c89:	push   %rbx
   94c8a:	sub    $0x38,%rsp
   94c8e:	mov    0x18(%rdi),%rbp
   94c92:	mov    0x10(%rbp),%r8
   94c96:	mov    %r8,%r13
   94c99:	sub    $0x1,%r13
   94c9d:	jb     9521a <<c30walk::ctor2d::W<false>>::finish+0x59a>
   94ca3:	mov    %rdi,%r11
   94ca6:	lea    (%rsi,%rsi,2),%rax
   94caa:	mov    0x1040(%rdi,%rax,8),%rdi
   94cb2:	mov    0x1048(%r11,%rax,8),%r12d
   94cba:	mov    0x1050(%r11,%rax,8),%ecx
   94cc2:	mov    0x1054(%r11,%rax,8),%eax
   94cca:	mov    0x40(%r11,%rsi,8),%r14
   94ccf:	sub    %rdi,%r14
   94cd2:	mov    %r14,0x8(%rsp)
   94cd7:	shr    $1,%r14
   94cda:	mov    0x8(%rbp),%r10
   94cde:	mov    %r13,%rdx
   94ce1:	shl    $0xa,%rdx
   94ce5:	add    %r10,%rdx
   94ce8:	mov    %r13,%rbx
   94ceb:	cmp    %r12d,(%rdx,%rsi,4)
   94cef:	jbe    94d68 <<c30walk::ctor2d::W<false>>::finish+0xe8>
   94cf1:	mov    %r11,%r15
   94cf4:	lea    (%r12,%r14,1),%edx
   94cf8:	dec    %edx
   94cfa:	mov    %r8,%r9
   94cfd:	shl    $0xa,%r9
   94d01:	lea    (%r9,%rsi,4),%r9
   94d05:	lea    (%r10,%r9,1),%r11
   94d09:	add    $0xfffffffffffffc00,%r11
   94d10:	mov    %r8,%rbx
   94d13:	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   94d20:	dec    %rbx
   94d23:	cmp    %r8,%rbx
   94d26:	jae    951ce <<c30walk::ctor2d::W<false>>::finish+0x54e>
   94d2c:	cmp    %r12d,(%r11)
   94d2f:	lea    -0x400(%r11),%r11
   94d36:	ja     94d20 <<c30walk::ctor2d::W<false>>::finish+0xa0>
   94d38:	add    %r10,%r9
   94d3b:	add    $0xfffffffffffffc00,%r9
   94d42:	mov    %r8,%r13
   94d45:	mov    %r15,%r11
   94d48:	nopl   0x0(%rax,%rax,1)
   94d50:	dec    %r13
   94d53:	cmp    %r8,%r13
   94d56:	jae    951e0 <<c30walk::ctor2d::W<false>>::finish+0x560>
   94d5c:	cmp    %edx,(%r9)
   94d5f:	lea    -0x400(%r9),%r9
   94d66:	ja     94d50 <<c30walk::ctor2d::W<false>>::finish+0xd0>
   94d68:	imul   $0xffffffffffffffe2,%rcx,%r15
   94d6c:	add    %r12,%r15
   94d6f:	cmp    $0xffffffff,%eax
   94d72:	mov    %r14,0x10(%rsp)
   94d77:	je     94e7b <<c30walk::ctor2d::W<false>>::finish+0x1fb>
   94d7d:	mov    $0xffffffff,%r9d
   94d83:	cmp    %r13,%rbx
   94d86:	jae    94da0 <<c30walk::ctor2d::W<false>>::finish+0x120>
   94d88:	lea    0x1(%rbx),%rcx
   94d8c:	cmp    %r8,%rcx
   94d8f:	jae    9523d <<c30walk::ctor2d::W<false>>::finish+0x5bd>
   94d95:	shl    $0xa,%rcx
   94d99:	add    %rcx,%r10
   94d9c:	mov    (%r10,%rsi,4),%r9d
   94da0:	mov    (%r11),%rdx
   94da3:	mov    0x10(%rdx),%rcx
   94da7:	cmp    %rax,%rcx
   94daa:	jbe    9522b <<c30walk::ctor2d::W<false>>::finish+0x5ab>
   94db0:	test   %r14,%r14
   94db3:	je     9527b <<c30walk::ctor2d::W<false>>::finish+0x5fb>
   94db9:	shl    $0x7,%rax
   94dbd:	add    0x8(%rdx),%rax
   94dc1:	lea    0x800(,%rsi,4),%rdx
   94dc9:	xor    %r11d,%r11d
   94dcc:	nopl   0x0(%rax)
   94dd0:	lea    (%r12,%r11,1),%r10d
   94dd4:	cmp    %r9d,%r10d
   94dd7:	jae    94e10 <<c30walk::ctor2d::W<false>>::finish+0x190>
   94dd9:	lea    (%r11,%r15,1),%rcx
   94ddd:	cmp    $0x20,%rcx
   94de1:	jae    951f2 <<c30walk::ctor2d::W<false>>::finish+0x572>
   94de7:	movzwl (%rdi,%r11,2),%r8d
   94dec:	inc    %r11
   94def:	mov    %ebx,%r10d
   94df2:	shl    $0x18,%r10d
   94df6:	shl    $0x8,%r8d
   94dfa:	or     %r10d,%r8d
   94dfd:	or     %esi,%r8d
   94e00:	mov    %r8d,(%rax,%rcx,4)
   94e04:	cmp    0x10(%rsp),%r11
   94e09:	jne    94dd0 <<c30walk::ctor2d::W<false>>::finish+0x150>
   94e0b:	jmp    94e59 <<c30walk::ctor2d::W<false>>::finish+0x1d9>
   94e0d:	nopl   (%rax)
   94e10:	mov    %rbx,%r14
   94e13:	shl    $0xa,%r14
   94e17:	add    %rdx,%r14
   94e1a:	mov    %rbx,%rcx
   94e1d:	jmp    94e2f <<c30walk::ctor2d::W<false>>::finish+0x1af>
   94e1f:	nop
   94e20:	add    $0x400,%r14
   94e27:	mov    %rbx,%rcx
   94e2a:	cmp    %r9d,%r10d
   94e2d:	jb     94dd9 <<c30walk::ctor2d::W<false>>::finish+0x159>
   94e2f:	lea    0x1(%rcx),%rbx
   94e33:	mov    $0xffffffff,%r9d
   94e39:	cmp    %r13,%rbx
   94e3c:	jae    94e20 <<c30walk::ctor2d::W<false>>::finish+0x1a0>
   94e3e:	add    $0x2,%rcx
   94e42:	mov    0x10(%rbp),%r8
   94e46:	cmp    %r8,%rcx
   94e49:	jae    951bc <<c30walk::ctor2d::W<false>>::finish+0x53c>
   94e4f:	mov    0x8(%rbp),%rcx
   94e53:	mov    (%rcx,%r14,1),%r9d
   94e57:	jmp    94e20 <<c30walk::ctor2d::W<false>>::finish+0x1a0>
   94e59:	mov    0x8(%rsp),%rdx
   94e5e:	and    $0xfffffffffffffffe,%rdx
   94e62:	mov    $0xff,%esi
   94e67:	add    $0x38,%rsp
   94e6b:	pop    %rbx
   94e6c:	pop    %r12
   94e6e:	pop    %r13
   94e70:	pop    %r14
   94e72:	pop    %r15
   94e74:	pop    %rbp
   94e75:	jmp    *0xa20c5(%rip)        # 136f40 <memset@GLIBC_2.2.5>
   94e7b:	movzwl (%rdi),%eax
   94e7e:	movzwl -0x2(%rdi,%r14,2),%r8d
   94e84:	mov    %ebx,%edx
   94e86:	shl    $0x10,%edx
   94e89:	or     %eax,%edx
   94e8b:	mov    %r13d,%r9d
   94e8e:	shl    $0x10,%r9d
   94e92:	or     %r8d,%r9d
   94e95:	sub    %edx,%r9d
   94e98:	cmp    $0xfffe,%r9d
   94e9f:	jae    94f1d <<c30walk::ctor2d::W<false>>::finish+0x29d>
   94ea1:	mov    0x28(%r11),%r8
   94ea5:	shl    $0x6,%rcx
   94ea9:	shl    $0x8,%edx
   94eac:	or     %edx,%esi
   94eae:	mov    %esi,0x3c(%r8,%rcx,1)
   94eb3:	test   %r14,%r14
   94eb6:	je     9527b <<c30walk::ctor2d::W<false>>::finish+0x5fb>
   94ebc:	cmp    $0x1e,%r14
   94ec0:	jne    950b6 <<c30walk::ctor2d::W<false>>::finish+0x436>
   94ec6:	movd   %eax,%xmm0
   94eca:	pshuflw $0x0,%xmm0,%xmm0
   94ecf:	pshufd $0x44,%xmm0,%xmm1
   94ed4:	movdqu (%rdi),%xmm2
   94ed8:	movdqu 0x10(%rdi),%xmm3
   94edd:	movdqu 0x20(%rdi),%xmm4
   94ee2:	psubw  %xmm1,%xmm2
   94ee6:	movdqu %xmm2,(%rdi)
   94eea:	psubw  %xmm1,%xmm3
   94eee:	movdqu %xmm3,0x10(%rdi)
   94ef3:	psubw  %xmm1,%xmm4
   94ef7:	movdqu %xmm4,0x20(%rdi)
   94efc:	movq   0x30(%rdi),%xmm1
   94f01:	psubw  %xmm0,%xmm1
   94f05:	movq   %xmm1,0x30(%rdi)
   94f0a:	movd   0x38(%rdi),%xmm1
   94f0f:	psubw  %xmm0,%xmm1
   94f13:	movd   %xmm1,0x38(%rdi)
   94f18:	jmp    9527b <<c30walk::ctor2d::W<false>>::finish+0x5fb>
   94f1d:	mov    (%r11),%r10
   94f20:	mov    0x10(%r10),%r9
   94f24:	cmp    (%r10),%r9
   94f27:	mov    %r11,0x30(%rsp)
   94f2c:	je     9517c <<c30walk::ctor2d::W<false>>::finish+0x4fc>
   94f32:	mov    0x8(%r10),%rax
   94f36:	mov    %r9,%rdx
   94f39:	shl    $0x7,%rdx
   94f3d:	pxor   %xmm0,%xmm0
   94f41:	movdqa %xmm0,0x70(%rax,%rdx,1)
   94f47:	movdqa %xmm0,0x60(%rax,%rdx,1)
   94f4d:	movdqa %xmm0,0x50(%rax,%rdx,1)
   94f53:	movdqa %xmm0,0x40(%rax,%rdx,1)
   94f59:	movdqa %xmm0,0x30(%rax,%rdx,1)
   94f5f:	movdqa %xmm0,0x20(%rax,%rdx,1)
   94f65:	movdqa %xmm0,0x10(%rax,%rdx,1)
   94f6b:	movdqa %xmm0,(%rax,%rdx,1)
   94f70:	lea    0x1(%r9),%rax
   94f74:	mov    %rax,0x10(%r10)
   94f78:	mov    $0xffffffff,%r8d
   94f7e:	cmp    %r13,%rbx
   94f81:	jae    94fa0 <<c30walk::ctor2d::W<false>>::finish+0x320>
   94f83:	lea    0x1(%rbx),%rax
   94f87:	mov    0x10(%rbp),%r8
   94f8b:	cmp    %r8,%rax
   94f8e:	jae    9525b <<c30walk::ctor2d::W<false>>::finish+0x5db>
   94f94:	shl    $0xa,%rax
   94f98:	add    0x8(%rbp),%rax
   94f9c:	mov    (%rax,%rsi,4),%r8d
   94fa0:	mov    %r9,0x8(%rsp)
   94fa5:	test   %r14,%r14
   94fa8:	je     95055 <<c30walk::ctor2d::W<false>>::finish+0x3d5>
   94fae:	add    0x8(%r10),%rdx
   94fb2:	lea    0x800(,%rsi,4),%r9
   94fba:	xor    %r10d,%r10d
   94fbd:	nopl   (%rax)
   94fc0:	lea    (%r12,%r10,1),%r11d
   94fc4:	cmp    %r8d,%r11d
   94fc7:	jae    95000 <<c30walk::ctor2d::W<false>>::finish+0x380>
   94fc9:	lea    (%r10,%r15,1),%rax
   94fcd:	cmp    $0x20,%rax
   94fd1:	jae    95206 <<c30walk::ctor2d::W<false>>::finish+0x586>
   94fd7:	movzwl (%rdi,%r10,2),%r11d
   94fdc:	inc    %r10
   94fdf:	mov    %ebx,%r14d
   94fe2:	shl    $0x18,%r14d
   94fe6:	shl    $0x8,%r11d
   94fea:	or     %r14d,%r11d
   94fed:	or     %esi,%r11d
   94ff0:	mov    %r11d,(%rdx,%rax,4)
   94ff4:	cmp    0x10(%rsp),%r10
   94ff9:	jne    94fc0 <<c30walk::ctor2d::W<false>>::finish+0x340>
   94ffb:	jmp    95055 <<c30walk::ctor2d::W<false>>::finish+0x3d5>
   94ffd:	nopl   (%rax)
   95000:	mov    %rbx,%r14
   95003:	shl    $0xa,%r14
   95007:	add    %r9,%r14
   9500a:	mov    %rbx,%rax
   9500d:	jmp    95027 <<c30walk::ctor2d::W<false>>::finish+0x3a7>
   9500f:	nop
   95010:	mov    0x8(%rbp),%rax
   95014:	mov    (%rax,%r14,1),%r8d
   95018:	add    $0x400,%r14
   9501f:	mov    %rbx,%rax
   95022:	cmp    %r8d,%r11d
   95025:	jb     94fc9 <<c30walk::ctor2d::W<false>>::finish+0x349>
   95027:	lea    0x1(%rax),%rbx
   9502b:	mov    $0xffffffff,%r8d
   95031:	cmp    %r13,%rbx
   95034:	jae    95018 <<c30walk::ctor2d::W<false>>::finish+0x398>
   95036:	add    $0x2,%rax
   9503a:	mov    0x10(%rbp),%r8
   9503e:	cmp    %r8,%rax
   95041:	jb     95010 <<c30walk::ctor2d::W<false>>::finish+0x390>
   95043:	lea    0x9e0a6(%rip),%rdx        # 1330f0 <__frame_dummy_init_array_entry+0x2630>
   9504a:	mov    %rax,%rdi
   9504d:	mov    %r8,%rsi
   95050:	call   da0a3 <core::panicking::panic_bounds_check>
   95055:	mov    0x30(%rsp),%r8
   9505a:	mov    0x28(%r8),%rax
   9505e:	mov    %rcx,%rdx
   95061:	shl    $0x6,%rdx
   95065:	mov    0x8(%rsp),%rsi
   9506a:	or     $0x10000000,%esi
   95070:	pcmpeqd %xmm0,%xmm0
   95074:	movdqu %xmm0,0x2c(%rax,%rdx,1)
   9507a:	movdqa %xmm0,0x20(%rax,%rdx,1)
   95080:	movdqa %xmm0,0x10(%rax,%rdx,1)
   95086:	movdqa %xmm0,(%rax,%rdx,1)
   9508b:	mov    %esi,0x3c(%rax,%rdx,1)
   9508f:	mov    %ecx,%edi
   95091:	shr    $0x6,%edi
   95094:	mov    0x10(%r8),%rsi
   95098:	cmp    %rsi,%rdi
   9509b:	jae    9524f <<c30walk::ctor2d::W<false>>::finish+0x5cf>
   950a1:	mov    $0x1,%eax
   950a6:	shl    %cl,%rax
   950a9:	mov    0x8(%r8),%rcx
   950ad:	or     %rax,(%rcx,%rdi,8)
   950b1:	jmp    9527b <<c30walk::ctor2d::W<false>>::finish+0x5fb>
   950b6:	mov    0x8(%rsp),%rcx
   950bb:	cmp    $0x8,%rcx
   950bf:	jae    950c8 <<c30walk::ctor2d::W<false>>::finish+0x448>
   950c1:	xor    %ecx,%ecx
   950c3:	jmp    95272 <<c30walk::ctor2d::W<false>>::finish+0x5f2>
   950c8:	movabs $0x7ffffffffffffff0,%rsi
   950d2:	cmp    $0x20,%rcx
   950d6:	jae    950dc <<c30walk::ctor2d::W<false>>::finish+0x45c>
   950d8:	xor    %ecx,%ecx
   950da:	jmp    9513b <<c30walk::ctor2d::W<false>>::finish+0x4bb>
   950dc:	mov    %r14,%rcx
   950df:	and    %rsi,%rcx
   950e2:	movd   %eax,%xmm0
   950e6:	pshuflw $0x0,%xmm0,%xmm0
   950eb:	pshufd $0x44,%xmm0,%xmm0
   950f0:	xor    %edx,%edx
   950f2:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   95100:	movdqu (%rdi,%rdx,2),%xmm1
   95105:	movdqu 0x10(%rdi,%rdx,2),%xmm2
   9510b:	psubw  %xmm0,%xmm1
   9510f:	psubw  %xmm0,%xmm2
   95113:	movdqu %xmm1,(%rdi,%rdx,2)
   95118:	movdqu %xmm2,0x10(%rdi,%rdx,2)
   9511e:	add    $0x10,%rdx
   95122:	cmp    %rdx,%rcx
   95125:	jne    95100 <<c30walk::ctor2d::W<false>>::finish+0x480>
   95127:	cmp    %rcx,%r14
   9512a:	je     9527b <<c30walk::ctor2d::W<false>>::finish+0x5fb>
   95130:	testb  $0x18,0x8(%rsp)
   95135:	je     95272 <<c30walk::ctor2d::W<false>>::finish+0x5f2>
   9513b:	mov    %rcx,%rdx
   9513e:	add    $0xc,%rsi
   95142:	mov    %rsi,%rcx
   95145:	and    %r14,%rcx
   95148:	movd   %eax,%xmm0
   9514c:	pshuflw $0x0,%xmm0,%xmm0
   95151:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   95160:	movq   (%rdi,%rdx,2),%xmm1
   95165:	psubw  %xmm0,%xmm1
   95169:	movq   %xmm1,(%rdi,%rdx,2)
   9516e:	add    $0x4,%rdx
   95172:	cmp    %rdx,%rcx
   95175:	jne    95160 <<c30walk::ctor2d::W<false>>::finish+0x4e0>
   95177:	jmp    9526d <<c30walk::ctor2d::W<false>>::finish+0x5ed>
   9517c:	mov    %rdi,0x8(%rsp)
   95181:	mov    %r10,%rdi
   95184:	mov    %rsi,0x20(%rsp)
   95189:	mov    %rcx,0x28(%rsp)
   9518e:	mov    %r9,%r14
   95191:	mov    %r10,0x18(%rsp)
   95196:	call   92c50 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   9519b:	mov    0x18(%rsp),%r10
   951a0:	mov    %r14,%r9
   951a3:	mov    0x28(%rsp),%rcx
   951a8:	mov    0x20(%rsp),%rsi
   951ad:	mov    0x10(%rsp),%r14
   951b2:	mov    0x8(%rsp),%rdi
   951b7:	jmp    94f32 <<c30walk::ctor2d::W<false>>::finish+0x2b2>
   951bc:	lea    0x9df2d(%rip),%rdx        # 1330f0 <__frame_dummy_init_array_entry+0x2630>
   951c3:	mov    %rcx,%rdi
   951c6:	mov    %r8,%rsi
   951c9:	call   da0a3 <core::panicking::panic_bounds_check>
   951ce:	lea    0x9e023(%rip),%rdx        # 1331f8 <__frame_dummy_init_array_entry+0x2738>
   951d5:	mov    %rbx,%rdi
   951d8:	mov    %r8,%rsi
   951db:	call   da0a3 <core::panicking::panic_bounds_check>
   951e0:	lea    0x9e011(%rip),%rdx        # 1331f8 <__frame_dummy_init_array_entry+0x2738>
   951e7:	mov    %r13,%rdi
   951ea:	mov    %r8,%rsi
   951ed:	call   da0a3 <core::panicking::panic_bounds_check>
   951f2:	lea    0x9dedf(%rip),%rdx        # 1330d8 <__frame_dummy_init_array_entry+0x2618>
   951f9:	mov    $0x20,%esi
   951fe:	mov    %rcx,%rdi
   95201:	call   da0a3 <core::panicking::panic_bounds_check>
   95206:	lea    0x9decb(%rip),%rdx        # 1330d8 <__frame_dummy_init_array_entry+0x2618>
   9520d:	mov    $0x20,%esi
   95212:	mov    %rax,%rdi
   95215:	call   da0a3 <core::panicking::panic_bounds_check>
   9521a:	lea    0x9df47(%rip),%rdx        # 133168 <__frame_dummy_init_array_entry+0x26a8>
   95221:	mov    %r13,%rdi
   95224:	xor    %esi,%esi
   95226:	call   da0a3 <core::panicking::panic_bounds_check>
   9522b:	lea    0x9de8e(%rip),%rdx        # 1330c0 <__frame_dummy_init_array_entry+0x2600>
   95232:	mov    %rax,%rdi
   95235:	mov    %rcx,%rsi
   95238:	call   da0a3 <core::panicking::panic_bounds_check>
   9523d:	lea    0x9de64(%rip),%rdx        # 1330a8 <__frame_dummy_init_array_entry+0x25e8>
   95244:	mov    %rcx,%rdi
   95247:	mov    %r8,%rsi
   9524a:	call   da0a3 <core::panicking::panic_bounds_check>
   9524f:	lea    0x9df2a(%rip),%rdx        # 133180 <__frame_dummy_init_array_entry+0x26c0>
   95256:	call   da0a3 <core::panicking::panic_bounds_check>
   9525b:	lea    0x9de46(%rip),%rdx        # 1330a8 <__frame_dummy_init_array_entry+0x25e8>
   95262:	mov    %rax,%rdi
   95265:	mov    %r8,%rsi
   95268:	call   da0a3 <core::panicking::panic_bounds_check>
   9526d:	cmp    %rcx,%r14
   95270:	je     9527b <<c30walk::ctor2d::W<false>>::finish+0x5fb>
   95272:	sub    %ax,(%rdi,%rcx,2)
   95276:	inc    %rcx
   95279:	jmp    9526d <<c30walk::ctor2d::W<false>>::finish+0x5ed>
   9527b:	add    $0x38,%rsp
   9527f:	pop    %rbx
   95280:	pop    %r12
   95282:	pop    %r13
   95284:	pop    %r14
   95286:	pop    %r15
   95288:	pop    %rbp
   95289:	ret
