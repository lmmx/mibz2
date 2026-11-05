   24ab0:	push   %rbp
   24ab1:	push   %r15
   24ab3:	push   %r14
   24ab5:	push   %r13
   24ab7:	push   %r12
   24ab9:	push   %rbx
   24aba:	sub    $0x5a8,%rsp
   24ac1:	mov    %rdx,%r14
   24ac4:	mov    %rsi,%rbx
   24ac7:	test   %rdi,%rdi
   24aca:	je     24c94 <handoff::old_give+0x1e4>
   24ad0:	cmp    $0x1,%rdi
   24ad4:	jne    250a8 <handoff::old_give+0x5f8>
   24ada:	mov    %r14,0x8(%rsp)
   24adf:	mov    0x80(%rbx),%rbp
   24ae6:	mov    0x88(%rbx),%r12
   24aed:	test   $0x1,%bpl
   24af1:	jne    254c9 <handoff::old_give+0xa19>
   24af7:	xor    %r14d,%r14d
   24afa:	xor    %r15d,%r15d
   24afd:	jmp    24b21 <handoff::old_give+0x71>
   24aff:	nop
   24b00:	call   *0x4c24a(%rip)        # 70d50 <_DYNAMIC+0x2c8>
   24b06:	inc    %r14d
   24b09:	mov    0x80(%rbx),%rbp
   24b10:	mov    0x88(%rbx),%r12
   24b17:	test   $0x1,%bpl
   24b1b:	jne    2542b <handoff::old_give+0x97b>
   24b21:	mov    %ebp,%r13d
   24b24:	shr    $1,%r13d
   24b27:	and    $0x1f,%r13d
   24b2b:	cmp    $0x1f,%r13d
   24b2f:	jne    24b80 <handoff::old_give+0xd0>
   24b31:	cmp    $0x7,%r14d
   24b35:	jae    24b00 <handoff::old_give+0x50>
   24b37:	test   %r14d,%r14d
   24b3a:	je     24b06 <handoff::old_give+0x56>
   24b3c:	mov    %r14d,%ecx
   24b3f:	imul   %r14d,%ecx
   24b43:	mov    %ecx,%eax
   24b45:	and    $0x5,%eax
   24b48:	cmp    $0x8,%ecx
   24b4b:	jb     24b70 <handoff::old_give+0xc0>
   24b4d:	and    $0x38,%ecx
   24b50:	pause
   24b52:	pause
   24b54:	pause
   24b56:	pause
   24b58:	pause
   24b5a:	pause
   24b5c:	pause
   24b5e:	pause
   24b60:	add    $0xfffffff8,%ecx
   24b63:	jne    24b50 <handoff::old_give+0xa0>
   24b65:	test   %eax,%eax
   24b67:	je     24b06 <handoff::old_give+0x56>
   24b69:	nopl   0x0(%rax)
   24b70:	pause
   24b72:	dec    %eax
   24b74:	jne    24b70 <handoff::old_give+0xc0>
   24b76:	jmp    24b06 <handoff::old_give+0x56>
   24b78:	nopl   0x0(%rax,%rax,1)
   24b80:	mov    %r13,%rax
   24b83:	xor    $0x1e,%rax
   24b87:	or     %r15,%rax
   24b8a:	jne    24bae <handoff::old_give+0xfe>
   24b8c:	call   *0x4c1fe(%rip)        # 70d90 <_DYNAMIC+0x308>
   24b92:	mov    $0x26c8,%edi
   24b97:	mov    $0x8,%esi
   24b9c:	call   *0x4c2e6(%rip)        # 70e88 <_DYNAMIC+0x400>
   24ba2:	mov    %rax,%r15
   24ba5:	test   %rax,%rax
   24ba8:	je     25593 <handoff::old_give+0xae3>
   24bae:	test   %r12,%r12
   24bb1:	je     24c1b <handoff::old_give+0x16b>
   24bb3:	lea    0x2(%rbp),%rcx
   24bb7:	mov    %rbp,%rax
   24bba:	lock cmpxchg %rcx,0x80(%rbx)
   24bc3:	je     254dd <handoff::old_give+0xa2d>
   24bc9:	cmp    $0x6,%r14d
   24bcd:	mov    $0x6,%ecx
   24bd2:	cmovb  %r14d,%ecx
   24bd6:	test   %r14d,%r14d
   24bd9:	je     24c69 <handoff::old_give+0x1b9>
   24bdf:	imul   %ecx,%ecx
   24be2:	mov    %ecx,%eax
   24be4:	and    $0x5,%eax
   24be7:	cmp    $0x8,%ecx
   24bea:	jb     24c10 <handoff::old_give+0x160>
   24bec:	and    $0x38,%ecx
   24bef:	nop
   24bf0:	pause
   24bf2:	pause
   24bf4:	pause
   24bf6:	pause
   24bf8:	pause
   24bfa:	pause
   24bfc:	pause
   24bfe:	pause
   24c00:	add    $0xfffffff8,%ecx
   24c03:	jne    24bf0 <handoff::old_give+0x140>
   24c05:	test   %eax,%eax
   24c07:	je     24b06 <handoff::old_give+0x56>
   24c0d:	nopl   (%rax)
   24c10:	pause
   24c12:	dec    %eax
   24c14:	jne    24c10 <handoff::old_give+0x160>
   24c16:	jmp    24b06 <handoff::old_give+0x56>
   24c1b:	call   *0x4c16f(%rip)        # 70d90 <_DYNAMIC+0x308>
   24c21:	mov    $0x26c8,%edi
   24c26:	mov    $0x8,%esi
   24c2b:	call   *0x4c257(%rip)        # 70e88 <_DYNAMIC+0x400>
   24c31:	test   %rax,%rax
   24c34:	je     255ab <handoff::old_give+0xafb>
   24c3a:	mov    %rax,%r12
   24c3d:	xor    %eax,%eax
   24c3f:	lock cmpxchg %r12,0x88(%rbx)
   24c48:	jne    24c74 <handoff::old_give+0x1c4>
   24c4a:	mov    %r12,0x8(%rbx)
   24c4e:	lea    0x2(%rbp),%rcx
   24c52:	mov    %rbp,%rax
   24c55:	lock cmpxchg %rcx,0x80(%rbx)
   24c5e:	jne    24bc9 <handoff::old_give+0x119>
   24c64:	jmp    254dd <handoff::old_give+0xa2d>
   24c69:	mov    $0x1,%r14d
   24c6f:	jmp    24b09 <handoff::old_give+0x59>
   24c74:	test   %r15,%r15
   24c77:	je     24c8c <handoff::old_give+0x1dc>
   24c79:	mov    $0x26c8,%esi
   24c7e:	mov    $0x8,%edx
   24c83:	mov    %r15,%rdi
   24c86:	call   *0x4c054(%rip)        # 70ce0 <_DYNAMIC+0x258>
   24c8c:	mov    %r12,%r15
   24c8f:	jmp    24b09 <handoff::old_give+0x59>
   24c94:	movl   $0xffffffff,0x470(%rsp)
   24c9f:	xorps  %xmm0,%xmm0
   24ca2:	movaps %xmm0,0x70(%rsp)
   24ca7:	movaps %xmm0,0x60(%rsp)
   24cac:	movq   $0x0,0x80(%rsp)
   24cb8:	mov    0x80(%rbx),%rbp
   24cbf:	mov    0x190(%rbx),%rax
   24cc6:	mov    $0x1,%r13d
   24ccc:	test   %rbp,%rax
   24ccf:	jne    254b8 <handoff::old_give+0xa08>
   24cd5:	mov    %r14,0x8(%rsp)
   24cda:	mov    %fs:0x0,%r15
   24ce3:	lea    -0x20(%r15),%r15
   24cea:	xor    %r12d,%r12d
   24ced:	jmp    24d1a <handoff::old_give+0x26a>
   24cef:	call   *0x4c05b(%rip)        # 70d50 <_DYNAMIC+0x2c8>
   24cf5:	data16 cs nopw 0x0(%rax,%rax,1)
   24d00:	inc    %r12d
   24d03:	mov    0x80(%rbx),%rbp
   24d0a:	mov    0x190(%rbx),%rax
   24d11:	test   %rbp,%rax
   24d14:	jne    254b8 <handoff::old_give+0xa08>
   24d1a:	dec    %rax
   24d1d:	and    %rbp,%rax
   24d20:	xor    %edx,%edx
   24d22:	sub    0x188(%rbx),%rdx
   24d29:	mov    0x198(%rbx),%r14
   24d30:	lea    (%rax,%rax,4),%rcx
   24d34:	shl    $0x6,%rcx
   24d38:	mov    (%r14,%rcx,1),%rsi
   24d3c:	cmp    %rsi,%rbp
   24d3f:	jne    24d70 <handoff::old_give+0x2c0>
   24d41:	inc    %rax
   24d44:	cmp    0x180(%rbx),%rax
   24d4b:	jae    24e00 <handoff::old_give+0x350>
   24d51:	lea    0x1(%rbp),%rdx
   24d55:	mov    %rbp,%rax
   24d58:	lock cmpxchg %rdx,0x80(%rbx)
   24d61:	jne    24e1c <handoff::old_give+0x36c>
   24d67:	jmp    25551 <handoff::old_give+0xaa1>
   24d6c:	nopl   0x0(%rax)
   24d70:	add    0x188(%rbx),%rsi
   24d77:	lea    0x1(%rbp),%rax
   24d7b:	cmp    %rax,%rsi
   24d7e:	jne    24ea0 <handoff::old_give+0x3f0>
   24d84:	lock orl $0x0,-0x40(%rsp)
   24d8a:	mov    (%rbx),%rax
   24d8d:	add    0x188(%rbx),%rax
   24d94:	cmp    %rbp,%rax
   24d97:	je     24efb <handoff::old_give+0x44b>
   24d9d:	cmp    $0x6,%r12d
   24da1:	mov    $0x6,%ecx
   24da6:	cmovb  %r12d,%ecx
   24daa:	test   %r12d,%r12d
   24dad:	je     24e70 <handoff::old_give+0x3c0>
   24db3:	imul   %ecx,%ecx
   24db6:	mov    %ecx,%eax
   24db8:	and    $0x5,%eax
   24dbb:	cmp    $0x8,%ecx
   24dbe:	jb     24df0 <handoff::old_give+0x340>
   24dc0:	and    $0x38,%ecx
   24dc3:	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   24dd0:	pause
   24dd2:	pause
   24dd4:	pause
   24dd6:	pause
   24dd8:	pause
   24dda:	pause
   24ddc:	pause
   24dde:	pause
   24de0:	add    $0xfffffff8,%ecx
   24de3:	jne    24dd0 <handoff::old_give+0x320>
   24de5:	test   %eax,%eax
   24de7:	je     24d00 <handoff::old_give+0x250>
   24ded:	nopl   (%rax)
   24df0:	pause
   24df2:	dec    %eax
   24df4:	jne    24df0 <handoff::old_give+0x340>
   24df6:	jmp    24d00 <handoff::old_give+0x250>
   24dfb:	nopl   0x0(%rax,%rax,1)
   24e00:	and    %rbp,%rdx
   24e03:	add    0x188(%rbx),%rdx
   24e0a:	mov    %rbp,%rax
   24e0d:	lock cmpxchg %rdx,0x80(%rbx)
   24e16:	je     25551 <handoff::old_give+0xaa1>
   24e1c:	cmp    $0x6,%r12d
   24e20:	mov    $0x6,%ecx
   24e25:	cmovb  %r12d,%ecx
   24e29:	test   %r12d,%r12d
   24e2c:	je     24e70 <handoff::old_give+0x3c0>
   24e2e:	imul   %ecx,%ecx
   24e31:	mov    %ecx,%eax
   24e33:	and    $0x5,%eax
   24e36:	cmp    $0x8,%ecx
   24e39:	jb     24e60 <handoff::old_give+0x3b0>
   24e3b:	and    $0x38,%ecx
   24e3e:	xchg   %ax,%ax
   24e40:	pause
   24e42:	pause
   24e44:	pause
   24e46:	pause
   24e48:	pause
   24e4a:	pause
   24e4c:	pause
   24e4e:	pause
   24e50:	add    $0xfffffff8,%ecx
   24e53:	jne    24e40 <handoff::old_give+0x390>
   24e55:	test   %eax,%eax
   24e57:	je     24d00 <handoff::old_give+0x250>
   24e5d:	nopl   (%rax)
   24e60:	pause
   24e62:	dec    %eax
   24e64:	jne    24e60 <handoff::old_give+0x3b0>
   24e66:	jmp    24d00 <handoff::old_give+0x250>
   24e6b:	nopl   0x0(%rax,%rax,1)
   24e70:	mov    $0x1,%r12d
   24e76:	mov    0x80(%rbx),%rbp
   24e7d:	mov    0x190(%rbx),%rax
   24e84:	test   %rbp,%rax
   24e87:	je     24d1a <handoff::old_give+0x26a>
   24e8d:	jmp    254b8 <handoff::old_give+0xa08>
   24e92:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   24ea0:	cmp    $0x7,%r12d
   24ea4:	jae    24cef <handoff::old_give+0x23f>
   24eaa:	test   %r12d,%r12d
   24ead:	je     24d00 <handoff::old_give+0x250>
   24eb3:	mov    %r12d,%ecx
   24eb6:	imul   %r12d,%ecx
   24eba:	mov    %ecx,%eax
   24ebc:	and    $0x5,%eax
   24ebf:	cmp    $0x8,%ecx
   24ec2:	jb     24ef0 <handoff::old_give+0x440>
   24ec4:	and    $0x38,%ecx
   24ec7:	nopw   0x0(%rax,%rax,1)
   24ed0:	pause
   24ed2:	pause
   24ed4:	pause
   24ed6:	pause
   24ed8:	pause
   24eda:	pause
   24edc:	pause
   24ede:	pause
   24ee0:	add    $0xfffffff8,%ecx
   24ee3:	jne    24ed0 <handoff::old_give+0x420>
   24ee5:	test   %eax,%eax
   24ee7:	je     24d00 <handoff::old_give+0x250>
   24eed:	nopl   (%rax)
   24ef0:	pause
   24ef2:	dec    %eax
   24ef4:	jne    24ef0 <handoff::old_give+0x440>
   24ef6:	jmp    24d00 <handoff::old_give+0x250>
   24efb:	mov    0x470(%rsp),%ebp
   24f02:	cmp    $0xffffffff,%ebp
   24f05:	je     24f29 <handoff::old_give+0x479>
   24f07:	mov    0x468(%rsp),%r14
   24f0f:	call   *0x4beab(%rip)        # 70dc0 <_DYNAMIC+0x338>
   24f15:	cmp    %r14,%rax
   24f18:	jne    24f23 <handoff::old_give+0x473>
   24f1a:	cmp    %ebp,%edx
   24f1c:	jb     24f29 <handoff::old_give+0x479>
   24f1e:	jmp    255a3 <handoff::old_give+0xaf3>
   24f23:	jge    255a3 <handoff::old_give+0xaf3>
   24f29:	lea    0x60(%rsp),%rax
   24f2e:	mov    %rax,0x328(%rsp)
   24f36:	mov    %rbx,0x330(%rsp)
   24f3e:	lea    0x468(%rsp),%rax
   24f46:	mov    %rax,0x338(%rsp)
   24f4e:	mov    $0xffffffffffffffe0,%rax
   24f55:	cmpb   $0x1,%fs:0x8(%rax)
   24f5a:	mov    %r15,%rbp
   24f5d:	jne    25012 <handoff::old_give+0x562>
   24f63:	mov    0x0(%rbp),%r12
   24f67:	movq   $0x0,0x0(%rbp)
   24f6f:	test   %r12,%r12
   24f72:	je     25044 <handoff::old_give+0x594>
   24f78:	mov    %r12,0x10(%rsp)
   24f7d:	movq   $0x0,0x18(%r12)
   24f86:	movq   $0x0,0x20(%r12)
   24f8f:	movq   $0x0,0x328(%rsp)
   24f9b:	lea    0x60(%rsp),%rax
   24fa0:	mov    %rax,0x1c8(%rsp)
   24fa8:	lea    0x330(%rsp),%rax
   24fb0:	movups (%rax),%xmm0
   24fb3:	lea    0x1d0(%rsp),%rax
   24fbb:	movups %xmm0,(%rax)
   24fbe:	lea    0x1c8(%rsp),%rdi
   24fc6:	mov    %r12,%rsi
   24fc9:	call   202d0 <<std::sync::mpmc::array::Channel<(u64, [u64; 38])>>::send::{closure#0}>
   24fce:	mov    0x0(%rbp),%rax
   24fd2:	mov    %rax,0x1c8(%rsp)
   24fda:	mov    %r12,0x0(%rbp)
   24fde:	test   %rax,%rax
   24fe1:	je     24ff6 <handoff::old_give+0x546>
   24fe3:	lock decq (%rax)
   24fe7:	jne    24ff6 <handoff::old_give+0x546>
   24fe9:	lea    0x1c8(%rsp),%rdi
   24ff1:	call   28130 <<alloc::sync::Arc<std::sync::mpmc::context::Inner>>::drop_slow>
   24ff6:	mov    0x80(%rbx),%rbp
   24ffd:	mov    0x190(%rbx),%rax
   25004:	test   %rbp,%rax
   25007:	je     24cea <handoff::old_give+0x23a>
   2500d:	jmp    254b8 <handoff::old_give+0xa08>
   25012:	mov    %fs:0x0,%rdi
   2501b:	lea    -0x20(%rdi),%rdi
   25022:	xor    %esi,%esi
   25024:	call   1ad90 <<std::sys::thread_local::native::lazy::Storage<core::cell::Cell<core::option::Option<std::sync::mpmc::context::Context>>, ()>>::get_or_init_slow::<<std::sync::mpmc::context::Context>::with::CONTEXT::__rust_std_internal_init_fn>>
   25029:	mov    %rax,%rbp
   2502c:	test   %rax,%rax
   2502f:	jne    24f63 <handoff::old_give+0x4b3>
   25035:	lea    0x328(%rsp),%rdi
   2503d:	call   1dee0 <<std::sync::mpmc::context::Context>::with::<<std::sync::mpmc::array::Channel<(u64, [u64; 38])>>::send::{closure#0}, ()>::{closure#2}>
   25042:	jmp    24ff6 <handoff::old_give+0x546>
   25044:	call   *0x4bc76(%rip)        # 70cc0 <_DYNAMIC+0x238>
   2504a:	mov    %rax,%r12
   2504d:	mov    %rax,0x10(%rsp)
   25052:	movq   $0x0,0x328(%rsp)
   2505e:	lea    0x60(%rsp),%rax
   25063:	mov    %rax,0x1c8(%rsp)
   2506b:	mov    %rbx,0x1d0(%rsp)
   25073:	lea    0x468(%rsp),%rax
   2507b:	mov    %rax,0x1d8(%rsp)
   25083:	lea    0x1c8(%rsp),%rdi
   2508b:	mov    %r12,%rsi
   2508e:	call   202d0 <<std::sync::mpmc::array::Channel<(u64, [u64; 38])>>::send::{closure#0}>
   25093:	lock decq (%r12)
   25098:	jne    24ff6 <handoff::old_give+0x546>
   2509e:	lea    0x10(%rsp),%rdi
   250a3:	jmp    24ff1 <handoff::old_give+0x541>
   250a8:	movl   $0xffffffff,0x58(%rsp)
   250b0:	xorps  %xmm0,%xmm0
   250b3:	movaps %xmm0,0x20(%rsp)
   250b8:	movaps %xmm0,0x10(%rsp)
   250bd:	movq   $0x0,0x30(%rsp)
   250c6:	mov    $0x1,%ecx
   250cb:	xor    %eax,%eax
   250cd:	lock cmpxchg %ecx,(%rbx)
   250d1:	jne    255d9 <handoff::old_give+0xb29>
   250d7:	mov    0x4bc12(%rip),%rax        # 70cf0 <_DYNAMIC+0x268>
   250de:	mov    (%rax),%rax
   250e1:	shl    $1,%rax
   250e4:	test   %rax,%rax
   250e7:	jne    255e7 <handoff::old_give+0xb37>
   250ed:	xor    %ebp,%ebp
   250ef:	movzbl 0x4(%rbx),%eax
   250f3:	test   %al,%al
   250f5:	jne    255ff <handoff::old_give+0xb4f>
   250fb:	mov    0x48(%rbx),%rax
   250ff:	test   %rax,%rax
   25102:	je     25289 <handoff::old_give+0x7d9>
   25108:	mov    %fs:0x0,%rcx
   25111:	lea    -0x78(%rcx),%rcx
   25118:	mov    0x40(%rbx),%rdx
   2511c:	shl    $0x3,%rax
   25120:	lea    (%rax,%rax,2),%rsi
   25124:	mov    $0x1,%r13d
   2512a:	xor    %r15d,%r15d
   2512d:	jmp    25140 <handoff::old_give+0x690>
   2512f:	nop
   25130:	add    $0x18,%r15
   25134:	inc    %r13
   25137:	cmp    %r15,%rsi
   2513a:	je     25289 <handoff::old_give+0x7d9>
   25140:	mov    (%rdx,%r15,1),%rdi
   25144:	cmp    %rcx,0x28(%rdi)
   25148:	je     25130 <handoff::old_give+0x680>
   2514a:	mov    0x8(%rdx,%r15,1),%r8
   2514f:	xor    %eax,%eax
   25151:	lock cmpxchg %r8,0x18(%rdi)
   25157:	jne    25130 <handoff::old_give+0x680>
   25159:	mov    0x10(%rdx,%r15,1),%rax
   2515e:	test   %rax,%rax
   25161:	je     25167 <handoff::old_give+0x6b7>
   25163:	mov    %rax,0x20(%rdi)
   25167:	mov    %r14,0x8(%rsp)
   2516c:	mov    0x10(%rdi),%rdi
   25170:	mov    $0x1,%eax
   25175:	xchg   %eax,0x28(%rdi)
   25178:	cmp    $0xffffffff,%eax
   2517b:	jne    25187 <handoff::old_give+0x6d7>
   2517d:	add    $0x28,%rdi
   25181:	call   *0x4bd09(%rip)        # 70e90 <_DYNAMIC+0x408>
   25187:	lea    -0x1(%r13),%rax
   2518b:	mov    0x48(%rbx),%r12
   2518f:	mov    %rax,0x40(%rsp)
   25194:	cmp    %r12,%rax
   25197:	jae    25730 <handoff::old_give+0xc80>
   2519d:	mov    0x40(%rbx),%rax
   251a1:	lea    (%rax,%r15,1),%rdi
   251a5:	mov    (%rax,%r15,1),%r14
   251a9:	movups 0x8(%rax,%r15,1),%xmm0
   251af:	movaps %xmm0,0x60(%rsp)
   251b4:	lea    (%rax,%r15,1),%rsi
   251b8:	add    $0x18,%rsi
   251bc:	mov    %r12,%rax
   251bf:	sub    %r13,%rax
   251c2:	shl    $0x3,%rax
   251c6:	lea    (%rax,%rax,2),%rdx
   251ca:	call   *0x4bb00(%rip)        # 70cd0 <memmove@GLIBC_2.2.5>
   251d0:	dec    %r12
   251d3:	mov    %r12,0x48(%rbx)
   251d7:	test   %r14,%r14
   251da:	je     25730 <handoff::old_give+0xc80>
   251e0:	movaps 0x60(%rsp),%xmm0
   251e5:	movups %xmm0,0x1d0(%rsp)
   251ed:	mov    %r14,0x1c8(%rsp)
   251f5:	mov    0x1d8(%rsp),%rax
   251fd:	mov    %rax,0x30(%rsp)
   25202:	test   %bpl,%bpl
   25205:	jne    25224 <handoff::old_give+0x774>
   25207:	mov    0x4bae2(%rip),%rax        # 70cf0 <_DYNAMIC+0x268>
   2520e:	mov    (%rax),%rax
   25211:	movabs $0x7fffffffffffffff,%rcx
   2521b:	test   %rcx,%rax
   2521e:	jne    2570c <handoff::old_give+0xc5c>
   25224:	xor    %eax,%eax
   25226:	xchg   %eax,(%rbx)
   25228:	cmp    $0x2,%eax
   2522b:	mov    0x8(%rsp),%r15
   25230:	je     256fe <handoff::old_give+0xc4e>
   25236:	mov    0x30(%rsp),%rbx
   2523b:	test   %rbx,%rbx
   2523e:	je     25747 <handoff::old_give+0xc97>
   25244:	movq   $0x1,(%rbx)
   2524b:	lea    0x8(%rbx),%rdi
   2524f:	mov    $0x138,%edx
   25254:	mov    %r15,%rsi
   25257:	call   *0x4ba7b(%rip)        # 70cd8 <memcpy@GLIBC_2.14>
   2525d:	movb   $0x1,0x140(%rbx)
   25264:	movq   $0xffffffffffffffff,0x60(%rsp)
   2526d:	lock decq (%r14)
   25271:	jne    254b3 <handoff::old_give+0xa03>
   25277:	lea    0x1c8(%rsp),%rdi
   2527f:	call   28130 <<alloc::sync::Arc<std::sync::mpmc::context::Inner>>::drop_slow>
   25284:	jmp    254b3 <handoff::old_give+0xa03>
   25289:	cmpb   $0x0,0x68(%rbx)
   2528d:	je     252b9 <handoff::old_give+0x809>
   2528f:	movq   $0x1,0x60(%rsp)
   25298:	lea    0x68(%rsp),%rdi
   2529d:	mov    $0x138,%edx
   252a2:	mov    %r14,%rsi
   252a5:	call   *0x4ba2d(%rip)        # 70cd8 <memcpy@GLIBC_2.14>
   252ab:	test   %bpl,%bpl
   252ae:	je     25489 <handoff::old_give+0x9d9>
   252b4:	jmp    254a6 <handoff::old_give+0x9f6>
   252b9:	lea    0x1c8(%rsp),%rdi
   252c1:	mov    $0x138,%edx
   252c6:	mov    %r14,%rsi
   252c9:	call   *0x4ba09(%rip)        # 70cd8 <memcpy@GLIBC_2.14>
   252cf:	lea    0x10(%rsp),%rax
   252d4:	mov    %rax,0x300(%rsp)
   252dc:	lea    0x50(%rsp),%r13
   252e1:	mov    %r13,0x308(%rsp)
   252e9:	mov    %rbx,0x310(%rsp)
   252f1:	mov    %rbx,0x318(%rsp)
   252f9:	mov    %bpl,0x320(%rsp)
   25301:	mov    $0xffffffffffffffe0,%rax
   25308:	mov    %fs:0x0,%r15
   25311:	add    %rax,%r15
   25314:	cmpb   $0x1,%fs:0x8(%rax)
   25319:	jne    25641 <handoff::old_give+0xb91>
   2531f:	mov    (%r15),%r12
   25322:	movq   $0x0,(%r15)
   25329:	test   %r12,%r12
   2532c:	je     2565c <handoff::old_give+0xbac>
   25332:	lea    0x321(%rsp),%r14
   2533a:	mov    %r12,0x48(%rsp)
   2533f:	movq   $0x0,0x18(%r12)
   25348:	movq   $0x0,0x20(%r12)
   25351:	movb   $0x2,0x320(%rsp)
   25359:	lea    0x60(%rsp),%rbx
   2535e:	lea    0x1c8(%rsp),%rsi
   25366:	mov    $0x158,%edx
   2536b:	mov    %rbx,%rdi
   2536e:	call   *0x4b964(%rip)        # 70cd8 <memcpy@GLIBC_2.14>
   25374:	mov    %bpl,0x1b8(%rsp)
   2537c:	mov    (%r14),%eax
   2537f:	mov    0x3(%r14),%ecx
   25383:	mov    %eax,0x1b9(%rsp)
   2538a:	mov    %ecx,0x1bc(%rsp)
   25391:	lea    0x328(%rsp),%rdi
   25399:	mov    %rbx,%rsi
   2539c:	mov    %r12,%rdx
   2539f:	call   1f9c0 <<std::sync::mpmc::zero::Channel<(u64, [u64; 38])>>::send::{closure#0}>
   253a4:	mov    (%r15),%rax
   253a7:	mov    %rax,0x60(%rsp)
   253ac:	mov    %r12,(%r15)
   253af:	test   %rax,%rax
   253b2:	je     253c4 <handoff::old_give+0x914>
   253b4:	lock decq (%rax)
   253b8:	jne    253c4 <handoff::old_give+0x914>
   253ba:	lea    0x60(%rsp),%rdi
   253bf:	call   28130 <<alloc::sync::Arc<std::sync::mpmc::context::Inner>>::drop_slow>
   253c4:	lea    0x468(%rsp),%rdi
   253cc:	lea    0x328(%rsp),%rsi
   253d4:	mov    $0x140,%edx
   253d9:	call   *0x4b8f9(%rip)        # 70cd8 <memcpy@GLIBC_2.14>
   253df:	mov    0x468(%rsp),%rbx
   253e7:	lea    0x470(%rsp),%rsi
   253ef:	lea    0x328(%rsp),%rdi
   253f7:	mov    $0x138,%edx
   253fc:	call   *0x4b8d6(%rip)        # 70cd8 <memcpy@GLIBC_2.14>
   25402:	cmp    $0xfffffffffffffffe,%rbx
   25406:	jne    25454 <handoff::old_give+0x9a4>
   25408:	lea    0x60(%rsp),%rdi
   2540d:	lea    0x1c8(%rsp),%rsi
   25415:	call   1dd70 <<std::sync::mpmc::context::Context>::with::<<std::sync::mpmc::zero::Channel<(u64, [u64; 38])>>::send::{closure#0}, core::result::Result<(), std::sync::mpmc::error::SendTimeoutError<(u64, [u64; 38])>>>::{closure#2}>
   2541a:	movzbl 0x320(%rsp),%eax
   25422:	cmp    $0x2,%al
   25424:	jne    2547d <handoff::old_give+0x9cd>
   25426:	jmp    254b3 <handoff::old_give+0xa03>
   2542b:	xor    %r12d,%r12d
   2542e:	xor    %r13d,%r13d
   25431:	test   %r15,%r15
   25434:	je     25449 <handoff::old_give+0x999>
   25436:	mov    $0x26c8,%esi
   2543b:	mov    $0x8,%edx
   25440:	mov    %r15,%rdi
   25443:	call   *0x4b897(%rip)        # 70ce0 <_DYNAMIC+0x258>
   25449:	test   %r12,%r12
   2544c:	jne    2550e <handoff::old_give+0xa5e>
   25452:	jmp    254c9 <handoff::old_give+0xa19>
   25454:	mov    %rbx,0x60(%rsp)
   25459:	lea    0x68(%rsp),%rdi
   2545e:	lea    0x328(%rsp),%rsi
   25466:	mov    $0x138,%edx
   2546b:	call   *0x4b867(%rip)        # 70cd8 <memcpy@GLIBC_2.14>
   25471:	movzbl 0x320(%rsp),%eax
   25479:	cmp    $0x2,%al
   2547b:	je     254b3 <handoff::old_give+0xa03>
   2547d:	mov    0x318(%rsp),%rbx
   25485:	test   $0x1,%al
   25487:	jne    254a6 <handoff::old_give+0x9f6>
   25489:	mov    0x4b860(%rip),%rax        # 70cf0 <_DYNAMIC+0x268>
   25490:	mov    (%rax),%rax
   25493:	movabs $0x7fffffffffffffff,%rcx
   2549d:	test   %rcx,%rax
   254a0:	jne    256e7 <handoff::old_give+0xc37>
   254a6:	xor    %eax,%eax
   254a8:	xchg   %eax,(%rbx)
   254aa:	cmp    $0x2,%eax
   254ad:	je     25633 <handoff::old_give+0xb83>
   254b3:	mov    0x60(%rsp),%r13
   254b8:	mov    $0x1,%al
   254ba:	cmp    $0xffffffffffffffff,%r13
   254be:	je     254cb <handoff::old_give+0xa1b>
   254c0:	test   %r13,%r13
   254c3:	je     255c0 <handoff::old_give+0xb10>
   254c9:	xor    %eax,%eax
   254cb:	add    $0x5a8,%rsp
   254d2:	pop    %rbx
   254d3:	pop    %r12
   254d5:	pop    %r13
   254d7:	pop    %r14
   254d9:	pop    %r15
   254db:	pop    %rbp
   254dc:	ret
   254dd:	cmp    $0x1e,%r13d
   254e1:	jne    25431 <handoff::old_give+0x981>
   254e7:	test   %r15,%r15
   254ea:	je     25723 <handoff::old_give+0xc73>
   254f0:	mov    %r15,0x88(%rbx)
   254f7:	lock addq $0x2,0x80(%rbx)
   25500:	mov    %r15,0x26c0(%r12)
   25508:	mov    $0x1e,%r13d
   2550e:	lea    0x0(,%r13,4),%r14
   25516:	add    %r13,%r14
   25519:	shl    $0x6,%r14d
   2551d:	lea    (%r12,%r14,1),%rdi
   25521:	mov    $0x138,%edx
   25526:	mov    0x8(%rsp),%rsi
   2552b:	call   *0x4b7a7(%rip)        # 70cd8 <memcpy@GLIBC_2.14>
   25531:	lock orq $0x1,0x138(%r12,%r14,1)
   2553b:	add    $0x100,%rbx
   25542:	mov    %rbx,%rdi
   25545:	call   27020 <<std::sync::mpmc::waker::SyncWaker>::notify>
   2554a:	mov    $0x1,%al
   2554c:	jmp    254cb <handoff::old_give+0xa1b>
   25551:	add    %rcx,%r14
   25554:	mov    %r14,0x60(%rsp)
   25559:	inc    %rbp
   2555c:	mov    %rbp,0x68(%rsp)
   25561:	lea    0x8(%r14),%rdi
   25565:	mov    $0x138,%edx
   2556a:	mov    0x8(%rsp),%rsi
   2556f:	call   *0x4b763(%rip)        # 70cd8 <memcpy@GLIBC_2.14>
   25575:	mov    %rbp,(%r14)
   25578:	add    $0x140,%rbx
   2557f:	mov    %rbx,%rdi
   25582:	call   27020 <<std::sync::mpmc::waker::SyncWaker>::notify>
   25587:	mov    $0xffffffffffffffff,%r13
   2558e:	jmp    254b8 <handoff::old_give+0xa08>
   25593:	mov    $0x8,%edi
   25598:	mov    $0x26c8,%esi
   2559d:	call   *0x4b805(%rip)        # 70da8 <_DYNAMIC+0x320>
   255a3:	xor    %r13d,%r13d
   255a6:	jmp    254b8 <handoff::old_give+0xa08>
   255ab:	mov    $0x8,%edi
   255b0:	mov    $0x26c8,%esi
   255b5:	call   *0x4b7ed(%rip)        # 70da8 <_DYNAMIC+0x320>
   255bb:	jmp    25754 <handoff::old_give+0xca4>
   255c0:	lea    -0x1b657(%rip),%rdi        # 9f70 <GCC_except_table142+0x248>
   255c7:	lea    0x48d0a(%rip),%rdx        # 6e2d8 <__frame_dummy_init_array_entry+0x3d0>
   255ce:	mov    $0x28,%esi
   255d3:	call   *0x4b6ef(%rip)        # 70cc8 <_DYNAMIC+0x240>
   255d9:	mov    %rbx,%rdi
   255dc:	call   *0x4b716(%rip)        # 70cf8 <_DYNAMIC+0x270>
   255e2:	jmp    250d7 <handoff::old_give+0x627>
   255e7:	call   *0x4b713(%rip)        # 70d00 <_DYNAMIC+0x278>
   255ed:	mov    %eax,%ebp
   255ef:	xor    $0x1,%bpl
   255f3:	movzbl 0x4(%rbx),%eax
   255f7:	test   %al,%al
   255f9:	je     250fb <handoff::old_give+0x64b>
   255ff:	mov    %rbx,0x60(%rsp)
   25604:	mov    %bpl,0x68(%rsp)
   25609:	lea    -0x195c8(%rip),%rdi        # c048 <GCC_except_table142+0x2320>
   25610:	lea    0x48ee1(%rip),%rcx        # 6e4f8 <__frame_dummy_init_array_entry+0x5f0>
   25617:	lea    0x49122(%rip),%r8        # 6e740 <__frame_dummy_init_array_entry+0x838>
   2561e:	lea    0x60(%rsp),%rdx
   25623:	mov    $0x2b,%esi
   25628:	call   *0x4b772(%rip)        # 70da0 <_DYNAMIC+0x318>
   2562e:	jmp    25754 <handoff::old_give+0xca4>
   25633:	mov    %rbx,%rdi
   25636:	call   *0x4b6cc(%rip)        # 70d08 <_DYNAMIC+0x280>
   2563c:	jmp    254b3 <handoff::old_give+0xa03>
   25641:	mov    %r15,%rdi
   25644:	xor    %esi,%esi
   25646:	call   1ad90 <<std::sys::thread_local::native::lazy::Storage<core::cell::Cell<core::option::Option<std::sync::mpmc::context::Context>>, ()>>::get_or_init_slow::<<std::sync::mpmc::context::Context>::with::CONTEXT::__rust_std_internal_init_fn>>
   2564b:	mov    %rax,%r15
   2564e:	test   %rax,%rax
   25651:	jne    2531f <handoff::old_give+0x86f>
   25657:	jmp    25408 <handoff::old_give+0x958>
   2565c:	call   *0x4b65e(%rip)        # 70cc0 <_DYNAMIC+0x238>
   25662:	mov    %rax,%r15
   25665:	mov    %rax,0x328(%rsp)
   2566d:	movb   $0x2,0x320(%rsp)
   25675:	lea    0x60(%rsp),%r12
   2567a:	mov    $0x138,%edx
   2567f:	mov    %r12,%rdi
   25682:	mov    %r14,%rsi
   25685:	call   *0x4b64d(%rip)        # 70cd8 <memcpy@GLIBC_2.14>
   2568b:	lea    0x10(%rsp),%rax
   25690:	mov    %rax,0x198(%rsp)
   25698:	mov    %r13,0x1a0(%rsp)
   256a0:	mov    %rbx,0x1a8(%rsp)
   256a8:	mov    %rbx,0x1b0(%rsp)
   256b0:	mov    %bpl,0x1b8(%rsp)
   256b8:	lea    0x468(%rsp),%rdi
   256c0:	mov    %r12,%rsi
   256c3:	mov    %r15,%rdx
   256c6:	call   1f9c0 <<std::sync::mpmc::zero::Channel<(u64, [u64; 38])>>::send::{closure#0}>
   256cb:	lock decq (%r15)
   256cf:	jne    253df <handoff::old_give+0x92f>
   256d5:	lea    0x328(%rsp),%rdi
   256dd:	call   28130 <<alloc::sync::Arc<std::sync::mpmc::context::Inner>>::drop_slow>
   256e2:	jmp    253df <handoff::old_give+0x92f>
   256e7:	call   *0x4b613(%rip)        # 70d00 <_DYNAMIC+0x278>
   256ed:	test   %al,%al
   256ef:	jne    254a6 <handoff::old_give+0x9f6>
   256f5:	movb   $0x1,0x4(%rbx)
   256f9:	jmp    254a6 <handoff::old_give+0x9f6>
   256fe:	mov    %rbx,%rdi
   25701:	call   *0x4b601(%rip)        # 70d08 <_DYNAMIC+0x280>
   25707:	jmp    25236 <handoff::old_give+0x786>
   2570c:	call   *0x4b5ee(%rip)        # 70d00 <_DYNAMIC+0x278>
   25712:	test   %al,%al
   25714:	jne    25224 <handoff::old_give+0x774>
   2571a:	movb   $0x1,0x4(%rbx)
   2571e:	jmp    25224 <handoff::old_give+0x774>
   25723:	lea    0x48f9e(%rip),%rdi        # 6e6c8 <__frame_dummy_init_array_entry+0x7c0>
   2572a:	call   *0x4b658(%rip)        # 70d88 <_DYNAMIC+0x300>
   25730:	lea    0x48e41(%rip),%rdx        # 6e578 <__frame_dummy_init_array_entry+0x670>
   25737:	mov    0x40(%rsp),%rdi
   2573c:	mov    %r12,%rsi
   2573f:	call   *0x4b69b(%rip)        # 70de0 <_DYNAMIC+0x358>
   25745:	jmp    25754 <handoff::old_give+0xca4>
   25747:	lea    0x4900a(%rip),%rdi        # 6e758 <__frame_dummy_init_array_entry+0x850>
   2574e:	call   *0x4b634(%rip)        # 70d88 <_DYNAMIC+0x300>
   25754:	ud2
   25756:	mov    %rax,%r13
   25759:	lock decq (%r12)
   2575e:	jne    2583b <handoff::old_give+0xd8b>
   25764:	lea    0x10(%rsp),%rdi
   25769:	call   28130 <<alloc::sync::Arc<std::sync::mpmc::context::Inner>>::drop_slow>
   2576e:	jmp    2583b <handoff::old_give+0xd8b>
   25773:	mov    %rax,%r13
   25776:	lock decq (%r15)
   2577a:	jne    257fa <handoff::old_give+0xd4a>
   2577c:	lea    0x328(%rsp),%rdi
   25784:	call   28130 <<alloc::sync::Arc<std::sync::mpmc::context::Inner>>::drop_slow>
   25789:	jmp    257fa <handoff::old_give+0xd4a>
   2578b:	mov    %rax,%r13
   2578e:	lock decq (%r12)
   25793:	jne    2583b <handoff::old_give+0xd8b>
   25799:	lea    0x10(%rsp),%rdi
   2579e:	call   28130 <<alloc::sync::Arc<std::sync::mpmc::context::Inner>>::drop_slow>
   257a3:	jmp    2583b <handoff::old_give+0xd8b>
   257a8:	call   *0x4b56a(%rip)        # 70d18 <_DYNAMIC+0x290>
   257ae:	mov    %rax,%r13
   257b1:	lock decq (%r14)
   257b5:	jne    2583b <handoff::old_give+0xd8b>
   257bb:	lea    0x1c8(%rsp),%rdi
   257c3:	call   28130 <<alloc::sync::Arc<std::sync::mpmc::context::Inner>>::drop_slow>
   257c8:	jmp    2583b <handoff::old_give+0xd8b>
   257ca:	mov    %rax,%r13
   257cd:	movzbl %bpl,%esi
   257d1:	mov    %rbx,%rdi
   257d4:	call   1c960 <core::ptr::drop_glue::<std::sync::poison::mutex::MutexGuard<handoff::Queued<(u64, [u64; 38])>>>>
   257d9:	jmp    2583b <handoff::old_give+0xd8b>
   257db:	call   *0x4b537(%rip)        # 70d18 <_DYNAMIC+0x290>
   257e1:	mov    %rax,%r13
   257e4:	jmp    257fa <handoff::old_give+0xd4a>
   257e6:	mov    %rax,%r13
   257e9:	lock decq (%r12)
   257ee:	jne    257fa <handoff::old_give+0xd4a>
   257f0:	lea    0x48(%rsp),%rdi
   257f5:	call   28130 <<alloc::sync::Arc<std::sync::mpmc::context::Inner>>::drop_slow>
   257fa:	mov    0x318(%rsp),%rdi
   25802:	mov    0x320(%rsp),%esi
   25809:	call   1bc20 <core::ptr::drop_glue::<core::option::Option<<std::sync::mpmc::zero::Channel<(u64, [u64; 38])>>::recv::{closure#1}>>>
   2580e:	jmp    2583b <handoff::old_give+0xd8b>
   25810:	call   *0x4b502(%rip)        # 70d18 <_DYNAMIC+0x290>
   25816:	call   *0x4b4fc(%rip)        # 70d18 <_DYNAMIC+0x290>
   2581c:	jmp    25833 <handoff::old_give+0xd83>
   2581e:	mov    %rax,%r13
   25821:	lea    0x60(%rsp),%rdi
   25826:	call   1c5f0 <core::ptr::drop_glue::<std::sync::poison::PoisonError<std::sync::poison::mutex::MutexGuard<handoff::State>>>>
   2582b:	jmp    2583b <handoff::old_give+0xd8b>
   2582d:	call   *0x4b4e5(%rip)        # 70d18 <_DYNAMIC+0x290>
   25833:	mov    %rax,%r13
   25836:	test   %r15,%r15
   25839:	jne    25843 <handoff::old_give+0xd93>
   2583b:	mov    %r13,%rdi
   2583e:	call   6cec0 <_Unwind_Resume@plt>
   25843:	mov    $0x26c8,%esi
   25848:	mov    $0x8,%edx
   2584d:	mov    %r15,%rdi
   25850:	call   *0x4b48a(%rip)        # 70ce0 <_DYNAMIC+0x258>
   25856:	mov    %r13,%rdi
   25859:	call   6cec0 <_Unwind_Resume@plt>
   2585e:	int3
   2585f:	int3

