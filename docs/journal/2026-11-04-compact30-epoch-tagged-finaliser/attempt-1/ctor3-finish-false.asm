# <c30walk::ctor3::W<false>>::finish
# size 0xbc
# attempt 1: u0 read by a plain load; LLVM proved field 0 - u0 = 0

target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

0000000000081090 <<c30walk::ctor3::W<false>>::finish>:
   81090:	movzbl %sil,%esi
   81094:	lea    (%rsi,%rsi,4),%rax
   81098:	mov    0x64(%rdi,%rax,8),%ecx
   8109c:	cmp    0x284c(%rdi),%ecx
   810a2:	jne    80a90 <<c30walk::ctor3::W<false>>::finish_slow>
   810a8:	lea    (%rdi,%rax,8),%rax
   810ac:	add    $0x40,%rax
   810b0:	mov    (%rax),%rdx
   810b3:	mov    0x10(%rax),%rax
   810b7:	sub    %rax,%rdx
   810ba:	cmp    $0x3c,%rdx
   810be:	jne    80a90 <<c30walk::ctor3::W<false>>::finish_slow>
   810c4:	and    $0x7fffffff,%ecx
   810ca:	or     %esi,%ecx
   810cc:	movq   (%rax),%xmm0
   810d0:	movzwl (%rax),%edx
   810d3:	movd   %edx,%xmm1
   810d7:	shl    $0x8,%edx
   810da:	or     %ecx,%edx
   810dc:	mov    %edx,0x3c(%rax)
   810df:	movq   0x8(%rax),%xmm2
   810e4:	xor    %ecx,%ecx
   810e6:	pshuflw $0x0,%xmm0,%xmm3
   810eb:	pinsrw $0x0,%ecx,%xmm0
   810f0:	punpcklqdq %xmm2,%xmm0
   810f4:	pshufd $0x44,%xmm3,%xmm2
   810f9:	pand   -0x732b1(%rip),%xmm2        # de50 <GCC_except_table943+0x3c8>
   81101:	psubw  %xmm2,%xmm0
   81105:	movdqu %xmm0,(%rax)
   81109:	pshuflw $0x0,%xmm1,%xmm0
   8110e:	pshufd $0x44,%xmm0,%xmm1
   81113:	movdqu 0x10(%rax),%xmm2
   81118:	movdqu 0x20(%rax),%xmm3
   8111d:	psubw  %xmm1,%xmm2
   81121:	movdqu %xmm2,0x10(%rax)
   81126:	psubw  %xmm1,%xmm3
   8112a:	movdqu %xmm3,0x20(%rax)
   8112f:	movq   0x30(%rax),%xmm1
   81134:	psubw  %xmm0,%xmm1
   81138:	movq   %xmm1,0x30(%rax)
   8113d:	movd   0x38(%rax),%xmm1
   81142:	psubw  %xmm0,%xmm1
   81146:	movd   %xmm1,0x38(%rax)
   8114b:	ret
