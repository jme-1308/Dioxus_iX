// SPDX-FileCopyrightText: 2026 Siemens AG
// SPDX-License-Identifier: MIT

//! Rust coordination shared by renderer owners in one native document.
//!
//! The key is a native document identity, never a provider or VirtualDom address.
//! Callbacks must restore their owning runtime and reject a dropped owner. This
//! module never retains a domain globally, or borrows it across a callback.

use crate::overlay_registry::{CloseBehavior, NativeFact, OverlayKind, Registry};
use crate::overlay_runtime::{Change, MountedFocusTarget, Reason};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    rc::{Rc, Weak},
};

thread_local! {
    static DOCUMENTS: RefCell<BTreeMap<String, Weak<RefCell<State>>>> = const {
        RefCell::new(BTreeMap::new())
    };
}

#[derive(Clone)]
pub(crate) struct DomainCallbacks {
    pub(crate) read: Rc<dyn Fn() -> Option<bool>>,
    pub(crate) request: Rc<dyn Fn(Change) -> bool>,
    pub(crate) suppress_trigger_restore: Rc<dyn Fn() -> bool>,
}

#[derive(Clone)]
struct Entry {
    owner: String,
    generation: u64,
    incarnation: u64,
    registration: u64,
    kind: OverlayKind,
    callbacks: DomainCallbacks,
}

type CategoryCallback = Rc<dyn Fn(usize, &[usize])>;
type FocusReader = Rc<dyn Fn(&str) -> Option<MountedFocusTarget>>;

#[derive(Clone)]
struct CategoryEntry {
    owner: String,
    incarnation: u64,
    callback: CategoryCallback,
}

#[derive(Default)]
struct State {
    registry: Registry,
    entries: BTreeMap<String, Entry>,
    order: Vec<String>,
    presentation: Vec<String>,
    incarnation: u64,
    // The shared native listener assigns one ID per document action. Input
    // broadcasts reuse it for every renderer; global policy claims it once.
    // Older actions cannot overwrite a later admitted input state.
    sequence: u64,
    keyboard: Option<bool>,
    categories: BTreeMap<usize, CategoryEntry>,
    category_order: Vec<usize>,
    focus_readers: BTreeMap<String, FocusReader>,
}

impl State {
    fn set_open(&mut self, id: &str, open: bool) {
        let previous = self.registry.is_open(id);
        self.registry.set_presence(id, open);
        if previous != open {
            self.presentation.retain(|candidate| candidate != id);
            if open {
                self.presentation.push(id.to_owned());
            }
        }
    }

    fn presented(&self) -> Vec<String> {
        let mut ids = self.presentation.clone();
        for id in &self.order {
            if self.registry.is_open(id) && !ids.contains(id) {
                ids.push(id.clone());
            }
        }
        ids
    }
}

#[derive(Clone)]
pub(crate) struct Domain(Rc<RefCell<State>>);

impl Domain {
    pub(crate) fn register_focus_reader(&self, owner: String, reader: FocusReader) {
        self.0.borrow_mut().focus_readers.insert(owner, reader);
    }
    pub(crate) fn focus_target(&self, id: &str) -> Option<MountedFocusTarget> {
        let readers = self
            .0
            .borrow()
            .focus_readers
            .values()
            .cloned()
            .collect::<Vec<_>>();
        readers.into_iter().find_map(|reader| reader(id))
    }
    pub(crate) fn join(document: &str) -> Self {
        DOCUMENTS.with(|documents| {
            let mut documents = documents.borrow_mut();
            documents.retain(|_, state| state.strong_count() != 0);
            if let Some(state) = documents.get(document).and_then(Weak::upgrade) {
                return Self(state);
            }
            let state = Rc::new(RefCell::new(State::default()));
            documents.insert(document.to_owned(), Rc::downgrade(&state));
            Self(state)
        })
    }

    pub(crate) fn register(
        &self,
        owner: String,
        id: String,
        generation: u64,
        kind: OverlayKind,
        behavior: CloseBehavior,
        callbacks: DomainCallbacks,
    ) {
        let mut state = self.0.borrow_mut();
        if state.entries.get(&id).is_some_and(|entry| {
            entry.generation > generation
                || (entry.generation == generation && entry.owner != owner)
        }) {
            return;
        }
        let replacement = state
            .entries
            .get(&id)
            .is_some_and(|entry| entry.generation != generation);
        if replacement {
            state.order.retain(|candidate| candidate != &id);
            state.presentation.retain(|candidate| candidate != &id);
        }
        if !state.order.contains(&id) {
            state.order.push(id.clone());
        }
        state
            .registry
            .register(id.clone(), generation, kind, behavior);
        state.incarnation = state.incarnation.wrapping_add(1);
        let incarnation = state.incarnation;
        let registration = state
            .entries
            .get(&id)
            .filter(|_| !replacement)
            .map_or(incarnation, |entry| entry.registration);
        state.entries.insert(
            id,
            Entry {
                owner,
                generation,
                incarnation,
                registration,
                kind,
                callbacks,
            },
        );
    }

    fn entry(&self, id: &str) -> Option<Entry> {
        self.0.borrow().entries.get(id).cloned()
    }

    /// Resolve raw document facts against the current registered generation,
    /// including nodes owned by another renderer runtime in this document.
    pub(crate) fn node_kind(&self, id: &str, generation: u64) -> Option<OverlayKind> {
        self.0
            .borrow()
            .entries
            .get(id)
            .filter(|entry| entry.generation == generation)
            .map(|entry| entry.kind)
    }

    fn current(&self, id: &str, snapshot: &Entry) -> bool {
        self.0.borrow().entries.get(id).is_some_and(|entry| {
            entry.generation == snapshot.generation && entry.incarnation == snapshot.incarnation
        })
    }

    fn read(&self, id: &str, snapshot: &Entry) -> Option<bool> {
        let open = (snapshot.callbacks.read)();
        self.current(id, snapshot).then_some(open).flatten()
    }

    /// Reconcile current Rust presence. Native attributes can lag accepted work.
    pub(crate) fn refresh(&self) {
        let snapshots = {
            let state = self.0.borrow();
            state
                .order
                .iter()
                .filter_map(|id| {
                    state
                        .entries
                        .get(id)
                        .map(|entry| (id.clone(), entry.clone()))
                })
                .collect::<Vec<_>>()
        };
        for (id, entry) in snapshots {
            let value = (entry.callbacks.read)();
            if !self.current(&id, &entry) {
                continue;
            }
            if let Some(open) = value {
                self.0.borrow_mut().set_open(&id, open);
            } else {
                self.unregister(&id, entry.generation);
            }
        }
    }

