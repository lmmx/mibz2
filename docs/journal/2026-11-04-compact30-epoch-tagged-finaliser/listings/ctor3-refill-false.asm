# <c30walk::ctor3::W<false>>::refill
# size 0xed

target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

0000000000081150 <<c30walk::ctor3::W<false>>::refill>:
   81150:	push   %r15
   81152:	push   %r14
   81154:	push   %rbx
   81155:	mov    %rdi,%rbx
   81158:	lea    (%rsi,%rsi,4),%rax
   8115c:	lea    (%rdi,%rax,8),%r15
   81160:	add    $0x40,%r15
   81164:	mov    0x40(%rdi,%rax,8),%r14
   81169:	sub    0x50(%rdi,%rax,8),%r14
   8116e:	je     81178 <<c30walk::ctor3::W<false>>::refill+0x28>
   81170:	mov    %rbx,%rdi
   81173:	call   81090 <<c30walk::ctor3::W<false>>::finish>
   81178:	shr    $1,%r14
   8117b:	add    0x18(%r15),%r14d
   8117f:	mov    %r14d,0x18(%r15)
   81183:	xorps  %xmm0,%xmm0
   81186:	movups %xmm0,(%r15)
   8118a:	movq   $0x0,0x10(%r15)
   81192:	mov    0x1c(%r15),%eax
   81196:	cmp    %eax,%r14d
   81199:	jae    81225 <<c30walk::ctor3::W<false>>::refill+0xd5>
   8119f:	mov    %r14d,%ecx
   811a2:	mov    $0x88888889,%edi
   811a7:	imul   %rcx,%rdi
   811ab:	mov    %rdi,%rcx
   811ae:	shr    $0x24,%rcx
   811b2:	mov    %ecx,%r9d
   811b5:	shl    $0x5,%r9d
   811b9:	sub    %ecx,%r9d
   811bc:	sub    %ecx,%r9d
   811bf:	lea    0x1e(%r9),%edx
   811c3:	cmp    %eax,%edx
   811c5:	cmovae %eax,%edx
   811c8:	shr    $0x2a,%rdi
   811cc:	mov    0x10(%rbx),%rsi
   811d0:	cmp    %rsi,%rdi
   811d3:	jae    81231 <<c30walk::ctor3::W<false>>::refill+0xe1>
   811d5:	mov    %r14d,%esi
   811d8:	mov    %rcx,%r8
   811db:	shl    $0x6,%r8
   811df:	add    0x28(%rbx),%r8
   811e3:	sub    %r9d,%esi
   811e6:	mov    0x8(%rbx),%r9
   811ea:	mov    (%r9,%rdi,8),%rdi
   811ee:	bt     %rcx,%rdi
   811f2:	jae    811ff <<c30walk::ctor3::W<false>>::refill+0xaf>
   811f4:	mov    $0xfffff,%edi
   811f9:	and    0x3c(%r8),%edi
   811fd:	jmp    81205 <<c30walk::ctor3::W<false>>::refill+0xb5>
   811ff:	mov    0x284c(%rbx),%edi
   81205:	mov    %esi,%esi
   81207:	lea    (%r8,%rsi,2),%rsi
   8120b:	mov    %rsi,0x10(%r15)
   8120f:	mov    %rsi,(%r15)
   81212:	sub    %r14d,%edx
   81215:	lea    (%rsi,%rdx,2),%rdx
   81219:	mov    %rdx,0x8(%r15)
   8121d:	mov    %ecx,0x20(%r15)
   81221:	mov    %edi,0x24(%r15)
   81225:	cmp    %eax,%r14d
   81228:	setb   %al
   8122b:	pop    %rbx
   8122c:	pop    %r14
   8122e:	pop    %r15
   81230:	ret
   81231:	lea    0x96050(%rip),%rdx        # 117288 <__frame_dummy_init_array_entry+0x258>
   81238:	call   c1643 <core::panicking::panic_bounds_check>
