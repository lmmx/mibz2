# <c30walk::ctor2c::W<false>>::finish_slow
# size 0x608

target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

000000000008a7b0 <<c30walk::ctor2c::W<false>>::finish_slow>:
   8a7b0:	push   %rbp
   8a7b1:	push   %r15
   8a7b3:	push   %r14
   8a7b5:	push   %r13
   8a7b7:	push   %r12
   8a7b9:	push   %rbx
   8a7ba:	sub    $0x38,%rsp
   8a7be:	mov    0x18(%rdi),%rbp
   8a7c2:	mov    0x10(%rbp),%r8
   8a7c6:	mov    %r8,%r13
   8a7c9:	sub    $0x1,%r13
   8a7cd:	jb     8ad4a <<c30walk::ctor2c::W<false>>::finish_slow+0x59a>
   8a7d3:	mov    %rdi,%r11
   8a7d6:	lea    (%rsi,%rsi,2),%rax
   8a7da:	mov    0x1040(%rdi,%rax,8),%rdi
   8a7e2:	mov    0x1048(%r11,%rax,8),%r12d
   8a7ea:	mov    0x1050(%r11,%rax,8),%ecx
   8a7f2:	mov    0x1054(%r11,%rax,8),%eax
   8a7fa:	mov    0x40(%r11,%rsi,8),%r14
   8a7ff:	sub    %rdi,%r14
   8a802:	mov    %r14,0x8(%rsp)
   8a807:	shr    $1,%r14
   8a80a:	mov    0x8(%rbp),%r10
   8a80e:	mov    %r13,%rdx
   8a811:	shl    $0xa,%rdx
   8a815:	add    %r10,%rdx
   8a818:	mov    %r13,%rbx
   8a81b:	cmp    %r12d,(%rdx,%rsi,4)
   8a81f:	jbe    8a898 <<c30walk::ctor2c::W<false>>::finish_slow+0xe8>
   8a821:	mov    %r11,%r15
   8a824:	lea    (%r12,%r14,1),%edx
   8a828:	dec    %edx
   8a82a:	mov    %r8,%r9
   8a82d:	shl    $0xa,%r9
   8a831:	lea    (%r9,%rsi,4),%r9
   8a835:	lea    (%r10,%r9,1),%r11
   8a839:	add    $0xfffffffffffffc00,%r11
   8a840:	mov    %r8,%rbx
   8a843:	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   8a850:	dec    %rbx
   8a853:	cmp    %r8,%rbx
   8a856:	jae    8acfe <<c30walk::ctor2c::W<false>>::finish_slow+0x54e>
   8a85c:	cmp    %r12d,(%r11)
   8a85f:	lea    -0x400(%r11),%r11
   8a866:	ja     8a850 <<c30walk::ctor2c::W<false>>::finish_slow+0xa0>
   8a868:	add    %r10,%r9
   8a86b:	add    $0xfffffffffffffc00,%r9
   8a872:	mov    %r8,%r13
   8a875:	mov    %r15,%r11
   8a878:	nopl   0x0(%rax,%rax,1)
   8a880:	dec    %r13
   8a883:	cmp    %r8,%r13
   8a886:	jae    8ad10 <<c30walk::ctor2c::W<false>>::finish_slow+0x560>
   8a88c:	cmp    %edx,(%r9)
   8a88f:	lea    -0x400(%r9),%r9
   8a896:	ja     8a880 <<c30walk::ctor2c::W<false>>::finish_slow+0xd0>
   8a898:	imul   $0xffffffffffffffe2,%rcx,%r15
   8a89c:	add    %r12,%r15
   8a89f:	test   %eax,%eax
   8a8a1:	mov    %r14,0x10(%rsp)
   8a8a6:	js     8a9ab <<c30walk::ctor2c::W<false>>::finish_slow+0x1fb>
   8a8ac:	mov    $0xffffffff,%r9d
   8a8b2:	cmp    %r13,%rbx
   8a8b5:	jae    8a8cf <<c30walk::ctor2c::W<false>>::finish_slow+0x11f>
   8a8b7:	lea    0x1(%rbx),%rcx
   8a8bb:	cmp    %r8,%rcx
   8a8be:	jae    8ad6d <<c30walk::ctor2c::W<false>>::finish_slow+0x5bd>
   8a8c4:	shl    $0xa,%rcx
   8a8c8:	add    %rcx,%r10
   8a8cb:	mov    (%r10,%rsi,4),%r9d
   8a8cf:	mov    (%r11),%rdx
   8a8d2:	mov    0x10(%rdx),%rcx
   8a8d6:	cmp    %rax,%rcx
   8a8d9:	jbe    8ad5b <<c30walk::ctor2c::W<false>>::finish_slow+0x5ab>
   8a8df:	test   %r14,%r14
   8a8e2:	je     8ada9 <<c30walk::ctor2c::W<false>>::finish_slow+0x5f9>
   8a8e8:	shl    $0x7,%rax
   8a8ec:	add    0x8(%rdx),%rax
   8a8f0:	lea    0x800(,%rsi,4),%rdx
   8a8f8:	xor    %r11d,%r11d
   8a8fb:	nopl   0x0(%rax,%rax,1)
   8a900:	lea    (%r12,%r11,1),%r10d
   8a904:	cmp    %r9d,%r10d
   8a907:	jae    8a940 <<c30walk::ctor2c::W<false>>::finish_slow+0x190>
   8a909:	lea    (%r11,%r15,1),%rcx
   8a90d:	cmp    $0x20,%rcx
   8a911:	jae    8ad22 <<c30walk::ctor2c::W<false>>::finish_slow+0x572>
   8a917:	movzwl (%rdi,%r11,2),%r8d
   8a91c:	inc    %r11
   8a91f:	mov    %ebx,%r10d
   8a922:	shl    $0x18,%r10d
   8a926:	shl    $0x8,%r8d
   8a92a:	or     %r10d,%r8d
   8a92d:	or     %esi,%r8d
   8a930:	mov    %r8d,(%rax,%rcx,4)
   8a934:	cmp    0x10(%rsp),%r11
   8a939:	jne    8a900 <<c30walk::ctor2c::W<false>>::finish_slow+0x150>
   8a93b:	jmp    8a989 <<c30walk::ctor2c::W<false>>::finish_slow+0x1d9>
   8a93d:	nopl   (%rax)
   8a940:	mov    %rbx,%r14
   8a943:	shl    $0xa,%r14
   8a947:	add    %rdx,%r14
   8a94a:	mov    %rbx,%rcx
   8a94d:	jmp    8a95f <<c30walk::ctor2c::W<false>>::finish_slow+0x1af>
   8a94f:	nop
   8a950:	add    $0x400,%r14
   8a957:	mov    %rbx,%rcx
   8a95a:	cmp    %r9d,%r10d
   8a95d:	jb     8a909 <<c30walk::ctor2c::W<false>>::finish_slow+0x159>
   8a95f:	lea    0x1(%rcx),%rbx
   8a963:	mov    $0xffffffff,%r9d
   8a969:	cmp    %r13,%rbx
   8a96c:	jae    8a950 <<c30walk::ctor2c::W<false>>::finish_slow+0x1a0>
   8a96e:	add    $0x2,%rcx
   8a972:	mov    0x10(%rbp),%r8
   8a976:	cmp    %r8,%rcx
   8a979:	jae    8acec <<c30walk::ctor2c::W<false>>::finish_slow+0x53c>
   8a97f:	mov    0x8(%rbp),%rcx
   8a983:	mov    (%rcx,%r14,1),%r9d
   8a987:	jmp    8a950 <<c30walk::ctor2c::W<false>>::finish_slow+0x1a0>
   8a989:	mov    0x8(%rsp),%rdx
   8a98e:	and    $0xfffffffffffffffe,%rdx
   8a992:	mov    $0xff,%esi
   8a997:	add    $0x38,%rsp
   8a99b:	pop    %rbx
   8a99c:	pop    %r12
   8a99e:	pop    %r13
   8a9a0:	pop    %r14
   8a9a2:	pop    %r15
   8a9a4:	pop    %rbp
   8a9a5:	jmp    *0xb9595(%rip)        # 143f40 <memset@GLIBC_2.2.5>
   8a9ab:	movzwl (%rdi),%eax
   8a9ae:	movzwl -0x2(%rdi,%r14,2),%r8d
   8a9b4:	mov    %ebx,%edx
   8a9b6:	shl    $0x10,%edx
   8a9b9:	or     %eax,%edx
   8a9bb:	mov    %r13d,%r9d
   8a9be:	shl    $0x10,%r9d
   8a9c2:	or     %r8d,%r9d
   8a9c5:	sub    %edx,%r9d
   8a9c8:	cmp    $0xfffe,%r9d
   8a9cf:	jae    8aa4d <<c30walk::ctor2c::W<false>>::finish_slow+0x29d>
   8a9d1:	mov    0x28(%r11),%r8
   8a9d5:	shl    $0x6,%rcx
   8a9d9:	shl    $0x8,%edx
   8a9dc:	or     %edx,%esi
   8a9de:	mov    %esi,0x3c(%r8,%rcx,1)
   8a9e3:	test   %r14,%r14
   8a9e6:	je     8ada9 <<c30walk::ctor2c::W<false>>::finish_slow+0x5f9>
   8a9ec:	cmp    $0x1e,%r14
   8a9f0:	jne    8abe6 <<c30walk::ctor2c::W<false>>::finish_slow+0x436>
   8a9f6:	movd   %eax,%xmm0
   8a9fa:	pshuflw $0x0,%xmm0,%xmm0
   8a9ff:	pshufd $0x44,%xmm0,%xmm1
   8aa04:	movdqu (%rdi),%xmm2
   8aa08:	movdqu 0x10(%rdi),%xmm3
   8aa0d:	movdqu 0x20(%rdi),%xmm4
   8aa12:	psubw  %xmm1,%xmm2
   8aa16:	movdqu %xmm2,(%rdi)
   8aa1a:	psubw  %xmm1,%xmm3
   8aa1e:	movdqu %xmm3,0x10(%rdi)
   8aa23:	psubw  %xmm1,%xmm4
   8aa27:	movdqu %xmm4,0x20(%rdi)
   8aa2c:	movq   0x30(%rdi),%xmm1
   8aa31:	psubw  %xmm0,%xmm1
   8aa35:	movq   %xmm1,0x30(%rdi)
   8aa3a:	movd   0x38(%rdi),%xmm1
   8aa3f:	psubw  %xmm0,%xmm1
   8aa43:	movd   %xmm1,0x38(%rdi)
   8aa48:	jmp    8ada9 <<c30walk::ctor2c::W<false>>::finish_slow+0x5f9>
   8aa4d:	mov    (%r11),%r10
   8aa50:	mov    0x10(%r10),%r9
   8aa54:	cmp    (%r10),%r9
   8aa57:	mov    %r11,0x30(%rsp)
   8aa5c:	je     8acac <<c30walk::ctor2c::W<false>>::finish_slow+0x4fc>
   8aa62:	mov    0x8(%r10),%rax
   8aa66:	mov    %r9,%rdx
   8aa69:	shl    $0x7,%rdx
   8aa6d:	pxor   %xmm0,%xmm0
   8aa71:	movdqa %xmm0,0x70(%rax,%rdx,1)
   8aa77:	movdqa %xmm0,0x60(%rax,%rdx,1)
   8aa7d:	movdqa %xmm0,0x50(%rax,%rdx,1)
   8aa83:	movdqa %xmm0,0x40(%rax,%rdx,1)
   8aa89:	movdqa %xmm0,0x30(%rax,%rdx,1)
   8aa8f:	movdqa %xmm0,0x20(%rax,%rdx,1)
   8aa95:	movdqa %xmm0,0x10(%rax,%rdx,1)
   8aa9b:	movdqa %xmm0,(%rax,%rdx,1)
   8aaa0:	lea    0x1(%r9),%rax
   8aaa4:	mov    %rax,0x10(%r10)
   8aaa8:	mov    $0xffffffff,%r8d
   8aaae:	cmp    %r13,%rbx
   8aab1:	jae    8aad0 <<c30walk::ctor2c::W<false>>::finish_slow+0x320>
   8aab3:	lea    0x1(%rbx),%rax
   8aab7:	mov    0x10(%rbp),%r8
   8aabb:	cmp    %r8,%rax
   8aabe:	jae    8ad8b <<c30walk::ctor2c::W<false>>::finish_slow+0x5db>
   8aac4:	shl    $0xa,%rax
   8aac8:	add    0x8(%rbp),%rax
   8aacc:	mov    (%rax,%rsi,4),%r8d
   8aad0:	mov    %r9,0x8(%rsp)
   8aad5:	test   %r14,%r14
   8aad8:	je     8ab85 <<c30walk::ctor2c::W<false>>::finish_slow+0x3d5>
   8aade:	add    0x8(%r10),%rdx
   8aae2:	lea    0x800(,%rsi,4),%r9
   8aaea:	xor    %r10d,%r10d
   8aaed:	nopl   (%rax)
   8aaf0:	lea    (%r12,%r10,1),%r11d
   8aaf4:	cmp    %r8d,%r11d
   8aaf7:	jae    8ab30 <<c30walk::ctor2c::W<false>>::finish_slow+0x380>
   8aaf9:	lea    (%r10,%r15,1),%rax
   8aafd:	cmp    $0x20,%rax
   8ab01:	jae    8ad36 <<c30walk::ctor2c::W<false>>::finish_slow+0x586>
   8ab07:	movzwl (%rdi,%r10,2),%r11d
   8ab0c:	inc    %r10
   8ab0f:	mov    %ebx,%r14d
   8ab12:	shl    $0x18,%r14d
   8ab16:	shl    $0x8,%r11d
   8ab1a:	or     %r14d,%r11d
   8ab1d:	or     %esi,%r11d
   8ab20:	mov    %r11d,(%rdx,%rax,4)
   8ab24:	cmp    0x10(%rsp),%r10
   8ab29:	jne    8aaf0 <<c30walk::ctor2c::W<false>>::finish_slow+0x340>
   8ab2b:	jmp    8ab85 <<c30walk::ctor2c::W<false>>::finish_slow+0x3d5>
   8ab2d:	nopl   (%rax)
   8ab30:	mov    %rbx,%r14
   8ab33:	shl    $0xa,%r14
   8ab37:	add    %r9,%r14
   8ab3a:	mov    %rbx,%rax
   8ab3d:	jmp    8ab57 <<c30walk::ctor2c::W<false>>::finish_slow+0x3a7>
   8ab3f:	nop
   8ab40:	mov    0x8(%rbp),%rax
   8ab44:	mov    (%rax,%r14,1),%r8d
   8ab48:	add    $0x400,%r14
   8ab4f:	mov    %rbx,%rax
   8ab52:	cmp    %r8d,%r11d
   8ab55:	jb     8aaf9 <<c30walk::ctor2c::W<false>>::finish_slow+0x349>
   8ab57:	lea    0x1(%rax),%rbx
   8ab5b:	mov    $0xffffffff,%r8d
   8ab61:	cmp    %r13,%rbx
   8ab64:	jae    8ab48 <<c30walk::ctor2c::W<false>>::finish_slow+0x398>
   8ab66:	add    $0x2,%rax
   8ab6a:	mov    0x10(%rbp),%r8
   8ab6e:	cmp    %r8,%rax
   8ab71:	jb     8ab40 <<c30walk::ctor2c::W<false>>::finish_slow+0x390>
   8ab73:	lea    0xb413e(%rip),%rdx        # 13ecb8 <__frame_dummy_init_array_entry+0x17d8>
   8ab7a:	mov    %rax,%rdi
   8ab7d:	mov    %r8,%rsi
   8ab80:	call   e6ac3 <core::panicking::panic_bounds_check>
   8ab85:	mov    0x30(%rsp),%r8
   8ab8a:	mov    0x28(%r8),%rax
   8ab8e:	mov    %rcx,%rdx
   8ab91:	shl    $0x6,%rdx
   8ab95:	mov    0x8(%rsp),%rsi
   8ab9a:	or     $0x10000000,%esi
   8aba0:	pcmpeqd %xmm0,%xmm0
   8aba4:	movdqu %xmm0,0x2c(%rax,%rdx,1)
   8abaa:	movdqa %xmm0,0x20(%rax,%rdx,1)
   8abb0:	movdqa %xmm0,0x10(%rax,%rdx,1)
   8abb6:	movdqa %xmm0,(%rax,%rdx,1)
   8abbb:	mov    %esi,0x3c(%rax,%rdx,1)
   8abbf:	mov    %ecx,%edi
   8abc1:	shr    $0x6,%edi
   8abc4:	mov    0x10(%r8),%rsi
   8abc8:	cmp    %rsi,%rdi
   8abcb:	jae    8ad7f <<c30walk::ctor2c::W<false>>::finish_slow+0x5cf>
   8abd1:	mov    $0x1,%eax
   8abd6:	shl    %cl,%rax
   8abd9:	mov    0x8(%r8),%rcx
   8abdd:	or     %rax,(%rcx,%rdi,8)
   8abe1:	jmp    8ada9 <<c30walk::ctor2c::W<false>>::finish_slow+0x5f9>
   8abe6:	mov    0x8(%rsp),%rcx
   8abeb:	cmp    $0x8,%rcx
   8abef:	jae    8abf8 <<c30walk::ctor2c::W<false>>::finish_slow+0x448>
   8abf1:	xor    %ecx,%ecx
   8abf3:	jmp    8ad9d <<c30walk::ctor2c::W<false>>::finish_slow+0x5ed>
   8abf8:	movabs $0x7ffffffffffffff0,%rsi
   8ac02:	cmp    $0x20,%rcx
   8ac06:	jae    8ac0c <<c30walk::ctor2c::W<false>>::finish_slow+0x45c>
   8ac08:	xor    %ecx,%ecx
   8ac0a:	jmp    8ac6b <<c30walk::ctor2c::W<false>>::finish_slow+0x4bb>
   8ac0c:	mov    %r14,%rcx
   8ac0f:	and    %rsi,%rcx
   8ac12:	movd   %eax,%xmm0
   8ac16:	pshuflw $0x0,%xmm0,%xmm0
   8ac1b:	pshufd $0x44,%xmm0,%xmm0
   8ac20:	xor    %edx,%edx
   8ac22:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   8ac30:	movdqu (%rdi,%rdx,2),%xmm1
   8ac35:	movdqu 0x10(%rdi,%rdx,2),%xmm2
   8ac3b:	psubw  %xmm0,%xmm1
   8ac3f:	psubw  %xmm0,%xmm2
   8ac43:	movdqu %xmm1,(%rdi,%rdx,2)
   8ac48:	movdqu %xmm2,0x10(%rdi,%rdx,2)
   8ac4e:	add    $0x10,%rdx
   8ac52:	cmp    %rdx,%rcx
   8ac55:	jne    8ac30 <<c30walk::ctor2c::W<false>>::finish_slow+0x480>
   8ac57:	cmp    %rcx,%r14
   8ac5a:	je     8ada9 <<c30walk::ctor2c::W<false>>::finish_slow+0x5f9>
   8ac60:	testb  $0x18,0x8(%rsp)
   8ac65:	je     8ad9d <<c30walk::ctor2c::W<false>>::finish_slow+0x5ed>
   8ac6b:	mov    %rcx,%rdx
   8ac6e:	add    $0xc,%rsi
   8ac72:	mov    %rsi,%rcx
   8ac75:	and    %r14,%rcx
   8ac78:	movd   %eax,%xmm0
   8ac7c:	pshuflw $0x0,%xmm0,%xmm0
   8ac81:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   8ac90:	movq   (%rdi,%rdx,2),%xmm1
   8ac95:	psubw  %xmm0,%xmm1
   8ac99:	movq   %xmm1,(%rdi,%rdx,2)
   8ac9e:	add    $0x4,%rdx
   8aca2:	cmp    %rdx,%rcx
   8aca5:	jne    8ac90 <<c30walk::ctor2c::W<false>>::finish_slow+0x4e0>
   8aca7:	jmp    8ada4 <<c30walk::ctor2c::W<false>>::finish_slow+0x5f4>
   8acac:	mov    %rdi,0x8(%rsp)
   8acb1:	mov    %r10,%rdi
   8acb4:	mov    %rsi,0x20(%rsp)
   8acb9:	mov    %rcx,0x28(%rsp)
   8acbe:	mov    %r9,%r14
   8acc1:	mov    %r10,0x18(%rsp)
   8acc6:	call   99a50 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   8accb:	mov    0x18(%rsp),%r10
   8acd0:	mov    %r14,%r9
   8acd3:	mov    0x28(%rsp),%rcx
   8acd8:	mov    0x20(%rsp),%rsi
   8acdd:	mov    0x10(%rsp),%r14
   8ace2:	mov    0x8(%rsp),%rdi
   8ace7:	jmp    8aa62 <<c30walk::ctor2c::W<false>>::finish_slow+0x2b2>
   8acec:	lea    0xb3fc5(%rip),%rdx        # 13ecb8 <__frame_dummy_init_array_entry+0x17d8>
   8acf3:	mov    %rcx,%rdi
   8acf6:	mov    %r8,%rsi
   8acf9:	call   e6ac3 <core::panicking::panic_bounds_check>
   8acfe:	lea    0xb408b(%rip),%rdx        # 13ed90 <__frame_dummy_init_array_entry+0x18b0>
   8ad05:	mov    %rbx,%rdi
   8ad08:	mov    %r8,%rsi
   8ad0b:	call   e6ac3 <core::panicking::panic_bounds_check>
   8ad10:	lea    0xb4079(%rip),%rdx        # 13ed90 <__frame_dummy_init_array_entry+0x18b0>
   8ad17:	mov    %r13,%rdi
   8ad1a:	mov    %r8,%rsi
   8ad1d:	call   e6ac3 <core::panicking::panic_bounds_check>
   8ad22:	lea    0xb3f77(%rip),%rdx        # 13eca0 <__frame_dummy_init_array_entry+0x17c0>
   8ad29:	mov    $0x20,%esi
   8ad2e:	mov    %rcx,%rdi
   8ad31:	call   e6ac3 <core::panicking::panic_bounds_check>
   8ad36:	lea    0xb3f63(%rip),%rdx        # 13eca0 <__frame_dummy_init_array_entry+0x17c0>
   8ad3d:	mov    $0x20,%esi
   8ad42:	mov    %rax,%rdi
   8ad45:	call   e6ac3 <core::panicking::panic_bounds_check>
   8ad4a:	lea    0xb3eef(%rip),%rdx        # 13ec40 <__frame_dummy_init_array_entry+0x1760>
   8ad51:	mov    %r13,%rdi
   8ad54:	xor    %esi,%esi
   8ad56:	call   e6ac3 <core::panicking::panic_bounds_check>
   8ad5b:	lea    0xb3f26(%rip),%rdx        # 13ec88 <__frame_dummy_init_array_entry+0x17a8>
   8ad62:	mov    %rax,%rdi
   8ad65:	mov    %rcx,%rsi
   8ad68:	call   e6ac3 <core::panicking::panic_bounds_check>
   8ad6d:	lea    0xb3efc(%rip),%rdx        # 13ec70 <__frame_dummy_init_array_entry+0x1790>
   8ad74:	mov    %rcx,%rdi
   8ad77:	mov    %r8,%rsi
   8ad7a:	call   e6ac3 <core::panicking::panic_bounds_check>
   8ad7f:	lea    0xb3ed2(%rip),%rdx        # 13ec58 <__frame_dummy_init_array_entry+0x1778>
   8ad86:	call   e6ac3 <core::panicking::panic_bounds_check>
   8ad8b:	lea    0xb3ede(%rip),%rdx        # 13ec70 <__frame_dummy_init_array_entry+0x1790>
   8ad92:	mov    %rax,%rdi
   8ad95:	mov    %r8,%rsi
   8ad98:	call   e6ac3 <core::panicking::panic_bounds_check>
   8ad9d:	sub    %ax,(%rdi,%rcx,2)
   8ada1:	inc    %rcx
   8ada4:	cmp    %rcx,%r14
   8ada7:	jne    8ad9d <<c30walk::ctor2c::W<false>>::finish_slow+0x5ed>
   8ada9:	add    $0x38,%rsp
   8adad:	pop    %rbx
   8adae:	pop    %r12
   8adb0:	pop    %r13
   8adb2:	pop    %r14
   8adb4:	pop    %r15
   8adb6:	pop    %rbp
   8adb7:	ret
