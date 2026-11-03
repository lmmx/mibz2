# fragment<false>, j > 0: the header read, RAW test, pattern bounds check and endpoint (span) check before the field stores
   23eb1:	mov    0x910(%rsp),%eax
   23eb8:	mov    %eax,0x18(%rsp)
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:330
   23ebc:	mov    0xf8(%rsp),%rcx
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   23ec4:	mov    %rdi,%rax
add<c30walk::Rec>():
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   23ec7:	shl    $0x6,%rax
   23ecb:	mov    %rax,0x80(%rsp)
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1733
   23ed3:	mov    0x3c(%rcx,%rax,1),%eax
fragment<false>():
./examples/c30walk/ctor.rs:442
   23ed7:	test   $0x10000000,%eax
   23edc:	mov    %rdi,0x8(%rsp)
   23ee1:	mov    %rsi,0xc0(%rsp)
   23ee9:	mov    %r9,0x68(%rsp)
   23eee:	jne    23f7d <<c30walk::ctor::Ctor>::build::<false>+0xb6d>
   23ef4:	mov    0x30(%rsp),%r8
./examples/c30walk/ctor.rs:448
   23ef9:	lea    -0x1(%r8),%rdi
   23efd:	cmp    $0x1d,%rdi
   23f01:	ja     24f2e <<c30walk::ctor::Ctor>::build::<false>+0x1b1e>
   23f07:	mov    %eax,%edi
   23f09:	shr    $0x8,%edi
   23f0c:	mov    0x18(%rsp),%edx
   23f10:	sub    %edi,%edx
   23f12:	movzwl -0x2(%r9,%r8,2),%edi
   23f18:	add    %edx,%edi
   23f1a:	cmp    $0xfffd,%edi
   23f20:	ja     249cd <<c30walk::ctor::Ctor>::build::<false>+0x15bd>
   23f26:	mov    0x10(%rsp),%rax
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   23f2b:	cmp    $0x1e,%rax
next<u16>():
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   23f2f:	mov    $0x1e,%edi
   23f34:	cmovb  %rax,%rdi
   23f38:	mov    %r10,%rax
   23f3b:	not    %rax
   23f3e:	add    %rdi,%rax
   23f41:	movabs $0x7fffffffffffffff,%rdi
   23f4b:	and    %rdi,%rax
   23f4e:	cmp    $0x3,%rax
   23f52:	jae    24060 <<c30walk::ctor::Ctor>::build::<false>+0xc50>
   23f58:	xor    %eax,%eax
   23f5a:	mov    %r9,%r10
   23f5d:	mov    0x80(%rsp),%r9
   23f65:	jmp    2431f <<c30walk::ctor::Ctor>::build::<false>+0xf0f>
run<false>():
   23f6a:	mov    0x60(%rsp),%rcx
./examples/c30walk/ctor.rs:572
   23f6f:	test   %rcx,%rcx
   23f72:	jne    243cc <<c30walk::ctor::Ctor>::build::<false>+0xfbc>
   23f78:	jmp    24413 <<c30walk::ctor::Ctor>::build::<false>+0x1003>
fragment<false>():

# fragment<false>, j > 0, ordinary: offsets d + pattern, 16 fields an iteration, then 4, then 1
   24178:	mov    %rsi,%rax
   2417b:	and    $0xfffffffffffffff0,%rax
   2417f:	movd   %edx,%xmm0
   24183:	pshuflw $0x0,%xmm0,%xmm0
   24188:	pshufd $0x44,%xmm0,%xmm0
