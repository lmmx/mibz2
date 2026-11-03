# complete<false>: the complete-record loop (two records an iteration) and its odd-record tail
   246d0:	mov    %rcx,%r9
   246d3:	and    $0xfffffffffffffffe,%r9
   246d7:	lea    (%rdx,%rsi,1),%r10
   246db:	add    $0x7c,%r10
   246df:	xor    %r8d,%r8d
<c30walk::ctor::Ctor>::build::<false>:
   246e2:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
write<c30walk::Rec>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   246f0:	movdqu 0x2c(%rdi),%xmm0
   246f5:	movdqu %xmm0,-0x50(%r10)
   246fb:	movdqa (%rdi),%xmm1
   246ff:	movdqa 0x10(%rdi),%xmm2
   24704:	movdqa 0x20(%rdi),%xmm3
   24709:	movdqa %xmm3,-0x5c(%r10)
   2470f:	movdqa %xmm2,-0x6c(%r10)
   24715:	movdqa %xmm1,-0x7c(%r10)
   2471b:	mov    %eax,-0x40(%r10)
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:971
   2471f:	add    $0x2,%r8
write<c30walk::Rec>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   24723:	movdqa %xmm1,-0x3c(%r10)
   24729:	movdqa %xmm2,-0x2c(%r10)
   2472f:	movdqa %xmm3,-0x1c(%r10)
   24735:	movdqu %xmm0,-0x10(%r10)
   2473b:	lea    0x2000(%rax),%r11d
   24742:	mov    %r11d,(%r10)
complete<false>():
./examples/c30walk/ctor.rs:490
   24745:	add    $0x4000,%eax
spec_next<usize>():
/rustc/2d8144b78/library/core/src/iter/range.rs:900
   2474a:	sub    $0xffffffffffffff80,%r10
   2474e:	cmp    %r8,%r9
   24751:	jne    246f0 <<c30walk::ctor::Ctor>::build::<false>+0x12e0>
   24753:	testb  $0x1,0x18(%rsp)
   24758:	mov    0x1150(%rsp),%r11
   24760:	mov    0x20(%rsp),%r9
   24765:	mov    0x28(%rsp),%r10
   2476a:	je     247a4 <<c30walk::ctor::Ctor>::build::<false>+0x1394>
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   2476c:	add    %rsi,%rdx
   2476f:	shl    $0x6,%r8
write<c30walk::Rec>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   24773:	movups 0x2c(%rdi),%xmm0
   24777:	movups %xmm0,0x2c(%rdx,%r8,1)
   2477d:	movdqa (%rdi),%xmm0
   24781:	movdqa 0x10(%rdi),%xmm1
   24786:	movdqa 0x20(%rdi),%xmm2
   2478b:	movdqa %xmm2,0x20(%rdx,%r8,1)
   24792:	movdqa %xmm1,0x10(%rdx,%r8,1)
   24799:	movdqa %xmm0,(%rdx,%r8,1)
   2479f:	mov    %eax,0x3c(%rdx,%r8,1)
complete<false>():
   247a4:	mov    0x18(%rsp),%rax
