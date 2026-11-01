# The walks' ordinary steps, disassembled (x86-64, release, this branch's build)

objdump -d of `target/release/examples/coop`, the inner `for k in 0..LANES` loop of each room walk. Instruction counts are in the journal, §5.

## o: production's entry (the control), the ordinary step

```text
   21540:	lea    0x1(%r10),%rax
   21544:	mov    (%r9),%r12d
   21547:	mov    (%rbx,%r12,4),%ecx
   2154b:	test   %ecx,%ecx
   2154d:	js     21580 <<coop::compact::Compact>::room::<0>+0xb40>
   2154f:	mov    %ecx,%edi
   21551:	shr    $0x8,%edi
   21554:	mov    %edi,(%r9)
   21557:	prefetcht0 (%rbx,%rdi,4)
   2155b:	mov    0x60(%rsp,%r10,8),%rdi
   21560:	mov    %cl,(%rdi,%rdx,1)
   21563:	lea    -0x1(%rax),%rcx
   21567:	add    $0x20,%r9
   2156b:	mov    %rax,%r10
   2156e:	cmp    $0x1f,%rcx
   21572:	jb     21540 <<coop::compact::Compact>::room::<0>+0xb00>
   21574:	jmp    21500 <<coop::compact::Compact>::room::<0>+0xac0>
```

## v: a 32-bit link, the byte from F (stage A), the ordinary step

```text
   236b0:	lea    0x1(%r9),%rbp
   236b4:	mov    %r9,%rcx
   236b7:	shl    $0x5,%rcx
   236bb:	lea    (%rsp,%rcx,1),%rax
   236bf:	add    $0x250,%rax
   236c5:	mov    0x268(%rsp,%rcx,1),%r10d
   236cd:	movslq 0x0(%r13,%r10,4),%rcx
   236d2:	test   %rcx,%rcx
   236d5:	js     23740 <<coop::compact::Compact>::room::<1>+0xc40>
   236d7:	movzbl 0x1c(%rax),%edi
   236db:	mov    %r10d,%r8d
   236de:	shr    $0x5,%r8d
   236e2:	mov    0x80(%rsp),%r14
   236ea:	movzbl (%r14,%r8,1),%r14d
   236ef:	inc    %r14
   236f2:	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   23700:	mov    %r14,%r8
   23703:	inc    %r14
   23706:	cmp    (%r12,%r8,4),%r10d
   2370a:	jae    23700 <<coop::compact::Compact>::room::<1>+0xc00>
   2370c:	dec    %r8
   2370f:	mov    %r8b,0x1c(%rax)
   23713:	mov    %ecx,0x18(%rax)
   23716:	prefetcht0 0x0(%r13,%rcx,4)
   2371c:	mov    0x88(%rsp,%r9,8),%rax
   23724:	mov    %dil,(%rax,%rdx,1)
   23728:	cmp    $0x1f,%r9
   2372c:	mov    %rbp,%r9
   2372f:	jb     236b0 <<coop::compact::Compact>::room::<1>+0xbb0>
   23735:	jmp    23680 <<coop::compact::Compact>::room::<1>+0xb80>
```

## u: a 24-bit link, the byte from F (stage A), the ordinary step

```text
   258b0:	lea    0x1(%r9),%rbp
   258b4:	mov    %r9,%rcx
   258b7:	shl    $0x5,%rcx
   258bb:	lea    (%rsp,%rcx,1),%rax
   258bf:	add    $0x240,%rax
   258c5:	mov    0x258(%rsp,%rcx,1),%r10d
   258cd:	lea    (%r10,%r10,2),%rcx
   258d1:	mov    0x0(%r13,%rcx,1),%ecx
   258d6:	and    $0xffffff,%ecx
   258dc:	cmp    $0x7fffff,%ecx
   258e2:	ja     25940 <<coop::compact::Compact>::room::<2>+0xc40>
   258e4:	movzbl 0x1c(%rax),%edi
   258e8:	mov    %r10d,%r8d
   258eb:	shr    $0x5,%r8d
   258ef:	mov    0x78(%rsp),%r14
   258f4:	movzbl (%r14,%r8,1),%r14d
   258f9:	inc    %r14
   258fc:	nopl   0x0(%rax)
   25900:	mov    %r14,%r8
   25903:	inc    %r14
   25906:	cmp    (%r12,%r8,4),%r10d
   2590a:	jae    25900 <<coop::compact::Compact>::room::<2>+0xc00>
   2590c:	dec    %r8
   2590f:	mov    %r8b,0x1c(%rax)
   25913:	mov    %ecx,0x18(%rax)
   25916:	lea    (%rcx,%rcx,2),%rax
   2591a:	prefetcht0 0x0(%r13,%rax,1)
   25920:	mov    0x80(%rsp,%r9,8),%rax
   25928:	mov    %dil,(%rax,%rdx,1)
   2592c:	cmp    $0x1f,%r9
   25930:	mov    %rbp,%r9
   25933:	jb     258b0 <<coop::compact::Compact>::room::<2>+0xbb0>
   25939:	jmp    25880 <<coop::compact::Compact>::room::<2>+0xb80>
```

