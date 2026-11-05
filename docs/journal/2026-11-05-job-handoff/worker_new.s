
/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/wt-jobq/target/release/e2e_new:     file format elf64-x86-64


Disassembly of section .text:

000000000003ed70 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>>:
std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>:
   3ed70:	push   %rbp
   3ed71:	push   %r15
   3ed73:	push   %r14
   3ed75:	push   %r13
   3ed77:	push   %r12
   3ed79:	push   %rbx
   3ed7a:	sub    $0x848,%rsp
   3ed81:	mov    %rdi,%rbx
   3ed84:	movabs $0x7fffffffffffffff,%r15
   3ed8e:	mov    (%rdi),%r13
   3ed91:	lea    0x10(%r13),%rax
   3ed95:	mov    %rax,0x8(%rsp)
   3ed9a:	movzbl 0x10(%rdi),%eax
   3ed9e:	mov    %al,0x5(%rsp)
   3eda2:	movzbl 0x11(%rdi),%eax
   3eda6:	mov    %al,0x7(%rsp)
   3edaa:	lea    0x1d8(%rsp),%rdi
   3edb2:	mov    $0x400,%edx
   3edb7:	xor    %esi,%esi
   3edb9:	call   *0x4b151(%rip)        # 89f10 <memset@GLIBC_2.2.5>
   3edbf:	xorps  %xmm0,%xmm0
   3edc2:	movups %xmm0,0x148(%rsp)
   3edca:	movups %xmm0,0x160(%rsp)
   3edd2:	movups %xmm0,0x178(%rsp)
   3edda:	movups %xmm0,0x190(%rsp)
   3ede2:	movups %xmm0,0x1a8(%rsp)
   3edea:	movups %xmm0,0x1b8(%rsp)
   3edf2:	movq   $0x0,0x108(%rsp)
   3edfe:	movq   $0x1,0x110(%rsp)
   3ee0a:	movups %xmm0,0x118(%rsp)
   3ee12:	movq   $0x4,0x128(%rsp)
   3ee1e:	movups %xmm0,0x130(%rsp)
   3ee26:	movq   $0x4,0x140(%rsp)
   3ee32:	movq   $0x4,0x158(%rsp)
   3ee3e:	movq   $0x8,0x170(%rsp)
   3ee4a:	movq   $0x8,0x188(%rsp)
   3ee56:	movq   $0x8,0x1a0(%rsp)
   3ee62:	movq   $0x1,0x1c8(%rsp)
   3ee6e:	movq   $0x0,0x1d0(%rsp)
   3ee7a:	mov    %rbx,0x38(%rsp)
   3ee7f:	mov    0x8(%rbx),%rax
   3ee83:	mov    %rax,0xc0(%rsp)
   3ee8b:	lea    0x58(%r13),%rax
   3ee8f:	mov    %rax,0xa8(%rsp)
   3ee97:	lea    0x40(%r13),%rax
   3ee9b:	mov    %rax,0x50(%rsp)
   3eea0:	lea    0x90(%r13),%rax
   3eea7:	mov    %rax,0xb8(%rsp)
   3eeaf:	lea    0x5d8(%rsp),%rbx
   3eeb7:	mov    0x4af92(%rip),%r14        # 89e50 <free@GLIBC_2.2.5>
   3eebe:	mov    %rax,0x48(%rsp)
   3eec3:	jmp    3eedb <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x16b>
   3eec5:	data16 cs nopw 0x0(%rax,%rax,1)
   3eed0:	mov    0x720(%rsp),%rdi
   3eed8:	call   *%r14
   3eedb:	mov    %rbx,%rdi
   3eede:	mov    0xc0(%rsp),%rsi
   3eee6:	call   2b560 <<mibz2::parallel::JobRx<(u64, u64, mibz2::block::Symbols)>>::recv>
   3eeeb:	cmpq   $0xffffffffffffffff,0x5e8(%rsp)
   3eef4:	je     3f96c <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xbfc>
   3eefa:	mov    0x5d8(%rsp),%rax
   3ef02:	mov    %rax,0x40(%rsp)
   3ef07:	mov    0x5e0(%rsp),%rax
   3ef0f:	mov    %rax,0x18(%rsp)
   3ef14:	mov    $0x130,%edx
   3ef19:	lea    0x718(%rsp),%rdi
   3ef21:	lea    0x5e8(%rsp),%rsi
   3ef29:	call   *0x4af39(%rip)        # 89e68 <memcpy@GLIBC_2.14>
   3ef2f:	xor    %eax,%eax
   3ef31:	mov    0x8(%rsp),%rcx
   3ef36:	mov    $0x1,%edx
   3ef3b:	lock cmpxchg %edx,(%rcx)
   3ef3f:	jne    3efde <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x26e>
   3ef45:	mov    0x4cb74(%rip),%rax        # 8bac0 <std::panicking::panic_count::GLOBAL_PANIC_COUNT>
   3ef4c:	test   %r15,%rax
   3ef4f:	jne    3eff8 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x288>
   3ef55:	movzbl 0x14(%r13),%eax
   3ef5a:	cmpb   $0x0,0x80(%r13)
   3ef62:	je     3efa0 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x230>
   3ef64:	mov    0x4cb55(%rip),%rax        # 8bac0 <std::panicking::panic_count::GLOBAL_PANIC_COUNT>
   3ef6b:	test   %r15,%rax
   3ef6e:	jne    3f6c9 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x959>
   3ef74:	xor    %eax,%eax
   3ef76:	mov    0x8(%rsp),%rcx
   3ef7b:	xchg   %eax,(%rcx)
   3ef7d:	cmp    $0x2,%eax
   3ef80:	je     3f019 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x2a9>
   3ef86:	cmpq   $0x0,0x718(%rsp)
   3ef8f:	jne    3eed0 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x160>
   3ef95:	jmp    3eedb <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x16b>
   3ef9a:	nopw   0x0(%rax,%rax,1)
   3efa0:	xor    %eax,%eax
   3efa2:	mov    0x38(%r13),%rcx
   3efa6:	test   %rcx,%rcx
   3efa9:	je     3f058 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x2e8>
   3efaf:	dec    %rcx
   3efb2:	mov    %rcx,0x38(%r13)
   3efb6:	mov    0x30(%r13),%rdx
   3efba:	lea    (%rcx,%rcx,2),%rcx
   3efbe:	mov    0x10(%rdx,%rcx,8),%rsi
   3efc3:	mov    %rsi,0x30(%rsp)
   3efc8:	movups (%rdx,%rcx,8),%xmm0
   3efcc:	movaps %xmm0,0x20(%rsp)
   3efd1:	test   %al,%al
   3efd3:	je     3f077 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x307>
   3efd9:	jmp    3f087 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x317>
   3efde:	mov    0x8(%rsp),%rdi
   3efe3:	call   56b00 <<std::sys::sync::mutex::futex::Mutex>::lock_contended>
   3efe8:	mov    0x4cad1(%rip),%rax        # 8bac0 <std::panicking::panic_count::GLOBAL_PANIC_COUNT>
   3efef:	test   %r15,%rax
   3eff2:	je     3ef55 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x1e5>
   3eff8:	call   56890 <std::panicking::panic_count::is_zero_slow_path>
   3effd:	movzbl 0x14(%r13),%ecx
   3f002:	cmpb   $0x0,0x80(%r13)
   3f00a:	je     3f049 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x2d9>
   3f00c:	test   %al,%al
   3f00e:	jne    3ef64 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x1f4>
   3f014:	jmp    3ef74 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x204>
   3f019:	mov    $0xca,%edi
   3f01e:	mov    0x8(%rsp),%rsi
   3f023:	mov    $0x81,%edx
   3f028:	mov    $0x1,%ecx
   3f02d:	xor    %eax,%eax
   3f02f:	call   *0x4ae5b(%rip)        # 89e90 <syscall@GLIBC_2.2.5>
   3f035:	cmpq   $0x0,0x718(%rsp)
   3f03e:	jne    3eed0 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x160>
   3f044:	jmp    3eedb <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x16b>
   3f049:	xor    $0x1,%al
   3f04b:	mov    0x38(%r13),%rcx
   3f04f:	test   %rcx,%rcx
   3f052:	jne    3efaf <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x23f>
   3f058:	movq   $0x0,0x20(%rsp)
   3f061:	movq   $0x1,0x28(%rsp)
   3f06a:	movq   $0x0,0x30(%rsp)
   3f073:	test   %al,%al
   3f075:	jne    3f087 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x317>
   3f077:	mov    0x4ca42(%rip),%rax        # 8bac0 <std::panicking::panic_count::GLOBAL_PANIC_COUNT>
   3f07e:	test   %r15,%rax
   3f081:	jne    3f6e0 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x970>
   3f087:	xor    %eax,%eax
   3f089:	mov    0x8(%rsp),%rcx
   3f08e:	xchg   %eax,(%rcx)
   3f090:	cmp    $0x2,%eax
   3f093:	je     3f0cc <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x35c>
   3f095:	movq   $0x0,0x30(%rsp)
   3f09e:	cmpb   $0x0,0x5(%rsp)
   3f0a3:	je     3f100 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x390>
   3f0a5:	lea    0xc8(%rsp),%rdi
   3f0ad:	lea    0x718(%rsp),%rsi
   3f0b5:	lea    0x108(%rsp),%rdx
   3f0bd:	lea    0x1d8(%rsp),%rcx
   3f0c5:	call   3fbf0 <mibz2::mtf::mtf_avx2>
   3f0ca:	jmp    3f125 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x3b5>
   3f0cc:	mov    $0xca,%edi
   3f0d1:	mov    0x8(%rsp),%rsi
   3f0d6:	mov    $0x81,%edx
   3f0db:	mov    $0x1,%ecx
   3f0e0:	xor    %eax,%eax
   3f0e2:	call   *0x4ada8(%rip)        # 89e90 <syscall@GLIBC_2.2.5>
   3f0e8:	movq   $0x0,0x30(%rsp)
   3f0f1:	cmpb   $0x0,0x5(%rsp)
   3f0f6:	jne    3f0a5 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x335>
   3f0f8:	nopl   0x0(%rax,%rax,1)
   3f100:	lea    0xc8(%rsp),%rdi
   3f108:	lea    0x718(%rsp),%rsi
   3f110:	lea    0x108(%rsp),%rdx
   3f118:	lea    0x1d8(%rsp),%rcx
   3f120:	call   349e0 <mibz2::mtf::mtf_scalar>
   3f125:	mov    0xc8(%rsp),%rcx
   3f12d:	mov    0xd0(%rsp),%rax
   3f135:	cmp    $0xffffffffffffffff,%rcx
   3f139:	je     3f17d <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x40d>
   3f13b:	mov    0xd8(%rsp),%rdx
   3f143:	mov    0xe0(%rsp),%rsi
   3f14b:	mov    %rax,%rdi
   3f14e:	shr    $0x20,%rdi
   3f152:	mov    %rdi,0x48(%rsp)
   3f157:	mov    %eax,0x14(%rsp)
   3f15b:	mov    %rdx,0x58(%rsp)
   3f160:	mov    %rcx,%rdi
   3f163:	mov    %rsi,0xb0(%rsp)
   3f16b:	mov    %rsi,%rcx
   3f16e:	movabs $0x7fffffffffffffff,%r15
   3f178:	jmp    3f263 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x4f3>
   3f17d:	mov    0x83c(%rsp),%r15d
   3f185:	cmp    %r15,%rax
   3f188:	jbe    3f238 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x4c8>
   3f18e:	mov    0x110(%rsp),%rdi
   3f196:	mov    0x118(%rsp),%rsi
   3f19e:	lea    0x1d8(%rsp),%rdx
   3f1a6:	lea    0x138(%rsp),%rcx
   3f1ae:	call   2cb20 <mibz2::bwt::build>
   3f1b3:	mov    0x140(%rsp),%rbp
   3f1bb:	mov    0x148(%rsp),%r12
   3f1c3:	cmp    $0x10000,%r12
   3f1ca:	jb     3f4c1 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x751>
   3f1d0:	mov    %rbp,%rdi
   3f1d3:	mov    %r12,%rsi
   3f1d6:	mov    %r15d,%edx
   3f1d9:	lea    0x150(%rsp),%rcx
   3f1e1:	call   27e60 <mibz2::bwt::room_segments::<32, 1>>
   3f1e6:	lea    0x150(%rsp),%rdi
   3f1ee:	lea    0x108(%rsp),%rsi
   3f1f6:	call   2e480 <<mibz2::bwt::SegScratch>::assemble>
   3f1fb:	mov    0x118(%rsp),%r15
   3f203:	mov    0x110(%rsp),%rsi
   3f20b:	cmpb   $0x0,0x5(%rsp)
   3f210:	je     3f540 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x7d0>
   3f216:	lea    0xe8(%rsp),%rdi
   3f21e:	mov    %r15,%rdx
   3f221:	lea    0x20(%rsp),%rcx
   3f226:	lea    0x5d8(%rsp),%rbx
   3f22e:	call   2e860 <mibz2::rle::unrle_avx2>
   3f233:	jmp    3f560 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x7f0>
   3f238:	movabs $0x7fffffffffffffff,%r15
   3f242:	lea    0x5(%r15),%rdi
   3f246:	mov    0x58(%rsp),%rdx
   3f24b:	mov    0x48(%rsp),%rcx
   3f250:	shl    $0x20,%rcx
   3f254:	mov    0x14(%rsp),%eax
   3f258:	or     %rcx,%rax
   3f25b:	mov    0xb0(%rsp),%rcx
   3f263:	movq   $0x1,0x68(%rsp)
   3f26c:	mov    0x40(%rsp),%rsi
   3f271:	mov    %rsi,0x70(%rsp)
   3f276:	movq   $0x1,0x78(%rsp)
   3f27f:	mov    0x18(%rsp),%rsi
   3f284:	mov    %rsi,0x80(%rsp)
   3f28c:	mov    %rdi,0x60(%rsp)
   3f291:	mov    %rdi,0x88(%rsp)
   3f299:	mov    %rax,0x90(%rsp)
   3f2a1:	mov    %rdx,0x98(%rsp)
   3f2a9:	mov    %rcx,0xa0(%rsp)
   3f2b1:	mov    $0x1,%r12b
   3f2b4:	xor    %ebp,%ebp
   3f2b6:	xor    %eax,%eax
   3f2b8:	mov    0x8(%rsp),%rcx
   3f2bd:	mov    $0x1,%edx
   3f2c2:	lock cmpxchg %edx,(%rcx)
   3f2c6:	jne    3f67e <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x90e>
   3f2cc:	mov    0x4c7ed(%rip),%rax        # 8bac0 <std::panicking::panic_count::GLOBAL_PANIC_COUNT>
   3f2d3:	test   %r15,%rax
   3f2d6:	jne    3f698 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x928>
   3f2dc:	movl   $0x0,0x18(%rsp)
   3f2e4:	movzbl 0x14(%r13),%eax
   3f2e9:	movzbl 0x80(%r13),%eax
   3f2f1:	mov    %al,0x6(%rsp)
   3f2f5:	test   %al,%al
   3f2f7:	jne    3f370 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x600>
   3f2f9:	mov    %rbx,%rdi
   3f2fc:	mov    0xa8(%rsp),%rsi
   3f304:	mov    0x40(%rsp),%rdx
   3f309:	lea    0x68(%rsp),%rcx
   3f30e:	call   2ef00 <<alloc::collections::btree::map::BTreeMap<u64, core::result::Result<alloc::vec::Vec<u8>, mibz2::Error>>>::insert>
   3f313:	mov    0x5d8(%rsp),%rax
   3f31b:	cmp    $0xfffffffffffffffe,%rax
   3f31f:	je     3f370 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x600>
   3f321:	cmp    $0xffffffff,%eax
   3f324:	jne    3f33e <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x5ce>
   3f326:	cmpq   $0x0,0x5e0(%rsp)
   3f32f:	je     3f370 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x600>
   3f331:	mov    0x5e8(%rsp),%rdi
   3f339:	call   *%r14
   3f33c:	jmp    3f370 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x600>
   3f33e:	mov    0x5f8(%rsp),%rax
   3f346:	lea    0x1(%r15),%rcx
   3f34a:	xor    %rax,%rcx
   3f34d:	test   %rax,%rax
   3f350:	mov    $0x16,%edx
   3f355:	cmovns %rdx,%rcx
   3f359:	cmp    $0x16,%rcx
   3f35d:	jae    3f62b <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x8bb>
   3f363:	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   3f370:	mov    %ebp,0x40(%rsp)
   3f374:	mov    0x50(%r13),%rbp
   3f378:	mov    0x88(%r13),%rbx
   3f37f:	cmp    %rbx,%rbp
   3f382:	jae    3f3f0 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x680>
   3f384:	mov    %r12d,0x10(%rsp)
   3f389:	mov    0x718(%rsp),%r14
   3f391:	mov    0x720(%rsp),%r15
   3f399:	mov    0x728(%rsp),%r12
   3f3a1:	mov    0x50(%rsp),%rax
   3f3a6:	cmp    (%rax),%rbp
   3f3a9:	jne    3f3b5 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x645>
   3f3ab:	mov    0x50(%rsp),%rdi
   3f3b0:	call   30060 <<alloc::raw_vec::RawVec<alloc::vec::Vec<u16>>>::grow_one>
   3f3b5:	mov    0x48(%r13),%rax
   3f3b9:	lea    0x0(,%rbp,2),%rcx
   3f3c1:	add    %rbp,%rcx
   3f3c4:	mov    %r14,(%rax,%rcx,8)
   3f3c8:	mov    %r15,0x8(%rax,%rcx,8)
   3f3cd:	mov    %r12,0x10(%rax,%rcx,8)
   3f3d2:	lea    0x1(%rbp),%rax
   3f3d6:	mov    %rax,0x50(%r13)
   3f3da:	mov    0x4aa6f(%rip),%r14        # 89e50 <free@GLIBC_2.2.5>
   3f3e1:	movabs $0x7fffffffffffffff,%r15
   3f3eb:	mov    0x10(%rsp),%r12d
   3f3f0:	cmpb   $0x0,0x18(%rsp)
   3f3f5:	jne    3f407 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x697>
   3f3f7:	mov    0x4c6c2(%rip),%rax        # 8bac0 <std::panicking::panic_count::GLOBAL_PANIC_COUNT>
   3f3fe:	test   %r15,%rax
   3f401:	jne    3f6f7 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x987>
   3f407:	xor    %eax,%eax
   3f409:	mov    0x8(%rsp),%rcx
   3f40e:	xchg   %eax,(%rcx)
   3f410:	cmp    $0x2,%eax
   3f413:	je     3f6a8 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x938>
   3f419:	mov    0xb8(%rsp),%rsi
   3f421:	lock incl (%rsi)
   3f424:	mov    $0xca,%edi
   3f429:	mov    $0x81,%edx
   3f42e:	mov    $0x7fffffff,%ecx
   3f433:	xor    %eax,%eax
   3f435:	call   *0x4aa55(%rip)        # 89e90 <syscall@GLIBC_2.2.5>
   3f43b:	cmpb   $0x0,0x6(%rsp)
   3f440:	je     3f47d <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x70d>
   3f442:	cmpb   $0x0,0x40(%rsp)
   3f447:	je     3f45b <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x6eb>
   3f449:	cmpq   $0x0,0x70(%rsp)
   3f44f:	je     3f47d <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x70d>
   3f451:	mov    0x78(%rsp),%rdi
   3f456:	call   *%r14
   3f459:	jmp    3f47d <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x70d>
   3f45b:	lea    0x1(%r15),%rax
   3f45f:	mov    0x60(%rsp),%rcx
   3f464:	xor    %rcx,%rax
   3f467:	test   %rcx,%rcx
   3f46a:	mov    $0x16,%ecx
   3f46f:	cmovns %rcx,%rax
   3f473:	cmp    $0x16,%rax
   3f477:	jae    3f617 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x8a7>
   3f47d:	cmpq   $0x0,0x20(%rsp)
   3f483:	setne  %al
   3f486:	test   %al,%r12b
   3f489:	je     3f493 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x723>
   3f48b:	mov    0x28(%rsp),%rdi
   3f490:	call   *%r14
   3f493:	cmp    %rbx,%rbp
   3f496:	jb     3f4b4 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x744>
   3f498:	cmpq   $0x0,0x718(%rsp)
   3f4a1:	lea    0x5d8(%rsp),%rbx
   3f4a9:	jne    3eed0 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x160>
   3f4af:	jmp    3eedb <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x16b>
   3f4b4:	lea    0x5d8(%rsp),%rbx
   3f4bc:	jmp    3eedb <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x16b>
   3f4c1:	movq   $0x0,0x118(%rsp)
   3f4cd:	cmp    0x108(%rsp),%r12
   3f4d5:	ja     3f70e <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x99e>
   3f4db:	cmp    %r15,%r12
   3f4de:	jbe    3f730 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x9c0>
   3f4e4:	mov    0x0(%rbp,%r15,4),%ebx
   3f4e9:	mov    0x118(%rsp),%r15
   3f4f1:	jmp    3f520 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x7b0>
   3f4f3:	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   3f500:	mov    0x110(%rsp),%rax
   3f508:	mov    %bl,(%rax,%r15,1)
   3f50c:	inc    %r15
   3f50f:	mov    %r15,0x118(%rsp)
   3f517:	dec    %r12
   3f51a:	je     3f203 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x493>
   3f520:	shr    $0x8,%ebx
   3f523:	mov    0x0(%rbp,%rbx,4),%ebx
   3f527:	cmp    0x108(%rsp),%r15
   3f52f:	jne    3f500 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x790>
   3f531:	lea    0x108(%rsp),%rdi
   3f539:	call   1e520 <<alloc::raw_vec::RawVec<u8>>::grow_one>
   3f53e:	jmp    3f500 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x790>
   3f540:	lea    0xe8(%rsp),%rdi
   3f548:	mov    %r15,%rdx
   3f54b:	lea    0x20(%rsp),%rcx
   3f550:	xor    %r8d,%r8d
   3f553:	lea    0x5d8(%rsp),%rbx
   3f55b:	call   2e5e0 <mibz2::rle::unrle_scalar>
   3f560:	mov    0xe8(%rsp),%rcx
   3f568:	cmp    $0xffffffffffffffff,%rcx
   3f56c:	je     3f594 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x824>
   3f56e:	movq   $0x0,0x30(%rsp)
   3f577:	mov    0xf0(%rsp),%rax
   3f57f:	mov    0xf8(%rsp),%rdx
   3f587:	mov    0x100(%rsp),%rsi
   3f58f:	jmp    3f14b <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x3db>
   3f594:	mov    0x30(%rsp),%r15
   3f599:	mov    0x28(%rsp),%r12
   3f59e:	movzbl 0x7(%rsp),%edx
   3f5a3:	mov    %r12,%rdi
   3f5a6:	mov    %r15,%rsi
   3f5a9:	call   2ebd0 <mibz2::crc::crc32>
   3f5ae:	mov    0x838(%rsp),%ecx
   3f5b5:	mov    %ecx,0x14(%rsp)
   3f5b9:	cmp    %ecx,%eax
   3f5bb:	jne    3f5f6 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x886>
   3f5bd:	mov    0x20(%rsp),%rcx
   3f5c2:	mov    %rcx,0x70(%rsp)
   3f5c7:	mov    %r12,0x78(%rsp)
   3f5cc:	mov    %r15,0x80(%rsp)
   3f5d4:	movq   $0xffffffffffffffff,0x68(%rsp)
   3f5dd:	mov    $0x1,%bpl
   3f5e0:	xor    %r12d,%r12d
   3f5e3:	mov    %eax,0x14(%rsp)
   3f5e7:	movabs $0x7fffffffffffffff,%r15
   3f5f1:	jmp    3f2b6 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x546>
   3f5f6:	movq   $0x0,0x30(%rsp)
   3f5ff:	movabs $0x7fffffffffffffff,%r15
   3f609:	lea    0x10(%r15),%rdi
   3f60d:	mov    %rax,0x48(%rsp)
   3f612:	jmp    3f246 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x4d6>
   3f617:	jne    3f63c <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x8cc>
   3f619:	mov    $0x8,%eax
   3f61e:	cmpq   $0x0,0x60(%rsp)
   3f624:	jne    3f650 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x8e0>
   3f626:	jmp    3f47d <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x70d>
   3f62b:	jne    3f65d <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x8ed>
   3f62d:	mov    $0x8,%ecx
   3f632:	test   %rax,%rax
   3f635:	jne    3f671 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x901>
   3f637:	jmp    3f370 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x600>
   3f63c:	mov    $0x10,%eax
   3f641:	cmpq   $0x0,0x90(%rsp)
   3f64a:	je     3f47d <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x70d>
   3f650:	mov    0x88(%rsp,%rax,1),%rdi
   3f658:	jmp    3f456 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x6e6>
   3f65d:	mov    $0x10,%ecx
   3f662:	cmpq   $0x0,0x600(%rsp)
   3f66b:	je     3f370 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x600>
   3f671:	mov    0x5f8(%rsp,%rcx,1),%rdi
   3f679:	jmp    3f339 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x5c9>
   3f67e:	mov    0x8(%rsp),%rdi
   3f683:	call   56b00 <<std::sys::sync::mutex::futex::Mutex>::lock_contended>
   3f688:	mov    0x4c431(%rip),%rax        # 8bac0 <std::panicking::panic_count::GLOBAL_PANIC_COUNT>
   3f68f:	test   %r15,%rax
   3f692:	je     3f2dc <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x56c>
   3f698:	call   56890 <std::panicking::panic_count::is_zero_slow_path>
   3f69d:	xor    $0x1,%al
   3f69f:	mov    %eax,0x18(%rsp)
   3f6a3:	jmp    3f2e4 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x574>
   3f6a8:	mov    $0xca,%edi
   3f6ad:	mov    0x8(%rsp),%rsi
   3f6b2:	mov    $0x81,%edx
   3f6b7:	mov    $0x1,%ecx
   3f6bc:	xor    %eax,%eax
   3f6be:	call   *0x4a7cc(%rip)        # 89e90 <syscall@GLIBC_2.2.5>
   3f6c4:	jmp    3f419 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x6a9>
   3f6c9:	call   56890 <std::panicking::panic_count::is_zero_slow_path>
   3f6ce:	test   %al,%al
   3f6d0:	jne    3ef74 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x204>
   3f6d6:	movb   $0x1,0x14(%r13)
   3f6db:	jmp    3ef74 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x204>
   3f6e0:	call   56890 <std::panicking::panic_count::is_zero_slow_path>
   3f6e5:	test   %al,%al
   3f6e7:	jne    3f087 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x317>
   3f6ed:	movb   $0x1,0x14(%r13)
   3f6f2:	jmp    3f087 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x317>
   3f6f7:	call   56890 <std::panicking::panic_count::is_zero_slow_path>
   3f6fc:	test   %al,%al
   3f6fe:	jne    3f407 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x697>
   3f704:	movb   $0x1,0x14(%r13)
   3f709:	jmp    3f407 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x697>
   3f70e:	mov    $0x1,%ecx
   3f713:	mov    $0x1,%r8d
   3f719:	lea    0x108(%rsp),%rdi
   3f721:	xor    %esi,%esi
   3f723:	mov    %r12,%rdx
   3f726:	call   297c0 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
   3f72b:	jmp    3f4db <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x76b>
   3f730:	mov    %r15,%rdi
   3f733:	mov    %r12,%rsi
   3f736:	lea    0x48ba3(%rip),%rdx        # 882e0 <__frame_dummy_init_array_entry+0xf98>
   3f73d:	call   1f94b <core::panicking::panic_bounds_check>
   3f742:	jmp    3faa5 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xd35>
   3f747:	jmp    3f749 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x9d9>
   3f749:	mov    %rax,%rdi
   3f74c:	call   50130 <__rustc::__rust_panic_cleanup>
   3f751:	lock decq 0x4c367(%rip)        # 8bac0 <std::panicking::panic_count::GLOBAL_PANIC_COUNT>
   3f759:	mov    %rax,%r12
   3f75c:	decq   %fs:0xffffffffffffffd0
   3f765:	mov    %rdx,%rbp
   3f768:	movb   $0x0,%fs:0xffffffffffffffd8
   3f771:	cmpq   $0x0,0x108(%rsp)
   3f77a:	je     3f78a <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xa1a>
   3f77c:	mov    0x110(%rsp),%rdi
   3f784:	call   *0x4a6c6(%rip)        # 89e50 <free@GLIBC_2.2.5>
   3f78a:	cmpq   $0x0,0x120(%rsp)
   3f793:	je     3f7a3 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xa33>
   3f795:	mov    0x128(%rsp),%rdi
   3f79d:	call   *0x4a6ad(%rip)        # 89e50 <free@GLIBC_2.2.5>
   3f7a3:	cmpq   $0x0,0x138(%rsp)
   3f7ac:	je     3f7bc <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xa4c>
   3f7ae:	mov    0x140(%rsp),%rdi
   3f7b6:	call   *0x4a694(%rip)        # 89e50 <free@GLIBC_2.2.5>
   3f7bc:	cmpq   $0x0,0x150(%rsp)
   3f7c5:	je     3f7d5 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xa65>
   3f7c7:	mov    0x158(%rsp),%rdi
   3f7cf:	call   *0x4a67b(%rip)        # 89e50 <free@GLIBC_2.2.5>
   3f7d5:	cmpq   $0x0,0x168(%rsp)
   3f7de:	je     3f7ee <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xa7e>
   3f7e0:	mov    0x170(%rsp),%rdi
   3f7e8:	call   *0x4a662(%rip)        # 89e50 <free@GLIBC_2.2.5>
   3f7ee:	mov    0x188(%rsp),%r15
   3f7f6:	mov    0x190(%rsp),%rbx
   3f7fe:	test   %rbx,%rbx
   3f801:	je     3f824 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xab4>
   3f803:	lea    0x8(%r15),%r14
   3f807:	jmp    3f812 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xaa2>
   3f809:	add    $0x18,%r14
   3f80d:	dec    %rbx
   3f810:	je     3f824 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xab4>
   3f812:	cmpq   $0x0,-0x8(%r14)
   3f817:	je     3f809 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xa99>
   3f819:	mov    (%r14),%rdi
   3f81c:	call   *0x4a62e(%rip)        # 89e50 <free@GLIBC_2.2.5>
   3f822:	jmp    3f809 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xa99>
   3f824:	cmpq   $0x0,0x180(%rsp)
   3f82d:	je     3f838 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xac8>
   3f82f:	mov    %r15,%rdi
   3f832:	call   *0x4a618(%rip)        # 89e50 <free@GLIBC_2.2.5>
   3f838:	cmpq   $0x0,0x198(%rsp)
   3f841:	je     3f851 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xae1>
   3f843:	mov    0x1a0(%rsp),%rdi
   3f84b:	call   *0x4a5ff(%rip)        # 89e50 <free@GLIBC_2.2.5>
   3f851:	cmpq   $0x0,0x1c0(%rsp)
   3f85a:	je     3f86a <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xafa>
   3f85c:	mov    0x1c8(%rsp),%rdi
   3f864:	call   *0x4a5e6(%rip)        # 89e50 <free@GLIBC_2.2.5>
   3f86a:	movq   $0x0,0x108(%rsp)
   3f876:	movq   $0x1,0x110(%rsp)
   3f882:	lea    0x1d8(%rsp),%rax
   3f88a:	xorps  %xmm0,%xmm0
   3f88d:	movups %xmm0,-0xc0(%rax)
   3f894:	movq   $0x4,0x128(%rsp)
   3f8a0:	movups %xmm0,-0xa8(%rax)
   3f8a7:	movq   $0x4,0x140(%rsp)
   3f8b3:	movups %xmm0,-0x90(%rax)
   3f8ba:	movq   $0x4,0x158(%rsp)
   3f8c6:	movups %xmm0,-0x78(%rax)
   3f8ca:	movq   $0x8,0x170(%rsp)
   3f8d6:	movups %xmm0,-0x60(%rax)
   3f8da:	movq   $0x8,0x188(%rsp)
   3f8e6:	movups %xmm0,-0x48(%rax)
   3f8ea:	movq   $0x8,0x1a0(%rsp)
   3f8f6:	movups %xmm0,-0x20(%rax)
   3f8fa:	movups %xmm0,-0x30(%rax)
   3f8fe:	movq   $0x1,0x1c8(%rsp)
   3f90a:	mov    $0x408,%edx
   3f90f:	lea    0x1d0(%rsp),%rdi
   3f917:	xor    %esi,%esi
   3f919:	call   *0x4a5f1(%rip)        # 89f10 <memset@GLIBC_2.2.5>
   3f91f:	lea    0x5d8(%rsp),%rdi
   3f927:	mov    %r12,%rsi
   3f92a:	mov    %rbp,%rdx
   3f92d:	call   2ed70 <mibz2::parallel::panic_message>
   3f932:	mov    0x5d8(%rsp),%rax
   3f93a:	mov    0x5e0(%rsp),%rdx
   3f942:	mov    0x5e8(%rsp),%rcx
   3f94a:	movabs $0x7fffffffffffffff,%r15
   3f954:	lea    0x18(%r15),%rdi
   3f958:	mov    0x4a4f1(%rip),%r14        # 89e50 <free@GLIBC_2.2.5>
   3f95f:	lea    0x5d8(%rsp),%rbx
   3f967:	jmp    3f263 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0x4f3>
   3f96c:	cmpq   $0x0,0x108(%rsp)
   3f975:	je     3f985 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xc15>
   3f977:	mov    0x110(%rsp),%rdi
   3f97f:	call   *0x4a4cb(%rip)        # 89e50 <free@GLIBC_2.2.5>
   3f985:	cmpq   $0x0,0x120(%rsp)
   3f98e:	je     3f99e <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xc2e>
   3f990:	mov    0x128(%rsp),%rdi
   3f998:	call   *0x4a4b2(%rip)        # 89e50 <free@GLIBC_2.2.5>
   3f99e:	cmpq   $0x0,0x138(%rsp)
   3f9a7:	je     3f9b7 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xc47>
   3f9a9:	mov    0x140(%rsp),%rdi
   3f9b1:	call   *0x4a499(%rip)        # 89e50 <free@GLIBC_2.2.5>
   3f9b7:	cmpq   $0x0,0x150(%rsp)
   3f9c0:	je     3f9d0 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xc60>
   3f9c2:	mov    0x158(%rsp),%rdi
   3f9ca:	call   *0x4a480(%rip)        # 89e50 <free@GLIBC_2.2.5>
   3f9d0:	cmpq   $0x0,0x168(%rsp)
   3f9d9:	je     3f9e9 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xc79>
   3f9db:	mov    0x170(%rsp),%rdi
   3f9e3:	call   *0x4a467(%rip)        # 89e50 <free@GLIBC_2.2.5>
   3f9e9:	mov    0x188(%rsp),%rbx
   3f9f1:	mov    0x190(%rsp),%r14
   3f9f9:	test   %r14,%r14
   3f9fc:	je     3fa2b <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xcbb>
   3f9fe:	lea    0x8(%rbx),%r15
   3fa02:	jmp    3fa19 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xca9>
   3fa04:	data16 data16 cs nopw 0x0(%rax,%rax,1)
   3fa10:	add    $0x18,%r15
   3fa14:	dec    %r14
   3fa17:	je     3fa2b <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xcbb>
   3fa19:	cmpq   $0x0,-0x8(%r15)
   3fa1e:	je     3fa10 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xca0>
   3fa20:	mov    (%r15),%rdi
   3fa23:	call   *0x4a427(%rip)        # 89e50 <free@GLIBC_2.2.5>
   3fa29:	jmp    3fa10 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xca0>
   3fa2b:	cmpq   $0x0,0x180(%rsp)
   3fa34:	je     3fa3f <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xccf>
   3fa36:	mov    %rbx,%rdi
   3fa39:	call   *0x4a411(%rip)        # 89e50 <free@GLIBC_2.2.5>
   3fa3f:	cmpq   $0x0,0x198(%rsp)
   3fa48:	je     3fa58 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xce8>
   3fa4a:	mov    0x1a0(%rsp),%rdi
   3fa52:	call   *0x4a3f8(%rip)        # 89e50 <free@GLIBC_2.2.5>
   3fa58:	cmpq   $0x0,0x1c0(%rsp)
   3fa61:	je     3fa71 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xd01>
   3fa63:	mov    0x1c8(%rsp),%rdi
   3fa6b:	call   *0x4a3df(%rip)        # 89e50 <free@GLIBC_2.2.5>
   3fa71:	lock decq 0x0(%r13)
   3fa76:	jne    3fa85 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xd15>
   3fa78:	mov    0x38(%rsp),%rax
   3fa7d:	mov    (%rax),%rdi
   3fa80:	call   301d0 <<alloc::sync::Arc<mibz2::parallel::Shared>>::drop_slow>
   3fa85:	mov    0x38(%rsp),%rax
   3fa8a:	mov    0x8(%rax),%rdi
   3fa8e:	call   30230 <core::ptr::drop_glue::<mibz2::parallel::JobRx<(u64, u64, mibz2::block::Symbols)>>>
   3fa93:	add    $0x848,%rsp
   3fa9a:	pop    %rbx
   3fa9b:	pop    %r12
   3fa9d:	pop    %r13
   3fa9f:	pop    %r14
   3faa1:	pop    %r15
   3faa3:	pop    %rbp
   3faa4:	ret
   3faa5:	ud2
   3faa7:	mov    %rax,%rbx
   3faaa:	mov    $0x1,%bpl
   3faad:	jmp    3fb29 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xdb9>
   3faaf:	mov    %rax,%rbx
   3fab2:	mov    0x38(%rsp),%rax
   3fab7:	mov    0x8(%rax),%rdi
   3fabb:	call   30230 <core::ptr::drop_glue::<mibz2::parallel::JobRx<(u64, u64, mibz2::block::Symbols)>>>
   3fac0:	mov    %rbx,%rdi
   3fac3:	call   86300 <_Unwind_Resume@plt>
   3fac8:	mov    %rax,%rbx
   3facb:	test   %r14,%r14
   3face:	je     3fad9 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xd69>
   3fad0:	mov    %r15,%rdi
   3fad3:	call   *0x4a377(%rip)        # 89e50 <free@GLIBC_2.2.5>
   3fad9:	movzbl 0x18(%rsp),%esi
   3fade:	mov    0x8(%rsp),%rdi
   3fae3:	call   411b0 <core::ptr::drop_glue::<std::sync::poison::mutex::MutexGuard<core::option::Option<std::sync::mpsc::SyncSender<alloc::vec::Vec<u8>>>>>>
   3fae8:	cmpb   $0x0,0x6(%rsp)
   3faed:	je     3fb04 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xd94>
   3faef:	lea    0x68(%rsp),%rdi
   3faf4:	call   300d0 <core::ptr::drop_glue::<core::result::Result<alloc::vec::Vec<u8>, mibz2::Error>>>
   3faf9:	cmpb   $0x0,0x10(%rsp)
   3fafe:	je     3fb5a <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xdea>
   3fb00:	xor    %ebp,%ebp
   3fb02:	jmp    3fb29 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xdb9>
   3fb04:	xor    %ebp,%ebp
   3fb06:	cmpb   $0x0,0x10(%rsp)
   3fb0b:	jne    3fb29 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xdb9>
   3fb0d:	jmp    3fb3c <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xdcc>
   3fb0f:	mov    %rax,%rbx
   3fb12:	movzbl 0x18(%rsp),%esi
   3fb17:	mov    0x8(%rsp),%rdi
   3fb1c:	call   411b0 <core::ptr::drop_glue::<std::sync::poison::mutex::MutexGuard<core::option::Option<std::sync::mpsc::SyncSender<alloc::vec::Vec<u8>>>>>>
   3fb21:	mov    $0x1,%bpl
   3fb24:	test   %r12b,%r12b
   3fb27:	je     3fb41 <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xdd1>
   3fb29:	cmpq   $0x0,0x20(%rsp)
   3fb2f:	je     3fb3c <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xdcc>
   3fb31:	mov    0x28(%rsp),%rdi
   3fb36:	call   *0x4a314(%rip)        # 89e50 <free@GLIBC_2.2.5>
   3fb3c:	test   %bpl,%bpl
   3fb3f:	je     3fb5a <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xdea>
   3fb41:	cmpq   $0x0,0x718(%rsp)
   3fb4a:	je     3fb5a <std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::{closure#0}, ()>+0xdea>
   3fb4c:	mov    0x720(%rsp),%rdi
   3fb54:	call   *0x4a2f6(%rip)        # 89e50 <free@GLIBC_2.2.5>
   3fb5a:	lea    0x108(%rsp),%rdi
   3fb62:	call   3fb90 <core::ptr::drop_glue::<mibz2::Scratch>>
   3fb67:	mov    0x38(%rsp),%rax
   3fb6c:	mov    (%rax),%rdi
   3fb6f:	mov    0x8(%rax),%rsi
   3fb73:	call   30190 <core::ptr::drop_glue::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Cooperative<2>>::{closure#0}>>
   3fb78:	mov    %rbx,%rdi
   3fb7b:	call   86300 <_Unwind_Resume@plt>
   3fb80:	call   22133 <core::panicking::panic_in_cleanup>
