   25860:	push   %rbp
   25861:	push   %r15
   25863:	push   %r14
   25865:	push   %r13
   25867:	push   %r12
   25869:	push   %rbx
   2586a:	sub    $0x448,%rsp
   25871:	mov    %rdi,%rbp
   25874:	mov    $0x1,%ecx
   25879:	xor    %eax,%eax
   2587b:	mov    %rsi,0x10(%rsp)
   25880:	lock cmpxchg %ecx,(%rsi)
   25884:	jne    26818 <handoff::old_take+0xfb8>
   2588a:	mov    0x4b45f(%rip),%rax        # 70cf0 <_DYNAMIC+0x268>
   25891:	mov    (%rax),%rax
   25894:	shl    $1,%rax
   25897:	test   %rax,%rax
   2589a:	jne    26828 <handoff::old_take+0xfc8>
   258a0:	movl   $0x0,0x4(%rsp)
   258a8:	mov    0x10(%rsp),%rcx
   258ad:	movzbl 0x4(%rcx),%eax
   258b1:	mov    0x8(%rcx),%rax
   258b5:	mov    0x10(%rcx),%r15
   258b9:	test   %rax,%rax
   258bc:	je     25c15 <handoff::old_take+0x3b5>
   258c2:	cmp    $0x1,%eax
   258c5:	jne    25f95 <handoff::old_take+0x735>
   258cb:	mov    %rbp,0x8(%rsp)
   258d0:	movl   $0xffffffff,0x20(%rsp)
   258d8:	mov    %fs:0x0,%rax
   258e1:	lea    -0x20(%rax),%rax
   258e8:	mov    %rax,0x68(%rsp)
   258ed:	xorps  %xmm0,%xmm0
   258f0:	movaps %xmm0,0x310(%rsp)
   258f8:	movaps %xmm0,0x300(%rsp)
   25900:	movq   $0x0,0x320(%rsp)
   2590c:	mov    0x4b43d(%rip),%r14        # 70d50 <_DYNAMIC+0x2c8>
   25913:	mov    (%r15),%rax
   25916:	mov    0x8(%r15),%r13
   2591a:	xor    %ebx,%ebx
   2591c:	jmp    2592c <handoff::old_take+0xcc>
   2591e:	xchg   %ax,%ax
   25920:	call   *%r14
   25923:	mov    (%r15),%rax
   25926:	mov    0x8(%r15),%r13
   2592a:	inc    %ebx
   2592c:	mov    %rax,%rcx
   2592f:	shr    $1,%rcx
   25932:	mov    %ecx,%ebp
   25934:	and    $0x1f,%ebp
   25937:	cmp    $0x1f,%rbp
   2593b:	jne    25990 <handoff::old_take+0x130>
   2593d:	cmp    $0x6,%ebx
   25940:	ja     25920 <handoff::old_take+0xc0>
   25942:	test   %ebx,%ebx
   25944:	je     25923 <handoff::old_take+0xc3>
   25946:	mov    %ebx,%ecx
   25948:	imul   %ebx,%ecx
   2594b:	mov    %ecx,%eax
   2594d:	and    $0x5,%eax
   25950:	cmp    $0x8,%ecx
   25953:	jb     25980 <handoff::old_take+0x120>
   25955:	and    $0x38,%ecx
   25958:	nopl   0x0(%rax,%rax,1)
   25960:	pause
   25962:	pause
   25964:	pause
   25966:	pause
   25968:	pause
   2596a:	pause
   2596c:	pause
   2596e:	pause
   25970:	add    $0xfffffff8,%ecx
   25973:	jne    25960 <handoff::old_take+0x100>
   25975:	test   %eax,%eax
   25977:	je     25923 <handoff::old_take+0xc3>
   25979:	nopl   0x0(%rax)
   25980:	pause
   25982:	dec    %eax
   25984:	jne    25980 <handoff::old_take+0x120>
   25986:	jmp    25923 <handoff::old_take+0xc3>
   25988:	nopl   0x0(%rax,%rax,1)
   25990:	lea    0x2(%rax),%r12
   25994:	test   $0x1,%al
   25996:	jne    259c3 <handoff::old_take+0x163>
   25998:	lock orl $0x0,-0x40(%rsp)
   2599e:	mov    0x80(%r15),%rdx
   259a5:	mov    %rdx,%rsi
   259a8:	shr    $1,%rsi
   259ab:	cmp    %rsi,%rcx
   259ae:	je     25a90 <handoff::old_take+0x230>
   259b4:	xor    %rax,%rdx
   259b7:	xor    %ecx,%ecx
   259b9:	cmp    $0x40,%rdx
   259bd:	setae  %cl
   259c0:	or     %rcx,%r12
   259c3:	test   %r13,%r13
   259c6:	je     25a2a <handoff::old_take+0x1ca>
   259c8:	lock cmpxchg %r12,(%r15)
   259cd:	je     261cf <handoff::old_take+0x96f>
   259d3:	cmp    $0x6,%ebx
   259d6:	mov    $0x6,%ecx
   259db:	cmovb  %ebx,%ecx
   259de:	test   %ebx,%ebx
   259e0:	je     25a7b <handoff::old_take+0x21b>
   259e6:	imul   %ecx,%ecx
   259e9:	mov    %ecx,%eax
   259eb:	and    $0x5,%eax
   259ee:	cmp    $0x8,%ecx
   259f1:	jb     25a20 <handoff::old_take+0x1c0>
   259f3:	and    $0x38,%ecx
   259f6:	cs nopw 0x0(%rax,%rax,1)
   25a00:	pause
   25a02:	pause
   25a04:	pause
   25a06:	pause
   25a08:	pause
   25a0a:	pause
   25a0c:	pause
   25a0e:	pause
   25a10:	add    $0xfffffff8,%ecx
   25a13:	jne    25a00 <handoff::old_take+0x1a0>
   25a15:	test   %eax,%eax
   25a17:	je     25a26 <handoff::old_take+0x1c6>
   25a19:	nopl   0x0(%rax)
   25a20:	pause
   25a22:	dec    %eax
   25a24:	jne    25a20 <handoff::old_take+0x1c0>
   25a26:	inc    %ebx
   25a28:	jmp    25a80 <handoff::old_take+0x220>
   25a2a:	cmp    $0x6,%ebx
   25a2d:	ja     25920 <handoff::old_take+0xc0>
   25a33:	test   %ebx,%ebx
   25a35:	je     25923 <handoff::old_take+0xc3>
   25a3b:	mov    %ebx,%ecx
   25a3d:	imul   %ebx,%ecx
   25a40:	mov    %ecx,%eax
   25a42:	and    $0x5,%eax
   25a45:	cmp    $0x8,%ecx
   25a48:	jb     25a70 <handoff::old_take+0x210>
   25a4a:	and    $0x38,%ecx
   25a4d:	nopl   (%rax)
   25a50:	pause
   25a52:	pause
   25a54:	pause
   25a56:	pause
   25a58:	pause
   25a5a:	pause
   25a5c:	pause
   25a5e:	pause
   25a60:	add    $0xfffffff8,%ecx
   25a63:	jne    25a50 <handoff::old_take+0x1f0>
   25a65:	test   %eax,%eax
   25a67:	je     25923 <handoff::old_take+0xc3>
   25a6d:	nopl   (%rax)
   25a70:	pause
   25a72:	dec    %eax
   25a74:	jne    25a70 <handoff::old_take+0x210>
   25a76:	jmp    25923 <handoff::old_take+0xc3>
   25a7b:	mov    $0x1,%ebx
   25a80:	mov    (%r15),%rax
   25a83:	mov    0x8(%r15),%r13
   25a87:	jmp    2592c <handoff::old_take+0xcc>
   25a8c:	nopl   0x0(%rax)
   25a90:	mov    $0x1,%bl
   25a92:	test   $0x1,%dl
   25a95:	jne    263f8 <handoff::old_take+0xb98>
   25a9b:	mov    %r14,%r12
   25a9e:	mov    0x20(%rsp),%ebp
   25aa2:	cmp    $0xffffffff,%ebp
   25aa5:	je     25ac9 <handoff::old_take+0x269>
   25aa7:	mov    0x18(%rsp),%r14
   25aac:	call   *0x4b30e(%rip)        # 70dc0 <_DYNAMIC+0x338>
   25ab2:	cmp    %r14,%rax
   25ab5:	jne    25ac0 <handoff::old_take+0x260>
   25ab7:	cmp    %ebp,%edx
   25ab9:	jb     25ac9 <handoff::old_take+0x269>
   25abb:	jmp    263f8 <handoff::old_take+0xb98>
   25ac0:	cmp    %r14,%rax
   25ac3:	jge    263f8 <handoff::old_take+0xb98>
   25ac9:	lea    0x300(%rsp),%rax
   25ad1:	mov    %rax,0x1c0(%rsp)
   25ad9:	mov    %r15,0x1c8(%rsp)
   25ae1:	lea    0x18(%rsp),%rax
   25ae6:	mov    %rax,0x1d0(%rsp)
   25aee:	mov    $0xffffffffffffffe0,%rax
   25af5:	cmpb   $0x1,%fs:0x8(%rax)
   25afa:	mov    0x68(%rsp),%r13
   25aff:	jne    25b92 <handoff::old_take+0x332>
   25b05:	mov    0x0(%r13),%rbp
   25b09:	movq   $0x0,0x0(%r13)
   25b11:	test   %rbp,%rbp
   25b14:	je     25bb9 <handoff::old_take+0x359>
   25b1a:	mov    %rbp,0x40(%rsp)
   25b1f:	movq   $0x0,0x18(%rbp)
   25b27:	movq   $0x0,0x20(%rbp)
   25b2f:	movq   $0x0,0x1c0(%rsp)
   25b3b:	lea    0x300(%rsp),%rax
   25b43:	mov    %rax,0x70(%rsp)
   25b48:	lea    0x1c8(%rsp),%rax
   25b50:	movups (%rax),%xmm0
   25b53:	lea    0x78(%rsp),%rax
   25b58:	movups %xmm0,(%rax)
   25b5b:	lea    0x70(%rsp),%rdi
   25b60:	mov    %rbp,%rsi
   25b63:	call   1f100 <<std::sync::mpmc::list::Channel<(u64, [u64; 38])>>::recv::{closure#1}>
   25b68:	mov    0x0(%r13),%rax
   25b6c:	mov    %rax,0x70(%rsp)
   25b71:	mov    %rbp,0x0(%r13)
   25b75:	test   %rax,%rax
   25b78:	je     25b8a <handoff::old_take+0x32a>
   25b7a:	lock decq (%rax)
   25b7e:	jne    25b8a <handoff::old_take+0x32a>
   25b80:	lea    0x70(%rsp),%rdi
   25b85:	call   28130 <<alloc::sync::Arc<std::sync::mpmc::context::Inner>>::drop_slow>
   25b8a:	mov    %r12,%r14
   25b8d:	jmp    25913 <handoff::old_take+0xb3>
   25b92:	mov    0x68(%rsp),%rdi
   25b97:	xor    %esi,%esi
   25b99:	call   1ad90 <<std::sys::thread_local::native::lazy::Storage<core::cell::Cell<core::option::Option<std::sync::mpmc::context::Context>>, ()>>::get_or_init_slow::<<std::sync::mpmc::context::Context>::with::CONTEXT::__rust_std_internal_init_fn>>
   25b9e:	mov    %rax,%r13
   25ba1:	test   %rax,%rax
   25ba4:	jne    25b05 <handoff::old_take+0x2a5>
   25baa:	lea    0x1c0(%rsp),%rdi
   25bb2:	call   1dc30 <<std::sync::mpmc::context::Context>::with::<<std::sync::mpmc::list::Channel<(u64, [u64; 38])>>::recv::{closure#1}, ()>::{closure#2}>
   25bb7:	jmp    25b8a <handoff::old_take+0x32a>
   25bb9:	call   *0x4b101(%rip)        # 70cc0 <_DYNAMIC+0x238>
   25bbf:	mov    %rax,%r13
   25bc2:	mov    %rax,0x40(%rsp)
   25bc7:	movq   $0x0,0x1c0(%rsp)
   25bd3:	lea    0x300(%rsp),%rax
   25bdb:	mov    %rax,0x70(%rsp)
   25be0:	mov    %r15,0x78(%rsp)
   25be5:	lea    0x18(%rsp),%rax
   25bea:	mov    %rax,0x80(%rsp)
   25bf2:	lea    0x70(%rsp),%rdi
   25bf7:	mov    %r13,%rsi
   25bfa:	call   1f100 <<std::sync::mpmc::list::Channel<(u64, [u64; 38])>>::recv::{closure#1}>
   25bff:	lock decq 0x0(%r13)
   25c04:	jne    25b8a <handoff::old_take+0x32a>
   25c06:	lea    0x40(%rsp),%rdi
   25c0b:	call   28130 <<alloc::sync::Arc<std::sync::mpmc::context::Inner>>::drop_slow>
   25c10:	jmp    25b8a <handoff::old_take+0x32a>
   25c15:	mov    %rbp,0x8(%rsp)
   25c1a:	movl   $0xffffffff,0x20(%rsp)
   25c22:	mov    %fs:0x0,%r12
   25c2b:	add    $0xffffffffffffffe0,%r12
   25c32:	xorps  %xmm0,%xmm0
   25c35:	movaps %xmm0,0x310(%rsp)
   25c3d:	movaps %xmm0,0x300(%rsp)
   25c45:	movq   $0x0,0x320(%rsp)
   25c51:	mov    0x4b0f8(%rip),%r14        # 70d50 <_DYNAMIC+0x2c8>
   25c58:	xor    %eax,%eax
   25c5a:	jmp    25c64 <handoff::old_take+0x404>
   25c5c:	call   *%r14
   25c5f:	nop
   25c60:	inc    %ebx
   25c62:	mov    %ebx,%eax
   25c64:	mov    %eax,%ebx
   25c66:	mov    (%r15),%r13
   25c69:	mov    0x190(%r15),%rdx
   25c70:	mov    0x198(%r15),%rbp
   25c77:	dec    %rdx
   25c7a:	and    %r13,%rdx
   25c7d:	xor    %eax,%eax
   25c7f:	sub    0x188(%r15),%rax
   25c86:	lea    (%rdx,%rdx,4),%rcx
   25c8a:	shl    $0x6,%rcx
   25c8e:	mov    0x0(%rbp,%rcx,1),%rsi
   25c93:	lea    0x1(%r13),%rdi
   25c97:	cmp    %rsi,%rdi
   25c9a:	jne    25d20 <handoff::old_take+0x4c0>
   25ca0:	inc    %rdx
   25ca3:	cmp    0x180(%r15),%rdx
   25caa:	jb     25cb9 <handoff::old_take+0x459>
   25cac:	and    %r13,%rax
   25caf:	add    0x188(%r15),%rax
   25cb6:	mov    %rax,%rsi
   25cb9:	mov    %r13,%rax
   25cbc:	lock cmpxchg %rsi,(%r15)
   25cc1:	je     26186 <handoff::old_take+0x926>
   25cc7:	cmp    $0x6,%ebx
   25cca:	mov    $0x6,%ecx
   25ccf:	cmovb  %ebx,%ecx
   25cd2:	mov    $0x1,%eax
   25cd7:	test   %ebx,%ebx
   25cd9:	je     25c64 <handoff::old_take+0x404>
   25cdb:	imul   %ecx,%ecx
   25cde:	mov    %ecx,%eax
   25ce0:	and    $0x5,%eax
   25ce3:	cmp    $0x8,%ecx
   25ce6:	jb     25d10 <handoff::old_take+0x4b0>
   25ce8:	and    $0x38,%ecx
   25ceb:	nopl   0x0(%rax,%rax,1)
   25cf0:	pause
   25cf2:	pause
   25cf4:	pause
   25cf6:	pause
   25cf8:	pause
   25cfa:	pause
   25cfc:	pause
   25cfe:	pause
   25d00:	add    $0xfffffff8,%ecx
   25d03:	jne    25cf0 <handoff::old_take+0x490>
   25d05:	test   %eax,%eax
   25d07:	je     25c60 <handoff::old_take+0x400>
   25d0d:	nopl   (%rax)
   25d10:	pause
   25d12:	dec    %eax
   25d14:	jne    25d10 <handoff::old_take+0x4b0>
   25d16:	jmp    25c60 <handoff::old_take+0x400>
   25d1b:	nopl   0x0(%rax,%rax,1)
   25d20:	cmp    %r13,%rsi
   25d23:	jne    25db0 <handoff::old_take+0x550>
   25d29:	lock orl $0x0,-0x40(%rsp)
   25d2f:	mov    0x80(%r15),%rax
   25d36:	mov    0x190(%r15),%rcx
   25d3d:	mov    %rcx,%rdx
   25d40:	not    %rdx
   25d43:	and    %rax,%rdx
   25d46:	cmp    %r13,%rdx
   25d49:	je     25e0b <handoff::old_take+0x5ab>
   25d4f:	cmp    $0x6,%ebx
   25d52:	mov    $0x6,%ecx
   25d57:	cmovb  %ebx,%ecx
   25d5a:	mov    $0x1,%eax
   25d5f:	test   %ebx,%ebx
   25d61:	je     25c64 <handoff::old_take+0x404>
   25d67:	imul   %ecx,%ecx
   25d6a:	mov    %ecx,%eax
   25d6c:	and    $0x5,%eax
   25d6f:	cmp    $0x8,%ecx
   25d72:	jb     25da0 <handoff::old_take+0x540>
   25d74:	and    $0x38,%ecx
   25d77:	nopw   0x0(%rax,%rax,1)
   25d80:	pause
   25d82:	pause
   25d84:	pause
   25d86:	pause
   25d88:	pause
   25d8a:	pause
   25d8c:	pause
   25d8e:	pause
   25d90:	add    $0xfffffff8,%ecx
   25d93:	jne    25d80 <handoff::old_take+0x520>
   25d95:	test   %eax,%eax
   25d97:	je     25c60 <handoff::old_take+0x400>
   25d9d:	nopl   (%rax)
   25da0:	pause
   25da2:	dec    %eax
   25da4:	jne    25da0 <handoff::old_take+0x540>
   25da6:	jmp    25c60 <handoff::old_take+0x400>
   25dab:	nopl   0x0(%rax,%rax,1)
   25db0:	cmp    $0x7,%ebx
   25db3:	jae    25c5c <handoff::old_take+0x3fc>
   25db9:	test   %ebx,%ebx
   25dbb:	je     25c60 <handoff::old_take+0x400>
   25dc1:	mov    %ebx,%ecx
   25dc3:	imul   %ebx,%ecx
   25dc6:	mov    %ecx,%eax
   25dc8:	and    $0x5,%eax
   25dcb:	cmp    $0x8,%ecx
   25dce:	jb     25e00 <handoff::old_take+0x5a0>
   25dd0:	and    $0x38,%ecx
   25dd3:	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   25de0:	pause
   25de2:	pause
   25de4:	pause
   25de6:	pause
   25de8:	pause
   25dea:	pause
   25dec:	pause
   25dee:	pause
   25df0:	add    $0xfffffff8,%ecx
   25df3:	jne    25de0 <handoff::old_take+0x580>
   25df5:	test   %eax,%eax
   25df7:	je     25c60 <handoff::old_take+0x400>
   25dfd:	nopl   (%rax)
   25e00:	pause
   25e02:	dec    %eax
   25e04:	jne    25e00 <handoff::old_take+0x5a0>
   25e06:	jmp    25c60 <handoff::old_take+0x400>
   25e0b:	mov    $0x1,%bl
   25e0d:	test   %rax,%rcx
   25e10:	jne    263f8 <handoff::old_take+0xb98>
   25e16:	mov    0x20(%rsp),%ebp
   25e1a:	cmp    $0xffffffff,%ebp
   25e1d:	je     25e41 <handoff::old_take+0x5e1>
   25e1f:	mov    0x18(%rsp),%r13
   25e24:	call   *0x4af96(%rip)        # 70dc0 <_DYNAMIC+0x338>
   25e2a:	cmp    %r13,%rax
   25e2d:	jne    25e38 <handoff::old_take+0x5d8>
   25e2f:	cmp    %ebp,%edx
   25e31:	jb     25e41 <handoff::old_take+0x5e1>
   25e33:	jmp    263f8 <handoff::old_take+0xb98>
   25e38:	cmp    %r13,%rax
   25e3b:	jge    263f8 <handoff::old_take+0xb98>
   25e41:	lea    0x300(%rsp),%rax
   25e49:	mov    %rax,0x1c0(%rsp)
   25e51:	mov    %r15,0x1c8(%rsp)
   25e59:	lea    0x18(%rsp),%rax
   25e5e:	mov    %rax,0x1d0(%rsp)
   25e66:	mov    $0xffffffffffffffe0,%rax
   25e6d:	cmpb   $0x1,%fs:0x8(%rax)
   25e72:	mov    %r12,%r13
   25e75:	jne    25f0d <handoff::old_take+0x6ad>
   25e7b:	mov    0x0(%r13),%rbp
   25e7f:	movq   $0x0,0x0(%r13)
   25e87:	test   %rbp,%rbp
   25e8a:	je     25f35 <handoff::old_take+0x6d5>
   25e90:	mov    %rbp,0x40(%rsp)
   25e95:	movq   $0x0,0x18(%rbp)
   25e9d:	movq   $0x0,0x20(%rbp)
   25ea5:	movq   $0x0,0x1c0(%rsp)
   25eb1:	lea    0x300(%rsp),%rax
   25eb9:	mov    %rax,0x70(%rsp)
   25ebe:	lea    0x1c8(%rsp),%rax
   25ec6:	movups (%rax),%xmm0
   25ec9:	lea    0x78(%rsp),%rax
   25ece:	movups %xmm0,(%rax)
   25ed1:	lea    0x70(%rsp),%rdi
   25ed6:	mov    %rbp,%rsi
   25ed9:	call   200f0 <<std::sync::mpmc::array::Channel<(u64, [u64; 38])>>::recv::{closure#1}>
   25ede:	mov    0x0(%r13),%rax
   25ee2:	mov    %rax,0x70(%rsp)
   25ee7:	mov    %rbp,0x0(%r13)
   25eeb:	test   %rax,%rax
   25eee:	je     25c58 <handoff::old_take+0x3f8>
   25ef4:	lock decq (%rax)
   25ef8:	jne    25c58 <handoff::old_take+0x3f8>
   25efe:	lea    0x70(%rsp),%rdi
   25f03:	call   28130 <<alloc::sync::Arc<std::sync::mpmc::context::Inner>>::drop_slow>
   25f08:	jmp    25c58 <handoff::old_take+0x3f8>
   25f0d:	mov    %r12,%rdi
   25f10:	xor    %esi,%esi
   25f12:	call   1ad90 <<std::sys::thread_local::native::lazy::Storage<core::cell::Cell<core::option::Option<std::sync::mpmc::context::Context>>, ()>>::get_or_init_slow::<<std::sync::mpmc::context::Context>::with::CONTEXT::__rust_std_internal_init_fn>>
   25f17:	mov    %rax,%r13
   25f1a:	test   %rax,%rax
   25f1d:	jne    25e7b <handoff::old_take+0x61b>
   25f23:	lea    0x1c0(%rsp),%rdi
   25f2b:	call   1de50 <<std::sync::mpmc::context::Context>::with::<<std::sync::mpmc::array::Channel<(u64, [u64; 38])>>::recv::{closure#1}, ()>::{closure#2}>
   25f30:	jmp    25c58 <handoff::old_take+0x3f8>
   25f35:	call   *0x4ad85(%rip)        # 70cc0 <_DYNAMIC+0x238>
   25f3b:	mov    %rax,%r13
   25f3e:	mov    %rax,0x40(%rsp)
   25f43:	movq   $0x0,0x1c0(%rsp)
   25f4f:	lea    0x300(%rsp),%rax
   25f57:	mov    %rax,0x70(%rsp)
   25f5c:	mov    %r15,0x78(%rsp)
   25f61:	lea    0x18(%rsp),%rax
   25f66:	mov    %rax,0x80(%rsp)
   25f6e:	lea    0x70(%rsp),%rdi
   25f73:	mov    %r13,%rsi
   25f76:	call   200f0 <<std::sync::mpmc::array::Channel<(u64, [u64; 38])>>::recv::{closure#1}>
   25f7b:	lock decq 0x0(%r13)
   25f80:	jne    25c58 <handoff::old_take+0x3f8>
   25f86:	lea    0x40(%rsp),%rdi
   25f8b:	call   28130 <<alloc::sync::Arc<std::sync::mpmc::context::Inner>>::drop_slow>
   25f90:	jmp    25c58 <handoff::old_take+0x3f8>
   25f95:	movl   $0xffffffff,0x1b8(%rsp)
   25fa0:	xorps  %xmm0,%xmm0
   25fa3:	movaps %xmm0,0x50(%rsp)
   25fa8:	movaps %xmm0,0x40(%rsp)
   25fad:	movq   $0x0,0x60(%rsp)
   25fb6:	mov    $0x1,%ecx
   25fbb:	xor    %eax,%eax
   25fbd:	lock cmpxchg %ecx,(%r15)
   25fc2:	jne    26860 <handoff::old_take+0x1000>
   25fc8:	mov    0x4ad21(%rip),%rax        # 70cf0 <_DYNAMIC+0x268>
   25fcf:	mov    (%rax),%rax
   25fd2:	movabs $0x7fffffffffffffff,%rcx
   25fdc:	test   %rcx,%rax
   25fdf:	jne    2686e <handoff::old_take+0x100e>
   25fe5:	xor    %r12d,%r12d
   25fe8:	movzbl 0x4(%r15),%eax
   25fed:	test   %al,%al
   25fef:	jne    26888 <handoff::old_take+0x1028>
   25ff5:	mov    0x18(%r15),%rax
   25ff9:	test   %rax,%rax
   25ffc:	je     2634b <handoff::old_take+0xaeb>
   26002:	mov    %fs:0x0,%rcx
   2600b:	lea    -0x78(%rcx),%rcx
   26012:	mov    0x10(%r15),%rdx
   26016:	shl    $0x3,%rax
   2601a:	lea    (%rax,%rax,2),%rsi
   2601e:	mov    $0x1,%ebx
   26023:	xor    %r14d,%r14d
   26026:	jmp    26040 <handoff::old_take+0x7e0>
   26028:	nopl   0x0(%rax,%rax,1)
   26030:	add    $0x18,%r14
   26034:	inc    %rbx
   26037:	cmp    %r14,%rsi
   2603a:	je     2634b <handoff::old_take+0xaeb>
   26040:	mov    (%rdx,%r14,1),%rdi
   26044:	cmp    %rcx,0x28(%rdi)
   26048:	je     26030 <handoff::old_take+0x7d0>
   2604a:	mov    0x8(%rdx,%r14,1),%r8
   2604f:	xor    %eax,%eax
   26051:	lock cmpxchg %r8,0x18(%rdi)
   26057:	jne    26030 <handoff::old_take+0x7d0>
   26059:	mov    0x10(%rdx,%r14,1),%rax
   2605e:	test   %rax,%rax
   26061:	je     26067 <handoff::old_take+0x807>
   26063:	mov    %rax,0x20(%rdi)
   26067:	mov    %rbp,0x8(%rsp)
   2606c:	mov    0x10(%rdi),%rdi
   26070:	mov    $0x1,%eax
   26075:	xchg   %eax,0x28(%rdi)
   26078:	cmp    $0xffffffff,%eax
   2607b:	jne    26087 <handoff::old_take+0x827>
   2607d:	add    $0x28,%rdi
   26081:	call   *0x4ae09(%rip)        # 70e90 <_DYNAMIC+0x408>
   26087:	lea    -0x1(%rbx),%r13
   2608b:	mov    0x18(%r15),%rbp
   2608f:	cmp    %rbp,%r13
   26092:	jae    269b7 <handoff::old_take+0x1157>
   26098:	mov    0x10(%r15),%rax
   2609c:	lea    (%rax,%r14,1),%rdi
   260a0:	mov    (%rax,%r14,1),%rcx
   260a4:	movups 0x8(%rax,%r14,1),%xmm0
   260aa:	movaps %xmm0,0x70(%rsp)
   260af:	lea    (%rax,%r14,1),%rsi
   260b3:	add    $0x18,%rsi
   260b7:	mov    %rcx,%r14
   260ba:	mov    %rbp,%rax
   260bd:	sub    %rbx,%rax
   260c0:	shl    $0x3,%rax
   260c4:	lea    (%rax,%rax,2),%rdx
   260c8:	call   *0x4ac02(%rip)        # 70cd0 <memmove@GLIBC_2.2.5>
   260ce:	dec    %rbp
   260d1:	mov    %rbp,0x18(%r15)
   260d5:	test   %r14,%r14
   260d8:	je     269b7 <handoff::old_take+0x1157>
   260de:	movaps 0x70(%rsp),%xmm0
   260e3:	movups %xmm0,0x308(%rsp)
   260eb:	mov    %r14,0x300(%rsp)
   260f3:	mov    0x310(%rsp),%rax
   260fb:	mov    %rax,0x60(%rsp)
   26100:	test   %r12b,%r12b
   26103:	jne    26122 <handoff::old_take+0x8c2>
   26105:	mov    0x4abe4(%rip),%rax        # 70cf0 <_DYNAMIC+0x268>
   2610c:	mov    (%rax),%rax
   2610f:	movabs $0x7fffffffffffffff,%rcx
   26119:	test   %rcx,%rax
   2611c:	jne    2696f <handoff::old_take+0x110f>
   26122:	xor    %eax,%eax
   26124:	xchg   %eax,(%r15)
   26127:	cmp    $0x2,%eax
   2612a:	mov    0x8(%rsp),%rbp
   2612f:	je     26949 <handoff::old_take+0x10e9>
   26135:	mov    0x60(%rsp),%r15
   2613a:	test   %r15,%r15
   2613d:	je     26698 <handoff::old_take+0xe38>
   26143:	cmpb   $0x0,0x141(%r15)
   2614b:	je     266ab <handoff::old_take+0xe4b>
   26151:	testb  $0x1,(%r15)
   26155:	movq   $0x0,(%r15)
   2615c:	je     2699f <handoff::old_take+0x113f>
   26162:	lea    0x8(%r15),%rsi
   26166:	lea    0x1c0(%rsp),%rdi
   2616e:	mov    $0x138,%edx
   26173:	call   *0x4ab5f(%rip)        # 70cd8 <memcpy@GLIBC_2.14>
   26179:	movb   $0x1,0x140(%r15)
   26181:	jmp    266f6 <handoff::old_take+0xe96>
   26186:	add    %rcx,%rbp
   26189:	mov    %rbp,0x300(%rsp)
   26191:	add    0x188(%r15),%r13
   26198:	mov    %r13,0x308(%rsp)
   261a0:	lea    0x8(%rbp),%rsi
   261a4:	lea    0x1c0(%rsp),%rdi
   261ac:	mov    $0x138,%edx
   261b1:	call   *0x4ab21(%rip)        # 70cd8 <memcpy@GLIBC_2.14>
   261b7:	mov    %r13,0x0(%rbp)
   261bb:	add    $0x100,%r15
   261c2:	mov    %r15,%rdi
   261c5:	call   27020 <<std::sync::mpmc::waker::SyncWaker>::notify>
   261ca:	jmp    263de <handoff::old_take+0xb7e>
   261cf:	cmp    $0x1e,%ebp
   261d2:	jne    26206 <handoff::old_take+0x9a6>
   261d4:	mov    0x26c0(%r13),%rax
   261db:	test   %rax,%rax
   261de:	je     26626 <handoff::old_take+0xdc6>
   261e4:	and    $0xfffffffffffffffe,%r12
   261e8:	mov    0x26c0(%rax),%rcx
   261ef:	xor    %edx,%edx
   261f1:	test   %rcx,%rcx
   261f4:	setne  %dl
   261f7:	lea    (%r12,%rdx,1),%rcx
   261fb:	add    $0x2,%rcx
   261ff:	mov    %rax,0x8(%r15)
   26203:	mov    %rcx,(%r15)
   26206:	mov    %r13,0x310(%rsp)
   2620e:	mov    %rbp,0x318(%rsp)
   26216:	lea    0x0(,%rbp,4),%rax
   2621e:	add    %rbp,%rax
   26221:	shl    $0x6,%eax
   26224:	lea    (%rax,%r13,1),%r15
   26228:	mov    0x138(%r13,%rax,1),%rax
   26230:	test   $0x1,%al
   26232:	jne    262a8 <handoff::old_take+0xa48>
   26234:	xor    %ebx,%ebx
   26236:	xor    %r14d,%r14d
   26239:	jmp    2625f <handoff::old_take+0x9ff>
   2623b:	nopl   0x0(%rax,%rax,1)
   26240:	call   *0x4ab0a(%rip)        # 70d50 <_DYNAMIC+0x2c8>
   26246:	lea    0x1(,%r14,2),%eax
   2624e:	inc    %r14d
   26251:	mov    0x138(%r15),%rcx
   26258:	add    %eax,%ebx
   2625a:	test   $0x1,%cl
   2625d:	jne    262a8 <handoff::old_take+0xa48>
   2625f:	cmp    $0x7,%r14d
   26263:	jae    26240 <handoff::old_take+0x9e0>
   26265:	test   %r14d,%r14d
   26268:	je     26246 <handoff::old_take+0x9e6>
   2626a:	lea    -0x1(%rbx),%ecx
   2626d:	mov    %ebx,%eax
   2626f:	and    $0x7,%eax
   26272:	cmp    $0x7,%ecx
   26275:	jb     262a0 <handoff::old_take+0xa40>
   26277:	mov    %ebx,%ecx
   26279:	and    $0xfffffff8,%ecx
   2627c:	nopl   0x0(%rax)
   26280:	pause
   26282:	pause
   26284:	pause
   26286:	pause
   26288:	pause
   2628a:	pause
   2628c:	pause
   2628e:	pause
   26290:	add    $0xfffffff8,%ecx
   26293:	jne    26280 <handoff::old_take+0xa20>
   26295:	test   %eax,%eax
   26297:	je     26246 <handoff::old_take+0x9e6>
   26299:	nopl   0x0(%rax)
   262a0:	pause
   262a2:	dec    %eax
   262a4:	jne    262a0 <handoff::old_take+0xa40>
   262a6:	jmp    26246 <handoff::old_take+0x9e6>
   262a8:	lea    0x1c0(%rsp),%rdi
   262b0:	mov    $0x138,%edx
   262b5:	mov    %r15,%rsi
   262b8:	call   *0x4aa1a(%rip)        # 70cd8 <memcpy@GLIBC_2.14>
   262be:	lea    0x1(%rbp),%rcx
   262c2:	cmp    $0x1f,%rcx
   262c6:	jne    2639b <handoff::old_take+0xb3b>
   262cc:	xor    %ecx,%ecx
   262ce:	jmp    262de <handoff::old_take+0xa7e>
   262d0:	add    $0x2,%rcx
   262d4:	cmp    $0x1e,%rcx
   262d8:	je     263cb <handoff::old_take+0xb6b>
   262de:	lea    (%rcx,%rcx,4),%rsi
   262e2:	shl    $0x6,%rsi
   262e6:	lea    (%rsi,%r13,1),%rdx
   262ea:	mov    0x138(%r13,%rsi,1),%rax
   262f2:	test   $0x2,%al
   262f4:	jne    2631a <handoff::old_take+0xaba>
   262f6:	mov    0x138(%rdx),%rax
   262fd:	nopl   (%rax)
   26300:	mov    %rax,%rdi
   26303:	or     $0x4,%rdi
   26307:	lock cmpxchg %rdi,0x138(%rdx)
   26310:	jne    26300 <handoff::old_take+0xaa0>
   26312:	test   $0x2,%al
   26314:	je     263de <handoff::old_take+0xb7e>
   2631a:	mov    0x278(%r13,%rsi,1),%rax
   26322:	test   $0x2,%al
   26324:	jne    262d0 <handoff::old_take+0xa70>
   26326:	mov    0x278(%rdx),%rax
   2632d:	nopl   (%rax)
   26330:	mov    %rax,%rsi
   26333:	or     $0x4,%rsi
   26337:	lock cmpxchg %rsi,0x278(%rdx)
   26340:	jne    26330 <handoff::old_take+0xad0>
   26342:	test   $0x2,%al
   26344:	jne    262d0 <handoff::old_take+0xa70>
   26346:	jmp    263de <handoff::old_take+0xb7e>
   2634b:	cmpb   $0x0,0x68(%r15)
   26350:	je     2646b <handoff::old_take+0xc0b>
   26356:	movw   $0x101,0x70(%rsp)
   2635d:	test   %r12b,%r12b
   26360:	jne    2637f <handoff::old_take+0xb1f>
   26362:	mov    0x4a987(%rip),%rax        # 70cf0 <_DYNAMIC+0x268>
   26369:	mov    (%rax),%rax
   2636c:	movabs $0x7fffffffffffffff,%rcx
   26376:	test   %rcx,%rax
   26379:	jne    26957 <handoff::old_take+0x10f7>
   2637f:	xor    %eax,%eax
   26381:	xchg   %eax,(%r15)
   26384:	cmp    $0x2,%eax
   26387:	jne    26727 <handoff::old_take+0xec7>
   2638d:	mov    %r15,%rdi
   26390:	call   *0x4a972(%rip)        # 70d08 <_DYNAMIC+0x280>
   26396:	jmp    26727 <handoff::old_take+0xec7>
   2639b:	mov    0x138(%r15),%rax
   263a2:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   263b0:	mov    %rax,%rdx
   263b3:	or     $0x2,%rdx
   263b7:	lock cmpxchg %rdx,0x138(%r15)
   263c0:	jne    263b0 <handoff::old_take+0xb50>
   263c2:	test   $0x4,%al
   263c4:	je     263de <handoff::old_take+0xb7e>
   263c6:	cmp    $0x1c,%ebp
   263c9:	jbe    26429 <handoff::old_take+0xbc9>
   263cb:	mov    $0x26c8,%esi
   263d0:	mov    $0x8,%edx
   263d5:	mov    %r13,%rdi
   263d8:	call   *0x4a902(%rip)        # 70ce0 <_DYNAMIC+0x258>
   263de:	lea    0x78(%rsp),%rdi
   263e3:	lea    0x1c0(%rsp),%rsi
   263eb:	mov    $0x138,%edx
   263f0:	call   *0x4a8e2(%rip)        # 70cd8 <memcpy@GLIBC_2.14>
   263f6:	xor    %ebx,%ebx
   263f8:	mov    0x8(%rsp),%rbp
   263fd:	test   %bl,%bl
   263ff:	je     26734 <handoff::old_take+0xed4>
   26405:	xor    %eax,%eax
   26407:	mov    %rax,0x0(%rbp)
   2640b:	cmpb   $0x0,0x4(%rsp)
   26410:	je     26758 <handoff::old_take+0xef8>
   26416:	jmp    26775 <handoff::old_take+0xf15>
   2641b:	nopl   0x0(%rax,%rax,1)
   26420:	inc    %rcx
   26423:	cmp    $0x1e,%rcx
   26427:	je     263cb <handoff::old_take+0xb6b>
   26429:	lea    (%rcx,%rcx,4),%rdx
   2642d:	shl    $0x6,%rdx
   26431:	mov    0x138(%r13,%rdx,1),%rax
   26439:	test   $0x2,%al
   2643b:	jne    26420 <handoff::old_take+0xbc0>
   2643d:	add    %r13,%rdx
   26440:	mov    0x138(%rdx),%rax
   26447:	nopw   0x0(%rax,%rax,1)
   26450:	mov    %rax,%rsi
   26453:	or     $0x4,%rsi
   26457:	lock cmpxchg %rsi,0x138(%rdx)
   26460:	jne    26450 <handoff::old_take+0xbf0>
   26462:	test   $0x2,%al
   26464:	jne    26420 <handoff::old_take+0xbc0>
   26466:	jmp    263de <handoff::old_take+0xb7e>
   2646b:	lea    0x40(%rsp),%rax
   26470:	mov    %rax,0x18(%rsp)
   26475:	lea    0x1b0(%rsp),%r14
   2647d:	mov    %r14,0x20(%rsp)
   26482:	mov    %r15,0x28(%rsp)
   26487:	mov    %r15,0x30(%rsp)
   2648c:	mov    %r12b,0x38(%rsp)
   26491:	mov    $0xffffffffffffffe0,%rax
   26498:	mov    %fs:0x0,%r13
   264a1:	add    %rax,%r13
   264a4:	cmpb   $0x1,%fs:0x8(%rax)
   264a9:	jne    268bc <handoff::old_take+0x105c>
   264af:	mov    %rbp,%rbx
   264b2:	mov    0x0(%r13),%rbp
   264b6:	movq   $0x0,0x0(%r13)
   264be:	test   %rbp,%rbp
   264c1:	je     268d7 <handoff::old_take+0x1077>
   264c7:	lea    0x39(%rsp),%rax
   264cc:	mov    %rbp,0x300(%rsp)
   264d4:	movq   $0x0,0x18(%rbp)
   264dc:	movq   $0x0,0x20(%rbp)
   264e4:	movb   $0x2,0x38(%rsp)
   264e9:	movups 0x18(%rsp),%xmm0
   264ee:	movups 0x28(%rsp),%xmm1
   264f3:	movaps %xmm0,0x1c0(%rsp)
   264fb:	movaps %xmm1,0x1d0(%rsp)
   26503:	mov    %r12b,0x1e0(%rsp)
   2650b:	mov    (%rax),%ecx
   2650d:	mov    0x3(%rax),%eax
   26510:	mov    %ecx,0x1e1(%rsp)
   26517:	mov    %eax,0x1e4(%rsp)
   2651e:	lea    0x70(%rsp),%rdi
   26523:	lea    0x1c0(%rsp),%rsi
   2652b:	mov    %rbp,%rdx
   2652e:	call   1f2d0 <<std::sync::mpmc::zero::Channel<(u64, [u64; 38])>>::recv::{closure#1}>
   26533:	mov    0x0(%r13),%rax
   26537:	mov    %rax,0x1c0(%rsp)
   2653f:	mov    %rbp,0x0(%r13)
   26543:	test   %rax,%rax
   26546:	mov    %rbx,%rbp
   26549:	je     2655e <handoff::old_take+0xcfe>
   2654b:	lock decq (%rax)
   2654f:	jne    2655e <handoff::old_take+0xcfe>
   26551:	lea    0x1c0(%rsp),%rdi
   26559:	call   28130 <<alloc::sync::Arc<std::sync::mpmc::context::Inner>>::drop_slow>
   2655e:	lea    0x1c0(%rsp),%rdi
   26566:	lea    0x70(%rsp),%rsi
   2656b:	mov    $0x140,%edx
   26570:	call   *0x4a762(%rip)        # 70cd8 <memcpy@GLIBC_2.14>
   26576:	movzbl 0x1c0(%rsp),%ebx
   2657e:	lea    0x1c1(%rsp),%rsi
   26586:	lea    0x300(%rsp),%rdi
   2658e:	mov    $0x13f,%edx
   26593:	call   *0x4a73f(%rip)        # 70cd8 <memcpy@GLIBC_2.14>
   26599:	cmp    $0x2,%bl
   2659c:	jne    265bb <handoff::old_take+0xd5b>
   2659e:	lea    0x70(%rsp),%rdi
   265a3:	lea    0x18(%rsp),%rsi
   265a8:	call   1dcc0 <<std::sync::mpmc::context::Context>::with::<<std::sync::mpmc::zero::Channel<(u64, [u64; 38])>>::recv::{closure#1}, core::result::Result<(u64, [u64; 38]), std::sync::mpsc::RecvTimeoutError>>::{closure#2}>
   265ad:	movzbl 0x38(%rsp),%eax
   265b2:	cmp    $0x2,%al
   265b4:	jne    265e4 <handoff::old_take+0xd84>
   265b6:	jmp    26727 <handoff::old_take+0xec7>
   265bb:	mov    %bl,0x70(%rsp)
   265bf:	lea    0x71(%rsp),%rdi
   265c4:	lea    0x300(%rsp),%rsi
   265cc:	mov    $0x13f,%edx
   265d1:	call   *0x4a701(%rip)        # 70cd8 <memcpy@GLIBC_2.14>
   265d7:	movzbl 0x38(%rsp),%eax
   265dc:	cmp    $0x2,%al
   265de:	je     26727 <handoff::old_take+0xec7>
   265e4:	mov    0x30(%rsp),%r15
   265e9:	test   $0x1,%al
   265eb:	jne    2660a <handoff::old_take+0xdaa>
   265ed:	mov    0x4a6fc(%rip),%rax        # 70cf0 <_DYNAMIC+0x268>
   265f4:	mov    (%rax),%rax
   265f7:	movabs $0x7fffffffffffffff,%rcx
   26601:	test   %rcx,%rax
   26604:	jne    26987 <handoff::old_take+0x1127>
   2660a:	xor    %eax,%eax
   2660c:	xchg   %eax,(%r15)
   2660f:	cmp    $0x2,%eax
   26612:	jne    26727 <handoff::old_take+0xec7>
   26618:	mov    %r15,%rdi
   2661b:	call   *0x4a6e7(%rip)        # 70d08 <_DYNAMIC+0x280>
   26621:	jmp    26727 <handoff::old_take+0xec7>
   26626:	xor    %ebx,%ebx
   26628:	xor    %r14d,%r14d
   2662b:	jmp    26653 <handoff::old_take+0xdf3>
   2662d:	nopl   (%rax)
   26630:	call   *0x4a71a(%rip)        # 70d50 <_DYNAMIC+0x2c8>
   26636:	lea    0x1(,%r14,2),%ecx
   2663e:	inc    %r14d
   26641:	mov    0x26c0(%r13),%rax
   26648:	add    %ecx,%ebx
   2664a:	test   %rax,%rax
   2664d:	jne    261e4 <handoff::old_take+0x984>
   26653:	cmp    $0x7,%r14d
   26657:	jae    26630 <handoff::old_take+0xdd0>
   26659:	test   %r14d,%r14d
   2665c:	je     26636 <handoff::old_take+0xdd6>
   2665e:	lea    -0x1(%rbx),%ecx
   26661:	mov    %ebx,%eax
   26663:	and    $0x7,%eax
   26666:	cmp    $0x7,%ecx
   26669:	jb     26690 <handoff::old_take+0xe30>
   2666b:	mov    %ebx,%ecx
   2666d:	and    $0xfffffff8,%ecx
   26670:	pause
   26672:	pause
   26674:	pause
   26676:	pause
   26678:	pause
   2667a:	pause
   2667c:	pause
   2667e:	pause
   26680:	add    $0xfffffff8,%ecx
   26683:	jne    26670 <handoff::old_take+0xe10>
   26685:	test   %eax,%eax
   26687:	je     26636 <handoff::old_take+0xdd6>
   26689:	nopl   0x0(%rax)
   26690:	pause
   26692:	dec    %eax
   26694:	jne    26690 <handoff::old_take+0xe30>
   26696:	jmp    26636 <handoff::old_take+0xdd6>
   26698:	movb   $0x1,0x71(%rsp)
   2669d:	mov    $0x1,%al
   2669f:	mov    %al,0x70(%rsp)
   266a3:	lock decq (%r14)
   266a7:	je     2671a <handoff::old_take+0xeba>
   266a9:	jmp    26727 <handoff::old_take+0xec7>
   266ab:	movzbl 0x140(%r15),%eax
   266b3:	test   %al,%al
   266b5:	je     26799 <handoff::old_take+0xf39>
   266bb:	testb  $0x1,(%r15)
   266bf:	movq   $0x0,(%r15)
   266c6:	je     269a8 <handoff::old_take+0x1148>
   266cc:	lea    0x8(%r15),%rsi
   266d0:	lea    0x1c0(%rsp),%rdi
   266d8:	mov    $0x138,%edx
   266dd:	call   *0x4a5f5(%rip)        # 70cd8 <memcpy@GLIBC_2.14>
   266e3:	mov    $0x148,%esi
   266e8:	mov    $0x8,%edx
   266ed:	mov    %r15,%rdi
   266f0:	call   *0x4a5ea(%rip)        # 70ce0 <_DYNAMIC+0x258>
   266f6:	lea    0x78(%rsp),%rdi
   266fb:	lea    0x1c0(%rsp),%rsi
   26703:	mov    $0x138,%edx
   26708:	call   *0x4a5ca(%rip)        # 70cd8 <memcpy@GLIBC_2.14>
   2670e:	xor    %eax,%eax
   26710:	mov    %al,0x70(%rsp)
   26714:	lock decq (%r14)
   26718:	jne    26727 <handoff::old_take+0xec7>
   2671a:	lea    0x300(%rsp),%rdi
   26722:	call   28130 <<alloc::sync::Arc<std::sync::mpmc::context::Inner>>::drop_slow>
   26727:	movzbl 0x70(%rsp),%ebx
   2672c:	test   %bl,%bl
   2672e:	jne    26405 <handoff::old_take+0xba5>
   26734:	lea    0x78(%rsp),%rsi
   26739:	lea    0x8(%rbp),%rdi
   2673d:	mov    $0x138,%edx
   26742:	call   *0x4a590(%rip)        # 70cd8 <memcpy@GLIBC_2.14>
   26748:	mov    $0x1,%eax
   2674d:	mov    %rax,0x0(%rbp)
   26751:	cmpb   $0x0,0x4(%rsp)
   26756:	jne    26775 <handoff::old_take+0xf15>
   26758:	mov    0x4a591(%rip),%rax        # 70cf0 <_DYNAMIC+0x268>
   2675f:	mov    (%rax),%rax
   26762:	movabs $0x7fffffffffffffff,%rcx
   2676c:	test   %rcx,%rax
   2676f:	jne    26844 <handoff::old_take+0xfe4>
   26775:	xor    %eax,%eax
   26777:	mov    0x10(%rsp),%rdi
   2677c:	xchg   %eax,(%rdi)
   2677e:	cmp    $0x2,%eax
   26781:	je     26839 <handoff::old_take+0xfd9>
   26787:	add    $0x448,%rsp
   2678e:	pop    %rbx
   2678f:	pop    %r12
   26791:	pop    %r13
   26793:	pop    %r14
   26795:	pop    %r15
   26797:	pop    %rbp
   26798:	ret
   26799:	xor    %ebx,%ebx
   2679b:	mov    0x4a5ae(%rip),%r12        # 70d50 <_DYNAMIC+0x2c8>
   267a2:	xor    %r13d,%r13d
   267a5:	jmp    267c7 <handoff::old_take+0xf67>
   267a7:	call   *%r12
   267aa:	lea    0x1(,%r13,2),%eax
   267b2:	inc    %r13d
   267b5:	movzbl 0x140(%r15),%ecx
   267bd:	add    %eax,%ebx
   267bf:	test   %cl,%cl
   267c1:	jne    266bb <handoff::old_take+0xe5b>
   267c7:	cmp    $0x7,%r13d
   267cb:	jae    267a7 <handoff::old_take+0xf47>
   267cd:	test   %r13d,%r13d
   267d0:	je     267aa <handoff::old_take+0xf4a>
   267d2:	lea    -0x1(%rbx),%ecx
   267d5:	mov    %ebx,%eax
   267d7:	and    $0x7,%eax
   267da:	cmp    $0x7,%ecx
   267dd:	jb     26810 <handoff::old_take+0xfb0>
   267df:	mov    %ebx,%ecx
   267e1:	and    $0xfffffff8,%ecx
   267e4:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   267f0:	pause
   267f2:	pause
   267f4:	pause
   267f6:	pause
   267f8:	pause
   267fa:	pause
   267fc:	pause
   267fe:	pause
   26800:	add    $0xfffffff8,%ecx
   26803:	jne    267f0 <handoff::old_take+0xf90>
   26805:	test   %eax,%eax
   26807:	je     267aa <handoff::old_take+0xf4a>
   26809:	nopl   0x0(%rax)
   26810:	pause
   26812:	dec    %eax
   26814:	jne    26810 <handoff::old_take+0xfb0>
   26816:	jmp    267aa <handoff::old_take+0xf4a>
   26818:	mov    0x10(%rsp),%rdi
   2681d:	call   *0x4a4d5(%rip)        # 70cf8 <_DYNAMIC+0x270>
   26823:	jmp    2588a <handoff::old_take+0x2a>
   26828:	call   *0x4a4d2(%rip)        # 70d00 <_DYNAMIC+0x278>
   2682e:	xor    $0x1,%al
   26830:	mov    %eax,0x4(%rsp)
   26834:	jmp    258a8 <handoff::old_take+0x48>
   26839:	call   *0x4a4c9(%rip)        # 70d08 <_DYNAMIC+0x280>
   2683f:	jmp    26787 <handoff::old_take+0xf27>
   26844:	call   *0x4a4b6(%rip)        # 70d00 <_DYNAMIC+0x278>
   2684a:	test   %al,%al
   2684c:	jne    26775 <handoff::old_take+0xf15>
   26852:	mov    0x10(%rsp),%rax
   26857:	movb   $0x1,0x4(%rax)
   2685b:	jmp    26775 <handoff::old_take+0xf15>
   26860:	mov    %r15,%rdi
   26863:	call   *0x4a48f(%rip)        # 70cf8 <_DYNAMIC+0x270>
   26869:	jmp    25fc8 <handoff::old_take+0x768>
   2686e:	call   *0x4a48c(%rip)        # 70d00 <_DYNAMIC+0x278>
   26874:	mov    %eax,%r12d
   26877:	xor    $0x1,%r12b
   2687b:	movzbl 0x4(%r15),%eax
   26880:	test   %al,%al
   26882:	je     25ff5 <handoff::old_take+0x795>
   26888:	mov    %r15,0x70(%rsp)
   2688d:	mov    %r12b,0x78(%rsp)
   26892:	lea    -0x1a851(%rip),%rdi        # c048 <GCC_except_table142+0x2320>
   26899:	lea    0x47c58(%rip),%rcx        # 6e4f8 <__frame_dummy_init_array_entry+0x5f0>
   268a0:	lea    0x47e81(%rip),%r8        # 6e728 <__frame_dummy_init_array_entry+0x820>
   268a7:	lea    0x70(%rsp),%rdx
   268ac:	mov    $0x2b,%esi
   268b1:	call   *0x4a4e9(%rip)        # 70da0 <_DYNAMIC+0x318>
   268b7:	jmp    269ca <handoff::old_take+0x116a>
   268bc:	mov    %r13,%rdi
   268bf:	xor    %esi,%esi
   268c1:	call   1ad90 <<std::sys::thread_local::native::lazy::Storage<core::cell::Cell<core::option::Option<std::sync::mpmc::context::Context>>, ()>>::get_or_init_slow::<<std::sync::mpmc::context::Context>::with::CONTEXT::__rust_std_internal_init_fn>>
   268c6:	mov    %rax,%r13
   268c9:	test   %rax,%rax
   268cc:	jne    264af <handoff::old_take+0xc4f>
   268d2:	jmp    2659e <handoff::old_take+0xd3e>
   268d7:	call   *0x4a3e3(%rip)        # 70cc0 <_DYNAMIC+0x238>
   268dd:	mov    %rax,%r13
   268e0:	mov    %rbx,%rbp
   268e3:	mov    %rax,0x300(%rsp)
   268eb:	movb   $0x2,0x38(%rsp)
   268f0:	lea    0x40(%rsp),%rax
   268f5:	mov    %rax,0x70(%rsp)
   268fa:	mov    %r14,0x78(%rsp)
   268ff:	mov    %r15,0x80(%rsp)
   26907:	mov    %r15,0x88(%rsp)
   2690f:	mov    %r12b,0x90(%rsp)
   26917:	lea    0x1c0(%rsp),%rdi
   2691f:	lea    0x70(%rsp),%rsi
   26924:	mov    %r13,%rdx
   26927:	call   1f2d0 <<std::sync::mpmc::zero::Channel<(u64, [u64; 38])>>::recv::{closure#1}>
   2692c:	lock decq 0x0(%r13)
   26931:	jne    26576 <handoff::old_take+0xd16>
   26937:	lea    0x300(%rsp),%rdi
   2693f:	call   28130 <<alloc::sync::Arc<std::sync::mpmc::context::Inner>>::drop_slow>
   26944:	jmp    26576 <handoff::old_take+0xd16>
   26949:	mov    %r15,%rdi
   2694c:	call   *0x4a3b6(%rip)        # 70d08 <_DYNAMIC+0x280>
   26952:	jmp    26135 <handoff::old_take+0x8d5>
   26957:	call   *0x4a3a3(%rip)        # 70d00 <_DYNAMIC+0x278>
   2695d:	test   %al,%al
   2695f:	jne    2637f <handoff::old_take+0xb1f>
   26965:	movb   $0x1,0x4(%r15)
   2696a:	jmp    2637f <handoff::old_take+0xb1f>
   2696f:	call   *0x4a38b(%rip)        # 70d00 <_DYNAMIC+0x278>
   26975:	test   %al,%al
   26977:	jne    26122 <handoff::old_take+0x8c2>
   2697d:	movb   $0x1,0x4(%r15)
   26982:	jmp    26122 <handoff::old_take+0x8c2>
   26987:	call   *0x4a373(%rip)        # 70d00 <_DYNAMIC+0x278>
   2698d:	test   %al,%al
   2698f:	jne    2660a <handoff::old_take+0xdaa>
   26995:	movb   $0x1,0x4(%r15)
   2699a:	jmp    2660a <handoff::old_take+0xdaa>
   2699f:	lea    0x47d6a(%rip),%rdi        # 6e710 <__frame_dummy_init_array_entry+0x808>
   269a6:	jmp    269af <handoff::old_take+0x114f>
   269a8:	lea    0x47d49(%rip),%rdi        # 6e6f8 <__frame_dummy_init_array_entry+0x7f0>
   269af:	call   *0x4a3d3(%rip)        # 70d88 <_DYNAMIC+0x300>
   269b5:	jmp    269ca <handoff::old_take+0x116a>
   269b7:	lea    0x47bba(%rip),%rdx        # 6e578 <__frame_dummy_init_array_entry+0x670>
   269be:	mov    %r13,%rdi
   269c1:	mov    %rbp,%rsi
   269c4:	call   *0x4a416(%rip)        # 70de0 <_DYNAMIC+0x358>
   269ca:	ud2
   269cc:	mov    %rax,%rbx
   269cf:	lock decq 0x0(%r13)
   269d4:	jne    26a4f <handoff::old_take+0x11ef>
   269d6:	lea    0x300(%rsp),%rdi
   269de:	call   28130 <<alloc::sync::Arc<std::sync::mpmc::context::Inner>>::drop_slow>
   269e3:	jmp    26a4f <handoff::old_take+0x11ef>
   269e5:	mov    %rax,%rbx
   269e8:	lock decq 0x0(%r13)
   269ed:	jne    26ae4 <handoff::old_take+0x1284>
   269f3:	lea    0x40(%rsp),%rdi
   269f8:	call   28130 <<alloc::sync::Arc<std::sync::mpmc::context::Inner>>::drop_slow>
   269fd:	jmp    26ae4 <handoff::old_take+0x1284>
   26a02:	mov    %rax,%rbx
   26a05:	lock decq 0x0(%r13)
   26a0a:	jne    26ae4 <handoff::old_take+0x1284>
   26a10:	lea    0x40(%rsp),%rdi
   26a15:	call   28130 <<alloc::sync::Arc<std::sync::mpmc::context::Inner>>::drop_slow>
   26a1a:	jmp    26ae4 <handoff::old_take+0x1284>
   26a1f:	mov    %rax,%rbx
   26a22:	movzbl %r12b,%esi
   26a26:	mov    %r15,%rdi
   26a29:	call   1c960 <core::ptr::drop_glue::<std::sync::poison::mutex::MutexGuard<handoff::Queued<(u64, [u64; 38])>>>>
   26a2e:	jmp    26ae4 <handoff::old_take+0x1284>
   26a33:	mov    %rax,%rbx
   26a36:	jmp    26a4f <handoff::old_take+0x11ef>
   26a38:	mov    %rax,%rbx
   26a3b:	lock decq 0x0(%rbp)
   26a40:	jne    26a4f <handoff::old_take+0x11ef>
   26a42:	lea    0x300(%rsp),%rdi
   26a4a:	call   28130 <<alloc::sync::Arc<std::sync::mpmc::context::Inner>>::drop_slow>
   26a4f:	mov    0x30(%rsp),%rdi
   26a54:	mov    0x38(%rsp),%esi
   26a58:	call   1bc20 <core::ptr::drop_glue::<core::option::Option<<std::sync::mpmc::zero::Channel<(u64, [u64; 38])>>::recv::{closure#1}>>>
   26a5d:	jmp    26ae4 <handoff::old_take+0x1284>
   26a62:	call   *0x4a2b0(%rip)        # 70d18 <_DYNAMIC+0x290>
   26a68:	call   *0x4a2aa(%rip)        # 70d18 <_DYNAMIC+0x290>
   26a6e:	jmp    26ab9 <handoff::old_take+0x1259>
   26a70:	jmp    26ab4 <handoff::old_take+0x1254>
   26a72:	mov    %rax,%rbx
   26a75:	lock decq 0x0(%rbp)
   26a7a:	jne    26ae4 <handoff::old_take+0x1284>
   26a7c:	lea    0x40(%rsp),%rdi
   26a81:	call   28130 <<alloc::sync::Arc<std::sync::mpmc::context::Inner>>::drop_slow>
   26a86:	jmp    26ae4 <handoff::old_take+0x1284>
   26a88:	call   *0x4a28a(%rip)        # 70d18 <_DYNAMIC+0x290>
   26a8e:	jmp    26ab4 <handoff::old_take+0x1254>
   26a90:	mov    %rax,%rbx
   26a93:	lock decq 0x0(%rbp)
   26a98:	jne    26ae4 <handoff::old_take+0x1284>
   26a9a:	lea    0x40(%rsp),%rdi
   26a9f:	call   28130 <<alloc::sync::Arc<std::sync::mpmc::context::Inner>>::drop_slow>
   26aa4:	jmp    26ae4 <handoff::old_take+0x1284>
   26aa6:	call   *0x4a26c(%rip)        # 70d18 <_DYNAMIC+0x290>
   26aac:	jmp    26ab4 <handoff::old_take+0x1254>
   26aae:	jmp    26ab4 <handoff::old_take+0x1254>
   26ab0:	jmp    26ab4 <handoff::old_take+0x1254>
   26ab2:	jmp    26ab4 <handoff::old_take+0x1254>
   26ab4:	mov    %rax,%rbx
   26ab7:	jmp    26ae4 <handoff::old_take+0x1284>
   26ab9:	mov    %rax,%rbx
   26abc:	lock decq (%r14)
   26ac0:	jne    26ae4 <handoff::old_take+0x1284>
   26ac2:	lea    0x300(%rsp),%rdi
   26aca:	call   28130 <<alloc::sync::Arc<std::sync::mpmc::context::Inner>>::drop_slow>
   26acf:	jmp    26ae4 <handoff::old_take+0x1284>
   26ad1:	call   *0x4a241(%rip)        # 70d18 <_DYNAMIC+0x290>
   26ad7:	mov    %rax,%rbx
   26ada:	lea    0x70(%rsp),%rdi
   26adf:	call   1c5f0 <core::ptr::drop_glue::<std::sync::poison::PoisonError<std::sync::poison::mutex::MutexGuard<handoff::State>>>>
   26ae4:	movzbl 0x4(%rsp),%esi
   26ae9:	mov    0x10(%rsp),%rdi
   26aee:	call   1c960 <core::ptr::drop_glue::<std::sync::poison::mutex::MutexGuard<handoff::Queued<(u64, [u64; 38])>>>>
   26af3:	mov    %rbx,%rdi
   26af6:	call   6cec0 <_Unwind_Resume@plt>
   26afb:	call   *0x4a217(%rip)        # 70d18 <_DYNAMIC+0x290>
   26b01:	call   *0x4a211(%rip)        # 70d18 <_DYNAMIC+0x290>
   26b07:	int3
   26b08:	int3
   26b09:	int3
   26b0a:	int3
   26b0b:	int3
   26b0c:	int3
   26b0d:	int3
   26b0e:	int3
   26b0f:	int3

