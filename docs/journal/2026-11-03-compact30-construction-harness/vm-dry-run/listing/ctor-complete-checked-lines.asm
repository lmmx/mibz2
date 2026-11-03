
./target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

0000000000058220 <<c30walk::ctor::W<false>>::complete_checked>:
<c30walk::ctor::W<false>>::complete_checked:
./examples/c30walk/ctor.rs:503
   58220:	push   %rbp
   58221:	push   %r15
   58223:	push   %r14
   58225:	push   %r13
   58227:	push   %r12
   58229:	push   %rbx
   5822a:	push   %rax
   5822b:	mov    %esi,%r8d
   5822e:	mov    0x10(%rdi),%rax
   58232:	mov    %rax,(%rsp)
   58236:	mov    0x8(%rdi),%rsi
   5823a:	mov    0x830(%rdi),%rax
   58241:	cmp    $0x1d,%rax
   58245:	ja     5858c <<c30walk::ctor::W<false>>::complete_checked+0x36c>
   5824b:	mov    (%rdi),%r10
   5824e:	shl    $0x6,%rax
   58252:	lea    -0x46899(%rip),%r11        # 119c0 <c30walk::ctor::PATTERNS>
   58259:	add    %rax,%r11
   5825c:	mov    0x20(%rdi),%rax
   58260:	movzbl %r8b,%ebx
   58264:	mov    0x838(%rdi),%r14d
./examples/c30walk/ctor.rs:323
   5826b:	mov    %r14d,%ebp
is_raw<false>():
./examples/c30walk/ctor.rs:323
   5826e:	shl    $0x8,%ebp
   58271:	or     %ebx,%ebp
   58273:	mov    %rdx,%r8
   58276:	shl    $0x6,%r8
   5827a:	lea    (%r8,%rax,1),%r15
   5827e:	add    $0x3c,%r15
   58282:	xor    %r12d,%r12d
   58285:	jmp    582e2 <<c30walk::ctor::W<false>>::complete_checked+0xc2>
<c30walk::ctor::W<false>>::complete_checked:
   58287:	nopw   0x0(%rax,%rax,1)
write<c30walk::Rec>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ptr/mod.rs:1933
   58290:	movups 0x2c(%r11),%xmm0
   58295:	movups %xmm0,-0x10(%r15,%r12,2)
   5829b:	movaps (%r11),%xmm0
   5829f:	movaps 0x10(%r11),%xmm1
   582a4:	movaps 0x20(%r11),%xmm2
   582a9:	movaps %xmm2,-0x1c(%r15,%r12,2)
   582af:	movaps %xmm1,-0x2c(%r15,%r12,2)
   582b5:	movaps %xmm0,-0x3c(%r15,%r12,2)
   582bb:	mov    %ebp,(%r15,%r12,2)
<c30walk::ctor::W<false>>::complete_checked:
   582bf:	lea    (%r14,%r12,1),%eax
   582c3:	add    $0x20,%eax
   582c6:	mov    %eax,0x838(%rdi)
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/cmp.rs:1916
   582cc:	add    $0x2000,%ebp
lt():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/cmp.rs:1916
   582d2:	add    $0x20,%r12
   582d6:	inc    %rdx
   582d9:	dec    %rcx
<c30walk::ctor::W<false>>::complete_checked:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/iter/range.rs:900
   582dc:	je     5856b <<c30walk::ctor::W<false>>::complete_checked+0x34b>
is_raw<false>():
./examples/c30walk/ctor.rs:323
   582e2:	mov    %rdx,%r8
   582e5:	shr    $0x6,%r8
   582e9:	cmp    (%rsp),%r8
   582ed:	jae    585b7 <<c30walk::ctor::W<false>>::complete_checked+0x397>
   582f3:	mov    (%rsi,%r8,8),%rax
<c30walk::ctor::W<false>>::complete_checked:
./examples/c30walk/ctor.rs:507
   582f7:	bt     %rdx,%rax
   582fb:	jae    58290 <<c30walk::ctor::W<false>>::complete_checked+0x70>
   582fd:	mov    (%r15,%r12,2),%eax
./examples/c30walk/ctor.rs:430
   58301:	and    $0xfffff,%eax
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/vec/mod.rs:1903
   58306:	mov    0x10(%r10),%r8
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/index.rs:278
   5830a:	cmp    %rax,%r8
index_mut<c30walk::RawG>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/index.rs:278
   5830d:	jbe    5857a <<c30walk::ctor::W<false>>::complete_checked+0x35a>
