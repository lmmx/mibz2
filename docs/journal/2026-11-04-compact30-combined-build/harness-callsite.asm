# c30walk::main, harness call site of build::<false>; columns: address, own count (v2 alone), own count (<c30walk::ctor2c::Ctor2c>::build::<false> alone), instruction
   7fa36       1      1  mov    0xc0(%rsp),%rbp
   7fa3e       1      1  test   %rbp,%rbp
   7fa41       1      1  je     7fb6f <c30walk::main+0x617f>
   7fa47       1      1  imul   $0x438,%rbp,%r12
   7fa4e       1      1  mov    0x8(%rsp),%r13
   7fa53       1      1  add    %r13,%r12
   7fa56       1      1  jmp    7fa61 <c30walk::main+0x6071>
   7fa58     434    434  cmp    %r12,%r13
   7fa5b     434    434  je     7fb6f <c30walk::main+0x617f>
   7fa61     434    434  lea    0x18(%r13),%rcx
   7fa65     434    434  mov    0x120(%rsp),%rax
   7fa6d     434    434  mov    0x128(%rsp),%rdx
   7fa75     434    434  cmp    $0x2,%rdx
   7fa79     434    434  jne    7fab8 <c30walk::main+0x60c8>
   7fa7b     434      0  cmpw   $0x3276,(%rax)
   7fa80     434      0  jne    819ae <c30walk::main+0x7fbe>
   7fa86     434      0  mov    0x8(%r13),%rsi
   7fa8a     434      0  mov    0x10(%r13),%rdx
   7fa8e     434      0  lea    0x60(%rsp),%rax
   7fa93     434      0  mov    %rax,(%rsp)
   7fa97     434      0  lea    0x350(%rsp),%rdi
   7fa9f     434      0  xor    %r8d,%r8d
   7faa2     434      0  lea    0xe0(%rsp),%r9
   7faaa     434      0  call   3cc30 <<c30walk::ctor2::Ctor2>::build::<false>>
   7faaf     434      0  test   %al,%al
   7fab1     434      0  jne    7fafc <c30walk::main+0x610c>
   7fab3       0      0  jmp    80511 <c30walk::main+0x6b21>
   7fab8       0    434  cmp    $0x1,%rdx
   7fabc       0    434  jne    80538 <c30walk::main+0x6b48>
   7fac2       0    434  cmpb   $0x63,(%rax)
   7fac5       0    434  jne    80533 <c30walk::main+0x6b43>
   7facb       0    434  mov    0x8(%r13),%rsi
   7facf       0    434  mov    0x10(%r13),%rdx
   7fad3       0    434  lea    0x60(%rsp),%rax
   7fad8       0    434  mov    %rax,(%rsp)
   7fadc       0    434  lea    0x2300(%rsp),%rdi
   7fae4       0    434  xor    %r8d,%r8d
   7fae7       0    434  lea    0xe0(%rsp),%r9
   7faef       0    434  call   436f0 <<c30walk::ctor2c::Ctor2c>::build::<false>>
   7faf4       0    434  test   %al,%al
   7faf6       0    434  je     80587 <c30walk::main+0x6b97>
   7fafc     434    434  add    $0x438,%r13
   7fb03     434    434  mov    0xe8(%rsp),%r15
# main own Ir 4499110476 -> 4499111344 (+868); per-instruction deltas sum +868 over 26 instructions; every other instruction of main equal