next<u16>():
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   2418d:	lea    -0x14ad4(%rip),%rdi        # f6c0 <c30walk::ctor::PATTERNS>
   24194:	mov    %r10,%rsi
   24197:	mov    0x58(%rsp),%r10
   2419c:	lea    0x10(%r10,%rdi,1),%r10
   241a1:	mov    0xa8(%rsp),%r11
   241a9:	not    %r11d
   241ac:	lea    (%rsi,%r15,1),%rdi
   241b0:	cmp    $0x1e,%rdi
   241b4:	mov    $0x1e,%ebp
   241b9:	cmovae %rbp,%rdi
   241bd:	movabs $0x7fffffffffffffe0,%rbp
   241c7:	or     %rbp,%r11
   241ca:	add    %rdi,%r11
   241cd:	movabs $0x7fffffffffffffff,%rdi
   241d7:	and    %rdi,%r11
   241da:	inc    %r11
   241dd:	and    $0xfffffffffffffff0,%r11
   241e1:	mov    0x98(%rsp),%rsi
   241e9:	lea    (%r9,%rsi,1),%rdi
   241ed:	lea    (%rcx,%rdi,1),%rbp
   241f1:	add    $0x10,%rbp
   241f5:	xor    %edi,%edi
fragment<false>():
   241f7:	nopw   0x0(%rax,%rax,1)
   24200:	movdqa -0x10(%r10,%rdi,2),%xmm1
./examples/c30walk/ctor.rs:457
   24207:	paddw  %xmm0,%xmm1
   2420b:	movdqa (%r10,%rdi,2),%xmm2
   24211:	paddw  %xmm0,%xmm2
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   24215:	movdqu %xmm1,-0x10(%rbp,%rdi,2)
write<u16>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   2421b:	movdqu %xmm2,0x0(%rbp,%rdi,2)
next<core::slice::iter::Iter<u16>>():
/rustc/2d8144b78/library/core/src/iter/adapters/enumerate.rs:82
   24221:	add    $0x10,%rdi
next<u16>():
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   24225:	cmp    %rdi,%r11
   24228:	jne    24200 <<c30walk::ctor::Ctor>::build::<false>+0xdf0>
   2422a:	mov    0xa0(%rsp),%rsi
   24232:	cmp    %rax,%rsi
   24235:	mov    0x1150(%rsp),%r11
   2423d:	mov    0x38(%rsp),%ebp
   24241:	je     24357 <<c30walk::ctor::Ctor>::build::<false>+0xf47>
fragment<false>():
   24247:	test   $0xc,%sil
   2424b:	je     249b6 <<c30walk::ctor::Ctor>::build::<false>+0x15a6>
   24251:	mov    0x40(%rsp),%r10
   24256:	mov    %rax,%rbp
   24259:	mov    %rsi,%rax
   2425c:	and    $0xfffffffffffffffc,%rax
   24260:	mov    0x58(%rsp),%rsi
   24265:	lea    (%rsi,%rbp,2),%r11
   24269:	lea    -0x14bb0(%rip),%rdi        # f6c0 <c30walk::ctor::PATTERNS>
   24270:	add    %rdi,%r11
   24273:	mov    0xa8(%rsp),%r9
   2427b:	not    %r9d
   2427e:	lea    (%r10,%r15,1),%rdi
   24282:	cmp    $0x1e,%rdi
   24286:	mov    $0x1e,%r10d
   2428c:	cmovae %r10,%rdi
   24290:	movabs $0x7fffffffffffffe0,%r10
   2429a:	or     %r10,%r9
   2429d:	add    %rdi,%r9
   242a0:	mov    0x68(%rsp),%rsi
   242a5:	lea    (%rsi,%rax,2),%r10
   242a9:	mov    %r9,%rsi
   242ac:	movd   %edx,%xmm0
   242b0:	pshuflw $0x0,%xmm0,%xmm0
   242b5:	movabs $0x7fffffffffffffff,%rdi
   242bf:	and    %rdi,%rsi
   242c2:	inc    %rsi
   242c5:	and    $0xfffffffffffffffc,%rsi
   242c9:	sub    %rbp,%rsi
   242cc:	mov    0x80(%rsp),%r9
   242d4:	lea    (%r9,%rbp,2),%rbp
   242d8:	add    0x98(%rsp),%rbp
   242e0:	add    %rcx,%rbp
   242e3:	xor    %edi,%edi
   242e5:	data16 cs nopw 0x0(%rax,%rax,1)
