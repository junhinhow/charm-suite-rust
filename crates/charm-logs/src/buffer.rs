use crate::Entry;
use std::sync::Mutex;

/// Thread-safe fixed-capacity ring buffer. The mutex intentionally protects
/// only the data, not the viewer's UI state.
#[derive(Debug)]
pub(crate) struct Buffer(Mutex<Ring>);

#[derive(Debug)]
struct Ring {
    entries: Vec<Option<Entry>>,
    start: usize,
    len: usize,
    next_seq: u64,
}

impl Buffer {
    pub(crate) fn new(capacity: usize) -> Self {
        Self(Mutex::new(Ring {
            entries: vec![None; capacity.max(10)],
            start: 0,
            len: 0,
            next_seq: 0,
        }))
    }
    pub(crate) fn append(&self, entry: Entry) {
        let mut ring = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        ring.next_seq = ring.next_seq.wrapping_add(1);
        let cap = ring.entries.len();
        if ring.len == cap {
            let start = ring.start;
            ring.entries[start] = Some(entry);
            ring.start = (start + 1) % cap;
        } else {
            let index = (ring.start + ring.len) % cap;
            ring.entries[index] = Some(entry);
            ring.len += 1;
        }
    }
    pub(crate) fn snapshot(&self) -> (Vec<(u64, Entry)>, u64) {
        let ring = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if ring.len == 0 {
            return (Vec::new(), 0);
        }
        let first = ring
            .next_seq
            .saturating_sub(ring.len as u64)
            .saturating_add(1);
        let out = (0..ring.len)
            .filter_map(|i| {
                ring.entries[(ring.start + i) % ring.entries.len()]
                    .clone()
                    .map(|e| (first + i as u64, e))
            })
            .collect();
        (out, first)
    }
    pub(crate) fn len(&self) -> usize {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len
    }
    pub(crate) fn clear(&self) {
        let mut ring = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        ring.entries.fill(None);
        ring.start = 0;
        ring.len = 0;
        ring.next_seq = 0;
    }
}
