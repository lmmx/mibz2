# Shape gate listing (objdump -d --no-show-raw-insn -C), branch perf/c30-staged: P and C built at a65575b4, L at b6b38e0a (the commit between changes only cfg(test) code).
# Built as bench.yml builds it: cargo build --release --example ab (Cargo.toml release profile: fat LTO, codegen-units 1), rustc 1.97.0.
# 1. mibz2::bwt::room_segments::<32, 1> (P, master's room walk) and <mibz2::c30::C30<false>>::room (C) from that build (DefaultInverse = Compact30).
# 2. <mibz2::c30::C30<true>>::room (L) from the same build with the one line DefaultInverse = Compact30Split (arm L's branch), the source then restored.

#### mibz2::bwt::room_segments::<32, 1>: 1304 instructions, 11 prefetcht0, 38 calls
   99870:	push   %rbp
   99871:	push   %r15
   99873:	push   %r14
   99875:	push   %r13
   99877:	push   %r12
   99879:	push   %rbx
   9987a:	sub    $0x1000,%rsp
   99881:	movq   $0x0,(%rsp)
   99889:	sub    $0x1000,%rsp
   99890:	movq   $0x0,(%rsp)
   99898:	sub    $0x638,%rsp
   9989f:	mov    %rsi,0xf0(%rsp)
   998a7:	cmp    $0x800001,%rsi
   998ae:	jae    9adde <mibz2::bwt::room_segments::<32, 1>+0x156e>
   998b4:	mov    %rdi,%r15
   998b7:	mov    %edx,%edi
   998b9:	cmp    %rdi,%rsi
   998bc:	jbe    9b08e <mibz2::bwt::room_segments::<32, 1>+0x181e>
   998c2:	mov    %rsi,0x50(%rsp)
   998c7:	mov    (%r15,%rdi,4),%esi
   998cb:	movq   $0x0,0x10(%rcx)
   998d3:	movabs $0x2545f4914f6cdd1d,%r14
   998dd:	cmpq   $0x1fe,(%rcx)
   998e4:	jbe    9ae18 <mibz2::bwt::room_segments::<32, 1>+0x15a8>
   998ea:	xor    %r9d,%r9d
   998ed:	shr    $0x8,%esi
   998f0:	mov    0x50(%rsp),%rdi
   998f5:	xor    %rdi,%r14
   998f8:	mov    %rcx,0x20(%rsp)
   998fd:	mov    0x8(%rcx),%rbx
   99901:	mov    $0x1ff,%ecx
   99906:	mov    %r9,%r12
   99909:	nopl   0x0(%rax)
   99910:	mov    %r12,%rbp
   99913:	mov    %r14,%rax
   99916:	shl    $0xd,%rax
   9991a:	xor    %r14,%rax
   9991d:	mov    %rax,%rdx
   99920:	shr    $0x7,%rdx
   99924:	xor    %rax,%rdx
   99927:	mov    %rdx,%r14
   9992a:	shl    $0x11,%r14
   9992e:	xor    %rdx,%r14
   99931:	mov    %r14,%rax
   99934:	xor    %edx,%edx
   99936:	div    %rdi
   99939:	mov    %edx,(%rbx,%r12,4)
   9993d:	inc    %r12
   99940:	dec    %rcx
   99943:	jne    99910 <mibz2::bwt::room_segments::<32, 1>+0xa0>
   99945:	mov    0x20(%rsp),%r14
   9994a:	mov    %r12,0x10(%r14)
   9994e:	cmp    (%r14),%r12
   99951:	mov    %esi,0x4c(%rsp)
   99955:	je     9aa8c <mibz2::bwt::room_segments::<32, 1>+0x121c>
   9995b:	mov    %esi,(%rbx,%r12,4)
   9995f:	lea    0x2(%rbp),%rax
   99963:	mov    %rax,%r13
   99966:	mov    %rax,0x10(%r14)
   9996a:	cmp    $0xfffffffffffffffd,%rbp
   9996e:	jbe    9ae45 <mibz2::bwt::room_segments::<32, 1>+0x15d5>
   99974:	mov    0x20(%rsp),%rbp
   99979:	mov    0x50(%rsp),%rsi
   9997e:	test   %r13,%r13
   99981:	je     9b137 <mibz2::bwt::room_segments::<32, 1>+0x18c7>
   99987:	lea    0x0(,%r13,4),%rax
   9998f:	xor    %ecx,%ecx
   99991:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   999a0:	mov    (%rbx,%rcx,1),%edi
   999a3:	cmp    %rdi,%rsi
   999a6:	jbe    9afa1 <mibz2::bwt::room_segments::<32, 1>+0x1731>
   999ac:	orb    $0x80,0x3(%r15,%rdi,4)
   999b2:	add    $0x4,%rcx
   999b6:	cmp    %rcx,%rax
   999b9:	jne    999a0 <mibz2::bwt::room_segments::<32, 1>+0x130>
   999bb:	lea    (%rsi,%rsi,1),%rax
   999bf:	mov    %rax,%rcx
   999c2:	or     %r13,%rcx
   999c5:	shr    $0x20,%rcx
   999c9:	je     999d2 <mibz2::bwt::room_segments::<32, 1>+0x162>
   999cb:	xor    %edx,%edx
   999cd:	div    %r13
   999d0:	jmp    999d7 <mibz2::bwt::room_segments::<32, 1>+0x167>
   999d2:	xor    %edx,%edx
   999d4:	div    %r13d
   999d7:	mov    %rax,0x58(%rsp)
   999dc:	mov    0x40(%rbp),%r14
   999e0:	cmp    %r13,%r14
   999e3:	mov    %r13,%rbx
   999e6:	cmova  %r14,%rbx
   999ea:	jae    9a7ff <mibz2::bwt::room_segments::<32, 1>+0xf8f>
   999f0:	sub    %r14,%rbx
   999f3:	mov    0x30(%rbp),%rax
   999f7:	sub    %r14,%rax
   999fa:	mov    %r14,%rdx
   999fd:	cmp    %rax,%rbx
   99a00:	ja     9aedb <mibz2::bwt::room_segments::<32, 1>+0x166b>
   99a06:	mov    0x38(%rbp),%rax
   99a0a:	inc    %r14
   99a0d:	test   $0x1,%bl
   99a10:	jne    9af09 <mibz2::bwt::room_segments::<32, 1>+0x1699>
   99a16:	cmp    %r14,%r13
   99a19:	jne    9af36 <mibz2::bwt::room_segments::<32, 1>+0x16c6>
   99a1f:	mov    %rdx,0x40(%rbp)
   99a23:	cmp    %rdx,%r13
   99a26:	ja     9a80f <mibz2::bwt::room_segments::<32, 1>+0xf9f>
   99a2c:	mov    0x58(%rsp),%rax
   99a31:	lea    0x40(%rax),%rbx
   99a35:	mov    0x38(%rbp),%r14
   99a39:	lea    0x0(,%r13,8),%rax
   99a41:	lea    (%rax,%rax,2),%r12
   99a45:	data16 cs nopw 0x0(%rax,%rax,1)
   99a50:	movq   $0x0,0x10(%r14)
   99a58:	cmp    (%r14),%rbx
   99a5b:	ja     99a69 <mibz2::bwt::room_segments::<32, 1>+0x1f9>
   99a5d:	add    $0x18,%r14
   99a61:	add    $0xffffffffffffffe8,%r12
   99a65:	jne    99a50 <mibz2::bwt::room_segments::<32, 1>+0x1e0>
   99a67:	jmp    99a8b <mibz2::bwt::room_segments::<32, 1>+0x21b>
   99a69:	mov    $0x1,%ecx
   99a6e:	mov    $0x1,%r8d
   99a74:	mov    %r14,%rdi
   99a77:	xor    %esi,%esi
   99a79:	mov    %rbx,%rdx
   99a7c:	call   98d90 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   99a81:	add    $0x18,%r14
   99a85:	add    $0xffffffffffffffe8,%r12
   99a89:	jne    99a50 <mibz2::bwt::room_segments::<32, 1>+0x1e0>
   99a8b:	movq   $0x0,0x58(%rbp)
   99a93:	cmp    0x48(%rbp),%r13
   99a97:	ja     9aea7 <mibz2::bwt::room_segments::<32, 1>+0x1637>
   99a9d:	xor    %ebx,%ebx
   99a9f:	mov    0x50(%rbp),%r14
   99aa3:	lea    (%r14,%rbx,8),%rdi
   99aa7:	cmp    $0x1,%r13
   99aab:	je     99ad2 <mibz2::bwt::room_segments::<32, 1>+0x262>
   99aad:	lea    -0x8(,%r13,8),%rdx
   99ab5:	mov    $0xff,%esi
   99aba:	call   *0x96c38(%rip)        # 1306f8 <memset@GLIBC_2.2.5>
   99ac0:	lea    (%rbx,%r13,1),%rax
   99ac4:	lea    (%r14,%rax,8),%rdi
   99ac8:	add    $0xfffffffffffffff8,%rdi
   99acc:	add    %r13,%rbx
   99acf:	dec    %rbx
   99ad2:	movq   $0xffffffffffffffff,(%rdi)
   99ad9:	inc    %rbx
   99adc:	mov    %rbx,0x58(%rbp)
   99ae0:	xorps  %xmm0,%xmm0
   99ae3:	movups %xmm0,0x138(%rsp)
   99aeb:	movl   $0x0,0x148(%rsp)
   99af6:	movups %xmm0,0x158(%rsp)
   99afe:	movl   $0x0,0x168(%rsp)
   99b09:	movups %xmm0,0x178(%rsp)
   99b11:	movl   $0x0,0x188(%rsp)
   99b1c:	movups %xmm0,0x198(%rsp)
   99b24:	movl   $0x0,0x1a8(%rsp)
   99b2f:	movups %xmm0,0x1b8(%rsp)
   99b37:	movl   $0x0,0x1c8(%rsp)
   99b42:	movups %xmm0,0x1d8(%rsp)
   99b4a:	movl   $0x0,0x1e8(%rsp)
   99b55:	movups %xmm0,0x1f8(%rsp)
   99b5d:	movl   $0x0,0x208(%rsp)
   99b68:	movups %xmm0,0x218(%rsp)
   99b70:	movl   $0x0,0x228(%rsp)
   99b7b:	movups %xmm0,0x238(%rsp)
   99b83:	movl   $0x0,0x248(%rsp)
   99b8e:	movups %xmm0,0x258(%rsp)
   99b96:	movl   $0x0,0x268(%rsp)
   99ba1:	movups %xmm0,0x278(%rsp)
   99ba9:	movl   $0x0,0x288(%rsp)
   99bb4:	movups %xmm0,0x298(%rsp)
   99bbc:	movl   $0x0,0x2a8(%rsp)
   99bc7:	movups %xmm0,0x2b8(%rsp)
   99bcf:	movl   $0x0,0x2c8(%rsp)
   99bda:	movups %xmm0,0x2d8(%rsp)
   99be2:	movl   $0x0,0x2e8(%rsp)
   99bed:	movups %xmm0,0x2f8(%rsp)
   99bf5:	movl   $0x0,0x308(%rsp)
   99c00:	movups %xmm0,0x318(%rsp)
   99c08:	movl   $0x0,0x328(%rsp)
   99c13:	movups %xmm0,0x338(%rsp)
   99c1b:	movl   $0x0,0x348(%rsp)
   99c26:	movups %xmm0,0x358(%rsp)
   99c2e:	movl   $0x0,0x368(%rsp)
   99c39:	movups %xmm0,0x378(%rsp)
   99c41:	movl   $0x0,0x388(%rsp)
   99c4c:	movups %xmm0,0x398(%rsp)
   99c54:	movl   $0x0,0x3a8(%rsp)
   99c5f:	movups %xmm0,0x3b8(%rsp)
   99c67:	movl   $0x0,0x3c8(%rsp)
   99c72:	movups %xmm0,0x3d8(%rsp)
   99c7a:	movl   $0x0,0x3e8(%rsp)
   99c85:	movups %xmm0,0x3f8(%rsp)
   99c8d:	movl   $0x0,0x408(%rsp)
   99c98:	movups %xmm0,0x418(%rsp)
   99ca0:	movl   $0x0,0x428(%rsp)
   99cab:	movups %xmm0,0x438(%rsp)
   99cb3:	movl   $0x0,0x448(%rsp)
   99cbe:	movups %xmm0,0x458(%rsp)
   99cc6:	movl   $0x0,0x468(%rsp)
   99cd1:	movups %xmm0,0x478(%rsp)
   99cd9:	movl   $0x0,0x488(%rsp)
   99ce4:	movups %xmm0,0x498(%rsp)
   99cec:	movl   $0x0,0x4a8(%rsp)
   99cf7:	movups %xmm0,0x4b8(%rsp)
   99cff:	movl   $0x0,0x4c8(%rsp)
   99d0a:	movups %xmm0,0x4d8(%rsp)
   99d12:	movl   $0x0,0x4e8(%rsp)
   99d1d:	movups %xmm0,0x4f8(%rsp)
   99d25:	movl   $0x0,0x508(%rsp)
   99d30:	movups %xmm0,0x518(%rsp)
   99d38:	movl   $0x0,0x528(%rsp)
   99d43:	movq   $0xffffffffffffffff,0x130(%rsp)
   99d4f:	movq   $0xffffffffffffffff,0x150(%rsp)
   99d5b:	movq   $0xffffffffffffffff,0x170(%rsp)
   99d67:	movq   $0xffffffffffffffff,0x190(%rsp)
   99d73:	movq   $0xffffffffffffffff,0x1b0(%rsp)
   99d7f:	movq   $0xffffffffffffffff,0x1d0(%rsp)
   99d8b:	movq   $0xffffffffffffffff,0x1f0(%rsp)
   99d97:	movq   $0xffffffffffffffff,0x210(%rsp)
   99da3:	movq   $0xffffffffffffffff,0x230(%rsp)
   99daf:	movq   $0xffffffffffffffff,0x250(%rsp)
   99dbb:	movq   $0xffffffffffffffff,0x270(%rsp)
   99dc7:	movq   $0xffffffffffffffff,0x290(%rsp)
   99dd3:	movq   $0xffffffffffffffff,0x2b0(%rsp)
   99ddf:	movq   $0xffffffffffffffff,0x2d0(%rsp)
   99deb:	movq   $0xffffffffffffffff,0x2f0(%rsp)
   99df7:	movq   $0xffffffffffffffff,0x310(%rsp)
   99e03:	movq   $0xffffffffffffffff,0x330(%rsp)
   99e0f:	movq   $0xffffffffffffffff,0x350(%rsp)
   99e1b:	movq   $0xffffffffffffffff,0x370(%rsp)
   99e27:	movq   $0xffffffffffffffff,0x390(%rsp)
   99e33:	movq   $0xffffffffffffffff,0x3b0(%rsp)
   99e3f:	movq   $0xffffffffffffffff,0x3d0(%rsp)
   99e4b:	movq   $0xffffffffffffffff,0x3f0(%rsp)
   99e57:	movq   $0xffffffffffffffff,0x410(%rsp)
   99e63:	movq   $0xffffffffffffffff,0x430(%rsp)
   99e6f:	movq   $0xffffffffffffffff,0x450(%rsp)
   99e7b:	movq   $0xffffffffffffffff,0x470(%rsp)
   99e87:	movq   $0xffffffffffffffff,0x490(%rsp)
   99e93:	movq   $0xffffffffffffffff,0x4b0(%rsp)
   99e9f:	movq   $0xffffffffffffffff,0x4d0(%rsp)
   99eab:	movq   $0xffffffffffffffff,0x4f0(%rsp)
   99eb7:	movq   $0xffffffffffffffff,0x510(%rsp)
   99ec3:	mov    0x38(%rbp),%rax
   99ec7:	mov    0x40(%rbp),%rdi
   99ecb:	mov    0x8(%rbp),%r9
   99ecf:	mov    0x10(%rbp),%r10
   99ed3:	add    $0x8,%rax
   99ed7:	xor    %r12d,%r12d
   99eda:	xor    %ecx,%ecx
   99edc:	nopl   0x0(%rax)
   99ee0:	cmp    %rcx,%rdi
   99ee3:	je     9afad <mibz2::bwt::room_segments::<32, 1>+0x173d>
   99ee9:	cmp    %rcx,%r10
   99eec:	je     9afbc <mibz2::bwt::room_segments::<32, 1>+0x174c>
   99ef2:	mov    (%rax),%rdx
   99ef5:	mov    (%r9,%rcx,4),%esi
   99ef9:	mov    (%r15,%rsi,4),%esi
   99efd:	mov    %sil,(%rdx)
   99f00:	shr    $0x8,%esi
   99f03:	and    $0x7fffff,%esi
   99f09:	lea    0x1(%rdx),%r8
   99f0d:	add    -0x8(%rax),%rdx
   99f11:	mov    %rcx,0x130(%rsp,%r12,1)
   99f19:	mov    %r8,0x138(%rsp,%r12,1)
   99f21:	mov    %rdx,0x140(%rsp,%r12,1)
   99f29:	mov    %esi,0x148(%rsp,%r12,1)
   99f31:	prefetcht0 (%r15,%rsi,4)
   99f36:	lea    0x1(%rcx),%rdx
   99f3a:	add    $0x20,%r12
   99f3e:	cmp    $0x1e,%rcx
   99f42:	ja     99f50 <mibz2::bwt::room_segments::<32, 1>+0x6e0>
   99f44:	add    $0x18,%rax
   99f48:	mov    %rdx,%rcx
   99f4b:	cmp    %r13,%rdx
   99f4e:	jb     99ee0 <mibz2::bwt::room_segments::<32, 1>+0x670>
   99f50:	cmp    $0x20,%rdx
   99f54:	jne    9a820 <mibz2::bwt::room_segments::<32, 1>+0xfb0>
   99f5a:	mov    %rdx,0xf8(%rsp)
   99f62:	mov    %r13,0x128(%rsp)
   99f6a:	mov    0x148(%rsp),%eax
   99f71:	mov    0x168(%rsp),%ecx
   99f78:	mov    %eax,0x68(%rsp)
   99f7c:	mov    %ecx,0x6c(%rsp)
   99f80:	mov    0x188(%rsp),%eax
   99f87:	mov    %eax,0x70(%rsp)
   99f8b:	mov    0x1a8(%rsp),%eax
   99f92:	mov    %eax,0x74(%rsp)
   99f96:	mov    0x1c8(%rsp),%eax
   99f9d:	mov    %eax,0x78(%rsp)
   99fa1:	mov    0x1e8(%rsp),%eax
   99fa8:	mov    %eax,0x7c(%rsp)
   99fac:	mov    0x208(%rsp),%eax
   99fb3:	mov    %eax,0x80(%rsp)
   99fba:	mov    0x228(%rsp),%eax
   99fc1:	mov    %eax,0x84(%rsp)
   99fc8:	mov    0x248(%rsp),%eax
   99fcf:	mov    %eax,0x88(%rsp)
   99fd6:	mov    0x268(%rsp),%eax
   99fdd:	mov    0x288(%rsp),%ecx
   99fe4:	mov    %eax,0x8c(%rsp)
   99feb:	mov    %ecx,0x90(%rsp)
   99ff2:	mov    0x2a8(%rsp),%eax
   99ff9:	mov    %eax,0x94(%rsp)
   9a000:	mov    0x2c8(%rsp),%eax
   9a007:	mov    %eax,0x98(%rsp)
   9a00e:	mov    0x2e8(%rsp),%eax
   9a015:	mov    %eax,0x9c(%rsp)
   9a01c:	mov    0x308(%rsp),%eax
   9a023:	mov    %eax,0xa0(%rsp)
   9a02a:	mov    0x328(%rsp),%eax
   9a031:	mov    %eax,0xa4(%rsp)
   9a038:	mov    0x348(%rsp),%eax
   9a03f:	mov    %eax,0xa8(%rsp)
   9a046:	mov    0x368(%rsp),%eax
   9a04d:	mov    %eax,0xac(%rsp)
   9a054:	mov    0x388(%rsp),%eax
   9a05b:	mov    %eax,0xb0(%rsp)
   9a062:	mov    0x3a8(%rsp),%eax
   9a069:	mov    %eax,0xb4(%rsp)
   9a070:	mov    0x3c8(%rsp),%eax
   9a077:	mov    %eax,0xb8(%rsp)
   9a07e:	mov    0x3e8(%rsp),%eax
   9a085:	mov    %eax,0xbc(%rsp)
   9a08c:	mov    0x408(%rsp),%eax
   9a093:	mov    %eax,0xc0(%rsp)
   9a09a:	mov    0x428(%rsp),%eax
   9a0a1:	mov    %eax,0xc4(%rsp)
   9a0a8:	mov    0x448(%rsp),%eax
   9a0af:	mov    %eax,0xc8(%rsp)
   9a0b6:	mov    0x468(%rsp),%eax
   9a0bd:	mov    %eax,0xcc(%rsp)
   9a0c4:	mov    0x488(%rsp),%eax
   9a0cb:	mov    %eax,0xd0(%rsp)
   9a0d2:	mov    0x4a8(%rsp),%eax
   9a0d9:	mov    %eax,0xd4(%rsp)
   9a0e0:	mov    0x4c8(%rsp),%eax
   9a0e7:	mov    %eax,0xd8(%rsp)
   9a0ee:	mov    0x4e8(%rsp),%eax
   9a0f5:	mov    %eax,0xdc(%rsp)
   9a0fc:	mov    0x508(%rsp),%eax
   9a103:	mov    %eax,0xe0(%rsp)
   9a10a:	mov    0x528(%rsp),%eax
   9a111:	mov    %eax,0xe4(%rsp)
   9a118:	lea    0x630(%rsp),%rdi
   9a120:	mov    $0x2000,%edx
   9a125:	xor    %esi,%esi
   9a127:	call   *0x965cb(%rip)        # 1306f8 <memset@GLIBC_2.2.5>
   9a12d:	addq   $0x3f,0x58(%rsp)
   9a133:	mov    0x38(%rbp),%rbx
   9a137:	mov    0x40(%rbp),%rsi
   9a13b:	mov    0x50(%rbp),%rax
   9a13f:	mov    %rax,0x60(%rsp)
   9a144:	mov    0x58(%rbp),%rax
   9a148:	mov    %rax,0x28(%rsp)
   9a14d:	mov    0x8(%rbp),%rax
   9a151:	mov    %rax,0x38(%rsp)
   9a156:	mov    0x10(%rbp),%rax
   9a15a:	mov    %rax,0x40(%rsp)
   9a15f:	mov    $0x20,%r9d
   9a165:	mov    %r12,0x100(%rsp)
   9a16d:	mov    %rsi,0x18(%rsp)
   9a172:	mov    %rbx,0xe8(%rsp)
   9a17a:	mov    %r15,0x108(%rsp)
   9a182:	jmp    9a199 <mibz2::bwt::room_segments::<32, 1>+0x929>
   9a184:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   9a190:	test   %r13,%r13
   9a193:	jne    9a25a <mibz2::bwt::room_segments::<32, 1>+0x9ea>
   9a199:	xor    %ebp,%ebp
   9a19b:	mov    0x58(%rsp),%r13
   9a1a0:	jmp    9a1dd <mibz2::bwt::room_segments::<32, 1>+0x96d>
   9a1a2:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   9a1b0:	add    %rax,%r14
   9a1b3:	mov    %r14,0x138(%rsp,%rbp,1)
   9a1bb:	add    %rax,%rdx
   9a1be:	mov    %rdx,0x140(%rsp,%rbp,1)
   9a1c6:	sub    %r14,%rdx
   9a1c9:	add    $0x20,%rbp
   9a1cd:	cmp    %r13,%rdx
   9a1d0:	cmovb  %rdx,%r13
   9a1d4:	cmp    $0x400,%rbp
   9a1db:	je     9a190 <mibz2::bwt::room_segments::<32, 1>+0x920>
   9a1dd:	mov    0x138(%rsp,%rbp,1),%r14
   9a1e5:	mov    0x140(%rsp,%rbp,1),%rdx
   9a1ed:	cmp    %rdx,%r14
   9a1f0:	jne    9a1c6 <mibz2::bwt::room_segments::<32, 1>+0x956>
   9a1f2:	mov    0x130(%rsp,%rbp,1),%rdi
   9a1fa:	cmp    %rsi,%rdi
   9a1fd:	jae    9af95 <mibz2::bwt::room_segments::<32, 1>+0x1725>
   9a203:	lea    (%rdi,%rdi,2),%rcx
   9a207:	mov    (%rbx,%rcx,8),%rdx
   9a20b:	mov    0x8(%rbx,%rcx,8),%rax
   9a210:	sub    %rax,%r14
   9a213:	mov    %r14,0x10(%rbx,%rcx,8)
   9a218:	mov    %rdx,%rdi
   9a21b:	sub    %r14,%rdi
   9a21e:	cmp    %rdi,%rdx
   9a221:	jbe    9a1b0 <mibz2::bwt::room_segments::<32, 1>+0x940>
   9a223:	lea    (%rbx,%rcx,8),%r12
   9a227:	mov    $0x1,%ecx
   9a22c:	mov    $0x1,%r8d
   9a232:	mov    %r12,%rdi
   9a235:	mov    %r14,%rsi
   9a238:	mov    %r9,0x8(%rsp)
   9a23d:	call   98d90 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   9a242:	mov    0x8(%rsp),%r9
   9a247:	mov    0x18(%rsp),%rsi
   9a24c:	mov    (%r12),%rdx
   9a250:	mov    0x8(%r12),%rax
   9a255:	jmp    9a1b0 <mibz2::bwt::room_segments::<32, 1>+0x940>
   9a25a:	xor    %ecx,%ecx
   9a25c:	mov    %r13,%rsi
   9a25f:	sub    %rcx,%rsi
   9a262:	cmp    $0x100,%rsi
   9a269:	mov    $0x100,%eax
   9a26e:	cmovae %rax,%rsi
   9a272:	xorps  %xmm0,%xmm0
   9a275:	movaps %xmm0,0x620(%rsp)
   9a27d:	movaps %xmm0,0x610(%rsp)
   9a285:	movaps %xmm0,0x600(%rsp)
   9a28d:	movaps %xmm0,0x5f0(%rsp)
   9a295:	movaps %xmm0,0x5e0(%rsp)
   9a29d:	movaps %xmm0,0x5d0(%rsp)
   9a2a5:	movaps %xmm0,0x5c0(%rsp)
   9a2ad:	movaps %xmm0,0x5b0(%rsp)
   9a2b5:	movaps %xmm0,0x5a0(%rsp)
   9a2bd:	movaps %xmm0,0x590(%rsp)
   9a2c5:	movaps %xmm0,0x580(%rsp)
   9a2cd:	movaps %xmm0,0x570(%rsp)
   9a2d5:	movaps %xmm0,0x560(%rsp)
   9a2dd:	movaps %xmm0,0x550(%rsp)
   9a2e5:	movaps %xmm0,0x540(%rsp)
   9a2ed:	movaps %xmm0,0x530(%rsp)
   9a2f5:	cmp    %rcx,%r13
   9a2f8:	mov    %rcx,0x110(%rsp)
   9a300:	mov    %rsi,0x30(%rsp)
   9a305:	jne    9a3e0 <mibz2::bwt::room_segments::<32, 1>+0xb70>
   9a30b:	mov    %r9,0x8(%rsp)
   9a310:	lea    0x538(%rsp),%r12
   9a318:	lea    0x630(%rsp),%rbp
   9a320:	xor    %ebx,%ebx
   9a322:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   9a330:	mov    -0x8(%r12),%rsi
   9a335:	mov    0x30(%rsp),%r15
   9a33a:	mov    %r15,%r14
   9a33d:	sub    %rsi,%r14
   9a340:	add    %rbp,%rsi
   9a343:	mov    0x138(%rsp,%rbx,1),%rdi
   9a34b:	mov    %r14,%rdx
   9a34e:	call   *0x9633c(%rip)        # 130690 <memcpy@GLIBC_2.14>
   9a354:	add    %r14,0x138(%rsp,%rbx,1)
   9a35c:	lea    0x40(%rbx),%rax
   9a360:	mov    %rax,0x10(%rsp)
   9a365:	mov    (%r12),%rax
   9a369:	sub    %rax,%r15
   9a36c:	lea    (%rax,%rbp,1),%rsi
   9a370:	add    $0x100,%rsi
   9a377:	mov    0x158(%rsp,%rbx,1),%rdi
   9a37f:	mov    %r15,%rdx
   9a382:	call   *0x96308(%rip)        # 130690 <memcpy@GLIBC_2.14>
   9a388:	add    %r15,0x158(%rsp,%rbx,1)
   9a390:	add    $0x200,%rbp
   9a397:	add    $0x10,%r12
   9a39b:	mov    0x10(%rsp),%rax
   9a3a0:	mov    %rax,%rbx
   9a3a3:	cmp    $0x400,%rax
   9a3a9:	jne    9a330 <mibz2::bwt::room_segments::<32, 1>+0xac0>
   9a3ab:	mov    0x110(%rsp),%rcx
   9a3b3:	add    0x30(%rsp),%rcx
   9a3b8:	cmp    %r13,%rcx
   9a3bb:	mov    0x18(%rsp),%rsi
   9a3c0:	mov    0xe8(%rsp),%rbx
   9a3c8:	mov    0x8(%rsp),%r9
   9a3cd:	mov    0x108(%rsp),%r15
   9a3d5:	jb     9a25c <mibz2::bwt::room_segments::<32, 1>+0x9ec>
   9a3db:	jmp    9a199 <mibz2::bwt::room_segments::<32, 1>+0x929>
   9a3e0:	lea    0xd30(%rsp),%rcx
   9a3e8:	xor    %eax,%eax
   9a3ea:	jmp    9a409 <mibz2::bwt::room_segments::<32, 1>+0xb99>
   9a3ec:	nopl   0x0(%rax)
   9a3f0:	mov    0x118(%rsp),%rcx
   9a3f8:	inc    %rcx
   9a3fb:	mov    0x30(%rsp),%rsi
   9a400:	cmp    %rsi,%rax
   9a403:	je     9a30b <mibz2::bwt::room_segments::<32, 1>+0xa9b>
   9a409:	mov    %rax,0x10(%rsp)
   9a40e:	mov    %rcx,0x118(%rsp)
   9a416:	mov    %rcx,%rax
   9a419:	xor    %ecx,%ecx
   9a41b:	xor    %ebp,%ebp
   9a41d:	jmp    9a43f <mibz2::bwt::room_segments::<32, 1>+0xbcf>
   9a41f:	nop
   9a420:	mov    $0x80,%edx
   9a425:	shl    %cl,%rdx
   9a428:	or     %rdx,%rbp
   9a42b:	add    $0x8,%rcx
   9a42f:	add    $0x800,%rax
   9a435:	cmp    $0x19,%rcx
   9a439:	jae    9a630 <mibz2::bwt::room_segments::<32, 1>+0xdc0>
   9a43f:	mov    0x68(%rsp,%rcx,4),%edx
   9a443:	mov    (%r15,%rdx,4),%edx
   9a447:	test   %edx,%edx
   9a449:	js     9a550 <mibz2::bwt::room_segments::<32, 1>+0xce0>
   9a44f:	mov    %dl,-0x700(%rax)
   9a455:	shr    $0x8,%edx
   9a458:	mov    %edx,0x68(%rsp,%rcx,4)
   9a45c:	prefetcht0 (%r15,%rdx,4)
   9a461:	mov    0x6c(%rsp,%rcx,4),%edx
   9a465:	mov    (%r15,%rdx,4),%edx
   9a469:	test   %edx,%edx
   9a46b:	js     9a564 <mibz2::bwt::room_segments::<32, 1>+0xcf4>
   9a471:	mov    %dl,-0x600(%rax)
   9a477:	shr    $0x8,%edx
   9a47a:	mov    %edx,0x6c(%rsp,%rcx,4)
   9a47e:	prefetcht0 (%r15,%rdx,4)
   9a483:	mov    0x70(%rsp,%rcx,4),%edx
   9a487:	mov    (%r15,%rdx,4),%edx
   9a48b:	test   %edx,%edx
   9a48d:	js     9a57f <mibz2::bwt::room_segments::<32, 1>+0xd0f>
   9a493:	mov    %dl,-0x500(%rax)
   9a499:	shr    $0x8,%edx
   9a49c:	mov    %edx,0x70(%rsp,%rcx,4)
   9a4a0:	prefetcht0 (%r15,%rdx,4)
   9a4a5:	mov    0x74(%rsp,%rcx,4),%edx
   9a4a9:	mov    (%r15,%rdx,4),%edx
   9a4ad:	test   %edx,%edx
   9a4af:	js     9a59a <mibz2::bwt::room_segments::<32, 1>+0xd2a>
   9a4b5:	mov    %dl,-0x400(%rax)
   9a4bb:	shr    $0x8,%edx
   9a4be:	mov    %edx,0x74(%rsp,%rcx,4)
   9a4c2:	prefetcht0 (%r15,%rdx,4)
   9a4c7:	mov    0x78(%rsp,%rcx,4),%edx
   9a4cb:	mov    (%r15,%rdx,4),%edx
   9a4cf:	test   %edx,%edx
   9a4d1:	js     9a5b5 <mibz2::bwt::room_segments::<32, 1>+0xd45>
   9a4d7:	mov    %dl,-0x300(%rax)
   9a4dd:	shr    $0x8,%edx
   9a4e0:	mov    %edx,0x78(%rsp,%rcx,4)
   9a4e4:	prefetcht0 (%r15,%rdx,4)
   9a4e9:	mov    0x7c(%rsp,%rcx,4),%edx
   9a4ed:	mov    (%r15,%rdx,4),%edx
   9a4f1:	test   %edx,%edx
   9a4f3:	js     9a5d0 <mibz2::bwt::room_segments::<32, 1>+0xd60>
   9a4f9:	mov    %dl,-0x200(%rax)
   9a4ff:	shr    $0x8,%edx
   9a502:	mov    %edx,0x7c(%rsp,%rcx,4)
   9a506:	prefetcht0 (%r15,%rdx,4)
   9a50b:	mov    0x80(%rsp,%rcx,4),%edx
   9a512:	mov    (%r15,%rdx,4),%edx
   9a516:	test   %edx,%edx
   9a518:	js     9a5ee <mibz2::bwt::room_segments::<32, 1>+0xd7e>
   9a51e:	mov    %dl,-0x100(%rax)
   9a524:	shr    $0x8,%edx
   9a527:	mov    %edx,0x80(%rsp,%rcx,4)
   9a52e:	prefetcht0 (%r15,%rdx,4)
   9a533:	mov    0x84(%rsp,%rcx,4),%edx
   9a53a:	mov    (%r15,%rdx,4),%edx
   9a53e:	test   %edx,%edx
   9a540:	jns    9a60c <mibz2::bwt::room_segments::<32, 1>+0xd9c>
   9a546:	jmp    9a420 <mibz2::bwt::room_segments::<32, 1>+0xbb0>
   9a54b:	nopl   0x0(%rax,%rax,1)
   9a550:	bts    %rcx,%rbp
   9a554:	mov    0x6c(%rsp,%rcx,4),%edx
   9a558:	mov    (%r15,%rdx,4),%edx
   9a55c:	test   %edx,%edx
   9a55e:	jns    9a471 <mibz2::bwt::room_segments::<32, 1>+0xc01>
   9a564:	mov    $0x2,%edx
   9a569:	shl    %cl,%rdx
   9a56c:	or     %rdx,%rbp
   9a56f:	mov    0x70(%rsp,%rcx,4),%edx
   9a573:	mov    (%r15,%rdx,4),%edx
   9a577:	test   %edx,%edx
   9a579:	jns    9a493 <mibz2::bwt::room_segments::<32, 1>+0xc23>
   9a57f:	mov    $0x4,%edx
   9a584:	shl    %cl,%rdx
   9a587:	or     %rdx,%rbp
   9a58a:	mov    0x74(%rsp,%rcx,4),%edx
   9a58e:	mov    (%r15,%rdx,4),%edx
   9a592:	test   %edx,%edx
   9a594:	jns    9a4b5 <mibz2::bwt::room_segments::<32, 1>+0xc45>
   9a59a:	mov    $0x8,%edx
   9a59f:	shl    %cl,%rdx
   9a5a2:	or     %rdx,%rbp
   9a5a5:	mov    0x78(%rsp,%rcx,4),%edx
   9a5a9:	mov    (%r15,%rdx,4),%edx
   9a5ad:	test   %edx,%edx
   9a5af:	jns    9a4d7 <mibz2::bwt::room_segments::<32, 1>+0xc67>
   9a5b5:	mov    $0x10,%edx
   9a5ba:	shl    %cl,%rdx
   9a5bd:	or     %rdx,%rbp
   9a5c0:	mov    0x7c(%rsp,%rcx,4),%edx
   9a5c4:	mov    (%r15,%rdx,4),%edx
   9a5c8:	test   %edx,%edx
   9a5ca:	jns    9a4f9 <mibz2::bwt::room_segments::<32, 1>+0xc89>
   9a5d0:	mov    $0x20,%edx
   9a5d5:	shl    %cl,%rdx
   9a5d8:	or     %rdx,%rbp
   9a5db:	mov    0x80(%rsp,%rcx,4),%edx
   9a5e2:	mov    (%r15,%rdx,4),%edx
   9a5e6:	test   %edx,%edx
   9a5e8:	jns    9a51e <mibz2::bwt::room_segments::<32, 1>+0xcae>
   9a5ee:	mov    $0x40,%edx
   9a5f3:	shl    %cl,%rdx
   9a5f6:	or     %rdx,%rbp
   9a5f9:	mov    0x84(%rsp,%rcx,4),%edx
   9a600:	mov    (%r15,%rdx,4),%edx
   9a604:	test   %edx,%edx
   9a606:	js     9a420 <mibz2::bwt::room_segments::<32, 1>+0xbb0>
   9a60c:	mov    %dl,(%rax)
   9a60e:	shr    $0x8,%edx
   9a611:	mov    %edx,0x84(%rsp,%rcx,4)
   9a618:	prefetcht0 (%r15,%rdx,4)
   9a61d:	jmp    9a42b <mibz2::bwt::room_segments::<32, 1>+0xbbb>
   9a622:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   9a630:	mov    0x10(%rsp),%rax
   9a635:	inc    %rax
   9a638:	test   %rbp,%rbp
   9a63b:	je     9a3f0 <mibz2::bwt::room_segments::<32, 1>+0xb80>
   9a641:	mov    %rax,0x120(%rsp)
   9a649:	nopl   0x0(%rax)
   9a650:	mov    %r9,0x8(%rsp)
   9a655:	tzcnt  %rbp,%r14
   9a65a:	cmp    $0x1f,%r14
   9a65e:	ja     9b034 <mibz2::bwt::room_segments::<32, 1>+0x17c4>
   9a664:	mov    %r14d,%ebx
   9a667:	shl    $0x5,%ebx
   9a66a:	mov    0x68(%rsp,%r14,4),%eax
   9a66f:	mov    %eax,0x148(%rsp,%rbx,1)
   9a676:	mov    %r14d,%eax
   9a679:	shl    $0x8,%eax
   9a67c:	lea    (%rsp,%rax,1),%rsi
   9a680:	add    $0x630,%rsi
   9a687:	mov    0x530(%rsp,%r14,8),%rax
   9a68f:	mov    0x10(%rsp),%r12
   9a694:	sub    %rax,%r12
   9a697:	add    %rax,%rsi
   9a69a:	mov    0x138(%rsp,%rbx,1),%rdi
   9a6a2:	mov    %r12,%rdx
   9a6a5:	call   *0x95fe5(%rip)        # 130690 <memcpy@GLIBC_2.14>
   9a6ab:	add    0x138(%rsp,%rbx,1),%r12
   9a6b3:	mov    %r12,0x138(%rsp,%rbx,1)
   9a6bb:	mov    0x130(%rsp,%rbx,1),%rdi
   9a6c3:	mov    0x18(%rsp),%r8
   9a6c8:	cmp    %r8,%rdi
   9a6cb:	jae    9b025 <mibz2::bwt::room_segments::<32, 1>+0x17b5>
   9a6d1:	lea    (%rsp,%rbx,1),%rax
   9a6d5:	add    $0x130,%rax
   9a6db:	lea    (%rdi,%rdi,2),%rcx
   9a6df:	mov    0xe8(%rsp),%rbx
   9a6e7:	sub    0x8(%rbx,%rcx,8),%r12
   9a6ec:	mov    %r12,0x10(%rbx,%rcx,8)
   9a6f1:	mov    0x40(%rsp),%rsi
   9a6f6:	test   %rsi,%rsi
   9a6f9:	je     9a710 <mibz2::bwt::room_segments::<32, 1>+0xea0>
   9a6fb:	mov    0x18(%rax),%edx
   9a6fe:	cmp    $0x1,%rsi
   9a702:	mov    0x38(%rsp),%r10
   9a707:	jne    9a720 <mibz2::bwt::room_segments::<32, 1>+0xeb0>
   9a709:	xor    %ecx,%ecx
   9a70b:	jmp    9a74d <mibz2::bwt::room_segments::<32, 1>+0xedd>
   9a70d:	nopl   (%rax)
   9a710:	mov    $0xffffffffffffffff,%rcx
   9a717:	mov    0x38(%rsp),%r10
   9a71c:	jmp    9a761 <mibz2::bwt::room_segments::<32, 1>+0xef1>
   9a71e:	xchg   %ax,%ax
   9a720:	mov    %rsi,%r11
   9a723:	xor    %ecx,%ecx
   9a725:	data16 cs nopw 0x0(%rax,%rax,1)
   9a730:	mov    %rcx,%r8
   9a733:	mov    %r11,%r9
   9a736:	shr    $1,%r9
   9a739:	add    %r9,%rcx
   9a73c:	cmp    %edx,(%r10,%rcx,4)
   9a740:	cmova  %r8,%rcx
   9a744:	sub    %r9,%r11
   9a747:	cmp    $0x1,%r11
   9a74b:	ja     9a730 <mibz2::bwt::room_segments::<32, 1>+0xec0>
   9a74d:	cmp    %edx,(%r10,%rcx,4)
   9a751:	mov    $0xffffffffffffffff,%rdx
   9a758:	cmovne %rdx,%rcx
   9a75c:	mov    0x18(%rsp),%r8
   9a761:	cmp    0x28(%rsp),%rdi
   9a766:	mov    0x8(%rsp),%r9
   9a76b:	jae    9b014 <mibz2::bwt::room_segments::<32, 1>+0x17a4>
   9a771:	lea    -0x1(%rbp),%rdx
   9a775:	and    %rdx,%rbp
   9a778:	mov    0x60(%rsp),%rdx
   9a77d:	mov    %rcx,(%rdx,%rdi,8)
   9a781:	cmp    0x128(%rsp),%r9
   9a789:	je     9a837 <mibz2::bwt::room_segments::<32, 1>+0xfc7>
   9a78f:	cmp    %r8,%r9
   9a792:	jae    9b057 <mibz2::bwt::room_segments::<32, 1>+0x17e7>
   9a798:	cmp    %rsi,%r9
   9a79b:	jae    9b048 <mibz2::bwt::room_segments::<32, 1>+0x17d8>
   9a7a1:	lea    (%r9,%r9,2),%rcx
   9a7a5:	mov    0x8(%rbx,%rcx,8),%rdx
   9a7aa:	mov    (%r10,%r9,4),%esi
   9a7ae:	mov    (%r15,%rsi,4),%esi
   9a7b2:	mov    %sil,(%rdx)
   9a7b5:	shr    $0x8,%esi
   9a7b8:	and    $0x7fffff,%esi
   9a7be:	lea    0x1(%rdx),%rdi
   9a7c2:	add    (%rbx,%rcx,8),%rdx
   9a7c6:	mov    %r9,(%rax)
   9a7c9:	mov    %rdi,0x8(%rax)
   9a7cd:	mov    %rdx,0x10(%rax)
   9a7d1:	mov    %esi,0x18(%rax)
   9a7d4:	mov    %esi,0x68(%rsp,%r14,4)
   9a7d9:	prefetcht0 (%r15,%rsi,4)
   9a7de:	inc    %r9
   9a7e1:	mov    0x120(%rsp),%rax
   9a7e9:	mov    %rax,0x530(%rsp,%r14,8)
   9a7f1:	test   %rbp,%rbp
   9a7f4:	jne    9a650 <mibz2::bwt::room_segments::<32, 1>+0xde0>
   9a7fa:	jmp    9a3f0 <mibz2::bwt::room_segments::<32, 1>+0xb80>
   9a7ff:	mov    %rbx,%rdx
   9a802:	mov    %rdx,0x40(%rbp)
   9a806:	cmp    %rdx,%r13
   9a809:	jbe    99a2c <mibz2::bwt::room_segments::<32, 1>+0x1bc>
   9a80f:	lea    0x92dca(%rip),%rcx        # 12d5e0 <__frame_dummy_init_array_entry+0x1490>
   9a816:	xor    %edi,%edi
   9a818:	mov    %r13,%rsi
   9a81b:	call   8b5e0 <core::slice::index::slice_index_fail>
   9a820:	mov    0x50(%rbp),%rax
   9a824:	mov    %rax,0x60(%rsp)
   9a829:	mov    0x58(%rbp),%rax
   9a82d:	mov    %rax,0x28(%rsp)
   9a832:	jmp    9a8cc <mibz2::bwt::room_segments::<32, 1>+0x105c>
   9a837:	not    %rbp
   9a83a:	xor    %r12d,%r12d
   9a83d:	xor    %r13d,%r13d
   9a840:	jmp    9a860 <mibz2::bwt::room_segments::<32, 1>+0xff0>
   9a842:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   9a850:	add    $0x20,%r13
   9a854:	inc    %r12
   9a857:	cmp    $0x400,%r13
   9a85e:	je     9a8b2 <mibz2::bwt::room_segments::<32, 1>+0x1042>
   9a860:	mov    0x68(%rsp,%r12,4),%eax
   9a865:	mov    %eax,0x148(%rsp,%r13,1)
   9a86d:	cmp    %r12,%r14
   9a870:	je     9a850 <mibz2::bwt::room_segments::<32, 1>+0xfe0>
   9a872:	mov    0x530(%rsp,%r12,8),%rax
   9a87a:	bt     %r12,%rbp
   9a87e:	mov    0x10(%rsp),%rbx
   9a883:	adc    $0x0,%rbx
   9a887:	sub    %rax,%rbx
   9a88a:	add    %rsp,%rax
   9a88d:	add    $0x630,%rax
   9a893:	lea    (%rax,%r13,8),%rsi
   9a897:	mov    0x138(%rsp,%r13,1),%rdi
   9a89f:	mov    %rbx,%rdx
   9a8a2:	call   *0x95de8(%rip)        # 130690 <memcpy@GLIBC_2.14>
   9a8a8:	add    %rbx,0x138(%rsp,%r13,1)
   9a8b0:	jmp    9a850 <mibz2::bwt::room_segments::<32, 1>+0xfe0>
   9a8b2:	mov    0x38(%rsp),%r9
   9a8b7:	mov    0x40(%rsp),%r10
   9a8bc:	mov    0x100(%rsp),%r12
   9a8c4:	mov    0xf8(%rsp),%rdx
   9a8cc:	shl    $0x5,%rdx
   9a8d0:	lea    (%rsp,%rdx,1),%r13
   9a8d4:	add    $0x130,%r13
   9a8db:	lea    0x130(%rsp),%r14
   9a8e3:	mov    0x60(%rsp),%rax
   9a8e8:	jmp    9a8fd <mibz2::bwt::room_segments::<32, 1>+0x108d>
   9a8ea:	nopw   0x0(%rax,%rax,1)
   9a8f0:	mov    %rbp,%r14
   9a8f3:	add    $0xffffffffffffffe0,%r12
   9a8f7:	je     9aa7b <mibz2::bwt::room_segments::<32, 1>+0x120b>
   9a8fd:	lea    0x20(%r14),%rbp
   9a901:	mov    (%r14),%rbx
   9a904:	cmp    $0xffffffffffffffff,%rbx
   9a908:	je     9a8f0 <mibz2::bwt::room_segments::<32, 1>+0x1080>
   9a90a:	cmp    0x28(%rsp),%rbx
   9a90f:	jae    9b069 <mibz2::bwt::room_segments::<32, 1>+0x17f9>
   9a915:	cmpq   $0xffffffffffffffff,(%rax,%rbx,8)
   9a91a:	jne    9a8f0 <mibz2::bwt::room_segments::<32, 1>+0x1080>
   9a91c:	mov    $0x80,%edi
   9a921:	call   *0x95d81(%rip)        # 1306a8 <malloc@GLIBC_2.2.5>
   9a927:	test   %rax,%rax
   9a92a:	je     9b09a <mibz2::bwt::room_segments::<32, 1>+0x182a>
   9a930:	mov    0x18(%r14),%rcx
   9a934:	mov    %rcx,0x18(%rax)
   9a938:	movups 0x8(%r14),%xmm0
   9a93d:	movups %xmm0,0x8(%rax)
   9a941:	mov    %rbx,(%rax)
   9a944:	movq   $0x4,0x530(%rsp)
   9a950:	mov    %rax,0x538(%rsp)
   9a958:	movq   $0x1,0x540(%rsp)
   9a964:	mov    $0x1,%r12d
   9a96a:	cmp    %r13,%rbp
   9a96d:	je     9aa37 <mibz2::bwt::room_segments::<32, 1>+0x11c7>
   9a973:	lea    0x530(%rsp),%rbx
   9a97b:	mov    0x20(%rsp),%rdx
   9a980:	mov    0x50(%rdx),%rcx
   9a984:	mov    0x58(%rdx),%rsi
   9a988:	mov    %rbp,%rdx
   9a98b:	jmp    9a99c <mibz2::bwt::room_segments::<32, 1>+0x112c>
   9a98d:	nopl   (%rax)
   9a990:	mov    %rbp,%rdx
   9a993:	cmp    %r13,%rbp
   9a996:	je     9aa37 <mibz2::bwt::room_segments::<32, 1>+0x11c7>
   9a99c:	lea    0x20(%rdx),%rbp
   9a9a0:	mov    (%rdx),%r14
   9a9a3:	cmp    $0xffffffffffffffff,%r14
   9a9a7:	je     9a990 <mibz2::bwt::room_segments::<32, 1>+0x1120>
   9a9a9:	cmp    %rsi,%r14
   9a9ac:	jae    9b07d <mibz2::bwt::room_segments::<32, 1>+0x180d>
   9a9b2:	cmpq   $0xffffffffffffffff,(%rcx,%r14,8)
   9a9b7:	jne    9a990 <mibz2::bwt::room_segments::<32, 1>+0x1120>
   9a9b9:	mov    0x18(%rdx),%rcx
   9a9bd:	mov    %rcx,0x640(%rsp)
   9a9c5:	movups 0x8(%rdx),%xmm0
   9a9c9:	movaps %xmm0,0x630(%rsp)
   9a9d1:	cmp    0x530(%rsp),%r12
   9a9d9:	jne    9a9fe <mibz2::bwt::room_segments::<32, 1>+0x118e>
   9a9db:	mov    $0x1,%edx
   9a9e0:	mov    $0x8,%ecx
   9a9e5:	mov    $0x20,%r8d
   9a9eb:	mov    %rbx,%rdi
   9a9ee:	mov    %r12,%rsi
   9a9f1:	call   98d90 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   9a9f6:	mov    0x538(%rsp),%rax
   9a9fe:	mov    %r12,%rcx
   9aa01:	shl    $0x5,%rcx
   9aa05:	mov    %r14,(%rax,%rcx,1)
   9aa09:	movaps 0x630(%rsp),%xmm0
   9aa11:	movups %xmm0,0x8(%rax,%rcx,1)
   9aa16:	mov    0x640(%rsp),%rdx
   9aa1e:	mov    %rdx,0x18(%rax,%rcx,1)
   9aa23:	inc    %r12
   9aa26:	mov    %r12,0x540(%rsp)
   9aa2e:	cmp    %r13,%rbp
   9aa31:	jne    9a97b <mibz2::bwt::room_segments::<32, 1>+0x110b>
   9aa37:	mov    0x530(%rsp),%rax
   9aa3f:	mov    %rax,0x18(%rsp)
   9aa44:	mov    0x538(%rsp),%rax
   9aa4c:	mov    %rax,0x8(%rsp)
   9aa51:	mov    %r12,%rbx
   9aa54:	shl    $0x5,%rbx
   9aa58:	mov    0x20(%rsp),%rax
   9aa5d:	mov    0x38(%rax),%rbp
   9aa61:	mov    0x40(%rax),%rax
   9aa65:	mov    %rax,0x10(%rsp)
   9aa6a:	xor    %r13d,%r13d
   9aa6d:	lea    0x92dc4(%rip),%rax        # 12d838 <__frame_dummy_init_array_entry+0x16e8>
   9aa74:	mov    %rax,0x30(%rsp)
   9aa79:	jmp    9aade <mibz2::bwt::room_segments::<32, 1>+0x126e>
   9aa7b:	mov    $0x8,%eax
   9aa80:	mov    %rax,0x8(%rsp)
   9aa85:	xor    %ebx,%ebx
   9aa87:	jmp    9ad16 <mibz2::bwt::room_segments::<32, 1>+0x14a6>
   9aa8c:	mov    %r14,%rdi
   9aa8f:	mov    %r9,%rbx
   9aa92:	call   9b1d0 <<alloc::raw_vec::RawVec<u32>>::grow_one>
   9aa97:	mov    %rbx,%r9
   9aa9a:	mov    0x4c(%rsp),%esi
   9aa9e:	mov    0x8(%r14),%rbx
   9aaa2:	jmp    9995b <mibz2::bwt::room_segments::<32, 1>+0xeb>
   9aaa7:	mov    0x40(%rsp),%rax
   9aaac:	mov    (%rax),%rdx
   9aaaf:	mov    0x8(%rax),%rax
   9aab3:	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   9aac0:	add    %rax,%r14
   9aac3:	mov    0x8(%rsp),%rcx
   9aac8:	mov    %r14,0x8(%rcx,%r13,1)
   9aacd:	add    %rax,%rdx
   9aad0:	mov    %rdx,0x10(%rcx,%r13,1)
   9aad5:	add    $0x20,%r13
   9aad9:	cmp    %r13,%rbx
   9aadc:	je     9ab4c <mibz2::bwt::room_segments::<32, 1>+0x12dc>
   9aade:	mov    0x8(%rsp),%rax
   9aae3:	mov    0x8(%rax,%r13,1),%r14
   9aae8:	cmp    0x10(%rax,%r13,1),%r14
   9aaed:	jne    9aad5 <mibz2::bwt::room_segments::<32, 1>+0x1265>
   9aaef:	mov    0x8(%rsp),%rax
   9aaf4:	mov    (%rax,%r13,1),%rdi
   9aaf8:	cmp    0x10(%rsp),%rdi
   9aafd:	jae    9afed <mibz2::bwt::room_segments::<32, 1>+0x177d>
   9ab03:	lea    (%rdi,%rdi,2),%rcx
   9ab07:	mov    0x0(%rbp,%rcx,8),%rdx
   9ab0c:	mov    0x8(%rbp,%rcx,8),%rax
   9ab11:	sub    %rax,%r14
   9ab14:	mov    %r14,0x10(%rbp,%rcx,8)
   9ab19:	mov    %rdx,%rsi
   9ab1c:	sub    %r14,%rsi
   9ab1f:	cmp    %rsi,%rdx
   9ab22:	jbe    9aac0 <mibz2::bwt::room_segments::<32, 1>+0x1250>
   9ab24:	lea    0x0(,%rcx,8),%rdi
   9ab2c:	add    %rbp,%rdi
   9ab2f:	mov    $0x1,%ecx
   9ab34:	mov    $0x1,%r8d
   9ab3a:	mov    %rdi,0x40(%rsp)
   9ab3f:	mov    %r14,%rsi
   9ab42:	call   98d90 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   9ab47:	jmp    9aaa7 <mibz2::bwt::room_segments::<32, 1>+0x1237>
   9ab4c:	mov    0x20(%rsp),%rax
   9ab51:	mov    0x8(%rax),%rcx
   9ab55:	mov    %rcx,0x38(%rsp)
   9ab5a:	mov    0x10(%rax),%rcx
   9ab5e:	mov    %rcx,0x40(%rsp)
   9ab63:	mov    0x50(%rax),%rcx
   9ab67:	mov    %rcx,0x60(%rsp)
   9ab6c:	mov    0x58(%rax),%rax
   9ab70:	mov    %rax,0x28(%rsp)
   9ab75:	xor    %r13d,%r13d
   9ab78:	jmp    9ab83 <mibz2::bwt::room_segments::<32, 1>+0x1313>
   9ab7a:	nopw   0x0(%rax,%rax,1)
   9ab80:	xor    %r13d,%r13d
   9ab83:	mov    %r13,%rbx
   9ab86:	shl    $0x5,%rbx
   9ab8a:	add    0x8(%rsp),%rbx
   9ab8f:	jmp    9abba <mibz2::bwt::room_segments::<32, 1>+0x134a>
   9ab91:	mov    0x58(%rsp),%rax
   9ab96:	mov    (%rax),%rdx
   9ab99:	mov    0x8(%rax),%rax
   9ab9d:	nopl   (%rax)
   9aba0:	add    %rax,%r14
   9aba3:	mov    %r14,0x8(%rbx)
   9aba7:	add    %rax,%rdx
   9abaa:	mov    %rdx,0x10(%rbx)
   9abae:	inc    %r13
   9abb1:	add    $0x20,%rbx
   9abb5:	cmp    %r12,%r13
   9abb8:	jae    9ab80 <mibz2::bwt::room_segments::<32, 1>+0x1310>
   9abba:	mov    0x18(%rbx),%eax
   9abbd:	mov    (%r15,%rax,4),%ecx
   9abc1:	test   %ecx,%ecx
   9abc3:	js     9ac40 <mibz2::bwt::room_segments::<32, 1>+0x13d0>
   9abc5:	mov    %ecx,%eax
   9abc7:	shr    $0x8,%eax
   9abca:	mov    %eax,0x18(%rbx)
   9abcd:	prefetcht0 (%r15,%rax,4)
   9abd2:	mov    0x8(%rbx),%rax
   9abd6:	mov    %cl,(%rax)
   9abd8:	mov    0x8(%rbx),%r14
   9abdc:	inc    %r14
   9abdf:	mov    %r14,0x8(%rbx)
   9abe3:	cmp    0x10(%rbx),%r14
   9abe7:	jne    9abae <mibz2::bwt::room_segments::<32, 1>+0x133e>
   9abe9:	mov    (%rbx),%rdi
   9abec:	cmp    0x10(%rsp),%rdi
   9abf1:	jae    9afed <mibz2::bwt::room_segments::<32, 1>+0x177d>
   9abf7:	lea    (%rdi,%rdi,2),%rcx
   9abfb:	mov    0x0(%rbp,%rcx,8),%rdx
   9ac00:	mov    0x8(%rbp,%rcx,8),%rax
   9ac05:	sub    %rax,%r14
   9ac08:	mov    %r14,0x10(%rbp,%rcx,8)
   9ac0d:	mov    %rdx,%rsi
   9ac10:	sub    %r14,%rsi
   9ac13:	cmp    %rsi,%rdx
   9ac16:	jbe    9aba0 <mibz2::bwt::room_segments::<32, 1>+0x1330>
   9ac18:	lea    0x0(,%rcx,8),%rdi
   9ac20:	add    %rbp,%rdi
   9ac23:	mov    $0x1,%ecx
   9ac28:	mov    $0x1,%r8d
   9ac2e:	mov    %rdi,0x58(%rsp)
   9ac33:	mov    %r14,%rsi
   9ac36:	call   98d90 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   9ac3b:	jmp    9ab91 <mibz2::bwt::room_segments::<32, 1>+0x1321>
   9ac40:	mov    (%rbx),%rdi
   9ac43:	cmp    0x10(%rsp),%rdi
   9ac48:	jae    9afce <mibz2::bwt::room_segments::<32, 1>+0x175e>
   9ac4e:	lea    (%rdi,%rdi,2),%rcx
   9ac52:	mov    0x8(%rbx),%rdx
   9ac56:	sub    0x8(%rbp,%rcx,8),%rdx
   9ac5b:	mov    %rdx,0x10(%rbp,%rcx,8)
   9ac60:	mov    0x40(%rsp),%rdx
   9ac65:	test   %rdx,%rdx
   9ac68:	je     9ac79 <mibz2::bwt::room_segments::<32, 1>+0x1409>
   9ac6a:	xor    %ecx,%ecx
   9ac6c:	cmp    $0x1,%rdx
   9ac70:	jne    9ac82 <mibz2::bwt::room_segments::<32, 1>+0x1412>
   9ac72:	mov    0x38(%rsp),%r9
   9ac77:	jmp    9acad <mibz2::bwt::room_segments::<32, 1>+0x143d>
   9ac79:	mov    $0xffffffffffffffff,%rcx
   9ac80:	jmp    9acbc <mibz2::bwt::room_segments::<32, 1>+0x144c>
   9ac82:	mov    0x38(%rsp),%r9
   9ac87:	nopw   0x0(%rax,%rax,1)
   9ac90:	mov    %rcx,%rsi
   9ac93:	mov    %rdx,%r8
   9ac96:	shr    $1,%r8
   9ac99:	add    %r8,%rcx
   9ac9c:	cmp    %eax,(%r9,%rcx,4)
   9aca0:	cmova  %rsi,%rcx
   9aca4:	sub    %r8,%rdx
   9aca7:	cmp    $0x1,%rdx
   9acab:	ja     9ac90 <mibz2::bwt::room_segments::<32, 1>+0x1420>
   9acad:	cmp    %eax,(%r9,%rcx,4)
   9acb1:	mov    $0xffffffffffffffff,%rax
   9acb8:	cmovne %rax,%rcx
   9acbc:	cmp    0x28(%rsp),%rdi
   9acc1:	jae    9afd7 <mibz2::bwt::room_segments::<32, 1>+0x1767>
   9acc7:	mov    0x60(%rsp),%rax
   9accc:	mov    %rcx,(%rax,%rdi,8)
   9acd0:	dec    %r12
   9acd3:	mov    %r12,%rax
   9acd6:	shl    $0x5,%rax
   9acda:	mov    0x8(%rsp),%rcx
   9acdf:	movups (%rcx,%rax,1),%xmm0
   9ace3:	movups 0x10(%rcx,%rax,1),%xmm1
   9ace8:	movups %xmm1,0x10(%rbx)
   9acec:	movups %xmm0,(%rbx)
   9acef:	cmp    %r12,%r13
   9acf2:	jb     9ab83 <mibz2::bwt::room_segments::<32, 1>+0x1313>
   9acf8:	mov    $0x0,%r13d
   9acfe:	test   %r12,%r12
   9ad01:	jne    9ab83 <mibz2::bwt::room_segments::<32, 1>+0x1313>
   9ad07:	mov    0x38(%rsp),%r9
   9ad0c:	mov    0x40(%rsp),%r10
   9ad11:	mov    0x18(%rsp),%rbx
   9ad16:	mov    $0xffffffffffffffff,%rax
   9ad1d:	test   %r10,%r10
   9ad20:	mov    0x20(%rsp),%r8
   9ad25:	mov    0x50(%rsp),%rsi
   9ad2a:	je     9adac <mibz2::bwt::room_segments::<32, 1>+0x153c>
   9ad30:	lea    0x0(,%r10,4),%rcx
   9ad38:	xor    %edx,%edx
   9ad3a:	nopw   0x0(%rax,%rax,1)
   9ad40:	mov    (%r9,%rdx,1),%edi
   9ad44:	cmp    %rdi,%rsi
   9ad47:	jbe    9b006 <mibz2::bwt::room_segments::<32, 1>+0x1796>
   9ad4d:	andb   $0x7f,0x3(%r15,%rdi,4)
   9ad53:	add    $0x4,%rdx
   9ad57:	cmp    %rdx,%rcx
   9ad5a:	jne    9ad40 <mibz2::bwt::room_segments::<32, 1>+0x14d0>
   9ad5c:	test   %r10,%r10
   9ad5f:	je     9adac <mibz2::bwt::room_segments::<32, 1>+0x153c>
   9ad61:	xor    %ecx,%ecx
   9ad63:	cmp    $0x1,%r10
   9ad67:	jne    9ad6f <mibz2::bwt::room_segments::<32, 1>+0x14ff>
   9ad69:	mov    0x4c(%rsp),%edi
   9ad6d:	jmp    9ad9d <mibz2::bwt::room_segments::<32, 1>+0x152d>
   9ad6f:	mov    0x4c(%rsp),%edi
   9ad73:	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   9ad80:	mov    %rcx,%rax
   9ad83:	mov    %r10,%rdx
   9ad86:	shr    $1,%rdx
   9ad89:	add    %rdx,%rcx
   9ad8c:	cmp    %edi,(%r9,%rcx,4)
   9ad90:	cmova  %rax,%rcx
   9ad94:	sub    %rdx,%r10
   9ad97:	cmp    $0x1,%r10
   9ad9b:	ja     9ad80 <mibz2::bwt::room_segments::<32, 1>+0x1510>
   9ad9d:	cmp    %edi,(%r9,%rcx,4)
   9ada1:	mov    $0xffffffffffffffff,%rax
   9ada8:	cmove  %rcx,%rax
   9adac:	mov    %rax,0x68(%r8)
   9adb0:	mov    0xf0(%rsp),%rax
   9adb8:	mov    %rax,0x60(%r8)
   9adbc:	test   %rbx,%rbx
   9adbf:	je     9adcc <mibz2::bwt::room_segments::<32, 1>+0x155c>
   9adc1:	mov    0x8(%rsp),%rdi
   9adc6:	call   *0x958bc(%rip)        # 130688 <free@GLIBC_2.2.5>
   9adcc:	add    $0x2638,%rsp
   9add3:	pop    %rbx
   9add4:	pop    %r12
   9add6:	pop    %r13
   9add8:	pop    %r14
   9adda:	pop    %r15
   9addc:	pop    %rbp
   9addd:	ret
   9adde:	lea    0xf0(%rsp),%rax
   9ade6:	mov    %rax,0x630(%rsp)
   9adee:	lea    -0x7095(%rip),%rax        # 93d60 <<u64 as core::fmt::Display>::fmt>
   9adf5:	mov    %rax,0x638(%rsp)
   9adfd:	lea    -0x8ed25(%rip),%rdi        # c0df <__abi_tag+0xbde3>
   9ae04:	lea    0x92745(%rip),%rdx        # 12d550 <__frame_dummy_init_array_entry+0x1400>
   9ae0b:	lea    0x630(%rsp),%rsi
   9ae13:	call   89b20 <core::panicking::panic_fmt>
   9ae18:	mov    $0x1ff,%edx
   9ae1d:	mov    %rcx,%rbx
   9ae20:	mov    $0x4,%ecx
   9ae25:	mov    $0x4,%r8d
   9ae2b:	mov    %esi,%ebp
   9ae2d:	mov    %rbx,%rdi
   9ae30:	xor    %esi,%esi
   9ae32:	call   98d90 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   9ae37:	mov    %ebp,%esi
   9ae39:	mov    %rbx,%rcx
   9ae3c:	mov    0x10(%rbx),%r9
   9ae40:	jmp    998ed <mibz2::bwt::room_segments::<32, 1>+0x7d>
   9ae45:	mov    %r13,%rsi
   9ae48:	cmp    $0x15,%r13
   9ae4c:	jae    9b0a9 <mibz2::bwt::room_segments::<32, 1>+0x1839>
   9ae52:	lea    (%rbx,%rsi,4),%rax
   9ae56:	lea    0x4(%rbx),%rcx
   9ae5a:	mov    $0x4,%edx
   9ae5f:	mov    0x20(%rsp),%rbp
   9ae64:	jmp    9ae7c <mibz2::bwt::room_segments::<32, 1>+0x160c>
   9ae66:	mov    %rbx,%rdi
   9ae69:	mov    %esi,(%rdi)
   9ae6b:	add    $0x4,%rcx
   9ae6f:	add    $0x4,%rdx
   9ae73:	cmp    %rax,%rcx
   9ae76:	je     9b0bc <mibz2::bwt::room_segments::<32, 1>+0x184c>
   9ae7c:	mov    -0x4(%rcx),%r8d
   9ae80:	mov    (%rcx),%esi
   9ae82:	cmp    %r8d,%esi
   9ae85:	jae    9ae6b <mibz2::bwt::room_segments::<32, 1>+0x15fb>
   9ae87:	mov    %rdx,%rdi
   9ae8a:	mov    %r8d,(%rbx,%rdi,1)
   9ae8e:	cmp    $0x4,%rdi
   9ae92:	je     9ae66 <mibz2::bwt::room_segments::<32, 1>+0x15f6>
   9ae94:	mov    -0x8(%rbx,%rdi,1),%r8d
   9ae99:	add    $0xfffffffffffffffc,%rdi
   9ae9d:	cmp    %r8d,%esi
   9aea0:	jb     9ae8a <mibz2::bwt::room_segments::<32, 1>+0x161a>
   9aea2:	add    %rbx,%rdi
   9aea5:	jmp    9ae69 <mibz2::bwt::room_segments::<32, 1>+0x15f9>
   9aea7:	lea    0x48(%rbp),%rdi
   9aeab:	mov    $0x8,%ecx
   9aeb0:	mov    $0x8,%r8d
   9aeb6:	xor    %esi,%esi
   9aeb8:	mov    %r13,%rdx
   9aebb:	call   98d90 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   9aec0:	mov    0x58(%rbp),%rbx
   9aec4:	mov    0x50(%rbp),%r14
   9aec8:	lea    (%r14,%rbx,8),%rdi
   9aecc:	cmp    $0x1,%r13
   9aed0:	jne    99aad <mibz2::bwt::room_segments::<32, 1>+0x23d>
   9aed6:	jmp    99ad2 <mibz2::bwt::room_segments::<32, 1>+0x262>
   9aedb:	lea    0x30(%rbp),%rdi
   9aedf:	mov    $0x8,%ecx
   9aee4:	mov    $0x18,%r8d
   9aeea:	mov    %r14,%rsi
   9aeed:	mov    %rbx,%rdx
   9aef0:	call   98d90 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   9aef5:	mov    0x40(%rbp),%rdx
   9aef9:	mov    0x38(%rbp),%rax
   9aefd:	inc    %r14
   9af00:	test   $0x1,%bl
   9af03:	je     99a16 <mibz2::bwt::room_segments::<32, 1>+0x1a6>
   9af09:	dec    %rbx
   9af0c:	lea    (%rdx,%rdx,2),%rcx
   9af10:	movq   $0x0,(%rax,%rcx,8)
   9af18:	movq   $0x1,0x8(%rax,%rcx,8)
   9af21:	movq   $0x0,0x10(%rax,%rcx,8)
   9af2a:	inc    %rdx
   9af2d:	cmp    %r14,%r13
   9af30:	je     99a1f <mibz2::bwt::room_segments::<32, 1>+0x1af>
   9af36:	lea    (%rdx,%rdx,2),%rcx
   9af3a:	lea    (%rax,%rcx,8),%rax
   9af3e:	add    $0x28,%rax
   9af42:	neg    %rbx
   9af45:	xor    %ecx,%ecx
   9af47:	xorps  %xmm0,%xmm0
   9af4a:	nopw   0x0(%rax,%rax,1)
   9af50:	movq   $0x0,-0x28(%rax)
   9af58:	movq   $0x1,-0x20(%rax)
   9af60:	movups %xmm0,-0x18(%rax)
   9af64:	movq   $0x1,-0x8(%rax)
   9af6c:	movq   $0x0,(%rax)
   9af73:	add    $0x30,%rax
   9af77:	add    $0xfffffffffffffffe,%rcx
   9af7b:	cmp    %rcx,%rbx
   9af7e:	jne    9af50 <mibz2::bwt::room_segments::<32, 1>+0x16e0>
   9af80:	sub    %rcx,%rdx
   9af83:	mov    %rdx,0x40(%rbp)
   9af87:	cmp    %rdx,%r13
   9af8a:	jbe    99a2c <mibz2::bwt::room_segments::<32, 1>+0x1bc>
   9af90:	jmp    9a80f <mibz2::bwt::room_segments::<32, 1>+0xf9f>
   9af95:	lea    0x9289c(%rip),%rdx        # 12d838 <__frame_dummy_init_array_entry+0x16e8>
   9af9c:	call   8a87b <core::panicking::panic_bounds_check>
   9afa1:	lea    0x925d8(%rip),%rdx        # 12d580 <__frame_dummy_init_array_entry+0x1430>
   9afa8:	call   8a87b <core::panicking::panic_bounds_check>
   9afad:	lea    0x928b4(%rip),%rdx        # 12d868 <__frame_dummy_init_array_entry+0x1718>
   9afb4:	mov    %rdi,%rsi
   9afb7:	call   8a87b <core::panicking::panic_bounds_check>
   9afbc:	lea    0x928bd(%rip),%rdx        # 12d880 <__frame_dummy_init_array_entry+0x1730>
   9afc3:	mov    %r10,%rdi
   9afc6:	mov    %r10,%rsi
   9afc9:	call   8a87b <core::panicking::panic_bounds_check>
   9afce:	lea    0x92833(%rip),%rax        # 12d808 <__frame_dummy_init_array_entry+0x16b8>
   9afd5:	jmp    9afe8 <mibz2::bwt::room_segments::<32, 1>+0x1778>
   9afd7:	mov    0x28(%rsp),%rax
   9afdc:	mov    %rax,0x10(%rsp)
   9afe1:	lea    0x92838(%rip),%rax        # 12d820 <__frame_dummy_init_array_entry+0x16d0>
   9afe8:	mov    %rax,0x30(%rsp)
   9afed:	mov    0x10(%rsp),%rsi
   9aff2:	mov    0x30(%rsp),%rdx
   9aff7:	mov    0x18(%rsp),%rbx
   9affc:	call   8a87b <core::panicking::panic_bounds_check>
   9b001:	jmp    9b08c <mibz2::bwt::room_segments::<32, 1>+0x181c>
   9b006:	lea    0x925bb(%rip),%rdx        # 12d5c8 <__frame_dummy_init_array_entry+0x1478>
   9b00d:	call   8a87b <core::panicking::panic_bounds_check>
   9b012:	jmp    9b08c <mibz2::bwt::room_segments::<32, 1>+0x181c>
   9b014:	lea    0x92805(%rip),%rdx        # 12d820 <__frame_dummy_init_array_entry+0x16d0>
   9b01b:	mov    0x28(%rsp),%rsi
   9b020:	call   8a87b <core::panicking::panic_bounds_check>
   9b025:	lea    0x927dc(%rip),%rdx        # 12d808 <__frame_dummy_init_array_entry+0x16b8>
   9b02c:	mov    %r8,%rsi
   9b02f:	call   8a87b <core::panicking::panic_bounds_check>
   9b034:	lea    0x92575(%rip),%rdx        # 12d5b0 <__frame_dummy_init_array_entry+0x1460>
   9b03b:	mov    $0x20,%esi
   9b040:	mov    %r14,%rdi
   9b043:	call   8a87b <core::panicking::panic_bounds_check>
   9b048:	lea    0x92831(%rip),%rdx        # 12d880 <__frame_dummy_init_array_entry+0x1730>
   9b04f:	mov    %r9,%rdi
   9b052:	call   8a87b <core::panicking::panic_bounds_check>
   9b057:	lea    0x9280a(%rip),%rdx        # 12d868 <__frame_dummy_init_array_entry+0x1718>
   9b05e:	mov    %r9,%rdi
   9b061:	mov    %r8,%rsi
   9b064:	call   8a87b <core::panicking::panic_bounds_check>
   9b069:	lea    0x927e0(%rip),%rdx        # 12d850 <__frame_dummy_init_array_entry+0x1700>
   9b070:	mov    %rbx,%rdi
   9b073:	mov    0x28(%rsp),%rsi
   9b078:	call   8a87b <core::panicking::panic_bounds_check>
   9b07d:	lea    0x927cc(%rip),%rdx        # 12d850 <__frame_dummy_init_array_entry+0x1700>
   9b084:	mov    %r14,%rdi
   9b087:	call   8a87b <core::panicking::panic_bounds_check>
   9b08c:	ud2
   9b08e:	lea    0x924d3(%rip),%rdx        # 12d568 <__frame_dummy_init_array_entry+0x1418>
   9b095:	call   8a87b <core::panicking::panic_bounds_check>
   9b09a:	mov    $0x8,%edi
   9b09f:	mov    $0x80,%esi
   9b0a4:	call   88c9b <alloc::raw_vec::handle_error>
   9b0a9:	mov    %rbx,%rdi
   9b0ac:	mov    %r9,%r14
   9b0af:	call   9b240 <core::slice::sort::unstable::ipnsort::<u32, <u32 as core::cmp::PartialOrd>::lt>>
   9b0b4:	mov    %r14,%r9
   9b0b7:	mov    0x20(%rsp),%rbp
   9b0bc:	xor    %ecx,%ecx
   9b0be:	mov    0x4(%rbx,%rcx,4),%eax
   9b0c2:	cmp    (%rbx,%rcx,4),%eax
   9b0c5:	je     9b0d4 <mibz2::bwt::room_segments::<32, 1>+0x1864>
   9b0c7:	inc    %rcx
   9b0ca:	cmp    %rcx,%r12
   9b0cd:	jne    9b0be <mibz2::bwt::room_segments::<32, 1>+0x184e>
   9b0cf:	jmp    99979 <mibz2::bwt::room_segments::<32, 1>+0x109>
   9b0d4:	lea    0x1(%rcx),%rdx
   9b0d8:	lea    0x2(%rcx),%rax
   9b0dc:	cmp    %r13,%rax
   9b0df:	jae    9b0f5 <mibz2::bwt::room_segments::<32, 1>+0x1885>
   9b0e1:	mov    %r9d,%esi
   9b0e4:	sub    %ecx,%esi
   9b0e6:	add    $0x1fd,%r9
   9b0ed:	test   $0x1,%sil
   9b0f1:	jne    9b0ff <mibz2::bwt::room_segments::<32, 1>+0x188f>
   9b0f3:	jmp    9b120 <mibz2::bwt::room_segments::<32, 1>+0x18b0>
   9b0f5:	mov    %rdx,%r13
   9b0f8:	mov    0x50(%rsp),%rsi
   9b0fd:	jmp    9b12a <mibz2::bwt::room_segments::<32, 1>+0x18ba>
   9b0ff:	mov    0x8(%rbx,%rcx,4),%eax
   9b103:	mov    %rdx,%rsi
   9b106:	cmp    (%rbx,%rcx,4),%eax
   9b109:	je     9b113 <mibz2::bwt::room_segments::<32, 1>+0x18a3>
   9b10b:	mov    %eax,0x4(%rbx,%rcx,4)
   9b10f:	lea    0x1(%rdx),%rsi
   9b113:	add    $0x2,%rdx
   9b117:	mov    %rdx,%rax
   9b11a:	mov    %rsi,%r13
   9b11d:	mov    %rsi,%rdx
   9b120:	cmp    %rcx,%r9
   9b123:	mov    0x50(%rsp),%rsi
   9b128:	jne    9b143 <mibz2::bwt::room_segments::<32, 1>+0x18d3>
   9b12a:	mov    %r13,0x10(%rbp)
   9b12e:	test   %r13,%r13
   9b131:	jne    99987 <mibz2::bwt::room_segments::<32, 1>+0x117>
   9b137:	lea    0x9245a(%rip),%rdi        # 12d598 <__frame_dummy_init_array_entry+0x1448>
   9b13e:	call   8d840 <core::panicking::panic_const::panic_const_div_by_zero>
   9b143:	inc    %r12
   9b146:	mov    %rdx,%r13
   9b149:	jmp    9b154 <mibz2::bwt::room_segments::<32, 1>+0x18e4>
   9b14b:	add    $0x2,%rax
   9b14f:	cmp    %rax,%r12
   9b152:	je     9b12a <mibz2::bwt::room_segments::<32, 1>+0x18ba>
   9b154:	mov    (%rbx,%rax,4),%ecx
   9b157:	cmp    -0x4(%rbx,%r13,4),%ecx
   9b15c:	jne    9b16b <mibz2::bwt::room_segments::<32, 1>+0x18fb>
   9b15e:	mov    0x4(%rbx,%rax,4),%ecx
   9b162:	cmp    -0x4(%rbx,%r13,4),%ecx
   9b167:	je     9b14b <mibz2::bwt::room_segments::<32, 1>+0x18db>
   9b169:	jmp    9b17d <mibz2::bwt::room_segments::<32, 1>+0x190d>
   9b16b:	mov    %ecx,(%rbx,%r13,4)
   9b16f:	inc    %r13
   9b172:	mov    0x4(%rbx,%rax,4),%ecx
   9b176:	cmp    -0x4(%rbx,%r13,4),%ecx
   9b17b:	je     9b14b <mibz2::bwt::room_segments::<32, 1>+0x18db>
   9b17d:	mov    %ecx,(%rbx,%r13,4)
   9b181:	inc    %r13
   9b184:	jmp    9b14b <mibz2::bwt::room_segments::<32, 1>+0x18db>
   9b186:	jmp    9b18a <mibz2::bwt::room_segments::<32, 1>+0x191a>
   9b188:	jmp    9b194 <mibz2::bwt::room_segments::<32, 1>+0x1924>
   9b18a:	mov    %rax,%r14
   9b18d:	mov    0x18(%rsp),%rbx
   9b192:	jmp    9b1b4 <mibz2::bwt::room_segments::<32, 1>+0x1944>
   9b194:	mov    %rax,%r14
   9b197:	cmpq   $0x0,0x530(%rsp)
   9b1a0:	je     9b1c4 <mibz2::bwt::room_segments::<32, 1>+0x1954>
   9b1a2:	mov    0x538(%rsp),%rax
   9b1aa:	mov    %rax,0x8(%rsp)
   9b1af:	jmp    9b1b9 <mibz2::bwt::room_segments::<32, 1>+0x1949>
   9b1b1:	mov    %rax,%r14
   9b1b4:	test   %rbx,%rbx
   9b1b7:	je     9b1c4 <mibz2::bwt::room_segments::<32, 1>+0x1954>
   9b1b9:	mov    0x8(%rsp),%rdi
   9b1be:	call   *0x954c4(%rip)        # 130688 <free@GLIBC_2.2.5>
   9b1c4:	mov    %r14,%rdi
   9b1c7:	call   12b0b0 <_Unwind_Resume@plt>
   9b1cc:	int3
   9b1cd:	int3
   9b1ce:	int3
   9b1cf:	int3

