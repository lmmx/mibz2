# <c30walk::ctor2d::W<false>>::finish
# size 0x60a

target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

0000000000093d20 <<c30walk::ctor2d::W<false>>::finish>:
   93d20:	push   %rbp
   93d21:	push   %r15
   93d23:	push   %r14
   93d25:	push   %r13
   93d27:	push   %r12
   93d29:	push   %rbx
   93d2a:	sub    $0x38,%rsp
   93d2e:	mov    0x18(%rdi),%rbp
   93d32:	mov    0x10(%rbp),%r8
   93d36:	mov    %r8,%r13
   93d39:	sub    $0x1,%r13
   93d3d:	jb     942ba <<c30walk::ctor2d::W<false>>::finish+0x59a>
   93d43:	mov    %rdi,%r11
   93d46:	lea    (%rsi,%rsi,2),%rax
   93d4a:	mov    0x1040(%rdi,%rax,8),%rdi
   93d52:	mov    0x1048(%r11,%rax,8),%r12d
   93d5a:	mov    0x1050(%r11,%rax,8),%ecx
   93d62:	mov    0x1054(%r11,%rax,8),%eax
   93d6a:	mov    0x40(%r11,%rsi,8),%r14
   93d6f:	sub    %rdi,%r14
   93d72:	mov    %r14,0x8(%rsp)
   93d77:	shr    $1,%r14
   93d7a:	mov    0x8(%rbp),%r10
   93d7e:	mov    %r13,%rdx
   93d81:	shl    $0xa,%rdx
   93d85:	add    %r10,%rdx
   93d88:	mov    %r13,%rbx
   93d8b:	cmp    %r12d,(%rdx,%rsi,4)
   93d8f:	jbe    93e08 <<c30walk::ctor2d::W<false>>::finish+0xe8>
   93d91:	mov    %r11,%r15
   93d94:	lea    (%r12,%r14,1),%edx
   93d98:	dec    %edx
   93d9a:	mov    %r8,%r9
   93d9d:	shl    $0xa,%r9
   93da1:	lea    (%r9,%rsi,4),%r9
   93da5:	lea    (%r10,%r9,1),%r11
   93da9:	add    $0xfffffffffffffc00,%r11
   93db0:	mov    %r8,%rbx
   93db3:	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   93dc0:	dec    %rbx
   93dc3:	cmp    %r8,%rbx
   93dc6:	jae    9426e <<c30walk::ctor2d::W<false>>::finish+0x54e>
   93dcc:	cmp    %r12d,(%r11)
   93dcf:	lea    -0x400(%r11),%r11
   93dd6:	ja     93dc0 <<c30walk::ctor2d::W<false>>::finish+0xa0>
   93dd8:	add    %r10,%r9
   93ddb:	add    $0xfffffffffffffc00,%r9
   93de2:	mov    %r8,%r13
   93de5:	mov    %r15,%r11
   93de8:	nopl   0x0(%rax,%rax,1)
   93df0:	dec    %r13
   93df3:	cmp    %r8,%r13
   93df6:	jae    94280 <<c30walk::ctor2d::W<false>>::finish+0x560>
   93dfc:	cmp    %edx,(%r9)
   93dff:	lea    -0x400(%r9),%r9
   93e06:	ja     93df0 <<c30walk::ctor2d::W<false>>::finish+0xd0>
   93e08:	imul   $0xffffffffffffffe2,%rcx,%r15
   93e0c:	add    %r12,%r15
   93e0f:	cmp    $0xffffffff,%eax
   93e12:	mov    %r14,0x10(%rsp)
   93e17:	je     93f1b <<c30walk::ctor2d::W<false>>::finish+0x1fb>
   93e1d:	mov    $0xffffffff,%r9d
   93e23:	cmp    %r13,%rbx
   93e26:	jae    93e40 <<c30walk::ctor2d::W<false>>::finish+0x120>
   93e28:	lea    0x1(%rbx),%rcx
   93e2c:	cmp    %r8,%rcx
   93e2f:	jae    942dd <<c30walk::ctor2d::W<false>>::finish+0x5bd>
   93e35:	shl    $0xa,%rcx
   93e39:	add    %rcx,%r10
   93e3c:	mov    (%r10,%rsi,4),%r9d
   93e40:	mov    (%r11),%rdx
   93e43:	mov    0x10(%rdx),%rcx
   93e47:	cmp    %rax,%rcx
   93e4a:	jbe    942cb <<c30walk::ctor2d::W<false>>::finish+0x5ab>
   93e50:	test   %r14,%r14
   93e53:	je     9431b <<c30walk::ctor2d::W<false>>::finish+0x5fb>
   93e59:	shl    $0x7,%rax
   93e5d:	add    0x8(%rdx),%rax
   93e61:	lea    0x800(,%rsi,4),%rdx
   93e69:	xor    %r11d,%r11d
   93e6c:	nopl   0x0(%rax)
   93e70:	lea    (%r12,%r11,1),%r10d
   93e74:	cmp    %r9d,%r10d
   93e77:	jae    93eb0 <<c30walk::ctor2d::W<false>>::finish+0x190>
   93e79:	lea    (%r11,%r15,1),%rcx
   93e7d:	cmp    $0x20,%rcx
   93e81:	jae    94292 <<c30walk::ctor2d::W<false>>::finish+0x572>
   93e87:	movzwl (%rdi,%r11,2),%r8d
   93e8c:	inc    %r11
   93e8f:	mov    %ebx,%r10d
   93e92:	shl    $0x18,%r10d
   93e96:	shl    $0x8,%r8d
   93e9a:	or     %r10d,%r8d
   93e9d:	or     %esi,%r8d
   93ea0:	mov    %r8d,(%rax,%rcx,4)
   93ea4:	cmp    0x10(%rsp),%r11
   93ea9:	jne    93e70 <<c30walk::ctor2d::W<false>>::finish+0x150>
   93eab:	jmp    93ef9 <<c30walk::ctor2d::W<false>>::finish+0x1d9>
   93ead:	nopl   (%rax)
   93eb0:	mov    %rbx,%r14
   93eb3:	shl    $0xa,%r14
   93eb7:	add    %rdx,%r14
   93eba:	mov    %rbx,%rcx
   93ebd:	jmp    93ecf <<c30walk::ctor2d::W<false>>::finish+0x1af>
   93ebf:	nop
   93ec0:	add    $0x400,%r14
   93ec7:	mov    %rbx,%rcx
   93eca:	cmp    %r9d,%r10d
   93ecd:	jb     93e79 <<c30walk::ctor2d::W<false>>::finish+0x159>
   93ecf:	lea    0x1(%rcx),%rbx
   93ed3:	mov    $0xffffffff,%r9d
   93ed9:	cmp    %r13,%rbx
   93edc:	jae    93ec0 <<c30walk::ctor2d::W<false>>::finish+0x1a0>
   93ede:	add    $0x2,%rcx
   93ee2:	mov    0x10(%rbp),%r8
   93ee6:	cmp    %r8,%rcx
   93ee9:	jae    9425c <<c30walk::ctor2d::W<false>>::finish+0x53c>
   93eef:	mov    0x8(%rbp),%rcx
   93ef3:	mov    (%rcx,%r14,1),%r9d
   93ef7:	jmp    93ec0 <<c30walk::ctor2d::W<false>>::finish+0x1a0>
   93ef9:	mov    0x8(%rsp),%rdx
   93efe:	and    $0xfffffffffffffffe,%rdx
   93f02:	mov    $0xff,%esi
   93f07:	add    $0x38,%rsp
   93f0b:	pop    %rbx
   93f0c:	pop    %r12
   93f0e:	pop    %r13
   93f10:	pop    %r14
   93f12:	pop    %r15
   93f14:	pop    %rbp
   93f15:	jmp    *0xa1f35(%rip)        # 135e50 <memset@GLIBC_2.2.5>
   93f1b:	movzwl (%rdi),%eax
   93f1e:	movzwl -0x2(%rdi,%r14,2),%r8d
   93f24:	mov    %ebx,%edx
   93f26:	shl    $0x10,%edx
   93f29:	or     %eax,%edx
   93f2b:	mov    %r13d,%r9d
   93f2e:	shl    $0x10,%r9d
   93f32:	or     %r8d,%r9d
   93f35:	sub    %edx,%r9d
   93f38:	cmp    $0xfffe,%r9d
   93f3f:	jae    93fbd <<c30walk::ctor2d::W<false>>::finish+0x29d>
   93f41:	mov    0x28(%r11),%r8
   93f45:	shl    $0x6,%rcx
   93f49:	shl    $0x8,%edx
   93f4c:	or     %edx,%esi
   93f4e:	mov    %esi,0x3c(%r8,%rcx,1)
   93f53:	test   %r14,%r14
   93f56:	je     9431b <<c30walk::ctor2d::W<false>>::finish+0x5fb>
   93f5c:	cmp    $0x1e,%r14
   93f60:	jne    94156 <<c30walk::ctor2d::W<false>>::finish+0x436>
   93f66:	movd   %eax,%xmm0
   93f6a:	pshuflw $0x0,%xmm0,%xmm0
   93f6f:	pshufd $0x44,%xmm0,%xmm1
   93f74:	movdqu (%rdi),%xmm2
   93f78:	movdqu 0x10(%rdi),%xmm3
   93f7d:	movdqu 0x20(%rdi),%xmm4
   93f82:	psubw  %xmm1,%xmm2
   93f86:	movdqu %xmm2,(%rdi)
   93f8a:	psubw  %xmm1,%xmm3
   93f8e:	movdqu %xmm3,0x10(%rdi)
   93f93:	psubw  %xmm1,%xmm4
   93f97:	movdqu %xmm4,0x20(%rdi)
   93f9c:	movq   0x30(%rdi),%xmm1
   93fa1:	psubw  %xmm0,%xmm1
   93fa5:	movq   %xmm1,0x30(%rdi)
   93faa:	movd   0x38(%rdi),%xmm1
   93faf:	psubw  %xmm0,%xmm1
   93fb3:	movd   %xmm1,0x38(%rdi)
   93fb8:	jmp    9431b <<c30walk::ctor2d::W<false>>::finish+0x5fb>
   93fbd:	mov    (%r11),%r10
   93fc0:	mov    0x10(%r10),%r9
   93fc4:	cmp    (%r10),%r9
   93fc7:	mov    %r11,0x30(%rsp)
   93fcc:	je     9421c <<c30walk::ctor2d::W<false>>::finish+0x4fc>
   93fd2:	mov    0x8(%r10),%rax
   93fd6:	mov    %r9,%rdx
   93fd9:	shl    $0x7,%rdx
   93fdd:	pxor   %xmm0,%xmm0
   93fe1:	movdqa %xmm0,0x70(%rax,%rdx,1)
   93fe7:	movdqa %xmm0,0x60(%rax,%rdx,1)
   93fed:	movdqa %xmm0,0x50(%rax,%rdx,1)
   93ff3:	movdqa %xmm0,0x40(%rax,%rdx,1)
   93ff9:	movdqa %xmm0,0x30(%rax,%rdx,1)
   93fff:	movdqa %xmm0,0x20(%rax,%rdx,1)
   94005:	movdqa %xmm0,0x10(%rax,%rdx,1)
   9400b:	movdqa %xmm0,(%rax,%rdx,1)
   94010:	lea    0x1(%r9),%rax
   94014:	mov    %rax,0x10(%r10)
   94018:	mov    $0xffffffff,%r8d
   9401e:	cmp    %r13,%rbx
   94021:	jae    94040 <<c30walk::ctor2d::W<false>>::finish+0x320>
   94023:	lea    0x1(%rbx),%rax
   94027:	mov    0x10(%rbp),%r8
   9402b:	cmp    %r8,%rax
   9402e:	jae    942fb <<c30walk::ctor2d::W<false>>::finish+0x5db>
   94034:	shl    $0xa,%rax
   94038:	add    0x8(%rbp),%rax
   9403c:	mov    (%rax,%rsi,4),%r8d
   94040:	mov    %r9,0x8(%rsp)
   94045:	test   %r14,%r14
   94048:	je     940f5 <<c30walk::ctor2d::W<false>>::finish+0x3d5>
   9404e:	add    0x8(%r10),%rdx
   94052:	lea    0x800(,%rsi,4),%r9
   9405a:	xor    %r10d,%r10d
   9405d:	nopl   (%rax)
   94060:	lea    (%r12,%r10,1),%r11d
   94064:	cmp    %r8d,%r11d
   94067:	jae    940a0 <<c30walk::ctor2d::W<false>>::finish+0x380>
   94069:	lea    (%r10,%r15,1),%rax
   9406d:	cmp    $0x20,%rax
   94071:	jae    942a6 <<c30walk::ctor2d::W<false>>::finish+0x586>
   94077:	movzwl (%rdi,%r10,2),%r11d
   9407c:	inc    %r10
   9407f:	mov    %ebx,%r14d
   94082:	shl    $0x18,%r14d
   94086:	shl    $0x8,%r11d
   9408a:	or     %r14d,%r11d
   9408d:	or     %esi,%r11d
   94090:	mov    %r11d,(%rdx,%rax,4)
   94094:	cmp    0x10(%rsp),%r10
   94099:	jne    94060 <<c30walk::ctor2d::W<false>>::finish+0x340>
   9409b:	jmp    940f5 <<c30walk::ctor2d::W<false>>::finish+0x3d5>
   9409d:	nopl   (%rax)
   940a0:	mov    %rbx,%r14
   940a3:	shl    $0xa,%r14
   940a7:	add    %r9,%r14
   940aa:	mov    %rbx,%rax
   940ad:	jmp    940c7 <<c30walk::ctor2d::W<false>>::finish+0x3a7>
   940af:	nop
   940b0:	mov    0x8(%rbp),%rax
   940b4:	mov    (%rax,%r14,1),%r8d
   940b8:	add    $0x400,%r14
   940bf:	mov    %rbx,%rax
   940c2:	cmp    %r8d,%r11d
   940c5:	jb     94069 <<c30walk::ctor2d::W<false>>::finish+0x349>
   940c7:	lea    0x1(%rax),%rbx
   940cb:	mov    $0xffffffff,%r8d
   940d1:	cmp    %r13,%rbx
   940d4:	jae    940b8 <<c30walk::ctor2d::W<false>>::finish+0x398>
   940d6:	add    $0x2,%rax
   940da:	mov    0x10(%rbp),%r8
   940de:	cmp    %r8,%rax
   940e1:	jb     940b0 <<c30walk::ctor2d::W<false>>::finish+0x390>
   940e3:	lea    0x9e066(%rip),%rdx        # 132150 <__frame_dummy_init_array_entry+0x2630>
   940ea:	mov    %rax,%rdi
   940ed:	mov    %r8,%rsi
   940f0:	call   d9103 <core::panicking::panic_bounds_check>
   940f5:	mov    0x30(%rsp),%r8
   940fa:	mov    0x28(%r8),%rax
   940fe:	mov    %rcx,%rdx
   94101:	shl    $0x6,%rdx
   94105:	mov    0x8(%rsp),%rsi
   9410a:	or     $0x10000000,%esi
   94110:	pcmpeqd %xmm0,%xmm0
   94114:	movdqu %xmm0,0x2c(%rax,%rdx,1)
   9411a:	movdqa %xmm0,0x20(%rax,%rdx,1)
   94120:	movdqa %xmm0,0x10(%rax,%rdx,1)
   94126:	movdqa %xmm0,(%rax,%rdx,1)
   9412b:	mov    %esi,0x3c(%rax,%rdx,1)
   9412f:	mov    %ecx,%edi
   94131:	shr    $0x6,%edi
   94134:	mov    0x10(%r8),%rsi
   94138:	cmp    %rsi,%rdi
   9413b:	jae    942ef <<c30walk::ctor2d::W<false>>::finish+0x5cf>
   94141:	mov    $0x1,%eax
   94146:	shl    %cl,%rax
   94149:	mov    0x8(%r8),%rcx
   9414d:	or     %rax,(%rcx,%rdi,8)
   94151:	jmp    9431b <<c30walk::ctor2d::W<false>>::finish+0x5fb>
   94156:	mov    0x8(%rsp),%rcx
   9415b:	cmp    $0x8,%rcx
   9415f:	jae    94168 <<c30walk::ctor2d::W<false>>::finish+0x448>
   94161:	xor    %ecx,%ecx
   94163:	jmp    94312 <<c30walk::ctor2d::W<false>>::finish+0x5f2>
   94168:	movabs $0x7ffffffffffffff0,%rsi
   94172:	cmp    $0x20,%rcx
   94176:	jae    9417c <<c30walk::ctor2d::W<false>>::finish+0x45c>
   94178:	xor    %ecx,%ecx
   9417a:	jmp    941db <<c30walk::ctor2d::W<false>>::finish+0x4bb>
   9417c:	mov    %r14,%rcx
   9417f:	and    %rsi,%rcx
   94182:	movd   %eax,%xmm0
   94186:	pshuflw $0x0,%xmm0,%xmm0
   9418b:	pshufd $0x44,%xmm0,%xmm0
   94190:	xor    %edx,%edx
   94192:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   941a0:	movdqu (%rdi,%rdx,2),%xmm1
   941a5:	movdqu 0x10(%rdi,%rdx,2),%xmm2
   941ab:	psubw  %xmm0,%xmm1
   941af:	psubw  %xmm0,%xmm2
   941b3:	movdqu %xmm1,(%rdi,%rdx,2)
   941b8:	movdqu %xmm2,0x10(%rdi,%rdx,2)
   941be:	add    $0x10,%rdx
   941c2:	cmp    %rdx,%rcx
   941c5:	jne    941a0 <<c30walk::ctor2d::W<false>>::finish+0x480>
   941c7:	cmp    %rcx,%r14
   941ca:	je     9431b <<c30walk::ctor2d::W<false>>::finish+0x5fb>
   941d0:	testb  $0x18,0x8(%rsp)
   941d5:	je     94312 <<c30walk::ctor2d::W<false>>::finish+0x5f2>
   941db:	mov    %rcx,%rdx
   941de:	add    $0xc,%rsi
   941e2:	mov    %rsi,%rcx
   941e5:	and    %r14,%rcx
   941e8:	movd   %eax,%xmm0
   941ec:	pshuflw $0x0,%xmm0,%xmm0
   941f1:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   94200:	movq   (%rdi,%rdx,2),%xmm1
   94205:	psubw  %xmm0,%xmm1
   94209:	movq   %xmm1,(%rdi,%rdx,2)
   9420e:	add    $0x4,%rdx
   94212:	cmp    %rdx,%rcx
   94215:	jne    94200 <<c30walk::ctor2d::W<false>>::finish+0x4e0>
   94217:	jmp    9430d <<c30walk::ctor2d::W<false>>::finish+0x5ed>
   9421c:	mov    %rdi,0x8(%rsp)
   94221:	mov    %r10,%rdi
   94224:	mov    %rsi,0x20(%rsp)
   94229:	mov    %rcx,0x28(%rsp)
   9422e:	mov    %r9,%r14
   94231:	mov    %r10,0x18(%rsp)
   94236:	call   91cf0 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   9423b:	mov    0x18(%rsp),%r10
   94240:	mov    %r14,%r9
   94243:	mov    0x28(%rsp),%rcx
   94248:	mov    0x20(%rsp),%rsi
   9424d:	mov    0x10(%rsp),%r14
   94252:	mov    0x8(%rsp),%rdi
   94257:	jmp    93fd2 <<c30walk::ctor2d::W<false>>::finish+0x2b2>
   9425c:	lea    0x9deed(%rip),%rdx        # 132150 <__frame_dummy_init_array_entry+0x2630>
   94263:	mov    %rcx,%rdi
   94266:	mov    %r8,%rsi
   94269:	call   d9103 <core::panicking::panic_bounds_check>
   9426e:	lea    0x9dfe3(%rip),%rdx        # 132258 <__frame_dummy_init_array_entry+0x2738>
   94275:	mov    %rbx,%rdi
   94278:	mov    %r8,%rsi
   9427b:	call   d9103 <core::panicking::panic_bounds_check>
   94280:	lea    0x9dfd1(%rip),%rdx        # 132258 <__frame_dummy_init_array_entry+0x2738>
   94287:	mov    %r13,%rdi
   9428a:	mov    %r8,%rsi
   9428d:	call   d9103 <core::panicking::panic_bounds_check>
   94292:	lea    0x9de9f(%rip),%rdx        # 132138 <__frame_dummy_init_array_entry+0x2618>
   94299:	mov    $0x20,%esi
   9429e:	mov    %rcx,%rdi
   942a1:	call   d9103 <core::panicking::panic_bounds_check>
   942a6:	lea    0x9de8b(%rip),%rdx        # 132138 <__frame_dummy_init_array_entry+0x2618>
   942ad:	mov    $0x20,%esi
   942b2:	mov    %rax,%rdi
   942b5:	call   d9103 <core::panicking::panic_bounds_check>
   942ba:	lea    0x9df07(%rip),%rdx        # 1321c8 <__frame_dummy_init_array_entry+0x26a8>
   942c1:	mov    %r13,%rdi
   942c4:	xor    %esi,%esi
   942c6:	call   d9103 <core::panicking::panic_bounds_check>
   942cb:	lea    0x9de4e(%rip),%rdx        # 132120 <__frame_dummy_init_array_entry+0x2600>
   942d2:	mov    %rax,%rdi
   942d5:	mov    %rcx,%rsi
   942d8:	call   d9103 <core::panicking::panic_bounds_check>
   942dd:	lea    0x9de24(%rip),%rdx        # 132108 <__frame_dummy_init_array_entry+0x25e8>
   942e4:	mov    %rcx,%rdi
   942e7:	mov    %r8,%rsi
   942ea:	call   d9103 <core::panicking::panic_bounds_check>
   942ef:	lea    0x9deea(%rip),%rdx        # 1321e0 <__frame_dummy_init_array_entry+0x26c0>
   942f6:	call   d9103 <core::panicking::panic_bounds_check>
   942fb:	lea    0x9de06(%rip),%rdx        # 132108 <__frame_dummy_init_array_entry+0x25e8>
   94302:	mov    %rax,%rdi
   94305:	mov    %r8,%rsi
   94308:	call   d9103 <core::panicking::panic_bounds_check>
   9430d:	cmp    %rcx,%r14
   94310:	je     9431b <<c30walk::ctor2d::W<false>>::finish+0x5fb>
   94312:	sub    %ax,(%rdi,%rcx,2)
   94316:	inc    %rcx
   94319:	jmp    9430d <<c30walk::ctor2d::W<false>>::finish+0x5ed>
   9431b:	add    $0x38,%rsp
   9431f:	pop    %rbx
   94320:	pop    %r12
   94322:	pop    %r13
   94324:	pop    %r14
   94326:	pop    %r15
   94328:	pop    %rbp
   94329:	ret
