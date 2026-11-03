
target/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

0000000000061d30 <<c30walk::ctor2::W<false>>::finish>:
<c30walk::ctor2::W<false>>::finish:
./examples/c30walk/ctor2.rs:400
   61d30:	push   %rbp
   61d31:	push   %r15
   61d33:	push   %r14
   61d35:	push   %r13
   61d37:	push   %r12
   61d39:	push   %rbx
   61d3a:	sub    $0x38,%rsp
./examples/c30walk/ctor2.rs:407
   61d3e:	mov    0x18(%rdi),%rbp
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:3054
   61d42:	mov    0x10(%rbp),%r8
./examples/c30walk/ctor2.rs:407
   61d46:	mov    %r8,%r13
   61d49:	sub    $0x1,%r13
/rustc/2d8144b78/library/core/src/slice/index.rs:272
   61d4d:	jb     622ba <<c30walk::ctor2::W<false>>::finish+0x58a>
   61d53:	mov    %rdi,%r11
   61d56:	lea    (%rsi,%rsi,4),%rax
   61d5a:	mov    0x40(%rdi,%rax,8),%r14
   61d5f:	mov    0x50(%rdi,%rax,8),%rdi
   61d64:	mov    0x58(%r11,%rax,8),%r12d
   61d69:	mov    0x60(%r11,%rax,8),%ecx
   61d6e:	mov    0x64(%r11,%rax,8),%eax
   61d73:	sub    %rdi,%r14
   61d76:	mov    %r14,0x8(%rsp)
   61d7b:	shr    $1,%r14
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:614
   61d7e:	mov    0x8(%rbp),%r10
index<[u32; 256]>():
/rustc/2d8144b78/library/core/src/slice/index.rs:272
   61d82:	mov    %r13,%rdx
   61d85:	shl    $0xa,%rdx
   61d89:	add    %r10,%rdx
   61d8c:	mov    %r13,%rbx
<c30walk::ctor2::W<false>>::finish:
./examples/c30walk/ctor2.rs:408
   61d8f:	cmp    %r12d,(%rdx,%rsi,4)
./examples/c30walk/ctor2.rs:409
   61d93:	jbe    61e08 <<c30walk::ctor2::W<false>>::finish+0xd8>
   61d95:	mov    %r11,%r15
   61d98:	lea    (%r12,%r14,1),%edx
   61d9c:	dec    %edx
/rustc/2d8144b78/library/core/src/slice/index.rs:272
   61d9e:	mov    %r8,%r9
index<[u32; 256]>():
/rustc/2d8144b78/library/core/src/slice/index.rs:272
   61da1:	shl    $0xa,%r9
   61da5:	lea    (%r9,%rsi,4),%r9
   61da9:	lea    (%r10,%r9,1),%r11
   61dad:	add    $0xfffffffffffffc00,%r11
   61db4:	mov    %r8,%rbx
   61db7:	nopw   0x0(%rax,%rax,1)
   61dc0:	dec    %rbx
   61dc3:	cmp    %r8,%rbx
   61dc6:	jae    6226e <<c30walk::ctor2::W<false>>::finish+0x53e>
<c30walk::ctor2::W<false>>::finish:
./examples/c30walk/ctor2.rs:337
   61dcc:	cmp    %r12d,(%r11)
epoch_of<false>():
./examples/c30walk/ctor2.rs:337
   61dcf:	lea    -0x400(%r11),%r11
   61dd6:	ja     61dc0 <<c30walk::ctor2::W<false>>::finish+0x90>
<c30walk::ctor2::W<false>>::finish:
/rustc/2d8144b78/library/core/src/slice/index.rs:272
   61dd8:	add    %r10,%r9
index<[u32; 256]>():
/rustc/2d8144b78/library/core/src/slice/index.rs:272
   61ddb:	add    $0xfffffffffffffc00,%r9
   61de2:	mov    %r8,%r13
   61de5:	mov    %r15,%r11
   61de8:	nopl   0x0(%rax,%rax,1)
   61df0:	dec    %r13
   61df3:	cmp    %r8,%r13
   61df6:	jae    62280 <<c30walk::ctor2::W<false>>::finish+0x550>