./examples/c30walk/ctor.rs:456
   242f0:	movq   (%r11,%rdi,2),%xmm1
./examples/c30walk/ctor.rs:457
   242f6:	paddw  %xmm0,%xmm1
write<u16>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   242fa:	movq   %xmm1,0x0(%rbp,%rdi,2)
next<u16>():
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   24300:	add    $0x4,%rdi
   24304:	cmp    %rdi,%rsi
   24307:	jne    242f0 <<c30walk::ctor::Ctor>::build::<false>+0xee0>
   24309:	cmp    %rax,0xa0(%rsp)
   24311:	mov    0x1150(%rsp),%r11
   24319:	mov    0x38(%rsp),%ebp
   2431d:	je     24357 <<c30walk::ctor::Ctor>::build::<false>+0xf47>
fragment<false>():
   2431f:	mov    0x68(%rsp),%rsi
./examples/c30walk/ctor.rs:456
   24324:	lea    (%rsi,%r8,2),%rsi
next<u16>():
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   24328:	lea    (%r9,%rax,2),%rax
   2432c:	mov    0x40(%rsp),%rdi
   24331:	lea    (%rax,%rdi,2),%rax
   24335:	add    %rax,%rcx
fragment<false>():
   24338:	nopl   0x0(%rax,%rax,1)
   24340:	movzwl (%r10),%eax
./examples/c30walk/ctor.rs:457
   24344:	add    %dx,%ax
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/non_null.rs:656
   24347:	add    $0x2,%r10
write<u16>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   2434b:	mov    %ax,(%rcx)
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/non_null.rs:1714
   2434e:	add    $0x2,%rcx
eq<u16>():
/rustc/2d8144b78/library/core/src/ptr/non_null.rs:1714
   24352:	cmp    %rsi,%r10

# fragment<false>, RAW entries: (y + pattern) << 8 | c, 8 entries an iteration, then 1
   2401c:	xor    %edx,%edx
   2401e:	mov    0x30(%rsp),%r8
   24023:	mov    0x58(%rsp),%rsi
/rustc/2d8144b78/library/core/src/iter/adapters/zip.rs:304
   24028:	lea    (%rsi,%rdx,2),%rsi
   2402c:	lea    -0x14973(%rip),%rdi        # f6c0 <c30walk::ctor::PATTERNS>
   24033:	add    %rdi,%rsi
fragment<false>():
   24036:	cs nopw 0x0(%rax,%rax,1)
./examples/c30walk/ctor.rs:465
   24040:	movzwl (%rsi),%edi
./examples/c30walk/ctor.rs:466
   24043:	add    0x18(%rsp),%edi
   24047:	shl    $0x8,%edi
   2404a:	or     %ecx,%edi
   2404c:	mov    %edi,(%rax,%rdx,4)
next<core::slice::iter::IterMut<u32>, core::slice::iter::Iter<u16>>():
/rustc/2d8144b78/library/core/src/iter/adapters/zip.rs:308
   2404f:	inc    %rdx
/rustc/2d8144b78/library/core/src/iter/adapters/zip.rs:304
   24052:	add    $0x2,%rsi
   24056:	cmp    %r8,%rdx
   24059:	jb     24040 <<c30walk::ctor::Ctor>::build::<false>+0xc30>
   2405b:	jmp    24357 <<c30walk::ctor::Ctor>::build::<false>+0xf47>
next<u16>():
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   24060:	mov    %rcx,%rdi
   24063:	lea    -0x149aa(%rip),%r9        # f6c0 <c30walk::ctor::PATTERNS>
   2406a:	sub    %r9,%rdi
   2406d:	mov    0x80(%rsp),%r9
   24075:	add    %r9,%rdi
   24078:	lea    (%r10,%r10,1),%esi
   2407c:	add    %rsi,%rdi
   2407f:	sub    0x58(%rsp),%rdi
   24084:	cmp    $0x20,%rdi
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/iter/adapters/enumerate.rs:82
   24088:	jae    24096 <<c30walk::ctor::Ctor>::build::<false>+0xc86>