<c30walk::ctor::W<false>>::complete_checked:
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc/src/raw_vec/mod.rs:614
   58313:	mov    0x8(%r10),%r8
index_mut<c30walk::RawG>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/index.rs:278
   58317:	shl    $0x7,%eax
fragment<false>():
./examples/c30walk/ctor.rs:466
   5831a:	mov    %ebp,(%r8,%rax,1)
./examples/c30walk/ctor.rs:465
   5831e:	movzwl 0x2(%r11),%r9d
./examples/c30walk/ctor.rs:466
   58323:	lea    (%r14,%r12,1),%r13d
   58327:	add    %r13d,%r9d
   5832a:	shl    $0x8,%r9d
   5832e:	or     %ebx,%r9d
   58331:	mov    %r9d,0x4(%r8,%rax,1)
./examples/c30walk/ctor.rs:465
   58336:	movzwl 0x4(%r11),%r9d
./examples/c30walk/ctor.rs:466
   5833b:	add    %r13d,%r9d
   5833e:	shl    $0x8,%r9d
   58342:	or     %ebx,%r9d
   58345:	mov    %r9d,0x8(%r8,%rax,1)
./examples/c30walk/ctor.rs:465
   5834a:	movzwl 0x6(%r11),%r9d
./examples/c30walk/ctor.rs:466
   5834f:	add    %r13d,%r9d
   58352:	shl    $0x8,%r9d
   58356:	or     %ebx,%r9d
   58359:	mov    %r9d,0xc(%r8,%rax,1)
./examples/c30walk/ctor.rs:465
   5835e:	movzwl 0x8(%r11),%r9d
./examples/c30walk/ctor.rs:466
   58363:	add    %r13d,%r9d
   58366:	shl    $0x8,%r9d
   5836a:	or     %ebx,%r9d
   5836d:	mov    %r9d,0x10(%r8,%rax,1)
./examples/c30walk/ctor.rs:465
   58372:	movzwl 0xa(%r11),%r9d
./examples/c30walk/ctor.rs:466
   58377:	add    %r13d,%r9d
   5837a:	shl    $0x8,%r9d
   5837e:	or     %ebx,%r9d
   58381:	mov    %r9d,0x14(%r8,%rax,1)
./examples/c30walk/ctor.rs:465
   58386:	movzwl 0xc(%r11),%r9d
./examples/c30walk/ctor.rs:466
   5838b:	add    %r13d,%r9d
   5838e:	shl    $0x8,%r9d
   58392:	or     %ebx,%r9d
   58395:	mov    %r9d,0x18(%r8,%rax,1)
./examples/c30walk/ctor.rs:465
   5839a:	movzwl 0xe(%r11),%r9d
./examples/c30walk/ctor.rs:466
   5839f:	add    %r13d,%r9d
   583a2:	shl    $0x8,%r9d
   583a6:	or     %ebx,%r9d
   583a9:	mov    %r9d,0x1c(%r8,%rax,1)
./examples/c30walk/ctor.rs:465
   583ae:	movzwl 0x10(%r11),%r9d
./examples/c30walk/ctor.rs:466
   583b3:	add    %r13d,%r9d
   583b6:	shl    $0x8,%r9d
   583ba:	or     %ebx,%r9d
   583bd:	mov    %r9d,0x20(%r8,%rax,1)
./examples/c30walk/ctor.rs:465
   583c2:	movzwl 0x12(%r11),%r9d
./examples/c30walk/ctor.rs:466
   583c7:	add    %r13d,%r9d
   583ca:	shl    $0x8,%r9d
   583ce:	or     %ebx,%r9d
   583d1:	mov    %r9d,0x24(%r8,%rax,1)
./examples/c30walk/ctor.rs:465
   583d6:	movzwl 0x14(%r11),%r9d
./examples/c30walk/ctor.rs:466
   583db:	add    %r13d,%r9d
   583de:	shl    $0x8,%r9d
   583e2:	or     %ebx,%r9d
   583e5:	mov    %r9d,0x28(%r8,%rax,1)
./examples/c30walk/ctor.rs:465
   583ea:	movzwl 0x16(%r11),%r9d
./examples/c30walk/ctor.rs:466
   583ef:	add    %r13d,%r9d
   583f2:	shl    $0x8,%r9d
   583f6:	or     %ebx,%r9d
   583f9:	mov    %r9d,0x2c(%r8,%rax,1)
./examples/c30walk/ctor.rs:465
   583fe:	movzwl 0x18(%r11),%r9d