## h, first cut (1b49c25): header start-mask and pend tests, the ordinary step: from 529a8 to the jmp at 52a2e, then 52983-529a2

```text
   52983:	and    $0xffffffe0,%esi
   52986:	prefetcht0 (%r12,%rsi,2)
   5298b:	mov    0x58(%rsp,%r8,8),%rdx
   52990:	mov    %cl,(%rdx,%rax,1)
   52993:	lea    -0x1(%r10),%rcx
   52997:	add    $0x28,%r11
   5299b:	mov    %r10,%r8
   5299e:	cmp    $0x1f,%rcx
   529a2:	jae    52920 <<coop::compact::Compact>::room_h+0x16d0>
   529a8:	lea    0x1(%r8),%r10
   529ac:	mov    0x4(%r11),%ecx
   529b0:	cmp    %r14,%rcx
   529b3:	jne    52960 <<coop::compact::Compact>::room_h+0x1710>
   529b5:	mov    (%r11),%ebx
   529b8:	mov    %ebx,%ecx
   529ba:	shr    $0x5,%ecx
   529bd:	shl    $0x6,%rcx
   529c1:	lea    (%r12,%rcx,1),%rdx
   529c5:	mov    0x38(%r12,%rcx,1),%rsi
   529ca:	mov    %ebx,%edi
   529cc:	and    $0x1f,%edi
   529cf:	mov    %rdi,%r15
   529d2:	add    %rdi,%r15
   529d5:	mov    %edi,%ecx
   529d7:	shl    $0x4,%ecx
   529da:	sub    %r15,%rcx
   529dd:	mov    %ecx,%r15d
   529e0:	shr    $0x3,%r15d
   529e4:	mov    (%r15,%rdx,1),%r13d
   529e8:	and    $0x6,%cl
   529eb:	shr    %cl,%r13d
   529ee:	mov    %ebx,%ecx
   529f0:	or     $0x20,%cl
   529f3:	movzbl %cl,%ecx
   529f6:	bt     %rcx,%rsi
   529fa:	jb     52a40 <<coop::compact::Compact>::room_h+0x17f0>
   529fc:	test   $0x10000000,%esi
   52a02:	jne    52ae0 <<coop::compact::Compact>::room_h+0x1890>
   52a08:	and    $0x3fff,%r13d
   52a0f:	movzbl 0x8(%r11),%ecx
   52a14:	mov    %esi,%edx
   52a16:	shr    $0x14,%edx
   52a19:	mov    %dl,0x8(%r11)
   52a1d:	and    $0xfffff,%esi
   52a23:	add    %r13d,%esi
   52a26:	mov    %esi,(%r11)
   52a29:	mov    0x40(%rsp),%r15
   52a2e:	jmp    52983 <<coop::compact::Compact>::room_h+0x1733>
```

## h, as frozen: the sentinel and the parking record, the ordinary step: 527cd-52818 (the test), then 52790-527cb (the FOR step and loop)

```text
   52790:	movzbl 0x8(%r9),%esi
   52795:	mov    %ebx,%edx
   52797:	shr    $0x14,%edx
   5279a:	mov    %dl,0x8(%r9)
   5279e:	and    $0xfffff,%ebx
   527a4:	add    %ebx,%r11d
   527a7:	mov    %r11d,(%r9)
   527aa:	and    $0xffffffe0,%r11d
   527ae:	prefetcht0 (%r15,%r11,2)
   527b3:	mov    0x68(%rsp,%rdi,8),%rdx
   527b8:	mov    %sil,(%rdx,%rax,1)
   527bc:	lea    -0x1(%rcx),%rdx
   527c0:	add    $0x28,%r9
   527c4:	mov    %rcx,%rdi
   527c7:	cmp    $0x1f,%rdx
   527cb:	jae    52760 <<coop::compact::Compact>::room_h+0x13f0>
   527cd:	mov    (%r9),%r10d
   527d0:	mov    %r10d,%edx
   527d3:	shr    $0x5,%edx
   527d6:	shl    $0x6,%rdx
   527da:	lea    (%r15,%rdx,1),%r11
   527de:	mov    %r10d,%esi
   527e1:	and    $0x1f,%esi
   527e4:	mov    %rsi,%rbx
   527e7:	add    %rsi,%rbx
   527ea:	mov    %esi,%ecx
   527ec:	shl    $0x4,%ecx
   527ef:	sub    %rbx,%rcx
   527f2:	mov    %ecx,%ebx
   527f4:	shr    $0x3,%ebx
   527f7:	mov    (%rbx,%r11,1),%r11d
   527fb:	and    $0x6,%cl
   527fe:	shr    %cl,%r11d
   52801:	lea    0x1(%rdi),%rcx
   52805:	mov    0x38(%r15,%rdx,1),%rbx
   5280a:	and    $0x3fff,%r11d
   52811:	cmp    $0x3fff,%r11d
   52818:	jne    52790 <<coop::compact::Compact>::room_h+0x1420>
```