    pub(crate) fn unregister(&self, id: &str, generation: u64) {
        let mut state = self.0.borrow_mut();
        if !state
            .entries
            .get(id)
            .is_some_and(|entry| entry.generation == generation)
        {
            return;
        }
        state.entries.remove(id);
        state.order.retain(|candidate| candidate != id);
        state.presentation.retain(|candidate| candidate != id);
        state.registry.unregister(id, generation);
    }

    pub(crate) fn remove_owner(&self, owner: &str) {
        let nodes = self
            .0
            .borrow()
            .entries
            .iter()
            .filter(|(_, entry)| entry.owner == owner)
            .map(|(id, entry)| (id.clone(), entry.generation))
            .collect::<Vec<_>>();
        for (id, generation) in nodes {
            self.unregister(&id, generation);
        }
        let mut state = self.0.borrow_mut();
        state.categories.retain(|_, entry| entry.owner != owner);
        state.focus_readers.remove(owner);
        let remaining = state.categories.keys().copied().collect::<BTreeSet<_>>();
        state.category_order.retain(|id| remaining.contains(id));
    }

    pub(crate) fn register_category(&self, owner: String, id: usize, callback: CategoryCallback) {
        let mut state = self.0.borrow_mut();
        if state
            .categories
            .get(&id)
            .is_some_and(|entry| entry.owner != owner)
        {
            return;
        }
        if !state.category_order.contains(&id) {
            state.category_order.push(id);
        }
        state.incarnation = state.incarnation.wrapping_add(1);
        let incarnation = state.incarnation;
        state.categories.insert(
            id,
            CategoryEntry {
                owner,
                incarnation,
                callback,
            },
        );
    }

    pub(crate) fn unregister_category(&self, owner: &str, id: usize) {
        let mut state = self.0.borrow_mut();
        if state
            .categories
            .get(&id)
            .is_some_and(|entry| entry.owner == owner)
        {
            state.categories.remove(&id);
            state.category_order.retain(|candidate| *candidate != id);
        }
    }

    pub(crate) fn broadcast_category(&self, source: usize, active: &[usize]) {
        let recipients = {
            let state = self.0.borrow();
            state
                .category_order
                .iter()
                .filter_map(|id| {
                    (*id != source)
                        .then(|| state.categories.get(id).cloned().map(|entry| (*id, entry)))
                        .flatten()
                })
                .collect::<Vec<_>>()
        };
        for (id, entry) in recipients {
            let current = self.0.borrow().categories.get(&id).is_some_and(|current| {
                current.incarnation == entry.incarnation && current.owner == entry.owner
            });
            if current {
                (entry.callback)(source, active);
            }
        }
    }

    pub(crate) fn sync_open(&self, id: &str, generation: u64, open: bool) {
        let mut state = self.0.borrow_mut();
        if state.registry.generation(id) == Some(generation) {
            state.set_open(id, open);
        }
    }

    /// Run same-type dismissal against the current registered hierarchy before
    /// native opening. Another host's veto does not gate this presentation.
    pub(crate) fn prepare_present(&self, id: &str) -> bool {
        let Some(target) = self.entry(id) else {
            return false;
        };
        if self.read(id, &target).is_none() {
            return false;
        }
        if target.kind == OverlayKind::Popover {
            let path = self.0.borrow().registry.cached_path_including(id);
            let mut visited = BTreeSet::new();
            loop {
                // NestedOverlayRegistry uses Map.forEach: callbacks can append
                // a newly connected instance to the same registration-order
                // pass. A callback update is not a fresh Map registration.
                let next = {
                    let state = self.0.borrow();
                    state.order.iter().find_map(|candidate| {
                        let entry = state.entries.get(candidate)?;
                        (!visited.contains(&entry.registration))
                            .then(|| (candidate.clone(), entry.clone()))
                    })
                };
                let Some((candidate, entry)) = next else {
                    break;
                };
                visited.insert(entry.registration);
                let eligible = entry.kind == OverlayKind::Popover
                    && !path.contains(&candidate)
                    && self
                        .0
                        .borrow()
                        .registry
                        .close_behavior(&candidate)
                        .is_some_and(CloseBehavior::allows_outside);
                if eligible {
                    // Closed Popovers still receive willDismiss in source.
                    // The owner callback preserves that kind-specific boundary.
                    self.request(
                        &candidate,
                        Change {
                            open: false,
                            reason: Reason::Outside,
                            restore_focus: true,
                        },
                    );
                }
            }
        }
        self.read(id, &target).is_some()
    }

    pub(crate) fn presented(&self, id: &str, generation: u64) {
        let mut state = self.0.borrow_mut();
        if state
            .entries
            .get(id)
            .is_some_and(|entry| entry.generation == generation)
        {
            state.registry.set_open(id, true);
            state.presentation.retain(|candidate| candidate != id);
            state.presentation.push(id.to_owned());
        }
    }

    pub(crate) fn set_close_behavior(&self, id: &str, generation: u64, behavior: CloseBehavior) {
        let mut state = self.0.borrow_mut();
        if state.registry.generation(id) == Some(generation) {
            state.registry.set_close_behavior(id, behavior);
        }
    }

    /// A complete snapshot of one owner's nodes. Missing foreign nodes are not
    /// disconnections. Facts cannot change another owner's partition.
    pub(crate) fn observe_owner(&self, owner: &str, facts: &[NativeFact]) {
        self.observe_partition(Some(owner), facts);
    }

    /// A complete native document snapshot, including all renderer owners.
    pub(crate) fn observe_all(&self, facts: &[NativeFact]) {
        self.observe_partition(None, facts);
    }

    fn observe_partition(&self, owner: Option<&str>, facts: &[NativeFact]) {
        let mut state = self.0.borrow_mut();
        let observed = state
            .order
            .iter()
            .filter_map(|id| {
                let entry = state.entries.get(id)?;
                if owner.is_some_and(|owner| entry.owner != owner) {
                    return None;
                }
                Some(
                    facts
                        .iter()
                        .find(|fact| fact.id == *id && fact.generation == entry.generation)
                        .cloned()
                        .unwrap_or_else(|| NativeFact {
                            id: id.clone(),
                            generation: entry.generation,
                            ..NativeFact::default()
                        }),
                )
            })
            .collect::<Vec<_>>();
        state.registry.observe(&observed);
    }