next<core::slice::iter::Iter<u16>>():
   2408a:	xor    %eax,%eax
   2408c:	mov    0x68(%rsp),%r10
/rustc/2d8144b78/library/core/src/iter/adapters/enumerate.rs:82
   24091:	jmp    2431f <<c30walk::ctor::Ctor>::build::<false>+0xf0f>
   24096:	mov    %rsi,0x98(%rsp)
   2409e:	lea    0x1(%rax),%rsi
next<u16>():
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   240a2:	cmp    $0xf,%rax
   240a6:	mov    %ebp,0x38(%rsp)
   240aa:	mov    %rsi,0xa0(%rsp)
   240b2:	jae    24178 <<c30walk::ctor::Ctor>::build::<false>+0xd68>
   240b8:	xor    %eax,%eax
   240ba:	jmp    24256 <<c30walk::ctor::Ctor>::build::<false>+0xe46>
fragment<false>():
   240bf:	mov    0x30(%rsp),%rdx
   240c4:	and    $0x18,%edx
   240c7:	movd   %ecx,%xmm0
   240cb:	pshufd $0x0,%xmm0,%xmm0
   240d0:	movd   0x18(%rsp),%xmm1
   240d6:	pshufd $0x0,%xmm1,%xmm1
   240db:	mov    0x40(%rsp),%r10
next<core::slice::iter::IterMut<u32>, core::slice::iter::Iter<u16>>():
/rustc/2d8144b78/library/core/src/iter/adapters/zip.rs:308
   240e0:	lea    (%r10,%r15,1),%rsi
   240e4:	cmp    $0x1e,%rsi
   240e8:	mov    $0x1e,%r9d
   240ee:	cmovae %r9,%rsi
   240f2:	sub    %r10d,%esi
   240f5:	and    $0x18,%esi
   240f8:	lea    -0x14a3f(%rip),%r9        # f6c0 <c30walk::ctor::PATTERNS>
   240ff:	mov    0x58(%rsp),%r10
   24104:	add    %r10,%r9
   24107:	add    $0x8,%r9
   2410b:	mov    0x40(%rsp),%r10
   24110:	lea    (%rdi,%r10,4),%rdi
   24114:	add    %r8,%rdi
   24117:	add    $0x10,%rdi
   2411b:	xor    %r8d,%r8d
fragment<false>():
   2411e:	xchg   %ax,%ax
./examples/c30walk/ctor.rs:465
   24120:	movq   -0x8(%r9,%r8,2),%xmm2
   24127:	movq   (%r9,%r8,2),%xmm3
./examples/c30walk/ctor.rs:466
   2412d:	punpcklwd %xmm4,%xmm2
   24131:	punpcklwd %xmm4,%xmm3
   24135:	paddd  %xmm1,%xmm2
   24139:	paddd  %xmm1,%xmm3
   2413d:	pslld  $0x8,%xmm2
   24142:	pslld  $0x8,%xmm3
   24147:	por    %xmm0,%xmm2
   2414b:	por    %xmm0,%xmm3
   2414f:	movdqu %xmm2,-0x10(%rdi,%r8,4)
   24156:	movdqu %xmm3,(%rdi,%r8,4)
next<core::slice::iter::IterMut<u32>, core::slice::iter::Iter<u16>>():
/rustc/2d8144b78/library/core/src/iter/adapters/zip.rs:308
   2415c:	add    $0x8,%r8
/rustc/2d8144b78/library/core/src/iter/adapters/zip.rs:304
   24160:	cmp    %r8,%rsi
   24163:	jne    24120 <<c30walk::ctor::Ctor>::build::<false>+0xd10>
   24165:	mov    0x30(%rsp),%r8
   2416a:	cmp    %rdx,%r8
   2416d:	jne    24023 <<c30walk::ctor::Ctor>::build::<false>+0xc13>
   24173:	jmp    24357 <<c30walk::ctor::Ctor>::build::<false>+0xf47>

