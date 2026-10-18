//! Immutable per-registration completion callbacks, independent of current open
//! state, request revision and the latest native event acknowledgement.

use std::{collections::VecDeque, rc::Rc};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CompletionFrame {
    pub(crate) sequence: u64,
    pub(crate) root: String,
    pub(crate) generation: u64,
}

struct Entry {
    frame: CompletionFrame,
    callback: Rc<dyn Fn()>,
    sent: bool,
}

#[derive(Default)]
pub(crate) struct CompletionQueue {
    sequence: u64,
    jobs: VecDeque<Entry>,
}

impl CompletionQueue {
    pub(crate) fn admit(
        &mut self,
        root: String,
        generation: u64,
        callback: Rc<dyn Fn()>,
    ) -> CompletionFrame {
        loop {
            self.sequence = self.sequence.wrapping_add(1).max(1);
            if !self
                .jobs
                .iter()
                .any(|entry| entry.frame.sequence == self.sequence)
            {
                break;
            }
        }
        let frame = CompletionFrame {
            sequence: self.sequence,
            root,
            generation,
        };
        self.jobs.push_back(Entry {
            frame: frame.clone(),
            callback,
            sent: false,
        });
        frame
    }

    pub(crate) fn unsent(&self) -> Vec<CompletionFrame> {
        self.jobs
            .iter()
            .filter(|entry| !entry.sent)
            .map(|entry| entry.frame.clone())
            .collect()
    }

    pub(crate) fn mark_sent(&mut self, frame: &CompletionFrame) {
        if let Some(entry) = self.jobs.iter_mut().find(|entry| entry.frame == *frame) {
            entry.sent = true;
        }
    }

    /// The explicit frame ACK also supports renderer-only test delivery before
    /// a bridge exists. Sent state only prevents duplicate native enrollment.
    pub(crate) fn take(
        &mut self,
        sequence: u64,
        root: &str,
        generation: u64,
    ) -> Option<Rc<dyn Fn()>> {
        let frame = &self.jobs.front()?.frame;
        if frame.sequence != sequence || frame.root != root || frame.generation != generation {
            return None;
        }
        self.jobs.pop_front().map(|entry| entry.callback)
    }

    pub(crate) fn remove_root(&mut self, root: &str, generation: u64) {
        self.jobs
            .retain(|entry| entry.frame.root != root || entry.frame.generation != generation);
    }

    pub(crate) fn clear(&mut self) {
        self.jobs.clear();
    }

    #[cfg(test)]
    pub(crate) fn frames(&self) -> Vec<CompletionFrame> {
        self.jobs.iter().map(|entry| entry.frame.clone()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::{Cell, RefCell};

    #[test]
    fn immutable_jobs_keep_each_write_while_current_state_and_handler_can_change() {
        let mut queue = CompletionQueue::default();
        let state = Rc::new(Cell::new(false));
        let current_handler = Rc::new(Cell::new("old"));
        let delivered = Rc::new(RefCell::new(Vec::new()));
        for value in [true, false, true] {
            state.set(value);
            let state = state.clone();
            let handler = current_handler.clone();
            let delivered = delivered.clone();
            queue.admit(
                "root".into(),
                1,
                Rc::new(move || {
                    delivered
                        .borrow_mut()
                        .push((value, state.get(), handler.get()));
                }),
            );
        }
        current_handler.set("current");
        assert!(delivered.borrow().is_empty());
        let frames = queue.frames();
        for frame in &frames {
            queue
                .take(frame.sequence, &frame.root, frame.generation)
                .unwrap()();
        }
        assert_eq!(
            *delivered.borrow(),
            [
                (true, true, "current"),
                (false, true, "current"),
                (true, true, "current")
            ]
        );
        assert!(queue.frames().is_empty());
    }

    #[test]
    fn fifo_exact_tuple_and_registration_cleanup_do_not_consume_unrelated_jobs() {
        let mut queue = CompletionQueue::default();
        let old = queue.admit("root".into(), 1, Rc::new(|| {}));
        let next = queue.admit("root".into(), 2, Rc::new(|| {}));
        assert!(queue.take(next.sequence, "root", 2).is_none());
        assert!(queue.take(old.sequence, "wrong", 1).is_none());
        assert!(queue.take(old.sequence, "root", 2).is_none());
        assert_eq!(queue.unsent(), [old.clone(), next.clone()]);
        queue.mark_sent(&old);
        assert_eq!(queue.unsent().as_slice(), std::slice::from_ref(&next));
        queue.remove_root("root", 1);
        assert!(queue.take(old.sequence, "root", 1).is_none());
        assert!(queue.take(next.sequence, "root", 2).is_some());
        assert!(queue.take(next.sequence, "root", 2).is_none());
        let retained = Rc::new(Cell::new(false));
        let weak = Rc::downgrade(&retained);
        queue.admit("other".into(), 1, Rc::new(move || retained.set(true)));
        queue.clear();
        assert!(
            weak.upgrade().is_none(),
            "cleanup releases stored callback captures"
        );
    }

    #[test]
    fn reentrant_completion_is_not_part_of_the_preexisting_frame_snapshot() {
        let queue = Rc::new(RefCell::new(CompletionQueue::default()));
        let delivered = Rc::new(RefCell::new(Vec::new()));
        let weak_queue = Rc::downgrade(&queue);
        let first_delivered = delivered.clone();
        queue.borrow_mut().admit(
            "root".into(),
            1,
            Rc::new(move || {
                first_delivered.borrow_mut().push(true);
                let later = first_delivered.clone();
                weak_queue.upgrade().unwrap().borrow_mut().admit(
                    "root".into(),
                    1,
                    Rc::new(move || later.borrow_mut().push(false)),
                );
            }),
        );
        let frames = queue.borrow().frames();
        for frame in frames {
            let callback = queue
                .borrow_mut()
                .take(frame.sequence, &frame.root, frame.generation)
                .unwrap();
            callback();
        }
        assert_eq!(*delivered.borrow(), [true]);
        let next = queue.borrow().frames();
        assert_eq!(next.len(), 1);
        let callback = queue
            .borrow_mut()
            .take(next[0].sequence, "root", 1)
            .unwrap();
        callback();
        assert_eq!(*delivered.borrow(), [true, false]);
    }
}
