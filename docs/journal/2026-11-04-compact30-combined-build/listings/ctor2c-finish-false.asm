# <c30walk::ctor2c::W<false>>::finish
# size 0xc2

target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

000000000008adc0 <<c30walk::ctor2c::W<false>>::finish>:
   8adc0:	movzbl %sil,%esi
   8adc4:	lea    (%rsi,%rsi,2),%rax
   8adc8:	mov    0x1054(%rdi,%rax,8),%ecx
   8adcf:	cmp    0x284c(%rdi),%ecx
   8add5:	jne    8a7b0 <<c30walk::ctor2c::W<false>>::finish_slow>
   8addb:	lea    (%rdi,%rax,8),%rax
   8addf:	add    $0x1040,%rax
   8ade5:	mov    0x40(%rdi,%rsi,8),%rdx
   8adea:	mov    (%rax),%rax
   8aded:	sub    %rax,%rdx
   8adf0:	cmp    $0x3c,%rdx
   8adf4:	jne    8a7b0 <<c30walk::ctor2c::W<false>>::finish_slow>
   8adfa:	and    $0x7fffffff,%ecx
   8ae00:	or     %esi,%ecx
   8ae02:	movq   (%rax),%xmm0
   8ae06:	movzwl (%rax),%edx
   8ae09:	movd   %edx,%xmm1
   8ae0d:	shl    $0x8,%edx
   8ae10:	or     %ecx,%edx
   8ae12:	mov    %edx,0x3c(%rax)
   8ae15:	movq   0x8(%rax),%xmm2
   8ae1a:	xor    %ecx,%ecx
   8ae1c:	pshuflw $0x0,%xmm0,%xmm3
   8ae21:	pinsrw $0x0,%ecx,%xmm0
   8ae26:	punpcklqdq %xmm2,%xmm0
   8ae2a:	pshufd $0x44,%xmm3,%xmm2
   8ae2f:	pand   -0x7ab57(%rip),%xmm2        # 102e0 <GCC_except_table992+0x41c>
   8ae37:	psubw  %xmm2,%xmm0
   8ae3b:	movdqu %xmm0,(%rax)
   8ae3f:	pshuflw $0x0,%xmm1,%xmm0
   8ae44:	pshufd $0x44,%xmm0,%xmm1
   8ae49:	movdqu 0x10(%rax),%xmm2
   8ae4e:	movdqu 0x20(%rax),%xmm3
   8ae53:	psubw  %xmm1,%xmm2
   8ae57:	movdqu %xmm2,0x10(%rax)
   8ae5c:	psubw  %xmm1,%xmm3
   8ae60:	movdqu %xmm3,0x20(%rax)
   8ae65:	movq   0x30(%rax),%xmm1
   8ae6a:	psubw  %xmm0,%xmm1
   8ae6e:	movq   %xmm1,0x30(%rax)
   8ae73:	movd   0x38(%rax),%xmm1
   8ae78:	psubw  %xmm0,%xmm1
   8ae7c:	movd   %xmm1,0x38(%rax)
   8ae81:	ret
