# c30walk::main, harness call site of build::<false>; columns: address, own count (v2 alone), own count (<c30walk::ctor2su::Ctor2su>::build::<false> alone), instruction
   84ea1       1      1  test   %r14,%r14
   84ea4       1      1  je     84fce <c30walk::main+0x6d4e>
   84eaa       1      1  mov    0xf0(%rsp),%r13
   84eb2       1      1  imul   $0x438,%r14,%rax
   84eb9       1      1  add    %r13,%rax
   84ebc       1      1  mov    %rax,0x40(%rsp)
   84ec1     434    434  mov    0x120(%rsp),%rax
   84ec9     434    434  mov    0x128(%rsp),%rcx
   84ed1     434    434  cmp    $0x2,%rcx
   84ed5     434    434  jne    85d5c <c30walk::main+0x7adc>
   84edb     434    434  lea    0x18(%r13),%rcx
   84edf     434    434  cmpw   $0x3276,(%rax)
   84ee4     434    434  je     84f23 <c30walk::main+0x6ca3>
   84ee6       0    434  cmpw   $0x7573,(%rax)
   84eeb       0    434  jne    85d57 <c30walk::main+0x7ad7>
   84ef1       0    434  mov    0x8(%r13),%rsi
   84ef5       0    434  mov    0x10(%r13),%rdx
   84ef9       0    434  lea    0x60(%rsp),%rax
   84efe       0    434  mov    %rax,(%rsp)
   84f02       0    434  lea    0x21a0(%rsp),%rdi
   84f0a       0    434  xor    %r8d,%r8d
   84f0d       0    434  lea    0xc0(%rsp),%r9
   84f15       0    434  call   47520 <<c30walk::ctor2su::Ctor2su>::build::<false>>
   84f1a       0    434  test   %al,%al
   84f1c       0    434  jne    84f54 <c30walk::main+0x6cd4>
   84f1e       0      0  jmp    85dab <c30walk::main+0x7b2b>
   84f23     434      0  mov    0x8(%r13),%rsi
   84f27     434      0  mov    0x10(%r13),%rdx
   84f2b     434      0  lea    0x60(%rsp),%rax
   84f30     434      0  mov    %rax,(%rsp)
   84f34     434      0  lea    0x360(%rsp),%rdi
   84f3c     434      0  xor    %r8d,%r8d
   84f3f     434      0  lea    0xc0(%rsp),%r9
   84f47     434      0  call   3d7d0 <<c30walk::ctor2::Ctor2>::build::<false>>
   84f4c     434      0  test   %al,%al
   84f4e     434      0  je     85dc8 <c30walk::main+0x7b48>
   84f54     434    434  add    $0x438,%r13
   84f5b     434    434  mov    0xc8(%rsp),%rbp
# main own Ir 4499105219 -> 4499106087 (+868); per-instruction deltas sum +868 over 22 instructions; every other instruction of main equal