    /// Admit a shared document delivery once. These monotonically increasing
    /// IDs are independent of each owner's transport sequence; input broadcasts
    /// intentionally deliver the same ID to several renderer owners.
    pub(crate) fn accept_event(&self, sequence: u64) -> bool {
        let mut state = self.0.borrow_mut();
        if sequence == 0 || sequence <= state.sequence {
            return false;
        }
        state.sequence = sequence;
        true
    }

    /// The current broadcast may still need to clear another owner's local
    /// pointer cue. Older deliveries must not overwrite newer input state.
    pub(crate) fn is_current_event(&self, sequence: u64) -> bool {
        sequence != 0 && self.0.borrow().sequence == sequence
    }

    /// Reduce admitted document input or a directly owned Dioxus key. Browser
    /// default prevention does not alter the input-mode decision.
    pub(crate) fn observe_input(&self, kind: &str, key: &str) {
        let keyboard = match kind {
            "keydown"
                if matches!(
                    key,
                    "Tab"
                        | "ArrowDown"
                        | "Space"
                        | "Escape"
                        | " "
                        | "Shift"
                        | "Enter"
                        | "ArrowLeft"
                        | "ArrowRight"
                        | "ArrowUp"
                        | "Home"
                        | "End"
                ) =>
            {
                Some(true)
            }
            "mousedown" | "touchstart" => Some(false),
            _ => None,
        };
        if let Some(keyboard) = keyboard {
            self.0.borrow_mut().keyboard = Some(keyboard);
        }
    }

    pub(crate) fn keyboard_mode(&self) -> Option<bool> {
        self.0.borrow().keyboard
    }

    /// Project accepted destination visibility while native hide may still be
    /// awaiting renderer/IPC work. These are current native Popover ancestors,
    /// not the originating Dropdown's presence or an earlier target path.
    pub(crate) fn focus_destinations_visible(&self, popovers: &[String]) -> bool {
        popovers.iter().all(|id| {
            self.entry(id)
                .filter(|entry| entry.kind == OverlayKind::Popover)
                .is_some_and(|entry| self.read(id, &entry) == Some(true))
        })
    }

    pub(crate) fn topmost(&self) -> Option<String> {
        self.refresh();
        self.0.borrow().registry.topmost()
    }

    /// Source document traps defer to focused topmost overlays and to any
    /// other open Popover layer. Otherwise exclude the topmost-to-host branch.
    pub(crate) fn focus_trap_exclusions(
        &self,
        host: &str,
        active_hosts: &[String],
        active_popover: &str,
    ) -> Option<Vec<String>> {
        self.refresh();
        let state = self.0.borrow();
        if active_popover != host
            && state.entries.get(active_popover).is_some_and(|entry| {
                entry.kind == OverlayKind::Popover && state.registry.is_open(active_popover)
            })
        {
            return None;
        }
        let Some(top) = state.presented().into_iter().rev().find(|id| {
            state.registry.is_open(id)
                && state.registry.is_connected(id)
                && state.entries.contains_key(id)
        }) else {
            return Some(Vec::new());
        };
        if top == host {
            return Some(Vec::new());
        }
        if active_hosts.contains(&top) {
            return None;
        }
        let mut excluded = vec![top.clone()];
        for ancestor in state.registry.focus_ancestors(&top) {
            if ancestor == host {
                return Some(excluded);
            }
            excluded.push(ancestor);
        }
        Some(Vec::new())
    }

    pub(crate) fn owns_topmost(&self, id: &str) -> bool {
        self.refresh();
        let state = self.0.borrow();
        state.registry.topmost().is_some_and(|top| {
            let same_kind = state
                .entries
                .get(&top)
                .zip(state.entries.get(id))
                .is_some_and(|(top, owner)| {
                    top.kind == owner.kind || top.kind.is_dropdown() && owner.kind.is_dropdown()
                });
            same_kind
                && (top == id
                    || state
                        .registry
                        .live_ancestors(&top)
                        .iter()
                        .any(|ancestor| ancestor == id))
        })
    }

    /// Historical query names retained internally; relationships are current.
    pub(crate) fn cached_root(&self, id: &str) -> Option<String> {
        self.0.borrow().registry.cached_root(id)
    }

    pub(crate) fn cached_parent(&self, id: &str) -> Option<String> {
        self.0.borrow().registry.cached_parent(id)
    }

    pub(crate) fn cached_children(&self, id: &str) -> Vec<String> {
        self.0.borrow().registry.cached_children(id)
    }

    #[cfg(test)]
    pub(crate) fn live_ancestors(&self, id: &str) -> Vec<String> {
        self.0.borrow().registry.live_ancestors(id)
    }

    pub(crate) fn has_popover_ancestor(&self, id: &str) -> bool {
        self.refresh();
        let state = self.0.borrow();
        state.registry.focus_ancestors(id).iter().any(|ancestor| {
            state
                .entries
                .get(ancestor)
                .is_some_and(|entry| entry.kind == OverlayKind::Popover)
        })
    }

    pub(crate) fn popover_focus_parent(&self, id: &str) -> Option<String> {
        let state = self.0.borrow();
        let parent = state.registry.focus_parent(id)?;
        state
            .entries
            .get(&parent)
            .is_some_and(|entry| entry.kind == OverlayKind::Popover)
            .then_some(parent)
    }

    pub(crate) fn child_path(&self, id: &str, hosts: &[String], triggers: &[String]) -> bool {
        self.0.borrow().registry.child_path(id, hosts, triggers)
    }

    /// The owner callback implements its before/apply lifecycle and child
    /// cascade. A veto is returned without falling through to another overlay.
    pub(crate) fn request(&self, id: &str, change: Change) -> bool {
        let Some(entry) = self.entry(id) else {
            return false;
        };
        if self.read(id, &entry).is_none() {
            return false;
        }
        let accepted = (entry.callbacks.request)(change);
        if let Some(open) = self.read(id, &entry) {
            self.0.borrow_mut().set_open(id, open);
        }
        accepted
    }

