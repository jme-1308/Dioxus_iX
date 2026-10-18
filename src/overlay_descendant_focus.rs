//! Independent descendant focus jobs bound to actual native objects.
//!
//! The runtime owns current-show and source-path admission. This queue only
//! authorizes each exact reference and phase once; newer jobs do not supersede
//! older admissions or block their preparation, focus or visual publication.

use crate::descendant_focus::DescendantChoice;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub(crate) enum DescendantPurpose {
    #[default]
    Opening,
    Submenu,
    Return {
        capture: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DescendantJob {
    pub(crate) sequence: u64,
    pub(crate) root: String,
    pub(crate) generation: u64,
    pub(crate) focus_version: u64,
    pub(crate) last: bool,
    pub(crate) allow_checked: bool,
    pub(crate) reference: Option<String>,
    pub(crate) purpose: DescendantPurpose,
}

impl DescendantJob {
    fn matches(&self, sequence: u64, root: &str, generation: u64) -> bool {
        self.sequence == sequence && self.root == root && self.generation == generation
    }

    fn bound_to(&self, sequence: u64, root: &str, generation: u64, reference: &str) -> bool {
        self.matches(sequence, root, generation)
            && !reference.is_empty()
            && self.reference.as_deref() == Some(reference)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Phase {
    Unarmed,
    Armed,
    Prepared(DescendantChoice),
    Focused(DescendantChoice),
}

struct Entry {
    job: DescendantJob,
    phase: Phase,
    sent: bool,
}

#[derive(Default)]
pub(crate) struct DescendantQueue {
    sequence: u64,
    jobs: VecDeque<Entry>,
}

impl DescendantQueue {
    pub(crate) fn admit(
        &mut self,
        root: String,
        generation: u64,
        focus_version: u64,
        last: bool,
        allow_checked: bool,
        reference: Option<String>,
    ) -> DescendantJob {
        self.admit_with_purpose(
            root,
            generation,
            focus_version,
            last,
            allow_checked,
            reference,
            DescendantPurpose::Opening,
        )
    }

    pub(crate) fn admit_submenu(
        &mut self,
        root: String,
        generation: u64,
        focus_version: u64,
        reference: Option<String>,
    ) -> DescendantJob {
        self.admit_with_purpose(
            root,
            generation,
            focus_version,
            false,
            false,
            reference,
            DescendantPurpose::Submenu,
        )
    }

    pub(crate) fn admit_return(
        &mut self,
        root: String,
        generation: u64,
        focus_version: u64,
        reference: Option<String>,
        capture: String,
    ) -> DescendantJob {
        self.admit_with_purpose(
            root,
            generation,
            focus_version,
            false,
            false,
            reference,
            DescendantPurpose::Return { capture },
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn admit_with_purpose(
        &mut self,
        root: String,
        generation: u64,
        focus_version: u64,
        last: bool,
        allow_checked: bool,
        reference: Option<String>,
        purpose: DescendantPurpose,
    ) -> DescendantJob {
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
        let job = DescendantJob {
            sequence: self.sequence,
            root,
            generation,
            focus_version,
            last,
            allow_checked,
            reference,
            purpose,
        };
        self.jobs.push_back(Entry {
            job: job.clone(),
            phase: Phase::Unarmed,
            sent: false,
        });
        job
    }

    /// Bootstrap retains admissions until the native bridge can enroll them.
    pub(crate) fn unsent(&self) -> Vec<DescendantJob> {
        self.jobs
            .iter()
            .filter(|entry| !entry.sent)
            .map(|entry| entry.job.clone())
            .collect()
    }

    pub(crate) fn mark_sent(&mut self, job: &DescendantJob) -> bool {
        let Some(entry) = self.jobs.iter_mut().find(|entry| entry.job == *job) else {
            return false;
        };
        if entry.sent {
            return false;
        }
        entry.sent = true;
        true
    }

    /// Bind an absent bootstrap reference once, or verify the known reference.
    pub(crate) fn arm(
        &mut self,
        sequence: u64,
        root: &str,
        generation: u64,
        reference: &str,
    ) -> bool {
        let Some(entry) = self
            .jobs
            .iter_mut()
            .find(|entry| entry.job.matches(sequence, root, generation))
        else {
            return false;
        };
        if entry.phase != Phase::Unarmed
            || reference.is_empty()
            || entry
                .job
                .reference
                .as_deref()
                .is_some_and(|known| known != reference)
        {
            return false;
        }
        entry.job.reference = Some(reference.into());
        entry.phase = Phase::Armed;
        true
    }

    /// Selection may read an armed job without advancing it or invoking code.
    pub(crate) fn pending_prepare(
        &self,
        sequence: u64,
        root: &str,
        generation: u64,
        reference: &str,
    ) -> Option<DescendantJob> {
        self.jobs
            .iter()
            .find(|entry| {
                entry.phase == Phase::Armed
                    && entry.job.bound_to(sequence, root, generation, reference)
            })
            .map(|entry| entry.job.clone())
    }

    pub(crate) fn prepare(
        &mut self,
        sequence: u64,
        root: &str,
        generation: u64,
        reference: &str,
        choice: DescendantChoice,
    ) -> bool {
        let Some(entry) = self.jobs.iter_mut().find(|entry| {
            entry.phase == Phase::Armed && entry.job.bound_to(sequence, root, generation, reference)
        }) else {
            return false;
        };
        if choice.target.is_empty() {
            return false;
        }
        entry.phase = Phase::Prepared(choice);
        true
    }

    pub(crate) fn focus(
        &mut self,
        sequence: u64,
        root: &str,
        generation: u64,
        reference: &str,
        target: &str,
    ) -> Option<DescendantChoice> {
        let entry = self
            .jobs
            .iter_mut()
            .find(|entry| entry.job.bound_to(sequence, root, generation, reference))?;
        let Phase::Prepared(choice) = &entry.phase else {
            return None;
        };
        if choice.target != target {
            return None;
        }
        let choice = choice.clone();
        entry.phase = Phase::Focused(choice.clone());
        Some(choice)
    }

    /// Consume publication separately from focus, after native focus callbacks.
    pub(crate) fn visual(
        &mut self,
        sequence: u64,
        root: &str,
        generation: u64,
        reference: &str,
        target: &str,
    ) -> Option<DescendantJob> {
        let index = self.jobs.iter().position(|entry| {
            entry.job.bound_to(sequence, root, generation, reference)
                && matches!(&entry.phase, Phase::Focused(choice) if choice.target == target)
        })?;
        self.jobs.remove(index).map(|entry| entry.job)
    }

    /// A failed bound execution releases only its exact job after arming.
    pub(crate) fn cancel(
        &mut self,
        sequence: u64,
        root: &str,
        generation: u64,
        reference: &str,
    ) -> bool {
        let Some(index) = self.jobs.iter().position(|entry| {
            entry.phase != Phase::Unarmed
                && entry.job.bound_to(sequence, root, generation, reference)
        }) else {
            return false;
        };
        self.jobs.remove(index);
        true
    }

    /// Duplicate enrollment failures cannot discard an already armed job.
    pub(crate) fn discard(&mut self, sequence: u64, root: &str, generation: u64) -> bool {
        let Some(index) = self.jobs.iter().position(|entry| {
            entry.phase == Phase::Unarmed && entry.job.matches(sequence, root, generation)
        }) else {
            return false;
        };
        self.jobs.remove(index);
        true
    }

    /// Keep exact native references until their independently admitted work is consumed.
    pub(crate) fn retains_token(&self, capture: &str) -> bool {
        self.jobs.iter().any(|entry| {
            entry.job.reference.as_deref() == Some(capture)
                || matches!(&entry.job.purpose,DescendantPurpose::Return { capture: target } if target == capture)
                || matches!(&entry.phase,Phase::Prepared(choice) | Phase::Focused(choice) if choice.target == capture)
        })
    }
    pub(crate) fn remove_root(&mut self, root: &str, generation: u64) {
        self.jobs
            .retain(|entry| entry.job.root != root || entry.job.generation != generation);
    }

    pub(crate) fn clear(&mut self) {
        self.jobs.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn admit(queue: &mut DescendantQueue, reference: Option<&str>) -> DescendantJob {
        queue.admit(
            "root".into(),
            3,
            7,
            false,
            true,
            reference.map(str::to_owned),
        )
    }

    fn choice(target: &str) -> DescendantChoice {
        DescendantChoice {
            candidate: Some("candidate".into()),
            target: target.into(),
            is_html: true,
        }
    }

    fn arm(queue: &mut DescendantQueue, job: &DescendantJob, reference: &str) -> bool {
        queue.arm(job.sequence, &job.root, job.generation, reference)
    }

    fn prepare(
        queue: &mut DescendantQueue,
        job: &DescendantJob,
        reference: &str,
        target: &str,
    ) -> bool {
        queue.prepare(
            job.sequence,
            &job.root,
            job.generation,
            reference,
            choice(target),
        )
    }

    #[test]
    fn exact_binding_and_phase_replays_leave_correct_authorization_available() {
        let mut queue = DescendantQueue::default();
        let job = admit(&mut queue, Some("reference"));
        assert!(
            queue
                .pending_prepare(job.sequence, "root", 3, "reference")
                .is_none()
        );
        assert!(!queue.arm(job.sequence, "other", 3, "reference"));
        assert!(!queue.arm(job.sequence, "root", 4, "reference"));
        assert!(!arm(&mut queue, &job, ""));
        assert!(!arm(&mut queue, &job, "copied-reference"));
        assert!(arm(&mut queue, &job, "reference"));
        assert!(!arm(&mut queue, &job, "reference"));
        assert!(
            queue
                .pending_prepare(job.sequence, "root", 3, "copied-reference")
                .is_none()
        );
        assert_eq!(
            queue.pending_prepare(job.sequence, "root", 3, "reference"),
            Some(job.clone())
        );
        assert_eq!(
            queue.pending_prepare(job.sequence, "root", 3, "reference"),
            Some(job.clone())
        );
        assert!(
            queue
                .focus(job.sequence, "root", 3, "reference", "target")
                .is_none()
        );
        assert!(!prepare(&mut queue, &job, "copied-reference", "target"));
        assert!(!prepare(&mut queue, &job, "reference", ""));
        assert!(prepare(&mut queue, &job, "reference", "target"));
        assert!(!prepare(&mut queue, &job, "reference", "replacement"));
        assert!(
            queue
                .visual(job.sequence, "root", 3, "reference", "target")
                .is_none()
        );
        assert!(
            queue
                .focus(job.sequence, "other", 3, "reference", "target")
                .is_none()
        );
        assert!(
            queue
                .focus(job.sequence, "root", 4, "reference", "target")
                .is_none()
        );
        assert!(
            queue
                .focus(job.sequence, "root", 3, "copied-reference", "target")
                .is_none()
        );
        assert!(
            queue
                .focus(job.sequence, "root", 3, "reference", "replacement")
                .is_none()
        );
        assert_eq!(
            queue.focus(job.sequence, "root", 3, "reference", "target"),
            Some(choice("target"))
        );
        assert!(
            queue
                .focus(job.sequence, "root", 3, "reference", "target")
                .is_none()
        );
        assert!(
            queue
                .visual(job.sequence, "other", 3, "reference", "target")
                .is_none()
        );
        assert!(
            queue
                .visual(job.sequence, "root", 4, "reference", "target")
                .is_none()
        );
        assert!(
            queue
                .visual(job.sequence, "root", 3, "copied-reference", "target")
                .is_none()
        );
        assert!(
            queue
                .visual(job.sequence, "root", 3, "reference", "replacement")
                .is_none()
        );
        assert_eq!(
            queue.visual(job.sequence, "root", 3, "reference", "target"),
            Some(job)
        );
        assert!(queue.visual(1, "root", 3, "reference", "target").is_none());
        assert!(queue.jobs.is_empty());
    }

    #[test]
    fn bootstrap_binds_once_and_duplicate_discard_cannot_cancel_execution() {
        let mut queue = DescendantQueue::default();
        let job = admit(&mut queue, None);
        assert_eq!(queue.unsent(), vec![job.clone()]);
        let mut copied_job = job.clone();
        copied_job.generation += 1;
        assert!(!queue.mark_sent(&copied_job));
        assert_eq!(queue.unsent(), vec![job.clone()]);
        assert!(queue.mark_sent(&job));
        assert!(!queue.mark_sent(&job));
        assert!(queue.unsent().is_empty());
        assert!(!queue.cancel(job.sequence, "root", 3, "reference"));
        assert!(arm(&mut queue, &job, "reference"));
        assert!(!queue.discard(job.sequence, "root", 3));
        assert!(!arm(&mut queue, &job, "other-reference"));
        let bound = queue
            .pending_prepare(job.sequence, "root", 3, "reference")
            .unwrap();
        assert_eq!(bound.reference.as_deref(), Some("reference"));
        assert!(!queue.cancel(job.sequence, "root", 3, "other-reference"));
        assert!(prepare(&mut queue, &job, "reference", "target"));
        assert!(!queue.discard(job.sequence, "root", 3));
        assert!(
            queue
                .focus(job.sequence, "root", 3, "reference", "target")
                .is_some()
        );
        assert!(!queue.discard(job.sequence, "root", 3));
        assert!(
            queue
                .visual(job.sequence, "root", 3, "reference", "target")
                .is_some()
        );
    }

    #[test]
    fn independent_newer_job_can_finish_before_older_job_prepares() {
        let mut queue = DescendantQueue::default();
        let older = admit(&mut queue, Some("old-reference"));
        let newer = queue.admit(
            "root".into(),
            3,
            99,
            true,
            false,
            Some("new-reference".into()),
        );
        assert!(arm(&mut queue, &older, "old-reference"));
        assert!(arm(&mut queue, &newer, "new-reference"));
        assert!(prepare(&mut queue, &newer, "new-reference", "new-target"));
        assert_eq!(
            queue.focus(newer.sequence, "root", 3, "new-reference", "new-target"),
            Some(choice("new-target"))
        );
        assert_eq!(
            queue.visual(newer.sequence, "root", 3, "new-reference", "new-target"),
            Some(newer)
        );
        assert_eq!(
            queue.pending_prepare(older.sequence, "root", 3, "old-reference"),
            Some(older.clone())
        );
        assert!(prepare(&mut queue, &older, "old-reference", "old-target"));
        assert!(
            queue
                .focus(older.sequence, "root", 3, "old-reference", "old-target")
                .is_some()
        );
        assert_eq!(
            queue.visual(older.sequence, "root", 3, "old-reference", "old-target"),
            Some(older)
        );
        assert!(queue.jobs.is_empty());
    }

    #[test]
    fn cancellation_and_owner_cleanup_release_only_exact_registration_resources() {
        let mut queue = DescendantQueue::default();
        let unarmed = admit(&mut queue, Some("unarmed"));
        let armed = admit(&mut queue, Some("armed"));
        let prepared = admit(&mut queue, Some("prepared"));
        let focused = admit(&mut queue, Some("focused"));
        let replacement = queue.admit("root".into(), 4, 1, false, true, Some("new".into()));
        let other = queue.admit("other".into(), 3, 1, false, true, Some("other".into()));
        assert!(arm(&mut queue, &armed, "armed"));
        assert!(arm(&mut queue, &prepared, "prepared"));
        assert!(prepare(&mut queue, &prepared, "prepared", "target"));
        assert!(arm(&mut queue, &focused, "focused"));
        assert!(prepare(&mut queue, &focused, "focused", "target"));
        assert!(
            queue
                .focus(focused.sequence, "root", 3, "focused", "target")
                .is_some()
        );
        assert!(!queue.discard(unarmed.sequence, "other", 3));
        assert!(!queue.cancel(prepared.sequence, "root", 4, "prepared"));
        assert!(!queue.cancel(focused.sequence, "root", 3, "wrong"));
        assert!(queue.cancel(prepared.sequence, "root", 3, "prepared"));
        assert!(!queue.cancel(prepared.sequence, "root", 3, "prepared"));
        queue.remove_root("root", 3);
        assert_eq!(queue.unsent(), vec![replacement.clone(), other.clone()]);
        assert!(!queue.cancel(focused.sequence, "root", 3, "focused"));
        assert!(!queue.discard(unarmed.sequence, "root", 3));
        assert!(queue.discard(replacement.sequence, "root", 4));
        assert_eq!(queue.unsent(), vec![other]);
        queue.clear();
        assert!(queue.unsent().is_empty());
        assert!(queue.jobs.is_empty());
    }

    #[test]
    fn purpose_wire_shape_and_default_opening_are_explicit() {
        let mut queue = DescendantQueue::default();
        assert_eq!(admit(&mut queue, None).purpose, DescendantPurpose::Opening);
        for (purpose, raw) in [
            (
                DescendantPurpose::Opening,
                serde_json::json!({"kind":"opening"}),
            ),
            (
                DescendantPurpose::Submenu,
                serde_json::json!({"kind":"submenu"}),
            ),
            (
                DescendantPurpose::Return {
                    capture: "held".into(),
                },
                serde_json::json!({"kind":"return","capture":"held"}),
            ),
        ] {
            assert_eq!(serde_json::to_value(&purpose).unwrap(), raw);
            assert_eq!(
                serde_json::from_value::<DescendantPurpose>(raw).unwrap(),
                purpose
            );
        }
    }

    #[test]
    fn mixed_purposes_replay_independently_and_root_cleanup_cannot_retarget_returns() {
        let mut queue = DescendantQueue::default();
        let opening = admit(&mut queue, Some("reference"));
        let submenu = queue.admit_submenu("root".into(), 3, 8, Some("reference".into()));
        let returning = queue.admit_return(
            "root".into(),
            3,
            9,
            Some("reference".into()),
            "held-trigger".into(),
        );
        assert_eq!(submenu.purpose, DescendantPurpose::Submenu);
        assert_eq!(
            returning.purpose,
            DescendantPurpose::Return {
                capture: "held-trigger".into()
            }
        );
        assert!(!submenu.last && !submenu.allow_checked);
        assert!(!returning.last && !returning.allow_checked);
        assert_eq!(
            queue.unsent(),
            vec![opening.clone(), submenu.clone(), returning.clone()]
        );
        for job in [&opening, &submenu, &returning] {
            assert!(arm(&mut queue, job, "reference"));
            assert!(!arm(&mut queue, job, "reference"));
        }
        // A return may finish before an older submenu or opening prepares.
        assert!(prepare(&mut queue, &returning, "reference", "held-trigger"));
        assert!(
            queue
                .focus(returning.sequence, "root", 3, "reference", "copy")
                .is_none()
        );
        assert!(
            queue
                .focus(returning.sequence, "root", 3, "reference", "held-trigger")
                .is_some()
        );
        assert_eq!(
            queue.visual(returning.sequence, "root", 3, "reference", "held-trigger"),
            Some(returning)
        );
        assert!(
            queue
                .visual(3, "root", 3, "reference", "held-trigger")
                .is_none()
        );
        assert_eq!(
            queue.pending_prepare(submenu.sequence, "root", 3, "reference"),
            Some(submenu.clone())
        );
        assert_eq!(
            queue.pending_prepare(opening.sequence, "root", 3, "reference"),
            Some(opening)
        );
        let replacement = queue.admit_return(
            "root".into(),
            4,
            1,
            Some("new-reference".into()),
            "replacement-trigger".into(),
        );
        queue.remove_root("root", 3);
        assert!(
            queue
                .pending_prepare(submenu.sequence, "root", 3, "reference")
                .is_none()
        );
        assert!(!arm(&mut queue, &replacement, "reference"));
        assert!(arm(&mut queue, &replacement, "new-reference"));
        assert_eq!(
            queue.pending_prepare(replacement.sequence, "root", 4, "new-reference"),
            Some(replacement.clone())
        );
        assert!(queue.cancel(replacement.sequence, "root", 4, "new-reference"));
        assert!(queue.jobs.is_empty());
    }
}