./examples/c30walk/ctor.rs:466
   58403:	add    %r13d,%r9d
   58406:	shl    $0x8,%r9d
   5840a:	or     %ebx,%r9d
   5840d:	mov    %r9d,0x30(%r8,%rax,1)
./examples/c30walk/ctor.rs:465
   58412:	movzwl 0x1a(%r11),%r9d
./examples/c30walk/ctor.rs:466
   58417:	add    %r13d,%r9d
   5841a:	shl    $0x8,%r9d
   5841e:	or     %ebx,%r9d
   58421:	mov    %r9d,0x34(%r8,%rax,1)
./examples/c30walk/ctor.rs:465
   58426:	movzwl 0x1c(%r11),%r9d
./examples/c30walk/ctor.rs:466
   5842b:	add    %r13d,%r9d
   5842e:	shl    $0x8,%r9d
   58432:	or     %ebx,%r9d
   58435:	mov    %r9d,0x38(%r8,%rax,1)
./examples/c30walk/ctor.rs:465
   5843a:	movzwl 0x1e(%r11),%r9d
./examples/c30walk/ctor.rs:466
   5843f:	add    %r13d,%r9d
   58442:	shl    $0x8,%r9d
   58446:	or     %ebx,%r9d
   58449:	mov    %r9d,0x3c(%r8,%rax,1)
./examples/c30walk/ctor.rs:465
   5844e:	movzwl 0x20(%r11),%r9d
./examples/c30walk/ctor.rs:466
   58453:	add    %r13d,%r9d
   58456:	shl    $0x8,%r9d
   5845a:	or     %ebx,%r9d
   5845d:	mov    %r9d,0x40(%r8,%rax,1)
./examples/c30walk/ctor.rs:465
   58462:	movzwl 0x22(%r11),%r9d
./examples/c30walk/ctor.rs:466
   58467:	add    %r13d,%r9d
   5846a:	shl    $0x8,%r9d
   5846e:	or     %ebx,%r9d
   58471:	mov    %r9d,0x44(%r8,%rax,1)
./examples/c30walk/ctor.rs:465
   58476:	movzwl 0x24(%r11),%r9d
./examples/c30walk/ctor.rs:466
   5847b:	add    %r13d,%r9d
   5847e:	shl    $0x8,%r9d
   58482:	or     %ebx,%r9d
   58485:	mov    %r9d,0x48(%r8,%rax,1)
./examples/c30walk/ctor.rs:465
   5848a:	movzwl 0x26(%r11),%r9d
./examples/c30walk/ctor.rs:466
   5848f:	add    %r13d,%r9d
   58492:	shl    $0x8,%r9d
   58496:	or     %ebx,%r9d
   58499:	mov    %r9d,0x4c(%r8,%rax,1)
./examples/c30walk/ctor.rs:465
   5849e:	movzwl 0x28(%r11),%r9d
./examples/c30walk/ctor.rs:466
   584a3:	add    %r13d,%r9d
   584a6:	shl    $0x8,%r9d
   584aa:	or     %ebx,%r9d
   584ad:	mov    %r9d,0x50(%r8,%rax,1)
./examples/c30walk/ctor.rs:465
   584b2:	movzwl 0x2a(%r11),%r9d
./examples/c30walk/ctor.rs:466
   584b7:	add    %r13d,%r9d
   584ba:	shl    $0x8,%r9d
   584be:	or     %ebx,%r9d
   584c1:	mov    %r9d,0x54(%r8,%rax,1)
./examples/c30walk/ctor.rs:465
   584c6:	movzwl 0x2c(%r11),%r9d
./examples/c30walk/ctor.rs:466
   584cb:	add    %r13d,%r9d
   584ce:	shl    $0x8,%r9d
   584d2:	or     %ebx,%r9d
   584d5:	mov    %r9d,0x58(%r8,%rax,1)
./examples/c30walk/ctor.rs:465
   584da:	movzwl 0x2e(%r11),%r9d
./examples/c30walk/ctor.rs:466
   584df:	add    %r13d,%r9d
   584e2:	shl    $0x8,%r9d
   584e6:	or     %ebx,%r9d
   584e9:	mov    %r9d,0x5c(%r8,%rax,1)
./examples/c30walk/ctor.rs:465
   584ee:	movzwl 0x30(%r11),%r9d
./examples/c30walk/ctor.rs:466
   584f3:	add    %r13d,%r9d
   584f6:	shl    $0x8,%r9d
   584fa:	or     %ebx,%r9d
   584fd:	mov    %r9d,0x60(%r8,%rax,1)
