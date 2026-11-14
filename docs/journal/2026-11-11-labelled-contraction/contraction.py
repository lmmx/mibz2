"""Packed baseline and stable two-order boundary-local first-return splice."""

from common import Piece, bisect_right, chosen_color, ordered


def select(top, bottom, marker, seed, c):
    """Use the original global color overlay, including origin exclusion."""
    edges, removed = [], []
    i = j = 0
    while i < len(bottom) and j < len(top):
        at, a = bottom[i]
        bs, b = top[j]
        lo, hi = max(at, bs), min(at + a.n, bs + b.n)
        c["overlay_pair_visits"] += 1
        if lo < hi:
            c["color_queries"] += 1
            yes = chosen_color(a.w, seed) == 0
            if yes:
                c["color_queries"] += 1
                yes = chosen_color(b.w, seed) == 1
            cuts = [lo, hi]
            c["selection_marker_checks"] += int(yes)
            if yes and lo <= marker < hi:
                cuts = ordered(list(set([lo, marker, marker + 1, hi])), lambda x: x, c)
            for yl, yh in zip(cuts, cuts[1:]):
                if yl == yh:
                    continue
                take = yes and yl != marker
                edges.append((yl, yh - yl, at, bs, a, b, take))
                c["overlay_records_created"] += 1
                if take:
                    if removed and removed[-1][1] == yl:
                        removed[-1] = (removed[-1][0], yh)
                    else:
                        removed.append((yl, yh))
                        c["removed_range_records"] += 1
        if at + a.n <= bs + b.n:
            i += 1
        if bs + b.n <= at + a.n:
            j += 1
    return edges, removed


def packed(pieces, grammar, marker, seed, c):
    """Instrument the original batch algorithm without changing its output."""
    images = ordered(pieces, lambda a: a.t, c)
    edges, removed = select([(a.s, a) for a in pieces], [(a.t, a) for a in images], marker, seed, c)
    c["selection_input_visits"] += 2 * len(pieces)
    c["peak_transient_entries"] = max(
        c["peak_transient_entries"], len(images) + len(edges) + len(removed)
    )
    if not removed:
        return pieces, marker, removed
    starts, ends, prefix = [a for a, _ in removed], [b for _, b in removed], [0]
    for a, b in removed:
        prefix.append(prefix[-1] + b - a)
        c["removal_prefix_additions"] += 1
    c["removal_index_entries"] += len(starts) + len(ends) + len(prefix)

    def rank(x):
        k = bisect_right(ends, x, c, "rank")
        result = prefix[k]
        c["rank_partial_tests"] += 1
        if k < len(starts) and starts[k] < x:
            result += x - starts[k]
        return x - result

    def keep_parts(lo, hi):
        k = bisect_right(ends, lo, c, "keep_search")
        x = lo
        while k < len(starts) and starts[k] < hi:
            c["removed_range_visits"] += 1
            if x < starts[k]:
                yield x, min(hi, starts[k])
            x = max(x, ends[k])
            k += 1
        if x < hi:
            yield x, hi

    raw = []
    for y, width, at, bs, a, b, take in edges:
        x = a.s + y - at
        target = b.t + y - bs if take else y
        word = grammar.cat(a.w, b.w) if take else a.w
        for lo, hi in keep_parts(x, x + width):
            dest = target + lo - x
            # Original assertion's extra rank calls are validation, not maintenance.
            raw.append(Piece(rank(lo), rank(dest), hi - lo, word))
            c["persistent_piece_records_created"] += 1
            c["coordinate_fields_written"] += 2
    out = []
    for a in ordered(raw, lambda a: a.s, c):
        c["coalescing_checks"] += 1
        if out and (
            out[-1].s + out[-1].n == a.s and out[-1].t + out[-1].n == a.t and out[-1].w == a.w
        ):
            b = out.pop()
            a = Piece(b.s, b.t, b.n + a.n, b.w)
            c["merges"] += 1
            c["persistent_piece_records_created"] += 1
            c["coordinate_fields_written"] += 2
        out.append(a)
    c["peak_live_piece_records"] = max(
        c["peak_live_piece_records"], len(pieces) + len(raw) + len(out)
    )
    c["peak_transient_entries"] = max(
        c["peak_transient_entries"],
        len(images)
        + len(edges)
        + len(removed)
        + len(starts)
        + len(ends)
        + len(prefix)
        + len(raw)
        + len(out),
    )
    return out, rank(marker), removed


