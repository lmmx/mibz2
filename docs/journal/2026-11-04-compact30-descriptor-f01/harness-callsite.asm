# c30walk::main, harness call site of build::<false>; columns: address, own count (v2 alone), own count (<c30walk::ctor2d::Ctor2d>::build::<false> alone), instruction
   7cbd2       1      1  mov    0xe0(%rsp),%r13
   7cbda       1      1  test   %r13,%r13
   7cbdd       1      1  je     7cd17 <c30walk::main+0x63f7>
   7cbe3       1      1  imul   $0x438,%r13,%r12
   7cbea       1      1  mov    0x10(%rsp),%rbp
   7cbef       1      1  add    %rbp,%r12
   7cbf2       1      1  jmp    7cbfd <c30walk::main+0x62dd>
   7cbf4     434    434  cmp    %r12,%rbp
   7cbf7     434    434  je     7cd17 <c30walk::main+0x63f7>
   7cbfd     434    434  lea    0x18(%rbp),%rcx
   7cc01     434    434  mov    0x120(%rsp),%rax
   7cc09     434    434  mov    0x128(%rsp),%rdx
   7cc11     434    434  cmp    $0x2,%rdx
   7cc15     434    434  jne    7cc57 <c30walk::main+0x6337>
   7cc17     434      0  cmpw   $0x3276,(%rax)
   7cc1c     434      0  jne    7dae8 <c30walk::main+0x71c8>
   7cc22     434      0  mov    0x8(%rbp),%rsi
   7cc26     434      0  mov    0x10(%rbp),%rdx
   7cc2a     434      0  lea    0x80(%rsp),%rax
   7cc32     434      0  mov    %rax,(%rsp)
   7cc36     434      0  lea    0x350(%rsp),%rdi
   7cc3e     434      0  xor    %r8d,%r8d
   7cc41     434      0  lea    0x100(%rsp),%r9
   7cc49     434      0  call   3e010 <<c30walk::ctor2::Ctor2>::build::<false>>
   7cc4e     434      0  test   %al,%al
   7cc50     434      0  jne    7cc9e <c30walk::main+0x637e>
   7cc52       0      0  jmp    7d00a <c30walk::main+0x66ea>
   7cc57       0    434  cmp    $0x1,%rdx
   7cc5b       0    434  jne    7d02e <c30walk::main+0x670e>
   7cc61       0    434  cmpb   $0x64,(%rax)
   7cc64       0    434  jne    7d029 <c30walk::main+0x6709>
   7cc6a       0    434  mov    0x8(%rbp),%rsi
   7cc6e       0    434  mov    0x10(%rbp),%rdx
   7cc72       0    434  lea    0x80(%rsp),%rax
   7cc7a       0    434  mov    %rax,(%rsp)
   7cc7e       0    434  lea    0x2190(%rsp),%rdi
   7cc86       0    434  xor    %r8d,%r8d
   7cc89       0    434  lea    0x100(%rsp),%r9
   7cc91       0    434  call   3a960 <<c30walk::ctor2d::Ctor2d>::build::<false>>
   7cc96       0    434  test   %al,%al
   7cc98       0    434  je     7d07d <c30walk::main+0x675d>
   7cc9e     434    434  add    $0x438,%rbp
   7cca5     434    434  mov    0x108(%rsp),%r15
# main own Ir 4499110446 -> 4499111314 (+868); per-instruction deltas sum +868 over 26 instructions; every other instruction of main equal