# fragment<false>, j == 0 (suffix): header, then the pattern's first k fields (scalar for short fragments, else 16/4-field vector loops)
   245e3:	jmp    2476c <<c30walk::ctor::Ctor>::build::<false>+0x135c>
fragment<false>():
./examples/c30walk/ctor.rs:433
   245e8:	mov    %r8d,%eax
   245eb:	shl    $0x8,%eax
   245ee:	movzbl 0x70(%rsp),%edx
   245f3:	or     %eax,%edx
<c30walk::ctor::Ctor>::build::<false>:
./examples/c30walk/ctor.rs:337
   245f5:	mov    0xf8(%rsp),%rcx
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   245fd:	shl    $0x6,%r9
add<c30walk::Rec>():
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   24601:	mov    %r9,0x8(%rsp)
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   24606:	mov    %edx,0x3c(%rcx,%r9,1)
write<u32>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   2460b:	cmp    $0x1f,%rsi
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/num/uint_macros.rs:1050
   2460f:	jae    24e8e <<c30walk::ctor::Ctor>::build::<false>+0x1a7e>
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   24615:	imul   $0xffffffffffffffc4,0x60(%rsp),%rax
next<u16>():
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   2461b:	mov    0x10(%rsp),%rdx
   24620:	lea    (%rax,%rdx,2),%rax
   24624:	add    $0xfffffffffffffffe,%rax
   24628:	cmp    $0x6,%rax
   2462c:	jb     24653 <<c30walk::ctor::Ctor>::build::<false>+0x1243>
   2462e:	mov    %r10,%r9
   24631:	mov    %r11,%r10
   24634:	mov    %rcx,%rdx
   24637:	lea    -0x14f7e(%rip),%r11        # f6c0 <c30walk::ctor::PATTERNS>
   2463e:	sub    %r11,%rdx
   24641:	add    0x8(%rsp),%rdx
   24646:	sub    %r9,%rdx
   24649:	cmp    $0x20,%rdx
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/iter/adapters/enumerate.rs:82
   2464d:	jae    24893 <<c30walk::ctor::Ctor>::build::<false>+0x1483>
fragment<false>():
   24653:	mov    %rdi,%rax
   24656:	xor    %r14d,%r14d
./examples/c30walk/ctor.rs:435
   24659:	lea    (%rdi,%rsi,2),%rdx
   2465d:	mov    0x8(%rsp),%r9
next<u16>():
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   24662:	lea    (%r9,%r14,2),%r9
   24666:	add    %r9,%rcx
fragment<false>():
   24669:	nopl   0x0(%rax)
./examples/c30walk/ctor.rs:435
   24670:	movzwl (%rax),%r9d
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/non_null.rs:656
   24674:	add    $0x2,%rax
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   24678:	mov    %r9w,(%rcx)
/rustc/2d8144b78/library/core/src/ptr/non_null.rs:1714
   2467c:	add    $0x2,%rcx
eq<u16>():
/rustc/2d8144b78/library/core/src/ptr/non_null.rs:1714
   24680:	cmp    %rdx,%rax
next<u16>():
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   24683:	jne    24670 <<c30walk::ctor::Ctor>::build::<false>+0x1260>
fragment<false>():
   24685:	mov    0x18(%rsp),%rdx
   24893:	mov    %rax,%r11
   24896:	shr    $1,%r11
   24899:	inc    %r11
