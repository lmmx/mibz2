# mibz2::bwt::build
# size 0xe97

/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/c30walk-e003a32:     file format elf64-x86-64


Disassembly of section .text:

000000000006dd50 <mibz2::bwt::build>:
   6dd50:	push   %rbp
   6dd51:	push   %r15
   6dd53:	push   %r14
   6dd55:	push   %r13
   6dd57:	push   %r12
   6dd59:	push   %rbx
   6dd5a:	sub    $0x808,%rsp
   6dd61:	mov    %rdx,%r15
   6dd64:	mov    %rsi,%rbx
   6dd67:	mov    %rdi,%r14
   6dd6a:	movq   $0x0,0x10(%rcx)
   6dd72:	cmp    $0xffffff,%rsi
   6dd79:	ja     6def6 <mibz2::bwt::build+0x1a6>
   6dd7f:	pxor   %xmm0,%xmm0
   6dd83:	xor    %eax,%eax
   6dd85:	pxor   %xmm2,%xmm2
   6dd89:	pxor   %xmm1,%xmm1
   6dd8d:	nopl   (%rax)
   6dd90:	movq   (%r15,%rax,4),%xmm3
   6dd96:	movq   0x8(%r15,%rax,4),%xmm4
   6dd9d:	punpckldq %xmm0,%xmm3
   6dda1:	paddq  %xmm2,%xmm3
   6dda5:	punpckldq %xmm0,%xmm4
   6dda9:	paddq  %xmm1,%xmm4
   6ddad:	movq   0x10(%r15,%rax,4),%xmm2
   6ddb4:	movq   0x18(%r15,%rax,4),%xmm1
   6ddbb:	punpckldq %xmm0,%xmm2
   6ddbf:	paddq  %xmm3,%xmm2
   6ddc3:	punpckldq %xmm0,%xmm1
   6ddc7:	paddq  %xmm4,%xmm1
   6ddcb:	add    $0x8,%rax
   6ddcf:	cmp    $0x100,%rax
   6ddd5:	jne    6dd90 <mibz2::bwt::build+0x40>
   6ddd7:	paddq  %xmm2,%xmm1
   6dddb:	pshufd $0xee,%xmm1,%xmm0
   6dde0:	paddq  %xmm1,%xmm0
   6dde4:	movq   %xmm0,%rax
   6dde9:	cmp    %rbx,%rax
   6ddec:	jne    6def6 <mibz2::bwt::build+0x1a6>
   6ddf2:	mov    %rcx,%r12
   6ddf5:	lea    0x8(%rsp),%rdi
   6ddfa:	xor    %ebp,%ebp
   6ddfc:	mov    0x4bb55(%rip),%r13        # b9958 <memset@GLIBC_2.2.5>
   6de03:	mov    $0x400,%edx
   6de08:	xor    %esi,%esi
   6de0a:	call   *%r13
   6de0d:	lea    0x408(%rsp),%rdi
   6de15:	mov    $0x400,%edx
   6de1a:	xor    %esi,%esi
   6de1c:	call   *%r13
   6de1f:	mov    $0x1,%eax
   6de24:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   6de30:	mov    %ebp,0x404(%rsp,%rax,4)
   6de37:	add    -0x4(%r15,%rax,4),%ebp
   6de3c:	mov    %ebp,0x4(%rsp,%rax,4)
   6de40:	mov    %ebp,0x408(%rsp,%rax,4)
   6de47:	add    (%r15,%rax,4),%ebp
   6de4b:	mov    %ebp,0x8(%rsp,%rax,4)
   6de4f:	add    $0x2,%rax
   6de53:	cmp    $0x101,%rax
   6de59:	jne    6de30 <mibz2::bwt::build+0xe0>
   6de5b:	mov    %r12,%rcx
   6de5e:	cmp    (%r12),%rbx
   6de62:	ja     6dfb6 <mibz2::bwt::build+0x266>
   6de68:	mov    0x8(%rcx),%rdx
   6de6c:	xor    %r8d,%r8d
   6de6f:	cmp    $0x10,%rbx
   6de73:	jae    6dfe2 <mibz2::bwt::build+0x292>
   6de79:	cmp    %rbx,%r8
   6de7c:	jae    6ded7 <mibz2::bwt::build+0x187>
   6de7e:	mov    %r8d,%eax
   6de81:	shl    $0x8,%eax
   6de84:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   6de90:	movzbl (%r14,%r8,1),%esi
   6de95:	lea    0x0(,%rsi,4),%r9d
   6de9d:	mov    0x408(%rsp,%r9,1),%edi
   6dea5:	cmp    0x8(%rsp,%r9,1),%edi
   6deaa:	jae    6def6 <mibz2::bwt::build+0x1a6>
   6deac:	cmp    %rdi,%rbx
   6deaf:	jbe    6e618 <mibz2::bwt::build+0x8c8>
   6deb5:	movzbl (%r14,%rdi,1),%r9d
   6deba:	add    %eax,%r9d
   6debd:	mov    %r9d,(%rdx,%rdi,4)
   6dec1:	inc    %edi
   6dec3:	mov    %edi,0x408(%rsp,%rsi,4)
   6deca:	inc    %r8
   6decd:	add    $0x100,%eax
   6ded2:	cmp    %r8,%rbx
   6ded5:	jne    6de90 <mibz2::bwt::build+0x140>
   6ded7:	lea    0x408(%rsp),%rdi
   6dedf:	lea    0x8(%rsp),%rsi
   6dee4:	mov    $0x400,%edx
   6dee9:	call   *0x4ba91(%rip)        # b9980 <bcmp@GLIBC_2.2.5>
   6deef:	mov    %r12,%rcx
   6def2:	test   %eax,%eax
   6def4:	je     6df20 <mibz2::bwt::build+0x1d0>
   6def6:	movq   $0x0,0x10(%rcx)
   6defe:	cmp    (%rcx),%rbx
   6df01:	ja     6df29 <mibz2::bwt::build+0x1d9>
   6df03:	mov    0x8(%rcx),%r13
   6df07:	xor    %r12d,%r12d
   6df0a:	test   %rbx,%rbx
   6df0d:	je     6e67a <mibz2::bwt::build+0x92a>
   6df13:	cmp    $0x8,%rbx
   6df17:	jae    6df57 <mibz2::bwt::build+0x207>
   6df19:	xor    %eax,%eax
   6df1b:	jmp    6e665 <mibz2::bwt::build+0x915>
   6df20:	mov    %rbx,0x10(%rcx)
   6df24:	jmp    6e653 <mibz2::bwt::build+0x903>
   6df29:	mov    %rcx,%r12
   6df2c:	mov    $0x4,%ecx
   6df31:	mov    $0x4,%r8d
   6df37:	mov    %r12,%rdi
   6df3a:	xor    %esi,%esi
   6df3c:	mov    %rbx,%rdx
   6df3f:	call   6ce90 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   6df44:	mov    %r12,%rcx
   6df47:	mov    0x8(%r12),%r13
   6df4c:	mov    0x10(%r12),%r12
   6df51:	cmp    $0x8,%rbx
   6df55:	jb     6df19 <mibz2::bwt::build+0x1c9>
   6df57:	movabs $0x7ffffffffffffff0,%rax
   6df61:	or     $0x8,%rax
   6df65:	and    %rbx,%rax
   6df68:	lea    0x10(,%r12,4),%rdx
   6df70:	add    %r13,%rdx
   6df73:	add    %rax,%r12
   6df76:	xor    %esi,%esi
   6df78:	pxor   %xmm0,%xmm0
   6df7c:	nopl   0x0(%rax)
   6df80:	movd   (%r14,%rsi,1),%xmm1
   6df86:	movd   0x4(%r14,%rsi,1),%xmm2
   6df8d:	punpcklbw %xmm0,%xmm1
   6df91:	punpcklwd %xmm0,%xmm1
   6df95:	punpcklbw %xmm0,%xmm2
   6df99:	punpcklwd %xmm0,%xmm2
   6df9d:	movdqu %xmm1,-0x10(%rdx,%rsi,4)
   6dfa3:	movdqu %xmm2,(%rdx,%rsi,4)
   6dfa8:	add    $0x8,%rsi
   6dfac:	cmp    %rsi,%rax
   6dfaf:	jne    6df80 <mibz2::bwt::build+0x230>
   6dfb1:	jmp    6e675 <mibz2::bwt::build+0x925>
   6dfb6:	mov    $0x4,%ecx
   6dfbb:	mov    $0x4,%r8d
   6dfc1:	mov    %r12,%rdi
   6dfc4:	xor    %esi,%esi
   6dfc6:	mov    %rbx,%rdx
   6dfc9:	call   6ce90 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   6dfce:	mov    %r12,%rcx
   6dfd1:	mov    0x8(%rcx),%rdx
   6dfd5:	xor    %r8d,%r8d
   6dfd8:	cmp    $0x10,%rbx
   6dfdc:	jb     6de79 <mibz2::bwt::build+0x129>
   6dfe2:	mov    $0x8,%esi
   6dfe7:	pxor   %xmm0,%xmm0
   6dfeb:	movdqa -0x64f33(%rip),%xmm1        # 90c0 <GCC_except_table733+0x74>
   6dff3:	movdqa -0x64a8b(%rip),%xmm2        # 9570 <GCC_except_table733+0x524>
   6dffb:	movdqa -0x646a3(%rip),%xmm3        # 9960 <GCC_except_table733+0x914>
   6e003:	jmp    6e02b <mibz2::bwt::build+0x2db>
   6e005:	data16 cs nopw 0x0(%rax,%rax,1)
   6e010:	lea    0x10(%rsi),%rax
   6e014:	add    $0x18,%rsi
   6e018:	add    $0x1000,%r8d
   6e01f:	cmp    %rbx,%rsi
   6e022:	mov    %rax,%rsi
   6e025:	ja     6e5e7 <mibz2::bwt::build+0x897>
   6e02b:	lea    -0x8(%rsi),%rax
   6e02f:	cmp    %rbx,%rsi
   6e032:	ja     6e627 <mibz2::bwt::build+0x8d7>
   6e038:	lea    0x7(%rsi),%rdi
   6e03c:	cmp    %rbx,%rdi
   6e03f:	jae    6e5fc <mibz2::bwt::build+0x8ac>
   6e045:	mov    -0x8(%r14,%rsi,1),%r9
   6e04a:	mov    (%r14,%rsi,1),%rax
   6e04e:	mov    %r9,%rdi
   6e051:	xor    %rax,%rdi
   6e054:	mov    %r9,%r10
   6e057:	rol    $0x8,%r10
   6e05b:	xor    %r9,%r10
   6e05e:	or     %rdi,%r10
   6e061:	jne    6e150 <mibz2::bwt::build+0x400>
   6e067:	movzbl %r9b,%r9d
   6e06b:	lea    0x0(,%r9,4),%r10d
   6e073:	mov    0x408(%rsp,%r10,1),%edi
   6e07b:	lea    0x10(%rdi),%rax
   6e07f:	mov    0x8(%rsp,%r10,1),%r10d
   6e084:	cmp    %r10,%rax
   6e087:	ja     6def6 <mibz2::bwt::build+0x1a6>
   6e08d:	cmp    %rbx,%rax
   6e090:	ja     6e606 <mibz2::bwt::build+0x8b6>
   6e096:	movzbl (%r14,%rdi,1),%eax
   6e09b:	add    %r8d,%eax
   6e09e:	mov    %eax,(%rdx,%rdi,4)
   6e0a1:	movd   0x1(%r14,%rdi,1),%xmm4
   6e0a8:	punpcklbw %xmm0,%xmm4
   6e0ac:	punpcklwd %xmm0,%xmm4
   6e0b0:	movd   %r8d,%xmm5
   6e0b5:	pshufd $0x0,%xmm5,%xmm5
   6e0ba:	por    %xmm5,%xmm4
   6e0be:	por    %xmm1,%xmm4
   6e0c2:	movdqu %xmm4,0x4(%rdx,%rdi,4)
   6e0c8:	movd   0x5(%r14,%rdi,1),%xmm4
   6e0cf:	punpcklbw %xmm0,%xmm4
   6e0d3:	punpcklwd %xmm0,%xmm4
   6e0d7:	por    %xmm5,%xmm4
   6e0db:	por    %xmm2,%xmm4
   6e0df:	movdqu %xmm4,0x14(%rdx,%rdi,4)
   6e0e5:	movd   0x9(%r14,%rdi,1),%xmm4
   6e0ec:	punpcklbw %xmm0,%xmm4
   6e0f0:	punpcklwd %xmm0,%xmm4
   6e0f4:	por    %xmm5,%xmm4
   6e0f8:	por    %xmm3,%xmm4
   6e0fc:	movdqu %xmm4,0x24(%rdx,%rdi,4)
   6e102:	movzbl 0xd(%r14,%rdi,1),%eax
   6e108:	lea    0xd00(%r8,%rax,1),%eax
   6e110:	mov    %eax,0x34(%rdx,%rdi,4)
   6e114:	movzbl 0xe(%r14,%rdi,1),%eax
   6e11a:	lea    0xe00(%r8,%rax,1),%eax
   6e122:	mov    %eax,0x38(%rdx,%rdi,4)
   6e126:	movzbl 0xf(%r14,%rdi,1),%eax
   6e12c:	lea    0xf00(%r8,%rax,1),%eax
   6e134:	mov    %eax,0x3c(%rdx,%rdi,4)
   6e138:	lea    0x10(%rdi),%eax
   6e13b:	mov    %eax,0x408(%rsp,%r9,4)
   6e143:	jmp    6e010 <mibz2::bwt::build+0x2c0>
   6e148:	nopl   0x0(%rax,%rax,1)
   6e150:	movzbl %r9b,%r10d
   6e154:	lea    0x0(,%r10,4),%r11d
   6e15c:	mov    0x408(%rsp,%r11,1),%edi
   6e164:	cmp    0x8(%rsp,%r11,1),%edi
   6e169:	jae    6def6 <mibz2::bwt::build+0x1a6>
   6e16f:	cmp    %rdi,%rbx
   6e172:	jbe    6e618 <mibz2::bwt::build+0x8c8>
   6e178:	mov    %r9,%r11
   6e17b:	shr    $0x8,%r11
   6e17f:	movzbl (%r14,%rdi,1),%ebp
   6e184:	add    %r8d,%ebp
   6e187:	mov    %ebp,(%rdx,%rdi,4)
   6e18a:	inc    %edi
   6e18c:	mov    %edi,0x408(%rsp,%r10,4)
   6e194:	movzbl %r11b,%r10d
   6e198:	lea    0x0(,%r10,4),%r11d
   6e1a0:	mov    0x408(%rsp,%r11,1),%edi
   6e1a8:	cmp    0x8(%rsp,%r11,1),%edi
   6e1ad:	jae    6def6 <mibz2::bwt::build+0x1a6>
   6e1b3:	cmp    %rdi,%rbx
   6e1b6:	jbe    6e618 <mibz2::bwt::build+0x8c8>
   6e1bc:	mov    %r9,%r11
   6e1bf:	shr    $0x10,%r11
   6e1c3:	movzbl (%r14,%rdi,1),%r13d
   6e1c8:	lea    (%r8,%r13,1),%ebp
   6e1cc:	add    $0x100,%ebp
   6e1d2:	mov    %ebp,(%rdx,%rdi,4)
   6e1d5:	inc    %edi
   6e1d7:	mov    %edi,0x408(%rsp,%r10,4)
   6e1df:	movzbl %r11b,%r10d
   6e1e3:	lea    0x0(,%r10,4),%r11d
   6e1eb:	mov    0x408(%rsp,%r11,1),%edi
   6e1f3:	cmp    0x8(%rsp,%r11,1),%edi
   6e1f8:	jae    6def6 <mibz2::bwt::build+0x1a6>
   6e1fe:	cmp    %rdi,%rbx
   6e201:	jbe    6e618 <mibz2::bwt::build+0x8c8>
   6e207:	mov    %r9,%r11
   6e20a:	shr    $0x18,%r11
   6e20e:	movzbl (%r14,%rdi,1),%r13d
   6e213:	lea    (%r8,%r13,1),%ebp
   6e217:	add    $0x200,%ebp
   6e21d:	mov    %ebp,(%rdx,%rdi,4)
   6e220:	inc    %edi
   6e222:	mov    %edi,0x408(%rsp,%r10,4)
   6e22a:	movzbl %r11b,%r10d
   6e22e:	lea    0x0(,%r10,4),%r11d
   6e236:	mov    0x408(%rsp,%r11,1),%edi
   6e23e:	cmp    0x8(%rsp,%r11,1),%edi
   6e243:	jae    6def6 <mibz2::bwt::build+0x1a6>
   6e249:	cmp    %rdi,%rbx
   6e24c:	jbe    6e618 <mibz2::bwt::build+0x8c8>
   6e252:	mov    %r9,%r11
   6e255:	shr    $0x20,%r11
   6e259:	movzbl (%r14,%rdi,1),%r13d
   6e25e:	lea    (%r8,%r13,1),%ebp
   6e262:	add    $0x300,%ebp
   6e268:	mov    %ebp,(%rdx,%rdi,4)
   6e26b:	inc    %edi
   6e26d:	mov    %edi,0x408(%rsp,%r10,4)
   6e275:	movzbl %r11b,%r10d
   6e279:	lea    0x0(,%r10,4),%r11d
   6e281:	mov    0x408(%rsp,%r11,1),%edi
   6e289:	cmp    0x8(%rsp,%r11,1),%edi
   6e28e:	jae    6def6 <mibz2::bwt::build+0x1a6>
   6e294:	cmp    %rdi,%rbx
   6e297:	jbe    6e618 <mibz2::bwt::build+0x8c8>
   6e29d:	mov    %r9,%r11
   6e2a0:	shr    $0x28,%r11
   6e2a4:	movzbl (%r14,%rdi,1),%r13d
   6e2a9:	lea    (%r8,%r13,1),%ebp
   6e2ad:	add    $0x400,%ebp
   6e2b3:	mov    %ebp,(%rdx,%rdi,4)
   6e2b6:	inc    %edi
   6e2b8:	mov    %edi,0x408(%rsp,%r10,4)
   6e2c0:	movzbl %r11b,%r10d
   6e2c4:	lea    0x0(,%r10,4),%r11d
   6e2cc:	mov    0x408(%rsp,%r11,1),%edi
   6e2d4:	cmp    0x8(%rsp,%r11,1),%edi
   6e2d9:	jae    6def6 <mibz2::bwt::build+0x1a6>
   6e2df:	cmp    %rdi,%rbx
   6e2e2:	jbe    6e618 <mibz2::bwt::build+0x8c8>
   6e2e8:	mov    %r9,%r11
   6e2eb:	shr    $0x30,%r11
   6e2ef:	movzbl (%r14,%rdi,1),%r13d
   6e2f4:	lea    (%r8,%r13,1),%ebp
   6e2f8:	add    $0x500,%ebp
   6e2fe:	mov    %ebp,(%rdx,%rdi,4)
   6e301:	inc    %edi
   6e303:	mov    %edi,0x408(%rsp,%r10,4)
   6e30b:	movzbl %r11b,%r10d
   6e30f:	lea    0x0(,%r10,4),%r11d
   6e317:	mov    0x408(%rsp,%r11,1),%edi
   6e31f:	cmp    0x8(%rsp,%r11,1),%edi
   6e324:	jae    6def6 <mibz2::bwt::build+0x1a6>
   6e32a:	cmp    %rdi,%rbx
   6e32d:	jbe    6e618 <mibz2::bwt::build+0x8c8>
   6e333:	shr    $0x38,%r9
   6e337:	movzbl (%r14,%rdi,1),%r11d
   6e33c:	add    %r8d,%r11d
   6e33f:	add    $0x600,%r11d
   6e346:	mov    %r11d,(%rdx,%rdi,4)
   6e34a:	inc    %edi
   6e34c:	mov    %edi,0x408(%rsp,%r10,4)
   6e354:	lea    0x0(,%r9,4),%r10d
   6e35c:	mov    0x408(%rsp,%r10,1),%edi
   6e364:	cmp    0x8(%rsp,%r10,1),%edi
   6e369:	jae    6def6 <mibz2::bwt::build+0x1a6>
   6e36f:	cmp    %rdi,%rbx
   6e372:	jbe    6e618 <mibz2::bwt::build+0x8c8>
   6e378:	movzbl (%r14,%rdi,1),%r10d
   6e37d:	add    %r8d,%r10d
   6e380:	add    $0x700,%r10d
   6e387:	mov    %r10d,(%rdx,%rdi,4)
   6e38b:	inc    %edi
   6e38d:	mov    %edi,0x408(%rsp,%r9,4)
   6e395:	movzbl %al,%r9d
   6e399:	lea    0x0(,%r9,4),%r10d
   6e3a1:	mov    0x408(%rsp,%r10,1),%edi
   6e3a9:	cmp    0x8(%rsp,%r10,1),%edi
   6e3ae:	jae    6def6 <mibz2::bwt::build+0x1a6>
   6e3b4:	cmp    %rdi,%rbx
   6e3b7:	jbe    6e618 <mibz2::bwt::build+0x8c8>
   6e3bd:	mov    %rax,%r10
   6e3c0:	shr    $0x8,%r10
   6e3c4:	movzbl (%r14,%rdi,1),%r11d
   6e3c9:	add    %r8d,%r11d
   6e3cc:	add    $0x800,%r11d
   6e3d3:	mov    %r11d,(%rdx,%rdi,4)
   6e3d7:	inc    %edi
   6e3d9:	mov    %edi,0x408(%rsp,%r9,4)
   6e3e1:	movzbl %r10b,%r9d
   6e3e5:	lea    0x0(,%r9,4),%r10d
   6e3ed:	mov    0x408(%rsp,%r10,1),%edi
   6e3f5:	cmp    0x8(%rsp,%r10,1),%edi
   6e3fa:	jae    6def6 <mibz2::bwt::build+0x1a6>
   6e400:	cmp    %rdi,%rbx
   6e403:	jbe    6e618 <mibz2::bwt::build+0x8c8>
   6e409:	mov    %rax,%r10
   6e40c:	shr    $0x10,%r10
   6e410:	movzbl (%r14,%rdi,1),%r11d
   6e415:	add    %r8d,%r11d
   6e418:	add    $0x900,%r11d
   6e41f:	mov    %r11d,(%rdx,%rdi,4)
   6e423:	inc    %edi
   6e425:	mov    %edi,0x408(%rsp,%r9,4)
   6e42d:	movzbl %r10b,%r9d
   6e431:	lea    0x0(,%r9,4),%r10d
   6e439:	mov    0x408(%rsp,%r10,1),%edi
   6e441:	cmp    0x8(%rsp,%r10,1),%edi
   6e446:	jae    6def6 <mibz2::bwt::build+0x1a6>
   6e44c:	cmp    %rdi,%rbx
   6e44f:	jbe    6e618 <mibz2::bwt::build+0x8c8>
   6e455:	mov    %rax,%r10
   6e458:	shr    $0x18,%r10
   6e45c:	movzbl (%r14,%rdi,1),%r11d
   6e461:	add    %r8d,%r11d
   6e464:	add    $0xa00,%r11d
   6e46b:	mov    %r11d,(%rdx,%rdi,4)
   6e46f:	inc    %edi
   6e471:	mov    %edi,0x408(%rsp,%r9,4)
   6e479:	movzbl %r10b,%r9d
   6e47d:	lea    0x0(,%r9,4),%r10d
   6e485:	mov    0x408(%rsp,%r10,1),%edi
   6e48d:	cmp    0x8(%rsp,%r10,1),%edi
   6e492:	jae    6def6 <mibz2::bwt::build+0x1a6>
   6e498:	cmp    %rdi,%rbx
   6e49b:	jbe    6e618 <mibz2::bwt::build+0x8c8>
   6e4a1:	mov    %rax,%r10
   6e4a4:	shr    $0x20,%r10
   6e4a8:	movzbl (%r14,%rdi,1),%r11d
   6e4ad:	add    %r8d,%r11d
   6e4b0:	add    $0xb00,%r11d
   6e4b7:	mov    %r11d,(%rdx,%rdi,4)
   6e4bb:	inc    %edi
   6e4bd:	mov    %edi,0x408(%rsp,%r9,4)
   6e4c5:	movzbl %r10b,%r9d
   6e4c9:	lea    0x0(,%r9,4),%r10d
   6e4d1:	mov    0x408(%rsp,%r10,1),%edi
   6e4d9:	cmp    0x8(%rsp,%r10,1),%edi
   6e4de:	jae    6def6 <mibz2::bwt::build+0x1a6>
   6e4e4:	cmp    %rdi,%rbx
   6e4e7:	jbe    6e618 <mibz2::bwt::build+0x8c8>
   6e4ed:	mov    %rax,%r10
   6e4f0:	shr    $0x28,%r10
   6e4f4:	movzbl (%r14,%rdi,1),%r11d
   6e4f9:	add    %r8d,%r11d
   6e4fc:	add    $0xc00,%r11d
   6e503:	mov    %r11d,(%rdx,%rdi,4)
   6e507:	inc    %edi
   6e509:	mov    %edi,0x408(%rsp,%r9,4)
   6e511:	movzbl %r10b,%r9d
   6e515:	lea    0x0(,%r9,4),%r10d
   6e51d:	mov    0x408(%rsp,%r10,1),%edi
   6e525:	cmp    0x8(%rsp,%r10,1),%edi
   6e52a:	jae    6def6 <mibz2::bwt::build+0x1a6>
   6e530:	cmp    %rdi,%rbx
   6e533:	jbe    6e618 <mibz2::bwt::build+0x8c8>
   6e539:	mov    %rax,%r10
   6e53c:	shr    $0x30,%r10
   6e540:	movzbl (%r14,%rdi,1),%r11d
   6e545:	add    %r8d,%r11d
   6e548:	add    $0xd00,%r11d
   6e54f:	mov    %r11d,(%rdx,%rdi,4)
   6e553:	inc    %edi
   6e555:	mov    %edi,0x408(%rsp,%r9,4)
   6e55d:	movzbl %r10b,%r9d
   6e561:	lea    0x0(,%r9,4),%r10d
   6e569:	mov    0x408(%rsp,%r10,1),%edi
   6e571:	cmp    0x8(%rsp,%r10,1),%edi
   6e576:	jae    6def6 <mibz2::bwt::build+0x1a6>
   6e57c:	cmp    %rdi,%rbx
   6e57f:	jbe    6e618 <mibz2::bwt::build+0x8c8>
   6e585:	shr    $0x38,%rax
   6e589:	movzbl (%r14,%rdi,1),%r10d
   6e58e:	add    %r8d,%r10d
   6e591:	add    $0xe00,%r10d
   6e598:	mov    %r10d,(%rdx,%rdi,4)
   6e59c:	inc    %edi
   6e59e:	mov    %edi,0x408(%rsp,%r9,4)
   6e5a6:	lea    0x0(,%rax,4),%r9d
   6e5ae:	mov    0x408(%rsp,%r9,1),%edi
   6e5b6:	cmp    0x8(%rsp,%r9,1),%edi
   6e5bb:	jae    6def6 <mibz2::bwt::build+0x1a6>
   6e5c1:	cmp    %rdi,%rbx
   6e5c4:	jbe    6e618 <mibz2::bwt::build+0x8c8>
   6e5c6:	movzbl (%r14,%rdi,1),%r9d
   6e5cb:	add    %r8d,%r9d
   6e5ce:	add    $0xf00,%r9d
   6e5d5:	mov    %r9d,(%rdx,%rdi,4)
   6e5d9:	inc    %edi
   6e5db:	mov    %edi,0x408(%rsp,%rax,4)
   6e5e2:	jmp    6e010 <mibz2::bwt::build+0x2c0>
   6e5e7:	add    $0xfffffffffffffff8,%rax
   6e5eb:	mov    %rax,%r8
   6e5ee:	cmp    %rbx,%r8
   6e5f1:	jb     6de7e <mibz2::bwt::build+0x12e>
   6e5f7:	jmp    6ded7 <mibz2::bwt::build+0x187>
   6e5fc:	mov    %ebx,%edi
   6e5fe:	and    $0xfffff0,%edi
   6e604:	jmp    6e639 <mibz2::bwt::build+0x8e9>
   6e606:	lea    0x497a3(%rip),%rcx        # b7db0 <__frame_dummy_init_array_entry+0x2460>
   6e60d:	mov    %rax,%rsi
   6e610:	mov    %rbx,%rdx
   6e613:	call   63690 <core::slice::index::slice_index_fail>
   6e618:	lea    0x49671(%rip),%rdx        # b7c90 <__frame_dummy_init_array_entry+0x2340>
   6e61f:	mov    %rbx,%rsi
   6e622:	call   6292b <core::panicking::panic_bounds_check>
   6e627:	lea    0x497e2(%rip),%rcx        # b7e10 <__frame_dummy_init_array_entry+0x24c0>
   6e62e:	mov    %rax,%rdi
   6e631:	mov    %rbx,%rdx
   6e634:	call   63690 <core::slice::index::slice_index_fail>
   6e639:	or     $0x8,%rdi
   6e63d:	add    $0x10,%rax
   6e641:	lea    0x497c8(%rip),%rcx        # b7e10 <__frame_dummy_init_array_entry+0x24c0>
   6e648:	mov    %rax,%rsi
   6e64b:	mov    %rbx,%rdx
   6e64e:	call   63690 <core::slice::index::slice_index_fail>
   6e653:	add    $0x808,%rsp
   6e65a:	pop    %rbx
   6e65b:	pop    %r12
   6e65d:	pop    %r13
   6e65f:	pop    %r14
   6e661:	pop    %r15
   6e663:	pop    %rbp
   6e664:	ret
   6e665:	movzbl (%r14,%rax,1),%edx
   6e66a:	mov    %edx,0x0(%r13,%r12,4)
   6e66f:	inc    %r12
   6e672:	inc    %rax
   6e675:	cmp    %rax,%rbx
   6e678:	jne    6e665 <mibz2::bwt::build+0x915>
   6e67a:	mov    %r12,0x10(%rcx)
   6e67e:	lea    0x8(%rsp),%rdi
   6e683:	xor    %ebp,%ebp
   6e685:	mov    $0x400,%edx
   6e68a:	xor    %esi,%esi
   6e68c:	call   *0x4b2c6(%rip)        # b9958 <memset@GLIBC_2.2.5>
   6e692:	xor    %eax,%eax
   6e694:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   6e6a0:	mov    %eax,0x8(%rsp,%rbp,4)
   6e6a4:	add    (%r15,%rbp,4),%eax
   6e6a8:	mov    %eax,0xc(%rsp,%rbp,4)
   6e6ac:	add    0x4(%r15,%rbp,4),%eax
   6e6b1:	mov    %eax,0x10(%rsp,%rbp,4)
   6e6b5:	add    0x8(%r15,%rbp,4),%eax
   6e6ba:	mov    %eax,0x14(%rsp,%rbp,4)
   6e6be:	add    0xc(%r15,%rbp,4),%eax
   6e6c3:	add    $0x4,%rbp
   6e6c7:	cmp    $0x100,%rbp
   6e6ce:	jne    6e6a0 <mibz2::bwt::build+0x950>
   6e6d0:	xor    %ecx,%ecx
   6e6d2:	cmp    $0x10,%rbx
   6e6d6:	jb     6eb6a <mibz2::bwt::build+0xe1a>
   6e6dc:	mov    $0x8,%esi
   6e6e1:	movdqa -0x64ea9(%rip),%xmm0        # 9840 <GCC_except_table733+0x7f4>
   6e6e9:	movdqa -0x65631(%rip),%xmm1        # 90c0 <GCC_except_table733+0x74>
   6e6f1:	movdqa -0x65189(%rip),%xmm2        # 9570 <GCC_except_table733+0x524>
   6e6f9:	movdqa -0x64da1(%rip),%xmm3        # 9960 <GCC_except_table733+0x914>
   6e701:	movq   -0x65219(%rip),%xmm4        # 94f0 <GCC_except_table733+0x4a4>
   6e709:	movq   -0x651e1(%rip),%xmm5        # 9530 <GCC_except_table733+0x4e4>
   6e711:	jmp    6e7ed <mibz2::bwt::build+0xa9d>
   6e716:	cs nopw 0x0(%rax,%rax,1)
   6e720:	movzbl %dl,%edx
   6e723:	mov    %edx,%eax
   6e725:	mov    0x8(%rsp,%rax,4),%edi
   6e729:	lea    0x10(%rdi),%rax
   6e72d:	cmp    %r12,%rax
   6e730:	ja     6ebc6 <mibz2::bwt::build+0xe76>
   6e736:	movzbl 0x0(%r13,%rdi,4),%eax
   6e73c:	add    %ecx,%eax
   6e73e:	mov    %eax,0x0(%r13,%rdi,4)
   6e743:	movdqu 0x4(%r13,%rdi,4),%xmm6
   6e74a:	movdqu 0x14(%r13,%rdi,4),%xmm7
   6e751:	movdqu 0x24(%r13,%rdi,4),%xmm8
   6e758:	pand   %xmm0,%xmm6
   6e75c:	movd   %ecx,%xmm9
   6e761:	pshufd $0x0,%xmm9,%xmm9
   6e767:	por    %xmm9,%xmm6
   6e76c:	por    %xmm1,%xmm6
   6e770:	movdqu %xmm6,0x4(%r13,%rdi,4)
   6e777:	pand   %xmm0,%xmm7
   6e77b:	por    %xmm9,%xmm7
   6e780:	por    %xmm2,%xmm7
   6e784:	movdqu %xmm7,0x14(%r13,%rdi,4)
   6e78b:	pand   %xmm0,%xmm8
   6e790:	por    %xmm9,%xmm8
   6e795:	por    %xmm3,%xmm8
   6e79a:	movdqu %xmm8,0x24(%r13,%rdi,4)
   6e7a1:	movq   0x34(%r13,%rdi,4),%xmm6
   6e7a8:	pand   %xmm4,%xmm6
   6e7ac:	por    %xmm9,%xmm6
   6e7b1:	por    %xmm5,%xmm6
   6e7b5:	movq   %xmm6,0x34(%r13,%rdi,4)
   6e7bc:	movzbl 0x3c(%r13,%rdi,4),%eax
   6e7c2:	add    %ecx,%eax
   6e7c4:	add    $0xf00,%eax
   6e7c9:	mov    %eax,0x3c(%r13,%rdi,4)
   6e7ce:	addl   $0x10,0x8(%rsp,%rdx,4)
   6e7d3:	lea    0x10(%rsi),%rax
   6e7d7:	add    $0x18,%rsi
   6e7db:	add    $0x1000,%ecx
   6e7e1:	cmp    %rbx,%rsi
   6e7e4:	mov    %rax,%rsi
   6e7e7:	ja     6eb63 <mibz2::bwt::build+0xe13>
   6e7ed:	lea    -0x8(%rsi),%rax
   6e7f1:	cmp    %rbx,%rsi
   6e7f4:	ja     6e627 <mibz2::bwt::build+0x8d7>
   6e7fa:	lea    0x7(%rsi),%rdx
   6e7fe:	cmp    %rbx,%rdx
   6e801:	jae    6ebb4 <mibz2::bwt::build+0xe64>
   6e807:	mov    -0x8(%r14,%rsi,1),%rdx
   6e80c:	mov    (%r14,%rsi,1),%rax
   6e810:	mov    %rdx,%rdi
   6e813:	xor    %rax,%rdi
   6e816:	mov    %rdx,%r8
   6e819:	rol    $0x8,%r8
   6e81d:	xor    %rdx,%r8
   6e820:	or     %rdi,%r8
   6e823:	je     6e720 <mibz2::bwt::build+0x9d0>
   6e829:	movzbl %dl,%r8d
   6e82d:	mov    %r8d,%edi
   6e830:	mov    0x8(%rsp,%rdi,4),%edi
   6e834:	cmp    %rdi,%r12
   6e837:	jbe    6ebd8 <mibz2::bwt::build+0xe88>
   6e83d:	mov    %rdx,%r9
   6e840:	shr    $0x8,%r9
   6e844:	movzbl 0x0(%r13,%rdi,4),%r10d
   6e84a:	add    %ecx,%r10d
   6e84d:	mov    %r10d,0x0(%r13,%rdi,4)
   6e852:	inc    %edi
   6e854:	mov    %edi,0x8(%rsp,%r8,4)
   6e859:	movzbl %r9b,%r8d
   6e85d:	mov    %r8d,%edi
   6e860:	mov    0x8(%rsp,%rdi,4),%edi
   6e864:	cmp    %rdi,%r12
   6e867:	jbe    6ebd8 <mibz2::bwt::build+0xe88>
   6e86d:	mov    %rdx,%r9
   6e870:	shr    $0x10,%r9
   6e874:	movzbl 0x0(%r13,%rdi,4),%r10d
   6e87a:	add    %ecx,%r10d
   6e87d:	add    $0x100,%r10d
   6e884:	mov    %r10d,0x0(%r13,%rdi,4)
   6e889:	inc    %edi
   6e88b:	mov    %edi,0x8(%rsp,%r8,4)
   6e890:	movzbl %r9b,%r8d
   6e894:	mov    %r8d,%edi
   6e897:	mov    0x8(%rsp,%rdi,4),%edi
   6e89b:	cmp    %rdi,%r12
   6e89e:	jbe    6ebd8 <mibz2::bwt::build+0xe88>
   6e8a4:	mov    %rdx,%r9
   6e8a7:	shr    $0x18,%r9
   6e8ab:	movzbl 0x0(%r13,%rdi,4),%r10d
   6e8b1:	add    %ecx,%r10d
   6e8b4:	add    $0x200,%r10d
   6e8bb:	mov    %r10d,0x0(%r13,%rdi,4)
   6e8c0:	inc    %edi
   6e8c2:	mov    %edi,0x8(%rsp,%r8,4)
   6e8c7:	movzbl %r9b,%r8d
   6e8cb:	mov    %r8d,%edi
   6e8ce:	mov    0x8(%rsp,%rdi,4),%edi
   6e8d2:	cmp    %rdi,%r12
   6e8d5:	jbe    6ebd8 <mibz2::bwt::build+0xe88>
   6e8db:	mov    %rdx,%r9
   6e8de:	shr    $0x20,%r9
   6e8e2:	movzbl 0x0(%r13,%rdi,4),%r10d
   6e8e8:	add    %ecx,%r10d
   6e8eb:	add    $0x300,%r10d
   6e8f2:	mov    %r10d,0x0(%r13,%rdi,4)
   6e8f7:	inc    %edi
   6e8f9:	mov    %edi,0x8(%rsp,%r8,4)
   6e8fe:	movzbl %r9b,%r8d
   6e902:	mov    %r8d,%edi
   6e905:	mov    0x8(%rsp,%rdi,4),%edi
   6e909:	cmp    %rdi,%r12
   6e90c:	jbe    6ebd8 <mibz2::bwt::build+0xe88>
   6e912:	mov    %rdx,%r9
   6e915:	shr    $0x28,%r9
   6e919:	movzbl 0x0(%r13,%rdi,4),%r10d
   6e91f:	add    %ecx,%r10d
   6e922:	add    $0x400,%r10d
   6e929:	mov    %r10d,0x0(%r13,%rdi,4)
   6e92e:	inc    %edi
   6e930:	mov    %edi,0x8(%rsp,%r8,4)
   6e935:	movzbl %r9b,%r8d
   6e939:	mov    %r8d,%edi
   6e93c:	mov    0x8(%rsp,%rdi,4),%edi
   6e940:	cmp    %rdi,%r12
   6e943:	jbe    6ebd8 <mibz2::bwt::build+0xe88>
   6e949:	mov    %rdx,%r9
   6e94c:	shr    $0x30,%r9
   6e950:	movzbl 0x0(%r13,%rdi,4),%r10d
   6e956:	add    %ecx,%r10d
   6e959:	add    $0x500,%r10d
   6e960:	mov    %r10d,0x0(%r13,%rdi,4)
   6e965:	inc    %edi
   6e967:	mov    %edi,0x8(%rsp,%r8,4)
   6e96c:	movzbl %r9b,%r8d
   6e970:	mov    %r8d,%edi
   6e973:	mov    0x8(%rsp,%rdi,4),%edi
   6e977:	cmp    %rdi,%r12
   6e97a:	jbe    6ebd8 <mibz2::bwt::build+0xe88>
   6e980:	shr    $0x38,%rdx
   6e984:	movzbl 0x0(%r13,%rdi,4),%r9d
   6e98a:	add    %ecx,%r9d
   6e98d:	add    $0x600,%r9d
   6e994:	mov    %r9d,0x0(%r13,%rdi,4)
   6e999:	inc    %edi
   6e99b:	mov    %edi,0x8(%rsp,%r8,4)
   6e9a0:	mov    0x8(%rsp,%rdx,4),%edi
   6e9a4:	cmp    %rdi,%r12
   6e9a7:	jbe    6ebd8 <mibz2::bwt::build+0xe88>
   6e9ad:	movzbl 0x0(%r13,%rdi,4),%r8d
   6e9b3:	add    %ecx,%r8d
   6e9b6:	add    $0x700,%r8d
   6e9bd:	mov    %r8d,0x0(%r13,%rdi,4)
   6e9c2:	inc    %edi
   6e9c4:	mov    %edi,0x8(%rsp,%rdx,4)
   6e9c8:	movzbl %al,%edx
   6e9cb:	mov    %edx,%edi
   6e9cd:	mov    0x8(%rsp,%rdi,4),%edi
   6e9d1:	cmp    %rdi,%r12
   6e9d4:	jbe    6ebd8 <mibz2::bwt::build+0xe88>
   6e9da:	mov    %rax,%r8
   6e9dd:	shr    $0x8,%r8
   6e9e1:	movzbl 0x0(%r13,%rdi,4),%r9d
   6e9e7:	add    %ecx,%r9d
   6e9ea:	add    $0x800,%r9d
   6e9f1:	mov    %r9d,0x0(%r13,%rdi,4)
   6e9f6:	inc    %edi
   6e9f8:	mov    %edi,0x8(%rsp,%rdx,4)
   6e9fc:	movzbl %r8b,%edx
   6ea00:	mov    %edx,%edi
   6ea02:	mov    0x8(%rsp,%rdi,4),%edi
   6ea06:	cmp    %rdi,%r12
   6ea09:	jbe    6ebd8 <mibz2::bwt::build+0xe88>
   6ea0f:	mov    %rax,%r8
   6ea12:	shr    $0x10,%r8
   6ea16:	movzbl 0x0(%r13,%rdi,4),%r9d
   6ea1c:	add    %ecx,%r9d
   6ea1f:	add    $0x900,%r9d
   6ea26:	mov    %r9d,0x0(%r13,%rdi,4)
   6ea2b:	inc    %edi
   6ea2d:	mov    %edi,0x8(%rsp,%rdx,4)
   6ea31:	movzbl %r8b,%edx
   6ea35:	mov    %edx,%edi
   6ea37:	mov    0x8(%rsp,%rdi,4),%edi
   6ea3b:	cmp    %rdi,%r12
   6ea3e:	jbe    6ebd8 <mibz2::bwt::build+0xe88>
   6ea44:	mov    %rax,%r8
   6ea47:	shr    $0x18,%r8
   6ea4b:	movzbl 0x0(%r13,%rdi,4),%r9d
   6ea51:	add    %ecx,%r9d
   6ea54:	add    $0xa00,%r9d
   6ea5b:	mov    %r9d,0x0(%r13,%rdi,4)
   6ea60:	inc    %edi
   6ea62:	mov    %edi,0x8(%rsp,%rdx,4)
   6ea66:	movzbl %r8b,%edx
   6ea6a:	mov    %edx,%edi
   6ea6c:	mov    0x8(%rsp,%rdi,4),%edi
   6ea70:	cmp    %rdi,%r12
   6ea73:	jbe    6ebd8 <mibz2::bwt::build+0xe88>
   6ea79:	mov    %rax,%r8
   6ea7c:	shr    $0x20,%r8
   6ea80:	movzbl 0x0(%r13,%rdi,4),%r9d
   6ea86:	add    %ecx,%r9d
   6ea89:	add    $0xb00,%r9d
   6ea90:	mov    %r9d,0x0(%r13,%rdi,4)
   6ea95:	inc    %edi
   6ea97:	mov    %edi,0x8(%rsp,%rdx,4)
   6ea9b:	movzbl %r8b,%edx
   6ea9f:	mov    %edx,%edi
   6eaa1:	mov    0x8(%rsp,%rdi,4),%edi
   6eaa5:	cmp    %rdi,%r12
   6eaa8:	jbe    6ebd8 <mibz2::bwt::build+0xe88>
   6eaae:	mov    %rax,%r8
   6eab1:	shr    $0x28,%r8
   6eab5:	movzbl 0x0(%r13,%rdi,4),%r9d
   6eabb:	add    %ecx,%r9d
   6eabe:	add    $0xc00,%r9d
   6eac5:	mov    %r9d,0x0(%r13,%rdi,4)
   6eaca:	inc    %edi
   6eacc:	mov    %edi,0x8(%rsp,%rdx,4)
   6ead0:	movzbl %r8b,%edx
   6ead4:	mov    %edx,%edi
   6ead6:	mov    0x8(%rsp,%rdi,4),%edi
   6eada:	cmp    %rdi,%r12
   6eadd:	jbe    6ebd8 <mibz2::bwt::build+0xe88>
   6eae3:	mov    %rax,%r8
   6eae6:	shr    $0x30,%r8
   6eaea:	movzbl 0x0(%r13,%rdi,4),%r9d
   6eaf0:	add    %ecx,%r9d
   6eaf3:	add    $0xd00,%r9d
   6eafa:	mov    %r9d,0x0(%r13,%rdi,4)
   6eaff:	inc    %edi
   6eb01:	mov    %edi,0x8(%rsp,%rdx,4)
   6eb05:	movzbl %r8b,%edx
   6eb09:	mov    %edx,%edi
   6eb0b:	mov    0x8(%rsp,%rdi,4),%edi
   6eb0f:	cmp    %rdi,%r12
   6eb12:	jbe    6ebd8 <mibz2::bwt::build+0xe88>
   6eb18:	shr    $0x38,%rax
   6eb1c:	movzbl 0x0(%r13,%rdi,4),%r8d
   6eb22:	add    %ecx,%r8d
   6eb25:	add    $0xe00,%r8d
   6eb2c:	mov    %r8d,0x0(%r13,%rdi,4)
   6eb31:	inc    %edi
   6eb33:	mov    %edi,0x8(%rsp,%rdx,4)
   6eb37:	mov    0x8(%rsp,%rax,4),%edi
   6eb3b:	cmp    %rdi,%r12
   6eb3e:	jbe    6ebd8 <mibz2::bwt::build+0xe88>
   6eb44:	movzbl 0x0(%r13,%rdi,4),%edx
   6eb4a:	add    %ecx,%edx
   6eb4c:	add    $0xf00,%edx
   6eb52:	mov    %edx,0x0(%r13,%rdi,4)
   6eb57:	lea    0x1(%rdi),%edx
   6eb5a:	mov    %edx,0x8(%rsp,%rax,4)
   6eb5e:	jmp    6e7d3 <mibz2::bwt::build+0xa83>
   6eb63:	add    $0xfffffffffffffff8,%rax
   6eb67:	mov    %rax,%rcx
   6eb6a:	cmp    %rbx,%rcx
   6eb6d:	jae    6e653 <mibz2::bwt::build+0x903>
   6eb73:	mov    %ecx,%eax
   6eb75:	shl    $0x8,%eax
   6eb78:	nopl   0x0(%rax,%rax,1)
   6eb80:	movzbl (%r14,%rcx,1),%edx
   6eb85:	mov    0x8(%rsp,%rdx,4),%edi
   6eb89:	cmp    %rdi,%r12
   6eb8c:	jbe    6ebd8 <mibz2::bwt::build+0xe88>
   6eb8e:	inc    %rcx
   6eb91:	movzbl 0x0(%r13,%rdi,4),%esi
   6eb97:	add    %eax,%esi
   6eb99:	mov    %esi,0x0(%r13,%rdi,4)
   6eb9e:	lea    0x1(%rdi),%esi
   6eba1:	mov    %esi,0x8(%rsp,%rdx,4)
   6eba5:	add    $0x100,%eax
   6ebaa:	cmp    %rcx,%rbx
   6ebad:	jne    6eb80 <mibz2::bwt::build+0xe30>
   6ebaf:	jmp    6e653 <mibz2::bwt::build+0x903>
   6ebb4:	movabs $0x7ffffffffffffff0,%rdi
   6ebbe:	and    %rbx,%rdi
   6ebc1:	jmp    6e639 <mibz2::bwt::build+0x8e9>
   6ebc6:	lea    0x491fb(%rip),%rcx        # b7dc8 <__frame_dummy_init_array_entry+0x2478>
   6ebcd:	mov    %rax,%rsi
   6ebd0:	mov    %r12,%rdx
   6ebd3:	call   63690 <core::slice::index::slice_index_fail>
   6ebd8:	lea    0x49201(%rip),%rdx        # b7de0 <__frame_dummy_init_array_entry+0x2490>
   6ebdf:	mov    %r12,%rsi
   6ebe2:	call   6292b <core::panicking::panic_bounds_check>