<c30walk::ctor2::W<false>>::finish:
./examples/c30walk/ctor2.rs:337
   61dfc:	cmp    %edx,(%r9)
epoch_of<false>():
./examples/c30walk/ctor2.rs:337
   61dff:	lea    -0x400(%r9),%r9
   61e06:	ja     61df0 <<c30walk::ctor2::W<false>>::finish+0xc0>
<c30walk::ctor2::W<false>>::finish:
   61e08:	imul   $0xffffffffffffffe2,%rcx,%r15
   61e0c:	add    %r12,%r15
./examples/c30walk/ctor2.rs:421
   61e0f:	cmp    $0xffffffff,%eax
   61e12:	mov    %r14,0x10(%rsp)
   61e17:	je     61f1b <<c30walk::ctor2::W<false>>::finish+0x1eb>
   61e1d:	mov    $0xffffffff,%r9d
./examples/c30walk/ctor2.rs:513
   61e23:	cmp    %r13,%rbx
write_entries<false>():
./examples/c30walk/ctor2.rs:513
   61e26:	jae    61e40 <<c30walk::ctor2::W<false>>::finish+0x110>
./examples/c30walk/ctor2.rs:514
   61e28:	lea    0x1(%rbx),%rcx
<c30walk::ctor2::W<false>>::finish:
/rustc/2d8144b78/library/core/src/slice/index.rs:272
   61e2c:	cmp    %r8,%rcx
index<[u32; 256]>():
/rustc/2d8144b78/library/core/src/slice/index.rs:272
   61e2f:	jae    622dd <<c30walk::ctor2::W<false>>::finish+0x5ad>
   61e35:	shl    $0xa,%rcx
   61e39:	add    %rcx,%r10
write_entries<false>():
./examples/c30walk/ctor2.rs:514
   61e3c:	mov    (%r10,%rsi,4),%r9d
./examples/c30walk/ctor2.rs:518
   61e40:	mov    (%r11),%rdx
<c30walk::ctor2::W<false>>::finish:
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:1903
   61e43:	mov    0x10(%rdx),%rcx
/rustc/2d8144b78/library/core/src/slice/index.rs:278
   61e47:	cmp    %rax,%rcx
index_mut<c30walk::RawG>():
/rustc/2d8144b78/library/core/src/slice/index.rs:278
   61e4a:	jbe    622cb <<c30walk::ctor2::W<false>>::finish+0x59b>
<c30walk::ctor2::W<false>>::finish:
/rustc/2d8144b78/library/core/src/cmp.rs:1916
   61e50:	test   %r14,%r14
/rustc/2d8144b78/library/core/src/iter/range.rs:900
   61e53:	je     6231b <<c30walk::ctor2::W<false>>::finish+0x5eb>
write_entries<false>():
   61e59:	shl    $0x7,%rax
   61e5d:	add    0x8(%rdx),%rax
./examples/c30walk/ctor2.rs:523
   61e61:	lea    0x800(,%rsi,4),%rdx
   61e69:	xor    %r11d,%r11d
   61e6c:	nopl   0x0(%rax)
./examples/c30walk/ctor2.rs:520
   61e70:	lea    (%r12,%r11,1),%r10d
./examples/c30walk/ctor2.rs:521
   61e74:	cmp    %r9d,%r10d
   61e77:	jae    61eb0 <<c30walk::ctor2::W<false>>::finish+0x180>
./examples/c30walk/ctor2.rs:530
   61e79:	lea    (%r11,%r15,1),%rcx
   61e7d:	cmp    $0x20,%rcx
   61e81:	jae    62292 <<c30walk::ctor2::W<false>>::finish+0x562>
<c30walk::ctor2::W<false>>::finish:
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1733
   61e87:	movzwl (%rdi,%r11,2),%r8d
write_entries<false>():
   61e8c:	inc    %r11
./examples/c30walk/ctor2.rs:530
   61e8f:	mov    %ebx,%r10d
   61e92:	shl    $0x18,%r10d
   61e96:	shl    $0x8,%r8d
   61e9a:	or     %r10d,%r8d
   61e9d:	or     %esi,%r8d
   61ea0:	mov    %r8d,(%rax,%rcx,4)
