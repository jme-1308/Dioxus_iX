//! Rust-owned deferred focus jobs, independent of the latest event ACK.
//!
//! Source Dropdown queues each computed exit target after dismissal callbacks,
//! including vetoed dismissal. Later same-owner visibility/revision changes do
//! not revoke those callbacks. Native code only schedules and executes a frame.

use std::collections::{BTreeMap, btree_map::Entry};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PendingFocus {
    pub(crate) sequence: u64,
    pub(crate) target: String,
    pub(crate) root: String,
    pub(crate) generation: u64,
}

impl PendingFocus {
    /// Dropped/replaced registration protection is an explicit Rust lifetime
    /// adapter. Visibility and revision are deliberately not authorization gates.
    pub(crate) fn matches_owner(&self, root: &str, generation: u64) -> bool {
        self.root == root && self.generation == generation
    }
}

struct PendingEntry {
    intent: PendingFocus,
    armed: bool,
    routed: bool,
}

#[derive(Default)]
pub(crate) struct PendingFocusQueue {
    jobs: BTreeMap<u64, PendingEntry>,
}

impl PendingFocusQueue {
    /// A validated per-runtime report admits one source job. Reentrant admission
    /// may arrive out of sequence, so no latest-sequence watermark is used here.
    /// Runtime report validation rejects replayed admissions after consumption.
    pub(crate) fn admit(&mut self, intent: PendingFocus) -> bool {
        match self.jobs.entry(intent.sequence) {
            Entry::Vacant(entry) => {
                entry.insert(PendingEntry {
                    intent,
                    armed: false,
                    routed: false,
                });
                true
            }
            Entry::Occupied(_) => false,
        }
    }

    /// Authorize native scheduling once; wrong tuple and duplicate scheduling
    /// requests do not consume the job or grant another animation-frame task.
    pub(crate) fn schedule(&mut self, sequence: u64, target: &str) -> bool {
        let Some(entry) = self.jobs.get_mut(&sequence) else {
            return false;
        };
        if entry.intent.target != target || entry.armed {
            return false;
        }
        entry.armed = true;
        true
    }

    /// Authorize native frame dispatch once for an armed Tab exit; wrong tuple
    /// and duplicate dispatch requests do not consume the job or grant another frame.
    pub(crate) fn route(&mut self, sequence: u64, target: &str) -> bool {
        let Some(entry) = self.jobs.get_mut(&sequence) else {
            return false;
        };
        if !entry.armed || entry.intent.target != target || entry.routed {
            return false;
        }
        entry.routed = true;
        true
    }

    /// Take an armed job before owner reads/callbacks. Release the queue borrow,
    /// then validate only registration/liveness before admitting native focus.
    pub(crate) fn take(&mut self, sequence: u64, target: &str) -> Option<PendingFocus> {
        let entry = self.jobs.get(&sequence)?;
        if !entry.armed || entry.intent.target != target {
            return None;
        }
        self.jobs.remove(&sequence).map(|entry| entry.intent)
    }

    /// Old registration cleanup must not cancel a replacement's work.
    pub(crate) fn remove_root(&mut self, root: &str, generation: u64) {
        self.jobs
            .retain(|_, entry| !entry.intent.matches_owner(root, generation));
    }

