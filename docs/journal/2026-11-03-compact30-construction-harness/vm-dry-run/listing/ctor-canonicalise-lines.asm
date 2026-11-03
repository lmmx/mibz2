
./target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

000000000005fab0 <c30walk::ctor::canonicalise>:
c30walk::ctor::canonicalise:
./examples/c30walk/ctor.rs:660
   5fab0:	push   %r15
   5fab2:	push   %r14
   5fab4:	push   %r13
   5fab6:	push   %r12
   5fab8:	push   %rbx
   5fab9:	sub    $0x10,%rsp
   5fabd:	xor    %r11d,%r11d
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/non_null.rs:1714
   5fac0:	test   %rcx,%rcx
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/iter/macros.rs:180
   5fac3:	je     5fb59 <c30walk::ctor::canonicalise+0xa9>
   5fac9:	lea    (%rdx,%rcx,8),%rcx
   5facd:	xor    %r10d,%r10d
./examples/c30walk/ctor.rs:664
   5fad0:	jmp    5faec <c30walk::ctor::canonicalise+0x3c>
   5fad2:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
next<core::slice::iter::Iter<u64>>():
   5fae0:	add    $0x8,%rdx
   5fae4:	inc    %r10
eq<u64>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/non_null.rs:1714
   5fae7:	cmp    %rcx,%rdx
next<u64>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/iter/macros.rs:180
   5faea:	je     5fb56 <c30walk::ctor::canonicalise+0xa6>
c30walk::ctor::canonicalise:
./examples/c30walk/ctor.rs:662
   5faec:	mov    (%rdx),%rbx
./examples/c30walk/ctor.rs:664
   5faef:	test   %rbx,%rbx
   5faf2:	je     5fae0 <c30walk::ctor::canonicalise+0x30>
   5faf4:	mov    %r10,%r14
   5faf7:	shl    $0xc,%r14
   5fafb:	add    %rsi,%r14
   5fafe:	mov    %r11d,%r15d
   5fb01:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/num/uint_macros.rs:178
   5fb10:	tzcnt  %rbx,%r12
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mut_ptr.rs:961
   5fb15:	shl    $0x6,%r12d
add<c30walk::Rec>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mut_ptr.rs:961
   5fb19:	mov    0x3c(%r14,%r12,1),%eax
c30walk::ctor::canonicalise:
./examples/c30walk/ctor.rs:668
   5fb1e:	and    $0xfffff,%eax
./examples/c30walk/ctor.rs:670
   5fb23:	cmp    %rax,%r9
   5fb26:	jbe    5fd91 <c30walk::ctor::canonicalise+0x2e1>
   5fb2c:	add    %r14,%r12
./examples/c30walk/ctor.rs:666
   5fb2f:	lea    -0x1(%rbx),%r13
./examples/c30walk/ctor.rs:670
   5fb33:	shl    $0x7,%eax
./examples/c30walk/ctor.rs:672
   5fb36:	lea    0x1(%r15),%r11d
./examples/c30walk/ctor.rs:670
   5fb3a:	mov    %r15d,0x7c(%r8,%rax,1)
./examples/c30walk/ctor.rs:671
   5fb3f:	mov    %r15d,%eax
   5fb42:	or     $0x10000000,%eax
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1933
   5fb47:	mov    %eax,0x3c(%r12)
write<u32>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1933
   5fb4c:	mov    %r11d,%r15d
c30walk::ctor::canonicalise:
./examples/c30walk/ctor.rs:664
   5fb4f:	and    %r13,%rbx
   5fb52:	jne    5fb10 <c30walk::ctor::canonicalise+0x60>
   5fb54:	jmp    5fae0 <c30walk::ctor::canonicalise+0x30>
./examples/c30walk/ctor.rs:675
   5fb56:	mov    %r11d,%r11d
   5fb59:	mov    %r11,(%rsp)
   5fb5d:	mov    %r9,0x8(%rsp)
   5fb62:	cmp    %r11,%r9
   5fb65:	jne    5fd71 <c30walk::ctor::canonicalise+0x2c1>
./examples/c30walk/ctor.rs:677
   5fb6b:	lea    -0x80(%r8),%r10
   5fb6f:	xor    %r11d,%r11d
   5fb72:	xor    %ecx,%ecx
   5fb74:	xor    %edx,%edx
   5fb76:	xor    %esi,%esi
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/range.rs:900
   5fb78:	mov    %r11,%rax
spec_next<usize>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/range.rs:900
   5fb7b:	shl    $0x7,%rax
   5fb7f:	add    %r10,%rax
   5fb82:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