lt():
/rustc/2d8144b78/library/core/src/cmp.rs:1916
   61ea4:	cmp    0x10(%rsp),%r11
spec_next<usize>():
/rustc/2d8144b78/library/core/src/iter/range.rs:900
   61ea9:	jne    61e70 <<c30walk::ctor2::W<false>>::finish+0x140>
   61eab:	jmp    61ef9 <<c30walk::ctor2::W<false>>::finish+0x1c9>
write_entries<false>():
   61ead:	nopl   (%rax)
./examples/c30walk/ctor2.rs:523
   61eb0:	mov    %rbx,%r14
   61eb3:	shl    $0xa,%r14
   61eb7:	add    %rdx,%r14
   61eba:	mov    %rbx,%rcx
   61ebd:	jmp    61ecf <<c30walk::ctor2::W<false>>::finish+0x19f>
   61ebf:	nop
./examples/c30walk/ctor2.rs:521
   61ec0:	add    $0x400,%r14
   61ec7:	mov    %rbx,%rcx
   61eca:	cmp    %r9d,%r10d
   61ecd:	jb     61e79 <<c30walk::ctor2::W<false>>::finish+0x149>
./examples/c30walk/ctor2.rs:522
   61ecf:	lea    0x1(%rcx),%rbx
   61ed3:	mov    $0xffffffff,%r9d
./examples/c30walk/ctor2.rs:523
   61ed9:	cmp    %r13,%rbx
   61edc:	jae    61ec0 <<c30walk::ctor2::W<false>>::finish+0x190>
./examples/c30walk/ctor2.rs:524
   61ede:	add    $0x2,%rcx
<c30walk::ctor2::W<false>>::finish:
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:1864
   61ee2:	mov    0x10(%rbp),%r8
/rustc/2d8144b78/library/core/src/slice/index.rs:272
   61ee6:	cmp    %r8,%rcx
index<[u32; 256]>():
/rustc/2d8144b78/library/core/src/slice/index.rs:272
   61ee9:	jae    6225c <<c30walk::ctor2::W<false>>::finish+0x52c>
<c30walk::ctor2::W<false>>::finish:
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:614
   61eef:	mov    0x8(%rbp),%rcx
write_entries<false>():
./examples/c30walk/ctor2.rs:524
   61ef3:	mov    (%rcx,%r14,1),%r9d
   61ef7:	jmp    61ec0 <<c30walk::ctor2::W<false>>::finish+0x190>
<c30walk::ctor2::W<false>>::finish:
   61ef9:	mov    0x8(%rsp),%rdx
/rustc/2d8144b78/library/core/src/iter/range.rs:900
   61efe:	and    $0xfffffffffffffffe,%rdx
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   61f02:	mov    $0xff,%esi
write<u16>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   61f07:	add    $0x38,%rsp
   61f0b:	pop    %rbx
   61f0c:	pop    %r12
   61f0e:	pop    %r13
   61f10:	pop    %r14
   61f12:	pop    %r15
   61f14:	pop    %rbp
   61f15:	jmp    *0x94d75(%rip)        # f6c90 <memset@GLIBC_2.2.5>
<c30walk::ctor2::W<false>>::finish:
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1733
   61f1b:	movzwl (%rdi),%eax
   61f1e:	movzwl -0x2(%rdi,%r14,2),%r8d
./examples/c30walk/ctor2.rs:425
   61f24:	mov    %ebx,%edx
   61f26:	shl    $0x10,%edx
   61f29:	or     %eax,%edx
./examples/c30walk/ctor2.rs:426
   61f2b:	mov    %r13d,%r9d
   61f2e:	shl    $0x10,%r9d
   61f32:	or     %r8d,%r9d
./examples/c30walk/ctor2.rs:428
   61f35:	sub    %edx,%r9d
./examples/c30walk/ctor2.rs:429
   61f38:	cmp    $0xfffe,%r9d
   61f3f:	jae    61fbd <<c30walk::ctor2::W<false>>::finish+0x28d>