#### <mibz2::c30::C30<false>>::room: 1439 instructions, 21 prefetcht0, 41 calls
   b5fb0:	push   %rbp
   b5fb1:	push   %r15
   b5fb3:	push   %r14
   b5fb5:	push   %r13
   b5fb7:	push   %r12
   b5fb9:	push   %rbx
   b5fba:	sub    $0x1000,%rsp
   b5fc1:	movq   $0x0,(%rsp)
   b5fc9:	sub    $0x1000,%rsp
   b5fd0:	movq   $0x0,(%rsp)
   b5fd8:	sub    $0x688,%rsp
   b5fdf:	mov    %esi,0x50(%rsp)
   b5fe3:	mov    0xd0(%rdi),%rax
   b5fea:	mov    %rax,0xd8(%rsp)
   b5ff2:	mov    0x10(%rdi),%eax
   b5ff5:	mov    0x40(%rdi),%rbx
   b5ff9:	movq   $0x0,0xc8(%rdi)
   b6004:	movq   $0x0,0xb0(%rdi)
   b600f:	test   %rbx,%rbx
   b6012:	je     b7ba8 <<mibz2::c30::C30<false>>::room+0x1bf8>
   b6018:	mov    %rdi,%r15
   b601b:	shl    $0x5,%eax
   b601e:	add    $0xffffffe0,%eax
   b6021:	mov    %eax,0x54(%rsp)
   b6025:	xor    %edi,%edi
   b6027:	lea    0xb8(%r15),%rax
   b602e:	mov    %rax,0x18(%rsp)
   b6033:	lea    0xa0(%r15),%rax
   b603a:	mov    %rax,0x8(%rsp)
   b603f:	mov    %rbx,0x38(%rsp)
   b6044:	jmp    b6076 <<mibz2::c30::C30<false>>::room+0xc6>
   b6046:	cs nopw 0x0(%rax,%rax,1)
   b6050:	inc    %rdi
   b6053:	mov    0xa8(%r15),%rax
   b605a:	mov    %bp,(%rax,%rbx,2)
   b605e:	inc    %rbx
   b6061:	mov    %rbx,0xb0(%r15)
   b6068:	mov    0x38(%rsp),%rbx
   b606d:	cmp    %rdi,%rbx
   b6070:	je     b61af <<mibz2::c30::C30<false>>::room+0x1ff>
   b6076:	mov    0x40(%r15),%rsi
   b607a:	cmp    %rsi,%rdi
   b607d:	jae    b7aa3 <<mibz2::c30::C30<false>>::room+0x1af3>
   b6083:	mov    0x38(%r15),%rax
   b6087:	mov    (%rax,%rdi,4),%ebp
   b608a:	mov    0x10(%r15),%rsi
   b608e:	mov    %ebp,%r12d
   b6091:	shr    $0x5,%r12d
   b6095:	cmp    %r12,%rsi
   b6098:	jbe    b7a80 <<mibz2::c30::C30<false>>::room+0x1ad0>
   b609e:	and    $0x1f,%ebp
   b60a1:	cmp    $0x1e,%ebp
   b60a4:	jae    b7a8f <<mibz2::c30::C30<false>>::room+0x1adf>
   b60aa:	mov    0x8(%r15),%rcx
   b60ae:	mov    %r12,%rdx
   b60b1:	shl    $0x6,%rdx
   b60b5:	lea    (%rcx,%rdx,1),%rax
   b60b9:	movzwl (%rax,%rbp,2),%eax
   b60bd:	mov    %r15,%rbx
   b60c0:	mov    0x3c(%rcx,%rdx,1),%r13d
   b60c5:	cmp    $0xfffe,%eax
   b60ca:	jae    b6150 <<mibz2::c30::C30<false>>::room+0x1a0>
   b60d0:	mov    %r13d,%r14d
   b60d3:	shr    $0x8,%r14d
   b60d7:	add    %eax,%r14d
   b60da:	mov    0xc8(%rbx),%r15
   b60e1:	cmp    0xb8(%rbx),%r15
   b60e8:	je     b618c <<mibz2::c30::C30<false>>::room+0x1dc>
   b60ee:	shl    $0x8,%r14d
   b60f2:	movzbl %r13b,%eax
   b60f6:	or     %r14d,%eax
   b60f9:	mov    0xc0(%rbx),%rcx
   b6100:	mov    %eax,(%rcx,%r15,4)
   b6104:	inc    %r15
   b6107:	mov    %r15,0xc8(%rbx)
   b610e:	mov    0x10(%rbx),%rsi
   b6112:	cmp    %r12,%rsi
   b6115:	jbe    b7a80 <<mibz2::c30::C30<false>>::room+0x1ad0>
   b611b:	mov    %rbx,%r15
   b611e:	add    0x8(%rbx),%rdx
   b6122:	movzwl (%rdx,%rbp,2),%ebp
   b6126:	mov    0xb0(%rbx),%rbx
   b612d:	cmp    0xa0(%r15),%rbx
   b6134:	jne    b6050 <<mibz2::c30::C30<false>>::room+0xa0>
   b613a:	mov    %rdi,%r14
   b613d:	mov    0x8(%rsp),%rdi
   b6142:	call   b7c50 <<alloc::raw_vec::RawVec<u16>>::grow_one>
   b6147:	mov    %r14,%rdi
   b614a:	jmp    b6050 <<mibz2::c30::C30<false>>::room+0xa0>
   b614f:	nop
   b6150:	and    $0xfffff,%r13d
   b6157:	mov    0x28(%rbx),%rsi
   b615b:	cmp    %r13,%rsi
   b615e:	jbe    b7b1d <<mibz2::c30::C30<false>>::room+0x1b6d>
   b6164:	shl    $0x7,%r13d
   b6168:	add    0x20(%rbx),%r13
   b616c:	mov    0x0(%r13,%rbp,4),%r13d
   b6171:	mov    %r13d,%r14d
   b6174:	shr    $0x8,%r14d
   b6178:	mov    0xc8(%rbx),%r15
   b617f:	cmp    0xb8(%rbx),%r15
   b6186:	jne    b60ee <<mibz2::c30::C30<false>>::room+0x13e>
   b618c:	mov    %rdi,0x28(%rsp)
   b6191:	mov    0x18(%rsp),%rdi
   b6196:	mov    %rdx,0x20(%rsp)
   b619b:	call   9b1d0 <<alloc::raw_vec::RawVec<u32>>::grow_one>
   b61a0:	mov    0x20(%rsp),%rdx
   b61a5:	mov    0x28(%rsp),%rdi
   b61aa:	jmp    b60ee <<mibz2::c30::C30<false>>::room+0x13e>
   b61af:	xor    %edi,%edi
   b61b1:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   b61c0:	mov    0x40(%r15),%rsi
   b61c4:	cmp    %rsi,%rdi
   b61c7:	jae    b7ad2 <<mibz2::c30::C30<false>>::room+0x1b22>
   b61cd:	mov    0x38(%r15),%rax
   b61d1:	mov    (%rax,%rdi,4),%eax
   b61d4:	mov    0x10(%r15),%rsi
   b61d8:	mov    %eax,%ecx
   b61da:	shr    $0x5,%ecx
   b61dd:	cmp    %rcx,%rsi
   b61e0:	jbe    b7ac3 <<mibz2::c30::C30<false>>::room+0x1b13>
   b61e6:	and    $0x1f,%eax
   b61e9:	cmp    $0x1e,%eax
   b61ec:	jae    b7aaf <<mibz2::c30::C30<false>>::room+0x1aff>
   b61f2:	inc    %rdi
   b61f5:	shl    $0x6,%rcx
   b61f9:	add    0x8(%r15),%rcx
   b61fd:	movw   $0xfffe,(%rcx,%rax,2)
   b6203:	cmp    %rdi,%rbx
   b6206:	jne    b61c0 <<mibz2::c30::C30<false>>::room+0x210>
   b6208:	mov    0x8(%r15),%r14
   b620c:	mov    0x20(%r15),%rax
   b6210:	mov    %rax,0x28(%rsp)
   b6215:	mov    0xd8(%rsp),%rax
   b621d:	add    %rax,%rax
   b6220:	mov    %rax,%rcx
   b6223:	or     %rbx,%rcx
   b6226:	shr    $0x20,%rcx
   b622a:	je     b6233 <<mibz2::c30::C30<false>>::room+0x283>
   b622c:	xor    %edx,%edx
   b622e:	div    %rbx
   b6231:	jmp    b6237 <<mibz2::c30::C30<false>>::room+0x287>
   b6233:	xor    %edx,%edx
   b6235:	div    %ebx
   b6237:	mov    %rax,0x48(%rsp)
   b623c:	mov    0x70(%r15),%r13
   b6240:	cmp    %rbx,%r13
   b6243:	mov    %rbx,%r12
   b6246:	cmova  %r13,%r12
   b624a:	mov    %r15,0x20(%rsp)
   b624f:	jae    b628c <<mibz2::c30::C30<false>>::room+0x2dc>
   b6251:	sub    %r13,%r12
   b6254:	mov    0x60(%r15),%rax
   b6258:	sub    %r13,%rax
   b625b:	mov    %r13,%rdx
   b625e:	cmp    %rax,%r12
   b6261:	ja     b6298 <<mibz2::c30::C30<false>>::room+0x2e8>
   b6263:	mov    0x68(%r15),%rax
   b6267:	inc    %r13
   b626a:	test   $0x1,%r12b
   b626e:	jne    b62c3 <<mibz2::c30::C30<false>>::room+0x313>
   b6270:	cmp    %r13,%rbx
   b6273:	mov    0x20(%rsp),%rsi
   b6278:	jne    b62f1 <<mibz2::c30::C30<false>>::room+0x341>
   b627a:	mov    %rdx,0x70(%rsi)
   b627e:	cmp    %rdx,%rbx
   b6281:	jbe    b6350 <<mibz2::c30::C30<false>>::room+0x3a0>
   b6287:	jmp    b7a41 <<mibz2::c30::C30<false>>::room+0x1a91>
   b628c:	mov    %r12,0x70(%r15)
   b6290:	mov    %r15,%rsi
   b6293:	jmp    b6350 <<mibz2::c30::C30<false>>::room+0x3a0>
   b6298:	lea    0x60(%r15),%rdi
   b629c:	mov    $0x8,%ecx
   b62a1:	mov    $0x18,%r8d
   b62a7:	mov    %r13,%rsi
   b62aa:	mov    %r12,%rdx
   b62ad:	call   98d90 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   b62b2:	mov    0x70(%r15),%rdx
   b62b6:	mov    0x68(%r15),%rax
   b62ba:	inc    %r13
   b62bd:	test   $0x1,%r12b
   b62c1:	je     b6270 <<mibz2::c30::C30<false>>::room+0x2c0>
   b62c3:	dec    %r12
   b62c6:	lea    (%rdx,%rdx,2),%rcx
   b62ca:	movq   $0x0,(%rax,%rcx,8)
   b62d2:	movq   $0x1,0x8(%rax,%rcx,8)
   b62db:	movq   $0x0,0x10(%rax,%rcx,8)
   b62e4:	inc    %rdx
   b62e7:	cmp    %r13,%rbx
   b62ea:	mov    0x20(%rsp),%rsi
   b62ef:	je     b627a <<mibz2::c30::C30<false>>::room+0x2ca>
   b62f1:	lea    (%rdx,%rdx,2),%rcx
   b62f5:	lea    (%rax,%rcx,8),%rax
   b62f9:	add    $0x28,%rax
   b62fd:	neg    %r12
   b6300:	xor    %ecx,%ecx
   b6302:	xorps  %xmm0,%xmm0
   b6305:	data16 cs nopw 0x0(%rax,%rax,1)
   b6310:	movq   $0x0,-0x28(%rax)
   b6318:	movq   $0x1,-0x20(%rax)
   b6320:	movups %xmm0,-0x18(%rax)
   b6324:	movq   $0x1,-0x8(%rax)
   b632c:	movq   $0x0,(%rax)
   b6333:	add    $0x30,%rax
   b6337:	add    $0xfffffffffffffffe,%rcx
   b633b:	cmp    %rcx,%r12
   b633e:	jne    b6310 <<mibz2::c30::C30<false>>::room+0x360>
   b6340:	sub    %rcx,%rdx
   b6343:	mov    %rdx,0x70(%rsi)
   b6347:	cmp    %rdx,%rbx
   b634a:	ja     b7a41 <<mibz2::c30::C30<false>>::room+0x1a91>
   b6350:	mov    0x48(%rsp),%rax
   b6355:	lea    0x40(%rax),%r12
   b6359:	mov    0x68(%rsi),%r13
   b635d:	lea    0x0(,%rbx,8),%rax
   b6365:	lea    (%rax,%rax,2),%rbx
   b6369:	nopl   0x0(%rax)
   b6370:	movq   $0x0,0x10(%r13)
   b6378:	cmp    0x0(%r13),%r12
   b637c:	ja     b638a <<mibz2::c30::C30<false>>::room+0x3da>
   b637e:	add    $0x18,%r13
   b6382:	add    $0xffffffffffffffe8,%rbx
   b6386:	jne    b6370 <<mibz2::c30::C30<false>>::room+0x3c0>
   b6388:	jmp    b63ac <<mibz2::c30::C30<false>>::room+0x3fc>
   b638a:	mov    $0x1,%ecx
   b638f:	mov    $0x1,%r8d
   b6395:	mov    %r13,%rdi
   b6398:	xor    %esi,%esi
   b639a:	mov    %r12,%rdx
   b639d:	call   98d90 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   b63a2:	add    $0x18,%r13
   b63a6:	add    $0xffffffffffffffe8,%rbx
   b63aa:	jne    b6370 <<mibz2::c30::C30<false>>::room+0x3c0>
   b63ac:	mov    0x20(%rsp),%r13
   b63b1:	movq   $0x0,0x88(%r13)
   b63bc:	mov    0x38(%rsp),%rdx
   b63c1:	cmp    0x78(%r13),%rdx
   b63c5:	ja     b7a52 <<mibz2::c30::C30<false>>::room+0x1aa2>
   b63cb:	xor    %ebx,%ebx
   b63cd:	mov    0x80(%r13),%r15
   b63d4:	lea    (%r15,%rbx,8),%rdi
   b63d8:	mov    0x38(%rsp),%r12
   b63dd:	cmp    $0x1,%r12
   b63e1:	je     b6408 <<mibz2::c30::C30<false>>::room+0x458>
   b63e3:	lea    -0x8(,%r12,8),%rdx
   b63eb:	mov    $0xff,%esi
   b63f0:	call   *0x7a302(%rip)        # 1306f8 <memset@GLIBC_2.2.5>
   b63f6:	lea    (%rbx,%r12,1),%rax
   b63fa:	lea    (%r15,%rax,8),%rdi
   b63fe:	add    $0xfffffffffffffff8,%rdi
   b6402:	add    %r12,%rbx
   b6405:	dec    %rbx
   b6408:	mov    0x28(%rsp),%rbp
   b640d:	movq   $0xffffffffffffffff,(%rdi)
   b6414:	inc    %rbx
   b6417:	mov    %rbx,0x88(%r13)
   b641e:	xorps  %xmm0,%xmm0
   b6421:	movups %xmm0,0x108(%rsp)
   b6429:	movq   $0x0,0x118(%rsp)
   b6435:	movups %xmm0,0x128(%rsp)
   b643d:	movq   $0x0,0x138(%rsp)
   b6449:	movups %xmm0,0x148(%rsp)
   b6451:	movq   $0x0,0x158(%rsp)
   b645d:	movups %xmm0,0x168(%rsp)
   b6465:	movq   $0x0,0x178(%rsp)
   b6471:	movups %xmm0,0x188(%rsp)
   b6479:	movq   $0x0,0x198(%rsp)
   b6485:	movups %xmm0,0x1a8(%rsp)
   b648d:	movq   $0x0,0x1b8(%rsp)
   b6499:	movups %xmm0,0x1c8(%rsp)
   b64a1:	movq   $0x0,0x1d8(%rsp)
   b64ad:	movups %xmm0,0x1e8(%rsp)
   b64b5:	movq   $0x0,0x1f8(%rsp)
   b64c1:	movups %xmm0,0x208(%rsp)
   b64c9:	movq   $0x0,0x218(%rsp)
   b64d5:	movups %xmm0,0x228(%rsp)
   b64dd:	movq   $0x0,0x238(%rsp)
   b64e9:	movups %xmm0,0x248(%rsp)
   b64f1:	movq   $0x0,0x258(%rsp)
   b64fd:	movups %xmm0,0x268(%rsp)
   b6505:	movq   $0x0,0x278(%rsp)
   b6511:	movups %xmm0,0x288(%rsp)
   b6519:	movq   $0x0,0x298(%rsp)
   b6525:	movups %xmm0,0x2a8(%rsp)
   b652d:	movq   $0x0,0x2b8(%rsp)
   b6539:	movups %xmm0,0x2c8(%rsp)
   b6541:	movq   $0x0,0x2d8(%rsp)
   b654d:	movups %xmm0,0x2e8(%rsp)
   b6555:	movq   $0x0,0x2f8(%rsp)
   b6561:	movups %xmm0,0x308(%rsp)
   b6569:	movq   $0x0,0x318(%rsp)
   b6575:	movups %xmm0,0x328(%rsp)
   b657d:	movq   $0x0,0x338(%rsp)
   b6589:	movups %xmm0,0x348(%rsp)
   b6591:	movq   $0x0,0x358(%rsp)
   b659d:	movups %xmm0,0x368(%rsp)
   b65a5:	movq   $0x0,0x378(%rsp)
   b65b1:	movups %xmm0,0x388(%rsp)
   b65b9:	movq   $0x0,0x398(%rsp)
   b65c5:	movups %xmm0,0x3a8(%rsp)
   b65cd:	movq   $0x0,0x3b8(%rsp)
   b65d9:	movups %xmm0,0x3c8(%rsp)
   b65e1:	movq   $0x0,0x3d8(%rsp)
   b65ed:	movups %xmm0,0x3e8(%rsp)
   b65f5:	movq   $0x0,0x3f8(%rsp)
   b6601:	movups %xmm0,0x408(%rsp)
   b6609:	movq   $0x0,0x418(%rsp)
   b6615:	movups %xmm0,0x428(%rsp)
   b661d:	movq   $0x0,0x438(%rsp)
   b6629:	movups %xmm0,0x448(%rsp)
   b6631:	movq   $0x0,0x458(%rsp)
   b663d:	movups %xmm0,0x468(%rsp)
   b6645:	movq   $0x0,0x478(%rsp)
   b6651:	movups %xmm0,0x488(%rsp)
   b6659:	movq   $0x0,0x498(%rsp)
   b6665:	movups %xmm0,0x4a8(%rsp)
   b666d:	movq   $0x0,0x4b8(%rsp)
   b6679:	movups %xmm0,0x4c8(%rsp)
   b6681:	movq   $0x0,0x4d8(%rsp)
   b668d:	movups %xmm0,0x4e8(%rsp)
   b6695:	movq   $0x0,0x4f8(%rsp)
   b66a1:	movq   $0xffffffffffffffff,0x100(%rsp)
   b66ad:	movq   $0xffffffffffffffff,0x120(%rsp)
   b66b9:	movq   $0xffffffffffffffff,0x140(%rsp)
   b66c5:	movq   $0xffffffffffffffff,0x160(%rsp)
   b66d1:	movq   $0xffffffffffffffff,0x180(%rsp)
   b66dd:	movq   $0xffffffffffffffff,0x1a0(%rsp)
   b66e9:	movq   $0xffffffffffffffff,0x1c0(%rsp)
   b66f5:	movq   $0xffffffffffffffff,0x1e0(%rsp)
   b6701:	movq   $0xffffffffffffffff,0x200(%rsp)
   b670d:	movq   $0xffffffffffffffff,0x220(%rsp)
   b6719:	movq   $0xffffffffffffffff,0x240(%rsp)
   b6725:	movq   $0xffffffffffffffff,0x260(%rsp)
   b6731:	movq   $0xffffffffffffffff,0x280(%rsp)
   b673d:	movq   $0xffffffffffffffff,0x2a0(%rsp)
   b6749:	movq   $0xffffffffffffffff,0x2c0(%rsp)
   b6755:	movq   $0xffffffffffffffff,0x2e0(%rsp)
   b6761:	movq   $0xffffffffffffffff,0x300(%rsp)
   b676d:	movq   $0xffffffffffffffff,0x320(%rsp)
   b6779:	movq   $0xffffffffffffffff,0x340(%rsp)
   b6785:	movq   $0xffffffffffffffff,0x360(%rsp)
   b6791:	movq   $0xffffffffffffffff,0x380(%rsp)
   b679d:	movq   $0xffffffffffffffff,0x3a0(%rsp)
   b67a9:	movq   $0xffffffffffffffff,0x3c0(%rsp)
   b67b5:	movq   $0xffffffffffffffff,0x3e0(%rsp)
   b67c1:	movq   $0xffffffffffffffff,0x400(%rsp)
   b67cd:	movq   $0xffffffffffffffff,0x420(%rsp)
   b67d9:	movq   $0xffffffffffffffff,0x440(%rsp)
   b67e5:	movq   $0xffffffffffffffff,0x460(%rsp)
   b67f1:	movq   $0xffffffffffffffff,0x480(%rsp)
   b67fd:	movq   $0xffffffffffffffff,0x4a0(%rsp)
   b6809:	movq   $0xffffffffffffffff,0x4c0(%rsp)
   b6815:	movq   $0xffffffffffffffff,0x4e0(%rsp)
   b6821:	mov    $0x2,%eax
   b6826:	xor    %r10d,%r10d
   b6829:	xor    %edi,%edi
   b682b:	mov    0x38(%rsp),%r9
   b6830:	mov    0x70(%r13),%rsi
   b6834:	cmp    %rsi,%rdi
   b6837:	jae    b7aea <<mibz2::c30::C30<false>>::room+0x1b3a>
   b683d:	mov    0xc8(%r13),%rsi
   b6844:	cmp    %rsi,%rdi
   b6847:	jae    b7ade <<mibz2::c30::C30<false>>::room+0x1b2e>
   b684d:	mov    0x68(%r13),%rcx
   b6851:	mov    0xc0(%r13),%rdx
   b6858:	mov    (%rcx,%rax,4),%rsi
   b685c:	movzbl (%rdx,%rdi,4),%r8d
   b6861:	mov    %r8b,(%rsi)
   b6864:	mov    (%rdx,%rdi,4),%edx
   b6867:	shr    $0x8,%edx
   b686a:	lea    0x1(%rsi),%r8
   b686e:	add    -0x8(%rcx,%rax,4),%rsi
   b6873:	mov    %rdi,0x100(%rsp,%r10,1)
   b687b:	mov    %r8,0x108(%rsp,%r10,1)
   b6883:	mov    %rsi,0x110(%rsp,%r10,1)
   b688b:	mov    %edx,0x118(%rsp,%r10,1)
   b6893:	movl   $0x0,0x11c(%rsp,%r10,1)
   b689f:	prefetcht0 (%r14,%rdx,2)
   b68a4:	lea    0x1(%rdi),%rcx
   b68a8:	add    $0x20,%r10
   b68ac:	cmp    $0x1e,%rdi
   b68b0:	ja     b68c2 <<mibz2::c30::C30<false>>::room+0x912>
   b68b2:	add    $0x6,%rax
   b68b6:	mov    %rcx,%rdi
   b68b9:	cmp    %r9,%rcx
   b68bc:	jb     b6830 <<mibz2::c30::C30<false>>::room+0x880>
   b68c2:	mov    %r10,0xf8(%rsp)
   b68ca:	cmp    $0x20,%rcx
   b68ce:	jne    b743d <<mibz2::c30::C30<false>>::room+0x148d>
   b68d4:	mov    %rcx,0xe0(%rsp)
   b68dc:	xorps  %xmm0,%xmm0
   b68df:	movaps %xmm0,0x570(%rsp)
   b68e7:	movaps %xmm0,0x560(%rsp)
   b68ef:	movaps %xmm0,0x550(%rsp)
   b68f7:	movaps %xmm0,0x540(%rsp)
   b68ff:	movaps %xmm0,0x530(%rsp)
   b6907:	movaps %xmm0,0x520(%rsp)
   b690f:	movaps %xmm0,0x510(%rsp)
   b6917:	movaps %xmm0,0x500(%rsp)
   b691f:	mov    0x118(%rsp),%eax
   b6926:	mov    0x138(%rsp),%ecx
   b692d:	mov    %eax,0x58(%rsp)
   b6931:	mov    %ecx,0x5c(%rsp)
   b6935:	mov    0x158(%rsp),%eax
   b693c:	mov    %eax,0x60(%rsp)
   b6940:	mov    0x178(%rsp),%eax
   b6947:	mov    %eax,0x64(%rsp)
   b694b:	mov    0x198(%rsp),%eax
   b6952:	mov    %eax,0x68(%rsp)
   b6956:	mov    0x1b8(%rsp),%eax
   b695d:	mov    %eax,0x6c(%rsp)
   b6961:	mov    0x1d8(%rsp),%eax
   b6968:	mov    %eax,0x70(%rsp)
   b696c:	mov    0x1f8(%rsp),%eax
   b6973:	mov    %eax,0x74(%rsp)
   b6977:	mov    0x218(%rsp),%eax
   b697e:	mov    %eax,0x78(%rsp)
   b6982:	mov    0x238(%rsp),%eax
   b6989:	mov    %eax,0x7c(%rsp)
   b698d:	mov    0x258(%rsp),%eax
   b6994:	mov    %eax,0x80(%rsp)
   b699b:	mov    0x278(%rsp),%eax
   b69a2:	mov    %eax,0x84(%rsp)
   b69a9:	mov    0x298(%rsp),%eax
   b69b0:	mov    %eax,0x88(%rsp)
   b69b7:	mov    0x2b8(%rsp),%eax
   b69be:	mov    %eax,0x8c(%rsp)
   b69c5:	mov    0x2d8(%rsp),%eax
   b69cc:	mov    %eax,0x90(%rsp)
   b69d3:	mov    0x2f8(%rsp),%eax
   b69da:	mov    %eax,0x94(%rsp)
   b69e1:	mov    0x318(%rsp),%eax
   b69e8:	mov    %eax,0x98(%rsp)
   b69ef:	mov    0x338(%rsp),%eax
   b69f6:	mov    %eax,0x9c(%rsp)
   b69fd:	mov    0x358(%rsp),%eax
   b6a04:	mov    %eax,0xa0(%rsp)
   b6a0b:	mov    0x378(%rsp),%eax
   b6a12:	mov    %eax,0xa4(%rsp)
   b6a19:	mov    0x398(%rsp),%eax
   b6a20:	mov    %eax,0xa8(%rsp)
   b6a27:	mov    0x3b8(%rsp),%eax
   b6a2e:	mov    %eax,0xac(%rsp)
   b6a35:	mov    0x3d8(%rsp),%eax
   b6a3c:	mov    %eax,0xb0(%rsp)
   b6a43:	mov    0x3f8(%rsp),%eax
   b6a4a:	mov    %eax,0xb4(%rsp)
   b6a51:	mov    0x418(%rsp),%eax
   b6a58:	mov    %eax,0xb8(%rsp)
   b6a5f:	mov    0x438(%rsp),%eax
   b6a66:	mov    %eax,0xbc(%rsp)
   b6a6d:	mov    0x458(%rsp),%eax
   b6a74:	mov    %eax,0xc0(%rsp)
   b6a7b:	mov    0x478(%rsp),%eax
   b6a82:	mov    %eax,0xc4(%rsp)
   b6a89:	mov    0x498(%rsp),%eax
   b6a90:	mov    %eax,0xc8(%rsp)
   b6a97:	mov    0x4b8(%rsp),%eax
   b6a9e:	mov    %eax,0xcc(%rsp)
   b6aa5:	mov    0x4d8(%rsp),%eax
   b6aac:	mov    %eax,0xd0(%rsp)
   b6ab3:	mov    0x4f8(%rsp),%eax
   b6aba:	mov    %eax,0xd4(%rsp)
   b6ac1:	lea    0x680(%rsp),%rdi
   b6ac9:	mov    $0x2000,%edx
   b6ace:	xor    %esi,%esi
   b6ad0:	call   *0x79c22(%rip)        # 1306f8 <memset@GLIBC_2.2.5>
   b6ad6:	addq   $0x3f,0x48(%rsp)
   b6adc:	mov    $0x20,%r11d
   b6ae2:	mov    0x79ba7(%rip),%r10        # 130690 <memcpy@GLIBC_2.14>
   b6ae9:	mov    %rbp,%r8
   b6aec:	jmp    b6af9 <<mibz2::c30::C30<false>>::room+0xb49>
   b6aee:	xchg   %ax,%ax
   b6af0:	test   %r15,%r15
   b6af3:	jne    b6bd0 <<mibz2::c30::C30<false>>::room+0xc20>
   b6af9:	mov    0x48(%rsp),%r15
   b6afe:	xor    %ebx,%ebx
   b6b00:	jmp    b6b3d <<mibz2::c30::C30<false>>::room+0xb8d>
   b6b02:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   b6b10:	add    %rax,%rbp
   b6b13:	mov    %rbp,0x108(%rsp,%rbx,1)
   b6b1b:	add    %rax,%rdx
   b6b1e:	mov    %rdx,0x110(%rsp,%rbx,1)
   b6b26:	sub    %rbp,%rdx
   b6b29:	add    $0x20,%rbx
   b6b2d:	cmp    %r15,%rdx
   b6b30:	cmovb  %rdx,%r15
   b6b34:	cmp    $0x400,%rbx
   b6b3b:	je     b6af0 <<mibz2::c30::C30<false>>::room+0xb40>
   b6b3d:	mov    0x108(%rsp,%rbx,1),%rbp
   b6b45:	mov    0x110(%rsp,%rbx,1),%rdx
   b6b4d:	cmp    %rdx,%rbp
   b6b50:	jne    b6b26 <<mibz2::c30::C30<false>>::room+0xb76>
   b6b52:	mov    0x70(%r13),%rsi
   b6b56:	mov    0x100(%rsp,%rbx,1),%rdi
   b6b5e:	cmp    %rsi,%rdi
   b6b61:	jae    b7a74 <<mibz2::c30::C30<false>>::room+0x1ac4>
   b6b67:	mov    0x68(%r13),%rcx
   b6b6b:	lea    (%rdi,%rdi,2),%rsi
   b6b6f:	mov    (%rcx,%rsi,8),%rdx
   b6b73:	mov    0x8(%rcx,%rsi,8),%rax
   b6b78:	sub    %rax,%rbp
   b6b7b:	mov    %rbp,0x10(%rcx,%rsi,8)
   b6b80:	mov    %rdx,%rdi
   b6b83:	sub    %rbp,%rdi
   b6b86:	cmp    %rdi,%rdx
   b6b89:	jbe    b6b10 <<mibz2::c30::C30<false>>::room+0xb60>
   b6b8b:	lea    (%rcx,%rsi,8),%r12
   b6b8f:	mov    $0x1,%ecx
   b6b94:	mov    $0x1,%r8d
   b6b9a:	mov    %r12,%rdi
   b6b9d:	mov    %rbp,%rsi
   b6ba0:	mov    %r11,0x10(%rsp)
   b6ba5:	call   98d90 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   b6baa:	mov    0x10(%rsp),%r11
   b6baf:	mov    0x79ada(%rip),%r10        # 130690 <memcpy@GLIBC_2.14>
   b6bb6:	mov    (%r12),%rdx
   b6bba:	mov    0x8(%r12),%rax
   b6bbf:	mov    0x28(%rsp),%r8
   b6bc4:	jmp    b6b10 <<mibz2::c30::C30<false>>::room+0xb60>
   b6bc9:	nopl   0x0(%rax)
   b6bd0:	xor    %ecx,%ecx
   b6bd2:	mov    %r8,%rbp
   b6bd5:	mov    %r15,%rdx
   b6bd8:	sub    %rcx,%rdx
   b6bdb:	cmp    $0x100,%rdx
   b6be2:	mov    $0x100,%eax
   b6be7:	cmovae %rax,%rdx
   b6beb:	xorps  %xmm0,%xmm0
   b6bee:	movaps %xmm0,0x670(%rsp)
   b6bf6:	movaps %xmm0,0x660(%rsp)
   b6bfe:	movaps %xmm0,0x650(%rsp)
   b6c06:	movaps %xmm0,0x640(%rsp)
   b6c0e:	movaps %xmm0,0x630(%rsp)
   b6c16:	movaps %xmm0,0x620(%rsp)
   b6c1e:	movaps %xmm0,0x610(%rsp)
   b6c26:	movaps %xmm0,0x600(%rsp)
   b6c2e:	movaps %xmm0,0x5f0(%rsp)
   b6c36:	movaps %xmm0,0x5e0(%rsp)
   b6c3e:	movaps %xmm0,0x5d0(%rsp)
   b6c46:	movaps %xmm0,0x5c0(%rsp)
   b6c4e:	movaps %xmm0,0x5b0(%rsp)
   b6c56:	movaps %xmm0,0x5a0(%rsp)
   b6c5e:	movaps %xmm0,0x590(%rsp)
   b6c66:	movaps %xmm0,0x580(%rsp)
   b6c6e:	cmp    %rcx,%r15
   b6c71:	mov    %rcx,0xe8(%rsp)
   b6c79:	mov    %rdx,0x18(%rsp)
   b6c7e:	jne    b6d50 <<mibz2::c30::C30<false>>::room+0xda0>
   b6c84:	mov    %r11,0x10(%rsp)
   b6c89:	lea    0x588(%rsp),%rbx
   b6c91:	lea    0x680(%rsp),%rbp
   b6c99:	xor    %r13d,%r13d
   b6c9c:	nopl   0x0(%rax)
   b6ca0:	mov    -0x8(%rbx),%rsi
   b6ca4:	mov    0x18(%rsp),%r12
   b6ca9:	sub    %rsi,%r12
   b6cac:	add    %rbp,%rsi
   b6caf:	mov    0x108(%rsp,%r13,1),%rdi
   b6cb7:	mov    %r12,%rdx
   b6cba:	call   *0x799d0(%rip)        # 130690 <memcpy@GLIBC_2.14>
   b6cc0:	add    %r12,0x108(%rsp,%r13,1)
   b6cc8:	lea    0x40(%r13),%rax
   b6ccc:	mov    %rax,0x8(%rsp)
   b6cd1:	mov    (%rbx),%rax
   b6cd4:	mov    0x18(%rsp),%r12
   b6cd9:	sub    %rax,%r12
   b6cdc:	lea    (%rax,%rbp,1),%rsi
   b6ce0:	add    $0x100,%rsi
   b6ce7:	mov    0x128(%rsp,%r13,1),%rdi
   b6cef:	mov    %r12,%rdx
   b6cf2:	call   *0x79998(%rip)        # 130690 <memcpy@GLIBC_2.14>
   b6cf8:	mov    0x79991(%rip),%r10        # 130690 <memcpy@GLIBC_2.14>
   b6cff:	add    %r12,0x128(%rsp,%r13,1)
   b6d07:	add    $0x200,%rbp
   b6d0e:	add    $0x10,%rbx
   b6d12:	mov    0x8(%rsp),%rax
   b6d17:	mov    %rax,%r13
   b6d1a:	cmp    $0x400,%rax
   b6d20:	jne    b6ca0 <<mibz2::c30::C30<false>>::room+0xcf0>
   b6d26:	mov    0xe8(%rsp),%rcx
   b6d2e:	add    0x18(%rsp),%rcx
   b6d33:	cmp    %r15,%rcx
   b6d36:	mov    0x20(%rsp),%r13
   b6d3b:	mov    0x28(%rsp),%rbp
   b6d40:	mov    0x10(%rsp),%r11
   b6d45:	jb     b6bd5 <<mibz2::c30::C30<false>>::room+0xc25>
   b6d4b:	jmp    b6ae9 <<mibz2::c30::C30<false>>::room+0xb39>
   b6d50:	lea    0xd80(%rsp),%rdx
   b6d58:	xor    %r12d,%r12d
   b6d5b:	movabs $0x1ffffffc0,%r9
   b6d65:	jmp    b6d8b <<mibz2::c30::C30<false>>::room+0xddb>
   b6d67:	nopw   0x0(%rax,%rax,1)
   b6d70:	mov    0xf0(%rsp),%rdx
   b6d78:	inc    %rdx
   b6d7b:	cmp    0x18(%rsp),%r12
   b6d80:	mov    0x28(%rsp),%rbp
   b6d85:	je     b6c84 <<mibz2::c30::C30<false>>::room+0xcd4>
   b6d8b:	mov    %r12,0x40(%rsp)
   b6d90:	lea    (%rsp,%r12,1),%rax
   b6d94:	add    $0x680,%rax
   b6d9a:	mov    %rdx,0xf0(%rsp)
   b6da2:	xor    %ecx,%ecx
   b6da4:	xor    %ebx,%ebx
   b6da6:	cs nopw 0x0(%rax,%rax,1)
   b6db0:	mov    0x58(%rsp,%rcx,4),%esi
   b6db4:	lea    (%rsi,%rsi,1),%rdi
   b6db8:	movzwl (%r14,%rsi,2),%r8d
   b6dbd:	and    %r9,%rdi
   b6dc0:	mov    0x3c(%r14,%rdi,1),%edi
   b6dc5:	cmp    $0xfffe,%r8d
   b6dcc:	jae    b6f86 <<mibz2::c30::C30<false>>::room+0xfd6>
   b6dd2:	mov    %dil,-0x700(%rdx)
   b6dd9:	shr    $0x8,%edi
   b6ddc:	add    %r8d,%edi
   b6ddf:	mov    %edi,0x58(%rsp,%rcx,4)
   b6de3:	prefetcht0 (%r14,%rdi,2)
   b6de8:	mov    0x5c(%rsp,%rcx,4),%esi
   b6dec:	lea    (%rsi,%rsi,1),%rdi
   b6df0:	movzwl (%r14,%rsi,2),%r8d
   b6df5:	and    %r9,%rdi
   b6df8:	mov    0x3c(%r14,%rdi,1),%edi
   b6dfd:	cmp    $0xfffe,%r8d
   b6e04:	jae    b6f91 <<mibz2::c30::C30<false>>::room+0xfe1>
   b6e0a:	mov    %dil,-0x600(%rdx)
   b6e11:	shr    $0x8,%edi
   b6e14:	add    %r8d,%edi
   b6e17:	mov    %edi,0x5c(%rsp,%rcx,4)
   b6e1b:	prefetcht0 (%r14,%rdi,2)
   b6e20:	mov    0x60(%rsp,%rcx,4),%esi
   b6e24:	lea    (%rsi,%rsi,1),%rdi
   b6e28:	movzwl (%r14,%rsi,2),%r8d
   b6e2d:	and    %r9,%rdi
   b6e30:	mov    0x3c(%r14,%rdi,1),%edi
   b6e35:	cmp    $0xfffe,%r8d
   b6e3c:	jae    b6fa1 <<mibz2::c30::C30<false>>::room+0xff1>
   b6e42:	mov    %dil,-0x500(%rdx)
   b6e49:	shr    $0x8,%edi
   b6e4c:	add    %r8d,%edi
   b6e4f:	mov    %edi,0x60(%rsp,%rcx,4)
   b6e53:	prefetcht0 (%r14,%rdi,2)
   b6e58:	mov    0x64(%rsp,%rcx,4),%esi
   b6e5c:	lea    (%rsi,%rsi,1),%rdi
   b6e60:	movzwl (%r14,%rsi,2),%r8d
   b6e65:	and    %r9,%rdi
   b6e68:	mov    0x3c(%r14,%rdi,1),%edi
   b6e6d:	cmp    $0xfffe,%r8d
   b6e74:	jae    b6fb1 <<mibz2::c30::C30<false>>::room+0x1001>
   b6e7a:	mov    %dil,-0x400(%rdx)
   b6e81:	shr    $0x8,%edi
   b6e84:	add    %r8d,%edi
   b6e87:	mov    %edi,0x64(%rsp,%rcx,4)
   b6e8b:	prefetcht0 (%r14,%rdi,2)
   b6e90:	mov    0x68(%rsp,%rcx,4),%esi
   b6e94:	lea    (%rsi,%rsi,1),%rdi
   b6e98:	movzwl (%r14,%rsi,2),%r8d
   b6e9d:	and    %r9,%rdi
   b6ea0:	mov    0x3c(%r14,%rdi,1),%edi
   b6ea5:	cmp    $0xfffe,%r8d
   b6eac:	jae    b6fc1 <<mibz2::c30::C30<false>>::room+0x1011>
   b6eb2:	mov    %dil,-0x300(%rdx)
   b6eb9:	shr    $0x8,%edi
   b6ebc:	add    %r8d,%edi
   b6ebf:	mov    %edi,0x68(%rsp,%rcx,4)
   b6ec3:	prefetcht0 (%r14,%rdi,2)
   b6ec8:	mov    0x6c(%rsp,%rcx,4),%esi
   b6ecc:	lea    (%rsi,%rsi,1),%rdi
   b6ed0:	movzwl (%r14,%rsi,2),%r8d
   b6ed5:	and    %r9,%rdi
   b6ed8:	mov    0x3c(%r14,%rdi,1),%edi
   b6edd:	cmp    $0xfffe,%r8d
   b6ee4:	jae    b6fd1 <<mibz2::c30::C30<false>>::room+0x1021>
   b6eea:	mov    %dil,-0x200(%rdx)
   b6ef1:	shr    $0x8,%edi
   b6ef4:	add    %r8d,%edi
   b6ef7:	mov    %edi,0x6c(%rsp,%rcx,4)
   b6efb:	prefetcht0 (%r14,%rdi,2)
   b6f00:	mov    0x70(%rsp,%rcx,4),%esi
   b6f04:	lea    (%rsi,%rsi,1),%rdi
   b6f08:	movzwl (%r14,%rsi,2),%r8d
   b6f0d:	and    %r9,%rdi
   b6f10:	mov    0x3c(%r14,%rdi,1),%edi
   b6f15:	cmp    $0xfffe,%r8d
   b6f1c:	jae    b6fe1 <<mibz2::c30::C30<false>>::room+0x1031>
   b6f22:	mov    %dil,-0x100(%rdx)
   b6f29:	shr    $0x8,%edi
   b6f2c:	add    %r8d,%edi
   b6f2f:	mov    %edi,0x70(%rsp,%rcx,4)
   b6f33:	prefetcht0 (%r14,%rdi,2)
   b6f38:	mov    0x74(%rsp,%rcx,4),%esi
   b6f3c:	lea    (%rsi,%rsi,1),%rdi
   b6f40:	movzwl (%r14,%rsi,2),%r8d
   b6f45:	and    %r9,%rdi
   b6f48:	mov    0x3c(%r14,%rdi,1),%edi
   b6f4d:	cmp    $0xfffe,%r8d
   b6f54:	jae    b6ff1 <<mibz2::c30::C30<false>>::room+0x1041>
   b6f5a:	mov    %dil,(%rdx)
   b6f5d:	shr    $0x8,%edi
   b6f60:	add    %r8d,%edi
   b6f63:	mov    %edi,0x74(%rsp,%rcx,4)
   b6f67:	prefetcht0 (%r14,%rdi,2)
   b6f6c:	add    $0x8,%rcx
   b6f70:	add    $0x800,%rdx
   b6f77:	cmp    $0x19,%rcx
   b6f7b:	jb     b6db0 <<mibz2::c30::C30<false>>::room+0xe00>
   b6f81:	jmp    b7180 <<mibz2::c30::C30<false>>::room+0x11d0>
   b6f86:	jne    b7001 <<mibz2::c30::C30<false>>::room+0x1051>
   b6f88:	bts    %rcx,%rbx
   b6f8c:	jmp    b6de8 <<mibz2::c30::C30<false>>::room+0xe38>
   b6f91:	jne    b7030 <<mibz2::c30::C30<false>>::room+0x1080>
   b6f97:	mov    $0x2,%esi
   b6f9c:	jmp    b704e <<mibz2::c30::C30<false>>::room+0x109e>
   b6fa1:	jne    b7059 <<mibz2::c30::C30<false>>::room+0x10a9>
   b6fa7:	mov    $0x4,%esi
   b6fac:	jmp    b707d <<mibz2::c30::C30<false>>::room+0x10cd>
   b6fb1:	jne    b7088 <<mibz2::c30::C30<false>>::room+0x10d8>
   b6fb7:	mov    $0x8,%esi
   b6fbc:	jmp    b70ac <<mibz2::c30::C30<false>>::room+0x10fc>
   b6fc1:	jne    b70b7 <<mibz2::c30::C30<false>>::room+0x1107>
   b6fc7:	mov    $0x10,%esi
   b6fcc:	jmp    b70db <<mibz2::c30::C30<false>>::room+0x112b>
   b6fd1:	jne    b70e6 <<mibz2::c30::C30<false>>::room+0x1136>
   b6fd7:	mov    $0x20,%esi
   b6fdc:	jmp    b710a <<mibz2::c30::C30<false>>::room+0x115a>
   b6fe1:	jne    b7115 <<mibz2::c30::C30<false>>::room+0x1165>
   b6fe7:	mov    $0x40,%esi
   b6fec:	jmp    b7139 <<mibz2::c30::C30<false>>::room+0x1189>
   b6ff1:	jne    b7144 <<mibz2::c30::C30<false>>::room+0x1194>
   b6ff7:	mov    $0x80,%esi
   b6ffc:	jmp    b7168 <<mibz2::c30::C30<false>>::room+0x11b8>
   b7001:	shl    $0x5,%edi
   b7004:	and    $0x1ffffe0,%edi
   b700a:	and    $0x1f,%esi
   b700d:	or     %edi,%esi
   b700f:	mov    %esi,0x500(%rsp,%rcx,4)
   b7016:	prefetcht0 0x0(%rbp,%rsi,4)
   b701b:	movabs $0x100000000,%rsi
   b7025:	shl    %cl,%rsi
   b7028:	or     %rsi,%rbx
   b702b:	jmp    b6de8 <<mibz2::c30::C30<false>>::room+0xe38>
   b7030:	shl    $0x5,%edi
   b7033:	and    $0x1ffffe0,%edi
   b7039:	and    $0x1f,%esi
   b703c:	or     %edi,%esi
   b703e:	mov    %esi,0x504(%rsp,%rcx,4)
   b7045:	prefetcht0 0x0(%rbp,%rsi,4)
   b704a:	lea    0x40(%r9),%rsi
   b704e:	shl    %cl,%rsi
   b7051:	or     %rsi,%rbx
   b7054:	jmp    b6e20 <<mibz2::c30::C30<false>>::room+0xe70>
   b7059:	shl    $0x5,%edi
   b705c:	and    $0x1ffffe0,%edi
   b7062:	and    $0x1f,%esi
   b7065:	or     %edi,%esi
   b7067:	mov    %esi,0x508(%rsp,%rcx,4)
   b706e:	prefetcht0 0x0(%rbp,%rsi,4)
   b7073:	movabs $0x400000000,%rsi
   b707d:	shl    %cl,%rsi
   b7080:	or     %rsi,%rbx
   b7083:	jmp    b6e58 <<mibz2::c30::C30<false>>::room+0xea8>
   b7088:	shl    $0x5,%edi
   b708b:	and    $0x1ffffe0,%edi
   b7091:	and    $0x1f,%esi
   b7094:	or     %edi,%esi
   b7096:	mov    %esi,0x50c(%rsp,%rcx,4)
   b709d:	prefetcht0 0x0(%rbp,%rsi,4)
   b70a2:	movabs $0x800000000,%rsi
   b70ac:	shl    %cl,%rsi
   b70af:	or     %rsi,%rbx
   b70b2:	jmp    b6e90 <<mibz2::c30::C30<false>>::room+0xee0>
   b70b7:	shl    $0x5,%edi
   b70ba:	and    $0x1ffffe0,%edi
   b70c0:	and    $0x1f,%esi
   b70c3:	or     %edi,%esi
   b70c5:	mov    %esi,0x510(%rsp,%rcx,4)
   b70cc:	prefetcht0 0x0(%rbp,%rsi,4)
   b70d1:	movabs $0x1000000000,%rsi
   b70db:	shl    %cl,%rsi
   b70de:	or     %rsi,%rbx
   b70e1:	jmp    b6ec8 <<mibz2::c30::C30<false>>::room+0xf18>
   b70e6:	shl    $0x5,%edi
   b70e9:	and    $0x1ffffe0,%edi
   b70ef:	and    $0x1f,%esi
   b70f2:	or     %edi,%esi
   b70f4:	mov    %esi,0x514(%rsp,%rcx,4)
   b70fb:	prefetcht0 0x0(%rbp,%rsi,4)
   b7100:	movabs $0x2000000000,%rsi
   b710a:	shl    %cl,%rsi
   b710d:	or     %rsi,%rbx
   b7110:	jmp    b6f00 <<mibz2::c30::C30<false>>::room+0xf50>
   b7115:	shl    $0x5,%edi
   b7118:	and    $0x1ffffe0,%edi
   b711e:	and    $0x1f,%esi
   b7121:	or     %edi,%esi
   b7123:	mov    %esi,0x518(%rsp,%rcx,4)
   b712a:	prefetcht0 0x0(%rbp,%rsi,4)
   b712f:	movabs $0x4000000000,%rsi
   b7139:	shl    %cl,%rsi
   b713c:	or     %rsi,%rbx
   b713f:	jmp    b6f38 <<mibz2::c30::C30<false>>::room+0xf88>
   b7144:	shl    $0x5,%edi
   b7147:	and    $0x1ffffe0,%edi
   b714d:	and    $0x1f,%esi
   b7150:	or     %edi,%esi
   b7152:	mov    %esi,0x51c(%rsp,%rcx,4)
   b7159:	prefetcht0 0x0(%rbp,%rsi,4)
   b715e:	movabs $0x8000000000,%rsi
   b7168:	shl    %cl,%rsi
   b716b:	or     %rsi,%rbx
   b716e:	jmp    b6f6c <<mibz2::c30::C30<false>>::room+0xfbc>
   b7173:	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   b7180:	mov    0x40(%rsp),%rcx
   b7185:	lea    0x1(%rcx),%r12
   b7189:	test   %rbx,%rbx
   b718c:	je     b6d70 <<mibz2::c30::C30<false>>::room+0xdc0>
   b7192:	mov    %rbx,%rcx
   b7195:	shr    $0x20,%rcx
   b7199:	je     b71d7 <<mibz2::c30::C30<false>>::room+0x1227>
   b719b:	nopl   0x0(%rax,%rax,1)
   b71a0:	tzcnt  %ecx,%edx
   b71a4:	lea    -0x1(%rcx),%esi
   b71a7:	lea    0x0(,%rdx,4),%edi
   b71ae:	mov    0x500(%rsp,%rdi,1),%r8d
   b71b6:	mov    0x0(%rbp,%r8,4),%r8d
   b71bb:	shl    $0x8,%edx
   b71be:	mov    %r8b,(%rax,%rdx,1)
   b71c2:	mov    %r8d,%edx
   b71c5:	shr    $0x8,%edx
   b71c8:	mov    %edx,0x58(%rsp,%rdi,1)
   b71cc:	prefetcht0 (%r14,%rdx,2)
   b71d1:	and    %ecx,%esi
   b71d3:	mov    %esi,%ecx
   b71d5:	jne    b71a0 <<mibz2::c30::C30<false>>::room+0x11f0>
   b71d7:	mov    $0xffffffff,%eax
   b71dc:	and    %rax,%rbx
   b71df:	je     b6d70 <<mibz2::c30::C30<false>>::room+0xdc0>
   b71e5:	data16 cs nopw 0x0(%rax,%rax,1)
   b71f0:	mov    %r11,0x10(%rsp)
   b71f5:	mov    %rbx,0x8(%rsp)
   b71fa:	tzcnt  %rbx,%rcx
   b71ff:	mov    %ecx,%ebx
   b7201:	shl    $0x5,%ebx
   b7204:	mov    0x58(%rsp,%rcx,4),%eax
   b7208:	mov    %eax,0x118(%rsp,%rbx,1)
   b720f:	mov    %ecx,%eax
   b7211:	shl    $0x8,%eax
   b7214:	lea    (%rsp,%rax,1),%rsi
   b7218:	add    $0x680,%rsi
   b721f:	mov    %rcx,0x30(%rsp)
   b7224:	mov    0x580(%rsp,%rcx,8),%rax
   b722c:	mov    0x40(%rsp),%rbp
   b7231:	sub    %rax,%rbp
   b7234:	add    %rax,%rsi
   b7237:	mov    0x108(%rsp,%rbx,1),%rdi
   b723f:	mov    %rbp,%rdx
   b7242:	call   *%r10
   b7245:	add    0x108(%rsp,%rbx,1),%rbp
   b724d:	mov    %rbp,0x108(%rsp,%rbx,1)
   b7255:	mov    0x70(%r13),%rsi
   b7259:	mov    0x100(%rsp,%rbx,1),%rdi
   b7261:	cmp    %rsi,%rdi
   b7264:	jae    b7b38 <<mibz2::c30::C30<false>>::room+0x1b88>
   b726a:	lea    (%rsp,%rbx,1),%rax
   b726e:	add    $0x100,%rax
   b7274:	mov    0x68(%r13),%rdx
   b7278:	mov    0x80(%r13),%rcx
   b727f:	lea    (%rdi,%rdi,2),%r8
   b7283:	sub    0x8(%rdx,%r8,8),%rbp
   b7288:	mov    0x88(%r13),%rsi
   b728f:	mov    %rbp,0x10(%rdx,%r8,8)
   b7294:	mov    0x40(%r13),%r10
   b7298:	test   %r10,%r10
   b729b:	je     b72f0 <<mibz2::c30::C30<false>>::room+0x1340>
   b729d:	mov    0x18(%rax),%r8d
   b72a1:	mov    %r13,%rbp
   b72a4:	mov    0x38(%r13),%r9
   b72a8:	xor    %edx,%edx
   b72aa:	cmp    $0x1,%r10
   b72ae:	je     b72cd <<mibz2::c30::C30<false>>::room+0x131d>
   b72b0:	mov    %rdx,%r11
   b72b3:	mov    %r10,%rbx
   b72b6:	shr    $1,%rbx
   b72b9:	add    %rbx,%rdx
   b72bc:	cmp    %r8d,(%r9,%rdx,4)
   b72c0:	cmova  %r11,%rdx
   b72c4:	sub    %rbx,%r10
   b72c7:	cmp    $0x1,%r10
   b72cb:	ja     b72b0 <<mibz2::c30::C30<false>>::room+0x1300>
   b72cd:	cmp    %r8d,(%r9,%rdx,4)
   b72d1:	mov    $0xffffffffffffffff,%r8
   b72d8:	cmovne %r8,%rdx
   b72dc:	mov    %rbp,%r13
   b72df:	jmp    b72f7 <<mibz2::c30::C30<false>>::room+0x1347>
   b72e1:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   b72f0:	mov    $0xffffffffffffffff,%rdx
   b72f7:	cmp    %rsi,%rdi
   b72fa:	mov    0x38(%rsp),%r8
   b72ff:	movabs $0x1ffffffc0,%r9
   b7309:	mov    0x79380(%rip),%r10        # 130690 <memcpy@GLIBC_2.14>
   b7310:	mov    0x10(%rsp),%r11
   b7315:	mov    0x8(%rsp),%rbx
   b731a:	jae    b7b2c <<mibz2::c30::C30<false>>::room+0x1b7c>
   b7320:	lea    -0x1(%rbx),%rsi
   b7324:	and    %rsi,%rbx
   b7327:	mov    %rdx,(%rcx,%rdi,8)
   b732b:	cmp    %r8,%r11
   b732e:	je     b73b8 <<mibz2::c30::C30<false>>::room+0x1408>
   b7334:	mov    0x70(%r13),%rsi
   b7338:	cmp    %rsi,%r11
   b733b:	jae    b7b44 <<mibz2::c30::C30<false>>::room+0x1b94>
   b7341:	mov    0xc8(%r13),%rsi
   b7348:	cmp    %rsi,%r11
   b734b:	jae    b7b53 <<mibz2::c30::C30<false>>::room+0x1ba3>
   b7351:	mov    0x68(%r13),%rcx
   b7355:	mov    0xc0(%r13),%rdx
   b735c:	lea    (%r11,%r11,2),%rsi
   b7360:	mov    0x8(%rcx,%rsi,8),%rdi
   b7365:	movzbl (%rdx,%r11,4),%r8d
   b736a:	mov    %r8b,(%rdi)
   b736d:	mov    (%rdx,%r11,4),%edx
   b7371:	shr    $0x8,%edx
   b7374:	lea    0x1(%rdi),%r8
   b7378:	add    (%rcx,%rsi,8),%rdi
   b737c:	mov    %r11,(%rax)
   b737f:	mov    %r8,0x8(%rax)
   b7383:	mov    %rdi,0x10(%rax)
   b7387:	mov    %edx,0x18(%rax)
   b738a:	movl   $0x0,0x1c(%rax)
   b7391:	mov    0x30(%rsp),%rax
   b7396:	mov    %edx,0x58(%rsp,%rax,4)
   b739a:	prefetcht0 (%r14,%rdx,2)
   b739f:	inc    %r11
   b73a2:	mov    %r12,0x580(%rsp,%rax,8)
   b73aa:	test   %rbx,%rbx
   b73ad:	jne    b71f0 <<mibz2::c30::C30<false>>::room+0x1240>
   b73b3:	jmp    b6d70 <<mibz2::c30::C30<false>>::room+0xdc0>
   b73b8:	not    %rbx
   b73bb:	mov    %rbx,0x8(%rsp)
   b73c0:	xor    %ebx,%ebx
   b73c2:	xor    %ebp,%ebp
   b73c4:	mov    0x30(%rsp),%r12
   b73c9:	jmp    b73e0 <<mibz2::c30::C30<false>>::room+0x1430>
   b73cb:	nopl   0x0(%rax,%rax,1)
   b73d0:	add    $0x20,%rbp
   b73d4:	inc    %rbx
   b73d7:	cmp    $0x400,%rbp
   b73de:	je     b7435 <<mibz2::c30::C30<false>>::room+0x1485>
   b73e0:	mov    0x58(%rsp,%rbx,4),%eax
   b73e4:	mov    %eax,0x118(%rsp,%rbp,1)
   b73eb:	cmp    %rbx,%r12
   b73ee:	je     b73d0 <<mibz2::c30::C30<false>>::room+0x1420>
   b73f0:	mov    0x580(%rsp,%rbx,8),%rax
   b73f8:	mov    0x8(%rsp),%rcx
   b73fd:	bt     %rbx,%rcx
   b7401:	mov    0x40(%rsp),%r15
   b7406:	adc    $0x0,%r15
   b740a:	sub    %rax,%r15
   b740d:	add    %rsp,%rax
   b7410:	add    $0x680,%rax
   b7416:	lea    (%rax,%rbp,8),%rsi
   b741a:	mov    0x108(%rsp,%rbp,1),%rdi
   b7422:	mov    %r15,%rdx
   b7425:	call   *0x79265(%rip)        # 130690 <memcpy@GLIBC_2.14>
   b742b:	add    %r15,0x108(%rsp,%rbp,1)
   b7433:	jmp    b73d0 <<mibz2::c30::C30<false>>::room+0x1420>
   b7435:	mov    0xe0(%rsp),%rcx
   b743d:	shl    $0x5,%rcx
   b7441:	lea    (%rsp,%rcx,1),%rbx
   b7445:	add    $0x100,%rbx
   b744c:	mov    0x80(%r13),%rax
   b7453:	mov    0x88(%r13),%rsi
   b745a:	lea    0x100(%rsp),%r12
   b7462:	jmp    b7482 <<mibz2::c30::C30<false>>::room+0x14d2>
   b7464:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   b7470:	mov    %rbp,%r12
   b7473:	addq   $0xffffffffffffffe0,0xf8(%rsp)
   b747c:	je     b75ce <<mibz2::c30::C30<false>>::room+0x161e>
   b7482:	lea    0x20(%r12),%rbp
   b7487:	mov    (%r12),%r15
   b748b:	cmp    $0xffffffffffffffff,%r15
   b748f:	je     b7470 <<mibz2::c30::C30<false>>::room+0x14c0>
   b7491:	cmp    %rsi,%r15
   b7494:	jae    b7b88 <<mibz2::c30::C30<false>>::room+0x1bd8>
   b749a:	cmpq   $0xffffffffffffffff,(%rax,%r15,8)
   b749f:	jne    b7470 <<mibz2::c30::C30<false>>::room+0x14c0>
   b74a1:	mov    $0x80,%edi
   b74a6:	call   *0x791fc(%rip)        # 1306a8 <malloc@GLIBC_2.2.5>
   b74ac:	test   %rax,%rax
   b74af:	je     b7bec <<mibz2::c30::C30<false>>::room+0x1c3c>
   b74b5:	mov    0x18(%r12),%rcx
   b74ba:	mov    %rcx,0x18(%rax)
   b74be:	movups 0x8(%r12),%xmm0
   b74c4:	movups %xmm0,0x8(%rax)
   b74c8:	mov    %r15,(%rax)
   b74cb:	movq   $0x4,0x580(%rsp)
   b74d7:	mov    %rax,0x588(%rsp)
   b74df:	movq   $0x1,0x590(%rsp)
   b74eb:	mov    $0x1,%r12d
   b74f1:	cmp    %rbx,%rbp
   b74f4:	je     b75e5 <<mibz2::c30::C30<false>>::room+0x1635>
   b74fa:	lea    0x580(%rsp),%r15
   b7502:	mov    0x80(%r13),%rcx
   b7509:	mov    0x88(%r13),%rsi
   b7510:	mov    %rbp,%rdx
   b7513:	jmp    b752c <<mibz2::c30::C30<false>>::room+0x157c>
   b7515:	data16 cs nopw 0x0(%rax,%rax,1)
   b7520:	mov    %rbp,%rdx
   b7523:	cmp    %rbx,%rbp
   b7526:	je     b75e0 <<mibz2::c30::C30<false>>::room+0x1630>
   b752c:	lea    0x20(%rdx),%rbp
   b7530:	mov    (%rdx),%r13
   b7533:	cmp    $0xffffffffffffffff,%r13
   b7537:	je     b7520 <<mibz2::c30::C30<false>>::room+0x1570>
   b7539:	cmp    %rsi,%r13
   b753c:	jae    b7b97 <<mibz2::c30::C30<false>>::room+0x1be7>
   b7542:	cmpq   $0xffffffffffffffff,(%rcx,%r13,8)
   b7547:	jne    b7520 <<mibz2::c30::C30<false>>::room+0x1570>
   b7549:	mov    0x18(%rdx),%rcx
   b754d:	mov    %rcx,0x690(%rsp)
   b7555:	movups 0x8(%rdx),%xmm0
   b7559:	movaps %xmm0,0x680(%rsp)
   b7561:	cmp    0x580(%rsp),%r12
   b7569:	jne    b758e <<mibz2::c30::C30<false>>::room+0x15de>
   b756b:	mov    $0x1,%edx
   b7570:	mov    $0x8,%ecx
   b7575:	mov    $0x20,%r8d
   b757b:	mov    %r15,%rdi
   b757e:	mov    %r12,%rsi
   b7581:	call   98d90 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   b7586:	mov    0x588(%rsp),%rax
   b758e:	mov    %r12,%rcx
   b7591:	shl    $0x5,%rcx
   b7595:	mov    %r13,(%rax,%rcx,1)
   b7599:	movaps 0x680(%rsp),%xmm0
   b75a1:	movups %xmm0,0x8(%rax,%rcx,1)
   b75a6:	mov    0x690(%rsp),%rdx
   b75ae:	mov    %rdx,0x18(%rax,%rcx,1)
   b75b3:	inc    %r12
   b75b6:	mov    %r12,0x590(%rsp)
   b75be:	cmp    %rbx,%rbp
   b75c1:	mov    0x20(%rsp),%r13
   b75c6:	jne    b7502 <<mibz2::c30::C30<false>>::room+0x1552>
   b75cc:	jmp    b75e5 <<mibz2::c30::C30<false>>::room+0x1635>
   b75ce:	mov    $0x8,%eax
   b75d3:	mov    %rax,0x8(%rsp)
   b75d8:	xor    %r14d,%r14d
   b75db:	jmp    b7910 <<mibz2::c30::C30<false>>::room+0x1960>
   b75e0:	mov    0x20(%rsp),%r13
   b75e5:	mov    0x580(%rsp),%rax
   b75ed:	mov    %rax,0x10(%rsp)
   b75f2:	mov    0x588(%rsp),%rax
   b75fa:	mov    %rax,0x8(%rsp)
   b75ff:	mov    %r12,%rbx
   b7602:	shl    $0x5,%rbx
   b7606:	xor    %r15d,%r15d
   b7609:	lea    0x76318(%rip),%rax        # 12d928 <__frame_dummy_init_array_entry+0x17d8>
   b7610:	mov    %rax,0x30(%rsp)
   b7615:	jmp    b764e <<mibz2::c30::C30<false>>::room+0x169e>
   b7617:	mov    0x18(%rsp),%rax
   b761c:	mov    (%rax),%rdx
   b761f:	mov    0x8(%rax),%rax
   b7623:	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   b7630:	add    %rax,%rbp
   b7633:	mov    0x8(%rsp),%rcx
   b7638:	mov    %rbp,0x8(%rcx,%r15,1)
   b763d:	add    %rax,%rdx
   b7640:	mov    %rdx,0x10(%rcx,%r15,1)
   b7645:	add    $0x20,%r15
   b7649:	cmp    %r15,%rbx
   b764c:	je     b76ba <<mibz2::c30::C30<false>>::room+0x170a>
   b764e:	mov    0x8(%rsp),%rax
   b7653:	mov    0x8(%rax,%r15,1),%rbp
   b7658:	cmp    0x10(%rax,%r15,1),%rbp
   b765d:	jne    b7645 <<mibz2::c30::C30<false>>::room+0x1695>
   b765f:	mov    0x70(%r13),%rsi
   b7663:	mov    0x8(%rsp),%rax
   b7668:	mov    (%rax,%r15,1),%rdi
   b766c:	cmp    %rsi,%rdi
   b766f:	jae    b7b77 <<mibz2::c30::C30<false>>::room+0x1bc7>
   b7675:	mov    0x68(%r13),%rcx
   b7679:	lea    (%rdi,%rdi,2),%rsi
   b767d:	mov    (%rcx,%rsi,8),%rdx
   b7681:	mov    0x8(%rcx,%rsi,8),%rax
   b7686:	sub    %rax,%rbp
   b7689:	mov    %rbp,0x10(%rcx,%rsi,8)
   b768e:	mov    %rdx,%rdi
   b7691:	sub    %rbp,%rdi
   b7694:	cmp    %rdi,%rdx
   b7697:	jbe    b7630 <<mibz2::c30::C30<false>>::room+0x1680>
   b7699:	lea    (%rcx,%rsi,8),%rdi
   b769d:	mov    $0x1,%ecx
   b76a2:	mov    $0x1,%r8d
   b76a8:	mov    %rdi,0x18(%rsp)
   b76ad:	mov    %rbp,%rsi
   b76b0:	call   98d90 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   b76b5:	jmp    b7617 <<mibz2::c30::C30<false>>::room+0x1667>
   b76ba:	movabs $0x1ffffffc0,%r13
   b76c4:	mov    0x8(%rsp),%rax
   b76c9:	add    $0x18,%rax
   b76cd:	mov    %rax,0x18(%rsp)
   b76d2:	xor    %ebx,%ebx
   b76d4:	jmp    b76e2 <<mibz2::c30::C30<false>>::room+0x1732>
   b76d6:	cs nopw 0x0(%rax,%rax,1)
   b76e0:	xor    %ebx,%ebx
   b76e2:	mov    %rbx,%r15
   b76e5:	shl    $0x5,%r15
   b76e9:	add    0x18(%rsp),%r15
   b76ee:	jmp    b771a <<mibz2::c30::C30<false>>::room+0x176a>
   b76f0:	mov    0x40(%rsp),%rax
   b76f5:	mov    (%rax),%rdx
   b76f8:	mov    0x8(%rax),%rax
   b76fc:	nopl   0x0(%rax)
   b7700:	add    %rax,%rbp
   b7703:	mov    %rbp,-0x10(%r15)
   b7707:	add    %rax,%rdx
   b770a:	mov    %rdx,-0x8(%r15)
   b770e:	inc    %rbx
   b7711:	add    $0x20,%r15
   b7715:	cmp    %r12,%rbx
   b7718:	jae    b76e0 <<mibz2::c30::C30<false>>::room+0x1730>
   b771a:	lea    -0x18(%r15),%rax
   b771e:	mov    (%r15),%ecx
   b7721:	lea    (%rcx,%rcx,1),%rdx
   b7725:	movzwl (%r14,%rcx,2),%esi
   b772a:	and    %r13,%rdx
   b772d:	mov    0x3c(%r14,%rdx,1),%edx
   b7732:	cmp    $0xfffe,%esi
   b7738:	jae    b7750 <<mibz2::c30::C30<false>>::room+0x17a0>
   b773a:	mov    %edx,%ecx
   b773c:	shr    $0x8,%ecx
   b773f:	add    %esi,%ecx
   b7741:	jmp    b7773 <<mibz2::c30::C30<false>>::room+0x17c3>
   b7743:	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   b7750:	je     b7800 <<mibz2::c30::C30<false>>::room+0x1850>
   b7756:	mov    0x54(%rsp),%esi
   b775a:	cmp    %esi,%ecx
   b775c:	jne    b7890 <<mibz2::c30::C30<false>>::room+0x18e0>
   b7762:	mov    0x4(%r15),%ecx
   b7766:	mov    0x28(%rsp),%rdx
   b776b:	mov    (%rdx,%rcx,4),%edx
   b776e:	mov    %edx,%ecx
   b7770:	shr    $0x8,%ecx
   b7773:	mov    %ecx,(%r15)
   b7776:	mov    %ecx,%ecx
   b7778:	prefetcht0 (%r14,%rcx,2)
   b777d:	mov    -0x10(%r15),%rcx
   b7781:	mov    %dl,(%rcx)
   b7783:	mov    -0x10(%r15),%rbp
   b7787:	inc    %rbp
   b778a:	mov    %rbp,-0x10(%r15)
   b778e:	cmp    -0x8(%r15),%rbp
   b7792:	jne    b770e <<mibz2::c30::C30<false>>::room+0x175e>
   b7798:	mov    0x20(%rsp),%rcx
   b779d:	mov    0x70(%rcx),%rsi
   b77a1:	mov    (%rax),%rdi
   b77a4:	cmp    %rsi,%rdi
   b77a7:	jae    b7b77 <<mibz2::c30::C30<false>>::room+0x1bc7>
   b77ad:	mov    0x68(%rcx),%rcx
   b77b1:	lea    (%rdi,%rdi,2),%rsi
   b77b5:	mov    (%rcx,%rsi,8),%rdx
   b77b9:	mov    0x8(%rcx,%rsi,8),%rax
   b77be:	sub    %rax,%rbp
   b77c1:	mov    %rbp,0x10(%rcx,%rsi,8)
   b77c6:	mov    %rdx,%rdi
   b77c9:	sub    %rbp,%rdi
   b77cc:	cmp    %rdi,%rdx
   b77cf:	jbe    b7700 <<mibz2::c30::C30<false>>::room+0x1750>
   b77d5:	lea    (%rcx,%rsi,8),%rdi
   b77d9:	mov    $0x1,%ecx
   b77de:	mov    $0x1,%r8d
   b77e4:	mov    %rdi,0x40(%rsp)
   b77e9:	mov    %rbp,%rsi
   b77ec:	call   98d90 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   b77f1:	jmp    b76f0 <<mibz2::c30::C30<false>>::room+0x1740>
   b77f6:	cs nopw 0x0(%rax,%rax,1)
   b7800:	mov    0x20(%rsp),%r11
   b7805:	mov    0x70(%r11),%rsi
   b7809:	mov    (%rax),%rdi
   b780c:	cmp    %rsi,%rdi
   b780f:	jae    b7b6b <<mibz2::c30::C30<false>>::room+0x1bbb>
   b7815:	mov    0x68(%r11),%r8
   b7819:	mov    0x80(%r11),%rdx
   b7820:	lea    (%rdi,%rdi,2),%r9
   b7824:	mov    0x8(%rax),%r10
   b7828:	sub    0x8(%r8,%r9,8),%r10
   b782d:	mov    0x88(%r11),%rsi
   b7834:	mov    %r10,0x10(%r8,%r9,8)
   b7839:	mov    0x40(%r11),%r10
   b783d:	test   %r10,%r10
   b7840:	je     b78bc <<mibz2::c30::C30<false>>::room+0x190c>
   b7842:	mov    0x38(%r11),%r9
   b7846:	xor    %r8d,%r8d
   b7849:	cmp    $0x1,%r10
   b784d:	je     b786d <<mibz2::c30::C30<false>>::room+0x18bd>
   b784f:	nop
   b7850:	mov    %r8,%r11
   b7853:	mov    %r10,%r15
   b7856:	shr    $1,%r15
   b7859:	add    %r15,%r8
   b785c:	cmp    %ecx,(%r9,%r8,4)
   b7860:	cmova  %r11,%r8
   b7864:	sub    %r15,%r10
   b7867:	cmp    $0x1,%r10
   b786b:	ja     b7850 <<mibz2::c30::C30<false>>::room+0x18a0>
   b786d:	cmp    %ecx,(%r9,%r8,4)
   b7871:	mov    $0xffffffffffffffff,%rcx
   b7878:	cmovne %rcx,%r8
   b787c:	cmp    %rsi,%rdi
   b787f:	jb     b78cc <<mibz2::c30::C30<false>>::room+0x191c>
   b7881:	jmp    b7b62 <<mibz2::c30::C30<false>>::room+0x1bb2>
   b7886:	cs nopw 0x0(%rax,%rax,1)
   b7890:	shl    $0x5,%edx
   b7893:	and    $0x1ffffe0,%edx
   b7899:	and    $0x1f,%ecx
   b789c:	or     %edx,%ecx
   b789e:	mov    %ecx,0x4(%r15)
   b78a2:	mov    0x28(%rsp),%rax
   b78a7:	prefetcht0 (%rax,%rcx,4)
   b78ab:	mov    %esi,(%r15)
   b78ae:	inc    %rbx
   b78b1:	cmp    %r12,%rbx
   b78b4:	jb     b76e2 <<mibz2::c30::C30<false>>::room+0x1732>
   b78ba:	jmp    b78f8 <<mibz2::c30::C30<false>>::room+0x1948>
   b78bc:	mov    $0xffffffffffffffff,%r8
   b78c3:	cmp    %rsi,%rdi
   b78c6:	jae    b7b62 <<mibz2::c30::C30<false>>::room+0x1bb2>
   b78cc:	mov    %r8,(%rdx,%rdi,8)
   b78d0:	dec    %r12
   b78d3:	mov    %r12,%rcx
   b78d6:	shl    $0x5,%rcx
   b78da:	mov    0x8(%rsp),%rdx
   b78df:	movups (%rdx,%rcx,1),%xmm0
   b78e3:	movups 0x10(%rdx,%rcx,1),%xmm1
   b78e8:	movups %xmm1,0x10(%rax)
   b78ec:	movups %xmm0,(%rax)
   b78ef:	cmp    %r12,%rbx
   b78f2:	jb     b76e2 <<mibz2::c30::C30<false>>::room+0x1732>
   b78f8:	mov    $0x0,%ebx
   b78fd:	test   %r12,%r12
   b7900:	jne    b76e2 <<mibz2::c30::C30<false>>::room+0x1732>
   b7906:	mov    0x20(%rsp),%r13
   b790b:	mov    0x10(%rsp),%r14
   b7910:	xor    %eax,%eax
   b7912:	lea    0x7638f(%rip),%rcx        # 12dca8 <__frame_dummy_init_array_entry+0x1b58>
   b7919:	lea    0x763a0(%rip),%r8        # 12dcc0 <__frame_dummy_init_array_entry+0x1b70>
   b7920:	lea    0x76429(%rip),%r9        # 12dd50 <__frame_dummy_init_array_entry+0x1c00>
   b7927:	lea    0x7643a(%rip),%rdx        # 12dd68 <__frame_dummy_init_array_entry+0x1c18>
   b792e:	mov    0x38(%rsp),%r11
   b7933:	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   b7940:	mov    0x40(%r13),%rsi
   b7944:	cmp    %rsi,%rax
   b7947:	jae    b7b0d <<mibz2::c30::C30<false>>::room+0x1b5d>
   b794d:	mov    0xb0(%r13),%rsi
   b7954:	cmp    %rsi,%rax
   b7957:	jae    b7b05 <<mibz2::c30::C30<false>>::room+0x1b55>
   b795d:	mov    0x38(%r13),%rsi
   b7961:	mov    (%rsi,%rax,4),%edi
   b7964:	mov    0x10(%r13),%rsi
   b7968:	mov    %edi,%r10d
   b796b:	shr    $0x5,%r10d
   b796f:	cmp    %r10,%rsi
   b7972:	jbe    b7afd <<mibz2::c30::C30<false>>::room+0x1b4d>
   b7978:	and    $0x1f,%edi
   b797b:	cmp    $0x1e,%edi
   b797e:	jae    b7af6 <<mibz2::c30::C30<false>>::room+0x1b46>
   b7984:	mov    0xa8(%r13),%rsi
   b798b:	movzwl (%rsi,%rax,2),%esi
   b798f:	inc    %rax
   b7992:	shl    $0x6,%r10
   b7996:	add    0x8(%r13),%r10
   b799a:	mov    %si,(%r10,%rdi,2)
   b799f:	cmp    %rax,%r11
   b79a2:	jne    b7940 <<mibz2::c30::C30<false>>::room+0x1990>
   b79a4:	mov    0x38(%r13),%rax
   b79a8:	mov    0x40(%r13),%rcx
   b79ac:	mov    0x50(%rsp),%edi
   b79b0:	mov    %edi,%esi
   b79b2:	mov    $0x88888889,%edx
   b79b7:	imul   %rsi,%rdx
   b79bb:	shr    $0x23,%rdx
   b79bf:	and    $0xfffffffe,%edx
   b79c2:	add    %edi,%edx
   b79c4:	cmp    $0x1,%rcx
   b79c8:	je     b79fe <<mibz2::c30::C30<false>>::room+0x1a4e>
   b79ca:	test   %rcx,%rcx
   b79cd:	je     b7bbb <<mibz2::c30::C30<false>>::room+0x1c0b>
   b79d3:	xor    %esi,%esi
   b79d5:	data16 cs nopw 0x0(%rax,%rax,1)
   b79e0:	mov    %rsi,%rdi
   b79e3:	mov    %rcx,%r8
   b79e6:	shr    $1,%r8
   b79e9:	add    %r8,%rsi
   b79ec:	cmp    %edx,(%rax,%rsi,4)
   b79ef:	cmova  %rdi,%rsi
   b79f3:	sub    %r8,%rcx
   b79f6:	cmp    $0x1,%rcx
   b79fa:	ja     b79e0 <<mibz2::c30::C30<false>>::room+0x1a30>
   b79fc:	jmp    b7a00 <<mibz2::c30::C30<false>>::room+0x1a50>
   b79fe:	xor    %esi,%esi
   b7a00:	cmp    %edx,(%rax,%rsi,4)
   b7a03:	jne    b7bb4 <<mibz2::c30::C30<false>>::room+0x1c04>
   b7a09:	mov    %rsi,0x98(%r13)
   b7a10:	mov    0xd8(%rsp),%rax
   b7a18:	mov    %rax,0x90(%r13)
   b7a1f:	test   %r14,%r14
   b7a22:	je     b7a2f <<mibz2::c30::C30<false>>::room+0x1a7f>
   b7a24:	mov    0x8(%rsp),%rdi
   b7a29:	call   *0x78c59(%rip)        # 130688 <free@GLIBC_2.2.5>
   b7a2f:	add    $0x2688,%rsp
   b7a36:	pop    %rbx
   b7a37:	pop    %r12
   b7a39:	pop    %r13
   b7a3b:	pop    %r14
   b7a3d:	pop    %r15
   b7a3f:	pop    %rbp
   b7a40:	ret
   b7a41:	lea    0x76290(%rip),%rcx        # 12dcd8 <__frame_dummy_init_array_entry+0x1b88>
   b7a48:	xor    %edi,%edi
   b7a4a:	mov    %rbx,%rsi
   b7a4d:	call   8b5e0 <core::slice::index::slice_index_fail>
   b7a52:	lea    0x78(%r13),%rdi
   b7a56:	mov    $0x8,%ecx
   b7a5b:	mov    $0x8,%r8d
   b7a61:	xor    %esi,%esi
   b7a63:	call   98d90 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   b7a68:	mov    0x88(%r13),%rbx
   b7a6f:	jmp    b63cd <<mibz2::c30::C30<false>>::room+0x41d>
   b7a74:	lea    0x75ead(%rip),%rdx        # 12d928 <__frame_dummy_init_array_entry+0x17d8>
   b7a7b:	call   8a87b <core::panicking::panic_bounds_check>
   b7a80:	lea    0x76299(%rip),%rdx        # 12dd20 <__frame_dummy_init_array_entry+0x1bd0>
   b7a87:	mov    %r12,%rdi
   b7a8a:	call   8a87b <core::panicking::panic_bounds_check>
   b7a8f:	lea    0x762a2(%rip),%rdx        # 12dd38 <__frame_dummy_init_array_entry+0x1be8>
   b7a96:	mov    $0x1e,%esi
   b7a9b:	mov    %rbp,%rdi
   b7a9e:	call   8a87b <core::panicking::panic_bounds_check>
   b7aa3:	lea    0x7625e(%rip),%rdx        # 12dd08 <__frame_dummy_init_array_entry+0x1bb8>
   b7aaa:	call   8a87b <core::panicking::panic_bounds_check>
   b7aaf:	lea    0x762b2(%rip),%rdx        # 12dd68 <__frame_dummy_init_array_entry+0x1c18>
   b7ab6:	mov    $0x1e,%esi
   b7abb:	mov    %rax,%rdi
   b7abe:	call   8a87b <core::panicking::panic_bounds_check>
   b7ac3:	lea    0x76286(%rip),%rdx        # 12dd50 <__frame_dummy_init_array_entry+0x1c00>
   b7aca:	mov    %rcx,%rdi
   b7acd:	call   8a87b <core::panicking::panic_bounds_check>
   b7ad2:	lea    0x76217(%rip),%rdx        # 12dcf0 <__frame_dummy_init_array_entry+0x1ba0>
   b7ad9:	call   8a87b <core::panicking::panic_bounds_check>
   b7ade:	lea    0x75dfb(%rip),%rdx        # 12d8e0 <__frame_dummy_init_array_entry+0x1790>
   b7ae5:	call   8a87b <core::panicking::panic_bounds_check>
   b7aea:	lea    0x75dd7(%rip),%rdx        # 12d8c8 <__frame_dummy_init_array_entry+0x1778>
   b7af1:	call   8a87b <core::panicking::panic_bounds_check>
   b7af6:	mov    $0x1e,%esi
   b7afb:	jmp    b7b13 <<mibz2::c30::C30<false>>::room+0x1b63>
   b7afd:	mov    %r10,%rdi
   b7b00:	mov    %r9,%rdx
   b7b03:	jmp    b7b13 <<mibz2::c30::C30<false>>::room+0x1b63>
   b7b05:	mov    %rax,%rdi
   b7b08:	mov    %r8,%rdx
   b7b0b:	jmp    b7b13 <<mibz2::c30::C30<false>>::room+0x1b63>
   b7b0d:	mov    %rax,%rdi
   b7b10:	mov    %rcx,%rdx
   b7b13:	call   8a87b <core::panicking::panic_bounds_check>
   b7b18:	jmp    b7bea <<mibz2::c30::C30<false>>::room+0x1c3a>
   b7b1d:	lea    0x7613c(%rip),%rdx        # 12dc60 <__frame_dummy_init_array_entry+0x1b10>
   b7b24:	mov    %r13,%rdi
   b7b27:	call   8a87b <core::panicking::panic_bounds_check>
   b7b2c:	lea    0x75ddd(%rip),%rdx        # 12d910 <__frame_dummy_init_array_entry+0x17c0>
   b7b33:	call   8a87b <core::panicking::panic_bounds_check>
   b7b38:	lea    0x75db9(%rip),%rdx        # 12d8f8 <__frame_dummy_init_array_entry+0x17a8>
   b7b3f:	call   8a87b <core::panicking::panic_bounds_check>
   b7b44:	lea    0x75d7d(%rip),%rdx        # 12d8c8 <__frame_dummy_init_array_entry+0x1778>
   b7b4b:	mov    %r11,%rdi
   b7b4e:	call   8a87b <core::panicking::panic_bounds_check>
   b7b53:	lea    0x75d86(%rip),%rdx        # 12d8e0 <__frame_dummy_init_array_entry+0x1790>
   b7b5a:	mov    %r11,%rdi
   b7b5d:	call   8a87b <core::panicking::panic_bounds_check>
   b7b62:	lea    0x75da7(%rip),%rax        # 12d910 <__frame_dummy_init_array_entry+0x17c0>
   b7b69:	jmp    b7b72 <<mibz2::c30::C30<false>>::room+0x1bc2>
   b7b6b:	lea    0x75d86(%rip),%rax        # 12d8f8 <__frame_dummy_init_array_entry+0x17a8>
   b7b72:	mov    %rax,0x30(%rsp)
   b7b77:	mov    0x30(%rsp),%rdx
   b7b7c:	mov    0x10(%rsp),%r14
   b7b81:	call   8a87b <core::panicking::panic_bounds_check>
   b7b86:	jmp    b7bea <<mibz2::c30::C30<false>>::room+0x1c3a>
   b7b88:	lea    0x75db1(%rip),%rdx        # 12d940 <__frame_dummy_init_array_entry+0x17f0>
   b7b8f:	mov    %r15,%rdi
   b7b92:	call   8a87b <core::panicking::panic_bounds_check>
   b7b97:	lea    0x75da2(%rip),%rdx        # 12d940 <__frame_dummy_init_array_entry+0x17f0>
   b7b9e:	mov    %r13,%rdi
   b7ba1:	call   8a87b <core::panicking::panic_bounds_check>
   b7ba6:	jmp    b7bea <<mibz2::c30::C30<false>>::room+0x1c3a>
   b7ba8:	lea    0x760c9(%rip),%rdi        # 12dc78 <__frame_dummy_init_array_entry+0x1b28>
   b7baf:	call   8d840 <core::panicking::panic_const::panic_const_div_by_zero>
   b7bb4:	adc    $0x0,%rsi
   b7bb8:	mov    %rsi,%rcx
   b7bbb:	mov    %rcx,0x680(%rsp)
   b7bc3:	lea    -0xa60f9(%rip),%rdi        # 11ad1 <core::num::imp::flt2dec::strategy::dragon::POW5TO256+0x27d>
   b7bca:	lea    0x7606f(%rip),%rcx        # 12dc40 <__frame_dummy_init_array_entry+0x1af0>
   b7bd1:	lea    0x760b8(%rip),%r8        # 12dc90 <__frame_dummy_init_array_entry+0x1b40>
   b7bd8:	lea    0x680(%rsp),%rdx
   b7be0:	mov    $0x19,%esi
   b7be5:	call   8d530 <core::result::unwrap_failed>
   b7bea:	ud2
   b7bec:	mov    $0x8,%edi
   b7bf1:	mov    $0x80,%esi
   b7bf6:	call   88c9b <alloc::raw_vec::handle_error>
   b7bfb:	jmp    b7bff <<mibz2::c30::C30<false>>::room+0x1c4f>
   b7bfd:	jmp    b7c09 <<mibz2::c30::C30<false>>::room+0x1c59>
   b7bff:	mov    %rax,%rbx
   b7c02:	mov    0x10(%rsp),%r14
   b7c07:	jmp    b7c29 <<mibz2::c30::C30<false>>::room+0x1c79>
   b7c09:	mov    %rax,%rbx
   b7c0c:	cmpq   $0x0,0x580(%rsp)
   b7c15:	je     b7c39 <<mibz2::c30::C30<false>>::room+0x1c89>
   b7c17:	mov    0x588(%rsp),%rax
   b7c1f:	mov    %rax,0x8(%rsp)
   b7c24:	jmp    b7c2e <<mibz2::c30::C30<false>>::room+0x1c7e>
   b7c26:	mov    %rax,%rbx
   b7c29:	test   %r14,%r14
   b7c2c:	je     b7c39 <<mibz2::c30::C30<false>>::room+0x1c89>
   b7c2e:	mov    0x8(%rsp),%rdi
   b7c33:	call   *0x78a4f(%rip)        # 130688 <free@GLIBC_2.2.5>
   b7c39:	mov    %rbx,%rdi
   b7c3c:	call   12b0b0 <_Unwind_Resume@plt>
   b7c41:	int3
   b7c42:	int3
   b7c43:	int3
   b7c44:	int3
   b7c45:	int3
   b7c46:	int3
   b7c47:	int3
   b7c48:	int3
   b7c49:	int3
   b7c4a:	int3
   b7c4b:	int3
   b7c4c:	int3
   b7c4d:	int3
   b7c4e:	int3
   b7c4f:	int3

#### <mibz2::c30::C30<true>>::room: 1509 instructions, 33 prefetcht0, 40 calls
   b5f50:	push   %rbp
   b5f51:	push   %r15
   b5f53:	push   %r14
   b5f55:	push   %r13
   b5f57:	push   %r12
   b5f59:	push   %rbx
   b5f5a:	sub    $0x1000,%rsp
   b5f61:	movq   $0x0,(%rsp)
   b5f69:	sub    $0x1000,%rsp
   b5f70:	movq   $0x0,(%rsp)
   b5f78:	sub    $0x698,%rsp
   b5f7f:	mov    %esi,0x54(%rsp)
   b5f83:	mov    0xd0(%rdi),%rax
   b5f8a:	mov    %rax,0xd8(%rsp)
   b5f92:	mov    0x10(%rdi),%eax
   b5f95:	mov    0x40(%rdi),%r12
   b5f99:	movq   $0x0,0xc8(%rdi)
   b5fa4:	movq   $0x0,0xb0(%rdi)
   b5faf:	test   %r12,%r12
   b5fb2:	je     b7caa <<mibz2::c30::C30<true>>::room+0x1d5a>
   b5fb8:	mov    %rdi,%rbp
   b5fbb:	shl    $0x5,%eax
   b5fbe:	add    $0xffffffe0,%eax
   b5fc1:	mov    %eax,0x44(%rsp)
   b5fc5:	xor    %r15d,%r15d
   b5fc8:	lea    0xb8(%rdi),%rax
   b5fcf:	mov    %rax,0x10(%rsp)
   b5fd4:	lea    0xa0(%rdi),%rax
   b5fdb:	mov    %rax,0x30(%rsp)
   b5fe0:	mov    %rdi,0xe0(%rsp)
   b5fe8:	mov    %r12,0x18(%rsp)
   b5fed:	jmp    b6019 <<mibz2::c30::C30<true>>::room+0xc9>
   b5fef:	nop
   b5ff0:	inc    %r15
   b5ff3:	mov    0xa8(%r14),%rax
   b5ffa:	mov    %bp,(%rax,%rbx,2)
   b5ffe:	inc    %rbx
   b6001:	mov    %rbx,0xb0(%r14)
   b6008:	mov    0x18(%rsp),%r12
   b600d:	cmp    %r15,%r12
   b6010:	mov    %r14,%rbp
   b6013:	je     b60de <<mibz2::c30::C30<true>>::room+0x18e>
   b6019:	mov    0x40(%rbp),%rsi
   b601d:	cmp    %rsi,%r15
   b6020:	jae    b7bee <<mibz2::c30::C30<true>>::room+0x1c9e>
   b6026:	mov    0x38(%rbp),%rax
   b602a:	mov    %rbp,%r14
   b602d:	mov    (%rax,%r15,4),%ebp
   b6031:	mov    %r14,%rdi
   b6034:	mov    %ebp,%esi
   b6036:	call   b7d50 <<mibz2::c30::C30<true>>::next>
   b603b:	mov    %eax,%r12d
   b603e:	mov    %edx,%r13d
   b6041:	mov    0xc8(%r14),%rbx
   b6048:	cmp    0xb8(%r14),%rbx
   b604f:	je     b60cf <<mibz2::c30::C30<true>>::room+0x17f>
   b6051:	shl    $0x8,%r12d
   b6055:	movzbl %r13b,%eax
   b6059:	or     %r12d,%eax
   b605c:	mov    0xc0(%r14),%rcx
   b6063:	mov    %eax,(%rcx,%rbx,4)
   b6066:	inc    %rbx
   b6069:	mov    %rbx,0xc8(%r14)
   b6070:	mov    0x8(%r14),%rax
   b6074:	mov    0x10(%r14),%rdx
   b6078:	mov    %ebp,0x110(%rsp)
   b607f:	mov    %ebp,%ecx
   b6081:	and    $0x1f,%ecx
   b6084:	cmp    $0x1d,%ecx
   b6087:	ja     b61df <<mibz2::c30::C30<true>>::room+0x28f>
   b608d:	shr    $0x5,%ebp
   b6090:	cmp    %rbp,%rdx
   b6093:	jbe    b61df <<mibz2::c30::C30<true>>::room+0x28f>
   b6099:	imul   $0x3c,%rbp,%rdx
   b609d:	add    %rdx,%rax
   b60a0:	movzwl (%rax,%rcx,2),%ebp
   b60a4:	mov    0xe0(%rsp),%r14
   b60ac:	mov    0xb0(%r14),%rbx
   b60b3:	cmp    0xa0(%r14),%rbx
   b60ba:	jne    b5ff0 <<mibz2::c30::C30<true>>::room+0xa0>
   b60c0:	mov    0x30(%rsp),%rdi
   b60c5:	call   b7e20 <<alloc::raw_vec::RawVec<u16>>::grow_one>
   b60ca:	jmp    b5ff0 <<mibz2::c30::C30<true>>::room+0xa0>
   b60cf:	mov    0x10(%rsp),%rdi
   b60d4:	call   9b1f0 <<alloc::raw_vec::RawVec<u32>>::grow_one>
   b60d9:	jmp    b6051 <<mibz2::c30::C30<true>>::room+0x101>
   b60de:	xor    %edi,%edi
   b60e0:	mov    0x40(%rbp),%rsi
   b60e4:	cmp    %rsi,%rdi
   b60e7:	jae    b7bfd <<mibz2::c30::C30<true>>::room+0x1cad>
   b60ed:	mov    0x38(%rbp),%rax
   b60f1:	mov    (%rax,%rdi,4),%eax
   b60f4:	mov    0x8(%rbp),%rcx
   b60f8:	mov    0x10(%rbp),%rsi
   b60fc:	mov    %eax,0x110(%rsp)
   b6103:	mov    %eax,%edx
   b6105:	and    $0x1f,%edx
   b6108:	cmp    $0x1d,%edx
   b610b:	ja     b6219 <<mibz2::c30::C30<true>>::room+0x2c9>
   b6111:	shr    $0x5,%eax
   b6114:	cmp    %rax,%rsi
   b6117:	jbe    b6219 <<mibz2::c30::C30<true>>::room+0x2c9>
   b611d:	inc    %rdi
   b6120:	imul   $0x3c,%rax,%rax
   b6124:	add    %rax,%rcx
   b6127:	movw   $0xfffe,(%rcx,%rdx,2)
   b612d:	cmp    %rdi,%r12
   b6130:	jne    b60e0 <<mibz2::c30::C30<true>>::room+0x190>
   b6132:	mov    0x8(%rbp),%rax
   b6136:	mov    %rax,0x10(%rsp)
   b613b:	mov    0x20(%rbp),%rax
   b613f:	mov    %rax,0x30(%rsp)
   b6144:	imul   $0x3c,0x10(%rbp),%r14
   b6149:	mov    0xd8(%rsp),%rax
   b6151:	add    %rax,%rax
   b6154:	mov    %rax,%rcx
   b6157:	or     %r12,%rcx
   b615a:	shr    $0x20,%rcx
   b615e:	je     b6167 <<mibz2::c30::C30<true>>::room+0x217>
   b6160:	xor    %edx,%edx
   b6162:	div    %r12
   b6165:	jmp    b616c <<mibz2::c30::C30<true>>::room+0x21c>
   b6167:	xor    %edx,%edx
   b6169:	div    %r12d
   b616c:	mov    0x70(%rbp),%r13
   b6170:	cmp    %r12,%r13
   b6173:	cmova  %r13,%r12
   b6177:	mov    %rax,0x38(%rsp)
   b617c:	jae    b61cc <<mibz2::c30::C30<true>>::room+0x27c>
   b617e:	sub    %r13,%r12
   b6181:	mov    0x60(%rbp),%rcx
   b6185:	sub    %r13,%rcx
   b6188:	mov    %r13,%rdx
   b618b:	cmp    %rcx,%r12
   b618e:	ja     b6253 <<mibz2::c30::C30<true>>::room+0x303>
   b6194:	mov    0x68(%rbp),%rsi
   b6198:	inc    %r13
   b619b:	test   $0x1,%r12b
   b619f:	jne    b6287 <<mibz2::c30::C30<true>>::room+0x337>
   b61a5:	cmp    %r13,0x18(%rsp)
   b61aa:	mov    0x10(%rsp),%rdi
   b61af:	jne    b62bb <<mibz2::c30::C30<true>>::room+0x36b>
   b61b5:	mov    %rdx,0x70(%rbp)
   b61b9:	mov    0x18(%rsp),%rsi
   b61be:	cmp    %rdx,%rsi
   b61c1:	jbe    b6315 <<mibz2::c30::C30<true>>::room+0x3c5>
   b61c7:	jmp    b7baf <<mibz2::c30::C30<true>>::room+0x1c5f>
   b61cc:	mov    %r12,0x70(%rbp)
   b61d0:	mov    0x18(%rsp),%rsi
   b61d5:	mov    0x10(%rsp),%rdi
   b61da:	jmp    b6315 <<mibz2::c30::C30<true>>::room+0x3c5>
   b61df:	lea    0x110(%rsp),%rax
   b61e7:	mov    %rax,0x690(%rsp)
   b61ef:	lea    -0x29436(%rip),%rax        # 8cdc0 <<u32 as core::fmt::Display>::fmt>
   b61f6:	mov    %rax,0x698(%rsp)
   b61fe:	lea    -0xab366(%rip),%rdi        # ae9f <__abi_tag+0xaba3>
   b6205:	lea    0x77ce4(%rip),%rdx        # 12def0 <__frame_dummy_init_array_entry+0x1bd0>
   b620c:	lea    0x690(%rsp),%rsi
   b6214:	call   89b50 <core::panicking::panic_fmt>
   b6219:	lea    0x110(%rsp),%rax
   b6221:	mov    %rax,0x690(%rsp)
   b6229:	lea    -0x29470(%rip),%rax        # 8cdc0 <<u32 as core::fmt::Display>::fmt>
   b6230:	mov    %rax,0x698(%rsp)
   b6238:	lea    -0xab3a0(%rip),%rdi        # ae9f <__abi_tag+0xaba3>
   b623f:	lea    0x77cc2(%rip),%rdx        # 12df08 <__frame_dummy_init_array_entry+0x1be8>
   b6246:	lea    0x690(%rsp),%rsi
   b624e:	call   89b50 <core::panicking::panic_fmt>
   b6253:	lea    0x60(%rbp),%rdi
   b6257:	mov    $0x8,%ecx
   b625c:	mov    $0x18,%r8d
   b6262:	mov    %r13,%rsi
   b6265:	mov    %r12,%rdx
   b6268:	call   98dd0 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   b626d:	mov    0x38(%rsp),%rax
   b6272:	mov    0x70(%rbp),%rdx
   b6276:	mov    0x68(%rbp),%rsi
   b627a:	inc    %r13
   b627d:	test   $0x1,%r12b
   b6281:	je     b61a5 <<mibz2::c30::C30<true>>::room+0x255>
   b6287:	dec    %r12
   b628a:	lea    (%rdx,%rdx,2),%rcx
   b628e:	movq   $0x0,(%rsi,%rcx,8)
   b6296:	movq   $0x1,0x8(%rsi,%rcx,8)
   b629f:	movq   $0x0,0x10(%rsi,%rcx,8)
   b62a8:	inc    %rdx
   b62ab:	cmp    %r13,0x18(%rsp)
   b62b0:	mov    0x10(%rsp),%rdi
   b62b5:	je     b61b5 <<mibz2::c30::C30<true>>::room+0x265>
   b62bb:	lea    (%rdx,%rdx,2),%rcx
   b62bf:	lea    (%rsi,%rcx,8),%rsi
   b62c3:	add    $0x28,%rsi
   b62c7:	neg    %r12
   b62ca:	xor    %ecx,%ecx
   b62cc:	xorps  %xmm0,%xmm0
   b62cf:	nop
   b62d0:	movq   $0x0,-0x28(%rsi)
   b62d8:	movq   $0x1,-0x20(%rsi)
   b62e0:	movups %xmm0,-0x18(%rsi)
   b62e4:	movq   $0x1,-0x8(%rsi)
   b62ec:	movq   $0x0,(%rsi)
   b62f3:	add    $0x30,%rsi
   b62f7:	add    $0xfffffffffffffffe,%rcx
   b62fb:	cmp    %rcx,%r12
   b62fe:	jne    b62d0 <<mibz2::c30::C30<true>>::room+0x380>
   b6300:	sub    %rcx,%rdx
   b6303:	mov    %rdx,0x70(%rbp)
   b6307:	mov    0x18(%rsp),%rsi
   b630c:	cmp    %rdx,%rsi
   b630f:	ja     b7baf <<mibz2::c30::C30<true>>::room+0x1c5f>
   b6315:	add    %rdi,%r14
   b6318:	lea    0x40(%rax),%r12
   b631c:	mov    0x68(%rbp),%r13
   b6320:	lea    0x0(,%rsi,8),%rax
   b6328:	lea    (%rax,%rax,2),%rbx
   b632c:	nopl   0x0(%rax)
   b6330:	movq   $0x0,0x10(%r13)
   b6338:	cmp    0x0(%r13),%r12
   b633c:	ja     b634a <<mibz2::c30::C30<true>>::room+0x3fa>
   b633e:	add    $0x18,%r13
   b6342:	add    $0xffffffffffffffe8,%rbx
   b6346:	jne    b6330 <<mibz2::c30::C30<true>>::room+0x3e0>
   b6348:	jmp    b636c <<mibz2::c30::C30<true>>::room+0x41c>
   b634a:	mov    $0x1,%ecx
   b634f:	mov    $0x1,%r8d
   b6355:	mov    %r13,%rdi
   b6358:	xor    %esi,%esi
   b635a:	mov    %r12,%rdx
   b635d:	call   98dd0 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   b6362:	add    $0x18,%r13
   b6366:	add    $0xffffffffffffffe8,%rbx
   b636a:	jne    b6330 <<mibz2::c30::C30<true>>::room+0x3e0>
   b636c:	movq   $0x0,0x88(%rbp)
   b6377:	mov    0x18(%rsp),%r12
   b637c:	cmp    0x78(%rbp),%r12
   b6380:	ja     b7bbd <<mibz2::c30::C30<true>>::room+0x1c6d>
   b6386:	xor    %ebx,%ebx
   b6388:	mov    0x10(%rsp),%r13
   b638d:	mov    0x80(%rbp),%r15
   b6394:	lea    (%r15,%rbx,8),%rdi
   b6398:	cmp    $0x1,%r12
   b639c:	je     b63c3 <<mibz2::c30::C30<true>>::room+0x473>
   b639e:	lea    -0x8(,%r12,8),%rdx
   b63a6:	mov    $0xff,%esi
   b63ab:	call   *0x7a4e7(%rip)        # 130898 <memset@GLIBC_2.2.5>
   b63b1:	lea    (%rbx,%r12,1),%rax
   b63b5:	lea    (%r15,%rax,8),%rdi
   b63b9:	add    $0xfffffffffffffff8,%rdi
   b63bd:	add    %r12,%rbx
   b63c0:	dec    %rbx
   b63c3:	movq   $0xffffffffffffffff,(%rdi)
   b63ca:	inc    %rbx
   b63cd:	mov    %rbx,0x88(%rbp)
   b63d4:	xorps  %xmm0,%xmm0
   b63d7:	movups %xmm0,0x118(%rsp)
   b63df:	movq   $0x0,0x128(%rsp)
   b63eb:	movups %xmm0,0x138(%rsp)
   b63f3:	movq   $0x0,0x148(%rsp)
   b63ff:	movups %xmm0,0x158(%rsp)
   b6407:	movq   $0x0,0x168(%rsp)
   b6413:	movups %xmm0,0x178(%rsp)
   b641b:	movq   $0x0,0x188(%rsp)
   b6427:	movups %xmm0,0x198(%rsp)
   b642f:	movq   $0x0,0x1a8(%rsp)
   b643b:	movups %xmm0,0x1b8(%rsp)
   b6443:	movq   $0x0,0x1c8(%rsp)
   b644f:	movups %xmm0,0x1d8(%rsp)
   b6457:	movq   $0x0,0x1e8(%rsp)
   b6463:	movups %xmm0,0x1f8(%rsp)
   b646b:	movq   $0x0,0x208(%rsp)
   b6477:	movups %xmm0,0x218(%rsp)
   b647f:	movq   $0x0,0x228(%rsp)
   b648b:	movups %xmm0,0x238(%rsp)
   b6493:	movq   $0x0,0x248(%rsp)
   b649f:	movups %xmm0,0x258(%rsp)
   b64a7:	movq   $0x0,0x268(%rsp)
   b64b3:	movups %xmm0,0x278(%rsp)
   b64bb:	movq   $0x0,0x288(%rsp)
   b64c7:	movups %xmm0,0x298(%rsp)
   b64cf:	movq   $0x0,0x2a8(%rsp)
   b64db:	movups %xmm0,0x2b8(%rsp)
   b64e3:	movq   $0x0,0x2c8(%rsp)
   b64ef:	movups %xmm0,0x2d8(%rsp)
   b64f7:	movq   $0x0,0x2e8(%rsp)
   b6503:	movups %xmm0,0x2f8(%rsp)
   b650b:	movq   $0x0,0x308(%rsp)
   b6517:	movups %xmm0,0x318(%rsp)
   b651f:	movq   $0x0,0x328(%rsp)
   b652b:	movups %xmm0,0x338(%rsp)
   b6533:	movq   $0x0,0x348(%rsp)
   b653f:	movups %xmm0,0x358(%rsp)
   b6547:	movq   $0x0,0x368(%rsp)
   b6553:	movups %xmm0,0x378(%rsp)
   b655b:	movq   $0x0,0x388(%rsp)
   b6567:	movups %xmm0,0x398(%rsp)
   b656f:	movq   $0x0,0x3a8(%rsp)
   b657b:	movups %xmm0,0x3b8(%rsp)
   b6583:	movq   $0x0,0x3c8(%rsp)
   b658f:	movups %xmm0,0x3d8(%rsp)
   b6597:	movq   $0x0,0x3e8(%rsp)
   b65a3:	movups %xmm0,0x3f8(%rsp)
   b65ab:	movq   $0x0,0x408(%rsp)
   b65b7:	movups %xmm0,0x418(%rsp)
   b65bf:	movq   $0x0,0x428(%rsp)
   b65cb:	movups %xmm0,0x438(%rsp)
   b65d3:	movq   $0x0,0x448(%rsp)
   b65df:	movups %xmm0,0x458(%rsp)
   b65e7:	movq   $0x0,0x468(%rsp)
   b65f3:	movups %xmm0,0x478(%rsp)
   b65fb:	movq   $0x0,0x488(%rsp)
   b6607:	movups %xmm0,0x498(%rsp)
   b660f:	movq   $0x0,0x4a8(%rsp)
   b661b:	movups %xmm0,0x4b8(%rsp)
   b6623:	movq   $0x0,0x4c8(%rsp)
   b662f:	movups %xmm0,0x4d8(%rsp)
   b6637:	movq   $0x0,0x4e8(%rsp)
   b6643:	movups %xmm0,0x4f8(%rsp)
   b664b:	movq   $0x0,0x508(%rsp)
   b6657:	movq   $0xffffffffffffffff,0x110(%rsp)
   b6663:	movq   $0xffffffffffffffff,0x130(%rsp)
   b666f:	movq   $0xffffffffffffffff,0x150(%rsp)
   b667b:	movq   $0xffffffffffffffff,0x170(%rsp)
   b6687:	movq   $0xffffffffffffffff,0x190(%rsp)
   b6693:	movq   $0xffffffffffffffff,0x1b0(%rsp)
   b669f:	movq   $0xffffffffffffffff,0x1d0(%rsp)
   b66ab:	movq   $0xffffffffffffffff,0x1f0(%rsp)
   b66b7:	movq   $0xffffffffffffffff,0x210(%rsp)
   b66c3:	movq   $0xffffffffffffffff,0x230(%rsp)
   b66cf:	movq   $0xffffffffffffffff,0x250(%rsp)
   b66db:	movq   $0xffffffffffffffff,0x270(%rsp)
   b66e7:	movq   $0xffffffffffffffff,0x290(%rsp)
   b66f3:	movq   $0xffffffffffffffff,0x2b0(%rsp)
   b66ff:	movq   $0xffffffffffffffff,0x2d0(%rsp)
   b670b:	movq   $0xffffffffffffffff,0x2f0(%rsp)
   b6717:	movq   $0xffffffffffffffff,0x310(%rsp)
   b6723:	movq   $0xffffffffffffffff,0x330(%rsp)
   b672f:	movq   $0xffffffffffffffff,0x350(%rsp)
   b673b:	movq   $0xffffffffffffffff,0x370(%rsp)
   b6747:	movq   $0xffffffffffffffff,0x390(%rsp)
   b6753:	movq   $0xffffffffffffffff,0x3b0(%rsp)
   b675f:	movq   $0xffffffffffffffff,0x3d0(%rsp)
   b676b:	movq   $0xffffffffffffffff,0x3f0(%rsp)
   b6777:	movq   $0xffffffffffffffff,0x410(%rsp)
   b6783:	movq   $0xffffffffffffffff,0x430(%rsp)
   b678f:	movq   $0xffffffffffffffff,0x450(%rsp)
   b679b:	movq   $0xffffffffffffffff,0x470(%rsp)
   b67a7:	movq   $0xffffffffffffffff,0x490(%rsp)
   b67b3:	movq   $0xffffffffffffffff,0x4b0(%rsp)
   b67bf:	movq   $0xffffffffffffffff,0x4d0(%rsp)
   b67cb:	movq   $0xffffffffffffffff,0x4f0(%rsp)
   b67d7:	mov    $0x2,%eax
   b67dc:	xor    %r10d,%r10d
   b67df:	xor    %edi,%edi
   b67e1:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   b67f0:	mov    0x70(%rbp),%rsi
   b67f4:	cmp    %rsi,%rdi
   b67f7:	jae    b7c15 <<mibz2::c30::C30<true>>::room+0x1cc5>
   b67fd:	mov    0xc8(%rbp),%rsi
   b6804:	cmp    %rsi,%rdi
   b6807:	jae    b7c09 <<mibz2::c30::C30<true>>::room+0x1cb9>
   b680d:	mov    0x68(%rbp),%rcx
   b6811:	mov    0xc0(%rbp),%rdx
   b6818:	mov    (%rcx,%rax,4),%rsi
   b681c:	movzbl (%rdx,%rdi,4),%r8d
   b6821:	mov    %r8b,(%rsi)
   b6824:	mov    (%rdx,%rdi,4),%edx
   b6827:	mov    %edx,%r8d
   b682a:	shr    $0x8,%r8d
   b682e:	lea    0x1(%rsi),%r9
   b6832:	add    -0x8(%rcx,%rax,4),%rsi
   b6837:	mov    %rdi,0x110(%rsp,%r10,1)
   b683f:	mov    %r9,0x118(%rsp,%r10,1)
   b6847:	mov    %rsi,0x120(%rsp,%r10,1)
   b684f:	mov    %r8d,0x128(%rsp,%r10,1)
   b6857:	movl   $0x0,0x12c(%rsp,%r10,1)
   b6863:	add    %r8,%r8
   b6866:	shr    $0xd,%edx
   b6869:	lea    0x0(,%rdx,4),%rcx
   b6871:	sub    %rcx,%r8
   b6874:	prefetcht0 0x0(%r13,%r8,1)
   b687a:	prefetcht0 (%r14,%rdx,4)
   b687f:	lea    0x1(%rdi),%rcx
   b6883:	add    $0x20,%r10
   b6887:	cmp    $0x1e,%rdi
   b688b:	ja     b689d <<mibz2::c30::C30<true>>::room+0x94d>
   b688d:	add    $0x6,%rax
   b6891:	mov    %rcx,%rdi
   b6894:	cmp    %r12,%rcx
   b6897:	jb     b67f0 <<mibz2::c30::C30<true>>::room+0x8a0>
   b689d:	cmp    $0x20,%rcx
   b68a1:	jne    b7566 <<mibz2::c30::C30<true>>::room+0x1616>
   b68a7:	mov    %rcx,0xe8(%rsp)
   b68af:	mov    %r10,0xf0(%rsp)
   b68b7:	xorps  %xmm0,%xmm0
   b68ba:	movaps %xmm0,0x580(%rsp)
   b68c2:	movaps %xmm0,0x570(%rsp)
   b68ca:	movaps %xmm0,0x560(%rsp)
   b68d2:	movaps %xmm0,0x550(%rsp)
   b68da:	movaps %xmm0,0x540(%rsp)
   b68e2:	movaps %xmm0,0x530(%rsp)
   b68ea:	movaps %xmm0,0x520(%rsp)
   b68f2:	movaps %xmm0,0x510(%rsp)
   b68fa:	mov    0x128(%rsp),%eax
   b6901:	mov    0x148(%rsp),%ecx
   b6908:	mov    %eax,0x58(%rsp)
   b690c:	mov    %ecx,0x5c(%rsp)
   b6910:	mov    0x168(%rsp),%eax
   b6917:	mov    %eax,0x60(%rsp)
   b691b:	mov    0x188(%rsp),%eax
   b6922:	mov    %eax,0x64(%rsp)
   b6926:	mov    0x1a8(%rsp),%eax
   b692d:	mov    %eax,0x68(%rsp)
   b6931:	mov    0x1c8(%rsp),%eax
   b6938:	mov    %eax,0x6c(%rsp)
   b693c:	mov    0x1e8(%rsp),%eax
   b6943:	mov    %eax,0x70(%rsp)
   b6947:	mov    0x208(%rsp),%eax
   b694e:	mov    %eax,0x74(%rsp)
   b6952:	mov    0x228(%rsp),%eax
   b6959:	mov    %eax,0x78(%rsp)
   b695d:	mov    0x248(%rsp),%eax
   b6964:	mov    %eax,0x7c(%rsp)
   b6968:	mov    0x268(%rsp),%eax
   b696f:	mov    %eax,0x80(%rsp)
   b6976:	mov    0x288(%rsp),%eax
   b697d:	mov    %eax,0x84(%rsp)
   b6984:	mov    0x2a8(%rsp),%eax
   b698b:	mov    %eax,0x88(%rsp)
   b6992:	mov    0x2c8(%rsp),%eax
   b6999:	mov    %eax,0x8c(%rsp)
   b69a0:	mov    0x2e8(%rsp),%eax
   b69a7:	mov    %eax,0x90(%rsp)
   b69ae:	mov    0x308(%rsp),%eax
   b69b5:	mov    %eax,0x94(%rsp)
   b69bc:	mov    0x328(%rsp),%eax
   b69c3:	mov    %eax,0x98(%rsp)
   b69ca:	mov    0x348(%rsp),%eax
   b69d1:	mov    %eax,0x9c(%rsp)
   b69d8:	mov    0x368(%rsp),%eax
   b69df:	mov    %eax,0xa0(%rsp)
   b69e6:	mov    0x388(%rsp),%eax
   b69ed:	mov    %eax,0xa4(%rsp)
   b69f4:	mov    0x3a8(%rsp),%eax
   b69fb:	mov    %eax,0xa8(%rsp)
   b6a02:	mov    0x3c8(%rsp),%eax
   b6a09:	mov    %eax,0xac(%rsp)
   b6a10:	mov    0x3e8(%rsp),%eax
   b6a17:	mov    %eax,0xb0(%rsp)
   b6a1e:	mov    0x408(%rsp),%eax
   b6a25:	mov    %eax,0xb4(%rsp)
   b6a2c:	mov    0x428(%rsp),%eax
   b6a33:	mov    %eax,0xb8(%rsp)
   b6a3a:	mov    0x448(%rsp),%eax
   b6a41:	mov    %eax,0xbc(%rsp)
   b6a48:	mov    0x468(%rsp),%eax
   b6a4f:	mov    %eax,0xc0(%rsp)
   b6a56:	mov    0x488(%rsp),%eax
   b6a5d:	mov    %eax,0xc4(%rsp)
   b6a64:	mov    0x4a8(%rsp),%eax
   b6a6b:	mov    %eax,0xc8(%rsp)
   b6a72:	mov    0x4c8(%rsp),%eax
   b6a79:	mov    %eax,0xcc(%rsp)
   b6a80:	mov    0x4e8(%rsp),%eax
   b6a87:	mov    %eax,0xd0(%rsp)
   b6a8e:	mov    0x508(%rsp),%eax
   b6a95:	mov    %eax,0xd4(%rsp)
   b6a9c:	lea    0x690(%rsp),%rdi
   b6aa4:	mov    $0x2000,%edx
   b6aa9:	xor    %esi,%esi
   b6aab:	call   *0x79de7(%rip)        # 130898 <memset@GLIBC_2.2.5>
   b6ab1:	mov    0x38(%rsp),%r9
   b6ab6:	add    $0x3f,%r9
   b6aba:	mov    $0x20,%r10d
   b6ac0:	mov    0x30(%rsp),%r13
   b6ac5:	mov    %r9,0x38(%rsp)
   b6aca:	mov    %r14,0xf8(%rsp)
   b6ad2:	jmp    b6ae9 <<mibz2::c30::C30<true>>::room+0xb99>
   b6ad4:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   b6ae0:	test   %r15,%r15
   b6ae3:	jne    b6ba5 <<mibz2::c30::C30<true>>::room+0xc55>
   b6ae9:	mov    %r9,%r15
   b6aec:	xor    %ebx,%ebx
   b6aee:	jmp    b6b1d <<mibz2::c30::C30<true>>::room+0xbcd>
   b6af0:	add    %rax,%r12
   b6af3:	mov    %r12,0x118(%rsp,%rbx,1)
   b6afb:	add    %rax,%rdx
   b6afe:	mov    %rdx,0x120(%rsp,%rbx,1)
   b6b06:	sub    %r12,%rdx
   b6b09:	add    $0x20,%rbx
   b6b0d:	cmp    %r15,%rdx
   b6b10:	cmovb  %rdx,%r15
   b6b14:	cmp    $0x400,%rbx
   b6b1b:	je     b6ae0 <<mibz2::c30::C30<true>>::room+0xb90>
   b6b1d:	mov    0x118(%rsp,%rbx,1),%r12
   b6b25:	mov    0x120(%rsp,%rbx,1),%rdx
   b6b2d:	cmp    %rdx,%r12
   b6b30:	jne    b6b06 <<mibz2::c30::C30<true>>::room+0xbb6>
   b6b32:	mov    0x70(%rbp),%rsi
   b6b36:	mov    0x110(%rsp,%rbx,1),%rdi
   b6b3e:	cmp    %rsi,%rdi
   b6b41:	jae    b7be2 <<mibz2::c30::C30<true>>::room+0x1c92>
   b6b47:	mov    0x68(%rbp),%rcx
   b6b4b:	lea    (%rdi,%rdi,2),%rsi
   b6b4f:	mov    (%rcx,%rsi,8),%rdx
   b6b53:	mov    0x8(%rcx,%rsi,8),%rax
   b6b58:	sub    %rax,%r12
   b6b5b:	mov    %r12,0x10(%rcx,%rsi,8)
   b6b60:	mov    %rdx,%rdi
   b6b63:	sub    %r12,%rdi
   b6b66:	cmp    %rdi,%rdx
   b6b69:	jbe    b6af0 <<mibz2::c30::C30<true>>::room+0xba0>
   b6b6b:	lea    (%rcx,%rsi,8),%rdi
   b6b6f:	mov    %rdi,0x8(%rsp)
   b6b74:	mov    $0x1,%ecx
   b6b79:	mov    $0x1,%r8d
   b6b7f:	mov    %r12,%rsi
   b6b82:	mov    %r10,(%rsp)
   b6b86:	call   98dd0 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   b6b8b:	mov    (%rsp),%r10
   b6b8f:	mov    0x38(%rsp),%r9
   b6b94:	mov    0x8(%rsp),%rax
   b6b99:	mov    (%rax),%rdx
   b6b9c:	mov    0x8(%rax),%rax
   b6ba0:	jmp    b6af0 <<mibz2::c30::C30<true>>::room+0xba0>
   b6ba5:	xor    %ecx,%ecx
   b6ba7:	mov    %r10,(%rsp)
   b6bab:	mov    %r15,%rdx
   b6bae:	sub    %rcx,%rdx
   b6bb1:	cmp    $0x100,%rdx
   b6bb8:	mov    $0x100,%eax
   b6bbd:	cmovae %rax,%rdx
   b6bc1:	xorps  %xmm0,%xmm0
   b6bc4:	movaps %xmm0,0x680(%rsp)
   b6bcc:	movaps %xmm0,0x670(%rsp)
   b6bd4:	movaps %xmm0,0x660(%rsp)
   b6bdc:	movaps %xmm0,0x650(%rsp)
   b6be4:	movaps %xmm0,0x640(%rsp)
   b6bec:	movaps %xmm0,0x630(%rsp)
   b6bf4:	movaps %xmm0,0x620(%rsp)
   b6bfc:	movaps %xmm0,0x610(%rsp)
   b6c04:	movaps %xmm0,0x600(%rsp)
   b6c0c:	movaps %xmm0,0x5f0(%rsp)
   b6c14:	movaps %xmm0,0x5e0(%rsp)
   b6c1c:	movaps %xmm0,0x5d0(%rsp)
   b6c24:	movaps %xmm0,0x5c0(%rsp)
   b6c2c:	movaps %xmm0,0x5b0(%rsp)
   b6c34:	movaps %xmm0,0x5a0(%rsp)
   b6c3c:	movaps %xmm0,0x590(%rsp)
   b6c44:	cmp    %rcx,%r15
   b6c47:	mov    %rcx,0x100(%rsp)
   b6c4f:	mov    %rdx,0x28(%rsp)
   b6c54:	jne    b6d30 <<mibz2::c30::C30<true>>::room+0xde0>
   b6c5a:	lea    0x598(%rsp),%rbx
   b6c62:	lea    0x690(%rsp),%r13
   b6c6a:	xor    %ebp,%ebp
   b6c6c:	nopl   0x0(%rax)
   b6c70:	mov    -0x8(%rbx),%rsi
   b6c74:	mov    0x28(%rsp),%r14
   b6c79:	mov    %r14,%r12
   b6c7c:	sub    %rsi,%r12
   b6c7f:	add    %r13,%rsi
   b6c82:	mov    0x118(%rsp,%rbp,1),%rdi
   b6c8a:	mov    %r12,%rdx
   b6c8d:	call   *0x79b9d(%rip)        # 130830 <memcpy@GLIBC_2.14>
   b6c93:	add    %r12,0x118(%rsp,%rbp,1)
   b6c9b:	lea    0x40(%rbp),%rax
   b6c9f:	mov    %rax,0x8(%rsp)
   b6ca4:	mov    (%rbx),%rax
   b6ca7:	sub    %rax,%r14
   b6caa:	lea    (%rax,%r13,1),%rsi
   b6cae:	add    $0x100,%rsi
   b6cb5:	mov    0x138(%rsp,%rbp,1),%rdi
   b6cbd:	mov    %r14,%rdx
   b6cc0:	call   *0x79b6a(%rip)        # 130830 <memcpy@GLIBC_2.14>
   b6cc6:	add    %r14,0x138(%rsp,%rbp,1)
   b6cce:	add    $0x200,%r13
   b6cd5:	add    $0x10,%rbx
   b6cd9:	mov    0x8(%rsp),%rax
   b6cde:	mov    %rax,%rbp
   b6ce1:	cmp    $0x400,%rax
   b6ce7:	jne    b6c70 <<mibz2::c30::C30<true>>::room+0xd20>
   b6ce9:	mov    0x100(%rsp),%rcx
   b6cf1:	add    0x28(%rsp),%rcx
   b6cf6:	cmp    %r15,%rcx
   b6cf9:	mov    0xe0(%rsp),%rbp
   b6d01:	mov    0x30(%rsp),%r13
   b6d06:	mov    0x38(%rsp),%r9
   b6d0b:	mov    (%rsp),%r10
   b6d0f:	mov    0xf8(%rsp),%r14
   b6d17:	jb     b6ba7 <<mibz2::c30::C30<true>>::room+0xc57>
   b6d1d:	jmp    b6ae9 <<mibz2::c30::C30<true>>::room+0xb99>
   b6d22:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   b6d30:	lea    0xd90(%rsp),%rdx
   b6d38:	xor    %r12d,%r12d
   b6d3b:	jmp    b6d5b <<mibz2::c30::C30<true>>::room+0xe0b>
   b6d3d:	nopl   (%rax)
   b6d40:	mov    0x108(%rsp),%rdx
   b6d48:	inc    %rdx
   b6d4b:	cmp    0x28(%rsp),%r12
   b6d50:	mov    0x30(%rsp),%r13
   b6d55:	je     b6c5a <<mibz2::c30::C30<true>>::room+0xd0a>
   b6d5b:	mov    %r12,0x20(%rsp)
   b6d60:	lea    (%rsp,%r12,1),%rax
   b6d64:	add    $0x690,%rax
   b6d6a:	mov    %rdx,0x108(%rsp)
   b6d72:	xor    %ecx,%ecx
   b6d74:	xor    %ebx,%ebx
   b6d76:	mov    0x10(%rsp),%r11
   b6d7b:	nopl   0x0(%rax,%rax,1)
   b6d80:	mov    0x58(%rsp,%rcx,4),%edi
   b6d84:	mov    %edi,%esi
   b6d86:	shr    $0x5,%esi
   b6d89:	lea    0x0(,%rsi,4),%r8
   b6d91:	lea    (%r11,%rdi,2),%r9
   b6d95:	sub    %r8,%r9
   b6d98:	movzwl (%r9),%r8d
   b6d9c:	mov    (%r14,%rsi,4),%esi
   b6da0:	cmp    $0xfffe,%r8d
   b6da7:	jae    b7066 <<mibz2::c30::C30<true>>::room+0x1116>
   b6dad:	mov    %sil,-0x700(%rdx)
   b6db4:	shr    $0x8,%esi
   b6db7:	add    %r8d,%esi
   b6dba:	mov    %esi,0x58(%rsp,%rcx,4)
   b6dbe:	lea    (%rsi,%rsi,1),%rdi
   b6dc2:	shr    $0x5,%esi
   b6dc5:	lea    0x0(,%rsi,4),%r8
   b6dcd:	sub    %r8,%rdi
   b6dd0:	prefetcht0 (%r11,%rdi,1)
   b6dd5:	prefetcht0 (%r14,%rsi,4)
   b6dda:	mov    0x5c(%rsp,%rcx,4),%edi
   b6dde:	mov    %edi,%esi
   b6de0:	shr    $0x5,%esi
   b6de3:	lea    0x0(,%rsi,4),%r8
   b6deb:	lea    (%r11,%rdi,2),%r9
   b6def:	sub    %r8,%r9
   b6df2:	movzwl (%r9),%r8d
   b6df6:	mov    (%r14,%rsi,4),%esi
   b6dfa:	cmp    $0xfffe,%r8d
   b6e01:	jae    b7071 <<mibz2::c30::C30<true>>::room+0x1121>
   b6e07:	mov    %sil,-0x600(%rdx)
   b6e0e:	shr    $0x8,%esi
   b6e11:	add    %r8d,%esi
   b6e14:	mov    %esi,0x5c(%rsp,%rcx,4)
   b6e18:	lea    (%rsi,%rsi,1),%rdi
   b6e1c:	shr    $0x5,%esi
   b6e1f:	lea    0x0(,%rsi,4),%r8
   b6e27:	sub    %r8,%rdi
   b6e2a:	prefetcht0 (%r11,%rdi,1)
   b6e2f:	prefetcht0 (%r14,%rsi,4)
   b6e34:	mov    0x60(%rsp,%rcx,4),%edi
   b6e38:	mov    %edi,%esi
   b6e3a:	shr    $0x5,%esi
   b6e3d:	lea    0x0(,%rsi,4),%r8
   b6e45:	lea    (%r11,%rdi,2),%r9
   b6e49:	sub    %r8,%r9
   b6e4c:	movzwl (%r9),%r8d
   b6e50:	mov    (%r14,%rsi,4),%esi
   b6e54:	cmp    $0xfffe,%r8d
   b6e5b:	jae    b7081 <<mibz2::c30::C30<true>>::room+0x1131>
   b6e61:	mov    %sil,-0x500(%rdx)
   b6e68:	shr    $0x8,%esi
   b6e6b:	add    %r8d,%esi
   b6e6e:	mov    %esi,0x60(%rsp,%rcx,4)
   b6e72:	lea    (%rsi,%rsi,1),%rdi
   b6e76:	shr    $0x5,%esi
   b6e79:	lea    0x0(,%rsi,4),%r8
   b6e81:	sub    %r8,%rdi
   b6e84:	prefetcht0 (%r11,%rdi,1)
   b6e89:	prefetcht0 (%r14,%rsi,4)
   b6e8e:	mov    0x64(%rsp,%rcx,4),%edi
   b6e92:	mov    %edi,%esi
   b6e94:	shr    $0x5,%esi
   b6e97:	lea    0x0(,%rsi,4),%r8
   b6e9f:	lea    (%r11,%rdi,2),%r9
   b6ea3:	sub    %r8,%r9
   b6ea6:	movzwl (%r9),%r8d
   b6eaa:	mov    (%r14,%rsi,4),%esi
   b6eae:	cmp    $0xfffe,%r8d
   b6eb5:	jae    b7091 <<mibz2::c30::C30<true>>::room+0x1141>
   b6ebb:	mov    %sil,-0x400(%rdx)
   b6ec2:	shr    $0x8,%esi
   b6ec5:	add    %r8d,%esi
   b6ec8:	mov    %esi,0x64(%rsp,%rcx,4)
   b6ecc:	lea    (%rsi,%rsi,1),%rdi
   b6ed0:	shr    $0x5,%esi
   b6ed3:	lea    0x0(,%rsi,4),%r8
   b6edb:	sub    %r8,%rdi
   b6ede:	prefetcht0 (%r11,%rdi,1)
   b6ee3:	prefetcht0 (%r14,%rsi,4)
   b6ee8:	mov    0x68(%rsp,%rcx,4),%edi
   b6eec:	mov    %edi,%esi
   b6eee:	shr    $0x5,%esi
   b6ef1:	lea    0x0(,%rsi,4),%r8
   b6ef9:	lea    (%r11,%rdi,2),%r9
   b6efd:	sub    %r8,%r9
   b6f00:	movzwl (%r9),%r8d
   b6f04:	mov    (%r14,%rsi,4),%esi
   b6f08:	cmp    $0xfffe,%r8d
   b6f0f:	jae    b70a1 <<mibz2::c30::C30<true>>::room+0x1151>
   b6f15:	mov    %sil,-0x300(%rdx)
   b6f1c:	shr    $0x8,%esi
   b6f1f:	add    %r8d,%esi
   b6f22:	mov    %esi,0x68(%rsp,%rcx,4)
   b6f26:	lea    (%rsi,%rsi,1),%rdi
   b6f2a:	shr    $0x5,%esi
   b6f2d:	lea    0x0(,%rsi,4),%r8
   b6f35:	sub    %r8,%rdi
   b6f38:	prefetcht0 (%r11,%rdi,1)
   b6f3d:	prefetcht0 (%r14,%rsi,4)
   b6f42:	mov    0x6c(%rsp,%rcx,4),%edi
   b6f46:	mov    %edi,%esi
   b6f48:	shr    $0x5,%esi
   b6f4b:	lea    0x0(,%rsi,4),%r8
   b6f53:	lea    (%r11,%rdi,2),%r9
   b6f57:	sub    %r8,%r9
   b6f5a:	movzwl (%r9),%r8d
   b6f5e:	mov    (%r14,%rsi,4),%esi
   b6f62:	cmp    $0xfffe,%r8d
   b6f69:	jae    b70b1 <<mibz2::c30::C30<true>>::room+0x1161>
   b6f6f:	mov    %sil,-0x200(%rdx)
   b6f76:	shr    $0x8,%esi
   b6f79:	add    %r8d,%esi
   b6f7c:	mov    %esi,0x6c(%rsp,%rcx,4)
   b6f80:	lea    (%rsi,%rsi,1),%rdi
   b6f84:	shr    $0x5,%esi
   b6f87:	lea    0x0(,%rsi,4),%r8
   b6f8f:	sub    %r8,%rdi
   b6f92:	prefetcht0 (%r11,%rdi,1)
   b6f97:	prefetcht0 (%r14,%rsi,4)
   b6f9c:	mov    0x70(%rsp,%rcx,4),%edi
   b6fa0:	mov    %edi,%esi
   b6fa2:	shr    $0x5,%esi
   b6fa5:	lea    0x0(,%rsi,4),%r8
   b6fad:	lea    (%r11,%rdi,2),%r9
   b6fb1:	sub    %r8,%r9
   b6fb4:	movzwl (%r9),%r8d
   b6fb8:	mov    (%r14,%rsi,4),%esi
   b6fbc:	cmp    $0xfffe,%r8d
   b6fc3:	jae    b70c1 <<mibz2::c30::C30<true>>::room+0x1171>
   b6fc9:	mov    %sil,-0x100(%rdx)
   b6fd0:	shr    $0x8,%esi
   b6fd3:	add    %r8d,%esi
   b6fd6:	mov    %esi,0x70(%rsp,%rcx,4)
   b6fda:	lea    (%rsi,%rsi,1),%rdi
   b6fde:	shr    $0x5,%esi
   b6fe1:	lea    0x0(,%rsi,4),%r8
   b6fe9:	sub    %r8,%rdi
   b6fec:	prefetcht0 (%r11,%rdi,1)
   b6ff1:	prefetcht0 (%r14,%rsi,4)
   b6ff6:	mov    0x74(%rsp,%rcx,4),%edi
   b6ffa:	mov    %edi,%esi
   b6ffc:	shr    $0x5,%esi
   b6fff:	lea    0x0(,%rsi,4),%r8
   b7007:	lea    (%r11,%rdi,2),%r9
   b700b:	sub    %r8,%r9
   b700e:	movzwl (%r9),%r8d
   b7012:	mov    (%r14,%rsi,4),%esi
   b7016:	cmp    $0xfffe,%r8d
   b701d:	jae    b70d1 <<mibz2::c30::C30<true>>::room+0x1181>
   b7023:	mov    %sil,(%rdx)
   b7026:	shr    $0x8,%esi
   b7029:	add    %r8d,%esi
   b702c:	mov    %esi,0x74(%rsp,%rcx,4)
   b7030:	lea    (%rsi,%rsi,1),%rdi
   b7034:	shr    $0x5,%esi
   b7037:	lea    0x0(,%rsi,4),%r8
   b703f:	sub    %r8,%rdi
   b7042:	prefetcht0 (%r11,%rdi,1)
   b7047:	prefetcht0 (%r14,%rsi,4)
   b704c:	add    $0x8,%rcx
   b7050:	add    $0x800,%rdx
   b7057:	cmp    $0x19,%rcx
   b705b:	jb     b6d80 <<mibz2::c30::C30<true>>::room+0xe30>
   b7061:	jmp    b7270 <<mibz2::c30::C30<true>>::room+0x1320>
   b7066:	jne    b70e1 <<mibz2::c30::C30<true>>::room+0x1191>
   b7068:	bts    %rcx,%rbx
   b706c:	jmp    b6dda <<mibz2::c30::C30<true>>::room+0xe8a>
   b7071:	jne    b7111 <<mibz2::c30::C30<true>>::room+0x11c1>
   b7077:	mov    $0x2,%esi
   b707c:	jmp    b7136 <<mibz2::c30::C30<true>>::room+0x11e6>
   b7081:	jne    b7141 <<mibz2::c30::C30<true>>::room+0x11f1>
   b7087:	mov    $0x4,%esi
   b708c:	jmp    b7166 <<mibz2::c30::C30<true>>::room+0x1216>
   b7091:	jne    b7171 <<mibz2::c30::C30<true>>::room+0x1221>
   b7097:	mov    $0x8,%esi
   b709c:	jmp    b7196 <<mibz2::c30::C30<true>>::room+0x1246>
   b70a1:	jne    b71a1 <<mibz2::c30::C30<true>>::room+0x1251>
   b70a7:	mov    $0x10,%esi
   b70ac:	jmp    b71c6 <<mibz2::c30::C30<true>>::room+0x1276>
   b70b1:	jne    b71d1 <<mibz2::c30::C30<true>>::room+0x1281>
   b70b7:	mov    $0x20,%esi
   b70bc:	jmp    b71f6 <<mibz2::c30::C30<true>>::room+0x12a6>
   b70c1:	jne    b7201 <<mibz2::c30::C30<true>>::room+0x12b1>
   b70c7:	mov    $0x40,%esi
   b70cc:	jmp    b7226 <<mibz2::c30::C30<true>>::room+0x12d6>
   b70d1:	jne    b7231 <<mibz2::c30::C30<true>>::room+0x12e1>
   b70d7:	mov    $0x80,%esi
   b70dc:	jmp    b7256 <<mibz2::c30::C30<true>>::room+0x1306>
   b70e1:	shl    $0x5,%esi
   b70e4:	and    $0x1ffffe0,%esi
   b70ea:	and    $0x1f,%edi
   b70ed:	or     %esi,%edi
   b70ef:	mov    %edi,0x510(%rsp,%rcx,4)
   b70f6:	prefetcht0 0x0(%r13,%rdi,4)
   b70fc:	movabs $0x100000000,%rsi
   b7106:	shl    %cl,%rsi
   b7109:	or     %rsi,%rbx
   b710c:	jmp    b6dda <<mibz2::c30::C30<true>>::room+0xe8a>
   b7111:	shl    $0x5,%esi
   b7114:	and    $0x1ffffe0,%esi
   b711a:	and    $0x1f,%edi
   b711d:	or     %esi,%edi
   b711f:	mov    %edi,0x514(%rsp,%rcx,4)
   b7126:	prefetcht0 0x0(%r13,%rdi,4)
   b712c:	movabs $0x200000000,%rsi
   b7136:	shl    %cl,%rsi
   b7139:	or     %rsi,%rbx
   b713c:	jmp    b6e34 <<mibz2::c30::C30<true>>::room+0xee4>
   b7141:	shl    $0x5,%esi
   b7144:	and    $0x1ffffe0,%esi
   b714a:	and    $0x1f,%edi
   b714d:	or     %esi,%edi
   b714f:	mov    %edi,0x518(%rsp,%rcx,4)
   b7156:	prefetcht0 0x0(%r13,%rdi,4)
   b715c:	movabs $0x400000000,%rsi
   b7166:	shl    %cl,%rsi
   b7169:	or     %rsi,%rbx
   b716c:	jmp    b6e8e <<mibz2::c30::C30<true>>::room+0xf3e>
   b7171:	shl    $0x5,%esi
   b7174:	and    $0x1ffffe0,%esi
   b717a:	and    $0x1f,%edi
   b717d:	or     %esi,%edi
   b717f:	mov    %edi,0x51c(%rsp,%rcx,4)
   b7186:	prefetcht0 0x0(%r13,%rdi,4)
   b718c:	movabs $0x800000000,%rsi
   b7196:	shl    %cl,%rsi
   b7199:	or     %rsi,%rbx
   b719c:	jmp    b6ee8 <<mibz2::c30::C30<true>>::room+0xf98>
   b71a1:	shl    $0x5,%esi
   b71a4:	and    $0x1ffffe0,%esi
   b71aa:	and    $0x1f,%edi
   b71ad:	or     %esi,%edi
   b71af:	mov    %edi,0x520(%rsp,%rcx,4)
   b71b6:	prefetcht0 0x0(%r13,%rdi,4)
   b71bc:	movabs $0x1000000000,%rsi
   b71c6:	shl    %cl,%rsi
   b71c9:	or     %rsi,%rbx
   b71cc:	jmp    b6f42 <<mibz2::c30::C30<true>>::room+0xff2>
   b71d1:	shl    $0x5,%esi
   b71d4:	and    $0x1ffffe0,%esi
   b71da:	and    $0x1f,%edi
   b71dd:	or     %esi,%edi
   b71df:	mov    %edi,0x524(%rsp,%rcx,4)
   b71e6:	prefetcht0 0x0(%r13,%rdi,4)
   b71ec:	movabs $0x2000000000,%rsi
   b71f6:	shl    %cl,%rsi
   b71f9:	or     %rsi,%rbx
   b71fc:	jmp    b6f9c <<mibz2::c30::C30<true>>::room+0x104c>
   b7201:	shl    $0x5,%esi
   b7204:	and    $0x1ffffe0,%esi
   b720a:	and    $0x1f,%edi
   b720d:	or     %esi,%edi
   b720f:	mov    %edi,0x528(%rsp,%rcx,4)
   b7216:	prefetcht0 0x0(%r13,%rdi,4)
   b721c:	movabs $0x4000000000,%rsi
   b7226:	shl    %cl,%rsi
   b7229:	or     %rsi,%rbx
   b722c:	jmp    b6ff6 <<mibz2::c30::C30<true>>::room+0x10a6>
   b7231:	shl    $0x5,%esi
   b7234:	and    $0x1ffffe0,%esi
   b723a:	and    $0x1f,%edi
   b723d:	or     %esi,%edi
   b723f:	mov    %edi,0x52c(%rsp,%rcx,4)
   b7246:	prefetcht0 0x0(%r13,%rdi,4)
   b724c:	movabs $0x8000000000,%rsi
   b7256:	shl    %cl,%rsi
   b7259:	or     %rsi,%rbx
   b725c:	jmp    b704c <<mibz2::c30::C30<true>>::room+0x10fc>
   b7261:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   b7270:	mov    0x20(%rsp),%rcx
   b7275:	lea    0x1(%rcx),%r12
   b7279:	test   %rbx,%rbx
   b727c:	je     b6d40 <<mibz2::c30::C30<true>>::room+0xdf0>
   b7282:	mov    %rbx,%rcx
   b7285:	shr    $0x20,%rcx
   b7289:	je     b72e2 <<mibz2::c30::C30<true>>::room+0x1392>
   b728b:	nopl   0x0(%rax,%rax,1)
   b7290:	tzcnt  %ecx,%esi
   b7294:	lea    -0x1(%rcx),%edx
   b7297:	lea    0x0(,%rsi,4),%edi
   b729e:	mov    0x510(%rsp,%rdi,1),%r8d
   b72a6:	mov    0x0(%r13,%r8,4),%r8d
   b72ab:	shl    $0x8,%esi
   b72ae:	mov    %r8d,%r9d
   b72b1:	mov    %r8b,(%rax,%rsi,1)
   b72b5:	mov    %r8d,%esi
   b72b8:	shr    $0x8,%esi
   b72bb:	lea    (%rsi,%rsi,1),%r8
   b72bf:	shr    $0xd,%r9d
   b72c3:	lea    0x0(,%r9,4),%r10
   b72cb:	sub    %r10,%r8
   b72ce:	mov    %esi,0x58(%rsp,%rdi,1)
   b72d2:	prefetcht0 (%r11,%r8,1)
   b72d7:	prefetcht0 (%r14,%r9,4)
   b72dc:	and    %ecx,%edx
   b72de:	mov    %edx,%ecx
   b72e0:	jne    b7290 <<mibz2::c30::C30<true>>::room+0x1340>
   b72e2:	mov    $0xffffffff,%eax
   b72e7:	and    %rax,%rbx
   b72ea:	mov    %r12,0x48(%rsp)
   b72ef:	je     b6d40 <<mibz2::c30::C30<true>>::room+0xdf0>
   b72f5:	data16 cs nopw 0x0(%rax,%rax,1)
   b7300:	mov    %rbx,0x8(%rsp)
   b7305:	tzcnt  %rbx,%r13
   b730a:	mov    %r13d,%ebx
   b730d:	shl    $0x5,%ebx
   b7310:	mov    0x58(%rsp,%r13,4),%eax
   b7315:	mov    %eax,0x128(%rsp,%rbx,1)
   b731c:	mov    %r13d,%eax
   b731f:	shl    $0x8,%eax
   b7322:	lea    (%rsp,%rax,1),%rsi
   b7326:	add    $0x690,%rsi
   b732d:	mov    0x590(%rsp,%r13,8),%rax
   b7335:	mov    0x20(%rsp),%r12
   b733a:	sub    %rax,%r12
   b733d:	add    %rax,%rsi
   b7340:	mov    0x118(%rsp,%rbx,1),%rdi
   b7348:	mov    %r12,%rdx
   b734b:	call   *0x794df(%rip)        # 130830 <memcpy@GLIBC_2.14>
   b7351:	add    0x118(%rsp,%rbx,1),%r12
   b7359:	mov    %r12,0x118(%rsp,%rbx,1)
   b7361:	mov    0x70(%rbp),%rsi
   b7365:	mov    0x110(%rsp,%rbx,1),%rdi
   b736d:	cmp    %rsi,%rdi
   b7370:	jae    b7c3a <<mibz2::c30::C30<true>>::room+0x1cea>
   b7376:	lea    (%rsp,%rbx,1),%rax
   b737a:	add    $0x110,%rax
   b7380:	mov    0x68(%rbp),%rdx
   b7384:	mov    0x80(%rbp),%rcx
   b738b:	lea    (%rdi,%rdi,2),%r8
   b738f:	sub    0x8(%rdx,%r8,8),%r12
   b7394:	mov    0x88(%rbp),%rsi
   b739b:	mov    %r12,0x10(%rdx,%r8,8)
   b73a0:	mov    0x40(%rbp),%r10
   b73a4:	test   %r10,%r10
   b73a7:	je     b73c0 <<mibz2::c30::C30<true>>::room+0x1470>
   b73a9:	mov    0x18(%rax),%r8d
   b73ad:	mov    0x38(%rbp),%r9
   b73b1:	xor    %edx,%edx
   b73b3:	cmp    $0x1,%r10
   b73b7:	jne    b73d0 <<mibz2::c30::C30<true>>::room+0x1480>
   b73b9:	mov    0x18(%rsp),%r12
   b73be:	jmp    b73fd <<mibz2::c30::C30<true>>::room+0x14ad>
   b73c0:	mov    $0xffffffffffffffff,%rdx
   b73c7:	mov    0x18(%rsp),%r12
   b73cc:	jmp    b740c <<mibz2::c30::C30<true>>::room+0x14bc>
   b73ce:	xchg   %ax,%ax
   b73d0:	mov    0x18(%rsp),%r12
   b73d5:	data16 cs nopw 0x0(%rax,%rax,1)
   b73e0:	mov    %rdx,%r11
   b73e3:	mov    %r10,%rbx
   b73e6:	shr    $1,%rbx
   b73e9:	add    %rbx,%rdx
   b73ec:	cmp    %r8d,(%r9,%rdx,4)
   b73f0:	cmova  %r11,%rdx
   b73f4:	sub    %rbx,%r10
   b73f7:	cmp    $0x1,%r10
   b73fb:	ja     b73e0 <<mibz2::c30::C30<true>>::room+0x1490>
   b73fd:	cmp    %r8d,(%r9,%rdx,4)
   b7401:	mov    $0xffffffffffffffff,%r8
   b7408:	cmovne %r8,%rdx
   b740c:	cmp    %rsi,%rdi
   b740f:	mov    (%rsp),%r10
   b7413:	mov    0x8(%rsp),%rbx
   b7418:	jae    b7c2e <<mibz2::c30::C30<true>>::room+0x1cde>
   b741e:	lea    -0x1(%rbx),%rsi
   b7422:	and    %rsi,%rbx
   b7425:	mov    %rdx,(%rcx,%rdi,8)
   b7429:	cmp    %r12,%r10
   b742c:	je     b74db <<mibz2::c30::C30<true>>::room+0x158b>
   b7432:	mov    0x70(%rbp),%rsi
   b7436:	cmp    %rsi,%r10
   b7439:	jae    b7c46 <<mibz2::c30::C30<true>>::room+0x1cf6>
   b743f:	mov    0xc8(%rbp),%rsi
   b7446:	cmp    %rsi,%r10
   b7449:	mov    0x48(%rsp),%r12
   b744e:	jae    b7c55 <<mibz2::c30::C30<true>>::room+0x1d05>
   b7454:	mov    0x68(%rbp),%rcx
   b7458:	mov    0xc0(%rbp),%rdx
   b745f:	lea    (%r10,%r10,2),%rsi
   b7463:	mov    0x8(%rcx,%rsi,8),%rdi
   b7468:	movzbl (%rdx,%r10,4),%r8d
   b746d:	mov    %r8b,(%rdi)
   b7470:	mov    (%rdx,%r10,4),%edx
   b7474:	mov    %edx,%r8d
   b7477:	lea    0x1(%rdi),%r9
   b747b:	add    (%rcx,%rsi,8),%rdi
   b747f:	shr    $0x8,%r8d
   b7483:	mov    %r10,(%rax)
   b7486:	mov    %r9,0x8(%rax)
   b748a:	mov    %rdi,0x10(%rax)
   b748e:	mov    %r8d,0x18(%rax)
   b7492:	movl   $0x0,0x1c(%rax)
   b7499:	mov    %r8d,0x58(%rsp,%r13,4)
   b749e:	add    %r8,%r8
   b74a1:	shr    $0xd,%edx
   b74a4:	lea    0x0(,%rdx,4),%rax
   b74ac:	sub    %rax,%r8
   b74af:	mov    0x10(%rsp),%rax
   b74b4:	prefetcht0 (%rax,%r8,1)
   b74b9:	prefetcht0 (%r14,%rdx,4)
   b74be:	inc    %r10
   b74c1:	mov    %r10,(%rsp)
   b74c5:	mov    %r12,0x590(%rsp,%r13,8)
   b74cd:	test   %rbx,%rbx
   b74d0:	jne    b7300 <<mibz2::c30::C30<true>>::room+0x13b0>
   b74d6:	jmp    b6d40 <<mibz2::c30::C30<true>>::room+0xdf0>
   b74db:	not    %rbx
   b74de:	mov    %rbx,0x8(%rsp)
   b74e3:	xor    %ebx,%ebx
   b74e5:	xor    %r12d,%r12d
   b74e8:	jmp    b7500 <<mibz2::c30::C30<true>>::room+0x15b0>
   b74ea:	nopw   0x0(%rax,%rax,1)
   b74f0:	add    $0x20,%r12
   b74f4:	inc    %rbx
   b74f7:	cmp    $0x400,%r12
   b74fe:	je     b7556 <<mibz2::c30::C30<true>>::room+0x1606>
   b7500:	mov    0x58(%rsp,%rbx,4),%eax
   b7504:	mov    %eax,0x128(%rsp,%r12,1)
   b750c:	cmp    %rbx,%r13
   b750f:	je     b74f0 <<mibz2::c30::C30<true>>::room+0x15a0>
   b7511:	mov    0x590(%rsp,%rbx,8),%rax
   b7519:	mov    0x8(%rsp),%rcx
   b751e:	bt     %rbx,%rcx
   b7522:	mov    0x20(%rsp),%r15
   b7527:	adc    $0x0,%r15
   b752b:	sub    %rax,%r15
   b752e:	add    %rsp,%rax
   b7531:	add    $0x690,%rax
   b7537:	lea    (%rax,%r12,8),%rsi
   b753b:	mov    0x118(%rsp,%r12,1),%rdi
   b7543:	mov    %r15,%rdx
   b7546:	call   *0x792e4(%rip)        # 130830 <memcpy@GLIBC_2.14>
   b754c:	add    %r15,0x118(%rsp,%r12,1)
   b7554:	jmp    b74f0 <<mibz2::c30::C30<true>>::room+0x15a0>
   b7556:	mov    0xf0(%rsp),%r10
   b755e:	mov    0xe8(%rsp),%rcx
   b7566:	shl    $0x5,%rcx
   b756a:	lea    (%rsp,%rcx,1),%rax
   b756e:	add    $0x110,%rax
   b7574:	mov    %rax,0x8(%rsp)
   b7579:	mov    0x80(%rbp),%rax
   b7580:	mov    0x88(%rbp),%rsi
   b7587:	lea    0x110(%rsp),%r12
   b758f:	jmp    b75ad <<mibz2::c30::C30<true>>::room+0x165d>
   b7591:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   b75a0:	mov    %rbx,%r12
   b75a3:	add    $0xffffffffffffffe0,%r10
   b75a7:	je     b76fd <<mibz2::c30::C30<true>>::room+0x17ad>
   b75ad:	lea    0x20(%r12),%rbx
   b75b2:	mov    (%r12),%r15
   b75b6:	cmp    $0xffffffffffffffff,%r15
   b75ba:	je     b75a0 <<mibz2::c30::C30<true>>::room+0x1650>
   b75bc:	cmp    %rsi,%r15
   b75bf:	jae    b7c8a <<mibz2::c30::C30<true>>::room+0x1d3a>
   b75c5:	cmpq   $0xffffffffffffffff,(%rax,%r15,8)
   b75ca:	jne    b75a0 <<mibz2::c30::C30<true>>::room+0x1650>
   b75cc:	mov    $0x80,%edi
   b75d1:	call   *0x79271(%rip)        # 130848 <malloc@GLIBC_2.2.5>
   b75d7:	test   %rax,%rax
   b75da:	je     b7cee <<mibz2::c30::C30<true>>::room+0x1d9e>
   b75e0:	mov    0x18(%r12),%rcx
   b75e5:	mov    %rcx,0x18(%rax)
   b75e9:	movups 0x8(%r12),%xmm0
   b75ef:	movups %xmm0,0x8(%rax)
   b75f3:	mov    %r15,(%rax)
   b75f6:	movq   $0x4,0x590(%rsp)
   b7602:	mov    %rax,0x598(%rsp)
   b760a:	movq   $0x1,0x5a0(%rsp)
   b7616:	cmp    0x8(%rsp),%rbx
   b761b:	je     b770e <<mibz2::c30::C30<true>>::room+0x17be>
   b7621:	mov    $0x1,%r12d
   b7627:	lea    0x590(%rsp),%r15
   b762f:	mov    0x80(%rbp),%rcx
   b7636:	mov    0x88(%rbp),%rsi
   b763d:	mov    %rbx,%rdx
   b7640:	jmp    b765e <<mibz2::c30::C30<true>>::room+0x170e>
   b7642:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   b7650:	mov    %rbx,%rdx
   b7653:	cmp    0x8(%rsp),%rbx
   b7658:	je     b7714 <<mibz2::c30::C30<true>>::room+0x17c4>
   b765e:	lea    0x20(%rdx),%rbx
   b7662:	mov    (%rdx),%r13
   b7665:	cmp    $0xffffffffffffffff,%r13
   b7669:	je     b7650 <<mibz2::c30::C30<true>>::room+0x1700>
   b766b:	cmp    %rsi,%r13
   b766e:	jae    b7c99 <<mibz2::c30::C30<true>>::room+0x1d49>
   b7674:	cmpq   $0xffffffffffffffff,(%rcx,%r13,8)
   b7679:	jne    b7650 <<mibz2::c30::C30<true>>::room+0x1700>
   b767b:	mov    0x18(%rdx),%rcx
   b767f:	mov    %rcx,0x6a0(%rsp)
   b7687:	movups 0x8(%rdx),%xmm0
   b768b:	movaps %xmm0,0x690(%rsp)
   b7693:	cmp    0x590(%rsp),%r12
   b769b:	jne    b76c0 <<mibz2::c30::C30<true>>::room+0x1770>
   b769d:	mov    $0x1,%edx
   b76a2:	mov    $0x8,%ecx
   b76a7:	mov    $0x20,%r8d
   b76ad:	mov    %r15,%rdi
   b76b0:	mov    %r12,%rsi
   b76b3:	call   98dd0 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   b76b8:	mov    0x598(%rsp),%rax
   b76c0:	mov    %r12,%rcx
   b76c3:	shl    $0x5,%rcx
   b76c7:	mov    %r13,(%rax,%rcx,1)
   b76cb:	movaps 0x690(%rsp),%xmm0
   b76d3:	movups %xmm0,0x8(%rax,%rcx,1)
   b76d8:	mov    0x6a0(%rsp),%rdx
   b76e0:	mov    %rdx,0x18(%rax,%rcx,1)
   b76e5:	inc    %r12
   b76e8:	mov    %r12,0x5a0(%rsp)
   b76f0:	cmp    0x8(%rsp),%rbx
   b76f5:	jne    b762f <<mibz2::c30::C30<true>>::room+0x16df>
   b76fb:	jmp    b7714 <<mibz2::c30::C30<true>>::room+0x17c4>
   b76fd:	mov    $0x8,%eax
   b7702:	mov    %rax,(%rsp)
   b7706:	xor    %r14d,%r14d
   b7709:	jmp    b7a4e <<mibz2::c30::C30<true>>::room+0x1afe>
   b770e:	mov    $0x1,%r12d
   b7714:	mov    0x590(%rsp),%rax
   b771c:	mov    %rax,0x20(%rsp)
   b7721:	mov    0x598(%rsp),%rax
   b7729:	mov    %rax,(%rsp)
   b772d:	mov    %r12,%rbx
   b7730:	shl    $0x5,%rbx
   b7734:	xor    %r15d,%r15d
   b7737:	lea    0x763ba(%rip),%rax        # 12daf8 <__frame_dummy_init_array_entry+0x17d8>
   b773e:	mov    %rax,0x28(%rsp)
   b7743:	jmp    b777d <<mibz2::c30::C30<true>>::room+0x182d>
   b7745:	mov    0x8(%rsp),%rax
   b774a:	mov    (%rax),%rdx
   b774d:	mov    0x8(%rax),%rax
   b7751:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   b7760:	add    %rax,%r13
   b7763:	mov    (%rsp),%rcx
   b7767:	mov    %r13,0x8(%rcx,%r15,1)
   b776c:	add    %rax,%rdx
   b776f:	mov    %rdx,0x10(%rcx,%r15,1)
   b7774:	add    $0x20,%r15
   b7778:	cmp    %r15,%rbx
   b777b:	je     b77e7 <<mibz2::c30::C30<true>>::room+0x1897>
   b777d:	mov    (%rsp),%rax
   b7781:	mov    0x8(%rax,%r15,1),%r13
   b7786:	cmp    0x10(%rax,%r15,1),%r13
   b778b:	jne    b7774 <<mibz2::c30::C30<true>>::room+0x1824>
   b778d:	mov    0x70(%rbp),%rsi
   b7791:	mov    (%rsp),%rax
   b7795:	mov    (%rax,%r15,1),%rdi
   b7799:	cmp    %rsi,%rdi
   b779c:	jae    b7c79 <<mibz2::c30::C30<true>>::room+0x1d29>
   b77a2:	mov    0x68(%rbp),%rcx
   b77a6:	lea    (%rdi,%rdi,2),%rsi
   b77aa:	mov    (%rcx,%rsi,8),%rdx
   b77ae:	mov    0x8(%rcx,%rsi,8),%rax
   b77b3:	sub    %rax,%r13
   b77b6:	mov    %r13,0x10(%rcx,%rsi,8)
   b77bb:	mov    %rdx,%rdi
   b77be:	sub    %r13,%rdi
   b77c1:	cmp    %rdi,%rdx
   b77c4:	jbe    b7760 <<mibz2::c30::C30<true>>::room+0x1810>
   b77c6:	lea    (%rcx,%rsi,8),%rdi
   b77ca:	mov    $0x1,%ecx
   b77cf:	mov    $0x1,%r8d
   b77d5:	mov    %rdi,0x8(%rsp)
   b77da:	mov    %r13,%rsi
   b77dd:	call   98dd0 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   b77e2:	jmp    b7745 <<mibz2::c30::C30<true>>::room+0x17f5>
   b77e7:	mov    (%rsp),%rax
   b77eb:	add    $0x18,%rax
   b77ef:	mov    %rax,0x8(%rsp)
   b77f4:	xor    %ebx,%ebx
   b77f6:	mov    0x10(%rsp),%r13
   b77fb:	jmp    b7802 <<mibz2::c30::C30<true>>::room+0x18b2>
   b77fd:	nopl   (%rax)
   b7800:	xor    %ebx,%ebx
   b7802:	mov    %rbx,%r15
   b7805:	shl    $0x5,%r15
   b7809:	add    0x8(%rsp),%r15
   b780e:	jmp    b783f <<mibz2::c30::C30<true>>::room+0x18ef>
   b7810:	mov    0x48(%rsp),%rax
   b7815:	mov    (%rax),%rdx
   b7818:	mov    0x8(%rax),%rax
   b781c:	nopl   0x0(%rax)
   b7820:	add    %rax,%r13
   b7823:	mov    %r13,-0x10(%r15)
   b7827:	add    %rax,%rdx
   b782a:	mov    %rdx,-0x8(%r15)
   b782e:	inc    %rbx
   b7831:	add    $0x20,%r15
   b7835:	cmp    %r12,%rbx
   b7838:	mov    0x10(%rsp),%r13
   b783d:	jae    b7800 <<mibz2::c30::C30<true>>::room+0x18b0>
   b783f:	lea    -0x18(%r15),%rax
   b7843:	mov    (%r15),%ecx
   b7846:	mov    %ecx,%edx
   b7848:	shr    $0x5,%edx
   b784b:	lea    0x0(,%rdx,4),%rsi
   b7853:	lea    0x0(,%rcx,2),%rdi
   b785b:	add    %r13,%rdi
   b785e:	sub    %rsi,%rdi
   b7861:	movzwl (%rdi),%esi
   b7864:	mov    (%r14,%rdx,4),%edx
   b7868:	cmp    $0xfffe,%esi
   b786e:	jae    b7880 <<mibz2::c30::C30<true>>::room+0x1930>
   b7870:	mov    %edx,%ecx
   b7872:	shr    $0x8,%ecx
   b7875:	add    %esi,%ecx
   b7877:	jmp    b78a1 <<mibz2::c30::C30<true>>::room+0x1951>
   b7879:	nopl   0x0(%rax)
   b7880:	je     b7940 <<mibz2::c30::C30<true>>::room+0x19f0>
   b7886:	cmp    0x44(%rsp),%ecx
   b788a:	jne    b79d0 <<mibz2::c30::C30<true>>::room+0x1a80>
   b7890:	mov    0x4(%r15),%ecx
   b7894:	mov    0x30(%rsp),%rdx
   b7899:	mov    (%rdx,%rcx,4),%edx
   b789c:	mov    %edx,%ecx
   b789e:	shr    $0x8,%ecx
   b78a1:	mov    %ecx,(%r15)
   b78a4:	mov    %ecx,%esi
   b78a6:	add    %rsi,%rsi
   b78a9:	shr    $0x5,%ecx
   b78ac:	lea    0x0(,%rcx,4),%rdi
   b78b4:	sub    %rdi,%rsi
   b78b7:	prefetcht0 0x0(%r13,%rsi,1)
   b78bd:	prefetcht0 (%r14,%rcx,4)
   b78c2:	mov    -0x10(%r15),%rcx
   b78c6:	mov    %dl,(%rcx)
   b78c8:	mov    -0x10(%r15),%r13
   b78cc:	inc    %r13
   b78cf:	mov    %r13,-0x10(%r15)
   b78d3:	cmp    -0x8(%r15),%r13
   b78d7:	jne    b782e <<mibz2::c30::C30<true>>::room+0x18de>
   b78dd:	mov    0x70(%rbp),%rsi
   b78e1:	mov    (%rax),%rdi
   b78e4:	cmp    %rsi,%rdi
   b78e7:	jae    b7c79 <<mibz2::c30::C30<true>>::room+0x1d29>
   b78ed:	mov    0x68(%rbp),%rcx
   b78f1:	lea    (%rdi,%rdi,2),%rsi
   b78f5:	mov    (%rcx,%rsi,8),%rdx
   b78f9:	mov    0x8(%rcx,%rsi,8),%rax
   b78fe:	sub    %rax,%r13
   b7901:	mov    %r13,0x10(%rcx,%rsi,8)
   b7906:	mov    %rdx,%rdi
   b7909:	sub    %r13,%rdi
   b790c:	cmp    %rdi,%rdx
   b790f:	jbe    b7820 <<mibz2::c30::C30<true>>::room+0x18d0>
   b7915:	lea    (%rcx,%rsi,8),%rdi
   b7919:	mov    $0x1,%ecx
   b791e:	mov    $0x1,%r8d
   b7924:	mov    %rdi,0x48(%rsp)
   b7929:	mov    %r13,%rsi
   b792c:	call   98dd0 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   b7931:	jmp    b7810 <<mibz2::c30::C30<true>>::room+0x18c0>
   b7936:	cs nopw 0x0(%rax,%rax,1)
   b7940:	mov    0x70(%rbp),%rsi
   b7944:	mov    (%rax),%rdi
   b7947:	cmp    %rsi,%rdi
   b794a:	jae    b7c6d <<mibz2::c30::C30<true>>::room+0x1d1d>
   b7950:	mov    0x68(%rbp),%r8
   b7954:	mov    0x80(%rbp),%rdx
   b795b:	lea    (%rdi,%rdi,2),%r9
   b795f:	mov    0x8(%rax),%r10
   b7963:	sub    0x8(%r8,%r9,8),%r10
   b7968:	mov    0x88(%rbp),%rsi
   b796f:	mov    %r10,0x10(%r8,%r9,8)
   b7974:	mov    0x40(%rbp),%r10
   b7978:	test   %r10,%r10
   b797b:	je     b7a00 <<mibz2::c30::C30<true>>::room+0x1ab0>
   b7981:	mov    0x38(%rbp),%r9
   b7985:	xor    %r8d,%r8d
   b7988:	cmp    $0x1,%r10
   b798c:	je     b79ad <<mibz2::c30::C30<true>>::room+0x1a5d>
   b798e:	xchg   %ax,%ax
   b7990:	mov    %r8,%r11
   b7993:	mov    %r10,%r15
   b7996:	shr    $1,%r15
   b7999:	add    %r15,%r8
   b799c:	cmp    %ecx,(%r9,%r8,4)
   b79a0:	cmova  %r11,%r8
   b79a4:	sub    %r15,%r10
   b79a7:	cmp    $0x1,%r10
   b79ab:	ja     b7990 <<mibz2::c30::C30<true>>::room+0x1a40>
   b79ad:	cmp    %ecx,(%r9,%r8,4)
   b79b1:	mov    $0xffffffffffffffff,%rcx
   b79b8:	cmovne %rcx,%r8
   b79bc:	cmp    %rsi,%rdi
   b79bf:	jb     b7a10 <<mibz2::c30::C30<true>>::room+0x1ac0>
   b79c1:	jmp    b7c64 <<mibz2::c30::C30<true>>::room+0x1d14>
   b79c6:	cs nopw 0x0(%rax,%rax,1)
   b79d0:	shl    $0x5,%edx
   b79d3:	and    $0x1ffffe0,%edx
   b79d9:	and    $0x1f,%ecx
   b79dc:	or     %edx,%ecx
   b79de:	mov    %ecx,0x4(%r15)
   b79e2:	mov    0x30(%rsp),%rax
   b79e7:	prefetcht0 (%rax,%rcx,4)
   b79eb:	mov    0x44(%rsp),%eax
   b79ef:	mov    %eax,(%r15)
   b79f2:	inc    %rbx
   b79f5:	cmp    %r12,%rbx
   b79f8:	jb     b7802 <<mibz2::c30::C30<true>>::room+0x18b2>
   b79fe:	jmp    b7a3b <<mibz2::c30::C30<true>>::room+0x1aeb>
   b7a00:	mov    $0xffffffffffffffff,%r8
   b7a07:	cmp    %rsi,%rdi
   b7a0a:	jae    b7c64 <<mibz2::c30::C30<true>>::room+0x1d14>
   b7a10:	mov    %r8,(%rdx,%rdi,8)
   b7a14:	dec    %r12
   b7a17:	mov    %r12,%rcx
   b7a1a:	shl    $0x5,%rcx
   b7a1e:	mov    (%rsp),%rdx
   b7a22:	movups (%rdx,%rcx,1),%xmm0
   b7a26:	movups 0x10(%rdx,%rcx,1),%xmm1
   b7a2b:	movups %xmm1,0x10(%rax)
   b7a2f:	movups %xmm0,(%rax)
   b7a32:	cmp    %r12,%rbx
   b7a35:	jb     b7802 <<mibz2::c30::C30<true>>::room+0x18b2>
   b7a3b:	mov    $0x0,%ebx
   b7a40:	test   %r12,%r12
   b7a43:	jne    b7802 <<mibz2::c30::C30<true>>::room+0x18b2>
   b7a49:	mov    0x20(%rsp),%r14
   b7a4e:	xor    %edi,%edi
   b7a50:	lea    0x76421(%rip),%rdx        # 12de78 <__frame_dummy_init_array_entry+0x1b58>
   b7a57:	lea    0x76432(%rip),%rax        # 12de90 <__frame_dummy_init_array_entry+0x1b70>
   b7a5e:	mov    0x18(%rsp),%r11
   b7a63:	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   b7a70:	mov    0x40(%rbp),%rsi
   b7a74:	cmp    %rsi,%rdi
   b7a77:	jae    b7c24 <<mibz2::c30::C30<true>>::room+0x1cd4>
   b7a7d:	mov    0xb0(%rbp),%rsi
   b7a84:	cmp    %rsi,%rdi
   b7a87:	jae    b7c21 <<mibz2::c30::C30<true>>::room+0x1cd1>
   b7a8d:	mov    0xa8(%rbp),%rcx
   b7a94:	mov    0x38(%rbp),%rsi
   b7a98:	mov    (%rsi,%rdi,4),%esi
   b7a9b:	movzwl (%rcx,%rdi,2),%ecx
   b7a9f:	mov    0x8(%rbp),%r8
   b7aa3:	mov    0x10(%rbp),%r10
   b7aa7:	mov    %esi,0x590(%rsp)
   b7aae:	mov    %esi,%r9d
   b7ab1:	and    $0x1f,%r9d
   b7ab5:	cmp    $0x1d,%r9d
   b7ab9:	ja     b7b70 <<mibz2::c30::C30<true>>::room+0x1c20>
   b7abf:	shr    $0x5,%esi
   b7ac2:	cmp    %rsi,%r10
   b7ac5:	jbe    b7b70 <<mibz2::c30::C30<true>>::room+0x1c20>
   b7acb:	inc    %rdi
   b7ace:	imul   $0x3c,%rsi,%rsi
   b7ad2:	add    %rsi,%r8
   b7ad5:	mov    %cx,(%r8,%r9,2)
   b7ada:	cmp    %rdi,%r11
   b7add:	jne    b7a70 <<mibz2::c30::C30<true>>::room+0x1b20>
   b7adf:	mov    0x38(%rbp),%rax
   b7ae3:	mov    0x40(%rbp),%rcx
   b7ae7:	mov    0x54(%rsp),%edi
   b7aeb:	mov    %edi,%esi
   b7aed:	mov    $0x88888889,%edx
   b7af2:	imul   %rsi,%rdx
   b7af6:	shr    $0x23,%rdx
   b7afa:	and    $0xfffffffe,%edx
   b7afd:	add    %edi,%edx
   b7aff:	cmp    $0x1,%rcx
   b7b03:	je     b7b2e <<mibz2::c30::C30<true>>::room+0x1bde>
   b7b05:	test   %rcx,%rcx
   b7b08:	je     b7cbd <<mibz2::c30::C30<true>>::room+0x1d6d>
   b7b0e:	xor    %esi,%esi
   b7b10:	mov    %rsi,%rdi
   b7b13:	mov    %rcx,%r8
   b7b16:	shr    $1,%r8
   b7b19:	add    %r8,%rsi
   b7b1c:	cmp    %edx,(%rax,%rsi,4)
   b7b1f:	cmova  %rdi,%rsi
   b7b23:	sub    %r8,%rcx
   b7b26:	cmp    $0x1,%rcx
   b7b2a:	ja     b7b10 <<mibz2::c30::C30<true>>::room+0x1bc0>
   b7b2c:	jmp    b7b30 <<mibz2::c30::C30<true>>::room+0x1be0>
   b7b2e:	xor    %esi,%esi
   b7b30:	cmp    %edx,(%rax,%rsi,4)
   b7b33:	jne    b7cb6 <<mibz2::c30::C30<true>>::room+0x1d66>
   b7b39:	mov    %rsi,0x98(%rbp)
   b7b40:	mov    0xd8(%rsp),%rax
   b7b48:	mov    %rax,0x90(%rbp)
   b7b4f:	test   %r14,%r14
   b7b52:	je     b7b5e <<mibz2::c30::C30<true>>::room+0x1c0e>
   b7b54:	mov    (%rsp),%rdi
   b7b58:	call   *0x78cca(%rip)        # 130828 <free@GLIBC_2.2.5>
   b7b5e:	add    $0x2698,%rsp
   b7b65:	pop    %rbx
   b7b66:	pop    %r12
   b7b68:	pop    %r13
   b7b6a:	pop    %r14
   b7b6c:	pop    %r15
   b7b6e:	pop    %rbp
   b7b6f:	ret
   b7b70:	lea    0x590(%rsp),%rax
   b7b78:	mov    %rax,0x690(%rsp)
   b7b80:	lea    -0x2adc7(%rip),%rax        # 8cdc0 <<u32 as core::fmt::Display>::fmt>
   b7b87:	mov    %rax,0x698(%rsp)
   b7b8f:	lea    -0xaccf7(%rip),%rdi        # ae9f <__abi_tag+0xaba3>
   b7b96:	lea    0x7636b(%rip),%rdx        # 12df08 <__frame_dummy_init_array_entry+0x1be8>
   b7b9d:	lea    0x690(%rsp),%rsi
   b7ba5:	call   89b50 <core::panicking::panic_fmt>
   b7baa:	jmp    b7cec <<mibz2::c30::C30<true>>::room+0x1d9c>
   b7baf:	lea    0x762f2(%rip),%rcx        # 12dea8 <__frame_dummy_init_array_entry+0x1b88>
   b7bb6:	xor    %edi,%edi
   b7bb8:	call   8b610 <core::slice::index::slice_index_fail>
   b7bbd:	lea    0x78(%rbp),%rdi
   b7bc1:	mov    $0x8,%ecx
   b7bc6:	mov    $0x8,%r8d
   b7bcc:	xor    %esi,%esi
   b7bce:	mov    %r12,%rdx
   b7bd1:	call   98dd0 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   b7bd6:	mov    0x88(%rbp),%rbx
   b7bdd:	jmp    b6388 <<mibz2::c30::C30<true>>::room+0x438>
   b7be2:	lea    0x75f0f(%rip),%rdx        # 12daf8 <__frame_dummy_init_array_entry+0x17d8>
   b7be9:	call   8a8ab <core::panicking::panic_bounds_check>
   b7bee:	lea    0x762e3(%rip),%rdx        # 12ded8 <__frame_dummy_init_array_entry+0x1bb8>
   b7bf5:	mov    %r15,%rdi
   b7bf8:	call   8a8ab <core::panicking::panic_bounds_check>
   b7bfd:	lea    0x762bc(%rip),%rdx        # 12dec0 <__frame_dummy_init_array_entry+0x1ba0>
   b7c04:	call   8a8ab <core::panicking::panic_bounds_check>
   b7c09:	lea    0x75ea0(%rip),%rdx        # 12dab0 <__frame_dummy_init_array_entry+0x1790>
   b7c10:	call   8a8ab <core::panicking::panic_bounds_check>
   b7c15:	lea    0x75e7c(%rip),%rdx        # 12da98 <__frame_dummy_init_array_entry+0x1778>
   b7c1c:	call   8a8ab <core::panicking::panic_bounds_check>
   b7c21:	mov    %rax,%rdx
   b7c24:	call   8a8ab <core::panicking::panic_bounds_check>
   b7c29:	jmp    b7cec <<mibz2::c30::C30<true>>::room+0x1d9c>
   b7c2e:	lea    0x75eab(%rip),%rdx        # 12dae0 <__frame_dummy_init_array_entry+0x17c0>
   b7c35:	call   8a8ab <core::panicking::panic_bounds_check>
   b7c3a:	lea    0x75e87(%rip),%rdx        # 12dac8 <__frame_dummy_init_array_entry+0x17a8>
   b7c41:	call   8a8ab <core::panicking::panic_bounds_check>
   b7c46:	lea    0x75e4b(%rip),%rdx        # 12da98 <__frame_dummy_init_array_entry+0x1778>
   b7c4d:	mov    %r10,%rdi
   b7c50:	call   8a8ab <core::panicking::panic_bounds_check>
   b7c55:	lea    0x75e54(%rip),%rdx        # 12dab0 <__frame_dummy_init_array_entry+0x1790>
   b7c5c:	mov    %r10,%rdi
   b7c5f:	call   8a8ab <core::panicking::panic_bounds_check>
   b7c64:	lea    0x75e75(%rip),%rax        # 12dae0 <__frame_dummy_init_array_entry+0x17c0>
   b7c6b:	jmp    b7c74 <<mibz2::c30::C30<true>>::room+0x1d24>
   b7c6d:	lea    0x75e54(%rip),%rax        # 12dac8 <__frame_dummy_init_array_entry+0x17a8>
   b7c74:	mov    %rax,0x28(%rsp)
   b7c79:	mov    0x28(%rsp),%rdx
   b7c7e:	mov    0x20(%rsp),%r14
   b7c83:	call   8a8ab <core::panicking::panic_bounds_check>
   b7c88:	jmp    b7cec <<mibz2::c30::C30<true>>::room+0x1d9c>
   b7c8a:	lea    0x75e7f(%rip),%rdx        # 12db10 <__frame_dummy_init_array_entry+0x17f0>
   b7c91:	mov    %r15,%rdi
   b7c94:	call   8a8ab <core::panicking::panic_bounds_check>
   b7c99:	lea    0x75e70(%rip),%rdx        # 12db10 <__frame_dummy_init_array_entry+0x17f0>
   b7ca0:	mov    %r13,%rdi
   b7ca3:	call   8a8ab <core::panicking::panic_bounds_check>
   b7ca8:	jmp    b7cec <<mibz2::c30::C30<true>>::room+0x1d9c>
   b7caa:	lea    0x76197(%rip),%rdi        # 12de48 <__frame_dummy_init_array_entry+0x1b28>
   b7cb1:	call   8d870 <core::panicking::panic_const::panic_const_div_by_zero>
   b7cb6:	adc    $0x0,%rsi
   b7cba:	mov    %rsi,%rcx
   b7cbd:	mov    %rcx,0x690(%rsp)
   b7cc5:	lea    -0xa61eb(%rip),%rdi        # 11ae1 <core::num::imp::flt2dec::strategy::dragon::POW5TO256+0x27d>
   b7ccc:	lea    0x7613d(%rip),%rcx        # 12de10 <__frame_dummy_init_array_entry+0x1af0>
   b7cd3:	lea    0x76186(%rip),%r8        # 12de60 <__frame_dummy_init_array_entry+0x1b40>
   b7cda:	lea    0x690(%rsp),%rdx
   b7ce2:	mov    $0x19,%esi
   b7ce7:	call   8d560 <core::result::unwrap_failed>
   b7cec:	ud2
   b7cee:	mov    $0x8,%edi
   b7cf3:	mov    $0x80,%esi
   b7cf8:	call   88ccb <alloc::raw_vec::handle_error>
   b7cfd:	jmp    b7d01 <<mibz2::c30::C30<true>>::room+0x1db1>
   b7cff:	jmp    b7d0b <<mibz2::c30::C30<true>>::room+0x1dbb>
   b7d01:	mov    %rax,%rbx
   b7d04:	mov    0x20(%rsp),%r14
   b7d09:	jmp    b7d2a <<mibz2::c30::C30<true>>::room+0x1dda>
   b7d0b:	mov    %rax,%rbx
   b7d0e:	cmpq   $0x0,0x590(%rsp)
   b7d17:	je     b7d39 <<mibz2::c30::C30<true>>::room+0x1de9>
   b7d19:	mov    0x598(%rsp),%rax
   b7d21:	mov    %rax,(%rsp)
   b7d25:	jmp    b7d2f <<mibz2::c30::C30<true>>::room+0x1ddf>
   b7d27:	mov    %rax,%rbx
   b7d2a:	test   %r14,%r14
   b7d2d:	je     b7d39 <<mibz2::c30::C30<true>>::room+0x1de9>
   b7d2f:	mov    (%rsp),%rdi
   b7d33:	call   *0x78aef(%rip)        # 130828 <free@GLIBC_2.2.5>
   b7d39:	mov    %rbx,%rdi
   b7d3c:	call   12b280 <_Unwind_Resume@plt>
   b7d41:	int3
   b7d42:	int3
   b7d43:	int3
   b7d44:	int3
   b7d45:	int3
   b7d46:	int3
   b7d47:	int3
   b7d48:	int3
   b7d49:	int3
   b7d4a:	int3
   b7d4b:	int3
   b7d4c:	int3
   b7d4d:	int3
   b7d4e:	int3
   b7d4f:	int3

