# <c30walk::ctor3::W<false>>::finish
# size 0xc2
# attempt 2 (the one permitted iteration): u0 read with read_volatile; LLVM still folded lane 0

target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

0000000000081050 <<c30walk::ctor3::W<false>>::finish>:
   81050:	movzbl %sil,%esi
   81054:	lea    (%rsi,%rsi,4),%rax
   81058:	mov    0x64(%rdi,%rax,8),%ecx
   8105c:	cmp    0x284c(%rdi),%ecx
   81062:	jne    80a50 <<c30walk::ctor3::W<false>>::finish_slow>
   81068:	lea    (%rdi,%rax,8),%rax
   8106c:	add    $0x40,%rax
   81070:	mov    (%rax),%rdx
   81073:	mov    0x10(%rax),%rax
   81077:	sub    %rax,%rdx
   8107a:	cmp    $0x3c,%rdx
   8107e:	jne    80a50 <<c30walk::ctor3::W<false>>::finish_slow>
   81084:	movzwl (%rax),%edx
   81087:	and    $0x7fffffff,%ecx
   8108d:	pxor   %xmm0,%xmm0
   81091:	pinsrw $0x1,%edx,%xmm0
   81096:	movd   %edx,%xmm1
   8109a:	shl    $0x8,%edx
   8109d:	or     %esi,%ecx
   8109f:	or     %edx,%ecx
   810a1:	mov    %ecx,0x3c(%rax)
   810a4:	movd   0x2(%rax),%xmm2
   810a9:	psllq  $0x10,%xmm2
   810ae:	pinsrw $0x3,0x6(%rax),%xmm2
   810b4:	movq   0x8(%rax),%xmm3
   810b9:	punpcklqdq %xmm3,%xmm2
   810bd:	pshuflw $0x54,%xmm0,%xmm0
   810c2:	pshufd $0x54,%xmm0,%xmm0
   810c7:	psubw  %xmm0,%xmm2
   810cb:	movdqu %xmm2,(%rax)
   810cf:	pshuflw $0x0,%xmm1,%xmm0
   810d4:	pshufd $0x44,%xmm0,%xmm1
   810d9:	movdqu 0x10(%rax),%xmm2
   810de:	movdqu 0x20(%rax),%xmm3
   810e3:	psubw  %xmm1,%xmm2
   810e7:	movdqu %xmm2,0x10(%rax)
   810ec:	psubw  %xmm1,%xmm3
   810f0:	movdqu %xmm3,0x20(%rax)
   810f5:	movq   0x30(%rax),%xmm1
   810fa:	psubw  %xmm0,%xmm1
   810fe:	movq   %xmm1,0x30(%rax)
   81103:	movd   0x38(%rax),%xmm1
   81108:	psubw  %xmm0,%xmm1
   8110c:	movd   %xmm1,0x38(%rax)
   81111:	ret