./examples/c30walk/ctor2.rs:430
   61f41:	mov    0x28(%r11),%r8
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   61f45:	shl    $0x6,%rcx
./examples/c30walk/ctor2.rs:430
   61f49:	shl    $0x8,%edx
   61f4c:	or     %edx,%esi
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   61f4e:	mov    %esi,0x3c(%r8,%rcx,1)
./examples/c30walk/ctor2.rs:431
   61f53:	test   %r14,%r14
   61f56:	je     6231b <<c30walk::ctor2::W<false>>::finish+0x5eb>
   61f5c:	cmp    $0x1e,%r14
   61f60:	jne    62156 <<c30walk::ctor2::W<false>>::finish+0x426>
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:2621
   61f66:	movd   %eax,%xmm0
wrapping_sub():
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:2621
   61f6a:	pshuflw $0x0,%xmm0,%xmm0
   61f6f:	pshufd $0x44,%xmm0,%xmm1
<c30walk::ctor2::W<false>>::finish:
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1733
   61f74:	movdqu (%rdi),%xmm2
read<[u16; 30]>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1733
   61f78:	movdqu 0x10(%rdi),%xmm3
   61f7d:	movdqu 0x20(%rdi),%xmm4
wrapping_sub():
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:2621
   61f82:	psubw  %xmm1,%xmm2
<c30walk::ctor2::W<false>>::finish:
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   61f86:	movdqu %xmm2,(%rdi)
wrapping_sub():
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:2621
   61f8a:	psubw  %xmm1,%xmm3
write<[u16; 30]>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   61f8e:	movdqu %xmm3,0x10(%rdi)
wrapping_sub():
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:2621
   61f93:	psubw  %xmm1,%xmm4
write<[u16; 30]>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   61f97:	movdqu %xmm4,0x20(%rdi)
read<[u16; 30]>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1733
   61f9c:	movq   0x30(%rdi),%xmm1
wrapping_sub():
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:2621
   61fa1:	psubw  %xmm0,%xmm1
write<[u16; 30]>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   61fa5:	movq   %xmm1,0x30(%rdi)
read<[u16; 30]>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1733
   61faa:	movd   0x38(%rdi),%xmm1
wrapping_sub():
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:2621
   61faf:	psubw  %xmm0,%xmm1
write<[u16; 30]>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   61fb3:	movd   %xmm1,0x38(%rdi)
   61fb8:	jmp    6231b <<c30walk::ctor2::W<false>>::finish+0x5eb>
<c30walk::ctor2::W<false>>::finish:
./examples/c30walk/ctor2.rs:460
   61fbd:	mov    (%r11),%r10
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:3054
   61fc0:	mov    0x10(%r10),%r9
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:1040
   61fc4:	cmp    (%r10),%r9
push_mut<c30walk::RawG, alloc::alloc::Global>():
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:1040
   61fc7:	mov    %r11,0x30(%rsp)
   61fcc:	je     6221c <<c30walk::ctor2::W<false>>::finish+0x4ec>
<c30walk::ctor2::W<false>>::finish:
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:614
   61fd2:	mov    0x8(%r10),%rax
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   61fd6:	mov    %r9,%rdx
add<c30walk::RawG>():
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   61fd9:	shl    $0x7,%rdx
<c30walk::ctor2::W<false>>::finish:
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   61fdd:	pxor   %xmm0,%xmm0
write<c30walk::RawG>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   61fe1:	movdqa %xmm0,0x70(%rax,%rdx,1)
   61fe7:	movdqa %xmm0,0x60(%rax,%rdx,1)
   61fed:	movdqa %xmm0,0x50(%rax,%rdx,1)
   61ff3:	movdqa %xmm0,0x40(%rax,%rdx,1)
   61ff9:	movdqa %xmm0,0x30(%rax,%rdx,1)
   61fff:	movdqa %xmm0,0x20(%rax,%rdx,1)
   62005:	movdqa %xmm0,0x10(%rax,%rdx,1)
   6200b:	movdqa %xmm0,(%rax,%rdx,1)
push_mut<c30walk::RawG, alloc::alloc::Global>():
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:1046
   62010:	lea    0x1(%r9),%rax
   62014:	mov    %rax,0x10(%r10)
   62018:	mov    $0xffffffff,%r8d
