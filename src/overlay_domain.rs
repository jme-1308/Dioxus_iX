// SPDX-FileCopyrightText: 2026 Siemens AG
// SPDX-License-Identifier: MIT

//! Rust coordination shared by renderer owners in one native document.
//!
//! The key is a native document identity, never a provider or VirtualDom address.
//! Callbacks must restore their owning runtime and reject a dropped owner. This
//! module never retains a domain globally, or borrows it across a callback.

use crate::overlay_registry::{CloseBehavior, NativeFact, OverlayKind, Registry};
use crate::overlay_runtime::{Change, MountedFocusTarget, Reason, SurfaceMetadata};
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
    surface_reader: Option<SurfaceReader>,
    surface_metadata: SurfaceMetadata,
}

type CategoryCallback = Rc<dyn Fn(usize, &[usize])>;
type FocusReader = Rc<dyn Fn(&str) -> Option<MountedFocusTarget>>;
pub(crate) type SurfaceValidity = Rc<dyn Fn() -> bool>;
type SurfaceReader = Rc<dyn Fn() -> Option<SurfaceValidity>>;

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
        let surface_reader = state
            .entries
            .get(&id)
            .filter(|_| !replacement)
            .and_then(|entry| entry.surface_reader.clone());
        let surface_metadata = state
            .entries
            .get(&id)
            .filter(|_| !replacement)
            .map_or_else(SurfaceMetadata::default, |entry| {
                entry.surface_metadata.clone()
            });
        state.entries.insert(
            id,
            Entry {
                owner,
                generation,
                incarnation,
                registration,
                kind,
                callbacks,
                surface_reader,
                surface_metadata,
            },
        );
    }

    pub(crate) fn set_surface_reader(
        &self,
        owner: &str,
        id: &str,
        generation: u64,
        reader: SurfaceReader,
    ) {
        if let Some(entry) = self
            .0
            .borrow_mut()
            .entries
            .get_mut(id)
            .filter(|entry| entry.owner == owner && entry.generation == generation)
        {
            entry.surface_reader = Some(reader);
        }
    }

    pub(crate) fn set_surface_metadata(
        &self,
        owner: &str,
        id: &str,
        generation: u64,
        metadata: SurfaceMetadata,
    ) {
        if let Some(entry) = self
            .0
            .borrow_mut()
            .entries
            .get_mut(id)
            .filter(|entry| entry.owner == owner && entry.generation == generation)
        {
            entry.surface_metadata = metadata;
        }
    }

    /// Current Dioxus-authored descriptors shared only inside this document.
    /// Readers run outside domain borrows and validate renderer/scope lifetime.
    /// Recheck all registrations after callbacks: a later reader can remove an
    /// earlier one, or replace its generation, without a native notification.
    pub(crate) fn surface_metadata(&self) -> Vec<(String, u64, SurfaceMetadata)> {
        let snapshots = {
            let state = self.0.borrow();
            state
                .order
                .iter()
                .filter_map(|id| {
                    let entry = state.entries.get(id)?;
                    state
                        .registry
                        .is_connected(id)
                        .then(|| (id.clone(), entry.clone()))
                })
                .collect::<Vec<_>>()
        };
        let surfaces = snapshots
            .into_iter()
            .filter_map(|(id, entry)| {
                if !self.current(&id, &entry) {
                    return None;
                }
                let valid = (entry.surface_reader.as_ref()?)()?;
                self.current(&id, &entry).then_some((id, entry, valid))
            })
            .collect::<Vec<_>>();
        surfaces
            .into_iter()
            .filter_map(|(id, entry, valid)| {
                if !valid() || !self.current(&id, &entry) {
                    return None;
                }
                let state = self.0.borrow();
                let current = state.entries.get(&id)?;
                state
                    .registry
                    .is_connected(&id)
                    .then(|| (id, current.generation, current.surface_metadata.clone()))
            })
            .collect()
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

    /// Rust-authored lifecycle and ownership facts for one owner. Missing foreign nodes are not
    /// disconnections. Facts cannot change another owner's partition.
    pub(crate) fn observe_owner(&self, owner: &str, facts: &[NativeFact]) {
        self.observe_partition(Some(owner), facts);
    }

    /// Pure graph-model snapshot used only by focused tests.
    #[cfg(test)]
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
        self.dismiss_dropdown_set(id, generation, false, None)
    }

    pub(crate) fn dismiss_dropdown_all(
        &self,
        id: &str,
        generation: u64,
        restore_focus: Option<bool>,
    ) -> bool {
        self.dismiss_dropdown_set(id, generation, true, restore_focus)
    }

    fn dismiss_dropdown_set(
        &self,
        id: &str,
        generation: u64,
        inside: bool,
        restore_focus: Option<bool>,
    ) -> bool {
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
                        restore_focus: restore_focus
                            .unwrap_or_else(|| !inside || self.keyboard_mode().unwrap_or(false)),
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
            // A selection may close its persistent host and open another field
            // before the document delivers this same click. The departing
            // host consumed that click; it must not dismiss the new surface.
            if hosts.iter().any(|id| {
                state.entries.get(id).is_some_and(|entry| {
                    entry.kind.is_dropdown()
                        && state.registry.is_connected(id)
                        && !state.registry.is_open(id)
                })
            }) {
                return false;
            }
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
mod tests;