class Edge:
    """Stable routing identity, shared word identity and width, without coordinates."""

    def __init__(self, identity, width, word):
        self.id, self.n, self.w = identity, width, word
        self.top = self.bottom = None


class Node:
    """A stable endpoint in one implicit order-statistics treap."""

    def __init__(self, edge, salt):
        self.edge = edge
        self.left = self.right = self.parent = None
        self.total = edge.n
        # Fixed priorities, not a random performance sample; height costs are counted.
        x = (edge.id * 0x9E3779B97F4A7C15 + salt) & ((1 << 64) - 1)
        x = ((x ^ (x >> 30)) * 0xBF58476D1CE4E5B9) & ((1 << 64) - 1)
        self.priority = ((x ^ (x >> 27)) * 0x94D049BB133111EB) & ((1 << 64) - 1)


class Order:
    """Implicit order with counted stable-node insertion, deletion and rank."""

    def __init__(self, c):
        self.root, self.c = None, c

    def pull(self, node):
        """Repair one subtree's weighted size."""
        self.c["tree_size_writes"] += 1
        node.total = (
            node.edge.n
            + (node.left.total if node.left else 0)
            + (node.right.total if node.right else 0)
        )

    def repair(self, node):
        """Repair ancestors after a local change."""
        while node:
            self.pull(node)
            self.c["tree_repair_hops"] += 1
            node = node.parent

    def rotate(self, child):
        """Rotate one child above its parent, preserving endpoint order."""
        parent, grand = child.parent, child.parent.parent
        if parent.left is child:
            parent.left = child.right
            if parent.left:
                parent.left.parent = parent
            child.right = parent
        else:
            parent.right = child.left
            if parent.right:
                parent.right.parent = parent
            child.left = parent
        child.parent, parent.parent = grand, child
        if grand is None:
            self.root = child
        elif grand.left is parent:
            grand.left = child
        else:
            grand.right = child
        # Count the actual optional child-parent link separately.
        self.c["tree_pointer_writes"] += 5 + int(
            (parent.left if child.right is parent else parent.right) is not None
        )
        self.c["tree_rotations"] += 1
        self.pull(parent)
        self.pull(child)

    def insert_after(self, prev, node):
        """Insert a node by attachment identity, without global rank rewriting."""
        self.c["tree_insertions"] += 1
        if self.root is None:
            self.root = node
            self.c["tree_pointer_writes"] += 1
            return
        if prev is None:
            parent = self.root
            while parent.left:
                self.c["tree_neighbor_hops"] += 1
                parent = parent.left
            parent.left = node
        elif prev.right is None:
            parent = prev
            parent.right = node
        else:
            parent = prev.right
            while parent.left:
                self.c["tree_neighbor_hops"] += 1
                parent = parent.left
            parent.left = node
        node.parent = parent
        self.c["tree_pointer_writes"] += 2
        self.repair(parent)
        while node.parent:
            self.c["tree_priority_comparisons"] += 1
            if node.priority >= node.parent.priority:
                break
            self.rotate(node)

    def remove(self, node):
        """Remove a stable endpoint without changing any surviving identity."""
        self.c["tree_deletions"] += 1
        while node.left or node.right:
            if node.left and node.right:
                self.c["tree_priority_comparisons"] += 1
                child = node.left if node.left.priority < node.right.priority else node.right
            else:
                child = node.left or node.right
            self.rotate(child)
        parent = node.parent
        if parent is None:
            self.root = None
        elif parent.left is node:
            parent.left = None
        else:
            parent.right = None
        node.parent = None
        self.c["tree_pointer_writes"] += 2
        self.repair(parent)

    def rank(self, node):
        """Find an endpoint's current packed rank only when needed."""
        self.c["tree_rank_queries"] += 1
        result = node.left.total if node.left else 0
        while node.parent:
            self.c["tree_rank_hops"] += 1
            if node.parent.right is node:
                result += node.parent.edge.n + (node.parent.left.total if node.parent.left else 0)
            node = node.parent
        return result

    def locate(self, position):
        """Find the stable interval and local offset at a current row rank."""
        self.c["tree_predecessor_queries"] += 1
        node = self.root
        while node:
            self.c["tree_predecessor_visits"] += 1
            left = node.left.total if node.left else 0
            if position < left:
                node = node.left
            elif position >= left + node.edge.n:
                position -= left + node.edge.n
                node = node.right
            else:
                return node, position - left
        raise AssertionError("rank outside surviving order")

    def neighbor(self, node, right=True):
        """Return an adjacent stable endpoint, counting traversal hops."""
        self.c["tree_neighbor_queries"] += 1
        child = node.right if right else node.left
        if child:
            node = child
            while node.left if right else node.right:
                self.c["tree_neighbor_hops"] += 1
                node = node.left if right else node.right
            return node
        while node.parent and (node.parent.right if right else node.parent.left) is node:
            self.c["tree_neighbor_hops"] += 1
            node = node.parent
        return node.parent

    def scan(self):
        """Enumerate current order for global selection; this work is not deferred."""
        stack, node, position = [], self.root, 0
        result = []
        while node or stack:
            while node:
                stack.append(node)
                self.c["tree_scan_stack_pushes"] += 1
                node = node.left
            node = stack.pop()
            result.append((position, node.edge))
            self.c["selection_input_visits"] += 1
            self.c["coordinate_prefix_additions"] += 1
            position += node.edge.n
            node = node.right
        return result