<c30walk::ctor2::W<false>>::finish:
./examples/c30walk/ctor2.rs:513
   6201e:	cmp    %r13,%rbx
write_entries<false>():
./examples/c30walk/ctor2.rs:513
   62021:	jae    62040 <<c30walk::ctor2::W<false>>::finish+0x310>
./examples/c30walk/ctor2.rs:514
   62023:	lea    0x1(%rbx),%rax
<c30walk::ctor2::W<false>>::finish:
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:1864
   62027:	mov    0x10(%rbp),%r8
/rustc/2d8144b78/library/core/src/slice/index.rs:272
   6202b:	cmp    %r8,%rax
index<[u32; 256]>():
/rustc/2d8144b78/library/core/src/slice/index.rs:272
   6202e:	jae    622fb <<c30walk::ctor2::W<false>>::finish+0x5cb>
   62034:	shl    $0xa,%rax
   62038:	add    0x8(%rbp),%rax
write_entries<false>():
./examples/c30walk/ctor2.rs:514
   6203c:	mov    (%rax,%rsi,4),%r8d
   62040:	mov    %r9,0x8(%rsp)
<c30walk::ctor2::W<false>>::finish:
/rustc/2d8144b78/library/core/src/cmp.rs:1916
   62045:	test   %r14,%r14
/rustc/2d8144b78/library/core/src/iter/range.rs:900
   62048:	je     620f5 <<c30walk::ctor2::W<false>>::finish+0x3c5>
write_entries<false>():
   6204e:	add    0x8(%r10),%rdx
./examples/c30walk/ctor2.rs:523
   62052:	lea    0x800(,%rsi,4),%r9
   6205a:	xor    %r10d,%r10d
   6205d:	nopl   (%rax)
./examples/c30walk/ctor2.rs:520
   62060:	lea    (%r12,%r10,1),%r11d
./examples/c30walk/ctor2.rs:521
   62064:	cmp    %r8d,%r11d
   62067:	jae    620a0 <<c30walk::ctor2::W<false>>::finish+0x370>
./examples/c30walk/ctor2.rs:530
   62069:	lea    (%r10,%r15,1),%rax
   6206d:	cmp    $0x20,%rax
   62071:	jae    622a6 <<c30walk::ctor2::W<false>>::finish+0x576>
<c30walk::ctor2::W<false>>::finish:
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1733
   62077:	movzwl (%rdi,%r10,2),%r11d
write_entries<false>():
   6207c:	inc    %r10
./examples/c30walk/ctor2.rs:530
   6207f:	mov    %ebx,%r14d
   62082:	shl    $0x18,%r14d
   62086:	shl    $0x8,%r11d
   6208a:	or     %r14d,%r11d
   6208d:	or     %esi,%r11d
   62090:	mov    %r11d,(%rdx,%rax,4)
lt():
/rustc/2d8144b78/library/core/src/cmp.rs:1916
   62094:	cmp    0x10(%rsp),%r10
spec_next<usize>():
/rustc/2d8144b78/library/core/src/iter/range.rs:900
   62099:	jne    62060 <<c30walk::ctor2::W<false>>::finish+0x330>
   6209b:	jmp    620f5 <<c30walk::ctor2::W<false>>::finish+0x3c5>
write_entries<false>():
   6209d:	nopl   (%rax)
./examples/c30walk/ctor2.rs:523
   620a0:	mov    %rbx,%r14
   620a3:	shl    $0xa,%r14
   620a7:	add    %r9,%r14
   620aa:	mov    %rbx,%rax
   620ad:	jmp    620c7 <<c30walk::ctor2::W<false>>::finish+0x397>
   620af:	nop
<c30walk::ctor2::W<false>>::finish:
/rustc/2d8144b78/library/alloc/src/raw_vec/mod.rs:614
   620b0:	mov    0x8(%rbp),%rax
write_entries<false>():
./examples/c30walk/ctor2.rs:524
   620b4:	mov    (%rax,%r14,1),%r8d
./examples/c30walk/ctor2.rs:521
   620b8:	add    $0x400,%r14
   620bf:	mov    %rbx,%rax
   620c2:	cmp    %r8d,%r11d
   620c5:	jb     62069 <<c30walk::ctor2::W<false>>::finish+0x339>
