tgt-base/release/c30cg:     file format elf64-x86-64
Disassembly of section .text:
000000000003db50 <mibz2::bwt::build>:
   3db50:	push   %rbp
   3db51:	push   %r15
   3db53:	push   %r14
   3db55:	push   %r13
   3db57:	push   %r12
   3db59:	push   %rbx
   3db5a:	sub    $0x808,%rsp
   3db61:	mov    %rdx,%r15
   3db64:	mov    %rsi,%rbx
   3db67:	mov    %rdi,%r14
   3db6a:	movq   $0x0,0x10(%rcx)
   3db72:	cmp    $0xffffff,%rsi
   3db79:	ja     3dcf8 <mibz2::bwt::build+0x1a8>
   3db7f:	pxor   %xmm0,%xmm0
   3db83:	xor    %eax,%eax
   3db85:	pxor   %xmm2,%xmm2
   3db89:	pxor   %xmm1,%xmm1
   3db8d:	nopl   (%rax)
   3db90:	movq   (%r15,%rax,4),%xmm3
   3db96:	movq   0x8(%r15,%rax,4),%xmm4
   3db9d:	punpckldq %xmm0,%xmm3
   3dba1:	movdqa %xmm2,%xmm5
   3dba5:	paddq  %xmm3,%xmm5
   3dba9:	punpckldq %xmm0,%xmm4
   3dbad:	movdqa %xmm1,%xmm3
   3dbb1:	paddq  %xmm4,%xmm3
   3dbb5:	movq   0x10(%r15,%rax,4),%xmm2
   3dbbc:	movq   0x18(%r15,%rax,4),%xmm1
   3dbc3:	punpckldq %xmm0,%xmm2
   3dbc7:	paddq  %xmm5,%xmm2
   3dbcb:	punpckldq %xmm0,%xmm1
   3dbcf:	paddq  %xmm3,%xmm1
   3dbd3:	add    $0x8,%rax
   3dbd7:	cmp    $0x100,%rax
   3dbdd:	jne    3db90 <mibz2::bwt::build+0x40>
   3dbdf:	paddq  %xmm2,%xmm1
   3dbe3:	pshufd $0xee,%xmm1,%xmm0
   3dbe8:	paddq  %xmm1,%xmm0
   3dbec:	movq   %xmm0,%rax
   3dbf1:	cmp    %rbx,%rax
   3dbf4:	jne    3dcf8 <mibz2::bwt::build+0x1a8>
   3dbfa:	mov    %rcx,%r12
   3dbfd:	lea    0x8(%rsp),%rdi
   3dc02:	xor    %ebp,%ebp
   3dc04:	mov    0x5febd(%rip),%r13        # 9dac8 <memset@GLIBC_2.2.5>
   3dc0b:	mov    $0x400,%edx
   3dc10:	xor    %esi,%esi
   3dc12:	call   *%r13
   3dc15:	lea    0x408(%rsp),%rdi
   3dc1d:	mov    $0x400,%edx
   3dc22:	xor    %esi,%esi
   3dc24:	call   *%r13
   3dc27:	mov    $0x1,%eax
   3dc2c:	nopl   0x0(%rax)
   3dc30:	mov    %ebp,0x404(%rsp,%rax,4)
   3dc37:	add    -0x4(%r15,%rax,4),%ebp
   3dc3c:	mov    %ebp,0x4(%rsp,%rax,4)
   3dc40:	mov    %ebp,0x408(%rsp,%rax,4)
   3dc47:	add    (%r15,%rax,4),%ebp
   3dc4b:	mov    %ebp,0x8(%rsp,%rax,4)
   3dc4f:	add    $0x2,%rax
   3dc53:	cmp    $0x101,%rax
   3dc59:	jne    3dc30 <mibz2::bwt::build+0xe0>
   3dc5b:	mov    %r12,%rcx
   3dc5e:	cmp    (%r12),%rbx
   3dc62:	ja     3ddb6 <mibz2::bwt::build+0x266>
   3dc68:	mov    0x8(%rcx),%rdx
   3dc6c:	xor    %r8d,%r8d
   3dc6f:	cmp    $0x10,%rbx
   3dc73:	jae    3dde2 <mibz2::bwt::build+0x292>
   3dc79:	xor    %eax,%eax
   3dc7b:	cmp    %rbx,%rax
   3dc7e:	jae    3dcd9 <mibz2::bwt::build+0x189>
   3dc80:	mov    %eax,%esi
   3dc82:	shl    $0x8,%esi
   3dc85:	data16 cs nopw 0x0(%rax,%rax,1)
   3dc90:	movzbl (%r14,%rax,1),%r8d
   3dc95:	lea    0x0(,%r8,4),%r9d
   3dc9d:	mov    0x408(%rsp,%r9,1),%edi
   3dca5:	cmp    0x8(%rsp,%r9,1),%edi
   3dcaa:	jae    3dcf8 <mibz2::bwt::build+0x1a8>
   3dcac:	cmp    %rdi,%rbx
   3dcaf:	jbe    3e3c6 <mibz2::bwt::build+0x876>
   3dcb5:	movzbl (%r14,%rdi,1),%r9d
   3dcba:	add    %esi,%r9d
   3dcbd:	mov    %r9d,(%rdx,%rdi,4)
   3dcc1:	inc    %edi
   3dcc3:	mov    %edi,0x408(%rsp,%r8,4)
   3dccb:	inc    %rax
   3dcce:	add    $0x100,%esi
   3dcd4:	cmp    %rax,%rbx
   3dcd7:	jne    3dc90 <mibz2::bwt::build+0x140>
   3dcd9:	lea    0x408(%rsp),%rdi
   3dce1:	lea    0x8(%rsp),%rsi
   3dce6:	mov    $0x400,%edx
   3dceb:	call   *0x5fddf(%rip)        # 9dad0 <bcmp@GLIBC_2.2.5>
   3dcf1:	mov    %r12,%rcx
   3dcf4:	test   %eax,%eax
   3dcf6:	je     3dd22 <mibz2::bwt::build+0x1d2>
   3dcf8:	movq   $0x0,0x10(%rcx)
   3dd00:	cmp    (%rcx),%rbx
   3dd03:	ja     3dd2b <mibz2::bwt::build+0x1db>
   3dd05:	mov    0x8(%rcx),%r13
   3dd09:	xor    %r12d,%r12d
   3dd0c:	test   %rbx,%rbx
   3dd0f:	je     3e42f <mibz2::bwt::build+0x8df>
   3dd15:	cmp    $0x8,%rbx
   3dd19:	jae    3dd59 <mibz2::bwt::build+0x209>
   3dd1b:	xor    %eax,%eax
   3dd1d:	jmp    3e41a <mibz2::bwt::build+0x8ca>
   3dd22:	mov    %rbx,0x10(%rcx)
   3dd26:	jmp    3e408 <mibz2::bwt::build+0x8b8>
   3dd2b:	mov    %rcx,%r12
   3dd2e:	mov    $0x4,%ecx
   3dd33:	mov    $0x4,%r8d
   3dd39:	mov    %r12,%rdi
   3dd3c:	xor    %esi,%esi
   3dd3e:	mov    %rbx,%rdx
   3dd41:	call   39ac0 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   3dd46:	mov    %r12,%rcx
   3dd49:	mov    0x8(%r12),%r13
   3dd4e:	mov    0x10(%r12),%r12
   3dd53:	cmp    $0x8,%rbx
   3dd57:	jb     3dd1b <mibz2::bwt::build+0x1cb>
   3dd59:	movabs $0x7ffffffffffffff0,%rax
   3dd63:	or     $0x8,%rax
   3dd67:	and    %rbx,%rax
   3dd6a:	lea    0x10(,%r12,4),%rdx
   3dd72:	add    %r13,%rdx
   3dd75:	add    %rax,%r12
   3dd78:	xor    %esi,%esi
   3dd7a:	pxor   %xmm0,%xmm0
   3dd7e:	xchg   %ax,%ax
   3dd80:	movd   (%r14,%rsi,1),%xmm1
   3dd86:	movd   0x4(%r14,%rsi,1),%xmm2
   3dd8d:	punpcklbw %xmm0,%xmm1
   3dd91:	punpcklwd %xmm0,%xmm1
   3dd95:	punpcklbw %xmm0,%xmm2
   3dd99:	punpcklwd %xmm0,%xmm2
   3dd9d:	movdqu %xmm1,-0x10(%rdx,%rsi,4)
   3dda3:	movdqu %xmm2,(%rdx,%rsi,4)
   3dda8:	add    $0x8,%rsi
   3ddac:	cmp    %rsi,%rax
   3ddaf:	jne    3dd80 <mibz2::bwt::build+0x230>
   3ddb1:	jmp    3e42a <mibz2::bwt::build+0x8da>
   3ddb6:	mov    $0x4,%ecx
   3ddbb:	mov    $0x4,%r8d
   3ddc1:	mov    %r12,%rdi
   3ddc4:	xor    %esi,%esi
   3ddc6:	mov    %rbx,%rdx
   3ddc9:	call   39ac0 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   3ddce:	mov    %r12,%rcx
   3ddd1:	mov    0x8(%rcx),%rdx
   3ddd5:	xor    %r8d,%r8d
   3ddd8:	cmp    $0x10,%rbx
   3dddc:	jb     3dc79 <mibz2::bwt::build+0x129>
   3dde2:	pxor   %xmm0,%xmm0
   3dde6:	movdqa -0x35bce(%rip),%xmm1        # 8220 <GCC_except_table759+0x5c>
   3ddee:	movdqa -0x359f6(%rip),%xmm2        # 8400 <GCC_except_table759+0x23c>
   3ddf6:	movdqa -0x3577e(%rip),%xmm3        # 8680 <GCC_except_table759+0x4bc>
   3ddfe:	xor    %esi,%esi
   3de00:	jmp    3de39 <mibz2::bwt::build+0x2e9>
   3de02:	movzbl (%r14,%rdi,1),%r9d
   3de07:	add    %r8d,%r9d
   3de0a:	add    $0xf00,%r9d
   3de11:	mov    %r9d,(%rdx,%rdi,4)
   3de15:	inc    %edi
   3de17:	mov    %edi,0x408(%rsp,%rax,4)
   3de1e:	lea    0x10(%rsi),%rax
   3de22:	add    $0x20,%rsi
   3de26:	add    $0x1000,%r8d
   3de2d:	cmp    %rbx,%rsi
   3de30:	mov    %rax,%rsi
   3de33:	ja     3dc7b <mibz2::bwt::build+0x12b>
   3de39:	lea    0xf(%rsi),%rax
   3de3d:	cmp    %rbx,%rax
   3de40:	jae    3e3d5 <mibz2::bwt::build+0x885>
   3de46:	mov    (%r14,%rsi,1),%r9
   3de4a:	mov    0x8(%r14,%rsi,1),%rax
   3de4f:	mov    %r9,%rdi
   3de52:	xor    %rax,%rdi
   3de55:	mov    %r9,%r10
   3de58:	rol    $0x8,%r10
   3de5c:	xor    %r9,%r10
   3de5f:	or     %rdi,%r10
   3de62:	jne    3df50 <mibz2::bwt::build+0x400>
   3de68:	movzbl %r9b,%r9d
   3de6c:	lea    0x0(,%r9,4),%r10d
   3de74:	mov    0x408(%rsp,%r10,1),%edi
   3de7c:	lea    0x10(%rdi),%rax
   3de80:	mov    0x8(%rsp,%r10,1),%r10d
   3de85:	cmp    %r10,%rax
   3de88:	ja     3dcf8 <mibz2::bwt::build+0x1a8>
   3de8e:	cmp    %rbx,%rax
   3de91:	ja     3e3df <mibz2::bwt::build+0x88f>
   3de97:	movzbl (%r14,%rdi,1),%eax
   3de9c:	add    %r8d,%eax
   3de9f:	mov    %eax,(%rdx,%rdi,4)
   3dea2:	movd   0x1(%r14,%rdi,1),%xmm4
   3dea9:	punpcklbw %xmm0,%xmm4
   3dead:	punpcklwd %xmm0,%xmm4
   3deb1:	movd   %r8d,%xmm5
   3deb6:	pshufd $0x0,%xmm5,%xmm5
   3debb:	por    %xmm5,%xmm4
   3debf:	por    %xmm1,%xmm4
   3dec3:	movdqu %xmm4,0x4(%rdx,%rdi,4)
   3dec9:	movd   0x5(%r14,%rdi,1),%xmm4
   3ded0:	punpcklbw %xmm0,%xmm4
   3ded4:	punpcklwd %xmm0,%xmm4
   3ded8:	por    %xmm5,%xmm4
   3dedc:	por    %xmm2,%xmm4
   3dee0:	movdqu %xmm4,0x14(%rdx,%rdi,4)
   3dee6:	movd   0x9(%r14,%rdi,1),%xmm4
   3deed:	punpcklbw %xmm0,%xmm4
   3def1:	punpcklwd %xmm0,%xmm4
   3def5:	por    %xmm5,%xmm4
   3def9:	por    %xmm3,%xmm4
   3defd:	movdqu %xmm4,0x24(%rdx,%rdi,4)
   3df03:	movzbl 0xd(%r14,%rdi,1),%eax
   3df09:	lea    0xd00(%r8,%rax,1),%eax
   3df11:	mov    %eax,0x34(%rdx,%rdi,4)
   3df15:	movzbl 0xe(%r14,%rdi,1),%eax
   3df1b:	lea    0xe00(%r8,%rax,1),%eax
   3df23:	mov    %eax,0x38(%rdx,%rdi,4)
   3df27:	movzbl 0xf(%r14,%rdi,1),%eax
   3df2d:	lea    0xf00(%r8,%rax,1),%eax
   3df35:	mov    %eax,0x3c(%rdx,%rdi,4)
   3df39:	lea    0x10(%rdi),%eax
   3df3c:	mov    %eax,0x408(%rsp,%r9,4)
   3df44:	jmp    3de1e <mibz2::bwt::build+0x2ce>
   3df49:	nopl   0x0(%rax)
   3df50:	movzbl %r9b,%r10d
   3df54:	lea    0x0(,%r10,4),%r11d
   3df5c:	mov    0x408(%rsp,%r11,1),%edi
   3df64:	cmp    0x8(%rsp,%r11,1),%edi
   3df69:	jae    3dcf8 <mibz2::bwt::build+0x1a8>
   3df6f:	cmp    %rdi,%rbx
   3df72:	jbe    3e3c6 <mibz2::bwt::build+0x876>
   3df78:	mov    %r9,%r11
   3df7b:	shr    $0x8,%r11
   3df7f:	movzbl (%r14,%rdi,1),%ebp
   3df84:	add    %r8d,%ebp
   3df87:	mov    %ebp,(%rdx,%rdi,4)
   3df8a:	inc    %edi
   3df8c:	mov    %edi,0x408(%rsp,%r10,4)
   3df94:	movzbl %r11b,%r10d
   3df98:	lea    0x0(,%r10,4),%r11d
   3dfa0:	mov    0x408(%rsp,%r11,1),%edi
   3dfa8:	cmp    0x8(%rsp,%r11,1),%edi
   3dfad:	jae    3dcf8 <mibz2::bwt::build+0x1a8>
   3dfb3:	cmp    %rdi,%rbx
   3dfb6:	jbe    3e3c6 <mibz2::bwt::build+0x876>
   3dfbc:	mov    %r9,%r11
   3dfbf:	shr    $0x10,%r11
   3dfc3:	movzbl (%r14,%rdi,1),%r13d
   3dfc8:	lea    (%r8,%r13,1),%ebp
   3dfcc:	add    $0x100,%ebp
   3dfd2:	mov    %ebp,(%rdx,%rdi,4)
   3dfd5:	inc    %edi
   3dfd7:	mov    %edi,0x408(%rsp,%r10,4)
   3dfdf:	movzbl %r11b,%r10d
   3dfe3:	lea    0x0(,%r10,4),%r11d
   3dfeb:	mov    0x408(%rsp,%r11,1),%edi
   3dff3:	cmp    0x8(%rsp,%r11,1),%edi
   3dff8:	jae    3dcf8 <mibz2::bwt::build+0x1a8>
   3dffe:	cmp    %rdi,%rbx
   3e001:	jbe    3e3c6 <mibz2::bwt::build+0x876>
   3e007:	mov    %r9,%r11
   3e00a:	shr    $0x18,%r11
   3e00e:	movzbl (%r14,%rdi,1),%r13d
   3e013:	lea    (%r8,%r13,1),%ebp
   3e017:	add    $0x200,%ebp
   3e01d:	mov    %ebp,(%rdx,%rdi,4)
   3e020:	inc    %edi
   3e022:	mov    %edi,0x408(%rsp,%r10,4)
   3e02a:	movzbl %r11b,%r10d
   3e02e:	lea    0x0(,%r10,4),%r11d
   3e036:	mov    0x408(%rsp,%r11,1),%edi
   3e03e:	cmp    0x8(%rsp,%r11,1),%edi
   3e043:	jae    3dcf8 <mibz2::bwt::build+0x1a8>
   3e049:	cmp    %rdi,%rbx
   3e04c:	jbe    3e3c6 <mibz2::bwt::build+0x876>
   3e052:	mov    %r9,%r11
   3e055:	shr    $0x20,%r11
   3e059:	movzbl (%r14,%rdi,1),%r13d
   3e05e:	lea    (%r8,%r13,1),%ebp
   3e062:	add    $0x300,%ebp
   3e068:	mov    %ebp,(%rdx,%rdi,4)
   3e06b:	inc    %edi
   3e06d:	mov    %edi,0x408(%rsp,%r10,4)
   3e075:	movzbl %r11b,%r10d
   3e079:	lea    0x0(,%r10,4),%r11d
   3e081:	mov    0x408(%rsp,%r11,1),%edi
   3e089:	cmp    0x8(%rsp,%r11,1),%edi
   3e08e:	jae    3dcf8 <mibz2::bwt::build+0x1a8>
   3e094:	cmp    %rdi,%rbx
   3e097:	jbe    3e3c6 <mibz2::bwt::build+0x876>
   3e09d:	mov    %r9,%r11
   3e0a0:	shr    $0x28,%r11
   3e0a4:	movzbl (%r14,%rdi,1),%r13d
   3e0a9:	lea    (%r8,%r13,1),%ebp
   3e0ad:	add    $0x400,%ebp
   3e0b3:	mov    %ebp,(%rdx,%rdi,4)
   3e0b6:	inc    %edi
   3e0b8:	mov    %edi,0x408(%rsp,%r10,4)
   3e0c0:	movzbl %r11b,%r10d
   3e0c4:	lea    0x0(,%r10,4),%r11d
   3e0cc:	mov    0x408(%rsp,%r11,1),%edi
   3e0d4:	cmp    0x8(%rsp,%r11,1),%edi
   3e0d9:	jae    3dcf8 <mibz2::bwt::build+0x1a8>
   3e0df:	cmp    %rdi,%rbx
   3e0e2:	jbe    3e3c6 <mibz2::bwt::build+0x876>
   3e0e8:	mov    %r9,%r11
   3e0eb:	shr    $0x30,%r11
   3e0ef:	movzbl (%r14,%rdi,1),%r13d
   3e0f4:	lea    (%r8,%r13,1),%ebp
   3e0f8:	add    $0x500,%ebp
   3e0fe:	mov    %ebp,(%rdx,%rdi,4)
   3e101:	inc    %edi
   3e103:	mov    %edi,0x408(%rsp,%r10,4)
   3e10b:	movzbl %r11b,%r10d
   3e10f:	lea    0x0(,%r10,4),%r11d
   3e117:	mov    0x408(%rsp,%r11,1),%edi
   3e11f:	cmp    0x8(%rsp,%r11,1),%edi
   3e124:	jae    3dcf8 <mibz2::bwt::build+0x1a8>
   3e12a:	cmp    %rdi,%rbx
   3e12d:	jbe    3e3c6 <mibz2::bwt::build+0x876>
   3e133:	shr    $0x38,%r9
   3e137:	movzbl (%r14,%rdi,1),%r11d
   3e13c:	add    %r8d,%r11d
   3e13f:	add    $0x600,%r11d
   3e146:	mov    %r11d,(%rdx,%rdi,4)
   3e14a:	inc    %edi
   3e14c:	mov    %edi,0x408(%rsp,%r10,4)
   3e154:	lea    0x0(,%r9,4),%r10d
   3e15c:	mov    0x408(%rsp,%r10,1),%edi
   3e164:	cmp    0x8(%rsp,%r10,1),%edi
   3e169:	jae    3dcf8 <mibz2::bwt::build+0x1a8>
   3e16f:	cmp    %rdi,%rbx
   3e172:	jbe    3e3c6 <mibz2::bwt::build+0x876>
   3e178:	movzbl (%r14,%rdi,1),%r10d
   3e17d:	add    %r8d,%r10d
   3e180:	add    $0x700,%r10d
   3e187:	mov    %r10d,(%rdx,%rdi,4)
   3e18b:	inc    %edi
   3e18d:	mov    %edi,0x408(%rsp,%r9,4)
   3e195:	movzbl %al,%r9d
   3e199:	lea    0x0(,%r9,4),%r10d
   3e1a1:	mov    0x408(%rsp,%r10,1),%edi
   3e1a9:	cmp    0x8(%rsp,%r10,1),%edi
   3e1ae:	jae    3dcf8 <mibz2::bwt::build+0x1a8>
   3e1b4:	cmp    %rdi,%rbx
   3e1b7:	jbe    3e3c6 <mibz2::bwt::build+0x876>
   3e1bd:	mov    %rax,%r10
   3e1c0:	shr    $0x8,%r10
   3e1c4:	movzbl (%r14,%rdi,1),%r11d
   3e1c9:	add    %r8d,%r11d
   3e1cc:	add    $0x800,%r11d
   3e1d3:	mov    %r11d,(%rdx,%rdi,4)
   3e1d7:	inc    %edi
   3e1d9:	mov    %edi,0x408(%rsp,%r9,4)
   3e1e1:	movzbl %r10b,%r9d
   3e1e5:	lea    0x0(,%r9,4),%r10d
   3e1ed:	mov    0x408(%rsp,%r10,1),%edi
   3e1f5:	cmp    0x8(%rsp,%r10,1),%edi
   3e1fa:	jae    3dcf8 <mibz2::bwt::build+0x1a8>
   3e200:	cmp    %rdi,%rbx
   3e203:	jbe    3e3c6 <mibz2::bwt::build+0x876>
   3e209:	mov    %rax,%r10
   3e20c:	shr    $0x10,%r10
   3e210:	movzbl (%r14,%rdi,1),%r11d
   3e215:	add    %r8d,%r11d
   3e218:	add    $0x900,%r11d
   3e21f:	mov    %r11d,(%rdx,%rdi,4)
   3e223:	inc    %edi
   3e225:	mov    %edi,0x408(%rsp,%r9,4)
   3e22d:	movzbl %r10b,%r9d
   3e231:	lea    0x0(,%r9,4),%r10d
   3e239:	mov    0x408(%rsp,%r10,1),%edi
   3e241:	cmp    0x8(%rsp,%r10,1),%edi
   3e246:	jae    3dcf8 <mibz2::bwt::build+0x1a8>
   3e24c:	cmp    %rdi,%rbx
   3e24f:	jbe    3e3c6 <mibz2::bwt::build+0x876>
   3e255:	mov    %rax,%r10
   3e258:	shr    $0x18,%r10
   3e25c:	movzbl (%r14,%rdi,1),%r11d
   3e261:	add    %r8d,%r11d
   3e264:	add    $0xa00,%r11d
   3e26b:	mov    %r11d,(%rdx,%rdi,4)
   3e26f:	inc    %edi
   3e271:	mov    %edi,0x408(%rsp,%r9,4)
   3e279:	movzbl %r10b,%r9d
   3e27d:	lea    0x0(,%r9,4),%r10d
   3e285:	mov    0x408(%rsp,%r10,1),%edi
   3e28d:	cmp    0x8(%rsp,%r10,1),%edi
   3e292:	jae    3dcf8 <mibz2::bwt::build+0x1a8>
   3e298:	cmp    %rdi,%rbx
   3e29b:	jbe    3e3c6 <mibz2::bwt::build+0x876>
   3e2a1:	mov    %rax,%r10
   3e2a4:	shr    $0x20,%r10
   3e2a8:	movzbl (%r14,%rdi,1),%r11d
   3e2ad:	add    %r8d,%r11d
   3e2b0:	add    $0xb00,%r11d
   3e2b7:	mov    %r11d,(%rdx,%rdi,4)
   3e2bb:	inc    %edi
   3e2bd:	mov    %edi,0x408(%rsp,%r9,4)
   3e2c5:	movzbl %r10b,%r9d
   3e2c9:	lea    0x0(,%r9,4),%r10d
   3e2d1:	mov    0x408(%rsp,%r10,1),%edi
   3e2d9:	cmp    0x8(%rsp,%r10,1),%edi
   3e2de:	jae    3dcf8 <mibz2::bwt::build+0x1a8>
   3e2e4:	cmp    %rdi,%rbx
   3e2e7:	jbe    3e3c6 <mibz2::bwt::build+0x876>
   3e2ed:	mov    %rax,%r10
   3e2f0:	shr    $0x28,%r10
   3e2f4:	movzbl (%r14,%rdi,1),%r11d
   3e2f9:	add    %r8d,%r11d
   3e2fc:	add    $0xc00,%r11d
   3e303:	mov    %r11d,(%rdx,%rdi,4)
   3e307:	inc    %edi
   3e309:	mov    %edi,0x408(%rsp,%r9,4)
   3e311:	movzbl %r10b,%r9d
   3e315:	lea    0x0(,%r9,4),%r10d
   3e31d:	mov    0x408(%rsp,%r10,1),%edi
   3e325:	cmp    0x8(%rsp,%r10,1),%edi
   3e32a:	jae    3dcf8 <mibz2::bwt::build+0x1a8>
   3e330:	cmp    %rdi,%rbx
   3e333:	jbe    3e3c6 <mibz2::bwt::build+0x876>
   3e339:	mov    %rax,%r10
   3e33c:	shr    $0x30,%r10
   3e340:	movzbl (%r14,%rdi,1),%r11d
   3e345:	add    %r8d,%r11d
   3e348:	add    $0xd00,%r11d
   3e34f:	mov    %r11d,(%rdx,%rdi,4)
   3e353:	inc    %edi
   3e355:	mov    %edi,0x408(%rsp,%r9,4)
   3e35d:	movzbl %r10b,%r9d
   3e361:	lea    0x0(,%r9,4),%r10d
   3e369:	mov    0x408(%rsp,%r10,1),%edi
   3e371:	cmp    0x8(%rsp,%r10,1),%edi
   3e376:	jae    3dcf8 <mibz2::bwt::build+0x1a8>
   3e37c:	cmp    %rdi,%rbx
   3e37f:	jbe    3e3c6 <mibz2::bwt::build+0x876>
   3e381:	shr    $0x38,%rax
   3e385:	movzbl (%r14,%rdi,1),%r10d
   3e38a:	add    %r8d,%r10d
   3e38d:	add    $0xe00,%r10d
   3e394:	mov    %r10d,(%rdx,%rdi,4)
   3e398:	inc    %edi
   3e39a:	mov    %edi,0x408(%rsp,%r9,4)
   3e3a2:	lea    0x0(,%rax,4),%r9d
   3e3aa:	mov    0x408(%rsp,%r9,1),%edi
   3e3b2:	cmp    0x8(%rsp,%r9,1),%edi
   3e3b7:	jae    3dcf8 <mibz2::bwt::build+0x1a8>
   3e3bd:	cmp    %rdi,%rbx
   3e3c0:	ja     3de02 <mibz2::bwt::build+0x2b2>
   3e3c6:	lea    0x5d6f3(%rip),%rdx        # 9bac0 <__frame_dummy_init_array_entry+0xeb8>
   3e3cd:	mov    %rbx,%rsi
   3e3d0:	call   2ec71 <core::panicking::panic_bounds_check>
   3e3d5:	mov    %ebx,%edi
   3e3d7:	and    $0xfffff0,%edi
   3e3dd:	jmp    3e3f1 <mibz2::bwt::build+0x8a1>
   3e3df:	lea    0x5da82(%rip),%rcx        # 9be68 <__frame_dummy_init_array_entry+0x1260>
   3e3e6:	mov    %rax,%rsi
   3e3e9:	mov    %rbx,%rdx
   3e3ec:	call   2eb90 <core::slice::index::slice_index_fail>
   3e3f1:	or     $0x8,%rdi
   3e3f5:	add    $0x10,%rsi
   3e3f9:	lea    0x5dac8(%rip),%rcx        # 9bec8 <__frame_dummy_init_array_entry+0x12c0>
   3e400:	mov    %rbx,%rdx
   3e403:	call   2eb90 <core::slice::index::slice_index_fail>
   3e408:	add    $0x808,%rsp
   3e40f:	pop    %rbx
   3e410:	pop    %r12
   3e412:	pop    %r13
   3e414:	pop    %r14
   3e416:	pop    %r15
   3e418:	pop    %rbp
   3e419:	ret
   3e41a:	movzbl (%r14,%rax,1),%edx
   3e41f:	mov    %edx,0x0(%r13,%r12,4)
   3e424:	inc    %r12
   3e427:	inc    %rax
   3e42a:	cmp    %rax,%rbx
   3e42d:	jne    3e41a <mibz2::bwt::build+0x8ca>
   3e42f:	mov    %r12,0x10(%rcx)
   3e433:	lea    0x8(%rsp),%rdi
   3e438:	xor    %ebp,%ebp
   3e43a:	mov    $0x400,%edx
   3e43f:	xor    %esi,%esi
   3e441:	call   *0x5f681(%rip)        # 9dac8 <memset@GLIBC_2.2.5>
   3e447:	xor    %eax,%eax
   3e449:	nopl   0x0(%rax)
   3e450:	mov    %eax,0x8(%rsp,%rbp,4)
   3e454:	add    (%r15,%rbp,4),%eax
   3e458:	mov    %eax,0xc(%rsp,%rbp,4)
   3e45c:	add    0x4(%r15,%rbp,4),%eax
   3e461:	mov    %eax,0x10(%rsp,%rbp,4)
   3e465:	add    0x8(%r15,%rbp,4),%eax
   3e46a:	mov    %eax,0x14(%rsp,%rbp,4)
   3e46e:	add    0xc(%r15,%rbp,4),%eax
   3e473:	add    $0x4,%rbp
   3e477:	cmp    $0x100,%rbp
   3e47e:	jne    3e450 <mibz2::bwt::build+0x900>
   3e480:	xor    %ecx,%ecx
   3e482:	cmp    $0x10,%rbx
   3e486:	jae    3e4d9 <mibz2::bwt::build+0x989>
   3e488:	xor    %eax,%eax
   3e48a:	cmp    %rbx,%rax
   3e48d:	jae    3e408 <mibz2::bwt::build+0x8b8>
   3e493:	mov    %eax,%ecx
   3e495:	shl    $0x8,%ecx
   3e498:	nopl   0x0(%rax,%rax,1)
   3e4a0:	movzbl (%r14,%rax,1),%edx
   3e4a5:	mov    0x8(%rsp,%rdx,4),%edi
   3e4a9:	cmp    %rdi,%r12
   3e4ac:	jbe    3e933 <mibz2::bwt::build+0xde3>
   3e4b2:	inc    %rax
   3e4b5:	movzbl 0x0(%r13,%rdi,4),%esi
   3e4bb:	add    %ecx,%esi
   3e4bd:	mov    %esi,0x0(%r13,%rdi,4)
   3e4c2:	lea    0x1(%rdi),%esi
   3e4c5:	mov    %esi,0x8(%rsp,%rdx,4)
   3e4c9:	add    $0x100,%ecx
   3e4cf:	cmp    %rax,%rbx
   3e4d2:	jne    3e4a0 <mibz2::bwt::build+0x950>
   3e4d4:	jmp    3e408 <mibz2::bwt::build+0x8b8>
   3e4d9:	movdqa -0x35f01(%rip),%xmm0        # 85e0 <GCC_except_table759+0x41c>
   3e4e1:	movdqa -0x35f89(%rip),%xmm1        # 8560 <GCC_except_table759+0x39c>
   3e4e9:	movdqa -0x36291(%rip),%xmm2        # 8260 <GCC_except_table759+0x9c>
   3e4f1:	movdqa -0x362b9(%rip),%xmm3        # 8240 <GCC_except_table759+0x7c>
   3e4f9:	movdqa -0x36021(%rip),%xmm4        # 84e0 <GCC_except_table759+0x31c>
   3e501:	xor    %esi,%esi
   3e503:	jmp    3e544 <mibz2::bwt::build+0x9f4>
   3e505:	data16 cs nopw 0x0(%rax,%rax,1)
   3e510:	movzbl 0x0(%r13,%rdi,4),%edx
   3e516:	add    %ecx,%edx
   3e518:	add    $0xf00,%edx
   3e51e:	mov    %edx,0x0(%r13,%rdi,4)
   3e523:	lea    0x1(%rdi),%edx
   3e526:	mov    %edx,0x8(%rsp,%rax,4)
   3e52a:	lea    0x10(%rsi),%rax
   3e52e:	add    $0x20,%rsi
   3e532:	add    $0x1000,%ecx
   3e538:	cmp    %rbx,%rsi
   3e53b:	mov    %rax,%rsi
   3e53e:	ja     3e48a <mibz2::bwt::build+0x93a>
   3e544:	lea    0xf(%rsi),%rax
   3e548:	cmp    %rbx,%rax
   3e54b:	jae    3e942 <mibz2::bwt::build+0xdf2>
   3e551:	mov    (%r14,%rsi,1),%rdx
   3e555:	mov    0x8(%r14,%rsi,1),%rax
   3e55a:	mov    %rdx,%rdi
   3e55d:	xor    %rax,%rdi
   3e560:	mov    %rdx,%r8
   3e563:	rol    $0x8,%r8
   3e567:	xor    %rdx,%r8
   3e56a:	or     %rdi,%r8
   3e56d:	jne    3e620 <mibz2::bwt::build+0xad0>
   3e573:	movzbl %dl,%edx
   3e576:	mov    %edx,%eax
   3e578:	mov    0x8(%rsp,%rax,4),%edi
   3e57c:	lea    0x10(%rdi),%rax
   3e580:	cmp    %r12,%rax
   3e583:	ja     3e954 <mibz2::bwt::build+0xe04>
   3e589:	movdqu 0x0(%r13,%rdi,4),%xmm6
   3e590:	movdqu 0x10(%r13,%rdi,4),%xmm7
   3e597:	movdqu 0x20(%r13,%rdi,4),%xmm8
   3e59e:	movdqu 0x30(%r13,%rdi,4),%xmm5
   3e5a5:	pand   %xmm0,%xmm6
   3e5a9:	movd   %ecx,%xmm9
   3e5ae:	movdqa %xmm1,%xmm10
   3e5b3:	movss  %xmm9,%xmm10
   3e5b8:	movd   %ecx,%xmm9
   3e5bd:	pshufd $0x1,%xmm9,%xmm11
   3e5c3:	por    %xmm10,%xmm11
   3e5c8:	por    %xmm6,%xmm11
   3e5cd:	movdqu %xmm11,0x0(%r13,%rdi,4)
   3e5d4:	pand   %xmm0,%xmm7
   3e5d8:	pshufd $0x0,%xmm9,%xmm6
   3e5de:	por    %xmm6,%xmm7
   3e5e2:	por    %xmm2,%xmm7
   3e5e6:	movdqu %xmm7,0x10(%r13,%rdi,4)
   3e5ed:	pand   %xmm0,%xmm8
   3e5f2:	por    %xmm6,%xmm8
   3e5f7:	por    %xmm3,%xmm8
   3e5fc:	movdqu %xmm8,0x20(%r13,%rdi,4)
   3e603:	pand   %xmm0,%xmm5
   3e607:	por    %xmm6,%xmm5
   3e60b:	por    %xmm4,%xmm5
   3e60f:	movdqu %xmm5,0x30(%r13,%rdi,4)
   3e616:	addl   $0x10,0x8(%rsp,%rdx,4)
   3e61b:	jmp    3e52a <mibz2::bwt::build+0x9da>
   3e620:	movzbl %dl,%r8d
   3e624:	mov    %r8d,%edi
   3e627:	mov    0x8(%rsp,%rdi,4),%edi
   3e62b:	cmp    %rdi,%r12
   3e62e:	jbe    3e933 <mibz2::bwt::build+0xde3>
   3e634:	mov    %rdx,%r9
   3e637:	shr    $0x8,%r9
   3e63b:	movzbl 0x0(%r13,%rdi,4),%r10d
   3e641:	add    %ecx,%r10d
   3e644:	mov    %r10d,0x0(%r13,%rdi,4)
   3e649:	inc    %edi
   3e64b:	mov    %edi,0x8(%rsp,%r8,4)
   3e650:	movzbl %r9b,%r8d
   3e654:	mov    %r8d,%edi
   3e657:	mov    0x8(%rsp,%rdi,4),%edi
   3e65b:	cmp    %rdi,%r12
   3e65e:	jbe    3e933 <mibz2::bwt::build+0xde3>
   3e664:	mov    %rdx,%r9
   3e667:	shr    $0x10,%r9
   3e66b:	movzbl 0x0(%r13,%rdi,4),%r10d
   3e671:	add    %ecx,%r10d
   3e674:	add    $0x100,%r10d
   3e67b:	mov    %r10d,0x0(%r13,%rdi,4)
   3e680:	inc    %edi
   3e682:	mov    %edi,0x8(%rsp,%r8,4)
   3e687:	movzbl %r9b,%r8d
   3e68b:	mov    %r8d,%edi
   3e68e:	mov    0x8(%rsp,%rdi,4),%edi
   3e692:	cmp    %rdi,%r12
   3e695:	jbe    3e933 <mibz2::bwt::build+0xde3>
   3e69b:	mov    %rdx,%r9
   3e69e:	shr    $0x18,%r9
   3e6a2:	movzbl 0x0(%r13,%rdi,4),%r10d
   3e6a8:	add    %ecx,%r10d
   3e6ab:	add    $0x200,%r10d
   3e6b2:	mov    %r10d,0x0(%r13,%rdi,4)
   3e6b7:	inc    %edi
   3e6b9:	mov    %edi,0x8(%rsp,%r8,4)
   3e6be:	movzbl %r9b,%r8d
   3e6c2:	mov    %r8d,%edi
   3e6c5:	mov    0x8(%rsp,%rdi,4),%edi
   3e6c9:	cmp    %rdi,%r12
   3e6cc:	jbe    3e933 <mibz2::bwt::build+0xde3>
   3e6d2:	mov    %rdx,%r9
   3e6d5:	shr    $0x20,%r9
   3e6d9:	movzbl 0x0(%r13,%rdi,4),%r10d
   3e6df:	add    %ecx,%r10d
   3e6e2:	add    $0x300,%r10d
   3e6e9:	mov    %r10d,0x0(%r13,%rdi,4)
   3e6ee:	inc    %edi
   3e6f0:	mov    %edi,0x8(%rsp,%r8,4)
   3e6f5:	movzbl %r9b,%r8d
   3e6f9:	mov    %r8d,%edi
   3e6fc:	mov    0x8(%rsp,%rdi,4),%edi
   3e700:	cmp    %rdi,%r12
   3e703:	jbe    3e933 <mibz2::bwt::build+0xde3>
   3e709:	mov    %rdx,%r9
   3e70c:	shr    $0x28,%r9
   3e710:	movzbl 0x0(%r13,%rdi,4),%r10d
   3e716:	add    %ecx,%r10d
   3e719:	add    $0x400,%r10d
   3e720:	mov    %r10d,0x0(%r13,%rdi,4)
   3e725:	inc    %edi
   3e727:	mov    %edi,0x8(%rsp,%r8,4)
   3e72c:	movzbl %r9b,%r8d
   3e730:	mov    %r8d,%edi
   3e733:	mov    0x8(%rsp,%rdi,4),%edi
   3e737:	cmp    %rdi,%r12
   3e73a:	jbe    3e933 <mibz2::bwt::build+0xde3>
   3e740:	mov    %rdx,%r9
   3e743:	shr    $0x30,%r9
   3e747:	movzbl 0x0(%r13,%rdi,4),%r10d
   3e74d:	add    %ecx,%r10d
   3e750:	add    $0x500,%r10d
   3e757:	mov    %r10d,0x0(%r13,%rdi,4)
   3e75c:	inc    %edi
   3e75e:	mov    %edi,0x8(%rsp,%r8,4)
   3e763:	movzbl %r9b,%r8d
   3e767:	mov    %r8d,%edi
   3e76a:	mov    0x8(%rsp,%rdi,4),%edi
   3e76e:	cmp    %rdi,%r12
   3e771:	jbe    3e933 <mibz2::bwt::build+0xde3>
   3e777:	shr    $0x38,%rdx
   3e77b:	movzbl 0x0(%r13,%rdi,4),%r9d
   3e781:	add    %ecx,%r9d
   3e784:	add    $0x600,%r9d
   3e78b:	mov    %r9d,0x0(%r13,%rdi,4)
   3e790:	inc    %edi
   3e792:	mov    %edi,0x8(%rsp,%r8,4)
   3e797:	mov    0x8(%rsp,%rdx,4),%edi
   3e79b:	cmp    %rdi,%r12
   3e79e:	jbe    3e933 <mibz2::bwt::build+0xde3>
   3e7a4:	movzbl 0x0(%r13,%rdi,4),%r8d
   3e7aa:	add    %ecx,%r8d
   3e7ad:	add    $0x700,%r8d
   3e7b4:	mov    %r8d,0x0(%r13,%rdi,4)
   3e7b9:	inc    %edi
   3e7bb:	mov    %edi,0x8(%rsp,%rdx,4)
   3e7bf:	movzbl %al,%edx
   3e7c2:	mov    %edx,%edi
   3e7c4:	mov    0x8(%rsp,%rdi,4),%edi
   3e7c8:	cmp    %rdi,%r12
   3e7cb:	jbe    3e933 <mibz2::bwt::build+0xde3>
   3e7d1:	mov    %rax,%r8
   3e7d4:	shr    $0x8,%r8
   3e7d8:	movzbl 0x0(%r13,%rdi,4),%r9d
   3e7de:	add    %ecx,%r9d
   3e7e1:	add    $0x800,%r9d
   3e7e8:	mov    %r9d,0x0(%r13,%rdi,4)
   3e7ed:	inc    %edi
   3e7ef:	mov    %edi,0x8(%rsp,%rdx,4)
   3e7f3:	movzbl %r8b,%edx
   3e7f7:	mov    %edx,%edi
   3e7f9:	mov    0x8(%rsp,%rdi,4),%edi
   3e7fd:	cmp    %rdi,%r12
   3e800:	jbe    3e933 <mibz2::bwt::build+0xde3>
   3e806:	mov    %rax,%r8
   3e809:	shr    $0x10,%r8
   3e80d:	movzbl 0x0(%r13,%rdi,4),%r9d
   3e813:	add    %ecx,%r9d
   3e816:	add    $0x900,%r9d
   3e81d:	mov    %r9d,0x0(%r13,%rdi,4)
   3e822:	inc    %edi
   3e824:	mov    %edi,0x8(%rsp,%rdx,4)
   3e828:	movzbl %r8b,%edx
   3e82c:	mov    %edx,%edi
   3e82e:	mov    0x8(%rsp,%rdi,4),%edi
   3e832:	cmp    %rdi,%r12
   3e835:	jbe    3e933 <mibz2::bwt::build+0xde3>
   3e83b:	mov    %rax,%r8
   3e83e:	shr    $0x18,%r8
   3e842:	movzbl 0x0(%r13,%rdi,4),%r9d
   3e848:	add    %ecx,%r9d
   3e84b:	add    $0xa00,%r9d
   3e852:	mov    %r9d,0x0(%r13,%rdi,4)
   3e857:	inc    %edi
   3e859:	mov    %edi,0x8(%rsp,%rdx,4)
   3e85d:	movzbl %r8b,%edx
   3e861:	mov    %edx,%edi
   3e863:	mov    0x8(%rsp,%rdi,4),%edi
   3e867:	cmp    %rdi,%r12
   3e86a:	jbe    3e933 <mibz2::bwt::build+0xde3>
   3e870:	mov    %rax,%r8
   3e873:	shr    $0x20,%r8
   3e877:	movzbl 0x0(%r13,%rdi,4),%r9d
   3e87d:	add    %ecx,%r9d
   3e880:	add    $0xb00,%r9d
   3e887:	mov    %r9d,0x0(%r13,%rdi,4)
   3e88c:	inc    %edi
   3e88e:	mov    %edi,0x8(%rsp,%rdx,4)
   3e892:	movzbl %r8b,%edx
   3e896:	mov    %edx,%edi
   3e898:	mov    0x8(%rsp,%rdi,4),%edi
   3e89c:	cmp    %rdi,%r12
   3e89f:	jbe    3e933 <mibz2::bwt::build+0xde3>
   3e8a5:	mov    %rax,%r8
   3e8a8:	shr    $0x28,%r8
   3e8ac:	movzbl 0x0(%r13,%rdi,4),%r9d
   3e8b2:	add    %ecx,%r9d
   3e8b5:	add    $0xc00,%r9d
   3e8bc:	mov    %r9d,0x0(%r13,%rdi,4)
   3e8c1:	inc    %edi
   3e8c3:	mov    %edi,0x8(%rsp,%rdx,4)
   3e8c7:	movzbl %r8b,%edx
   3e8cb:	mov    %edx,%edi
   3e8cd:	mov    0x8(%rsp,%rdi,4),%edi
   3e8d1:	cmp    %rdi,%r12
   3e8d4:	jbe    3e933 <mibz2::bwt::build+0xde3>
   3e8d6:	mov    %rax,%r8
   3e8d9:	shr    $0x30,%r8
   3e8dd:	movzbl 0x0(%r13,%rdi,4),%r9d
   3e8e3:	add    %ecx,%r9d
   3e8e6:	add    $0xd00,%r9d
   3e8ed:	mov    %r9d,0x0(%r13,%rdi,4)
   3e8f2:	inc    %edi
   3e8f4:	mov    %edi,0x8(%rsp,%rdx,4)
   3e8f8:	movzbl %r8b,%edx
   3e8fc:	mov    %edx,%edi
   3e8fe:	mov    0x8(%rsp,%rdi,4),%edi
   3e902:	cmp    %rdi,%r12
   3e905:	jbe    3e933 <mibz2::bwt::build+0xde3>
   3e907:	shr    $0x38,%rax
   3e90b:	movzbl 0x0(%r13,%rdi,4),%r8d
   3e911:	add    %ecx,%r8d
   3e914:	add    $0xe00,%r8d
   3e91b:	mov    %r8d,0x0(%r13,%rdi,4)
   3e920:	inc    %edi
   3e922:	mov    %edi,0x8(%rsp,%rdx,4)
   3e926:	mov    0x8(%rsp,%rax,4),%edi
   3e92a:	cmp    %rdi,%r12
   3e92d:	ja     3e510 <mibz2::bwt::build+0x9c0>
   3e933:	lea    0x5d55e(%rip),%rdx        # 9be98 <__frame_dummy_init_array_entry+0x1290>
   3e93a:	mov    %r12,%rsi
   3e93d:	call   2ec71 <core::panicking::panic_bounds_check>
   3e942:	movabs $0x7ffffffffffffff0,%rdi
   3e94c:	and    %rbx,%rdi
   3e94f:	jmp    3e3f1 <mibz2::bwt::build+0x8a1>
   3e954:	lea    0x5d525(%rip),%rcx        # 9be80 <__frame_dummy_init_array_entry+0x1278>
   3e95b:	mov    %rax,%rsi
   3e95e:	mov    %r12,%rdx
   3e961:	call   2eb90 <core::slice::index::slice_index_fail>
Disassembly of section .init:
Disassembly of section .fini:
Disassembly of section .plt:
