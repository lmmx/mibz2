
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/wt-jobq/target/release/e2e_base:     file format elf64-x86-64


Disassembly of section .text:

00000000000436d0 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>>:
   436d0:	push   %rbp
   436d1:	push   %r15
   436d3:	push   %r14
   436d5:	push   %r13
   436d7:	push   %r12
   436d9:	push   %rbx
   436da:	sub    $0x898,%rsp
   436e1:	movabs $0x7fffffffffffffff,%r12
   436eb:	mov    (%rdi),%r14
   436ee:	mov    0x8(%rdi),%r15
   436f2:	lea    0x10(%r14),%rbx
   436f6:	lea    0x10(%r15),%rax
   436fa:	mov    %rax,0x28(%rsp)
   436ff:	movzbl 0x10(%rdi),%eax
   43703:	mov    %al,0xd(%rsp)
   43707:	mov    %rdi,0x48(%rsp)
   4370c:	movzbl 0x11(%rdi),%eax
   43710:	mov    %al,0xf(%rsp)
   43714:	lea    0x228(%rsp),%rdi
   4371c:	mov    $0x400,%edx
   43721:	xor    %esi,%esi
   43723:	call   *0x4b5d7(%rip)        # 8ed00 <memset@GLIBC_2.2.5>
   43729:	xorps  %xmm0,%xmm0
   4372c:	movups %xmm0,0x198(%rsp)
   43734:	movups %xmm0,0x1b0(%rsp)
   4373c:	movups %xmm0,0x1c8(%rsp)
   43744:	movups %xmm0,0x1e0(%rsp)
   4374c:	movups %xmm0,0x1f8(%rsp)
   43754:	movups %xmm0,0x208(%rsp)
   4375c:	movq   $0x0,0x158(%rsp)
   43768:	movq   $0x1,0x160(%rsp)
   43774:	movups %xmm0,0x168(%rsp)
   4377c:	movq   $0x4,0x178(%rsp)
   43788:	movups %xmm0,0x180(%rsp)
   43790:	movq   $0x4,0x190(%rsp)
   4379c:	movq   $0x4,0x1a8(%rsp)
   437a8:	movq   $0x8,0x1c0(%rsp)
   437b4:	movq   $0x8,0x1d8(%rsp)
   437c0:	movq   $0x8,0x1f0(%rsp)
   437cc:	movq   $0x1,0x218(%rsp)
   437d8:	movq   $0x0,0x220(%rsp)
   437e4:	lea    0x58(%r14),%rax
   437e8:	mov    %rax,0xc0(%rsp)
   437f0:	lea    0x40(%r14),%rax
   437f4:	mov    %rax,0x68(%rsp)
   437f9:	lea    0x90(%r14),%rax
   43800:	mov    %rax,0xd0(%rsp)
   43808:	mov    %rax,0x60(%rsp)
   4380d:	mov    %r14,0x58(%rsp)
   43812:	mov    %rbx,0x10(%rsp)
   43817:	jmp    4382e <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x15e>
   43819:	nopl   0x0(%rax)
   43820:	mov    0x630(%rsp),%rdi
   43828:	call   *0x4b412(%rip)        # 8ec40 <free@GLIBC_2.2.5>
   4382e:	xor    %eax,%eax
   43830:	mov    0x28(%rsp),%rcx
   43835:	mov    $0x1,%edx
   4383a:	lock cmpxchg %edx,(%rcx)
   4383e:	jne    4397e <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x2ae>
   43844:	mov    0x4d065(%rip),%rax        # 908b0 <std::panicking::panic_count::GLOBAL_PANIC_COUNT>
   4384b:	test   %r12,%rax
   4384e:	jne    43998 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x2c8>
   43854:	xor    %ebp,%ebp
   43856:	movzbl 0x14(%r15),%eax
   4385b:	mov    0x18(%r15),%rsi
   4385f:	mov    0x20(%r15),%rdx
   43863:	lea    0x758(%rsp),%rdi
   4386b:	call   2bcb0 <<std::sync::mpmc::Receiver<(u64, u64, mibz2::block::Symbols)>>::recv>
   43870:	test   %bpl,%bpl
   43873:	jne    43885 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x1b5>
   43875:	mov    0x4d034(%rip),%rax        # 908b0 <std::panicking::panic_count::GLOBAL_PANIC_COUNT>
   4387c:	test   %r12,%rax
   4387f:	jne    43a11 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x341>
   43885:	xor    %eax,%eax
   43887:	mov    0x28(%rsp),%rcx
   4388c:	xchg   %eax,(%rcx)
   4388e:	cmp    $0x2,%eax
   43891:	je     439a8 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x2d8>
   43897:	cmpq   $0xffffffffffffffff,0x768(%rsp)
   438a0:	je     4439a <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xcca>
   438a6:	mov    0x758(%rsp),%rax
   438ae:	mov    %rax,0x50(%rsp)
   438b3:	mov    0x760(%rsp),%rax
   438bb:	mov    %rax,0x20(%rsp)
   438c0:	mov    $0x130,%edx
   438c5:	lea    0x628(%rsp),%rdi
   438cd:	lea    0x768(%rsp),%rsi
   438d5:	call   *0x4b37d(%rip)        # 8ec58 <memcpy@GLIBC_2.14>
   438db:	xor    %eax,%eax
   438dd:	mov    $0x1,%ecx
   438e2:	lock cmpxchg %ecx,(%rbx)
   438e6:	jne    439d8 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x308>
   438ec:	mov    0x4cfbd(%rip),%rax        # 908b0 <std::panicking::panic_count::GLOBAL_PANIC_COUNT>
   438f3:	test   %r12,%rax
   438f6:	jne    439f0 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x320>
   438fc:	movzbl 0x14(%r14),%eax
   43901:	cmpb   $0x0,0x80(%r14)
   43909:	je     43940 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x270>
   4390b:	mov    0x4cf9e(%rip),%rax        # 908b0 <std::panicking::panic_count::GLOBAL_PANIC_COUNT>
   43912:	test   %r12,%rax
   43915:	jne    440fa <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xa2a>
   4391b:	xor    %eax,%eax
   4391d:	xchg   %eax,(%rbx)
   4391f:	cmp    $0x2,%eax
   43922:	je     43a28 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x358>
   43928:	cmpq   $0x0,0x628(%rsp)
   43931:	jne    43820 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x150>
   43937:	jmp    4382e <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x15e>
   4393c:	nopl   0x0(%rax)
   43940:	xor    %eax,%eax
   43942:	mov    0x38(%r14),%rcx
   43946:	test   %rcx,%rcx
   43949:	je     43a65 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x395>
   4394f:	dec    %rcx
   43952:	mov    %rcx,0x38(%r14)
   43956:	mov    0x30(%r14),%rdx
   4395a:	lea    (%rcx,%rcx,2),%rcx
   4395e:	mov    0x10(%rdx,%rcx,8),%rsi
   43963:	mov    %rsi,0x40(%rsp)
   43968:	movups (%rdx,%rcx,8),%xmm0
   4396c:	movaps %xmm0,0x30(%rsp)
   43971:	test   %al,%al
   43973:	je     43a84 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x3b4>
   43979:	jmp    43a94 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x3c4>
   4397e:	mov    0x28(%rsp),%rdi
   43983:	call   5b8f0 <<std::sys::sync::mutex::futex::Mutex>::lock_contended>
   43988:	mov    0x4cf21(%rip),%rax        # 908b0 <std::panicking::panic_count::GLOBAL_PANIC_COUNT>
   4398f:	test   %r12,%rax
   43992:	je     43854 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x184>
   43998:	call   5b680 <std::panicking::panic_count::is_zero_slow_path>
   4399d:	mov    %eax,%ebp
   4399f:	xor    $0x1,%bpl
   439a3:	jmp    43856 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x186>
   439a8:	mov    $0xca,%edi
   439ad:	mov    0x28(%rsp),%rsi
   439b2:	mov    $0x81,%edx
   439b7:	mov    $0x1,%ecx
   439bc:	xor    %eax,%eax
   439be:	call   *0x4b2bc(%rip)        # 8ec80 <syscall@GLIBC_2.2.5>
   439c4:	cmpq   $0xffffffffffffffff,0x768(%rsp)
   439cd:	jne    438a6 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x1d6>
   439d3:	jmp    4439a <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xcca>
   439d8:	mov    %rbx,%rdi
   439db:	call   5b8f0 <<std::sys::sync::mutex::futex::Mutex>::lock_contended>
   439e0:	mov    0x4cec9(%rip),%rax        # 908b0 <std::panicking::panic_count::GLOBAL_PANIC_COUNT>
   439e7:	test   %r12,%rax
   439ea:	je     438fc <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x22c>
   439f0:	call   5b680 <std::panicking::panic_count::is_zero_slow_path>
   439f5:	movzbl 0x14(%r14),%ecx
   439fa:	cmpb   $0x0,0x80(%r14)
   43a02:	je     43a56 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x386>
   43a04:	test   %al,%al
   43a06:	jne    4390b <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x23b>
   43a0c:	jmp    4391b <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x24b>
   43a11:	call   5b680 <std::panicking::panic_count::is_zero_slow_path>
   43a16:	test   %al,%al
   43a18:	jne    43885 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x1b5>
   43a1e:	movb   $0x1,0x14(%r15)
   43a23:	jmp    43885 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x1b5>
   43a28:	mov    $0xca,%edi
   43a2d:	mov    %rbx,%rsi
   43a30:	mov    $0x81,%edx
   43a35:	mov    $0x1,%ecx
   43a3a:	xor    %eax,%eax
   43a3c:	call   *0x4b23e(%rip)        # 8ec80 <syscall@GLIBC_2.2.5>
   43a42:	cmpq   $0x0,0x628(%rsp)
   43a4b:	jne    43820 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x150>
   43a51:	jmp    4382e <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x15e>
   43a56:	xor    $0x1,%al
   43a58:	mov    0x38(%r14),%rcx
   43a5c:	test   %rcx,%rcx
   43a5f:	jne    4394f <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x27f>
   43a65:	movq   $0x0,0x30(%rsp)
   43a6e:	movq   $0x1,0x38(%rsp)
   43a77:	movq   $0x0,0x40(%rsp)
   43a80:	test   %al,%al
   43a82:	jne    43a94 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x3c4>
   43a84:	mov    0x4ce25(%rip),%rax        # 908b0 <std::panicking::panic_count::GLOBAL_PANIC_COUNT>
   43a8b:	test   %r12,%rax
   43a8e:	jne    44111 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xa41>
   43a94:	xor    %eax,%eax
   43a96:	xchg   %eax,(%rbx)
   43a98:	cmp    $0x2,%eax
   43a9b:	je     43ad4 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x404>
   43a9d:	movq   $0x0,0x40(%rsp)
   43aa6:	cmpb   $0x0,0xd(%rsp)
   43aab:	je     43b00 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x430>
   43aad:	lea    0xd8(%rsp),%rdi
   43ab5:	lea    0x628(%rsp),%rsi
   43abd:	lea    0x158(%rsp),%rdx
   43ac5:	lea    0x228(%rsp),%rcx
   43acd:	call   44650 <mibz2::mtf::mtf_avx2>
   43ad2:	jmp    43b25 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x455>
   43ad4:	mov    $0xca,%edi
   43ad9:	mov    %rbx,%rsi
   43adc:	mov    $0x81,%edx
   43ae1:	mov    $0x1,%ecx
   43ae6:	xor    %eax,%eax
   43ae8:	call   *0x4b192(%rip)        # 8ec80 <syscall@GLIBC_2.2.5>
   43aee:	movq   $0x0,0x40(%rsp)
   43af7:	cmpb   $0x0,0xd(%rsp)
   43afc:	jne    43aad <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x3dd>
   43afe:	xchg   %ax,%ax
   43b00:	lea    0xd8(%rsp),%rdi
   43b08:	lea    0x628(%rsp),%rsi
   43b10:	lea    0x158(%rsp),%rdx
   43b18:	lea    0x228(%rsp),%rcx
   43b20:	call   36b80 <mibz2::mtf::mtf_scalar>
   43b25:	mov    0xd8(%rsp),%rcx
   43b2d:	mov    0xe0(%rsp),%rax
   43b35:	cmp    $0xffffffffffffffff,%rcx
   43b39:	je     43b7d <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x4ad>
   43b3b:	mov    0xe8(%rsp),%rdx
   43b43:	mov    0xf0(%rsp),%rsi
   43b4b:	mov    %rax,%rdi
   43b4e:	shr    $0x20,%rdi
   43b52:	mov    %rdi,0x60(%rsp)
   43b57:	mov    %eax,0x1c(%rsp)
   43b5b:	mov    %rdx,0x70(%rsp)
   43b60:	mov    %rcx,%rdi
   43b63:	mov    %rsi,0xc8(%rsp)
   43b6b:	mov    %rsi,%rcx
   43b6e:	movabs $0x7fffffffffffffff,%r12
   43b78:	jmp    43c64 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x594>
   43b7d:	mov    0x74c(%rsp),%r12d
   43b85:	cmp    %r12,%rax
   43b88:	jbe    43c35 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x565>
   43b8e:	mov    0x160(%rsp),%rdi
   43b96:	mov    0x168(%rsp),%rsi
   43b9e:	lea    0x228(%rsp),%rdx
   43ba6:	lea    0x188(%rsp),%rcx
   43bae:	call   2e160 <mibz2::bwt::build>
   43bb3:	mov    0x190(%rsp),%rbp
   43bbb:	mov    0x198(%rsp),%r13
   43bc3:	cmp    $0x10000,%r13
   43bca:	jb     43edf <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x80f>
   43bd0:	mov    %rbp,%rdi
   43bd3:	mov    %r13,%rsi
   43bd6:	mov    %r12d,%edx
   43bd9:	lea    0x1a0(%rsp),%rcx
   43be1:	call   28390 <mibz2::bwt::room_segments::<32, 1>>
   43be6:	lea    0x1a0(%rsp),%rdi
   43bee:	lea    0x158(%rsp),%rsi
   43bf6:	call   2fac0 <<mibz2::bwt::SegScratch>::assemble>
   43bfb:	mov    0x168(%rsp),%r12
   43c03:	mov    0x160(%rsp),%rsi
   43c0b:	cmpb   $0x0,0xd(%rsp)
   43c10:	je     43f60 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x890>
   43c16:	lea    0xf8(%rsp),%rdi
   43c1e:	mov    %r12,%rdx
   43c21:	lea    0x30(%rsp),%rcx
   43c26:	mov    0x10(%rsp),%rbx
   43c2b:	call   2fea0 <mibz2::rle::unrle_avx2>
   43c30:	jmp    43f7d <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x8ad>
   43c35:	movabs $0x7fffffffffffffff,%r12
   43c3f:	lea    0x5(%r12),%rsi
   43c44:	mov    0x70(%rsp),%rdx
   43c49:	mov    0x60(%rsp),%rcx
   43c4e:	shl    $0x20,%rcx
   43c52:	mov    0x1c(%rsp),%eax
   43c56:	or     %rcx,%rax
   43c59:	mov    0xc8(%rsp),%rcx
   43c61:	mov    %rsi,%rdi
   43c64:	movq   $0x1,0x80(%rsp)
   43c70:	mov    0x50(%rsp),%rsi
   43c75:	mov    %rsi,0x88(%rsp)
   43c7d:	movq   $0x1,0x90(%rsp)
   43c89:	mov    0x20(%rsp),%rsi
   43c8e:	mov    %rsi,0x98(%rsp)
   43c96:	mov    %rdi,0x78(%rsp)
   43c9b:	mov    %rdi,0xa0(%rsp)
   43ca3:	mov    %rax,0xa8(%rsp)
   43cab:	mov    %rdx,0xb0(%rsp)
   43cb3:	mov    %rcx,0xb8(%rsp)
   43cbb:	mov    $0x1,%r13b
   43cbe:	xor    %ebp,%ebp
   43cc0:	xor    %eax,%eax
   43cc2:	mov    $0x1,%ecx
   43cc7:	lock cmpxchg %ecx,(%rbx)
   43ccb:	jne    44028 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x958>
   43cd1:	mov    0x4cbd8(%rip),%rax        # 908b0 <std::panicking::panic_count::GLOBAL_PANIC_COUNT>
   43cd8:	test   %r12,%rax
   43cdb:	jne    44040 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x970>
   43ce1:	movl   $0x0,0x20(%rsp)
   43ce9:	movzbl 0x14(%r14),%eax
   43cee:	movzbl 0x80(%r14),%eax
   43cf6:	mov    %al,0xe(%rsp)
   43cfa:	test   %al,%al
   43cfc:	jne    43d80 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x6b0>
   43d02:	lea    0x118(%rsp),%rdi
   43d0a:	mov    0xc0(%rsp),%rsi
   43d12:	mov    0x50(%rsp),%rdx
   43d17:	lea    0x80(%rsp),%rcx
   43d1f:	call   30540 <<alloc::collections::btree::map::BTreeMap<u64, core::result::Result<alloc::vec::Vec<u8>, mibz2::Error>>>::insert>
   43d24:	mov    0x118(%rsp),%rax
   43d2c:	cmp    $0xfffffffffffffffe,%rax
   43d30:	je     43d80 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x6b0>
   43d32:	cmp    $0xffffffff,%eax
   43d35:	jne    43d52 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x682>
   43d37:	cmpq   $0x0,0x120(%rsp)
   43d40:	je     43d80 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x6b0>
   43d42:	mov    0x128(%rsp),%rdi
   43d4a:	call   *0x4aef0(%rip)        # 8ec40 <free@GLIBC_2.2.5>
   43d50:	jmp    43d80 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x6b0>
   43d52:	mov    0x138(%rsp),%rax
   43d5a:	lea    0x1(%r12),%rcx
   43d5f:	xor    %rax,%rcx
   43d62:	test   %rax,%rax
   43d65:	mov    $0x16,%edx
   43d6a:	cmovns %rdx,%rcx
   43d6e:	cmp    $0x16,%rcx
   43d72:	jae    44086 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x9b6>
   43d78:	nopl   0x0(%rax,%rax,1)
   43d80:	mov    %ebp,0x50(%rsp)
   43d84:	mov    0x50(%r14),%rbp
   43d88:	mov    0x88(%r14),%rbx
   43d8f:	cmp    %rbx,%rbp
   43d92:	jae    43e07 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x737>
   43d94:	mov    %r13d,0x18(%rsp)
   43d99:	mov    %r15,%r14
   43d9c:	mov    0x628(%rsp),%r15
   43da4:	mov    0x630(%rsp),%r12
   43dac:	mov    0x638(%rsp),%r13
   43db4:	mov    0x68(%rsp),%rax
   43db9:	cmp    (%rax),%rbp
   43dbc:	jne    43dc8 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x6f8>
   43dbe:	mov    0x68(%rsp),%rdi
   43dc3:	call   316a0 <<alloc::raw_vec::RawVec<alloc::vec::Vec<u16>>>::grow_one>
   43dc8:	mov    0x58(%rsp),%rdx
   43dcd:	mov    0x48(%rdx),%rax
   43dd1:	lea    0x0(,%rbp,2),%rcx
   43dd9:	add    %rbp,%rcx
   43ddc:	mov    %r15,(%rax,%rcx,8)
   43de0:	mov    %r12,0x8(%rax,%rcx,8)
   43de5:	mov    %r13,0x10(%rax,%rcx,8)
   43dea:	lea    0x1(%rbp),%rax
   43dee:	mov    %rax,0x50(%rdx)
   43df2:	mov    %r14,%r15
   43df5:	mov    %rdx,%r14
   43df8:	movabs $0x7fffffffffffffff,%r12
   43e02:	mov    0x18(%rsp),%r13d
   43e07:	cmpb   $0x0,0x20(%rsp)
   43e0c:	jne    43e1e <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x74e>
   43e0e:	mov    0x4ca9b(%rip),%rax        # 908b0 <std::panicking::panic_count::GLOBAL_PANIC_COUNT>
   43e15:	test   %r12,%rax
   43e18:	jne    44128 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xa58>
   43e1e:	xor    %eax,%eax
   43e20:	mov    0x10(%rsp),%rcx
   43e25:	xchg   %eax,(%rcx)
   43e27:	cmp    $0x2,%eax
   43e2a:	je     440d9 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xa09>
   43e30:	mov    0xd0(%rsp),%rsi
   43e38:	lock incl (%rsi)
   43e3b:	mov    $0xca,%edi
   43e40:	mov    $0x81,%edx
   43e45:	mov    $0x7fffffff,%ecx
   43e4a:	xor    %eax,%eax
   43e4c:	call   *0x4ae2e(%rip)        # 8ec80 <syscall@GLIBC_2.2.5>
   43e52:	cmpb   $0x0,0xe(%rsp)
   43e57:	je     43e9e <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x7ce>
   43e59:	cmpb   $0x0,0x50(%rsp)
   43e5e:	je     43e7b <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x7ab>
   43e60:	cmpq   $0x0,0x88(%rsp)
   43e69:	je     43e9e <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x7ce>
   43e6b:	mov    0x90(%rsp),%rdi
   43e73:	call   *0x4adc7(%rip)        # 8ec40 <free@GLIBC_2.2.5>
   43e79:	jmp    43e9e <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x7ce>
   43e7b:	lea    0x1(%r12),%rax
   43e80:	mov    0x78(%rsp),%rcx
   43e85:	xor    %rcx,%rax
   43e88:	test   %rcx,%rcx
   43e8b:	mov    $0x16,%ecx
   43e90:	cmovns %rcx,%rax
   43e94:	cmp    $0x16,%rax
   43e98:	jae    44072 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x9a2>
   43e9e:	cmpq   $0x0,0x30(%rsp)
   43ea4:	setne  %al
   43ea7:	test   %al,%r13b
   43eaa:	je     43eb7 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x7e7>
   43eac:	mov    0x38(%rsp),%rdi
   43eb1:	call   *0x4ad89(%rip)        # 8ec40 <free@GLIBC_2.2.5>
   43eb7:	cmp    %rbx,%rbp
   43eba:	jb     43ed5 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x805>
   43ebc:	cmpq   $0x0,0x628(%rsp)
   43ec5:	mov    0x10(%rsp),%rbx
   43eca:	jne    43820 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x150>
   43ed0:	jmp    4382e <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x15e>
   43ed5:	mov    0x10(%rsp),%rbx
   43eda:	jmp    4382e <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x15e>
   43edf:	movq   $0x0,0x168(%rsp)
   43eeb:	cmp    0x158(%rsp),%r13
   43ef3:	ja     4413f <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xa6f>
   43ef9:	cmp    %r12,%r13
   43efc:	jbe    44161 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xa91>
   43f02:	mov    0x0(%rbp,%r12,4),%ebx
   43f07:	mov    0x168(%rsp),%r12
   43f0f:	jmp    43f40 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x870>
   43f11:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   43f20:	mov    0x160(%rsp),%rax
   43f28:	mov    %bl,(%rax,%r12,1)
   43f2c:	inc    %r12
   43f2f:	mov    %r12,0x168(%rsp)
   43f37:	dec    %r13
   43f3a:	je     43c03 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x533>
   43f40:	shr    $0x8,%ebx
   43f43:	mov    0x0(%rbp,%rbx,4),%ebx
   43f47:	cmp    0x158(%rsp),%r12
   43f4f:	jne    43f20 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x850>
   43f51:	lea    0x158(%rsp),%rdi
   43f59:	call   1ea50 <<alloc::raw_vec::RawVec<u8>>::grow_one>
   43f5e:	jmp    43f20 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x850>
   43f60:	lea    0xf8(%rsp),%rdi
   43f68:	mov    %r12,%rdx
   43f6b:	lea    0x30(%rsp),%rcx
   43f70:	xor    %r8d,%r8d
   43f73:	mov    0x10(%rsp),%rbx
   43f78:	call   2fc20 <mibz2::rle::unrle_scalar>
   43f7d:	mov    0xf8(%rsp),%rcx
   43f85:	cmp    $0xffffffffffffffff,%rcx
   43f89:	je     43fb1 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x8e1>
   43f8b:	movq   $0x0,0x40(%rsp)
   43f94:	mov    0x100(%rsp),%rax
   43f9c:	mov    0x108(%rsp),%rdx
   43fa4:	mov    0x110(%rsp),%rsi
   43fac:	jmp    43b4b <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x47b>
   43fb1:	mov    0x40(%rsp),%r12
   43fb6:	mov    0x38(%rsp),%r13
   43fbb:	movzbl 0xf(%rsp),%edx
   43fc0:	mov    %r13,%rdi
   43fc3:	mov    %r12,%rsi
   43fc6:	call   30210 <mibz2::crc::crc32>
   43fcb:	mov    0x748(%rsp),%ecx
   43fd2:	mov    %ecx,0x1c(%rsp)
   43fd6:	cmp    %ecx,%eax
   43fd8:	jne    44050 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x980>
   43fda:	mov    0x30(%rsp),%rcx
   43fdf:	mov    %rcx,0x88(%rsp)
   43fe7:	mov    %r13,0x90(%rsp)
   43fef:	mov    %r12,0x98(%rsp)
   43ff7:	movq   $0xffffffffffffffff,0x80(%rsp)
   44003:	mov    $0x1,%bpl
   44006:	xor    %r13d,%r13d
   44009:	mov    %eax,0x1c(%rsp)
   4400d:	movabs $0x7fffffffffffffff,%r12
   44017:	xor    %eax,%eax
   44019:	mov    $0x1,%ecx
   4401e:	lock cmpxchg %ecx,(%rbx)
   44022:	je     43cd1 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x601>
   44028:	mov    %rbx,%rdi
   4402b:	call   5b8f0 <<std::sys::sync::mutex::futex::Mutex>::lock_contended>
   44030:	mov    0x4c879(%rip),%rax        # 908b0 <std::panicking::panic_count::GLOBAL_PANIC_COUNT>
   44037:	test   %r12,%rax
   4403a:	je     43ce1 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x611>
   44040:	call   5b680 <std::panicking::panic_count::is_zero_slow_path>
   44045:	xor    $0x1,%al
   44047:	mov    %eax,0x20(%rsp)
   4404b:	jmp    43ce9 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x619>
   44050:	movq   $0x0,0x40(%rsp)
   44059:	movabs $0x7fffffffffffffff,%r12
   44063:	lea    0x10(%r12),%rsi
   44068:	mov    %rax,0x60(%rsp)
   4406d:	jmp    43c44 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x574>
   44072:	jne    44097 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x9c7>
   44074:	mov    $0x8,%eax
   44079:	cmpq   $0x0,0x78(%rsp)
   4407f:	jne    440ab <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x9db>
   44081:	jmp    43e9e <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x7ce>
   44086:	jne    440b8 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x9e8>
   44088:	mov    $0x8,%ecx
   4408d:	test   %rax,%rax
   44090:	jne    440cc <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x9fc>
   44092:	jmp    43d80 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x6b0>
   44097:	mov    $0x10,%eax
   4409c:	cmpq   $0x0,0xa8(%rsp)
   440a5:	je     43e9e <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x7ce>
   440ab:	mov    0xa0(%rsp,%rax,1),%rdi
   440b3:	jmp    43e73 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x7a3>
   440b8:	mov    $0x10,%ecx
   440bd:	cmpq   $0x0,0x140(%rsp)
   440c6:	je     43d80 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x6b0>
   440cc:	mov    0x138(%rsp,%rcx,1),%rdi
   440d4:	jmp    43d4a <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x67a>
   440d9:	mov    $0xca,%edi
   440de:	mov    0x10(%rsp),%rsi
   440e3:	mov    $0x81,%edx
   440e8:	mov    $0x1,%ecx
   440ed:	xor    %eax,%eax
   440ef:	call   *0x4ab8b(%rip)        # 8ec80 <syscall@GLIBC_2.2.5>
   440f5:	jmp    43e30 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x760>
   440fa:	call   5b680 <std::panicking::panic_count::is_zero_slow_path>
   440ff:	test   %al,%al
   44101:	jne    4391b <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x24b>
   44107:	movb   $0x1,0x14(%r14)
   4410c:	jmp    4391b <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x24b>
   44111:	call   5b680 <std::panicking::panic_count::is_zero_slow_path>
   44116:	test   %al,%al
   44118:	jne    43a94 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x3c4>
   4411e:	movb   $0x1,0x14(%r14)
   44123:	jmp    43a94 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x3c4>
   44128:	call   5b680 <std::panicking::panic_count::is_zero_slow_path>
   4412d:	test   %al,%al
   4412f:	jne    43e1e <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x74e>
   44135:	movb   $0x1,0x14(%r14)
   4413a:	jmp    43e1e <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x74e>
   4413f:	mov    $0x1,%ecx
   44144:	mov    $0x1,%r8d
   4414a:	lea    0x158(%rsp),%rdi
   44152:	xor    %esi,%esi
   44154:	mov    %r13,%rdx
   44157:	call   29cf0 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   4415c:	jmp    43ef9 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x829>
   44161:	mov    %r12,%rdi
   44164:	mov    %r13,%rsi
   44167:	lea    0x48f62(%rip),%rdx        # 8d0d0 <__frame_dummy_init_array_entry+0xf98>
   4416e:	call   1fe7b <core::panicking::panic_bounds_check>
   44173:	jmp    444e1 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xe11>
   44178:	jmp    4417a <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xaaa>
   4417a:	mov    %rax,%rdi
   4417d:	call   54f20 <__rustc::__rust_panic_cleanup>
   44182:	lock decq 0x4c726(%rip)        # 908b0 <std::panicking::panic_count::GLOBAL_PANIC_COUNT>
   4418a:	mov    %rax,%r13
   4418d:	decq   %fs:0xffffffffffffffd0
   44196:	mov    %rdx,%rbp
   44199:	movb   $0x0,%fs:0xffffffffffffffd8
   441a2:	cmpq   $0x0,0x158(%rsp)
   441ab:	je     441bb <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xaeb>
   441ad:	mov    0x160(%rsp),%rdi
   441b5:	call   *0x4aa85(%rip)        # 8ec40 <free@GLIBC_2.2.5>
   441bb:	cmpq   $0x0,0x170(%rsp)
   441c4:	je     441d4 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xb04>
   441c6:	mov    0x178(%rsp),%rdi
   441ce:	call   *0x4aa6c(%rip)        # 8ec40 <free@GLIBC_2.2.5>
   441d4:	cmpq   $0x0,0x188(%rsp)
   441dd:	je     441ed <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xb1d>
   441df:	mov    0x190(%rsp),%rdi
   441e7:	call   *0x4aa53(%rip)        # 8ec40 <free@GLIBC_2.2.5>
   441ed:	cmpq   $0x0,0x1a0(%rsp)
   441f6:	je     44206 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xb36>
   441f8:	mov    0x1a8(%rsp),%rdi
   44200:	call   *0x4aa3a(%rip)        # 8ec40 <free@GLIBC_2.2.5>
   44206:	cmpq   $0x0,0x1b8(%rsp)
   4420f:	je     4421f <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xb4f>
   44211:	mov    0x1c0(%rsp),%rdi
   44219:	call   *0x4aa21(%rip)        # 8ec40 <free@GLIBC_2.2.5>
   4421f:	mov    0x1d8(%rsp),%r12
   44227:	mov    0x1e0(%rsp),%rbx
   4422f:	test   %rbx,%rbx
   44232:	je     44256 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xb86>
   44234:	lea    0x8(%r12),%r14
   44239:	jmp    44244 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xb74>
   4423b:	add    $0x18,%r14
   4423f:	dec    %rbx
   44242:	je     44256 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xb86>
   44244:	cmpq   $0x0,-0x8(%r14)
   44249:	je     4423b <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xb6b>
   4424b:	mov    (%r14),%rdi
   4424e:	call   *0x4a9ec(%rip)        # 8ec40 <free@GLIBC_2.2.5>
   44254:	jmp    4423b <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xb6b>
   44256:	cmpq   $0x0,0x1d0(%rsp)
   4425f:	je     4426a <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xb9a>
   44261:	mov    %r12,%rdi
   44264:	call   *0x4a9d6(%rip)        # 8ec40 <free@GLIBC_2.2.5>
   4426a:	cmpq   $0x0,0x1e8(%rsp)
   44273:	je     44283 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xbb3>
   44275:	mov    0x1f0(%rsp),%rdi
   4427d:	call   *0x4a9bd(%rip)        # 8ec40 <free@GLIBC_2.2.5>
   44283:	cmpq   $0x0,0x210(%rsp)
   4428c:	je     4429c <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xbcc>
   4428e:	mov    0x218(%rsp),%rdi
   44296:	call   *0x4a9a4(%rip)        # 8ec40 <free@GLIBC_2.2.5>
   4429c:	movq   $0x0,0x158(%rsp)
   442a8:	movq   $0x1,0x160(%rsp)
   442b4:	lea    0x228(%rsp),%rax
   442bc:	xorps  %xmm0,%xmm0
   442bf:	movups %xmm0,-0xc0(%rax)
   442c6:	movq   $0x4,0x178(%rsp)
   442d2:	movups %xmm0,-0xa8(%rax)
   442d9:	movq   $0x4,0x190(%rsp)
   442e5:	movups %xmm0,-0x90(%rax)
   442ec:	movq   $0x4,0x1a8(%rsp)
   442f8:	movups %xmm0,-0x78(%rax)
   442fc:	movq   $0x8,0x1c0(%rsp)
   44308:	movups %xmm0,-0x60(%rax)
   4430c:	movq   $0x8,0x1d8(%rsp)
   44318:	movups %xmm0,-0x48(%rax)
   4431c:	movq   $0x8,0x1f0(%rsp)
   44328:	movups %xmm0,-0x20(%rax)
   4432c:	movups %xmm0,-0x30(%rax)
   44330:	movq   $0x1,0x218(%rsp)
   4433c:	mov    $0x408,%edx
   44341:	lea    0x220(%rsp),%rdi
   44349:	xor    %esi,%esi
   4434b:	call   *0x4a9af(%rip)        # 8ed00 <memset@GLIBC_2.2.5>
   44351:	lea    0x118(%rsp),%rdi
   44359:	mov    %r13,%rsi
   4435c:	mov    %rbp,%rdx
   4435f:	call   303b0 <mibz2::parallel::panic_message>
   44364:	mov    0x118(%rsp),%rax
   4436c:	mov    0x120(%rsp),%rdx
   44374:	mov    0x128(%rsp),%rcx
   4437c:	movabs $0x7fffffffffffffff,%r12
   44386:	lea    0x18(%r12),%rdi
   4438b:	mov    0x58(%rsp),%r14
   44390:	mov    0x10(%rsp),%rbx
   44395:	jmp    43c64 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x594>
   4439a:	cmpq   $0x0,0x158(%rsp)
   443a3:	je     443b3 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xce3>
   443a5:	mov    0x160(%rsp),%rdi
   443ad:	call   *0x4a88d(%rip)        # 8ec40 <free@GLIBC_2.2.5>
   443b3:	cmpq   $0x0,0x170(%rsp)
   443bc:	je     443cc <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xcfc>
   443be:	mov    0x178(%rsp),%rdi
   443c6:	call   *0x4a874(%rip)        # 8ec40 <free@GLIBC_2.2.5>
   443cc:	cmpq   $0x0,0x188(%rsp)
   443d5:	je     443e5 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xd15>
   443d7:	mov    0x190(%rsp),%rdi
   443df:	call   *0x4a85b(%rip)        # 8ec40 <free@GLIBC_2.2.5>
   443e5:	cmpq   $0x0,0x1a0(%rsp)
   443ee:	je     443fe <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xd2e>
   443f0:	mov    0x1a8(%rsp),%rdi
   443f8:	call   *0x4a842(%rip)        # 8ec40 <free@GLIBC_2.2.5>
   443fe:	cmpq   $0x0,0x1b8(%rsp)
   44407:	je     44417 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xd47>
   44409:	mov    0x1c0(%rsp),%rdi
   44411:	call   *0x4a829(%rip)        # 8ec40 <free@GLIBC_2.2.5>
   44417:	mov    0x1d8(%rsp),%rbx
   4441f:	mov    0x1e0(%rsp),%r14
   44427:	test   %r14,%r14
   4442a:	je     4445d <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xd8d>
   4442c:	lea    0x8(%rbx),%r12
   44430:	jmp    44449 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xd79>
   44432:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   44440:	add    $0x18,%r12
   44444:	dec    %r14
   44447:	je     4445d <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xd8d>
   44449:	cmpq   $0x0,-0x8(%r12)
   4444f:	je     44440 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xd70>
   44451:	mov    (%r12),%rdi
   44455:	call   *0x4a7e5(%rip)        # 8ec40 <free@GLIBC_2.2.5>
   4445b:	jmp    44440 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xd70>
   4445d:	cmpq   $0x0,0x1d0(%rsp)
   44466:	je     44471 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xda1>
   44468:	mov    %rbx,%rdi
   4446b:	call   *0x4a7cf(%rip)        # 8ec40 <free@GLIBC_2.2.5>
   44471:	cmpq   $0x0,0x1e8(%rsp)
   4447a:	mov    0x58(%rsp),%rbx
   4447f:	je     4448f <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xdbf>
   44481:	mov    0x1f0(%rsp),%rdi
   44489:	call   *0x4a7b1(%rip)        # 8ec40 <free@GLIBC_2.2.5>
   4448f:	cmpq   $0x0,0x210(%rsp)
   44498:	je     444a8 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xdd8>
   4449a:	mov    0x218(%rsp),%rdi
   444a2:	call   *0x4a798(%rip)        # 8ec40 <free@GLIBC_2.2.5>
   444a8:	lock decq (%rbx)
   444ac:	jne    444bb <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xdeb>
   444ae:	mov    0x48(%rsp),%rax
   444b3:	mov    (%rax),%rdi
   444b6:	call   31820 <<alloc::sync::Arc<mibz2::parallel::Shared>>::drop_slow>
   444bb:	lock decq (%r15)
   444bf:	jne    444cf <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xdff>
   444c1:	mov    0x48(%rsp),%rax
   444c6:	mov    0x8(%rax),%rdi
   444ca:	call   31880 <<alloc::sync::Arc<std::sync::poison::mutex::Mutex<std::sync::mpsc::Receiver<(u64, u64, mibz2::block::Symbols)>>>>::drop_slow>
   444cf:	add    $0x898,%rsp
   444d6:	pop    %rbx
   444d7:	pop    %r12
   444d9:	pop    %r13
   444db:	pop    %r14
   444dd:	pop    %r15
   444df:	pop    %rbp
   444e0:	ret
   444e1:	ud2
   444e3:	mov    %rax,%rbx
   444e6:	mov    $0x1,%bpl
   444e9:	jmp    44575 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xea5>
   444ee:	mov    %rax,%rbx
   444f1:	lock decq (%r15)
   444f5:	jne    445d7 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xf07>
   444fb:	mov    0x48(%rsp),%rax
   44500:	mov    0x8(%rax),%rdi
   44504:	call   31880 <<alloc::sync::Arc<std::sync::poison::mutex::Mutex<std::sync::mpsc::Receiver<(u64, u64, mibz2::block::Symbols)>>>>::drop_slow>
   44509:	jmp    445d7 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xf07>
   4450e:	call   22663 <core::panicking::panic_in_cleanup>
   44513:	mov    %rax,%rbx
   44516:	test   %r15,%r15
   44519:	je     44524 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xe54>
   4451b:	mov    %r12,%rdi
   4451e:	call   *0x4a71c(%rip)        # 8ec40 <free@GLIBC_2.2.5>
   44524:	movzbl 0x20(%rsp),%esi
   44529:	mov    0x10(%rsp),%rdi
   4452e:	call   45c10 <core::ptr::drop_glue::<std::sync::poison::mutex::MutexGuard<core::option::Option<std::sync::mpsc::SyncSender<alloc::vec::Vec<u8>>>>>>
   44533:	cmpb   $0x0,0xe(%rsp)
   44538:	je     44552 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xe82>
   4453a:	lea    0x80(%rsp),%rdi
   44542:	call   31710 <core::ptr::drop_glue::<core::result::Result<alloc::vec::Vec<u8>, mibz2::Error>>>
   44547:	cmpb   $0x0,0x18(%rsp)
   4454c:	je     445b9 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xee9>
   4454e:	xor    %ebp,%ebp
   44550:	jmp    44575 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xea5>
   44552:	xor    %ebp,%ebp
   44554:	cmpb   $0x0,0x18(%rsp)
   44559:	jne    44575 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xea5>
   4455b:	jmp    44588 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xeb8>
   4455d:	mov    %rbx,%rdi
   44560:	mov    %rax,%rbx
   44563:	movzbl 0x20(%rsp),%esi
   44568:	call   45c10 <core::ptr::drop_glue::<std::sync::poison::mutex::MutexGuard<core::option::Option<std::sync::mpsc::SyncSender<alloc::vec::Vec<u8>>>>>>
   4456d:	mov    $0x1,%bpl
   44570:	test   %r13b,%r13b
   44573:	je     4458d <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xebd>
   44575:	cmpq   $0x0,0x30(%rsp)
   4457b:	je     44588 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xeb8>
   4457d:	mov    0x38(%rsp),%rdi
   44582:	call   *0x4a6b8(%rip)        # 8ec40 <free@GLIBC_2.2.5>
   44588:	test   %bpl,%bpl
   4458b:	je     445b9 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xee9>
   4458d:	cmpq   $0x0,0x628(%rsp)
   44596:	je     445b9 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xee9>
   44598:	mov    0x630(%rsp),%rdi
   445a0:	call   *0x4a69a(%rip)        # 8ec40 <free@GLIBC_2.2.5>
   445a6:	jmp    445b9 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xee9>
   445a8:	mov    %rax,%rbx
   445ab:	movzbl %bpl,%esi
   445af:	mov    0x28(%rsp),%rdi
   445b4:	call   45c10 <core::ptr::drop_glue::<std::sync::poison::mutex::MutexGuard<core::option::Option<std::sync::mpsc::SyncSender<alloc::vec::Vec<u8>>>>>>
   445b9:	lea    0x158(%rsp),%rdi
   445c1:	call   445f0 <core::ptr::drop_glue::<mibz2::Scratch>>
   445c6:	mov    0x48(%rsp),%rax
   445cb:	mov    (%rax),%rdi
   445ce:	mov    0x8(%rax),%rsi
   445d2:	call   317d0 <core::ptr::drop_glue::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Cooperative<2>>::{closure#0}>>
   445d7:	mov    %rbx,%rdi
   445da:	call   8b0f0 <_Unwind_Resume@plt>
   445df:	call   22663 <core::panicking::panic_in_cleanup>
