# The ladder's rungs, their main loops at 32 lanes (x86-64 baseline target)

objdump -d of `target/release/examples/walker_ladder`: each rung's innermost loop over the lanes that loads the table without a bounds check (the full-lane phase; the tail uses checked indexing). P is the library's `room_segments::<32, 1>`, the shipped walk, inside its room pass. Loads and stores are data accesses (lane state, table, output pointer, output), not the prefetch.

| rung | instructions | loads | stores | what differs from the rung before |
|---|--:|--:|--:|---|
| A | 19 | 3 (remaining count, row, entry) | 2 (count, row) | — : counter termination, the byte summed into one register |
| B | 20 | 3 | 2 | + prefetch |
| C | 17 | 2 (row, entry) | 1 (row) | flag termination replaces the counter (its load, decrement and store) |
| D | 19 | 3 (row, entry, output pointer) | 3 (row, byte, pointer) | + the byte stored through a per-lane pointer; − the sum |
| M | 13 | 2 (row, entry) | 1 (row) | C without the sum or the step count: the shipped step without its output store |
| P | 16 | 3 (row, entry, room base) | 2 (row, byte) | M + the output store at base + offset (the room pass), and its buffers, growth and assembly outside the step |

## A: 19 instructions

```text
   15400:	lea    (%r10,%rcx,1),%r11
   15404:	inc    %r11
   15407:	mov    0x0(%rbp,%rcx,4),%r8d
   1540c:	test   %r8d,%r8d
   1540f:	je     15450 <walker_ladder::ladder::<32, false, false, false>+0x1b0>
   15411:	mov    (%rax,%rcx,4),%esi
   15414:	mov    (%r14,%rsi,4),%esi
   15418:	dec    %r8d
   1541b:	mov    %r8d,0x0(%rbp,%rcx,4)
   15420:	movzbl %sil,%r8d
   15424:	shr    $0x8,%esi
   15427:	mov    %esi,(%rax,%rcx,4)
   1542a:	add    %r8,%r15
   1542d:	inc    %rcx
   15430:	dec    %r11
   15433:	add    $0x4,%r13
   15437:	add    $0x4,%rdx
   1543b:	cmp    $0x1f,%r11
   1543f:	jb     15400 <walker_ladder::ladder::<32, false, false, false>+0x160>
```

## B: 20 instructions

```text
   16150:	lea    (%r10,%rcx,1),%r11
   16154:	inc    %r11
   16157:	mov    0x0(%rbp,%rcx,4),%r8d
   1615c:	test   %r8d,%r8d
   1615f:	je     161a0 <walker_ladder::ladder::<32, true, false, false>+0x1b0>
   16161:	mov    (%rax,%rcx,4),%esi
   16164:	mov    (%r14,%rsi,4),%esi
   16168:	dec    %r8d
   1616b:	mov    %r8d,0x0(%rbp,%rcx,4)
   16170:	movzbl %sil,%r8d
   16174:	shr    $0x8,%esi
   16177:	mov    %esi,(%rax,%rcx,4)
   1617a:	prefetcht0 (%r14,%rsi,4)
   1617f:	add    %r8,%r15
   16182:	inc    %rcx
   16185:	dec    %r11
   16188:	add    $0x4,%r13
   1618c:	add    $0x4,%rdx
   16190:	cmp    $0x1f,%r11
   16194:	jb     16150 <walker_ladder::ladder::<32, true, false, false>+0x160>
```

## C: 17 instructions

```text
   15d40:	lea    0x1(%r10),%r9
   15d44:	mov    (%r11),%ecx
   15d47:	mov    (%r15,%rcx,4),%ecx
   15d4b:	test   %ecx,%ecx
   15d4d:	js     15d80 <walker_ladder::ladder::<32, true, true, false>+0x120>
   15d4f:	mov    %ecx,%r10d
   15d52:	shr    $0x8,%r10d
   15d56:	mov    %r10d,(%r11)
   15d59:	prefetcht0 (%r15,%r10,4)
   15d5e:	movzbl %cl,%ecx
   15d61:	add    %rcx,%r14
   15d64:	inc    %r12
   15d67:	lea    -0x1(%r9),%rcx
   15d6b:	add    $0x4,%r11
   15d6f:	mov    %r9,%r10
   15d72:	cmp    $0x1f,%rcx
   15d76:	jb     15d40 <walker_ladder::ladder::<32, true, true, false>+0xe0>
```

## D: 19 instructions

```text
   158c0:	lea    (%r11,%r8,1),%r13
   158c4:	inc    %r13
   158c7:	mov    (%rax,%r8,4),%ecx
   158cb:	mov    (%r12,%rcx,4),%r9d
   158cf:	test   %r9d,%r9d
   158d2:	js     15910 <walker_ladder::ladder::<32, true, true, true>+0x210>
   158d4:	mov    %r9d,%ecx
   158d7:	shr    $0x8,%ecx
   158da:	mov    %ecx,(%rax,%r8,4)
   158de:	prefetcht0 (%r12,%rcx,4)
   158e3:	mov    (%r10,%r8,8),%rcx
   158e7:	mov    %r9b,(%r15,%rcx,1)
   158eb:	inc    %rcx
   158ee:	mov    %rcx,(%r10,%r8,8)
   158f2:	inc    %r8
   158f5:	dec    %r13
   158f8:	add    $0x4,%rbx
   158fc:	cmp    $0x1f,%r13
   15900:	jb     158c0 <walker_ladder::ladder::<32, true, true, true>+0x1c0>
```

## M: 13 instructions

```text
   18f30:	lea    0x1(%r10),%r9
   18f34:	mov    (%r11),%r14d
   18f37:	mov    (%r15,%r14,4),%eax
   18f3b:	test   %eax,%eax
   18f3d:	js     18f60 <walker_ladder::minimal::<32>+0x1a0>
   18f3f:	shr    $0x8,%eax
   18f42:	mov    %eax,(%r11)
   18f45:	prefetcht0 (%r15,%rax,4)
   18f4a:	lea    -0x1(%r9),%rax
   18f4e:	add    $0x4,%r11
   18f52:	mov    %r9,%r10
   18f55:	cmp    $0x1f,%rax
   18f59:	jb     18f30 <walker_ladder::minimal::<32>+0x170>
```

## P: 16 instructions

```text
   34a60:	lea    0x1(%r8),%r10
   34a64:	mov    (%r11),%ebx
   34a67:	mov    (%r15,%rbx,4),%eax
   34a6b:	test   %eax,%eax
   34a6d:	js     34aa0 <mibz2::bwt::room_segments::<32, 1>+0xb30>
   34a6f:	mov    %eax,%edi
   34a71:	shr    $0x8,%edi
   34a74:	mov    %edi,(%r11)
   34a77:	prefetcht0 (%r15,%rdi,4)
   34a7c:	mov    0x70(%rsp,%r8,8),%rdi
   34a81:	mov    %al,(%rdi,%rcx,1)
   34a84:	lea    -0x1(%r10),%rax
   34a88:	add    $0x20,%r11
   34a8c:	mov    %r10,%r8
   34a8f:	cmp    $0x1f,%rax
   34a93:	jb     34a60 <mibz2::bwt::room_segments::<32, 1>+0xaf0>
```
