//! Registration-owned callbacks delivered by Dioxus effects. Keeping captures
//! here lets unregister/replacement release them before a queued effect runs.

use std::{collections::BTreeMap, rc::Rc};

struct Entry {
    root: String,
    generation: u64,
    callback: Rc<dyn Fn()>,
}

#[derive(Default)]
pub(crate) struct CompletionQueue {
    sequence: u64,
    jobs: BTreeMap<u64, Entry>,
}

impl CompletionQueue {
    pub(crate) fn admit(&mut self, root: String, generation: u64, callback: Rc<dyn Fn()>) -> u64 {
        loop {
            self.sequence = self.sequence.wrapping_add(1).max(1);
            if !self.jobs.contains_key(&self.sequence) {
                break;
            }
        }
        self.jobs.insert(
            self.sequence,
            Entry {
                root,
                generation,
                callback,
            },
        );
        self.sequence
    }

    pub(crate) fn take(
        &mut self,
        sequence: u64,
        root: &str,
        generation: u64,
    ) -> Option<Rc<dyn Fn()>> {
        let entry = self.jobs.get(&sequence)?;
        if entry.root != root || entry.generation != generation {
            return None;
        }
        self.jobs.remove(&sequence).map(|entry| entry.callback)
    }

    pub(crate) fn remove_root(&mut self, root: &str, generation: u64) {
        self.jobs
            .retain(|_, entry| entry.root != root || entry.generation != generation);
    }

    pub(crate) fn clear(&mut self) {
        self.jobs.clear();
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.jobs.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::{Cell, RefCell};

    #[test]
    fn immutable_jobs_keep_each_write_and_resolve_current_handler() {
        let mut queue = CompletionQueue::default();
        let current_handler = Rc::new(Cell::new("old"));
        let delivered = Rc::new(RefCell::new(Vec::new()));
        let mut tickets = Vec::new();
        for value in [true, false, true] {
            let handler = current_handler.clone();
            let delivered = delivered.clone();
            tickets.push(queue.admit(
                "root".into(),
                1,
                Rc::new(move || {
                    delivered.borrow_mut().push((value, handler.get()));
                }),
            ));
        }
        current_handler.set("current");
        assert!(delivered.borrow().is_empty());
        for ticket in tickets {
            queue.take(ticket, "root", 1).unwrap()();
        }
        assert_eq!(
            *delivered.borrow(),
            [(true, "current"), (false, "current"), (true, "current")]
        );
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn registration_cleanup_releases_captures_and_does_not_retarget_jobs() {
        let mut queue = CompletionQueue::default();
        let retained = Rc::new(Cell::new(false));
        let weak = Rc::downgrade(&retained);
        let old = queue.admit("root".into(), 1, Rc::new(move || retained.set(true)));
        let current = queue.admit("root".into(), 2, Rc::new(|| {}));
        assert!(queue.take(old, "root", 2).is_none());
        queue.remove_root("root", 1);
        assert!(weak.upgrade().is_none());
        assert!(queue.take(old, "root", 1).is_none());
        assert!(queue.take(current, "root", 2).is_some());
        assert!(queue.take(current, "root", 2).is_none());
    }
}
