# <c30walk::ctor2c::W<false>>::refill
# size 0x120

target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

000000000008ae90 <<c30walk::ctor2c::W<false>>::refill>:
   8ae90:	push   %r15
   8ae92:	push   %r14
   8ae94:	push   %r12
   8ae96:	push   %rbx
   8ae97:	push   %rax
   8ae98:	mov    %rsi,%rbx
   8ae9b:	mov    %rdi,%r14
   8ae9e:	mov    0x40(%rdi,%rsi,8),%r15
   8aea3:	lea    (%rsi,%rsi,2),%rax
   8aea7:	lea    (%rdi,%rax,8),%r12
   8aeab:	add    $0x1040,%r12
   8aeb2:	sub    0x1040(%rdi,%rax,8),%r15
   8aeba:	je     8aec6 <<c30walk::ctor2c::W<false>>::refill+0x36>
   8aebc:	mov    %r14,%rdi
   8aebf:	mov    %ebx,%esi
   8aec1:	call   8adc0 <<c30walk::ctor2c::W<false>>::finish>
   8aec6:	shr    $1,%r15
   8aec9:	add    0x8(%r12),%r15d
   8aece:	mov    %r15d,0x8(%r12)
   8aed3:	movq   $0x0,(%r12)
   8aedb:	movq   $0x0,0x40(%r14,%rbx,8)
   8aee4:	movq   $0x0,0x840(%r14,%rbx,8)
   8aef0:	mov    0xc(%r12),%eax
   8aef5:	cmp    %eax,%r15d
   8aef8:	jae    8af92 <<c30walk::ctor2c::W<false>>::refill+0x102>
   8aefe:	mov    %r15d,%ecx
   8af01:	mov    $0x88888889,%edi
   8af06:	imul   %rcx,%rdi
   8af0a:	mov    %rdi,%rcx
   8af0d:	shr    $0x24,%rcx
   8af11:	mov    %ecx,%r9d
   8af14:	shl    $0x5,%r9d
   8af18:	sub    %ecx,%r9d
   8af1b:	sub    %ecx,%r9d
   8af1e:	lea    0x1e(%r9),%edx
   8af22:	cmp    %eax,%edx
   8af24:	cmovae %eax,%edx
   8af27:	shr    $0x2a,%rdi
   8af2b:	mov    0x10(%r14),%rsi
   8af2f:	cmp    %rsi,%rdi
   8af32:	jae    8afa4 <<c30walk::ctor2c::W<false>>::refill+0x114>
   8af34:	mov    %r15d,%esi
   8af37:	mov    %rcx,%r8
   8af3a:	shl    $0x6,%r8
   8af3e:	add    0x28(%r14),%r8
   8af42:	sub    %r9d,%esi
   8af45:	mov    0x8(%r14),%r9
   8af49:	mov    (%r9,%rdi,8),%rdi
   8af4d:	bt     %rcx,%rdi
   8af51:	jae    8af5e <<c30walk::ctor2c::W<false>>::refill+0xce>
   8af53:	mov    $0xfffff,%edi
   8af58:	and    0x3c(%r8),%edi
   8af5c:	jmp    8af65 <<c30walk::ctor2c::W<false>>::refill+0xd5>
   8af5e:	mov    0x284c(%r14),%edi
   8af65:	mov    %esi,%esi
   8af67:	lea    (%r8,%rsi,2),%rsi
   8af6b:	mov    %rsi,0x40(%r14,%rbx,8)
   8af70:	sub    %r15d,%edx
   8af73:	lea    (%rsi,%rdx,2),%rdx
   8af77:	mov    %rdx,0x840(%r14,%rbx,8)
   8af7f:	mov    %rsi,(%r12)
   8af83:	mov    %r15d,0x8(%r12)
   8af88:	mov    %ecx,0x10(%r12)
   8af8d:	mov    %edi,0x14(%r12)
   8af92:	cmp    %eax,%r15d
   8af95:	setb   %al
   8af98:	add    $0x8,%rsp
   8af9c:	pop    %rbx
   8af9d:	pop    %r12
   8af9f:	pop    %r14
   8afa1:	pop    %r15
   8afa3:	ret
   8afa4:	lea    0xb27c5(%rip),%rdx        # 13d770 <__frame_dummy_init_array_entry+0x290>
   8afab:	call   e6ac3 <core::panicking::panic_bounds_check>
