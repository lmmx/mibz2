
target/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

0000000000062330 <<c30walk::ctor2::W<false>>::refill>:
<c30walk::ctor2::W<false>>::refill:
./examples/c30walk/ctor2.rs:387
   62330:	push   %r15
   62332:	push   %r14
   62334:	push   %rbx
   62335:	mov    %rdi,%rbx
./examples/c30walk/ctor2.rs:372
   62338:	lea    (%rsi,%rsi,4),%rax
close<false>():
./examples/c30walk/ctor2.rs:372
   6233c:	lea    (%rdi,%rax,8),%r15
   62340:	add    $0x40,%r15
   62344:	mov    0x40(%rdi,%rax,8),%r14
./examples/c30walk/ctor2.rs:373
   62349:	sub    0x50(%rdi,%rax,8),%r14
   6234e:	je     62358 <<c30walk::ctor2::W<false>>::refill+0x28>
./examples/c30walk/ctor2.rs:374
   62350:	mov    %rbx,%rdi
   62353:	call   61d30 <<c30walk::ctor2::W<false>>::finish>
<c30walk::ctor2::W<false>>::refill:
./examples/c30walk/ctor2.rs:247
   62358:	shr    $1,%r14
rank():
./examples/c30walk/ctor2.rs:247
   6235b:	add    0x18(%r15),%r14d
close<false>():
./examples/c30walk/ctor2.rs:377
   6235f:	mov    %r14d,0x18(%r15)
./examples/c30walk/ctor2.rs:379
   62363:	xorps  %xmm0,%xmm0
   62366:	movups %xmm0,(%r15)
   6236a:	movq   $0x0,0x10(%r15)
<c30walk::ctor2::W<false>>::refill:
./examples/c30walk/ctor2.rs:390
   62372:	mov    0x1c(%r15),%eax
   62376:	cmp    %eax,%r14d
   62379:	jae    62404 <<c30walk::ctor2::W<false>>::refill+0xd4>
./examples/c30walk/ctor2.rs:291
   6237f:	mov    %r14d,%ecx
field<false>():
./examples/c30walk/ctor2.rs:291
   62382:	mov    $0x88888889,%edi
   62387:	imul   %rcx,%rdi
   6238b:	mov    %rdi,%rcx
   6238e:	shr    $0x24,%rcx
   62392:	mov    %ecx,%r9d
   62395:	shl    $0x5,%r9d
   62399:	sub    %ecx,%r9d
   6239c:	sub    %ecx,%r9d
<c30walk::ctor2::W<false>>::refill:
./examples/c30walk/ctor2.rs:350
   6239f:	lea    0x1e(%r9),%edx
/rustc/2d8144b78/library/core/src/cmp.rs:1077
   623a3:	cmp    %eax,%edx
min<u32>():
/rustc/2d8144b78/library/core/src/cmp.rs:1077
   623a5:	cmovae %eax,%edx
<c30walk::ctor2::W<false>>::refill:
./examples/c30walk/ctor2.rs:285
   623a8:	shr    $0x2a,%rdi
is_raw<false>():
./examples/c30walk/ctor2.rs:285
   623ac:	mov    0x10(%rbx),%rsi
   623b0:	cmp    %rsi,%rdi
   623b3:	jae    62410 <<c30walk::ctor2::W<false>>::refill+0xe0>
open<false>():
   623b5:	mov    %r14d,%esi
   623b8:	mov    %rcx,%r8
   623bb:	shl    $0x6,%r8
   623bf:	add    0x28(%rbx),%r8
   623c3:	sub    %r9d,%esi
is_raw<false>():
./examples/c30walk/ctor2.rs:285
   623c6:	mov    0x8(%rbx),%r9
   623ca:	mov    (%r9,%rdi,8),%rdi
open<false>():
./examples/c30walk/ctor2.rs:352
   623ce:	bt     %rcx,%rdi
   623d2:	jae    623df <<c30walk::ctor2::W<false>>::refill+0xaf>
   623d4:	mov    $0xfffff,%edi
./examples/c30walk/ctor2.rs:353
   623d9:	and    0x3c(%r8),%edi
   623dd:	jmp    623e4 <<c30walk::ctor2::W<false>>::refill+0xb4>
   623df:	mov    $0xffffffff,%edi
field<false>():
./examples/c30walk/ctor2.rs:291
   623e4:	mov    %esi,%esi
<c30walk::ctor2::W<false>>::refill:
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   623e6:	lea    (%r8,%rsi,2),%rsi
open<false>():
./examples/c30walk/ctor2.rs:361
   623ea:	mov    %rsi,0x10(%r15)
./examples/c30walk/ctor2.rs:362
   623ee:	mov    %rsi,(%r15)
./examples/c30walk/ctor2.rs:363
   623f1:	sub    %r14d,%edx
<c30walk::ctor2::W<false>>::refill:
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   623f4:	lea    (%rsi,%rdx,2),%rdx
open<false>():
./examples/c30walk/ctor2.rs:363
   623f8:	mov    %rdx,0x8(%r15)
./examples/c30walk/ctor2.rs:365
   623fc:	mov    %ecx,0x20(%r15)
./examples/c30walk/ctor2.rs:366
   62400:	mov    %edi,0x24(%r15)
<c30walk::ctor2::W<false>>::refill:
./examples/c30walk/ctor2.rs:390
   62404:	cmp    %eax,%r14d
   62407:	setb   %al
./examples/c30walk/ctor2.rs:395
   6240a:	pop    %rbx
   6240b:	pop    %r14
   6240d:	pop    %r15
   6240f:	ret
is_raw<false>():
./examples/c30walk/ctor2.rs:285
   62410:	lea    0x8f8b9(%rip),%rdx        # f1cd0 <__frame_dummy_init_array_entry+0x240>
   62417:	call   9afa3 <core::panicking::panic_bounds_check>
