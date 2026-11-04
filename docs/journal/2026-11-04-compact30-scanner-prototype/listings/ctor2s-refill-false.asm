# <c30walk::ctor2s::W<false>>::refill
# size 0xec

target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

000000000008c100 <<c30walk::ctor2s::W<false>>::refill>:
   8c100:	push   %r15
   8c102:	push   %r14
   8c104:	push   %rbx
   8c105:	mov    %rdi,%rbx
   8c108:	lea    (%rsi,%rsi,4),%rax
   8c10c:	lea    (%rdi,%rax,8),%r15
   8c110:	add    $0x40,%r15
   8c114:	mov    0x40(%rdi,%rax,8),%r14
   8c119:	sub    0x50(%rdi,%rax,8),%r14
   8c11e:	je     8c128 <<c30walk::ctor2s::W<false>>::refill+0x28>
   8c120:	mov    %rbx,%rdi
   8c123:	call   8bb00 <<c30walk::ctor2s::W<false>>::finish>
   8c128:	shr    $1,%r14
   8c12b:	add    0x18(%r15),%r14d
   8c12f:	mov    %r14d,0x18(%r15)
   8c133:	xorps  %xmm0,%xmm0
   8c136:	movups %xmm0,(%r15)
   8c13a:	movq   $0x0,0x10(%r15)
   8c142:	mov    0x1c(%r15),%eax
   8c146:	cmp    %eax,%r14d
   8c149:	jae    8c1d4 <<c30walk::ctor2s::W<false>>::refill+0xd4>
   8c14f:	mov    %r14d,%ecx
   8c152:	mov    $0x88888889,%edi
   8c157:	imul   %rcx,%rdi
   8c15b:	mov    %rdi,%rcx
   8c15e:	shr    $0x24,%rcx
   8c162:	mov    %ecx,%r9d
   8c165:	shl    $0x5,%r9d
   8c169:	sub    %ecx,%r9d
   8c16c:	sub    %ecx,%r9d
   8c16f:	lea    0x1e(%r9),%edx
   8c173:	cmp    %eax,%edx
   8c175:	cmovae %eax,%edx
   8c178:	shr    $0x2a,%rdi
   8c17c:	mov    0x10(%rbx),%rsi
   8c180:	cmp    %rsi,%rdi
   8c183:	jae    8c1e0 <<c30walk::ctor2s::W<false>>::refill+0xe0>
   8c185:	mov    %r14d,%esi
   8c188:	mov    %rcx,%r8
   8c18b:	shl    $0x6,%r8
   8c18f:	add    0x28(%rbx),%r8
   8c193:	sub    %r9d,%esi
   8c196:	mov    0x8(%rbx),%r9
   8c19a:	mov    (%r9,%rdi,8),%rdi
   8c19e:	bt     %rcx,%rdi
   8c1a2:	jae    8c1af <<c30walk::ctor2s::W<false>>::refill+0xaf>
   8c1a4:	mov    $0xfffff,%edi
   8c1a9:	and    0x3c(%r8),%edi
   8c1ad:	jmp    8c1b4 <<c30walk::ctor2s::W<false>>::refill+0xb4>
   8c1af:	mov    $0xffffffff,%edi
   8c1b4:	mov    %esi,%esi
   8c1b6:	lea    (%r8,%rsi,2),%rsi
   8c1ba:	mov    %rsi,0x10(%r15)
   8c1be:	mov    %rsi,(%r15)
   8c1c1:	sub    %r14d,%edx
   8c1c4:	lea    (%rsi,%rdx,2),%rdx
   8c1c8:	mov    %rdx,0x8(%r15)
   8c1cc:	mov    %ecx,0x20(%r15)
   8c1d0:	mov    %edi,0x24(%r15)
   8c1d4:	cmp    %eax,%r14d
   8c1d7:	setb   %al
   8c1da:	pop    %rbx
   8c1db:	pop    %r14
   8c1dd:	pop    %r15
   8c1df:	ret
   8c1e0:	lea    0xa2b99(%rip),%rdx        # 12ed80 <__frame_dummy_init_array_entry+0x270>
   8c1e7:	call   d8123 <core::panicking::panic_bounds_check>