class Stable:
    """Two stable orders with boundary-local splice and exact local coalescing."""

    def __init__(self, pieces, grammar, marker, c):
        self.c, self.grammar = c, grammar
        self.top, self.bottom = Order(c), Order(c)
        self.edges, self.next_id = {}, 0
        self.marker, self.offset = None, 0
        by_start, prev = {}, None
        for a in pieces:
            edge = self.allocate(a.n, a.w)
            by_start[a.s] = edge
            self.top.insert_after(prev, edge.top)
            prev = edge.top
            if a.s <= marker < a.s + a.n:
                self.marker, self.offset = edge, marker - a.s
        prev = None
        for a in ordered(pieces, lambda a: a.t, c):
            edge = by_start[a.s]
            self.bottom.insert_after(prev, edge.bottom)
            prev = edge.bottom
        assert self.marker is not None
        c["marker_records"] = 1

    def allocate(self, width, word):
        """Allocate one stable edge and its two endpoint nodes."""
        edge = Edge(self.next_id, width, word)
        self.next_id += 1
        edge.top, edge.bottom = Node(edge, 17), Node(edge, 91)
        self.edges[edge.id] = edge
        self.c["persistent_edge_records_created"] += 1
        self.c["persistent_tree_nodes_created"] += 2
        self.c["peak_live_edges"] = max(self.c["peak_live_edges"], len(self.edges))
        self.c["peak_live_tree_nodes"] = max(self.c["peak_live_tree_nodes"], 2 * len(self.edges))
        return edge

    def split(self, edge, offset):
        """Split only at a required boundary, keeping the left identity stable."""
        if offset == 0:
            return edge
        assert 0 < offset < edge.n
        other = self.allocate(edge.n - offset, edge.w)
        edge.n = offset
        self.c["width_writes"] += 1
        self.top.repair(edge.top)
        self.bottom.repair(edge.bottom)
        self.top.insert_after(edge.top, other.top)
        self.bottom.insert_after(edge.bottom, other.bottom)
        self.c["splits"] += 1
        self.c["marker_identity_checks"] += 1
        if self.marker is edge and self.offset >= offset:
            self.marker, self.offset = other, self.offset - offset
            self.c["marker_updates"] += 1
        return other

    def coalesce(self, candidates):
        """Recheck only changed adjacencies, including across deleted gaps."""
        pending = list(candidates)
        while pending:
            identity = pending.pop()
            self.c["coalescing_identity_queries"] += 1
            if identity not in self.edges:
                continue
            a = self.edges[identity]
            next_node = self.top.neighbor(a.top)
            self.c["coalescing_checks"] += 1
            if next_node is None:
                continue
            b = next_node.edge
            if a.w != b.w or self.bottom.neighbor(a.bottom) is not b.bottom:
                continue
            prev = self.top.neighbor(a.top, False)
            old_width = a.n
            self.top.remove(b.top)
            self.bottom.remove(b.bottom)
            a.n += b.n
            self.c["width_writes"] += 1
            self.top.repair(a.top)
            self.bottom.repair(a.bottom)
            self.c["marker_identity_checks"] += 1
            if self.marker is b:
                self.marker, self.offset = a, self.offset + old_width
                self.c["marker_updates"] += 1
            del self.edges[b.id]
            self.c["edges_retired"] += 1
            self.c["merges"] += 1
            pending.extend([a.id] + ([prev.edge.id] if prev else []))

    def contract(self, seed):
        """Select globally, then splice selected ranges from right to left."""
        top, bottom = self.top.scan(), self.bottom.scan()
        marker = self.top.rank(self.marker.top) + self.offset
        edges, removed = select(top, bottom, marker, seed, self.c)
        # Match the original grammar's interning order exactly, so later color
        # seeds choose identical deletions. Route identities remain separate.
        words = {}
        for _, _, _, _, a, b, take in edges:
            if take:
                words[(a.w, b.w)] = self.grammar.cat(a.w, b.w)
                self.c["prepared_word_assignments"] += 1
        self.c["peak_transient_entries"] = max(
            self.c["peak_transient_entries"],
            len(top) + len(bottom) + len(edges) + len(removed) + len(words),
        )
        for lo, hi in reversed(removed):
            remaining = hi - lo
            while remaining:
                an, ao = self.bottom.locate(lo)
                bn, bo = self.top.locate(lo)
                a, b = an.edge, bn.edge
                assert a is not b
                width = min(remaining, a.n - ao, b.n - bo)
                a, b = self.split(a, ao), self.split(b, bo)
                if a.n > width:
                    self.split(a, width)
                if b.n > width:
                    self.split(b, width)
                assert a.n == b.n == width and self.marker is not b
                affected = {a.id}
                for order, node in ((self.top, b.top), (self.bottom, a.bottom), (self.top, a.top)):
                    for direction in (False, True):
                        neighbor = order.neighbor(node, direction)
                        if neighbor:
                            affected.add(neighbor.edge.id)
                self.c["prepared_word_queries"] += 1
                word = words[(a.w, b.w)]
                self.bottom.remove(a.bottom)
                self.top.remove(b.top)
                a.bottom = b.bottom
                a.bottom.edge = a
                a.w = word
                self.c["edge_attachment_writes"] += 2
                self.c["word_id_writes"] += 1
                del self.edges[b.id]
                self.c["edges_retired"] += 1
                self.c["splice_updates"] += 1
                remaining -= width
                self.coalesce(affected)
        return removed

    def export(self):
        """Validation-only packed view; never used by selection or contraction."""

        def visit(node):
            if node:
                yield from visit(node.left)
                yield node.edge
                yield from visit(node.right)

        images, total = {}, 0
        for a in visit(self.bottom.root):
            images[a.id] = total
            total += a.n
        result, source = [], 0
        marker = None
        for a in visit(self.top.root):
            result.append(Piece(source, images[a.id], a.n, a.w))
            if a is self.marker:
                marker = source + self.offset
            source += a.n
        assert source == total and marker is not None
        return result, marker

    def decode(self, n):
        """Traverse stable routing directly, charging needed rank/lookup work."""
        edge, offset = self.marker, self.offset
        emitted = bytearray()
        while True:
            self.c["output_piece_visits"] += 1
            emitted.extend(self.grammar.expand(edge.w))
            target = self.bottom.rank(edge.bottom) + offset
            node, offset = self.top.locate(target)
            edge = node.edge
            if edge is self.marker and offset == self.offset:
                break
            assert len(emitted) <= n
        reverse = bytes(reversed(emitted))
        repeat = (n + len(reverse) - 1) // len(reverse)
        self.c["cycle_reversal_bytes"] += len(reverse)
        self.c["repeated_bytes"] += repeat * len(reverse)
        self.c["output_bytes"] += n
        return (reverse * repeat)[:n]