next<u16>():
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   2489c:	cmp    $0x1e,%rax
   248a0:	jae    248aa <<c30walk::ctor::Ctor>::build::<false>+0x149a>
   248a2:	xor    %r14d,%r14d
   248a5:	jmp    24939 <<c30walk::ctor::Ctor>::build::<false>+0x1529>
   248aa:	mov    %r11,0x10(%rsp)
   248af:	mov    %r11,%r14
   248b2:	and    $0xfffffffffffffff0,%r14
   248b6:	lea    -0x151fd(%rip),%rax        # f6c0 <c30walk::ctor::PATTERNS>
   248bd:	lea    (%r9,%rax,1),%rbp
   248c1:	add    $0x10,%rbp
   248c5:	lea    (%r10,%r15,1),%rax
   248c9:	lea    (%rax,%rax,1),%r11
   248cd:	movabs $0x8888888888888889,%rdx
   248d7:	mul    %rdx
   248da:	shr    $0x4,%rdx
   248de:	imul   $0x3c,%rdx,%rax
   248e2:	sub    %rax,%r11
   248e5:	add    $0xfffffffffffffffe,%r11
   248e9:	shr    $1,%r11
   248ec:	inc    %r11
   248ef:	and    $0xfffffffffffffff0,%r11
   248f3:	mov    0x8(%rsp),%rax
   248f8:	add    %rcx,%rax
   248fb:	add    $0x10,%rax
   248ff:	xor    %edx,%edx
fragment<false>():
./examples/c30walk/ctor.rs:435
   24901:	movdqa -0x10(%rbp,%rdx,2),%xmm0
   24907:	movdqa 0x0(%rbp,%rdx,2),%xmm1
write<u16>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   2490d:	movdqu %xmm0,-0x10(%rax,%rdx,2)
   24913:	movdqu %xmm1,(%rax,%rdx,2)
next<core::slice::iter::Iter<u16>>():
/rustc/2d8144b78/library/core/src/iter/adapters/enumerate.rs:82
   24918:	add    $0x10,%rdx
next<u16>():
/rustc/2d8144b78/library/core/src/slice/iter/macros.rs:180
   2491c:	cmp    %rdx,%r11
   2491f:	jne    24901 <<c30walk::ctor::Ctor>::build::<false>+0x14f1>
   24921:	mov    0x10(%rsp),%r11
   24926:	cmp    %r14,%r11
   24929:	je     24685 <<c30walk::ctor::Ctor>::build::<false>+0x1275>

# one<false>, j > 0: header read, RAW test, offset check, one 2-byte store
   23cd7:	or     %eax,%r14d
./examples/c30walk/ctor.rs:611
   23cda:	test   %r13d,%r13d
   23cdd:	je     23d30 <<c30walk::ctor::Ctor>::build::<false>+0x920>
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   23cdf:	mov    %rsi,%rcx
add<c30walk::Rec>():
/rustc/2d8144b78/library/core/src/ptr/mut_ptr.rs:961
   23ce2:	shl    $0x6,%rcx
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1733
   23ce6:	mov    0x3c(%rbx,%rcx,1),%eax
one<false>():
./examples/c30walk/ctor.rs:621
   23cea:	test   $0x10000000,%eax
   23cef:	jne    23d70 <<c30walk::ctor::Ctor>::build::<false>+0x960>
   23cf1:	mov    %rdx,%rbx
./examples/c30walk/ctor.rs:625
   23cf4:	mov    %eax,%edi
   23cf6:	shr    $0x8,%edi
   23cf9:	mov    %r12d,%edx
   23cfc:	sub    %edi,%edx
./examples/c30walk/ctor.rs:626
   23cfe:	cmp    $0xfffe,%edx
   23d04:	jae    23d9a <<c30walk::ctor::Ctor>::build::<false>+0x98a>
   23d0a:	mov    0x8(%rsp),%rax
   23d0f:	add    %rax,%rcx
<c30walk::ctor::Ctor>::build::<false>:
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   23d12:	mov    %dx,(%rcx,%r13,2)
write<u16>():
/rustc/2d8144b78/library/core/src/ptr/mod.rs:1933
   23d17:	mov    %rbx,%rdx
   23d1a:	mov    %rax,%rbx
one<false>():