c30walk::ctor::canonicalise:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/cmp.rs:1916
   5fb90:	cmp    %r9,%r11
spec_next<usize>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/range.rs:900
   5fb93:	jae    5fc9c <c30walk::ctor::canonicalise+0x1ec>
c30walk::ctor::canonicalise:
./examples/c30walk/ctor.rs:677
   5fb99:	mov    %r11,%rbx
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/num/uint_macros.rs:971
   5fb9c:	inc    %r11
./examples/c30walk/ctor.rs:678
   5fb9f:	mov    0xfc(%rax),%r14d
   5fba6:	sub    $0xffffffffffffff80,%rax
   5fbaa:	cmp    %r14,%rbx
   5fbad:	je     5fb90 <c30walk::ctor::canonicalise+0xe0>
./examples/c30walk/ctor.rs:683
   5fbaf:	movaps (%rax),%xmm0
   5fbb2:	movaps 0x10(%rax),%xmm1
   5fbb6:	movaps 0x20(%rax),%xmm2
   5fbba:	movaps 0x30(%rax),%xmm3
   5fbbe:	movaps 0x40(%rax),%xmm4
   5fbc2:	movaps 0x50(%rax),%xmm5
   5fbc6:	movaps 0x60(%rax),%xmm6
   5fbca:	movdqa 0x70(%rax),%xmm7
   5fbcf:	xor    %r14d,%r14d
   5fbd2:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
./examples/c30walk/ctor.rs:686
   5fbe0:	pshufd $0xee,%xmm7,%xmm8
   5fbe6:	movq   %xmm8,%rax
   5fbeb:	shr    $0x20,%rax
./examples/c30walk/ctor.rs:687
   5fbef:	cmp    %rax,%r9
   5fbf2:	jbe    5fda3 <c30walk::ctor::canonicalise+0x2f3>
   5fbf8:	mov    %rax,%r15
   5fbfb:	shl    $0x7,%r15
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1451
   5fbff:	movaps (%r8,%r15,1),%xmm8
swap_chunk<8>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1451
   5fc04:	movaps 0x10(%r8,%r15,1),%xmm9
   5fc0a:	movaps 0x20(%r8,%r15,1),%xmm10
   5fc10:	movaps 0x30(%r8,%r15,1),%xmm11
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1453
   5fc16:	movaps %xmm0,(%r8,%r15,1)
   5fc1b:	movaps %xmm1,0x10(%r8,%r15,1)
   5fc21:	movaps %xmm2,0x20(%r8,%r15,1)
   5fc27:	movaps %xmm3,0x30(%r8,%r15,1)
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1451
   5fc2d:	movaps 0x40(%r8,%r15,1),%xmm12
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1453
   5fc33:	movaps %xmm4,0x40(%r8,%r15,1)
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1451
   5fc39:	movaps 0x50(%r8,%r15,1),%xmm13
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1453
   5fc3f:	movaps %xmm5,0x50(%r8,%r15,1)
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1451
   5fc45:	movaps 0x60(%r8,%r15,1),%xmm14
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1453
   5fc4b:	movaps %xmm6,0x60(%r8,%r15,1)
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1451
   5fc51:	movaps 0x70(%r8,%r15,1),%xmm15
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1453
   5fc57:	movdqa %xmm7,0x70(%r8,%r15,1)
c30walk::ctor::canonicalise:
./examples/c30walk/ctor.rs:688
   5fc5e:	inc    %r14
   5fc61:	movaps %xmm8,%xmm0
   5fc65:	movaps %xmm9,%xmm1
   5fc69:	movaps %xmm10,%xmm2
   5fc6d:	movaps %xmm11,%xmm3
   5fc71:	movaps %xmm12,%xmm4
   5fc75:	movaps %xmm13,%xmm5
   5fc79:	movaps %xmm14,%xmm6
   5fc7d:	movaps %xmm15,%xmm7
./examples/c30walk/ctor.rs:689
   5fc81:	cmp    %rax,%rbx
   5fc84:	jne    5fbe0 <c30walk::ctor::canonicalise+0x130>
./examples/c30walk/ctor.rs:693
   5fc8a:	add    %r14,%rcx
./examples/c30walk/ctor.rs:694
   5fc8d:	inc    %rdx
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/cmp.rs:1038
   5fc90:	cmp    %rsi,%r14
max<u64>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/cmp.rs:1038
   5fc93:	cmova  %r14,%rsi
