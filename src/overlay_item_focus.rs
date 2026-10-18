//! Independently admitted item-focus and nearest-scroll jobs.
//!
//! Each source item-focus call captures one object, focuses in its first native
//! frame and scrolls that same object in a second frame. The native scheduler
//! preserves enrollment order; this queue authorizes each phase once without
//! making an earlier job's pending scroll block another job's focus.

use std::collections::VecDeque;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ItemFocusJob {
    pub(crate) sequence: u64,
    pub(crate) root: String,
    pub(crate) generation: u64,
    pub(crate) target: String,
    pub(crate) focus_native: bool,
    pub(crate) capture: Option<String>,
}

impl ItemFocusJob {
    fn matches(&self, sequence: u64, root: &str, generation: u64, target: &str) -> bool {
        self.sequence == sequence
            && self.root == root
            && self.generation == generation
            && self.target == target
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Admitted,
    Armed,
    AwaitingScroll,
}

struct Entry {
    job: ItemFocusJob,
    phase: Phase,
    sent: bool,
}

#[derive(Default)]
pub(crate) struct ItemFocusQueue {
    sequence: u64,
    jobs: VecDeque<Entry>,
}

impl ItemFocusQueue {
    /// Registration and source-path eligibility are checked by the runtime.
    /// No visibility, revision or latest-intent state belongs in this queue.
    pub(crate) fn admit(
        &mut self,
        root: String,
        generation: u64,
        target: String,
        focus_native: bool,
        capture: Option<String>,
    ) -> ItemFocusJob {
        loop {
            self.sequence = self.sequence.wrapping_add(1).max(1);
            if !self
                .jobs
                .iter()
                .any(|entry| entry.job.sequence == self.sequence)
            {
                break;
            }
        }
        let job = ItemFocusJob {
            sequence: self.sequence,
            root,
            generation,
            target,
            focus_native,
            capture,
        };
        self.jobs.push_back(Entry {
            job: job.clone(),
            phase: Phase::Admitted,
            sent: false,
        });
        job
    }

    /// Bridge enrollment is independent of frame authorization. Jobs admitted
    /// before bootstrap remain available for the first bridge flush.
    pub(crate) fn unsent(&self) -> Vec<ItemFocusJob> {
        self.jobs
            .iter()
            .filter(|entry| !entry.sent)
            .map(|entry| entry.job.clone())
            .collect()
    }

    pub(crate) fn mark_sent(&mut self, job: &ItemFocusJob) -> bool {
        let Some(entry) = self.jobs.iter_mut().find(|entry| entry.job == *job) else {
            return false;
        };
        if entry.sent {
            return false;
        }
        entry.sent = true;
        true
    }

    /// Bind the actual native object once. A capture supplied at admission must
    /// match; an absent bootstrap capture may be filled by this first arm only.
    pub(crate) fn arm(
        &mut self,
        sequence: u64,
        root: &str,
        generation: u64,
        target: &str,
        capture: &str,
    ) -> bool {
        let Some(entry) = self
            .jobs
            .iter_mut()
            .find(|entry| entry.job.matches(sequence, root, generation, target))
        else {
            return false;
        };
        if entry.phase != Phase::Admitted
            || capture.is_empty()
            || entry
                .job
                .capture
                .as_deref()
                .is_some_and(|expected| expected != capture)
        {
            return false;
        }
        entry.job.capture = Some(capture.into());
        entry.phase = Phase::Armed;
        true
    }

    /// Consume first-frame authorization before native focus. The job retains
    /// the same capture for its independent second-frame scroll authorization,
    /// including when native focus was skipped or did not change activeElement.
    pub(crate) fn take_focus(
        &mut self,
        sequence: u64,
        root: &str,
        generation: u64,
        target: &str,
        capture: &str,
    ) -> Option<ItemFocusJob> {
        let entry = self
            .jobs
            .iter_mut()
            .find(|entry| entry.job.matches(sequence, root, generation, target))?;
        if entry.phase != Phase::Armed || entry.job.capture.as_deref() != Some(capture) {
            return None;
        }
        entry.phase = Phase::AwaitingScroll;
        Some(entry.job.clone())
    }

    /// Consume second-frame authorization for the first-frame object. Wrong
    /// tuples and replays never consume another admitted job or retarget it.
    pub(crate) fn take_scroll(
        &mut self,
        sequence: u64,
        root: &str,
        generation: u64,
        target: &str,
        capture: &str,
    ) -> Option<ItemFocusJob> {
        let index = self.jobs.iter().position(|entry| {
            entry.job.matches(sequence, root, generation, target)
                && entry.phase == Phase::AwaitingScroll
                && entry.job.capture.as_deref() == Some(capture)
        })?;
        self.jobs.remove(index).map(|entry| entry.job)
    }

    /// Dropped or replaced owners cancel only their own registration's work.
    pub(crate) fn remove_root(&mut self, root: &str, generation: u64) {
        self.jobs
            .retain(|entry| entry.job.root != root || entry.job.generation != generation);
    }