    /// Source dismissOthers, reached after an actual trigger toggle attempt or
    /// a host click's fallthrough. Public controller requests do not acquire it.
    /// True means the originating registration was admitted, not that every
    /// eligible target accepted its close request.
    pub(crate) fn dismiss_dropdown_others(&self, id: &str, generation: u64) -> bool {
        self.dismiss_dropdown_set(id, generation, false)
    }

    pub(crate) fn dismiss_dropdown_all(&self, id: &str, generation: u64) -> bool {
        self.dismiss_dropdown_set(id, generation, true)
    }

    fn dismiss_dropdown_set(&self, id: &str, generation: u64, inside: bool) -> bool {
        let Some(origin) = self
            .entry(id)
            .filter(|entry| entry.generation == generation && entry.kind.is_dropdown())
        else {
            return false;
        };
        if self.read(id, &origin).is_none() {
            return false;
        }
        self.refresh();
        if !self.current(id, &origin) {
            return false;
        }
        let excluded = if inside {
            Vec::new()
        } else {
            self.0.borrow().registry.dropdown_dismiss_exclusions(id)
        };
        let mut visited = BTreeSet::new();
        loop {
            let next = {
                let state = self.0.borrow();
                state.order.iter().find_map(|candidate| {
                    let entry = state.entries.get(candidate)?;
                    (!visited.contains(&entry.registration))
                        .then(|| (candidate.clone(), entry.clone()))
                })
            };
            let Some((candidate, entry)) = next else {
                break;
            };
            visited.insert(entry.registration);
            let eligible = entry.kind.is_dropdown()
                && !excluded.contains(&candidate)
                && (inside && candidate == id
                    || self
                        .0
                        .borrow()
                        .registry
                        .close_behavior(&candidate)
                        .is_some_and(CloseBehavior::allows_outside));
            if eligible && self.read(&candidate, &entry) == Some(true) {
                self.request(
                    &candidate,
                    Change {
                        open: false,
                        reason: if inside {
                            Reason::Inside
                        } else {
                            Reason::TriggerOther
                        },
                        restore_focus: !inside || self.keyboard_mode().unwrap_or(false),
                    },
                );
            }
        }
        true
    }

    pub(crate) fn close_children(&self, id: &str, reason: Reason, restore_focus: bool) {
        let cross_type = {
            let state = self.0.borrow();
            let kind = state.entries.get(id).map(|entry| entry.kind);
            state
                .presented()
                .into_iter()
                .rev()
                .filter(|child| {
                    state.entries.get(child).is_some_and(|entry| {
                        kind.is_some_and(|kind| {
                            entry.kind != kind && !(entry.kind.is_dropdown() && kind.is_dropdown())
                        })
                    }) && state.registry.live_parent(child).as_deref() == Some(id)
                })
                .collect::<Vec<_>>()
        };
        for child in cross_type {
            self.request(
                &child,
                Change {
                    open: false,
                    reason,
                    // Cross-type source callbacks map parent-close to the
                    // child's own focus policy: Popover restores its trigger.
                    restore_focus: self
                        .entry(&child)
                        .is_some_and(|entry| entry.kind == OverlayKind::Popover),
                },
            );
        }
        let children = self.cached_children(id);
        for child in children {
            self.request(
                &child,
                Change {
                    open: false,
                    reason,
                    restore_focus: (matches!(
                        reason,
                        Reason::AnchorVisibilityParent | Reason::TriggerOther
                    ) && self
                        .entry(&child)
                        .is_some_and(|entry| entry.kind.is_dropdown()))
                        || (restore_focus
                            && self
                                .entry(id)
                                .is_some_and(|entry| entry.kind == OverlayKind::Popover)),
                },
            );
        }
    }

    pub(crate) fn dismiss_root(&self, id: &str, reason: Reason, restore_focus: bool) -> bool {
        self.refresh();
        let Some(root) = self.cached_root(id) else {
            return false;
        };
        let restore_focus = restore_focus
            && self
                .entry(&root)
                .is_some_and(|entry| entry.kind != OverlayKind::Header);
        self.request(
            &root,
            Change {
                open: false,
                reason,
                restore_focus,
            },
        )
    }

    /// Tab release suppresses each currently open related Dropdown
    /// before the root's cancelable before callback. Veto leaves that state.
    pub(crate) fn close_and_release_focus(&self, id: &str) -> bool {
        self.refresh();
        let Some(root) = self.cached_root(id) else {
            return false;
        };
        let Some(entry) = self.entry(&root).filter(|entry| entry.kind.is_dropdown()) else {
            return false;
        };
        self.suppress_cached_trigger_restore(&root, &mut BTreeSet::new());
        if !self.current(&root, &entry) {
            return false;
        }
        self.request(
            &root,
            Change {
                open: false,
                reason: Reason::Keyboard,
                restore_focus: false,
            },
        )
    }

    fn suppress_cached_trigger_restore(&self, id: &str, visited: &mut BTreeSet<String>) {
        if !visited.insert(id.to_owned()) {
            return;
        }
        let Some(entry) = self.entry(id).filter(|entry| entry.kind.is_dropdown()) else {
            return;
        };
        if self.read(id, &entry) != Some(true)
            || !(entry.callbacks.suppress_trigger_restore)()
            || !self.current(id, &entry)
        {
            return;
        }
        // A closed intermediate stops recursion, matching the source method.
        for child in self.cached_children(id) {
            self.suppress_cached_trigger_restore(&child, visited);
        }
    }

    /// Called only after the native event reaches Window. Source does not gate
    /// this path on default prevention, focus membership, or outside policy.
    pub(crate) fn escape(&self) -> bool {
        let Some(top) = self.topmost() else {
            return false;
        };
        if self
            .entry(&top)
            .is_some_and(|entry| entry.kind == OverlayKind::Popover)
        {
            self.request(
                &top,
                Change {
                    open: false,
                    reason: Reason::Escape,
                    restore_focus: true,
                },
            )
        } else {
            self.dismiss_root(&top, Reason::Escape, true)
        }
    }

