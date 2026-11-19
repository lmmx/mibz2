"""Monotone queues for first-return cohorts whose remaining occurrence counts only shrink."""
from collections import deque


class MassBuckets:
    def __init__(self, source_mass, batch):
        levels = source_mass.bit_length()
        self.queues = [[deque() for _ in range(levels)] for _ in range(2)]
        self.bits = [0,0]
        self.batch = batch
        self.side = 0
        self.streak = 0
        self.owner = None
        self.pops = 0
        self.demotions = 0
        self.side_changes = 0

    def add(self, mass, side, gid):
        level = mass.bit_length()-1
        self.queues[side][level].append(gid)
        self.bits[side] |= 1<<level

    def next(self, live):
        while any(self.bits):
            level = (self.bits[0] | self.bits[1]).bit_length()-1
            side = self.side
            if self.bits[1-side] & (1<<level) and (self.streak >= self.batch or not self.bits[side] & (1<<level)):
                side = 1-side
            queue = self.queues[side][level]
            gid = queue.popleft()
            self.pops += 1
            if not queue:
                self.bits[side] &= ~(1<<level)
            mass = live[gid][3]
            if mass == 0:
                del live[gid]
            elif mass.bit_length()-1 != level:
                self.demotions += 1
                self.add(mass, side, gid)
            else:
                if self.owner != side:
                    self.side_changes += self.owner is not None
                    self.streak = 0
                self.owner = self.side = side
                self.streak += 1
                return gid
        return None