./examples/c30walk/ctor2.rs:522
   620c7:	lea    0x1(%rax),%rbx
   620cb:	mov    $0xffffffff,%r8d
./examples/c30walk/ctor2.rs:523
   620d1:	cmp    %r13,%rbx
   620d4:	jae    620b8 <<c30walk::ctor2::W<false>>::finish+0x388>
./examples/c30walk/ctor2.rs:524
   620d6:	add    $0x2,%rax
<c30walk::ctor2::W<false>>::finish:
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:1864
   620da:	mov    0x10(%rbp),%r8
/rustc/2d8144b78/library/core/src/slice/index.rs:272
   620de:	cmp    %r8,%rax
index<[u32; 256]>():
/rustc/2d8144b78/library/core/src/slice/index.rs:272
   620e1:	jb     620b0 <<c30walk::ctor2::W<false>>::finish+0x380>
   620e3:	lea    0x90ed6(%rip),%rdx        # f2fc0 <__frame_dummy_init_array_entry+0x1530>
<c30walk::ctor2::W<false>>::finish:
/rustc/2d8144b78/library/core/src/slice/index.rs:272
   620ea:	mov    %rax,%rdi
index<[u32; 256]>():
/rustc/2d8144b78/library/core/src/slice/index.rs:272
   620ed:	mov    %r8,%rsi
   620f0:	call   9afa3 <core::panicking::panic_bounds_check>
<c30walk::ctor2::W<false>>::finish:
   620f5:	mov    0x30(%rsp),%r8
./examples/c30walk/ctor2.rs:463
   620fa:	mov    0x28(%r8),%rax
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   620fe:	mov    %rcx,%rdx
add<c30walk::Rec>():
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   62101:	shl    $0x6,%rdx
   62105:	mov    0x8(%rsp),%rsi
<c30walk::ctor2::W<false>>::finish:
./examples/c30walk/ctor2.rs:465
   6210a:	or     $0x10000000,%esi
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   62110:	pcmpeqd %xmm0,%xmm0
write<c30walk::Rec>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   62114:	movdqu %xmm0,0x2c(%rax,%rdx,1)
   6211a:	movdqa %xmm0,0x20(%rax,%rdx,1)
   62120:	movdqa %xmm0,0x10(%rax,%rdx,1)
   62126:	movdqa %xmm0,(%rax,%rdx,1)
   6212b:	mov    %esi,0x3c(%rax,%rdx,1)
<c30walk::ctor2::W<false>>::finish:
./examples/c30walk/ctor2.rs:467
   6212f:	mov    %ecx,%edi
   62131:	shr    $0x6,%edi
   62134:	mov    0x10(%r8),%rsi
   62138:	cmp    %rsi,%rdi
   6213b:	jae    622ef <<c30walk::ctor2::W<false>>::finish+0x5bf>
   62141:	mov    $0x1,%eax
   62146:	shl    %cl,%rax
   62149:	mov    0x8(%r8),%rcx
   6214d:	or     %rax,(%rcx,%rdi,8)
./examples/c30walk/ctor2.rs:429
   62151:	jmp    6231b <<c30walk::ctor2::W<false>>::finish+0x5eb>
   62156:	mov    0x8(%rsp),%rcx
/rustc/2d8144b78/library/core/src/iter/range.rs:900
   6215b:	cmp    $0x8,%rcx
spec_next<usize>():
/rustc/2d8144b78/library/core/src/iter/range.rs:900
   6215f:	jae    62168 <<c30walk::ctor2::W<false>>::finish+0x438>
   62161:	xor    %ecx,%ecx
   62163:	jmp    62312 <<c30walk::ctor2::W<false>>::finish+0x5e2>
   62168:	movabs $0x7ffffffffffffff0,%rsi
   62172:	cmp    $0x20,%rcx
   62176:	jae    6217c <<c30walk::ctor2::W<false>>::finish+0x44c>
   62178:	xor    %ecx,%ecx
   6217a:	jmp    621db <<c30walk::ctor2::W<false>>::finish+0x4ab>
