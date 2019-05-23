use super::task::{Id, Status, Task};
use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashSet},
    time::Instant,
};

#[derive(PartialEq, Eq)]
struct Record {
    when: Instant,
    tid: Id,
}

impl Ord for Record {
    fn cmp(&self, other: &Record) -> Ordering {
        // `BinaryHeap` is a max-heap, so we need to reverse the order of
        // comparisons in order to get `peek()` and `pop()` to return the
        // smallest time.
        match self.when.cmp(&other.when) {
            Ordering::Equal => Ordering::Equal,
            Ordering::Less => Ordering::Greater,
            Ordering::Greater => Ordering::Less,
        }
    }
}

impl PartialOrd for Record {
    fn partial_cmp(&self, other: &Record) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Default)]
pub struct Schedule {
    ids: HashSet<Id>,
    heap: BinaryHeap<Record>,
    clock: Option<Instant>,
}

impl Schedule {
    pub fn schedule<'a, T>(&mut self, t: &Task<'a, T>) {
        // todo: there must be a way to segregate tasks from different
        // namespaces.
        match t.status() {
            Status::Completed(_) => {
                panic!("attempt to schedule a completed task")
            }
            Status::AsleepUntil(when) => {
                self.ids.insert(*t.id());
                self.heap.push(Record {
                    when: *when,
                    tid: *t.id(),
                });
            }
        }
    }

    pub fn cancel(&mut self, id: Id) {
        assert!(self.ids.remove(&id));
    }

    pub fn poll(&mut self, now: Instant) -> Option<Id> {
        self.advance_clock(now);
        if let Some(rec) = self.heap.peek() {
            if rec.when < now {
                // next task due isn't due yet.
                None
            } else {
                // next task is due.
                let rec = self.heap.pop().unwrap();
                if self.ids.contains(&rec.tid) {
                    self.cancel(rec.tid);
                    Some(rec.tid)
                } else {
                    // task is due but was cancelled; discard and try again.
                    self.poll(now)
                }
            }
        } else {
            // nothing in the heap.
            None
        }
    }

    fn advance_clock(&mut self, now: Instant) {
        if let Some(clock) = self.clock.as_mut() {
            assert!(*clock <= now);
            *clock = now;
        } else {
            self.clock = Some(now)
        }
    }
}
