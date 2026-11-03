# <c30walk::ctor::W<false>>::complete_checked
# size 0x3be

/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/c30walk-e003a32:     file format elf64-x86-64


Disassembly of section .text:

000000000004d8f0 <<c30walk::ctor::W<false>>::complete_checked>:
   4d8f0:	push   %rbp
   4d8f1:	push   %r15
   4d8f3:	push   %r14
   4d8f5:	push   %r13
   4d8f7:	push   %r12
   4d8f9:	push   %rbx
   4d8fa:	push   %rax
   4d8fb:	mov    %esi,%r8d
   4d8fe:	mov    0x10(%rdi),%rax
   4d902:	mov    %rax,(%rsp)
   4d906:	mov    0x8(%rdi),%rsi
   4d90a:	mov    0x830(%rdi),%rax
   4d911:	cmp    $0x1d,%rax
   4d915:	ja     4dc5c <<c30walk::ctor::W<false>>::complete_checked+0x36c>
   4d91b:	mov    (%rdi),%r10
   4d91e:	shl    $0x6,%rax
   4d922:	lea    -0x3e269(%rip),%r11        # f6c0 <c30walk::ctor::PATTERNS>
   4d929:	add    %rax,%r11
   4d92c:	mov    0x20(%rdi),%rax
   4d930:	movzbl %r8b,%ebx
   4d934:	mov    0x838(%rdi),%r14d
   4d93b:	mov    %r14d,%ebp
   4d93e:	shl    $0x8,%ebp
   4d941:	or     %ebx,%ebp
   4d943:	mov    %rdx,%r8
   4d946:	shl    $0x6,%r8
   4d94a:	lea    (%r8,%rax,1),%r15
   4d94e:	add    $0x3c,%r15
   4d952:	xor    %r12d,%r12d
   4d955:	jmp    4d9b2 <<c30walk::ctor::W<false>>::complete_checked+0xc2>
   4d957:	nopw   0x0(%rax,%rax,1)
   4d960:	movups 0x2c(%r11),%xmm0
   4d965:	movups %xmm0,-0x10(%r15,%r12,2)
   4d96b:	movaps (%r11),%xmm0
   4d96f:	movaps 0x10(%r11),%xmm1
   4d974:	movaps 0x20(%r11),%xmm2
   4d979:	movaps %xmm2,-0x1c(%r15,%r12,2)
   4d97f:	movaps %xmm1,-0x2c(%r15,%r12,2)
   4d985:	movaps %xmm0,-0x3c(%r15,%r12,2)
   4d98b:	mov    %ebp,(%r15,%r12,2)
   4d98f:	lea    (%r14,%r12,1),%eax
   4d993:	add    $0x20,%eax
   4d996:	mov    %eax,0x838(%rdi)
   4d99c:	add    $0x2000,%ebp
   4d9a2:	add    $0x20,%r12
   4d9a6:	inc    %rdx
   4d9a9:	dec    %rcx
   4d9ac:	je     4dc3b <<c30walk::ctor::W<false>>::complete_checked+0x34b>
   4d9b2:	mov    %rdx,%r8
   4d9b5:	shr    $0x6,%r8
   4d9b9:	cmp    (%rsp),%r8
   4d9bd:	jae    4dc87 <<c30walk::ctor::W<false>>::complete_checked+0x397>
   4d9c3:	mov    (%rsi,%r8,8),%rax
   4d9c7:	bt     %rdx,%rax
   4d9cb:	jae    4d960 <<c30walk::ctor::W<false>>::complete_checked+0x70>
   4d9cd:	mov    (%r15,%r12,2),%eax
   4d9d1:	and    $0xfffff,%eax
   4d9d6:	mov    0x10(%r10),%r8
   4d9da:	cmp    %rax,%r8
   4d9dd:	jbe    4dc4a <<c30walk::ctor::W<false>>::complete_checked+0x35a>
   4d9e3:	mov    0x8(%r10),%r8
   4d9e7:	shl    $0x7,%eax
   4d9ea:	mov    %ebp,(%r8,%rax,1)
   4d9ee:	movzwl 0x2(%r11),%r9d
   4d9f3:	lea    (%r14,%r12,1),%r13d
   4d9f7:	add    %r13d,%r9d
   4d9fa:	shl    $0x8,%r9d
   4d9fe:	or     %ebx,%r9d
   4da01:	mov    %r9d,0x4(%r8,%rax,1)
   4da06:	movzwl 0x4(%r11),%r9d
   4da0b:	add    %r13d,%r9d
   4da0e:	shl    $0x8,%r9d
   4da12:	or     %ebx,%r9d
   4da15:	mov    %r9d,0x8(%r8,%rax,1)
   4da1a:	movzwl 0x6(%r11),%r9d
   4da1f:	add    %r13d,%r9d
   4da22:	shl    $0x8,%r9d
   4da26:	or     %ebx,%r9d
   4da29:	mov    %r9d,0xc(%r8,%rax,1)
   4da2e:	movzwl 0x8(%r11),%r9d
   4da33:	add    %r13d,%r9d
   4da36:	shl    $0x8,%r9d
   4da3a:	or     %ebx,%r9d
   4da3d:	mov    %r9d,0x10(%r8,%rax,1)
   4da42:	movzwl 0xa(%r11),%r9d
   4da47:	add    %r13d,%r9d
   4da4a:	shl    $0x8,%r9d
   4da4e:	or     %ebx,%r9d
   4da51:	mov    %r9d,0x14(%r8,%rax,1)
   4da56:	movzwl 0xc(%r11),%r9d
   4da5b:	add    %r13d,%r9d
   4da5e:	shl    $0x8,%r9d
   4da62:	or     %ebx,%r9d
   4da65:	mov    %r9d,0x18(%r8,%rax,1)
   4da6a:	movzwl 0xe(%r11),%r9d
   4da6f:	add    %r13d,%r9d
   4da72:	shl    $0x8,%r9d
   4da76:	or     %ebx,%r9d
   4da79:	mov    %r9d,0x1c(%r8,%rax,1)
   4da7e:	movzwl 0x10(%r11),%r9d
   4da83:	add    %r13d,%r9d
   4da86:	shl    $0x8,%r9d
   4da8a:	or     %ebx,%r9d
   4da8d:	mov    %r9d,0x20(%r8,%rax,1)
   4da92:	movzwl 0x12(%r11),%r9d
   4da97:	add    %r13d,%r9d
   4da9a:	shl    $0x8,%r9d
   4da9e:	or     %ebx,%r9d
   4daa1:	mov    %r9d,0x24(%r8,%rax,1)
   4daa6:	movzwl 0x14(%r11),%r9d
   4daab:	add    %r13d,%r9d
   4daae:	shl    $0x8,%r9d
   4dab2:	or     %ebx,%r9d
   4dab5:	mov    %r9d,0x28(%r8,%rax,1)
   4daba:	movzwl 0x16(%r11),%r9d
   4dabf:	add    %r13d,%r9d
   4dac2:	shl    $0x8,%r9d
   4dac6:	or     %ebx,%r9d
   4dac9:	mov    %r9d,0x2c(%r8,%rax,1)
   4dace:	movzwl 0x18(%r11),%r9d
   4dad3:	add    %r13d,%r9d
   4dad6:	shl    $0x8,%r9d
   4dada:	or     %ebx,%r9d
   4dadd:	mov    %r9d,0x30(%r8,%rax,1)
   4dae2:	movzwl 0x1a(%r11),%r9d
   4dae7:	add    %r13d,%r9d
   4daea:	shl    $0x8,%r9d
   4daee:	or     %ebx,%r9d
   4daf1:	mov    %r9d,0x34(%r8,%rax,1)
   4daf6:	movzwl 0x1c(%r11),%r9d
   4dafb:	add    %r13d,%r9d
   4dafe:	shl    $0x8,%r9d
   4db02:	or     %ebx,%r9d
   4db05:	mov    %r9d,0x38(%r8,%rax,1)
   4db0a:	movzwl 0x1e(%r11),%r9d
   4db0f:	add    %r13d,%r9d
   4db12:	shl    $0x8,%r9d
   4db16:	or     %ebx,%r9d
   4db19:	mov    %r9d,0x3c(%r8,%rax,1)
   4db1e:	movzwl 0x20(%r11),%r9d
   4db23:	add    %r13d,%r9d
   4db26:	shl    $0x8,%r9d
   4db2a:	or     %ebx,%r9d
   4db2d:	mov    %r9d,0x40(%r8,%rax,1)
   4db32:	movzwl 0x22(%r11),%r9d
   4db37:	add    %r13d,%r9d
   4db3a:	shl    $0x8,%r9d
   4db3e:	or     %ebx,%r9d
   4db41:	mov    %r9d,0x44(%r8,%rax,1)
   4db46:	movzwl 0x24(%r11),%r9d
   4db4b:	add    %r13d,%r9d
   4db4e:	shl    $0x8,%r9d
   4db52:	or     %ebx,%r9d
   4db55:	mov    %r9d,0x48(%r8,%rax,1)
   4db5a:	movzwl 0x26(%r11),%r9d
   4db5f:	add    %r13d,%r9d
   4db62:	shl    $0x8,%r9d
   4db66:	or     %ebx,%r9d
   4db69:	mov    %r9d,0x4c(%r8,%rax,1)
   4db6e:	movzwl 0x28(%r11),%r9d
   4db73:	add    %r13d,%r9d
   4db76:	shl    $0x8,%r9d
   4db7a:	or     %ebx,%r9d
   4db7d:	mov    %r9d,0x50(%r8,%rax,1)
   4db82:	movzwl 0x2a(%r11),%r9d
   4db87:	add    %r13d,%r9d
   4db8a:	shl    $0x8,%r9d
   4db8e:	or     %ebx,%r9d
   4db91:	mov    %r9d,0x54(%r8,%rax,1)
   4db96:	movzwl 0x2c(%r11),%r9d
   4db9b:	add    %r13d,%r9d
   4db9e:	shl    $0x8,%r9d
   4dba2:	or     %ebx,%r9d
   4dba5:	mov    %r9d,0x58(%r8,%rax,1)
   4dbaa:	movzwl 0x2e(%r11),%r9d
   4dbaf:	add    %r13d,%r9d
   4dbb2:	shl    $0x8,%r9d
   4dbb6:	or     %ebx,%r9d
   4dbb9:	mov    %r9d,0x5c(%r8,%rax,1)
   4dbbe:	movzwl 0x30(%r11),%r9d
   4dbc3:	add    %r13d,%r9d
   4dbc6:	shl    $0x8,%r9d
   4dbca:	or     %ebx,%r9d
   4dbcd:	mov    %r9d,0x60(%r8,%rax,1)
   4dbd2:	movzwl 0x32(%r11),%r9d
   4dbd7:	add    %r13d,%r9d
   4dbda:	shl    $0x8,%r9d
   4dbde:	or     %ebx,%r9d
   4dbe1:	mov    %r9d,0x64(%r8,%rax,1)
   4dbe6:	movzwl 0x34(%r11),%r9d
   4dbeb:	add    %r13d,%r9d
   4dbee:	shl    $0x8,%r9d
   4dbf2:	or     %ebx,%r9d
   4dbf5:	mov    %r9d,0x68(%r8,%rax,1)
   4dbfa:	movzwl 0x36(%r11),%r9d
   4dbff:	add    %r13d,%r9d
   4dc02:	shl    $0x8,%r9d
   4dc06:	or     %ebx,%r9d
   4dc09:	mov    %r9d,0x6c(%r8,%rax,1)
   4dc0e:	movzwl 0x38(%r11),%r9d
   4dc13:	add    %r13d,%r9d
   4dc16:	shl    $0x8,%r9d
   4dc1a:	or     %ebx,%r9d
   4dc1d:	mov    %r9d,0x70(%r8,%rax,1)
   4dc22:	movzwl 0x3a(%r11),%r9d
   4dc27:	add    %r13d,%r9d
   4dc2a:	shl    $0x8,%r9d
   4dc2e:	or     %ebx,%r9d
   4dc31:	mov    %r9d,0x74(%r8,%rax,1)
   4dc36:	jmp    4d98f <<c30walk::ctor::W<false>>::complete_checked+0x9f>
   4dc3b:	add    $0x8,%rsp
   4dc3f:	pop    %rbx
   4dc40:	pop    %r12
   4dc42:	pop    %r13
   4dc44:	pop    %r14
   4dc46:	pop    %r15
   4dc48:	pop    %rbp
   4dc49:	ret
   4dc4a:	lea    0x6909f(%rip),%rdx        # b6cf0 <__frame_dummy_init_array_entry+0x13a0>
   4dc51:	mov    %rax,%rdi
   4dc54:	mov    %r8,%rsi
   4dc57:	call   6292b <core::panicking::panic_bounds_check>
   4dc5c:	mov    %rdx,%r8
   4dc5f:	shr    $0x6,%r8
   4dc63:	cmp    (%rsp),%r8
   4dc67:	jae    4dc87 <<c30walk::ctor::W<false>>::complete_checked+0x397>
   4dc69:	mov    (%rsi,%r8,8),%rcx
   4dc6d:	bt     %rdx,%rcx
   4dc71:	jb     4dc9a <<c30walk::ctor::W<false>>::complete_checked+0x3aa>
   4dc73:	lea    0x68fb6(%rip),%rdx        # b6c30 <__frame_dummy_init_array_entry+0x12e0>
   4dc7a:	mov    $0x1e,%esi
   4dc7f:	mov    %rax,%rdi
   4dc82:	call   6292b <core::panicking::panic_bounds_check>
   4dc87:	lea    0x67ec2(%rip),%rdx        # b5b50 <__frame_dummy_init_array_entry+0x200>
   4dc8e:	mov    %r8,%rdi
   4dc91:	mov    (%rsp),%rsi
   4dc95:	call   6292b <core::panicking::panic_bounds_check>
   4dc9a:	lea    0x69007(%rip),%rdx        # b6ca8 <__frame_dummy_init_array_entry+0x1358>
   4dca1:	mov    $0x1e,%esi
   4dca6:	mov    %rax,%rdi
   4dca9:	call   6292b <core::panicking::panic_bounds_check>
