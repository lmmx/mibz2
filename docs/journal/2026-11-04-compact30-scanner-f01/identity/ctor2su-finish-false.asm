# <c30walk::ctor2su::W<false>>::finish
# size 0x5fa

target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

0000000000097060 <<c30walk::ctor2su::W<false>>::finish>:
   97060:	push   %rbp
   97061:	push   %r15
   97063:	push   %r14
   97065:	push   %r13
   97067:	push   %r12
   97069:	push   %rbx
   9706a:	sub    $0x38,%rsp
   9706e:	mov    0x18(%rdi),%rbp
   97072:	mov    0x10(%rbp),%r8
   97076:	mov    %r8,%r13
   97079:	sub    $0x1,%r13
   9707d:	jb     975ea <<c30walk::ctor2su::W<false>>::finish+0x58a>
   97083:	mov    %rdi,%r11
   97086:	lea    (%rsi,%rsi,4),%rax
   9708a:	mov    0x40(%rdi,%rax,8),%r14
   9708f:	mov    0x50(%rdi,%rax,8),%rdi
   97094:	mov    0x58(%r11,%rax,8),%r12d
   97099:	mov    0x60(%r11,%rax,8),%ecx
   9709e:	mov    0x64(%r11,%rax,8),%eax
   970a3:	sub    %rdi,%r14
   970a6:	mov    %r14,0x8(%rsp)
   970ab:	shr    $1,%r14
   970ae:	mov    0x8(%rbp),%r10
   970b2:	mov    %r13,%rdx
   970b5:	shl    $0xa,%rdx
   970b9:	add    %r10,%rdx
   970bc:	mov    %r13,%rbx
   970bf:	cmp    %r12d,(%rdx,%rsi,4)
   970c3:	jbe    97138 <<c30walk::ctor2su::W<false>>::finish+0xd8>
   970c5:	mov    %r11,%r15
   970c8:	lea    (%r12,%r14,1),%edx
   970cc:	dec    %edx
   970ce:	mov    %r8,%r9
   970d1:	shl    $0xa,%r9
   970d5:	lea    (%r9,%rsi,4),%r9
   970d9:	lea    (%r10,%r9,1),%r11
   970dd:	add    $0xfffffffffffffc00,%r11
   970e4:	mov    %r8,%rbx
   970e7:	nopw   0x0(%rax,%rax,1)
   970f0:	dec    %rbx
   970f3:	cmp    %r8,%rbx
   970f6:	jae    9759e <<c30walk::ctor2su::W<false>>::finish+0x53e>
   970fc:	cmp    %r12d,(%r11)
   970ff:	lea    -0x400(%r11),%r11
   97106:	ja     970f0 <<c30walk::ctor2su::W<false>>::finish+0x90>
   97108:	add    %r10,%r9
   9710b:	add    $0xfffffffffffffc00,%r9
   97112:	mov    %r8,%r13
   97115:	mov    %r15,%r11
   97118:	nopl   0x0(%rax,%rax,1)
   97120:	dec    %r13
   97123:	cmp    %r8,%r13
   97126:	jae    975b0 <<c30walk::ctor2su::W<false>>::finish+0x550>
   9712c:	cmp    %edx,(%r9)
   9712f:	lea    -0x400(%r9),%r9
   97136:	ja     97120 <<c30walk::ctor2su::W<false>>::finish+0xc0>
   97138:	imul   $0xffffffffffffffe2,%rcx,%r15
   9713c:	add    %r12,%r15
   9713f:	cmp    $0xffffffff,%eax
   97142:	mov    %r14,0x10(%rsp)
   97147:	je     9724b <<c30walk::ctor2su::W<false>>::finish+0x1eb>
   9714d:	mov    $0xffffffff,%r9d
   97153:	cmp    %r13,%rbx
   97156:	jae    97170 <<c30walk::ctor2su::W<false>>::finish+0x110>
   97158:	lea    0x1(%rbx),%rcx
   9715c:	cmp    %r8,%rcx
   9715f:	jae    9760d <<c30walk::ctor2su::W<false>>::finish+0x5ad>
   97165:	shl    $0xa,%rcx
   97169:	add    %rcx,%r10
   9716c:	mov    (%r10,%rsi,4),%r9d
   97170:	mov    (%r11),%rdx
   97173:	mov    0x10(%rdx),%rcx
   97177:	cmp    %rax,%rcx
   9717a:	jbe    975fb <<c30walk::ctor2su::W<false>>::finish+0x59b>
   97180:	test   %r14,%r14
   97183:	je     9764b <<c30walk::ctor2su::W<false>>::finish+0x5eb>
   97189:	shl    $0x7,%rax
   9718d:	add    0x8(%rdx),%rax
   97191:	lea    0x800(,%rsi,4),%rdx
   97199:	xor    %r11d,%r11d
   9719c:	nopl   0x0(%rax)
   971a0:	lea    (%r12,%r11,1),%r10d
   971a4:	cmp    %r9d,%r10d
   971a7:	jae    971e0 <<c30walk::ctor2su::W<false>>::finish+0x180>
   971a9:	lea    (%r11,%r15,1),%rcx
   971ad:	cmp    $0x20,%rcx
   971b1:	jae    975c2 <<c30walk::ctor2su::W<false>>::finish+0x562>
   971b7:	movzwl (%rdi,%r11,2),%r8d
   971bc:	inc    %r11
   971bf:	mov    %ebx,%r10d
   971c2:	shl    $0x18,%r10d
   971c6:	shl    $0x8,%r8d
   971ca:	or     %r10d,%r8d
   971cd:	or     %esi,%r8d
   971d0:	mov    %r8d,(%rax,%rcx,4)
   971d4:	cmp    0x10(%rsp),%r11
   971d9:	jne    971a0 <<c30walk::ctor2su::W<false>>::finish+0x140>
   971db:	jmp    97229 <<c30walk::ctor2su::W<false>>::finish+0x1c9>
   971dd:	nopl   (%rax)
   971e0:	mov    %rbx,%r14
   971e3:	shl    $0xa,%r14
   971e7:	add    %rdx,%r14
   971ea:	mov    %rbx,%rcx
   971ed:	jmp    971ff <<c30walk::ctor2su::W<false>>::finish+0x19f>
   971ef:	nop
   971f0:	add    $0x400,%r14
   971f7:	mov    %rbx,%rcx
   971fa:	cmp    %r9d,%r10d
   971fd:	jb     971a9 <<c30walk::ctor2su::W<false>>::finish+0x149>
   971ff:	lea    0x1(%rcx),%rbx
   97203:	mov    $0xffffffff,%r9d
   97209:	cmp    %r13,%rbx
   9720c:	jae    971f0 <<c30walk::ctor2su::W<false>>::finish+0x190>
   9720e:	add    $0x2,%rcx
   97212:	mov    0x10(%rbp),%r8
   97216:	cmp    %r8,%rcx
   97219:	jae    9758c <<c30walk::ctor2su::W<false>>::finish+0x52c>
   9721f:	mov    0x8(%rbp),%rcx
   97223:	mov    (%rcx,%r14,1),%r9d
   97227:	jmp    971f0 <<c30walk::ctor2su::W<false>>::finish+0x190>
   97229:	mov    0x8(%rsp),%rdx
   9722e:	and    $0xfffffffffffffffe,%rdx
   97232:	mov    $0xff,%esi
   97237:	add    $0x38,%rsp
   9723b:	pop    %rbx
   9723c:	pop    %r12
   9723e:	pop    %r13
   97240:	pop    %r14
   97242:	pop    %r15
   97244:	pop    %rbp
   97245:	jmp    *0xb6e25(%rip)        # 14e070 <memset@GLIBC_2.2.5>
   9724b:	movzwl (%rdi),%eax
   9724e:	movzwl -0x2(%rdi,%r14,2),%r8d
   97254:	mov    %ebx,%edx
   97256:	shl    $0x10,%edx
   97259:	or     %eax,%edx
   9725b:	mov    %r13d,%r9d
   9725e:	shl    $0x10,%r9d
   97262:	or     %r8d,%r9d
   97265:	sub    %edx,%r9d
   97268:	cmp    $0xfffe,%r9d
   9726f:	jae    972ed <<c30walk::ctor2su::W<false>>::finish+0x28d>
   97271:	mov    0x28(%r11),%r8
   97275:	shl    $0x6,%rcx
   97279:	shl    $0x8,%edx
   9727c:	or     %edx,%esi
   9727e:	mov    %esi,0x3c(%r8,%rcx,1)
   97283:	test   %r14,%r14
   97286:	je     9764b <<c30walk::ctor2su::W<false>>::finish+0x5eb>
   9728c:	cmp    $0x1e,%r14
   97290:	jne    97486 <<c30walk::ctor2su::W<false>>::finish+0x426>
   97296:	movd   %eax,%xmm0
   9729a:	pshuflw $0x0,%xmm0,%xmm0
   9729f:	pshufd $0x44,%xmm0,%xmm1
   972a4:	movdqu (%rdi),%xmm2
   972a8:	movdqu 0x10(%rdi),%xmm3
   972ad:	movdqu 0x20(%rdi),%xmm4
   972b2:	psubw  %xmm1,%xmm2
   972b6:	movdqu %xmm2,(%rdi)
   972ba:	psubw  %xmm1,%xmm3
   972be:	movdqu %xmm3,0x10(%rdi)
   972c3:	psubw  %xmm1,%xmm4
   972c7:	movdqu %xmm4,0x20(%rdi)
   972cc:	movq   0x30(%rdi),%xmm1
   972d1:	psubw  %xmm0,%xmm1
   972d5:	movq   %xmm1,0x30(%rdi)
   972da:	movd   0x38(%rdi),%xmm1
   972df:	psubw  %xmm0,%xmm1
   972e3:	movd   %xmm1,0x38(%rdi)
   972e8:	jmp    9764b <<c30walk::ctor2su::W<false>>::finish+0x5eb>
   972ed:	mov    (%r11),%r10
   972f0:	mov    0x10(%r10),%r9
   972f4:	cmp    (%r10),%r9
   972f7:	mov    %r11,0x30(%rsp)
   972fc:	je     9754c <<c30walk::ctor2su::W<false>>::finish+0x4ec>
   97302:	mov    0x8(%r10),%rax
   97306:	mov    %r9,%rdx
   97309:	shl    $0x7,%rdx
   9730d:	pxor   %xmm0,%xmm0
   97311:	movdqa %xmm0,0x70(%rax,%rdx,1)
   97317:	movdqa %xmm0,0x60(%rax,%rdx,1)
   9731d:	movdqa %xmm0,0x50(%rax,%rdx,1)
   97323:	movdqa %xmm0,0x40(%rax,%rdx,1)
   97329:	movdqa %xmm0,0x30(%rax,%rdx,1)
   9732f:	movdqa %xmm0,0x20(%rax,%rdx,1)
   97335:	movdqa %xmm0,0x10(%rax,%rdx,1)
   9733b:	movdqa %xmm0,(%rax,%rdx,1)
   97340:	lea    0x1(%r9),%rax
   97344:	mov    %rax,0x10(%r10)
   97348:	mov    $0xffffffff,%r8d
   9734e:	cmp    %r13,%rbx
   97351:	jae    97370 <<c30walk::ctor2su::W<false>>::finish+0x310>
   97353:	lea    0x1(%rbx),%rax
   97357:	mov    0x10(%rbp),%r8
   9735b:	cmp    %r8,%rax
   9735e:	jae    9762b <<c30walk::ctor2su::W<false>>::finish+0x5cb>
   97364:	shl    $0xa,%rax
   97368:	add    0x8(%rbp),%rax
   9736c:	mov    (%rax,%rsi,4),%r8d
   97370:	mov    %r9,0x8(%rsp)
   97375:	test   %r14,%r14
   97378:	je     97425 <<c30walk::ctor2su::W<false>>::finish+0x3c5>
   9737e:	add    0x8(%r10),%rdx
   97382:	lea    0x800(,%rsi,4),%r9
   9738a:	xor    %r10d,%r10d
   9738d:	nopl   (%rax)
   97390:	lea    (%r12,%r10,1),%r11d
   97394:	cmp    %r8d,%r11d
   97397:	jae    973d0 <<c30walk::ctor2su::W<false>>::finish+0x370>
   97399:	lea    (%r10,%r15,1),%rax
   9739d:	cmp    $0x20,%rax
   973a1:	jae    975d6 <<c30walk::ctor2su::W<false>>::finish+0x576>
   973a7:	movzwl (%rdi,%r10,2),%r11d
   973ac:	inc    %r10
   973af:	mov    %ebx,%r14d
   973b2:	shl    $0x18,%r14d
   973b6:	shl    $0x8,%r11d
   973ba:	or     %r14d,%r11d
   973bd:	or     %esi,%r11d
   973c0:	mov    %r11d,(%rdx,%rax,4)
   973c4:	cmp    0x10(%rsp),%r10
   973c9:	jne    97390 <<c30walk::ctor2su::W<false>>::finish+0x330>
   973cb:	jmp    97425 <<c30walk::ctor2su::W<false>>::finish+0x3c5>
   973cd:	nopl   (%rax)
   973d0:	mov    %rbx,%r14
   973d3:	shl    $0xa,%r14
   973d7:	add    %r9,%r14
   973da:	mov    %rbx,%rax
   973dd:	jmp    973f7 <<c30walk::ctor2su::W<false>>::finish+0x397>
   973df:	nop
   973e0:	mov    0x8(%rbp),%rax
   973e4:	mov    (%rax,%r14,1),%r8d
   973e8:	add    $0x400,%r14
   973ef:	mov    %rbx,%rax
   973f2:	cmp    %r8d,%r11d
   973f5:	jb     97399 <<c30walk::ctor2su::W<false>>::finish+0x339>
   973f7:	lea    0x1(%rax),%rbx
   973fb:	mov    $0xffffffff,%r8d
   97401:	cmp    %r13,%rbx
   97404:	jae    973e8 <<c30walk::ctor2su::W<false>>::finish+0x388>
   97406:	add    $0x2,%rax
   9740a:	mov    0x10(%rbp),%r8
   9740e:	cmp    %r8,%rax
   97411:	jb     973e0 <<c30walk::ctor2su::W<false>>::finish+0x380>
   97413:	lea    0xb210e(%rip),%rdx        # 149528 <__frame_dummy_init_array_entry+0x2308>
   9741a:	mov    %rax,%rdi
   9741d:	mov    %r8,%rsi
   97420:	call   f0803 <core::panicking::panic_bounds_check>
   97425:	mov    0x30(%rsp),%r8
   9742a:	mov    0x28(%r8),%rax
   9742e:	mov    %rcx,%rdx
   97431:	shl    $0x6,%rdx
   97435:	mov    0x8(%rsp),%rsi
   9743a:	or     $0x10000000,%esi
   97440:	pcmpeqd %xmm0,%xmm0
   97444:	movdqu %xmm0,0x2c(%rax,%rdx,1)
   9744a:	movdqa %xmm0,0x20(%rax,%rdx,1)
   97450:	movdqa %xmm0,0x10(%rax,%rdx,1)
   97456:	movdqa %xmm0,(%rax,%rdx,1)
   9745b:	mov    %esi,0x3c(%rax,%rdx,1)
   9745f:	mov    %ecx,%edi
   97461:	shr    $0x6,%edi
   97464:	mov    0x10(%r8),%rsi
   97468:	cmp    %rsi,%rdi
   9746b:	jae    9761f <<c30walk::ctor2su::W<false>>::finish+0x5bf>
   97471:	mov    $0x1,%eax
   97476:	shl    %cl,%rax
   97479:	mov    0x8(%r8),%rcx
   9747d:	or     %rax,(%rcx,%rdi,8)
   97481:	jmp    9764b <<c30walk::ctor2su::W<false>>::finish+0x5eb>
   97486:	mov    0x8(%rsp),%rcx
   9748b:	cmp    $0x8,%rcx
   9748f:	jae    97498 <<c30walk::ctor2su::W<false>>::finish+0x438>
   97491:	xor    %ecx,%ecx
   97493:	jmp    97642 <<c30walk::ctor2su::W<false>>::finish+0x5e2>
   97498:	movabs $0x7ffffffffffffff0,%rsi
   974a2:	cmp    $0x20,%rcx
   974a6:	jae    974ac <<c30walk::ctor2su::W<false>>::finish+0x44c>
   974a8:	xor    %ecx,%ecx
   974aa:	jmp    9750b <<c30walk::ctor2su::W<false>>::finish+0x4ab>
   974ac:	mov    %r14,%rcx
   974af:	and    %rsi,%rcx
   974b2:	movd   %eax,%xmm0
   974b6:	pshuflw $0x0,%xmm0,%xmm0
   974bb:	pshufd $0x44,%xmm0,%xmm0
   974c0:	xor    %edx,%edx
   974c2:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   974d0:	movdqu (%rdi,%rdx,2),%xmm1
   974d5:	movdqu 0x10(%rdi,%rdx,2),%xmm2
   974db:	psubw  %xmm0,%xmm1
   974df:	psubw  %xmm0,%xmm2
   974e3:	movdqu %xmm1,(%rdi,%rdx,2)
   974e8:	movdqu %xmm2,0x10(%rdi,%rdx,2)
   974ee:	add    $0x10,%rdx
   974f2:	cmp    %rdx,%rcx
   974f5:	jne    974d0 <<c30walk::ctor2su::W<false>>::finish+0x470>
   974f7:	cmp    %rcx,%r14
   974fa:	je     9764b <<c30walk::ctor2su::W<false>>::finish+0x5eb>
   97500:	testb  $0x18,0x8(%rsp)
   97505:	je     97642 <<c30walk::ctor2su::W<false>>::finish+0x5e2>
   9750b:	mov    %rcx,%rdx
   9750e:	add    $0xc,%rsi
   97512:	mov    %rsi,%rcx
   97515:	and    %r14,%rcx
   97518:	movd   %eax,%xmm0
   9751c:	pshuflw $0x0,%xmm0,%xmm0
   97521:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   97530:	movq   (%rdi,%rdx,2),%xmm1
   97535:	psubw  %xmm0,%xmm1
   97539:	movq   %xmm1,(%rdi,%rdx,2)
   9753e:	add    $0x4,%rdx
   97542:	cmp    %rdx,%rcx
   97545:	jne    97530 <<c30walk::ctor2su::W<false>>::finish+0x4d0>
   97547:	jmp    9763d <<c30walk::ctor2su::W<false>>::finish+0x5dd>
   9754c:	mov    %rdi,0x8(%rsp)
   97551:	mov    %r10,%rdi
   97554:	mov    %rsi,0x20(%rsp)
   97559:	mov    %rcx,0x28(%rsp)
   9755e:	mov    %r9,%r14
   97561:	mov    %r10,0x18(%rsp)
   97566:	call   9c910 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   9756b:	mov    0x18(%rsp),%r10
   97570:	mov    %r14,%r9
   97573:	mov    0x28(%rsp),%rcx
   97578:	mov    0x20(%rsp),%rsi
   9757d:	mov    0x10(%rsp),%r14
   97582:	mov    0x8(%rsp),%rdi
   97587:	jmp    97302 <<c30walk::ctor2su::W<false>>::finish+0x2a2>
   9758c:	lea    0xb1f95(%rip),%rdx        # 149528 <__frame_dummy_init_array_entry+0x2308>
   97593:	mov    %rcx,%rdi
   97596:	mov    %r8,%rsi
   97599:	call   f0803 <core::panicking::panic_bounds_check>
   9759e:	lea    0xb208b(%rip),%rdx        # 149630 <__frame_dummy_init_array_entry+0x2410>
   975a5:	mov    %rbx,%rdi
   975a8:	mov    %r8,%rsi
   975ab:	call   f0803 <core::panicking::panic_bounds_check>
   975b0:	lea    0xb2079(%rip),%rdx        # 149630 <__frame_dummy_init_array_entry+0x2410>
   975b7:	mov    %r13,%rdi
   975ba:	mov    %r8,%rsi
   975bd:	call   f0803 <core::panicking::panic_bounds_check>
   975c2:	lea    0xb1f47(%rip),%rdx        # 149510 <__frame_dummy_init_array_entry+0x22f0>
   975c9:	mov    $0x20,%esi
   975ce:	mov    %rcx,%rdi
   975d1:	call   f0803 <core::panicking::panic_bounds_check>
   975d6:	lea    0xb1f33(%rip),%rdx        # 149510 <__frame_dummy_init_array_entry+0x22f0>
   975dd:	mov    $0x20,%esi
   975e2:	mov    %rax,%rdi
   975e5:	call   f0803 <core::panicking::panic_bounds_check>
   975ea:	lea    0xb1faf(%rip),%rdx        # 1495a0 <__frame_dummy_init_array_entry+0x2380>
   975f1:	mov    %r13,%rdi
   975f4:	xor    %esi,%esi
   975f6:	call   f0803 <core::panicking::panic_bounds_check>
   975fb:	lea    0xb1ef6(%rip),%rdx        # 1494f8 <__frame_dummy_init_array_entry+0x22d8>
   97602:	mov    %rax,%rdi
   97605:	mov    %rcx,%rsi
   97608:	call   f0803 <core::panicking::panic_bounds_check>
   9760d:	lea    0xb1ecc(%rip),%rdx        # 1494e0 <__frame_dummy_init_array_entry+0x22c0>
   97614:	mov    %rcx,%rdi
   97617:	mov    %r8,%rsi
   9761a:	call   f0803 <core::panicking::panic_bounds_check>
   9761f:	lea    0xb1f92(%rip),%rdx        # 1495b8 <__frame_dummy_init_array_entry+0x2398>
   97626:	call   f0803 <core::panicking::panic_bounds_check>
   9762b:	lea    0xb1eae(%rip),%rdx        # 1494e0 <__frame_dummy_init_array_entry+0x22c0>
   97632:	mov    %rax,%rdi
   97635:	mov    %r8,%rsi
   97638:	call   f0803 <core::panicking::panic_bounds_check>
   9763d:	cmp    %rcx,%r14
   97640:	je     9764b <<c30walk::ctor2su::W<false>>::finish+0x5eb>
   97642:	sub    %ax,(%rdi,%rcx,2)
   97646:	inc    %rcx
   97649:	jmp    9763d <<c30walk::ctor2su::W<false>>::finish+0x5dd>
   9764b:	add    $0x38,%rsp
   9764f:	pop    %rbx
   97650:	pop    %r12
   97652:	pop    %r13
   97654:	pop    %r14
   97656:	pop    %r15
   97658:	pop    %rbp
   97659:	ret