<c30walk::ctor2::W<false>>::finish:
   6217c:	mov    %r14,%rcx
   6217f:	and    %rsi,%rcx
   62182:	movd   %eax,%xmm0
   62186:	pshuflw $0x0,%xmm0,%xmm0
   6218b:	pshufd $0x44,%xmm0,%xmm0
   62190:	xor    %edx,%edx
   62192:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1733
   621a0:	movdqu (%rdi,%rdx,2),%xmm1
read<u16>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1733
   621a5:	movdqu 0x10(%rdi,%rdx,2),%xmm2
<c30walk::ctor2::W<false>>::finish:
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:2621
   621ab:	psubw  %xmm0,%xmm1
wrapping_sub():
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:2621
   621af:	psubw  %xmm0,%xmm2
<c30walk::ctor2::W<false>>::finish:
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   621b3:	movdqu %xmm1,(%rdi,%rdx,2)
write<u16>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   621b8:	movdqu %xmm2,0x10(%rdi,%rdx,2)
<c30walk::ctor2::W<false>>::finish:
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:971
   621be:	add    $0x10,%rdx
spec_next<usize>():
/rustc/2d8144b78/library/core/src/iter/range.rs:900
   621c2:	cmp    %rdx,%rcx
   621c5:	jne    621a0 <<c30walk::ctor2::W<false>>::finish+0x470>
   621c7:	cmp    %rcx,%r14
   621ca:	je     6231b <<c30walk::ctor2::W<false>>::finish+0x5eb>
<c30walk::ctor2::W<false>>::finish:
   621d0:	testb  $0x18,0x8(%rsp)
   621d5:	je     62312 <<c30walk::ctor2::W<false>>::finish+0x5e2>
   621db:	mov    %rcx,%rdx
   621de:	add    $0xc,%rsi
   621e2:	mov    %rsi,%rcx
   621e5:	and    %r14,%rcx
   621e8:	movd   %eax,%xmm0
   621ec:	pshuflw $0x0,%xmm0,%xmm0
   621f1:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
read<u16>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1733
   62200:	movq   (%rdi,%rdx,2),%xmm1
wrapping_sub():
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:2621
   62205:	psubw  %xmm0,%xmm1
write<u16>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   62209:	movq   %xmm1,(%rdi,%rdx,2)
unchecked_add():
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:971
   6220e:	add    $0x4,%rdx
spec_next<usize>():
/rustc/2d8144b78/library/core/src/iter/range.rs:900
   62212:	cmp    %rdx,%rcx
   62215:	jne    62200 <<c30walk::ctor2::W<false>>::finish+0x4d0>
   62217:	jmp    6230d <<c30walk::ctor2::W<false>>::finish+0x5dd>
<c30walk::ctor2::W<false>>::finish:
   6221c:	mov    %rdi,0x8(%rsp)
push_mut<c30walk::RawG, alloc::alloc::Global>():
/rustc/2d8144b78/library/alloc/src/vec/mod.rs:1041
   62221:	mov    %r10,%rdi
   62224:	mov    %rsi,0x20(%rsp)
   62229:	mov    %rcx,0x28(%rsp)
   6222e:	mov    %r9,%r14
   62231:	mov    %r10,0x18(%rsp)
   62236:	call   65930 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   6223b:	mov    0x18(%rsp),%r10
   62240:	mov    %r14,%r9
   62243:	mov    0x28(%rsp),%rcx
   62248:	mov    0x20(%rsp),%rsi
   6224d:	mov    0x10(%rsp),%r14
   62252:	mov    0x8(%rsp),%rdi
   62257:	jmp    61fd2 <<c30walk::ctor2::W<false>>::finish+0x2a2>
index<[u32; 256]>():
/rustc/2d8144b78/library/core/src/slice/index.rs:272
   6225c:	lea    0x90d5d(%rip),%rdx        # f2fc0 <__frame_dummy_init_array_entry+0x1530>
<c30walk::ctor2::W<false>>::finish:
/rustc/2d8144b78/library/core/src/slice/index.rs:272
   62263:	mov    %rcx,%rdi
