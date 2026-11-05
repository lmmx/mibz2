2b560  push   %rbp
2b561  push   %r15
2b563  push   %r14
2b565  push   %r13
2b567  push   %r12
2b569  push   %rbx
2b56a  sub    $0x38,%rsp
2b56e  mov    %rsi,%r14
2b571  mov    %rdi,0x8(%rsp)
2b576  lea    0x10(%rsi),%rbx
2b57a  mov    $0x1,%ecx
2b57f  xor    %eax,%eax
2b581  lock cmpxchg %ecx,0x10(%rsi)
2b586  jne    2b76c <<mibz2::parallel::JobRx<(u64, u64, mibz2::block::Symbols)>>::recv+0x20c>
2b58c  mov    %rbx,0x10(%rsp)
2b591  mov    0x60528(%rip),%rcx        # 8bac0 <std::panicking::panic_count::GLOBAL_PANIC_COUNT>
2b598  shl    $1,%rcx
2b59b  mov    $0x0,%eax
2b5a0  test   %rcx,%rcx
2b5a3  jne    2b779 <<mibz2::parallel::JobRx<(u64, u64, mibz2::block::Symbols)>>::recv+0x219>
2b5a9  mov    %rax,0x18(%rsp)
2b5ae  movzbl 0x14(%r14),%eax
2b5b3  lea    0x50(%r14),%r13
2b5b7  lea    0x28(%rsp),%r12
2b5bc  mov    0x5e8cd(%rip),%rbx        # 89e90 <syscall@GLIBC_2.2.5>
2b5c3  mov    0x5e8b6(%rip),%r15        # 89e80 <__errno_location@GLIBC_2.2.5>
2b5ca  mov    0x30(%r14),%rax
2b5ce  test   %rax,%rax
2b5d1  je     2b650 <<mibz2::parallel::JobRx<(u64, u64, mibz2::block::Symbols)>>::recv+0xf0>
2b5d3  data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
2b5e0  mov    0x18(%r14),%rcx
2b5e4  mov    0x28(%r14),%rdx
2b5e8  lea    0x1(%rdx),%rsi
2b5ec  cmp    %rcx,%rsi
2b5ef  mov    $0x0,%esi
2b5f4  cmovae %rcx,%rsi
2b5f8  neg    %rsi
2b5fb  lea    (%rdx,%rsi,1),%rcx
2b5ff  inc    %rcx
2b602  mov    %rcx,0x28(%r14)
2b606  dec    %rax
2b609  mov    %rax,0x30(%r14)
2b60d  lea    (%rdx,%rdx,4),%rsi
2b611  shl    $0x6,%rsi
2b615  add    0x20(%r14),%rsi
2b619  mov    $0x140,%edx
2b61e  mov    0x8(%rsp),%rbp
2b623  mov    %rbp,%rdi
2b626  call   *0x5e83c(%rip)        # 89e68 <memcpy@GLIBC_2.14>
2b62c  cmpq   $0xffffffffffffffff,0x10(%rbp)
2b631  jne    2b732 <<mibz2::parallel::JobRx<(u64, u64, mibz2::block::Symbols)>>::recv+0x1d2>
2b637  cmpb   $0x0,0x48(%r14)
2b63c  je     2b668 <<mibz2::parallel::JobRx<(u64, u64, mibz2::block::Symbols)>>::recv+0x108>
2b63e  jmp    2b725 <<mibz2::parallel::JobRx<(u64, u64, mibz2::block::Symbols)>>::recv+0x1c5>
2b643  data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
2b650  mov    0x8(%rsp),%rax
2b655  movq   $0xffffffffffffffff,0x10(%rax)
2b65d  cmpb   $0x0,0x48(%r14)
2b662  jne    2b725 <<mibz2::parallel::JobRx<(u64, u64, mibz2::block::Symbols)>>::recv+0x1c5>
2b668  incq   0x38(%r14)
2b66c  mov    0x50(%r14),%ebp
2b670  xor    %eax,%eax
2b672  xchg   %eax,0x10(%r14)
2b676  cmp    $0x2,%eax
2b679  je     2b6fe <<mibz2::parallel::JobRx<(u64, u64, mibz2::block::Symbols)>>::recv+0x19e>
2b67f  movq   $0x0,0x20(%rsp)
2b688  nopl   0x0(%rax,%rax,1)
2b690  mov    0x0(%r13),%eax
2b694  cmp    %ebp,%eax
2b696  jne    2b6d1 <<mibz2::parallel::JobRx<(u64, u64, mibz2::block::Symbols)>>::recv+0x171>
2b698  cmpb   $0x0,0x20(%rsp)
2b69d  mov    $0x0,%r8d
2b6a3  cmovne %r12,%r8
2b6a7  movl   $0xffffffff,(%rsp)
2b6ae  mov    $0xca,%edi
2b6b3  mov    %r13,%rsi
2b6b6  mov    $0x89,%edx
2b6bb  mov    %ebp,%ecx
2b6bd  xor    %r9d,%r9d
2b6c0  xor    %eax,%eax
2b6c2  call   *%rbx
2b6c4  test   %rax,%rax
2b6c7  jns    2b6d1 <<mibz2::parallel::JobRx<(u64, u64, mibz2::block::Symbols)>>::recv+0x171>
2b6c9  call   *%r15
2b6cc  cmpl   $0x4,(%rax)
2b6cf  je     2b690 <<mibz2::parallel::JobRx<(u64, u64, mibz2::block::Symbols)>>::recv+0x130>
2b6d1  xor    %eax,%eax
2b6d3  lea    0x10(%r14),%rbp
2b6d7  mov    $0x1,%ecx
2b6dc  lock cmpxchg %ecx,0x0(%rbp)
2b6e1  jne    2b71a <<mibz2::parallel::JobRx<(u64, u64, mibz2::block::Symbols)>>::recv+0x1ba>
2b6e3  movzbl 0x14(%r14),%eax
2b6e8  decq   0x38(%r14)
2b6ec  mov    0x30(%r14),%rax
2b6f0  test   %rax,%rax
2b6f3  jne    2b5e0 <<mibz2::parallel::JobRx<(u64, u64, mibz2::block::Symbols)>>::recv+0x80>
2b6f9  jmp    2b650 <<mibz2::parallel::JobRx<(u64, u64, mibz2::block::Symbols)>>::recv+0xf0>
2b6fe  mov    $0xca,%edi
2b703  lea    0x10(%r14),%rsi
2b707  mov    $0x81,%edx
2b70c  mov    $0x1,%ecx
2b711  xor    %eax,%eax
2b713  call   *%rbx
2b715  jmp    2b67f <<mibz2::parallel::JobRx<(u64, u64, mibz2::block::Symbols)>>::recv+0x11f>
2b71a  lea    0x10(%r14),%rdi
2b71e  call   56b00 <<std::sys::sync::mutex::futex::Mutex>::lock_contended>
2b723  jmp    2b6e3 <<mibz2::parallel::JobRx<(u64, u64, mibz2::block::Symbols)>>::recv+0x183>
2b725  mov    0x8(%rsp),%rax
2b72a  movq   $0xffffffffffffffff,0x10(%rax)
2b732  cmpb   $0x0,0x18(%rsp)
2b737  jne    2b74f <<mibz2::parallel::JobRx<(u64, u64, mibz2::block::Symbols)>>::recv+0x1ef>
2b739  mov    0x60380(%rip),%rax        # 8bac0 <std::panicking::panic_count::GLOBAL_PANIC_COUNT>
2b740  movabs $0x7fffffffffffffff,%rcx
2b74a  test   %rcx,%rax
2b74d  jne    2b7aa <<mibz2::parallel::JobRx<(u64, u64, mibz2::block::Symbols)>>::recv+0x24a>
2b74f  xor    %eax,%eax
2b751  mov    0x10(%rsp),%rsi
2b756  xchg   %eax,(%rsi)
2b758  cmp    $0x2,%eax
2b75b  je     2b785 <<mibz2::parallel::JobRx<(u64, u64, mibz2::block::Symbols)>>::recv+0x225>
2b75d  add    $0x38,%rsp
2b761  pop    %rbx
2b762  pop    %r12
2b764  pop    %r13
2b766  pop    %r14
2b768  pop    %r15
2b76a  pop    %rbp
2b76b  ret
2b76c  mov    %rbx,%rdi
2b76f  call   56b00 <<std::sys::sync::mutex::futex::Mutex>::lock_contended>
2b774  jmp    2b58c <<mibz2::parallel::JobRx<(u64, u64, mibz2::block::Symbols)>>::recv+0x2c>
2b779  call   56890 <std::panicking::panic_count::is_zero_slow_path>
2b77e  xor    $0x1,%al
2b780  jmp    2b5a9 <<mibz2::parallel::JobRx<(u64, u64, mibz2::block::Symbols)>>::recv+0x49>
2b785  mov    $0xca,%edi
2b78a  mov    $0x81,%edx
2b78f  mov    $0x1,%ecx
2b794  xor    %eax,%eax
2b796  add    $0x38,%rsp
2b79a  pop    %rbx
2b79b  pop    %r12
2b79d  pop    %r13
2b79f  pop    %r14
2b7a1  pop    %r15
2b7a3  pop    %rbp
2b7a4  jmp    *0x5e6e6(%rip)        # 89e90 <syscall@GLIBC_2.2.5>
2b7aa  call   56890 <std::panicking::panic_count::is_zero_slow_path>
2b7af  test   %al,%al
2b7b1  jne    2b74f <<mibz2::parallel::JobRx<(u64, u64, mibz2::block::Symbols)>>::recv+0x1ef>
2b7b3  movb   $0x1,0x14(%r14)
2b7b8  jmp    2b74f <<mibz2::parallel::JobRx<(u64, u64, mibz2::block::Symbols)>>::recv+0x1ef>