    pub(crate) fn clear(&mut self) {
        self.jobs.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn intent(sequence: u64, target: &str, root: &str, generation: u64) -> PendingFocus {
        PendingFocus {
            sequence,
            target: target.into(),
            root: root.into(),
            generation,
        }
    }

    #[test]
    fn native_frame_routing_authorizes_armed_tabs_once_across_out_of_order_reports() {
        let mut queue = PendingFocusQueue::default();
        assert!(queue.admit(intent(3, "unarmed", "root", 1)));
        assert!(!queue.route(3, "unarmed"));
        assert!(!queue.route(99, "target"));
        for sequence in [1, 2, 7, 4, 9] {
            assert!(queue.admit(intent(sequence, "target", "root", 1)));
            assert!(queue.schedule(sequence, "target"));
        }
        assert!(queue.route(1, "target"));
        assert!(queue.route(2, "target"));
        assert!(!queue.route(7, "wrong"));
        assert!(queue.route(7, "target"));
        assert!(!queue.route(7, "target"));
        assert!(queue.route(4, "target"));
        assert!(queue.take(7, "target").is_some());
        assert!(queue.take(4, "target").is_some());
        assert!(queue.route(9, "target"));
    }

    #[test]
    fn later_reports_do_not_erase_independently_admitted_older_job() {
        let mut queue = PendingFocusQueue::default();
        let older = intent(7, "parent-next", "dropdown", 1);
        let newer = intent(99, "another-next", "dropdown", 1);
        assert!(queue.admit(older.clone()));
        assert!(queue.admit(newer.clone()));
        assert!(queue.schedule(99, "another-next"));
        assert_eq!(queue.take(99, "another-next"), Some(newer));
        assert!(queue.schedule(7, "parent-next"));
        assert_eq!(queue.take(7, "parent-next"), Some(older));
    }

    #[test]
    fn reentrant_nested_report_does_not_supersede_resumed_outer_admission() {
        let mut queue = PendingFocusQueue::default();
        // A nested callback admits a later report before its outer callback
        // returns. Their independent source RAF jobs both remain executable.
        let nested = intent(12, "same-target", "root", 2);
        let outer = intent(10, "same-target", "root", 2);
        assert!(queue.admit(nested.clone()));
        assert!(queue.admit(outer.clone()));
        assert!(queue.schedule(12, "same-target"));
        assert!(queue.schedule(10, "same-target"));
        assert_eq!(queue.take(12, "same-target"), Some(nested));
        assert_eq!(queue.take(10, "same-target"), Some(outer));
    }

    #[test]
    fn scheduling_and_execution_each_consume_only_their_matching_admission() {
        let mut queue = PendingFocusQueue::default();
        let job = intent(4, "adjacent", "root", 3);
        assert!(queue.admit(job.clone()));
        assert!(!queue.admit(intent(4, "replacement", "other", 99)));
        assert!(
            queue.take(4, "adjacent").is_none(),
            "unarmed job cannot execute"
        );
        assert!(!queue.schedule(5, "adjacent"));
        assert!(!queue.schedule(4, "wrong"));
        assert!(queue.schedule(4, "adjacent"));
        assert!(
            !queue.schedule(4, "adjacent"),
            "one admission schedules one RAF"
        );
        assert!(queue.take(4, "wrong").is_none());
        assert_eq!(queue.take(4, "adjacent"), Some(job));
        assert!(
            queue.take(4, "adjacent").is_none(),
            "execution consumes once"
        );
        assert!(!queue.schedule(4, "adjacent"));
    }

    #[test]
    fn owner_authorization_ignores_same_registration_visibility_and_revision_changes() {
        let mut queue = PendingFocusQueue::default();
        let job = intent(8, "next", "root", 4);
        assert!(queue.admit(job.clone()));
        assert!(queue.schedule(8, "next"));
        // An admitted source callback remains valid after veto, accepted close,
        // reopen, another Tab, or observer revisions within this registration.
        assert!(job.matches_owner("root", 4));
        assert!(!job.matches_owner("root", 5));
        assert!(!job.matches_owner("other", 4));
        assert_eq!(queue.take(8, "next"), Some(job));
    }

    #[test]
    fn cleanup_rejects_dropped_owner_without_canceling_replacement_or_foreign_job() {
        let mut queue = PendingFocusQueue::default();
        assert!(queue.admit(intent(1, "old", "root", 1)));
        assert!(queue.admit(intent(2, "new", "root", 2)));
        assert!(queue.admit(intent(3, "foreign", "other", 1)));
        assert!(queue.schedule(1, "old"));
        queue.remove_root("root", 1);
        assert!(queue.take(1, "old").is_none());
        assert!(queue.schedule(2, "new"));
        assert!(queue.take(2, "new").is_some());
        assert!(queue.schedule(3, "foreign"));
        queue.clear();
        assert!(queue.take(3, "foreign").is_none());
        assert!(!queue.schedule(3, "foreign"));
    }
}