index<[u32; 256]>():
/rustc/2d8144b78/library/core/src/slice/index.rs:272
   62266:	mov    %r8,%rsi
   62269:	call   9afa3 <core::panicking::panic_bounds_check>
   6226e:	lea    0x90e53(%rip),%rdx        # f30c8 <__frame_dummy_init_array_entry+0x1638>
   62275:	mov    %rbx,%rdi
   62278:	mov    %r8,%rsi
   6227b:	call   9afa3 <core::panicking::panic_bounds_check>
   62280:	lea    0x90e41(%rip),%rdx        # f30c8 <__frame_dummy_init_array_entry+0x1638>
   62287:	mov    %r13,%rdi
   6228a:	mov    %r8,%rsi
   6228d:	call   9afa3 <core::panicking::panic_bounds_check>
write_entries<false>():
./examples/c30walk/ctor2.rs:530
   62292:	lea    0x90d0f(%rip),%rdx        # f2fa8 <__frame_dummy_init_array_entry+0x1518>
   62299:	mov    $0x20,%esi
   6229e:	mov    %rcx,%rdi
   622a1:	call   9afa3 <core::panicking::panic_bounds_check>
   622a6:	lea    0x90cfb(%rip),%rdx        # f2fa8 <__frame_dummy_init_array_entry+0x1518>
   622ad:	mov    $0x20,%esi
   622b2:	mov    %rax,%rdi
   622b5:	call   9afa3 <core::panicking::panic_bounds_check>
index<[u32; 256]>():
/rustc/2d8144b78/library/core/src/slice/index.rs:272
   622ba:	lea    0x90d77(%rip),%rdx        # f3038 <__frame_dummy_init_array_entry+0x15a8>
   622c1:	mov    %r13,%rdi
   622c4:	xor    %esi,%esi
   622c6:	call   9afa3 <core::panicking::panic_bounds_check>
index_mut<c30walk::RawG>():
/rustc/2d8144b78/library/core/src/slice/index.rs:278
   622cb:	lea    0x90cbe(%rip),%rdx        # f2f90 <__frame_dummy_init_array_entry+0x1500>
   622d2:	mov    %rax,%rdi
   622d5:	mov    %rcx,%rsi
   622d8:	call   9afa3 <core::panicking::panic_bounds_check>
index<[u32; 256]>():
/rustc/2d8144b78/library/core/src/slice/index.rs:272
   622dd:	lea    0x90c94(%rip),%rdx        # f2f78 <__frame_dummy_init_array_entry+0x14e8>
   622e4:	mov    %rcx,%rdi
   622e7:	mov    %r8,%rsi
   622ea:	call   9afa3 <core::panicking::panic_bounds_check>
<c30walk::ctor2::W<false>>::finish:
./examples/c30walk/ctor2.rs:467
   622ef:	lea    0x90d5a(%rip),%rdx        # f3050 <__frame_dummy_init_array_entry+0x15c0>
   622f6:	call   9afa3 <core::panicking::panic_bounds_check>
index<[u32; 256]>():
/rustc/2d8144b78/library/core/src/slice/index.rs:272
   622fb:	lea    0x90c76(%rip),%rdx        # f2f78 <__frame_dummy_init_array_entry+0x14e8>
   62302:	mov    %rax,%rdi
   62305:	mov    %r8,%rsi
   62308:	call   9afa3 <core::panicking::panic_bounds_check>
spec_next<usize>():
/rustc/2d8144b78/library/core/src/iter/range.rs:900
   6230d:	cmp    %rcx,%r14
   62310:	je     6231b <<c30walk::ctor2::W<false>>::finish+0x5eb>
write<u16>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   62312:	sub    %ax,(%rdi,%rcx,2)
unchecked_add():
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:971
   62316:	inc    %rcx
<c30walk::ctor2::W<false>>::finish:
/rustc/2d8144b78/library/core/src/cmp.rs:1916
   62319:	jmp    6230d <<c30walk::ctor2::W<false>>::finish+0x5dd>
./examples/c30walk/ctor2.rs:495
   6231b:	add    $0x38,%rsp
   6231f:	pop    %rbx
   62320:	pop    %r12
   62322:	pop    %r13
   62324:	pop    %r14
   62326:	pop    %r15
   62328:	pop    %rbp
   62329:	ret