    /// Source snapshots presentation order, then rechecks each entry after any
    /// earlier consumer callback. Reentrant new entries are not added mid-pass.
    pub(crate) fn outside(&self, hosts: &[String], triggers: &[String]) -> bool {
        self.refresh();
        let presented = {
            let state = self.0.borrow();
            state
                .presented()
                .into_iter()
                .filter_map(|id| state.entries.get(&id).cloned().map(|entry| (id, entry)))
                .collect::<Vec<_>>()
        };
        let mut accepted = false;
        for (id, snapshot) in presented.into_iter().rev() {
            self.refresh();
            if !self.current(&id, &snapshot) {
                continue;
            }
            let eligible = {
                let state = self.0.borrow();
                state.registry.is_open(&id)
                    && state
                        .registry
                        .close_behavior(&id)
                        .is_some_and(CloseBehavior::allows_outside)
                    && !state.registry.path_in_hierarchy(&id, hosts, triggers)
            };
            if eligible {
                accepted |= self.request(
                    &id,
                    Change {
                        open: false,
                        reason: Reason::Outside,
                        restore_focus: false,
                    },
                );
            }
        }
        accepted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    struct Model {
        open: Rc<Cell<bool>>,
        allow: Rc<Cell<bool>>,
    }

    fn model(
        domain: &Domain,
        owner: &str,
        id: &str,
        kind: OverlayKind,
        log: Rc<RefCell<Vec<String>>>,
    ) -> Model {
        let open = Rc::new(Cell::new(false));
        let allow = Rc::new(Cell::new(true));
        let read = open.clone();
        let apply = open.clone();
        let accept = allow.clone();
        let label = id.to_owned();
        domain.register(
            owner.to_owned(),
            id.to_owned(),
            1,
            kind,
            CloseBehavior::Both,
            DomainCallbacks {
                suppress_trigger_restore: Rc::new(|| true),
                read: Rc::new(move || Some(read.get())),
                request: Rc::new(move |change| {
                    log.borrow_mut().push(format!(
                        "{label}:{}:{:?}:{}",
                        change.open, change.reason, change.restore_focus
                    ));
                    if !accept.get() {
                        return false;
                    }
                    apply.set(change.open);
                    true
                }),
            },
        );
        Model { open, allow }
    }

    fn fact(id: &str, ancestors: &[&str]) -> NativeFact {
        NativeFact {
            id: id.into(),
            generation: 1,
            connected: true,
            trigger_ancestors: ancestors.iter().map(|id| (*id).to_owned()).collect(),
            assignment_ancestors: None,
            discovery_ready: true,
        }
    }

    fn close() -> Change {
        Change {
            open: false,
            reason: Reason::Outside,
            restore_focus: false,
        }
    }

    #[test]
    fn popover_prepare_uses_current_ancestors_policy_veto_and_registration_order() {
        let domain = Domain::join("popover controller prepare");
        let log = Rc::new(RefCell::new(Vec::new()));
        let parent = model(&domain, "a", "parent", OverlayKind::Popover, log.clone());
        let target = model(&domain, "a", "target", OverlayKind::Popover, log.clone());
        let sibling = model(&domain, "b", "sibling", OverlayKind::Popover, log.clone());
        let sticky = model(&domain, "b", "sticky", OverlayKind::Popover, log.clone());
        let veto = model(&domain, "b", "veto", OverlayKind::Popover, log.clone());
        let closed = model(&domain, "b", "closed", OverlayKind::Popover, log.clone());
        let dropdown = model(&domain, "c", "dropdown", OverlayKind::Dropdown, log.clone());
        domain.observe_all(&[
            fact("parent", &[]),
            fact("target", &["parent"]),
            fact("sibling", &["parent"]),
            fact("sticky", &[]),
            fact("veto", &[]),
            fact("closed", &[]),
            fact("dropdown", &[]),
        ]);
        for open in [&parent, &sibling, &sticky, &veto, &dropdown] {
            open.open.set(true);
        }
        domain.set_close_behavior("sticky", 1, CloseBehavior::Inside);
        veto.allow.set(false);
        assert!(domain.prepare_present("target"));
        assert!(parent.open.get() && sticky.open.get() && veto.open.get());
        assert!(dropdown.open.get());
        assert!(!target.open.get() && !sibling.open.get() && !closed.open.get());
        assert_eq!(
            *log.borrow(),
            [
                "sibling:false:Outside:true",
                "veto:false:Outside:true",
                "closed:false:Outside:true"
            ]
        );
        assert_eq!(domain.cached_parent("target").as_deref(), Some("parent"));
        assert_ne!(domain.topmost().as_deref(), Some("target"));
        domain.observe_owner("a", &[fact("parent", &[]), fact("target", &[])]);
        assert_eq!(
            domain.cached_parent("target"),
            None,
            "detachment updates the current relationship"
        );
    }

    #[test]
    fn popover_children_follow_late_binding_and_detach_while_parent_is_open() {
        let domain = Domain::join("popover current children");
        let log = Rc::new(RefCell::new(Vec::new()));
        let parent = model(&domain, "a", "parent", OverlayKind::Popover, log.clone());
        let child = model(&domain, "b", "child", OverlayKind::Popover, log);
        let mut pending = fact("child", &["parent"]);
        pending.discovery_ready = false;
        domain.observe_all(&[fact("parent", &[]), pending.clone()]);
        assert!(domain.prepare_present("parent"));
        parent.open.set(true);
        domain.presented("parent", 1);
        assert!(domain.cached_children("parent").is_empty());
        pending.discovery_ready = true;
        domain.observe_owner("b", &[pending]);
        assert_eq!(domain.cached_children("parent"), ["child"]);
        assert_eq!(domain.topmost().as_deref(), Some("parent"));
        child.open.set(true);
        domain.presented("child", 1);
        domain.observe_owner("b", &[fact("child", &[])]);
        assert!(domain.cached_children("parent").is_empty());
        domain.close_children("parent", Reason::Parent, true);
        assert!(
            parent.open.get() && child.open.get(),
            "a detached child is no longer dismissed by its former parent"
        );
    }

    #[test]
    fn presenting_popover_ancestor_dismisses_current_descendant() {
        let domain = Domain::join("popover descendant opening");
        let log = Rc::new(RefCell::new(Vec::new()));
        let parent = model(&domain, "a", "parent", OverlayKind::Popover, log.clone());
        let child = model(&domain, "b", "child", OverlayKind::Popover, log.clone());
        domain.observe_all(&[fact("parent", &[]), fact("child", &["parent"])]);
        child.open.set(true);
        domain.presented("child", 1);
        assert!(domain.prepare_present("parent"));
        assert!(!parent.open.get() && !child.open.get());
        assert_eq!(*log.borrow(), ["child:false:Outside:true"]);
        assert_eq!(domain.cached_children("parent"), ["child"]);
    }

    #[test]
    fn prepare_present_visits_new_registration_without_holding_callback_borrow() {
        let domain = Domain::join("popover live registration pass");
        let log = Rc::new(RefCell::new(Vec::new()));
        let target = model(&domain, "a", "target", OverlayKind::Popover, log.clone());
        let append = domain.clone();
        let append_log = log.clone();
        let once = Rc::new(Cell::new(true));
        domain.register(
            "a".into(),
            "first".into(),
            1,
            OverlayKind::Popover,
            CloseBehavior::Both,
            DomainCallbacks {
                suppress_trigger_restore: Rc::new(|| true),
                read: Rc::new(|| Some(false)),
                request: Rc::new(move |_| {
                    append_log.borrow_mut().push("first".into());
                    if once.replace(false) {
                        let late_log = append_log.clone();
                        append.register(
                            "b".into(),
                            "late".into(),
                            1,
                            OverlayKind::Popover,
                            CloseBehavior::Both,
                            DomainCallbacks {
                                suppress_trigger_restore: Rc::new(|| true),
                                read: Rc::new(|| Some(false)),
                                request: Rc::new(move |_| {
                                    late_log.borrow_mut().push("late".into());
                                    true
                                }),
                            },
                        );
                    }
                    false
                }),
            },
        );
        domain.observe_all(&[fact("target", &[]), fact("first", &[])]);
        assert!(domain.prepare_present("target"));
        assert!(!target.open.get());
        assert_eq!(*log.borrow(), ["first", "late"]);
    }

    #[test]
    fn prepare_present_rejects_removed_target_after_dismissal_callback() {
        let domain = Domain::join("popover removed target");
        let log = Rc::new(RefCell::new(Vec::new()));
        let target = model(&domain, "target-owner", "target", OverlayKind::Popover, log);
        let remove = domain.clone();
        domain.register(
            "other-owner".into(),
            "other".into(),
            1,
            OverlayKind::Popover,
            CloseBehavior::Both,
            DomainCallbacks {
                suppress_trigger_restore: Rc::new(|| true),
                read: Rc::new(|| Some(true)),
                request: Rc::new(move |_| {
                    remove.remove_owner("target-owner");
                    true
                }),
            },
        );
        domain.observe_all(&[fact("target", &[]), fact("other", &[])]);
        assert!(!domain.prepare_present("target"));
        assert!(!target.open.get());
        assert!(domain.entry("target").is_none());
        assert!(!domain.prepare_present("missing"));
    }

    #[test]
    fn native_document_identity_shares_owners_and_isolates_other_documents() {
        let first = Domain::join("same native document");
        let second = Domain::join("same native document");
        let foreign = Domain::join("another native document");
        assert!(Rc::ptr_eq(&first.0, &second.0));
        assert!(!Rc::ptr_eq(&first.0, &foreign.0));
        let log = Rc::new(RefCell::new(Vec::new()));
        let a = model(&first, "a", "a", OverlayKind::Dropdown, log.clone());
        let b = model(&second, "b", "b", OverlayKind::Dropdown, log.clone());
        first.observe_owner("a", &[fact("a", &[])]);
        second.observe_owner("b", &[fact("b", &[])]);
        a.open.set(true);
        first.sync_open("a", 1, true);
        b.open.set(true);
        second.sync_open("b", 1, true);
        assert_eq!(first.topmost().as_deref(), Some("b"));
        assert!(foreign.topmost().is_none());
        assert!(first.escape());
        assert!(a.open.get() && !b.open.get());
        assert_eq!(*log.borrow(), ["b:false:Escape:true"]);
    }

    #[test]
    fn owner_partition_does_not_disconnect_or_accept_facts_for_foreign_nodes() {
        let domain = Domain::join("partitioned");
        let log = Rc::new(RefCell::new(Vec::new()));
        let a = model(&domain, "owner-a", "a", OverlayKind::Dropdown, log.clone());
        let b = model(&domain, "owner-b", "b", OverlayKind::Dropdown, log);
        domain.observe_owner("owner-a", &[fact("a", &[])]);
        domain.observe_owner("owner-b", &[fact("b", &[])]);
        a.open.set(true);
        b.open.set(true);
        domain.sync_open("a", 1, true);
        domain.sync_open("b", 1, true);
        domain.observe_owner(
            "owner-a",
            &[NativeFact {
                id: "b".into(),
                generation: 1,
                ..NativeFact::default()
            }],
        );
        assert_eq!(domain.topmost().as_deref(), Some("b"));
        domain.remove_owner("owner-a");
        assert_eq!(domain.topmost().as_deref(), Some("b"));
        domain.observe_all(&[]);
        assert!(
            domain.topmost().is_none(),
            "complete document snapshot can disconnect b"
        );
    }

    #[test]
    fn one_document_delivery_is_accepted_once_across_owners() {
        let first = Domain::join("event document");
        let second = Domain::join("event document");
        let foreign = Domain::join("event foreign document");
        assert!(!first.accept_event(0));
        assert!(first.accept_event(7));
        assert!(!second.accept_event(7));
        assert!(
            second.is_current_event(7),
            "the same input broadcast can reach another local cue"
        );
        assert!(!second.accept_event(6));
        assert!(!second.is_current_event(6));
        assert!(second.accept_event(8));
        assert!(
            !first.accept_event(7),
            "an older delivery cannot overwrite the newer document event"
        );
        assert!(!first.is_current_event(7));
        assert!(first.is_current_event(8));
        assert!(foreign.accept_event(7));
    }

    #[test]
    fn document_bubble_modality_uses_exact_source_keys_and_preserves_unmatched_input() {
        let domain = Domain::join("shared input document");
        let second_owner = Domain::join("shared input document");
        let foreign = Domain::join("foreign input document");
        assert_eq!(domain.keyboard_mode(), None);
        for key in [
            "a", "PageDown", "PageUp", "Control", "Meta", "Alt", "Spacebar",
        ] {
            domain.observe_input("keydown", key);
            assert_eq!(domain.keyboard_mode(), None);
        }
        for key in [
            "Tab",
            "ArrowDown",
            "Space",
            "Escape",
            " ",
            "Shift",
            "Enter",
            "ArrowLeft",
            "ArrowRight",
            "ArrowUp",
            "Home",
            "End",
        ] {
            domain.observe_input("mousedown", "");
            assert_eq!(second_owner.keyboard_mode(), Some(false));
            second_owner.observe_input("keydown", key);
            assert_eq!(
                domain.keyboard_mode(),
                Some(true),
                "recognized source key {key:?}"
            );
            for (kind, unmatched) in [
                ("keydown", "a"),
                ("pointerdown", ""),
                ("mousemove", ""),
                ("keyup", key),
            ] {
                second_owner.observe_input(kind, unmatched);
                assert_eq!(domain.keyboard_mode(), Some(true));
            }
        }
        domain.observe_input("touchstart", "");
        domain.observe_input("keydown", "PageDown");
        assert_eq!(second_owner.keyboard_mode(), Some(false));
        assert_eq!(foreign.keyboard_mode(), None);
        foreign.observe_input("keydown", "Tab");
        assert_eq!(foreign.keyboard_mode(), Some(true));
        assert_eq!(domain.keyboard_mode(), Some(false));
    }

    #[test]
    fn vetoed_topmost_escape_never_falls_through() {
        let domain = Domain::join("veto escape");
        let log = Rc::new(RefCell::new(Vec::new()));
        let a = model(&domain, "a", "a", OverlayKind::Dropdown, log.clone());
        let b = model(&domain, "b", "b", OverlayKind::Dropdown, log.clone());
        domain.observe_all(&[fact("a", &[]), fact("b", &[])]);
        a.open.set(true);
        domain.sync_open("a", 1, true);
        b.open.set(true);
        domain.sync_open("b", 1, true);
        b.allow.set(false);
        assert!(!domain.escape());
        assert!(a.open.get() && b.open.get());
        assert_eq!(*log.borrow(), ["b:false:Escape:true"]);
    }

    #[test]
    fn outside_snapshot_rechecks_later_presence_and_reverse_presentation_order() {
        let domain = Domain::join("outside order");
        let log = Rc::new(RefCell::new(Vec::new()));
        let a = model(&domain, "a", "a", OverlayKind::Dropdown, log.clone());
        let b = model(&domain, "b", "b", OverlayKind::Dropdown, log.clone());
        domain.observe_all(&[fact("a", &[]), fact("b", &[])]);
        b.open.set(true);
        domain.sync_open("b", 1, true);
        a.open.set(true);
        domain.sync_open("a", 1, true);
        domain.sync_open("b", 1, true);
        a.allow.set(false);
        assert!(domain.outside(&[], &[]));
        assert!(a.open.get() && !b.open.get());
        assert_eq!(
            *log.borrow(),
            ["a:false:Outside:false", "b:false:Outside:false"]
        );
    }

    #[test]
    fn current_assignment_follows_cross_owner_trigger_detachment() {
        let domain = Domain::join("current cross-owner detachment");
        let log = Rc::new(RefCell::new(Vec::new()));
        let parent = model(&domain, "a", "parent", OverlayKind::Dropdown, log.clone());
        let child = model(&domain, "b", "child", OverlayKind::Dropdown, log.clone());
        domain.observe_owner("a", &[fact("parent", &[])]);
        domain.observe_owner("b", &[fact("child", &["parent"])]);
        parent.open.set(true);
        domain.presented("parent", 1);
        child.open.set(true);
        domain.presented("child", 1);
        assert_eq!(domain.cached_root("child").as_deref(), Some("parent"));
        assert!(domain.owns_topmost("parent"));
        domain.observe_owner("b", &[fact("child", &[])]);
        assert!(domain.live_ancestors("child").is_empty());
        assert_eq!(domain.cached_root("child").as_deref(), Some("child"));
        assert!(domain.escape());
        assert_eq!(*log.borrow(), ["child:false:Escape:true"]);
        assert!(parent.open.get() && !child.open.get());
    }

    #[test]
    fn popover_escape_selects_self_and_header_adapter_suppresses_restore() {
        let domain = Domain::join("kind-specific escape");
        let log = Rc::new(RefCell::new(Vec::new()));
        let parent = model(&domain, "a", "parent", OverlayKind::Popover, log.clone());
        let child = model(&domain, "b", "child", OverlayKind::Popover, log.clone());
        domain.observe_all(&[fact("parent", &[]), fact("child", &["parent"])]);
        parent.open.set(true);
        domain.sync_open("parent", 1, true);
        child.open.set(true);
        domain.sync_open("child", 1, true);
        assert!(domain.escape());
        assert!(parent.open.get() && !child.open.get());
        let header = model(&domain, "a", "header", OverlayKind::Header, log.clone());
        domain.observe_owner("a", &[fact("parent", &[]), fact("header", &[])]);
        header.open.set(true);
        domain.sync_open("header", 1, true);
        assert!(domain.escape());
        assert_eq!(
            *log.borrow(),
            ["child:false:Escape:true", "header:false:Escape:false"]
        );
    }

    #[test]
    fn current_child_focus_policy_propagates_only_same_type_popover_restore() {
        for kind in [OverlayKind::Popover, OverlayKind::Dropdown] {
            for restore in [false, true] {
                let domain = Domain::join(&format!("current child focus {kind:?} {restore}"));
                let log = Rc::new(RefCell::new(Vec::new()));
                let parent = model(&domain, "a", "parent", kind, log.clone());
                let child = model(&domain, "b", "child", kind, log.clone());
                domain.observe_all(&[fact("parent", &[]), fact("child", &["parent"])]);
                parent.open.set(true);
                domain.presented("parent", 1);
                child.open.set(true);
                domain.presented("child", 1);
                assert_eq!(domain.cached_children("parent"), ["child"]);

                domain.close_children("parent", Reason::Parent, restore);
                assert!(parent.open.get() && !child.open.get());
                let expected_restore = kind == OverlayKind::Popover && restore;
                assert_eq!(
                    *log.borrow(),
                    [format!("child:false:Parent:{expected_restore}")]
                );
            }
        }
    }

    #[test]
    fn cross_type_child_close_uses_live_parent_and_child_focus_policy() {
        for (parent_kind, child_kind, expected_restore) in [
            (OverlayKind::Dropdown, OverlayKind::Popover, true),
            (OverlayKind::Popover, OverlayKind::Dropdown, false),
        ] {
            let domain = Domain::join(&format!("cross type focus {parent_kind:?}"));
            let log = Rc::new(RefCell::new(Vec::new()));
            let parent = model(&domain, "a", "parent", parent_kind, log.clone());
            let child = model(&domain, "b", "child", child_kind, log.clone());
            domain.observe_all(&[fact("parent", &[]), fact("child", &["parent"])]);
            parent.open.set(true);
            domain.presented("parent", 1);
            child.open.set(true);
            domain.presented("child", 1);
            assert!(domain.cached_children("parent").is_empty());
            assert_eq!(domain.live_ancestors("child"), ["parent"]);

            // Source cross-type parent-close selects the child's policy even
            // when the parent request releases its own trigger focus.
            domain.close_children("parent", Reason::Parent, false);
            assert!(parent.open.get() && !child.open.get());
            assert_eq!(
                *log.borrow(),
                [format!("child:false:Parent:{expected_restore}")]
            );

            // Detachment also removes cross-type focus ancestry.
            log.borrow_mut().clear();
            child.open.set(true);
            domain.presented("child", 1);
            domain.observe_owner("b", &[fact("child", &[])]);
            domain.close_children("parent", Reason::Parent, true);
            assert!(child.open.get());
            assert!(log.borrow().is_empty());
        }
    }

    #[test]
    fn native_child_before_foreign_owner_registration_retries_controller_discovery() {
        let domain = Domain::join("late native ancestor owner");
        let log = Rc::new(RefCell::new(Vec::new()));
        let child = model(
            &domain,
            "child owner",
            "child",
            OverlayKind::Dropdown,
            log.clone(),
        );
        // The native foreign host is already an ancestor, but its Rust owner
        // has not registered yet. This must not latch a permanent root result.
        domain.observe_owner("child owner", &[fact("child", &["parent"])]);
        child.open.set(true);
        domain.presented("child", 1);
        assert_eq!(domain.cached_root("child").as_deref(), Some("child"));

        let parent = model(
            &domain,
            "parent owner",
            "parent",
            OverlayKind::Dropdown,
            log.clone(),
        );
        domain.observe_owner("parent owner", &[fact("parent", &[])]);
        // Parent readiness admits the retained current ancestry immediately;
        // no replay or child presentation is required.
        assert_eq!(domain.cached_root("child").as_deref(), Some("parent"));
        parent.open.set(true);
        domain.presented("parent", 1);
        assert_eq!(domain.cached_children("parent"), ["child"]);
        assert_eq!(domain.cached_root("child").as_deref(), Some("parent"));
        domain.close_children("parent", Reason::Parent, false);
        assert!(parent.open.get() && !child.open.get());
        assert_eq!(*log.borrow(), ["child:false:Parent:false"]);
    }

    #[test]
    fn callbacks_can_reenter_and_remove_an_owner_without_domain_borrows() {
        let domain = Domain::join("reentrant request");
        let open = Rc::new(Cell::new(true));
        let read = open.clone();
        let apply = open.clone();
        let nested = domain.clone();
        domain.register(
            "a".into(),
            "a".into(),
            1,
            OverlayKind::Dropdown,
            CloseBehavior::Both,
            DomainCallbacks {
                suppress_trigger_restore: Rc::new(|| true),
                read: Rc::new(move || Some(read.get())),
                request: Rc::new(move |_| {
                    assert!(nested.accept_event(19));
                    nested.remove_owner("a");
                    apply.set(false);
                    true
                }),
            },
        );
        domain.observe_all(&[fact("a", &[])]);
        assert!(domain.request("a", close()));
        assert!(!open.get() && domain.topmost().is_none());
    }

    #[test]
    fn category_broadcast_skips_own_id_and_resumes_outer_registration_order() {
        let domain = Domain::join("category nested broadcasts");
        let log = Rc::new(RefCell::new(Vec::new()));
        let first = log.clone();
        let nested = Rc::downgrade(&domain.0);
        domain.register_category(
            "a".into(),
            1,
            Rc::new(move |source, active| {
                first.borrow_mut().push(format!("a:{source}:{active:?}"));
                if source == 99 {
                    Domain(nested.upgrade().unwrap()).broadcast_category(1, &[1]);
                }
            }),
        );
        let second = log.clone();
        domain.register_category(
            "b".into(),
            2,
            Rc::new(move |source, active| {
                second.borrow_mut().push(format!("b:{source}:{active:?}"));
            }),
        );
        domain.broadcast_category(99, &[2]);
        assert_eq!(*log.borrow(), ["a:99:[2]", "b:1:[1]", "b:99:[2]"]);
        domain.unregister_category("wrong owner", 2);
        domain.remove_owner("a");
        log.borrow_mut().clear();
        domain.broadcast_category(99, &[]);
        assert_eq!(*log.borrow(), ["b:99:[]"]);
        domain.unregister_category("b", 2);
        assert!(domain.0.borrow().categories.is_empty());
    }

    #[test]
    fn replacement_during_read_rejects_stale_callback_and_old_generation_cleanup() {
        let domain = Domain::join("replacement during read");
        let replacing = domain.clone();
        let once = Rc::new(Cell::new(true));
        domain.register(
            "a".into(),
            "node".into(),
            1,
            OverlayKind::Dropdown,
            CloseBehavior::Both,
            DomainCallbacks {
                suppress_trigger_restore: Rc::new(|| true),
                read: Rc::new(move || {
                    if once.replace(false) {
                        replacing.register(
                            "b".into(),
                            "node".into(),
                            2,
                            OverlayKind::Dropdown,
                            CloseBehavior::Both,
                            DomainCallbacks {
                                suppress_trigger_restore: Rc::new(|| true),
                                read: Rc::new(|| Some(true)),
                                request: Rc::new(|_| false),
                            },
                        );
                    }
                    Some(true)
                }),
                request: Rc::new(|_| panic!("replaced callback cannot receive a command")),
            },
        );
        assert!(!domain.request("node", close()));
        domain.unregister("node", 1);
        domain.remove_owner("a");
        assert_eq!(domain.entry("node").unwrap().generation, 2);
    }

    #[test]
    fn weak_document_table_does_not_retain_dropped_domains() {
        let domain = Domain::join("weak lifetime");
        let weak = Rc::downgrade(&domain.0);
        assert!(domain.accept_event(42));
        drop(domain);
        assert!(weak.upgrade().is_none());
        let fresh = Domain::join("weak lifetime");
        assert!(
            fresh.accept_event(1),
            "zero-subscriber state retention is intentionally not claimed"
        );
    }
}
