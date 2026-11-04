# <c30walk::ctor2su::W<false>>::finish
# size 0x5fa

target/rust-1.97.0/release/examples/c30walk:     file format elf64-x86-64


Disassembly of section .text:

0000000000097070 <<c30walk::ctor2su::W<false>>::finish>:
   97070:	push   %rbp
   97071:	push   %r15
   97073:	push   %r14
   97075:	push   %r13
   97077:	push   %r12
   97079:	push   %rbx
   9707a:	sub    $0x38,%rsp
   9707e:	mov    0x18(%rdi),%rbp
   97082:	mov    0x10(%rbp),%r8
   97086:	mov    %r8,%r13
   97089:	sub    $0x1,%r13
   9708d:	jb     975fa <<c30walk::ctor2su::W<false>>::finish+0x58a>
   97093:	mov    %rdi,%r11
   97096:	lea    (%rsi,%rsi,4),%rax
   9709a:	mov    0x40(%rdi,%rax,8),%r14
   9709f:	mov    0x50(%rdi,%rax,8),%rdi
   970a4:	mov    0x58(%r11,%rax,8),%r12d
   970a9:	mov    0x60(%r11,%rax,8),%ecx
   970ae:	mov    0x64(%r11,%rax,8),%eax
   970b3:	sub    %rdi,%r14
   970b6:	mov    %r14,0x8(%rsp)
   970bb:	shr    $1,%r14
   970be:	mov    0x8(%rbp),%r10
   970c2:	mov    %r13,%rdx
   970c5:	shl    $0xa,%rdx
   970c9:	add    %r10,%rdx
   970cc:	mov    %r13,%rbx
   970cf:	cmp    %r12d,(%rdx,%rsi,4)
   970d3:	jbe    97148 <<c30walk::ctor2su::W<false>>::finish+0xd8>
   970d5:	mov    %r11,%r15
   970d8:	lea    (%r12,%r14,1),%edx
   970dc:	dec    %edx
   970de:	mov    %r8,%r9
   970e1:	shl    $0xa,%r9
   970e5:	lea    (%r9,%rsi,4),%r9
   970e9:	lea    (%r10,%r9,1),%r11
   970ed:	add    $0xfffffffffffffc00,%r11
   970f4:	mov    %r8,%rbx
   970f7:	nopw   0x0(%rax,%rax,1)
   97100:	dec    %rbx
   97103:	cmp    %r8,%rbx
   97106:	jae    975ae <<c30walk::ctor2su::W<false>>::finish+0x53e>
   9710c:	cmp    %r12d,(%r11)
   9710f:	lea    -0x400(%r11),%r11
   97116:	ja     97100 <<c30walk::ctor2su::W<false>>::finish+0x90>
   97118:	add    %r10,%r9
   9711b:	add    $0xfffffffffffffc00,%r9
   97122:	mov    %r8,%r13
   97125:	mov    %r15,%r11
   97128:	nopl   0x0(%rax,%rax,1)
   97130:	dec    %r13
   97133:	cmp    %r8,%r13
   97136:	jae    975c0 <<c30walk::ctor2su::W<false>>::finish+0x550>
   9713c:	cmp    %edx,(%r9)
   9713f:	lea    -0x400(%r9),%r9
   97146:	ja     97130 <<c30walk::ctor2su::W<false>>::finish+0xc0>
   97148:	imul   $0xffffffffffffffe2,%rcx,%r15
   9714c:	add    %r12,%r15
   9714f:	cmp    $0xffffffff,%eax
   97152:	mov    %r14,0x10(%rsp)
   97157:	je     9725b <<c30walk::ctor2su::W<false>>::finish+0x1eb>
   9715d:	mov    $0xffffffff,%r9d
   97163:	cmp    %r13,%rbx
   97166:	jae    97180 <<c30walk::ctor2su::W<false>>::finish+0x110>
   97168:	lea    0x1(%rbx),%rcx
   9716c:	cmp    %r8,%rcx
   9716f:	jae    9761d <<c30walk::ctor2su::W<false>>::finish+0x5ad>
   97175:	shl    $0xa,%rcx
   97179:	add    %rcx,%r10
   9717c:	mov    (%r10,%rsi,4),%r9d
   97180:	mov    (%r11),%rdx
   97183:	mov    0x10(%rdx),%rcx
   97187:	cmp    %rax,%rcx
   9718a:	jbe    9760b <<c30walk::ctor2su::W<false>>::finish+0x59b>
   97190:	test   %r14,%r14
   97193:	je     9765b <<c30walk::ctor2su::W<false>>::finish+0x5eb>
   97199:	shl    $0x7,%rax
   9719d:	add    0x8(%rdx),%rax
   971a1:	lea    0x800(,%rsi,4),%rdx
   971a9:	xor    %r11d,%r11d
   971ac:	nopl   0x0(%rax)
   971b0:	lea    (%r12,%r11,1),%r10d
   971b4:	cmp    %r9d,%r10d
   971b7:	jae    971f0 <<c30walk::ctor2su::W<false>>::finish+0x180>
   971b9:	lea    (%r11,%r15,1),%rcx
   971bd:	cmp    $0x20,%rcx
   971c1:	jae    975d2 <<c30walk::ctor2su::W<false>>::finish+0x562>
   971c7:	movzwl (%rdi,%r11,2),%r8d
   971cc:	inc    %r11
   971cf:	mov    %ebx,%r10d
   971d2:	shl    $0x18,%r10d
   971d6:	shl    $0x8,%r8d
   971da:	or     %r10d,%r8d
   971dd:	or     %esi,%r8d
   971e0:	mov    %r8d,(%rax,%rcx,4)
   971e4:	cmp    0x10(%rsp),%r11
   971e9:	jne    971b0 <<c30walk::ctor2su::W<false>>::finish+0x140>
   971eb:	jmp    97239 <<c30walk::ctor2su::W<false>>::finish+0x1c9>
   971ed:	nopl   (%rax)
   971f0:	mov    %rbx,%r14
   971f3:	shl    $0xa,%r14
   971f7:	add    %rdx,%r14
   971fa:	mov    %rbx,%rcx
   971fd:	jmp    9720f <<c30walk::ctor2su::W<false>>::finish+0x19f>
   971ff:	nop
   97200:	add    $0x400,%r14
   97207:	mov    %rbx,%rcx
   9720a:	cmp    %r9d,%r10d
   9720d:	jb     971b9 <<c30walk::ctor2su::W<false>>::finish+0x149>
   9720f:	lea    0x1(%rcx),%rbx
   97213:	mov    $0xffffffff,%r9d
   97219:	cmp    %r13,%rbx
   9721c:	jae    97200 <<c30walk::ctor2su::W<false>>::finish+0x190>
   9721e:	add    $0x2,%rcx
   97222:	mov    0x10(%rbp),%r8
   97226:	cmp    %r8,%rcx
   97229:	jae    9759c <<c30walk::ctor2su::W<false>>::finish+0x52c>
   9722f:	mov    0x8(%rbp),%rcx
   97233:	mov    (%rcx,%r14,1),%r9d
   97237:	jmp    97200 <<c30walk::ctor2su::W<false>>::finish+0x190>
   97239:	mov    0x8(%rsp),%rdx
   9723e:	and    $0xfffffffffffffffe,%rdx
   97242:	mov    $0xff,%esi
   97247:	add    $0x38,%rsp
   9724b:	pop    %rbx
   9724c:	pop    %r12
   9724e:	pop    %r13
   97250:	pop    %r14
   97252:	pop    %r15
   97254:	pop    %rbp
   97255:	jmp    *0xb6e9d(%rip)        # 14e0f8 <memset@GLIBC_2.2.5>
   9725b:	movzwl (%rdi),%eax
   9725e:	movzwl -0x2(%rdi,%r14,2),%r8d
   97264:	mov    %ebx,%edx
   97266:	shl    $0x10,%edx
   97269:	or     %eax,%edx
   9726b:	mov    %r13d,%r9d
   9726e:	shl    $0x10,%r9d
   97272:	or     %r8d,%r9d
   97275:	sub    %edx,%r9d
   97278:	cmp    $0xfffe,%r9d
   9727f:	jae    972fd <<c30walk::ctor2su::W<false>>::finish+0x28d>
   97281:	mov    0x28(%r11),%r8
   97285:	shl    $0x6,%rcx
   97289:	shl    $0x8,%edx
   9728c:	or     %edx,%esi
   9728e:	mov    %esi,0x3c(%r8,%rcx,1)
   97293:	test   %r14,%r14
   97296:	je     9765b <<c30walk::ctor2su::W<false>>::finish+0x5eb>
   9729c:	cmp    $0x1e,%r14
   972a0:	jne    97496 <<c30walk::ctor2su::W<false>>::finish+0x426>
   972a6:	movd   %eax,%xmm0
   972aa:	pshuflw $0x0,%xmm0,%xmm0
   972af:	pshufd $0x44,%xmm0,%xmm1
   972b4:	movdqu (%rdi),%xmm2
   972b8:	movdqu 0x10(%rdi),%xmm3
   972bd:	movdqu 0x20(%rdi),%xmm4
   972c2:	psubw  %xmm1,%xmm2
   972c6:	movdqu %xmm2,(%rdi)
   972ca:	psubw  %xmm1,%xmm3
   972ce:	movdqu %xmm3,0x10(%rdi)
   972d3:	psubw  %xmm1,%xmm4
   972d7:	movdqu %xmm4,0x20(%rdi)
   972dc:	movq   0x30(%rdi),%xmm1
   972e1:	psubw  %xmm0,%xmm1
   972e5:	movq   %xmm1,0x30(%rdi)
   972ea:	movd   0x38(%rdi),%xmm1
   972ef:	psubw  %xmm0,%xmm1
   972f3:	movd   %xmm1,0x38(%rdi)
   972f8:	jmp    9765b <<c30walk::ctor2su::W<false>>::finish+0x5eb>
   972fd:	mov    (%r11),%r10
   97300:	mov    0x10(%r10),%r9
   97304:	cmp    (%r10),%r9
   97307:	mov    %r11,0x30(%rsp)
   9730c:	je     9755c <<c30walk::ctor2su::W<false>>::finish+0x4ec>
   97312:	mov    0x8(%r10),%rax
   97316:	mov    %r9,%rdx
   97319:	shl    $0x7,%rdx
   9731d:	pxor   %xmm0,%xmm0
   97321:	movdqa %xmm0,0x70(%rax,%rdx,1)
   97327:	movdqa %xmm0,0x60(%rax,%rdx,1)
   9732d:	movdqa %xmm0,0x50(%rax,%rdx,1)
   97333:	movdqa %xmm0,0x40(%rax,%rdx,1)
   97339:	movdqa %xmm0,0x30(%rax,%rdx,1)
   9733f:	movdqa %xmm0,0x20(%rax,%rdx,1)
   97345:	movdqa %xmm0,0x10(%rax,%rdx,1)
   9734b:	movdqa %xmm0,(%rax,%rdx,1)
   97350:	lea    0x1(%r9),%rax
   97354:	mov    %rax,0x10(%r10)
   97358:	mov    $0xffffffff,%r8d
   9735e:	cmp    %r13,%rbx
   97361:	jae    97380 <<c30walk::ctor2su::W<false>>::finish+0x310>
   97363:	lea    0x1(%rbx),%rax
   97367:	mov    0x10(%rbp),%r8
   9736b:	cmp    %r8,%rax
   9736e:	jae    9763b <<c30walk::ctor2su::W<false>>::finish+0x5cb>
   97374:	shl    $0xa,%rax
   97378:	add    0x8(%rbp),%rax
   9737c:	mov    (%rax,%rsi,4),%r8d
   97380:	mov    %r9,0x8(%rsp)
   97385:	test   %r14,%r14
   97388:	je     97435 <<c30walk::ctor2su::W<false>>::finish+0x3c5>
   9738e:	add    0x8(%r10),%rdx
   97392:	lea    0x800(,%rsi,4),%r9
   9739a:	xor    %r10d,%r10d
   9739d:	nopl   (%rax)
   973a0:	lea    (%r12,%r10,1),%r11d
   973a4:	cmp    %r8d,%r11d
   973a7:	jae    973e0 <<c30walk::ctor2su::W<false>>::finish+0x370>
   973a9:	lea    (%r10,%r15,1),%rax
   973ad:	cmp    $0x20,%rax
   973b1:	jae    975e6 <<c30walk::ctor2su::W<false>>::finish+0x576>
   973b7:	movzwl (%rdi,%r10,2),%r11d
   973bc:	inc    %r10
   973bf:	mov    %ebx,%r14d
   973c2:	shl    $0x18,%r14d
   973c6:	shl    $0x8,%r11d
   973ca:	or     %r14d,%r11d
   973cd:	or     %esi,%r11d
   973d0:	mov    %r11d,(%rdx,%rax,4)
   973d4:	cmp    0x10(%rsp),%r10
   973d9:	jne    973a0 <<c30walk::ctor2su::W<false>>::finish+0x330>
   973db:	jmp    97435 <<c30walk::ctor2su::W<false>>::finish+0x3c5>
   973dd:	nopl   (%rax)
   973e0:	mov    %rbx,%r14
   973e3:	shl    $0xa,%r14
   973e7:	add    %r9,%r14
   973ea:	mov    %rbx,%rax
   973ed:	jmp    97407 <<c30walk::ctor2su::W<false>>::finish+0x397>
   973ef:	nop
   973f0:	mov    0x8(%rbp),%rax
   973f4:	mov    (%rax,%r14,1),%r8d
   973f8:	add    $0x400,%r14
   973ff:	mov    %rbx,%rax
   97402:	cmp    %r8d,%r11d
   97405:	jb     973a9 <<c30walk::ctor2su::W<false>>::finish+0x339>
   97407:	lea    0x1(%rax),%rbx
   9740b:	mov    $0xffffffff,%r8d
   97411:	cmp    %r13,%rbx
   97414:	jae    973f8 <<c30walk::ctor2su::W<false>>::finish+0x388>
   97416:	add    $0x2,%rax
   9741a:	mov    0x10(%rbp),%r8
   9741e:	cmp    %r8,%rax
   97421:	jb     973f0 <<c30walk::ctor2su::W<false>>::finish+0x380>
   97423:	lea    0xb216e(%rip),%rdx        # 149598 <__frame_dummy_init_array_entry+0x2308>
   9742a:	mov    %rax,%rdi
   9742d:	mov    %r8,%rsi
   97430:	call   f0873 <core::panicking::panic_bounds_check>
   97435:	mov    0x30(%rsp),%r8
   9743a:	mov    0x28(%r8),%rax
   9743e:	mov    %rcx,%rdx
   97441:	shl    $0x6,%rdx
   97445:	mov    0x8(%rsp),%rsi
   9744a:	or     $0x10000000,%esi
   97450:	pcmpeqd %xmm0,%xmm0
   97454:	movdqu %xmm0,0x2c(%rax,%rdx,1)
   9745a:	movdqa %xmm0,0x20(%rax,%rdx,1)
   97460:	movdqa %xmm0,0x10(%rax,%rdx,1)
   97466:	movdqa %xmm0,(%rax,%rdx,1)
   9746b:	mov    %esi,0x3c(%rax,%rdx,1)
   9746f:	mov    %ecx,%edi
   97471:	shr    $0x6,%edi
   97474:	mov    0x10(%r8),%rsi
   97478:	cmp    %rsi,%rdi
   9747b:	jae    9762f <<c30walk::ctor2su::W<false>>::finish+0x5bf>
   97481:	mov    $0x1,%eax
   97486:	shl    %cl,%rax
   97489:	mov    0x8(%r8),%rcx
   9748d:	or     %rax,(%rcx,%rdi,8)
   97491:	jmp    9765b <<c30walk::ctor2su::W<false>>::finish+0x5eb>
   97496:	mov    0x8(%rsp),%rcx
   9749b:	cmp    $0x8,%rcx
   9749f:	jae    974a8 <<c30walk::ctor2su::W<false>>::finish+0x438>
   974a1:	xor    %ecx,%ecx
   974a3:	jmp    97652 <<c30walk::ctor2su::W<false>>::finish+0x5e2>
   974a8:	movabs $0x7ffffffffffffff0,%rsi
   974b2:	cmp    $0x20,%rcx
   974b6:	jae    974bc <<c30walk::ctor2su::W<false>>::finish+0x44c>
   974b8:	xor    %ecx,%ecx
   974ba:	jmp    9751b <<c30walk::ctor2su::W<false>>::finish+0x4ab>
   974bc:	mov    %r14,%rcx
   974bf:	and    %rsi,%rcx
   974c2:	movd   %eax,%xmm0
   974c6:	pshuflw $0x0,%xmm0,%xmm0
   974cb:	pshufd $0x44,%xmm0,%xmm0
   974d0:	xor    %edx,%edx
   974d2:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   974e0:	movdqu (%rdi,%rdx,2),%xmm1
   974e5:	movdqu 0x10(%rdi,%rdx,2),%xmm2
   974eb:	psubw  %xmm0,%xmm1
   974ef:	psubw  %xmm0,%xmm2
   974f3:	movdqu %xmm1,(%rdi,%rdx,2)
   974f8:	movdqu %xmm2,0x10(%rdi,%rdx,2)
   974fe:	add    $0x10,%rdx
   97502:	cmp    %rdx,%rcx
   97505:	jne    974e0 <<c30walk::ctor2su::W<false>>::finish+0x470>
   97507:	cmp    %rcx,%r14
   9750a:	je     9765b <<c30walk::ctor2su::W<false>>::finish+0x5eb>
   97510:	testb  $0x18,0x8(%rsp)
   97515:	je     97652 <<c30walk::ctor2su::W<false>>::finish+0x5e2>
   9751b:	mov    %rcx,%rdx
   9751e:	add    $0xc,%rsi
   97522:	mov    %rsi,%rcx
   97525:	and    %r14,%rcx
   97528:	movd   %eax,%xmm0
   9752c:	pshuflw $0x0,%xmm0,%xmm0
   97531:	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   97540:	movq   (%rdi,%rdx,2),%xmm1
   97545:	psubw  %xmm0,%xmm1
   97549:	movq   %xmm1,(%rdi,%rdx,2)
   9754e:	add    $0x4,%rdx
   97552:	cmp    %rdx,%rcx
   97555:	jne    97540 <<c30walk::ctor2su::W<false>>::finish+0x4d0>
   97557:	jmp    9764d <<c30walk::ctor2su::W<false>>::finish+0x5dd>
   9755c:	mov    %rdi,0x8(%rsp)
   97561:	mov    %r10,%rdi
   97564:	mov    %rsi,0x20(%rsp)
   97569:	mov    %rcx,0x28(%rsp)
   9756e:	mov    %r9,%r14
   97571:	mov    %r10,0x18(%rsp)
   97576:	call   9c920 <<alloc::raw_vec::RawVec<c30walk::RawG>>::grow_one>
   9757b:	mov    0x18(%rsp),%r10
   97580:	mov    %r14,%r9
   97583:	mov    0x28(%rsp),%rcx
   97588:	mov    0x20(%rsp),%rsi
   9758d:	mov    0x10(%rsp),%r14
   97592:	mov    0x8(%rsp),%rdi
   97597:	jmp    97312 <<c30walk::ctor2su::W<false>>::finish+0x2a2>
   9759c:	lea    0xb1ff5(%rip),%rdx        # 149598 <__frame_dummy_init_array_entry+0x2308>
   975a3:	mov    %rcx,%rdi
   975a6:	mov    %r8,%rsi
   975a9:	call   f0873 <core::panicking::panic_bounds_check>
   975ae:	lea    0xb20eb(%rip),%rdx        # 1496a0 <__frame_dummy_init_array_entry+0x2410>
   975b5:	mov    %rbx,%rdi
   975b8:	mov    %r8,%rsi
   975bb:	call   f0873 <core::panicking::panic_bounds_check>
   975c0:	lea    0xb20d9(%rip),%rdx        # 1496a0 <__frame_dummy_init_array_entry+0x2410>
   975c7:	mov    %r13,%rdi
   975ca:	mov    %r8,%rsi
   975cd:	call   f0873 <core::panicking::panic_bounds_check>
   975d2:	lea    0xb1fa7(%rip),%rdx        # 149580 <__frame_dummy_init_array_entry+0x22f0>
   975d9:	mov    $0x20,%esi
   975de:	mov    %rcx,%rdi
   975e1:	call   f0873 <core::panicking::panic_bounds_check>
   975e6:	lea    0xb1f93(%rip),%rdx        # 149580 <__frame_dummy_init_array_entry+0x22f0>
   975ed:	mov    $0x20,%esi
   975f2:	mov    %rax,%rdi
   975f5:	call   f0873 <core::panicking::panic_bounds_check>
   975fa:	lea    0xb200f(%rip),%rdx        # 149610 <__frame_dummy_init_array_entry+0x2380>
   97601:	mov    %r13,%rdi
   97604:	xor    %esi,%esi
   97606:	call   f0873 <core::panicking::panic_bounds_check>
   9760b:	lea    0xb1f56(%rip),%rdx        # 149568 <__frame_dummy_init_array_entry+0x22d8>
   97612:	mov    %rax,%rdi
   97615:	mov    %rcx,%rsi
   97618:	call   f0873 <core::panicking::panic_bounds_check>
   9761d:	lea    0xb1f2c(%rip),%rdx        # 149550 <__frame_dummy_init_array_entry+0x22c0>
   97624:	mov    %rcx,%rdi
   97627:	mov    %r8,%rsi
   9762a:	call   f0873 <core::panicking::panic_bounds_check>
   9762f:	lea    0xb1ff2(%rip),%rdx        # 149628 <__frame_dummy_init_array_entry+0x2398>
   97636:	call   f0873 <core::panicking::panic_bounds_check>
   9763b:	lea    0xb1f0e(%rip),%rdx        # 149550 <__frame_dummy_init_array_entry+0x22c0>
   97642:	mov    %rax,%rdi
   97645:	mov    %r8,%rsi
   97648:	call   f0873 <core::panicking::panic_bounds_check>
   9764d:	cmp    %rcx,%r14
   97650:	je     9765b <<c30walk::ctor2su::W<false>>::finish+0x5eb>
   97652:	sub    %ax,(%rdi,%rcx,2)
   97656:	inc    %rcx
   97659:	jmp    9764d <<c30walk::ctor2su::W<false>>::finish+0x5dd>
   9765b:	add    $0x38,%rsp
   9765f:	pop    %rbx
   97660:	pop    %r12
   97662:	pop    %r13
   97664:	pop    %r14
   97666:	pop    %r15
   97668:	pop    %rbp
   97669:	ret