./examples/c30walk/ctor.rs:465
   58502:	movzwl 0x32(%r11),%r9d
./examples/c30walk/ctor.rs:466
   58507:	add    %r13d,%r9d
   5850a:	shl    $0x8,%r9d
   5850e:	or     %ebx,%r9d
   58511:	mov    %r9d,0x64(%r8,%rax,1)
./examples/c30walk/ctor.rs:465
   58516:	movzwl 0x34(%r11),%r9d
./examples/c30walk/ctor.rs:466
   5851b:	add    %r13d,%r9d
   5851e:	shl    $0x8,%r9d
   58522:	or     %ebx,%r9d
   58525:	mov    %r9d,0x68(%r8,%rax,1)
./examples/c30walk/ctor.rs:465
   5852a:	movzwl 0x36(%r11),%r9d
./examples/c30walk/ctor.rs:466
   5852f:	add    %r13d,%r9d
   58532:	shl    $0x8,%r9d
   58536:	or     %ebx,%r9d
   58539:	mov    %r9d,0x6c(%r8,%rax,1)
./examples/c30walk/ctor.rs:465
   5853e:	movzwl 0x38(%r11),%r9d
./examples/c30walk/ctor.rs:466
   58543:	add    %r13d,%r9d
   58546:	shl    $0x8,%r9d
   5854a:	or     %ebx,%r9d
   5854d:	mov    %r9d,0x70(%r8,%rax,1)
./examples/c30walk/ctor.rs:465
   58552:	movzwl 0x3a(%r11),%r9d
./examples/c30walk/ctor.rs:466
   58557:	add    %r13d,%r9d
   5855a:	shl    $0x8,%r9d
   5855e:	or     %ebx,%r9d
   58561:	mov    %r9d,0x74(%r8,%rax,1)
<c30walk::ctor::W<false>>::complete_checked:
./examples/c30walk/ctor.rs:509
   58566:	jmp    582bf <<c30walk::ctor::W<false>>::complete_checked+0x9f>
./examples/c30walk/ctor.rs:529
   5856b:	add    $0x8,%rsp
   5856f:	pop    %rbx
   58570:	pop    %r12
   58572:	pop    %r13
   58574:	pop    %r14
   58576:	pop    %r15
   58578:	pop    %rbp
   58579:	ret
index_mut<c30walk::RawG>():
/rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/slice/index.rs:278
   5857a:	lea    0x7abbf(%rip),%rdx        # d3140 <__frame_dummy_init_array_entry+0x1790>
   58581:	mov    %rax,%rdi
   58584:	mov    %r8,%rsi
   58587:	call   7b81b <core::panicking::panic_bounds_check>
is_raw<false>():
./examples/c30walk/ctor.rs:323
   5858c:	mov    %rdx,%r8
   5858f:	shr    $0x6,%r8
   58593:	cmp    (%rsp),%r8
   58597:	jae    585b7 <<c30walk::ctor::W<false>>::complete_checked+0x397>
   58599:	mov    (%rsi,%r8,8),%rcx
<c30walk::ctor::W<false>>::complete_checked:
./examples/c30walk/ctor.rs:507
   5859d:	bt     %rdx,%rcx
   585a1:	jb     585ca <<c30walk::ctor::W<false>>::complete_checked+0x3aa>
./examples/c30walk/ctor.rs:514
   585a3:	lea    0x7aad6(%rip),%rdx        # d3080 <__frame_dummy_init_array_entry+0x16d0>
   585aa:	mov    $0x1e,%esi
   585af:	mov    %rax,%rdi
   585b2:	call   7b81b <core::panicking::panic_bounds_check>
is_raw<false>():
./examples/c30walk/ctor.rs:323
   585b7:	lea    0x795f2(%rip),%rdx        # d1bb0 <__frame_dummy_init_array_entry+0x200>
   585be:	mov    %r8,%rdi
   585c1:	mov    (%rsp),%rsi
   585c5:	call   7b81b <core::panicking::panic_bounds_check>
fragment<false>():
./examples/c30walk/ctor.rs:426
   585ca:	lea    0x7ab27(%rip),%rdx        # d30f8 <__frame_dummy_init_array_entry+0x1748>
<c30walk::ctor::W<false>>::complete_checked:
   585d1:	mov    $0x1e,%esi
   585d6:	mov    %rax,%rdi
   585d9:	call   7b81b <core::panicking::panic_bounds_check>