c30walk::ctor::canonicalise:
./examples/c30walk/ctor.rs:677
   5fc97:	jmp    5fb78 <c30walk::ctor::canonicalise+0xc8>
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/non_null.rs:1714
   5fc9c:	test   %r9,%r9
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/iter/macros.rs:180
   5fc9f:	je     5fd58 <c30walk::ctor::canonicalise+0x2a8>
./examples/c30walk/ctor.rs:697
   5fca5:	shl    $0x7,%r9
next<c30walk::RawG>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/iter/macros.rs:180
   5fca9:	lea    -0x80(%r9),%r10
   5fcad:	mov    %r10d,%r11d
   5fcb0:	not    %r11d
   5fcb3:	mov    %r8,%rax
   5fcb6:	test   $0x380,%r11d
   5fcbd:	je     5fcf9 <c30walk::ctor::canonicalise+0x249>
   5fcbf:	mov    %r10d,%r11d
   5fcc2:	shr    $0x7,%r11d
   5fcc6:	inc    %r11d
   5fcc9:	and    $0x7,%r11d
   5fccd:	neg    %r11
   5fcd0:	mov    %r8,%rax
c30walk::ctor::canonicalise:
   5fcd3:	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
./examples/c30walk/ctor.rs:699
   5fce0:	movl   $0x0,0x7c(%rax)
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/non_null.rs:656
   5fce7:	sub    $0xffffffffffffff80,%rax
next<c30walk::RawG>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/iter/macros.rs:180
   5fceb:	inc    %r11
   5fcee:	jne    5fce0 <c30walk::ctor::canonicalise+0x230>
   5fcf0:	cmp    $0x380,%r10
   5fcf7:	jb     5fd58 <c30walk::ctor::canonicalise+0x2a8>
c30walk::ctor::canonicalise:
./examples/c30walk/ctor.rs:697
   5fcf9:	add    %r9,%r8
   5fcfc:	nopl   0x0(%rax)
./examples/c30walk/ctor.rs:699
   5fd00:	movl   $0x0,0x7c(%rax)
   5fd07:	movl   $0x0,0xfc(%rax)
   5fd11:	movl   $0x0,0x17c(%rax)
   5fd1b:	movl   $0x0,0x1fc(%rax)
   5fd25:	movl   $0x0,0x27c(%rax)
   5fd2f:	movl   $0x0,0x2fc(%rax)
   5fd39:	movl   $0x0,0x37c(%rax)
   5fd43:	movl   $0x0,0x3fc(%rax)
add<c30walk::RawG>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/non_null.rs:656
   5fd4d:	add    $0x400,%rax
eq<c30walk::RawG>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/non_null.rs:1714
   5fd53:	cmp    %r8,%rax
next<c30walk::RawG>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/iter/macros.rs:180
   5fd56:	jne    5fd00 <c30walk::ctor::canonicalise+0x250>
c30walk::ctor::canonicalise:
./examples/c30walk/ctor.rs:701
   5fd58:	mov    %rcx,(%rdi)
   5fd5b:	mov    %rdx,0x8(%rdi)
   5fd5f:	mov    %rsi,0x10(%rdi)
./examples/c30walk/ctor.rs:702
   5fd63:	add    $0x10,%rsp
   5fd67:	pop    %rbx
   5fd68:	pop    %r12
   5fd6a:	pop    %r13
   5fd6c:	pop    %r14
   5fd6e:	pop    %r15
   5fd70:	ret
./examples/c30walk/ctor.rs:675
   5fd71:	lea    -0x4e60f(%rip),%rdx        # 11769 <GCC_except_table819+0x6f9d>
   5fd78:	lea    0x73871(%rip),%r8        # d35f0 <__frame_dummy_init_array_entry+0x1c40>
   5fd7f:	mov    %rsp,%rdi
   5fd82:	lea    0x8(%rsp),%rsi
   5fd87:	mov    $0x33,%ecx
   5fd8c:	call   7bbc2 <core::panicking::assert_failed::<usize, usize>>
./examples/c30walk/ctor.rs:670
   5fd91:	lea    0x73888(%rip),%rdx        # d3620 <__frame_dummy_init_array_entry+0x1c70>
   5fd98:	mov    %rax,%rdi
   5fd9b:	mov    %r9,%rsi
   5fd9e:	call   7b81b <core::panicking::panic_bounds_check>
./examples/c30walk/ctor.rs:687
   5fda3:	lea    0x7385e(%rip),%rdx        # d3608 <__frame_dummy_init_array_entry+0x1c58>
   5fdaa:	mov    %rax,%rdi
   5fdad:	mov    %r9,%rsi
   5fdb0:	call   7b81b <core::panicking::panic_bounds_check>