    pub(crate) fn clear(&mut self) {
        self.jobs.clear();
    }

    /// Failed native enrollment releases only that still-unarmed admission.
    /// A duplicate command cannot cancel an already executing job.
    pub(crate) fn discard(
        &mut self,
        sequence: u64,
        root: &str,
        generation: u64,
        target: &str,
        capture: Option<&str>,
    ) -> bool {
        let Some(index) = self.jobs.iter().position(|entry| {
            entry.job.matches(sequence, root, generation, target)
                && entry.phase == Phase::Admitted
                && entry.job.capture.as_deref() == capture
        }) else {
            return false;
        };
        self.jobs.remove(index);
        true
    }

    /// Native candidates that disappeared during a Rust callback remain held
    /// only while an admitted job still owns their exact object token.
    pub(crate) fn retains_token(&self, capture: &str) -> bool {
        self.jobs
            .iter()
            .any(|entry| entry.job.capture.as_deref() == Some(capture))
    }

    #[cfg(test)]
    pub(crate) fn jobs(&self) -> Vec<ItemFocusJob> {
        self.jobs.iter().map(|entry| entry.job.clone()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn admit(queue: &mut ItemFocusQueue, target: &str, capture: &str) -> ItemFocusJob {
        queue.admit("root".into(), 3, target.into(), true, Some(capture.into()))
    }

    fn arm(queue: &mut ItemFocusQueue, job: &ItemFocusJob) -> bool {
        queue.arm(
            job.sequence,
            &job.root,
            job.generation,
            &job.target,
            job.capture.as_deref().unwrap(),
        )
    }

    fn focus(queue: &mut ItemFocusQueue, job: &ItemFocusJob) -> Option<ItemFocusJob> {
        queue.take_focus(
            job.sequence,
            &job.root,
            job.generation,
            &job.target,
            job.capture.as_deref().unwrap(),
        )
    }

    fn scroll(queue: &mut ItemFocusQueue, job: &ItemFocusJob) -> Option<ItemFocusJob> {
        queue.take_scroll(
            job.sequence,
            &job.root,
            job.generation,
            &job.target,
            job.capture.as_deref().unwrap(),
        )
    }

    #[test]
    fn rapid_targets_focus_in_one_frame_before_either_second_frame_scroll() {
        let mut queue = ItemFocusQueue::default();
        let first = admit(&mut queue, "a", "actual-a");
        let second = admit(&mut queue, "b", "actual-b");
        assert!(arm(&mut queue, &first));
        assert!(arm(&mut queue, &second));
        assert_eq!(focus(&mut queue, &first), Some(first.clone()));
        assert_eq!(focus(&mut queue, &second), Some(second.clone()));
        assert_eq!(scroll(&mut queue, &first), Some(first));
        assert_eq!(scroll(&mut queue, &second), Some(second));
        assert!(queue.unsent().is_empty());
    }

    #[test]
    fn every_phase_rejects_wrong_tuple_and_replay_without_consuming_the_job() {
        let mut queue = ItemFocusQueue::default();
        let job = admit(&mut queue, "item", "actual");
        assert!(focus(&mut queue, &job).is_none());
        assert!(scroll(&mut queue, &job).is_none());
        for (sequence, root, generation, target, capture) in [
            (job.sequence + 1, "root", 3, "item", "actual"),
            (job.sequence, "other", 3, "item", "actual"),
            (job.sequence, "root", 4, "item", "actual"),
            (job.sequence, "root", 3, "other", "actual"),
            (job.sequence, "root", 3, "item", "replacement"),
            (job.sequence, "root", 3, "item", ""),
        ] {
            assert!(!queue.arm(sequence, root, generation, target, capture));
        }
        assert!(arm(&mut queue, &job));
        assert!(!arm(&mut queue, &job));
        for (sequence, root, generation, target, capture) in [
            (job.sequence + 1, "root", 3, "item", "actual"),
            (job.sequence, "other", 3, "item", "actual"),
            (job.sequence, "root", 4, "item", "actual"),
            (job.sequence, "root", 3, "other", "actual"),
            (job.sequence, "root", 3, "item", "replacement"),
        ] {
            assert!(
                queue
                    .take_focus(sequence, root, generation, target, capture)
                    .is_none()
            );
        }
        assert!(scroll(&mut queue, &job).is_none());
        assert_eq!(focus(&mut queue, &job), Some(job.clone()));
        assert!(focus(&mut queue, &job).is_none());
        for (sequence, root, generation, target, capture) in [
            (job.sequence + 1, "root", 3, "item", "actual"),
            (job.sequence, "other", 3, "item", "actual"),
            (job.sequence, "root", 4, "item", "actual"),
            (job.sequence, "root", 3, "other", "actual"),
            (job.sequence, "root", 3, "item", "replacement"),
        ] {
            assert!(
                queue
                    .take_scroll(sequence, root, generation, target, capture)
                    .is_none()
            );
        }
        assert_eq!(scroll(&mut queue, &job), Some(job.clone()));
        assert!(scroll(&mut queue, &job).is_none());
        assert!(!arm(&mut queue, &job));
    }

    #[test]
    fn bootstrap_capture_binds_once_and_same_id_replacement_cannot_retarget() {
        let mut queue = ItemFocusQueue::default();
        let job = queue.admit("root".into(), 1, "same-id".into(), false, None);
        assert!(!queue.arm(job.sequence, "root", 1, "same-id", ""));
        assert!(queue.arm(job.sequence, "root", 1, "same-id", "original-object"));
        assert!(!queue.arm(job.sequence, "root", 1, "same-id", "new-object"));
        assert!(
            queue
                .take_focus(job.sequence, "root", 1, "same-id", "new-object")
                .is_none()
        );
        let focused = queue
            .take_focus(job.sequence, "root", 1, "same-id", "original-object")
            .unwrap();
        assert!(
            !focused.focus_native,
            "proxy-only focus still admits nearest scroll"
        );
        assert_eq!(focused.capture.as_deref(), Some("original-object"));
        assert!(
            queue
                .take_scroll(job.sequence, "root", 1, "same-id", "new-object")
                .is_none()
        );
        assert_eq!(scroll(&mut queue, &focused), Some(focused));
    }

    #[test]
    fn repeated_same_target_admissions_survive_later_jobs_and_bridge_enrollment() {
        let mut queue = ItemFocusQueue::default();
        let first = admit(&mut queue, "same", "actual");
        let second = admit(&mut queue, "same", "actual");
        assert_ne!(first.sequence, second.sequence);
        assert_eq!(queue.unsent(), [first.clone(), second.clone()]);
        let mut wrong = first.clone();
        wrong.focus_native = false;
        assert!(!queue.mark_sent(&wrong));
        assert!(queue.mark_sent(&first));
        assert!(!queue.mark_sent(&first));
        assert_eq!(queue.unsent().as_slice(), std::slice::from_ref(&second));
        assert!(arm(&mut queue, &first));
        assert_eq!(focus(&mut queue, &first), Some(first.clone()));
        let later = admit(&mut queue, "later", "later-object");
        assert!(arm(&mut queue, &second));
        assert_eq!(focus(&mut queue, &second), Some(second.clone()));
        assert_eq!(scroll(&mut queue, &first), Some(first));
        assert_eq!(scroll(&mut queue, &second), Some(second));
        assert!(arm(&mut queue, &later));
        assert_eq!(focus(&mut queue, &later), Some(later.clone()));
        assert_eq!(scroll(&mut queue, &later), Some(later));
    }

    #[test]
    fn registration_cleanup_preserves_replacement_and_unrelated_phases() {
        let mut queue = ItemFocusQueue::default();
        let old = admit(&mut queue, "old", "old-object");
        assert!(arm(&mut queue, &old));
        assert!(focus(&mut queue, &old).is_some());
        let old_admitted = admit(&mut queue, "not-armed", "unarmed-object");
        let old_armed = admit(&mut queue, "not-focused", "armed-object");
        assert!(arm(&mut queue, &old_armed));
        let replacement = queue.admit(
            "root".into(),
            4,
            "new".into(),
            true,
            Some("new-object".into()),
        );
        let unrelated = queue.admit(
            "other".into(),
            3,
            "peer".into(),
            true,
            Some("peer-object".into()),
        );
        assert!(arm(&mut queue, &replacement));
        assert!(arm(&mut queue, &unrelated));
        queue.remove_root("root", 3);
        assert!(scroll(&mut queue, &old).is_none());
        assert!(!arm(&mut queue, &old));
        assert!(!arm(&mut queue, &old_admitted));
        assert!(focus(&mut queue, &old_armed).is_none());
        assert!(focus(&mut queue, &replacement).is_some());
        assert!(focus(&mut queue, &unrelated).is_some());
        assert_eq!(scroll(&mut queue, &replacement), Some(replacement));
        queue.clear();
        assert!(scroll(&mut queue, &unrelated).is_none());
        assert!(queue.unsent().is_empty());
    }

    #[test]
    fn failed_capture_enrollment_releases_only_the_exact_unarmed_admission() {
        let mut queue = ItemFocusQueue::default();
        let missing = queue.admit("root".into(), 1, "missing".into(), true, None);
        let captured = admit(&mut queue, "item", "actual");
        assert!(!queue.discard(missing.sequence, "root", 2, "missing", None));
        assert!(!queue.discard(missing.sequence, "root", 1, "replacement", None));
        assert!(!queue.discard(missing.sequence, "root", 1, "missing", Some("new")));
        assert!(queue.discard(missing.sequence, "root", 1, "missing", None));
        assert!(!queue.discard(missing.sequence, "root", 1, "missing", None));
        assert!(!queue.discard(captured.sequence, "root", 3, "item", Some("replacement")));
        assert!(arm(&mut queue, &captured));
        assert!(!queue.discard(captured.sequence, "root", 3, "item", Some("actual")));
        assert!(focus(&mut queue, &captured).is_some());
        assert_eq!(scroll(&mut queue, &captured), Some(captured));
    }
}
